//! Percent reporting for long workspace actions, without Leptos or `deck_gen`.
//!
//! [`Progress`] only stores a callback. The three macros mark statement ranges
//! so a caller can drive a 0..=100 signal. WASM UI yield (`TimeoutFuture`) is
//! left to the caller, between blocks.

mod macros;
mod progress;

pub use progress::Progress;

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;
    use std::rc::Rc;

    fn recorder() -> (Progress, Rc<RefCell<Vec<f32>>>) {
        let log = Rc::new(RefCell::new(Vec::new()));
        let log2 = Rc::clone(&log);
        let progress = Progress::new(move |pct| log2.borrow_mut().push(pct));
        (progress, log)
    }

    #[test]
    fn wrapper_sets_0_then_100() {
        let (progress, log) = recorder();
        let value = progress_wrapper!(progress, { 7 });
        assert_eq!(value, 7);
        assert_eq!(*log.borrow(), vec![0.0, 100.0]);
    }

    #[test]
    fn block_sets_from_then_to_and_returns_body() {
        let (progress, log) = recorder();
        let value = progress_block!(progress, 10.0, 90.0, { 42 });
        assert_eq!(value, 42);
        assert_eq!(*log.borrow(), vec![10.0, 90.0]);
    }

    #[test]
    fn loop_spreads_range_evenly_then_sets_to() {
        let (progress, log) = recorder();
        let mut seen = Vec::new();
        progress_loop!(progress, 20.0, 40.0, vec![10, 20, 30, 40], |item| {
            seen.push(item);
        });
        assert_eq!(seen, vec![10, 20, 30, 40]);
        assert_eq!(*log.borrow(), vec![20.0, 25.0, 30.0, 35.0, 40.0]);
    }

    #[test]
    fn empty_loop_only_sets_to() {
        let (progress, log) = recorder();
        let empty: Vec<i32> = Vec::new();
        progress_loop!(progress, 20.0, 40.0, empty, |_item| {});
        assert_eq!(*log.borrow(), vec![40.0]);
    }

    #[test]
    fn set_clamps_to_unit_interval() {
        let (progress, log) = recorder();
        progress.set(-5.0);
        progress.set(150.0);
        assert_eq!(*log.borrow(), vec![0.0, 100.0]);
    }

    #[test]
    fn wrapper_around_blocks_bookends_inner_sets() {
        let (progress, log) = recorder();
        progress_wrapper!(progress, {
            progress_block!(progress, 0.0, 10.0, {});
            progress_block!(progress, 80.0, 100.0, {});
        });
        assert_eq!(*log.borrow(), vec![0.0, 0.0, 10.0, 80.0, 100.0, 100.0]);
    }
}
