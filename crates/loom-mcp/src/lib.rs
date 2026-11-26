use loom_core::capsule::{CapsuleRequest, compile_capsule};
use loom_core::config::{LoomConfig, default_providers_dir};
use loom_core::gates::{GateConfig, evaluate_diff, validate_raster_image};
use loom_core::providers::ProviderRegistry;
use loom_core::router::{TaskRole, read_claudehub_quota, route_task};
use loom_core::skills::SkillIndex;
use loom_core::types::Sensitivity;
use loom_drivers::doctor::run_doctor;
use loom_drivers::image_gen::{ImageBackend, ImageGenRequest, generate_image};
use serde_json::{Value, json};
use std::path::PathBuf;

pub struct McpServer {
    config: LoomConfig,
}

impl McpServer {
    pub fn new(config: LoomConfig) -> Self {
        Self { config }
    }

    pub async fn handle_request(&self, req: Value) -> Option<Value> {
        let method = req.get("method").and_then(Value::as_str)?;
        let id = req.get("id").cloned();

        if id.is_none() || method == "notifications/initialized" {
            return None;
        }
        let id = id.unwrap_or(Value::Null);

        let result = match method {
            "initialize" => Ok(json!({
                "protocolVersion": "2024-11-05",
                "capabilities": { "tools": {} },
                "serverInfo": {
                    "name": "loom-mcp",
                    "version": env!("CARGO_PKG_VERSION")
                }
            })),
            "ping" => Ok(json!({})),
            "tools/list" => Ok(json!({ "tools": tool_definitions() })),
            "tools/call" => {
                let params = req.get("params").cloned().unwrap_or(Value::Null);
                let name = params.get("name").and_then(Value::as_str).unwrap_or("");
                let args = params
                    .get("arguments")
                    .cloned()
                    .unwrap_or_else(|| json!({}));
                self.call_tool(name, &args).await
            }
            _ => Err((-32601, format!("metodo no soportado: {method}"))),
        };

        Some(match result {
            Ok(res) => json!({ "jsonrpc": "2.0", "id": id, "result": res }),
            Err((code, message)) => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": code, "message": message } })
            }
        })
    }

    async fn call_tool(&self, name: &str, args: &Value) -> Result<Value, (i64, String)> {
        match name {
            "loom_doctor" => {
                let registry =
                    ProviderRegistry::load(&default_providers_dir(), &self.config.omniroute_url)
                        .map_err(|e| (-32000, e.to_string()))?;
                let report = run_doctor(&self.config, &registry).await;
                tool_json(&report)
            }
            "loom_get_context" => {
                let repo = str_arg(args, "repo_root").ok_or((-32602, "falta repo_root".into()))?;
                let objective =
                    str_arg(args, "objective").ok_or((-32602, "falta objective".into()))?;
                let allowed_files = str_array_arg(args, "allowed_files");
                let skill_names = str_array_arg(args, "skills");
                let cap = compile_capsule(&CapsuleRequest {
                    repo_root: PathBuf::from(repo),
                    vault_path: self.config.vault_path.clone(),
                    project_slug: str_arg(args, "project_slug").map(str::to_string),
                    objective: objective.to_string(),
                    allowed_files,
                    verification_cmd: str_arg(args, "verification_cmd").map(str::to_string),
                    skill_names,
                });
                tool_json(&cap)
            }
            "loom_list_skills" => {
                let repo = str_arg(args, "repo_root").map(PathBuf::from);
                let query = str_arg(args, "query").unwrap_or("");
                let index = SkillIndex::discover(repo.as_deref());
                let items: Vec<_> = index.search(query).into_iter().cloned().collect();
                tool_json(&items)
            }
            "loom_load_skill" => {
                let skill_name = str_arg(args, "name").ok_or((-32602, "falta name".into()))?;
                let repo = str_arg(args, "repo_root").map(PathBuf::from);
                let index = SkillIndex::discover(repo.as_deref());
                let content = index
                    .load_content(skill_name)
                    .map_err(|e| (-32000, e.to_string()))?;
                Ok(tool_text(content))
            }
            "loom_route_task" => {
                let role_str = str_arg(args, "role").unwrap_or("implementer");
                let sens_str = str_arg(args, "sensitivity").unwrap_or("internal");
                let role: TaskRole = serde_json::from_value(Value::String(role_str.to_string()))
                    .map_err(|_| (-32602, format!("rol invalido: {role_str}")))?;
                let sensitivity: Sensitivity =
                    serde_json::from_value(Value::String(sens_str.to_string()))
                        .map_err(|_| (-32602, format!("sensibilidad invalida: {sens_str}")))?;
                let registry =
                    ProviderRegistry::load(&default_providers_dir(), &self.config.omniroute_url)
                        .map_err(|e| (-32000, e.to_string()))?;
                let quota = read_claudehub_quota(None);
                let decision = route_task(role, sensitivity, &registry, &quota)
                    .map_err(|e| (-32000, e.to_string()))?;
                tool_json(&decision)
            }
            "loom_gate_check" => {
                let diff = str_arg(args, "diff").unwrap_or("");
                let changed = str_array_arg(args, "changed_files");
                let allowed_paths = str_array_arg(args, "allowed_paths");
                let report = evaluate_diff(
                    diff,
                    &changed,
                    &GateConfig {
                        allowed_paths,
                        ..Default::default()
                    },
                );
                tool_json(&report)
            }
            "loom_validate_image" => {
                let path = str_arg(args, "path").ok_or((-32602, "falta path".into()))?;
                let info = validate_raster_image(&PathBuf::from(path), 64)
                    .map_err(|e| (-32000, e.to_string()))?;
                tool_json(&info)
            }
            "loom_generate_image" => {
                let prompt = str_arg(args, "prompt").ok_or((-32602, "falta prompt".into()))?;
                let out =
                    str_arg(args, "output_path").ok_or((-32602, "falta output_path".into()))?;
                let backend = match str_arg(args, "backend").unwrap_or("auto") {
                    "gemini" => ImageBackend::Gemini,
                    "openai" => ImageBackend::OpenAi,
                    "custom_http" => ImageBackend::CustomHttp,
                    _ => ImageBackend::Auto,
                };
                let res = generate_image(&ImageGenRequest {
                    prompt: prompt.to_string(),
                    style_context: str_arg(args, "style_context").map(str::to_string),
                    output_path: PathBuf::from(out),
                    backend,
                    custom_endpoint: str_arg(args, "custom_endpoint").map(str::to_string),
                })
                .await
                .map_err(|e| (-32000, e.to_string()))?;
                tool_json(&res)
            }
            _ => Err((-32601, format!("herramienta desconocida: {name}"))),
        }
    }
}

fn tool_json<T: serde::Serialize>(val: &T) -> Result<Value, (i64, String)> {
    let text = serde_json::to_string_pretty(val).map_err(|e| (-32000, e.to_string()))?;
    Ok(tool_text(text))
}

fn tool_text(text: String) -> Value {
    json!({
        "content": [{ "type": "text", "text": text }]
    })
}

fn str_arg<'a>(args: &'a Value, key: &str) -> Option<&'a str> {
    args.get(key).and_then(Value::as_str)
}

fn str_array_arg(args: &Value, key: &str) -> Vec<String> {
    args.get(key)
        .and_then(Value::as_array)
        .map(|arr| {
            arr.iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn tool_definitions() -> Value {
    json!([
        {
            "name": "loom_doctor",
            "description": "Diagnostica herramientas CLI, servicios locales y estado del entorno.",
            "inputSchema": { "type": "object", "properties": {} }
        },
        {
            "name": "loom_get_context",
            "description": "Compila una capsula de contexto corta desde Obsidian/.loom y tokens UI del repo.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_root": { "type": "string" },
                    "objective": { "type": "string" },
                    "project_slug": { "type": "string" },
                    "allowed_files": { "type": "array", "items": { "type": "string" } },
                    "verification_cmd": { "type": "string" },
                    "skills": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["repo_root", "objective"]
            }
        },
        {
            "name": "loom_list_skills",
            "description": "Lista o busca skills SKILL.md indexadas en el proyecto y directorios globales.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "repo_root": { "type": "string" },
                    "query": { "type": "string" }
                }
            }
        },
        {
            "name": "loom_load_skill",
            "description": "Carga el contenido completo de una skill por nombre.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "repo_root": { "type": "string" }
                },
                "required": ["name"]
            }
        },
        {
            "name": "loom_route_task",
            "description": "Selecciona el mejor proveedor respetando Privacy Gate y cuotas de ClaudeHub.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "role": { "type": "string" },
                    "sensitivity": { "type": "string" }
                }
            }
        },
        {
            "name": "loom_gate_check",
            "description": "Valida un diff contra alcance, reglas TypeScript, emojis y tokens UI.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "diff": { "type": "string" },
                    "changed_files": { "type": "array", "items": { "type": "string" } },
                    "allowed_paths": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["diff", "changed_files"]
            }
        },
        {
            "name": "loom_validate_image",
            "description": "Verifica que una imagen sea PNG/WebP/JPEG real y rechaza SVG disfrazados.",
            "inputSchema": {
                "type": "object",
                "properties": { "path": { "type": "string" } },
                "required": ["path"]
            }
        },
        {
            "name": "loom_generate_image",
            "description": "Genera una imagen raster real con contexto visual del proyecto y Media Quality Gate.",
            "inputSchema": {
                "type": "object",
                "properties": {
                    "prompt": { "type": "string" },
                    "output_path": { "type": "string" },
                    "style_context": { "type": "string" },
                    "backend": { "type": "string" },
                    "custom_endpoint": { "type": "string" }
                },
                "required": ["prompt", "output_path"]
            }
        }
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn handles_initialize_tools_list_route_skills_and_gate() {
        let server = McpServer::new(LoomConfig::default());

        let init = server
            .handle_request(json!({ "jsonrpc": "2.0", "id": 1, "method": "initialize" }))
            .await
            .unwrap();
        assert_eq!(init["result"]["protocolVersion"], "2024-11-05");

        let list = server
            .handle_request(json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" }))
            .await
            .unwrap();
        assert_eq!(list["result"]["tools"].as_array().unwrap().len(), 8);

        let route = server
            .handle_request(json!({
                "jsonrpc": "2.0",
                "id": 3,
                "method": "tools/call",
                "params": {
                    "name": "loom_route_task",
                    "arguments": { "role": "fast_free", "sensitivity": "confidential" }
                }
            }))
            .await
            .unwrap();
        let route_text = route["result"]["content"][0]["text"].as_str().unwrap();
        assert!(route_text.contains("\"provider_id\": \"claude\""));

        let skills = server
            .handle_request(json!({
                "jsonrpc": "2.0",
                "id": 4,
                "method": "tools/call",
                "params": {
                    "name": "loom_list_skills",
                    "arguments": { "query": "no-match-xyz-999" }
                }
            }))
            .await
            .unwrap();
        assert_eq!(
            skills["result"]["content"][0]["text"].as_str().unwrap(),
            "[]"
        );

        let gate = server
            .handle_request(json!({
                "jsonrpc": "2.0",
                "id": 5,
                "method": "tools/call",
                "params": {
                    "name": "loom_gate_check",
                    "arguments": {
                        "diff": "+++ b/src/a.ts\n+const x: any = 1",
                        "changed_files": ["src/a.ts"],
                        "allowed_paths": ["src/a.ts"]
                    }
                }
            }))
            .await
            .unwrap();
        let gate_text = gate["result"]["content"][0]["text"].as_str().unwrap();
        assert!(gate_text.contains("\"passed\": false"));
    }
}
