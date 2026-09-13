//! Bounded concurrent join for WASM async I/O.
//!
//! The browser is one CPU thread; overlapping `fetch` and `File` reads still
//! helps because those wait on the network and disk. Chunk size comes from
//! [`crate::conf::io`].

use std::future::Future;

use futures::future::join_all;

/// Map `items` through `f` and join at most `limit` futures at a time.
///
/// Output order matches input order. `limit == 0` is treated as 1.
pub async fn map_join<T, F, Fut, R>(items: impl IntoIterator<Item = T>, limit: usize, f: F) -> Vec<R>
where
    F: Fn(T) -> Fut,
    Fut: Future<Output = R>,
{
    let limit = limit.max(1);
    let items: Vec<T> = items.into_iter().collect();
    let mut out = Vec::with_capacity(items.len());
    let mut iter = items.into_iter();
    loop {
        let mut batch = Vec::with_capacity(limit);
        for _ in 0..limit {
            match iter.next() {
                Some(item) => batch.push(f(item)),
                None => break,
            }
        }
        if batch.is_empty() {
            break;
        }
        out.extend(join_all(batch).await);
    }
    out
}
