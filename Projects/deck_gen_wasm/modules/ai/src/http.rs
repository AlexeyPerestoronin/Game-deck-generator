//! POST JSON via the browser crate (WASM) or a native stub.
//!
//! One POST may be repeated when the provider answers HTTP 503 (peak load).
//! Other failures, including 409, are returned as-is so the UI can warn.

use deck_gen_wasm_conf as wconf;

pub(crate) fn with_proxy(proxy_url: &str, url: &str) -> String {
    let proxy = proxy_url.trim();
    if proxy.is_empty() {
        return url.to_string();
    }
    if proxy.ends_with('?') || proxy.ends_with('=') || proxy.ends_with('/') {
        format!("{proxy}{url}")
    } else {
        format!("{}/{url}", proxy.trim_end_matches('/'))
    }
}

pub(crate) async fn post_json(
    url: &str,
    extra_headers: &[(&str, &str)],
    body: &str,
) -> Result<String, String> {
    let max_attempts = wconf::ai::HTTP_503_MAX_RETRIES.max(1);
    let mut attempt = 0;
    loop {
        attempt += 1;
        match post_json_once(url, extra_headers, body).await {
            Ok(text) => return Ok(text),
            Err(err) => {
                if !should_retry_503(&err, attempt, max_attempts) {
                    return Err(err);
                }
                sleep_ms(wconf::ai::HTTP_503_RETRY_DELAY_MS).await;
            }
        }
    }
}

async fn post_json_once(
    url: &str,
    extra_headers: &[(&str, &str)],
    body: &str,
) -> Result<String, String> {
    #[cfg(target_arch = "wasm32")]
    {
        deck_gen_wasm_browser::post_json(url, extra_headers, body).await
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = (url, extra_headers, body);
        Err("HTTP is only available in the browser WASM build".into())
    }
}

fn http_status_from_err(err: &str) -> Option<u16> {
    const MARK: &str = "HTTP ";
    let idx = err.find(MARK)?;
    let rest = &err[idx + MARK.len()..];
    let n = rest.bytes().take_while(u8::is_ascii_digit).count();
    if n == 0 {
        return None;
    }
    rest[..n].parse().ok()
}

fn should_retry_503(err: &str, attempt: u32, max_attempts: u32) -> bool {
    attempt < max_attempts && http_status_from_err(err) == Some(503)
}

async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    {
        gloo_timers::future::TimeoutFuture::new(ms).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
    }
}

pub(crate) async fn throttle(requests_per_second: Option<u32>, not_first: bool) {
    if !not_first {
        return;
    }
    let Some(rps) = requests_per_second else {
        return;
    };
    let ms = (1000u32 / rps.max(1)).max(1);
    sleep_ms(ms).await;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proxy_prefix_question_mark() {
        let u = with_proxy(
            "https://corsproxy.io/?",
            "https://api.deepseek.com/chat/completions",
        );
        assert_eq!(
            u,
            "https://corsproxy.io/?https://api.deepseek.com/chat/completions"
        );
    }

    #[test]
    fn empty_proxy_keeps_url() {
        assert_eq!(with_proxy("", "http://x"), "http://x");
        assert_eq!(with_proxy("  ", "http://x"), "http://x");
    }

    fn drive_retries(
        responses: &[Result<&str, &str>],
        max_attempts: u32,
    ) -> Result<String, String> {
        for (i, result) in responses.iter().enumerate() {
            let attempt = i as u32 + 1;
            match result {
                Ok(body) => return Ok((*body).to_string()),
                Err(err) => {
                    if !should_retry_503(err, attempt, max_attempts) {
                        return Err((*err).to_string());
                    }
                }
            }
        }
        Err("fixture ended while retrying".into())
    }

    #[test]
    fn status_from_browser_err_shapes() {
        assert_eq!(http_status_from_err("https://x: HTTP 503"), Some(503));
        assert_eq!(
            http_status_from_err("https://x: HTTP 503: overloaded"),
            Some(503)
        );
        assert_eq!(
            http_status_from_err("https://x: HTTP 409: conflict"),
            Some(409)
        );
        assert_eq!(http_status_from_err("https://x: HTTP 400: bad"), Some(400));
        assert_eq!(http_status_from_err("https://x: HTTP 401"), Some(401));
        assert_eq!(http_status_from_err("https://x: network down"), None);
        assert_eq!(
            http_status_from_err("HTTP is only available in the browser WASM build"),
            None
        );
    }

    #[test]
    fn retries_503_until_success() {
        let body = drive_retries(
            &[
                Err("https://x: HTTP 503"),
                Err("https://x: HTTP 503: busy"),
                Ok("ok"),
            ],
            20,
        )
        .unwrap();
        assert_eq!(body, "ok");
    }

    #[test]
    fn other_errors_fail_without_retry() {
        for err in [
            "https://x: HTTP 409: conflict",
            "https://x: HTTP 400: bad request",
            "https://x: HTTP 401: unauthorized",
            "https://x: failed to fetch",
        ] {
            let got = drive_retries(&[Err(err), Ok("nope")], 20).unwrap_err();
            assert_eq!(got, err);
        }
    }

    #[test]
    fn exhausted_503_is_err() {
        let err = "https://x: HTTP 503: overloaded";
        assert!(should_retry_503(err, 1, 3));
        assert!(should_retry_503(err, 2, 3));
        assert!(!should_retry_503(err, 3, 3));
        let got = drive_retries(&[Err(err), Err(err), Err(err), Ok("too late")], 3).unwrap_err();
        assert_eq!(got, err);
        assert!(!should_retry_503(err, 20, 20));
        assert!(should_retry_503(err, 19, wconf::ai::HTTP_503_MAX_RETRIES));
        assert!(!should_retry_503(err, 20, wconf::ai::HTTP_503_MAX_RETRIES));
    }
}
