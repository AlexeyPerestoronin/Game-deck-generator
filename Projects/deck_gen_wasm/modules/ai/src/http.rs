//! POST JSON via the browser crate (WASM) or a native stub.

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

pub(crate) async fn throttle(requests_per_second: Option<u32>, not_first: bool) {
    if !not_first {
        return;
    }
    let Some(rps) = requests_per_second else {
        return;
    };
    let ms = (1000u32 / rps.max(1)).max(1);
    #[cfg(target_arch = "wasm32")]
    {
        gloo_timers::future::TimeoutFuture::new(ms).await;
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::thread::sleep(std::time::Duration::from_millis(ms as u64));
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
}
