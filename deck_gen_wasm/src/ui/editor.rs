//! Open-file pane: tab strip plus source editor or HTML/Markdown preview.
//!
//! Tabs come from [`Workspace::tabs`]. Markdown / JSON / HTML / SCSS use
//! syntect HTML behind a transparent textarea; everything else is a plain
//! `<textarea>`. Preview tabs render HTML in an iframe and Markdown as HTML.

use leptos::html;
use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::HtmlTextAreaElement;

use super::highlight::{can_highlight, highlight_html};
use crate::fs::{file_ext, file_name, join_path, parent_path, Vfs};
use crate::workspace::{OpenTab, TabKind, Workspace};

/// Center pane: tab strip plus the active editor or preview.
#[component]
pub fn Editor(workspace: Workspace) -> impl IntoView {
    view! {
        <main class="editor">
            <header class="editor-tabs" role="tablist">
                <For
                    each=move || workspace.tabs.get()
                    key=|tab| {
                        let kind = match tab.kind {
                            TabKind::Edit => "edit",
                            TabKind::Preview => "preview",
                        };
                        format!("{kind}:{}", tab.path)
                    }
                    children=move |tab| {
                        view! { <EditorTab workspace=workspace tab=tab /> }
                    }
                />
            </header>
            {move || match workspace.active_tab.get() {
                None => view! {
                    <div class="editor-empty">"Select a file to edit, or create one in the explorer."</div>
                }
                .into_any(),
                Some(tab) if tab.kind == TabKind::Preview => view! {
                    <PreviewPane workspace=workspace path=tab.path />
                }
                .into_any(),
                Some(tab) if can_highlight(&tab.path) => view! {
                    <HighlightedEditor workspace=workspace path=tab.path />
                }
                .into_any(),
                Some(tab) => view! {
                    <PlainEditor workspace=workspace path=tab.path />
                }
                .into_any(),
            }}
        </main>
    }
}

#[component]
fn EditorTab(workspace: Workspace, tab: OpenTab) -> impl IntoView {
    let tab_for_active = tab.clone();
    let tab_for_click = tab.clone();
    let tab_for_close = tab.clone();
    let label = match tab.kind {
        TabKind::Edit => file_name(&tab.path).to_string(),
        TabKind::Preview => format!("Preview {}", file_name(&tab.path)),
    };
    view! {
        <div
            class="editor-tab"
            class:active=move || workspace.active_tab.get().as_ref() == Some(&tab_for_active)
            role="tab"
            title=tab.path.clone()
            on:click=move |_| workspace.activate_tab(tab_for_click.clone())
        >
            <span class="editor-tab-label">{label}</span>
            <button
                class="editor-tab-close"
                type="button"
                title="Close"
                on:click=move |ev| {
                    ev.stop_propagation();
                    workspace.close_tab(tab_for_close.clone());
                }
            >
                "×"
            </button>
        </div>
    }
}

#[component]
fn PreviewPane(workspace: Workspace, path: String) -> impl IntoView {
    let ext = file_ext(&path).unwrap_or("").to_ascii_lowercase();
    match ext.as_str() {
        "html" | "htm" => view! { <HtmlPreview workspace=workspace path=path /> }.into_any(),
        _ => view! { <MarkdownPreview workspace=workspace path=path /> }.into_any(),
    }
}

#[component]
fn HtmlPreview(workspace: Workspace, path: String) -> impl IntoView {
    view! {
        <iframe
            class="preview-frame"
            prop:srcdoc=move || {
                let vfs = workspace.vfs.get();
                let Some(html) = vfs.read_file(&path) else {
                    return String::new();
                };
                inline_relative_iframes(&vfs, &path, html)
            }
        />
    }
}

#[component]
fn MarkdownPreview(workspace: Workspace, path: String) -> impl IntoView {
    view! {
        <div
            class="preview-md"
            inner_html=move || {
                let vfs = workspace.vfs.get();
                markdown_to_html(vfs.read_file(&path).unwrap_or(""))
            }
        ></div>
    }
}

fn markdown_to_html(src: &str) -> String {
    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_STRIKETHROUGH);
    let parser = pulldown_cmark::Parser::new_ext(src, options);
    let mut html = String::new();
    pulldown_cmark::html::push_html(&mut html, parser);
    html
}

#[component]
fn PlainEditor(workspace: Workspace, path: String) -> impl IntoView {
    let path_for_value = path.clone();
    let path_for_input = path;
    view! {
        <textarea
            class="editor-area"
            prop:value=move || {
                workspace
                    .vfs
                    .get()
                    .read_file(&path_for_value)
                    .map(str::to_string)
                    .unwrap_or_default()
            }
            on:input=move |ev| {
                let value = event_target_value(&ev);
                workspace.vfs.update(|vfs| {
                    let _ = vfs.write_file(&path_for_input, value);
                });
            }
        />
    }
}

#[component]
fn HighlightedEditor(workspace: Workspace, path: String) -> impl IntoView {
    let pre_ref = NodeRef::<html::Pre>::new();
    let path_for_value = path.clone();
    let path_for_html = path.clone();
    let path_for_input = path;

    view! {
        <div class="code-editor">
            <pre
                node_ref=pre_ref
                class="code-highlight"
                inner_html=move || {
                    let vfs = workspace.vfs.get();
                    let text = vfs.read_file(&path_for_html).unwrap_or("");
                    highlight_html(&path_for_html, text).unwrap_or_else(|| html_escape(text))
                }
            ></pre>
            <textarea
                class="editor-area code-input"
                spellcheck="false"
                prop:value=move || {
                    workspace
                        .vfs
                        .get()
                        .read_file(&path_for_value)
                        .map(str::to_string)
                        .unwrap_or_default()
                }
                on:input=move |ev| {
                    let value = event_target_value(&ev);
                    workspace.vfs.update(|vfs| {
                        let _ = vfs.write_file(&path_for_input, value);
                    });
                }
                on:scroll=move |ev| {
                    let Ok(area) = ev.target().unwrap().dyn_into::<HtmlTextAreaElement>() else {
                        return;
                    };
                    if let Some(pre) = pre_ref.get() {
                        pre.set_scroll_top(area.scroll_top());
                        pre.set_scroll_left(area.scroll_left());
                    }
                }
            />
        </div>
    }
}

fn html_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Nested `<iframe src="face.html">` inside `srcdoc` resolves against the IDE
/// page, so the app chrome is painted into the card strip. Inline VFS siblings.
fn inline_relative_iframes(vfs: &Vfs, html_path: &str, html: &str) -> String {
    let lower = html.to_ascii_lowercase();
    let mut out = String::with_capacity(html.len());
    let mut i = 0;
    while let Some(rel) = lower[i..].find("<iframe") {
        let start = i + rel;
        out.push_str(&html[i..start]);
        let Some(gt) = html[start..].find('>') else {
            out.push_str(&html[start..]);
            return out;
        };
        let end = start + gt;
        out.push_str(&rewrite_iframe_tag(vfs, html_path, &html[start..end]));
        out.push('>');
        i = end + 1;
    }
    out.push_str(&html[i..]);
    out
}

fn rewrite_iframe_tag(vfs: &Vfs, html_path: &str, tag: &str) -> String {
    if attr_value(tag, "srcdoc").is_some() {
        return tag.to_string();
    }
    let Some((quote, url, span)) = attr_span(tag, "src") else {
        return tag.to_string();
    };
    let Some(resolved) = resolve_relative(html_path, url) else {
        return tag.to_string();
    };
    let Some(content) = vfs.read_file(&resolved) else {
        return tag.to_string();
    };
    let mut out = String::with_capacity(tag.len() + content.len());
    out.push_str(&tag[..span.start]);
    out.push_str("srcdoc=");
    out.push(quote);
    out.push_str(&escape_srcdoc(content));
    out.push(quote);
    out.push_str(&tag[span.end..]);
    out
}

fn attr_value<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    attr_span(tag, name).map(|(_, value, _)| value)
}

fn attr_span<'a>(tag: &'a str, name: &str) -> Option<(char, &'a str, std::ops::Range<usize>)> {
    let lower = tag.to_ascii_lowercase();
    let needle = format!("{name}=");
    let mut from = 0;
    while let Some(rel) = lower[from..].find(&needle) {
        let start = from + rel;
        if start > 0 {
            let prev = tag.as_bytes()[start - 1];
            if !prev.is_ascii_whitespace() {
                from = start + 1;
                continue;
            }
        }
        let val_at = start + needle.len();
        let bytes = tag.as_bytes();
        let quote = *bytes.get(val_at)? as char;
        if quote != '"' && quote != '\'' {
            from = start + 1;
            continue;
        }
        let rest = &tag[val_at + 1..];
        let close = rest.find(quote)?;
        let end = val_at + 1 + close + 1;
        return Some((quote, &rest[..close], start..end));
    }
    None
}

fn resolve_relative(from_file: &str, href: &str) -> Option<String> {
    let href = href.trim();
    if href.is_empty() || is_external_src(href) {
        return None;
    }
    let href = href.strip_prefix("./").unwrap_or(href);
    if href.starts_with('/') || href.split('/').any(|part| part.is_empty() || part == "." || part == "..")
    {
        return None;
    }
    Some(join_path(&parent_path(from_file), href))
}

fn is_external_src(src: &str) -> bool {
    src.starts_with("//")
        || src.starts_with('#')
        || src.contains("://")
        || src.starts_with("data:")
        || src.starts_with("blob:")
        || src.starts_with("about:")
        || src.starts_with("javascript:")
}

fn escape_srcdoc(html: &str) -> String {
    html.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inlines_sibling_iframe_src() {
        let mut vfs = Vfs::default();
        vfs.put_file("d/face.html", r#"<p class="c">card</p>"#.into())
            .unwrap();
        let preview = r#"<iframe class="faces" src="face.html" title="Face"></iframe>"#;
        let out = inline_relative_iframes(&vfs, "d/preview.html", preview);
        assert!(out.contains(r#"srcdoc="&lt;p class=&quot;c&quot;>card&lt;/p>""#));
        assert!(!out.contains(r#"src="face.html""#));
    }

    #[test]
    fn leaves_external_iframe_src() {
        let vfs = Vfs::default();
        let preview = r#"<iframe src="https://example.com"></iframe>"#;
        let out = inline_relative_iframes(&vfs, "d/preview.html", preview);
        assert_eq!(out, preview);
    }
}
