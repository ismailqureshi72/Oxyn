//! affectUI fidelity pass, chunk 2 (plan #32): exact rebuilds of affectUI's **Model Picker**
//! ([`ModelPickerMenu`]) and **Run Controls** ([`AgentRunPanel`]). Behaviour and look only;
//! models in `oxyn_core::parity::chatfid`. Styles: `chatfid.css` (classes `oxmq-*`, `oxrq-*`), in
//! `PRIMITIVES_CSS`. Both follow the nearest `data-cstyle` (Clean / Classic) unless `style` is set.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
        chatfid::{
            Phase, PickerModel, Piece, RunPermission, RunTurn, THINK_MS, WORD_MS, context_pct,
            demo_models, phase_at, took, tray_values, words_at,
        },
        compose::Effort,
    },
};

use crate::{brand::BrandIcon, composer::icons::Icon};

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

/// Stroke paths this file needs beyond the shared icon set.
fn path(name: &str) -> &'static str {
    match name {
        "bulb" => {
            "M9 18h6M10 21h4M12 3a6 6 0 0 0-3.6 10.8c.7.6 1.1 1.3 1.1 2.2h5c0-.9.4-1.6 1.1-2.2A6 6 0 0 0 12 3z"
        }
        "shield" => "M12 3.5 5 6.2v5.3c0 4.3 2.9 7.9 7 9 4.1-1.1 7-4.7 7-9V6.2z",
        "edit" => "M4 20h4L19 9l-4-4L4 16zM13.5 6.5l4 4",
        "queue" => "M4 7h16M4 12h16M4 17h10",
        "return" => "M9 10 5 14l4 4M5 14h9a5 5 0 0 0 0-10h-2",
        _ => "",
    }
}

#[component]
fn Stroke(name: String, #[props(default = 16.0)] size: f32) -> Element {
    rsx! {
        svg { class: "oxmq-g", width: "{size}", height: "{size}", view_box: "0 0 24 24", fill: "none", stroke: "currentColor",
            stroke_width: "1.7", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "{path(&name)}" }
        }
    }
}

// ─────────────────────────────────────────────────────────────── Model Picker ──

/// Model Picker (affectUI): the model menu of an AI chat, opened from a pill in a small composer.
/// Point at a model and its speed, quality and cost dashes fill blue while the tray rolls its
/// first-token time and price; switch on **Extended thinking** and the Low / Medium / High effort
/// control unfolds (the time multiplies ×1.6 / ×2.5 / ×4); **Web search** swaps the tray for a
/// note. Rows show letter tiles as affectUI does, or coloured brand marks with `marks: "brands"`.
/// Clean: uppercase headers, blue dashes, rounded-square send; Classic: title-case headers,
/// monochrome dot-dashes, a round send and the tray on its own shelf.
#[component]
pub fn ModelPickerMenu(
    /// Models (the demo set when empty).
    #[props(default)]
    models: Vec<PickerModel>,
    /// Chosen id (the second model, as affectUI's demo, when empty and there are several).
    #[props(default)]
    value: String,
    /// Start with the menu open.
    #[props(default = true)]
    default_open: bool,
    /// Extended thinking on at the start.
    #[props(default)]
    thinking: bool,
    /// Web search on at the start.
    #[props(default)]
    web_search: bool,
    /// Starting effort: `low` · `medium` · `high`.
    #[props(default = "medium".to_string())]
    effort: String,
    /// `letters` (affectUI) or `brands` (coloured brand marks).
    #[props(default = "letters".to_string())]
    marks: String,
    /// Placeholder of the composer.
    #[props(default = "Ask anything…".to_string())]
    placeholder: String,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Picked id.
    #[props(default)]
    on_change: EventHandler<String>,
    /// Thinking switch.
    #[props(default)]
    on_thinking_change: EventHandler<bool>,
    /// Web search switch.
    #[props(default)]
    on_web_search_change: EventHandler<bool>,
    /// Effort (`low` · `medium` · `high`).
    #[props(default)]
    on_effort_change: EventHandler<String>,
    /// Menu opened or closed.
    #[props(default)]
    on_open_change: EventHandler<bool>,
) -> Element {
    let models = if models.is_empty() {
        demo_models()
    } else {
        models
    };
    let n = models.len();
    let first = models
        .iter()
        .position(|m| m.id == value)
        .unwrap_or(if n > 1 { 1 } else { 0 });
    let ms = use_signal(move || models.clone());
    let mut chosen = use_signal(move || first);
    let mut lit = use_signal(|| 0usize);
    // A row is only filled once the user points at it (mouse or arrows); on open the chosen row
    // shows just its check and blue dashes, as affectUI does.
    let mut moved = use_signal(|| false);
    let mut open = use_signal(move || default_open);
    // The click-away scrim only follows a menu the user opened: a menu that starts open (as on
    // a docs page) must not cover the page and swallow its first click.
    let mut veil = use_signal(|| false);
    let mut think = use_signal(move || thinking);
    let mut web = use_signal(move || web_search);
    let mut eff = use_signal(move || Effort::from_id(&effort));
    let mut menu = use_signal(|| None::<Rc<MountedData>>);
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let mut text = use_signal(String::new);
    let all = ms();
    let cur = all[chosen().min(n - 1)].clone();
    let pointed = all[if moved() { lit() } else { chosen() }.min(n - 1)].clone();
    let brands = marks == "brands";
    let tray = tray_values(&pointed, think(), eff());
    let mut set_open = move |o: bool| {
        open.set(o);
        veil.set(o);
        on_open_change.call(o);
        if o {
            lit.set(chosen());
            moved.set(false);
            focus(menu());
        } else {
            focus(trig());
        }
    };
    let mut pick = move |i: usize| {
        chosen.set(i);
        let id = ms.peek()[i].id.clone();
        on_change.call(id);
    };
    let keys = move |e: KeyboardEvent| match e.key() {
        Key::ArrowDown => {
            e.prevent_default();
            if open() {
                moved.set(true);
                lit.set((lit() + 1) % n);
            } else {
                set_open(true)
            }
        }
        Key::ArrowUp if open() => {
            e.prevent_default();
            moved.set(true);
            lit.set((lit() + n - 1) % n)
        }
        Key::Home if open() => lit.set(0),
        Key::End if open() => lit.set(n - 1),
        Key::Enter if open() => {
            e.prevent_default();
            pick(lit());
            set_open(false)
        }
        Key::Escape if open() => {
            e.prevent_default();
            set_open(false)
        }
        _ => {}
    };
    let name_line = if think() {
        format!("Model: {}, extended thinking", cur.name)
    } else {
        format!("Model: {}", cur.name)
    };
    let tile = move |m: &PickerModel| -> Element {
        if brands && !m.brand.is_empty() {
            rsx! { span { class: "oxmq-tile brand", BrandIcon { name: m.brand.clone(), size: 13.0, coloured: true, fallback: true } } }
        } else if m.varies() {
            rsx! { span { class: "oxmq-tile", Icon { name: "plus", size: 11.0, stroke: 2.2 } } }
        } else {
            rsx! { span { class: "oxmq-tile", "{m.mark}" } }
        }
    };
    let dashes = |v: Option<u8>| -> Element {
        let v = v.unwrap_or(0);
        rsx! { span { class: "oxmq-dash", "aria-hidden": "true", for i in 1..=5u8 { i { key: "{i}", "data-on": "{i <= v}" } } } }
    };
    rsx! {
        div { class: "oxmq", "data-style": style_attr(&style), "data-open": "{open()}",
            onkeydown: keys,
            div { class: "oxmq-card",
                input { class: "oxmq-in", value: "{text}", placeholder: "{placeholder}", "aria-label": "Message",
                    oninput: move |e| text.set(e.value()) }
                div { class: "oxmq-row",
                    button { class: "oxmq-trig", r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{open()}", "aria-label": "{name_line}",
                        onmounted: move |e| trig.set(Some(e.data())),
                        onclick: move |_| set_open(!open()),
                        {tile(&cur)}
                        span { key: "{cur.id}", class: "oxmq-tn", "{cur.name}" }
                        if think() { span { class: "oxmq-bulb", Stroke { name: "bulb", size: 14.0 } } }
                        span { class: "oxmq-chev", Icon { name: "chevron", size: 12.0, stroke: 2.0 } }
                    }
                    span { class: "oxmq-sp" }
                    button { class: "oxmq-send", r#type: "button", "aria-label": "Send message", disabled: text().trim().is_empty(),
                        onclick: move |_| text.set(String::new()),
                        Icon { name: "up", size: 15.0, stroke: 2.0 } }
                }
                if open() {
                    if veil() { div { class: "oxmq-scrim", onclick: move |_| set_open(false) } }
                    div { class: "oxmq-menu", role: "dialog", "aria-label": "Model settings", tabindex: "-1",
                        onmounted: move |e| menu.set(Some(e.data())),
                        div { class: "oxmq-head", "aria-hidden": "true",
                            span { class: "oxmq-hm", "Model" }
                            span { class: "oxmq-hc", "Speed" } span { class: "oxmq-hc", "Quality" } span { class: "oxmq-hc", "Cost" }
                        }
                        div { class: "oxmq-list", role: "listbox", "aria-label": "Model", "aria-activedescendant": "oxmq-o{lit()}",
                            for (i, m) in all.iter().enumerate() {
                                div { key: "{m.id}", id: "oxmq-o{i}", class: "oxmq-opt", role: "option", "aria-selected": "{chosen() == i}", "data-lit": "{moved() && lit() == i}",
                                    "aria-label": if m.varies() { format!("{}, {}, strengths vary", m.name, m.description) } else { format!("{}, {}. Speed {}, quality {}, cost {} of 5", m.name, m.description, m.speed.unwrap_or(0), m.quality.unwrap_or(0), m.cost.unwrap_or(0)) },
                                    onmouseenter: move |_| { moved.set(true); lit.set(i) },
                                    onclick: move |_| { pick(i); set_open(false) },
                                    {tile(m)}
                                    span { class: "oxmq-nm",
                                        span { class: "oxmq-n", "{m.name}" if chosen() == i { span { class: "oxmq-ck", Icon { name: "check", size: 13.0, stroke: 2.2 } } } }
                                        span { class: "oxmq-d", "{m.description}" }
                                    }
                                    if m.varies() {
                                        span { class: "oxmq-varies", "Varies" }
                                    } else {
                                        {dashes(m.speed)} {dashes(m.quality)} {dashes(m.cost)}
                                    }
                                }
                            }
                        }
                        div { class: "oxmq-sep" }
                        button { class: "oxmq-sw", r#type: "button", role: "switch", "aria-checked": "{think()}",
                            onclick: move |_| { let v = !think(); think.set(v); on_thinking_change.call(v) },
                            span { class: "oxmq-si", Stroke { name: "bulb", size: 16.0 } }
                            span { class: "oxmq-sl", "Extended thinking" }
                            span { class: "oxmq-tg", i {} }
                        }
                        div { class: "oxmq-unfold", "data-open": "{think()}",
                            div { class: "oxmq-unin",
                                div { class: "oxmq-eff",
                                    span { class: "oxmq-el", "Effort" }
                                    div { class: "oxmq-seg", role: "radiogroup", "aria-label": "Effort",
                                        for e in [Effort::Low, Effort::Medium, Effort::High] {
                                            button { key: "{e.id()}", r#type: "button", role: "radio", "aria-checked": "{eff() == e}", tabindex: if think() { "0" } else { "-1" },
                                                onclick: move |_| { eff.set(e); on_effort_change.call(e.id().to_string()) }, "{e.label()}" }
                                        }
                                    }
                                }
                            }
                        }
                        button { class: "oxmq-sw", r#type: "button", role: "switch", "aria-checked": "{web()}",
                            onclick: move |_| { let v = !web(); web.set(v); on_web_search_change.call(v) },
                            span { class: "oxmq-si", Icon { name: "globe", size: 16.0 } }
                            span { class: "oxmq-sl", "Web search" }
                            span { class: "oxmq-tg", i {} }
                        }
                        div { class: "oxmq-tray", "aria-live": "polite",
                            if web() {
                                span { class: "oxmq-tt", "Looks things up on the web when it helps" }
                            } else if let Some((l, p)) = &tray {
                                span { class: "oxmq-tt", b { key: "{l}", "{l}" } " first token · " b { key: "{p}", "{p}" } " / M tokens" }
                            } else {
                                span { class: "oxmq-tt", "Time and price vary by message" }
                            }
                            span { class: "oxmq-sp" }
                            kbd { "↑↓" } kbd { "↵" }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────── Run Controls ──

/// The 3×3 pixel mark of the run header.
#[component]
fn PixelMark(#[props(default = 14.0)] size: f32) -> Element {
    rsx! {
        svg { class: "oxrq-mark", width: "{size}", height: "{size}", view_box: "0 0 14 14", "aria-hidden": "true",
            for r in 0..3 { for c in 0..3 {
                if (r * 3 + c) % 2 == 0 || r == 1 {
                    rect { key: "{r}{c}", x: "{c as f32 * 5.0}", y: "{r as f32 * 5.0}", width: "3.5", height: "3.5", rx: ".6", opacity: if (r + c) % 2 == 1 { ".55" } else { "1" } }
                }
            } }
        }
    }
}

/// One turn as shown: its pieces, phase, words shown, and whether a steer has landed.
fn view_turn(
    t: &RunTurn,
    start: ClockMs,
    now: ClockMs,
    stop: Option<usize>,
) -> (Vec<Piece>, Phase, usize) {
    let total = t.words();
    let el = now.since(start).as_millis() as u64;
    let w = stop.unwrap_or_else(|| words_at(el, total));
    let ph = phase_at(el, total, stop.is_some());
    (t.upto(w), ph, w)
}

/// Run Controls (affectUI): a run panel and its composer. The run thinks, then streams with a
/// block caret and "Esc to stop"; a steer lands mid-sentence as a blue pill and the status reads
/// "Writing · steered"; a queued message waits in the band above the composer ("Next", ✕) and runs
/// when the turn is done. Stop freezes the text ("Stopped at 29 words") with Resume, Retry and
/// Dismiss; Done shows the time and Retry. The composer has Chat | Act (Chat disables the
/// permissions), the permissions menu (Ask first, Auto-edit, red Full access), a context ring and
/// one round send / stop button. Enter steers while running, ⌥ Enter queues, Esc stops.
#[component]
pub fn AgentRunPanel(
    /// Agent name in the header.
    #[props(default = "Release writer".to_string())]
    agent: String,
    /// Turns: the first runs on its own; the rest are the replies to queued or sent messages, in
    /// order. Text: blank lines are paragraphs, `[[steer:…]]` a steer pill.
    turns: Vec<RunTurn>,
    /// Messages waiting in the queue at the start.
    #[props(default)]
    queue: Vec<String>,
    /// Start running (`false`: idle with the empty state).
    #[props(default = true)]
    autoplay: bool,
    /// Stop the first turn at this word at the start.
    #[props(default)]
    stopped_at: Option<usize>,
    /// How far into the first turn it starts, ms (a large value starts it done).
    #[props(default)]
    elapsed: u64,
    /// `act` or `chat`.
    #[props(default = "act".to_string())]
    mode: String,
    /// `ask`, `auto` or `full`.
    #[props(default = "ask".to_string())]
    permission: String,
    /// Open the permissions menu at the start.
    #[props(default)]
    menu_open: bool,
    /// Context in use before the run, percent.
    #[props(default = 38.0)]
    context: f32,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Shared clock.
    now: ClockMs,
    /// Actions: (`send` | `steer` | `queue` | `stop` | `resume` | `retry` | `dismiss` | `remove`,
    /// text).
    #[props(default)]
    on_action: EventHandler<(String, String)>,
) -> Element {
    let start_ms = if let Some(w) = stopped_at {
        THINK_MS + w as u64 * WORD_MS
    } else {
        elapsed
    };
    // (turn index, start time, stopped at)
    let mut cur = use_signal(move || {
        if autoplay || stopped_at.is_some() {
            Some((0usize, ClockMs(now.0.saturating_sub(start_ms)), stopped_at))
        } else {
            None
        }
    });
    let ts = use_signal(move || turns.clone());
    let mut q = use_signal(move || queue.clone());
    let mut extra = use_signal(Vec::<(usize, String)>::new); // steers typed by the user: (word, text)
    let mut done = use_signal(Vec::<usize>::new); // finished turns kept above
    let mut dismissed = use_signal(|| false);
    let mut md = use_signal(move || mode == "chat");
    let mut perm = use_signal(move || RunPermission::from_id(&permission));
    let mut menu = use_signal(move || menu_open);
    let mut draft = use_signal(String::new);
    let mut live = use_signal(String::new);
    let all = ts();
    let state = cur().map(|(i, st, stop)| {
        let mut t = all
            .get(i)
            .cloned()
            .unwrap_or_else(|| RunTurn::parse("", ""));
        // User steers land as pills after the word they were sent at.
        let ex = extra();
        if !ex.is_empty() {
            let mut out = vec![];
            let mut seen = 0usize;
            for p in t.pieces {
                if let Piece::Text(s) = &p {
                    let w: Vec<&str> = s.split_whitespace().collect();
                    let mut from = 0usize;
                    for (at, msg) in ex.iter() {
                        if *at > seen && *at < seen + w.len() {
                            let k = at - seen;
                            out.push(Piece::Text(w[from..k].join(" ")));
                            out.push(Piece::Steer(msg.clone()));
                            from = k;
                        }
                    }
                    out.push(Piece::Text(w[from..].join(" ")));
                    seen += w.len();
                } else {
                    out.push(p);
                }
            }
            t.pieces = out;
        }
        let (pieces, ph, w) = view_turn(&t, st, now, stop);
        (i, t, pieces, ph, w)
    });
    let phase = state.as_ref().map_or(Phase::Idle, |s| s.3);
    let running = phase.running();
    let steered = state
        .as_ref()
        .is_some_and(|s| s.2.iter().any(|p| matches!(p, Piece::Steer(_))));
    let words_now = state.as_ref().map_or(0, |s| s.4);
    let pct = context_pct(context, words_now + done().len() * 60);
    // A finished turn hands over to the next queued message (Act mode only).
    if phase == Phase::Done && !q.peek().is_empty() && !*md.peek() {
        let i = state.as_ref().map_or(0, |s| s.0);
        if i + 1 < all.len() {
            spawn(async move {
                // Re-check: another render may have handed over already.
                if (*cur.peek()).map(|c| c.0) != Some(i) || q.peek().is_empty() {
                    return;
                }
                let msg = q.write().remove(0);
                done.write().push(i);
                extra.set(vec![]);
                dismissed.set(false);
                cur.set(Some((i + 1, now, None)));
                live.set(format!("Running the queued message: {msg}"));
            });
        }
    }
    let queued = q().len();
    let sr = if queued > 0 {
        format!(
            "{agent}: {}, {queued} queued {}",
            phase.label(steered),
            live()
        )
    } else {
        format!("{agent}: {} {}", phase.label(steered), live())
    };
    let mut stop = move || {
        if let Some((i, st, None)) = cur() {
            let t = &ts.peek()[i];
            let w = words_at(now.since(st).as_millis() as u64, t.words());
            cur.set(Some((i, st, Some(w))));
            on_action.call(("stop".into(), String::new()));
            live.set(format!("Stopped at {w} words."));
        }
    };
    let mut submit = move |alt: bool| {
        let t = draft().trim().to_string();
        if t.is_empty() {
            return;
        }
        if running && !alt {
            if let Some((i, st, _)) = cur() {
                let w = words_at(now.since(st).as_millis() as u64, ts.peek()[i].words());
                extra.write().push((w.max(1), t.clone()));
            }
            on_action.call(("steer".into(), t));
        } else if running || alt {
            q.write().push(t.clone());
            on_action.call(("queue".into(), t));
            live.set("Message queued.".into());
        } else {
            // Idle or finished: the next turn answers it (or the last one again).
            let next = cur().map_or(0, |c| (c.0 + 1).min(ts.peek().len().saturating_sub(1)));
            if let Some((i, _, _)) = cur() {
                done.write().push(i);
            }
            extra.set(vec![]);
            dismissed.set(false);
            cur.set(Some((next, now, None)));
            on_action.call(("send".into(), t));
        }
        draft.set(String::new());
    };
    let header = |name: &str, label: &str| -> Element {
        rsx! { div { class: "oxrq-hd", PixelMark {} b { "{name}" } if !label.is_empty() { span { class: "oxrq-st", "{label}" } } } }
    };
    let body = |pieces: &[Piece], caret: bool| -> Element {
        // Paragraphs: split on breaks; pills sit inline.
        let mut paras: Vec<Vec<Piece>> = vec![vec![]];
        for p in pieces {
            if *p == Piece::Break {
                paras.push(vec![]);
            } else if let Some(l) = paras.last_mut() {
                l.push(p.clone());
            }
        }
        let last = paras.len() - 1;
        rsx! {
            for (k, para) in paras.into_iter().enumerate() {
                p { key: "{k}", class: "oxrq-p",
                    for (j, x) in para.into_iter().enumerate() {
                        match x {
                            Piece::Text(s) => rsx! { span { key: "{j}", "{s} " } },
                            Piece::Steer(s) => rsx! { span { key: "{j}", br {} span { class: "oxrq-steer", Stroke { name: "return", size: 12.0 } "{s}" } " " } },
                            Piece::Break => rsx! {},
                        }
                    }
                    if caret && k == last { span { class: "oxrq-caret", "aria-hidden": "true" } }
                }
            }
        }
    };
    let chat = md();
    let p = perm();
    rsx! {
        div { class: "oxrq", "data-style": style_attr(&style),
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { if menu() { menu.set(false) } else if running { e.prevent_default(); stop() } },
            div { class: "oxrq-run", role: "log", "aria-live": "polite", "aria-busy": "{running}", tabindex: "0", "aria-label": "Run",
              div { class: "oxrq-runin",
                if state.is_none() {
                    div { class: "oxrq-empty", PixelMark { size: 18.0 } p { "Ask {agent} anything – steer it or stop it while it works." } }
                }
                for i in done() {
                    div { key: "d{i}", class: "oxrq-turn",
                        if let Some(t) = all.get(i) {
                            if !t.prompt.is_empty() { div { class: "oxrq-you", span { "{t.prompt}" } } }
                            {header(&agent, "Done")}
                            div { class: "oxrq-bd", {body(&t.upto(t.words()), false)} }
                        }
                    }
                }
                if let Some((i, t, pieces, ph, w)) = &state {
                    div { key: "c{i}", class: "oxrq-turn",
                        if !t.prompt.is_empty() { div { class: "oxrq-you", span { "{t.prompt}" } } }
                        {header(&agent, ph.label(steered))}
                        div { class: "oxrq-bd",
                            if *ph == Phase::Thinking { p { class: "oxrq-p", span { class: "oxrq-caret" } } } else { {body(pieces, ph.running() || *ph == Phase::Stopped)} }
                            if ph.running() {
                                div { class: "oxrq-ft", kbd { "Esc" } span { "to stop" } }
                            } else if *ph == Phase::Stopped && !dismissed() {
                                div { class: "oxrq-ft",
                                    span { "Stopped at {w} words" }
                                    span { class: "oxrq-sp" }
                                    button { class: "oxrq-resume", r#type: "button", onclick: move |_| { if let Some((i, _, Some(w))) = cur() { cur.set(Some((i, ClockMs(now.0.saturating_sub(THINK_MS + w as u64 * WORD_MS)), None))); on_action.call(("resume".into(), String::new())) } },
                                        Icon { name: "play", size: 10.0 } "Resume" }
                                    button { class: "oxrq-ghost", r#type: "button", onclick: move |_| { if let Some((i, _, _)) = cur() { extra.set(vec![]); cur.set(Some((i, now, None))); on_action.call(("retry".into(), String::new())) } },
                                        Icon { name: "retry", size: 12.0 } "Retry" }
                                    button { class: "oxrq-ghost", r#type: "button", onclick: move |_| { dismissed.set(true); on_action.call(("dismiss".into(), String::new())) },
                                        Icon { name: "x", size: 12.0 } "Dismiss" }
                                }
                            } else if *ph == Phase::Done {
                                div { class: "oxrq-ft",
                                    span { "Done in {took(t.words())}" }
                                    span { class: "oxrq-sp" }
                                    button { class: "oxrq-ghost", r#type: "button", onclick: move |_| { if let Some((i, _, _)) = cur() { extra.set(vec![]); cur.set(Some((i, now, None))); on_action.call(("retry".into(), String::new())) } },
                                        Icon { name: "retry", size: 12.0 } "Retry" }
                                }
                            }
                        }
                    }
                }
              }
            }
            if !q().is_empty() {
                ul { class: "oxrq-q", "aria-label": "Queued messages",
                    for (k, m) in q().into_iter().enumerate() {
                        li { key: "{m}",
                            Stroke { name: "queue", size: 14.0 }
                            span { class: "oxrq-qt", "{m}" }
                            if k == 0 { span { class: "oxrq-next", "Next" } }
                            button { class: "oxrq-qx", r#type: "button", "aria-label": "Remove from queue", onclick: move |_| { let t = q.write().remove(k); on_action.call(("remove".into(), t)) }, Icon { name: "x", size: 12.0 } }
                        }
                    }
                }
            }
            div { class: "oxrq-comp",
                textarea { class: "oxrq-ta", rows: "1", value: "{draft}", "aria-label": "Message the agent",
                    placeholder: if running { "Steer the agent, or queue what's next…" } else { "Ask the agent…" },
                    oninput: move |e| draft.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(e.modifiers().alt()) },
                }
                div { class: "oxrq-bar",
                    div { class: "oxrq-mode", role: "radiogroup", "aria-label": "Mode",
                        button { r#type: "button", role: "radio", "aria-checked": "{chat}", onclick: move |_| { md.set(true); menu.set(false) }, "Chat" }
                        button { r#type: "button", role: "radio", "aria-checked": "{!chat}", onclick: move |_| md.set(false), "Act" }
                    }
                    div { class: "oxrq-pw",
                        button { class: "oxrq-perm", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{menu()}", "aria-disabled": "{chat}", "data-level": "{p.id()}",
                            title: if chat { "Chat only answers – it never changes anything" } else { p.desc() },
                            onclick: move |_| if !md() { menu.set(!menu()) },
                            match p { RunPermission::FullAccess => rsx! { Icon { name: "bolt", size: 14.0 } }, RunPermission::AutoEdit => rsx! { Stroke { name: "edit", size: 14.0 } }, RunPermission::AskFirst => rsx! { Stroke { name: "shield", size: 14.0 } } }
                            span { "{p.label()}" }
                            Icon { name: "chevron", size: 11.0 }
                        }
                        if menu() && !chat {
                            div { class: "oxrq-menu", role: "listbox", "aria-label": "What the agent may do",
                                for x in RunPermission::ALL {
                                    button { key: "{x.id()}", class: "oxrq-mi", r#type: "button", role: "option", "aria-selected": "{p == x}", "data-level": "{x.id()}",
                                        onclick: move |_| { perm.set(x); menu.set(false); live.set(format!("{}.", x.label())) },
                                        span { class: "oxrq-mic", match x { RunPermission::FullAccess => rsx! { Icon { name: "bolt", size: 15.0 } }, RunPermission::AutoEdit => rsx! { Stroke { name: "edit", size: 15.0 } }, RunPermission::AskFirst => rsx! { Stroke { name: "shield", size: 15.0 } } } }
                                        span { class: "oxrq-ml", span { class: "oxrq-mt", "{x.label()}" } span { class: "oxrq-md", "{x.desc()}" } }
                                        if p == x { span { class: "oxrq-ck", Icon { name: "check", size: 14.0, stroke: 2.0 } } }
                                    }
                                }
                            }
                        }
                    }
                    span { class: "oxrq-sp" }
                    span { class: "oxrq-ctx", title: "{pct}% of the context window in use",
                        svg { width: "14", height: "14", view_box: "0 0 14 14", "aria-hidden": "true",
                            circle { cx: "7", cy: "7", r: "5.5", fill: "none", class: "trk", stroke_width: "2" }
                            circle { cx: "7", cy: "7", r: "5.5", fill: "none", class: "arc", stroke_width: "2", stroke_linecap: "round",
                                stroke_dasharray: "{34.56 * pct as f32 / 100.0:.2} 34.56", transform: "rotate(-90 7 7)" }
                        }
                        "{pct}%"
                    }
                    if running && draft().trim().is_empty() {
                        button { class: "oxrq-send stop", r#type: "button", "aria-label": "Stop", onclick: move |_| stop(), span { class: "oxrq-sq" } }
                    } else {
                        button { class: "oxrq-send", r#type: "button", disabled: draft().trim().is_empty(), "aria-label": if running { "Steer" } else { "Send" }, onclick: move |_| submit(false),
                            Icon { name: "up", size: 14.0, stroke: 2.0 } }
                    }
                }
            }
            div { class: "oxrq-sr", role: "status", "{sr}" }
        }
    }
}
