//! HTTP GET used by GitHub for template blobs.
//!
//! The “new game” action needs the same “fetch URL → UTF-8 body” path for the
//! git tree and raw files. Keeping it here avoids duplicating status-code
//! handling in [`crate::github`].

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
