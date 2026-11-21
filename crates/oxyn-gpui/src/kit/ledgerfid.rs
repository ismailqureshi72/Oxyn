//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32) for GPUI: the exact
//! **Change ledger** ([`ChangeLedgerEl`]) over `oxyn_core::kit::ledgerfid`. The element is
//! `RenderOnce`: the host keeps the entries, the undone ids and the pointed change, and routes
//! [`Act`] ids back (`undo-<n>` / `redo-<n>` for the n-th row from the top). A static snapshot:
//! Undo shows on the pointed row (GPUI has no hover state here), no strike animation.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::kit::ledgerfid::{
    LedgerEntry, ledger_ago, ledger_at, ledger_sorted, ledger_undo_plan,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c};
use crate::brand_mark;

const W: f32 = 640.0;
const UNDO_ACTS: [&str; 12] = [
    "undo-0", "undo-1", "undo-2", "undo-3", "undo-4", "undo-5", "undo-6", "undo-7", "undo-8",
    "undo-9", "undo-10", "undo-11",
];
const REDO_ACTS: [&str; 12] = [
    "redo-0", "redo-1", "redo-2", "redo-3", "redo-4", "redo-5", "redo-6", "redo-7", "redo-8",
    "redo-9", "redo-10", "redo-11",
];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    line: Hsla,
    div: Hsla,
    fg: Hsla,
    fg2: Hsla,
    mute: Hsla,
    faint: Hsla,
    hover: Hsla,
    info: Hsla,
    info_bg: Hsla,
    info_pill: Hsla,
    warn: Hsla,
    warn_bg: Hsla,
    warn_pill: Hsla,
    band: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x1C1C1E),
            line: h(0x353538),
            div: h(0x29292B),
            fg: h(0xFAFAFA),
            fg2: h(0xD4D4D8),
            mute: h(0x8F8F94),
            faint: h(0x67676C),
            hover: h(0x262628),
            info: h(0x4FB0FE),
            info_bg: h(0x1D2732),
            info_pill: h(0x22384B),
            warn: h(0xF5B454),
            warn_bg: h(0x2A241A), // warm-ok: semantic warning tint
            warn_pill: h(0x3E3018),
            band: h(0x232326),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            line: h(0xE9E9EC),
            div: h(0xF1F1F2),
            fg: h(0x18181B),
            fg2: h(0x3F3F46),
            mute: h(0x71717A),
            faint: h(0xA1A1AA),
            hover: h(0xF6F6F7),
            info: h(0x2563EB),
            info_bg: h(0xEEF4FF),
            info_pill: h(0xDCE8FD),
            warn: h(0xB45309),
            warn_bg: h(0xFFF6E9),
            warn_pill: h(0xFDEBD0),
            band: h(0xF4F4F5),
        }
    }
}

/// `agent-ledger` (affectUI Agent Ledger): the host passes the entries, undone ids and the
/// pointed change; acts `undo-<n>` and `redo-<n>` (row n from the top).
#[derive(IntoElement)]
pub struct ChangeLedgerEl {
    entries: Vec<LedgerEntry>,
    undone: Vec<String>,
    point: Option<String>,
    now: u64,
    title: String,
    agent: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ChangeLedgerEl {
    fn default() -> Self {
        let s = ledger_at(9_000);
        Self {
            entries: s.entries,
            undone: vec![],
            point: None,
            now: s.now,
            title: "Ledger".into(),
            agent: "Release agent".into(),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ChangeLedgerEl {
    /// The demo `ms` into its pass.
    pub fn demo(mut self, ms: u64) -> Self {
        let s = ledger_at(ms);
        self.entries = s.entries;
        self.undone = s.undone;
        self.point = s.point;
        self.now = s.now;
        self
    }
    /// Your own entries and undone ids, `now` on their clock.
    pub fn entries(mut self, e: Vec<LedgerEntry>, undone: Vec<String>, now: u64) -> Self {
        self.entries = e;
        self.undone = undone;
        self.now = now;
        self
    }
    /// The change whose Undo is pointed at.
    pub fn point(mut self, id: Option<String>) -> Self {
        self.point = id;
        self
    }
    /// Heading and who made the changes.
    pub fn title(mut self, t: &str, agent: &str) -> Self {
        self.title = t.into();
        self.agent = agent.into();
        self
    }
    /// Classic style.
    pub fn classic(mut self, on: bool) -> Self {
        self.classic = on;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Acts.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

fn fire(a: &Option<Act>, id: &'static str) -> impl Fn(&mut Window, &mut App) + 'static {
    let a = a.clone();
    move |w, cx| {
        if let Some(f) = &a {
            f(id, w, cx)
        }
    }
}

impl RenderOnce for ChangeLedgerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let gone = &self.undone;
        let (lit, kept) = match self.point.as_deref() {
            Some(x) if !gone.iter().any(|g| g == x) => ledger_undo_plan(&self.entries, x, gone),
            _ => (vec![], vec![]),
        };
        let rows = ledger_sorted(&self.entries);
        let n = rows.len();
        let n_undone = rows.iter().filter(|e| gone.contains(&e.id)).count();

        let cnt = |num: usize, word: &str| {
            div()
                .flex()
                .gap(px(3.))
                .text_size(px(12.))
                .child(
                    div()
                        .text_color(p.fg)
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(num.to_string()),
                )
                .child(div().text_color(p.mute).child(word.to_string()))
        };
        let mut bar = div()
            .h(px(if self.classic { 34. } else { 28. }))
            .when(!self.classic, |d| d.mb(px(12.)))
            .when(self.classic, |d| d.px(px(10.)))
            .flex()
            .items_center()
            .gap(px(8.))
            .child(div().text_color(p.info).child("≣"))
            .child(
                div()
                    .text_size(px(13.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .text_color(p.mute)
                    .child(self.agent.clone()),
            )
            .child(div().flex_1())
            .child(cnt(n, if n == 1 { "change" } else { "changes" }));
        if n_undone > 0 {
            bar = bar.child(cnt(n_undone, "undone"));
        }

        let mut list = div()
            .px(px(6.))
            .py(px(4.))
            .rounded(px(10.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .when(self.classic, |d| d.shadow_sm())
            .flex()
            .flex_col();
        for (i, e) in rows.iter().enumerate().take(UNDO_ACTS.len()) {
            let is_gone = gone.contains(&e.id);
            let is_self = lit.first() == Some(&e.id);
            let also = !is_self && lit.contains(&e.id);
            let is_kept = kept.contains(&e.id);
            let bg = if is_self || also {
                Some(p.info_bg)
            } else if is_kept {
                Some(p.warn_bg)
            } else {
                None
            };
            let mut tile = div()
                .size(px(32.))
                .flex_none()
                .rounded(px(if self.classic { 16. } else { 8. }))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .flex()
                .items_center()
                .justify_center()
                .when(is_gone, |d| d.opacity(0.55));
            let mark_col = if is_gone { p.faint } else { p.fg };
            tile = match brand_mark(&e.mark(), 17.0, mark_col) {
                Some(m) if !is_gone => tile.child(m),
                _ => tile.child(
                    div()
                        .text_color(mark_col)
                        .child(e.system.chars().next().unwrap_or('·').to_string()),
                ),
            };
            let l1 = div()
                .flex()
                .gap(px(6.))
                .text_size(px(13.))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(p.fg)
                        .when(is_gone, |d| d.line_through())
                        .child(e.action.clone()),
                )
                .child(
                    div()
                        .text_color(p.fg2)
                        .when(is_gone, |d| d.line_through())
                        .child(e.target.clone()),
                );
            let l2 = if e.has_change() {
                div()
                    .flex()
                    .gap(px(6.))
                    .text_size(px(12.))
                    .child(
                        div()
                            .text_color(p.faint)
                            .line_through()
                            .child(e.change.0.clone()),
                    )
                    .child(div().text_color(p.faint).child("→"))
                    .child(
                        div()
                            .text_color(p.fg2)
                            .when(is_gone, |d| d.line_through())
                            .child(e.change.1.clone()),
                    )
            } else {
                div()
                    .text_size(px(12.))
                    .text_color(p.mute)
                    .when(is_gone, |d| d.line_through())
                    .child(e.detail.clone())
            };
            let pill = |txt: &'static str, fg: Hsla, bg: Hsla| {
                div()
                    .h(px(20.))
                    .px(px(7.))
                    .rounded(px(6.))
                    .bg(bg)
                    .text_color(fg)
                    .text_size(px(11.5))
                    .font_weight(FontWeight::MEDIUM)
                    .flex()
                    .items_center()
                    .child(txt)
            };
            let btn = |txt: &'static str, id: &'static str, on: bool| {
                div()
                    .id(SharedString::from(format!("oxlg-{id}")))
                    .h(px(24.))
                    .px(px(8.))
                    .rounded(px(7.))
                    .bg(if on { p.info_pill } else { p.hover })
                    .text_color(if on { p.info } else { p.fg2 })
                    .text_size(px(12.5))
                    .flex()
                    .items_center()
                    .cursor_pointer()
                    .on_click({
                        let f = fire(&self.act, id);
                        move |_, w, cx| f(w, cx)
                    })
                    .child(txt)
            };
            let slot = if also {
                pill("Also undone", p.info, p.info_pill).into_any_element()
            } else if is_kept {
                pill("🔒 Can't undo", p.warn, p.warn_pill).into_any_element()
            } else if is_self {
                btn("↶ Undo", UNDO_ACTS[i], true).into_any_element()
            } else if is_gone {
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().text_color(p.mute).child("Undone"))
                    .child(btn("↷ Redo", REDO_ACTS[i], false))
                    .into_any_element()
            } else {
                div()
                    .text_color(p.mute)
                    .child(format!("{} · {}", ledger_ago(self.now, e.at), e.system))
                    .into_any_element()
            };
            list = list.child(
                div()
                    .h(px(56.))
                    .px(px(10.))
                    .rounded(px(8.))
                    .when_some(bg, |d, b| d.bg(b))
                    .when(i > 0 && bg.is_none(), |d| {
                        d.border_t_1().border_color(alpha(p.div, 1.0))
                    })
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(tile)
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .overflow_hidden()
                            .when(is_gone, |d| d.opacity(0.55))
                            .child(l1)
                            .child(l2),
                    )
                    .child(
                        div()
                            .flex_none()
                            .min_w(px(96.))
                            .flex()
                            .justify_end()
                            .text_size(px(12.))
                            .child(slot),
                    ),
            );
        }

        div()
            .w(px(W))
            .text_color(p.fg)
            .when(self.classic, |d| {
                d.p(px(6.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.band)
            })
            .flex()
            .flex_col()
            .child(bar)
            .child(list)
    }
}
