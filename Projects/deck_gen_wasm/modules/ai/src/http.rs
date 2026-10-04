//! POST JSON via the browser crate (WASM) or a native stub.
//!
//! One POST may be repeated when the provider answers HTTP 503 (peak load).
//! Other failures, including 409, are returned as-is so the UI can warn.
//! LLM calls are paced by `requests_per_minute` (gap + 60s sliding window).

use std::sync::Mutex;

use deck_gen_wasm_conf as wconf;

const WINDOW_MS: u64 = 60_000;

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

fn now_ms() -> u64 {
    #[cfg(target_arch = "wasm32")]
    {
        js_sys::Date::now() as u64
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

/// Minimum gap between LLM HTTP calls for RPM `n`.
pub(crate) fn interval_ms(n: u32) -> u32 {
    if n == 0 {
        return 0;
    }
    (60_000 / n).max(1)
}

/// Whether another LLM HTTP call may start now under quota+gap for `n`.
pub(crate) fn can_request(n: u32, now_ms: u64, stamps: &[u64]) -> bool {
    throttle_wait_ms(n, now_ms, stamps) == 0
}

/// Ms to wait before the next LLM HTTP call. `0` = send now.
pub(crate) fn throttle_wait_ms(n: u32, now_ms: u64, stamps: &[u64]) -> u64 {
    if n == 0 {
        return 0;
    }
    let mut wait = 0u64;
    if let Some(&last) = stamps.last() {
        let interval = u64::from(interval_ms(n));
        let elapsed = now_ms.saturating_sub(last);
        if elapsed < interval {
            wait = interval - elapsed;
        }
    }
    let start = now_ms.saturating_sub(WINDOW_MS);
    let recent: Vec<u64> = stamps.iter().copied().filter(|&t| t > start).collect();
    if recent.len() >= n as usize {
        let oldest = recent[recent.len() - n as usize];
        let free_at = oldest.saturating_add(WINDOW_MS);
        if free_at > now_ms {
            wait = wait.max(free_at - now_ms);
        }
    }
    wait
}

pub(crate) async fn throttle(requests_per_minute: Option<u32>, stamps: &Mutex<Vec<u64>>) {
    let Some(n) = requests_per_minute else {
        return;
    };
    loop {
        let wait = {
            let mut guard = stamps.lock().unwrap_or_else(|e| e.into_inner());
            let now = now_ms();
            let wait = throttle_wait_ms(n, now, &guard);
            if wait == 0 {
                let start = now.saturating_sub(WINDOW_MS);
                guard.retain(|&t| t > start);
                guard.push(now);
                return;
            }
            wait
        };
        let ms = wait.min(u64::from(u32::MAX)) as u32;
        sleep_ms(ms.max(1)).await;
    }
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

    #[test]
    fn interval_for_rpm_2_is_at_least_30s() {
        assert!(interval_ms(2) >= 30_000);
        assert_eq!(interval_ms(2), 30_000);
        let wait = throttle_wait_ms(2, 0, &[0]);
        assert!(wait >= 30_000, "{wait}");
    }

    #[test]
    fn quota_rpm_1_blocks_second_in_window() {
        let stamps = [10_000u64];
        assert!(!can_request(1, 20_000, &stamps));
        assert!(can_request(1, 10_000 + WINDOW_MS, &stamps));
    }
}
