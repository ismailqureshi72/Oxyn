//! Composers (plan #28, chunk 4b): `RichTextEditor` (Arc Rich text editor), `AiComposer` (Arc AI
//! composer), `ChatComposer` (affectUI Chat Composer), `RunControls` (affectUI Run Controls, with
//! the send menu), `ModelMenu` (affectUI Model Picker) and `ModelPill` (Arc Model picker block).
//! Behaviour-only rebuilds; models in `oxyn_core::parity::compose`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::compose::{
        BOLD, CODE, COMMANDS, Doc, Effort, ITALIC, Kind, RingTone, RunSeg, RunStatus, STRIKE,
        SendAct, Trigger, dictated, filter, pixel_mark, price_pair, ring, send_key, snap, splice,
        stream_upto, trigger,
    },
};

use crate::composer::icons::Icon;

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

fn mss(secs: u64) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

// ─────────────────────────────────────────────────────────────── Rich text editor ──

/// Rich text editor (Arc behaviour): a small block editor drawn by Oxyn (no contentEditable).
/// Markdown shortcuts at a line start (`# `, `## `, `- `, `1. `, `[] `, `> `, ```` ``` ````, `---`),
/// ⌘B / ⌘I / ⌘⇧X / ⌘E marks, ⌘K links, a floating toolbar over a selection whose face swaps to the
/// link field, a `/` menu that filters as you type, Tab / Shift+Tab list indent, its own undo and
/// redo (⌘Z / ⌘⇧Z / ⌘Y; ⌘U is blocked), and clean HTML and Markdown out.
#[component]
pub fn RichTextEditor(
    /// Starting content as Markdown.
    #[props(default)]
    markdown: String,
    /// Shown when empty.
    #[props(default = "Start writing".to_string())]
    placeholder: String,
    /// Hint on an empty line holding the caret.
    #[props(default = "Type / for blocks".to_string())]
    block_hint: String,
    /// No editing, toolbar or shortcuts.
    #[props(default)]
    read_only: bool,
    /// Undo and redo buttons above the editor.
    #[props(default)]
    history_bar: bool,
    /// Accessible name.
    #[props(default = "Editor".to_string())]
    label: String,
    /// After each edit: HTML, Markdown and plain text.
    #[props(default)]
    on_change: EventHandler<(String, String, String)>,
) -> Element {
    let mut d = use_signal(move || {
        if markdown.is_empty() {
            Doc::empty()
        } else {
            Doc::from_markdown(&markdown)
        }
    });
    let mut focused = use_signal(|| false);
    let mut down = use_signal(|| false);
    let mut link_face = use_signal(|| None::<String>);
    let mut root = use_signal(|| None::<Rc<MountedData>>);
    let doc = d();
    let sel = doc.selection();
    let marks = doc.active_marks();
    let emit = move || {
        let x = d.peek();
        on_change.call((x.html(), x.markdown(), x.text()));
    };
    let keydown = move |e: KeyboardEvent| {
        if read_only || link_face().is_some() {
            return;
        }
        let m = e.modifiers();
        let cmd = m.meta() || m.ctrl();
        let key = e.key();
        let slash = d.peek().slash.is_some();
        let mut changed = true;
        match key {
            Key::Character(ref c) if cmd => {
                let c = c.to_lowercase();
                match c.as_str() {
                    "b" => d.write().toggle(BOLD),
                    "i" => d.write().toggle(ITALIC),
                    "e" => d.write().toggle(CODE),
                    "x" if m.shift() => d.write().toggle(STRIKE),
                    "z" if m.shift() => d.write().redo(),
                    "z" => d.write().undo(),
                    "y" => d.write().redo(),
                    "a" => {
                        d.write().select_block();
                        changed = false
                    }
                    "k" => {
                        if d.peek().selection().is_some() {
                            link_face.set(Some(d.peek().link_here().unwrap_or_default()))
                        }
                        changed = false
                    }
                    "u" => changed = false,
                    _ => return,
                }
            }
            Key::Character(ref c) => {
                for ch in c.chars() {
                    d.write().type_char(ch);
                }
            }
            Key::Enter => d.write().enter(m.shift()),
            Key::Backspace => d.write().backspace(),
            Key::Delete => {
                let mut w = d.write();
                if w.selection().is_none() {
                    w.move_h(1, true);
                }
                w.backspace();
            }
            Key::Tab if slash => d.write().slash_choose(None),
            Key::Tab => {
                if !d.write().tab(m.shift()) {
                    return;
                }
            }
            Key::ArrowUp if slash => {
                d.write().slash_move(-1);
                changed = false
            }
            Key::ArrowDown if slash => {
                d.write().slash_move(1);
                changed = false
            }
            Key::Escape if slash => {
                d.write().slash_close();
                changed = false
            }
            Key::ArrowLeft => {
                d.write().move_h(-1, m.shift());
                changed = false
            }
            Key::ArrowRight => {
                d.write().move_h(1, m.shift());
                changed = false
            }
            Key::ArrowUp => {
                d.write().move_v(-1);
                changed = false
            }
            Key::ArrowDown => {
                d.write().move_v(1);
                changed = false
            }
            Key::Home => {
                d.write().edge(false, m.shift());
                changed = false
            }
            Key::End => {
                d.write().edge(true, m.shift());
                changed = false
            }
            _ => return,
        }
        e.prevent_default();
        if changed {
            emit();
        }
    };
    let slash_items = doc.slash_items();
    let empty = doc.is_empty();
    rsx! {
        div { class: "oxk-rte-wrap",
            if history_bar {
                div { class: "oxk-hist",
                    button { class: "oxk-ib", r#type: "button", disabled: !doc.can_undo(), "aria-label": "Undo", onclick: move |_| { d.write().undo(); emit(); }, Icon { name: "undo", size: 13.0 } }
                    button { class: "oxk-ib flip", r#type: "button", disabled: !doc.can_redo(), "aria-label": "Redo", onclick: move |_| { d.write().redo(); emit(); }, Icon { name: "undo", size: 13.0 } }
                    span { class: "oxk-histn", if doc.can_undo() { "Edited" } else { "No changes" } }
                }
            }
            div { class: "oxk-rte", tabindex: "0", role: "textbox", "aria-multiline": "true", "aria-label": "{label}",
                "aria-readonly": "{read_only}", "data-focus": "{focused()}",
                onmounted: move |e| root.set(Some(e.data())),
                onfocus: move |_| focused.set(true),
                onblur: move |_| { focused.set(false); down.set(false) },
                onkeydown: keydown,
                onmouseup: move |_| down.set(false),
                if empty && !focused() { span { class: "oxk-ph", "{placeholder}" } }
                for (li, l) in doc.lines.iter().enumerate() {
                    {
                        let is_caret = doc.caret.0 == li;
                        let n = l.chars.len();
                        let tag = match l.kind { Kind::H1 => "h1", Kind::H2 => "h2", Kind::H3 => "h3", Kind::Quote => "quote", Kind::Code => "code", Kind::Bullet => "ul", Kind::Numbered => "ol", Kind::Todo => "todo", Kind::Divider => "hr", Kind::P => "p" };
                        let num = doc.lines[..=li].iter().rev().take_while(|x| x.kind == Kind::Numbered).count();
                        let checked = l.checked;
                        let sel_here = if is_caret { sel } else { None };
                        let chars: Vec<(usize, char, String)> = (0..n).map(|i| {
                            let m = l.marks[i];
                            let mut c = String::from("oxk-ch");
                            for (bit, k) in [(BOLD, " b"), (ITALIC, " i"), (STRIKE, " s"), (CODE, " c")] { if m & bit != 0 { c.push_str(k) } }
                            if l.link[i] > 0 { c.push_str(" a") }
                            if sel_here.is_some_and(|s| i >= s.0 && i < s.1) { c.push_str(" sel") }
                            (i, l.chars[i], c)
                        }).collect();
                        let caret_at = if is_caret && focused() && sel_here.is_none() { Some(doc.caret.1) } else { None };
                        let tb_left = sel_here.map_or(0, |s| (s.0 * 7).min(260));
                        rsx! {
                            div { key: "{li}", class: "oxk-line {tag}", style: "--ind:{l.indent}",
                                onmousedown: move |e: MouseEvent| { e.prevent_default(); focus(root()); d.write().place(li, n); down.set(true); },
                                if l.kind == Kind::Bullet { span { class: "oxk-mk", "•" } }
                                if l.kind == Kind::Numbered { span { class: "oxk-mk", "{num}." } }
                                if l.kind == Kind::Todo {
                                    span { class: if checked { "oxk-todo on" } else { "oxk-todo" }, onmousedown: move |e: MouseEvent| { e.stop_propagation(); e.prevent_default(); d.write().check(li); emit(); },
                                        if checked { Icon { name: "check", size: 10.0 } } }
                                }
                                if l.kind == Kind::Divider { span { class: "oxk-hr" } }
                                span { class: if checked { "oxk-txt done" } else { "oxk-txt" },
                                    for (i, c, cls) in chars.iter().cloned() {
                                        if caret_at == Some(i) { span { class: "oxk-caret" } }
                                        span { class: "{cls}",
                                            onmousedown: move |e: MouseEvent| { e.stop_propagation(); e.prevent_default(); focus(root()); d.write().place(li, i); down.set(true); },
                                            onmouseenter: move |_| if down() { d.write().drag_to(li, i + 1) },
                                            if c == '\n' { br {} } else { "{c}" }
                                        }
                                    }
                                    if caret_at == Some(n) { span { class: "oxk-caret" } }
                                    if n == 0 && is_caret && focused() && l.kind == Kind::P && !empty { span { class: "oxk-hint", "{block_hint}" } }
                                    if n == 0 && empty && focused() { span { class: "oxk-hint", "{block_hint}" } }
                                }
                                if sel_here.is_some() && !read_only {
                                    div { class: "oxk-tb", style: "left:{tb_left}px", onmousedown: move |e: MouseEvent| { e.stop_propagation(); e.prevent_default(); },
                                        if let Some(href) = link_face() {
                                            div { class: "oxk-face",
                                                Icon { name: "link", size: 12.0 }
                                                input { class: "oxk-linkin", r#type: "url", value: "{href}", placeholder: "Paste a link", "aria-label": "Link address",
                                                    onmounted: move |e| { let el = e.data(); spawn(async move { let _ = el.set_focus(true).await; }); },
                                                    onmousedown: move |e: MouseEvent| e.stop_propagation(),
                                                    oninput: move |e| link_face.set(Some(e.value())),
                                                    onkeydown: move |e: KeyboardEvent| match e.key() {
                                                        Key::Enter => { e.prevent_default(); let h = link_face().unwrap_or_default(); d.write().set_link(&h); link_face.set(None); emit(); focus(root()); }
                                                        Key::Escape => { e.prevent_default(); link_face.set(None); focus(root()); }
                                                        _ => {}
                                                    },
                                                }
                                            }
                                        } else {
                                            div { class: "oxk-face",
                                                button { class: if marks & BOLD != 0 { "oxk-tbb on" } else { "oxk-tbb" }, r#type: "button", "aria-pressed": "{marks & BOLD != 0}", "aria-label": "Bold", onclick: move |_| { d.write().toggle(BOLD); emit() }, b { "B" } }
                                                button { class: if marks & ITALIC != 0 { "oxk-tbb on" } else { "oxk-tbb" }, r#type: "button", "aria-pressed": "{marks & ITALIC != 0}", "aria-label": "Italic", onclick: move |_| { d.write().toggle(ITALIC); emit() }, i { "I" } }
                                                button { class: if marks & STRIKE != 0 { "oxk-tbb on" } else { "oxk-tbb" }, r#type: "button", "aria-pressed": "{marks & STRIKE != 0}", "aria-label": "Strikethrough", onclick: move |_| { d.write().toggle(STRIKE); emit() }, s { "S" } }
                                                button { class: if marks & CODE != 0 { "oxk-tbb on" } else { "oxk-tbb" }, r#type: "button", "aria-pressed": "{marks & CODE != 0}", "aria-label": "Inline code", onclick: move |_| { d.write().toggle(CODE); emit() }, Icon { name: "code", size: 12.0 } }
                                                span { class: "oxk-tbsep" }
                                                button { class: "oxk-tbb", r#type: "button", "aria-label": "Link", onclick: move |_| link_face.set(Some(d.peek().link_here().unwrap_or_default())), Icon { name: "link", size: 12.0 } }
                                            }
                                        }
                                    }
                                }
                                if is_caret && doc.slash.is_some() {
                                    div { class: "oxk-slash", role: "listbox", "aria-label": "Blocks", onmousedown: move |e: MouseEvent| { e.stop_propagation(); e.prevent_default(); },
                                        if !slash_items.is_empty() {
                                            span { class: "oxk-shl", style: "transform:translateY({doc.slash.as_ref().map_or(0, |s| s.active) * 30}px)" }
                                        }
                                        for (si, (id, lab, _)) in slash_items.iter().enumerate() {
                                            {
                                            let ic = match *id { "h1" => "H1", "h2" => "H2", "h3" => "H3", "bullet" => "•", "numbered" => "1.", "todo" => "☐", "quote" => "❝", "code" => "</>", _ => "T" };
                                            rsx! {
                                            div { key: "{id}", class: "oxk-sopt", role: "option", "aria-selected": "{doc.slash.as_ref().is_some_and(|s| s.active == si)}",
                                                onmousedown: move |e: MouseEvent| { e.stop_propagation(); e.prevent_default(); d.write().slash_choose(Some(si)); emit(); },
                                                span { class: "oxk-sic", "{ic}" }
                                                "{lab}"
                                            }
                                            }
                                            }
                                        }
                                        if slash_items.is_empty() { div { class: "oxk-sempty", "No blocks match" } }
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

// ─────────────────────────────────────────────────────────────── AI composer ──

/// A piece of the AI composer's message: plain text or an atomic token.
#[derive(Clone, Debug, PartialEq)]
enum Seg {
    Text(String),
    Cmd(String),
    Mention(String),
}

/// AI composer (Arc behaviour): the prompt box of an AI app. `/` at a line start opens commands,
/// `@` after a space opens files and docs; Arrow keys move through the list while the field keeps
/// focus and Enter or Tab inserts the choice as one atomic token (Backspace removes it whole).
/// The model and effort menu sits in the toolbar; a ring counter appears at 80 % of the limit and
/// warns at 95 %, and sending stops above it. Sending lifts the message into the thread; a
/// Thinking shimmer covers the moment before the first token, the reply streams in whole-word
/// chunks, and the send arrow becomes a stop square ringed by the reply's progress.
#[component]
pub fn AiComposer(
    /// Heading of the new chat state.
    #[props(default = "What should we work on?".to_string())]
    greeting: String,
    /// Starters under the composer: label and prompt.
    #[props(default)]
    suggestions: Vec<(String, String)>,
    /// Models: id, short name, note.
    #[props(default)]
    models: Vec<(String, String, String)>,
    /// Files and docs for `@`.
    #[props(default)]
    mentions: Vec<String>,
    /// Placeholder.
    #[props(default = "Ask anything, / for commands, @ to add context".to_string())]
    placeholder: String,
    /// Character limit.
    #[props(default = 4000)]
    max_length: usize,
    /// Starting text.
    #[props(default)]
    text: String,
    /// The reply the demo streams back.
    #[props(default = "Here's a plan: add a token bucket per API key in the gateway, return 429 with Retry-After when it empties, and cover it with two integration tests. I'll start with the middleware.".to_string())]
    reply: String,
    /// Shared clock (streaming).
    now: ClockMs,
    /// Sent: plain text with tokens written as /cmd and @name, and the model id.
    #[props(default)]
    on_send: EventHandler<(String, String)>,
) -> Element {
    let mut segs = use_signal(Vec::<Seg>::new);
    let mut draft = use_signal(move || text.clone());
    let mut active = use_signal(|| 0usize);
    let mut closed = use_signal(|| false);
    let mut model = use_signal(|| 0usize);
    let mut model_open = use_signal(|| false);
    let mut effort = use_signal(|| Effort::Medium);
    let mut web = use_signal(|| false);
    let mut sent = use_signal(|| None::<(String, ClockMs)>);
    let mut stopped = use_signal(|| None::<usize>);
    let mut field = use_signal(|| None::<Rc<MountedData>>);
    let t = draft();
    let trig = if closed() { None } else { trigger(&t) };
    let cmd_names: Vec<&str> = COMMANDS.iter().map(|c| c.0).collect();
    let men: Vec<&str> = mentions.iter().map(String::as_str).collect();
    let options: Vec<String> = match &trig {
        Some((Trigger::Slash, _, q)) => filter(&cmd_names, q)
            .into_iter()
            .map(str::to_string)
            .collect(),
        Some((Trigger::Mention, _, q)) => filter(&men, q).into_iter().map(str::to_string).collect(),
        None => vec![],
    };
    let seg_len: usize = segs()
        .iter()
        .map(|s| match s {
            Seg::Text(x) => x.chars().count(),
            Seg::Cmd(x) | Seg::Mention(x) => x.chars().count() + 1,
        })
        .sum();
    let len = seg_len + t.chars().count();
    let r = ring(len, max_length);
    let over = r.is_some_and(|x| x.1 == RingTone::Over);
    let empty = len == 0;
    // Reply state on the clock.
    let reply_view = sent().map(|(_, at)| {
        let el = now.since(at).as_millis() as u64;
        let think = if web() { 1200 } else { 500 };
        if el < think {
            (0usize, true, false)
        } else {
            let upto = stopped().unwrap_or_else(|| stream_upto(&reply, el - think));
            (upto, false, upto >= reply.len() || stopped().is_some())
        }
    });
    let streaming = reply_view.is_some_and(|v| !v.2);
    let prog = reply_view.map_or(0.0, |v| v.0 as f32 / reply.len().max(1) as f32);
    let mut insert = move |choice: String| {
        let d = draft();
        let Some((kind, at, _)) = trigger(&d) else {
            return;
        };
        let mut s = segs.write();
        let head = d[..at].to_string();
        if !head.is_empty() {
            s.push(Seg::Text(head));
        }
        s.push(match kind {
            Trigger::Slash => Seg::Cmd(choice),
            Trigger::Mention => Seg::Mention(choice),
        });
        draft.set(" ".into());
        active.set(0);
    };
    let model_ids = use_signal(|| models.iter().map(|m| m.0.clone()).collect::<Vec<_>>());
    let mut send = move || {
        if over || streaming {
            return;
        }
        let mut out = String::new();
        for s in segs() {
            match s {
                Seg::Text(x) => out.push_str(&x),
                Seg::Cmd(x) => out.push_str(&format!("/{x}")),
                Seg::Mention(x) => out.push_str(&format!("@{x}")),
            }
        }
        out.push_str(&draft());
        let out = out.trim().to_string();
        if out.is_empty() {
            return;
        }
        on_send.call((
            out.clone(),
            model_ids.peek().get(model()).cloned().unwrap_or_default(),
        ));
        sent.set(Some((out, now)));
        stopped.set(None);
        segs.set(vec![]);
        draft.set(String::new());
    };
    let n_opts = options.len();
    let opts_k = options.clone();
    let seg_list = segs();
    let is_slash = matches!(trig, Some((Trigger::Slash, _, _)));
    let opt_rows: Vec<(usize, String, &str)> = options
        .iter()
        .enumerate()
        .map(|(i, o)| {
            (
                i,
                o.clone(),
                COMMANDS
                    .iter()
                    .find(|c| c.0 == o.as_str())
                    .map_or("", |c| c.1),
            )
        })
        .collect();
    let cur_model = models.get(model()).cloned().unwrap_or_default();
    rsx! {
        div { class: "oxk-ai",
            if let Some((msg, _)) = sent() {
                div { class: "oxk-thread",
                    div { class: "oxk-you", "{msg}" }
                    if let Some((upto, thinking, done)) = reply_view {
                        div { class: "oxk-bot",
                            if thinking {
                                span { class: "oxk-shim", if web() { "Searching the web…" } else { "Thinking…" } }
                            } else {
                                if web() { span { class: "oxk-step", Icon { name: "globe", size: 11.0 } "Searched 6 sites" } }
                                p { class: "oxk-reply", "{&reply[..upto]}" if !done { span { class: "oxk-cur" } } }
                                if stopped().is_some() { span { class: "oxk-step", "Stopped" } }
                            }
                        }
                    }
                }
            } else {
                h2 { class: "oxk-greet", "{greeting}" }
            }
            div { class: "oxk-box", "data-over": "{over}",
                div { class: "oxk-field", onclick: move |_| focus(field()),
                    for (si, s) in seg_list.iter().enumerate() {
                        {match s {
                            Seg::Text(x) => rsx! { span { key: "{si}", class: "oxk-frozen", "{x}" } },
                            Seg::Cmd(x) => rsx! { span { key: "{si}", class: "oxk-tok cmd", "/{x}" } },
                            Seg::Mention(x) => rsx! { span { key: "{si}", class: "oxk-tok", Icon { name: "file", size: 10.0 } "{x}" } },
                        }}
                    }
                    textarea { class: "oxk-ta", rows: "1", value: "{t}",
                        placeholder: if seg_list.is_empty() { "{placeholder}" },
                        style: "--rows:{t.lines().count().clamp(1, 8)}",
                        "aria-label": "Message", "aria-expanded": "{!options.is_empty()}", "aria-controls": "oxk-ai-list",
                        "aria-activedescendant": if !options.is_empty() { "oxk-ai-o{active()}" },
                        onmounted: move |e| field.set(Some(e.data())),
                        oninput: move |e| { draft.set(e.value()); closed.set(false); active.set(0) },
                        onkeydown: move |e: KeyboardEvent| {
                            let open = n_opts > 0;
                            match e.key() {
                                Key::ArrowDown if open => { e.prevent_default(); active.set((active() + 1) % n_opts) }
                                Key::ArrowUp if open => { e.prevent_default(); active.set((active() + n_opts - 1) % n_opts) }
                                Key::Enter | Key::Tab if open => { e.prevent_default(); if let Some(c) = opts_k.get(active()).cloned() { insert(c) } }
                                Key::Escape if open => { e.prevent_default(); closed.set(true) }
                                Key::Enter if !e.modifiers().shift() => { e.prevent_default(); send() }
                                Key::Backspace if draft().is_empty() => {
                                    let last = segs.write().pop();
                                    if let Some(Seg::Text(x)) = last { draft.set(x) }
                                }
                                _ => {}
                            }
                        },
                    }
                }
                if !options.is_empty() {
                    div { id: "oxk-ai-list", class: "oxk-pop", role: "listbox",
                        span { class: "oxk-plabel", if is_slash { "Commands" } else { "Files and docs" } }
                        for (i, o, hint) in opt_rows.iter().cloned() {
                            div { key: "{o}", id: "oxk-ai-o{i}", class: "oxk-opt", role: "option", "aria-selected": "{active() == i}",
                                onmousedown: { let o = o.clone(); move |e: MouseEvent| { e.prevent_default(); insert(o.clone()) } },
                                onmouseenter: move |_| active.set(i),
                                if is_slash {
                                    span { class: "oxk-on", "/{o}" }
                                    span { class: "oxk-oh", "{hint}" }
                                } else {
                                    Icon { name: "file", size: 11.0 }
                                    span { class: "oxk-on", "{o}" }
                                }
                            }
                        }
                    }
                }
                div { class: "oxk-bar",
                    button { class: "oxk-ib", r#type: "button", "aria-label": "Attach", Icon { name: "plus", size: 14.0 } }
                    div { class: "oxk-menuwrap",
                        button { class: "oxk-chip", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{model_open()}",
                            onclick: move |_| model_open.set(!model_open()),
                            Icon { name: "spark", size: 11.0 } "{cur_model.1}" span { class: "oxk-eff", "· {effort().label()}" } Icon { name: "chevron", size: 10.0 }
                        }
                        if model_open() {
                            div { class: "oxk-menu", role: "menu",
                                for (mi, m) in models.iter().enumerate() {
                                    button { key: "{m.0}", class: "oxk-mi", role: "menuitemradio", "aria-checked": "{model() == mi}", r#type: "button",
                                        onclick: move |_| { model.set(mi); },
                                        span { class: "oxk-on", "{m.1}" } span { class: "oxk-oh", "{m.2}" }
                                        if model() == mi { Icon { name: "check", size: 11.0 } }
                                    }
                                }
                                div { class: "oxk-msep" }
                                div { class: "oxk-effrow", role: "radiogroup", "aria-label": "Effort",
                                    for e in [Effort::Low, Effort::Medium, Effort::High] {
                                        button { key: "{e.id()}", class: if effort() == e { "oxk-seg on" } else { "oxk-seg" }, r#type: "button", role: "radio", "aria-checked": "{effort() == e}", onclick: move |_| effort.set(e), "{e.label()}" }
                                    }
                                }
                                button { class: "oxk-mi", role: "menuitemcheckbox", "aria-checked": "{web()}", r#type: "button", onclick: move |_| web.set(!web()),
                                    Icon { name: "globe", size: 11.0 } span { class: "oxk-on", "Web search" } if web() { Icon { name: "check", size: 11.0 } }
                                }
                            }
                        }
                    }
                    span { class: "oxk-sp" }
                    if let Some((f, tone)) = r {
                        span { class: "oxk-ringc", "data-tone": "{tone:?}", title: "{len} / {max_length}",
                            svg { view_box: "0 0 20 20", width: "18", height: "18", "aria-hidden": "true",
                                circle { cx: "10", cy: "10", r: "8", fill: "none", stroke_width: "2.5", class: "oxk-rt" }
                                circle { cx: "10", cy: "10", r: "8", fill: "none", stroke_width: "2.5", class: "oxk-rf",
                                    stroke_dasharray: "{f * 50.27} 50.27", transform: "rotate(-90 10 10)" }
                            }
                            if tone != RingTone::Quiet { span { class: "oxk-left", "{max_length as i64 - len as i64}" } }
                        }
                    }
                    button { class: if streaming { "oxk-send stop" } else { "oxk-send" }, r#type: "button",
                        disabled: !streaming && (empty || over),
                        "aria-label": if streaming { "Stop" } else { "Send" },
                        onclick: move |_| if streaming { stopped.set(Some(reply_view.map_or(0, |v| v.0))) } else { send() },
                        if streaming {
                            svg { class: "oxk-prog", view_box: "0 0 36 36", "aria-hidden": "true",
                                circle { cx: "18", cy: "18", r: "16", fill: "none", stroke_width: "2", stroke_dasharray: "{prog * 100.53} 100.53", transform: "rotate(-90 18 18)" }
                            }
                            span { class: "oxk-sq" }
                        } else {
                            span { class: "oxk-arrow", Icon { name: "up", size: 14.0 } }
                        }
                    }
                }
            }
            if sent().is_none() && !suggestions.is_empty() {
                div { class: "oxk-sugg",
                    for (si, (l, p)) in suggestions.iter().enumerate() {
                        {
                            let p = p.clone();
                            rsx! {
                                button { key: "{si}", class: "oxk-sg", r#type: "button",
                                    onclick: move |_| {
                                        // Known commands at the start become tokens.
                                        let mut rest = p.clone();
                                        segs.set(vec![]);
                                        if let Some(c) = COMMANDS.iter().find(|c| rest.starts_with(&format!("/{} ", c.0))) {
                                            segs.write().push(Seg::Cmd(c.0.to_string()));
                                            rest = rest[c.0.len() + 1..].to_string();
                                        }
                                        draft.set(rest);
                                        focus(field());
                                    },
                                    Icon { name: match si { 0 => "text", 1 => "pencil", _ => "alert" }, size: 12.0 }
                                    "{l}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────── Chat composer ──

/// Chat composer (affectUI behaviour): a prompt card with a model picker, voice input whose demo
/// transcript appears word by word in grey while recording, a "Work in a folder" picker (picking
/// the chosen folder again leaves it), a "Connect apps" menu whose first three marks show in the
/// row, and a Request approval button with a loading state.
#[component]
pub fn ChatComposer(
    /// Placeholder.
    #[props(default = "Start by typing...".to_string())]
    placeholder: String,
    /// Model names; the first is selected.
    #[props(default = vec!["Auto".to_string(), "Claude".to_string(), "ChatGPT".to_string()])]
    models: Vec<String>,
    /// Loading time of Request approval, ms.
    #[props(default = 1300)]
    load_ms: u64,
    /// Demo transcript for dictation.
    #[props(default = "Draft a short update for the team about the launch moving to Thursday".to_string())]
    dictation: String,
    /// Folders; empty leaves the picker out.
    #[props(default = vec!["oxyn".to_string(), "website".to_string(), "research".to_string()])]
    folders: Vec<String>,
    /// Starting folder.
    #[props(default)]
    folder: Option<String>,
    /// Apps: id and name.
    #[props(default = vec![("gmail".to_string(), "Gmail".to_string()), ("slack".to_string(), "Slack".to_string()), ("clickup".to_string(), "ClickUp".to_string())])]
    apps: Vec<(String, String)>,
    /// Connected app ids at the start.
    #[props(default = vec!["gmail".to_string()])]
    connected: Vec<String>,
    /// Shared clock (dictation and loading).
    now: ClockMs,
    /// Sent message and model.
    #[props(default)]
    on_send: EventHandler<(String, String)>,
    /// Folder picked (`None` when left).
    #[props(default)]
    on_folder_change: EventHandler<Option<String>>,
    /// Connected app ids.
    #[props(default)]
    on_apps_change: EventHandler<Vec<String>>,
) -> Element {
    let mut text = use_signal(String::new);
    let mut model = use_signal(|| 0usize);
    let mut open = use_signal(|| "");
    let mut rec = use_signal(|| None::<ClockMs>);
    let mut fold = use_signal(move || folder.clone());
    let mut conn = use_signal(move || connected.clone());
    let mut approval = use_signal(|| None::<ClockMs>);
    let mut last = use_signal(|| None::<String>);
    let heard = rec().map(|t| dictated(&dictation, now.since(t).as_millis() as u64, 170));
    let appr = approval().map(|t| now.since(t).as_millis() as u64 >= load_ms);
    let menu = open();
    let dict2 = dictation.clone();
    let model_names = use_signal(|| models.clone());
    let mut send = move || {
        let t = text().trim().to_string();
        if t.is_empty() {
            return;
        }
        on_send.call((
            t.clone(),
            model_names.peek().get(model()).cloned().unwrap_or_default(),
        ));
        last.set(Some(t));
        text.set(String::new());
    };
    rsx! {
        div { class: "oxk-cc",
            if let Some(l) = last() { div { class: "oxk-sent", Icon { name: "check", size: 11.0 } "Sent to {models.get(model()).cloned().unwrap_or_default()}: {l}" } }
            div { class: "oxk-card",
                div { class: "oxk-ccfield",
                    textarea { class: "oxk-ta", rows: "2", value: "{text}", placeholder: "{placeholder}", "aria-label": "Message",
                        oninput: move |e| text.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); send() },
                    }
                    if let Some(h) = &heard { span { class: "oxk-heard", "aria-live": "polite", "{h}" span { class: "oxk-cur" } } }
                }
                div { class: "oxk-bar",
                    button { class: "oxk-ib", r#type: "button", "aria-label": "Attach", Icon { name: "plus", size: 14.0 } }
                    div { class: "oxk-menuwrap",
                        button { class: "oxk-chip", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{menu == \"model\"}",
                            onclick: move |_| open.set(if open() == "model" { "" } else { "model" }),
                            "{models.get(model()).cloned().unwrap_or_default()}" Icon { name: "chevron", size: 10.0 }
                        }
                        if menu == "model" {
                            div { class: "oxk-menu", role: "menu",
                                for (i, m) in models.iter().enumerate() {
                                    button { key: "{m}", class: "oxk-mi", role: "menuitemradio", "aria-checked": "{model() == i}", r#type: "button",
                                        onclick: move |_| { model.set(i); open.set("") },
                                        span { class: "oxk-on", if crate::brand::has_brand(m) { crate::brand::BrandIcon { name: m.clone(), size: 13.0 } } "{m}" } if model() == i { Icon { name: "check", size: 11.0 } }
                                    }
                                }
                            }
                        }
                    }
                    span { class: "oxk-sp" }
                    button { class: if rec().is_some() { "oxk-ib rec" } else { "oxk-ib" }, r#type: "button", "aria-pressed": "{rec().is_some()}",
                        "aria-label": if rec().is_some() { "Stop dictation" } else { "Dictate" },
                        onclick: move |_| match rec() {
                            Some(t) => { let h = dictated(&dict2, now.since(t).as_millis() as u64, 170); if !h.is_empty() { text.set(format!("{}{}{h}", text(), if text().is_empty() { "" } else { " " })) } rec.set(None) }
                            None => rec.set(Some(now)),
                        },
                        Icon { name: "mic", size: 14.0 }
                    }
                    button { class: "oxk-send", r#type: "button", disabled: text().trim().is_empty(), "aria-label": "Send", onclick: move |_| send(),
                        span { class: "oxk-arrow", Icon { name: "up", size: 14.0 } }
                    }
                }
            }
            div { class: "oxk-under",
                if !folders.is_empty() {
                    div { class: "oxk-menuwrap",
                        button { class: "oxk-chip quiet", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{menu == \"folder\"}",
                            onclick: move |_| open.set(if open() == "folder" { "" } else { "folder" }),
                            Icon { name: "folder", size: 12.0 }
                            {match fold() { Some(f) => rsx! { "{f}" }, None => rsx! { "Work in a folder" } }}
                        }
                        if menu == "folder" {
                            div { class: "oxk-menu up", role: "menu",
                                for f in folders.iter() {
                                    {
                                        let f2 = f.clone();
                                        let on = fold().as_deref() == Some(f.as_str());
                                        rsx! {
                                            button { key: "{f}", class: "oxk-mi", role: "menuitemradio", "aria-checked": "{on}", r#type: "button",
                                                onclick: move |_| { let n = if fold().as_deref() == Some(f2.as_str()) { None } else { Some(f2.clone()) }; fold.set(n.clone()); on_folder_change.call(n); open.set("") },
                                                Icon { name: "folder", size: 11.0 } span { class: "oxk-on", "{f}" } if on { Icon { name: "check", size: 11.0 } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                button { class: "oxk-chip quiet", r#type: "button", disabled: approval().is_some() && appr == Some(false),
                    onclick: move |_| approval.set(Some(now)),
                    {match appr {
                        Some(false) => rsx! { span { class: "oxk-spin" } "Requesting…" },
                        Some(true) => rsx! { Icon { name: "check", size: 12.0 } "Approval requested" },
                        None => rsx! { Icon { name: "hand", size: 12.0 } "Request approval" },
                    }}
                }
                span { class: "oxk-sp" }
                div { class: "oxk-menuwrap",
                    button { class: "oxk-chip quiet", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{menu == \"apps\"}",
                        onclick: move |_| open.set(if open() == "apps" { "" } else { "apps" }),
                        span { class: "oxk-marks",
                            for (id, name) in apps.iter().take(3) {
                                span { key: "{id}", class: "oxk-appmark", "data-on": "{conn().contains(id)}", if crate::brand::has_brand(name) { crate::brand::BrandIcon { name: name.clone(), size: 10.0 } } else { "{name.chars().next().unwrap_or('?')}" } }
                            }
                        }
                        "Connect apps"
                    }
                    if menu == "apps" {
                        div { class: "oxk-menu up end", role: "menu",
                            for (id, name) in apps.iter() {
                                {
                                    let id2 = id.clone();
                                    let on = conn().contains(id);
                                    rsx! {
                                        button { key: "{id}", class: "oxk-mi", role: "menuitemcheckbox", "aria-checked": "{on}", r#type: "button",
                                            onclick: move |_| { let mut c = conn.write(); if let Some(p) = c.iter().position(|x| *x == id2) { c.remove(p); } else { c.push(id2.clone()); } on_apps_change.call(c.clone()); },
                                            span { class: "oxk-appmark", "data-on": "{on}", if crate::brand::has_brand(name) { crate::brand::BrandIcon { name: name.clone(), size: 10.0 } } else { "{name.chars().next().unwrap_or('?')}" } }
                                            span { class: "oxk-on", "{name}" }
                                            span { class: if on { "oxk-sw on" } else { "oxk-sw" }, span {} }
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

// ─────────────────────────────────────────────────────────────── Run controls ──

/// Run controls (affectUI behaviour, plus a send menu): every control a running agent needs in
/// one composer. Running shows the streaming answer with a caret and Stop; stopped and error show
/// Resume (from the word it stopped at), Retry and Dismiss; done shows "Done in 0:14" and Retry.
/// While running, Enter steers (the note is drawn into the answer at the character it landed on)
/// and ⌥ Enter queues (messages wait above the composer, × removes one, Run next sends the head
/// once the run is done). The send button's chevron opens the same choices as a menu. Chat only
/// answers; Act may change things and carries the permission menu (Full access turns red). The
/// context ring turns amber from 80 %.
#[component]
pub fn RunControls(
    /// Agent name.
    #[props(default = "Agent".to_string())]
    agent: String,
    /// Agent id (its pixel mark).
    #[props(default = "agent".to_string())]
    agent_id: String,
    /// What was asked.
    #[props(default)]
    prompt: String,
    /// The full answer, streamed on the clock.
    #[props(default)]
    answer: String,
    /// Starting status: `idle` · `running` · `stopped` · `error` · `done`.
    #[props(default = "idle".to_string())]
    status: String,
    /// Queued messages at the start.
    #[props(default)]
    queue: Vec<String>,
    /// `chat` · `act`.
    #[props(default = "act".to_string())]
    mode: String,
    /// `ask` · `edits` · `full`.
    #[props(default = "ask".to_string())]
    permission: String,
    /// Share of the context window in use.
    #[props(default = 0.42)]
    context: f32,
    /// Placeholder.
    #[props(default = "Ask the agent…".to_string())]
    placeholder: String,
    /// Shared clock.
    now: ClockMs,
    /// Send, steer, queue, stop, resume, retry and dismiss, with any text.
    #[props(default)]
    on_action: EventHandler<(String, String)>,
) -> Element {
    let st0 = RunStatus::from_id(&status);
    let full_chars = answer.chars().count();
    let ans = use_signal(move || answer.clone());
    let mut st = use_signal(move || st0);
    let mut q = use_signal(move || {
        queue
            .iter()
            .enumerate()
            .map(|(i, t)| (i as u64, t.clone()))
            .collect::<Vec<_>>()
    });
    let mut ask = use_signal(move || prompt.clone());
    // Characters streamed before the current stretch, and when the stretch began.
    let mut base = use_signal(move || match st0 {
        RunStatus::Done => usize::MAX,
        RunStatus::Stopped | RunStatus::Error => full_chars * 2 / 5,
        _ => 0,
    });
    let mut since = use_signal(move || now);
    let mut steers = use_signal(Vec::<(usize, String)>::new);
    let mut text = use_signal(String::new);
    let mut md = use_signal(move || mode.clone());
    let mut perm = use_signal(move || permission.clone());
    let mut menu = use_signal(|| "");
    let mut took = use_signal(|| None::<u64>);
    let streamed = move |st: RunStatus, base: usize, since: ClockMs| -> usize {
        match st {
            RunStatus::Running => {
                let a = ans.peek();
                let chunk = &a[a
                    .char_indices()
                    .nth(base.min(full_chars))
                    .map_or(a.len(), |x| x.0)..];
                let b = stream_upto(chunk, now.since(since).as_millis() as u64 * 2 / 5);
                (base + chunk[..b].chars().count()).min(full_chars)
            }
            RunStatus::Idle => 0,
            _ => base.min(full_chars),
        }
    };
    let shown = streamed(st(), base(), since());
    let eff = if st() == RunStatus::Running && shown >= full_chars {
        RunStatus::Done
    } else {
        st()
    };
    let took_s = took().unwrap_or_else(|| now.since(since()).as_secs());
    let segs = splice(
        &ans.peek().chars().take(shown).collect::<String>(),
        &steers(),
    );
    let ql = q();
    let act = md() == "act";
    let ctx_warn = context >= 0.8;
    let mark = pixel_mark(&agent_id);
    let fire = move |what: &str, t: String| on_action.call((what.to_string(), t));
    let mut submit = move |alt: bool, forced: Option<SendAct>| {
        let t = text().trim().to_string();
        let cur = streamed(st(), base(), since());
        let effective = if st() == RunStatus::Running && cur >= full_chars {
            RunStatus::Done
        } else {
            st()
        };
        let a = forced.unwrap_or_else(|| send_key(effective, alt));
        if t.is_empty() {
            return;
        }
        match a {
            SendAct::Send => {
                ask.set(t.clone());
                steers.set(vec![]);
                base.set(0);
                since.set(now);
                took.set(None);
                st.set(RunStatus::Running);
                fire("send", t);
            }
            SendAct::Steer => {
                steers.write().push((cur, t.clone()));
                fire("steer", t);
            }
            SendAct::Queue => {
                let id = q().iter().map(|x| x.0).max().unwrap_or(0) + 1;
                q.write().push((id, t.clone()));
                fire("queue", t);
            }
        }
        text.set(String::new());
        menu.set("");
    };
    let mut ctl = move |what: &'static str| {
        let cur = streamed(st(), base(), since());
        match what {
            "stop" => {
                base.set(cur);
                took.set(Some(now.since(since()).as_secs()));
                st.set(RunStatus::Stopped);
            }
            "resume" => {
                since.set(now);
                st.set(RunStatus::Running);
            }
            "retry" => {
                base.set(0);
                steers.set(vec![]);
                since.set(now);
                took.set(None);
                st.set(RunStatus::Running);
            }
            "dismiss" => {
                base.set(0);
                steers.set(vec![]);
                st.set(RunStatus::Idle);
                ask.set(String::new());
            }
            _ => {}
        }
        fire(what, String::new());
    };
    let running = eff == RunStatus::Running;
    rsx! {
        div { class: "oxk-run", "data-status": "{eff:?}",
            div { class: "oxk-runhead",
                span { class: "oxk-pix", "aria-hidden": "true",
                    for (i, on) in mark.iter().enumerate() { span { key: "{i}", "data-on": "{on}" } }
                }
                span { class: "oxk-agent", "{agent}" }
                span { class: "oxk-runst", key: "{eff:?}",
                    {match eff {
                        RunStatus::Running => rsx! { span { class: "oxk-shim", "Working" } },
                        RunStatus::Done => rsx! { "Done in {mss(took_s)}" },
                        RunStatus::Stopped => rsx! { "Stopped" },
                        RunStatus::Error => rsx! { span { class: "oxk-err", "Failed · rate limited" } },
                        RunStatus::Idle => rsx! { "Ready" },
                    }}
                }
            }
            div { class: "oxk-runbody", "aria-live": "polite",
                if ask().is_empty() {
                    div { class: "oxk-quiet", "Ask something to start a run." }
                } else {
                    div { class: "oxk-you", "{ask}" }
                    p { class: "oxk-ans",
                        for (i, s) in segs.iter().enumerate() {
                            {match s {
                                RunSeg::Text(t) => rsx! { span { key: "{i}", "{t}" } },
                                RunSeg::Steer(t) => rsx! { span { key: "{i}", class: "oxk-steer", Icon { name: "steer", size: 10.0 } "{t}" } },
                            }}
                        }
                        if running { span { class: "oxk-cur" } }
                    }
                    div { class: "oxk-runbtns",
                        for (a, ic, lab) in eff.actions().iter().copied().filter(|a| *a != "stop").map(|a| (a, match a { "resume" => "play", "retry" => "retry", _ => "x" }, match a { "resume" => "Resume", "retry" => "Retry", _ => "Dismiss" })) {
                            button { key: "{a}", class: "oxk-pill", r#type: "button", onclick: move |_| ctl(a),
                                Icon { name: ic, size: 11.0 }
                                "{lab}"
                            }
                        }
                    }
                }
            }
            if !ql.is_empty() {
                div { class: "oxk-queue", "aria-label": "Queued messages",
                    for (qi, (id, t)) in ql.iter().enumerate() {
                        {
                            let id = *id;
                            let t2 = t.clone();
                            rsx! {
                                div { key: "{id}", class: "oxk-qi",
                                    Icon { name: "clock", size: 11.0 }
                                    span { class: "oxk-qt", "{t}" }
                                    if qi == 0 && !running {
                                        button { class: "oxk-pill", r#type: "button", onclick: move |_| { q.write().retain(|x| x.0 != id); text.set(t2.clone()); submit(false, Some(SendAct::Send)) }, "Run next" }
                                    }
                                    button { class: "oxk-x", r#type: "button", "aria-label": "Remove from queue", onclick: move |_| { q.write().retain(|x| x.0 != id); fire("unqueue", String::new()) }, Icon { name: "x", size: 10.0 } }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxk-card",
                textarea { class: "oxk-ta", rows: "2", value: "{text}", "aria-label": "Message",
                    placeholder: if running { "Steer with Enter · queue with ⌥ Enter" } else { "{placeholder}" },
                    oninput: move |e| text.set(e.value()),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !e.modifiers().shift() { e.prevent_default(); submit(e.modifiers().alt(), None) },
                }
                div { class: "oxk-bar",
                    div { class: "oxk-seg2", role: "radiogroup", "aria-label": "Mode",
                        for m in ["chat", "act"] {
                            button { key: "{m}", class: if md() == m { "on" } else { "" }, r#type: "button", role: "radio", "aria-checked": "{md() == m}",
                                onclick: move |_| { md.set(m.to_string()); fire("mode", m.to_string()) }, if m == "chat" { "Chat" } else { "Act" } }
                        }
                    }
                    if act {
                        div { class: "oxk-menuwrap",
                            button { class: if perm() == "full" { "oxk-chip danger" } else { "oxk-chip" }, r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{menu() == \"perm\"}",
                                onclick: move |_| menu.set(if menu() == "perm" { "" } else { "perm" }),
                                Icon { name: if perm() == "full" { "alert" } else { "lock" }, size: 11.0 }
                                {match perm().as_str() { "edits" => "Auto-edit", "full" => "Full access", _ => "Ask first" }}
                                Icon { name: "chevron", size: 10.0 }
                            }
                            if menu() == "perm" {
                                div { class: "oxk-menu up", role: "menu",
                                    for (id, l, h) in [("ask", "Ask first", "Asks before every change"), ("edits", "Auto-edit", "Edits files, asks for commands"), ("full", "Full access", "Runs anything without asking")] {
                                        button { key: "{id}", class: if id == "full" { "oxk-mi danger" } else { "oxk-mi" }, role: "menuitemradio", "aria-checked": "{perm() == id}", r#type: "button",
                                            onclick: move |_| { perm.set(id.to_string()); menu.set(""); fire("permission", id.to_string()) },
                                            span { class: "oxk-on", "{l}" } span { class: "oxk-oh", "{h}" } if perm() == id { Icon { name: "check", size: 11.0 } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    span { class: "oxk-sp" }
                    span { class: "oxk-ctx", "data-warn": "{ctx_warn}", title: "Context {(context * 100.0).round()}% used",
                        svg { view_box: "0 0 20 20", width: "18", height: "18", "aria-hidden": "true",
                            circle { cx: "10", cy: "10", r: "7.5", fill: "none", stroke_width: "2.5", class: "oxk-rt" }
                            circle { cx: "10", cy: "10", r: "7.5", fill: "none", stroke_width: "2.5", class: "oxk-rf", stroke_dasharray: "{context * 47.12} 47.12", transform: "rotate(-90 10 10)" }
                        }
                        "{(context * 100.0).round()}%"
                    }
                    if running && text().trim().is_empty() {
                        button { class: "oxk-send stop", r#type: "button", "aria-label": "Stop", onclick: move |_| ctl("stop"), span { class: "oxk-sq" } }
                    } else {
                        div { class: "oxk-split",
                            button { class: "oxk-send", r#type: "button", disabled: text().trim().is_empty(),
                                "aria-label": if running { "Steer" } else { "Send" }, onclick: move |_| submit(false, None),
                                span { class: "oxk-arrow", Icon { name: if running { "steer" } else { "up" }, size: 14.0 } }
                            }
                            button { class: "oxk-caretbtn", r#type: "button", "aria-label": "More send options", "aria-haspopup": "menu", "aria-expanded": "{menu() == \"send\"}",
                                onclick: move |_| menu.set(if menu() == "send" { "" } else { "send" }), Icon { name: "chevron", size: 10.0 } }
                            if menu() == "send" {
                                div { class: "oxk-menu up end", role: "menu",
                                    button { class: "oxk-mi", role: "menuitem", r#type: "button", disabled: text().trim().is_empty() || running, onclick: move |_| submit(false, Some(SendAct::Send)),
                                        Icon { name: "up", size: 11.0 } span { class: "oxk-on", "Send now" } kbd { "↵" } }
                                    button { class: "oxk-mi", role: "menuitem", r#type: "button", disabled: text().trim().is_empty() || !running, onclick: move |_| submit(false, Some(SendAct::Steer)),
                                        Icon { name: "steer", size: 11.0 } span { class: "oxk-on", "Steer this run" } kbd { "↵" } }
                                    button { class: "oxk-mi", role: "menuitem", r#type: "button", disabled: text().trim().is_empty(), onclick: move |_| submit(true, Some(SendAct::Queue)),
                                        Icon { name: "clock", size: 11.0 } span { class: "oxk-on", "Queue for later" } kbd { "⌥↵" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// ─────────────────────────────────────────────────────────────── Model menus ──

/// A model row for [`ModelMenu`]. Strengths go 1..=5 (cost 5 is the most expensive); leave all
/// three `None` for a router and the row says "Varies".
#[derive(Clone, Debug, PartialEq)]
pub struct MenuModel {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// One line.
    pub description: String,
    /// Speed 1..=5.
    pub speed: Option<u8>,
    /// Quality 1..=5.
    pub quality: Option<u8>,
    /// Cost 1..=5.
    pub cost: Option<u8>,
    /// First-token time.
    pub latency_ms: Option<u32>,
    /// Price per 1M tokens.
    pub price_per_m: Option<f64>,
}

/// Model menu (affectUI Model Picker behaviour): point at a model and its speed, quality and cost
/// bars fill; switch on extended thinking and its effort control unfolds; the tray rolls the
/// first-token time and price of the pointed model. Opens below or above (`side: "top"` for a
/// composer at the bottom of the screen) and lines up with either edge.
#[component]
pub fn ModelMenu(
    /// Models.
    models: Vec<MenuModel>,
    /// Chosen id (first when empty).
    #[props(default)]
    value: String,
    /// Thinking on at the start.
    #[props(default)]
    thinking: bool,
    /// Web search on at the start.
    #[props(default)]
    web_search: bool,
    /// Starting effort: `low` · `medium` · `high`.
    #[props(default = "medium".to_string())]
    effort: String,
    /// `start` · `end`.
    #[props(default = "start".to_string())]
    align: String,
    /// `bottom` · `top`.
    #[props(default = "bottom".to_string())]
    side: String,
    /// Picked id.
    #[props(default)]
    on_change: EventHandler<String>,
    /// Thinking, web search and effort changes: (`thinking` | `web` | `effort`, value).
    #[props(default)]
    on_option: EventHandler<(String, String)>,
) -> Element {
    let first = models.iter().position(|m| m.id == value).unwrap_or(0);
    let mut chosen = use_signal(move || first);
    let mut hi = use_signal(move || first);
    let mut open = use_signal(|| false);
    let mut think = use_signal(move || thinking);
    let mut web = use_signal(move || web_search);
    let mut eff = use_signal(move || Effort::from_id(&effort));
    let mut list = use_signal(|| None::<Rc<MountedData>>);
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let n = models.len();
    let cur = models.get(chosen()).cloned();
    let pointed = models.get(hi()).cloned();
    let bars = |v: Option<u8>| -> Element {
        match v {
            Some(v) => {
                rsx! { span { class: "oxk-bars", "data-v": "{v}", for i in 1..=5u8 { span { key: "{i}", "data-on": "{i <= v}" } } } }
            }
            None => rsx! { span { class: "oxk-varies", "Varies" } },
        }
    };
    let ids = use_signal(|| models.iter().map(|m| m.id.clone()).collect::<Vec<_>>());
    let tray = pointed.as_ref().map(|p| {
        (
            p.id.clone(),
            p.latency_ms.map_or("varies".to_string(), |l| {
                format!("{:.2} s", l as f64 / 1000.0)
            }),
            p.price_per_m
                .map_or("Varies".to_string(), |v| format!("${v:.2} / 1M")),
        )
    });
    let cur_name = cur.as_ref().map_or(String::new(), |c| c.name.clone());
    let keys = move |e: KeyboardEvent| match e.key() {
        Key::ArrowDown => {
            e.prevent_default();
            if !open() {
                open.set(true);
                focus(list())
            } else {
                hi.set((hi() + 1) % n.max(1))
            }
        }
        Key::ArrowUp => {
            e.prevent_default();
            if open() {
                hi.set((hi() + n.max(1) - 1) % n.max(1))
            }
        }
        Key::Enter if open() => {
            e.prevent_default();
            chosen.set(hi());
            open.set(false);
            on_change.call(ids.peek()[hi()].clone());
            focus(trig())
        }
        Key::Escape if open() => {
            e.prevent_default();
            open.set(false);
            focus(trig())
        }
        _ => {}
    };
    rsx! {
        div { class: "oxk-mm {align} {side}", "data-open": "{open()}",
            button { class: "oxk-chip", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{open()}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| { open.set(!open()); if !open() { return } hi.set(chosen()); focus(list()) },
                onkeydown: keys,
                Icon { name: "spark", size: 11.0 }
                span { key: "{cur_name}", class: "oxk-mname", "{cur_name}" }
                if think() { span { class: "oxk-eff", "· {eff().label()}" } }
                Icon { name: "chevron", size: 10.0 }
            }
            if open() {
                div { class: "oxk-scrim", onclick: move |_| open.set(false) }
                div { class: "oxk-mmenu",
                    div { class: "oxk-mlist", role: "listbox", tabindex: "0", "aria-label": "Models", "aria-activedescendant": "oxk-mm-{hi()}",
                        onmounted: move |e| list.set(Some(e.data())),
                        onkeydown: keys,
                        for (i, m) in models.iter().enumerate() {
                            div { key: "{m.id}", id: "oxk-mm-{i}", class: "oxk-mrow", role: "option", "aria-selected": "{chosen() == i}", "data-hi": "{hi() == i}",
                                onmouseenter: move |_| hi.set(i),
                                onclick: move |_| { chosen.set(i); open.set(false); on_change.call(ids.peek()[i].clone()) },
                                div { class: "oxk-mtop", span { class: "oxk-on", "{m.name}" } if chosen() == i { Icon { name: "check", size: 11.0 } } }
                                span { class: "oxk-oh", "{m.description}" }
                                if hi() == i {
                                    div { class: "oxk-stats",
                                        if m.speed.is_none() && m.quality.is_none() && m.cost.is_none() {
                                            span { class: "oxk-varies", "Picks the best model per message · strengths vary" }
                                        } else {
                                            span { class: "oxk-stat", "Speed" {bars(m.speed)} }
                                            span { class: "oxk-stat", "Quality" {bars(m.quality)} }
                                            span { class: "oxk-stat", "Cost" {bars(m.cost)} }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxk-opts",
                        button { class: "oxk-mi", role: "switch", "aria-checked": "{think()}", r#type: "button",
                            onclick: move |_| { think.set(!think()); on_option.call(("thinking".into(), think().to_string())) },
                            Icon { name: "brain", size: 12.0 } span { class: "oxk-on", "Extended thinking" } span { class: if think() { "oxk-sw on" } else { "oxk-sw" }, span {} }
                        }
                        div { class: "oxk-unfold", "data-open": "{think()}",
                            div { class: "oxk-unin",
                                div { class: "oxk-effrow", role: "radiogroup", "aria-label": "Thinking effort",
                                    for e in [Effort::Low, Effort::Medium, Effort::High] {
                                        button { key: "{e.id()}", class: if eff() == e { "oxk-seg on" } else { "oxk-seg" }, r#type: "button", role: "radio", "aria-checked": "{eff() == e}", tabindex: if think() { "0" } else { "-1" },
                                            onclick: move |_| { eff.set(e); on_option.call(("effort".into(), e.id().into())) }, "{e.label()}" }
                                    }
                                }
                            }
                        }
                        button { class: "oxk-mi", role: "switch", "aria-checked": "{web()}", r#type: "button",
                            onclick: move |_| { web.set(!web()); on_option.call(("web".into(), web().to_string())) },
                            Icon { name: "globe", size: 12.0 } span { class: "oxk-on", "Web search" } span { class: if web() { "oxk-sw on" } else { "oxk-sw" }, span {} }
                        }
                    }
                    if let Some((pid, lat, price)) = &tray {
                        div { class: "oxk-tray", "aria-live": "polite", "data-model": "{pid}",
                            span { "First token " span { key: "{pid}l", class: "oxk-rollv", "{lat}" } }
                            span { key: "{pid}p", class: "oxk-rollv", "{price}" }
                        }
                    }
                }
            }
        }
    }
}

/// A model for [`ModelPill`].
#[derive(Clone, Debug, PartialEq)]
pub struct PillModel {
    /// API id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Provider id.
    pub provider: String,
    /// Short menu line.
    pub blurb: String,
    /// Longer description for the summary.
    pub description: String,
    /// Context window.
    pub context: String,
    /// Price in / out per 1M tokens.
    pub price: (f64, f64),
    /// Effort levels it accepts.
    pub efforts: Vec<Effort>,
    /// Its default effort.
    pub default_effort: Effort,
}

/// Model pill (Arc Model picker block behaviour): one quiet pill. Open it to choose from featured
/// models with a one-line note each, set the reasoning effort in a submenu (a level the model does
/// not accept snaps to the nearest one it does), or browse every other model by provider under
/// More models. The pill swaps to the choice in place (mark and name crossfade, width springs);
/// the summary under it shows description, context, prices, effort and a short confirmation.
/// Type a letter to jump to a model; Right / Left open and close the submenus.
#[component]
pub fn ModelPill(
    /// The catalog.
    models: Vec<PillModel>,
    /// Providers: id and name, in More models order.
    providers: Vec<(String, String)>,
    /// Featured ids, in order (the first four when empty).
    #[props(default)]
    featured: Vec<String>,
    /// Show the summary under the pill.
    #[props(default = true)]
    show_summary: bool,
    /// Shared clock (the confirmation's timing).
    now: ClockMs,
    /// Picked id.
    #[props(default)]
    on_change: EventHandler<String>,
    /// Effort changes (including snaps).
    #[props(default)]
    on_effort: EventHandler<String>,
) -> Element {
    let feat: Vec<usize> = if featured.is_empty() {
        (0..models.len().min(4)).collect()
    } else {
        featured
            .iter()
            .filter_map(|f| models.iter().position(|m| &m.id == f))
            .collect()
    };
    let feat2 = feat.clone();
    let mut chosen = use_signal(move || feat2.first().copied().unwrap_or(0));
    let mut eff = use_signal(|| None::<Effort>);
    let mut open = use_signal(|| false);
    let mut sub = use_signal(|| "");
    let mut hi = use_signal(|| 0usize);
    let mut note = use_signal(|| None::<(String, ClockMs)>);
    let mut list = use_signal(|| None::<Rc<MountedData>>);
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let cur = models[chosen().min(models.len() - 1)].clone();
    let e_now = eff().map_or(cur.default_effort, |e| snap(e, &cur.efforts));
    // Rows of the main page: featured models, then Effort and More models.
    let rows: Vec<String> = feat
        .iter()
        .map(|i| models[*i].id.clone())
        .chain(["effort".to_string(), "more".to_string()])
        .collect();
    let nrows = rows.len();
    let mark = |p: &str| -> String { p.chars().next().unwrap_or('?').to_uppercase().collect() };
    let ms = use_signal(|| models.clone());
    let rows2 = use_signal(|| rows.clone());
    let mut pick = move |i: usize| {
        let all = ms.peek().clone();
        let m = &all[i];
        let old = eff().unwrap_or(m.default_effort);
        let snapped = snap(old, &m.efforts);
        chosen.set(i);
        if snapped != old {
            eff.set(Some(snapped));
            on_effort.call(snapped.id().to_string());
        }
        note.set(Some((format!("Switched to {}", m.name), now)));
        on_change.call(m.id.clone());
        open.set(false);
        sub.set("");
        focus(trig());
    };
    let keys = move |e: KeyboardEvent| match e.key() {
        Key::ArrowDown => {
            e.prevent_default();
            if !open() {
                open.set(true);
                hi.set(0);
                focus(list())
            } else {
                hi.set((hi() + 1) % nrows)
            }
        }
        Key::ArrowUp if open() => {
            e.prevent_default();
            hi.set((hi() + nrows - 1) % nrows)
        }
        Key::ArrowRight if open() => {
            e.prevent_default();
            let r = rows2.peek()[hi()].clone();
            if r == "effort" {
                sub.set("effort")
            } else if r == "more" {
                sub.set("more")
            }
        }
        Key::ArrowLeft if open() => {
            e.prevent_default();
            sub.set("")
        }
        Key::Escape if open() => {
            e.prevent_default();
            if sub().is_empty() {
                open.set(false);
                focus(trig())
            } else {
                sub.set("")
            }
        }
        Key::Enter | Key::Character(_)
            if open() && (e.key() == Key::Enter || e.key() == Key::Character(" ".into())) =>
        {
            e.prevent_default();
            let r = rows2.peek()[hi()].clone();
            let pos = ms.peek().iter().position(|m| m.id == r);
            if r == "effort" {
                sub.set("effort")
            } else if r == "more" {
                sub.set("more")
            } else if let Some(i) = pos {
                pick(i)
            }
        }
        Key::Character(c) if open() => {
            if let Some(ch) = c.chars().next() {
                let ch = ch.to_ascii_lowercase();
                let start = hi() + 1;
                let all = ms.peek().clone();
                let rr = rows2.peek().clone();
                if let Some(k) = (0..nrows).map(|k| (start + k) % nrows).find(|k| {
                    all.iter()
                        .find(|m| m.id == rr[*k])
                        .is_some_and(|m| m.name.to_lowercase().starts_with(ch))
                }) {
                    hi.set(k)
                }
            }
        }
        _ => {}
    };
    let show_note = note()
        .filter(|n| now.since(n.1).as_millis() < 2200)
        .map(|n| n.0);
    let prov_name = providers
        .iter()
        .find(|p| p.0 == cur.provider)
        .map_or(cur.provider.clone(), |p| p.1.clone());
    rsx! {
        div { class: "oxk-mp", "data-open": "{open()}",
            button { class: "oxk-pillbtn", r#type: "button", "aria-haspopup": "menu", "aria-expanded": "{open()}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| { open.set(!open()); sub.set(""); hi.set(0); focus(list()) },
                onkeydown: keys,
                span { key: "{cur.id}m", class: "oxk-pmark", "data-p": "{cur.provider}", if crate::brand::has_brand(&cur.provider) { crate::brand::BrandIcon { name: cur.provider.clone(), size: 12.0 } } else { "{mark(&cur.provider)}" } }
                span { key: "{cur.id}", class: "oxk-pname", "{cur.name}" }
                span { class: "oxk-eff", "{e_now.label()}" }
                Icon { name: "chevron", size: 10.0 }
            }
            if open() {
                div { class: "oxk-scrim", onclick: move |_| open.set(false) }
                div { class: "oxk-pmenu", role: "menu", tabindex: "-1", onmounted: move |e| list.set(Some(e.data())), onkeydown: keys,
                    if sub().is_empty() {
                        div { class: "oxk-page",
                            for (ri, r) in rows.iter().enumerate() {
                                if let Some(mi) = models.iter().position(|m| &m.id == r) {
                                    button { key: "{r}", class: "oxk-mi tall", role: "menuitemradio", r#type: "button", "aria-checked": "{chosen() == mi}", "data-hi": "{hi() == ri}",
                                        onmouseenter: move |_| hi.set(ri), onclick: move |_| pick(mi),
                                        span { class: "oxk-pmark", "data-p": "{models[mi].provider}", if crate::brand::has_brand(&models[mi].provider) { crate::brand::BrandIcon { name: models[mi].provider.clone(), size: 12.0 } } else { "{mark(&models[mi].provider)}" } }
                                        span { class: "oxk-col", span { class: "oxk-on", "{models[mi].name}" } span { class: "oxk-oh", "{models[mi].blurb}" } }
                                        if chosen() == mi { Icon { name: "check", size: 11.0 } }
                                    }
                                } else if r == "effort" {
                                    button { class: "oxk-mi", role: "menuitem", r#type: "button", "aria-haspopup": "menu", "data-hi": "{hi() == ri}",
                                        onmouseenter: move |_| hi.set(ri), onclick: move |_| sub.set("effort"),
                                        Icon { name: "brain", size: 12.0 } span { class: "oxk-on", "Effort" } span { class: "oxk-oh", "{e_now.label()}" } Icon { name: "chevron-right", size: 10.0 }
                                    }
                                } else {
                                    button { class: "oxk-mi", role: "menuitem", r#type: "button", "aria-haspopup": "menu", "data-hi": "{hi() == ri}",
                                        onmouseenter: move |_| hi.set(ri), onclick: move |_| sub.set("more"),
                                        Icon { name: "more", size: 12.0 } span { class: "oxk-on", "More models" } span { class: "oxk-oh", "{models.len() - feat.len()}" } Icon { name: "chevron-right", size: 10.0 }
                                    }
                                }
                            }
                        }
                    } else if sub() == "effort" {
                        div { class: "oxk-page sub",
                            button { class: "oxk-back", r#type: "button", onclick: move |_| sub.set(""), Icon { name: "chevron-left", size: 10.0 } "Effort for {cur.name}" }
                            for e in Effort::ALL {
                                {
                                    let ok = cur.efforts.contains(&e);
                                    rsx! {
                                        button { key: "{e.id()}", class: "oxk-mi", role: "menuitemradio", r#type: "button", disabled: !ok, "aria-checked": "{e_now == e}",
                                            onclick: move |_| { eff.set(Some(e)); on_effort.call(e.id().to_string()); note.set(Some((format!("Effort set to {}", e.label()), now))); sub.set(""); open.set(false); focus(trig()) },
                                            span { class: "oxk-on", "{e.label()}" }
                                            if !ok { span { class: "oxk-oh", "Not offered" } }
                                            if e_now == e { Icon { name: "check", size: 11.0 } }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        div { class: "oxk-page sub",
                            button { class: "oxk-back", r#type: "button", onclick: move |_| sub.set(""), Icon { name: "chevron-left", size: 10.0 } "More models" }
                            for (pid, pname) in providers.iter() {
                                if models.iter().enumerate().any(|(i, m)| &m.provider == pid && !feat.contains(&i)) {
                                    div { key: "{pid}", class: "oxk-group",
                                        span { class: "oxk-plabel", "{pname}" }
                                        for (mi, m) in models.iter().enumerate().filter(|(i, m)| &m.provider == pid && !feat.contains(i)) {
                                            button { key: "{m.id}", class: "oxk-mi tall", role: "menuitemradio", r#type: "button", "aria-checked": "{chosen() == mi}", onclick: move |_| pick(mi),
                                                span { class: "oxk-col", span { class: "oxk-on", "{m.name}" } span { class: "oxk-oh", "{m.blurb}" } }
                                                if chosen() == mi { Icon { name: "check", size: 11.0 } }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if show_summary {
                div { class: "oxk-sum",
                    p { key: "{cur.id}d", class: "oxk-sumd", "{cur.description}" }
                    dl { class: "oxk-sumg",
                        dt { "Provider" } dd { key: "{cur.id}a", "{prov_name}" }
                        dt { "Context" } dd { key: "{cur.id}b", "{cur.context}" }
                        dt { "Input / output" } dd { key: "{cur.id}c", "{price_pair(cur.price.0, cur.price.1)} per 1M" }
                        dt { "Effort" } dd { key: "{e_now.id()}", "{e_now.label()}" }
                    }
                    div { class: "oxk-confirm", role: "status", if let Some(n) = &show_note { span { key: "{n}", "{n}" } } }
                }
            }
        }
    }
}
