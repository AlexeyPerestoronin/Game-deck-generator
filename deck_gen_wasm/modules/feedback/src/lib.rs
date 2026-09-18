//! Feedback mailto composer. Pure function + thin WASM sender.
//! No Leptos, no HTTP, no forms. Opens default mail client via location.href.

use deck_gen_wasm_conf as conf;

/// Percent-encode a string for mailto: query (subject/body).
/// Keeps unreserved chars, turns space to %20, \n to %0A, everything else byte-hex.
fn encode_component(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => out.push(c),
            ' ' => out.push_str("%20"),
            '\n' => out.push_str("%0A"),
            '\r' => out.push_str("%0D"),
            _ => {
                for b in c.to_string().as_bytes() {
                    out.push_str(&format!("%{:02X}", b));
                }
            }
        }
    }
    out
}

/// Build a mailto: URL with encoded subject and body.
/// Does not validate email format.
pub fn compose_mailto(email: &str, subject: &str, body: &str) -> String {
    let subj = encode_component(subject);
    let bod = encode_component(body);
    format!("mailto:{}?subject={}&body={}", email, subj, bod)
}

/// Read conf::feedback values, compose mailto, and navigate via window.location.
/// Returns Err if not in a browser context (no window).
pub fn send_feedback_via_email() -> Result<(), String> {
    let url = compose_mailto(
        conf::feedback::EMAIL,
        conf::feedback::SUBJECT,
        conf::feedback::TEMPLATE,
    );

    let window = web_sys::window().ok_or_else(|| "no browser window".to_string())?;
    let loc = window.location();
    loc.set_href(&url)
        .map_err(|e| format!("failed to set location: {:?}", e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compose_includes_email_subject_and_body() {
        let url = compose_mailto("Alexey.Perestoronin@yandex.ru", "Game-Deck-Generator Feedback", "hello");
        assert!(url.starts_with("mailto:Alexey.Perestoronin@yandex.ru?subject="));
        assert!(url.contains("Game-Deck-Generator%20Feedback"));
        assert!(url.contains("hello"));
    }

    #[test]
    fn compose_encodes_special_characters() {
        let url = compose_mailto("e@x", "s&b#", "line1\nline2\r\nok");
        assert!(url.contains("s%26b%23"));
        assert!(url.contains("line1%0Aline2%0D%0Aok"));
    }

    #[test]
    fn compose_preserves_allowed_chars() {
        let url = compose_mailto("e", "A_Z.a~9", "x");
        assert!(url.contains("subject=A_Z.a~9"));
    }

    #[test]
    fn uses_conf_feedback_values_and_template() {
        // Ensures include_str in conf works and values are present
        let url = compose_mailto(conf::feedback::EMAIL, conf::feedback::SUBJECT, conf::feedback::TEMPLATE);
        assert!(url.starts_with("mailto:Alexey.Perestoronin@yandex.ru?subject=Game-Deck-Generator%20Feedback"));
        assert!(url.contains("What%20I%20was%20trying%20to%20do"));
        assert!(url.contains("Browser%20%2F%20OS"));
    }
}
