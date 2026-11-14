//! Composer parts (plan #27, chunk 3): `VoiceRecorder`, `DictationButton`, `MentionInput`,
//! `ModeSwitch`, `PermissionSelector`, `ToolsMenu`, `QueueStack` and `AgentSteering`.
//!
//! Logic lives in `oxyn_core::composer::parts`. Time comes from the host: pass `now_ms` (the
//! host clock in milliseconds) and the parts read their timers from it, so the same markup renders
//! on the server and plays live in the browser. Styles are in [`PARTS_CSS`]. Behaviour follows
//! Arc UI (Voice recorder, Dictation button, Mention input) and affectUI (Agent Steering), rebuilt
//! from their docs; no source used.

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    composer::parts::{
        Applied, Choice, DictPhase, Mode, ModeSwitch as ModeModel, Permissions, Preset, Queue,
        RecPhase, Recorder, Risk, Steer, SteerPhase, Suggestion, Tool, Trigger, accept,
        active_token, default_modes, default_presets, insert_dictation, interim, levels, mmss,
        pills, rank, toggle_group, toggle_tool, tool_groups,
    },
};

use super::icons::Icon;

/// Stylesheet for the composer parts.
pub const PARTS_CSS: &str = include_str!("parts.css");

fn cls(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

// ─────────────────────────────────────── Voice recorder ──

/// Voice recorder: mic → permission → recording (live level bars, timer, pause) → review
/// (play, discard, send). Stops itself at `max_s`; the last ten seconds count down.
#[component]
pub fn VoiceRecorder(
    /// Host clock (ms).
    now_ms: u64,
    /// Limit in seconds.
    #[props(default = 120)]
    max_s: u64,
    /// Starting phase (`idle` · `recording` · `paused` · `review` · `denied`), for demos.
    #[props(default = "idle".to_string())]
    phase: String,
    /// The permission prompt answers no.
    #[props(default)]
    deny: bool,
    /// Bars in the meter.
    #[props(default = 28)]
    bars: usize,
    /// Variant: `inline` (inside a composer row) or `card`.
    #[props(default = "inline".to_string())]
    variant: String,
    /// A recording was sent (its length in ms).
    #[props(default)]
    on_send: EventHandler<u64>,
) -> Element {
    let now = ClockMs(now_ms);
    let mut m = use_signal(|| {
        let mut r = Recorder::new(Duration::from_secs(max_s));
        match RecPhase::from_id(&phase) {
            RecPhase::Recording => {
                r.request();
                r.permission(true, ClockMs(now_ms.saturating_sub(4_200)));
            }
            RecPhase::Paused => {
                r.request();
                r.permission(true, ClockMs(now_ms.saturating_sub(12_600)));
                r.toggle_pause(ClockMs(now_ms));
            }
            RecPhase::Review => {
                r.request();
                r.permission(true, ClockMs(0));
                r.stop(ClockMs(18_400));
            }
            RecPhase::Denied => {
                r.request();
                r.permission(false, ClockMs(0));
            }
            _ => {}
        }
        r
    });
    let mut live = use_signal(String::new);
    let mut sent = use_signal(|| None::<u64>);
    if m.peek().phase == RecPhase::Recording && m.peek().elapsed(now) >= m.peek().max {
        // Reached the limit: the next render shows review (writes are deferred to an event).
    }
    let r = m.read().clone();
    let el = r.elapsed(now);
    let at_limit = r.phase == RecPhase::Recording && el >= r.max;
    let shown_phase = if at_limit { RecPhase::Review } else { r.phase };
    let lv = levels(
        if shown_phase == RecPhase::Recording {
            now_ms
        } else {
            0
        },
        bars,
        if shown_phase == RecPhase::Recording {
            1.0
        } else {
            0.35
        },
    );
    let near = r.near_limit(now) && !at_limit;
    let timer = if near {
        format!("−{}", mmss(r.max - el))
    } else {
        mmss(el)
    };
    rsx! {
        div { class: cls(&["oxp-rec", if variant == "card" { "card" } else { "" }, shown_phase.id()]), role: "group", "aria-label": "Voice message",
            div { class: "oxp-sr", role: "status", "aria-live": "polite", "{live}" }
            match shown_phase {
                RecPhase::Idle | RecPhase::Denied => rsx! {
                    button { class: "p-mic", r#type: "button", "aria-label": "Record a voice message",
                        onclick: move |_| { m.write().request(); live.set("Waiting for microphone permission".into()); },
                        Icon { name: "mic", size: 16.0 }
                    }
                    if shown_phase == RecPhase::Denied {
                        div { class: "p-denied", role: "alert",
                            Icon { name: "ban", size: 13.0 }
                            span { "Microphone blocked. Allow it in your browser’s site settings, then try again." }
                        }
                    } else if let Some(ms) = sent() {
                        span { class: "p-hint", Icon { name: "check", size: 12.0, stroke: 2.0 } "Sent a {mmss(ms)} voice message" }
                    } else {
                        span { class: "p-hint", "Hold a thought? Record up to {mmss(max_s * 1000)}." }
                    }
                },
                RecPhase::Requesting => rsx! {
                    span { class: "p-req", span { class: "p-spin" } "Allow microphone access…" }
                    button { class: "oxyn-btn primary xs", r#type: "button", onclick: move |_| { m.write().permission(!deny, now); live.set(if deny { "Microphone blocked".into() } else { "Recording".into() }) }, if deny { "Answer prompt (Block)" } else { "Answer prompt (Allow)" } }
                    button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| m.write().discard(), "Cancel" }
                },
                RecPhase::Recording | RecPhase::Paused => rsx! {
                    button { class: "p-ic danger", r#type: "button", "aria-label": "Discard recording", onclick: move |_| { m.write().discard(); live.set("Recording discarded".into()) }, Icon { name: "trash", size: 14.0 } }
                    span { class: cls(&["p-dot", if shown_phase == RecPhase::Paused { "held" } else { "" }]), "aria-hidden": "true" }
                    span { class: cls(&["p-time", if near { "near" } else { "" }]), "aria-label": "Recorded {mmss(el)}", "{timer}" }
                    div { class: "p-bars", "aria-hidden": "true",
                        for (i, h) in lv.iter().enumerate() { i { key: "{i}", style: "--h:{h}" } }
                    }
                    button { class: "p-ic", r#type: "button", "aria-label": if shown_phase == RecPhase::Paused { "Resume" } else { "Pause" }, "aria-pressed": "{shown_phase == RecPhase::Paused}",
                        onclick: move |_| { m.write().toggle_pause(now); let p = m.peek().phase == RecPhase::Paused; live.set(if p { "Paused".into() } else { "Recording".into() }) },
                        Icon { name: if shown_phase == RecPhase::Paused { "play" } else { "pause" }, size: 14.0 }
                    }
                    button { class: "p-ic solid", r#type: "button", "aria-label": "Stop and review", onclick: move |_| { m.write().stop(now); live.set("Stopped. Review your recording".into()) }, Icon { name: "stop", size: 13.0 } }
                },
                RecPhase::Review => rsx! {
                    button { class: "p-ic", r#type: "button", "aria-label": "Discard recording", onclick: move |_| { m.write().discard(); live.set("Recording discarded".into()) }, Icon { name: "trash", size: 14.0 } }
                    span { class: "p-play", Icon { name: "play", size: 12.0 } }
                    div { class: "p-bars review", "aria-hidden": "true",
                        for (i, h) in levels(7_777, bars, 0.8).iter().enumerate() { i { key: "{i}", style: "--h:{h}" } }
                    }
                    span { class: "p-time", {mmss(r.elapsed(now).max(if at_limit { r.max } else { 0 }))} }
                    button { class: "p-ic solid", r#type: "button", "aria-label": "Send voice message",
                        onclick: move |_| { let mut w = m.write(); if w.phase == RecPhase::Recording { w.stop(now) } let ms = w.banked; w.discard(); drop(w); sent.set(Some(ms)); live.set(format!("Sent a {} voice message", mmss(ms))); on_send.call(ms) },
                        Icon { name: "up", size: 14.0, stroke: 2.0 }
                    }
                },
            }
        }
    }
}

// ─────────────────────────────────────── Dictation button ──

/// Dictation: a mic inside a text field. Listening streams interim words in muted text; Stop (or
/// Enter) commits them at the caret; Escape cancels; a failure keeps what was typed.
#[component]
pub fn DictationButton(
    /// Host clock (ms).
    now_ms: u64,
    /// What the "speaker" says (demo script).
    #[props(default = "add a thirty second leeway to the token expiry check and rerun the auth tests".to_string())]
    script: String,
    /// Words per minute.
    #[props(default = 150.0)]
    wpm: f32,
    /// Initial text.
    #[props(default)]
    value: String,
    /// Recognition fails after a second.
    #[props(default)]
    fail: bool,
    /// Placeholder.
    #[props(default = "Type or dictate…".to_string())]
    placeholder: String,
    /// Variant: `field` or `pill` (the mic as a round button beside the field).
    #[props(default = "field".to_string())]
    variant: String,
    /// Text changed.
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let mut text = use_signal(|| value.clone());
    let mut phase = use_signal(|| DictPhase::Idle);
    let mut since = use_signal(|| 0u64);
    let mut live = use_signal(String::new);
    let ph = phase();
    let heard = if ph == DictPhase::Listening {
        interim(
            &script,
            Duration::from_millis(now_ms.saturating_sub(since())),
            wpm,
        )
    } else {
        String::new()
    };
    let failed_now = ph == DictPhase::Listening && fail && now_ms.saturating_sub(since()) > 1_200;
    let ph = if failed_now { DictPhase::Error } else { ph };
    let commit = {
        let script = script.clone();
        move |now: u64| {
            let words = interim(
                &script,
                Duration::from_millis(now.saturating_sub(*since.peek())),
                wpm,
            );
            if !words.is_empty() {
                let t = text.peek().clone();
                let (n, _) = insert_dictation(&t, t.len(), &words);
                text.set(n.clone());
                on_change.call(n);
            }
            phase.set(DictPhase::Idle);
            live.set(if words.is_empty() {
                "Nothing heard".into()
            } else {
                "Dictation inserted".into()
            });
        }
    };
    let mut commit2 = commit.clone();
    let mut commit3 = commit.clone();
    rsx! {
        div { class: cls(&["oxp-dict", if variant == "pill" { "pill" } else { "" }, match ph { DictPhase::Listening => "listening", DictPhase::Error => "error", _ => "" }]),
            div { class: "oxp-sr", role: "status", "aria-live": "polite", "{live}" }
            div { class: "d-field",
                if ph == DictPhase::Listening {
                    div { class: "d-in d-ghost", "aria-hidden": "true",
                        span { "{text}" }
                        if !text().is_empty() && !heard.is_empty() { " " }
                        span { class: "d-interim", "{heard}" }
                        span { class: "d-caret" }
                    }
                } else {
                    textarea { class: "d-in", rows: "2", placeholder: "{placeholder}", value: "{text}", "aria-label": "Message",
                        oninput: move |e| { text.set(e.value()); on_change.call(e.value()) },
                    }
                }
                button { class: cls(&["d-mic", if ph == DictPhase::Listening { "on" } else { "" }]), r#type: "button",
                    "aria-label": if ph == DictPhase::Listening { "Stop dictation" } else { "Dictate" }, "aria-pressed": "{ph == DictPhase::Listening}",
                    onclick: move |_| { if *phase.peek() == DictPhase::Listening { commit2(now_ms) } else { since.set(now_ms); phase.set(DictPhase::Listening); live.set("Listening".into()) } },
                    onkeydown: move |e: KeyboardEvent| { if *phase.peek() == DictPhase::Listening { if e.key() == Key::Escape { e.prevent_default(); phase.set(DictPhase::Idle); live.set("Dictation cancelled".into()) } else if e.key() == Key::Enter { e.prevent_default(); commit3(now_ms) } } },
                    if ph == DictPhase::Listening { span { class: "d-wave", i {} i {} i {} } } else { Icon { name: "mic", size: 15.0 } }
                }
            }
            match ph {
                DictPhase::Listening => rsx! { div { class: "d-note", "Listening… press the mic or Enter to insert, Escape to cancel" } },
                DictPhase::Error => rsx! { div { class: "d-note err", role: "alert", Icon { name: "alert", size: 12.0 } "Couldn’t hear that. Your text is kept. " button { class: "d-retry", r#type: "button", onclick: move |_| { since.set(now_ms); phase.set(DictPhase::Listening) }, "Try again" } } },
                _ => rsx! {},
            }
        }
    }
}

// ─────────────────────────────────────── Mention input ──

/// Mention and slash input: `@` opens people, files and agents; `/` at the start of a line opens
/// commands. Arrow keys move, Enter or Tab inserts a pill, Escape closes.
#[component]
pub fn MentionInput(
    /// `@` suggestions.
    suggestions: Vec<Suggestion>,
    /// `/` commands.
    #[props(default)]
    commands: Vec<Suggestion>,
    /// Initial text.
    #[props(default)]
    value: String,
    /// Max rows in the list.
    #[props(default = 6)]
    limit: usize,
    /// Placeholder.
    #[props(default = "Message, @ to mention, / for commands".to_string())]
    placeholder: String,
    /// Opens upward (composer at the bottom).
    #[props(default = true)]
    up: bool,
    /// Submitted text.
    #[props(default)]
    on_submit: EventHandler<String>,
) -> Element {
    let mut text = use_signal(|| value.clone());
    let mut hl = use_signal(|| 0usize);
    let mut closed = use_signal(|| false);
    let mut live = use_signal(String::new);
    let t = text();
    let tok = active_token(&t, t.len());
    let list: Vec<Suggestion> = match &tok {
        Some((Trigger::Mention, _, q)) => rank(&suggestions, q, limit),
        Some((Trigger::Slash, _, q)) => rank(&commands, q, limit),
        None => vec![],
    };
    let open = tok.is_some() && !closed();
    let n = list.len();
    let h = hl().min(n.saturating_sub(1));
    let known: Vec<String> = suggestions
        .iter()
        .chain(commands.iter())
        .map(|s| s.id.clone())
        .collect();
    let preview = pills(&t, &known);
    let kind = tok.as_ref().map(|x| x.0);
    let start = tok.as_ref().map(|x| x.1).unwrap_or(0);
    let pick = {
        let list = list.clone();
        move |i: usize| {
            if let (Some(k), Some(s)) = (kind, list.get(i)) {
                let cur = text.peek().clone();
                let (nt, _) = accept(&cur, cur.len(), k, start, &s.id);
                text.set(nt);
                hl.set(0);
                live.set(format!("Inserted {}", s.label));
            }
        }
    };
    let mut pick_k = pick.clone();
    let mut groups: Vec<(String, Vec<usize>)> = vec![];
    for (i, s) in list.iter().enumerate() {
        match groups.iter_mut().find(|(g, _)| *g == s.group) {
            Some((_, v)) => v.push(i),
            None => groups.push((s.group.clone(), vec![i])),
        }
    }
    let q_now = tok.as_ref().map(|x| x.2.clone()).unwrap_or_default();
    rsx! {
        div { class: cls(&["oxp-mention", if up { "up" } else { "" }]),
            div { class: "oxp-sr", role: "status", "aria-live": "polite", if open { "{n} suggestions" } else { "{live}" } }
            if open {
                div { class: "m-pop", id: "oxp-m-list", role: "listbox", "aria-label": if kind == Some(Trigger::Slash) { "Commands" } else { "Mentions" },
                    if n == 0 {
                        div { class: "m-empty", if kind == Some(Trigger::Slash) { "No command matches “/{q_now}”" } else { "No one matches “@{q_now}”" } }
                    }
                    for (g, ix) in groups.iter().cloned() {
                        div { key: "{g}", class: "m-grp", role: "presentation", "{g}" }
                        for i in ix {
                            {
                                let s = list[i].clone();
                                let mut pk = pick.clone();
                                rsx! {
                                    div { key: "{s.id}", id: "oxp-m-{i}", class: cls(&["m-row", if i == h { "is-hl" } else { "" }]), role: "option", "aria-selected": "{i == h}",
                                        onpointermove: move |_| if *hl.peek() != i { hl.set(i) },
                                        onmousedown: move |e| { e.prevent_default(); pk(i) },
                                        span { class: "m-av", if kind == Some(Trigger::Slash) { "/" } else { {s.label.chars().next().map(|c| c.to_string()).unwrap_or_default()} } }
                                        span { class: "m-l", "{s.label}" }
                                        if !s.hint.is_empty() { span { class: "m-h", "{s.hint}" } }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "m-box",
                textarea { class: "m-in", rows: "2", placeholder: "{placeholder}", value: "{t}", "aria-label": "Message",
                    role: "combobox", "aria-expanded": "{open}", "aria-controls": "oxp-m-list", "aria-autocomplete": "list",
                    "aria-activedescendant": if open && n > 0 { "oxp-m-{h}" } else { "" },
                    oninput: move |e| { text.set(e.value()); closed.set(false); hl.set(0) },
                    onkeydown: move |e: KeyboardEvent| {
                        if open {
                            match e.key() {
                                Key::ArrowDown => { e.prevent_default(); if n > 0 { hl.set((h + 1) % n) } }
                                Key::ArrowUp => { e.prevent_default(); if n > 0 { hl.set((h + n - 1) % n) } }
                                Key::Enter | Key::Tab if n > 0 => { e.prevent_default(); pick_k(h) }
                                Key::Escape => { e.prevent_default(); closed.set(true) }
                                _ => {}
                            }
                        } else if e.key() == Key::Enter && !e.modifiers().shift() {
                            e.prevent_default();
                            let v = text.peek().trim().to_string();
                            if !v.is_empty() { on_submit.call(v.clone()); live.set(format!("Sent: {v}")); text.set(String::new()) }
                        }
                    },
                }
                if preview.iter().any(|(p, _)| *p) {
                    div { class: "m-prev", "aria-hidden": "true",
                        for (i, (p, s)) in preview.iter().enumerate() {
                            if *p { span { key: "{i}", class: "m-pill", "{s}" } } else { span { key: "{i}", "{s}" } }
                        }
                    }
                }
            }
        }
    }
}

// ───────────────────────────────────────── Mode switch ──

/// Ask · Plan · Agent switch. Shift+Tab cycles. While the agent is busy, a more restrictive mode
/// applies now and a looser one waits for the next turn (shown as pending).
#[component]
pub fn ModeSwitch(
    /// Modes (default Ask · Plan · Agent).
    #[props(default = default_modes())]
    modes: Vec<Mode>,
    /// Initial mode id.
    #[props(default = "agent".to_string())]
    value: String,
    /// The agent is running.
    #[props(default)]
    busy: bool,
    /// Variant: `segmented` or `menu` (a compact chip with a list).
    #[props(default = "segmented".to_string())]
    variant: String,
    /// Show the composer preview (placeholder follows the mode).
    #[props(default = true)]
    preview: bool,
    /// Changed: (mode id, applied now).
    #[props(default)]
    on_change: EventHandler<(String, bool)>,
) -> Element {
    let mut m = use_signal(|| ModeModel::new(modes.clone(), &value));
    let mut open = use_signal(|| false);
    let mut live = use_signal(String::new);
    let s = m.read().clone();
    let cur = s.mode().clone();
    let mut choose = move |i: usize| {
        let r = m.write().select(i, busy);
        let md = m.peek().modes[i].clone();
        match r {
            Some(Applied::Now) => {
                live.set(format!("{} mode", md.label));
                on_change.call((md.id, true))
            }
            Some(Applied::NextTurn) => {
                live.set(format!("{} mode after this turn", md.label));
                on_change.call((md.id, false))
            }
            None => {}
        }
        open.set(false);
    };
    rsx! {
        div { class: cls(&["oxp-mode", if variant == "menu" { "menu" } else { "" }, "m-{cur.id}"]),
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Tab && e.modifiers().shift() {
                e.prevent_default();
                let r = m.write().cycle(busy);
                let st = m.peek().clone();
                let md = st.pending.map(|p| st.modes[p].clone()).unwrap_or_else(|| st.mode().clone());
                if let Some(a) = r { live.set(if a == Applied::Now { format!("{} mode", md.label) } else { format!("{} mode after this turn", md.label) }) }
            },
            div { class: "oxp-sr", role: "status", "aria-live": "polite", "{live}" }
            if variant == "menu" {
                div { class: "mo-wrap",
                    button { class: "mo-chip", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{open()}", onclick: move |_| { let o = !open(); open.set(o) },
                        span { class: "mo-dot" } "{cur.label}"
                        if let Some(p) = s.pending { span { class: "mo-next", "→ {s.modes[p].label}" } }
                        Icon { name: "chevron", size: 11.0, stroke: 2.0 }
                    }
                    if open() {
                        div { class: "mo-list", role: "listbox", "aria-label": "Mode",
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.prevent_default(); open.set(false) },
                            for (i, md) in s.modes.iter().cloned().enumerate() {
                                button { key: "{md.id}", class: cls(&["mo-opt", if i == s.current { "on" } else { "" }]), r#type: "button", role: "option", "aria-selected": "{i == s.current}", onclick: move |_| choose(i),
                                    span { class: "mo-l", "{md.label}" if s.pending == Some(i) { span { class: "mo-badge", "Next turn" } } }
                                    span { class: "mo-d", "{md.desc}" }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "mo-seg", role: "radiogroup", "aria-label": "Mode",
                    for (i, md) in s.modes.iter().cloned().enumerate() {
                        button { key: "{md.id}", class: cls(&["mo-b", if i == s.current { "on" } else { "" }, if s.pending == Some(i) { "pending" } else { "" }]), r#type: "button", role: "radio", "aria-checked": "{i == s.current}", title: "{md.desc}",
                            onclick: move |_| choose(i),
                            "{md.label}"
                            if s.pending == Some(i) { span { class: "mo-badge", "next" } }
                        }
                    }
                }
            }
            if s.pending.is_some() {
                div { class: "mo-note",
                    "Switches when this turn ends. "
                    button { class: "mo-link", r#type: "button", onclick: move |_| { m.write().turn_end(); let l = m.peek().mode().label.clone(); live.set(format!("{l} mode")) }, "End turn now" }
                }
            }
            if preview {
                div { class: "mo-prev", span { "{cur.placeholder}" } span { class: "mo-hint", "⇧⇥ to switch" } }
            }
        }
    }
}

// ─────────────────────────────────── Permission selector ──

/// Permission presets with a risk mark. Full access asks for a confirm; presets above the
/// workspace `ceiling` are locked with the policy reason.
#[component]
pub fn PermissionSelector(
    /// Presets (default Read only · Ask before edits · Auto-edit · Full access).
    #[props(default = default_presets())]
    presets: Vec<Preset>,
    /// Initial id.
    #[props(default = "ask".to_string())]
    value: String,
    /// Highest risk allowed (`low` · `medium` · `high`).
    #[props(default = "high".to_string())]
    ceiling: String,
    /// Variant: `menu` (chip + list) or `cards`.
    #[props(default = "menu".to_string())]
    variant: String,
    /// Start open (menu).
    #[props(default)]
    open: bool,
    /// Changed.
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let mut p = use_signal(|| {
        let mut x = Permissions::new(presets.clone(), &value);
        x.ceiling = match ceiling.as_str() {
            "low" => Risk::Low,
            "medium" => Risk::Medium,
            _ => Risk::High,
        };
        x
    });
    let mut is_open = use_signal(|| open);
    let mut live = use_signal(String::new);
    let st = p.read().clone();
    let cur = st.preset().cloned();
    let choose = move |id: String| {
        let r = p.write().choose(&id);
        match r {
            Some(Choice::Applied(id)) => {
                live.set(format!("Permissions: {id}"));
                on_change.call(id)
            }
            Some(Choice::Confirm(_)) => live.set("Confirm full access".into()),
            Some(Choice::Blocked(_)) => live.set("Blocked by workspace policy".into()),
            None => {}
        }
    };
    let list = rsx! {
        div { class: "pe-list", role: "radiogroup", "aria-label": "Permissions",
            for pr in st.presets.iter().cloned() {
                {
                    let locked = pr.risk > st.ceiling;
                    let on = pr.id == st.current;
                    let id = pr.id.clone();
                    let mut ch = choose;
                    rsx! {
                        button { key: "{pr.id}", class: cls(&["pe-opt", if on { "on" } else { "" }, if locked { "locked" } else { "" }, pr.risk.id()]), r#type: "button", role: "radio", "aria-checked": "{on}", "aria-disabled": "{locked}",
                            onclick: move |_| ch(id.clone()),
                            span { class: "pe-risk", "aria-hidden": "true", Icon { name: if locked { "lock" } else if pr.risk == Risk::High { "alert" } else if pr.risk == Risk::Low { "eye" } else { "pencil" }, size: 13.0 } }
                            span { class: "pe-tx",
                                span { class: "pe-l", "{pr.label}" span { class: "pe-rk", " · {pr.risk.id()} risk" } }
                                span { class: "pe-d", if locked { "Turned off by your workspace admin." } else { "{pr.desc}" } }
                            }
                            if on { Icon { name: "check", size: 13.0, stroke: 2.0 } }
                        }
                    }
                }
            }
        }
    };
    rsx! {
        div { class: cls(&["oxp-perm", if variant == "cards" { "cards" } else { "" }]),
            div { class: "oxp-sr", role: "status", "aria-live": "polite", "{live}" }
            if variant == "cards" {
                {list}
            } else {
                button { class: cls(&["pe-chip", cur.as_ref().map(|c| c.risk.id()).unwrap_or("")]), r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{is_open()}",
                    onclick: move |_| { let o = !is_open(); is_open.set(o) },
                    Icon { name: "lock", size: 12.0 }
                    "{cur.as_ref().map(|c| c.label.clone()).unwrap_or_default()}"
                    Icon { name: "chevron", size: 11.0, stroke: 2.0 }
                }
                if is_open() {
                    div { class: "pe-pop", role: "dialog", "aria-label": "Permissions",
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.prevent_default(); is_open.set(false) },
                        {list}
                    }
                }
            }
            if let Some(c) = st.confirming.clone() {
                div { class: "pe-confirm", role: "alertdialog", "aria-label": "Confirm full access",
                    Icon { name: "alert", size: 14.0 }
                    div { class: "pe-ct",
                        strong { "Give the agent full access?" }
                        span { "It can run commands, push and spend without asking. You can switch back at any time." }
                    }
                    div { class: "pe-ca",
                        button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| { p.write().confirm(false); live.set("Kept the current permissions".into()) }, "Cancel" }
                        button { class: "oxyn-btn destructive xs", r#type: "button", onclick: move |_| { if let Some(id) = p.write().confirm(true) { live.set("Full access on".into()); on_change.call(id) } }, "Allow {c}" }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────── Tools menu ──

/// Tools menu: a chip with the enabled count opens a searchable list of tools grouped by source,
/// with per-tool and per-group switches. Tools that need sign-in show Connect instead.
#[component]
pub fn ToolsMenu(
    /// Tools.
    tools: Vec<Tool>,
    /// Start open.
    #[props(default = true)]
    open: bool,
    /// Group switches.
    #[props(default = true)]
    group_toggles: bool,
    /// Search field.
    #[props(default = true)]
    search: bool,
    /// Changed: enabled ids.
    #[props(default)]
    on_change: EventHandler<Vec<String>>,
    /// Connect pressed for a tool that needs sign-in.
    #[props(default)]
    on_connect: EventHandler<String>,
) -> Element {
    let mut ts = use_signal(|| tools.clone());
    let mut is_open = use_signal(|| open);
    let mut q = use_signal(String::new);
    let mut live = use_signal(String::new);
    let all = ts.read().clone();
    let on_n = all.iter().filter(|t| t.on).count();
    let groups = tool_groups(&all, &q());
    let emit = move |ts: &Vec<Tool>| {
        on_change.call(ts.iter().filter(|t| t.on).map(|t| t.id.clone()).collect())
    };
    rsx! {
        div { class: "oxp-tools",
            div { class: "oxp-sr", role: "status", "aria-live": "polite", "{live}" }
            button { class: "tl-chip", r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{is_open()}", onclick: move |_| { let o = !is_open(); is_open.set(o) },
                Icon { name: "tool", size: 13.0 } "Tools" span { class: "tl-n", "{on_n}" }
            }
            if is_open() {
                div { class: "tl-pop", role: "dialog", "aria-label": "Tools",
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.prevent_default(); is_open.set(false) },
                    if search {
                        div { class: "tl-search", Icon { name: "search", size: 13.0 }
                            input { r#type: "text", placeholder: "Search tools…", value: "{q}", "aria-label": "Search tools", oninput: move |e| q.set(e.value()) }
                        }
                    }
                    if groups.is_empty() { div { class: "tl-empty", "No tools match “{q}”" } }
                    for (g, ix) in groups.iter().cloned() {
                        div { key: "{g}", class: "tl-grp", role: "group", "aria-label": "{g}",
                            div { class: "tl-gh",
                                span { "{g}" }
                                if group_toggles {
                                    {
                                        let gg = g.clone();
                                        let allon = ix.iter().all(|&i| all[i].on || all[i].needs_auth);
                                        rsx! { button { class: "tl-gt", r#type: "button", onclick: move |_| { toggle_group(&mut ts.write(), &gg); emit(&ts.peek()); live.set(format!("{gg} {}", if allon { "off" } else { "on" })) }, if allon { "Turn all off" } else { "Turn all on" } } }
                                    }
                                }
                            }
                            for i in ix {
                                {
                                    let t = all[i].clone();
                                    rsx! {
                                        div { key: "{t.id}", class: cls(&["tl-row", if t.needs_auth { "auth" } else { "" }]),
                                            span { class: "tl-l", "{t.label}" }
                                            if t.needs_auth {
                                                button { class: "oxyn-btn secondary xs", r#type: "button", onclick: { let id = t.id.clone(); move |_| on_connect.call(id.clone()) }, Icon { name: "plug", size: 12.0 } "Connect" }
                                            } else {
                                                button { class: cls(&["p-sw", if t.on { "on" } else { "" }]), r#type: "button", role: "switch", "aria-checked": "{t.on}", "aria-label": "{t.label}",
                                                    onclick: move |_| { if toggle_tool(&mut ts.write(), i) { let tt = ts.peek()[i].clone(); live.set(format!("{} {}", tt.label, if tt.on { "on" } else { "off" })); emit(&ts.peek()) } },
                                                    span { class: "p-thumb" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────── Queue stack ──

/// Queue stack: follow-ups typed while the agent runs wait above the composer. One sends at the
/// end of each turn (`turn_ms`) unless paused; items can be edited, reordered (Alt+↑/↓), removed or
/// sent now.
#[component]
pub fn QueueStack(
    /// Queued texts.
    items: Vec<String>,
    /// The agent is running (turns end every `turn_ms`).
    #[props(default = true)]
    busy: bool,
    /// Turn length (ms).
    #[props(default = 6000)]
    turn_ms: u64,
    /// Start paused.
    #[props(default)]
    paused: bool,
    /// Collapse beyond this many rows (0 = never).
    #[props(default = 3)]
    max_rows: usize,
    /// An item was sent (`true` = interrupting now).
    #[props(default)]
    on_send: EventHandler<(String, bool)>,
) -> Element {
    let mut q = use_signal(|| {
        let v: Vec<&str> = items.iter().map(|s| s.as_str()).collect();
        let mut x = Queue::new(&v);
        x.paused = paused;
        x
    });
    let mut draft = use_signal(String::new);
    let mut edit_text = use_signal(String::new);
    let mut expanded = use_signal(|| false);
    let mut sent = use_signal(Vec::<String>::new);
    let mut live = use_signal(String::new);
    let st = q.read().clone();
    let n = st.items.len();
    let shown = if max_rows == 0 || expanded() {
        n
    } else {
        n.min(max_rows)
    };
    rsx! {
        div { class: "oxp-queue",
            div { class: "oxp-sr", role: "status", "aria-live": "polite", "{live}" }
            if !sent().is_empty() {
                div { class: "qu-sent", for (i, s) in sent().iter().enumerate() { div { key: "{i}", class: "qu-msg", "{s}" } } }
            }
            div { class: "qu-run", span { class: cls(&["qu-dot", if busy { "busy" } else { "" }]) } if busy { "Agent is working · turn ends in a moment" } else { "Agent is idle" } }
            if busy && !st.paused && n > 0 {
                span { key: "{n}-{sent().len()}", class: "oxyn-timer", style: "animation-duration:{turn_ms}ms",
                    onanimationend: move |_| { if let Some(it) = q.write().turn_end() { live.set(format!("Sent: {}", it.text)); sent.write().push(it.text.clone()); on_send.call((it.text, false)) } }
                }
            }
            if n > 0 {
                div { class: "qu-stack",
                    div { class: "qu-h",
                        span { "Queued · {n}" }
                        button { class: "qu-link", r#type: "button", "aria-pressed": "{st.paused}", onclick: move |_| { let p = !q.peek().paused; q.write().paused = p; live.set(if p { "Queue paused".into() } else { "Queue resumed".into() }) },
                            Icon { name: if st.paused { "play" } else { "pause" }, size: 11.0 } if st.paused { "Resume" } else { "Pause" }
                        }
                    }
                    ol { class: "qu-list", "aria-label": "Queued messages",
                        for (i, it) in st.items.iter().take(shown).cloned().enumerate() {
                            li { key: "{it.id}", class: cls(&["qu-row", if i == 0 && !st.paused { "next" } else { "" }]), tabindex: "0",
                                onkeydown: { let id = it.id.clone(); move |e: KeyboardEvent| if e.modifiers().alt() { match e.key() { Key::ArrowUp => { e.prevent_default(); q.write().shift(&id, -1) } Key::ArrowDown => { e.prevent_default(); q.write().shift(&id, 1) } _ => {} } } },
                                span { class: "qu-i", "{i + 1}" }
                                if st.editing.as_deref() == Some(it.id.as_str()) {
                                    input { class: "qu-ed", r#type: "text", value: "{edit_text}", "aria-label": "Edit queued message",
                                        oninput: move |e| edit_text.set(e.value()),
                                        onkeydown: { let id = it.id.clone(); move |e: KeyboardEvent| { if e.key() == Key::Enter { e.prevent_default(); let t = edit_text.peek().clone(); q.write().edit(&id, &t) } else if e.key() == Key::Escape { e.prevent_default(); q.write().editing = None } } },
                                    }
                                } else {
                                    span { class: "qu-t", "{it.text}" }
                                }
                                span { class: "qu-act",
                                    button { class: "p-ic sm", r#type: "button", "aria-label": "Edit", onclick: { let id = it.id.clone(); let tx = it.text.clone(); move |_| { edit_text.set(tx.clone()); q.write().editing = Some(id.clone()) } }, Icon { name: "pencil", size: 12.0 } }
                                    button { class: "p-ic sm", r#type: "button", "aria-label": "Move up", disabled: i == 0, onclick: { let id = it.id.clone(); move |_| q.write().shift(&id, -1) }, Icon { name: "up", size: 12.0 } }
                                    button { class: "p-ic sm", r#type: "button", "aria-label": "Send now (interrupts)", onclick: { let id = it.id.clone(); move |_| { if let Some(x) = q.write().send_now(&id) { live.set(format!("Interrupted with: {}", x.text)); sent.write().push(x.text.clone()); on_send.call((x.text, true)) } } }, Icon { name: "bolt", size: 12.0 } }
                                    button { class: "p-ic sm", r#type: "button", "aria-label": "Remove", onclick: { let id = it.id.clone(); move |_| { q.write().remove(&id); live.set("Removed from queue".into()) } }, Icon { name: "x", size: 12.0 } }
                                }
                            }
                        }
                    }
                    if shown < n { button { class: "qu-more", r#type: "button", onclick: move |_| expanded.set(true), "+{n - shown} more" } }
                }
            }
            div { class: "qu-comp",
                input { class: "qu-in", r#type: "text", placeholder: if busy { "Queue a follow-up…" } else { "Message…" }, value: "{draft}", "aria-label": "Message",
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); let d = draft.peek().trim().to_string(); if !d.is_empty() { if busy { q.write().push(&d); live.set("Queued".into()) } else { sent.write().push(d.clone()); on_send.call((d, false)) } draft.set(String::new()) } },
                }
                span { class: "qu-kbd", if busy { "↵ queue" } else { "↵ send" } }
            }
        }
    }
}

// ─────────────────────────────────────────── Agent steering ──

/// Agent steering (affectUI): steer a running agent without stopping it. Notes wait for the next
/// step boundary (Pending → Delivered → Applied); Interrupt delivers at once and restarts the
/// current step.
#[component]
pub fn AgentSteering(
    /// Host clock (ms).
    now_ms: u64,
    /// Step length (ms).
    #[props(default = 3000)]
    step_ms: u64,
    /// Steps the agent works through (loops).
    #[props(default = vec!["Reading auth/token.rs".to_string(), "Running the test suite".to_string(), "Tracing the clock skew".to_string(), "Drafting the fix".to_string()])]
    steps: Vec<String>,
    /// Pre-filled steering notes (text, sent `ms` ago), for demos.
    #[props(default)]
    notes: Vec<(String, u64)>,
    /// Interrupt by default.
    #[props(default)]
    interrupt: bool,
    /// A note was sent: (text, interrupt).
    #[props(default)]
    on_steer: EventHandler<(String, bool)>,
) -> Element {
    let mut list = use_signal(|| {
        notes
            .iter()
            .map(|(t, ago)| Steer {
                text: t.clone(),
                at: ClockMs(now_ms.saturating_sub(*ago)),
                interrupt: false,
            })
            .collect::<Vec<_>>()
    });
    let mut draft = use_signal(String::new);
    let mut intr = use_signal(|| interrupt);
    let mut stopped = use_signal(|| false);
    let mut live = use_signal(String::new);
    let step = Duration::from_millis(step_ms.max(500));
    let k = (now_ms / step_ms.max(500)) as usize;
    let cur = steps
        .get(k % steps.len().max(1))
        .cloned()
        .unwrap_or_default();
    let prog = (now_ms % step_ms.max(500)) as f32 / step_ms.max(500) as f32;
    let notes_now = list.read().clone();
    rsx! {
        div { class: "oxp-steer",
            div { class: "oxp-sr", role: "status", "aria-live": "polite", "{live}" }
            div { class: "st-run",
                span { class: cls(&["st-dot", if stopped() { "stop" } else { "" }]), "aria-hidden": "true" }
                div { class: "st-tx",
                    span { class: "st-l", if stopped() { "Stopped" } else { "{cur}" } }
                    span { class: "st-m", if stopped() { "Send a message to continue" } else { "Step {k % steps.len().max(1) + 1} of {steps.len()} · notes land at the next step" } }
                }
                button { class: "oxyn-btn secondary xs", r#type: "button", onclick: move |_| { let s = !stopped(); stopped.set(s); live.set(if s { "Agent stopped".into() } else { "Agent resumed".into() }) }, Icon { name: if stopped() { "play" } else { "stop" }, size: 11.0 } if stopped() { "Resume" } else { "Stop" } }
            }
            if !stopped() { div { class: "st-bar", "aria-hidden": "true", i { style: "--p:{prog}" } } }
            if !notes_now.is_empty() {
                ul { class: "st-notes", "aria-label": "Steering notes",
                    for (i, n) in notes_now.iter().enumerate() {
                        {
                            let ph = n.phase(ClockMs(now_ms), step);
                            let (lab, c) = match ph { SteerPhase::Pending => ("Waiting for next step", "pending"), SteerPhase::Delivered => ("Read by agent", "delivered"), SteerPhase::Applied => ("Applied", "applied") };
                            rsx! {
                                li { key: "{i}", class: cls(&["st-note", c]),
                                    Icon { name: "steer", size: 13.0 }
                                    span { class: "st-nt", "{n.text}" }
                                    span { class: "st-ph", if n.interrupt { "Interrupt · " } "{lab}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "st-comp",
                input { class: "st-in", r#type: "text", placeholder: "Steer the agent…", value: "{draft}", "aria-label": "Steering note",
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter { e.prevent_default(); let d = draft.peek().trim().to_string(); if !d.is_empty() { let it = *intr.peek() || e.modifiers().meta() || e.modifiers().ctrl(); list.write().push(Steer { text: d.clone(), at: ClockMs(now_ms), interrupt: it }); stopped.set(false); live.set(if it { "Interrupting".into() } else { "Note queued for the next step".into() }); on_steer.call((d, it)); draft.set(String::new()) } },
                }
                button { class: cls(&["st-mode", if intr() { "on" } else { "" }]), r#type: "button", "aria-pressed": "{intr()}", title: "⌘↵ interrupts once", onclick: move |_| { let v = !intr(); intr.set(v) }, Icon { name: "bolt", size: 12.0 } "Interrupt" }
            }
        }
    }
}
