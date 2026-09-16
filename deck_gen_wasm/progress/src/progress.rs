//! Callback-only 0..=100 reporter.

use std::rc::Rc;

/// Reports a percentage through a caller-supplied callback.
///
/// The callback is stored as `Rc<dyn Fn(f32)>`. It must not spawn tasks;
/// a WASM UI typically writes a signal here and yields between macro blocks.
pub struct Progress {
    on_set: Rc<dyn Fn(f32)>,
}

impl Progress {
    /// Wrap `on_set`. Values passed to [`Self::set`] are clamped to `0.0..=100.0`.
    pub fn new(on_set: impl Fn(f32) + 'static) -> Self {
        Self {
            on_set: Rc::new(on_set),
        }
    }

    /// Record `pct`, clamped to `0.0..=100.0`.
    pub fn set(&self, pct: f32) {
        (self.on_set)(pct.clamp(0.0, 100.0));
    }
}
