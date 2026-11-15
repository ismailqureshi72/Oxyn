//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32): the exact **Agent
//! Sequence** ([`AgentSequence`]). Behaviour and look only; the model is
//! `oxyn_core::parity::seqfid`. Styles: `seqfid.css` (classes `oxsq-*`), in `PRIMITIVES_CSS`.
//! Follows the nearest `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as
//! `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::seqfid::{
        SEQ_LOOP_MS, SEQ_ROW, SEQ_STEP_MS, SeqKind, SeqMessage, SeqParticipant, SeqStatus,
        seq_answer, seq_bars, seq_demo_messages, seq_demo_participants, seq_frames, seq_height,
        seq_label, seq_rows, seq_shown, seq_stats, seq_status, seq_time,
    },
};

use crate::brand::BrandIcon;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

/// The 3 × 3 pixel mark: full for a person, a pattern for an agent.
fn pixel_mark(p: &SeqParticipant, i: usize) -> Element {
    let on: [bool; 9] = match (p.kind, i % 3) {
        (SeqKind::Person, _) => [true; 9],
        (_, 0) => [true, false, true, false, true, false, true, false, true],
        (_, 1) => [true, true, false, true, false, true, false, true, true],
        _ => [false, true, true, true, true, false, true, false, true],
    };
    rsx! {
        span { class: "oxsq-px", "data-c": "{p.color}", "aria-hidden": "true",
            for (k, b) in on.iter().enumerate() { i { key: "{k}", "data-k": "{k}", "data-on": "{b}" } }
        }
    }
}

fn mark(p: &SeqParticipant, i: usize) -> Element {
    if p.kind == SeqKind::Tool && !p.brand.is_empty() {
        rsx! { span { class: "oxsq-tool", BrandIcon { name: p.brand.clone(), size: 13.0, coloured: true, fallback: true } } }
    } else {
        pixel_mark(p, i)
    }
}

/// Agent Sequence (affectUI): one agent run as a sequence diagram – a column per participant with
/// a dashed lifeline, each message an arrow (calls solid, replies dashed) with its label as a tag
/// in its line's colour, bars while a participant works, parallel and retried messages in
/// labelled frames; a click on a participant or a message opens its card.
#[component]
pub fn AgentSequence(
    /// Your own participants (the demo's when `None`).
    #[props(default)]
    participants: Option<Vec<SeqParticipant>>,
    /// Your own messages, in order.
    #[props(default)]
    messages: Option<Vec<SeqMessage>>,
    /// The header's name.
    #[props(default = "Fix the failing sign-up".to_string())]
    title: String,
    /// Freeze the demo at this many ms into a pass.
    #[props(default)]
    at: Option<u64>,
    /// Still running: open bars shimmer and working marks twinkle.
    #[props(default = true)]
    running: bool,
    /// A message open at the start (id).
    #[props(default)]
    open: String,
    /// A participant open at the start (id).
    #[props(default)]
    open_participant: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A message selected (its id), or `None` when its card closes.
    #[props(default)]
    on_select: EventHandler<Option<String>>,
) -> Element {
    let start = use_signal(move || now);
    let mut sel = use_signal(move || (!open.is_empty()).then(|| open.clone()));
    let mut psel =
        use_signal(move || (!open_participant.is_empty()).then(|| open_participant.clone()));

    let raw = now.since(start()).as_millis() as u64;
    let demo = messages.is_none();
    let (lap, el) = match (demo, at) {
        (true, Some(a)) => (0, a),
        (true, None) => (raw / SEQ_LOOP_MS, raw % SEQ_LOOP_MS),
        _ => (0, raw),
    };
    let parts = participants.clone().unwrap_or_else(seq_demo_participants);
    let all = messages.clone().unwrap_or_else(seq_demo_messages);
    let n = if demo { seq_shown(el) } else { all.len() };
    // Keep only messages between known participants.
    let msgs: Vec<SeqMessage> = all
        .iter()
        .take(n)
        .filter(|m| parts.iter().any(|p| p.id == m.from) && parts.iter().any(|p| p.id == m.to))
        .cloned()
        .collect();
    let live = running && (!demo || n < all.len());
    let cols = parts.len().max(1);
    let col_of = |id: &str| parts.iter().position(|p| p.id == id).unwrap_or(0);
    let cx = |i: usize| (i as f32 + 0.5) / cols as f32 * 100.0;
    let ys = seq_rows(&msgs);
    // The diagram grows with the run, as the lifelines do.
    let h = seq_height(&msgs).max(150.0);
    let frames = seq_frames(&msgs);
    let bars = seq_bars(&msgs);
    let (working, failed) = seq_status(&msgs);
    let elapsed = msgs.last().map_or(0, |m| m.at);
    let fresh = if demo {
        el.saturating_sub(n.saturating_sub(1) as u64 * SEQ_STEP_MS) < 500
    } else {
        false
    };
    let last_id = msgs.last().map(|m| m.id.clone()).unwrap_or_default();
    let lap_k = format!("lap{lap}");
    let said = msgs
        .last()
        .map(|m| seq_label(m, &parts[col_of(&m.from)].name, &parts[col_of(&m.to)].name))
        .unwrap_or_default();
    let open_msg = sel().and_then(|id| msgs.iter().position(|m| m.id == id));
    let open_part = psel().and_then(|id| parts.iter().position(|p| p.id == id));

    rsx! {
        div { class: "oxsq", "data-style": style_attr(&style), "data-live": "{live}",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { sel.set(None); psel.set(None); on_select.call(None) },
            div { class: "oxsq-head",
                b { class: "oxsq-ttl", "{title}" }
                span { class: "oxsq-sp" }
                if failed > 0 { span { class: "oxsq-st", "data-k": "failed", "{failed} failure" if failed > 1 { "s" } } }
                else if working > 0 && live { span { class: "oxsq-st", "data-k": "working", "{working} working" } }
                span { class: "oxsq-st", "{msgs.len()} message" if msgs.len() != 1 { "s" } }
                span { class: "oxsq-st", "{seq_time(elapsed)}" }
            }
            div { class: "oxsq-body",
                div { class: "oxsq-chips",
                    span { class: "oxsq-gut" }
                    div { class: "oxsq-cr",
                        for (i, p) in parts.iter().cloned().enumerate() {
                            {
                                let pid = p.id.clone();
                                let is_open = open_part == Some(i);
                                let busy = bars.iter().any(|b| b.0 == p.id && b.2.is_none()) && live;
                                rsx! {
                                    span { key: "{p.id}", "data-k": "{p.id}", class: "oxsq-cc", style: "left:{cx(i):.3}%",
                                        button { class: "oxsq-chip", r#type: "button", "aria-expanded": "{is_open}", "aria-haspopup": "dialog", "data-busy": "{busy}",
                                            onclick: move |_| { sel.set(None); psel.set(if psel().as_deref() == Some(pid.as_str()) { None } else { Some(pid.clone()) }) },
                                            {mark(&p, i)} span { "{p.name}" }
                                        }
                                        if is_open {
                                            {
                                                let (s, r, t, b) = seq_stats(&msgs, &p.id, elapsed);
                                let kind = match p.kind { SeqKind::Person => "Person", SeqKind::Agent => "Agent", SeqKind::Tool => "Tool" };
                                let edge = if i == 0 { "l" } else if i + 1 == cols { "r" } else { "" };
                                                rsx! {
                                                    div { class: "oxsq-card", "data-k": "part", role: "dialog", "aria-label": "{p.name}", "data-edge": "{edge}",
                                                        div { class: "oxsq-ch", {mark(&p, i)} b { "{p.name}" } span { class: "oxsq-sp" }
                                                            button { class: "oxsq-x", r#type: "button", "aria-label": "Close", onclick: move |_| psel.set(None), "✕" } }
                                                        div { class: "oxsq-tags",
                                                            span { class: "oxsq-tg", "{kind}" }
                                                            span { class: "oxsq-tg", if busy { "Working" } else { "Idle" } }
                                                            if !p.model.is_empty() { span { class: "oxsq-tg", "{p.model}" } }
                                                        }
                                                        if !p.description.is_empty() { p { "{p.description}" } }
                                                        dl { class: "oxsq-stats",
                                                            div { dt { "Sent" } dd { "{s}" } }
                                                            div { dt { "Received" } dd { "{r}" } }
                                                            div { dt { "Tasks" } dd { "{t}" } }
                                                            div { dt { "Busy" } dd { "{b.div_ceil(1000)}s" } }
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
                div { class: "oxsq-dia", style: "height:{h:.0}px", key: "{lap_k}", "data-lap": "{lap_k}",
                    div { class: "oxsq-gut",
                        for (m, y) in msgs.iter().zip(ys.iter()) {
                            span { key: "{m.id}", "data-k": "{m.id}", class: "oxsq-t", style: "top:{y:.1}px", "{seq_time(m.at)}" }
                        }
                    }
                    div { class: "oxsq-area",
                        for i in 0..cols { i { key: "{i}", "data-k": "{i}", class: "oxsq-life", style: "left:{cx(i):.3}%" } }
                        if msgs.is_empty() { span { class: "oxsq-empty", "No messages yet" } }
                        for (k, (tag, a, b)) in frames.iter().cloned().enumerate() {
                            {
                                let span_cols: Vec<usize> = msgs[a..=b].iter().flat_map(|m| [col_of(&m.from), col_of(&m.to)]).collect();
                                let lo = *span_cols.iter().min().unwrap_or(&0);
                                let hi = *span_cols.iter().max().unwrap_or(&0);
                                let l = (lo as f32 + 0.5) / cols as f32 * 100.0;
                                let r = (hi as f32 + 0.5) / cols as f32 * 100.0;
                                let top = ys[a] - SEQ_ROW / 2.0 - 24.0;
                                let bot = ys[b] + SEQ_ROW / 2.0 - 4.0;
                                rsx! {
                                    div { key: "{k}", "data-k": "{k}", class: "oxsq-frame", style: "left:calc({l:.3}% - 18px);width:calc({r - l:.3}% + 36px);top:{top:.1}px;height:{bot - top:.1}px",
                                        span { class: "oxsq-ft", "{tag}" }
                                    }
                                }
                            }
                        }
                        for (k, (pid, a, b)) in bars.iter().cloned().enumerate() {
                            {
                                let c = col_of(&pid);
                                let top = ys[a];
                                let end = b.map_or(h - 8.0, |b| ys[b]);
                                let open_bar = b.is_none();
                                let color = parts[c].color.clone();
                                rsx! { i { key: "{k}", "data-k": "{k}", class: "oxsq-bar", "data-c": "{color}", "data-open": "{open_bar && live}", style: "left:{cx(c):.3}%;top:{top:.1}px;height:{(end - top).max(8.0):.1}px" } }
                            }
                        }
                        ol { class: "oxsq-msgs", "aria-label": "Messages",
                            for (i, m) in msgs.iter().cloned().enumerate() {
                                {
                                    let a = col_of(&m.from);
                                    let b = col_of(&m.to);
                                    let y = ys[i];
                                    let color = parts[a].color.clone();
                                    let label = seq_label(&m, &parts[a].name, &parts[b].name);
                                    let is_open = open_msg == Some(i);
                                    let dim = open_msg.is_some() && !is_open;
                                    let mid = m.id.clone();
                                    let new = fresh && m.id == last_id;
                                    let st = match m.status { SeqStatus::Failed => "failed", SeqStatus::Waiting => "waiting", SeqStatus::Ok => "" };
                                    let (l, w, dir) = if a == b { (cx(a), 0.0, "self") } else if a < b { (cx(a), cx(b) - cx(a), "r") } else { (cx(b), cx(a) - cx(b), "l") };
                                    // A tag sits centred over its arrow; a note's sits beside its loop.
                                    let tag_left = if a == b { cx(a) } else { (cx(a) + cx(b)) / 2.0 };
                                    let right_room = tag_left < 70.0;
                                    let up = y > h - 190.0 && y > 190.0;
                                    let tag_style = if dir == "self" { format!("left:calc({tag_left:.3}% + 24px);top:{:.1}px", y - 9.0) } else { format!("left:{tag_left:.3}%;top:{:.1}px", y - 22.0) };
                                    let card_top = if up { y - 26.0 } else { y + 10.0 };
                                    let answer = seq_answer(&msgs, i);
                                    let (fname, tname) = (parts[a].name.clone(), parts[b].name.clone());
                                    let (pa, pb) = (parts[a].clone(), parts[b].clone());
                                    rsx! {
                                        li { key: "{m.id}", "data-k": "{m.id}", class: "oxsq-m", "data-dim": "{dim}", "data-new": "{new}", "data-st": "{st}", "data-reply": "{m.is_reply()}", "data-c": "{color}",
                                            if dir == "self" {
                                                i { class: "oxsq-loop", style: "left:{l:.3}%;top:{y - 8.0:.1}px" }
                                            } else {
                                                i { class: "oxsq-ln", "data-dir": "{dir}", style: "left:{l:.3}%;width:{w:.3}%;top:{y:.1}px" }
                                                if m.status == SeqStatus::Failed { i { class: "oxsq-cross", style: "left:{cx(b):.3}%;top:{y:.1}px", "✕" } }
                                            }
                                            button { class: "oxsq-tag", "data-self": "{a == b}", r#type: "button", "aria-label": "{label}, opens a dialog", "aria-expanded": "{is_open}",
                                                tabindex: if i == 0 { "0" } else { "-1" },
                                                style: "{tag_style}",
                                                onclick: move |_| { psel.set(None); let next = if sel().as_deref() == Some(mid.as_str()) { None } else { Some(mid.clone()) }; sel.set(next.clone()); on_select.call(next) },
                                                "{m.text}"
                                            }
                                            if is_open {
                                                div { class: "oxsq-card", "data-k": "msg", role: "dialog", "aria-label": "{m.text}", "data-left": "{!right_room}", "data-up": "{up}",
                                                    style: "left:{tag_left:.3}%;top:{card_top:.1}px",
                                                    div { class: "oxsq-ch",
                                                        {mark(&pa, a)} b { "{fname}" } span { class: "oxsq-arr", "→" } {mark(&pb, b)} b { "{tname}" }
                                                        span { class: "oxsq-sp" }
                                                        button { class: "oxsq-x", r#type: "button", "aria-label": "Close", onclick: move |_| { sel.set(None); on_select.call(None) }, "✕" }
                                                    }
                                                    span { class: "oxsq-mt", "data-c": "{color}", "data-st": "{st}", "{m.text}" }
                                                    div { class: "oxsq-tags",
                                                        span { class: "oxsq-tg", if m.is_reply() { "Reply" } else if a == b { "Note" } else { "Call" } }
                                                        if !m.frame.is_empty() { span { class: "oxsq-tg", "{m.frame.to_uppercase()}" } }
                                                        if m.status == SeqStatus::Failed { span { class: "oxsq-tg", "data-k": "failed", "Failed" } }
                                                    }
                                                    if !m.detail.is_empty() { code { class: "oxsq-detail", "{m.detail}" } }
                                                    span { class: "oxsq-foot", "Sent {seq_time(m.at)}" if !answer.is_empty() { "   ·   {answer}" } }
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
            div { class: "oxsq-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
