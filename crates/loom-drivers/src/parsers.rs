use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StreamEvent {
    SessionInit {
        session_id: String,
    },
    Thinking {
        text: String,
    },
    ToolUse {
        name: String,
        target: Option<String>,
    },
    TextDelta {
        text: String,
    },
    Usage {
        input: u64,
        output: u64,
        cache_read: u64,
        cache_write: u64,
    },
    Error {
        message: String,
    },
    Completed,
}

pub fn parse_jsonl_line(provider_id: &str, line: &str) -> Option<StreamEvent> {
    let trimmed = line.trim();
    if trimmed.is_empty() {
        return None;
    }

    if provider_id == "omniroute" {
        return parse_omniroute_sse(trimmed);
    }

    let val: Value = serde_json::from_str(trimmed).ok()?;
    match provider_id {
        "claude" => parse_claude(&val),
        "codex" => parse_codex(&val),
        "gemini" => parse_gemini(&val),
        "opencode" => parse_opencode(&val),
        _ => parse_generic(&val),
    }
}

fn parse_claude(val: &Value) -> Option<StreamEvent> {
    let kind = val.get("type").and_then(Value::as_str)?;
    match kind {
        "system" => {
            let sid = val.get("session_id").and_then(Value::as_str)?;
            Some(StreamEvent::SessionInit {
                session_id: sid.to_string(),
            })
        }
        "assistant" => {
            let blocks = val.pointer("/message/content").and_then(Value::as_array)?;
            for block in blocks {
                let btype = block.get("type").and_then(Value::as_str).unwrap_or("");
                if btype == "thinking"
                    && let Some(t) = block.get("thinking").and_then(Value::as_str)
                {
                    return Some(StreamEvent::Thinking {
                        text: t.to_string(),
                    });
                }
                if btype == "tool_use"
                    && let Some(name) = block.get("name").and_then(Value::as_str)
                {
                    let target = block
                        .pointer("/input/file_path")
                        .or_else(|| block.pointer("/input/command"))
                        .and_then(Value::as_str)
                        .map(str::to_string);
                    return Some(StreamEvent::ToolUse {
                        name: name.to_string(),
                        target,
                    });
                }
                if btype == "text"
                    && let Some(t) = block.get("text").and_then(Value::as_str)
                {
                    return Some(StreamEvent::TextDelta {
                        text: t.to_string(),
                    });
                }
            }
            None
        }
        "result" => {
            if val.get("is_error").and_then(Value::as_bool) == Some(true) {
                let msg = val
                    .get("result")
                    .and_then(Value::as_str)
                    .unwrap_or("error desconocido");
                return Some(StreamEvent::Error {
                    message: msg.to_string(),
                });
            }
            if let Some(u) = val.get("usage") {
                return Some(StreamEvent::Usage {
                    input: u64_at(u, "input_tokens"),
                    output: u64_at(u, "output_tokens"),
                    cache_read: u64_at(u, "cache_read_input_tokens"),
                    cache_write: u64_at(u, "cache_creation_input_tokens"),
                });
            }
            Some(StreamEvent::Completed)
        }
        _ => None,
    }
}

fn parse_codex(val: &Value) -> Option<StreamEvent> {
    let kind = val.get("type").and_then(Value::as_str)?;
    match kind {
        "thread.started" => {
            let sid = val.get("thread_id").and_then(Value::as_str)?;
            Some(StreamEvent::SessionInit {
                session_id: sid.to_string(),
            })
        }
        "item.completed" => {
            let item = val.get("item")?;
            let itype = item.get("type").and_then(Value::as_str).unwrap_or("");
            match itype {
                "reasoning" => Some(StreamEvent::Thinking {
                    text: item
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                }),
                "command_execution" | "mcp_tool_call" => Some(StreamEvent::ToolUse {
                    name: itype.to_string(),
                    target: item
                        .get("command")
                        .or_else(|| item.get("tool"))
                        .and_then(Value::as_str)
                        .map(str::to_string),
                }),
                "agent_message" => Some(StreamEvent::TextDelta {
                    text: item
                        .get("text")
                        .and_then(Value::as_str)
                        .unwrap_or("")
                        .to_string(),
                }),
                _ => None,
            }
        }
        "turn.completed" => {
            if let Some(u) = val.get("usage") {
                return Some(StreamEvent::Usage {
                    input: u64_at(u, "input_tokens"),
                    output: u64_at(u, "output_tokens"),
                    cache_read: u64_at(u, "cached_input_tokens"),
                    cache_write: 0,
                });
            }
            Some(StreamEvent::Completed)
        }
        "error" | "turn.failed" => Some(StreamEvent::Error {
            message: val
                .get("message")
                .or_else(|| val.pointer("/error/message"))
                .and_then(Value::as_str)
                .unwrap_or("fallo en codex")
                .to_string(),
        }),
        _ => None,
    }
}

fn parse_gemini(val: &Value) -> Option<StreamEvent> {
    let kind = val.get("type").and_then(Value::as_str)?;
    match kind {
        "init" => Some(StreamEvent::SessionInit {
            session_id: val
                .get("session_id")
                .and_then(Value::as_str)
                .unwrap_or("gemini")
                .to_string(),
        }),
        "message" => {
            let text = val.get("content").and_then(Value::as_str)?;
            if val.get("role").and_then(Value::as_str) == Some("assistant") {
                Some(StreamEvent::TextDelta {
                    text: text.to_string(),
                })
            } else {
                None
            }
        }
        "tool_use" => Some(StreamEvent::ToolUse {
            name: val
                .get("tool_name")
                .or_else(|| val.get("name"))
                .and_then(Value::as_str)
                .unwrap_or("tool")
                .to_string(),
            target: None,
        }),
        "result" => {
            if let Some(stats) = val.get("stats") {
                return Some(StreamEvent::Usage {
                    input: u64_at(stats, "input_tokens"),
                    output: u64_at(stats, "output_tokens"),
                    cache_read: u64_at(stats, "cached"),
                    cache_write: 0,
                });
            }
            Some(StreamEvent::Completed)
        }
        "error" => Some(StreamEvent::Error {
            message: val
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("error en gemini")
                .to_string(),
        }),
        _ => None,
    }
}

fn parse_opencode(val: &Value) -> Option<StreamEvent> {
    let kind = val.get("type").and_then(Value::as_str)?;
    match kind {
        "step_start" => Some(StreamEvent::SessionInit {
            session_id: val
                .get("sessionID")
                .and_then(Value::as_str)
                .unwrap_or("opencode")
                .to_string(),
        }),
        "text" => Some(StreamEvent::TextDelta {
            text: val
                .pointer("/part/text")
                .or_else(|| val.get("text"))
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string(),
        }),
        "tool_use" => Some(StreamEvent::ToolUse {
            name: val
                .pointer("/part/tool")
                .and_then(Value::as_str)
                .unwrap_or("tool")
                .to_string(),
            target: None,
        }),
        "step_finish" => {
            if let Some(u) = val.pointer("/part/tokens") {
                return Some(StreamEvent::Usage {
                    input: u64_at(u, "input"),
                    output: u64_at(u, "output"),
                    cache_read: u64_at(u.get("cache").unwrap_or(u), "read"),
                    cache_write: u64_at(u.get("cache").unwrap_or(u), "write"),
                });
            }
            Some(StreamEvent::Completed)
        }
        "error" => Some(StreamEvent::Error {
            message: val
                .pointer("/error/message")
                .and_then(Value::as_str)
                .unwrap_or("error en opencode")
                .to_string(),
        }),
        _ => None,
    }
}

fn parse_omniroute_sse(line: &str) -> Option<StreamEvent> {
    let payload = line.strip_prefix("data:")?.trim();
    if payload == "[DONE]" {
        return Some(StreamEvent::Completed);
    }
    let val: Value = serde_json::from_str(payload).ok()?;
    if let Some(u) = val.get("usage") {
        return Some(StreamEvent::Usage {
            input: u64_at(u, "prompt_tokens"),
            output: u64_at(u, "completion_tokens"),
            cache_read: val
                .pointer("/usage/prompt_tokens_details/cached_tokens")
                .and_then(Value::as_u64)
                .unwrap_or(0),
            cache_write: 0,
        });
    }
    if let Some(delta) = val
        .pointer("/choices/0/delta/content")
        .and_then(Value::as_str)
        && !delta.is_empty()
    {
        return Some(StreamEvent::TextDelta {
            text: delta.to_string(),
        });
    }
    None
}

fn parse_generic(val: &Value) -> Option<StreamEvent> {
    if let Some(msg) = val.pointer("/error/message").and_then(Value::as_str) {
        return Some(StreamEvent::Error {
            message: msg.to_string(),
        });
    }
    if let Some(text) = val
        .get("text")
        .or_else(|| val.get("content"))
        .and_then(Value::as_str)
    {
        return Some(StreamEvent::TextDelta {
            text: text.to_string(),
        });
    }
    None
}

fn u64_at(obj: &Value, key: &str) -> u64 {
    obj.get(key).and_then(Value::as_u64).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_all_five_provider_formats_and_ignores_corrupt_lines() {
        assert_eq!(
            parse_jsonl_line(
                "claude",
                r#"{"type":"system","subtype":"init","session_id":"c-123"}"#
            ),
            Some(StreamEvent::SessionInit {
                session_id: "c-123".into()
            })
        );
        assert_eq!(
            parse_jsonl_line(
                "claude",
                r#"{"type":"result","usage":{"input_tokens":10,"output_tokens":20,"cache_read_input_tokens":300,"cache_creation_input_tokens":5}}"#
            ),
            Some(StreamEvent::Usage {
                input: 10,
                output: 20,
                cache_read: 300,
                cache_write: 5
            })
        );

        assert_eq!(
            parse_jsonl_line("codex", r#"{"type":"thread.started","thread_id":"t-9"}"#),
            Some(StreamEvent::SessionInit {
                session_id: "t-9".into()
            })
        );
        assert_eq!(
            parse_jsonl_line(
                "codex",
                r#"{"type":"turn.completed","usage":{"input_tokens":100,"cached_input_tokens":80,"output_tokens":12}}"#
            ),
            Some(StreamEvent::Usage {
                input: 100,
                output: 12,
                cache_read: 80,
                cache_write: 0
            })
        );

        assert_eq!(
            parse_jsonl_line(
                "gemini",
                r#"{"type":"message","role":"assistant","content":"listo"}"#
            ),
            Some(StreamEvent::TextDelta {
                text: "listo".into()
            })
        );

        assert_eq!(
            parse_jsonl_line(
                "opencode",
                r#"{"type":"step_finish","part":{"tokens":{"input":50,"output":15,"cache":{"read":10,"write":2}}}}"#
            ),
            Some(StreamEvent::Usage {
                input: 50,
                output: 15,
                cache_read: 10,
                cache_write: 2
            })
        );

        assert_eq!(
            parse_jsonl_line(
                "omniroute",
                r#"data: {"choices":[{"delta":{"content":"OK"}}]}"#
            ),
            Some(StreamEvent::TextDelta { text: "OK".into() })
        );
        assert_eq!(
            parse_jsonl_line("omniroute", "data: [DONE]"),
            Some(StreamEvent::Completed)
        );

        assert_eq!(parse_jsonl_line("claude", ""), None);
        assert_eq!(parse_jsonl_line("codex", "{corrupto"), None);
        assert_eq!(parse_jsonl_line("gemini", "texto plano suelto"), None);
    }
}
