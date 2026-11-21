//! affectUI fidelity pass, overnight batch, family B part 4 (plan #32) for GPUI: the exact
//! **Sequence diagram** ([`SequenceDiagramEl`]) over `oxyn_core::kit::seqfid`. The element is
//! `RenderOnce`: the host passes the participants and the messages in so far and routes [`Act`]
//! ids back (`part-<n>` for the n-th participant chip, `msg-<n>` for the n-th message tag). A
//! static snapshot: arrows do not draw themselves, open bars do not shimmer, no cards.

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, Styled, Window, div,
    prelude::*, px, rgb,
};
use oxyn_core::kit::seqfid::{
    SEQ_ROW, SeqKind, SeqMessage, SeqParticipant, SeqStatus, seq_bars, seq_demo_messages,
    seq_demo_participants, seq_frames, seq_height, seq_rows, seq_status, seq_time,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c};
use crate::brand_mark;

const W: f32 = 800.0;
const GUT: f32 = 40.0;
const PART: [&str; 8] = [
    "part-0", "part-1", "part-2", "part-3", "part-4", "part-5", "part-6", "part-7",
];
const MSG: [&str; 24] = [
    "msg-0", "msg-1", "msg-2", "msg-3", "msg-4", "msg-5", "msg-6", "msg-7", "msg-8", "msg-9",
    "msg-10", "msg-11", "msg-12", "msg-13", "msg-14", "msg-15", "msg-16", "msg-17", "msg-18",
    "msg-19", "msg-20", "msg-21", "msg-22", "msg-23",
];

#[derive(Clone, Copy)]
struct Pal {
    bg: Hsla,
    card: Hsla,
    bd: Hsla,
    life: Hsla,
    fg: Hsla,
    fg2: Hsla,
    mute: Hsla,
    faint: Hsla,
    sub: Hsla,
    red: Hsla,
    amber: Hsla,
    blue: Hsla,
    purple: Hsla,
    cyan: Hsla,
    green: Hsla,
    orange: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            bg: h(0x1A1A1A),
            card: h(0x232323),
            bd: h(0x333336),
            life: h(0x3A3A3E),
            fg: h(0xF4F4F5),
            fg2: h(0xD4D4D8),
            mute: h(0x8E8E93),
            faint: h(0x5C5C61),
            sub: h(0x1F1F21),
            red: h(0xF87171),
            amber: h(0xF5B454),
            blue: h(0x5EA2F0),
            purple: h(0xC084FC),
            cyan: h(0x38BDD8),
            green: h(0x4ADE80),
            orange: h(0xF2A33A),
        }
    } else {
        Pal {
            bg: h(0xFAFAFA),
            card: h(0xFFFFFF),
            bd: h(0xE7E7EA),
            life: h(0xD6D6DA),
            fg: h(0x18181B),
            fg2: h(0x3F3F46),
            mute: h(0x8A8A8E),
            faint: h(0xB4B4B8),
            sub: h(0xF7F7F8),
            red: h(0xD14343),
            amber: h(0xC27A0E),
            blue: h(0x2F7DD1),
            purple: h(0x9B51C9),
            cyan: h(0x1A9DB8),
            green: h(0x1E9E5A),
            orange: h(0xD9822B),
        }
    }
}

impl Pal {
    fn tint(&self, name: &str) -> Hsla {
        match name {
            "blue" => self.blue,
            "purple" => self.purple,
            "cyan" => self.cyan,
            "green" => self.green,
            "orange" => self.orange,
            _ => self.fg,
        }
    }
}

/// `agent-sequence` (affectUI Agent Sequence): one run as a sequence diagram. The host passes
/// the participants and the messages that have arrived.
#[derive(IntoElement)]
pub struct SequenceDiagramEl {
    participants: Vec<SeqParticipant>,
    messages: Vec<SeqMessage>,
    title: String,
    running: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for SequenceDiagramEl {
    fn default() -> Self {
        Self {
            participants: seq_demo_participants(),
            messages: seq_demo_messages(),
            title: "Fix the failing sign-up".into(),
            running: false,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl SequenceDiagramEl {
    /// Your own run: its participants (up to eight) and the messages so far (up to 24 shown).
    pub fn run(mut self, participants: Vec<SeqParticipant>, messages: Vec<SeqMessage>) -> Self {
        self.participants = participants;
        self.messages = messages;
        self
    }
    /// The header's name.
    pub fn title(mut self, t: &str) -> Self {
        self.title = t.into();
        self
    }
    /// Still running: the header counts who is working.
    pub fn running(mut self, on: bool) -> Self {
        self.running = on;
        self
    }
    /// Classic style: a white card with a border.
    pub fn classic(mut self, on: bool) -> Self {
        self.classic = on;
        self
    }
    /// Theme.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Chip and tag clicks.
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

/// The 3 × 3 pixel mark (full for a person) or a tool's logo.
fn mark(p: &Pal, part: &SeqParticipant, i: usize) -> AnyElement {
    if part.kind == SeqKind::Tool
        && let Some(m) = brand_mark(&part.brand, 13.0, p.fg)
    {
        return m;
    }
    let on: [bool; 9] = match (part.kind, i % 3) {
        (SeqKind::Person, _) => [true; 9],
        (_, 0) => [true, false, true, false, true, false, true, false, true],
        (_, 1) => [true, true, false, true, false, true, false, true, true],
        _ => [false, true, true, true, true, false, true, false, true],
    };
    let col = p.tint(&part.color);
    let mut g = div().flex_none().flex().flex_col().gap(px(1.));
    for r in 0..3 {
        let mut row = div().flex().gap(px(1.));
        for k in 0..3 {
            row = row.child(div().size(px(3.)).rounded(px(0.5)).bg(if on[r * 3 + k] {
                col
            } else {
                alpha(col, 0.22)
            }));
        }
        g = g.child(row);
    }
    g.into_any_element()
}

impl RenderOnce for SequenceDiagramEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let parts = &self.participants;
        let cols = parts.len().clamp(1, 8);
        let msgs: Vec<SeqMessage> = self
            .messages
            .iter()
            .filter(|m| parts.iter().any(|x| x.id == m.from) && parts.iter().any(|x| x.id == m.to))
            .take(MSG.len())
            .cloned()
            .collect();
        let col_of = |id: &str| {
            parts
                .iter()
                .position(|x| x.id == id)
                .unwrap_or(0)
                .min(cols - 1)
        };
        let area = W - 40.0 - GUT;
        let cx = |i: usize| (i as f32 + 0.5) / cols as f32 * area;
        let ys = seq_rows(&msgs);
        let h = seq_height(&msgs).max(150.0);
        let (working, failed) = seq_status(&msgs);
        let elapsed = msgs.last().map_or(0, |m| m.at);
        let st = |t: String, col: Hsla| div().text_color(col).child(t);

        let mut head = div()
            .flex()
            .items_center()
            .gap(px(10.))
            .h(px(28.))
            .text_size(px(12.))
            .child(
                div()
                    .flex_1()
                    .text_size(px(13.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.fg)
                    .child(self.title.clone()),
            );
        if failed > 0 {
            let s = if failed > 1 { "s" } else { "" };
            head = head.child(st(format!("{failed} failure{s}"), p.red));
        } else if working > 0 && self.running {
            head = head.child(st(format!("{working} working"), p.blue));
        }
        let s = if msgs.len() == 1 { "" } else { "s" };
        head = head
            .child(st(format!("{} message{s}", msgs.len()), p.mute))
            .child(st(seq_time(elapsed), p.mute));

        // Chips, centred on their lifelines.
        let mut chips = div().relative().h(px(40.)).ml(px(GUT));
        for (i, part) in parts.iter().take(cols).enumerate() {
            chips = chips.child(
                div()
                    .id(PART[i])
                    .absolute()
                    .top(px(4.))
                    .left(px(cx(i) - 56.0))
                    .w(px(112.))
                    .flex()
                    .justify_center()
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(6.))
                            .h(px(26.))
                            .px(px(9.))
                            .rounded(px(if self.classic { 13. } else { 7. }))
                            .border_1()
                            .border_color(p.bd)
                            .bg(p.card)
                            .text_size(px(12.5))
                            .font_weight(FontWeight::MEDIUM)
                            .text_color(p.fg)
                            .child(mark(&p, part, i))
                            .child(part.name.clone()),
                    )
                    .on_click({
                        let f = fire(&self.act, PART[i]);
                        move |_, w, cx| f(w, cx)
                    }),
            );
        }

        // The diagram.
        let mut dia = div().relative().h(px(h)).ml(px(GUT));
        for i in 0..cols {
            dia = dia.child(
                div()
                    .absolute()
                    .top(px(-6.))
                    .bottom_0()
                    .left(px(cx(i)))
                    .w(px(0.))
                    .border_l_1()
                    .border_dashed()
                    .border_color(p.life),
            );
        }
        for (tag, a, b) in seq_frames(&msgs) {
            let span: Vec<usize> = msgs[a..=b]
                .iter()
                .flat_map(|m| [col_of(&m.from), col_of(&m.to)])
                .collect();
            let lo = cx(*span.iter().min().unwrap_or(&0)) - 18.0;
            let hi = cx(*span.iter().max().unwrap_or(&0)) + 18.0;
            let top = ys[a] - SEQ_ROW / 2.0 - 24.0;
            let bot = ys[b] + SEQ_ROW / 2.0 - 4.0;
            dia = dia.child(
                div()
                    .absolute()
                    .left(px(lo))
                    .w(px(hi - lo))
                    .top(px(top))
                    .h(px(bot - top))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(p.bd)
                    .bg(if self.classic { p.sub } else { p.card })
                    .child(
                        div()
                            .pl(px(8.))
                            .pt(px(4.))
                            .text_size(px(10.))
                            .text_color(p.mute)
                            .child(tag.to_uppercase()),
                    ),
            );
        }
        for (pid, a, b) in seq_bars(&msgs) {
            let k = col_of(&pid);
            let top = ys[a];
            let end = b.map_or(h - 8.0, |b| ys[b]);
            dia = dia.child(
                div()
                    .absolute()
                    .left(px(cx(k) - 2.5))
                    .top(px(top))
                    .w(px(5.))
                    .h(px((end - top).max(8.0)))
                    .rounded(px(3.))
                    .bg(alpha(p.tint(&parts[k].color), 0.32)),
            );
        }
        let mut gutter = div()
            .absolute()
            .left(px(-GUT))
            .top_0()
            .w(px(GUT - 10.))
            .h(px(h));
        for (i, m) in msgs.iter().enumerate() {
            let y = ys[i];
            gutter = gutter.child(
                div()
                    .absolute()
                    .right_0()
                    .top(px(y - 7.0))
                    .text_size(px(11.))
                    .text_color(p.faint)
                    .child(seq_time(m.at)),
            );
            let (a, b) = (col_of(&m.from), col_of(&m.to));
            let col = match m.status {
                SeqStatus::Failed => p.red,
                SeqStatus::Waiting => p.amber,
                SeqStatus::Ok => p.tint(&parts[a].color),
            };
            let ink = parts[a].color == "ink" && m.status == SeqStatus::Ok;
            let (tag_bg, tag_fg) = if ink {
                (alpha(p.fg, 0.07), p.fg2)
            } else {
                (alpha(col, 0.12), col)
            };
            let tag_w = (m.text.chars().count() as f32 * 6.6 + 12.0).min(area / 3.0);
            if a == b {
                dia = dia.child(
                    div()
                        .absolute()
                        .left(px(cx(a)))
                        .top(px(y - 8.0))
                        .w(px(18.))
                        .h(px(20.))
                        .border_1()
                        .border_l_0()
                        .rounded_r(px(6.))
                        .border_color(col),
                );
            } else {
                let (l, r) = (cx(a.min(b)), cx(a.max(b)));
                let mut line = div()
                    .absolute()
                    .left(px(l))
                    .w(px(r - l))
                    .top(px(y))
                    .h(px(0.))
                    .border_t_1()
                    .border_color(col);
                if m.is_reply() {
                    line = line.border_dashed();
                }
                dia = dia.child(line);
                if m.status == SeqStatus::Failed {
                    dia = dia.child(
                        div()
                            .absolute()
                            .left(px(cx(b) - 4.0))
                            .top(px(y - 7.0))
                            .text_size(px(10.))
                            .font_weight(FontWeight::BOLD)
                            .text_color(p.red)
                            .child("✕"),
                    );
                }
            }
            let tag_left = if a == b {
                cx(a) + 24.0
            } else {
                (cx(a) + cx(b)) / 2.0 - tag_w / 2.0
            };
            let tag_top = if a == b { y - 9.0 } else { y - 22.0 };
            dia = dia.child(
                div()
                    .id(MSG[i])
                    .absolute()
                    .left(px(tag_left))
                    .top(px(tag_top))
                    .w(px(tag_w))
                    .h(px(19.))
                    .px(px(6.))
                    .rounded(px(if self.classic { 10. } else { 5. }))
                    .bg(tag_bg)
                    .text_color(tag_fg)
                    .text_size(px(12.))
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(m.text.clone())
                    .on_click({
                        let f = fire(&self.act, MSG[i]);
                        move |_, w, cx| f(w, cx)
                    }),
            );
        }
        if msgs.is_empty() {
            dia = dia.child(
                div()
                    .absolute()
                    .left_0()
                    .right_0()
                    .top(px(40.))
                    .flex()
                    .justify_center()
                    .text_color(p.mute)
                    .child("No messages yet"),
            );
        }
        dia = dia.child(gutter);

        div()
            .w(px(W))
            .max_w_full()
            .flex()
            .flex_col()
            .px(px(20.))
            .pt(px(20.))
            .pb(px(16.))
            .rounded(px(if self.classic { 16. } else { 14. }))
            .border_1()
            .border_color(if self.classic { p.bd } else { alpha(p.bd, 0.0) })
            .bg(if self.classic { p.card } else { p.bg })
            .text_size(px(12.5))
            .text_color(p.fg)
            .overflow_hidden()
            .child(head)
            .child(div().h(px(12.)))
            .child(chips)
            .child(dia)
    }
}
