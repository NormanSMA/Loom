use crate::types::is_sensitive_path;
use serde::{Deserialize, Serialize};
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GateError {
    #[error("no se pudo leer la imagen {0}: {1}")]
    Io(String, #[source] std::io::Error),
    #[error("imagen invalida: {0}")]
    InvalidImage(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateConfig {
    pub allowed_paths: Vec<String>,
    pub check_ts_rules: bool,
    pub check_ui_tokens: bool,
    pub forbid_emojis: bool,
}

impl Default for GateConfig {
    fn default() -> Self {
        Self {
            allowed_paths: Vec::new(),
            check_ts_rules: true,
            check_ui_tokens: true,
            forbid_emojis: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateViolation {
    pub rule: String,
    pub file: Option<String>,
    pub line: Option<usize>,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GateReport {
    pub passed: bool,
    pub violations: Vec<GateViolation>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageValidation {
    pub format: String,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub bytes: usize,
}

pub fn evaluate_diff(diff_text: &str, changed_files: &[String], cfg: &GateConfig) -> GateReport {
    let mut violations = Vec::new();
    let allowed: Vec<String> = cfg.allowed_paths.iter().map(|p| normalize_rel(p)).collect();

    for file in changed_files {
        let norm = normalize_rel(file);
        if is_sensitive_path(&norm) {
            violations.push(GateViolation {
                rule: "sensitive_file".into(),
                file: Some(norm.clone()),
                line: None,
                detail: "archivo sensible prohibido en diff".into(),
            });
        }
        if !allowed.is_empty() && !allowed.iter().any(|a| a == &norm) {
            violations.push(GateViolation {
                rule: "out_of_scope".into(),
                file: Some(norm),
                line: None,
                detail: "archivo modificado fuera de alcance permitido".into(),
            });
        }
    }

    let mut current_file: Option<String> = None;
    for (idx, line) in diff_text.lines().enumerate() {
        if let Some(rest) = line.strip_prefix("+++ b/") {
            current_file = Some(normalize_rel(rest.trim()));
            continue;
        }
        if line.starts_with("+++") || !line.starts_with('+') {
            continue;
        }

        let added = &line[1..];
        let file_ref = current_file.as_deref().unwrap_or("");
        let line_no = idx + 1;

        if cfg.forbid_emojis && has_emoji(added) {
            violations.push(GateViolation {
                rule: "no_emoji".into(),
                file: current_file.clone(),
                line: Some(line_no),
                detail: "emoji prohibido en codigo".into(),
            });
        }

        if cfg.check_ts_rules
            && (file_ref.ends_with(".ts") || file_ref.ends_with(".tsx"))
            && has_forbidden_ts(added)
        {
            violations.push(GateViolation {
                rule: "strict_ts".into(),
                file: current_file.clone(),
                line: Some(line_no),
                detail: "prohibido any o @ts-ignore".into(),
            });
        }

        if cfg.check_ui_tokens
            && (file_ref.ends_with(".tsx") || file_ref.ends_with(".css"))
            && !is_token_file(file_ref)
            && has_raw_color(added)
        {
            violations.push(GateViolation {
                rule: "ui_tokens".into(),
                file: current_file.clone(),
                line: Some(line_no),
                detail: "color hex o rgb suelto fuera de tokens".into(),
            });
        }
    }

    GateReport {
        passed: violations.is_empty(),
        violations,
    }
}

pub fn validate_raster_image(path: &Path, min_bytes: usize) -> Result<ImageValidation, GateError> {
    let bytes = std::fs::read(path).map_err(|e| GateError::Io(path.display().to_string(), e))?;
    if bytes.len() < min_bytes {
        return Err(GateError::InvalidImage(format!(
            "tamano {}B menor al minimo {}B",
            bytes.len(),
            min_bytes
        )));
    }

    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(128)])
        .trim_start()
        .to_ascii_lowercase();
    if head.starts_with("<svg") || head.starts_with("<?xml") {
        return Err(GateError::InvalidImage(
            "SVG o XML rechazado; se requiere imagen raster real".into(),
        ));
    }

    if bytes.len() >= 24 && bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
        let width = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
        let height = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
        return Ok(ImageValidation {
            format: "png".into(),
            width: Some(width),
            height: Some(height),
            bytes: bytes.len(),
        });
    }

    if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Ok(ImageValidation {
            format: "webp".into(),
            width: None,
            height: None,
            bytes: bytes.len(),
        });
    }

    if bytes.len() >= 3 && bytes[0] == 0xFF && bytes[1] == 0xD8 && bytes[2] == 0xFF {
        return Ok(ImageValidation {
            format: "jpeg".into(),
            width: None,
            height: None,
            bytes: bytes.len(),
        });
    }

    Err(GateError::InvalidImage(
        "cabecera binaria no corresponde a PNG, WebP ni JPEG".into(),
    ))
}

fn normalize_rel(p: &str) -> String {
    p.replace('\\', "/").trim_start_matches("./").to_string()
}

fn is_token_file(file: &str) -> bool {
    file.ends_with("globals.css") || file.ends_with("tokens.css")
}

fn has_emoji(s: &str) -> bool {
    s.chars().any(|c| {
        let code = u32::from(c);
        (0x1F300..=0x1FAFF).contains(&code) || (0x2600..=0x27BF).contains(&code)
    })
}

fn has_forbidden_ts(line: &str) -> bool {
    line.contains(": any")
        || line.contains("as any")
        || line.contains("<any>")
        || line.contains("@ts-ignore")
}

fn has_raw_color(line: &str) -> bool {
    if line.contains("rgb(") || line.contains("rgba(") {
        return true;
    }
    let bytes = line.as_bytes();
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'#' {
            let rest = &bytes[i + 1..];
            let hex_len = rest.iter().take_while(|c| c.is_ascii_hexdigit()).count();
            if hex_len == 3 || hex_len == 6 || hex_len == 8 {
                return true;
            }
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_out_of_scope_any_emoji_and_raw_hex() {
        let diff = "\
+++ b/src/Button.tsx
+const x: any = 1
+const bg = '#ff00aa'
+// 🚀
";
        let report = evaluate_diff(
            diff,
            &["src/Button.tsx".into(), "src/Other.tsx".into()],
            &GateConfig {
                allowed_paths: vec!["src/Button.tsx".into()],
                ..Default::default()
            },
        );
        assert!(!report.passed);
        let rules: Vec<&str> = report.violations.iter().map(|v| v.rule.as_str()).collect();
        assert!(rules.contains(&"out_of_scope"));
        assert!(rules.contains(&"strict_ts"));
        assert!(rules.contains(&"ui_tokens"));
        assert!(rules.contains(&"no_emoji"));
    }

    #[test]
    fn validates_real_png_and_rejects_disguised_svg() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("loom-gate-img-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let png_path = dir.join("ok.png");
        let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        png.extend_from_slice(&13_u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&64_u32.to_be_bytes());
        png.extend_from_slice(&32_u32.to_be_bytes());
        png.resize(80, 0);
        std::fs::write(&png_path, &png).unwrap();

        let info = validate_raster_image(&png_path, 64).unwrap();
        assert_eq!(info.format, "png");
        assert_eq!(info.width, Some(64));
        assert_eq!(info.height, Some(32));

        let fake_path = dir.join("fake.png");
        std::fs::write(
            &fake_path,
            b"<svg xmlns='http://www.w3.org/2000/svg' width='100' height='100'></svg>",
        )
        .unwrap();
        assert!(validate_raster_image(&fake_path, 16).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
