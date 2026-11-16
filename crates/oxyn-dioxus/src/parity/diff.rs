//! Diff review (plan #30): `DiffViewer` (unified, split and stacked, per-hunk actions, file
//! list, minimap, line comments, review bar), `InlineDiff`, `ChangeSummary`, `DiffTable`,
//! `DiffTimeline` and `CodeSuggestion`. Styles: `diff.css` (classes `oxyn-diff-*`, `oxyn-chs-*`,
//! `oxyn-dtb-*`, `oxyn-dtl-*`, `oxyn-sug-*`, `oxyn-idf-*`).

use std::collections::BTreeSet;
use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::diff::{
        Chunk, DLine, Decision, FileDiff, HunkRef, LineComment, LineKind, Review, ReviewMode,
        Revision, RowChange, RowKind, Seg, SugRow, SugStatus, Suggestion, change_totals,
        files_changed, revision_diff, revision_stats, rewrite_apply, rewrite_chunks, rewrite_next,
        split_rows, stat_blocks, stat_label,
    },
    primitives::chrome::highlight,
};

use crate::composer::icons::Icon;

fn src_line(text: &str, words: &[Seg], lang: &str, word_diff: bool) -> Element {
    if word_diff && !words.is_empty() {
        return rsx! {
            for (k, s) in words.iter().enumerate() {
                span { key: "{k}", class: if s.changed { "oxyn-diff-w" } else { "" }, "{s.text}" }
            }
        };
    }
    let toks = highlight(text, lang).into_iter().next().unwrap_or_default();
    rsx! {
        for (k, (t, s)) in toks.into_iter().enumerate() {
            span { key: "{k}", class: "t-{t.id()}", "{s}" }
        }
    }
}

fn progress_glyph(c: (usize, usize, usize)) -> &'static str {
    let total = c.0 + c.1 + c.2;
    if total == 0 || c.0 == total {
        "○"
    } else if c.0 == 0 {
        "●"
    } else {
        "◐"
    }
}

/// One line of the unified layout.
#[component]
fn DiffRow(
    line: DLine,
    lang: String,
    word_diff: bool,
    line_numbers: bool,
    comments: bool,
    on_comment: EventHandler<usize>,
) -> Element {
    let k = line.kind;
    let at = line.new.or(line.old).unwrap_or(0);
    rsx! {
        div { class: "oxyn-diff-ln", "data-k": k.id(),
            if line_numbers {
                span { class: "oxyn-diff-no", "aria-hidden": "true", {line.old.map(|n| n.to_string()).unwrap_or_default()} }
                span { class: "oxyn-diff-no", "aria-hidden": "true", {line.new.map(|n| n.to_string()).unwrap_or_default()} }
            }
            span { class: "oxyn-diff-sg", "aria-hidden": "true", "{k.sign()}" }
            span { class: "oxyn-diff-src",
                if !k.spoken().is_empty() { span { class: "oxyn-diff-sr", "{k.spoken()}" } }
                {src_line(&line.text, &line.words, &lang, word_diff)}
            }
            if comments {
                button { class: "oxyn-diff-add", r#type: "button", "aria-label": "Comment on line {at}", title: "Comment (c)",
                    onclick: move |_| on_comment.call(at), "+"
                }
            }
        }
    }
}

/// A review of an agent's change set: unified, split or stacked layouts; Accept / Reject (or
/// Keep / Undo once applied) per hunk, per file and for all with undo; a file list with review
/// progress and Viewed marks; hidden context that opens on demand; draft line comments sent to the
/// agent together; a minimap and a review bar. Keys: `j` / `k` hunk, `[` / `]` file, `y` yes,
/// `x` no, `c` comment, `z` undo, `⌘↵` send comments.
#[component]
pub fn DiffViewer(
    /// Files in first-touch order.
    files: Vec<FileDiff>,
    /// `Proposed` (Accept / Reject) or `Applied` (Keep / Undo).
    #[props(default)]
    mode: ReviewMode,
    /// `unified` · `split` · `stacked`.
    #[props(default = "unified".to_string())]
    layout: String,
    /// `background` · `markers` · `bars`.
    #[props(default = "background".to_string())]
    change_style: String,
    /// File list on the left.
    #[props(default = true)]
    file_list: bool,
    /// Word-level highlights on paired lines.
    #[props(default = true)]
    word_diff: bool,
    /// Wrap long lines.
    #[props(default)]
    wrap: bool,
    /// Line-number gutters.
    #[props(default = true)]
    line_numbers: bool,
    /// Overview ruler at the end edge.
    #[props(default = true)]
    minimap: bool,
    /// Line comments.
    #[props(default = true)]
    comments: bool,
    /// Dim every hunk but the current one.
    #[props(default)]
    focus_dim: bool,
    /// Layout switcher in the header.
    #[props(default = true)]
    switcher: bool,
    /// Heading.
    #[props(default = "Changes".to_string())]
    title: String,
    /// A hunk was decided (`Pending` = restored).
    #[props(default)]
    on_decide: EventHandler<(HunkRef, Decision)>,
    /// Comments sent to the agent.
    #[props(default)]
    on_send: EventHandler<Vec<LineComment>>,
    /// A file's Viewed mark changed.
    #[props(default)]
    on_viewed: EventHandler<(usize, bool)>,
) -> Element {
    let init = files.clone();
    let mut rv = use_signal(move || Review::new(init, mode));
    let mut lay = use_signal(|| layout.clone());
    let mut wrapping = use_signal(|| wrap);
    let mut open_gaps = use_signal(BTreeSet::<HunkRef>::new);
    let mut composer = use_signal(|| None::<(usize, usize, usize)>);
    let mut draft = use_signal(String::new);
    let mut live = use_signal(String::new);
    let mut sent = use_signal(|| 0usize);
    let mut hunk_els = use_signal(std::collections::BTreeMap::<HunkRef, Rc<MountedData>>::new);
    let r = rv.read().clone();
    let (pend, _yes, _no) = r.counts();
    let (pos, total) = r.position();
    let (la, lr) = r.live_stat();
    let (nf, _, _) = change_totals(&r.files);
    let viewed_n = r.viewed.len();
    let cur = r.cursor;
    let m = r.mode;
    let layout_now = lay();
    let reveal = move |h: HunkRef| {
        if let Some(el) = hunk_els.peek().get(&h).cloned() {
            spawn(async move {
                let _ = el.scroll_to(ScrollBehavior::Smooth).await;
            });
        }
    };
    let mut decide = move |h: HunkRef, d: Decision| {
        let changed = rv.write().decide(h, d);
        if changed {
            on_decide.call((h, d));
            let msg = rv.peek().announce(h, d);
            live.set(msg);
        }
    };
    let mut all = move |d: Decision| {
        let refs: Vec<HunkRef> = {
            let p = rv.peek();
            p.hunk_refs()
                .into_iter()
                .filter(|x| p.decision(*x) == Decision::Pending)
                .collect()
        };
        let n = rv.write().decide_all(d);
        for h in refs {
            on_decide.call((h, d));
        }
        let w = if d == Decision::Yes {
            m.yes_done()
        } else {
            m.no_done()
        };
        live.set(format!(
            "{w} {n} {}",
            if n == 1 { "change" } else { "changes" }
        ));
    };
    let mut send = move || {
        let c = rv.write().send_comments();
        if !c.is_empty() {
            let n = c.len();
            on_send.call(c);
            sent.set(n);
            live.set(format!(
                "Sent {n} {} to the agent",
                if n == 1 { "comment" } else { "comments" }
            ));
        }
    };
    let ncomments = r.comments.len();
    // Minimap marks: one per changed line, positioned over all rendered rows.
    let mut marks: Vec<(f32, &'static str, bool)> = Vec::new();
    let mut rows_total = 0usize;
    for (fi, f) in r.files.iter().enumerate() {
        rows_total += 1;
        for h in &f.hunks {
            rows_total += 1;
            for l in &h.lines {
                if l.kind != LineKind::Ctx {
                    marks.push((rows_total as f32, l.kind.id(), (fi, h.id) == cur));
                }
                rows_total += 1;
            }
        }
    }
    let rows_total = rows_total.max(1) as f32;
    rsx! {
        section { class: "oxyn-diff", "data-layout": "{layout_now}", "data-style": "{change_style}", "data-dim": "{focus_dim}",
            "data-wrap": "{wrapping()}", "data-files": "{file_list}", "aria-label": "{title}", tabindex: "0",
            onkeydown: move |e: KeyboardEvent| {
                let mods = e.modifiers();
                if composer.peek().is_some() { return; }
                let key = e.key();
                let ch = if let Key::Character(c) = &key { c.clone() } else { String::new() };
                if key == Key::Enter && (mods.meta() || mods.ctrl()) { e.prevent_default(); send(); return; }
                if mods.meta() || mods.ctrl() || mods.alt() { return; }
                let c = rv.peek().cursor;
                match ch.as_str() {
                    "j" => { rv.write().step(1); let n = rv.peek().cursor; reveal(n); }
                    "k" => { rv.write().step(-1); let n = rv.peek().cursor; reveal(n); }
                    "]" => { rv.write().step_file(1); let n = rv.peek().cursor; reveal(n); }
                    "[" => { rv.write().step_file(-1); let n = rv.peek().cursor; reveal(n); }
                    "y" => { decide(c, Decision::Yes); let n = rv.peek().cursor; reveal(n); }
                    "x" => { decide(c, Decision::No); let n = rv.peek().cursor; reveal(n); }
                    "z" => { let n = rv.write().undo(); if n > 0 { live.set(format!("Restored {n}")); } }
                    "c" if comments => {
                        let line = rv.peek().files.get(c.0).and_then(|f| f.hunks.get(c.1)).map(|h| h.new_start).unwrap_or(1);
                        e.prevent_default();
                        composer.set(Some((c.0, c.1, line)));
                    }
                    _ => {}
                }
            },
            header { class: "oxyn-diff-hd",
                span { class: "oxyn-diff-tt", "{title}" }
                span { class: "oxyn-diff-meta", "{files_changed(nf)}" }
                span { class: "oxyn-diff-stat", "aria-label": "{la} added, {lr} removed",
                    span { class: "a", "+{la}" } span { class: "r", "−{lr}" }
                }
                span { class: "oxyn-diff-sp" }
                if switcher {
                    div { class: "oxyn-diff-seg", role: "radiogroup", "aria-label": "Layout",
                        for (id, label) in [("unified", "Unified"), ("split", "Split"), ("stacked", "Stacked")] {
                            button { key: "{id}", r#type: "button", role: "radio", "aria-checked": "{layout_now == id}",
                                onclick: move |_| lay.set(id.to_string()), "{label}"
                            }
                        }
                    }
                }
                button { class: "oxyn-diff-ib", r#type: "button", "aria-pressed": "{wrapping()}", "aria-label": "Wrap lines", title: "Wrap lines",
                    onclick: move |_| { let w = !wrapping(); wrapping.set(w) }, Icon { name: "text", size: 13.0 }
                }
            }
            div { class: "oxyn-diff-body",
                if file_list {
                    nav { class: "oxyn-diff-files", "aria-label": "Changed files",
                        div { class: "oxyn-diff-fsum", "{viewed_n} of {nf} viewed" }
                        for (fi, f) in r.files.iter().enumerate() {
                            {
                                let fc = r.file_counts(fi);
                                let is_cur = cur.0 == fi;
                                let viewed = r.viewed.contains(&fi);
                                let first = (fi, 0usize);
                                rsx! {
                                    button { key: "{fi}", class: "oxyn-diff-frow", r#type: "button", "aria-current": if is_cur { "true" } else { "false" },
                                        "data-viewed": "{viewed}", "aria-label": "{f.path}, {f.status.word()}, {f.added} added, {f.removed} removed",
                                        onclick: move |_| { rv.write().cursor = first; reveal(first); },
                                        span { class: "oxyn-diff-fst", "data-s": f.status.id(), "{f.status.letter()}" }
                                        span { class: "oxyn-diff-fname", "{f.name()}" }
                                        span { class: "oxyn-diff-fstat", "{stat_label(f.added, f.removed)}" }
                                        span { class: "oxyn-diff-fprog", "aria-hidden": "true", if viewed { "✓" } else { "{progress_glyph(fc)}" } }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "oxyn-diff-main",
                    for (fi, f) in r.files.iter().enumerate() {
                        {
                            let viewed = r.viewed.contains(&fi);
                            let lang = f.lang.to_string();
                            let fc = r.file_counts(fi);
                            rsx! {
                                article { key: "{fi}", class: "oxyn-diff-file", "data-viewed": "{viewed}", "aria-label": "{f.path}",
                                    div { class: "oxyn-diff-fh",
                                        span { class: "oxyn-diff-fst", "data-s": f.status.id(), title: f.status.word(), "{f.status.letter()}" }
                                        span { class: "oxyn-diff-path",
                                            if !f.dir().is_empty() { span { class: "d", "{f.dir()}/" } }
                                            "{f.name()}"
                                            if let oxyn_core::parity::diff::FileStatus::Renamed(from) = &f.status { span { class: "d", " ← {from}" } }
                                        }
                                        span { class: "oxyn-diff-fstat", "{stat_label(f.added, f.removed)}" }
                                        span { class: "oxyn-diff-sp" }
                                        if fc.0 > 0 {
                                            button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| { let n = rv.write().decide_file(fi, Decision::No); live.set(format!("{} {n} in file", m.no_done())); }, "{m.no()} file" }
                                            button { class: "oxyn-diff-btn", r#type: "button", onclick: move |_| { let n = rv.write().decide_file(fi, Decision::Yes); live.set(format!("{} {n} in file", m.yes_done())); }, "{m.yes()} file" }
                                        }
                                        label { class: "oxyn-diff-viewed",
                                            input { r#type: "checkbox", checked: viewed, onchange: move |_| { rv.write().toggle_viewed(fi); on_viewed.call((fi, !viewed)); } }
                                            "Viewed"
                                        }
                                    }
                                    if !viewed {
                                        for h in f.hunks.iter() {
                                            {
                                                let hr = (fi, h.id);
                                                let d = r.decision(hr);
                                                let is_cur = hr == cur;
                                                let gap_open = open_gaps.read().contains(&hr);
                                                let gap = h.gap_before.clone();
                                                let hl = h.label();
                                                let lines = h.lines.clone();
                                                let rows = split_rows(&h.lines);
                                                let drafts: Vec<LineComment> = r.comments.iter().filter(|c| c.file == fi && c.hunk == h.id).cloned().collect();
                                                let comp = composer().filter(|c| c.0 == fi && c.1 == h.id).map(|c| c.2);
                                                let lang2 = lang.clone();
                                                let lang3 = lang.clone();
                                                let lang4 = lang.clone();
                                                rsx! {
                                                    div { key: "{h.id}", class: "oxyn-diff-hk", "data-state": d.id(), "data-current": "{is_cur}",
                                                        onmounted: move |e| { hunk_els.write().insert(hr, e.data()); },
                                                        if !gap.is_empty() {
                                                            if gap_open {
                                                                for l in gap.iter() {
                                                                    DiffRow { key: "g{l.old.unwrap_or(0)}", line: l.clone(), lang: lang.clone(), word_diff, line_numbers, comments: false, on_comment: move |_| {} }
                                                                }
                                                            } else {
                                                                button { class: "oxyn-diff-gap", r#type: "button", onclick: move |_| { open_gaps.write().insert(hr); },
                                                                    {format!("⋯ Show {} hidden {} ⋯", gap.len(), if gap.len() == 1 { "line" } else { "lines" })}
                                                                }
                                                            }
                                                        }
                                                        div { class: "oxyn-diff-hh", onclick: move |_| rv.write().cursor = hr,
                                                            code { class: "oxyn-diff-rng", "{h.range()}" }
                                                            if !h.header.is_empty() { span { class: "oxyn-diff-sym", "{h.header}" } }
                                                            span { class: "oxyn-diff-fstat", "{stat_label(h.added, h.removed)}" }
                                                            span { class: "oxyn-diff-sp" }
                                                            match d {
                                                                Decision::Pending => rsx! {
                                                                    button { class: "oxyn-diff-btn q", r#type: "button", "aria-label": "{m.no()} {hl}", "aria-keyshortcuts": "x",
                                                                        onclick: move |e| { e.stop_propagation(); decide(hr, Decision::No) }, Icon { name: "x", size: 12.0 } "{m.no()}" }
                                                                    button { class: "oxyn-diff-btn y", r#type: "button", "aria-label": "{m.yes()} {hl}", "aria-keyshortcuts": "y",
                                                                        onclick: move |e| { e.stop_propagation(); decide(hr, Decision::Yes) }, Icon { name: "check", size: 12.0 } "{m.yes()}" }
                                                                },
                                                                _ => rsx! {
                                                                    span { class: "oxyn-diff-chip", "data-d": d.id(),
                                                                        if d == Decision::Yes { Icon { name: "check", size: 11.0, stroke: 2.2 } "{m.yes_done()}" } else { Icon { name: "undo", size: 11.0 } "{m.no_done()}" }
                                                                    }
                                                                    button { class: "oxyn-diff-lnk", r#type: "button", onclick: move |e| { e.stop_propagation(); decide(hr, Decision::Pending) }, "Restore" }
                                                                },
                                                            }
                                                        }
                                                        match layout_now.as_str() {
                                                            "split" => rsx! {
                                                                div { class: "oxyn-diff-split",
                                                                    for (ri, (a, b)) in rows.into_iter().enumerate() {
                                                                        div { key: "{ri}", class: "oxyn-diff-srow",
                                                                            for (side, l) in [("old", a), ("new", b)] {
                                                                                match l {
                                                                                    Some(l) => rsx! {
                                                                                        div { key: "{side}", class: "oxyn-diff-half", "data-k": l.kind.id(),
                                                                                            if line_numbers { span { class: "oxyn-diff-no", "aria-hidden": "true", {if side == "old" { l.old } else { l.new }.map(|n| n.to_string()).unwrap_or_default()} } }
                                                                                            span { class: "oxyn-diff-sg", "aria-hidden": "true", "{l.kind.sign()}" }
                                                                                            span { class: "oxyn-diff-src", if !l.kind.spoken().is_empty() { span { class: "oxyn-diff-sr", "{l.kind.spoken()}" } } {src_line(&l.text, &l.words, &lang2, word_diff)} }
                                                                                        }
                                                                                    },
                                                                                    None => rsx! { div { key: "{side}", class: "oxyn-diff-half oxyn-diff-fill", "aria-hidden": "true" } },
                                                                                }
                                                                            }
                                                                        }
                                                                    }
                                                                }
                                                            },
                                                            "stacked" => rsx! {
                                                                div { class: "oxyn-diff-stack",
                                                                    div { class: "oxyn-diff-blk", "data-k": "del",
                                                                        div { class: "oxyn-diff-bl", "Before" }
                                                                        for (li, l) in lines.iter().filter(|l| l.kind != LineKind::Add).enumerate() {
                                                                            DiffRow { key: "{li}", line: l.clone(), lang: lang3.clone(), word_diff, line_numbers: false, comments: false, on_comment: move |_| {} }
                                                                        }
                                                                    }
                                                                    div { class: "oxyn-diff-blk", "data-k": "add",
                                                                        div { class: "oxyn-diff-bl", "After" }
                                                                        for (li, l) in lines.iter().filter(|l| l.kind != LineKind::Del).enumerate() {
                                                                            DiffRow { key: "{li}", line: l.clone(), lang: lang3.clone(), word_diff, line_numbers: false, comments, on_comment: move |ln| { draft.set(String::new()); composer.set(Some((fi, hr.1, ln))) } }
                                                                        }
                                                                    }
                                                                }
                                                            },
                                                            _ => rsx! {
                                                                div { class: "oxyn-diff-uni",
                                                                    for (li, l) in lines.iter().enumerate() {
                                                                        DiffRow { key: "{li}", line: l.clone(), lang: lang4.clone(), word_diff, line_numbers, comments, on_comment: move |ln| { draft.set(String::new()); composer.set(Some((fi, hr.1, ln))) } }
                                                                    }
                                                                }
                                                            },
                                                        }
                                                        for (ci, c) in drafts.iter().enumerate() {
                                                            div { key: "c{ci}", class: "oxyn-diff-cm",
                                                                span { class: "oxyn-diff-cl", "L{c.line}" }
                                                                span { "{c.text}" }
                                                                span { class: "oxyn-diff-draft", "Draft" }
                                                            }
                                                        }
                                                        if let Some(ln) = comp {
                                                            div { class: "oxyn-diff-comp",
                                                                label { class: "oxyn-diff-cl", r#for: "oxyn-diff-ta-{fi}-{hr.1}", "Comment on line {ln}" }
                                                                textarea { id: "oxyn-diff-ta-{fi}-{hr.1}", rows: "2", placeholder: "Tell the agent what to change…", value: "{draft}",
                                                                    onmounted: move |e| { let el = e.data(); spawn(async move { let _ = el.set_focus(true).await; }); },
                                                                    oninput: move |e| draft.set(e.value()),
                                                                    onkeydown: move |e: KeyboardEvent| {
                                                                        e.stop_propagation();
                                                                        if e.key() == Key::Escape { composer.set(None); }
                                                                        if e.key() == Key::Enter && (e.modifiers().meta() || e.modifiers().ctrl()) {
                                                                            e.prevent_default();
                                                                            let t = draft.peek().clone();
                                                                            rv.write().comment(fi, hr.1, ln, &t);
                                                                            draft.set(String::new());
                                                                            composer.set(None);
                                                                        }
                                                                    }
                                                                }
                                                                div { class: "oxyn-diff-crow",
                                                                    button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| composer.set(None), "Cancel" }
                                                                    button { class: "oxyn-diff-btn", r#type: "button", onclick: move |_| { let t = draft.peek().clone(); rv.write().comment(fi, hr.1, ln, &t); draft.set(String::new()); composer.set(None); }, "Add comment" }
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
                if minimap {
                    div { class: "oxyn-diff-mini", "aria-hidden": "true",
                        for (mi, (y, k, c)) in marks.into_iter().enumerate() {
                            span { key: "{mi}", class: "oxyn-diff-mk", "data-k": "{k}", "data-cur": "{c}", style: "top:{y / rows_total * 100.0}%" }
                        }
                    }
                }
            }
            footer { class: "oxyn-diff-bar",
                button { class: "oxyn-diff-ib", r#type: "button", "aria-label": "Previous hunk", "aria-keyshortcuts": "k", onclick: move |_| { rv.write().step(-1); let n = rv.peek().cursor; reveal(n); }, Icon { name: "chevron-left", size: 13.0 } }
                span { class: "oxyn-diff-pos", "Hunk {pos} of {total}" }
                button { class: "oxyn-diff-ib", r#type: "button", "aria-label": "Next hunk", "aria-keyshortcuts": "j", onclick: move |_| { rv.write().step(1); let n = rv.peek().cursor; reveal(n); }, Icon { name: "chevron-right", size: 13.0 } }
                span { class: "oxyn-diff-left", if pend == 0 { "All reviewed" } else { "{pend} to review" } }
                span { class: "oxyn-diff-sp" }
                if ncomments > 0 {
                    button { class: "oxyn-diff-btn", r#type: "button", "aria-keyshortcuts": "Meta+Enter", onclick: move |_| send(),
                        Icon { name: "comment", size: 12.0 } "Send {ncomments} to agent" span { class: "oxyn-diff-kbd", "⌘↵" }
                    }
                } else if sent() > 0 {
                    span { class: "oxyn-diff-sent", Icon { name: "check", size: 11.0 } "Sent" }
                }
                if r.can_undo() {
                    button { class: "oxyn-diff-btn q", r#type: "button", "aria-keyshortcuts": "z", onclick: move |_| { let n = rv.write().undo(); live.set(format!("Restored {n}")); }, Icon { name: "undo", size: 12.0 } "Undo" }
                }
                if pend > 0 {
                    button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| all(Decision::No), "{m.no()} all" }
                    button { class: "oxyn-diff-btn p", r#type: "button", onclick: move |_| all(Decision::Yes), "{m.yes()} all" }
                }
            }
            div { class: "oxyn-diff-sr", role: "status", "aria-live": "polite", "{live}" }
        }
    }
}

/// A word-level diff for prose: removed words struck through, added words underlined, in one
/// paragraph (`inline`) or as a before line over an after line (`lines`).
#[component]
pub fn InlineDiff(
    /// Old text.
    before: String,
    /// New text.
    after: String,
    /// `inline` · `lines` · `after-only`.
    #[props(default = "inline".to_string())]
    form: String,
    /// Show the +/− word counts.
    #[props(default = true)]
    counts: bool,
    /// Review mode: each change is a suggestion to accept (Y) or reject (N), one at a time or
    /// all at once, with Undo (plan #32, ref-3).
    #[props(default)]
    review: bool,
    /// Review mode: the line above the text.
    #[props(default = "Suggested edit · Tighten it".to_string())]
    label: String,
    /// Review mode: decisions made from the start (`Some(true)` accepted).
    #[props(default)]
    decided: Vec<Option<bool>>,
    /// Review mode: `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Review mode: the text after each decision.
    #[props(default)]
    on_change: EventHandler<String>,
    /// Review mode: the final text once nothing is left to decide.
    #[props(default)]
    on_done: EventHandler<String>,
) -> Element {
    if review {
        return rsx! { InlineDiffReview { before, after, label, decided, style, on_change, on_done } };
    }
    let chunks = rewrite_chunks(&before, &after);
    let (mut a, mut r) = (0usize, 0usize);
    for c in &chunks {
        if let Chunk::Change { del, ins, .. } = c {
            r += del.split_whitespace().count();
            a += ins.split_whitespace().count();
        }
    }
    rsx! {
        div { class: "oxyn-idf", "data-form": "{form}",
            if counts {
                div { class: "oxyn-idf-hd", span { class: "a", "+{a} words" } span { class: "r", "−{r} words" } }
            }
            match form.as_str() {
                "lines" => rsx! {
                    p { class: "oxyn-idf-p", "data-side": "before", span { class: "oxyn-idf-lab", "Before" }
                        for (k, c) in chunks.iter().enumerate() {
                            match c {
                                Chunk::Same(t) => rsx! { span { key: "{k}", "{t}" } },
                                Chunk::Change { del, .. } => rsx! { del { key: "{k}", class: "oxyn-idf-del", "{del}" } },
                            }
                        }
                    }
                    p { class: "oxyn-idf-p", "data-side": "after", span { class: "oxyn-idf-lab", "After" }
                        for (k, c) in chunks.iter().enumerate() {
                            match c {
                                Chunk::Same(t) => rsx! { span { key: "{k}", "{t}" } },
                                Chunk::Change { ins, .. } => rsx! { ins { key: "{k}", class: "oxyn-idf-ins", "{ins}" } },
                            }
                        }
                    }
                },
                _ => rsx! {
                    p { class: "oxyn-idf-p",
                        for (k, c) in chunks.iter().enumerate() {
                            match c {
                                Chunk::Same(t) => rsx! { span { key: "{k}", "{t}" } },
                                Chunk::Change { del, ins, .. } => rsx! {
                                    span { key: "{k}", class: "oxyn-idf-ch",
                                        if form != "after-only" && !del.is_empty() { del { class: "oxyn-idf-del", "{del}" } }
                                        if !ins.is_empty() { ins { class: "oxyn-idf-ins", "{ins}" } }
                                    }
                                },
                            }
                        }
                    }
                },
            }
        }
    }
}

/// What an agent changed, at a glance: "4 files changed +58 −21", a block bar and a row per file
/// with its status and counts; `compact` folds to the headline with an expander.
#[component]
pub fn ChangeSummary(
    /// Files.
    files: Vec<FileDiff>,
    /// Start folded.
    #[props(default)]
    compact: bool,
    /// Blocks in the bar.
    #[props(default = 5)]
    blocks: usize,
    /// A file row was pressed.
    #[props(default)]
    on_open: EventHandler<String>,
) -> Element {
    let mut open = use_signal(move || !compact);
    let (n, a, r) = change_totals(&files);
    let (ba, br, bn) = stat_blocks(a, r, blocks);
    rsx! {
        div { class: "oxyn-chs", "data-open": "{open()}",
            button { class: "oxyn-chs-hd", r#type: "button", "aria-expanded": "{open()}", onclick: move |_| { let o = !open(); open.set(o) },
                Icon { name: "file", size: 13.0 }
                span { class: "oxyn-chs-t", "{files_changed(n)}" }
                span { class: "oxyn-chs-a", "+{a}" } span { class: "oxyn-chs-r", "−{r}" }
                span { class: "oxyn-chs-blocks", "aria-hidden": "true",
                    for i in 0..ba { i { key: "a{i}", "data-k": "add" } }
                    for i in 0..br { i { key: "r{i}", "data-k": "del" } }
                    for i in 0..bn { i { key: "n{i}" } }
                }
                span { class: "oxyn-chs-sp" }
                span { class: "oxyn-chs-chev", Icon { name: "chevron", size: 12.0 } }
            }
            if open() {
                ul { class: "oxyn-chs-list",
                    for f in files.iter() {
                        {
                            let p = f.path.clone();
                            let (fa, fr, fnn) = stat_blocks(f.added, f.removed, blocks);
                            rsx! {
                                li { key: "{f.path}",
                                    button { class: "oxyn-chs-row", r#type: "button", onclick: move |_| on_open.call(p.clone()),
                                        span { class: "oxyn-diff-fst", "data-s": f.status.id(), title: f.status.word(), "{f.status.letter()}" }
                                        span { class: "oxyn-chs-path", if !f.dir().is_empty() { span { class: "d", "{f.dir()}/" } } "{f.name()}" }
                                        span { class: "oxyn-chs-n", "{stat_label(f.added, f.removed)}" }
                                        span { class: "oxyn-chs-blocks", "aria-hidden": "true",
                                            for i in 0..fa { i { key: "a{i}", "data-k": "add" } }
                                            for i in 0..fr { i { key: "r{i}", "data-k": "del" } }
                                            for i in 0..fnn { i { key: "n{i}" } }
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

/// Row-level changes in a keyed table (settings, records, environment): added, removed and
/// changed rows with before → after, a filter for changed rows only and per-kind counts.
#[component]
pub fn DiffTable(
    /// Rows from `table_diff`.
    rows: Vec<RowChange>,
    /// Column header for the key.
    #[props(default = "Key".to_string())]
    key_label: String,
    /// Hide unchanged rows.
    #[props(default = true)]
    changed_only: bool,
    /// Show the before column (otherwise before is struck inline).
    #[props(default = true)]
    columns: bool,
) -> Element {
    let mut only = use_signal(move || changed_only);
    let count = |k: RowKind| rows.iter().filter(|r| r.kind == k).count();
    let (na, nr, nc) = (
        count(RowKind::Added),
        count(RowKind::Removed),
        count(RowKind::Changed),
    );
    let shown: Vec<RowChange> = rows
        .iter()
        .filter(|r| !only() || r.kind != RowKind::Same)
        .cloned()
        .collect();
    rsx! {
        div { class: "oxyn-dtb",
            div { class: "oxyn-dtb-hd",
                span { class: "oxyn-dtb-c", "data-k": "added", "+{na} added" }
                span { class: "oxyn-dtb-c", "data-k": "changed", "~{nc} changed" }
                span { class: "oxyn-dtb-c", "data-k": "removed", "−{nr} removed" }
                span { class: "oxyn-diff-sp" }
                label { class: "oxyn-diff-viewed", input { r#type: "checkbox", checked: only(), onchange: move |_| { let o = !only(); only.set(o) } } "Changed only" }
            }
            div { class: "oxyn-dtb-scroll", tabindex: "0", role: "region", "aria-label": "Table changes",
                table { class: "oxyn-dtb-t",
                    thead { tr {
                        th { scope: "col", class: "oxyn-dtb-s", span { class: "oxyn-diff-sr", "Change" } }
                        th { scope: "col", "{key_label}" }
                        if columns { th { scope: "col", "Before" } }
                        th { scope: "col", if columns { "After" } else { "Value" } }
                    } }
                    tbody {
                        for r in shown.iter() {
                            tr { key: "{r.key}", "data-k": r.kind.id(),
                                td { class: "oxyn-dtb-s", span { "aria-hidden": "true", "{r.kind.sign()}" } span { class: "oxyn-diff-sr", "{r.kind.id()}" } }
                                th { scope: "row", class: "oxyn-dtb-k", "{r.key}" }
                                if columns {
                                    td { class: "oxyn-dtb-b", {r.before.clone().unwrap_or_else(|| "—".into())} }
                                    td { class: "oxyn-dtb-a", {r.after.clone().unwrap_or_else(|| "—".into())} }
                                } else {
                                    td { class: "oxyn-dtb-a",
                                        if r.kind == RowKind::Changed { if let Some(b) = &r.before { del { class: "oxyn-idf-del", "{b}" } " " } }
                                        {r.after.clone().or(r.before.clone()).unwrap_or_default()}
                                    }
                                }
                            }
                        }
                        if shown.is_empty() { tr { td { colspan: "4", class: "oxyn-dtb-empty", "No changes" } } }
                    }
                }
            }
        }
    }
}

/// One file across saved revisions: a rail of revisions (who, what, +/−), pick two points to
/// compare, and the diff between them below. ← / → move the target; Shift moves the base.
#[component]
pub fn DiffTimeline(
    /// File path.
    path: String,
    /// Revisions, oldest first.
    revisions: Vec<Revision>,
    /// Clock for the time labels.
    now: ClockMs,
    /// Diff layout under the rail.
    #[props(default = "unified".to_string())]
    layout: String,
    /// The compare range changed.
    #[props(default)]
    on_range: EventHandler<(usize, usize)>,
) -> Element {
    let last = revisions.len().saturating_sub(1);
    let mut range = use_signal(move || (last.saturating_sub(1), last));
    let (from, to) = range();
    let stats = revision_stats(&revisions);
    let diff = revision_diff(&path, &revisions, from, to);
    let n = revisions.len();
    rsx! {
        div { class: "oxyn-dtl",
            div { class: "oxyn-dtl-rail", role: "listbox", tabindex: "0", "aria-label": "Revisions of {path}", "aria-activedescendant": "oxyn-dtl-r{to}",
                onkeydown: move |e: KeyboardEvent| {
                    let (f, t) = *range.peek();
                    let shift = e.modifiers().shift();
                    let next = match e.key() {
                        Key::ArrowRight if shift => ((f + 1).min(t.saturating_sub(1)), t),
                        Key::ArrowLeft if shift => (f.saturating_sub(1), t),
                        Key::ArrowRight => (f, (t + 1).min(n.saturating_sub(1))),
                        Key::ArrowLeft => { let nt = t.saturating_sub(1).max(1); (f.min(nt - 1), nt) }
                        Key::Home => (0, t),
                        Key::End => (f, n.saturating_sub(1)),
                        _ => return,
                    };
                    e.prevent_default();
                    range.set(next);
                    on_range.call(next);
                },
                for (i, rv) in revisions.iter().enumerate() {
                    {
                        let (a, r) = stats[i];
                        let role = if i == to { "to" } else if i == from { "from" } else if i > from && i < to { "in" } else { "" };
                        let ago = oxyn_core::parity::diff::waited_label(rv.at, now);
                        rsx! {
                            button { key: "{rv.id}", id: "oxyn-dtl-r{i}", class: "oxyn-dtl-node", r#type: "button", role: "option", "aria-selected": "{i == to}",
                                "data-role": "{role}", "data-who": if rv.who == "You" { "you" } else { "agent" },
                                title: "Click: compare up to here · Shift+click: compare from here",
                                onclick: move |e: MouseEvent| {
                                    let (f, t) = *range.peek();
                                    let next = if e.modifiers().shift() { (i.min(t.saturating_sub(1)), t.max(i + 1).min(n - 1)) } else if i == 0 { (0, 1.min(n - 1)) } else { (f.min(i - 1), i) };
                                    range.set(next);
                                    on_range.call(next);
                                },
                                span { class: "oxyn-dtl-dot", "aria-hidden": "true" }
                                span { class: "oxyn-dtl-lab", "{rv.label}" }
                                span { class: "oxyn-dtl-sub", "{rv.who} · {ago}" }
                                span { class: "oxyn-dtl-n", span { class: "a", "+{a}" } " " span { class: "r", "−{r}" } }
                            }
                        }
                    }
                }
            }
            div { class: "oxyn-dtl-cap",
                "Comparing " b { {revisions.get(from).map(|r| r.label.clone()).unwrap_or_default()} } " → " b { {revisions.get(to).map(|r| r.label.clone()).unwrap_or_default()} }
                span { class: "oxyn-diff-sp" }
                span { class: "oxyn-diff-stat", span { class: "a", "+{diff.added}" } span { class: "r", "−{diff.removed}" } }
            }
            for k in [format!("{from}-{to}-{layout}")] {
                DiffViewer { key: "{k}", files: vec![diff.clone()], layout: layout.clone(), file_list: false, minimap: false, comments: false, switcher: false, wrap: true, title: path.clone(), mode: ReviewMode::Applied }
            }
        }
    }
}

/// An AI code edit for one file: the instruction and model in the header, a file tab with a
/// running +/− count, hunks that stream in under a short "Reading" shimmer, Accept / Reject under
/// each hunk, and Accept all (⌘↵) / Reject all (⌘⌫). Decided hunks fold into the file.
#[component]
pub fn CodeSuggestion(
    /// The suggestion.
    suggestion: Suggestion,
    /// Clock (drives the stream).
    now: ClockMs,
    /// When the request started; `None` shows the finished suggestion.
    #[props(default)]
    started: Option<ClockMs>,
    /// Force a status (`error` shows the message and Retry).
    #[props(default)]
    status: Option<SugStatus>,
    /// Header title.
    #[props(default = "Add retry with backoff".to_string())]
    instruction: String,
    /// Model line (empty hides it).
    #[props(default = "Claude Sonnet".to_string())]
    model: String,
    /// File name in the tab.
    #[props(default = "fetch-invoice.ts".to_string())]
    file_name: String,
    /// Folder after the file name.
    #[props(default)]
    file_path: Option<String>,
    /// Error message.
    #[props(default = "The model stopped responding. Nothing was changed.".to_string())]
    error: String,
    /// ⌘↵ / ⌘⌫ shortcuts.
    #[props(default = true)]
    shortcuts: bool,
    /// Code area cap, px (0 = none).
    #[props(default)]
    max_code_height: u32,
    /// Hunk summaries in the bars.
    #[props(default = true)]
    summaries: bool,
    /// A hunk was decided: (id, accepted, file as it stands).
    #[props(default)]
    on_decision: EventHandler<(String, bool, String)>,
    /// Every hunk decided: final code.
    #[props(default)]
    on_resolve: EventHandler<String>,
    /// Stop pressed (shown while thinking or streaming).
    #[props(default)]
    on_stop: Option<EventHandler<()>>,
    /// Regenerate / Retry pressed.
    #[props(default)]
    on_regenerate: Option<EventHandler<()>>,
) -> Element {
    let init = suggestion.clone();
    let mut s = use_signal(move || init);
    let mut live = use_signal(String::new);
    let mut stopped = use_signal(|| None::<usize>);
    let sv = s.read().clone();
    let (mut st, mut shown) = match started {
        Some(t) => sv.play(now.0.saturating_sub(t.0)),
        None => (SugStatus::Ready, usize::MAX),
    };
    if let Some(n) = stopped() {
        st = SugStatus::Ready;
        shown = shown.min(n);
    }
    if let Some(f) = status {
        st = f;
    }
    let rows = if st == SugStatus::Thinking {
        sv.rows(0, false)
            .into_iter()
            .filter_map(|r| match r {
                SugRow::Del(_, n, t) => Some(SugRow::Line(n, t)),
                SugRow::Add(..) | SugRow::Bar(_) => None,
                x => Some(x),
            })
            .collect()
    } else {
        sv.rows(shown, st == SugStatus::Ready)
    };
    let (sa, sr) = if st == SugStatus::Thinking {
        (0, 0)
    } else {
        sv.stat()
    };
    let busy = matches!(st, SugStatus::Thinking | SugStatus::Streaming);
    let ready = st == SugStatus::Ready;
    let pending = sv.pending();
    let resolved = sv.resolved();
    let mut decide = move |id: String, yes: bool| {
        s.write().decide(&id, yes);
        let code = s.peek().apply();
        on_decision.call((id.clone(), yes, code.clone()));
        let sum = s
            .peek()
            .hunks
            .iter()
            .find(|h| h.id == id)
            .map(|h| h.summary.clone())
            .unwrap_or_default();
        let line = s.peek().status_line();
        live.set(format!(
            "{}: {sum}. {line}",
            if yes { "Accepted" } else { "Rejected" }
        ));
        if s.peek().resolved() {
            on_resolve.call(code);
        }
    };
    let mut all = move |yes: bool| {
        let ids = s.write().decide_all(yes);
        let code = s.peek().apply();
        for id in ids {
            on_decision.call((id, yes, code.clone()));
        }
        live.set(s.peek().status_line());
        on_resolve.call(code);
    };
    let cap = if max_code_height > 0 {
        format!("max-height:{max_code_height}px")
    } else {
        String::new()
    };
    let label = if busy {
        if st == SugStatus::Thinking {
            format!("Reading {file_name}")
        } else {
            format!("Writing {file_name}")
        }
    } else if st == SugStatus::Error {
        error.clone()
    } else if sv.hunks.is_empty() {
        "No changes needed".to_string()
    } else {
        sv.status_line()
    };
    let icon = match st {
        SugStatus::Error => "alert",
        _ if busy => "",
        _ if resolved => "check",
        _ => "dot",
    };
    rsx! {
        div { class: "oxyn-sug", role: "group", "aria-label": "Code suggestion: {instruction}", "data-status": st.id(),
            onkeydown: move |e: KeyboardEvent| {
                if !shortcuts || !ready { return; }
                let m = e.modifiers();
                if !(m.meta() || m.ctrl()) { return; }
                match e.key() {
                    Key::Enter => { e.prevent_default(); all(true) }
                    Key::Backspace => { e.prevent_default(); all(false) }
                    _ => {}
                }
            },
            div { class: "oxyn-sug-hd",
                span { class: "oxyn-sug-mark", Icon { name: "spark", size: 14.0 } }
                div { class: "oxyn-sug-ttl",
                    div { class: "oxyn-sug-t", "{instruction}" }
                    if !model.is_empty() { div { class: "oxyn-sug-m", "{model}" } }
                }
            }
            div { class: "oxyn-sug-panel",
                div { class: "oxyn-sug-tab",
                    span { class: "oxyn-sug-lang", "TS" }
                    span { class: "oxyn-sug-fn", "{file_name}" }
                    if let Some(p) = &file_path { span { class: "oxyn-sug-dir", "{p}" } }
                    span { class: "oxyn-diff-sp" }
                    span { class: "oxyn-sug-stat", "aria-label": "{sa} lines added, {sr} removed",
                        span { class: "a", "data-zero": "{sa == 0}", "+{sa}" } span { class: "r", "data-zero": "{sr == 0}", "−{sr}" }
                    }
                }
                div { class: "oxyn-sug-code", tabindex: "0", role: "region", "aria-label": "{file_name}", style: "{cap}",
                    for (ri, row) in rows.into_iter().enumerate() {
                        match row {
                            SugRow::Line(n, t) => rsx! {
                                div { key: "{ri}", class: "oxyn-sug-ln", span { class: "oxyn-sug-no", "aria-hidden": "true", "{n}" } span { class: "oxyn-sug-sg" } span { class: "oxyn-sug-src", {src_line(&t, &[], "ts", false)} } }
                            },
                            SugRow::Del(_, n, t) => rsx! {
                                div { key: "{ri}", class: "oxyn-sug-ln", "data-k": "del", span { class: "oxyn-sug-no", "aria-hidden": "true", "{n}" } span { class: "oxyn-sug-sg", "aria-hidden": "true", "−" } span { class: "oxyn-sug-src", span { class: "oxyn-diff-sr", "Removed: " } {src_line(&t, &[], "ts", false)} } }
                            },
                            SugRow::Add(_, t) => rsx! {
                                div { key: "{ri}", class: "oxyn-sug-ln", "data-k": "add", span { class: "oxyn-sug-no", "aria-hidden": "true" } span { class: "oxyn-sug-sg", "aria-hidden": "true", "+" } span { class: "oxyn-sug-src", span { class: "oxyn-diff-sr", "Added: " } {src_line(&t, &[], "ts", false)} } }
                            },
                            SugRow::Bar(id) => {
                                let sum = sv.hunks.iter().find(|h| h.id == id).map(|h| h.summary.clone()).unwrap_or_default();
                                let id2 = id.clone();
                                rsx! {
                                    div { key: "{ri}", class: "oxyn-sug-bar",
                                        if summaries { span { class: "oxyn-sug-sum", "{sum}" } }
                                        span { class: "oxyn-diff-sp" }
                                        button { class: "oxyn-diff-btn q", r#type: "button", "aria-label": "Reject {sum}", onclick: move |_| decide(id.clone(), false), "Reject" }
                                        button { class: "oxyn-diff-btn y", r#type: "button", "aria-label": "Accept {sum}", onclick: move |_| decide(id2.clone(), true), "Accept" }
                                    }
                                }
                            },
                        }
                    }
                }
            }
            div { class: "oxyn-sug-ft",
                span { class: "oxyn-sug-st",
                    if busy { span { class: "oxyn-sug-spin", "aria-hidden": "true" } } else if icon == "dot" { span { class: "oxyn-sug-dot", "aria-hidden": "true" } } else { Icon { name: icon, size: 13.0 } }
                    span { class: if busy { "oxyn-sug-shim" } else { "" }, "{label}" }
                }
                span { class: "oxyn-diff-sp" }
                if busy {
                    if let Some(h) = on_stop {
                        button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| { stopped.set(Some(shown)); h.call(()) }, Icon { name: "stop", size: 11.0 } "Stop" }
                    }
                } else if st == SugStatus::Error || (resolved || sv.hunks.is_empty()) {
                    if let Some(h) = on_regenerate {
                        button { class: "oxyn-diff-btn q", r#type: "button", onclick: move |_| h.call(()), Icon { name: "retry", size: 12.0 } if st == SugStatus::Error { "Retry" } else { "Regenerate" } }
                    }
                } else if pending > 0 {
                    button { class: "oxyn-diff-btn q", r#type: "button", "aria-keyshortcuts": "Meta+Backspace", onclick: move |_| all(false), "Reject all" }
                    button { class: "oxyn-diff-btn p", r#type: "button", "aria-keyshortcuts": "Meta+Enter", onclick: move |_| all(true), "Accept all" if shortcuts { span { class: "oxyn-diff-kbd", "⌘↵" } } }
                }
            }
            div { class: "oxyn-diff-sr", role: "status", "aria-live": "polite", "{live}" }
        }
    }
}

fn chunk_words(ch: &[Chunk], i: usize) -> String {
    ch.iter()
        .find_map(|x| match x {
            Chunk::Change { id, del, ins } if *id == i => Some(match (del.trim(), ins.trim()) {
                ("", a) => format!("add “{a}”"),
                (r, "") => format!("remove “{r}”"),
                (r, a) => format!("replace “{r}” with “{a}”"),
            }),
            _ => None,
        })
        .unwrap_or_default()
}

/// The review form of [`InlineDiff`]: struck removals and tinted insertions in running prose; the
/// highlighted change carries a small Accept (Y) / Reject (N) bar above it; prev / next step
/// through what is left; Accept all / Reject all; Undo walks back one decision.
#[component]
fn InlineDiffReview(
    before: String,
    after: String,
    label: String,
    decided: Vec<Option<bool>>,
    style: String,
    on_change: EventHandler<String>,
    on_done: EventHandler<String>,
) -> Element {
    let chunks = rewrite_chunks(&before, &after);
    let n = oxyn_core::parity::diff::change_count(&chunks);
    let init = {
        let mut d = decided.clone();
        d.resize(n, None);
        d
    };
    let first = rewrite_next(&init, n.saturating_sub(1)).unwrap_or(0);
    let mut dec = use_signal(move || init.clone());
    let mut hist = use_signal(Vec::<Vec<Option<bool>>>::new);
    let mut cur = use_signal(move || first);
    let mut live = use_signal(String::new);
    // Chunks live in a signal so the handlers below stay `Copy` (the demo re-keys on new text).
    let chs = use_signal(move || chunks.clone());
    let chunks = chs();
    let left = dec().iter().filter(|d| d.is_none()).count();
    let finished = n > 0 && left == 0;
    let c = cur().min(n.saturating_sub(1));
    let words = move |i: usize| -> String { chunk_words(&chs.read(), i) };
    let mut decide = move |i: usize, yes: bool| {
        let mut d = dec();
        if i >= d.len() || d[i].is_some() {
            return;
        }
        hist.with_mut(|h| h.push(d.clone()));
        d[i] = Some(yes);
        dec.set(d.clone());
        let left = d.iter().filter(|x| x.is_none()).count();
        let what = words(i);
        let what = format!("{}{}", what[..1].to_uppercase(), &what[1..]);
        live.set(format!(
            "{}: {what}. {left} left.",
            if yes { "Accepted" } else { "Rejected" }
        ));
        let text = rewrite_apply(&chs.read(), &d);
        on_change.call(text.clone());
        if left == 0 {
            on_done.call(text);
        } else if let Some(nx) = rewrite_next(&d, i) {
            cur.set(nx);
        }
    };
    let mut all = move |yes: bool| {
        let d0 = dec();
        if d0.iter().all(|x| x.is_some()) {
            return;
        }
        hist.with_mut(|h| h.push(d0.clone()));
        let d: Vec<Option<bool>> = d0.iter().map(|x| x.or(Some(yes))).collect();
        dec.set(d.clone());
        live.set(if yes {
            "Accepted every change.".into()
        } else {
            "Rejected every change.".into()
        });
        let text = rewrite_apply(&chs.read(), &d);
        on_change.call(text.clone());
        on_done.call(text);
    };
    let undo = move |_| {
        if let Some(p) = hist.with_mut(|h| h.pop()) {
            let back = p.iter().zip(dec().iter()).position(|(a, b)| a != b);
            dec.set(p);
            if let Some(b) = back {
                cur.set(b);
            }
            live.set("Undone.".into());
        }
    };
    let step = move |fwd: bool| {
        let d = dec();
        let open: Vec<usize> = (0..d.len()).filter(|&i| d[i].is_none()).collect();
        let c = cur();
        if fwd {
            open.iter().copied().find(|&i| i > c)
        } else {
            open.iter().rev().copied().find(|&i| i < c)
        }
    };
    let has_prev = step(false).is_some();
    let has_next = step(true).is_some();
    let count_s = if left == 1 {
        "1 suggestion".to_string()
    } else {
        format!("{left} suggestions")
    };
    let style_a = (!style.is_empty()).then(|| style.clone());
    rsx! {
        div { class: "oxyn-idr", "data-style": style_a, "data-done": finished, role: "group", "aria-label": "{label}", tabindex: "-1",
            onkeydown: move |e: KeyboardEvent| {
                if finished { return; }
                match e.key() {
                    Key::Character(k) if k == "y" || k == "Y" => { e.prevent_default(); decide(cur(), true); }
                    Key::Character(k) if k == "n" || k == "N" => { e.prevent_default(); decide(cur(), false); }
                    _ => {}
                }
            },
            div { class: "oxyn-idr-lab", Icon { name: "spark", size: 12.0 } span { "{label}" } }
            p { class: "oxyn-idr-p",
                for x in chunks.iter() {
                    match x {
                        Chunk::Same(t) => rsx! { "{t}" },
                        Chunk::Change { id, del, ins } => {
                            let i = *id;
                            let d = dec().get(i).copied().flatten();
                            let on = !finished && d.is_none() && i == c;
                            let aria = format!("Change {} of {n}: {}. Y keeps it, N drops it", i + 1, words(i));
                            match d {
                                Some(true) => rsx! { span { key: "c{i}", "data-k": "c{i}", class: "oxyn-idr-ok", "{ins}" } },
                                Some(false) => rsx! { span { key: "c{i}", "data-k": "c{i}", class: "oxyn-idr-ok", "{del}" } },
                                None => rsx! {
                                    span { key: "c{i}", "data-k": "c{i}", class: "oxyn-idr-ch", "data-on": on,
                                        button { class: "oxyn-idr-hit", r#type: "button", "aria-label": "{aria}", "aria-pressed": on,
                                            onclick: move |_| cur.set(i),
                                            if !del.is_empty() { del { "{del}" } }
                                            if !ins.is_empty() { ins { "{ins}" } }
                                        }
                                        if on {
                                            span { class: "oxyn-idr-pop", role: "toolbar", "aria-label": "Review suggestion",
                                                button { class: "oxyn-idr-yes", r#type: "button", onclick: move |_| decide(i, true), Icon { name: "check", size: 11.0 } "Accept" kbd { "Y" } }
                                                button { class: "oxyn-idr-no", r#type: "button", onclick: move |_| decide(i, false), Icon { name: "x", size: 11.0 } "Reject" kbd { "N" } }
                                            }
                                        }
                                    }
                                },
                            }
                        }
                    }
                }
            }
            div { class: "oxyn-idr-ft",
                if finished {
                    span { class: "oxyn-idr-fin", Icon { name: "check", size: 12.0 } "All changes reviewed" }
                    span { class: "oxyn-diff-sp" }
                    button { class: "oxyn-idr-btn", r#type: "button", onclick: undo, Icon { name: "undo", size: 12.0 } "Undo" }
                } else {
                    span { class: "oxyn-idr-cnt", "{count_s}" }
                    button { class: "oxyn-idr-arw", r#type: "button", "aria-label": "Previous suggestion", disabled: !has_prev, onclick: move |_| if let Some(p) = step(false) { cur.set(p) }, Icon { name: "chevron-left", size: 12.0 } }
                    button { class: "oxyn-idr-arw", r#type: "button", "aria-label": "Next suggestion", disabled: !has_next, onclick: move |_| if let Some(p) = step(true) { cur.set(p) }, Icon { name: "chevron-right", size: 12.0 } }
                    span { class: "oxyn-diff-sp" }
                    if !hist().is_empty() { button { class: "oxyn-idr-btn q", r#type: "button", onclick: undo, Icon { name: "undo", size: 12.0 } "Undo" } }
                    button { class: "oxyn-idr-btn", r#type: "button", onclick: move |_| all(false), "Reject all" }
                    button { class: "oxyn-idr-btn p", r#type: "button", onclick: move |_| all(true), "Accept all" }
                }
            }
            div { class: "oxyn-diff-sr", role: "status", "aria-live": "polite", "{live}" }
        }
    }
}
