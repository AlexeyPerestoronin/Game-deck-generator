//! Tiny executor for tests: futures that never wait (no paint hook).

use std::future::Future;
use std::task::{Context, Poll, RawWaker, RawWakerVTable, Waker};

fn dummy_waker() -> Waker {
    fn clone(_: *const ()) -> RawWaker {
        raw()
    }
    fn wake(_: *const ()) {}
    fn wake_by_ref(_: *const ()) {}
    fn drop(_: *const ()) {}
    fn raw() -> RawWaker {
        RawWaker::new(
            std::ptr::null(),
            &RawWakerVTable::new(clone, wake, wake_by_ref, drop),
        )
    }
    // Safety: vtable no-ops; the pointer is never dereferenced.
    unsafe { Waker::from_raw(raw()) }
}

/// Poll `fut` once. Panics if it waits (a paint hook that yields).
pub fn poll_now<F: Future>(fut: F) -> F::Output {
    let waker = dummy_waker();
    let mut cx = Context::from_waker(&waker);
    let mut fut = std::pin::pin!(fut);
    match fut.as_mut().poll(&mut cx) {
        Poll::Ready(v) => v,
        Poll::Pending => panic!("poll_now: future pending"),
    }
}
