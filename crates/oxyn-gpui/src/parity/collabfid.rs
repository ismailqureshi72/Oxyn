//! affectUI fidelity pass, overnight batch, family B part 6 (plan #32) for GPUI: the exact
//! **Collaboration Stream** ([`CollabStreamEl`]) over `oxyn_core::parity::collabfid`. The element
//! is `RenderOnce`: the host keeps the messages, the picked message and the agent tab and routes
//! [`Act`] ids back (`tab-<n>`, `msg-<n>`, `chip-<n>` for the n-th message's first chip). A
//! static snapshot: no composer, no typing shimmer, no preview window.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, rgb,
};
use oxyn_core::parity::collabfid::{
    StreamAgent, StreamKind, StreamMessage, stream_demo, stream_demo_agents, stream_involves,
    stream_mark, stream_name, stream_needs_you, stream_quote, stream_time,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const W: f32 = 760.0;
const TAB: [&str; 8] = [
    "tab-0", "tab-1", "tab-2", "tab-3", "tab-4", "tab-5", "tab-6", "tab-7",
];
const MSG: [&str; 12] = [
    "msg-0", "msg-1", "msg-2", "msg-3", "msg-4", "msg-5", "msg-6", "msg-7", "msg-8", "msg-9",
    "msg-10", "msg-11",
];
const CHIP: [&str; 12] = [
    "chip-0", "chip-1", "chip-2", "chip-3", "chip-4", "chip-5", "chip-6", "chip-7", "chip-8",
    "chip-9", "chip-10", "chip-11",
];

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    bd: Hsla,
    fg: Hsla,
    body: Hsla,
    mute: Hsla,
    faint: Hsla,
    chip: Hsla,
    tab: Hsla,
    wait: (Hsla, Hsla),
    slots: [Hsla; 6],
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            card: h(0x151517),
            bd: h(0x26262A),
            fg: h(0xF4F4F5),
            body: h(0xD4D4D8),
            mute: h(0x8A8A93),
            faint: h(0x6B7280),
            chip: h(0x1E1E21),
            tab: h(0x26262A),
            wait: (h(0xF2C14E), h(0x3A2C0E)),
            slots: [
                h(0xE5E7EB),
                h(0xA78BFA),
                h(0x22D3EE),
                h(0x60A5FA),
                h(0xFBBF24),
                h(0xFB7185),
            ],
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            bd: h(0xECECEC),
            fg: h(0x111111),
            body: h(0x1F1F1F),
            mute: h(0x6B7280),
            faint: h(0xA3A3A3),
            chip: h(0xF4F4F5),
            tab: h(0xF1F1F2),
            wait: (h(0x8A5B00), h(0xFDF0D5)),
            slots: [
                h(0x1F2937),
                h(0x8B5CF6),
                h(0x06B6D4),
                h(0x3B82F6),
                h(0xF59E0B),
                h(0xF43F5E),
            ],
        }
    }
}

/// `collaboration-stream` (affectUI Collaboration Stream): the host passes the run.
#[derive(IntoElement)]
pub struct CollabStreamEl {
    agents: Vec<StreamAgent>,
    messages: Vec<StreamMessage>,
    selected: Option<String>,
    focus_agent: Option<String>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for CollabStreamEl {
    fn default() -> Self {
        Self {
            agents: stream_demo_agents(),
            messages: stream_demo(),
            selected: None,
            focus_agent: None,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl CollabStreamEl {
    /// The run's agents and messages.
    pub fn run(mut self, agents: Vec<StreamAgent>, messages: Vec<StreamMessage>) -> Self {
        self.agents = agents;
        self.messages = messages;
        self
    }
    /// The picked message.
    pub fn selected(mut self, id: Option<String>) -> Self {
        self.selected = id;
        self
    }
    /// The picked agent tab.
    pub fn focus_agent(mut self, id: Option<String>) -> Self {
        self.focus_agent = id;
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
    /// Tabs, messages and chips.
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

fn mark(p: &Pal, agents: &[StreamAgent], id: &str, size: f32) -> gpui::Div {
    let (slot, pat) = stream_mark(agents, id);
    let dot = size / 4.0;
    let mut g = div().w(px(size)).flex().flex_wrap().gap(px(dot / 3.0));
    for on in pat {
        g = g.child(div().size(px(dot)).when(on, |d| d.bg(p.slots[slot])));
    }
    g
}

impl RenderOnce for CollabStreamEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let mut tabs = div().flex().items_center().gap(px(6.));
        let mut ids: Vec<(String, String)> = vec![
            (String::new(), "Everyone".into()),
            ("you".into(), "You".into()),
        ];
        ids.extend(self.agents.iter().map(|a| (a.id.clone(), a.name.clone())));
        for (i, (id, name)) in ids.iter().enumerate().take(8) {
            let on = self.focus_agent.clone().unwrap_or_default() == *id;
            let f = fire(&self.act, TAB[i]);
            tabs = tabs.child(
                div()
                    .id(SharedString::from(format!("oxcs-tab-{i}")))
                    .h(px(26.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(7.))
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(if on { p.fg } else { p.mute })
                    .when(on, |d| d.bg(p.tab))
                    .on_click(move |_, w, cx| f(w, cx))
                    .when(!id.is_empty(), |d| d.child(mark(&p, &self.agents, id, 12.)))
                    .child(name.clone()),
            );
        }
        let mut log = div().flex().flex_col().gap(px(9.));
        for (i, m) in self.messages.iter().enumerate() {
            let dim = self
                .focus_agent
                .as_ref()
                .is_some_and(|a| !stream_involves(m, a))
                || self.selected.as_ref().is_some_and(|s| s != &m.id);
            let op = if dim { 0.35 } else { 1.0 };
            if m.kind == StreamKind::Spawn {
                log = log.child(
                    div()
                        .pl(px(34.))
                        .opacity(op)
                        .text_size(px(12.5))
                        .text_color(p.faint)
                        .child(m.text.clone()),
                );
                continue;
            }
            let needs = stream_needs_you(&self.messages, m);
            let fm = fire(&self.act, MSG[i.min(11)]);
            let mut bub = div()
                .id(SharedString::from(format!("oxcs-msg-{i}")))
                .max_w(px(560.))
                .p(px(12.))
                .flex()
                .flex_col()
                .gap(px(9.))
                .rounded(px(10.))
                .when(!self.classic, |d| {
                    d.border_1().border_color(p.bd).bg(p.card)
                })
                .when(self.classic, |d| d.bg(p.chip))
                .on_click(move |_, w, cx| fm(w, cx))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(
                            div()
                                .text_size(px(13.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(stream_name(&self.agents, &m.from)),
                        )
                        .when(needs, |d| {
                            d.child(
                                div()
                                    .px(px(6.))
                                    .rounded(px(5.))
                                    .bg(p.wait.1)
                                    .text_color(p.wait.0)
                                    .text_size(px(11.5))
                                    .child("Needs you"),
                            )
                        })
                        .when(m.kind == StreamKind::Handoff, |d| {
                            d.child(
                                div()
                                    .px(px(6.))
                                    .rounded(px(5.))
                                    .bg(p.tab)
                                    .text_size(px(11.5))
                                    .child("Handoff"),
                            )
                        })
                        .child(div().flex_1())
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.faint)
                                .child(stream_time(m.at)),
                        ),
                );
            let (to, sub) = match stream_quote(&self.messages, i) {
                Some(q) => (m.to.clone(), Some(q.text.clone())),
                None => (m.to.clone(), None),
            };
            bub = bub.child(
                div()
                    .px(px(8.))
                    .py(px(3.))
                    .rounded(px(6.))
                    .bg(p.chip)
                    .text_size(px(11.5))
                    .text_color(p.faint)
                    .flex()
                    .flex_col()
                    .child(format!("to {}", stream_name(&self.agents, &to)))
                    .children(sub.map(|s| div().text_color(p.mute).child(s))),
            );
            bub = bub.child(
                div()
                    .text_size(px(13.))
                    .text_color(p.body)
                    .child(m.text.clone()),
            );
            if let Some(a) = m.attachments.first() {
                let fc = fire(&self.act, CHIP[i.min(11)]);
                bub = bub.child(
                    div()
                        .id(SharedString::from(format!("oxcs-chip-{i}")))
                        .h(px(26.))
                        .px(px(9.))
                        .flex()
                        .items_center()
                        .rounded(px(7.))
                        .bg(p.chip)
                        .text_size(px(12.))
                        .on_click(move |_, w, cx| fc(w, cx))
                        .child(a.label.clone()),
                );
            }
            log = log.child(
                div()
                    .pl(px(10.))
                    .flex()
                    .items_start()
                    .gap(px(12.))
                    .opacity(op)
                    .child(
                        div()
                            .pt(px(16.))
                            .child(mark(&p, &self.agents, &m.from, 12.)),
                    )
                    .child(bub),
            );
        }
        div()
            .w(px(W))
            .flex()
            .flex_col()
            .gap(px(12.))
            .text_color(p.fg)
            .child(tabs.when(self.classic, |d| d.p(px(6.)).rounded(px(12.)).bg(p.tab)))
            .child(log)
    }
}
