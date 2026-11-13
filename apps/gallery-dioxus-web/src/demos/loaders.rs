//! Loaders: glyph loader, status indicator, spinner.

use dioxus::prelude::*;
use oxyn_core::{ClockMs, loader::FREE_GLYPH_TABLES};
use oxyn_dioxus::{GlyphLoader, IndicatorSize, IndicatorVariant, Spinner, StatusIndicator};

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

fn indicator(v: &Vals, now: ClockMs) -> Element {
    let variant = match v.s("variant") {
        "inline" => IndicatorVariant::Inline,
        "dot" => IndicatorVariant::Dot,
        _ => IndicatorVariant::Pill,
    };
    let size = if v.s("size") == "small" {
        IndicatorSize::Small
    } else {
        IndicatorSize::Medium
    };
    let all = v.b("all");
    rsx! {
        if all {
            div { class: "oxd-states",
                for s in data::STATES {
                    div { class: "oxd-state",
                        span { class: "oxd-sl", "{s}" }
                        StatusIndicator { state: data::state(s), now, pack: v.string("pack"), variant, size }
                    }
                }
            }
        } else {
            StatusIndicator { state: data::state(v.s("state")), now, pack: v.string("pack"), variant, size }
        }
    }
}

fn indicator_code(v: &Vals) -> String {
    let variant = match v.s("variant") {
        "inline" => "IndicatorVariant::Inline",
        "dot" => "IndicatorVariant::Dot",
        _ => "IndicatorVariant::Pill",
    };
    let mut c = Code::new("StatusIndicator")
        .pre(format!("let state = {};", data::state_code(v.s("state"))))
        .raw("state", "state")
        .raw("now", "clock.now()")
        .s("pack", v.s("pack"), "neutral");
    if variant != "IndicatorVariant::Pill" {
        c = c.raw("variant", variant);
    }
    if v.s("size") == "small" {
        c = c.raw("size", "IndicatorSize::Small");
    }
    c.done()
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
    vec![
        Demo {
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
            slug: "status-indicator",
            name: "Status indicator",
            category: "loaders",
            summary: "A dot and a label for any of the 13 agent states, as a pill, inline or a dot alone; the verb comes from a verb pack and stalls show how long.",
            dioxus: "StatusIndicator",
            gpui: "StatusIndicatorEl",
            spec: "status-indicator",
            source: "status_indicator.rs",
            controls: vec![
                sel("state", "Agent state", &data::STATES, "tool-running"),
                sel("variant", "Variant", &["pill", "inline", "dot"], "pill"),
                sel("size", "Size", &["medium", "small"], "medium"),
                sel(
                    "pack",
                    "Verb pack",
                    &["neutral", "coder", "terse", "crm"],
                    "neutral",
                ),
                tog("all", "Show all 13 states", false),
            ],
            variants: vec![
                var("pill", "Dot, label and a hairline pill.", &[]),
                var(
                    "inline",
                    "No surface.",
                    &[("variant", "inline"), ("state", "thinking")],
                ),
                var(
                    "dot",
                    "Dot alone; the label is the accessible name.",
                    &[("variant", "dot"), ("state", "awaiting-approval")],
                ),
                var("all-states", "Every state.", &[("all", "true")]),
            ],
            render: indicator,
            code: indicator_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "role=status with aria-live=polite, so state changes are announced.",
                "The dot variant keeps the label as its accessible name.",
                "The pulse is removed under prefers-reduced-motion.",
            ],
            responsive: vec![
                "small (6 px dot, 12 px text) for dense rows; medium (8 px, 13 px) elsewhere.",
            ],
            height: 160,
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
    ]
}
