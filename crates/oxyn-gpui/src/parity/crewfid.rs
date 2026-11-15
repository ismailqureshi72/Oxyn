//! affectUI fidelity pass, overnight batch, family B part 2 (plan #32) for GPUI: the exact
//! **Agent Crew** ([`AgentCrewEl`]) over `oxyn_core::parity::crewfid`. The element is
//! `RenderOnce`: the host keeps the clock, the project, the open card and any steering, and
//! routes [`Act`] ids back (`card-<n>` opens card n, `close`, `steer-<action>`, `project-<n>`).

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, rgb,
};
use oxyn_core::parity::crewfid::{
    CrewCard, CrewState, clamp_context, context_tone, crew_at, crew_cost, crew_duration,
    running_ms, short_tokens, steer_label, tokens_label,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c};
use crate::brand_mark;

const CW: f32 = 205.0;
const GAP: f32 = 38.0;
const IND: f32 = 28.0;
const MAIN_H: f32 = 112.0;
const HELP_H: f32 = 79.0;
const VGAP: f32 = 40.0;
const HGAP: f32 = 14.0;
const WIDE: f32 = 760.0;

const CARD_ACTS: [&str; 12] = [
    "card-0", "card-1", "card-2", "card-3", "card-4", "card-5", "card-6", "card-7", "card-8",
    "card-9", "card-10", "card-11",
];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    line: Hsla,
    fg: Hsla,
    mute: Hsla,
    chip: Hsla,
    track: Hsla,
    band: Hsla,
    ink: Hsla,
    ink_fg: Hsla,
    amber: Hsla,
    green: Hsla,
    red: Hsla,
    colors: [Hsla; 6],
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x202023),
            line: h(0x2C2C30),
            fg: h(0xF2F3F5),
            mute: h(0x9A9EA6),
            chip: h(0x2A2C31),
            track: h(0x34343A),
            band: h(0x1A1A1D),
            ink: h(0xF2F2F4),
            ink_fg: h(0x111114),
            amber: h(0xF5A524),
            green: h(0x4ADE80),
            red: h(0xF87171),
            colors: [
                h(0x5B8DEF),
                h(0xA374F2),
                h(0x36A9DC),
                h(0x43B672),
                h(0xEC6AAE),
                h(0xF29A45),
            ],
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            line: h(0xE7E7EA),
            fg: h(0x17181B),
            mute: h(0x6B7280),
            chip: h(0xEFEFF1),
            track: h(0xECECEF),
            band: h(0xF6F6F7),
            ink: h(0x111114),
            ink_fg: h(0xFFFFFF),
            amber: h(0xB45309),
            green: h(0x16A34A),
            red: h(0xDC2626),
            colors: [
                h(0x2F7DD1),
                h(0x8B5CF6),
                h(0x1E9BD0),
                h(0x2FA35C),
                h(0xD9468F),
                h(0xE57A12),
            ],
        }
    }
}

/// One placed card: (agent, depth, parent, colour index, x, y, height).
type Placed = (CrewCard, usize, Option<usize>, usize, f32, f32, f32);

fn place(root: &CrewCard) -> (Vec<Placed>, f32) {
    let flat = root.flatten();
    let n = root.children.len();
    let span = if n == 0 {
        CW
    } else {
        n as f32 * CW + (n as f32 - 1.0) * GAP
    };
    let extra = root
        .children
        .last()
        .map(|k| {
            k.flatten()
                .iter()
                .map(|x| x.0 as f32 * IND)
                .fold(0.0, f32::max)
        })
        .unwrap_or(0.0);
    let ox = ((WIDE - span - extra) / 2.0).max(0.0);
    let mut out: Vec<Placed> = vec![];
    let mut col_y = vec![MAIN_H + VGAP + MAIN_H + HGAP; n.max(1)];
    let (mut col, mut kid) = (0usize, 0usize);
    for (depth, parent, card) in flat {
        let h = if depth <= 1 { MAIN_H } else { HELP_H };
        let (colour, x, y) = match depth {
            0 => (0, ox + (span - CW) / 2.0, 0.0),
            1 => {
                col = kid;
                kid += 1;
                (1 + (col % 5), ox + col as f32 * (CW + GAP), MAIN_H + VGAP)
            }
            d => {
                let y = col_y[col];
                col_y[col] += HELP_H + HGAP;
                (
                    parent.map(|p| out[p].3).unwrap_or(0),
                    ox + col as f32 * (CW + GAP) + (d as f32 - 1.0) * IND,
                    y,
                )
            }
        };
        out.push((card.clone(), depth, parent, colour, x, y, h));
    }
    let hgt = out.iter().map(|x| x.5 + x.6).fold(MAIN_H, f32::max);
    (out, hgt)
}

/// `agent-crew` (affectUI Agent Crew): the host passes the project, the milliseconds into the
/// run (or its own crew), and the open card; acts `card-<n>`, `close`, `steer-<action>`.
#[derive(IntoElement)]
pub struct AgentCrewEl {
    root: CrewCard,
    title: String,
    elapsed: u64,
    open: Option<usize>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AgentCrewEl {
    fn default() -> Self {
        Self {
            root: crew_at("checkout", 9_500).0,
            title: "Fix the checkout crash".into(),
            elapsed: 9_500,
            open: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AgentCrewEl {
    /// A demo project (`checkout`, `launch`, `postgres`) `ms` into its run.
    pub fn demo(mut self, project: &str, title: &str, ms: u64) -> Self {
        self.root = crew_at(project, ms).0;
        self.title = title.into();
        self.elapsed = ms;
        self
    }
    /// Your own crew and title, `ms` into the run.
    pub fn crew(mut self, root: CrewCard, title: &str, ms: u64) -> Self {
        self.root = root;
        self.title = title.into();
        self.elapsed = ms;
        self
    }
    /// The card (depth-first index) whose details are open.
    pub fn open(mut self, i: Option<usize>) -> Self {
        self.open = i;
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

fn badge(st: CrewState, icon_only: bool, p: Pal, classic: bool) -> AnyElement {
    let (fg, bg) = match st {
        CrewState::Active => (p.colors[0], alpha(p.colors[0], 0.14)),
        CrewState::Done => (p.green, alpha(p.green, 0.14)),
        CrewState::Waiting => (p.amber, alpha(p.amber, 0.14)),
        CrewState::Failed => (p.red, alpha(p.red, 0.14)),
        _ => (p.mute, p.chip),
    };
    let (fg, bg) =
        if classic && matches!(st, CrewState::Active | CrewState::Done | CrewState::Waiting) {
            (rgb(0xFFFFFF).into(), fg)
        } else {
            (fg, bg)
        };
    let glyph = match st {
        CrewState::Active => "◌",
        CrewState::Done => "✓",
        CrewState::Waiting => "◷",
        CrewState::Failed => "!",
        CrewState::Paused => "‖",
        CrewState::Idle => "•",
    };
    let mut b = div()
        .h(px(18.))
        .rounded(px(5.))
        .bg(bg)
        .text_color(fg)
        .text_size(px(11.))
        .flex()
        .items_center()
        .justify_center()
        .gap(px(4.))
        .child(glyph);
    b = if icon_only {
        b.w(px(18.))
    } else {
        b.px(px(6.)).child(st.label())
    };
    b.into_any_element()
}

fn mark(color: Hsla, active: bool) -> AnyElement {
    let mut g = div().size(px(12.)).flex().flex_wrap().gap(px(1.6));
    for k in 0..9 {
        let a = if active && (k % 2 == 1) { 0.45 } else { 1.0 };
        g = g.child(div().size(px(2.4)).rounded(px(0.6)).bg(alpha(color, a)));
    }
    g.into_any_element()
}

impl RenderOnce for AgentCrewEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let (placed, hgt) = place(&self.root);
        let (working, needs, all) = self.root.counts();
        let mut head = div()
            .h(px(28.))
            .mb(px(18.))
            .flex()
            .items_center()
            .gap(px(10.))
            .child(div().text_color(p.mute).child("▭"))
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            )
            .child(div().text_color(p.mute).text_size(px(11.)).child("⌄"))
            .child(div().flex_1());
        if working > 0 {
            head = head.child(
                div()
                    .text_size(px(12.5))
                    .text_color(p.colors[0])
                    .child(format!("{working} working")),
            );
        }
        if needs > 0 {
            head = head.child(
                div()
                    .text_size(px(12.5))
                    .text_color(p.amber)
                    .child(format!("{needs} needs you")),
            );
        }
        head = head.child(
            div()
                .text_size(px(12.5))
                .text_color(p.mute)
                .child(format!("{all} agents")),
        );

        let mut tree = div().relative().w(px(WIDE)).h(px(hgt));
        // Connectors first, under the cards.
        for (card, depth, parent, _, x, y, _) in placed.iter() {
            let Some(pi) = parent else { continue };
            let par = &placed[*pi];
            let col = alpha(p.colors[par.3], 0.55);
            let _ = card;
            if *depth == 1 {
                let x0 = par.4 + CW / 2.0;
                let y0 = par.5 + par.6 + 7.5;
                let bus = par.5 + par.6 + 20.0;
                let x1 = x + CW / 2.0;
                tree = tree.child(
                    div()
                        .absolute()
                        .left(px(x0 - 0.75))
                        .top(px(y0))
                        .w(px(1.5))
                        .h(px(bus - y0))
                        .bg(col),
                );
                let (l, w) = if x1 < x0 {
                    (x1, x0 - x1)
                } else {
                    (x0, x1 - x0)
                };
                let mut e = div()
                    .absolute()
                    .left(px(l - 0.75))
                    .top(px(bus - 0.75))
                    .w(px(w + 1.5))
                    .h(px(y - bus + 0.75))
                    .border_color(col);
                if w < 1.0 {
                    e = e.border_l(px(1.5));
                } else if x1 < x0 {
                    e = e.border_t(px(1.5)).border_l(px(1.5)).rounded_tl(px(6.));
                } else {
                    e = e.border_t(px(1.5)).border_r(px(1.5)).rounded_tr(px(6.));
                }
                tree = tree.child(e);
            } else {
                let x0 = par.4 + 19.0;
                let y0 = par.5 + par.6 + 7.5;
                let ye = y + 21.0;
                tree = tree.child(
                    div()
                        .absolute()
                        .left(px(x0 - 0.75))
                        .top(px(y0))
                        .w(px(x - x0 + 0.75))
                        .h(px(ye - y0 + 0.75))
                        .border_l(px(1.5))
                        .border_b(px(1.5))
                        .rounded_bl(px(6.))
                        .border_color(col),
                );
            }
        }
        for (i, (card, depth, _, colour, x, y, h)) in placed.iter().enumerate() {
            let ac = p.colors[*colour];
            let helper = *depth >= 2;
            let ctx = clamp_context(card.context);
            let bar = match context_tone(ctx) {
                "hot" => p.red,
                "warn" => p.colors[5],
                _ => ac,
            };
            let f = self.act.clone();
            let id = CARD_ACTS[i.min(CARD_ACTS.len() - 1)];
            let mut name = div().flex_1().flex().flex_col().child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(card.name.clone()),
            );
            if !helper && !card.model.is_empty() {
                name = name.child(
                    div()
                        .text_size(px(11.))
                        .text_color(p.mute)
                        .child(card.model.clone()),
                );
            }
            let mut body = div()
                .id(SharedString::from(format!("acw-{i}")))
                .absolute()
                .left(px(*x))
                .top(px(*y))
                .w(px(CW))
                .h(px(*h))
                .pt(px(11.))
                .px(px(12.))
                .flex()
                .flex_col()
                .rounded(px(if self.classic { 9. } else { 10. }))
                .border_1()
                .border_color(if self.open == Some(i) {
                    alpha(ac, 0.6)
                } else {
                    p.line
                })
                .bg(p.card)
                .cursor_pointer()
                .when(self.classic, |d| d.shadow_md())
                .on_click(move |_, w, cx| {
                    if let Some(f) = &f {
                        f(id, w, cx)
                    }
                })
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap(px(9.))
                        .min_h(px(if helper { 20. } else { 30. }))
                        .child(mark(ac, card.state == CrewState::Active))
                        .child(name)
                        .child(badge(card.state, helper, p, self.classic)),
                )
                .child(
                    div()
                        .h(px(18.))
                        .mt(px(5.))
                        .text_size(px(12.5))
                        .overflow_hidden()
                        .child(card.activity.clone()),
                )
                .child(
                    div()
                        .h(px(22.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(
                            div().flex_1().h(px(3.)).rounded_full().bg(p.track).child(
                                div()
                                    .h_full()
                                    .rounded_full()
                                    .bg(bar)
                                    .w(gpui::relative(ctx.max(0.02))),
                            ),
                        )
                        .child(
                            div()
                                .text_size(px(11.))
                                .text_color(if bar == ac { p.mute } else { bar })
                                .child(tokens_label(card)),
                        ),
                );
            if !helper {
                let mut ft = div()
                    .h(px(22.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .when(self.classic, |d| d.bg(p.band).mx(px(-12.)).px(px(12.)));
                let mut tools = div().flex().gap(px(3.));
                for (b, _) in card.tools.iter().take(4) {
                    tools = tools.children(brand_mark(b, 12., p.fg));
                }
                ft = ft.child(tools);
                if let Some((from, text)) = &card.last {
                    ft = ft.child(
                        div()
                            .flex_1()
                            .flex()
                            .gap(px(4.))
                            .text_size(px(11.))
                            .overflow_hidden()
                            .child(div().text_color(p.mute).child("↳"))
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(from.clone()))
                            .child(div().text_color(p.mute).child(text.clone())),
                    );
                }
                body = body.child(ft);
            }
            tree = tree.child(body);
        }
        if let Some(i) = self.open
            && let Some((card, depth, _, _, x, y, h)) = placed.get(i)
        {
            let _ = depth;
            let ran = running_ms(card, self.elapsed);
            let mut steer = div().flex().gap(px(5.));
            for a in card.state.steering() {
                let primary = matches!(*a, "approve" | "resume" | "retry");
                let f = self.act.clone();
                let act: &'static str = match *a {
                    "message" => "steer-message",
                    "pause" => "steer-pause",
                    "resume" => "steer-resume",
                    "stop" => "steer-stop",
                    "approve" => "steer-approve",
                    "decline" => "steer-decline",
                    _ => "steer-retry",
                };
                steer = steer.child(
                    div()
                        .id(SharedString::from(format!("acw-s-{a}")))
                        .flex_1()
                        .h(px(26.))
                        .rounded(px(7.))
                        .border_1()
                        .border_color(if primary { p.ink } else { p.line })
                        .bg(if primary { p.ink } else { p.card })
                        .text_color(if primary { p.ink_fg } else { p.fg })
                        .text_size(px(12.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .cursor_pointer()
                        .on_click(move |_, w, cx| {
                            if let Some(f) = &f {
                                f(act, w, cx)
                            }
                        })
                        .child(steer_label(a)),
                );
            }
            let stat = |k: &str, v: String| {
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .text_size(px(10.5))
                            .text_color(p.mute)
                            .child(k.to_string()),
                    )
                    .child(
                        div()
                            .text_size(px(12.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(v),
                    )
            };
            let f = self.act.clone();
            let dlg = div()
                .absolute()
                .left(px(*x))
                .top(px(y + h + 6.))
                .w(px(CW))
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(9.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .shadow_lg()
                .child(
                    div()
                        .flex()
                        .items_center()
                        .child(
                            div()
                                .flex_1()
                                .text_size(px(14.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(card.name.clone()),
                        )
                        .child(
                            div()
                                .id("acw-close")
                                .text_color(p.mute)
                                .cursor_pointer()
                                .on_click(move |_, w, cx| {
                                    if let Some(f) = &f {
                                        f("close", w, cx)
                                    }
                                })
                                .child("×"),
                        ),
                )
                .child(div().text_size(px(12.5)).child(card.task.clone()))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(5.))
                        .text_size(px(11.5))
                        .children(brand_mark(&card.model_brand, 12., p.fg))
                        .child(card.model.clone()),
                )
                .child(
                    div()
                        .flex()
                        .py(px(7.))
                        .border_t_1()
                        .border_b_1()
                        .border_color(p.line)
                        .child(stat(
                            if card.state == CrewState::Active {
                                "Running"
                            } else {
                                "Ran"
                            },
                            crew_duration(ran),
                        ))
                        .child(stat("Cost", crew_cost(card.cost)))
                        .child(stat("Tokens", short_tokens(card.tokens()))),
                )
                .child(steer);
            tree = tree.child(dlg);
        }
        div()
            .w(px(WIDE))
            .text_color(p.fg)
            .text_size(px(13.))
            .child(head)
            .child(tree)
    }
}
