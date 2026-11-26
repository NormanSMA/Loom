use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use loom_core::gates::{GateError, ImageValidation, validate_raster_image};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::path::PathBuf;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ImageGenError {
    #[error("no hay backend o clave API disponible para generar imagenes ({0})")]
    MissingConfig(String),
    #[error("fallo HTTP al generar imagen: {0}")]
    Http(#[from] reqwest::Error),
    #[error("respuesta inesperada del proveedor ({status}): {detail}")]
    Provider { status: u16, detail: String },
    #[error("error base64 en respuesta: {0}")]
    Base64(#[from] base64::DecodeError),
    #[error("error de archivo al guardar imagen: {0}")]
    Io(#[from] std::io::Error),
    #[error("Media Quality Gate rechazo la salida: {0}")]
    Gate(#[from] GateError),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ImageBackend {
    #[default]
    Auto,
    Gemini,
    OpenAi,
    CustomHttp,
}

#[derive(Debug, Clone)]
pub struct ImageGenRequest {
    pub prompt: String,
    pub style_context: Option<String>,
    pub output_path: PathBuf,
    pub backend: ImageBackend,
    pub custom_endpoint: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageGenResult {
    pub backend_used: String,
    pub output_path: PathBuf,
    pub validation: ImageValidation,
}

pub fn build_enriched_prompt(prompt: &str, style_context: Option<&str>) -> String {
    match style_context.map(str::trim).filter(|s| !s.is_empty()) {
        Some(ctx) => format!("{}\nContexto visual del proyecto: {}", prompt.trim(), ctx),
        None => prompt.trim().to_string(),
    }
}

pub async fn generate_image(req: &ImageGenRequest) -> Result<ImageGenResult, ImageGenError> {
    let gemini_key = std::env::var("GEMINI_API_KEY").ok();
    let openai_key = std::env::var("OPENAI_API_KEY").ok();
    generate_image_with_endpoints(
        req,
        "https://generativelanguage.googleapis.com",
        "https://api.openai.com",
        gemini_key.as_deref(),
        openai_key.as_deref(),
    )
    .await
}

pub async fn generate_image_with_endpoints(
    req: &ImageGenRequest,
    gemini_base: &str,
    openai_base: &str,
    gemini_key: Option<&str>,
    openai_key: Option<&str>,
) -> Result<ImageGenResult, ImageGenError> {
    let resolved = match req.backend {
        ImageBackend::Auto => {
            if gemini_key.is_some() {
                ImageBackend::Gemini
            } else if openai_key.is_some() {
                ImageBackend::OpenAi
            } else if req.custom_endpoint.is_some() {
                ImageBackend::CustomHttp
            } else {
                return Err(ImageGenError::MissingConfig(
                    "define GEMINI_API_KEY, OPENAI_API_KEY o custom_endpoint".into(),
                ));
            }
        }
        other => other,
    };

    let enriched = build_enriched_prompt(&req.prompt, req.style_context.as_deref());
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(45))
        .build()?;

    let (backend_used, bytes) = match resolved {
        ImageBackend::Gemini => {
            let key = gemini_key
                .ok_or_else(|| ImageGenError::MissingConfig("falta GEMINI_API_KEY".into()))?;
            let url = format!(
                "{}/v1beta/models/imagen-3.0-generate-002:predict?key={key}",
                gemini_base.trim_end_matches('/')
            );
            let body = json!({
                "instances": [{ "prompt": enriched }],
                "parameters": { "sampleCount": 1 }
            });
            let val = post_json(&client, &url, None, body).await?;
            let b64 = val
                .pointer("/predictions/0/bytesBase64Encoded")
                .and_then(Value::as_str)
                .ok_or_else(|| ImageGenError::Provider {
                    status: 200,
                    detail: "falta predictions[0].bytesBase64Encoded".into(),
                })?;
            ("gemini".to_string(), STANDARD.decode(b64.trim())?)
        }
        ImageBackend::OpenAi => {
            let key = openai_key
                .ok_or_else(|| ImageGenError::MissingConfig("falta OPENAI_API_KEY".into()))?;
            let url = format!(
                "{}/v1/images/generations",
                openai_base.trim_end_matches('/')
            );
            let body = json!({
                "model": "gpt-image-1",
                "prompt": enriched,
                "n": 1
            });
            let val = post_json(&client, &url, Some(key), body).await?;
            let b64 = val
                .pointer("/data/0/b64_json")
                .and_then(Value::as_str)
                .ok_or_else(|| ImageGenError::Provider {
                    status: 200,
                    detail: "falta data[0].b64_json".into(),
                })?;
            ("openai".to_string(), STANDARD.decode(b64.trim())?)
        }
        ImageBackend::CustomHttp => {
            let endpoint = req
                .custom_endpoint
                .as_deref()
                .ok_or_else(|| ImageGenError::MissingConfig("falta custom_endpoint".into()))?;
            let body = json!({ "prompt": enriched, "n": 1 });
            let val = post_json(&client, endpoint, None, body).await?;
            let b64 = val
                .pointer("/data/0/b64_json")
                .or_else(|| val.pointer("/predictions/0/bytesBase64Encoded"))
                .and_then(Value::as_str)
                .ok_or_else(|| ImageGenError::Provider {
                    status: 200,
                    detail: "respuesta sin payload base64".into(),
                })?;
            ("custom_http".to_string(), STANDARD.decode(b64.trim())?)
        }
        ImageBackend::Auto => unreachable!(),
    };

    if let Some(parent) = req.output_path.parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&req.output_path, &bytes)?;

    match validate_raster_image(&req.output_path, 64) {
        Ok(validation) => Ok(ImageGenResult {
            backend_used,
            output_path: req.output_path.clone(),
            validation,
        }),
        Err(err) => {
            let _ = std::fs::remove_file(&req.output_path);
            Err(ImageGenError::Gate(err))
        }
    }
}

async fn post_json(
    client: &reqwest::Client,
    url: &str,
    bearer: Option<&str>,
    body: Value,
) -> Result<Value, ImageGenError> {
    let mut req = client.post(url).json(&body);
    if let Some(token) = bearer {
        req = req.bearer_auth(token);
    }
    let resp = req.send().await?;
    let status = resp.status().as_u16();
    if !resp.status().is_success() {
        let detail = resp.text().await.unwrap_or_default();
        return Err(ImageGenError::Provider { status, detail });
    }
    Ok(resp.json::<Value>().await?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;

    async fn mock_server(body: String) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            if let Ok((mut sock, _)) = listener.accept().await {
                let mut buf = [0_u8; 2048];
                let _ = sock.read(&mut buf).await;
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\n\r\n{}",
                    body.len(),
                    body
                );
                let _ = sock.write_all(resp.as_bytes()).await;
            }
        });
        format!("http://{addr}")
    }

    #[tokio::test]
    async fn generates_valid_png_and_rejects_svg_disguise() {
        let mut dir = std::env::temp_dir();
        dir.push(format!("loom-img-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let mut png = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
        png.extend_from_slice(&13_u32.to_be_bytes());
        png.extend_from_slice(b"IHDR");
        png.extend_from_slice(&128_u32.to_be_bytes());
        png.extend_from_slice(&128_u32.to_be_bytes());
        png.resize(80, 0);

        let ok_b64 = STANDARD.encode(&png);
        let gemini_url =
            mock_server(json!({ "predictions": [{ "bytesBase64Encoded": ok_b64 }] }).to_string())
                .await;

        let out_ok = dir.join("hero.png");
        let res = generate_image_with_endpoints(
            &ImageGenRequest {
                prompt: "Icono minimalista".into(),
                style_context: Some("Negro mate y azul electrico".into()),
                output_path: out_ok.clone(),
                backend: ImageBackend::Gemini,
                custom_endpoint: None,
            },
            &gemini_url,
            "http://127.0.0.1:1",
            Some("test-key"),
            None,
        )
        .await
        .unwrap();

        assert_eq!(res.backend_used, "gemini");
        assert_eq!(res.validation.width, Some(128));
        assert!(out_ok.is_file());

        let svg_b64 = STANDARD.encode(
            b"<svg xmlns='http://www.w3.org/2000/svg' width='100' height='100'><rect/></svg>",
        );
        let openai_url =
            mock_server(json!({ "data": [{ "b64_json": svg_b64 }] }).to_string()).await;
        let out_bad = dir.join("fake.png");
        let err = generate_image_with_endpoints(
            &ImageGenRequest {
                prompt: "Fake".into(),
                style_context: None,
                output_path: out_bad.clone(),
                backend: ImageBackend::OpenAi,
                custom_endpoint: None,
            },
            "http://127.0.0.1:1",
            &openai_url,
            None,
            Some("test-key"),
        )
        .await
        .unwrap_err();

        assert!(matches!(err, ImageGenError::Gate(_)));
        assert!(!out_bad.exists());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
