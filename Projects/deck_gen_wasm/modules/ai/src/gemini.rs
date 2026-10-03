//! Gemini generateContent + function calling.

use serde_json::{json, Value};

use crate::conf::ModelConf;
use crate::engine::{LlmTurn, Msg, MsgRole, ToolCall};
use crate::http::{post_json, with_proxy};
use crate::tools::tool_declarations;

pub(crate) async fn complete(
    conf: &ModelConf,
    system: &str,
    messages: &[Msg],
) -> Result<LlmTurn, String> {
    let url = gemini_url(conf);
    let body = request_body(system, messages);
    let text = post_json(&url, &[], &body.to_string()).await?;
    parse_response(&text)
}

fn gemini_url(conf: &ModelConf) -> String {
    let mut url = format!(
        "{}/models/{}:generateContent",
        conf.base_url.trim_end_matches('/'),
        conf.id
    );
    let sep = if url.contains('?') { '&' } else { '?' };
    url = format!("{url}{sep}key={}", conf.api_key.trim());
    with_proxy(&conf.proxy_url, &url)
}

fn request_body(system: &str, messages: &[Msg]) -> Value {
    let decls = tool_declarations();
    json!({
        "system_instruction": { "parts": [{ "text": system }] },
        "contents": gemini_contents(messages),
        "tools": [{ "functionDeclarations": decls }]
    })
}

fn gemini_contents(messages: &[Msg]) -> Vec<Value> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < messages.len() {
        match messages[i].role {
            MsgRole::User => {
                out.push(json!({
                    "role": "user",
                    "parts": [{ "text": messages[i].text }]
                }));
                i += 1;
            }
            MsgRole::Assistant => {
                let mut parts = Vec::new();
                if !messages[i].text.is_empty() {
                    parts.push(json!({ "text": messages[i].text }));
                }
                for call in &messages[i].tool_calls {
                    parts.push(json!({
                        "functionCall": {
                            "name": call.name,
                            "args": call.args
                        }
                    }));
                }
                out.push(json!({ "role": "model", "parts": parts }));
                i += 1;
                let mut responses = Vec::new();
                while i < messages.len() && matches!(messages[i].role, MsgRole::Tool) {
                    responses.push(json!({
                        "functionResponse": {
                            "name": messages[i].tool_name,
                            "response": { "result": messages[i].text }
                        }
                    }));
                    i += 1;
                }
                if !responses.is_empty() {
                    out.push(json!({ "role": "user", "parts": responses }));
                }
            }
            MsgRole::Tool => {
                out.push(json!({
                    "role": "user",
                    "parts": [{
                        "functionResponse": {
                            "name": messages[i].tool_name,
                            "response": { "result": messages[i].text }
                        }
                    }]
                }));
                i += 1;
            }
        }
    }
    out
}

fn parse_response(text: &str) -> Result<LlmTurn, String> {
    let v: Value = serde_json::from_str(text)
        .map_err(|err| format!("gemini json: {err}; body: {}", crate::log::clip(text)))?;
    if let Some(err) = v.get("error") {
        let msg = err
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("gemini error");
        return Err(msg.to_string());
    }
    let parts = v
        .get("candidates")
        .and_then(Value::as_array)
        .and_then(|c| c.first())
        .and_then(|c| c.get("content"))
        .and_then(|c| c.get("parts"))
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let mut text_out = String::new();
    let mut reasoning = String::new();
    let mut tool_calls = Vec::new();
    for (idx, part) in parts.iter().enumerate() {
        if part.get("thought").and_then(Value::as_bool) == Some(true) {
            if let Some(t) = part.get("text").and_then(Value::as_str) {
                if !reasoning.is_empty() {
                    reasoning.push('\n');
                }
                reasoning.push_str(t);
            }
            continue;
        }
        if let Some(t) = part.get("text").and_then(Value::as_str) {
            text_out.push_str(t);
        }
        if let Some(fc) = part.get("functionCall") {
            let name = fc
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or("")
                .to_string();
            let args = fc.get("args").cloned().unwrap_or_else(|| json!({}));
            tool_calls.push(ToolCall {
                id: format!("call_{idx}"),
                name,
                args,
            });
        }
    }
    if text_out.is_empty() && tool_calls.is_empty() && reasoning.is_empty() {
        return Err(format!(
            "gemini: empty candidates; body: {}",
            crate::log::clip(text)
        ));
    }
    Ok(LlmTurn {
        text: text_out,
        reasoning,
        tool_calls,
    })
}
