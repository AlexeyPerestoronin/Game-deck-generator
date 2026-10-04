//! OpenAI-compatible chat/completions + tool calling (DeepSeek, Grok, custom).

use serde_json::{json, Value};

use crate::conf::ModelConf;
use crate::engine::{bearer_headers, LlmTurn, Msg, MsgRole, ToolCall};
use crate::http::{post_json, with_proxy};
use crate::tools::tool_declarations;

pub(crate) async fn complete(
    conf: &ModelConf,
    system: &str,
    messages: &[Msg],
) -> Result<LlmTurn, String> {
    let url = openai_url(conf);
    let body = request_body(conf, system, messages);
    let headers = bearer_headers(conf);
    let header_refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let text = post_json(&url, &header_refs, &body.to_string()).await?;
    parse_response(&text)
}

/// One user turn: same URL, headers, and response parse as [`complete`], no tools.
pub(crate) async fn complete_plain(conf: &ModelConf, question: &str) -> Result<LlmTurn, String> {
    let url = openai_url(conf);
    let body = json!({
        "model": conf.id,
        "messages": openai_messages(&[Msg::user(question)])
    });
    let headers = bearer_headers(conf);
    let header_refs: Vec<(&str, &str)> = headers.iter().map(|(k, v)| (*k, v.as_str())).collect();
    let text = post_json(&url, &header_refs, &body.to_string()).await?;
    parse_response(&text)
}

fn openai_url(conf: &ModelConf) -> String {
    let base = conf.base_url.trim_end_matches('/');
    let url = if base.ends_with("/chat/completions") {
        base.to_string()
    } else {
        format!("{base}/chat/completions")
    };
    with_proxy(&conf.proxy_url, &url)
}

fn request_body(conf: &ModelConf, system: &str, messages: &[Msg]) -> Value {
    let mut msgs = vec![json!({"role": "system", "content": system})];
    msgs.extend(openai_messages(messages));
    let decls = tool_declarations();
    let tools: Vec<Value> = decls
        .as_array()
        .unwrap_or(&Vec::new())
        .iter()
        .map(|d| {
            json!({
                "type": "function",
                "function": d
            })
        })
        .collect();
    json!({
        "model": conf.id,
        "messages": msgs,
        "tools": tools
    })
}

fn openai_messages(messages: &[Msg]) -> Vec<Value> {
    let mut out = Vec::new();
    for msg in messages {
        match msg.role {
            MsgRole::User => {
                out.push(json!({"role": "user", "content": msg.text}));
            }
            MsgRole::Assistant => {
                let mut m = json!({
                    "role": "assistant",
                    "content": if msg.text.is_empty() { Value::Null } else { Value::String(msg.text.clone()) }
                });
                if !msg.tool_calls.is_empty() {
                    let calls: Vec<Value> = msg
                        .tool_calls
                        .iter()
                        .map(|c| {
                            json!({
                                "id": c.id,
                                "type": "function",
                                "function": {
                                    "name": c.name,
                                    "arguments": c.args.to_string()
                                }
                            })
                        })
                        .collect();
                    m["tool_calls"] = Value::Array(calls);
                }
                out.push(m);
            }
            MsgRole::Tool => {
                out.push(json!({
                    "role": "tool",
                    "tool_call_id": msg.tool_id,
                    "content": msg.text
                }));
            }
        }
    }
    out
}

fn parse_response(text: &str) -> Result<LlmTurn, String> {
    let v: Value = serde_json::from_str(text)
        .map_err(|err| format!("openai json: {err}; body: {}", crate::log::clip(text)))?;
    if let Some(err) = v.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .or_else(|| err.as_str())
            .unwrap_or("openai error");
        return Err(msg.to_string());
    }
    let message = v
        .get("choices")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .and_then(|c| c.get("message"))
        .cloned()
        .ok_or_else(|| format!("openai: no choices; body: {}", crate::log::clip(text)))?;
    let text_out = message
        .get("content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let reasoning = message
        .get("reasoning_content")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let mut tool_calls = Vec::new();
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        for (idx, call) in calls.iter().enumerate() {
            let id = call
                .get("id")
                .and_then(Value::as_str)
                .map(str::to_string)
                .unwrap_or_else(|| format!("call_{idx}"));
            let func = call.get("function").cloned().unwrap_or(Value::Null);
            let name = func
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let args_raw = func
                .get("arguments")
                .and_then(Value::as_str)
                .unwrap_or("{}");
            let args = serde_json::from_str(args_raw).unwrap_or_else(|_| json!({}));
            tool_calls.push(ToolCall { id, name, args });
        }
    }
    Ok(LlmTurn {
        text: text_out,
        reasoning,
        tool_calls,
    })
}
