//! Declarative range markers around statements that report through [`Progress`](crate::Progress).
//!
//! Bodies run as written, including `.await`. After each `set` the macros
//! `paint().await` so a WASM UI can redraw. Must sit in an async context.

/// Run `$body`, reporting `0` before it and `100` after it.
#[macro_export]
macro_rules! progress_wrapper {
    ($progress:expr, $body:block) => {{
        $progress.set(0.0);
        $progress.paint().await;
        let __progress_wrapper_result = $body;
        $progress.set(100.0);
        $progress.paint().await;
        __progress_wrapper_result
    }};
}

/// Run `$body`, reporting `$from` before it and `$to` after it.
#[macro_export]
macro_rules! progress_block {
    ($progress:expr, $from:expr, $to:expr, $body:block) => {{
        $progress.set($from);
        $progress.paint().await;
        let __progress_block_result = $body;
        $progress.set($to);
        $progress.paint().await;
        __progress_block_result
    }};
}

/// Walk `$iter`, reporting `from + (to-from)*i/n` before item `i`, then `$to`.
#[macro_export]
macro_rules! progress_loop {
    ($progress:expr, $from:expr, $to:expr, $iter:expr, |$item:pat_param| $body:block) => {{
        let __items: Vec<_> = ::std::iter::IntoIterator::into_iter($iter).collect();
        let __n = __items.len();
        let __from: f32 = $from;
        let __span: f32 = $to - __from;
        for (__i, $item) in __items.into_iter().enumerate() {
            $progress.set(__from + __span * (__i as f32) / (__n as f32));
            $progress.paint().await;
            $body
        }
        $progress.set($to);
        $progress.paint().await;
    }};
}
