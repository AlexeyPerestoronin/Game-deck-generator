//! Callback-only 0..=100 reporter.

use std::future::Future;
use std::pin::Pin;
use std::rc::Rc;

type PaintFn = Rc<dyn Fn() -> Pin<Box<dyn Future<Output = ()>>>>;

/// Reports a percentage through a caller-supplied callback.
///
/// The callback is stored as `Rc<dyn Fn(f32)>`. It must not spawn tasks;
/// a WASM UI typically writes a signal here. Optional [`Self::with_paint`]
/// yields to the event loop after each macro `set` so the ray can redraw.
#[derive(Clone)]
pub struct Progress {
    on_set: Rc<dyn Fn(f32)>,
    paint: Option<PaintFn>,
}

impl Progress {
    /// Wrap `on_set`. Values passed to [`Self::set`] are clamped to `0.0..=100.0`.
    pub fn new(on_set: impl Fn(f32) + 'static) -> Self {
        Self {
            on_set: Rc::new(on_set),
            paint: None,
        }
    }

    /// After every macro `set`, run `paint` (WASM: one 0ms timer so Leptos can draw).
    pub fn with_paint<F, Fut>(self, paint: F) -> Self
    where
        F: Fn() -> Fut + 'static,
        Fut: Future<Output = ()> + 'static,
    {
        Self {
            on_set: self.on_set,
            paint: Some(Rc::new(move || {
                Box::pin(paint()) as Pin<Box<dyn Future<Output = ()>>>
            })),
        }
    }

    /// Record `pct`, clamped to `0.0..=100.0`.
    pub fn set(&self, pct: f32) {
        (self.on_set)(pct.clamp(0.0, 100.0));
    }

    /// Yield so the UI can show the last [`Self::set`]. No-op without [`Self::with_paint`].
    pub async fn paint(&self) {
        if let Some(paint) = &self.paint {
            paint().await;
        }
    }

    /// Child reporter whose `0..=100` is mapped into this reporter's `[from, to]`.
    ///
    /// Inner macros still speak 0..=100; `set(0)` writes `from` on the parent,
    /// `set(100)` writes `to`. The paint hook is shared with the parent.
    pub fn new_subprocess(&self, from: f32, to: f32) -> Progress {
        let from = from.clamp(0.0, 100.0);
        let to = to.clamp(0.0, 100.0);
        let parent = self.clone();
        let child = Progress::new(move |pct| {
            parent.set(from + (to - from) * (pct / 100.0));
        });
        Progress {
            on_set: child.on_set,
            paint: self.paint.clone(),
        }
    }
}
