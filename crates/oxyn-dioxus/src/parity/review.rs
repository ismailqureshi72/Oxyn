//! Review surfaces (plan #30): `AiInlineEdit`, `ImageCompare`, `CommentThread`, `ReviewCenter`
//! and `ReviewCard`. Styles: `review.css` (classes `oxyn-ie-*`, `oxyn-icmp-*`, `oxyn-cmt-*`,
//! `oxyn-rvc-*`, `oxyn-rvk-*`).

use std::collections::BTreeMap;
use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::diff::{
        Center, Chunk, Comment, CompareMode, EditPhase, Filter, Finding, FindingState, LineKind,
        Order, Person, QUICK_ACTIONS, QuickAction, REJECT_REASONS, ReviewItem, Span, Thread,
        ThreadEvent, Verdict, apply_chunks, caption_opacity, change_count, chunks_reached,
        compare_key, compare_valuetext, mention_matches, mention_query, mention_spans,
        review_verdict, rewrite_chunks, scripted_completion, scripted_rewrite, severity_counts,
        spans_text, stat_label, waited_label, words_shown,
    },
};

use crate::composer::icons::Icon;

// ---------------------------------------------------------------------------------------------
// AI inline edit
// ---------------------------------------------------------------------------------------------

fn sentence_at(p: &str, word: usize) -> (usize, usize) {
    // Word indices (whitespace split) of the sentence holding `word`.
    let words: Vec<&str> = p.split_whitespace().collect();
    let mut start = 0;
    for (i, w) in words.iter().enumerate() {
        if i >= word {
            break;
        }
        if w.ends_with(['.', '!', '?']) {
            start = i + 1;
        }
    }
    let mut end = words.len().saturating_sub(1);
    for (i, w) in words.iter().enumerate().skip(word) {
        if w.ends_with(['.', '!', '?']) {
            end = i;
            break;
        }
    }
    (start, end.max(start))
}

fn words_range(p: &str, a: usize, b: usize) -> String {
    p.split_whitespace()
        .skip(a)
        .take(b + 1 - a)
        .collect::<Vec<_>>()
        .join(" ")
}

/// A document you edit with AI in place: click a word to select its sentence (Shift+click
/// extends), ⌘K or Edit opens a prompt under it with quick actions and a free instruction, the
/// rewrite streams in as a word diff (struck deletions, highlighted insertions, unreached text
/// dimmed), each change gets its own Accept / Reject, and ⌘Z / ⇧⌘Z undo and redo. At the end of
/// a paragraph, ghost text offers a continuation that Tab accepts.
#[component]
pub fn AiInlineEdit(
    /// Title.
    #[props(default = "Launch plan".to_string())]
    title: String,
    /// Paragraphs.
    paragraphs: Vec<String>,
    /// Clock (drives the stream).
    now: ClockMs,
    /// Quick actions.
    #[props(default = QUICK_ACTIONS.to_vec())]
    actions: Vec<QuickAction>,
    /// Stream speed multiplier.
    #[props(default = 1.0)]
    speed: f32,
    /// Ghost-text completions.
    #[props(default = true)]
    ghost: bool,
    /// Author in the header.
    #[props(default = "You".to_string())]
    author: String,
    /// Changes start accepted on Esc-less review (`false` = reject when undecided at apply).
    #[props(default = true)]
    review_each: bool,
    /// The document changed.
    #[props(default)]
    on_change: EventHandler<Vec<String>>,
) -> Element {
    let init = paragraphs.clone();
    let mut doc = use_signal(move || init);
    let mut undo_stack = use_signal(Vec::<Vec<String>>::new);
    let mut redo_stack = use_signal(Vec::<Vec<String>>::new);
    let mut sel = use_signal(|| None::<(usize, usize, usize)>);
    let mut phase = use_signal(|| EditPhase::Idle);
    let mut query = use_signal(String::new);
    let mut active = use_signal(|| 0usize);
    let mut started = use_signal(|| None::<ClockMs>);
    let mut rewrite = use_signal(String::new);
    let mut instr = use_signal(String::new);
    let mut stop_words = use_signal(|| None::<usize>);
    let mut decisions = use_signal(BTreeMap::<usize, bool>::new);
    let mut ghost_on = use_signal(|| None::<usize>);
    let mut live = use_signal(String::new);
    let mut receipt = use_signal(String::new);
    let d = doc.read().clone();
    let selection = sel();
    let original = selection
        .map(|(p, a, b)| words_range(&d[p], a, b))
        .unwrap_or_default();
    let chunks = if phase() == EditPhase::Streaming || phase() == EditPhase::Review {
        rewrite_chunks(&original, &rewrite())
    } else {
        Vec::new()
    };
    let total_words = rewrite().split_whitespace().count();
    let shown = match (phase(), started()) {
        (EditPhase::Streaming, Some(t)) => words_shown(now.0.saturating_sub(t.0), speed),
        _ => usize::MAX,
    };
    let shown = stop_words().map_or(shown, |s| s.min(shown));
    let streaming_done =
        phase() == EditPhase::Streaming && (shown >= total_words || stop_words().is_some());
    let reached = chunks_reached(&chunks, shown);
    let in_review = phase() == EditPhase::Review || streaming_done;
    let nchanges = change_count(&chunks);
    let pending: Vec<usize> = chunks
        .iter()
        .zip(reached.iter())
        .filter_map(|(c, r)| match c {
            Chunk::Change { id, .. } if *r && !decisions.read().contains_key(id) => Some(*id),
            _ => None,
        })
        .collect();
    let q = query();
    let opts: Vec<QuickAction> = actions
        .iter()
        .filter(|a| q.is_empty() || a.label.to_lowercase().contains(&q.to_lowercase()))
        .cloned()
        .collect();
    let free = !q.trim().is_empty();
    let nopts = opts.len() + usize::from(free);
    let mut run = move |id: String, text: String| {
        let Some((p, a, b)) = *sel.peek() else { return };
        let src = words_range(&doc.peek()[p], a, b);
        rewrite.set(scripted_rewrite(&id, &src));
        instr.set(text);
        decisions.set(BTreeMap::new());
        stop_words.set(None);
        started.set(Some(now));
        phase.set(EditPhase::Streaming);
        live.set("Rewriting…".into());
    };
    let mut apply = move |accept_rest: bool| {
        let Some((p, a, b)) = *sel.peek() else { return };
        let before = doc.peek().clone();
        let src = words_range(&before[p], a, b);
        let chunks_now = rewrite_chunks(&src, &rewrite.peek());
        let reached_now = chunks_reached(&chunks_now, shown);
        let mut dec = decisions.peek().clone();
        for (c, r) in chunks_now.iter().zip(reached_now.iter()) {
            if let Chunk::Change { id, .. } = c {
                let v = dec.get(id).copied().unwrap_or(accept_rest && *r);
                dec.insert(*id, v && *r);
            }
        }
        let new_text = apply_chunks(&chunks_now, &dec, false);
        let accepted = dec.values().filter(|x| **x).count();
        let words: Vec<String> = before[p].split_whitespace().map(String::from).collect();
        let mut out: Vec<String> = words[..a].to_vec();
        if !new_text.is_empty() {
            out.push(new_text);
        }
        out.extend(words[(b + 1).min(words.len())..].iter().cloned());
        let mut next = before.clone();
        next[p] = out.join(" ");
        if accepted > 0 {
            undo_stack.write().push(before);
            redo_stack.write().clear();
            doc.set(next.clone());
            on_change.call(next);
        }
        let n = change_count(&chunks_now);
        let msg = if accepted == 0 {
            "No changes applied".to_string()
        } else {
            format!("Applied {accepted} of {n} changes")
        };
        receipt.set(msg.clone());
        live.set(msg);
        phase.set(EditPhase::Applied);
        sel.set(None);
    };
    let mut undo = move || {
        if let Some(prev) = undo_stack.write().pop() {
            redo_stack.write().push(doc.peek().clone());
            doc.set(prev.clone());
            on_change.call(prev);
            live.set("Undone".into());
            receipt.set("Undone".into());
        }
    };
    let mut redo = move || {
        if let Some(n) = redo_stack.write().pop() {
            undo_stack.write().push(doc.peek().clone());
            doc.set(n.clone());
            on_change.call(n);
            live.set("Redone".into());
        }
    };
    let ph = phase();
    let status_word = if ph == EditPhase::Streaming && !streaming_done {
        "Rewriting"
    } else if in_review {
        "Review"
    } else {
        ""
    };
    rsx! {
        div { class: "oxyn-ie", role: "region", "aria-label": "{title} editor", "data-phase": if in_review { "review" } else { ph.id() },
            onkeydown: move |e: KeyboardEvent| {
                let m = e.modifiers();
                let cmd = m.meta() || m.ctrl();
                let ch = if let Key::Character(c) = e.key() { c.to_lowercase() } else { String::new() };
                if cmd && ch == "k" {
                    e.prevent_default();
                    if *phase.peek() == EditPhase::Prompt { phase.set(EditPhase::Idle); return; }
                    if sel.peek().is_none() { sel.set(Some((0, 0, sentence_at(&doc.peek()[0], 0).1))); }
                    query.set(String::new()); active.set(0); phase.set(EditPhase::Prompt);
                    return;
                }
                if cmd && ch == "z" {
                    e.prevent_default();
                    if m.shift() { redo() } else { undo() }
                    return;
                }
                if e.key() == Key::Escape {
                    let cur_phase = *phase.peek();
                    match cur_phase {
                        EditPhase::Prompt => { phase.set(EditPhase::Idle); }
                        EditPhase::Streaming if !streaming_done => { stop_words.set(Some(shown)); live.set("Stopped. The words shown so far are ready to review.".into()); }
                        EditPhase::Streaming | EditPhase::Review => { apply(false) }
                        _ => { ghost_on.set(None); sel.set(None); }
                    }
                }
                if cmd && e.key() == Key::Enter && in_review { e.prevent_default(); apply(true) }
            },
            div { class: "oxyn-ie-hd",
                span { class: "oxyn-ie-av", "aria-hidden": "true", {author.chars().next().map(|c| c.to_string()).unwrap_or_default()} }
                span { class: "oxyn-ie-au", "{author}" }
                span { class: "oxyn-diff-sp" }
                if !undo_stack.read().is_empty() { button { class: "oxyn-diff-ib", r#type: "button", "aria-label": "Undo AI edit", title: "Undo (⌘Z)", onclick: move |_| undo(), Icon { name: "undo", size: 13.0 } } }
                span { class: "oxyn-ie-kbd", "Select a word · ⌘K" }
            }
            h3 { class: "oxyn-ie-title", "{title}" }
            for (pi, para) in d.iter().enumerate() {
                {
                    let words: Vec<String> = para.split_whitespace().map(String::from).collect();
                    let sel_here = selection.filter(|s| s.0 == pi);
                    let last = pi + 1 == d.len();
                    let ghost_txt = if ghost && ghost_on() == Some(pi) && ph == EditPhase::Idle { scripted_completion(para) } else { None };
                    let gt = ghost_txt.clone();
                    rsx! {
                        div { key: "{pi}", class: "oxyn-ie-pw",
                            p { class: "oxyn-ie-p", tabindex: "0", "aria-label": "Paragraph {pi + 1}",
                                onfocus: move |_| if ghost && last { ghost_on.set(Some(pi)) },
                                onkeydown: move |e: KeyboardEvent| {
                                    if e.key() == Key::Tab && !e.modifiers().shift()
                                        && let Some(g) = gt.clone()
                                    {
                                        {
                                            e.prevent_default();
                                            let before = doc.peek().clone();
                                            let mut next = before.clone();
                                            next[pi] = format!("{}{}", next[pi].trim_end(), g);
                                            undo_stack.write().push(before);
                                            doc.set(next.clone());
                                            on_change.call(next);
                                            ghost_on.set(None);
                                            live.set("Suggestion accepted".into());
                                        }
                                    }
                                },
                                for (wi, w) in words.iter().enumerate() {
                                    {
                                        let in_sel = sel_here.is_some_and(|s| wi >= s.1 && wi <= s.2);
                                        let first_sel = sel_here.is_some_and(|s| wi == s.1);
                                        let show_diff = in_sel && (ph == EditPhase::Streaming || ph == EditPhase::Review);
                                        rsx! {
                                            if show_diff {
                                                if first_sel {
                                                    span { class: "oxyn-ie-diff",
                                                        for (ci, c) in chunks.iter().enumerate() {
                                                            {
                                                                let r = reached.get(ci).copied().unwrap_or(false);
                                                                match c {
                                                                    Chunk::Same(t) => rsx! { span { key: "{ci}", class: if r { "" } else { "oxyn-ie-wait" }, "{t}" } },
                                                                    Chunk::Change { id, del, ins } => {
                                                                        let id = *id;
                                                                        let dec = decisions.read().get(&id).copied();
                                                                        if !r {
                                                                            rsx! { span { key: "{ci}", class: "oxyn-ie-wait", "{del}" } }
                                                                        } else {
                                                                            rsx! {
                                                                                span { key: "{ci}", class: "oxyn-ie-ch", tabindex: if in_review { "0" } else { "-1" }, role: "group",
                                                                                    "data-d": match dec { Some(true) => "yes", Some(false) => "no", None => "pending" },
                                                                                    "aria-label": "Change “{del.trim()}” to “{ins.trim()}”",
                                                                                    onkeydown: move |e: KeyboardEvent| {
                                                                                        if !in_review { return; }
                                                                                        match e.key() {
                                                                                            Key::Enter if !(e.modifiers().meta() || e.modifiers().ctrl()) => { e.prevent_default(); e.stop_propagation(); decisions.write().insert(id, true); live.set("Accepted".into()); }
                                                                                            Key::Backspace => { e.prevent_default(); decisions.write().insert(id, false); live.set("Rejected".into()); }
                                                                                            _ => {}
                                                                                        }
                                                                                    },
                                                                                    if dec != Some(true) && !del.is_empty() { del { class: "oxyn-idf-del", "{del}" } }
                                                                                    if dec != Some(false) && !ins.is_empty() { ins { class: "oxyn-idf-ins", "{ins}" } }
                                                                                    if in_review && dec.is_none() {
                                                                                        span { class: "oxyn-ie-tb",
                                                                                            button { r#type: "button", "aria-label": "Reject", onclick: move |_| { decisions.write().insert(id, false); }, Icon { name: "x", size: 11.0 } }
                                                                                            button { r#type: "button", "aria-label": "Accept", onclick: move |_| { decisions.write().insert(id, true); }, Icon { name: "check", size: 11.0 } }
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
                                                    " "
                                                }
                                            } else {
                                                span { class: if in_sel { "oxyn-ie-w sel" } else { "oxyn-ie-w" },
                                                    onclick: move |e: MouseEvent| {
                                                        if !matches!(*phase.peek(), EditPhase::Idle | EditPhase::Applied | EditPhase::Prompt) { return; }
                                                        let cur = *sel.peek();
                                                        let next = match cur {
                                                            Some((p, a, b)) if e.modifiers().shift() && p == pi => (pi, a.min(wi), b.max(wi)),
                                                            _ => { let (a, b) = sentence_at(&doc.peek()[pi], wi); (pi, a, b) }
                                                        };
                                                        sel.set(Some(next));
                                                        if *phase.peek() == EditPhase::Applied { phase.set(EditPhase::Idle); }
                                                    },
                                                    "{w}"
                                                }
                                                " "
                                            }
                                        }
                                    }
                                }
                                if let Some(g) = ghost_txt { span { class: "oxyn-ie-ghost", "aria-hidden": "true", "{g}" } }
                            }
                            if ghost_on() == Some(pi) && ph == EditPhase::Idle && ghost {
                                div { class: "oxyn-ie-hint", "Tab to accept · Esc to dismiss" }
                            }
                            if sel_here.is_some() {
                                match ph {
                                    EditPhase::Idle | EditPhase::Applied => rsx! {
                                        div { class: "oxyn-ie-layer", "data-k": "bubble",
                                            button { class: "oxyn-diff-btn", r#type: "button", onclick: move |_| { query.set(String::new()); active.set(0); phase.set(EditPhase::Prompt); },
                                                Icon { name: "spark", size: 12.0 } "Edit" span { class: "oxyn-diff-kbd", "⌘K" }
                                            }
                                        }
                                    },
                                    EditPhase::Prompt => rsx! {
                                        div { class: "oxyn-ie-layer", "data-k": "prompt", role: "dialog", "aria-label": "Edit with AI",
                                            input { class: "oxyn-ie-in", r#type: "text", role: "combobox", "aria-expanded": "true", "aria-controls": "oxyn-ie-acts",
                                                "aria-activedescendant": "oxyn-ie-a{active()}", placeholder: "Ask AI to edit…", value: "{q}",
                                                onmounted: move |e| { let el = e.data(); spawn(async move { let _ = el.set_focus(true).await; }); },
                                                oninput: move |e| { query.set(e.value()); active.set(0); },
                                                onkeydown: {
                                                    let opts = opts.clone();
                                                    move |e: KeyboardEvent| match e.key() {
                                                        Key::ArrowDown => { e.prevent_default(); active.set((active() + 1) % nopts.max(1)); }
                                                        Key::ArrowUp => { e.prevent_default(); active.set((active() + nopts.max(1) - 1) % nopts.max(1)); }
                                                        Key::Enter => {
                                                            e.prevent_default();
                                                            let i = active();
                                                            if free && i == 0 { run("free".into(), query.peek().clone()); }
                                                            else if let Some(a) = opts.get(i - usize::from(free)) { run(a.id.to_string(), a.instruction.to_string()); }
                                                        }
                                                        _ => {}
                                                    }
                                                }
                                            }
                                            ul { id: "oxyn-ie-acts", class: "oxyn-ie-acts", role: "listbox", "aria-label": "Quick actions",
                                                if free {
                                                    li { id: "oxyn-ie-a0", role: "option", "aria-selected": "{active() == 0}", onclick: move |_| { let t = query.peek().clone(); run("free".into(), t) },
                                                        Icon { name: "up", size: 12.0 } "Send “{q}”"
                                                    }
                                                }
                                                for (i, a) in opts.iter().enumerate() {
                                                    {
                                                        let idx = i + usize::from(free);
                                                        let a2 = a.clone();
                                                        rsx! {
                                                            li { key: "{a.id}", id: "oxyn-ie-a{idx}", role: "option", "aria-selected": "{active() == idx}",
                                                                onmouseenter: move |_| active.set(idx),
                                                                onclick: move |_| run(a2.id.to_string(), a2.instruction.to_string()),
                                                                Icon { name: "spark", size: 12.0 } "{a.label}"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    },
                                    _ => rsx! {
                                        div { class: "oxyn-ie-layer", "data-k": if in_review { "review" } else { "stream" },
                                            if in_review {
                                                span { class: "oxyn-ie-st", {format!("{nchanges} {} · {} left", if nchanges == 1 { "change" } else { "changes" }, pending.len())} }
                                                span { class: "oxyn-diff-sp" }
                                                button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| apply(false), "Reject all" }
                                                button { class: "oxyn-diff-btn p", r#type: "button", "aria-keyshortcuts": "Meta+Enter", onclick: move |_| apply(true), "Accept all" span { class: "oxyn-diff-kbd", "⌘↵" } }
                                            } else {
                                                span { class: "oxyn-sug-spin", "aria-hidden": "true" }
                                                span { class: "oxyn-ie-st oxyn-sug-shim", "{status_word}: {instr}" }
                                                span { class: "oxyn-diff-sp" }
                                                button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| { stop_words.set(Some(shown)); }, Icon { name: "stop", size: 11.0 } "Stop" }
                                            }
                                        }
                                    },
                                }
                            }
                        }
                    }
                }
            }
            if ph == EditPhase::Applied && !receipt().is_empty() {
                div { class: "oxyn-ie-receipt", Icon { name: "check", size: 12.0 } "{receipt}"
                    if !undo_stack.read().is_empty() { button { class: "oxyn-diff-lnk", r#type: "button", onclick: move |_| undo(), "Undo" } }
                }
            }
            div { class: "oxyn-diff-sr", role: "status", "aria-live": "polite", "{live}" }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Image compare
// ---------------------------------------------------------------------------------------------

/// Two images under one divider: drag anywhere on the frame, arrows move 1 % (Shift 10),
/// Page Up / Down 10 %, Home / End show one side, double-click recentres. Oxyn plus: side by side,
/// onion skin with an opacity slider and a difference blend.
#[component]
pub fn ImageCompare(
    /// The original (left or top).
    before: Element,
    /// The result (right or bottom).
    after: Element,
    /// Controlled position, percent.
    #[props(default)]
    position: Option<f32>,
    /// Starting position.
    #[props(default = 50.0)]
    default_position: f32,
    /// `horizontal` · `vertical`.
    #[props(default = "horizontal".to_string())]
    orientation: String,
    /// Captions (`None` hides them).
    #[props(default = Some(("Before".to_string(), "After".to_string())))]
    labels: Option<(String, String)>,
    /// Divider's accessible name.
    #[props(default = "Before and after".to_string())]
    label: String,
    /// CSS aspect ratio.
    #[props(default = "3 / 2".to_string())]
    aspect_ratio: String,
    /// Compare mode.
    #[props(default = CompareMode::Slider)]
    mode: CompareMode,
    /// Onion-skin opacity of the after image, 0..1.
    #[props(default = 0.5)]
    opacity: f32,
    /// The divider settled.
    #[props(default)]
    on_position_change: EventHandler<f32>,
) -> Element {
    let mut pos = use_signal(move || default_position);
    let mut dragging = use_signal(|| false);
    let mut bump = use_signal(|| 0u32);
    let mut frame = use_signal(|| None::<Rc<MountedData>>);
    let mut size = use_signal(|| (1.0f64, 1.0f64, 0.0f64, 0.0f64));
    let p = position.unwrap_or(pos());
    let vertical = orientation == "vertical";
    let measure = move || {
        if let Some(el) = frame.peek().clone() {
            spawn(async move {
                if let Ok(r) = el.get_client_rect().await {
                    size.set((r.size.width, r.size.height, r.origin.x, r.origin.y));
                }
            });
        }
    };
    let mut at = move |x: f64, y: f64| {
        let (w, h, ox, oy) = *size.peek();
        let v = if vertical {
            (y - oy) / h.max(1.0)
        } else {
            (x - ox) / w.max(1.0)
        };
        pos.set((v * 100.0).clamp(0.0, 100.0) as f32);
    };
    let clip = if vertical {
        format!("inset(0 0 {}% 0)", 100.0 - p)
    } else {
        format!("inset(0 {}% 0 0)", 100.0 - p)
    };
    let (lb, la) = labels.clone().unwrap_or_default();
    let mut onion = use_signal(move || opacity);
    let m = mode;
    rsx! {
        div { class: "oxyn-icmp", "data-mode": m.id(), "data-orient": "{orientation}", "data-drag": "{dragging()}",
            match m {
                CompareMode::Side => rsx! {
                    div { class: "oxyn-icmp-side",
                        figure { class: "oxyn-icmp-fig", style: "aspect-ratio:{aspect_ratio}", {before.clone()} if labels.is_some() { figcaption { class: "oxyn-icmp-cap", "{lb}" } } }
                        figure { class: "oxyn-icmp-fig", style: "aspect-ratio:{aspect_ratio}", {after.clone()} if labels.is_some() { figcaption { class: "oxyn-icmp-cap", "data-side": "after", "{la}" } } }
                    }
                },
                CompareMode::Onion | CompareMode::Difference => rsx! {
                    div { class: "oxyn-icmp-frame", style: "aspect-ratio:{aspect_ratio}",
                        div { class: "oxyn-icmp-img", {before.clone()} }
                        div { class: "oxyn-icmp-img oxyn-icmp-top", style: if m == CompareMode::Onion { format!("opacity:{}", onion()) } else { "mix-blend-mode:difference".to_string() }, {after.clone()} }
                        if m == CompareMode::Difference { span { class: "oxyn-icmp-cap", "Changed areas light up" } }
                    }
                    if m == CompareMode::Onion {
                        label { class: "oxyn-icmp-op", "{lb}"
                            input { r#type: "range", min: "0", max: "100", value: "{(onion() * 100.0).round()}", "aria-label": "After opacity", oninput: move |e| onion.set(e.value().parse::<f32>().unwrap_or(50.0) / 100.0) }
                            "{la}"
                        }
                    }
                },
                CompareMode::Slider => rsx! {
                    div { class: "oxyn-icmp-frame", style: "aspect-ratio:{aspect_ratio}", onmounted: move |e| { frame.set(Some(e.data())); measure(); },
                        div { class: "oxyn-icmp-img", {after.clone()} }
                        div { class: "oxyn-icmp-img oxyn-icmp-top", style: "clip-path:{clip}", {before.clone()} }
                        if labels.is_some() {
                            span { class: "oxyn-icmp-cap", "aria-hidden": "true", style: "opacity:{caption_opacity(p, true)}", "{lb}" }
                            span { class: "oxyn-icmp-cap", "data-side": "after", "aria-hidden": "true", style: "opacity:{caption_opacity(p, false)}", "{la}" }
                        }
                        div { class: "oxyn-icmp-line", "aria-hidden": "true", style: if vertical { format!("top:{p}%") } else { format!("left:{p}%") },
                            span { class: "oxyn-icmp-knob", key: "{bump()}", "data-bump": "{bump() > 0}" }
                        }
                        div { class: "oxyn-icmp-hit", role: "slider", tabindex: "0", "aria-label": "{label}", "aria-orientation": "{orientation}",
                            "aria-valuemin": "0", "aria-valuemax": "100", "aria-valuenow": "{p.round()}", "aria-valuetext": "{compare_valuetext(p)}",
                            onpointerenter: move |_| measure(),
                            onpointerdown: move |e: PointerEvent| { e.prevent_default(); dragging.set(true); let c = e.client_coordinates(); at(c.x, c.y); },
                            onpointermove: move |e: PointerEvent| if dragging() { let c = e.client_coordinates(); at(c.x, c.y); },
                            onpointerup: move |_| { dragging.set(false); on_position_change.call(*pos.peek()); },
                            onpointerleave: move |_| if dragging() { dragging.set(false); on_position_change.call(*pos.peek()); },
                            ondoubleclick: move |_| { pos.set(50.0); on_position_change.call(50.0); },
                            onkeydown: move |e: KeyboardEvent| {
                                let k = e.key().to_string();
                                let was = *pos.peek();
                                if let Some(n) = compare_key(was, &k, e.modifiers().shift()) {
                                    e.prevent_default();
                                    if (n - was).abs() < f32::EPSILON { bump += 1; }
                                    pos.set(n);
                                    on_position_change.call(n);
                                }
                            },
                        }
                    }
                },
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Comment thread
// ---------------------------------------------------------------------------------------------

fn avatar(p: &Person) -> Element {
    rsx! {
        span { class: "oxyn-cmt-av", "data-agent": "{p.agent}", "aria-hidden": "true",
            if p.agent { Icon { name: "spark", size: 12.0 } } else { "{p.initials()}" }
        }
    }
}

#[component]
fn MentionBox(
    people: Vec<Person>,
    placeholder: String,
    submit: String,
    #[props(default)] initial: String,
    on_submit: EventHandler<String>,
    #[props(default)] on_cancel: Option<EventHandler<()>>,
    #[props(default)] autofocus: bool,
) -> Element {
    let mut text = use_signal(move || initial.clone());
    let mut act = use_signal(|| 0usize);
    let t = text();
    let qy = mention_query(&t).map(String::from);
    let matches: Vec<Person> = qy
        .as_deref()
        .map(|q| mention_matches(q, &people).into_iter().cloned().collect())
        .unwrap_or_default();
    let open = !matches.is_empty();
    let lid = format!("oxyn-cmt-l{}", placeholder.len() + submit.len());
    let mut insert = move |p: Person| {
        let cur = text.peek().clone();
        if let Some(i) = cur.rfind('@') {
            text.set(format!("{}@{} ", &cur[..i], p.name));
        }
        act.set(0);
    };
    let m2 = matches.clone();
    rsx! {
        div { class: "oxyn-cmt-box",
            textarea { class: "oxyn-cmt-ta", rows: "2", placeholder: "{placeholder}", value: "{t}", role: "combobox",
                "aria-expanded": "{open}", "aria-controls": "{lid}", "aria-activedescendant": if open { format!("{lid}-{}", act()) } else { String::new() },
                onmounted: move |e| if autofocus { let el = e.data(); spawn(async move { let _ = el.set_focus(true).await; }); },
                oninput: move |e| text.set(e.value()),
                onkeydown: move |e: KeyboardEvent| {
                    let n = m2.len();
                    if n > 0 {
                        match e.key() {
                            Key::ArrowDown => { e.prevent_default(); act.set((act() + 1) % n); return; }
                            Key::ArrowUp => { e.prevent_default(); act.set((act() + n - 1) % n); return; }
                            Key::Enter | Key::Tab if !(e.modifiers().meta() || e.modifiers().ctrl()) => { e.prevent_default(); if let Some(p) = m2.get(act()).cloned() { insert(p) } return; }
                            Key::Escape => { e.prevent_default(); let c = text.peek().clone(); text.set(format!("{c} ")); return; }
                            _ => {}
                        }
                    }
                    if e.key() == Key::Enter && (e.modifiers().meta() || e.modifiers().ctrl()) {
                        e.prevent_default();
                        let v = text.peek().clone();
                        if !v.trim().is_empty() { on_submit.call(v); text.set(String::new()); }
                    }
                    if e.key() == Key::Escape && let Some(c) = on_cancel { e.stop_propagation(); c.call(()) }
                },
            }
            if open {
                ul { id: "{lid}", class: "oxyn-cmt-ml", role: "listbox", "aria-label": "Mentions",
                    for (i, p) in matches.iter().enumerate() {
                        {
                            let p2 = p.clone();
                            rsx! {
                                li { key: "{p.id}", id: "{lid}-{i}", role: "option", "aria-selected": "{act() == i}", onmousedown: move |e| { e.prevent_default(); insert(p2.clone()) },
                                    {avatar(p)} "{p.name}"
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxyn-cmt-brow",
                span { class: "oxyn-cmt-hint", "⌘↵ to send · @ to mention" }
                span { class: "oxyn-diff-sp" }
                if let Some(c) = on_cancel { button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| c.call(()), "Cancel" } }
                button { class: "oxyn-diff-btn p", r#type: "button", disabled: t.trim().is_empty(), onclick: move |_| { let v = text.peek().clone(); if !v.trim().is_empty() { on_submit.call(v); text.set(String::new()); } }, "{submit}" }
            }
        }
    }
}

#[component]
fn CommentItem(
    c: Comment,
    thread: Signal<Thread>,
    me: Person,
    people: Vec<Person>,
    reactions: Vec<String>,
    max_depth: usize,
    depth: usize,
    on_event: EventHandler<ThreadEvent>,
) -> Element {
    let mut replying = use_signal(|| false);
    let mut editing = use_signal(|| false);
    let mut confirm = use_signal(|| false);
    let mut picker = use_signal(|| false);
    let mut folded = use_signal(|| false);
    let mine = c.author.id == me.id;
    let id = c.id.clone();
    let nrep = c.replies.len();
    let spans = mention_spans(&c.body, &people);
    let label = if c.deleted {
        "Deleted comment".to_string()
    } else {
        format!("{} · {}", c.author.name, c.at)
    };
    rsx! {
        li { class: "oxyn-cmt-li", "data-depth": "{depth}",
            article { class: "oxyn-cmt-c", "aria-label": "{label}", "data-deleted": "{c.deleted}",
                {avatar(&c.author)}
                div { class: "oxyn-cmt-main",
                    div { class: "oxyn-cmt-meta",
                        if !c.deleted { b { "{c.author.name}" } } else { i { "Deleted comment" } }
                        if c.author.agent { span { class: "oxyn-cmt-tag", "Agent" } }
                        span { class: "oxyn-cmt-at", "{c.at}" }
                        if c.edited && !c.deleted { span { class: "oxyn-cmt-at", "· edited" } }
                    }
                    if editing() {
                        {
                            let id2 = id.clone();
                            rsx! { MentionBox { people: people.clone(), placeholder: "Edit comment", submit: "Save", initial: c.body.clone(), autofocus: true,
                                on_submit: move |v: String| { let ok = thread.write().edit(&id2, &v); if ok { on_event.call(ThreadEvent::Edit(id2.clone())) } editing.set(false) },
                                on_cancel: move |_| editing.set(false) } }
                        }
                    } else if !c.deleted {
                        p { class: "oxyn-cmt-body",
                            for (k, (t, m)) in spans.iter().enumerate() { span { key: "{k}", class: if *m { "oxyn-cmt-men" } else { "" }, "{t}" } }
                        }
                    }
                    if !c.deleted && !editing() {
                        div { class: "oxyn-cmt-acts",
                            for (emo, users) in c.reactions.iter() {
                                {
                                    let mine_r = users.contains(&me.id);
                                    let (id3, e2, me2) = (id.clone(), emo.clone(), me.id.clone());
                                    let n = users.len();
                                    rsx! {
                                        button { key: "{emo}", class: "oxyn-cmt-rx", r#type: "button", "aria-pressed": "{mine_r}",
                                            "aria-label": if mine_r { format!("{emo} {n}, including you") } else { format!("{emo} {n}") },
                                            onclick: move |_| { thread.write().react(&id3, &e2, &me2); on_event.call(ThreadEvent::React(id3.clone(), e2.clone())); },
                                            "{emo}" span { key: "{n}", class: "oxyn-cmt-n", "{n}" }
                                        }
                                    }
                                }
                            }
                            button { class: "oxyn-cmt-ab", r#type: "button", "aria-label": "Add reaction", "aria-expanded": "{picker()}", onclick: move |_| { let v = !picker(); picker.set(v) }, "☺+" }
                            button { class: "oxyn-cmt-ab", r#type: "button", onclick: move |_| replying.set(true), "Reply" }
                            if nrep > 0 { button { class: "oxyn-cmt-ab", r#type: "button", "aria-expanded": "{!folded()}", onclick: move |_| { let v = !folded(); folded.set(v) }, if folded() { {format!("Show {nrep} {}", if nrep == 1 { "reply" } else { "replies" })} } else { "Hide replies" } } }
                            if mine {
                                button { class: "oxyn-cmt-ab", r#type: "button", onclick: move |_| editing.set(true), "Edit" }
                                button { class: "oxyn-cmt-ab", r#type: "button", onclick: move |_| confirm.set(true), "Delete" }
                            }
                        }
                        if picker() {
                            div { class: "oxyn-cmt-pick", role: "group", "aria-label": "Reactions",
                                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { picker.set(false) },
                                for emo in reactions.iter() {
                                    {
                                        let (id3, e2, me2) = (id.clone(), emo.clone(), me.id.clone());
                                        rsx! { button { key: "{emo}", r#type: "button", "aria-label": "React with {emo}", onclick: move |_| { thread.write().react(&id3, &e2, &me2); on_event.call(ThreadEvent::React(id3.clone(), e2.clone())); picker.set(false) }, "{emo}" } }
                                    }
                                }
                            }
                        }
                        if confirm() {
                            {
                                let id4 = id.clone();
                                rsx! {
                                    div { class: "oxyn-cmt-conf", role: "group", "aria-label": "Delete comment?",
                                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { confirm.set(false) },
                                        span { "Delete this comment?" }
                                        button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| confirm.set(false), "Cancel" }
                                        button { class: "oxyn-diff-btn d", r#type: "button", onclick: move |_| { thread.write().delete(&id4); on_event.call(ThreadEvent::Delete(id4.clone())); }, "Delete" }
                                    }
                                }
                            }
                        }
                    }
                    if replying() {
                        {
                            let id5 = id.clone();
                            let me2 = me.clone();
                            rsx! { MentionBox { people: people.clone(), placeholder: "Reply to {c.author.name}", submit: "Reply", autofocus: true,
                                on_submit: move |v: String| { let r = thread.write().reply(Some(&id5), &me2, &v, max_depth); if let Some(nid) = r { on_event.call(ThreadEvent::Reply(nid)) } replying.set(false); folded.set(false) },
                                on_cancel: move |_| replying.set(false) } }
                        }
                    }
                }
            }
            if nrep > 0 && !folded() {
                ul { class: "oxyn-cmt-ul",
                    for r in c.replies.iter() {
                        CommentItem { key: "{r.id}", c: r.clone(), thread, me: me.clone(), people: people.clone(), reactions: reactions.clone(), max_depth, depth: depth + 1, on_event }
                    }
                }
            }
        }
    }
}

/// Threaded comments: nested replies that fold, reactions that toggle with counts, edit and
/// delete for your own comments (a deleted comment with replies stays as a placeholder),
/// `@` mentions with suggestions, and Resolve, which folds the thread into a chip.
#[component]
pub fn CommentThread(
    /// The thread.
    thread: Thread,
    /// The person writing.
    me: Person,
    /// People who can be mentioned (default: everyone in the thread and you).
    #[props(default)]
    people: Vec<Person>,
    /// Header title.
    #[props(default)]
    title: Option<String>,
    /// Emoji in the picker.
    #[props(default = vec!["👍".into(), "❤️".into(), "🎉".into(), "👀".into(), "🚀".into(), "✅".into()])]
    reactions: Vec<String>,
    /// Composer placeholder.
    #[props(default = "Add a comment…".to_string())]
    placeholder: String,
    /// Deepest reply level.
    #[props(default = 2)]
    max_depth: usize,
    /// Starts resolved.
    #[props(default)]
    default_resolved: bool,
    /// The thread changed.
    #[props(default)]
    on_change: EventHandler<(Thread, ThreadEvent)>,
    /// Resolved or reopened.
    #[props(default)]
    on_resolved: EventHandler<bool>,
) -> Element {
    let init = thread.clone();
    let mut th = use_signal(move || {
        let mut t = init;
        t.resolved = default_resolved;
        t
    });
    let tv = th.read().clone();
    let mut pp = if people.is_empty() {
        tv.people()
    } else {
        people.clone()
    };
    if !pp.iter().any(|p| p.id == me.id) {
        pp.push(me.clone());
    }
    let n = tv.count();
    let ev = move |e: ThreadEvent| on_change.call((th.peek().clone(), e));
    let me2 = me.clone();
    rsx! {
        section { class: "oxyn-cmt", "data-resolved": "{tv.resolved}", "aria-label": title.clone().unwrap_or_else(|| "Comments".into()),
            div { class: "oxyn-cmt-hd",
                div {
                    if let Some(t) = &title { h4 { class: "oxyn-cmt-t", "{t}" } }
                    span { class: "oxyn-cmt-cnt", {format!("{n} {} · {} people", if n == 1 { "comment" } else { "comments" }, tv.people().len())} }
                }
                span { class: "oxyn-diff-sp" }
                button { class: "oxyn-diff-btn", r#type: "button", "aria-pressed": "{tv.resolved}",
                    onclick: move |_| { let v = !th.peek().resolved; th.write().resolved = v; on_resolved.call(v) },
                    Icon { name: if tv.resolved { "undo" } else { "check" }, size: 12.0 } if tv.resolved { "Reopen" } else { "Resolve" }
                }
            }
            if tv.resolved {
                button { class: "oxyn-cmt-chip", r#type: "button", onclick: move |_| { th.write().resolved = false; on_resolved.call(false) },
                    Icon { name: "check-circle", size: 13.0 } "Resolved · {n} comments · Show"
                }
            } else {
                if tv.comments.is_empty() { div { class: "oxyn-cmt-empty", "No comments yet. Start the thread." } }
                ul { class: "oxyn-cmt-ul top",
                    for c in tv.comments.iter() {
                        CommentItem { key: "{c.id}", c: c.clone(), thread: th, me: me.clone(), people: pp.clone(), reactions: reactions.clone(), max_depth, depth: 0, on_event: ev }
                    }
                }
                div { class: "oxyn-cmt-new",
                    {avatar(&me)}
                    MentionBox { people: pp.clone(), placeholder: placeholder.clone(), submit: "Send",
                        on_submit: move |v: String| { let nid = th.write().reply(None, &me2, &v, max_depth); if let Some(id) = nid { on_change.call((th.peek().clone(), ThreadEvent::Reply(id))) } } }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Review center
// ---------------------------------------------------------------------------------------------

fn spans_el(sp: &[Span], flag_cursor: Option<usize>, base: usize) -> Element {
    let mut k = base;
    rsx! {
        for (i, s) in sp.iter().enumerate() {
            match s {
                Span::Text(t) => rsx! { span { key: "{i}", "{t}" } },
                Span::Flag { text, note, source } => {
                    let idx = k;
                    k += 1;
                    let open = flag_cursor == Some(idx);
                    rsx! {
                        span { key: "{i}", class: "oxyn-rvc-flag", tabindex: "0", "data-open": "{open}", "aria-describedby": "oxyn-rvc-n{idx}",
                            "{text}"
                            span { id: "oxyn-rvc-n{idx}", class: "oxyn-rvc-note", role: "tooltip",
                                "{note}" if let Some(s) = source { span { class: "oxyn-rvc-src", " · {s}" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// A focused workspace for what agents want to do: the queue sorted by risk (or oldest, or as
/// given) with All / High risk / Unsure filters, one item at a time with its reasoning, fields,
/// body, files or steps and the agent's shaky spans flagged, and Approve (A), Edit (E), Reject
/// (R, then 1–4 for a reason) and Skip (S) on single keys; J / K move, F shows the next flag's
/// note, Z undoes. Waiting times turn amber at 75 % of the service level and red past it.
#[component]
pub fn ReviewCenter(
    /// Everything waiting.
    items: Vec<ReviewItem>,
    /// Clock.
    now: ClockMs,
    /// Reviewed before (the header counts on from here).
    #[props(default)]
    reviewed: usize,
    /// Service level, ms.
    #[props(default = 1_800_000)]
    sla: u64,
    /// Below this confidence an item is Unsure.
    #[props(default = 0.7)]
    low_confidence: f32,
    /// Quick rejection reasons (1–4).
    #[props(default = REJECT_REASONS.iter().map(|s| s.to_string()).collect())]
    reasons: Vec<String>,
    /// Queue order.
    #[props(default = Order::Risk)]
    order: Order,
    /// Starting filter.
    #[props(default = Filter::All)]
    default_filter: Filter,
    /// Title.
    #[props(default = "Review center".to_string())]
    title: String,
    /// Batch "Approve all low risk" (Oxyn plus).
    #[props(default)]
    batch_low: bool,
    /// Approved or rejected.
    #[props(default)]
    on_decide: EventHandler<(String, Verdict)>,
    /// The last decision was taken back.
    #[props(default)]
    on_undo: EventHandler<String>,
    /// Skipped.
    #[props(default)]
    on_skip: EventHandler<String>,
    /// Filter changed.
    #[props(default)]
    on_filter_change: EventHandler<Filter>,
) -> Element {
    let init = items.clone();
    let mut c = use_signal(move || {
        let mut c = Center::new(init, order);
        c.low_confidence = low_confidence;
        c.reviewed_before = reviewed;
        c.set_filter(default_filter);
        c
    });
    let mut editing = use_signal(|| None::<String>);
    let mut rejecting = use_signal(|| false);
    let mut flag = use_signal(|| None::<usize>);
    let mut live = use_signal(String::new);
    let mut stamp = use_signal(|| None::<(String, &'static str)>);
    let cv = c.read().clone();
    let q: Vec<ReviewItem> = cv.queue().into_iter().cloned().collect();
    let cur = cv.current_item().cloned();
    let (done, total) = cv.progress();
    let pct = if total == 0 {
        100.0
    } else {
        done as f32 / total as f32 * 100.0
    };
    let nflags = cur.as_ref().map(|i| i.flags().len()).unwrap_or(0);
    let reasons2 = reasons.clone();
    let mut approve = move || {
        let ed = editing.peek().clone();
        let id = c.write().approve(ed.clone());
        if let Some(id) = id {
            on_decide.call((id.clone(), Verdict::Approved(ed)));
            stamp.set(Some((id, "Approved")));
            let (d, t) = c.peek().progress();
            live.set(format!("Approved. {d} of {t} reviewed"));
        }
        editing.set(None);
        flag.set(None);
    };
    let mut reject = move |r: String| {
        let id = c.write().reject(&r);
        if let Some(id) = id {
            on_decide.call((id.clone(), Verdict::Rejected(r.clone())));
            stamp.set(Some((id, "Rejected")));
            let (d, t) = c.peek().progress();
            live.set(format!("Rejected: {r}. {d} of {t} reviewed"));
        }
        rejecting.set(false);
        flag.set(None);
    };
    let mut skip = move || {
        if let Some(id) = c.write().skip() {
            on_skip.call(id);
            live.set("Skipped".into());
        }
        flag.set(None);
    };
    let mut undo = move || {
        if let Some((id, _)) = c.write().undo() {
            on_undo.call(id);
            stamp.set(None);
            live.set("Decision undone".into());
        }
    };
    let filters = [
        (Filter::All, "All"),
        (Filter::High, "High risk"),
        (Filter::Unsure, "Unsure"),
    ];
    let low_ids: Vec<String> = q
        .iter()
        .filter(|i| i.risk == oxyn_core::parity::diff::Risk::Low && !cv.unsure(i))
        .map(|i| i.id.clone())
        .collect();
    rsx! {
        section { class: "oxyn-rvc", "aria-label": "{title}", tabindex: "0",
            onkeydown: move |e: KeyboardEvent| {
                let m = e.modifiers();
                if editing.peek().is_some() {
                    if e.key() == Key::Escape { editing.set(None); }
                    if e.key() == Key::Enter && (m.meta() || m.ctrl()) { e.prevent_default(); approve(); }
                    return;
                }
                let ch = if let Key::Character(x) = e.key() { x.to_lowercase() } else { String::new() };
                if (m.meta() || m.ctrl()) && ch == "z" { e.prevent_default(); undo(); return; }
                if m.meta() || m.ctrl() || m.alt() { return; }
                if *rejecting.peek() {
                    if let Ok(n) = ch.parse::<usize>() && n >= 1 && n <= reasons2.len() { reject(reasons2[n - 1].clone()); }
                    if e.key() == Key::Escape { rejecting.set(false); }
                    return;
                }
                match (ch.as_str(), e.key()) {
                    ("a", _) => approve(),
                    ("e", _) => { e.prevent_default(); let b = c.peek().current_item().map(|i| spans_text(&i.body)); editing.set(b); }
                    ("r", _) => rejecting.set(true),
                    ("s", _) => skip(),
                    ("j", _) | (_, Key::ArrowDown) => { e.prevent_default(); c.write().step(1); flag.set(None); }
                    ("k", _) | (_, Key::ArrowUp) => { e.prevent_default(); c.write().step(-1); flag.set(None); }
                    ("f", _) => { if nflags > 0 { let n = flag.peek().map_or(0, |f| (f + 1) % nflags); flag.set(Some(n)); } }
                    ("z", _) => undo(),
                    (_, Key::Escape) => flag.set(None),
                    _ => {}
                }
            },
            header { class: "oxyn-rvc-hd",
                h4 { class: "oxyn-rvc-t", "{title}" }
                span { class: "oxyn-rvc-prog", "{done} of {total} reviewed" }
                span { class: "oxyn-rvc-bar", "aria-hidden": "true", i { style: "width:{pct}%" } }
                span { class: "oxyn-diff-sp" }
                div { class: "oxyn-rvc-tabs", role: "tablist", "aria-label": "Filter",
                    for (f, l) in filters {
                        button { key: "{f.id()}", r#type: "button", role: "tab", "aria-selected": "{cv.filter == f}",
                            onclick: move |_| { c.write().set_filter(f); on_filter_change.call(f) },
                            "{l}" span { key: "{cv.filter_count(f)}", class: "oxyn-rvc-cnt", "{cv.filter_count(f)}" }
                        }
                    }
                }
            }
            div { class: "oxyn-rvc-body",
                ul { class: "oxyn-rvc-q", "aria-label": "Queue",
                    if q.is_empty() { li { class: "oxyn-rvc-empty", "Nothing waiting" } }
                    for it in q.iter() {
                        {
                            let id = it.id.clone();
                            let is_cur = cur.as_ref().is_some_and(|x| x.id == it.id);
                            let tone = cv.sla(it, now, sla);
                            let w = waited_label(it.created, now);
                            let skipped = cv.skipped.contains(&it.id);
                            rsx! {
                                li { key: "{it.id}",
                                    button { class: "oxyn-rvc-row", r#type: "button", "aria-current": if is_cur { "true" } else { "false" }, "data-risk": it.risk.id(),
                                        "aria-label": "{it.title}, {it.kind}, {it.risk.label()}, {(it.confidence * 100.0).round()}% confidence, waiting {w}",
                                        onclick: move |_| { c.write().select(&id); flag.set(None) },
                                        span { class: "oxyn-rvc-rt", "{it.title}" }
                                        span { class: "oxyn-rvc-rs", "{it.kind}" if skipped { " · skipped" } }
                                        span { class: "oxyn-rvc-w", "data-sla": tone.id(), "{w}" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "oxyn-rvc-item",
                    match cur.clone() {
                        None => rsx! {
                            div { class: "oxyn-rvc-done", Icon { name: "check-circle", size: 22.0 } div { "All caught up" } div { class: "oxyn-rvc-sub", "{done} reviewed" }
                                if !cv.decided.is_empty() { button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| undo(), "Undo last" } }
                            }
                        },
                        Some(it) => {
                            let unsure = cv.unsure(&it);
                            let mut fbase = 0usize;
                            rsx! {
                                div { key: "{it.id}", class: "oxyn-rvc-card",
                                    div { class: "oxyn-rvc-agent",
                                        span { class: "oxyn-cmt-av", "data-agent": "true", Icon { name: "spark", size: 12.0 } }
                                        b { "{it.agent}" }
                                        if let Some(m) = &it.model { span { class: "oxyn-rvc-sub", "{m}" } }
                                        if it.risk != oxyn_core::parity::diff::Risk::Low { span { class: "oxyn-rvc-risk", "data-risk": it.risk.id(), "{it.risk.label()}" } }
                                        span { class: "oxyn-rvc-conf", "data-unsure": "{unsure}", "{(it.confidence * 100.0).round()}% sure" }
                                        span { class: "oxyn-diff-sp" }
                                        if nflags > 0 { span { class: "oxyn-rvc-fc", "{nflags} flagged · F" } }
                                    }
                                    h5 { class: "oxyn-rvc-it", "{it.title}" }
                                    if let Some(r) = &it.reasoning { p { class: "oxyn-rvc-why", "{r}" } }
                                    if !it.fields.is_empty() {
                                        dl { class: "oxyn-rvc-fields",
                                            for (fi, (l, v)) in it.fields.iter().enumerate() {
                                                {
                                                    let base = fbase;
                                                    fbase += v.iter().filter(|s| matches!(s, Span::Flag { .. })).count();
                                                    rsx! { div { key: "{fi}", dt { "{l}" } dd { {spans_el(v, flag(), base)} } } }
                                                }
                                            }
                                        }
                                    }
                                    if !it.body.is_empty() {
                                        div { class: "oxyn-rvc-bl", {it.body_label.clone().unwrap_or_else(|| "Proposal".into())} }
                                        if let Some(d) = editing() {
                                            textarea { class: "oxyn-rvc-ed", rows: "4", value: "{d}", "aria-label": "Edit proposal",
                                                onmounted: move |e| { let el = e.data(); spawn(async move { let _ = el.set_focus(true).await; }); },
                                                oninput: move |e| editing.set(Some(e.value())) }
                                            div { class: "oxyn-rvc-sub", "⌘↵ approves with your edits · Esc cancels" }
                                        } else {
                                            p { class: "oxyn-rvc-text", {spans_el(&it.body, flag(), fbase)} }
                                        }
                                    }
                                    for f in it.files.iter() {
                                        div { key: "{f.path}", class: "oxyn-rvc-file",
                                            div { class: "oxyn-rvc-fh", span { class: "oxyn-diff-path", "{f.path}" } span { class: "oxyn-diff-fstat", "{stat_label(f.diff.added, f.diff.removed)}" } }
                                            div { class: "oxyn-rvc-diff", tabindex: "0", role: "region", "aria-label": "Diff of {f.path}",
                                                for h in f.diff.hunks.iter() {
                                                    for (li, l) in h.lines.iter().enumerate() {
                                                        div { key: "{h.id}-{li}", class: "oxyn-diff-ln", "data-k": l.kind.id(),
                                                            span { class: "oxyn-diff-no", "aria-hidden": "true", {l.new.or(l.old).map(|n| n.to_string()).unwrap_or_default()} }
                                                            span { class: "oxyn-diff-sg", "aria-hidden": "true", "{l.kind.sign()}" }
                                                            span { class: "oxyn-diff-src", if l.kind != LineKind::Ctx { span { class: "oxyn-diff-sr", "{l.kind.spoken()}" } } "{l.text}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    if !it.steps.is_empty() {
                                        ol { class: "oxyn-rvc-steps",
                                            for (si, s) in it.steps.iter().enumerate() {
                                                li { key: "{si}", "data-flag": "{s.flagged}",
                                                    span { class: "oxyn-rvc-sn", "{si + 1}" } span { "{s.title}" }
                                                    if let Some(t) = &s.tool { code { class: "oxyn-rvc-tool", "{t}" } }
                                                }
                                            }
                                        }
                                    }
                                    if rejecting() {
                                        div { class: "oxyn-rvc-reasons", role: "group", "aria-label": "Reason for rejecting",
                                            for (ri, r) in reasons.iter().enumerate() {
                                                {
                                                    let r2 = r.clone();
                                                    rsx! { button { key: "{ri}", class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| reject(r2.clone()), span { class: "oxyn-diff-kbd", "{ri + 1}" } "{r}" } }
                                                }
                                            }
                                            button { class: "oxyn-diff-lnk", r#type: "button", onclick: move |_| rejecting.set(false), "Cancel" }
                                        }
                                    }
                                    div { class: "oxyn-rvc-acts",
                                        button { class: "oxyn-diff-btn p", r#type: "button", "aria-keyshortcuts": "a", onclick: move |_| approve(), Icon { name: "check", size: 12.0 } "Approve" span { class: "oxyn-diff-kbd", "A" } }
                                        button { class: "oxyn-diff-btn", r#type: "button", "aria-keyshortcuts": "e", onclick: move |_| { let b = c.peek().current_item().map(|i| spans_text(&i.body)); editing.set(b) }, Icon { name: "pencil", size: 12.0 } "Edit" span { class: "oxyn-diff-kbd", "E" } }
                                        button { class: "oxyn-diff-btn d", r#type: "button", "aria-keyshortcuts": "r", onclick: move |_| rejecting.set(true), Icon { name: "x", size: 12.0 } "Reject" span { class: "oxyn-diff-kbd", "R" } }
                                        button { class: "oxyn-diff-btn q", r#type: "button", "aria-keyshortcuts": "s", onclick: move |_| skip(), "Skip" span { class: "oxyn-diff-kbd", "S" } }
                                    }
                                }
                            }
                        },
                    }
                }
            }
            footer { class: "oxyn-rvc-ft",
                if let Some((_, w)) = stamp() {
                    span { class: "oxyn-rvc-stamp", "data-w": "{w}", if w == "Approved" { Icon { name: "check", size: 11.0 } } else { Icon { name: "x", size: 11.0 } } "{w}" }
                    button { class: "oxyn-diff-lnk", r#type: "button", "aria-keyshortcuts": "z", onclick: move |_| undo(), "Undo" }
                }
                span { class: "oxyn-diff-sp" }
                if batch_low && !low_ids.is_empty() {
                    button { class: "oxyn-diff-btn q", r#type: "button",
                        onclick: move |_| { for id in low_ids.clone() { c.write().select(&id); if let Some(x) = c.write().approve(None) { on_decide.call((x, Verdict::Approved(None))) } } live.set("Approved all low-risk items".into()); },
                        "Approve all low risk" }
                }
                span { class: "oxyn-rvc-keys", "A approve · E edit · R reject · S skip · J/K move · Z undo" }
            }
            div { class: "oxyn-diff-sr", role: "status", "aria-live": "polite", "{live}" }
        }
    }
}

// ---------------------------------------------------------------------------------------------
// Review card
// ---------------------------------------------------------------------------------------------

/// An agent's code review in one card: the verdict, open findings counted by severity, and each
/// finding with its file and line, detail, a suggested fix as a two-line diff, and Apply fix /
/// Dismiss / Reopen. The verdict follows the open findings.
#[component]
pub fn ReviewCard(
    /// Findings.
    findings: Vec<Finding>,
    /// Reviewer name.
    #[props(default = "Review agent".to_string())]
    reviewer: String,
    /// What was reviewed ("PR #48 · 6 files").
    #[props(default)]
    subject: String,
    /// Start with the first finding open.
    #[props(default = true)]
    expand_first: bool,
    /// Hide nits.
    #[props(default)]
    hide_nits: bool,
    /// A finding changed state.
    #[props(default)]
    on_change: EventHandler<(String, FindingState)>,
) -> Element {
    let init = findings.clone();
    let mut fs = use_signal(move || init);
    let first = findings.first().map(|f| f.id.clone());
    let mut open = use_signal(move || if expand_first { first } else { None });
    let mut nits = use_signal(move || !hide_nits);
    let v = fs.read().clone();
    let verdict = review_verdict(&v);
    let counts = severity_counts(&v);
    let mut set = move |id: String, st: FindingState| {
        if let Some(f) = fs.write().iter_mut().find(|f| f.id == id) {
            f.state = st;
        }
        on_change.call((id, st));
    };
    let has_nits = v
        .iter()
        .any(|f| f.severity == oxyn_core::parity::diff::Severity::Nit);
    rsx! {
        div { class: "oxyn-rvk", "data-verdict": verdict.id(),
            div { class: "oxyn-rvk-hd",
                span { class: "oxyn-cmt-av", "data-agent": "true", Icon { name: "spark", size: 12.0 } }
                div { class: "oxyn-rvk-who", b { "{reviewer}" } if !subject.is_empty() { span { class: "oxyn-rvc-sub", "{subject}" } } }
                span { class: "oxyn-diff-sp" }
                span { class: "oxyn-rvk-v", "data-verdict": verdict.id(), role: "status", "{verdict.label()}" }
            }
            div { class: "oxyn-rvk-counts",
                if counts.is_empty() { span { class: "oxyn-rvc-sub", "No open findings" } }
                for (s, n) in counts.iter() { span { key: "{s.id()}", class: "oxyn-rvk-sev", "data-s": s.id(), "{n} {s.label()}" } }
                span { class: "oxyn-diff-sp" }
                if has_nits { label { class: "oxyn-diff-viewed", input { r#type: "checkbox", checked: nits(), onchange: move |_| { let x = !nits(); nits.set(x) } } "Show nits" } }
            }
            ul { class: "oxyn-rvk-list",
                for f in v.iter().filter(|f| nits() || f.severity != oxyn_core::parity::diff::Severity::Nit) {
                    {
                        let is_open = open().as_deref() == Some(&f.id);
                        let (id1, id2, id3, id4) = (f.id.clone(), f.id.clone(), f.id.clone(), f.id.clone());
                        let st = match f.state { FindingState::Open => "open", FindingState::Applied => "applied", FindingState::Dismissed => "dismissed" };
                        rsx! {
                            li { key: "{f.id}", class: "oxyn-rvk-f", "data-state": "{st}",
                                button { class: "oxyn-rvk-fh", r#type: "button", "aria-expanded": "{is_open}",
                                    onclick: move |_| { let cur = open.peek().clone(); open.set(if cur.as_deref() == Some(&id1) { None } else { Some(id1.clone()) }) },
                                    span { class: "oxyn-rvk-sev", "data-s": f.severity.id(), "{f.severity.label()}" }
                                    span { class: "oxyn-rvk-ft", "{f.title}" }
                                    span { class: "oxyn-diff-sp" }
                                    code { class: "oxyn-rvk-loc", "{f.file}:{f.line}" }
                                    if f.state != FindingState::Open { span { class: "oxyn-diff-chip", "data-d": if f.state == FindingState::Applied { "yes" } else { "no" }, if f.state == FindingState::Applied { "Fixed" } else { "Dismissed" } } }
                                }
                                if is_open {
                                    div { class: "oxyn-rvk-body",
                                        if !f.body.is_empty() { p { "{f.body}" } }
                                        if let Some((o, n)) = &f.fix {
                                            div { class: "oxyn-rvk-fix",
                                                div { class: "oxyn-diff-ln", "data-k": "del", span { class: "oxyn-diff-sg", "−" } span { class: "oxyn-diff-src", span { class: "oxyn-diff-sr", "Removed: " } "{o}" } }
                                                div { class: "oxyn-diff-ln", "data-k": "add", span { class: "oxyn-diff-sg", "+" } span { class: "oxyn-diff-src", span { class: "oxyn-diff-sr", "Added: " } "{n}" } }
                                            }
                                        }
                                        div { class: "oxyn-rvk-acts",
                                            if f.state == FindingState::Open {
                                                if f.fix.is_some() { button { class: "oxyn-diff-btn p", r#type: "button", onclick: move |_| set(id2.clone(), FindingState::Applied), "Apply fix" } }
                                                button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| set(id3.clone(), FindingState::Dismissed), "Dismiss" }
                                            } else {
                                                button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| set(id4.clone(), FindingState::Open), "Reopen" }
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
