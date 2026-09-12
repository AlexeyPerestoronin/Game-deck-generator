//! HTTP GET used by GitHub and the local Trunk copy of help.
//!
//! GitHub blobs and first-load help need the same “fetch URL → UTF-8 body”
//! path. Keeping it here avoids duplicating status-code handling in
//! [`crate::github`] and [`crate::help`].

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
