//! Tool calls (plan #28, chunk 3): `ToolCallCard`, `ToolCallGroup`, `ToolCall`, `ToolChips`,
//! `JsonViewer`, `ExpandableCard`, `LinkUnfurl`, and the tool-kind map in `ToolKindLegend`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::tools::{
        CallCard, CallStatus, Json, JsonTree, ToolKindMap, Unfurl, UnfurlState, group_label,
        ms_label,
    },
    verbs::VerbPack,
};

use crate::composer::icons::Icon;

fn tint(text: &str) -> Element {
    // Light JSON tint: keys, strings, numbers, literals.
    let mut parts: Vec<(String, &'static str)> = Vec::new();
    for line in text.lines() {
        let t = line.trim_start();
        let pad = &line[..line.len() - t.len()];
        if let Some((k, v)) = t.split_once("\": ")
            && t.starts_with('"')
        {
            parts.push((format!("{pad}{k}\""), "k"));
            parts.push((": ".into(), ""));
            let cls = if v.starts_with('"') {
                "s"
            } else if v.starts_with(['t', 'f', 'n']) {
                "l"
            } else if v.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
                "n"
            } else {
                ""
            };
            parts.push((format!("{v}\n"), cls));
        } else {
            parts.push((
                format!("{line}\n"),
                if t.starts_with('"') { "s" } else { "" },
            ));
        }
    }
    rsx! { for (t, c) in parts { span { class: "oxt-{c}", "{t}" } } }
}

fn pretty(s: &str) -> String {
    Json::parse(s)
        .map(|j| j.pretty(0))
        .unwrap_or_else(|| s.to_string())
}

/// One tool call as a line: a tile, the name in mono, a summary, a live timer or final duration,
/// status; folds open to highlighted arguments and result with copy, the error with Retry, and
/// code capped at `max_code` px with a fade and Show all.
#[component]
pub fn ToolCallCard(
    /// Card state.
    card: CallCard,
    /// Shared clock.
    now: ClockMs,
    /// Icon in the tile.
    #[props(default = "tool".to_string())]
    icon: String,
    /// Code height cap, px.
    #[props(default = 220)]
    max_code: u32,
    /// Header toggled.
    #[props(default)]
    on_toggle: EventHandler<()>,
    /// Retry pressed (shows the button).
    #[props(default)]
    on_retry: Option<EventHandler<()>>,
) -> Element {
    let mut full = use_signal(|| false);
    let st = card.status;
    let time = card.time(now);
    let cap = if full() {
        "none".to_string()
    } else {
        format!("{max_code}px")
    };
    let mut copied = use_signal(|| false);
    rsx! {
        div { class: "oxt-card", "data-status": st.id(), "data-open": "{card.open}",
            button { class: "oxt-head", r#type: "button", "aria-expanded": "{card.open}", onclick: move |_| on_toggle.call(()),
                span { class: "oxt-tile", Icon { name: icon.clone(), size: 14.0 } }
                span { class: "oxt-name", "{card.name}" }
                span { class: "oxt-sum", "{card.summary}" }
                span { class: "oxt-sp" }
                match st {
                    CallStatus::Running => rsx! { span { class: "oxp-spin" } },
                    CallStatus::Success => rsx! { span { class: "oxt-ok", Icon { name: "check", size: 12.0, stroke: 2.2 } } },
                    CallStatus::Error => rsx! { span { class: "oxt-bad", "Failed" } },
                    CallStatus::Approval => rsx! { span { class: "oxt-wait", "Needs approval" } },
                    CallStatus::Denied => rsx! { span { class: "oxt-bad", "Denied" } },
                }
                if let Some(t) = time { span { class: "oxt-time", "{t}" } }
                span { class: "oxt-chev", Icon { name: "chevron", size: 12.0 } }
            }
            if card.open {
                div { class: "oxt-body",
                    if let Some(a) = card.args.clone() {
                        div { class: "oxt-sec",
                            div { class: "oxt-sh", "Arguments" span { class: "oxt-sp" } button { class: "oxt-copy", r#type: "button", "aria-label": "Copy arguments", "data-copy": "{a}", onclick: move |_| copied.set(true), Icon { name: if copied() { "check" } else { "copy" }, size: 12.0 } } }
                            pre { class: "oxt-code", style: "max-height:{cap}", {tint(&pretty(&a))} }
                        }
                    }
                    if let Some(e) = card.error.clone() {
                        div { class: "oxt-err", Icon { name: "alert", size: 13.0 } span { "{e}" } span { class: "oxt-sp" }
                            if let Some(r) = on_retry { button { class: "oxr-btn", r#type: "button", onclick: move |_| r.call(()), Icon { name: "retry", size: 12.0 } "Retry" } } }
                    } else if let Some(r) = card.result.clone() {
                        div { class: "oxt-sec",
                            div { class: "oxt-sh", "Result" span { class: "oxt-sp" } button { class: "oxt-copy", r#type: "button", "aria-label": "Copy result", "data-copy": "{r}", onclick: move |_| copied.set(true), Icon { name: if copied() { "check" } else { "copy" }, size: 12.0 } } }
                            pre { class: "oxt-code", "data-capped": "{!full()}", style: "max-height:{cap}", {tint(&pretty(&r))} }
                            if pretty(&r).lines().count() > (max_code / 18) as usize { button { class: "oxr-lnk", r#type: "button", onclick: move |_| full.toggle(), if full() { "Show less" } else { "Show all" } } }
                        }
                    } else if st == CallStatus::Running {
                        div { class: "oxt-wait-r oxr-shim", "Waiting for result…" }
                    }
                }
            }
        }
    }
}

/// A group of tool calls with a header that sums them up ("Used 4 tools · 2.4s · 1 failed")
/// and folds them away.
#[component]
pub fn ToolCallGroup(
    /// Cards.
    cards: Vec<CallCard>,
    /// Shared clock.
    now: ClockMs,
    /// Open.
    #[props(default = true)]
    open: bool,
    /// Header toggled.
    #[props(default)]
    on_open: EventHandler<bool>,
    /// A card toggled.
    #[props(default)]
    on_toggle: EventHandler<usize>,
    /// Retry a card.
    #[props(default)]
    on_retry: EventHandler<usize>,
) -> Element {
    let label = group_label(&cards);
    let live = cards.iter().any(|c| c.status == CallStatus::Running);
    let map = ToolKindMap::standard();
    rsx! {
        div { class: "oxt-group",
            button { class: "oxt-ghead", r#type: "button", "aria-expanded": "{open}", onclick: move |_| on_open.call(!open),
                span { class: "oxt-tiles", for c in cards.iter().take(4) { span { class: "oxt-tile sm", Icon { name: map.kind(&c.name).icon(), size: 11.0 } } } }
                span { class: if live { "oxr-shim" } else { "" }, "{label}" }
                span { class: "oxt-chev", Icon { name: "chevron", size: 12.0 } }
            }
            if open {
                div { class: "oxt-gbody",
                    for (i, c) in cards.iter().cloned().enumerate() {
                        ToolCallCard { card: c.clone(), now, icon: map.kind(&c.name).icon().to_string(), on_toggle: move |_| on_toggle.call(i),
                            on_retry: EventHandler::new(move |_| on_retry.call(i)) }
                    }
                }
            }
        }
    }
}

/// Output shape of a [`ToolCall`].
#[derive(Clone, Debug, PartialEq)]
pub enum RowOutput {
    /// Numbered list of `(title, tag)`.
    List(Vec<(String, String)>),
    /// Table: columns and rows.
    Table(Vec<String>, Vec<Vec<String>>),
    /// JSON text.
    Json(String),
    /// Plain text.
    Text(String),
}

/// A tool call row for transcripts: the call in plain words with a verb per status (running
/// shimmers), an approval state with Allow / Deny, error with Retry, and Input / Output a click
/// away (list, table, JSON or text output, folding after five rows).
#[component]
pub fn ToolCall(
    /// Raw tool name (drives the kind icon and default verbs).
    tool: String,
    /// Settled phrase ("Searched issues in arc-ui/web").
    title: String,
    /// Running phrase.
    #[props(default)]
    running_title: Option<String>,
    /// Status.
    status: CallStatus,
    /// One line under the title (error reason, or what allowing will do).
    #[props(default)]
    message: Option<String>,
    /// Input key/values.
    #[props(default)]
    input: Vec<(String, String)>,
    /// Output.
    #[props(default)]
    output: Option<RowOutput>,
    /// Duration ms.
    #[props(default)]
    duration_ms: Option<u64>,
    /// Open.
    #[props(default)]
    open: bool,
    /// Toggle.
    #[props(default)]
    on_toggle: EventHandler<()>,
    /// Allow (approval).
    #[props(default)]
    on_allow: EventHandler<()>,
    /// Deny (approval).
    #[props(default)]
    on_deny: EventHandler<()>,
    /// Retry (error).
    #[props(default)]
    on_retry: EventHandler<()>,
) -> Element {
    let mut more = use_signal(|| false);
    let info = ToolKindMap::standard().info(&tool, VerbPack::neutral());
    let t = match status {
        CallStatus::Running => running_title
            .clone()
            .unwrap_or(format!("{}…", info.present)),
        _ => title.clone(),
    };
    let summary = match &output {
        Some(RowOutput::List(l)) => Some(format!("{} results", l.len())),
        Some(RowOutput::Table(_, r)) => Some(format!("{} rows", r.len())),
        _ => None,
    };
    let lim = if more() { usize::MAX } else { 5 };
    rsx! {
        div { class: "oxt-row", "data-status": status.id(),
            button { class: "oxt-rhead", r#type: "button", "aria-expanded": "{open}", onclick: move |_| on_toggle.call(()),
                span { class: "oxt-mark", Icon { name: info.icon, size: 13.0 } if let Some(s) = info.server.clone() { span { class: "oxt-srv", "{s}" } } }
                span { class: if status == CallStatus::Running { "oxt-title oxr-shim" } else { "oxt-title" }, "{t}" }
                span { class: "oxt-sp" }
                if let Some(s) = summary.clone().filter(|_| status == CallStatus::Success) { span { class: "oxt-time", "{s}" } }
                if let Some(d) = duration_ms { span { class: "oxt-time", "{ms_label(d)}" } }
                match status {
                    CallStatus::Running => rsx! { span { class: "oxp-spin" } },
                    CallStatus::Success => rsx! { span { class: "oxt-ok", Icon { name: "check", size: 12.0, stroke: 2.2 } } },
                    CallStatus::Error => rsx! { span { class: "oxt-bad", "Failed" } },
                    CallStatus::Denied => rsx! { span { class: "oxt-bad", "Denied" } },
                    CallStatus::Approval => rsx! { span { class: "oxt-wait", "Approval" } },
                }
            }
            if let Some(m) = message.clone() { div { class: "oxt-msg", "{m}" } }
            if status == CallStatus::Approval {
                div { class: "oxt-appr", button { class: "oxr-btn", r#type: "button", onclick: move |_| on_deny.call(()), "Deny" } button { class: "oxr-btn go", r#type: "button", onclick: move |_| on_allow.call(()), "Allow" } }
            }
            if status == CallStatus::Error { div { class: "oxt-appr", button { class: "oxr-btn", r#type: "button", onclick: move |_| on_retry.call(()), Icon { name: "retry", size: 12.0 } "Retry" } } }
            if open {
                div { class: "oxt-io",
                    if !input.is_empty() {
                        div { class: "oxt-sh", "Input" }
                        dl { class: "oxt-kv", for (k, v) in input.iter() { div { dt { "{k}" } dd { "{v}" } } } }
                    }
                    if let Some(o) = output.clone() {
                        div { class: "oxt-sh", "Output" }
                        match o {
                            RowOutput::List(l) => rsx! { ol { class: "oxt-list", for (i, (a, b)) in l.iter().enumerate().take(lim) { li { span { class: "oxr-n", "{i + 1}" } span { "{a}" } span { class: "oxt-sp" } span { class: "oxr-tool", "{b}" } } } } if l.len() > 5 { button { class: "oxr-lnk", r#type: "button", onclick: move |_| more.toggle(), if more() { "Show less" } else { "Show {l.len() - 5} more" } } } },
                            RowOutput::Table(c, r) => rsx! { table { class: "oxt-tbl", thead { tr { for h in c.iter() { th { "{h}" } } } } tbody { for row in r.iter().take(lim) { tr { for x in row.iter() { td { "{x}" } } } } } } if r.len() > 5 { button { class: "oxr-lnk", r#type: "button", onclick: move |_| more.toggle(), if more() { "Show less" } else { "Show {r.len() - 5} more" } } } },
                            RowOutput::Json(j) => rsx! { pre { class: "oxt-code", {tint(&pretty(&j))} } },
                            RowOutput::Text(t) => rsx! { p { class: "oxt-msg", "{t}" } },
                        }
                    }
                }
            }
        }
    }
}

/// The group around [`ToolCall`] rows: a header that reads "Using tools…" (shimmering) while
/// the turn is active and "Used 4 tools · 2.4s" after, with the call marks stacked, folding the
/// rows away. Rows are its children.
#[component]
pub fn ToolCallList(
    /// Raw tool names of the calls (marks and count).
    tools: Vec<String>,
    /// Total duration ms.
    #[props(default)]
    total_ms: u64,
    /// The agent is still working in this turn.
    #[props(default)]
    active: bool,
    /// Open.
    open: bool,
    /// Header toggled.
    #[props(default)]
    on_open: EventHandler<bool>,
    /// The rows.
    children: Element,
) -> Element {
    let map = ToolKindMap::standard();
    let n = tools.len();
    let label = if active {
        "Using tools…".to_string()
    } else if total_ms > 0 {
        format!(
            "Used {n} tool{} · {}",
            if n == 1 { "" } else { "s" },
            ms_label(total_ms)
        )
    } else {
        format!("Used {n} tool{}", if n == 1 { "" } else { "s" })
    };
    rsx! {
        div { class: "oxt-group oxt-list-g", "data-open": "{open}",
            button { class: "oxt-ghead", r#type: "button", "aria-expanded": "{open}", onclick: move |_| on_open.call(!open),
                span { class: "oxt-tiles", for t in tools.iter().take(4) { span { class: "oxt-tile sm", Icon { name: map.kind(t).icon(), size: 11.0 } } } }
                span { class: if active { "oxr-shim" } else { "" }, "{label}" }
                span { class: "oxt-chev", Icon { name: "chevron", size: 12.0 } }
            }
            if open { div { class: "oxt-gbody oxt-rows", {children} } }
        }
    }
}

/// One step of [`ToolChips`].
#[derive(Clone, Debug, PartialEq)]
pub struct ChipStep {
    /// Tool name (icon from the kind map).
    pub tool: String,
    /// Label ("Searched").
    pub label: String,
    /// Chip text ("router.ts").
    pub chip: String,
    /// Chip in mono.
    pub mono: bool,
    /// Live state.
    pub status: CallStatus,
    /// Detail lines shown when the row opens.
    pub detail: Vec<String>,
}

/// One file-diff chip of [`ToolChips`].
#[derive(Clone, Debug, PartialEq)]
pub struct DiffChip {
    /// File.
    pub file: String,
    /// Added lines.
    pub add: usize,
    /// Removed lines.
    pub del: usize,
    /// Diff lines (`+`/`-`/` ` prefixed).
    pub lines: Vec<String>,
}

/// Tool chips (Beautiful UI): an agent run as compact rows of tool calls with inline chips and a
/// live state per chip, then file-diff chips whose diff opens on hover or focus (and pins on
/// click). Rows, chips or cards density.
#[component]
pub fn ToolChips(
    /// Steps.
    steps: Vec<ChipStep>,
    /// Diffs.
    diffs: Vec<DiffChip>,
    /// Messages in the run (header count).
    #[props(default)]
    messages: usize,
    /// `rows` · `chips` · `cards`.
    #[props(default = "rows".to_string())]
    density: String,
    /// Start open.
    #[props(default = true)]
    open: bool,
) -> Element {
    let mut is_open = use_signal(|| open);
    let mut row = use_signal(|| None::<usize>);
    let mut peek = use_signal(|| None::<usize>);
    let mut pinned = use_signal(|| None::<usize>);
    let map = ToolKindMap::standard();
    let n = steps.len();
    let shown = pinned().or(peek());
    rsx! {
        div { class: "oxt-chips {density}",
            button { class: "oxt-ghead", r#type: "button", "aria-expanded": "{is_open()}", onclick: move |_| is_open.toggle(),
                span { "{n} tool calls" if messages > 0 { ", {messages} messages" } }
                span { class: "oxt-chev", Icon { name: "chevron", size: 12.0 } }
            }
            if is_open() {
                ul { class: "oxt-steps",
                    for (i, s) in steps.iter().enumerate() {
                        li { class: "oxt-step", "data-status": s.status.id(),
                            button { class: "oxt-shead", r#type: "button", "aria-expanded": "{row() == Some(i)}", onclick: move |_| row.set(if row() == Some(i) { None } else { Some(i) }),
                                Icon { name: map.kind(&s.tool).icon(), size: 13.0 }
                                span { "{s.label}" }
                                span { class: if s.mono { "oxt-chip mono" } else { "oxt-chip" }, "{s.chip}" }
                                match s.status { CallStatus::Running => rsx! { span { class: "oxp-spin" } }, CallStatus::Error => rsx! { span { class: "oxt-bad", "failed" } }, CallStatus::Approval => rsx! { span { class: "oxt-wait", "needs approval" } }, _ => rsx! {} }
                                span { class: "oxt-chev", Icon { name: "chevron", size: 11.0 } }
                            }
                            if row() == Some(i) { ul { class: "oxt-sdet", for d in s.detail.iter() { li { "{d}" } } } }
                        }
                    }
                }
                div { class: "oxt-diffs",
                    for (i, d) in diffs.iter().enumerate() {
                        button { class: "oxt-dchip", r#type: "button", "aria-expanded": "{shown == Some(i)}", "aria-pressed": "{pinned() == Some(i)}",
                            onmouseenter: move |_| peek.set(Some(i)), onmouseleave: move |_| peek.set(None),
                            onfocus: move |_| peek.set(Some(i)), onblur: move |_| peek.set(None),
                            onclick: move |_| pinned.set(if pinned() == Some(i) { None } else { Some(i) }),
                            Icon { name: "file", size: 12.0 } span { class: "oxt-mono", "{d.file}" }
                            span { class: "oxt-add", "+{d.add}" } span { class: "oxt-del", "−{d.del}" }
                        }
                    }
                }
                if let Some(i) = shown {
                    if let Some(d) = diffs.get(i) {
                        div { class: "oxt-dprev", role: "region", "aria-label": "Diff of {d.file}",
                            for l in d.lines.iter() { div { class: if l.starts_with('+') { "add" } else if l.starts_with('-') { "del" } else { "ctx" }, "{l}" } }
                        }
                    }
                }
            }
        }
    }
}

/// JSON viewer: a collapsible tree with search (matches open their branches), paging for long
/// arrays, copy value or path, a current-row path bar and full keyboard moves.
#[component]
pub fn JsonViewer(
    /// Tree state.
    tree: JsonTree,
    /// Show the search field.
    #[props(default = true)]
    searchable: bool,
    /// Show copy actions.
    #[props(default = true)]
    copyable: bool,
    /// Show the current path.
    #[props(default = true)]
    show_path: bool,
    /// Max height px.
    #[props(default = 320)]
    max_height: u32,
    /// Accessible name.
    #[props(default = "JSON".to_string())]
    label: String,
    /// Toggle a branch.
    #[props(default)]
    on_toggle: EventHandler<String>,
    /// Query changed.
    #[props(default)]
    on_query: EventHandler<String>,
    /// Next page of a branch.
    #[props(default)]
    on_more: EventHandler<String>,
    /// Key: `up` · `down` · `left` · `right` · `home` · `end`, or select a row index.
    #[props(default)]
    on_key: EventHandler<String>,
) -> Element {
    let rows = tree.rows();
    let cur = tree.current.min(rows.len().saturating_sub(1));
    let path = rows.get(cur).map(|r| r.path.clone()).unwrap_or_default();
    let mut copied = use_signal(|| None::<String>);
    let mut search_el = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut tree_el = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let focus_el = |el: Option<std::rc::Rc<MountedData>>| {
        if let Some(el) = el {
            spawn(async move {
                let _ = el.set_focus(true).await;
            });
        }
    };
    let cur_value = rows
        .get(cur)
        .and_then(|r| tree.value_text(&r.path))
        .unwrap_or_default();
    let cur_path = path.clone();
    rsx! {
        div { class: "oxt-json",
            if searchable {
                label { class: "oxt-jsearch", Icon { name: "search", size: 12.0 }
                    input { r#type: "search", placeholder: "Search keys and values", value: "{tree.query}", "aria-label": "Search JSON",
                        onmounted: move |e| search_el.set(Some(e.data())),
                        oninput: move |e| on_query.call(e.value()),
                        onkeydown: move |e: KeyboardEvent| match e.key() {
                            Key::Enter => { e.prevent_default(); on_key.call(if e.modifiers().shift() { "prev" } else { "next" }.to_string()) }
                            Key::Escape => { e.prevent_default(); on_query.call(String::new()) }
                            Key::ArrowDown => { e.prevent_default(); focus_el(tree_el()) }
                            _ => {}
                        },
                    }
                    kbd { class: "oxt-kbd", "aria-hidden": "true", "/" }
                }
            }
            div { class: "oxt-jtree", role: "tree", "aria-label": "{label}", tabindex: "0", style: "max-height:{max_height}px",
                "aria-activedescendant": "oxt-j-{cur}",
                onmounted: move |e| tree_el.set(Some(e.data())),
                onkeydown: move |e: KeyboardEvent| {
                    let copy_mod = e.modifiers().ctrl() || e.modifiers().meta();
                    if copy_mod && matches!(e.key(), Key::Character(ref c) if c.eq_ignore_ascii_case("c")) {
                        copied.set(Some(if e.modifiers().shift() { "path".into() } else { "value".into() }));
                        return;
                    }
                    let k = match e.key() {
                        Key::ArrowDown => "down", Key::ArrowUp => "up", Key::ArrowLeft => "left", Key::ArrowRight => "right",
                        Key::Home => "home", Key::End => "end", Key::Enter => "toggle",
                        Key::Character(ref c) if c == " " => "toggle",
                        Key::Character(ref c) if c == "/" && searchable => { e.prevent_default(); focus_el(search_el()); "" }
                        _ => "",
                    };
                    if !k.is_empty() { e.prevent_default(); on_key.call(k.to_string()) }
                },
                for (i, r) in rows.iter().cloned().enumerate() {
                    div { class: "oxt-jrow", role: "treeitem", id: "oxt-j-{i}", "aria-level": "{r.depth + 1}", "aria-selected": "{i == cur}", "aria-expanded": if r.branch { if r.open { "true" } else { "false" } } else { "" },
                        "data-hit": "{r.hit}", "data-kind": r.kind, style: "--d:{r.depth}",
                        onclick: move |_| on_key.call(format!("#{i}")),
                        if let Some(_n) = r.more {
                            button { class: "oxr-lnk", r#type: "button", onclick: { let p = r.path.clone(); move |e: MouseEvent| { e.stop_propagation(); on_more.call(p.clone()) } }, "{r.preview}" }
                        } else {
                            if r.branch {
                                button { class: "oxr-fold", r#type: "button", tabindex: "-1", "data-folded": "{!r.open}", "aria-label": "Toggle {r.key}",
                                    onclick: { let p = r.path.clone(); move |e: MouseEvent| { e.stop_propagation(); on_toggle.call(p.clone()) } }, Icon { name: "chevron", size: 11.0 } }
                            } else { span { class: "oxt-jpad" } }
                            span { class: "oxt-jk", "{r.key}" }
                            span { class: "oxt-jv oxt-{r.kind}", "{r.preview}" }
                            if copyable {
                                span { class: "oxt-jact",
                                    button { r#type: "button", "aria-label": "Copy value", "data-copy": "{tree.value_text(&r.path).unwrap_or_default()}", onclick: move |e: MouseEvent| { e.stop_propagation(); copied.set(Some("value".into())) }, Icon { name: "copy", size: 11.0 } }
                                    button { r#type: "button", "aria-label": "Copy path", "data-copy": "{r.path}", onclick: move |e: MouseEvent| { e.stop_propagation(); copied.set(Some("path".into())) }, Icon { name: "link", size: 11.0 } }
                                }
                            }
                        }
                    }
                }
                if rows.is_empty() { div { class: "oxr-dim oxt-jempty", "No matches" } }
            }
            span { class: "oxr-sr", "data-copy-value": "{cur_value}", "data-copy-path": "{cur_path}" }
            if show_path { div { class: "oxt-jpath", span { class: "oxt-mono", "{path}" } span { class: "oxt-sp" } if let Some(c) = copied() { span { class: "oxr-dim", role: "status", "Copied {c}" } } } }
        }
    }
}

/// Expandable card: a dense card that gives itself more room on request. Width and height morph
/// together, details fade in after the box has made room, Escape closes from inside.
#[component]
pub fn ExpandableCard(
    /// Title.
    title: String,
    /// Summary line.
    description: String,
    /// Icon.
    #[props(default = "tool".to_string())]
    icon: String,
    /// Expanded.
    expanded: bool,
    /// Collapsed width cap px (0 = fill).
    #[props(default)]
    width: u32,
    /// Expanded width cap px (0 = width).
    #[props(default)]
    expanded_width: u32,
    /// Toggled.
    #[props(default)]
    on_toggle: EventHandler<bool>,
    /// Details.
    children: Element,
) -> Element {
    let w = if expanded && expanded_width > 0 {
        expanded_width
    } else {
        width
    };
    let style = if w > 0 {
        format!("width:{w}px")
    } else {
        String::new()
    };
    rsx! {
        div { class: "oxt-xcard", "data-open": "{expanded}", style: "{style}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape && expanded { on_toggle.call(false) },
            button { class: "oxt-xhead", r#type: "button", "aria-expanded": "{expanded}", onclick: move |_| on_toggle.call(!expanded),
                span { class: "oxt-tile", Icon { name: icon.clone(), size: 14.0 } }
                span { class: "oxt-xt",
                    span { class: "oxt-xtitle", "{title}" }
                    span { class: "oxt-xdesc", key: "{description}", "{description}" }
                }
                span { class: "oxt-chev", Icon { name: "chevron", size: 12.0 } }
            }
            div { class: "oxt-xbody", "aria-hidden": "{!expanded}", div { class: "oxt-xin", {children} } }
        }
    }
}

/// Link unfurl: a message field where a pasted link shimmers while it resolves, then unfurls
/// into a preview card (site, title, description, image), which can fold back to an inline
/// link with its title or be removed.
#[component]
pub fn LinkUnfurl(
    /// Message text.
    text: String,
    /// Links in the message.
    links: Vec<Unfurl>,
    /// Shared clock.
    now: ClockMs,
    /// Field label / placeholder.
    #[props(default = "Message #lisbon-offsite".to_string())]
    label: String,
    /// Text changed.
    #[props(default)]
    on_input: EventHandler<String>,
    /// Card folded or opened.
    #[props(default)]
    on_toggle: EventHandler<usize>,
    /// Link removed.
    #[props(default)]
    on_remove: EventHandler<usize>,
    /// Send.
    #[props(default)]
    on_send: EventHandler<()>,
) -> Element {
    let _ = now;
    rsx! {
        div { class: "oxt-unfurl",
            div { class: "oxt-ufield",
                textarea { rows: "2", placeholder: "{label}", "aria-label": "{label}", value: "{text}", oninput: move |e| on_input.call(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); on_send.call(()) } }
                div { class: "oxt-ulinks",
                    for (i, l) in links.iter().enumerate() {
                        match l.state.clone() {
                            UnfurlState::Loading => rsx! { span { class: "oxt-ulink loading", Icon { name: "link", size: 11.0 } span { class: "oxr-shim", "{l.host()}" } } },
                            UnfurlState::Plain => rsx! { span { class: "oxt-ulink", Icon { name: "link", size: 11.0 } "{l.url}" } },
                            UnfurlState::Inline(p) => rsx! { button { class: "oxt-ulink on", r#type: "button", onclick: move |_| on_toggle.call(i), span { class: "oxt-fav", "{p.site.chars().next().unwrap_or('·')}" } "{p.title}" } },
                            UnfurlState::Card(p) => rsx! {
                                div { class: "oxt-ucard",
                                    if p.image { div { class: "oxt-uimg", "aria-hidden": "true" } }
                                    div { class: "oxt-ubody",
                                        div { class: "oxt-usite", span { class: "oxt-fav", "{p.site.chars().next().unwrap_or('·')}" } "{p.site}" }
                                        div { class: "oxt-utitle", "{p.title}" }
                                        div { class: "oxr-dim oxr-small", "{p.description}" }
                                    }
                                    div { class: "oxt-uact",
                                        button { r#type: "button", "aria-label": "Collapse preview", onclick: move |_| on_toggle.call(i), Icon { name: "minus", size: 12.0 } }
                                        button { r#type: "button", "aria-label": "Remove preview", onclick: move |_| on_remove.call(i), Icon { name: "x", size: 12.0 } }
                                    }
                                }
                            },
                        }
                    }
                }
            }
            div { class: "oxt-ubar", span { class: "oxt-sp" } button { class: "oxt-usend", r#type: "button", "aria-label": "Send message", onclick: move |_| on_send.call(()), Icon { name: "up", size: 14.0 } } }
        }
    }
}

/// The tool-kind map as a legend: each raw name, its kind, icon and verbs (for docs and Studio).
#[component]
pub fn ToolKindLegend(
    /// Raw names to show.
    names: Vec<String>,
) -> Element {
    let m = ToolKindMap::standard();
    let p = VerbPack::neutral();
    rsx! {
        div { class: "oxt-legend-w", tabindex: "0", role: "region", "aria-label": "Tool-kind map",
        table { class: "oxt-tbl oxt-legend",
            thead { tr { th { "Tool name" } th { "Kind" } th { "Running" } th { "Done" } } }
            tbody {
                for n in names.iter() {
                    {
                        let i = m.info(n, p);
                        rsx! { tr { td { span { class: "oxt-mono", Icon { name: i.icon, size: 12.0 } " {n}" } } td { "{i.kind:?}" if let Some(s) = i.server.clone() { span { class: "oxr-dim", " · {s}" } } } td { "{i.present}" } td { "{i.past}" } } }
                    }
                }
            }
        }
    }
    }
}
