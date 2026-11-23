//! Diff review for GPUI (plan #30), drawn from `oxyn_core::kit::diff`: diff viewer (unified,
//! split, stacked) with per-hunk actions, inline diff, change summary, diff table, diff timeline,
//! code suggestion, AI inline edit, image compare, comment thread, review center and review card.
//! Elements are `RenderOnce`; the host keeps the model (a `Review`, `Suggestion`, `Thread`,
//! `Center`…), routes the callbacks into it and re-renders.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, relative,
};
use oxyn_core::{
    ClockMs,
    kit::diff::{
        Center, Chunk, Comment, DLine, Decision, FileDiff, Finding, FindingState, HunkRef,
        LineKind, Review, Revision, RowChange, RowKind, Sla, SugRow, SugStatus, Suggestion, Thread,
        change_totals, files_changed, mention_spans, review_verdict, revision_diff, revision_stats,
        rewrite_chunks, severity_counts, split_rows, stat_blocks, stat_label, waited_label,
    },
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Weight, alpha, button, c, card, muted, tag};

/// Hunk callback: the hunk, the decision (`Pending` = restore) and the window.
pub type HunkAct = Rc<dyn Fn(HunkRef, Decision, &mut Window, &mut App)>;
/// Id callback: an item id, an action id and the window.
pub type IdAct = Rc<dyn Fn(String, &'static str, &mut Window, &mut App)>;

fn mono() -> gpui::Div {
    div().font_family("monospace").text_size(px(12.))
}

fn line_bg(t: &'static Theme, k: LineKind) -> Option<Hsla> {
    match k {
        LineKind::Add => Some(c(t, ColorRole::DiffAddBg)),
        LineKind::Del => Some(c(t, ColorRole::DiffDelBg)),
        LineKind::Ctx => None,
    }
}

fn sign_fg(t: &'static Theme, k: LineKind) -> Hsla {
    match k {
        LineKind::Add => c(t, ColorRole::DiffAddFg),
        LineKind::Del => c(t, ColorRole::DiffDelFg),
        LineKind::Ctx => c(t, ColorRole::FgSubtle),
    }
}

fn line_text(t: &'static Theme, l: &DLine) -> gpui::Div {
    let mut row = div().flex().flex_1().min_w(px(0.));
    if l.words.is_empty() {
        return row.child(l.text.clone());
    }
    let word_bg = if l.kind == LineKind::Add {
        c(t, ColorRole::DiffAddWord)
    } else {
        c(t, ColorRole::DiffDelWord)
    };
    for s in &l.words {
        row = row.child(
            div()
                .when(s.changed, |d| d.bg(word_bg).rounded(px(2.)))
                .child(s.text.replace(' ', "\u{a0}")),
        );
    }
    row
}

fn diff_line(t: &'static Theme, l: &DLine, numbers: bool) -> gpui::Div {
    let no = |n: Option<usize>| {
        div()
            .w(px(36.))
            .flex_none()
            .text_color(c(t, ColorRole::FgSubtle))
            .child(n.map(|n| n.to_string()).unwrap_or_default())
    };
    mono()
        .flex()
        .h(px(20.))
        .items_center()
        .when_some(line_bg(t, l.kind), |d, b| d.bg(b))
        .text_color(c(t, ColorRole::FgDefault))
        .when(numbers, |d| d.child(no(l.old)).child(no(l.new)))
        .child(
            div()
                .w(px(16.))
                .flex_none()
                .text_color(sign_fg(t, l.kind))
                .child(l.kind.sign()),
        )
        .child(line_text(t, l))
}

fn stat(t: &'static Theme, a: usize, r: usize) -> gpui::Div {
    mono()
        .flex()
        .gap(px(6.))
        .child(
            div()
                .text_color(c(t, ColorRole::DiffAddFg))
                .child(format!("+{a}")),
        )
        .child(
            div()
                .text_color(c(t, ColorRole::DiffDelFg))
                .child(format!("−{r}")),
        )
}

fn status_badge(t: &'static Theme, f: &FileDiff) -> impl IntoElement {
    let role = match f.status.id() {
        "added" => ColorRole::DiffAddFg,
        "deleted" => ColorRole::DiffDelFg,
        _ => ColorRole::FgMuted,
    };
    tag(t, f.status.letter(), Some(role))
}

/// `diff-viewer`: a review over a change set in `unified`, `split` or `stacked` layout, with
/// Accept / Reject (or Keep / Undo) per hunk, per file and for all.
#[derive(IntoElement)]
pub struct DiffViewerEl {
    review: Review,
    layout: &'static str,
    numbers: bool,
    on_hunk: Option<HunkAct>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl DiffViewerEl {
    /// A viewer over a review model.
    pub fn new(review: Review) -> Self {
        Self {
            review,
            layout: "unified",
            numbers: true,
            on_hunk: None,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// `unified` · `split` · `stacked`.
    pub fn layout(mut self, l: &'static str) -> Self {
        self.layout = l;
        self
    }
    /// Line-number gutters.
    pub fn numbers(mut self, on: bool) -> Self {
        self.numbers = on;
        self
    }
    /// Per-hunk decisions.
    pub fn on_hunk(mut self, f: HunkAct) -> Self {
        self.on_hunk = Some(f);
        self
    }
    /// Review bar: `yes-all`, `no-all`, `undo`, `prev`, `next`, `send`.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
    /// Theme.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
}

impl RenderOnce for DiffViewerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let rv = &self.review;
        let m = rv.mode;
        let (nf, _, _) = change_totals(&rv.files);
        let (la, lr) = rv.live_stat();
        let (pend, _, _) = rv.counts();
        let (pos, total) = rv.position();
        let mut root = card(t).p(px(0.)).gap(px(0.)).overflow_hidden().child(
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .px(px(14.))
                .h(px(40.))
                .border_b_1()
                .border_color(c(t, ColorRole::BorderSubtle))
                .child(div().font_weight(FontWeight::SEMIBOLD).child("Changes"))
                .child(muted(t, files_changed(nf)))
                .child(stat(t, la, lr)),
        );
        for (fi, f) in rv.files.iter().enumerate() {
            root = root.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(10.))
                    .h(px(34.))
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .child(status_badge(t, f))
                    .child(mono().child(f.path.clone()))
                    .child(muted(t, stat_label(f.added, f.removed))),
            );
            for h in &f.hunks {
                let hr = (fi, h.id);
                let d = rv.decision(hr);
                let cur = rv.cursor == hr;
                let mut head = div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(10.))
                    .h(px(32.))
                    .bg(c(t, ColorRole::DiffHunkBg))
                    .text_size(px(12.))
                    .text_color(c(t, ColorRole::DiffHunkHeader))
                    .when(cur, |x| {
                        x.border_l_2().border_color(c(t, ColorRole::FgDefault))
                    })
                    .child(mono().text_color(c(t, ColorRole::FgMuted)).child(h.range()))
                    .child(mono().child(h.header.clone()))
                    .child(div().flex_1());
                match d {
                    Decision::Pending => {
                        for (label, dec, id) in [
                            (m.no(), Decision::No, "no"),
                            (m.yes(), Decision::Yes, "yes"),
                        ] {
                            let cb = self.on_hunk.clone();
                            head = head.child(
                                div()
                                    .id(SharedString::from(format!("h{fi}-{}-{id}", h.id)))
                                    .px(px(9.))
                                    .h(px(24.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(7.))
                                    .border_1()
                                    .border_color(c(t, ColorRole::BorderDefault))
                                    .bg(c(t, ColorRole::SurfaceRaised))
                                    .text_color(c(t, ColorRole::FgDefault))
                                    .cursor_pointer()
                                    .child(label)
                                    .when_some(cb, move |x, f| {
                                        x.on_click(move |_, w, cx| f(hr, dec, w, cx))
                                    }),
                            );
                        }
                    }
                    _ => {
                        let word = if d == Decision::Yes {
                            m.yes_done()
                        } else {
                            m.no_done()
                        };
                        let role = if d == Decision::Yes {
                            ColorRole::DiffKept
                        } else {
                            ColorRole::DiffUndone
                        };
                        let cb = self.on_hunk.clone();
                        head = head.child(tag(t, word, Some(role))).child(
                            div()
                                .id(SharedString::from(format!("h{fi}-{}-restore", h.id)))
                                .text_color(c(t, ColorRole::FgMuted))
                                .cursor_pointer()
                                .child("Restore")
                                .when_some(cb, move |x, f| {
                                    x.on_click(move |_, w, cx| f(hr, Decision::Pending, w, cx))
                                }),
                        );
                    }
                }
                let mut body = div().flex().flex_col();
                match self.layout {
                    "split" => {
                        for (a, b) in split_rows(&h.lines) {
                            let half = |l: Option<DLine>| match l {
                                Some(l) => diff_line(t, &l, false).flex_1().min_w(px(0.)),
                                None => div()
                                    .flex_1()
                                    .h(px(20.))
                                    .bg(alpha(c(t, ColorRole::BorderSubtle), 0.5)),
                            };
                            body = body.child(div().flex().child(half(a)).child(half(b)));
                        }
                    }
                    "stacked" => {
                        for (label, skip, role) in [
                            ("BEFORE", LineKind::Add, ColorRole::DiffDelFg),
                            ("AFTER", LineKind::Del, ColorRole::DiffAddFg),
                        ] {
                            let mut blk = div()
                                .flex()
                                .flex_col()
                                .m(px(6.))
                                .rounded(px(8.))
                                .border_1()
                                .border_color(c(t, ColorRole::BorderSubtle))
                                .overflow_hidden()
                                .child(
                                    div()
                                        .px(px(10.))
                                        .py(px(3.))
                                        .text_size(px(11.))
                                        .font_weight(FontWeight::SEMIBOLD)
                                        .text_color(c(t, role))
                                        .child(label),
                                );
                            for l in h.lines.iter().filter(|l| l.kind != skip) {
                                blk = blk.child(diff_line(t, l, false));
                            }
                            body = body.child(blk);
                        }
                    }
                    _ => {
                        for l in &h.lines {
                            body = body.child(diff_line(t, l, self.numbers));
                        }
                    }
                }
                root = root.child(
                    div()
                        .flex()
                        .flex_col()
                        .when(d != Decision::Pending, |x| x.opacity(0.75))
                        .child(head)
                        .child(body),
                );
            }
        }
        let a = self.act.clone();
        let b = self.act.clone();
        root.child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .px(px(10.))
                .h(px(40.))
                .border_t_1()
                .border_color(c(t, ColorRole::BorderSubtle))
                .child(
                    div()
                        .text_size(px(12.))
                        .child(format!("Hunk {pos} of {total}")),
                )
                .child(muted(
                    t,
                    if pend == 0 {
                        "All reviewed".to_string()
                    } else {
                        format!("{pend} to review")
                    },
                ))
                .child(div().flex_1())
                .when(pend > 0, |x| {
                    x.child(button(
                        t,
                        "no-all",
                        format!("{} all", m.no()),
                        Weight::Quiet,
                        "no-all",
                        a,
                    ))
                    .child(button(
                        t,
                        "yes-all",
                        format!("{} all", m.yes()),
                        Weight::Primary,
                        "yes-all",
                        b,
                    ))
                }),
        )
    }
}

/// `inline-diff`: prose with removed words struck and added words underlined.
#[derive(IntoElement)]
pub struct InlineDiffEl {
    before: String,
    after: String,
    theme: &'static Theme,
}

impl InlineDiffEl {
    /// From old to new text.
    pub fn new(before: impl Into<String>, after: impl Into<String>) -> Self {
        Self {
            before: before.into(),
            after: after.into(),
            theme: &Theme::DARK,
        }
    }
}

fn chunks_row(t: &'static Theme, chunks: &[Chunk], decisions: &BTreeMap<usize, bool>) -> gpui::Div {
    let mut p = div()
        .flex()
        .flex_wrap()
        .text_size(px(14.))
        .line_height(px(24.));
    for ch in chunks {
        match ch {
            Chunk::Same(s) => {
                for w in s.split_inclusive(' ') {
                    p = p.child(div().child(w.replace(' ', "\u{a0}")));
                }
            }
            Chunk::Change { id, del, ins } => {
                let d = decisions.get(id).copied();
                if d != Some(true) && !del.is_empty() {
                    p = p.child(
                        div()
                            .line_through()
                            .text_color(c(t, ColorRole::DiffDelFg))
                            .bg(c(t, ColorRole::DiffDelBg))
                            .rounded(px(3.))
                            .child(del.replace(' ', "\u{a0}")),
                    );
                }
                if d != Some(false) && !ins.is_empty() {
                    p = p.child(
                        div()
                            .underline()
                            .text_color(c(t, ColorRole::DiffAddFg))
                            .bg(c(t, ColorRole::DiffAddBg))
                            .rounded(px(3.))
                            .child(ins.replace(' ', "\u{a0}")),
                    );
                }
            }
        }
    }
    p
}

impl RenderOnce for InlineDiffEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let ch = rewrite_chunks(&self.before, &self.after);
        card(t)
            .max_w(px(640.))
            .child(chunks_row(t, &ch, &BTreeMap::new()))
    }
}

/// `diff-timeline`: a file across revisions with the diff between two of them.
#[derive(IntoElement)]
pub struct DiffTimelineEl {
    path: String,
    revisions: Vec<Revision>,
    range: (usize, usize),
    now: ClockMs,
    on_pick: Option<IdAct>,
    theme: &'static Theme,
}

impl DiffTimelineEl {
    /// Revisions, oldest first; compares the last two.
    pub fn new(path: impl Into<String>, revisions: Vec<Revision>, now: ClockMs) -> Self {
        let n = revisions.len().saturating_sub(1);
        Self {
            path: path.into(),
            revisions,
            range: (n.saturating_sub(1), n),
            now,
            on_pick: None,
            theme: &Theme::DARK,
        }
    }
    /// Compare range (from, to).
    pub fn range(mut self, r: (usize, usize)) -> Self {
        self.range = r;
        self
    }
    /// Reports `(revision id, "to" | "from")` when a node is clicked.
    pub fn on_pick(mut self, f: IdAct) -> Self {
        self.on_pick = Some(f);
        self
    }
}

impl RenderOnce for DiffTimelineEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let stats = revision_stats(&self.revisions);
        let (from, to) = self.range;
        let mut rail = div().flex().gap(px(4.));
        for (i, r) in self.revisions.iter().enumerate() {
            let on = i == from || i == to;
            let cb = self.on_pick.clone();
            let id = r.id.clone();
            rail = rail.child(
                div()
                    .id(SharedString::from(format!("rev-{i}")))
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .flex_1()
                    .p(px(8.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                    .child(
                        div()
                            .size(px(10.))
                            .rounded_full()
                            .border_2()
                            .border_color(c(t, ColorRole::BorderStrong))
                            .when(on, |d| d.bg(c(t, ColorRole::FgDefault))),
                    )
                    .child(
                        div()
                            .text_size(px(12.5))
                            .font_weight(FontWeight::MEDIUM)
                            .child(r.label.clone()),
                    )
                    .child(muted(
                        t,
                        format!("{} · {}", r.who, waited_label(r.at, self.now)),
                    ))
                    .child(stat(t, stats[i].0, stats[i].1))
                    .when_some(cb, move |d, f| {
                        d.on_click(move |_, w, cx| f(id.clone(), "to", w, cx))
                    }),
            );
        }
        let diff = revision_diff(&self.path, &self.revisions, from, to);
        div().flex().flex_col().gap(px(10.)).child(rail).child(
            DiffViewerEl::new(Review::new(
                vec![diff],
                oxyn_core::kit::diff::ReviewMode::Applied,
            ))
            .theme(t),
        )
    }
}

/// `code-suggestion`: an AI edit for one file with Accept / Reject under each hunk.
#[derive(IntoElement)]
pub struct CodeSuggestionEl {
    suggestion: Suggestion,
    status: SugStatus,
    shown: usize,
    instruction: SharedString,
    model: SharedString,
    file: SharedString,
    on_hunk: Option<IdAct>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl CodeSuggestionEl {
    /// A finished suggestion.
    pub fn new(suggestion: Suggestion) -> Self {
        Self {
            suggestion,
            status: SugStatus::Ready,
            shown: usize::MAX,
            instruction: "Add retry with backoff".into(),
            model: "Claude Sonnet".into(),
            file: "fetch-invoice.ts".into(),
            on_hunk: None,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Status and inserted lines shown (from `Suggestion::play`).
    pub fn play(mut self, s: (SugStatus, usize)) -> Self {
        self.status = s.0;
        self.shown = s.1;
        self
    }
    /// Header text.
    pub fn instruction(mut self, s: impl Into<SharedString>) -> Self {
        self.instruction = s.into();
        self
    }
    /// Reports `(hunk id, "accept" | "reject")`.
    pub fn on_hunk(mut self, f: IdAct) -> Self {
        self.on_hunk = Some(f);
        self
    }
    /// Reports `accept-all`, `reject-all`, `stop`, `regenerate`.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for CodeSuggestionEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let s = &self.suggestion;
        let ready = self.status == SugStatus::Ready;
        let (sa, sr) = s.stat();
        let mut code = div().flex().flex_col().py(px(6.));
        let num = |n: String| {
            div()
                .w(px(36.))
                .flex_none()
                .text_color(c(t, ColorRole::FgSubtle))
                .child(n)
        };
        for row in s.rows(
            if self.status == SugStatus::Thinking {
                0
            } else {
                self.shown
            },
            ready,
        ) {
            code = code.child(match row {
                SugRow::Line(n, txt) => mono()
                    .flex()
                    .h(px(20.))
                    .child(num(n.to_string()))
                    .child(div().w(px(16.)))
                    .child(txt),
                SugRow::Del(_, n, txt) => mono()
                    .flex()
                    .h(px(20.))
                    .bg(c(t, ColorRole::DiffDelBg))
                    .child(num(n.to_string()))
                    .child(
                        div()
                            .w(px(16.))
                            .text_color(c(t, ColorRole::DiffDelFg))
                            .child("−"),
                    )
                    .child(txt),
                SugRow::Add(_, txt) => mono()
                    .flex()
                    .h(px(20.))
                    .bg(c(t, ColorRole::DiffAddBg))
                    .child(num(String::new()))
                    .child(
                        div()
                            .w(px(16.))
                            .text_color(c(t, ColorRole::DiffAddFg))
                            .child("+"),
                    )
                    .child(txt),
                SugRow::Bar(id) => {
                    let sum = s
                        .hunks
                        .iter()
                        .find(|h| h.id == id)
                        .map(|h| h.summary.clone())
                        .unwrap_or_default();
                    let mut bar = div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .pl(px(52.))
                        .pr(px(8.))
                        .py(px(4.))
                        .border_t_1()
                        .border_b_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .child(muted(t, sum))
                        .child(div().flex_1());
                    for (label, a) in [("Reject", "reject"), ("Accept", "accept")] {
                        let cb = self.on_hunk.clone();
                        let id2 = id.clone();
                        bar = bar.child(
                            div()
                                .id(SharedString::from(format!("{id}-{a}")))
                                .px(px(9.))
                                .h(px(24.))
                                .flex()
                                .items_center()
                                .rounded(px(7.))
                                .border_1()
                                .border_color(c(t, ColorRole::BorderDefault))
                                .cursor_pointer()
                                .child(label)
                                .when_some(cb, move |d, f| {
                                    d.on_click(move |_, w, cx| f(id2.clone(), a, w, cx))
                                }),
                        );
                    }
                    bar
                }
            });
        }
        let label: String = match self.status {
            SugStatus::Thinking => format!("Reading {}", self.file),
            SugStatus::Streaming => format!("Writing {}", self.file),
            SugStatus::Error => "The model stopped responding. Nothing was changed.".into(),
            SugStatus::Ready => s.status_line(),
        };
        let (a1, a2) = (self.act.clone(), self.act.clone());
        card(t)
            .max_w(px(720.))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.instruction.clone()),
            )
            .child(muted(t, self.model.clone()))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .rounded(px(10.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .bg(c(t, ColorRole::SurfaceBase))
                    .overflow_hidden()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .px(px(10.))
                            .h(px(32.))
                            .border_b_1()
                            .border_color(c(t, ColorRole::BorderSubtle))
                            .child(mono().child(self.file.clone()))
                            .child(div().flex_1())
                            .child(stat(t, sa, sr)),
                    )
                    .child(code),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(muted(t, label))
                    .child(div().flex_1())
                    .when(ready && s.pending() > 0, |d| {
                        d.child(button(
                            t,
                            "sug-rej",
                            "Reject all",
                            Weight::Quiet,
                            "reject-all",
                            a1,
                        ))
                        .child(button(
                            t,
                            "sug-acc",
                            "Accept all",
                            Weight::Primary,
                            "accept-all",
                            a2,
                        ))
                    }),
            )
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
