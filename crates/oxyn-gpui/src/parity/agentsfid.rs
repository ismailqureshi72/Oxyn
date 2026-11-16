//! affectUI fidelity pass, overnight batch, family B (plan #32) for GPUI: the exact **Agent
//! Findings** ([`AgentFindingsEl`]) and **Agent Inbox** ([`AgentInboxEl`]) over
//! `oxyn_core::parity::agentsfid`. Elements are `RenderOnce`: the host keeps the clock, the filter
//! and the task states, and routes [`Act`] ids back.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::agentsfid::{
    AgentFinding, FindingAgent, FindingSeverity, INBOX_COLUMNS, InboxTask, TaskState, age, clock,
    demo_findings, demo_inbox, findings_at, inbox_summary, severity_counts,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};
use crate::brand_mark;

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    line: Hsla,
    fg: Hsla,
    mute: Hsla,
    chip: Hsla,
    blue: Hsla,
    red: Hsla,
    amber: Hsla,
    green: Hsla,
    note: Hsla,
    track: Hsla,
    col: Hsla,
    send: Hsla,
    send_fg: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x17181B),
            line: h(0x2A2C30),
            fg: h(0xF2F3F5),
            mute: h(0x9A9EA6),
            chip: h(0x2A2C31),
            blue: h(0x4C8DF6),
            red: h(0xF87171),
            amber: h(0xF5A524),
            green: h(0x4ADE80),
            note: h(0x8AA4C8),
            track: h(0x2A2C31),
            col: h(0x1C1C1F),
            send: h(0xECECEE),
            send_fg: h(0x111113),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            line: h(0xE6E6E9),
            fg: h(0x17181B),
            mute: h(0x6B7280),
            chip: h(0xEFEFF1),
            blue: h(0x3B82F6),
            red: h(0xDC2626),
            amber: h(0xB45309),
            green: h(0x16A34A),
            note: h(0x5B7AA6),
            track: h(0xE6E8EE),
            col: h(0xEFEFF1),
            send: h(0x18181B),
            send_fg: h(0xFFFFFF),
        }
    }
}

fn sev_color(s: FindingSeverity, p: Pal) -> Hsla {
    match s {
        FindingSeverity::Critical => p.red,
        FindingSeverity::Warning => p.amber,
        FindingSeverity::Note => p.note,
    }
}

const AGENT_ACTS: [&str; 8] = [
    "agent-0", "agent-1", "agent-2", "agent-3", "agent-4", "agent-5", "agent-6", "agent-7",
];

/// `agent-findings` (affectUI Agent Findings): the host passes the milliseconds since the run
/// started and the agent filter; acts: `agent-0`…`agent-7` (toggle that filter), `all`.
#[derive(IntoElement)]
pub struct AgentFindingsEl {
    agents: Vec<FindingAgent>,
    findings: Vec<AgentFinding>,
    title: String,
    rows: usize,
    elapsed: u64,
    filter: Option<usize>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AgentFindingsEl {
    fn default() -> Self {
        let (a, f) = demo_findings();
        Self {
            agents: a,
            findings: f,
            title: "Verification".into(),
            rows: 4,
            elapsed: 0,
            filter: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AgentFindingsEl {
    /// Milliseconds since the run started.
    pub fn elapsed(mut self, ms: u64) -> Self {
        self.elapsed = ms;
        self
    }
    /// Only this agent's findings.
    pub fn filter(mut self, a: Option<usize>) -> Self {
        self.filter = a;
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

impl RenderOnce for AgentFindingsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let el = self.elapsed;
        let found = findings_at(&self.findings, el);
        let done: u32 = self.agents.iter().map(|a| a.done_at(el)).sum();
        let total: u32 = self.agents.iter().map(|a| a.total).sum();
        let (crit, warn) = severity_counts(&found, None);
        let head = div()
            .h(px(48.))
            .px(px(16.))
            .flex()
            .items_center()
            .gap(px(8.))
            .border_b_1()
            .border_color(p.line)
            .child(div().text_color(p.blue).child("✦"))
            .child(
                div()
                    .text_size(px(15.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.title.clone()),
            )
            .child(
                div()
                    .text_color(p.mute)
                    .child(format!("{done} of {total} checks")),
            )
            .child(div().flex_1())
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(if crit > 0 { p.red } else { p.mute })
                    .child(format!("● {crit} critical")),
            )
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(if warn > 0 { p.amber } else { p.mute })
                    .child(format!("● {warn} warnings")),
            );
        let mut agents = div().flex().flex_col().p(px(6.));
        for (i, a) in self.agents.iter().enumerate() {
            let d = a.done_at(el);
            let (c, w) = severity_counts(&found, Some(i));
            let f = self.act.clone();
            let id = AGENT_ACTS[i.min(7)];
            let pressed = self.filter == Some(i);
            agents = agents.child(
                div()
                    .id(SharedString::from(format!("afd-{i}")))
                    .min_h(px(44.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .rounded(px(10.))
                    .when(pressed, |x| x.bg(p.chip))
                    .when(self.filter.is_some() && !pressed, |x| x.opacity(0.38))
                    .cursor_pointer()
                    .child(
                        div()
                            .size(px(24.))
                            .rounded(px(6.))
                            .bg(p.chip)
                            .flex()
                            .items_center()
                            .justify_center()
                            .children(brand_mark(&a.brand, 14., p.fg)),
                    )
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(a.name.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(p.mute)
                                    .child(a.step_at(d)),
                            ),
                    )
                    .child(
                        div().w(px(96.)).h(px(4.)).rounded_full().bg(p.track).child(
                            div()
                                .h_full()
                                .rounded_full()
                                .bg(if self.classic { p.fg } else { p.blue })
                                .w(px(96. * d as f32 / a.total.max(1) as f32)),
                        ),
                    )
                    .child(
                        div()
                            .w(px(40.))
                            .text_size(px(13.5))
                            .child(format!("{d}/{}", a.total)),
                    )
                    .child(
                        div()
                            .w(px(30.))
                            .text_size(px(12.5))
                            .text_color(if c > 0 { p.red } else { p.mute })
                            .child(format!("▪ {c}")),
                    )
                    .child(
                        div()
                            .w(px(30.))
                            .text_size(px(12.5))
                            .text_color(if w > 0 { p.amber } else { p.mute })
                            .child(format!("▪ {w}")),
                    )
                    .on_click(move |_, win, cx| {
                        if let Some(f) = &f {
                            f(id, win, cx)
                        }
                    }),
            );
        }
        let names: Vec<String> = self.agents.iter().map(|a| a.name.clone()).collect();
        let mut feed = div()
            .flex()
            .flex_col()
            .px(px(6.))
            .pt(px(4.))
            .pb(px(8.))
            .border_t_1()
            .border_color(p.line)
            .when(self.classic, |x| x.bg(p.col))
            .child(
                div()
                    .h(px(34.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.mute)
                    .child(if self.classic {
                        "Latest findings"
                    } else {
                        "LATEST FINDINGS"
                    })
                    .when_some(self.filter, |x, a| {
                        x.child(
                            div()
                                .px(px(8.))
                                .rounded_full()
                                .bg(p.chip)
                                .text_color(p.fg)
                                .child(format!("{} ×", names.get(a).cloned().unwrap_or_default())),
                        )
                    }),
            );
        for f in found
            .iter()
            .filter(|f| self.filter.is_none_or(|a| f.agent == a))
            .take(self.rows)
        {
            feed = feed.child(
                div()
                    .h(px(36.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_size(px(13.5))
                    .child(div().text_color(sev_color(f.severity, p)).child("▪"))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(f.title.clone()),
                    )
                    .child(div().text_color(p.mute).child(format!(
                        "{} · {}",
                        self.agents.get(f.agent).map(|a| a.name.clone()).unwrap_or_default(),
                        f.location
                    )))
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(px(12.5))
                            .text_color(p.mute)
                            .child(age(el.saturating_sub(f.at_ms))),
                    ),
            );
        }
        div()
            .w(px(480.))
            .max_w_full()
            .flex()
            .flex_col()
            .rounded(px(if self.classic { 16. } else { 12. }))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .text_color(p.fg)
            .text_size(px(14.))
            .overflow_hidden()
            .child(head)
            .child(agents)
            .child(feed)
    }
}

// ────────────────────────────────────────────────────────────────── Agent Inbox ──

/// `agent-inbox` (affectUI Agent Inbox): the host passes each task's state and clock seconds, the
/// layout and the open card; acts: `board`, `list`, `toggle`, `approve`, `reject`, `retry`,
/// `open`.
#[derive(IntoElement)]
pub struct AgentInboxEl {
    tasks: Vec<InboxTask>,
    clocks: Vec<u64>,
    list: bool,
    open: Option<usize>,
    title: String,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AgentInboxEl {
    fn default() -> Self {
        let t = demo_inbox();
        Self {
            clocks: t.iter().map(|x| x.elapsed_s).collect(),
            tasks: t,
            list: false,
            open: Some(0),
            title: "Agent inbox".into(),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AgentInboxEl {
    /// Tasks (each with its current state) and their clocks in seconds.
    pub fn tasks(mut self, tasks: Vec<InboxTask>, clocks: Vec<u64>) -> Self {
        self.tasks = tasks;
        self.clocks = clocks;
        self
    }
    /// List layout instead of the board, and the open card.
    pub fn layout(mut self, list: bool, open: Option<usize>) -> Self {
        self.list = list;
        self.open = open;
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

impl RenderOnce for AgentInboxEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let states: Vec<TaskState> = self.tasks.iter().map(|t| t.state).collect();
        let tone = |s: TaskState| match s {
            TaskState::Waiting => p.amber,
            TaskState::Running => p.blue,
            TaskState::Done => p.green,
            TaskState::Failed => p.red,
        };
        let card = |i: usize, t: &InboxTask| {
            let secs = self.clocks.get(i).copied().unwrap_or(t.elapsed_s);
            let open = self.open == Some(i);
            let toggle = fire(&self.act, "toggle");
            let tag = match t.state {
                TaskState::Waiting => format!("Waiting for you · {}", clock(secs)),
                TaskState::Running => clock(secs),
                TaskState::Done => clock(secs),
                TaskState::Failed => "Failed".into(),
            };
            let mut el = div()
                .id(SharedString::from(format!("aib-{i}")))
                .flex()
                .flex_col()
                .gap(px(6.))
                .p(px(14.))
                .rounded(px(14.))
                .bg(p.card)
                .when(self.classic, |x| x.border_1().border_color(p.line))
                .cursor_pointer()
                .child(
                    div()
                        .flex()
                        .gap(px(7.))
                        .text_size(px(15.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(div().text_color(tone(t.state)).child("✦"))
                        .child(t.title.clone()),
                )
                .child(
                    div()
                        .text_size(px(13.5))
                        .text_color(p.mute)
                        .when(!open, |x| x.line_clamp(2))
                        .child(t.description.clone()),
                )
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(12.5))
                        .text_color(p.mute)
                        .child(
                            div()
                                .px(px(8.))
                                .rounded_full()
                                .text_color(tone(t.state))
                                .when(t.state != TaskState::Waiting, |x| {
                                    x.bg(tone(t.state).opacity(0.14))
                                })
                                .child(tag),
                        )
                        .child(div().flex_1())
                        .child(format!("⌕ {}", t.tools))
                        .child(format!("⛓ {}", t.sources)),
                )
                .on_click(move |_, w, cx| toggle(w, cx));
            if open {
                let mut steps = div()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .pt(px(10.))
                    .border_t_1()
                    .border_color(p.line)
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(p.mute)
                            .child(t.agent.clone()),
                    );
                for (l, w) in &t.steps {
                    steps = steps.child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .text_size(px(13.5))
                            .child(div().text_color(p.mute).child("○"))
                            .child(div().flex_1().child(l.clone()))
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(p.mute)
                                    .child(w.clone()),
                            ),
                    );
                }
                let btn = |id: &'static str, label: &'static str, solid: bool| {
                    let f = fire(&self.act, id);
                    div()
                        .id(SharedString::from(format!("aib-{id}-{i}")))
                        .h(px(32.))
                        .px(px(14.))
                        .flex()
                        .items_center()
                        .rounded(px(10.))
                        .when(solid, |x| x.bg(p.send).text_color(p.send_fg))
                        .when(!solid, |x| x.border_1().border_color(p.line))
                        .text_size(px(13.5))
                        .child(label)
                        .on_click(move |_, w, cx| f(w, cx))
                };
                let acts = match t.state {
                    TaskState::Waiting => div()
                        .flex()
                        .justify_end()
                        .gap(px(8.))
                        .child(btn("reject", "Reject", false))
                        .child(btn("approve", "Approve", true)),
                    TaskState::Failed => div()
                        .flex()
                        .justify_end()
                        .child(btn("retry", "Retry", true)),
                    _ => div().flex().justify_end().child(btn("open", "Open", false)),
                };
                el = el.child(steps).child(acts);
            }
            el
        };
        let mut cols = div().flex().gap(px(10.)).when(self.list, |x| x.flex_col());
        for (ci, name) in INBOX_COLUMNS.iter().enumerate() {
            let items: Vec<usize> = (0..self.tasks.len())
                .filter(|i| states[*i].column() == ci)
                .collect();
            let mut col = div()
                .flex_1()
                .min_w(px(0.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .child(
                    div()
                        .h(px(28.))
                        .px(px(9.))
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .rounded(px(8.))
                        .when(!self.classic, |x| x.bg(p.col))
                        .text_size(px(11.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(p.mute)
                        .child(div().flex_1().child(if self.classic {
                            name.to_string()
                        } else {
                            name.to_uppercase()
                        }))
                        .child(
                            div()
                                .px(px(6.))
                                .rounded_full()
                                .bg(p.card)
                                .text_color(p.fg)
                                .child(items.len().to_string()),
                        ),
                );
            if items.is_empty() {
                col = col.child(
                    div()
                        .h(px(72.))
                        .rounded(px(12.))
                        .bg(p.col)
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(13.))
                        .text_color(p.mute)
                        .child("Nothing here"),
                );
            }
            for i in items {
                col = col.child(card(i, &self.tasks[i]));
            }
            cols = cols.child(col);
        }
        let board = fire(&self.act, "board");
        let list = fire(&self.act, "list");
        div()
            .w(px(740.))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(px(10.))
            .text_color(p.fg)
            .child(
                div()
                    .h(px(34.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(px(13.))
                    .child(div().text_color(p.mute).child("Agents /"))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.title.clone()),
                    )
                    .child(div().flex_1())
                    .child(
                        div()
                            .flex()
                            .p(px(2.))
                            .rounded(px(9.))
                            .bg(p.col)
                            .child(
                                div()
                                    .id("aib-board")
                                    .h(px(26.))
                                    .px(px(10.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(7.))
                                    .when(!self.list, |x| x.bg(p.card))
                                    .child("Board")
                                    .on_click(move |_, w, cx| board(w, cx)),
                            )
                            .child(
                                div()
                                    .id("aib-list")
                                    .h(px(26.))
                                    .px(px(10.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(7.))
                                    .when(self.list, |x| x.bg(p.card))
                                    .child("List")
                                    .on_click(move |_, w, cx| list(w, cx)),
                            ),
                    ),
            )
            .child(cols)
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(p.mute)
                    .child(inbox_summary(&states)),
            )
    }
}
