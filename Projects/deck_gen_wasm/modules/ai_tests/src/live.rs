//! Ignored live Q&A against bundled model json5. Not run in CI.
//!
//! From the workspace root (`projects/`):
//! `cargo test -p deck_gen_wasm_ai_tests -- --ignored --nocapture --test-threads=1`

use std::io::{self, Write};

use deck_gen_wasm_ai::{complete_user_text, parse_model_conf, ModelConf};

const QUESTIONS: [&str; 3] = [
    "Reply with exactly one word: hello",
    "What is 1+1? Reply with a single number.",
    "Name any color in one word.",
];

const GEMINI_FLASH: &str = include_str!("../bundled/gemini-2.0-flash.json5");
const GEMINI_LITE: &str = include_str!("../bundled/gemini-3.1-flash-lite.json5");
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

fn run_live_qa(label: &str, json5: &str, path: &str) {
    let mut conf = parse_model_conf(json5, path).expect("bundled json5 must parse");
    let key = read_api_key(label);
    assert!(!key.is_empty(), "empty API key");
    conf.api_key = key;
    println!("\n=== {label} ({}) ===", conf.id);
    for (i, q) in QUESTIONS.iter().enumerate() {
        println!("Q{}: {q}", i + 1);
        let a = ask(&conf, q).unwrap_or_else(|err| panic!("A{} ERROR: {err}", i + 1));
        assert!(!a.trim().is_empty(), "A{} empty reply: {a}", i + 1);
        if q.contains("1+1") {
            assert!(a.contains('2'), "A{} expected 2, got: {a}", i + 1);
        }
        println!("A{}: {a}\n", i + 1);
    }
}

fn ask(conf: &ModelConf, question: &str) -> Result<String, String> {
    pollster::block_on(complete_user_text(conf, question))
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
fn live_qa_gemini_3_1_flash_lite() {
    run_live_qa(
        "Gemini 3.1 Flash-Lite",
        GEMINI_LITE,
        "ai-models/gemini-3.1-flash-lite.json5",
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
