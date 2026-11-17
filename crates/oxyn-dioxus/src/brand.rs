//! Brand and model marks (`oxyn_core::brand`): Lobe Icons (MIT) for AI models and providers,
//! Simple Icons (CC0) for companies and integrations. Official colours by default: marks with more
//! than one colour (Gmail, Slack, Google Drive, Claude, Gemini, Figma, Meta, Codex…) draw their
//! full-colour version, the rest their brand colour. Brands whose guidelines ask for black or white
//! (OpenAI, xAI, Vercel, Cursor…) draw in the theme's ink (`--ox-brand-ink`: black on the light theme,
//! white on the dark one), never grey. `coloured: false` or `--ox-icon-tone: mono` on a host opts out.

use dioxus::prelude::*;
use oxyn_core::brand::brand;

/// A brand or model mark resolved from an id, title, alias or model name ("claude-sonnet-4",
/// "GitHub"). Renders nothing when the name is unknown unless `fallback` is set, which draws the
/// first letter instead.
#[component]
pub fn BrandIcon(
    /// Id, title, alias or model name.
    name: String,
    /// Size px.
    #[props(default = 16.0)]
    size: f32,
    /// Official colours (the default). `false` draws the mark in `currentColor`.
    #[props(default = true)]
    coloured: bool,
    /// Draw the initial when no mark matches.
    #[props(default)]
    fallback: bool,
) -> Element {
    let Some(b) = brand(&name) else {
        if !fallback {
            return rsx! {};
        }
        let i = name
            .chars()
            .find(|c| c.is_alphanumeric())
            .map(|c| c.to_ascii_uppercase())
            .unwrap_or('?');
        return rsx! { span { class: "oxyn-brand oxyn-brand-ini", "aria-hidden": "true", "{i}" } };
    };
    // Multi-colour marks (Gmail, Slack, Google, Claude…) carry their own fills.
    if coloured && let Some(body) = b.coloured_body() {
        return rsx! {
            svg {
                class: "oxyn-brand",
                width: "{size}",
                height: "{size}",
                view_box: "{b.view_box}",
                "aria-hidden": "true",
                "data-brand": "{b.id}",
                "data-coloured": "true",
                dangerous_inner_html: "{body}",
            }
        };
    }
    let ink = coloured && b.ink();
    // The attribute is the plain colour; the style lets a host retone (`--ox-icon-tone: mono`).
    let (fill, style) = match (coloured, b.display_hex()) {
        (true, Some(h)) => (format!("#{h}"), format!("fill:var(--ox-brand-tone,#{h})")),
        _ if ink => (
            "currentColor".to_string(),
            "fill:var(--ox-brand-ink,currentColor)".to_string(),
        ),
        _ => ("currentColor".to_string(), String::new()),
    };
    rsx! {
        svg {
            class: "oxyn-brand",
            "data-ink": if ink { "true" },
            width: "{size}",
            height: "{size}",
            view_box: "{b.view_box}",
            fill: "{fill}",
            style: "{style}",
            fill_rule: "{b.fill_rule}",
            clip_rule: "{b.fill_rule}",
            "aria-hidden": "true",
            "data-brand": "{b.id}",
            dangerous_inner_html: "{b.body}",
        }
    }
}

/// The official mark in front of a model or brand name shown as text ("Claude Sonnet 4.5",
/// "Linear"): nothing when the name has no mark, so a label never gains a stray monogram.
#[component]
pub fn MarkFor(
    /// Model, product or brand name.
    name: String,
    /// Size px.
    #[props(default = 12.0)]
    size: f32,
) -> Element {
    if !has_brand(&name) {
        return rsx! {};
    }
    rsx! {
        span { class: "oxyn-mk", BrandIcon { name, size } }
    }
}

/// True when `name` resolves to a brand mark.
pub fn has_brand(name: &str) -> bool {
    brand(name).is_some()
}
