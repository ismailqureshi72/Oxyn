//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32) for GPUI: the exact
//! **Context treemap** ([`ContextTreemapEl`]) over `oxyn_core::kit::ctxmapfid`. The element is
//! `RenderOnce`: the host passes the blocks, the limit and the compaction side and routes [`Act`]
//! ids back (`tile-<n>` for the n-th block, `free`, `before`, `after`). A static snapshot: no
//! glide, no hover card, no agent menu.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative, rgb,
};
use oxyn_core::kit::ctxmapfid::{
    CtxBlock, CtxCompaction, CtxKind, ctx_demo, ctx_fmt, ctx_layout, ctx_pct, ctx_used,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const W: f32 = 680.0;
const H: f32 = 282.0;
const TILE: [&str; 16] = [
    "tile-0", "tile-1", "tile-2", "tile-3", "tile-4", "tile-5", "tile-6", "tile-7", "tile-8",
    "tile-9", "tile-10", "tile-11", "tile-12", "tile-13", "tile-14", "tile-15",
];

fn tint(k: CtxKind, dark: bool) -> (u32, u32) {
    match (k, dark) {
        (CtxKind::System, false) => (0xEDEDED, 0x5F5F5F),
        (CtxKind::Tools, false) => (0xE1F3FA, 0x0D7D93),
        (CtxKind::Memory, false) => (0xF7EAFC, 0xA855C7),
        (CtxKind::File, false) => (0xE3EEFC, 0x2563EB),
        (CtxKind::Chat, false) => (0xF6EAFF, 0x9333EA),
        (CtxKind::Result, false) => (0xFFF0DC, 0xB45309),
        (CtxKind::Summary, false) => (0xDFF7E6, 0x15803D),
        (CtxKind::System, true) => (0x2F2F2F, 0xD4D4D8),
        (CtxKind::Tools, true) => (0x17323D, 0x67E8F9),
        (CtxKind::Memory, true) => (0x3A2947, 0xD8B4FE),
        (CtxKind::File, true) => (0x1E2A3D, 0x93C5FD),
        (CtxKind::Chat, true) => (0x322347, 0xC4B5FD),
        (CtxKind::Result, true) => (0x3B2A16, 0xFBBF24),
        (CtxKind::Summary, true) => (0x173A25, 0x86EFAC),
    }
}

/// `context-map` (affectUI Context Map): the host passes the context.
#[derive(IntoElement)]
pub struct ContextTreemapEl {
    blocks: Vec<CtxBlock>,
    limit: u32,
    compaction: Option<CtxCompaction>,
    before: bool,
    title: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ContextTreemapEl {
    fn default() -> Self {
        Self {
            blocks: ctx_demo(0.35),
            limit: 200_000,
            compaction: None,
            before: false,
            title: "Context".into(),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ContextTreemapEl {
    /// The parts in tile order and the window's size.
    pub fn blocks(mut self, b: Vec<CtxBlock>, limit: u32) -> Self {
        self.blocks = b;
        self.limit = limit;
        self
    }
    /// The last compaction and whether its Before side shows.
    pub fn compaction(mut self, c: Option<CtxCompaction>, before: bool) -> Self {
        self.compaction = c;
        self.before = before;
        self
    }
    /// Heading.
    pub fn title(mut self, t: &str) -> Self {
        self.title = t.into();
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
    /// Tiles and Before | After.
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

impl RenderOnce for ContextTreemapEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, chip) = if dark {
            (
                h(0x161616),
                h(0x2A2A2A),
                h(0xF4F4F5),
                h(0xA1A1AA),
                h(0x262626),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xECECEC),
                h(0x18181B),
                h(0x71717A),
                h(0xF1F1F1),
            )
        };
        let shown = match (&self.compaction, self.before) {
            (Some(c), true) => c.before.clone(),
            _ => self.blocks.clone(),
        };
        let used = ctx_used(&shown);
        let free = self.limit.saturating_sub(used);
        let mut weights: Vec<f32> = shown.iter().map(|b| b.tokens as f32).collect();
        weights.push(free as f32);
        let rects = ctx_layout(&weights, W / H);
        let mut bar = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(32.))
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            )
            .child(div().flex_1());
        if self.compaction.is_some() {
            for (i, (id, label)) in [("before", "Before"), ("after", "After")]
                .iter()
                .enumerate()
            {
                let on = self.before == (i == 0);
                let f = fire(&self.act, id);
                bar = bar.child(
                    div()
                        .id(SharedString::from(format!("oxcm-{id}")))
                        .h(px(24.))
                        .px(px(10.))
                        .flex()
                        .items_center()
                        .rounded(px(6.))
                        .text_size(px(12.5))
                        .when(on, |d| d.bg(card).border_1().border_color(bd))
                        .text_color(if on { fg } else { mute })
                        .on_click(move |_, w, cx| f(w, cx))
                        .child(*label),
                );
            }
        }
        bar = bar
            .child(div().text_size(px(13.)).child(format!(
                "{} / {}",
                ctx_fmt(used),
                ctx_fmt(self.limit)
            )))
            .child(
                div()
                    .px(px(7.))
                    .rounded(px(6.))
                    .bg(chip)
                    .text_size(px(12.))
                    .child(format!("{}%", ctx_pct(used, self.limit))),
            );
        let mut map = div()
            .relative()
            .h(px(H))
            .rounded(px(9.))
            .border_1()
            .border_color(bd)
            .bg(card);
        for r in rects {
            let is_free = r.i >= shown.len();
            let (bg, tfg, label, line, id) = if is_free {
                (card, mute, "Free".to_string(), ctx_fmt(free), "free")
            } else {
                let b = &shown[r.i];
                let (bgc, fgc) = tint(b.kind, dark);
                (
                    h(bgc),
                    h(fgc),
                    b.label.clone(),
                    format!("{} · {}", ctx_fmt(b.tokens), b.kind.word()),
                    TILE[r.i.min(15)],
                )
            };
            let f = fire(&self.act, id);
            map = map.child(
                div()
                    .id(SharedString::from(format!("oxcm-tile-{}", r.i)))
                    .absolute()
                    .left(relative(r.x))
                    .top(relative(r.y))
                    .w(relative(r.w))
                    .h(relative(r.h))
                    .p(px(2.))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(
                        div()
                            .size_full()
                            .overflow_hidden()
                            .p(px(8.))
                            .rounded(px(6.))
                            .bg(bg)
                            .border_1()
                            .border_color(if is_free { bd } else { bg })
                            .text_color(tfg)
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(label),
                            )
                            .child(div().text_size(px(11.5)).child(line)),
                    ),
            );
        }
        div()
            .w(px(W))
            .flex()
            .flex_col()
            .gap(px(10.))
            .when(self.classic, |d| d.p(px(12.)).rounded(px(20.)).bg(chip))
            .text_color(fg)
            .child(bar)
            .child(map)
    }
}
