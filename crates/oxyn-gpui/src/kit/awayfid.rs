//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32) for GPUI: the exact
//! **Return brief** ([`ReturnBriefEl`]) over `oxyn_core::kit::awayfid`. The element is
//! `RenderOnce`: the host keeps which rows resolved, whether it is folded or showing every row,
//! and routes [`Act`] ids back (`act-<n>` for the n-th row, `more`, `got-it`, `open`). A static
//! snapshot: no breathing mark, no rolling counts.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::kit::awayfid::{
    AWAY_MINUTES, AwayItem, AwayKind, away_counts, away_demo, away_divider, away_duration,
    away_needs_word, away_order, away_resolve,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const W: f32 = 600.0;
const ACT: [&str; 10] = [
    "act-0", "act-1", "act-2", "act-3", "act-4", "act-5", "act-6", "act-7", "act-8", "act-9",
];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    bd: Hsla,
    line: Hsla,
    fg: Hsla,
    mute: Hsla,
    ghost: Hsla,
    band: Hsla,
    needs: Hsla,
    failed: Hsla,
    done: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x171718),
            bd: h(0x2B2B2D),
            line: h(0x2A2A2C),
            fg: h(0xF4F4F5),
            mute: h(0x9A9A9E),
            ghost: h(0xB4B4B8),
            band: h(0x1D1D1F),
            needs: h(0xE5B30C),
            failed: h(0xEF5350),
            done: h(0x2F9BFF),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            bd: h(0xEBEBEB),
            line: h(0xF1F1F1),
            fg: h(0x0A0A0A),
            mute: h(0x848484),
            ghost: h(0x5C5C5C),
            band: h(0xF3F3F4),
            needs: h(0xE3AC05),
            failed: h(0xF04B39),
            done: h(0x1592F5),
        }
    }
}

/// `away-summary` (affectUI Away Summary): the host passes the items and their state.
#[derive(IntoElement)]
pub struct ReturnBriefEl {
    items: Vec<AwayItem>,
    resolved: Vec<String>,
    away: u32,
    limit: usize,
    expanded: bool,
    folded: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ReturnBriefEl {
    fn default() -> Self {
        Self {
            items: away_demo(),
            resolved: vec![],
            away: AWAY_MINUTES,
            limit: 5,
            expanded: false,
            folded: false,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ReturnBriefEl {
    /// Your own items and how long the person was away (minutes).
    pub fn items(mut self, i: Vec<AwayItem>, away: u32) -> Self {
        self.items = i;
        self.away = away;
        self
    }
    /// Rows that resolved (ids).
    pub fn resolved(mut self, r: Vec<String>) -> Self {
        self.resolved = r;
        self
    }
    /// Every row (Show less) or the first `limit`.
    pub fn expanded(mut self, on: bool) -> Self {
        self.expanded = on;
        self
    }
    /// Folded into the divider.
    pub fn folded(mut self, on: bool) -> Self {
        self.folded = on;
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
    /// Row actions, Show more, Got it and the divider.
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

impl RenderOnce for ReturnBriefEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let rows: Vec<AwayItem> = away_order(&self.items)
            .iter()
            .map(|i| {
                if self.resolved.contains(&i.id) {
                    away_resolve(i)
                } else {
                    i.clone()
                }
            })
            .collect();
        let (n, f, d) = away_counts(&rows);
        let head = away_duration(self.away);
        if self.folded {
            let open = fire(&self.act, "open");
            return div()
                .id("oxaw-divider")
                .w(px(W))
                .h(px(32.))
                .flex()
                .items_center()
                .gap(px(12.))
                .text_size(px(12.5))
                .text_color(p.mute)
                .on_click(move |_, w, cx| open(w, cx))
                .child(div().flex_1().h(px(1.)).bg(p.line))
                .child(away_divider(&head, rows.len()))
                .child(div().flex_1().h(px(1.)).bg(p.line));
        }
        let dot = |col: Hsla| div().size(px(6.)).rounded_full().bg(col);
        let cnt = |col: Hsla, txt: String| {
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(dot(col))
                .child(txt)
        };
        let mut counts = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .text_size(px(12.5))
            .text_color(p.mute);
        if n > 0 {
            counts = counts.child(cnt(p.needs, away_needs_word(n)));
        }
        if f > 0 {
            counts = counts.child(cnt(p.failed, format!("{f} failed")));
        }
        if d > 0 {
            counts = counts.child(cnt(p.done, format!("{d} done")));
        }
        let header = div()
            .h(px(40.))
            .px(px(14.))
            .flex()
            .items_center()
            .border_b_1()
            .border_color(p.line)
            .when(self.classic, |x| x.bg(p.band))
            .child(
                div()
                    .flex_1()
                    .flex()
                    .text_size(px(13.5))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("While you were away"),
                    )
                    .child(div().text_color(p.mute).child(format!(" · {head}"))),
            )
            .child(counts);
        let shown = if self.expanded {
            rows.len()
        } else {
            rows.len().min(self.limit)
        };
        let mut list = div().py(px(4.)).flex().flex_col();
        for (i, it) in rows.iter().take(shown).enumerate() {
            let col = match it.kind {
                AwayKind::NeedsYou => p.needs,
                AwayKind::Failed => p.failed,
                AwayKind::Done => p.done,
            };
            let mark = if it.kind == AwayKind::NeedsYou {
                div()
                    .size(px(12.))
                    .rounded_full()
                    .border_2()
                    .border_color(col)
            } else {
                div().size(px(12.)).rounded_full().bg(col)
            };
            let fa = fire(&self.act, ACT[i.min(9)]);
            let raised = it.kind == AwayKind::NeedsYou;
            list = list.child(
                div()
                    .h(px(48.))
                    .pl(px(15.))
                    .pr(px(12.))
                    .flex()
                    .items_center()
                    .gap(px(11.))
                    .child(mark)
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(it.title.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(p.mute)
                                    .child(it.detail.clone()),
                            ),
                    )
                    .children(it.action_label().map(|l| {
                        div()
                            .id(SharedString::from(format!("oxaw-act-{i}")))
                            .h(px(26.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .rounded(px(7.))
                            .text_size(px(12.5))
                            .font_weight(FontWeight::MEDIUM)
                            .when(raised, |x| {
                                x.bg(p.card).border_1().border_color(p.bd).text_color(p.fg)
                            })
                            .when(!raised, |x| x.text_color(p.ghost))
                            .on_click(move |_, w, cx| fa(w, cx))
                            .child(l)
                    })),
            );
        }
        let hidden = rows.len().saturating_sub(self.limit);
        let more = fire(&self.act, "more");
        let got = fire(&self.act, "got-it");
        let foot = div()
            .h(px(38.))
            .px(px(14.))
            .flex()
            .items_center()
            .border_t_1()
            .border_color(p.line)
            .text_size(px(12.5))
            .text_color(p.ghost)
            .child(
                div()
                    .id("oxaw-more")
                    .flex_1()
                    .on_click(move |_, w, cx| more(w, cx))
                    .child(if hidden == 0 {
                        String::new()
                    } else if self.expanded {
                        "Show less".to_string()
                    } else {
                        format!("Show {hidden} more")
                    }),
            )
            .child(
                div()
                    .id("oxaw-got")
                    .on_click(move |_, w, cx| got(w, cx))
                    .child("✓ Got it"),
            );
        div()
            .id("oxaw")
            .w(px(W))
            .flex()
            .flex_col()
            .rounded(px(if self.classic { 16. } else { 12. }))
            .border_1()
            .border_color(p.bd)
            .bg(p.card)
            .text_color(p.fg)
            .overflow_hidden()
            .child(header)
            .child(list)
            .child(foot)
    }
}
