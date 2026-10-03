//! HTTP GET used by GitHub for template blobs, and POST JSON for AI clients.
//!
//! The “new game” action needs the same “fetch URL → UTF-8 body” path for the
//! git tree and raw files. Keeping it here avoids duplicating status-code
//! handling in [`crate::github`]. LLM clients POST JSON through [`post_json`].

use gloo_net::http::Request;

/// GET `url` and return the response body as text.
pub async fn fetch_text(url: &str) -> Result<String, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|err| format!("{url}: {err}"))?;
    if !response.ok() {
        return Err(format!("{url}: HTTP {}", response.status()));
    }
    response.text().await.map_err(|err| format!("{url}: {err}"))
}

/// GET `url` and return the response body as bytes.
pub async fn fetch_bytes(url: &str) -> Result<Vec<u8>, String> {
    let response = Request::get(url)
        .send()
        .await
        .map_err(|err| format!("{url}: {err}"))?;
    if !response.ok() {
        return Err(format!("{url}: HTTP {}", response.status()));
    }
    response
        .binary()
        .await
        .map_err(|err| format!("{url}: {err}"))
}

/// POST JSON `body` to `url` and return the response body as text.
pub async fn post_json(
    url: &str,
    extra_headers: &[(&str, &str)],
    body: &str,
) -> Result<String, String> {
    let mut builder = Request::post(url).header("Content-Type", "application/json");
    for (key, value) in extra_headers {
        builder = builder.header(key, value);
    }
    let request = builder
        .body(body)
        .map_err(|err| format!("{url}: {err}"))?;
    let response = request
        .send()
        .await
        .map_err(|err| format!("{url}: {err}"))?;
    if !response.ok() {
        let status = response.status();
        let text = response.text().await.unwrap_or_default();
        if text.is_empty() {
            return Err(format!("{url}: HTTP {status}"));
        }
        return Err(format!("{url}: HTTP {status}: {text}"));
    }
    response.text().await.map_err(|err| format!("{url}: {err}"))
}
