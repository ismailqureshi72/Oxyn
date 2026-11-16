//! Plan #32 (affectUI fidelity pass), overnight batch family B part 6 demos: the exact Autonomy
//! Dial, Away Summary, Collaboration Stream, Context Map and Cost Waterfall, each with every affectUI style and state.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::autonomyfid::{AutonomyAction, autonomy_demo},
};
use oxyn_dioxus::{AutonomyDial, AwaySummary, CollabStream, ContextMap, CostWaterfall};

use super::{Code, mk, sel, var};
use crate::{Align, Demo, Vals};

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

fn cstyle(v: &Vals) -> String {
    if v.s("style") == "classic" {
        "classic".into()
    } else {
        String::new()
    }
}

// ---------- Autonomy Dial ----------

fn detailed() -> Vec<AutonomyAction> {
    let details = [
        "Articles, macros and past answers",
        "Saved as drafts on the ticket",
        "Priority, tags and the assignee",
        "Replies go straight to the customer",
        "Files older than 90 days",
        "Up to the order's full amount",
    ];
    autonomy_demo()
        .into_iter()
        .zip(details)
        .map(|(a, d)| AutonomyAction {
            detail: d.into(),
            ..a
        })
        .collect()
}

fn autonomy_dial(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("style"),
        v.s("level"),
        v.s("state"),
        v.s("width")
    );
    let level = match v.s("level") {
        "always" => 0,
        "full" => 2,
        _ => 1,
    };
    let state = v.s("state");
    let pins: Option<Vec<String>> = (state == "locked").then(|| {
        vec![
            "read".to_string(),
            "purge".to_string(),
            "refund".to_string(),
        ]
    });
    let point = if state == "pointed" { "draft" } else { "" };
    let actions = (state == "details").then(detailed);
    let width = if v.s("width") == "narrow" {
        "320px"
    } else {
        "432px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            AutonomyDial { actions, default_level: level, default_pinned: pins, point: point.to_string(),
                title: "Support agent".to_string(), style: cstyle(v),
                on_level_change: move |l: u8| msg.set(format!("onLevelChange({l})")),
                on_pinned_change: move |p: Vec<String>| msg.set(format!("onPinnedChange({p:?})")) }
            {status(msg)}
        }
    }
}

fn autonomy_dial_code(_: &Vals) -> String {
    Code::new("AutonomyDial")
        .raw("actions", "Some(agent.actions())")
        .raw("default_level", "1")
        .raw("default_pinned", "Some(vec![\"refund\".into()])")
        .raw("title", "\"Support agent\".into()")
        .raw("on_level_change", "move |l| settings.set_level(l)")
        .raw("on_pinned_change", "move |p| settings.set_locks(p)")
        .done()
}

// ---------- Away Summary ----------

fn agent_glyph() -> Element {
    rsx! {
        svg { width: "13", height: "13", view_box: "0 0 12 12", "aria-hidden": "true",
            for (i, o) in [1.0, 0.45, 1.0, 0.45, 1.0, 0.45, 1.0, 0.45, 1.0].iter().enumerate() {
                rect { key: "{i}", "data-k": "{i}", x: "{(i % 3) * 4}", y: "{(i / 3) * 4}", width: "3.2", height: "3.2", rx: ".6", fill: "#8B5CF6", opacity: "{o}" }
            }
        }
    }
}

fn away_summary(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let resolved: Vec<String> = if state == "resolved" {
        vec!["export".into()]
    } else {
        vec![]
    };
    let since_clock = (state == "folded").then_some(9 * 60 + 40);
    let work_ms = if state == "slow" { 2_400 } else { 0 };
    let width = if v.s("width") == "narrow" {
        "380px"
    } else {
        "600px"
    };
    let mute = "var(--oxyn-color-fg-muted)";
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:14px;font-size:13.5px",
            div { style: "align-self:flex-end;max-width:85%;padding:6px 12px;border-radius:14px;background:color-mix(in srgb,currentColor 7%,transparent);line-height:1.35",
                "Move the nightly export to the new bucket – shout if anything's unclear." }
            div { style: "display:flex;align-items:center;gap:10px;line-height:1.35",
                {agent_glyph()}
                span { "Sure. I'll break it into steps and carry on while you're away." }
            }
            AwaySummary { now, resolved, since_clock, work_ms, expanded: state == "expanded", default_collapsed: state == "folded", style: cstyle(v),
                on_action: move |id: String| msg.set(format!("onAction({id:?})")),
                on_dismiss: move |_| msg.set("onDismiss()".into()),
                on_collapsed_change: move |c: bool| msg.set(format!("onCollapsedChange({c})")) }
            div { style: "display:flex;align-items:center;height:46px;padding:0 11px 0 14px;border:1px solid color-mix(in srgb,currentColor 10%,transparent);border-radius:12px;background:var(--oxaw-card);box-shadow:var(--oxaw-sh);color:{mute};font-size:13px",
                span { style: "flex:1", "Reply to the agent…" }
                span { "aria-hidden": "true", style: "display:inline-flex;align-items:center;justify-content:center;width:24px;height:24px;border-radius:50%;background:color-mix(in srgb,currentColor 30%,transparent)",
                    svg { width: "12", height: "12", view_box: "0 0 16 16", fill: "none", stroke: "#fff", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round", path { d: "M8 13V3.5M4 7l4-4 4 4" } } }
            }
            {status(msg)}
        }
    }
}

fn away_summary_code(_: &Vals) -> String {
    Code::new("AwaySummary")
        .raw("items", "Some(thread.away_items())")
        .raw("away", "minutes_away")
        .raw("now", "clock.now()")
        .raw("on_action", "move |id| agent.act(id)")
        .raw("on_dismiss", "move |_| seen.set(true)")
        .done()
}

// ---------- Collaboration Stream ----------

fn collab_stream(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let live = state == "live";
    let selected = if state == "selected" { "find" } else { "" }.to_string();
    let focus_agent = if state == "agent" { "analyst" } else { "" }.to_string();
    let preview = (state == "preview").then(|| ("find".to_string(), 0usize));
    let width = if v.s("width") == "narrow" {
        "380px"
    } else {
        "760px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            CollabStream { now, live, selected, focus_agent, preview, mention_open: state == "mention", style: cstyle(v),
                on_selected_change: move |id: String| msg.set(format!("onSelectedChange({id:?})")),
                on_focus_agent_change: move |id: String| msg.set(format!("onFocusAgentChange({id:?})")),
                on_send: move |(t, to, r): (String, Vec<String>, Option<String>)| msg.set(format!("onSend({t:?}, {to:?}, {r:?})")) }
            {status(msg)}
        }
    }
}

fn collab_stream_code(_: &Vals) -> String {
    Code::new("CollabStream")
        .raw("agents", "Some(run.agents())")
        .raw("messages", "Some(run.messages())")
        .raw("live", "run.is_live()")
        .raw("now", "clock.now()")
        .raw(
            "on_send",
            "move |(text, to, reply_to)| run.post_from_you(text, to, reply_to)",
        )
        .done()
}

// ---------- Context Map ----------

fn context_map(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let live = state == "live";
    let compacted = matches!(state, "after" | "before");
    let view = if state == "before" { "before" } else { "after" }.to_string();
    let grow = match state {
        "full" => 1.0,
        "card" => 0.5,
        _ => 0.12,
    };
    let point = if state == "card" { "tools" } else { "" }.to_string();
    let width = if v.s("width") == "narrow" {
        "380px"
    } else {
        "680px"
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            ContextMap { now, live, compacted, view, grow, point, menu_open: state == "menu", style: cstyle(v),
                on_select: move |id: String| msg.set(format!("onSelect({id:?})")),
                on_agent_change: move |id: String| msg.set(format!("onAgentChange({id:?})")),
                on_view_change: move |w: String| msg.set(format!("onViewChange({w:?})")) }
            {status(msg)}
        }
    }
}

fn context_map_code(_: &Vals) -> String {
    Code::new("ContextMap")
        .raw("blocks", "Some(session.context_blocks())")
        .raw("limit", "200_000")
        .raw("compaction", "session.last_compaction()")
        .raw("now", "clock.now()")
        .raw("on_select", "move |id| inspect(id)")
        .done()
}

// ---------- Cost Waterfall ----------

fn cost_waterfall(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("style"), v.s("state"), v.s("width"));
    let state = v.s("state");
    let live = state == "live";
    let tokens = state == "tokens";
    let open_id = match state {
        "details" => "write",
        "failed" => "links",
        _ => "",
    }
    .to_string();
    let collapsed: Vec<String> = if state == "folded" {
        vec!["scout".into()]
    } else {
        vec![]
    };
    let width = match v.s("width") {
        "medium" => "640px",
        "narrow" => "380px",
        _ => "756px",
    };
    rsx! {
        div { "data-k": "{key}", key: "{key}", "data-state": "{state}", style: "width:{width};max-width:100%;display:flex;flex-direction:column;gap:8px",
            CostWaterfall { now, live, tokens, open_id, collapsed, style: cstyle(v),
                on_mode_change: move |m: String| msg.set(format!("onModeChange({m:?})")),
                on_open_change: move |id: String| msg.set(format!("onOpenChange({id:?})")),
                on_collapsed_change: move |c: Vec<String>| msg.set(format!("onCollapsedChange({c:?})")) }
            {status(msg)}
        }
    }
}

fn cost_waterfall_code(_: &Vals) -> String {
    Code::new("CostWaterfall")
        .raw("steps", "Some(run.cost_steps())")
        .raw("task", "run.task.clone()")
        .raw("now", "clock.now()")
        .raw("on_open_change", "move |id| inspect(id)")
        .done()
}

/// Family B part 6 demos.
pub(crate) fn demos() -> Vec<Demo> {
    let style = || sel("style", "Style", &["clean", "classic"], "clean");
    vec![
        Demo {
            controls: vec![
                style(),
                sel("level", "Level", &["risky", "always", "full"], "risky"),
                sel(
                    "state",
                    "State",
                    &["default", "locked", "pointed", "details"],
                    "default",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "if-risky",
                    "As affectUI ships it: If risky – the two safe actions run on their own, the risky ones and the two locked destructive ones ask first.",
                    &[],
                ),
                var(
                    "full-auto",
                    "Full auto: everything runs except what you locked.",
                    &[("level", "full")],
                ),
                var(
                    "always-ask",
                    "Always ask: every action waits for you, and the first group says so.",
                    &[("level", "always")],
                ),
                var(
                    "locked",
                    "A lock set on a safe action: it always asks, whatever the level.",
                    &[("state", "locked")],
                ),
                var(
                    "pointed",
                    "A row pointed at: the row tints and its lock shows.",
                    &[("state", "pointed")],
                ),
                var(
                    "details",
                    "Actions with a detail line under the name.",
                    &[("state", "details")],
                ),
                var(
                    "narrow",
                    "Under 340 px the detail lines are left out and the levels tighten.",
                    &[("width", "narrow"), ("state", "details")],
                ),
                var(
                    "classic",
                    "Classic style: the levels in one track inside a white card, each action its own card on a grey panel.",
                    &[("style", "classic")],
                ),
            ],
            render: autonomy_dial,
            code: autonomy_dial_code,
            keyboard: vec![
                ("← → ↑ ↓", "Picks the previous or next level"),
                ("Tab", "From the levels to the locks of the actions"),
                ("Space / Enter", "Sets or removes the focused lock"),
            ],
            a11y: vec![
                "The levels are a radio group named Autonomy, described by what the chosen level means.",
                "Both groups are labelled sections; every lock is a toggle button with aria-pressed that names its action.",
                "Each change is read out politely – the new level and the counts of actions that run alone or wait for you.",
            ],
            responsive: vec![
                "432 px up to its container.",
                "Under 340 px the detail lines go and the levels get tighter; without hover the locks always show and rows grow taller.",
            ],
            height: 560,
            ..mk(
                "autonomy-dial",
                "Autonomy Dial",
                "tools",
                "How much an agent may do on its own – three levels, and its real actions gliding between Runs on its own and Asks you first, with a lock for what always asks.",
                "AutonomyDial",
                "AutonomyDialEl",
                "autonomy-dial",
                "parity/autonomyfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["default", "resolved", "expanded", "folded", "slow"],
                    "default",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "default",
                    "As affectUI ships it: two need you, one failed, five done – five rows, Show 3 more and Got it, in the thread where you left off.",
                    &[],
                ),
                var(
                    "resolved",
                    "A decision made: the row turns done in place, its tick draws and the counts roll.",
                    &[("state", "resolved")],
                ),
                var(
                    "expanded",
                    "Show 3 more: every row, and Show less.",
                    &[("state", "expanded")],
                ),
                var(
                    "folded",
                    "Since a time, folded at first: a one-line divider that opens the card again.",
                    &[("state", "folded")],
                ),
                var(
                    "slow-retry",
                    "A retry that takes a while: the button works for a moment before the row settles.",
                    &[("state", "slow")],
                ),
                var(
                    "narrow",
                    "Narrow: the counts drop under the title and long lines end in an ellipsis.",
                    &[("width", "narrow")],
                ),
                var(
                    "classic",
                    "Classic style: the header in a grey band, a stronger border and shadow.",
                    &[("style", "classic")],
                ),
            ],
            render: away_summary,
            code: away_summary_code,
            keyboard: vec![
                ("Tab", "Through the row actions, then Show more and Got it"),
                (
                    "Enter / Space",
                    "Runs the focused action; Got it folds the card and focuses the divider, which opens it again",
                ),
                ("Escape", "Folds the card from anywhere inside it"),
            ],
            a11y: vec![
                "A region named While you were away with a heading that says how long; the rows are a list.",
                "Every action names its task (\"Decide: Export job\") and each row says its state in words.",
                "Got it and the divider are buttons with aria-expanded; a working button stays focusable (aria-disabled); changes are announced politely.",
            ],
            responsive: vec![
                "600 px up to its container; rows keep one height in every state.",
                "Under 460 px the counts drop under the title; titles, details and long action labels end in an ellipsis.",
            ],
            height: 600,
            live: true,
            ..mk(
                "away-summary",
                "Away Summary",
                "tools",
                "The While you were away card for a chat thread – how long you were gone, what needs you, what failed and what's done as a short list with one action per row; Got it folds it into a one-line divider.",
                "AwaySummary",
                "AwaySummaryEl",
                "away-summary",
                "parity/awayfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["live", "settled", "selected", "agent", "preview", "mention"],
                    "live",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "live",
                    "As affectUI ships it: the run comes in message by message – the sender's mark twinkles and Writing… shimmers first – with a Live dot.",
                    &[],
                ),
                var(
                    "settled",
                    "The whole run: a quiet start line, an answer with its files, a handoff, and a question that needs you with Reply.",
                    &[("state", "settled")],
                ),
                var(
                    "details",
                    "A message picked: the rest fades and its details drop in under it.",
                    &[("state", "selected")],
                ),
                var(
                    "agent-tab",
                    "An agent tab picked: only what it sent or got stays.",
                    &[("state", "agent")],
                ),
                var(
                    "attachment-preview",
                    "A file chip opened: the preview window over the stream, Escape or × closes it.",
                    &[("state", "preview")],
                ),
                var(
                    "mention",
                    "@ in the composer: the agents with their models, arrows and Enter to name one.",
                    &[("state", "mention")],
                ),
                var(
                    "narrow",
                    "Narrow: bubbles wrap and the tabs scroll sideways.",
                    &[("width", "narrow"), ("state", "settled")],
                ),
                var(
                    "classic",
                    "Classic style: tabs in a grey strip, flat grey bubbles, a flat composer.",
                    &[("style", "classic"), ("state", "settled")],
                ),
            ],
            render: collab_stream,
            code: collab_stream_code,
            keyboard: vec![
                (
                    "Tab",
                    "The tabs are one stop, the stream another; then the focused message's Reply and chips, then the composer",
                ),
                ("← → / Home End", "Between the agent tabs"),
                ("↑ ↓ / Home End", "Between the messages"),
                (
                    "Enter / Space",
                    "Picks the focused message and opens its details, or lets it go",
                ),
                (
                    "Escape",
                    "Lets go of the message or tab; drops the reply; closes the preview",
                ),
                (
                    "@",
                    "Opens the agents in the composer – type to filter, arrows, Enter or Tab to name one",
                ),
                (
                    "Enter",
                    "Sends from the composer; Shift+Enter starts a new line",
                ),
            ],
            a11y: vec![
                "A polite live log; each message is named by author, recipient, time and content.",
                "Roving focus: one message and one tab are in the tab order; tabs are toggle buttons with aria-pressed in a labelled toolbar.",
                "The @ list is a listbox tied to the field with aria-expanded; the preview is a modal dialog that hands focus back to its chip.",
                "Needs you and Handoff are words, not just colours; pixel marks and logos are hidden from assistive tech.",
            ],
            responsive: vec![
                "760 px up to its container; bubbles hug their words up to 560 px.",
                "Under 520 px the gutters shrink; the tabs scroll sideways with a fade at the edge; the preview fits the component.",
            ],
            height: 900,
            live: true,
            align: Align::Start,
            ..mk(
                "collaboration-stream",
                "Collaboration Stream",
                "tools",
                "What agents say to each other – a quiet chat where every message shows who it is from and who it is for, handoffs and questions for you tagged, files as chips that open a preview, details under a message, and a composer that names agents with @.",
                "CollabStream",
                "CollabStreamEl",
                "collaboration-stream",
                "parity/collabfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["live", "fresh", "full", "after", "before", "card", "menu"],
                    "live",
                ),
                sel("width", "Width", &["wide", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "live",
                    "As affectUI ships it: the session grows tile by tile, hits the limit, compacts – old turns squeeze into a summary – and Before | After appears.",
                    &[],
                ),
                var(
                    "fresh",
                    "A fresh session: small tiles, the dashed free space, no compaction so far.",
                    &[("state", "fresh")],
                ),
                var(
                    "nearly-full",
                    "Nearly full: the percent turns amber and the free space is a sliver.",
                    &[("state", "full")],
                ),
                var(
                    "after-compaction",
                    "After a compaction: kept parts, a green summary, and the line of what was kept, summarized and dropped.",
                    &[("state", "after")],
                ),
                var(
                    "before-compaction",
                    "Before: the parts that were summarized or dropped come back, dashed and tagged.",
                    &[("state", "before")],
                ),
                var(
                    "open-a-part",
                    "A tile pointed at: its card with tokens, share and what it holds.",
                    &[("state", "card")],
                ),
                var(
                    "agent-menu",
                    "The agent menu: each agent with its model and window size.",
                    &[("state", "menu")],
                ),
                var(
                    "narrow",
                    "Under 480 px the map is shorter and the numbers take a line of their own.",
                    &[("width", "narrow"), ("state", "after")],
                ),
                var(
                    "classic",
                    "Classic style: a grey band, more saturated tiles with a solid edge, a white percent.",
                    &[("style", "classic"), ("state", "after")],
                ),
            ],
            render: context_map,
            code: context_map_code,
            keyboard: vec![
                (
                    "Tab",
                    "Through the agent menu, Before | After and the tiles",
                ),
                (
                    "Enter / Space",
                    "Pins the focused tile's card, or lets it go",
                ),
                ("Escape", "Lets go of the pinned card and closes the menu"),
            ],
            a11y: vec![
                "The map is a list named by its title; every tile is an item with its name, kind and tokens.",
                "The fill level is text in the bar – its colour is extra, not the only signal.",
                "Before | After is a radio group; the agent menu is a listbox.",
            ],
            responsive: vec![
                "680 px up to its container; the map keeps its height and tiles too small for a label show only their colour.",
                "Under 480 px the map is 220 px tall and the numbers move to a line of their own.",
            ],
            height: 520,
            live: true,
            align: Align::Start,
            ..mk(
                "context-map",
                "Context Map",
                "tools",
                "A treemap of everything in an agent's context: each prompt, tool list, memory, file, chat turn and tool result is a tile sized by tokens, tiles slide as the window fills, unused room stays hatched, and compaction folds old turns into one summary tile.",
                "ContextMap",
                "ContextMapEl",
                "context-map",
                "parity/ctxmapfid.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "State",
                    &["live", "settled", "tokens", "details", "failed", "folded"],
                    "live",
                ),
                sel("width", "Width", &["wide", "medium", "narrow"], "wide"),
            ],
            variants: vec![
                var(
                    "live",
                    "As affectUI ships it: the run plays out – rows slide in, the running bar shines, the totals climb, Live turns Done.",
                    &[],
                ),
                var(
                    "settled",
                    "The finished run in dollars: the most expensive step tagged Top and named in the stats.",
                    &[("state", "settled")],
                ),
                var(
                    "tokens",
                    "Open in the tokens view: the axis and every bar re-based on tokens in and out.",
                    &[("state", "tokens")],
                ),
                var(
                    "details",
                    "A step opened: its prompt and response under the row.",
                    &[("state", "details")],
                ),
                var(
                    "failed-step",
                    "A failed tool call: a red sliver, a red time, the error in its details.",
                    &[("state", "failed")],
                ),
                var(
                    "folded",
                    "Fold finished sub-agents: the collector folded with a +4 chip.",
                    &[("state", "folded")],
                ),
                var(
                    "medium",
                    "Under 720 px the In and Out columns step aside.",
                    &[("width", "medium"), ("state", "settled")],
                ),
                var(
                    "narrow",
                    "Under 520 px each row stacks: name, cost and time, the bar under it.",
                    &[("width", "narrow"), ("state", "settled")],
                ),
                var(
                    "classic",
                    "Classic style: the header on a grey band, sentence-case stats, saturated bars.",
                    &[("style", "classic"), ("state", "settled")],
                ),
            ],
            render: cost_waterfall,
            code: cost_waterfall_code,
            keyboard: vec![
                (
                    "Tab",
                    "The switch, the most expensive step, then the rows – one stop",
                ),
                ("↑ ↓ / Home End", "Between the rows"),
                ("→ / ←", "Opens or folds a sub-agent"),
                (
                    "Enter / Space",
                    "Opens or closes the focused step's details",
                ),
                ("Escape", "Closes the details"),
            ],
            a11y: vec![
                "The rows are a tree: every step a treeitem with its level, labelled with name, kind, status, cost, tokens and time; aria-expanded on sub-agents.",
                "Roving focus: only one row is in the tab order.",
                "Cost and Tokens are pressed-state buttons in a named group; Live or Done is a status region.",
            ],
            responsive: vec![
                "756 px up to its container: steps left, the waterfall in the middle, the numbers right.",
                "Under 720 px In and Out step aside; under 520 px each row stacks with the bar across the width.",
            ],
            height: 560,
            live: true,
            align: Align::Start,
            ..mk(
                "cost-waterfall",
                "Cost Waterfall",
                "tools",
                "A run's spend, step by step: model calls, tools and sub-agents stack across to the total, each with its model, tokens in and out, price and duration; the priciest step is flagged and one toggle swaps money for tokens.",
                "CostWaterfall",
                "CostWaterfallEl",
                "cost-waterfall",
                "parity/costfid.rs",
            )
        },
    ]
}
