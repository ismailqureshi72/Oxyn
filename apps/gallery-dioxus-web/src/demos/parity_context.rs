//! Context Visualizer family (variants 2) demos: what fills the context window, four ways – a
//! stacked bar or field of dots, a breakdown list, a stack of slabs and a tide glass.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{ContextBar, ContextBreakdown, ContextStack, ContextTide};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

const CAT: &str = "context-visualizer";

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

fn limit(v: &Vals) -> u32 {
    match v.s("limit") {
        "180K" => 180_000,
        "256K" => 256_000,
        "1M" => 1_000_000,
        _ => 200_000,
    }
}

// ---------- Context bar ----------

fn context_bar(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}{}",
        v.s("look"),
        v.s("mono"),
        v.s("limit"),
        v.s("compacted"),
        v.s("button")
    );
    let look = v.string("look");
    let lim = limit(v);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:320px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            ContextBar { look, mono: v.b("mono"), limit: lim, default_compacted: v.b("compacted"), compact: v.b("button"),
                on_compact: move |c: bool| msg.set(format!("onCompact({c})")) }
            {status(msg)}
        }
    }
}

fn context_bar_code(v: &Vals) -> String {
    Code::new("ContextBar")
        .raw("parts", "Some(parts())")
        .s("look", v.s("look"), "bar")
        .b("mono", v.b("mono"))
        .raw("limit", limit(v).to_string())
        .raw(
            "on_compact",
            "move |on| if on { compact() } else { undo() }",
        )
        .done()
}

fn context_bar_demo() -> Demo {
    Demo {
        controls: vec![
            sel("look", "Look", &["bar", "field"], "bar"),
            tog("mono", "Monochrome", false),
            sel("limit", "Window", &["200K", "180K", "256K", "1M"], "200K"),
            tog("button", "Compact button", true),
            tog("compacted", "Start compacted", false),
        ],
        variants: vec![
            var(
                "bar",
                "One stacked bar: point at a kind in the legend to light it and read its own count; Compact squeezes the chat.",
                &[],
            ),
            var(
                "field",
                "The same window as a field of 200 dots, one per half percent, in load order.",
                &[("look", "field")],
            ),
            var(
                "mono",
                "Monochrome: the kinds become steps of grey, for a quiet toolbar.",
                &[("mono", "true")],
            ),
            var(
                "big window",
                "A 1M window after a compaction: the same parts read as a thin sliver.",
                &[("limit", "1M"), ("compacted", "true")],
            ),
        ],
        render: context_bar,
        code: context_bar_code,
        keyboard: vec![
            ("Tab", "Compact / Undo, then each kind in the legend"),
            ("Focus on a kind", "Lights it and shows its count"),
            ("Enter / Space", "Compact or Undo"),
        ],
        a11y: vec![
            "The bar (or field) is an image whose label reads the used tokens, the limit and the percent.",
            "Each legend chip names its kind and token count; Compact is a toggle button (aria-pressed) and the result is said in a status line.",
        ],
        responsive: vec![
            "320 px card that shrinks with its container; the field keeps 20 columns and the legend wraps.",
        ],
        height: 300,
        live: false,
        align: Align::Center,
        ..mk(
            "context-bar",
            "Context bar",
            CAT,
            "The context window as one stacked bar or a field of dots, with a legend that lights each kind and a Compact / Undo button.",
            "ContextBar",
            "ContextBarEl",
            "context-bar",
            "kit/ctxvisfid.rs",
        )
    }
}

// ---------- Context breakdown ----------

fn context_breakdown(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("sort"),
        v.s("open"),
        v.s("drop"),
        v.s("mono")
    );
    let open = match v.s("open") {
        "chat" => Some(0),
        "files" => Some(1),
        "tools" => Some(2),
        _ => None,
    };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:360px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            ContextBreakdown { default_sort: v.string("sort"), default_open: open, droppable: v.b("drop"), mono: v.b("mono"),
                on_drop: move |(p, i): (usize, usize)| msg.set(format!("onDrop(part {p}, item {i})")) }
            {status(msg)}
        }
    }
}

fn context_breakdown_code(v: &Vals) -> String {
    Code::new("ContextBreakdown")
        .raw("parts", "Some(parts())")
        .s("default_sort", v.s("sort"), "size")
        .raw("on_drop", "move |(part, item)| unload(part, item)")
        .done()
}

fn context_breakdown_demo() -> Demo {
    Demo {
        controls: vec![
            sel("sort", "Order", &["size", "load"], "size"),
            sel("open", "Open", &["none", "chat", "files", "tools"], "files"),
            tog("drop", "Unload buttons", true),
            tog("mono", "Monochrome", false),
        ],
        variants: vec![
            var(
                "files open",
                "Files opened: each file with its tokens and an Unload button that gives the room back.",
                &[],
            ),
            var(
                "load order",
                "Kinds in the order they load – system first, chat last – instead of by size.",
                &[("sort", "load"), ("open", "none")],
            ),
            var(
                "read only",
                "No Unload buttons: a breakdown to read, monochrome.",
                &[("drop", "false"), ("open", "chat"), ("mono", "true")],
            ),
        ],
        render: context_breakdown,
        code: context_breakdown_code,
        keyboard: vec![
            ("Tab", "Order switch, each kind, then its Unload buttons"),
            ("Enter / Space on a kind", "Opens or closes it"),
        ],
        a11y: vec![
            "Each kind is a button with aria-expanded; its items are a list under it.",
            "The fill pill is text (83%) and turns amber when high and red at the compaction line – the colour is extra.",
            "Unloading says what was freed in a status line.",
        ],
        responsive: vec!["360 px card; the share bar flexes and names never wrap."],
        height: 440,
        live: false,
        align: Align::Center,
        ..mk(
            "context-breakdown",
            "Context breakdown",
            CAT,
            "What fills the window as a list of kinds with their share; open one to see its files, tools or turns and unload what you do not need.",
            "ContextBreakdown",
            "ContextBreakdownEl",
            "context-breakdown",
            "kit/ctxvisfid.rs",
        )
    }
}

// ---------- Context stack ----------

fn context_stack(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("auto"), v.s("turn"), v.s("mono"));
    let auto = v.f("auto") as u32;
    let per = v.f("turn") as u32 * 1_000;
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:340px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            ContextStack { auto, per_turn: per, mono: v.b("mono"),
                on_compact: move |a: bool| msg.set(format!("onCompact(auto: {a})")) }
            {status(msg)}
        }
    }
}

fn context_stack_code(v: &Vals) -> String {
    Code::new("ContextStack")
        .raw("parts", "Some(parts())")
        .s("auto", v.s("auto"), "92")
        .raw("on_compact", "move |auto| log(auto)")
        .done()
}

fn context_stack_demo() -> Demo {
    Demo {
        controls: vec![
            rng("auto", "Compacts at %", 80.0, 98.0, 1.0, "92"),
            rng("turn", "Turn size (K)", 2.0, 20.0, 1.0, "6"),
            tog("mono", "Monochrome", false),
        ],
        variants: vec![
            var(
                "default",
                "Slabs from system (bottom) to chat (top); add turns until the chat crosses the dashed line and is pressed into a summary.",
                &[],
            ),
            var(
                "early line",
                "The line at 85 %: one more turn compacts it.",
                &[("auto", "85")],
            ),
            var(
                "mono",
                "Monochrome slabs with big turns.",
                &[("mono", "true"), ("turn", "12")],
            ),
        ],
        render: context_stack,
        code: context_stack_code,
        keyboard: vec![("Tab", "Add a turn, Compact, Reset")],
        a11y: vec![
            "The column is an image labelled with the used tokens and percent; the legend repeats every kind as text.",
            "Each action says the new state in a status line, including an automatic compaction.",
        ],
        responsive: vec!["340 px card; the legend takes the width the column leaves."],
        height: 400,
        live: false,
        align: Align::Center,
        ..mk(
            "context-stack",
            "Context stack",
            CAT,
            "An Oxyn original: the window as a pile of slabs under the line where it compacts by itself; each turn thickens the chat until it is pressed into a summary.",
            "ContextStack",
            "ContextStackEl",
            "context-stack",
            "kit/ctxvisfid.rs",
        )
    }
}

// ---------- Context tide ----------

fn context_tide(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("auto"), v.s("tint"), v.s("turn"));
    let auto = v.f("auto") as u32;
    let per = v.f("turn") as u32 * 1_000;
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:340px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            ContextTide { auto, per_turn: per, tint: v.b("tint"),
                on_change: move |u: u32| msg.set(format!("onChange({u})")) }
            {status(msg)}
        }
    }
}

fn context_tide_code(v: &Vals) -> String {
    Code::new("ContextTide")
        .raw("parts", "Some(parts())")
        .s("auto", v.s("auto"), "92")
        .b("tint", v.b("tint"))
        .done()
}

fn context_tide_demo() -> Demo {
    Demo {
        controls: vec![
            rng("auto", "Compacts at %", 80.0, 98.0, 1.0, "92"),
            rng("turn", "Turn size (K)", 2.0, 20.0, 1.0, "6"),
            tog("tint", "Tint as it fills", true),
        ],
        variants: vec![
            var(
                "default",
                "Water at 83 %, amber above the 75 mark; send turns to raise it, and at the red line it drains on its own.",
                &[],
            ),
            var(
                "quiet",
                "Grey water all the way up: no amber or red.",
                &[("tint", "false")],
            ),
            var(
                "big turns",
                "Twelve-thousand-token turns: the next one compacts.",
                &[("turn", "12")],
            ),
        ],
        render: context_tide,
        code: context_tide_code,
        keyboard: vec![("Tab", "Send a turn, Compact, Reset")],
        a11y: vec![
            "The glass is a meter (aria-valuenow is the percent, the value text the tokens).",
            "Turns left before compaction are written next to it; changes are said in a status line.",
        ],
        responsive: vec!["340 px card; glass and text side by side down to 300 px."],
        height: 300,
        live: false,
        align: Align::Center,
        ..mk(
            "context-tide",
            "Context tide",
            CAT,
            "An Oxyn original: a tall glass whose water is the used share, with marks at 75 % and at the compaction line; turns raise it and Compact drains it.",
            "ContextTide",
            "ContextTideEl",
            "context-tide",
            "kit/ctxvisfid.rs",
        )
    }
}

/// Every Context Visualizer (variants 2) demo, in sidebar order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        context_bar_demo(),
        context_breakdown_demo(),
        context_stack_demo(),
        context_tide_demo(),
    ]
}
