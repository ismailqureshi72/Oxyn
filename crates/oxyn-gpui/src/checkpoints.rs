//! The Checkpoints family in GPUI as one live view: lanes, side list, inline marker, restore
//! toast, branch tree, film strip, undo ladder and restore preview sheet. Every variant runs on
//! the `oxyn_core::checkpoint` models the Dioxus components use (`CheckpointRun`, `branch_tree`,
//! `FilmStrip`, `UndoLadder`, `RestorePreview`), so keys and presses behave the same in both
//! renderers. Keys: ↑ / ↓ / ← / → / Home / End move, Enter or Space acts, Esc cancels,
//! ⌘Z / Ctrl+Z undo and ⇧⌘Z / Ctrl+Y redo on the ladder.

use gpui::{
    App, ClickEvent, Context, Div, FocusHandle, Focusable, FontWeight, Hsla, IntoElement,
    KeyDownEvent, ParentElement, Render, Styled, Window, div, prelude::*, px,
};
use oxyn_core::checkpoint::{
    Checkpoint, CheckpointRun, CheckpointStatus, CpKey, FileChange, FilmStrip, PreviewFile,
    PreviewPhase, RestoreAction, RestorePreview, UndoLadder, UndoStep, branch_tree, format_at,
    iso_at, roving, thumb_bars,
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

/// Which member of the family the view draws.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpVariant {
    /// Attempts as lanes.
    Lanes,
    /// Side list grouped by ISO date.
    List,
    /// Inline marker with its restore menu.
    Marker,
    /// Restore toast with Undo.
    Toast,
    /// Forks as an indented tree.
    Tree,
    /// Thumbnails on a scrubbable track.
    FilmStrip,
    /// Vertical stepper of reversible steps.
    UndoLadder,
    /// Diff summary with Restore, confirm and Cancel.
    PreviewSheet,
}

impl CpVariant {
    /// Every variant, in gallery order.
    pub const ALL: [CpVariant; 8] = [
        Self::Lanes,
        Self::List,
        Self::Marker,
        Self::Toast,
        Self::Tree,
        Self::FilmStrip,
        Self::UndoLadder,
        Self::PreviewSheet,
    ];
}

/// Sample run day and start for absolute ISO times.
const DAY: &str = "2026-11-14";
const START: (u32, u32) = (15, 30);

fn sample() -> Vec<Checkpoint> {
    vec![
        Checkpoint::new("c1", None, "Read the brief", 9),
        Checkpoint::new("c2", Some("c1"), "Built the pricing table", 64).files(3, 120, 4),
        Checkpoint::new("c3", Some("c2"), "Added the Stripe checkout", 98).files(2, 61, 9),
        Checkpoint::new("c4", Some("c3"), "Tests failing – 4 of 12", 131)
            .with(CheckpointStatus::Failed)
            .files(1, 12, 3),
        Checkpoint::new("c5", Some("c2"), "Switched to Stripe Elements", 169).files(4, 88, 41),
        Checkpoint::new("c6", Some("c5"), "Wiring the webhooks", 212)
            .with(CheckpointStatus::Running),
    ]
}

/// The family's state, shared by every variant; pure, so keys and presses are testable without
/// a window.
#[derive(Clone, Debug)]
pub struct CpState {
    /// The run (lanes, tree, list).
    pub run: CheckpointRun,
    /// Folded fork ids (tree).
    pub folded: Vec<String>,
    /// Focused visible tree row.
    pub tree_focus: usize,
    /// Selected list row (checkpoint index).
    pub list_sel: usize,
    /// Marker menu open, and its focused item.
    pub menu: Option<usize>,
    /// Last restore action chosen (marker, list).
    pub last: Option<RestoreAction>,
    /// Toast: restored and undo still offered.
    pub toast_undo: bool,
    /// Film strip.
    pub film: FilmStrip,
    /// Undo ladder.
    pub ladder: UndoLadder,
    /// Restore preview.
    pub preview: RestorePreview,
}

impl Default for CpState {
    fn default() -> Self {
        let cps = sample();
        let n = cps.len();
        Self {
            run: CheckpointRun::new(cps, None),
            folded: vec![],
            tree_focus: 0,
            list_sel: n - 1,
            menu: None,
            last: None,
            toast_undo: true,
            film: FilmStrip::new(n, n - 2),
            ladder: UndoLadder::new(vec![
                UndoStep::new("Added the pricing table", "3 files"),
                UndoStep::new("Renamed Plan to Tier", "6 files"),
                UndoStep::new("Deployed the preview build", "preview environment").fixed(),
                UndoStep::new("Moved prices to cents", "2 files"),
                UndoStep::new("Added the annual toggle", "2 files"),
            ]),
            preview: RestorePreview::new(
                vec![
                    pf("src/pricing/table.rs", FileChange::Modified, (14, 61)),
                    pf("src/checkout/stripe.rs", FileChange::Removed, (0, 88)),
                    pf("src/pricing/plan.rs", FileChange::Restored, (36, 0)),
                ],
                3,
                RestoreAction::Both,
            ),
        }
    }
}

fn pf(p: &str, change: FileChange, lines: (u32, u32)) -> PreviewFile {
    PreviewFile {
        path: p.into(),
        change,
        lines,
    }
}

impl CpState {
    /// Visible tree rows.
    fn tree_rows(&self) -> Vec<oxyn_core::checkpoint::TreeRow> {
        branch_tree(
            &self.run.checkpoints,
            self.run.head.as_deref(),
            &self.folded,
        )
        .into_iter()
        .filter(|r| !r.hidden)
        .collect()
    }

    /// One key for `v`.
    pub fn key(&mut self, v: CpVariant, k: CpKey) {
        let n = self.run.checkpoints.len();
        match v {
            CpVariant::Lanes => {
                self.run.key(k);
            }
            CpVariant::List => match k {
                CpKey::Enter => self.last = Some(RestoreAction::Both),
                _ => {
                    // Drawn newest first.
                    let row = roving(n, n - 1 - self.list_sel.min(n - 1), k);
                    self.list_sel = n - 1 - row;
                }
            },
            CpVariant::Marker => match (self.menu, k) {
                (None, CpKey::Enter) => self.menu = Some(0),
                (Some(_), CpKey::Escape) => self.menu = None,
                (Some(i), CpKey::Enter) => {
                    self.last = Some(RestoreAction::ALL[i].0);
                    self.menu = None;
                }
                (Some(i), _) => self.menu = Some(roving(4, i, k)),
                _ => {}
            },
            CpVariant::Toast => {
                if k == CpKey::Enter {
                    self.toast_undo = !self.toast_undo;
                }
            }
            CpVariant::Tree => {
                let rows = self.tree_rows();
                let f = self.tree_focus.min(rows.len().saturating_sub(1));
                let Some(r) = rows.get(f) else { return };
                let id = self.run.checkpoints[r.index].id.clone();
                let forks = self
                    .run
                    .checkpoints
                    .iter()
                    .filter(|c| c.parent.as_deref() == Some(id.as_str()))
                    .count()
                    > 1;
                match k {
                    CpKey::Enter => {
                        self.run.rewind(r.index);
                    }
                    CpKey::Left if forks => {
                        if !self.folded.contains(&id) {
                            self.folded.push(id)
                        }
                    }
                    CpKey::Right if forks => self.folded.retain(|x| *x != id),
                    CpKey::Left | CpKey::Right => {}
                    _ => self.tree_focus = roving(rows.len(), f, k),
                }
            }
            CpVariant::FilmStrip => {
                if k == CpKey::Enter {
                    self.last = Some(RestoreAction::Both);
                } else {
                    self.film.key(k);
                }
            }
            CpVariant::UndoLadder => self.ladder.key(k),
            CpVariant::PreviewSheet => {
                if matches!(
                    self.preview.phase,
                    PreviewPhase::Done | PreviewPhase::Cancelled
                ) && k == CpKey::Enter
                {
                    self.preview.phase = PreviewPhase::Review;
                } else {
                    self.preview.key(k);
                }
            }
        }
    }

    /// A press on item `i` of `v` (row, frame, rung, menu item; for the sheet 0 = primary,
    /// 1 = cancel).
    pub fn press(&mut self, v: CpVariant, i: usize) {
        match v {
            CpVariant::Lanes => {
                self.run.focus = i;
                self.run.rewind(i);
            }
            CpVariant::List => self.list_sel = i,
            CpVariant::Marker => {
                if let Some(&(a, _)) = RestoreAction::ALL.get(i) {
                    self.last = Some(a);
                }
                self.menu = None;
            }
            CpVariant::Toast => self.toast_undo = false,
            CpVariant::Tree => {
                self.tree_focus = i;
                if let Some(r) = self.tree_rows().get(i) {
                    self.run.rewind(r.index);
                }
            }
            CpVariant::FilmStrip => self.film.at = i.min(self.film.len.saturating_sub(1)),
            CpVariant::UndoLadder => {
                self.ladder.jump(i);
            }
            CpVariant::PreviewSheet => match (i, self.preview.phase) {
                (_, PreviewPhase::Done | PreviewPhase::Cancelled) => {
                    self.preview.phase = PreviewPhase::Review
                }
                (0, _) => self.preview.press(),
                _ => self.preview.cancel(),
            },
        }
    }
}

/// The Checkpoints family as a live, focusable GPUI view (see the module docs).
pub struct CheckpointsView {
    variant: CpVariant,
    state: CpState,
    dark: bool,
    focus: FocusHandle,
}

impl Focusable for CheckpointsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl CheckpointsView {
    /// A view of `variant` on the sample run, focused so the keys work at once.
    pub fn new(variant: CpVariant, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            variant,
            state: CpState::default(),
            dark: true,
            focus,
        }
    }

    /// Switch variant.
    pub fn set_variant(&mut self, v: CpVariant, cx: &mut Context<Self>) {
        self.variant = v;
        cx.notify();
    }

    /// Light or dark.
    pub fn set_dark(&mut self, dark: bool, cx: &mut Context<Self>) {
        self.dark = dark;
        cx.notify();
    }

    /// The state (for hosts and tests).
    pub fn state(&self) -> &CpState {
        &self.state
    }

    fn key(&mut self, ev: &KeyDownEvent, cx: &mut Context<Self>) {
        let m = &ev.keystroke.modifiers;
        let name = ev.keystroke.key.as_str();
        let k = if m.platform || m.control {
            match name {
                "z" if m.shift => Some(CpKey::Redo),
                "z" => Some(CpKey::Undo),
                "y" => Some(CpKey::Redo),
                _ => None,
            }
        } else {
            CpKey::from_name(name)
        };
        if let Some(k) = k {
            self.state.key(self.variant, k);
            cx.notify();
        }
    }

    fn theme(&self) -> &'static Theme {
        if self.dark {
            &Theme::DARK
        } else {
            &Theme::LIGHT
        }
    }
}

fn mono(t: &Theme, s: String) -> Div {
    div()
        .text_size(px(11.5))
        .text_color(c(t, ColorRole::FgSubtle))
        .child(s)
}

fn badge(t: &Theme, s: &'static str, role: ColorRole) -> Div {
    div()
        .px(px(6.))
        .rounded(px(999.))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .text_size(px(10.5))
        .text_color(c(t, role))
        .child(s)
}

fn card(t: &Theme) -> Div {
    div()
        .flex()
        .flex_col()
        .rounded(px(12.))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .bg(c(t, ColorRole::SurfaceRaised))
        .overflow_hidden()
}

fn bar(t: &Theme, title: &'static str, right: String) -> Div {
    div()
        .flex()
        .items_center()
        .gap(px(8.))
        .px(px(12.))
        .py(px(10.))
        .border_b_1()
        .border_color(c(t, ColorRole::BorderSubtle))
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(c(t, ColorRole::FgDefault))
                .child(title),
        )
        .child(div().flex_1())
        .child(mono(t, right))
}

fn status_dot(t: &Theme, s: CheckpointStatus, on: bool) -> Div {
    let col = match s {
        CheckpointStatus::Failed => c(t, ColorRole::StatusDangerFg),
        _ if on => c(t, ColorRole::FgDefault),
        _ => c(t, ColorRole::FgSubtle),
    };
    div().size(px(8.)).rounded(px(4.)).bg(col)
}

impl Render for CheckpointsView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let t = self.theme();
        let v = self.variant;
        let s = &self.state;
        let fg = c(t, ColorRole::FgDefault);
        let hover = c(t, ColorRole::SurfaceHover);
        let iso = |cp: &Checkpoint| iso_at(DAY, START, cp.at);
        let body: Div = match v {
            CpVariant::Lanes => {
                let rows = s.run.rows();
                let mut list = div().flex().flex_col().p(px(6.));
                for r in rows {
                    let cp = &s.run.checkpoints[r.index];
                    let i = r.index;
                    list = list.child(
                        div()
                            .id(("cp-lane", i))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .h(px(36.))
                            .px(px(6.))
                            .rounded(px(8.))
                            .when(i == s.run.focus, |d| d.bg(hover))
                            .when(!r.on_path, |d| d.opacity(0.5))
                            .child(div().w(px(18. * r.lane as f32 + 2.)))
                            .child(status_dot(t, cp.status, r.on_path))
                            .child(div().text_color(fg).child(cp.title.clone()))
                            .when(cp.status == CheckpointStatus::Failed, |d| {
                                d.child(badge(t, "Failed", ColorRole::StatusDangerFg))
                            })
                            .when(cp.status == CheckpointStatus::Stopped, |d| {
                                d.child(badge(t, "Stopped", ColorRole::FgMuted))
                            })
                            .when(r.current, |d| {
                                d.child(badge(t, "Head", ColorRole::FgDefault))
                            })
                            .child(div().flex_1())
                            .child(mono(t, iso(cp)))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.state.press(CpVariant::Lanes, i);
                                cx.notify();
                            })),
                    );
                }
                card(t)
                    .child(bar(
                        t,
                        "Checkpoints",
                        oxyn_core::checkpoint::header(&s.run.checkpoints),
                    ))
                    .child(list)
            }
            CpVariant::List => {
                let n = s.run.checkpoints.len();
                let mut list = div().flex().flex_col().p(px(6.)).child(
                    div()
                        .px(px(8.))
                        .py(px(6.))
                        .text_size(px(11.))
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child(DAY),
                );
                for i in (0..n).rev() {
                    let cp = &s.run.checkpoints[i];
                    list = list.child(
                        div()
                            .id(("cp-list", i))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .px(px(8.))
                            .py(px(6.))
                            .rounded(px(8.))
                            .when(i == s.list_sel, |d| d.bg(hover))
                            .child(
                                div()
                                    .text_size(px(9.))
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child("◆"),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .flex_1()
                                    .child(div().text_color(fg).child(cp.title.clone()))
                                    .child(mono(t, iso(cp))),
                            )
                            .when(cp.files > 0, |d| {
                                d.child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(c(t, ColorRole::DiffAddFg))
                                        .child(format!("+{}", cp.lines.0)),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(c(t, ColorRole::DiffDelFg))
                                        .child(format!("\u{2212}{}", cp.lines.1)),
                                )
                            })
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.state.press(CpVariant::List, i);
                                cx.notify();
                            })),
                    );
                }
                card(t).child(list)
            }
            CpVariant::Marker => {
                let line = || div().flex_1().h(px(1.)).bg(c(t, ColorRole::BorderSubtle));
                let mut d = div().flex().flex_col().gap(px(6.)).child(
                    div()
                        .id("cp-marker")
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .h(px(28.))
                        .text_size(px(12.))
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child(line())
                        .child("◆")
                        .child(format!("Checkpoint · {DAY} 15:42 · 3 files"))
                        .child(line())
                        .child(
                            div()
                                .px(px(8.))
                                .py(px(2.))
                                .rounded(px(6.))
                                .border_1()
                                .border_color(c(t, ColorRole::BorderDefault))
                                .text_color(fg)
                                .child("Restore ▾"),
                        )
                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                            this.state.menu = if this.state.menu.is_some() {
                                None
                            } else {
                                Some(0)
                            };
                            cx.notify();
                        })),
                );
                if let Some(f) = s.menu {
                    let mut m = div()
                        .flex()
                        .flex_col()
                        .p(px(4.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .bg(c(t, ColorRole::SurfaceOverlay));
                    for (i, (a, key)) in RestoreAction::ALL.iter().enumerate() {
                        m = m.child(
                            div()
                                .id(("cp-menu", i))
                                .flex()
                                .items_center()
                                .h(px(30.))
                                .px(px(8.))
                                .rounded(px(6.))
                                .text_color(fg)
                                .when(i == f, |d| d.bg(hover))
                                .child(a.menu())
                                .child(div().flex_1())
                                .child(mono(t, key.to_string()))
                                .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                    this.state.press(CpVariant::Marker, i);
                                    cx.notify();
                                })),
                        );
                    }
                    d = d.child(m);
                }
                if let Some(a) = s.last {
                    d = d.child(mono(t, format!("Chose: {}", a.menu())));
                }
                d
            }
            CpVariant::Toast => div().child(
                div()
                    .id("cp-toast")
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .h(px(38.))
                    .px(px(12.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .bg(c(t, ColorRole::SurfaceOverlay))
                    .text_color(fg)
                    .child(if s.toast_undo {
                        format!("Restored 3 files to {DAY} 15:42")
                    } else {
                        "Restore undone".to_string()
                    })
                    .when(s.toast_undo, |d| {
                        d.child(div().font_weight(FontWeight::SEMIBOLD).child("Undo"))
                    })
                    .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                        this.state.press(CpVariant::Toast, 0);
                        cx.notify();
                    })),
            ),
            CpVariant::Tree => {
                let rows = s.tree_rows();
                let mut list = div().flex().flex_col().p(px(6.));
                for (ri, r) in rows.iter().enumerate() {
                    let cp = &s.run.checkpoints[r.index];
                    list = list.child(
                        div()
                            .id(("cp-tree", ri))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .h(px(34.))
                            .px(px(6.))
                            .rounded(px(8.))
                            .when(ri == s.tree_focus, |d| d.bg(hover))
                            .when(!r.on_path, |d| d.opacity(0.5))
                            .child(div().w(px(16. * r.depth as f32)))
                            .when(r.fork, |d| {
                                d.child(
                                    div()
                                        .text_color(c(t, ColorRole::BorderStrong))
                                        .child(if r.last { "└" } else { "├" }),
                                )
                            })
                            .child(status_dot(t, cp.status, r.on_path))
                            .child(div().text_color(fg).child(cp.title.clone()))
                            .when(r.fork, |d| d.child(badge(t, "Fork", ColorRole::FgMuted)))
                            .when(r.current, |d| {
                                d.child(badge(t, "Head", ColorRole::FgDefault))
                            })
                            .child(div().flex_1())
                            .child(mono(t, iso(cp)))
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.state.press(CpVariant::Tree, ri);
                                cx.notify();
                            })),
                    );
                }
                card(t)
                    .child(bar(
                        t,
                        "Branches",
                        format!("{} checkpoints", s.run.checkpoints.len()),
                    ))
                    .child(list)
            }
            CpVariant::FilmStrip => {
                let mut frames = div().flex().gap(px(8.)).p(px(10.));
                for (i, cp) in s.run.checkpoints.iter().enumerate() {
                    let mut th = div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .p(px(7.))
                        .h(px(66.))
                        .rounded(px(7.))
                        .bg(c(t, ColorRole::SurfaceSunken));
                    for (w, hot) in thumb_bars(cp, 6) {
                        th = th.child(div().h(px(4.)).w(px(w as f32 * 0.98)).rounded(px(2.)).bg(
                            if hot {
                                c(t, ColorRole::DiffAddFg)
                            } else {
                                c(t, ColorRole::BorderStrong)
                            },
                        ));
                    }
                    frames = frames.child(
                        div()
                            .id(("cp-frame", i))
                            .w(px(124.))
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .p(px(6.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(if i == s.film.at {
                                fg
                            } else {
                                c(t, ColorRole::SurfaceRaised)
                            })
                            .child(th)
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child(format!("{} {}", i + 1, cp.title)),
                            )
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.state.press(CpVariant::FilmStrip, i);
                                cx.notify();
                            })),
                    );
                }
                let ph = s.film.playhead();
                let cur = &s.run.checkpoints[s.film.at.min(s.run.checkpoints.len() - 1)];
                card(t)
                    .child(frames)
                    .child(
                        div()
                            .mx(px(16.))
                            .h(px(2.))
                            .rounded(px(999.))
                            .bg(c(t, ColorRole::BorderDefault))
                            .child(
                                div()
                                    .h(px(2.))
                                    .w(gpui::relative(ph))
                                    .rounded(px(999.))
                                    .bg(fg),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .p(px(12.))
                            .child(mono(t, format!("{} of {}", s.film.at + 1, s.film.len)))
                            .child(div().text_color(fg).child(cur.title.clone()))
                            .child(div().flex_1())
                            .child(mono(t, iso(cur))),
                    )
            }
            CpVariant::UndoLadder => {
                let l = &s.ladder;
                let mut list = div().flex().flex_col().p(px(10.)).gap(px(4.));
                for (i, st) in l.steps.iter().enumerate() {
                    let state = l.state(i);
                    list = list.child(
                        div()
                            .id(("cp-rung", i))
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .min_h(px(40.))
                            .px(px(6.))
                            .rounded(px(8.))
                            .when(i == l.focus, |d| d.bg(hover))
                            .child(
                                div()
                                    .size(px(22.))
                                    .rounded(px(11.))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .text_size(px(11.))
                                    .when(state == "undone", |d| {
                                        d.border_1()
                                            .border_color(c(t, ColorRole::BorderStrong))
                                            .text_color(c(t, ColorRole::FgSubtle))
                                    })
                                    .when(state != "undone", |d| {
                                        d.bg(fg).text_color(c(t, ColorRole::SurfaceRaised))
                                    })
                                    .child(format!("{}", i + 1)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_color(if state == "undone" {
                                                c(t, ColorRole::FgSubtle)
                                            } else {
                                                fg
                                            })
                                            .child(st.title.clone()),
                                    )
                                    .child(mono(t, st.detail.clone())),
                            )
                            .child(div().flex_1())
                            .when(!st.reversible, |d| {
                                d.child(badge(t, "Can't undo", ColorRole::StatusWarningFg))
                            })
                            .when(state == "undone", |d| {
                                d.child(badge(t, "Undone", ColorRole::FgMuted))
                            })
                            .on_click(cx.listener(move |this, _: &ClickEvent, _, cx| {
                                this.state.press(CpVariant::UndoLadder, i);
                                cx.notify();
                            })),
                    );
                }
                card(t).child(bar(t, "Steps", l.summary())).child(list)
            }
            CpVariant::PreviewSheet => {
                let p = &s.preview;
                if matches!(p.phase, PreviewPhase::Done | PreviewPhase::Cancelled) {
                    div().child(
                        div()
                            .id("cp-sheet-again")
                            .flex()
                            .gap(px(10.))
                            .p(px(12.))
                            .rounded(px(12.))
                            .border_1()
                            .border_color(c(t, ColorRole::BorderDefault))
                            .text_color(fg)
                            .child(if p.phase == PreviewPhase::Done {
                                format!("Restored {} to {DAY} 15:31:04", p.summary())
                            } else {
                                "Nothing changed".to_string()
                            })
                            .child(div().flex_1())
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("Preview again"),
                            )
                            .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                this.state.press(CpVariant::PreviewSheet, 0);
                                cx.notify();
                            })),
                    )
                } else {
                    let mut files = div()
                        .flex()
                        .flex_col()
                        .p(px(4.))
                        .rounded(px(10.))
                        .bg(c(t, ColorRole::SurfaceSunken));
                    for f in &p.files {
                        let role = match f.change {
                            FileChange::Modified => ColorRole::StatusWarningFg,
                            FileChange::Restored => ColorRole::DiffAddFg,
                            FileChange::Removed => ColorRole::DiffDelFg,
                        };
                        files = files.child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(8.))
                                .h(px(30.))
                                .px(px(6.))
                                .child(div().text_color(c(t, role)).child(f.change.tag()))
                                .child(div().text_color(fg).child(f.path.clone()))
                                .child(div().flex_1())
                                .child(mono(t, format!("+{} \u{2212}{}", f.lines.0, f.lines.1))),
                        );
                    }
                    let confirm = p.phase == PreviewPhase::Confirm;
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        .p(px(16.))
                        .rounded(px(14.))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .bg(c(t, ColorRole::SurfaceOverlay))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(fg)
                                .child("Restore “Built the pricing table”"),
                        )
                        .child(mono(t, format!("{DAY} 15:31:04 · {}", p.summary())))
                        .child(files)
                        .when(confirm, |d| {
                            d.child(div().text_size(px(12.)).text_color(fg).child(
                                "Your current changes are kept as a checkpoint, so you can come back to them.",
                            ))
                        })
                        .child(
                            div()
                                .flex()
                                .gap(px(8.))
                                .child(div().flex_1())
                                .child(
                                    div()
                                        .id("cp-sheet-cancel")
                                        .px(px(12.))
                                        .py(px(6.))
                                        .rounded(px(8.))
                                        .border_1()
                                        .border_color(c(t, ColorRole::BorderDefault))
                                        .text_color(fg)
                                        .child(if confirm { "Back" } else { "Cancel" })
                                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                            this.state.press(CpVariant::PreviewSheet, 1);
                                            cx.notify();
                                        })),
                                )
                                .child(
                                    div()
                                        .id("cp-sheet-primary")
                                        .px(px(12.))
                                        .py(px(6.))
                                        .rounded(px(8.))
                                        .bg(if confirm {
                                            c(t, ColorRole::StatusDangerFg)
                                        } else {
                                            fg
                                        })
                                        .text_color(c(t, ColorRole::SurfaceRaised))
                                        .child(p.primary())
                                        .on_click(cx.listener(|this, _: &ClickEvent, _, cx| {
                                            this.state.press(CpVariant::PreviewSheet, 0);
                                            cx.notify();
                                        })),
                                ),
                        )
                }
            }
        };
        div()
            .id("oxyn-checkpoints")
            .key_context("OxynCheckpoints")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, _, cx| this.key(ev, cx)))
            .p(px(16.))
            .size_full()
            .bg(c(t, ColorRole::BgBase))
            .text_size(px(13.))
            .child(body)
            .child(mono(
                t,
                format!(
                    "Run time {}",
                    format_at(s.run.checkpoints.last().map(|c| c.at).unwrap_or_default())
                ),
            ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn c1a_keys_and_presses() {
        let mut s = CpState::default();
        // Lanes: Home, Down, Enter rewinds to c2 and starts attempt three.
        s.key(CpVariant::Lanes, CpKey::Home);
        s.key(CpVariant::Lanes, CpKey::Down);
        s.key(CpVariant::Lanes, CpKey::Enter);
        assert_eq!(s.run.head_id(), "r1");
        assert_eq!(s.run.checkpoints[5].status, CheckpointStatus::Stopped);
        // Tree: fold the forks of c2, then rewind from the root.
        let mut s = CpState::default();
        s.key(CpVariant::Tree, CpKey::Down);
        s.key(CpVariant::Tree, CpKey::Left);
        assert_eq!(s.folded, vec!["c2".to_string()]);
        assert_eq!(s.tree_rows().len(), 2);
        s.key(CpVariant::Tree, CpKey::Right);
        assert!(s.folded.is_empty());
        // Marker menu: open, move, choose.
        s.key(CpVariant::Marker, CpKey::Enter);
        s.key(CpVariant::Marker, CpKey::Down);
        s.key(CpVariant::Marker, CpKey::Enter);
        assert_eq!((s.menu, s.last), (None, Some(RestoreAction::Chat)));
        // Film strip and ladder.
        s.key(CpVariant::FilmStrip, CpKey::Home);
        assert_eq!(s.film.at, 0);
        s.press(CpVariant::FilmStrip, 3);
        assert_eq!(s.film.at, 3);
        s.key(CpVariant::UndoLadder, CpKey::Undo);
        s.key(CpVariant::UndoLadder, CpKey::Undo);
        s.key(CpVariant::UndoLadder, CpKey::Undo);
        assert_eq!(s.ladder.applied, 3);
        s.key(CpVariant::UndoLadder, CpKey::Redo);
        assert_eq!(s.ladder.applied, 4);
        // Sheet: Restore → confirm → done → preview again.
        s.press(CpVariant::PreviewSheet, 0);
        s.key(CpVariant::PreviewSheet, CpKey::Escape);
        assert_eq!(s.preview.phase, PreviewPhase::Review);
        s.press(CpVariant::PreviewSheet, 0);
        s.press(CpVariant::PreviewSheet, 0);
        assert_eq!(s.preview.phase, PreviewPhase::Done);
        s.key(CpVariant::PreviewSheet, CpKey::Enter);
        assert_eq!(s.preview.phase, PreviewPhase::Review);
        // List: drawn newest first, so Down moves to an older checkpoint.
        s.key(CpVariant::List, CpKey::Down);
        assert_eq!(s.list_sel, 4);
        assert_eq!(CpVariant::ALL.len(), 8);
    }
}
