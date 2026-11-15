//! affectUI fidelity pass, chunk 2 (plan #32) for GPUI: the exact **Model Picker**
//! ([`ModelPickerMenuEl`]) and **Run Controls** ([`AgentRunPanelEl`]) in affectUI's greys, over
//! `oxyn_core::parity::chatfid`. Elements are `RenderOnce`; the host keeps the state, drives the
//! clock and routes [`Act`] ids and [`Pick`] indices back to it.

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, rgb, rgba,
};
use oxyn_core::parity::{
    chatfid::{Phase, PickerModel, Piece, RunPermission, RunTurn, demo_models, tray_values},
    compose::Effort,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Pick, c};
use crate::brand_mark;

#[derive(Clone, Copy)]
struct Pal {
    surf: Hsla,
    line: Hsla,
    line2: Hsla,
    fg: Hsla,
    ph: Hsla,
    label: Hsla,
    lit: Hsla,
    tile: Hsla,
    trig: Hsla,
    blue: Hsla,
    dash: Hsla,
    dash_on: Hsla,
    track: Hsla,
    band: Hsla,
    steer: Hsla,
    steer_fg: Hsla,
    danger: Hsla,
    send: Hsla,
    send_fg: Hsla,
    off: Hsla,
    you: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            surf: h(0x1C1C1E),
            line: h(0x2E2E31),
            line2: rgba(0xffffff12).into(),
            fg: h(0xF4F4F5),
            ph: h(0x8A8A8E),
            label: h(0xA1A1A3),
            lit: rgba(0xffffff13).into(),
            tile: h(0x2A2A2C),
            trig: h(0x262628),
            blue: h(0x4C9CF8),
            dash: h(0x37373B),
            dash_on: h(0x6E6E73),
            track: h(0x3A3A3E),
            band: h(0x232326),
            steer: rgba(0x3B82F62E).into(),
            steer_fg: h(0x6AA6F8),
            danger: h(0xF87171),
            send: h(0xECECEE),
            send_fg: h(0x111113),
            off: h(0x2F2F31),
            you: h(0x2A2A2D),
        }
    } else {
        Pal {
            surf: h(0xFFFFFF),
            line: h(0xE4E4E7),
            line2: rgba(0x00000012).into(),
            fg: h(0x18181B),
            ph: h(0x8A8A8E),
            label: h(0x52525B),
            lit: h(0xF4F4F5),
            tile: h(0xFFFFFF),
            trig: h(0xF1F1F2),
            blue: h(0x3B82F6),
            dash: h(0xE3E5E9),
            dash_on: h(0xB4B7BE),
            track: h(0xE3E5E9),
            band: h(0xF4F4F5),
            steer: h(0xDDEBFE),
            steer_fg: h(0x2563EB),
            danger: h(0xDC2626),
            send: h(0x18181B),
            send_fg: h(0xFFFFFF),
            off: h(0xE9E9EB),
            you: h(0xF1F1F2),
        }
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

// ─────────────────────────────────────────────────────────────── Model Picker ──

/// `model-picker` (affectUI Model Picker): the trigger pill in a small composer and the
/// "Model settings" menu – rows with letter tiles (or coloured brand marks), speed / quality / cost
/// dashes that fill blue on the lit row, Extended thinking with its Low / Medium / High effort,
/// Web search, and the tray's first-token time and price. Acts: `toggle`, `thinking`, `web`,
/// `effort:low|medium|high`; rows arrive through [`Pick`] (pick) and the host sets `lit` on hover.
#[derive(IntoElement)]
pub struct ModelPickerMenuEl {
    models: Vec<PickerModel>,
    chosen: usize,
    lit: usize,
    open: bool,
    thinking: bool,
    web: bool,
    effort: Effort,
    brands: bool,
    classic: bool,
    act: Option<Act>,
    pick: Option<Pick>,
    theme: &'static Theme,
}

impl Default for ModelPickerMenuEl {
    fn default() -> Self {
        Self::new(demo_models())
    }
}

impl ModelPickerMenuEl {
    /// A picker over `models` (the second chosen and nothing pointed, as affectUI's demo opens:
    /// the chosen row shows only its check and blue dashes).
    pub fn new(models: Vec<PickerModel>) -> Self {
        Self {
            chosen: usize::from(models.len() > 1),
            models,
            lit: usize::MAX,
            open: true,
            thinking: false,
            web: false,
            effort: Effort::Medium,
            brands: false,
            classic: false,
            act: None,
            pick: None,
            theme: &Theme::DARK,
        }
    }
    /// Chosen and pointed rows (`usize::MAX` for none), open.
    pub fn state(mut self, chosen: usize, lit: usize, open: bool) -> Self {
        self.chosen = chosen;
        self.lit = lit;
        self.open = open;
        self
    }
    /// Thinking, its effort, and web search.
    pub fn options(mut self, thinking: bool, effort: Effort, web: bool) -> Self {
        self.thinking = thinking;
        self.effort = effort;
        self.web = web;
        self
    }
    /// Coloured brand marks instead of letter tiles.
    pub fn brands(mut self, on: bool) -> Self {
        self.brands = on;
        self
    }
    /// Classic style (title-case headers, monochrome dashes, round send, tray shelf).
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
    /// Row picks.
    pub fn on_pick(mut self, p: Pick) -> Self {
        self.pick = Some(p);
        self
    }
}

fn tile(m: &PickerModel, brands: bool, p: Pal) -> AnyElement {
    let t = div()
        .flex_none()
        .size(px(20.))
        .rounded(px(6.))
        .border_1()
        .border_color(p.line2)
        .bg(p.tile)
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(11.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(p.fg);
    if brands
        && !m.brand.is_empty()
        && let Some(mark) = brand_mark(&m.brand, 13., p.fg)
    {
        return t.child(mark).into_any_element();
    }
    t.child(m.mark.clone()).into_any_element()
}

fn dashes(v: Option<u8>, lit: bool, classic: bool, p: Pal) -> impl IntoElement {
    let v = v.unwrap_or(0);
    let on = if classic {
        p.fg
    } else if lit {
        p.blue
    } else {
        p.dash_on
    };
    div()
        .flex_none()
        .flex()
        .gap(px(2.))
        .w(px(40.))
        .children((1..=5u8).map(move |i| {
            div()
                .flex_1()
                .h(px(if classic { 4.5 } else { 3. }))
                .rounded(px(2.))
                .bg(if i <= v { on } else { p.dash })
        }))
}

fn switch(on: bool, classic: bool, p: Pal) -> impl IntoElement {
    div()
        .flex_none()
        .w(px(26.))
        .h(px(15.))
        .rounded_full()
        .bg(if !on {
            p.track
        } else if classic {
            p.fg
        } else {
            p.blue
        })
        .p(px(2.))
        .flex()
        .when(on, |d| d.justify_end())
        .child(div().size(px(11.)).rounded_full().bg(if on && classic {
            p.surf
        } else {
            gpui::white()
        }))
}

impl RenderOnce for ModelPickerMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let n = self.models.len().max(1);
        let cur = self.models[self.chosen.min(n - 1)].clone();
        let lit = self.models[if self.lit < n { self.lit } else { self.chosen }.min(n - 1)].clone();
        let classic = self.classic;
        let head = |s: &'static str| -> SharedString {
            if classic {
                let mut c = s.chars();
                c.next()
                    .map(|f| f.to_string() + &c.as_str().to_lowercase())
                    .unwrap_or_default()
                    .into()
            } else {
                s.into()
            }
        };
        let trig = div()
            .id("mp-trigger")
            .flex()
            .items_center()
            .gap(px(7.))
            .h(px(28.))
            .pl(px(4.))
            .pr(px(9.))
            .rounded_full()
            .bg(p.trig)
            .text_size(px(13.))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(p.fg)
            .cursor_pointer()
            .child(tile(&cur, self.brands, p))
            .child(cur.name.clone())
            .when(self.thinking, |d| {
                d.child(div().text_color(p.blue).child("💡"))
            })
            .child(
                div()
                    .text_color(p.ph)
                    .child(if self.open { "⌃" } else { "⌄" }),
            )
            .on_click({
                let f = fire(&self.act, "toggle");
                move |_, w, cx| f(w, cx)
            });
        let card = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .pt(px(14.))
            .px(px(12.))
            .pb(px(12.))
            .rounded(px(16.))
            .border_1()
            .border_color(p.line)
            .bg(p.surf)
            .child(
                div()
                    .px(px(4.))
                    .text_size(px(13.5))
                    .text_color(p.ph)
                    .child("Ask anything…"),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .child(trig)
                    .child(div().flex_1())
                    .child(
                        div()
                            .size(px(30.))
                            .rounded(if classic { px(15.) } else { px(9.) })
                            .bg(p.send)
                            .text_color(p.send_fg)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child("↑"),
                    ),
            );
        let mut root = div().relative().w(px(362.)).flex().flex_col().child(card);
        if !self.open {
            return root;
        }
        let mut menu = div()
            .absolute()
            .top(px(84.))
            .left(px(12.))
            .w(px(320.))
            .flex()
            .flex_col()
            .p(px(6.))
            .rounded(px(if classic { 20. } else { 14. }))
            .border_1()
            .border_color(p.line)
            .bg(p.surf)
            .shadow_lg()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(30.))
                    .pl(px(8.))
                    .pr(px(10.))
                    .text_size(px(10.5))
                    .text_color(p.ph)
                    .child(div().flex_1().child(head("MODEL")))
                    .child(div().w(px(40.)).child(head("SPEED")))
                    .child(div().w(px(48.)).child(head("QUALITY")))
                    .child(div().w(px(40.)).child(head("COST"))),
            );
        for (i, m) in self.models.iter().enumerate() {
            let is_lit = i == self.lit;
            let pk = self.pick.clone();
            let mut row = div()
                .id(SharedString::from(format!("mp-row-{i}")))
                .flex()
                .items_center()
                .gap(px(12.))
                .h(px(48.))
                .px(px(10.))
                .rounded(px(10.))
                .cursor_pointer()
                .when(is_lit, |d| d.bg(p.lit))
                .child(tile(m, self.brands, p))
                .child(
                    div()
                        .flex_1()
                        .flex()
                        .flex_col()
                        .child(
                            div()
                                .flex()
                                .gap(px(6.))
                                .text_size(px(13.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(p.fg)
                                .child(m.name.clone())
                                .when(i == self.chosen, |d| {
                                    d.child(
                                        div()
                                            .text_color(if classic { p.fg } else { p.blue })
                                            .child("✓"),
                                    )
                                }),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.ph)
                                .child(m.description.clone()),
                        ),
                )
                .on_click(move |_, w, cx| {
                    if let Some(f) = &pk {
                        f(i, w, cx)
                    }
                });
            row = if m.varies() {
                row.child(
                    div()
                        .w(px(128.))
                        .text_size(px(12.))
                        .text_color(p.ph)
                        .child("Varies"),
                )
            } else {
                let blue = is_lit || i == self.chosen;
                row.child(dashes(m.speed, blue, classic, p))
                    .child(dashes(m.quality, blue, classic, p))
                    .child(dashes(m.cost, blue, classic, p))
            };
            menu = menu.child(row);
        }
        let sw_row = |id: &'static str, glyph: &'static str, label: &'static str, on: bool| {
            div()
                .id(id)
                .flex()
                .items_center()
                .gap(px(12.))
                .h(px(36.))
                .px(px(10.))
                .rounded(px(10.))
                .text_size(px(13.))
                .text_color(p.fg)
                .cursor_pointer()
                .child(div().w(px(20.)).text_color(p.ph).child(glyph))
                .child(div().flex_1().child(label))
                .child(switch(on, classic, p))
                .on_click({
                    let f = fire(&self.act, id);
                    move |_, w, cx| f(w, cx)
                })
        };
        menu = menu
            .child(div().h(px(1.)).my(px(6.)).mx(px(-6.)).bg(p.line2))
            .child(sw_row("thinking", "💡", "Extended thinking", self.thinking));
        if self.thinking {
            let mut seg = div().flex().gap(px(2.));
            for (e, id) in [
                (Effort::Low, "effort:low"),
                (Effort::Medium, "effort:medium"),
                (Effort::High, "effort:high"),
            ] {
                let on = e == self.effort;
                let f = fire(&self.act, id);
                seg = seg.child(
                    div()
                        .id(id)
                        .h(px(22.))
                        .px(px(7.))
                        .rounded(px(7.))
                        .text_size(px(12.))
                        .text_color(if on { p.fg } else { p.ph })
                        .when(on, |d| d.bg(p.lit).border_1().border_color(p.line2))
                        .cursor_pointer()
                        .child(e.label())
                        .on_click(move |_, w, cx| f(w, cx)),
                );
            }
            menu = menu.child(
                div()
                    .flex()
                    .items_center()
                    .h(px(32.))
                    .pl(px(42.))
                    .pr(px(10.))
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(13.))
                            .text_color(p.ph)
                            .child("Effort"),
                    )
                    .child(seg),
            );
        }
        menu = menu.child(sw_row("web", "🌐", "Web search", self.web));
        let tray_text: SharedString = if self.web {
            "Looks things up on the web when it helps".into()
        } else if let Some((l, pr)) = tray_values(&lit, self.thinking, self.effort) {
            format!("{l} first token · {pr} / M tokens").into()
        } else {
            "Time and price vary by message".into()
        };
        let kbd = |s: &'static str| {
            div()
                .h(px(19.))
                .px(px(4.))
                .rounded(px(5.))
                .border_1()
                .border_color(p.line)
                .text_size(px(11.))
                .text_color(p.ph)
                .child(s)
        };
        menu = menu.child(
            div()
                .flex()
                .items_center()
                .gap(px(4.))
                .h(px(38.))
                .mt(px(6.))
                .mx(px(-6.))
                .mb(px(-6.))
                .pl(px(16.))
                .pr(px(10.))
                .text_size(px(12.))
                .text_color(p.ph)
                .when(classic, |d| d.bg(p.band).rounded_b(px(19.)))
                .when(!classic, |d| d.border_t_1().border_color(p.line2))
                .child(div().flex_1().child(tray_text))
                .child(kbd("↑↓"))
                .child(kbd("↵")),
        );
        root = root.child(menu);
        root
    }
}

// ─────────────────────────────────────────────────────────────── Run Controls ──

/// `run-controls` (affectUI Run Controls): the run panel (pixel mark, agent, status, the
/// streamed text with steer pills and a block caret, "Esc to stop", the Stopped and Done rows),
/// the queue band and the composer (Chat | Act, permissions, context ring, round send / stop).
/// Acts: `stop`, `send`, `resume`, `retry`, `dismiss`, `remove`, `chat`, `act`, `permissions`,
/// `perm:ask|auto|full`.
#[derive(IntoElement)]
pub struct AgentRunPanelEl {
    agent: SharedString,
    done: Vec<RunTurn>,
    turn: Option<(RunTurn, Vec<Piece>, Phase, usize)>,
    queue: Vec<String>,
    draft: SharedString,
    chat: bool,
    permission: RunPermission,
    menu: bool,
    context: u32,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl AgentRunPanelEl {
    /// An idle panel for `agent`.
    pub fn new(agent: impl Into<SharedString>) -> Self {
        Self {
            agent: agent.into(),
            done: vec![],
            turn: None,
            queue: vec![],
            draft: SharedString::default(),
            chat: false,
            permission: RunPermission::AskFirst,
            menu: false,
            context: 38,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Finished turns above, and the current one: the turn, its shown pieces (see
    /// `RunTurn::upto`), its phase and the words shown.
    pub fn turns(
        mut self,
        done: Vec<RunTurn>,
        current: Option<(RunTurn, Vec<Piece>, Phase, usize)>,
    ) -> Self {
        self.done = done;
        self.turn = current;
        self
    }
    /// Queued messages, the composer's text and the context in use (percent).
    pub fn composer(
        mut self,
        queue: Vec<String>,
        draft: impl Into<SharedString>,
        context: u32,
    ) -> Self {
        self.queue = queue;
        self.draft = draft.into();
        self.context = context;
        self
    }
    /// Chat mode, permission and whether its menu is open.
    pub fn mode(mut self, chat: bool, permission: RunPermission, menu: bool) -> Self {
        self.chat = chat;
        self.permission = permission;
        self.menu = menu;
        self
    }
    /// Classic style (the run as a white panel on a grey tray).
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

fn mark(p: Pal) -> impl IntoElement {
    let _ = p;
    div()
        .flex_none()
        .w(px(14.))
        .flex()
        .flex_wrap()
        .gap(px(1.5))
        .children((0..9).map(|k| {
            let on = k % 2 == 0 || (3..6).contains(&k);
            div().size(px(3.5)).rounded(px(0.6)).bg(if on {
                rgb(0x3B82F6)
            } else {
                rgba(0x3B82F600)
            })
        }))
}

fn pieces_el(pieces: &[Piece], caret: bool, p: Pal) -> AnyElement {
    let mut paras: Vec<Vec<Piece>> = vec![vec![]];
    for x in pieces {
        if *x == Piece::Break {
            paras.push(vec![]);
        } else if let Some(l) = paras.last_mut() {
            l.push(x.clone());
        }
    }
    let last = paras.len() - 1;
    div()
        .flex()
        .flex_col()
        .gap(px(20.))
        .pl(px(24.))
        .children(paras.into_iter().enumerate().map(move |(k, para)| {
            div()
                .flex()
                .flex_wrap()
                .items_center()
                .gap(px(4.))
                .text_size(px(13.5))
                .line_height(px(20.))
                .text_color(p.fg)
                .children(para.into_iter().map(move |x| {
                    match x {
                        Piece::Steer(s) => div()
                            .h(px(22.))
                            .px(px(9.))
                            .rounded_full()
                            .bg(p.steer)
                            .text_color(p.steer_fg)
                            .text_size(px(12.5))
                            .child(format!("↩ {s}"))
                            .into_any_element(),
                        Piece::Text(t) => div().child(t).into_any_element(),
                        Piece::Break => div().into_any_element(),
                    }
                }))
                .when(caret && k == last, |d| {
                    d.child(div().w(px(7.)).h(px(14.)).bg(p.ph))
                })
        }))
        .into_any_element()
}

impl RenderOnce for AgentRunPanelEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let header = |name: SharedString, status: &'static str| {
            div()
                .flex()
                .items_center()
                .gap(px(10.))
                .h(px(20.))
                .text_size(px(13.))
                .child(mark(p))
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.fg)
                        .child(name),
                )
                .child(div().text_size(px(12.5)).text_color(p.ph).child(status))
        };
        let you = |t: &str| {
            div().flex().justify_end().child(
                div()
                    .px(px(12.))
                    .py(px(7.))
                    .rounded(px(14.))
                    .bg(p.you)
                    .text_size(px(13.5))
                    .text_color(p.fg)
                    .child(t.to_string()),
            )
        };
        let mut run = div()
            .id("rq-run")
            .h(px(250.))
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .gap(px(18.))
            .pt(px(18.))
            .px(px(16.))
            .pb(px(10.));
        if self.turn.is_none() && self.done.is_empty() {
            run = run.child(
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(10.))
                    .text_color(p.ph)
                    .child(mark(p))
                    .child(format!(
                        "Ask {} anything – steer it or stop it while it works.",
                        self.agent
                    )),
            );
        }
        for t in &self.done {
            run = run.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .when(!t.prompt.is_empty(), |d| d.child(you(&t.prompt)))
                    .child(header(self.agent.clone(), "Done"))
                    .child(pieces_el(&t.upto(t.words()), false, p)),
            );
        }
        let running = self.turn.as_ref().is_some_and(|t| t.2.running());
        if let Some((t, shown, ph, w)) = &self.turn {
            let steered = shown.iter().any(|x| matches!(x, Piece::Steer(_)));
            let ghost = |id: &'static str, label: &'static str| {
                let f = fire(&self.act, id);
                div()
                    .id(id)
                    .h(px(28.))
                    .px(px(8.))
                    .rounded(px(8.))
                    .text_size(px(12.5))
                    .text_color(p.ph)
                    .cursor_pointer()
                    .child(label)
                    .on_click(move |_, w, cx| f(w, cx))
            };
            let foot = div()
                .flex()
                .items_center()
                .gap(px(6.))
                .min_h(px(28.))
                .pl(px(24.))
                .text_size(px(12.5))
                .text_color(p.ph);
            let foot = match ph {
                Phase::Thinking | Phase::Writing => foot
                    .child(
                        div()
                            .h(px(18.))
                            .px(px(5.))
                            .rounded(px(5.))
                            .border_1()
                            .border_color(p.line)
                            .text_size(px(11.))
                            .child("Esc"),
                    )
                    .child("to stop"),
                Phase::Stopped => foot
                    .child(format!("Stopped at {w} words"))
                    .child(div().flex_1())
                    .child({
                        let f = fire(&self.act, "resume");
                        div()
                            .id("resume")
                            .h(px(28.))
                            .px(px(12.))
                            .rounded_full()
                            .bg(p.send)
                            .text_color(p.send_fg)
                            .font_weight(FontWeight::SEMIBOLD)
                            .cursor_pointer()
                            .child("▶ Resume")
                            .on_click(move |_, w, cx| f(w, cx))
                    })
                    .child(ghost("retry", "↻ Retry"))
                    .child(ghost("dismiss", "✕ Dismiss")),
                Phase::Done => foot
                    .child(format!(
                        "Done in {}",
                        oxyn_core::parity::chatfid::took(t.words())
                    ))
                    .child(div().flex_1())
                    .child(ghost("retry", "↻ Retry")),
                Phase::Idle => foot,
            };
            run = run.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .when(!t.prompt.is_empty(), |d| d.child(you(&t.prompt)))
                    .child(header(self.agent.clone(), ph.label(steered)))
                    .child(pieces_el(shown, ph.running() || *ph == Phase::Stopped, p))
                    .child(foot),
            );
        }
        let mut root = div()
            .relative()
            .w(px(520.))
            .flex()
            .flex_col()
            .text_color(p.fg)
            .when(!self.classic, |d| {
                d.rounded(px(16.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.surf)
            })
            .when(self.classic, |d| d.rounded(px(22.)).bg(p.band).p(px(6.)))
            .child(if self.classic {
                run.rounded(px(16.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.surf)
            } else {
                run
            });
        if !self.queue.is_empty() {
            root = root.child(
                div()
                    .flex()
                    .flex_col()
                    .when(!self.classic, |d| d.bg(p.band))
                    .children(self.queue.iter().enumerate().map(|(k, m)| {
                        let f = fire(&self.act, "remove");
                        div()
                            .flex()
                            .items_center()
                            .gap(px(10.))
                            .h(px(40.))
                            .pl(px(16.))
                            .pr(px(12.))
                            .text_size(px(13.))
                            .child(div().text_color(p.ph).child("≡"))
                            .child(div().flex_1().text_color(p.fg).child(m.clone()))
                            .when(k == 0, |d| {
                                d.child(div().text_size(px(12.)).text_color(p.ph).child("Next"))
                            })
                            .child(
                                div()
                                    .id(SharedString::from(format!("rq-x{k}")))
                                    .text_color(p.ph)
                                    .cursor_pointer()
                                    .child("✕")
                                    .on_click(move |_, w, cx| f(w, cx)),
                            )
                    })),
            );
        }
        let perm = self.permission;
        let perm_glyph = |x: RunPermission| match x {
            RunPermission::AskFirst => "⛉",
            RunPermission::AutoEdit => "✎",
            RunPermission::FullAccess => "ϟ",
        };
        let seg = |id: &'static str, label: &'static str, on: bool| {
            let f = fire(&self.act, id);
            div()
                .id(id)
                .h(px(24.))
                .px(px(10.))
                .rounded(px(6.))
                .text_size(px(12.5))
                .text_color(if on { p.fg } else { p.ph })
                .when(on, |d| {
                    d.bg(p.surf).font_weight(FontWeight::SEMIBOLD).shadow_sm()
                })
                .cursor_pointer()
                .child(label)
                .on_click(move |_, w, cx| f(w, cx))
        };
        let send = {
            let stop = running && self.draft.is_empty();
            let f = fire(&self.act, if stop { "stop" } else { "send" });
            div()
                .id("rq-send")
                .size(px(30.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(if stop || !self.draft.is_empty() {
                    p.send
                } else {
                    p.off
                })
                .text_color(if stop || !self.draft.is_empty() {
                    p.send_fg
                } else {
                    p.ph
                })
                .cursor_pointer()
                .child(if stop {
                    div()
                        .size(px(9.))
                        .rounded(px(2.))
                        .bg(p.send_fg)
                        .into_any_element()
                } else {
                    div().child("↑").into_any_element()
                })
                .on_click(move |_, w, cx| f(w, cx))
        };
        let mut pw = div().relative().child({
            let f = fire(&self.act, "permissions");
            div()
                .id("rq-perm")
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(28.))
                .px(px(8.))
                .rounded(px(8.))
                .text_size(px(12.5))
                .text_color(if perm == RunPermission::FullAccess {
                    p.danger
                } else {
                    p.label
                })
                .when(self.chat, |d| d.opacity(0.45))
                .cursor_pointer()
                .child(perm_glyph(perm))
                .child(perm.label())
                .child("⌄")
                .on_click(move |_, w, cx| f(w, cx))
        });
        if self.menu && !self.chat {
            let mut m = div()
                .absolute()
                .bottom(px(36.))
                .left_0()
                .w(px(268.))
                .p(px(5.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .bg(p.surf)
                .shadow_lg()
                .flex()
                .flex_col();
            for x in RunPermission::ALL {
                let id: &'static str = match x {
                    RunPermission::AskFirst => "perm:ask",
                    RunPermission::AutoEdit => "perm:auto",
                    RunPermission::FullAccess => "perm:full",
                };
                let f = fire(&self.act, id);
                let col = if x == RunPermission::FullAccess {
                    p.danger
                } else {
                    p.fg
                };
                m = m.child(
                    div()
                        .id(id)
                        .flex()
                        .gap(px(10.))
                        .px(px(10.))
                        .py(px(9.))
                        .rounded(px(9.))
                        .cursor_pointer()
                        .child(div().text_color(col).child(perm_glyph(x)))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .child(div().text_size(px(13.)).text_color(col).child(x.label()))
                                .child(div().text_size(px(12.)).text_color(p.ph).child(x.desc())),
                        )
                        .when(x == perm, |d| d.child("✓"))
                        .on_click(move |_, w, cx| f(w, cx)),
                );
            }
            pw = pw.child(m);
        }
        let comp = div()
            .relative()
            .m(px(4.))
            .when(self.classic, |d| d.mx_0().mb_0().mt(px(6.)))
            .flex()
            .flex_col()
            .gap(px(10.))
            .pt(px(12.))
            .pb(px(10.))
            .pl(px(12.))
            .pr(px(10.))
            .rounded(px(if self.classic { 16. } else { 13. }))
            .border_1()
            .border_color(p.line)
            .bg(p.surf)
            .child(
                div()
                    .text_size(px(13.5))
                    .text_color(if self.draft.is_empty() { p.ph } else { p.fg })
                    .child(if !self.draft.is_empty() {
                        self.draft.clone()
                    } else if running {
                        "Steer the agent, or queue what's next…".into()
                    } else {
                        "Ask the agent…".into()
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .child(
                        div()
                            .flex()
                            .p(px(2.))
                            .rounded(px(8.))
                            .border_1()
                            .border_color(p.line)
                            .child(seg("chat", "Chat", self.chat))
                            .child(seg("act", "Act", !self.chat)),
                    )
                    .child(pw)
                    .child(div().flex_1())
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .text_size(px(12.))
                            .text_color(p.ph)
                            .child(
                                div()
                                    .size(px(14.))
                                    .rounded_full()
                                    .border_2()
                                    .border_color(p.track),
                            )
                            .child(format!("{}%", self.context)),
                    )
                    .child(send),
            );
        root.child(comp)
    }
}
