//! Ignored live Q&A against bundled model json5. Not run in CI.
//!
//! From the workspace root (`projects/`):
//! `cargo test -p deck_gen_wasm_ai -- --ignored --nocapture --test-threads=1 live_qa_`

use std::io::{self, Write};
use std::time::Duration;

use serde_json::{json, Value};

use crate::conf::{parse_model_conf, ModelConf, ModelKind};

const QUESTIONS: [&str; 3] = [
    "Reply with exactly one word: hello",
    "What is 1+1? Reply with a single number.",
    "Name any color in one word.",
];

const GEMINI_FLASH: &str = include_str!("../bundled/gemini-2.0-flash.json5");
const GEMINI_LITE: &str = include_str!("../bundled/gemini-2.0-flash-lite.json5");
const DEEPSEEK: &str = include_str!("../bundled/deepseek-chat.json5");
const GROK: &str = include_str!("../bundled/grok-3-mini.json5");

fn read_api_key(label: &str) -> String {
    eprint!("API key for {label}: ");
    let _ = io::stderr().flush();
    let mut line = String::new();
    io::stdin()
        .read_line(&mut line)
        .expect("stdin is required for the API key");
    line.trim().to_string()
}

fn post_json(url: &str, headers: &[(&str, &str)], body: &str) -> Result<String, String> {
    let mut req = ureq::post(url).timeout(Duration::from_secs(120));
    for (k, v) in headers {
        req = req.set(k, v);
    }
    let resp = req
        .set("Content-Type", "application/json")
        .send_string(body)
        .map_err(|err| format!("{url}: {err}"))?;
    resp.into_string()
        .map_err(|err| format!("{url}: {err}"))
}

fn ask(conf: &ModelConf, question: &str) -> Result<String, String> {
    match conf.kind {
        ModelKind::Gemini => {
            let url = format!(
                "{}/models/{}:generateContent?key={}",
                conf.base_url.trim_end_matches('/'),
                conf.id,
                conf.api_key.trim()
            );
            let body = json!({
                "contents": [{ "role": "user", "parts": [{ "text": question }] }]
            });
            let raw = post_json(&url, &[], &body.to_string())?;
            Ok(extract_gemini(&raw).unwrap_or(raw))
        }
        ModelKind::OpenAiCompat => {
            let base = conf.base_url.trim_end_matches('/');
            let url = if base.ends_with("/chat/completions") {
                base.to_string()
            } else {
                format!("{base}/chat/completions")
            };
            let body = json!({
                "model": conf.id,
                "messages": [{ "role": "user", "content": question }]
            });
            let auth = format!("Bearer {}", conf.api_key.trim());
            let raw = post_json(&url, &[("Authorization", auth.as_str())], &body.to_string())?;
            Ok(extract_openai(&raw).unwrap_or(raw))
        }
    }
}

fn extract_gemini(raw: &str) -> Option<String> {
    let v: Value = serde_json::from_str(raw).ok()?;
    let mut out = String::new();
    let parts = v
        .get("candidates")?
        .as_array()?
        .first()?
        .get("content")?
        .get("parts")?
        .as_array()?;
    for part in parts {
        if let Some(t) = part.get("text").and_then(Value::as_str) {
            out.push_str(t);
        }
    }
    if out.is_empty() {
        None
    } else {
        Some(out)
    }
}

fn extract_openai(raw: &str) -> Option<String> {
    let v: Value = serde_json::from_str(raw).ok()?;
    v.get("choices")?
        .as_array()?
        .first()?
        .get("message")?
        .get("content")?
        .as_str()
        .map(str::to_string)
}

fn run_live_qa(label: &str, json5: &str, path: &str) {
    let mut conf = parse_model_conf(json5, path).expect("bundled json5 must parse");
    let key = read_api_key(label);
    assert!(!key.is_empty(), "empty API key");
    conf.api_key = key;
    println!("\n=== {label} ({}) ===", conf.id);
    for (i, q) in QUESTIONS.iter().enumerate() {
        println!("Q{}: {q}", i + 1);
        match ask(&conf, q) {
            Ok(a) => println!("A{}: {a}\n", i + 1),
            Err(err) => println!("A{} ERROR: {err}\n", i + 1),
        }
    }
}

#[test]
#[ignore]
fn live_qa_gemini_2_0_flash() {
    run_live_qa(
        "Gemini 2.0 Flash",
        GEMINI_FLASH,
        "ai-models/gemini-2.0-flash.json5",
    );
}

#[test]
#[ignore]
fn live_qa_gemini_2_0_flash_lite() {
    run_live_qa(
        "Gemini 2.0 Flash-Lite",
        GEMINI_LITE,
        "ai-models/gemini-2.0-flash-lite.json5",
    );
}

#[test]
#[ignore]
fn live_qa_deepseek_chat() {
    run_live_qa("DeepSeek Chat", DEEPSEEK, "ai-models/deepseek-chat.json5");
}

#[test]
#[ignore]
fn live_qa_grok_3_mini() {
    run_live_qa("Grok 3 Mini", GROK, "ai-models/grok-3-mini.json5");
}
