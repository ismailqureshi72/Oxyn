//! `GlyphLoader`: the T1 glyph loaders (`docs/specs/loader-glyph.md`), same engine as GPUI.

use std::{cell::RefCell, rc::Rc};

use dioxus::prelude::*;
use oxyn_core::{
    AgentState, ClockMs,
    loader::{GlyphLoader as Engine, GlyphTable},
};

use crate::color_var;

/// Stylesheet for glyph loaders.
pub const GLYPH_LOADER_CSS: &str = r#"
.oxyn-gl{display:inline-flex;font-family:var(--oxyn-typography-code-font-family,monospace);
  font-variant-numeric:tabular-nums;line-height:1.35;white-space:pre;vertical-align:middle}
.oxyn-gl>span{display:inline-block;width:1ch;text-align:center}
"#;

/// A glyph loader driven by an [`AgentState`]. The host re-renders it with the shared clock's
/// `now`; frames come from the shared engine, so every instance of a variant is in phase.
#[component]
pub fn GlyphLoader(
    /// The agent state.
    state: AgentState,
    /// Shared clock time.
    now: ClockMs,
    /// Variant id (`glyph-braille`, `loader/glyph-sand`, …). Unknown ids fall back to braille.
    #[props(default = "glyph-braille".to_string())]
    variant: String,
    /// Draw still frames (reduced motion, mirrored loader, poster).
    #[props(default)]
    still: bool,
    /// Font size in px.
    #[props(default = 13.0)]
    size: f32,
) -> Element {
    let table = GlyphTable::by_id(&variant).unwrap_or(&oxyn_core::loader::FREE_GLYPH_TABLES[0]);
    let engine = use_hook(|| Rc::new(RefCell::new(Engine::new(table))));
    if engine.borrow().table().id != table.id {
        *engine.borrow_mut() = Engine::new(table);
    }
    engine.borrow_mut().update(&state, now);
    let frame = engine.borrow().frame(now, still);
    let cells = table.cells();
    let Some(f) = frame else {
        // Reserve the box so the line doesn't move when the loader appears.
        return rsx! {
            span { class: "oxyn-gl", "aria-hidden": "true", style: "font-size:{size}px;width:{cells}ch" }
        };
    };
    let to = color_var(f.color);
    let colour = if f.color_t >= 1.0 {
        to
    } else {
        let from = color_var(f.color_from);
        format!(
            "color-mix(in oklab, {from}, {to} {:.0}%)",
            f.color_t * 100.0
        )
    };
    let opacity = f.opacity;
    rsx! {
        span {
            class: "oxyn-gl",
            "aria-hidden": "true",
            "data-variant": table.id,
            style: "font-size:{size}px;color:{colour};opacity:{opacity:.3}",
            for (ch, a) in f.text.chars().zip(f.cell_alpha.into_iter()) {
                span { style: "opacity:{a:.2}", "{ch}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn html(state: AgentState, now: u64, variant: &str, still: bool) -> String {
        let variant = variant.to_string();
        dioxus_ssr::render_element(
            rsx! { GlyphLoader { state, now: ClockMs(now), variant, still } },
        )
    }

    #[test]
    fn hidden_reserves_its_width() {
        let h = html(AgentState::Idle, 0, "glyph-scanner", false);
        assert!(h.contains("width:8ch"), "{h}");
    }

    #[test]
    fn submitted_draws_the_first_frame_in_accent() {
        let h = html(AgentState::Submitted, 0, "glyph-braille", false);
        assert!(
            h.contains("⠋") && h.contains("var(--oxyn-color-loader-accent)"),
            "{h}"
        );
    }

    #[test]
    fn failed_is_a_still_cross_in_danger() {
        let h = html(
            AgentState::Error {
                kind: oxyn_core::ErrorKind::Offline,
            },
            0,
            "glyph-bar",
            true,
        );
        assert!(h.contains("✕") && h.contains("loader-danger"), "{h}");
    }
}
