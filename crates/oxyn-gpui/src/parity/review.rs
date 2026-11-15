//! Diff review for GPUI (plan #30), drawn from `oxyn_core::parity::diff`: diff viewer (unified,
//! split, stacked) with per-hunk actions, inline diff, change summary, diff table, diff timeline,
//! code suggestion, AI inline edit, image compare, comment thread, review center and review card.
//! Elements are `RenderOnce`; the host keeps the model (a `Review`, `Suggestion`, `Thread`,
//! `Center`…), routes the callbacks into it and re-renders.

use std::collections::BTreeMap;
use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, relative,
};
use oxyn_core::{
    ClockMs,
    parity::diff::{
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

/// `change-summary`: "4 files changed +58 −21" with a block bar and a row per file.
#[derive(IntoElement)]
pub struct ChangeSummaryEl {
    files: Vec<FileDiff>,
    open: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ChangeSummaryEl {
    /// Over a change set.
    pub fn new(files: Vec<FileDiff>) -> Self {
        Self {
            files,
            open: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Rows shown.
    pub fn open(mut self, on: bool) -> Self {
        self.open = on;
        self
    }
    /// Reports `toggle`.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

fn blocks(t: &'static Theme, a: usize, r: usize) -> gpui::Div {
    let (ba, br, bn) = stat_blocks(a, r, 5);
    let sq = |col: Hsla| div().size(px(8.)).rounded(px(2.)).bg(col);
    let mut row = div().flex().gap(px(2.));
    for _ in 0..ba {
        row = row.child(sq(c(t, ColorRole::DiffAddStripe)));
    }
    for _ in 0..br {
        row = row.child(sq(c(t, ColorRole::DiffDelStripe)));
    }
    for _ in 0..bn {
        row = row.child(sq(c(t, ColorRole::BorderDefault)));
    }
    row
}

impl RenderOnce for ChangeSummaryEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let (n, a, r) = change_totals(&self.files);
        let act = self.act.clone();
        let mut root = card(t).max_w(px(560.)).child(
            div()
                .id("chs-head")
                .flex()
                .items_center()
                .gap(px(8.))
                .cursor_pointer()
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(files_changed(n)),
                )
                .child(stat(t, a, r))
                .child(blocks(t, a, r))
                .when_some(act, |d, f| d.on_click(move |_, w, cx| f("toggle", w, cx))),
        );
        if self.open {
            for f in &self.files {
                root = root.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .h(px(26.))
                        .child(status_badge(t, f))
                        .child(mono().flex_1().child(f.path.clone()))
                        .child(muted(t, stat_label(f.added, f.removed)))
                        .child(blocks(t, f.added, f.removed)),
                );
            }
        }
        root
    }
}

/// `diff-table`: keyed rows added, removed and changed, with before → after.
#[derive(IntoElement)]
pub struct DiffTableEl {
    rows: Vec<RowChange>,
    changed_only: bool,
    theme: &'static Theme,
}

impl DiffTableEl {
    /// Over rows from `table_diff`.
    pub fn new(rows: Vec<RowChange>) -> Self {
        Self {
            rows,
            changed_only: true,
            theme: &Theme::DARK,
        }
    }
    /// Hide unchanged rows.
    pub fn changed_only(mut self, on: bool) -> Self {
        self.changed_only = on;
        self
    }
}

impl RenderOnce for DiffTableEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let mut root = card(t)
            .p(px(0.))
            .gap(px(0.))
            .max_w(px(680.))
            .overflow_hidden();
        for r in self
            .rows
            .iter()
            .filter(|r| !self.changed_only || r.kind != RowKind::Same)
        {
            let bg = match r.kind {
                RowKind::Added => Some(c(t, ColorRole::DiffAddBg)),
                RowKind::Removed => Some(c(t, ColorRole::DiffDelBg)),
                _ => None,
            };
            root = root.child(
                mono()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .px(px(10.))
                    .h(px(30.))
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .when_some(bg, |d, b| d.bg(b))
                    .child(
                        div()
                            .w(px(14.))
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(r.kind.sign()),
                    )
                    .child(
                        div()
                            .w(px(140.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(r.key.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(c(t, ColorRole::DiffDelFg))
                            .when(r.kind == RowKind::Removed, |d| d.line_through())
                            .child(r.before.clone().unwrap_or_else(|| "—".into())),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_color(c(t, ColorRole::DiffAddFg))
                            .child(r.after.clone().unwrap_or_else(|| "—".into())),
                    ),
            );
        }
        root
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
                oxyn_core::parity::diff::ReviewMode::Applied,
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

/// `ai-inline-edit`: a paragraph with a rewrite shown as accepted / rejected / pending changes.
#[derive(IntoElement)]
pub struct InlineEditEl {
    title: SharedString,
    before: String,
    after: String,
    decisions: BTreeMap<usize, bool>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl InlineEditEl {
    /// A selection and its rewrite.
    pub fn new(
        title: impl Into<SharedString>,
        before: impl Into<String>,
        after: impl Into<String>,
    ) -> Self {
        Self {
            title: title.into(),
            before: before.into(),
            after: after.into(),
            decisions: BTreeMap::new(),
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Decisions by change id.
    pub fn decisions(mut self, d: BTreeMap<usize, bool>) -> Self {
        self.decisions = d;
        self
    }
    /// Reports `accept-all`, `reject-all`.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for InlineEditEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let ch = rewrite_chunks(&self.before, &self.after);
        let n = ch
            .iter()
            .filter(|c| matches!(c, Chunk::Change { .. }))
            .count();
        let (a1, a2) = (self.act.clone(), self.act.clone());
        card(t)
            .max_w(px(680.))
            .child(
                div()
                    .text_size(px(20.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            )
            .child(chunks_row(t, &ch, &self.decisions))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .p(px(6.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .bg(c(t, ColorRole::SurfaceOverlay))
                    .child(muted(t, format!("{n} changes")))
                    .child(div().flex_1())
                    .child(button(
                        t,
                        "ie-rej",
                        "Reject all",
                        Weight::Quiet,
                        "reject-all",
                        a1,
                    ))
                    .child(button(
                        t,
                        "ie-acc",
                        "Accept all",
                        Weight::Primary,
                        "accept-all",
                        a2,
                    )),
            )
    }
}

/// `image-compare`: two elements under a divider at `position` percent.
#[derive(IntoElement)]
pub struct ImageCompareEl {
    before: AnyElement,
    after: AnyElement,
    position: f32,
    width: f32,
    height: f32,
    theme: &'static Theme,
}

impl ImageCompareEl {
    /// Before and after, `width` × `height` px.
    pub fn new(before: impl IntoElement, after: impl IntoElement, width: f32, height: f32) -> Self {
        Self {
            before: before.into_any_element(),
            after: after.into_any_element(),
            position: 50.,
            width,
            height,
            theme: &Theme::DARK,
        }
    }
    /// Divider position, percent.
    pub fn position(mut self, p: f32) -> Self {
        self.position = p.clamp(0., 100.);
        self
    }
}

impl RenderOnce for ImageCompareEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let p = self.position / 100.;
        div()
            .relative()
            .w(px(self.width))
            .h(px(self.height))
            .rounded(px(12.))
            .overflow_hidden()
            .bg(c(t, ColorRole::SurfaceSunken))
            .child(
                div()
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .size_full()
                    .child(self.after),
            )
            .child(
                div()
                    .absolute()
                    .top(px(0.))
                    .left(px(0.))
                    .h_full()
                    .w(relative(p))
                    .overflow_hidden()
                    .child(
                        div()
                            .w(px(self.width))
                            .h(px(self.height))
                            .child(self.before),
                    ),
            )
            .child(
                div()
                    .absolute()
                    .top(px(0.))
                    .left(relative(p))
                    .h_full()
                    .w(px(2.))
                    .bg(gpui::white()),
            )
            .child(
                div()
                    .absolute()
                    .top(px(self.height / 2. - 14.))
                    .left(px(self.width * p - 14.))
                    .size(px(28.))
                    .rounded_full()
                    .bg(gpui::white()),
            )
    }
}

/// `comment-thread`: a thread with replies, reactions and a resolved state.
#[derive(IntoElement)]
pub struct CommentThreadEl {
    thread: Thread,
    title: SharedString,
    act: Option<Act>,
    theme: &'static Theme,
}

impl CommentThreadEl {
    /// A thread.
    pub fn new(thread: Thread) -> Self {
        Self {
            thread,
            title: "Comments".into(),
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Header title.
    pub fn title(mut self, s: impl Into<SharedString>) -> Self {
        self.title = s.into();
        self
    }
    /// Reports `resolve`.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

fn comment_el(
    t: &'static Theme,
    cm: &Comment,
    people: &[oxyn_core::parity::diff::Person],
    depth: usize,
) -> gpui::Div {
    let mut body = div().flex().flex_wrap().text_size(px(13.5));
    for (s, m) in mention_spans(&cm.body, people) {
        body = body.child(
            div()
                .when(m, |d| {
                    d.font_weight(FontWeight::SEMIBOLD)
                        .bg(c(t, ColorRole::SurfaceSelected))
                        .rounded(px(4.))
                })
                .child(s.replace(' ', "\u{a0}")),
        );
    }
    let mut reacts = div().flex().gap(px(4.));
    for (e, users) in &cm.reactions {
        reacts = reacts.child(tag(t, format!("{e} {}", users.len()), None));
    }
    let mut col = div()
        .flex()
        .flex_col()
        .gap(px(3.))
        .when(depth > 0, |d| {
            d.ml(px(30.))
                .pl(px(12.))
                .border_l_1()
                .border_color(c(t, ColorRole::BorderSubtle))
        })
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(
                    div()
                        .size(px(22.))
                        .rounded_full()
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(9.5))
                        .bg(if cm.author.agent {
                            c(t, ColorRole::FgDefault)
                        } else {
                            c(t, ColorRole::SurfaceSunken)
                        })
                        .text_color(if cm.author.agent {
                            c(t, ColorRole::BgBase)
                        } else {
                            c(t, ColorRole::FgDefault)
                        })
                        .child(cm.author.initials()),
                )
                .child(
                    div()
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(if cm.deleted {
                            "Deleted comment".to_string()
                        } else {
                            cm.author.name.clone()
                        }),
                )
                .child(muted(t, cm.at.clone())),
        )
        .when(!cm.deleted, |d| d.child(body).child(reacts));
    for r in &cm.replies {
        col = col.child(comment_el(t, r, people, depth + 1));
    }
    col
}

impl RenderOnce for CommentThreadEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let people = self.thread.people();
        let n = self.thread.count();
        let act = self.act.clone();
        let mut root = card(t).max_w(px(560.)).child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(self.title.clone()),
                )
                .child(muted(t, format!("{n} comments")))
                .child(div().flex_1())
                .child(button(
                    t,
                    "cmt-res",
                    if self.thread.resolved {
                        "Reopen"
                    } else {
                        "Resolve"
                    },
                    Weight::Plain,
                    "resolve",
                    act,
                )),
        );
        if self.thread.resolved {
            root = root.child(tag(
                t,
                format!("Resolved · {n} comments"),
                Some(ColorRole::FgMuted),
            ));
        } else {
            for cm in &self.thread.comments {
                root = root.child(comment_el(t, cm, &people, 0));
            }
        }
        root
    }
}

/// `review-center`: the queue on the left, the current item with Approve / Edit / Reject / Skip.
#[derive(IntoElement)]
pub struct ReviewCenterEl {
    center: Center,
    now: ClockMs,
    sla: u64,
    on_item: Option<IdAct>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ReviewCenterEl {
    /// Over a queue model.
    pub fn new(center: Center, now: ClockMs) -> Self {
        Self {
            center,
            now,
            sla: 1_800_000,
            on_item: None,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Service level, ms.
    pub fn sla(mut self, ms: u64) -> Self {
        self.sla = ms;
        self
    }
    /// Reports `(item id, "select")`.
    pub fn on_item(mut self, f: IdAct) -> Self {
        self.on_item = Some(f);
        self
    }
    /// Reports `approve`, `edit`, `reject`, `skip`, `undo`.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for ReviewCenterEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let cv = &self.center;
        let (done, total) = cv.progress();
        let mut queue = div()
            .flex()
            .flex_col()
            .gap(px(2.))
            .w(px(250.))
            .p(px(6.))
            .bg(c(t, ColorRole::SurfaceSunken));
        for it in cv.queue() {
            let on = cv.current.as_deref() == Some(it.id.as_str());
            let tone = match cv.sla(it, self.now, self.sla) {
                Sla::Ok => ColorRole::FgMuted,
                Sla::Warn => ColorRole::StatusWarningFg,
                Sla::Late => ColorRole::StatusDangerFg,
            };
            let cb = self.on_item.clone();
            let id = it.id.clone();
            queue = queue.child(
                div()
                    .id(SharedString::from(format!("q-{}", it.id)))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(10.))
                    .py(px(7.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .when(on, |d| d.bg(c(t, ColorRole::SurfaceRaised)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .min_w(px(0.))
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(it.title.clone()),
                            )
                            .child(muted(t, it.kind.clone())),
                    )
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(c(t, tone))
                            .child(waited_label(it.created, self.now)),
                    )
                    .when_some(cb, move |d, f| {
                        d.on_click(move |_, w, cx| f(id.clone(), "select", w, cx))
                    }),
            );
        }
        let mut item = div().flex().flex_col().gap(px(10.)).flex_1().p(px(16.));
        if let Some(it) = cv.current_item() {
            let risk_role = match it.risk {
                oxyn_core::parity::diff::Risk::High => ColorRole::StatusDangerFg,
                oxyn_core::parity::diff::Risk::Medium => ColorRole::StatusWarningFg,
                oxyn_core::parity::diff::Risk::Low => ColorRole::FgMuted,
            };
            item = item
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_size(px(12.5))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(it.agent.clone()),
                        )
                        .child(tag(t, it.risk.label(), Some(risk_role)))
                        .child(muted(
                            t,
                            format!("{}% sure", (it.confidence * 100.).round()),
                        )),
                )
                .child(
                    div()
                        .text_size(px(16.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(it.title.clone()),
                )
                .when_some(it.reasoning.clone(), |d, r| d.child(muted(t, r)));
            for (l, v) in &it.fields {
                item = item.child(
                    div()
                        .flex()
                        .gap(px(14.))
                        .child(
                            div()
                                .w(px(80.))
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(l.clone()),
                        )
                        .child(oxyn_core::parity::diff::spans_text(v)),
                );
            }
            if !it.body.is_empty() {
                item = item.child(
                    div()
                        .p(px(10.))
                        .rounded(px(9.))
                        .bg(c(t, ColorRole::SurfaceBase))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .child(oxyn_core::parity::diff::spans_text(&it.body)),
                );
            }
            let mut acts = div().flex().gap(px(6.));
            for (id, label, w) in [
                ("approve", "Approve  A", Weight::Primary),
                ("edit", "Edit  E", Weight::Plain),
                ("reject", "Reject  R", Weight::Danger),
                ("skip", "Skip  S", Weight::Quiet),
            ] {
                acts = acts.child(button(t, id, label, w, id, self.act.clone()));
            }
            item = item.child(acts);
        } else {
            item = item.child(muted(t, "All caught up"));
        }
        card(t)
            .p(px(0.))
            .gap(px(0.))
            .overflow_hidden()
            .w(px(900.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .px(px(14.))
                    .h(px(44.))
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Review center"),
                    )
                    .child(muted(t, format!("{done} of {total} reviewed"))),
            )
            .child(div().flex().h(px(460.)).child(queue).child(item))
    }
}

/// `review-card`: an agent's code review with a verdict and findings.
#[derive(IntoElement)]
pub struct ReviewCardEl {
    findings: Vec<Finding>,
    reviewer: SharedString,
    on_finding: Option<IdAct>,
    theme: &'static Theme,
}

impl ReviewCardEl {
    /// Over findings.
    pub fn new(findings: Vec<Finding>) -> Self {
        Self {
            findings,
            reviewer: "Review agent".into(),
            on_finding: None,
            theme: &Theme::DARK,
        }
    }
    /// Reports `(finding id, "apply" | "dismiss" | "reopen")`.
    pub fn on_finding(mut self, f: IdAct) -> Self {
        self.on_finding = Some(f);
        self
    }
}

impl RenderOnce for ReviewCardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let v = review_verdict(&self.findings);
        let vrole = match v.id() {
            "approve" => ColorRole::StatusSuccessFg,
            "comment" => ColorRole::StatusWarningFg,
            _ => ColorRole::StatusDangerFg,
        };
        let mut counts = div().flex().gap(px(6.));
        for (s, n) in severity_counts(&self.findings) {
            counts = counts.child(tag(t, format!("{n} {}", s.label()), None));
        }
        let mut root = card(t)
            .max_w(px(600.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.reviewer.clone()),
                    )
                    .child(div().flex_1())
                    .child(tag(t, v.label(), Some(vrole))),
            )
            .child(counts);
        for f in &self.findings {
            let cb = self.on_finding.clone();
            let id = f.id.clone();
            let action: &'static str = if f.state == FindingState::Open {
                if f.fix.is_some() { "apply" } else { "dismiss" }
            } else {
                "reopen"
            };
            let label = match action {
                "apply" => "Apply fix",
                "dismiss" => "Dismiss",
                _ => "Reopen",
            };
            root = root.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .py(px(6.))
                    .border_t_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .when(f.state != FindingState::Open, |d| d.opacity(0.65))
                    .child(tag(t, f.severity.label(), None))
                    .child(div().flex_1().text_size(px(13.)).child(f.title.clone()))
                    .child(
                        mono()
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(format!("{}:{}", f.file, f.line)),
                    )
                    .child(
                        div()
                            .id(SharedString::from(format!("f-{}", f.id)))
                            .text_size(px(12.))
                            .text_color(c(t, ColorRole::FgMuted))
                            .cursor_pointer()
                            .child(label)
                            .when_some(cb, move |d, g| {
                                d.on_click(move |_, w, cx| g(id.clone(), action, w, cx))
                            }),
                    ),
            );
        }
        root
    }
}
