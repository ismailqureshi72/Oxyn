//! affectUI fidelity pass, overnight batch, family B part 3 (plan #32) for GPUI: the exact
//! **Agent Handoff** ([`AgentHandoffEl`]) over `oxyn_core::parity::handofffid`. The element is
//! `RenderOnce`: the host keeps the clock, the open card and the relay, and routes [`Act`] ids
//! back (`card-<n>` toggles handoff n of the scene, `restore-<n>` adds back the open card's lost
//! item n). A static snapshot: the light along the lines and dragging are web-only.

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, rgb,
};
use oxyn_core::parity::handofffid::{
    HandoffScene, TaskStatus, handoff_at, handoff_cost, handoff_status, items_line,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c};
use crate::brand_mark;

const CARD_W: f32 = 252.0;
const GAP: f32 = 16.0;
const CARD_ACTS: [&str; 12] = [
    "card-0", "card-1", "card-2", "card-3", "card-4", "card-5", "card-6", "card-7", "card-8",
    "card-9", "card-10", "card-11",
];
const RESTORE_ACTS: [&str; 6] = [
    "restore-0",
    "restore-1",
    "restore-2",
    "restore-3",
    "restore-4",
    "restore-5",
];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    line: Hsla,
    fg: Hsla,
    mute: Hsla,
    faint: Hsla,
    chip: Hsla,
    wire: Hsla,
    blue: Hsla,
    blue_bg: Hsla,
    green: Hsla,
    amber: Hsla,
    red: Hsla,
    red_bg: Hsla,
    ink: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x18181B),
            line: h(0x2A2A2F),
            fg: h(0xF4F4F5),
            mute: h(0x9A9AA3),
            faint: h(0x6B6B73),
            chip: h(0x222226),
            wire: h(0x4D93D9),
            blue: h(0x6AA5F0),
            blue_bg: h(0x1A2A40),
            green: h(0x43B672),
            amber: h(0xF2B233),
            red: h(0xF06A6E),
            red_bg: h(0x3A1C1E),
            ink: h(0x0B0B0D),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            line: h(0xECECEF),
            fg: h(0x18181B),
            mute: h(0x71717A),
            faint: h(0xA1A1AA),
            chip: h(0xF4F4F5),
            wire: h(0x7DB6EA),
            blue: h(0x2F7DD1),
            blue_bg: h(0xE8F1FD),
            green: h(0x2FA35C),
            amber: h(0xE5A00D),
            red: h(0xD9474B),
            red_bg: h(0xFDECEC),
            ink: h(0x17171A),
        }
    }
}

/// `agent-handoff` (affectUI Agent Handoff): the host passes the relay (the demo's at some ms, or
/// its own) and the open handoff card; acts `card-<n>` and `restore-<n>`.
#[derive(IntoElement)]
pub struct AgentHandoffEl {
    scene: HandoffScene,
    elapsed: u64,
    open: Option<String>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AgentHandoffEl {
    fn default() -> Self {
        Self {
            scene: handoff_at(18_000),
            elapsed: 18_000,
            open: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AgentHandoffEl {
    /// The demo relay `ms` into its pass.
    pub fn demo(mut self, ms: u64) -> Self {
        self.scene = handoff_at(ms);
        self.elapsed = ms;
        self
    }
    /// Your own relay, `ms` on its clock.
    pub fn scene(mut self, scene: HandoffScene, ms: u64) -> Self {
        self.scene = scene;
        self.elapsed = ms;
        self
    }
    /// The open handoff card (its id).
    pub fn open(mut self, id: Option<&str>) -> Self {
        self.open = id.map(str::to_string);
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

fn logo(name: &str, size: f32, p: Pal) -> AnyElement {
    brand_mark(name, size, p.fg).unwrap_or_else(|| {
        div()
            .size(px(size))
            .rounded(px(3.))
            .bg(p.chip)
            .into_any_element()
    })
}

impl RenderOnce for AgentHandoffEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let sc = &self.scene;
        let n = sc.agents.len();
        let mut agents = div().flex().gap(px(GAP));
        for (i, a) in sc.agents.iter().enumerate() {
            let task = sc.agent_task(i);
            let working = task
                .as_ref()
                .is_some_and(|(t, up, _)| !up && t.status == TaskStatus::Working);
            let chip = match task {
                Some((t, up, more)) => div()
                    .h(px(32.))
                    .px(px(9.))
                    .rounded(px(8.))
                    .bg(p.chip)
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(px(12.5))
                    .when(up, |d| d.child(div().text_color(p.faint).child("Up next")))
                    .child(logo(&t.logo, 13.0, p))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(t.title.clone()),
                    )
                    .when(!up && !t.note.is_empty(), |d| {
                        d.child(div().text_color(p.mute).child(format!("· {}", t.note)))
                    })
                    .when(up && more > 0, |d| {
                        d.child(div().flex_1())
                            .child(div().text_color(p.mute).child(format!("+{more}")))
                    }),
                None => div()
                    .h(px(32.))
                    .px(px(9.))
                    .rounded(px(8.))
                    .bg(p.chip)
                    .flex()
                    .items_center()
                    .text_size(px(12.5))
                    .text_color(p.faint)
                    .child("Ready for a task"),
            };
            agents = agents.child(
                div()
                    .w(px(CARD_W))
                    .p(px(12.))
                    .rounded(px(if self.classic { 11. } else { 14. }))
                    .border_1()
                    .border_color(if self.classic {
                        alpha(p.mute, 0.3)
                    } else {
                        p.line
                    })
                    .bg(p.card)
                    .when(!self.classic, |d| d.shadow_sm())
                    .flex()
                    .flex_col()
                    .gap(px(9.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(9.))
                            .child(
                                div()
                                    .size(px(28.))
                                    .rounded(px(8.))
                                    .border_1()
                                    .border_color(p.line)
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child(logo(&a.brand, 15.0, p)),
                            )
                            .child(
                                div()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_size(px(13.5))
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .child(a.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(11.5))
                                            .text_color(p.mute)
                                            .child(a.model.clone()),
                                    ),
                            )
                            .child(div().flex_1())
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .text_color(p.mute)
                                    .child(handoff_cost(a.cost)),
                            )
                            .child(
                                div()
                                    .h(px(20.))
                                    .px(px(7.))
                                    .rounded(px(6.))
                                    .bg(if working { p.blue_bg } else { p.chip })
                                    .text_color(if working { p.blue } else { p.mute })
                                    .text_size(px(11.5))
                                    .flex()
                                    .items_center()
                                    .child(if working { "◌ Working" } else { "Idle" }),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(p.mute)
                            .child(a.description.clone()),
                    )
                    .child(chip),
            );
        }
        // Wires: a straight drop from each agent and a bus into each column.
        let mut wires = div()
            .relative()
            .h(px(60.))
            .w(px(n as f32 * CARD_W + (n.saturating_sub(1)) as f32 * GAP));
        for j in 0..n.saturating_sub(1) {
            let a0 = j as f32 * (CARD_W + GAP) + CARD_W / 2.0;
            let a1 = a0 + CARD_W + GAP;
            let cx = (a0 + a1) / 2.0;
            for (ax, ex) in [(a0, cx - 14.0), (a1, cx + 14.0)] {
                wires = wires
                    .child(
                        div()
                            .absolute()
                            .left(px(ax - 0.75))
                            .top(px(0.))
                            .w(px(1.5))
                            .h(px(30.))
                            .bg(p.wire),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(ax.min(ex)))
                            .top(px(29.25))
                            .w(px((ax - ex).abs()))
                            .h(px(1.5))
                            .bg(p.wire),
                    )
                    .child(
                        div()
                            .absolute()
                            .left(px(ex - 0.75))
                            .top(px(30.))
                            .w(px(1.5))
                            .h(px(30.))
                            .bg(p.wire),
                    );
            }
        }
        let mut cols = div().flex().gap(px(GAP)).pl(px((CARD_W + GAP) / 2.0));
        let mut k = 0usize;
        for j in 0..n.saturating_sub(1) {
            let (from, to) = (&sc.agents[j], &sc.agents[j + 1]);
            let col = sc.column(&from.id, &to.id);
            let mut list = div().w(px(CARD_W)).flex().flex_col().gap(px(6.)).child(
                div()
                    .h(px(28.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(5.))
                    .text_size(px(12.))
                    .text_color(p.mute)
                    .rounded(px(8.))
                    .when(self.classic, |d| d.bg(p.chip))
                    .child(logo(&from.brand, 12.0, p))
                    .child(from.name.clone())
                    .child("→")
                    .child(logo(&to.brand, 12.0, p))
                    .child(to.name.clone())
                    .child(div().flex_1())
                    .child(
                        div()
                            .px(px(6.))
                            .rounded(px(5.))
                            .bg(p.chip)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(col.len().to_string()),
                    ),
            );
            for h in col.iter().take(3) {
                let (kind, text) = handoff_status(h, &sc.tasks, &sc.agents, self.elapsed);
                let open = self.open.as_deref() == Some(h.id.as_str());
                let tlogo = sc
                    .tasks
                    .iter()
                    .find(|t| t.title == h.task)
                    .map(|t| t.logo.clone())
                    .unwrap_or_default();
                let (glyph, gcol) = match kind {
                    "spin" => ("◌", p.blue),
                    "warn" => ("▲", p.amber),
                    _ => ("●", p.green),
                };
                let act = CARD_ACTS[k.min(CARD_ACTS.len() - 1)];
                k += 1;
                let mut card = div()
                    .id(SharedString::from(format!("aho-{}", h.id)))
                    .px(px(11.))
                    .py(px(9.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
                    .flex()
                    .flex_col()
                    .gap(px(3.))
                    .cursor_pointer()
                    .on_click({
                        let f = fire(&self.act, act);
                        move |_, w, cx| f(w, cx)
                    })
                    .child(
                        div()
                            .flex()
                            .gap(px(7.))
                            .items_center()
                            .child(logo(&tlogo, 13.0, p))
                            .child(
                                div()
                                    .text_size(px(13.5))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(h.task.clone()),
                            ),
                    );
                if !open {
                    card = card.child(
                        div()
                            .pl(px(20.))
                            .text_size(px(12.5))
                            .text_color(p.mute)
                            .child(items_line(&h.items)),
                    );
                }
                card = card.child(
                    div()
                        .pl(px(20.))
                        .flex()
                        .gap(px(6.))
                        .text_size(px(12.5))
                        .text_color(p.mute)
                        .child(div().text_color(gcol).child(glyph))
                        .child(text),
                );
                if open {
                    for (ii, it) in h.items.iter().enumerate().take(RESTORE_ACTS.len()) {
                        let row = div()
                            .h(px(26.))
                            .px(px(8.))
                            .rounded(px(7.))
                            .flex()
                            .items_center()
                            .gap(px(7.))
                            .text_size(px(12.5));
                        card = card.child(if it.lost {
                            div()
                                .flex()
                                .flex_col()
                                .gap(px(4.))
                                .child(
                                    row.bg(p.red_bg)
                                        .text_color(p.red)
                                        .child("⊘")
                                        .child(it.label.clone()),
                                )
                                .child(
                                    div()
                                        .p(px(10.))
                                        .rounded(px(9.))
                                        .bg(p.ink)
                                        .text_color(rgb(0xF4F4F5))
                                        .flex()
                                        .flex_col()
                                        .gap(px(8.))
                                        .child(it.why.clone())
                                        .child(
                                            div()
                                                .id(SharedString::from(format!("aho-r-{}", it.id)))
                                                .px(px(10.))
                                                .h(px(26.))
                                                .rounded(px(7.))
                                                .bg(alpha(rgb(0xFFFFFF).into(), 0.12))
                                                .flex()
                                                .items_center()
                                                .cursor_pointer()
                                                .on_click({
                                                    let f = fire(&self.act, RESTORE_ACTS[ii]);
                                                    move |_, w, cx| f(w, cx)
                                                })
                                                .child("Add back"),
                                        ),
                                )
                                .into_any_element()
                        } else {
                            row.bg(p.chip).child(it.label.clone()).into_any_element()
                        });
                    }
                }
                list = list.child(card);
            }
            cols = cols.child(list);
        }
        div()
            .text_color(p.fg)
            .flex()
            .flex_col()
            .child(agents)
            .child(wires)
            .child(cols)
    }
}
