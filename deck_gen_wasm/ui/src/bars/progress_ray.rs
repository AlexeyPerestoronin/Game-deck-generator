//! Full-height 10px ray between the activity bar and the explorer.
//!
//! Idle is a solid green fill. While [`Workspace::loading`](deck_gen_wasm_workspace::Workspace)
//! is true, a blue beam grows from the vertical center toward both edges.

use leptos::prelude::*;

use deck_gen_wasm_workspace::Workspace;

/// Vertical progress strip bound to [`Workspace`] loading / progress signals.
#[component]
pub fn ProgressRay(workspace: Workspace) -> impl IntoView {
    view! {
        <div
            class="progress-ray"
            class:is-idle=move || !workspace.loading.get()
            style=move || {
                let p = workspace.progress.get().clamp(0.0, 100.0);
                format!("--ray-arm: {}%", p / 2.0)
            }
            aria-hidden="true"
        >
            <div class="progress-ray-arm progress-ray-arm-top"></div>
            <div class="progress-ray-seed"></div>
            <div class="progress-ray-arm progress-ray-arm-bottom"></div>
        </div>
    }
}
