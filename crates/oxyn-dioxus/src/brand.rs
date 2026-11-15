//! Brand and model marks (`oxyn_core::brand`): Lobe Icons (MIT) for AI models and providers,
//! Simple Icons (CC0) for companies and integrations. Monochrome (`currentColor`) by default per
//! the colour rule; `coloured` fills the brand colour (near-black marks keep the text colour so
//! they read on the dark theme).

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
    /// Brand colour instead of `currentColor`.
    #[props(default)]
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
    let fill = match (coloured, b.display_hex()) {
        (true, Some(h)) => format!("#{h}"),
        _ => "currentColor".to_string(),
    };
    rsx! {
        svg {
            class: "oxyn-brand",
            width: "{size}",
            height: "{size}",
            view_box: "{b.view_box}",
            fill: "{fill}",
            fill_rule: "{b.fill_rule}",
            clip_rule: "{b.fill_rule}",
            "aria-hidden": "true",
            "data-brand": "{b.id}",
            dangerous_inner_html: "{b.body}",
        }
    }
}

/// True when `name` resolves to a brand mark.
pub fn has_brand(name: &str) -> bool {
    brand(name).is_some()
}
