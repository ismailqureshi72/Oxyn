//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Memory Panel**
//! ([`MemoryPanelEl`]) over `oxyn_core::kit::memoryfid`. The element is `RenderOnce`: the host
//! keeps the memories, filter and switch and gets [`Act`] ids back (`switch`, `cat-<n>` for the
//! n-th chip with 0 = All, `pause-<n>` / `forget-<n>` for the n-th row, `new`). A static snapshot:
//! no search field, editing or undo line.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::kit::memoryfid::{
    MpMemory, mp_categories, mp_counts, mp_demo, mp_groups, mp_status,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const CAT: [&str; 6] = ["cat-0", "cat-1", "cat-2", "cat-3", "cat-4", "cat-5"];
const PAUSE: [&str; 10] = [
    "pause-0", "pause-1", "pause-2", "pause-3", "pause-4", "pause-5", "pause-6", "pause-7",
    "pause-8", "pause-9",
];
const FORGET: [&str; 10] = [
    "forget-0", "forget-1", "forget-2", "forget-3", "forget-4", "forget-5", "forget-6", "forget-7",
    "forget-8", "forget-9",
];

/// `memory-panel` (affectUI Memory Panel): the host passes the memories.
#[derive(IntoElement)]
pub struct MemoryPanelEl {
    memories: Vec<MpMemory>,
    filter: Option<String>,
    enabled: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for MemoryPanelEl {
    fn default() -> Self {
        Self {
            memories: mp_demo(),
            filter: None,
            enabled: true,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl MemoryPanelEl {
    /// The memories.
    pub fn memories(mut self, m: Vec<MpMemory>) -> Self {
        self.memories = m;
        self
    }
    /// Category filter (`None` = All).
    pub fn filter(mut self, f: Option<String>) -> Self {
        self.filter = f;
        self
    }
    /// The master switch.
    pub fn enabled(mut self, on: bool) -> Self {
        self.enabled = on;
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
    /// Switch, chips and row actions.
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

impl RenderOnce for MemoryPanelEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, chip, blue, amb) = if dark {
            (
                h(0x17171A),
                h(0x2A2A2E),
                h(0xEDEDEF),
                h(0x8A8A93),
                h(0x232327),
                h(0x4C8DFF),
                h(0xFBBF24),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE5E7EB),
                h(0x111827),
                h(0x6B7280),
                h(0xF3F4F6),
                h(0x2563EB),
                h(0xB45309),
            )
        };
        let cats = mp_categories();
        let counts = mp_counts(&self.memories, &cats);
        let sw = fire(&self.act, "switch");
        let mut chips = div().flex().gap(px(2.));
        let all = std::iter::once(("All".to_string(), self.memories.len())).chain(counts);
        for (i, (label, n)) in all.enumerate().take(6) {
            let on = if i == 0 {
                self.filter.is_none()
            } else {
                self.filter.as_deref() == Some(label.as_str())
            };
            let f = fire(&self.act, CAT[i]);
            chips = chips.child(
                div()
                    .id(SharedString::from(format!("oxmp-c{i}")))
                    .h(px(24.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .rounded_full()
                    .text_size(px(11.5))
                    .when(on, |d| d.border_1().border_color(bd).bg(card))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(format!("{label} {n}")),
            );
        }
        let mut list = div().flex().flex_col().gap(px(6.));
        let mut row_i = 0usize;
        for (g, rows) in mp_groups(&self.memories, &cats, self.filter.as_deref(), "") {
            list = list.child(
                div()
                    .text_size(px(10.))
                    .text_color(mute)
                    .child(g.to_uppercase()),
            );
            for m in rows {
                let i = row_i.min(9);
                row_i += 1;
                let pf = fire(&self.act, PAUSE[i]);
                let ff = fire(&self.act, FORGET[i]);
                list = list.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .py(px(6.))
                        .border_b_1()
                        .border_color(chip)
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_size(px(12.5))
                                        .text_color(if m.paused { mute } else { fg })
                                        .child(m.text.clone()),
                                )
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(6.))
                                        .text_size(px(11.))
                                        .text_color(mute)
                                        .child(format!("{} · {}", m.source, m.date))
                                        .when(m.paused, |d| {
                                            d.child(div().text_color(amb).child("Paused"))
                                        }),
                                ),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("oxmp-p{i}")))
                                .text_size(px(11.))
                                .text_color(mute)
                                .on_click(move |_, w, cx| pf(w, cx))
                                .child(if m.paused { "▶" } else { "❚❚" }),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("oxmp-f{i}")))
                                .text_size(px(11.))
                                .text_color(mute)
                                .on_click(move |_, w, cx| ff(w, cx))
                                .child("✕"),
                        ),
                );
            }
        }
        let nf = fire(&self.act, "new");
        div()
            .w(px(320.))
            .p(px(14.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .border_1()
            .border_color(bd)
            .rounded(px(if self.classic { 12. } else { 14. }))
            .bg(card)
            .text_color(fg)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .child(
                        div()
                            .text_size(px(13.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Memory"),
                    )
                    .child(
                        div()
                            .text_size(px(11.))
                            .text_color(mute)
                            .child(self.memories.len().to_string()),
                    )
                    .child(div().flex_1())
                    .child(div().text_size(px(12.)).child("Use memory"))
                    .child(
                        div()
                            .id("oxmp-sw")
                            .w(px(30.))
                            .h(px(18.))
                            .px(px(2.))
                            .flex()
                            .items_center()
                            .rounded_full()
                            .bg(if self.enabled { blue } else { bd })
                            .when(self.enabled, |d| d.justify_end())
                            .on_click(move |_, w, cx| sw(w, cx))
                            .child(div().size(px(14.)).rounded_full().bg(h(0xFFFFFF))),
                    ),
            )
            .child(chips)
            .child(div().when(!self.enabled, |d| d.opacity(0.45)).child(list))
            .child(
                div()
                    .id("oxmp-new")
                    .text_size(px(12.))
                    .text_color(mute)
                    .on_click(move |_, w, cx| nf(w, cx))
                    .child("+ New memory"),
            )
            .child(div().text_size(px(11.)).text_color(mute).child(mp_status(
                &self.memories,
                self.enabled,
                None,
            )))
    }
}
