//! Loaders: glyph loader, status indicator, spinner.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{ClockMs, loader::FREE_GLYPH_TABLES};
use oxyn_dioxus::{GlyphLoader, Spinner};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{IndicatorSize, IndicatorVariant, StatusIndicator};

use super::{Code, rng, sel, tog, var};
use crate::{Align, Demo, Vals, data};

const GLYPHS: [&str; 14] = [
    "glyph-braille",
    "glyph-sand",
    "glyph-block",
    "glyph-bar",
    "glyph-dots",
    "glyph-line",
    "glyph-arc",
    "glyph-quadrant",
    "glyph-bloom",
    "glyph-half",
    "glyph-bullet",
    "glyph-scanner",
    "glyph-spark",
    "glyph-pulse",
];

fn glyph(v: &Vals, now: ClockMs) -> Element {
    let st = data::state(v.s("state"));
    let size = v.f("size") as f32;
    let all = v.b("all");
    rsx! {
        if all {
            div { class: "oxd-glyphs",
                for t in FREE_GLYPH_TABLES.iter() {
                    div { class: "oxd-glyph",
                        GlyphLoader { state: st.clone(), now, variant: t.id.to_string(), size, still: v.b("still") }
                        span { "{t.id}" }
                    }
                }
            }
        } else {
            div { class: "oxd-line",
                GlyphLoader { state: st.clone(), now, variant: v.string("variant"), size, still: v.b("still") }
                span { class: "oxd-status", "Running cargo test" }
            }
        }
    }
}

fn glyph_code(v: &Vals) -> String {
    Code::new("GlyphLoader")
        .raw("state", "agent.state()")
        .raw(
            "now",
            "clock.now() // one shared clock keeps every loader in phase",
        )
        .s("variant", v.s("variant"), "glyph-braille")
        .raw("size", format!("{}.0", v.s("size")))
        .b("still", v.b("still"))
        .done()
}

fn spinner(v: &Vals, _: ClockMs) -> Element {
    let label = v.b("label").then(|| "Loading runs".to_string());
    rsx! {
        div { class: "oxd-line",
            Spinner { size: v.f("size") as f32, style: v.string("style"), speed_ms: v.f("speed") as u32, arc: v.f("arc") as f32, label }
            if v.b("label") { span { class: "oxd-status", "Loading runs" } }
        }
    }
}

fn spinner_code(v: &Vals) -> String {
    let mut c = Code::new("Spinner");
    if v.s("size") != "14" {
        c = c.raw("size", format!("{}.0", v.s("size")));
    }
    c = c.s("style", v.s("style"), "arc");
    if v.s("speed") != "750" {
        c = c.raw("speed_ms", v.s("speed"));
    }
    if v.s("arc") != "0.25" && v.s("style") == "arc" {
        c = c.raw("arc", v.s("arc"));
    }
    c.opt("label", if v.b("label") { "Loading runs" } else { "" })
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![        Demo {
            slug: "glyph-loader",
            name: "Glyph loader",
            category: "loaders",
            summary: "Fourteen text-glyph loaders (braille, sand, blocks, scanner…) that settle into a check, a warning or a cross when the agent finishes, waits or fails. One table each, phase-locked to a shared clock.",
            dioxus: "GlyphLoader",
            gpui: "GlyphLoaderEl",
            spec: "loader-glyph",
            source: "glyph_loader.rs",
            controls: vec![
                sel("variant", "Variant", &GLYPHS, "glyph-braille"),
                sel("state", "Agent state", &data::BUSY, "thinking"),
                rng("size", "Size", 10.0, 32.0, 1.0, "18"),
                tog("still", "Still frame (reduced motion)", false),
                tog("all", "Show all 14", false),
            ],
            variants: vec![
                var("glyph-braille", "The default spinner.", &[]),
                var(
                    "glyph-scanner",
                    "A scanning bar for IDE status rows.",
                    &[("variant", "glyph-scanner")],
                ),
                var(
                    "glyph-sand",
                    "An hourglass of dots.",
                    &[("variant", "glyph-sand")],
                ),
                var(
                    "all-states",
                    "All 14 in the waiting state.",
                    &[("all", "true"), ("state", "awaiting-approval")],
                ),
            ],
            render: glyph,
            code: glyph_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "The glyphs are aria-hidden; pair it with status text (or a StatusIndicator) that names the state.",
                "still draws one frame for reduced motion and posters.",
            ],
            responsive: vec!["Text-sized: scales with font size and sits on the text baseline."],
            height: 200,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "spinner",
            name: "Spinner",
            category: "loaders",
            summary: "A small turning arc for work that runs right now, plus dots, ray and orbit styles; any size, speed and arc length, announced only when it has a label.",
            dioxus: "Spinner",
            gpui: "SpinnerEl",
            spec: "display-atoms",
            source: "primitives/display.rs",
            controls: vec![
                sel("style", "Style", &["arc", "dots", "ray", "orbit"], "arc"),
                rng("size", "Size", 10.0, 64.0, 2.0, "24"),
                rng("speed", "Turn (ms)", 300.0, 2000.0, 50.0, "750"),
                rng("arc", "Arc length", 0.1, 0.9, 0.05, "0.25"),
                tog("label", "Label", true),
            ],
            variants: vec![
                var("arc", "A quarter arc.", &[]),
                var("dots", "Three dots.", &[("style", "dots")]),
                var("ray", "Eight rays.", &[("style", "ray")]),
                var("orbit", "A dot in orbit.", &[("style", "orbit")]),
            ],
            render: spinner,
            code: spinner_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "Silent without a label; with one it is a role=status that reads the label.",
                "Slows to one turn per 2.4 s under prefers-reduced-motion.",
            ],
            responsive: vec!["Any size from 10 px (inside a button) up."],
            height: 160,
            align: Align::Center,
            live: false,
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 3] = ["glyph-loader", "status-indicator", "spinner"];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
