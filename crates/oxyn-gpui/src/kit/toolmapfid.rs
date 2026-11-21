//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Tool Access Map**
//! ([`ToolReachEl`]) over `oxyn_core::kit::toolmapfid`. The element is `RenderOnce`: the
//! host passes the tools, the agents and the agent shown, and gets [`Act`] ids back (`agent-<n>`
//! for the tabs, `tool-<n>` for the cards). A static snapshot of the map: no wires' glint and no
//! calls view (open a tool's calls in the host).

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px, rgb,
};
use oxyn_core::kit::toolmapfid::{
    TmAgent, TmState, TmStatus, TmTool, tm_calls, tm_counts, tm_demo_agents, tm_demo_tools, tm_dur,
    tm_sides, tm_state,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const AGENT: [&str; 6] = [
    "agent-0", "agent-1", "agent-2", "agent-3", "agent-4", "agent-5",
];
const TOOL: [&str; 12] = [
    "tool-0", "tool-1", "tool-2", "tool-3", "tool-4", "tool-5", "tool-6", "tool-7", "tool-8",
    "tool-9", "tool-10", "tool-11",
];

/// `tool-access-map` (affectUI Tool Access Map): the host passes the run's tools and agents.
#[derive(IntoElement)]
pub struct ToolReachEl {
    tools: Vec<TmTool>,
    agents: Vec<TmAgent>,
    agent: usize,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ToolReachEl {
    fn default() -> Self {
        Self {
            tools: tm_demo_tools(),
            agents: tm_demo_agents(),
            agent: 0,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ToolReachEl {
    /// The run's tools and agents, and which agent shows.
    pub fn run(mut self, tools: Vec<TmTool>, agents: Vec<TmAgent>, agent: usize) -> Self {
        self.tools = tools;
        self.agents = agents;
        self.agent = agent;
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
    /// Tab and tool clicks.
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

impl RenderOnce for ToolReachEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let (card, bd, fg, mute, track, blue, red, amb) = if dark {
            (
                h(0x17181C),
                h(0x26272B),
                h(0xF4F5F7),
                h(0x8B9099),
                h(0x1F2024),
                h(0x4D8DFF),
                h(0xF87171),
                h(0xFBBF24),
            )
        } else {
            (
                h(0xFFFFFF),
                h(0xE6E6EA),
                h(0x17181C),
                h(0x8A8F98),
                h(0xF1F1F3),
                h(0x2F6FED),
                h(0xDC2626),
                h(0xB45309),
            )
        };
        let radius = if self.classic { 6. } else { 10. };
        let Some(a) = self.agents.get(self.agent).or(self.agents.first()) else {
            return div();
        };
        let (reach, total, calls, running) = tm_counts(a, &self.tools);
        let mut tabs = div()
            .flex()
            .gap(px(2.))
            .p(px(3.))
            .rounded(px(10.))
            .bg(track);
        for (i, x) in self.agents.iter().enumerate().take(6) {
            let on = x.id == a.id;
            let f = fire(&self.act, AGENT[i]);
            tabs = tabs.child(
                div()
                    .id(SharedString::from(format!("oxtm-tab-{i}")))
                    .px(px(11.))
                    .py(px(5.))
                    .rounded(px(8.))
                    .text_size(px(12.))
                    .text_color(if on { fg } else { mute })
                    .when(on, |d| d.bg(card))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(x.name.clone()),
            );
        }
        let tool_card = |i: usize, t: &TmTool| -> Div {
            let st = tm_state(a, &t.id);
            let cs = tm_calls(a, &t.id).cloned().unwrap_or_default();
            let line = match st {
                TmState::NoAccess => "No access".to_string(),
                TmState::Unused => "Not used yet".to_string(),
                TmState::Running => "Running".to_string(),
                TmState::Failed => "Failed".to_string(),
                TmState::Used => {
                    format!("Last call {}", tm_dur(cs.last().map(|c| c.ms).unwrap_or(0)))
                }
            };
            let tone = match st {
                TmState::Running => blue,
                TmState::Failed => red,
                _ => mute,
            };
            let f = fire(&self.act, TOOL[i.min(11)]);
            div().child(
                div()
                    .id(SharedString::from(format!("oxtm-tool-{i}")))
                    .h(px(46.))
                    .px(px(9.))
                    .py(px(7.))
                    .flex()
                    .flex_col()
                    .border_1()
                    .border_color(if st == TmState::Running { blue } else { bd })
                    .rounded(px(radius))
                    .bg(card)
                    .when(st == TmState::NoAccess, |d| d.opacity(0.5))
                    .when(st != TmState::NoAccess, |d| {
                        d.on_click(move |_, w, cx| f(w, cx))
                    })
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(12.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(t.name.clone()),
                            )
                            .when(!cs.is_empty(), |d| {
                                d.child(
                                    div()
                                        .text_size(px(11.))
                                        .text_color(mute)
                                        .child(format!("{}×", cs.len())),
                                )
                            }),
                    )
                    .child(div().text_size(px(10.5)).text_color(tone).child(line)),
            )
        };
        let (l, r) = tm_sides(self.tools.len());
        let mut left = div().flex_1().flex().flex_col().gap(px(8.));
        for i in l {
            left = left.child(tool_card(i, &self.tools[i]));
        }
        let mut right = div().flex_1().flex().flex_col().gap(px(8.));
        for i in r {
            right = right.child(tool_card(i, &self.tools[i]));
        }
        let badge = match a.status {
            TmStatus::Running => blue,
            TmStatus::Waiting => amb,
            TmStatus::Failed => red,
            _ => mute,
        };
        let agent = div()
            .w(px(190.))
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(3.))
            .border_1()
            .border_color(bd)
            .rounded(px(radius))
            .bg(card)
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(a.name.clone()),
            )
            .child(
                div()
                    .text_size(px(11.))
                    .text_color(mute)
                    .child(a.model.clone()),
            )
            .child(
                div()
                    .text_size(px(10.5))
                    .text_color(badge)
                    .child(a.status.label()),
            )
            .child(div().text_size(px(12.)).child(a.task.clone()));
        div()
            .w(px(720.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .text_color(fg)
            .when(self.classic, |d| {
                d.p(px(12.)).border_1().border_color(bd).rounded(px(12.))
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Tool access"),
                    )
                    .child(div().flex_1())
                    .when(running > 0, |d| {
                        d.child(
                            div()
                                .text_size(px(11.))
                                .text_color(blue)
                                .child(format!("{running} running")),
                        )
                    })
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(mute)
                            .child(format!("{reach}/{total} tools · {calls} calls")),
                    ),
            )
            .child(tabs)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(40.))
                    .child(left)
                    .child(agent)
                    .child(right),
            )
    }
}
