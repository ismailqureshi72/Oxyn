//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32) for GPUI: the exact
//! **Autonomy Dial** ([`AutonomyDialEl`]) over `oxyn_core::parity::autonomyfid`. The element is
//! `RenderOnce`: the host keeps the level and the locks and routes [`Act`] ids back (`level-0` …
//! `level-2`, `lock-<n>` for the n-th action). A static snapshot: no glide, the locks always show
//! on locked rows only.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::autonomyfid::{
    AUTONOMY_LEVELS, AutonomyAction, AutonomyRisk, autonomy_demo, autonomy_demo_pinned,
    autonomy_split,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};
use crate::brand_mark;

const W: f32 = 432.0;
const LEVEL: [&str; 3] = ["level-0", "level-1", "level-2"];
const LOCK: [&str; 8] = [
    "lock-0", "lock-1", "lock-2", "lock-3", "lock-4", "lock-5", "lock-6", "lock-7",
];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    bd: Hsla,
    line: Hsla,
    fg: Hsla,
    mute: Hsla,
    stop: Hsla,
    stop_fg: Hsla,
    sel: Hsla,
    sel_bd: Hsla,
    inset: Hsla,
    blue: Hsla,
    risky: (Hsla, Hsla),
    dest: (Hsla, Hsla),
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x18181A),
            bd: h(0x2A2A2D),
            line: h(0x262629),
            fg: h(0xF3F2F0),
            mute: h(0x8B8A86),
            stop: h(0x242427),
            stop_fg: h(0xA3A29E),
            sel: h(0x323236),
            sel_bd: h(0x3D3D42),
            inset: h(0x1F1F22),
            blue: h(0x5B9BF5),
            risky: (h(0xD99A3D), h(0x2A2013)),
            dest: (h(0xE5786B), h(0x2B1715)),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            bd: h(0xECECEC),
            line: h(0xEFEFEF),
            fg: h(0x0A0A0A),
            mute: h(0x858585),
            stop: h(0xF0F0F0),
            stop_fg: h(0x555555),
            sel: h(0xFFFFFF),
            sel_bd: h(0xE8E8E8),
            inset: h(0xF5F5F5),
            blue: h(0x2F7BEA),
            risky: (h(0x94600A), h(0xFFEED5)),
            dest: (h(0xC8432B), h(0xFEECE5)),
        }
    }
}

/// `autonomy-dial` (affectUI Autonomy Dial): the host passes the level and the locks.
#[derive(IntoElement)]
pub struct AutonomyDialEl {
    actions: Vec<AutonomyAction>,
    level: u8,
    pinned: Vec<String>,
    title: String,
    subtitle: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AutonomyDialEl {
    fn default() -> Self {
        Self {
            actions: autonomy_demo(),
            level: 1,
            pinned: autonomy_demo_pinned(),
            title: "Agent permissions".into(),
            subtitle: "What the agent may do on its own".into(),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AutonomyDialEl {
    /// Your own actions.
    pub fn actions(mut self, a: Vec<AutonomyAction>) -> Self {
        self.actions = a;
        self
    }
    /// The level (0 Always ask, 1 If risky, 2 Full auto).
    pub fn level(mut self, l: u8) -> Self {
        self.level = l.min(2);
        self
    }
    /// Ids that always ask.
    pub fn pinned(mut self, p: Vec<String>) -> Self {
        self.pinned = p;
        self
    }
    /// Heading and the line under it.
    pub fn title(mut self, t: &str, sub: &str) -> Self {
        self.title = t.into();
        self.subtitle = sub.into();
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
    /// Level picks and lock toggles.
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

impl RenderOnce for AutonomyDialEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let (own, ask) = autonomy_split(&self.actions, self.level, &self.pinned);
        let mut seg = div().flex().gap(px(if self.classic { 0. } else { 9. }));
        if self.classic {
            seg = seg.p(px(3.)).rounded(px(10.)).bg(p.stop);
        }
        for (i, name) in AUTONOMY_LEVELS.iter().enumerate() {
            let on = i as u8 == self.level;
            let f = fire(&self.act, LEVEL[i]);
            seg = seg.child(
                div()
                    .id(SharedString::from(format!("oxad-{i}")))
                    .flex_1()
                    .h(px(26.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(7.))
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .border_1()
                    .when(on, |d| d.bg(p.sel).border_color(p.sel_bd).text_color(p.fg))
                    .when(!on, |d| {
                        d.border_color(gpui::transparent_black())
                            .text_color(p.stop_fg)
                            .when(!self.classic, |d| d.bg(p.stop))
                    })
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(name.to_string()),
            );
        }
        let top = div()
            .flex()
            .flex_col()
            .gap(px(17.))
            .pt(px(17.))
            .px(px(16.))
            .pb(px(19.))
            .when(self.classic, |d| {
                d.bg(p.card).rounded(px(14.)).border_1().border_color(p.bd)
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(12.))
                    .child(div().size(px(10.)).rounded(px(2.)).bg(p.blue))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(2.))
                            .child(
                                div()
                                    .text_size(px(14.5))
                                    .font_weight(FontWeight::MEDIUM)
                                    .text_color(p.fg)
                                    .child(self.title.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(p.mute)
                                    .child(self.subtitle.clone()),
                            ),
                    ),
            )
            .child(seg);
        let index = |a: &AutonomyAction| {
            self.actions
                .iter()
                .position(|x| x.id == a.id)
                .unwrap_or(0)
                .min(7)
        };
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(12.))
            .pt(px(13.))
            .px(px(16.))
            .pb(px(12.))
            .when(!self.classic, |d| d.border_t_1().border_color(p.line));
        for (gi, (name, rows)) in [("Runs on its own", own), ("Asks you first", ask)]
            .into_iter()
            .enumerate()
        {
            let mut grp = div().flex().flex_col().gap(px(4.)).child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .text_size(px(12.5))
                    .text_color(p.mute)
                    .child(name)
                    .child(
                        div()
                            .px(px(4.))
                            .rounded(px(5.))
                            .border_1()
                            .border_color(p.bd)
                            .text_size(px(11.))
                            .child(rows.len().to_string()),
                    ),
            );
            if rows.is_empty() {
                grp = grp.child(
                    div()
                        .h(px(40.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(8.))
                        .bg(p.inset)
                        .text_size(px(12.5))
                        .text_color(p.mute)
                        .child(if gi == 0 {
                            "Every action waits for you"
                        } else {
                            "Nothing waits for you"
                        }),
                );
            }
            for a in rows.iter() {
                let locked = self.pinned.contains(&a.id);
                let f = fire(&self.act, LOCK[index(a)]);
                let tag = match a.risk {
                    AutonomyRisk::Safe => None,
                    AutonomyRisk::Risky => Some(p.risky),
                    AutonomyRisk::Destructive => Some(p.dest),
                };
                let mut row = div()
                    .h(px(if self.classic { 42. } else { 32.6 }))
                    .flex()
                    .items_center()
                    .gap(px(14.))
                    .when(self.classic, |d| {
                        d.px(px(10.))
                            .rounded(px(10.))
                            .border_1()
                            .border_color(p.bd)
                            .bg(p.card)
                    });
                row = row.children(brand_mark(&a.brand, 16.0, p.fg));
                row = row.child(
                    div()
                        .flex_1()
                        .flex()
                        .items_center()
                        .gap(px(9.))
                        .child(
                            div()
                                .text_size(px(14.))
                                .text_color(p.fg)
                                .child(a.label.clone()),
                        )
                        .children(tag.map(|(fg, bg)| {
                            div()
                                .h(px(20.))
                                .px(px(6.))
                                .flex()
                                .items_center()
                                .rounded(px(5.))
                                .bg(bg)
                                .text_color(fg)
                                .text_size(px(12.))
                                .font_weight(FontWeight::MEDIUM)
                                .child(a.risk.tag())
                        })),
                );
                row = row.child(
                    div()
                        .id(SharedString::from(format!("oxad-lock-{}", a.id)))
                        .size(px(28.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(7.))
                        .text_size(px(12.))
                        .text_color(p.mute)
                        .on_click(move |_, w, cx| f(w, cx))
                        .child(if locked { "🔒" } else { "" }),
                );
                grp = grp.child(row);
            }
            body = body.child(grp);
        }
        div()
            .w(px(W))
            .flex()
            .flex_col()
            .rounded(px(if self.classic { 18. } else { 12. }))
            .border_1()
            .border_color(p.bd)
            .bg(if self.classic { p.inset } else { p.card })
            .when(self.classic, |d| d.p(px(6.)))
            .text_color(p.fg)
            .child(top)
            .child(body)
    }
}
