//! `SubagentTree` (`subagent-tree.md`): indented, rail, dock, inline-wrapper and terminal.

use std::collections::BTreeSet;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs, VerbPack,
    run::{NodeState, RunGraph, TreeRow, TreeVariant},
};

use super::tool_call::tone;
use crate::composer::icons::Icon;

const AGENT: [&str; 8] = [
    "red", "orange", "yellow", "green", "blue", "violet", "pink", "cyan",
];
const BRAILLE: [&str; 10] = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];

fn dot(s: &NodeState, animate: bool) -> Element {
    match s {
        NodeState::Working(_)
        | NodeState::ToolRunning(_)
        | NodeState::Starting
        | NodeState::Orphaned(_) => {
            rsx! { span { class: if animate { "nd breathe" } else { "nd" } } }
        }
        NodeState::WaitingOnChildren(_) | NodeState::Stalled => rsx! { span { class: "nd" } },
        NodeState::AwaitingYou(_) => rsx! { Icon { name: "hand", size: 13.0 } },
        NodeState::Done(_) => rsx! { Icon { name: "check", size: 13.0, stroke: 2.0 } },
        NodeState::Failed(_) => rsx! { Icon { name: "x", size: 13.0, stroke: 2.0 } },
        NodeState::Interrupted(_) => rsx! { Icon { name: "stop", size: 10.0 } },
        s => rsx! { span { class: "g", "{s.glyph()}" } },
    }
}

/// The run's agents as a tree.
#[component]
pub fn SubagentTree(
    /// The graph.
    graph: RunGraph,
    /// Shared clock.
    now: ClockMs,
    /// `indented`, `rail`, `dock`, `inline-wrapper` or `terminal`.
    #[props(default = "indented".to_string())]
    variant: String,
    /// Levels shown before folding (3).
    #[props(default = 3)]
    display_depth: usize,
    /// Siblings shown before "+ n more" (8).
    #[props(default = 8)]
    max_children: usize,
    /// Initially collapsed node ids.
    #[props(default)]
    collapsed: Vec<String>,
    /// Row pressed: focus that agent.
    #[props(default)]
    on_focus: EventHandler<String>,
    /// Review pressed on an agent that needs you.
    #[props(default)]
    on_review: EventHandler<String>,
) -> Element {
    let v = TreeVariant::ALL
        .into_iter()
        .find(|x| x.id() == variant)
        .unwrap_or_default();
    let mut folded = use_signal(|| collapsed.iter().cloned().collect::<BTreeSet<String>>());
    let mut wrapper_open = use_signal(|| false);
    let view = graph.view(
        v,
        &folded(),
        display_depth,
        max_children,
        VerbPack::neutral(),
        now,
    );
    let frame = (now.0 / 80) as usize % BRAILLE.len();
    let row = move |r: TreeRow| {
        let id = r.id.clone();
        let id2 = r.id.clone();
        let id3 = r.id.clone();
        let has_kids = r.hidden_children > 0
            || r.expanded
                && graph
                    .nodes
                    .iter()
                    .any(|n| n.parent.as_deref() == Some(&r.id));
        let colour = AGENT[r.colour as usize % 8];
        if let Some(t) = r.truncated.clone() {
            return rsx! { div { class: "tr more", style: "--d:{r.depth}", if v == TreeVariant::Rail || v == TreeVariant::Terminal { span { class: "rl", "{r.rail}" } } "{t}" } };
        }
        let line = r.state.line();
        let awaiting = matches!(r.state, NodeState::AwaitingYou(_));
        if v == TreeVariant::Terminal {
            let g = if r.state.is_live() {
                BRAILLE[(frame + r.depth) % BRAILLE.len()]
            } else {
                r.state.glyph()
            };
            let g = if matches!(r.state, NodeState::Failed(_)) {
                "✗"
            } else {
                g
            };
            return rsx! {
                div { class: "tr term t-{tone(r.state.tone())}",
                    span { class: "rl", "{r.rail}" }
                    span { class: "g", "{g} " }
                    span { class: "nm a-{colour}", "{r.name}" }
                    span { class: "ln", "  {line}" }
                    if let Some(e) = r.elapsed.clone() { span { class: "du", "  {e}" } }
                    if r.hidden_children > 0 { span { class: "cnt", "  +{r.hidden_children}" } }
                }
            };
        }
        let rcls = format!(
            "tr t-{}{}",
            tone(r.state.tone()),
            if r.detached { " detached" } else { "" }
        );
        rsx! {
            div { class: "{rcls}", role: "treeitem",
                "aria-level": "{r.depth + 1}", "aria-expanded": "{r.expanded}", "aria-label": "{r.aria}",
                style: "--d:{r.depth}",
                if v == TreeVariant::Rail { span { class: "rl", "{r.rail}" } }
                if v == TreeVariant::Indented {
                    if has_kids {
                        button { class: if r.expanded { "tw open" } else { "tw" }, r#type: "button", "aria-label": "Toggle",
                            onclick: move |_| { let mut s = folded.write(); if !s.remove(&id) { s.insert(id.clone()); } },
                            Icon { name: "chevron", size: 11.0 }
                        }
                    } else { span { class: "tw" } }
                }
                span { class: "dt", {dot(&r.state, r.animate)} }
                span { class: "sw a-{colour}" }
                button { class: "nm", r#type: "button", onclick: move |_| on_focus.call(id2.clone()), "{r.name}" }
                span { class: if r.state.is_live() && r.animate { "ln shim" } else { "ln" }, "{line}" }
                if r.detached { span { class: "tag", "detached" } }
                span { class: "sp" }
                if let Some(ru) = r.rollup.clone() { span { class: "ru t-{tone(ru.tone())}", "{ru.glyph()}" } }
                if r.hidden_children > 0 { span { class: "cnt", "{r.hidden_children}" } }
                if let Some(e) = r.elapsed.clone() { span { class: "du", if r.paused { Icon { name: "pause", size: 10.0, stroke: 2.0 } } "{e}" } }
                if awaiting { button { class: "review", r#type: "button", onclick: move |_| on_review.call(id3.clone()), "Review" } }
            }
        }
    };
    if v == TreeVariant::InlineWrapper {
        let root = view.rows.first().cloned();
        let Some(root) = root else { return rsx! {} };
        let kids: Vec<TreeRow> = view
            .rows
            .iter()
            .skip(1)
            .filter(|r| r.depth == 1)
            .cloned()
            .collect();
        let line = root.state.line();
        return rsx! {
            div { class: "oxyn-tree inline t-{tone(root.state.tone())}",
                button { class: "hd", r#type: "button", "aria-expanded": "{wrapper_open()}", onclick: move |_| wrapper_open.toggle(),
                    span { class: "dt", {dot(&root.state, root.animate)} }
                    Icon { name: "branch", size: 14.0 }
                    span { class: "vb", "Delegated to {kids.len()} agents" }
                    span { class: if root.state.is_live() { "ln shim" } else { "ln" }, "· {line}" }
                    span { class: "sp" }
                    if let Some(e) = root.elapsed.clone() { span { class: "du", "{e}" } }
                    span { class: "chev", Icon { name: "chevron", size: 12.0 } }
                }
                if wrapper_open() { div { class: "kids", for r in kids { {row(r)} } } }
            }
        };
    }
    rsx! {
        div { class: "oxyn-tree {v.id()}", role: "tree", "aria-label": "{view.header}",
            div { class: "hdr",
                Icon { name: "agents", size: 14.0 }
                span { "{view.header}" }
            }
            for r in view.rows.clone() { {row(r)} }
            if view.folded_done > 0 { div { class: "tr more", "{view.folded_done} done" } }
        }
    }
}
