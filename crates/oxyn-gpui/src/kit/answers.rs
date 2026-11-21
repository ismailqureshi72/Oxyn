//! affectUI fidelity pass, overnight batch, family A (plan #32) for GPUI: the exact **AI Answer**
//! ([`AnswerCardEl`]), **Cited Answer** ([`CitedAnswerEl`]), **Message Actions**
//! ([`MessageActionsEl`]), **Thinking pill** ([`ThinkingPillEl`]), **Thinking panel**
//! ([`ThinkingPanelEl`]) and **Writing Assistant** ([`WritingAssistantEl`]) in affectUI's greys,
//! over `oxyn_core::kit::answers`. Elements are `RenderOnce`: the host keeps the state (the
//! elapsed time, what is lit, open or voted), drives the clock and routes [`Act`] ids back.

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, rgb, rgba,
};
use oxyn_core::kit::answers::{
    ANSWER_REASONS, AnswerDoc, AnswerPhase, AnswerSource, Block, CitedSource, LOADER_INTERVAL,
    LOADER_PHASES, MESSAGE_REASONS, Seg, THINK_STEP_MS, ThinkStep, answer_phase, answer_words_at,
    backed, cited_in, demo_answer, demo_cited, demo_message, demo_think_steps, loader_phase,
    parse_cited, quote_runs, reply_at, sample_reply, sentences_citing, think_progress, thought_for,
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
    tint: Hsla,
    ok: Hsla,
    okbg: Hsla,
    band: Hsla,
    you: Hsla,
    cls: Hsla,
    send: Hsla,
    send_fg: Hsla,
    off: Hsla,
    violet: Hsla,
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
            tint: h(0x25304A),
            ok: h(0x34C77B),
            okbg: h(0x16281E),
            band: h(0x1A1B1E),
            you: h(0x26272B),
            cls: h(0x1C1C1F),
            send: h(0xECECEE),
            send_fg: h(0x111113),
            off: h(0x2F2F31),
            violet: h(0x8B5CF6),
        }
    } else {
        Pal {
            card: h(0xFFFFFF),
            line: h(0xE6E6E9),
            fg: h(0x17181B),
            mute: h(0x6B7280),
            chip: h(0xEFEFF1),
            blue: h(0x3B82F6),
            tint: h(0xE7EDFB),
            ok: h(0x16A34A),
            okbg: h(0xEAF7EF),
            band: h(0xFAFAFA),
            you: h(0xF4F4F5),
            cls: h(0xF4F4F5),
            send: h(0x18181B),
            send_fg: h(0xFFFFFF),
            off: h(0xE9E9EB),
            violet: h(0x8B5CF6),
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

fn tile(brand: &str, size: f32, p: Pal) -> AnyElement {
    let t = div()
        .flex_none()
        .size(px(size))
        .rounded(px(4.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(size * 0.55))
        .font_weight(FontWeight::BOLD)
        .text_color(p.fg);
    match brand_mark(brand, size * 0.72, p.fg) {
        Some(m) => t.child(m).into_any_element(),
        None => t
            .child(
                brand
                    .chars()
                    .next()
                    .map(|c| c.to_ascii_uppercase().to_string())
                    .unwrap_or_default(),
            )
            .into_any_element(),
    }
}

fn icon_btn(
    id: &'static str,
    glyph: &'static str,
    on: bool,
    p: Pal,
    act: &Option<Act>,
) -> impl IntoElement {
    let f = fire(act, id);
    div()
        .id(SharedString::from(format!("ans-{id}")))
        .size(px(30.))
        .rounded(px(8.))
        .flex()
        .items_center()
        .justify_center()
        .text_size(px(15.))
        .text_color(if on { p.fg } else { p.mute })
        .when(on, |d| d.bg(p.chip))
        .cursor_pointer()
        .child(glyph)
        .on_click(move |_, w, cx| f(w, cx))
}

fn chip_row(labels: &[&str], p: Pal, act: &Option<Act>, prefix: &'static str) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap(px(6.))
        .children(labels.iter().enumerate().map(|(i, l)| {
            let f = fire(act, prefix);
            div()
                .id(SharedString::from(format!("{prefix}-{i}")))
                .h(px(28.))
                .px(px(11.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .border_1()
                .border_color(p.line)
                .text_size(px(13.))
                .cursor_pointer()
                .child(l.to_string())
                .on_click(move |_, w, cx| f(w, cx))
        }))
}

// ───────────────────────────────────────────────────────────────────── AI Answer ──

/// `ai-answer` (affectUI AI Answer). The host passes the milliseconds since the stream started
/// (`elapsed`) and the feedback state; acts: `copy`, `regenerate`, `up`, `down`, `stop`,
/// `reason`, `follow-up`.
#[derive(IntoElement)]
pub struct AnswerCardEl {
    question: String,
    sources: Vec<AnswerSource>,
    more: usize,
    doc: AnswerDoc,
    related: Vec<String>,
    took: String,
    elapsed: u64,
    vote: u8,
    copied: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AnswerCardEl {
    fn default() -> Self {
        let (q, s, m, t, r) = demo_answer();
        Self {
            question: q,
            sources: s,
            more: m,
            doc: AnswerDoc::parse(&t),
            related: r,
            took: "2.9s".into(),
            elapsed: u64::MAX / 4,
            vote: 0,
            copied: false,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AnswerCardEl {
    /// Milliseconds since the stream started (a large value shows it finished).
    pub fn elapsed(mut self, ms: u64) -> Self {
        self.elapsed = ms;
        self
    }
    /// 0 none, 1 thumbs up, 2 thumbs down; and whether Copy was just pressed.
    pub fn feedback(mut self, vote: u8, copied: bool) -> Self {
        self.vote = vote;
        self.copied = copied;
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

fn seg_line(segs: &[Seg], srcs: &[AnswerSource], p: Pal) -> impl IntoElement {
    div()
        .flex()
        .flex_wrap()
        .gap_x(px(4.))
        .gap_y(px(2.))
        .children(segs.iter().map(|s| {
            match s {
                Seg::Word(w) => div().child(w.clone()).into_any_element(),
                Seg::Bold(w) => div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(w.clone())
                    .into_any_element(),
                Seg::Cite(ids) => {
                    let first = ids
                        .first()
                        .and_then(|i| srcs.get(*i))
                        .map(|s| s.name.clone())
                        .unwrap_or_default();
                    let l = if ids.len() > 1 {
                        format!("{first} +{}", ids.len() - 1)
                    } else {
                        first
                    };
                    div()
                        .h(px(20.))
                        .px(px(8.))
                        .rounded_full()
                        .bg(p.chip)
                        .text_color(p.mute)
                        .text_size(px(11.5))
                        .flex()
                        .items_center()
                        .child(l)
                        .into_any_element()
                }
            }
        }))
}

impl RenderOnce for AnswerCardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let total = self.doc.words();
        let phase = answer_phase(self.elapsed, total, 38);
        let shown = self.doc.upto(answer_words_at(self.elapsed, total, 38));
        let writing = matches!(phase, AnswerPhase::Reading | AnswerPhase::Writing);
        let status = match phase {
            AnswerPhase::Reading => format!("Reading {} sources", self.sources.len() + self.more),
            AnswerPhase::Writing => "Writing".into(),
            _ => "Answer".into(),
        };
        let extra = self.more + self.sources.len().saturating_sub(3);
        let srcs = self.sources.clone();
        let card = |s: &AnswerSource, i: usize| {
            div()
                .flex_1()
                .min_w(px(0.))
                .h(px(72.))
                .p(px(10.))
                .flex()
                .flex_col()
                .gap(px(6.))
                .rounded(px(9.))
                .when(!self.classic, |d| d.border_1().border_color(p.line))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .text_size(px(12.))
                        .text_color(p.mute)
                        .child(tile(&s.brand, 16., p))
                        .child(div().flex_1().child(s.name.clone()))
                        .child((i + 1).to_string()),
                )
                .child(
                    div()
                        .text_size(px(12.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .line_clamp(2)
                        .child(s.title.clone()),
                )
        };
        let mut row = div().flex().gap(px(8.)).when(self.classic, |d| {
            d.gap(px(0.)).p(px(4.)).rounded(px(12.)).bg(p.cls)
        });
        for (i, s) in self.sources.iter().take(3).enumerate() {
            row = row.child(card(s, i));
        }
        if extra > 0 {
            row = row.child(
                div()
                    .w(px(64.))
                    .h(px(72.))
                    .px(px(10.))
                    .flex()
                    .flex_col()
                    .justify_center()
                    .rounded(px(9.))
                    .when(!self.classic, |d| d.border_1().border_color(p.line))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("+{extra}")),
                    )
                    .child(div().text_size(px(12.)).text_color(p.mute).child("more")),
            );
        }
        let mut body = div().flex().flex_col().gap(px(10.)).text_size(px(15.));
        for b in &shown.blocks {
            body = match b {
                Block::Para(s) => body.child(seg_line(s, &srcs, p)),
                Block::Bullets(items) => body.child(div().flex().flex_col().gap(px(6.)).children(
                    items.iter().map(|s| {
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(div().text_color(p.mute).child("•"))
                            .child(seg_line(s, &srcs, p))
                    }),
                )),
            };
        }
        let foot = if self.copied {
            "Copied".to_string()
        } else if self.vote == 2 {
            "What went wrong?".to_string()
        } else if self.vote == 1 {
            "Thanks for the feedback".to_string()
        } else {
            format!("Answered in {}", self.took)
        };
        let mut footer = div()
            .flex()
            .items_center()
            .gap(px(4.))
            .min_h(px(52.))
            .px(px(14.))
            .when(!self.classic, |d| d.border_t_1().border_color(p.line))
            .when(self.classic, |d| d.bg(p.cls));
        footer = if writing {
            let f = fire(&self.act, "stop");
            footer.child(
                div()
                    .id("ans-stop")
                    .h(px(30.))
                    .px(px(12.))
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .rounded_full()
                    .border_1()
                    .border_color(p.line)
                    .text_size(px(13.))
                    .child(div().size(px(9.)).rounded(px(2.)).bg(p.fg))
                    .child("Stop")
                    .on_click(move |_, w, cx| f(w, cx)),
            )
        } else {
            footer
                .child(icon_btn(
                    "copy",
                    if self.copied { "✓" } else { "⧉" },
                    false,
                    p,
                    &self.act,
                ))
                .child(icon_btn("regenerate", "↻", false, p, &self.act))
                .child(icon_btn("up", "▲", self.vote == 1, p, &self.act))
                .child(icon_btn("down", "▼", self.vote == 2, p, &self.act))
                .child(div().flex_1())
                .child(
                    div()
                        .text_size(px(12.5))
                        .text_color(if self.copied { p.ok } else { p.mute })
                        .child(foot),
                )
        };
        let mut card_el = div()
            .w(px(480.))
            .max_w_full()
            .flex()
            .flex_col()
            .gap(px(14.))
            .pt(px(20.))
            .rounded(px(if self.classic { 18. } else { 12. }))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .text_color(p.fg)
            .overflow_hidden()
            .child(
                div()
                    .px(px(20.))
                    .text_size(px(17.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.question.clone()),
            )
            .child(div().px(px(20.)).child(row))
            .child(
                div()
                    .px(px(20.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(px(13.))
                    .text_color(p.mute)
                    .child(if self.classic {
                        div()
                            .size(px(20.))
                            .rounded_full()
                            .bg(p.blue)
                            .text_color(gpui::white())
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(11.))
                            .child("✦")
                    } else {
                        div().text_color(p.blue).child("✦")
                    })
                    .child(status),
            )
            .child(div().px(px(20.)).child(body));
        if !writing && !self.related.is_empty() {
            let mut rel = div().px(px(20.)).flex().flex_col().child(
                div()
                    .pb(px(8.))
                    .text_size(px(11.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.mute)
                    .when(!self.classic, |d| d.border_b_1().border_color(p.line))
                    .child(if self.classic { "Related" } else { "RELATED" }),
            );
            for (i, r) in self.related.iter().enumerate() {
                let f = fire(&self.act, "follow-up");
                rel = rel.child(
                    div()
                        .id(SharedString::from(format!("ans-fu-{i}")))
                        .h(px(40.))
                        .flex()
                        .items_center()
                        .when(!self.classic, |d| d.border_b_1().border_color(p.line))
                        .text_size(px(14.5))
                        .cursor_pointer()
                        .child(div().flex_1().child(r.clone()))
                        .child(div().text_color(p.mute).child(if self.classic {
                            "→"
                        } else {
                            "›"
                        }))
                        .on_click(move |_, w, cx| f(w, cx)),
                );
            }
            card_el = card_el.child(rel);
        }
        card_el = card_el.child(footer);
        if !writing && self.vote == 2 {
            card_el = card_el.child(div().px(px(14.)).pb(px(14.)).child(chip_row(
                &ANSWER_REASONS,
                p,
                &self.act,
                "reason",
            )));
        }
        card_el
    }
}

// ───────────────────────────────────────────────────────────────── Cited Answer ──

/// `cited-answer` (affectUI Cited Answer). The host keeps which source is lit and which quote is
/// open; acts: `source-1`…`source-9` (toggle that quote), `close`, `copy`, `follow-up`.
#[derive(IntoElement)]
pub struct CitedAnswerEl {
    header: String,
    text: String,
    sources: Vec<CitedSource>,
    lit: Option<usize>,
    open: Option<usize>,
    copied: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for CitedAnswerEl {
    fn default() -> Self {
        let (h, t, s) = demo_cited();
        Self {
            header: h,
            text: t,
            sources: s,
            lit: Some(1),
            open: None,
            copied: false,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

const SOURCE_ACTS: [&str; 9] = [
    "source-1", "source-2", "source-3", "source-4", "source-5", "source-6", "source-7", "source-8",
    "source-9",
];

impl CitedAnswerEl {
    /// Lit source and open quote (0-based), and whether the quote was just copied.
    pub fn state(mut self, lit: Option<usize>, open: Option<usize>, copied: bool) -> Self {
        self.lit = lit;
        self.open = open;
        self.copied = copied;
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

fn letter(s: &CitedSource, p: Pal) -> impl IntoElement {
    let tint: Hsla = rgb(u32::from_str_radix(&s.tint, 16).unwrap_or(0x71717A)).into();
    div()
        .size(px(16.))
        .rounded(px(4.5))
        .bg(tint.opacity(0.16))
        .text_color(tint)
        .text_size(px(9.5))
        .font_weight(FontWeight::BOLD)
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .border_color(p.card)
        .child(s.mark.clone())
}

impl RenderOnce for CitedAnswerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let paras = parse_cited(&self.text);
        let on = self.open.or(self.lit);
        let (b, all) = backed(&paras);
        let foot = if self.copied {
            "Quote copied".to_string()
        } else if let Some(i) = on {
            self.sources
                .get(i)
                .map(|s| format!("{} · {}", s.name, cited_in(sentences_citing(&paras, i))))
                .unwrap_or_default()
        } else {
            format!("{b} of {all} sentences backed by a source")
        };
        let mut text = div().flex().flex_col().gap(px(12.)).text_size(px(15.));
        for para in &paras {
            text = text.child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_x(px(4.))
                    .gap_y(px(3.))
                    .children(para.iter().map(|s| {
                        let lit = on.is_some_and(|o| s.cites.contains(&o));
                        let dim = on.is_some() && !lit;
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(3.))
                            .child(
                                div()
                                    .when(lit, |d| {
                                        d.bg(p.tint).rounded(px(4.)).font_weight(FontWeight::MEDIUM)
                                    })
                                    .when(dim, |d| d.opacity(0.38))
                                    .child(s.text.clone()),
                            )
                            .children(s.cites.iter().map(|c| {
                                let lit_m = on == Some(*c);
                                div()
                                    .min_w(px(17.))
                                    .h(px(17.))
                                    .px(px(3.))
                                    .rounded(px(5.))
                                    .bg(if lit_m { p.blue } else { p.chip })
                                    .text_color(if lit_m { gpui::white() } else { p.mute })
                                    .text_size(px(10.5))
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .child((c + 1).to_string())
                            }))
                    })),
            );
        }
        let chips = div()
            .flex()
            .gap(px(8.))
            .px(px(20.))
            .pt(px(6.))
            .pb(px(14.))
            .children(self.sources.iter().enumerate().map(|(i, s)| {
                let f = fire(&self.act, SOURCE_ACTS[i.min(8)]);
                let lit = on == Some(i);
                div()
                    .id(SharedString::from(format!("cited-{i}")))
                    .h(px(32.))
                    .px(px(9.))
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .rounded(px(9.))
                    .border_1()
                    .border_color(if self.open == Some(i) { p.blue } else { p.line })
                    .when(lit, |d| d.bg(p.chip))
                    .text_size(px(13.))
                    .text_color(if lit { p.fg } else { p.mute })
                    .cursor_pointer()
                    .child(letter(s, p))
                    .child(s.name.clone())
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(p.mute)
                            .child((i + 1).to_string()),
                    )
                    .on_click(move |_, w, cx| f(w, cx))
            }));
        let quote = self
            .open
            .and_then(|i| self.sources.get(i).cloned().map(|s| (i, s)))
            .map(|(i, s)| {
                let close = fire(&self.act, "close");
                let copy = fire(&self.act, "copy");
                div()
                    .mx(px(20.))
                    .mb(px(14.))
                    .p(px(14.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .text_size(px(12.5))
                            .text_color(p.mute)
                            .child(letter(&s, p))
                            .child(div().flex_1().child(format!("{} · {}", s.domain, s.date)))
                            .child(
                                div()
                                    .px(px(6.))
                                    .rounded(px(6.))
                                    .bg(p.chip)
                                    .text_color(p.fg)
                                    .child((i + 1).to_string()),
                            )
                            .child(
                                div()
                                    .id("cited-close")
                                    .px(px(6.))
                                    .cursor_pointer()
                                    .child("×")
                                    .on_click(move |_, w, cx| close(w, cx)),
                            ),
                    )
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(s.title.clone()),
                    )
                    .child(
                        div()
                            .pl(px(12.))
                            .border_l_2()
                            .border_color(p.line)
                            .text_size(px(14.))
                            .text_color(p.mute)
                            .flex()
                            .flex_wrap()
                            .children(quote_runs(&s.quote, &s.marked).into_iter().map(|(t, m)| {
                                div()
                                    .when(m, |d| {
                                        d.bg(p.tint)
                                            .text_color(p.fg)
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .rounded(px(3.))
                                    })
                                    .child(t)
                            })),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .text_size(px(13.))
                            .child(
                                div()
                                    .h(px(32.))
                                    .px(px(12.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(8.))
                                    .border_1()
                                    .border_color(p.line)
                                    .child("↗ Open source"),
                            )
                            .child(
                                div()
                                    .id("cited-copy")
                                    .h(px(32.))
                                    .px(px(12.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(8.))
                                    .when(self.copied, |d| d.bg(p.okbg).text_color(p.ok))
                                    .when(!self.copied, |d| d.text_color(p.mute))
                                    .cursor_pointer()
                                    .child(if self.copied {
                                        "✓ Copied"
                                    } else {
                                        "⧉ Copy quote"
                                    })
                                    .on_click(move |_, w, cx| copy(w, cx)),
                            ),
                    )
            });
        let follow = fire(&self.act, "follow-up");
        let mut tray = div()
            .w(px(460.))
            .max_w_full()
            .flex()
            .flex_col()
            .rounded(px(if self.classic { 18. } else { 12. }))
            .border_1()
            .border_color(p.line)
            .bg(if self.classic { p.cls } else { p.card })
            .text_color(p.fg)
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .px(px(20.))
                    .pt(px(18.))
                    .pb(px(6.))
                    .when(self.classic, |d| {
                        d.m(px(4.))
                            .rounded(px(14.))
                            .border_1()
                            .border_color(p.line)
                            .bg(p.card)
                    })
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .text_size(px(12.5))
                            .text_color(p.mute)
                            .child(
                                div()
                                    .flex()
                                    .children(self.sources.iter().take(3).map(|s| letter(s, p))),
                            )
                            .child(self.header.clone()),
                    )
                    .child(text),
            );
        if self.classic {
            if let Some(q) = quote {
                tray = tray.child(q);
            }
            tray = tray.child(chips);
        } else {
            tray = tray.child(chips);
            if let Some(q) = quote {
                tray = tray.child(q);
            }
        }
        tray.child(
            div()
                .min_h(px(48.))
                .px(px(20.))
                .flex()
                .items_center()
                .gap(px(10.))
                .when(!self.classic, |d| d.border_t_1().border_color(p.line))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(12.5))
                        .text_color(if self.copied { p.ok } else { p.mute })
                        .child(foot),
                )
                .child(
                    div()
                        .id("cited-follow")
                        .h(px(32.))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .rounded(px(8.))
                        .when(!self.classic, |d| d.border_1().border_color(p.line))
                        .text_size(px(13.))
                        .cursor_pointer()
                        .child("Follow up →")
                        .on_click(move |_, w, cx| follow(w, cx)),
                ),
        )
    }
}

// ────────────────────────────────────────────────────────────── Message Actions ──

/// `message-actions` (affectUI Message Actions). The host keeps the versions and the bar's state;
/// acts: `copy`, `regenerate`, `prev`, `next`, `up`, `down`, `read`, `reason`, `send`, `close`.
#[derive(IntoElement)]
pub struct MessageActionsEl {
    prompt: String,
    model: String,
    versions: Vec<String>,
    current: usize,
    vote: u8,
    feedback: bool,
    reason: Option<usize>,
    sent: bool,
    copied: bool,
    spinning: bool,
    reading: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for MessageActionsEl {
    fn default() -> Self {
        let (pr, m, _, v, _) = demo_message();
        Self {
            prompt: pr,
            model: m,
            current: v.len().saturating_sub(1),
            versions: v,
            vote: 0,
            feedback: false,
            reason: None,
            sent: false,
            copied: false,
            spinning: false,
            reading: false,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl MessageActionsEl {
    /// The version shown (0-based).
    pub fn current(mut self, i: usize) -> Self {
        self.current = i;
        self
    }
    /// Vote (0 none, 1 up, 2 down), reasons open, the chosen reason, and sent.
    pub fn feedback(mut self, vote: u8, open: bool, reason: Option<usize>, sent: bool) -> Self {
        self.vote = vote;
        self.feedback = open;
        self.reason = reason;
        self.sent = sent;
        self
    }
    /// Copied, regenerating and reading aloud right now.
    pub fn busy(mut self, copied: bool, spinning: bool, reading: bool) -> Self {
        self.copied = copied;
        self.spinning = spinning;
        self.reading = reading;
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

impl RenderOnce for MessageActionsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let n = self.versions.len();
        let cur = self.current.min(n.saturating_sub(1));
        let text = self.versions.get(cur).cloned().unwrap_or_default();
        let mut bar = div()
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(if self.classic { 48. } else { 56. }))
            .px(px(20.))
            .child(icon_btn(
                "copy",
                if self.copied { "✓" } else { "⧉" },
                self.copied,
                p,
                &self.act,
            ))
            .child(icon_btn("regenerate", "↻", self.spinning, p, &self.act));
        if n > 1 {
            let prev = fire(&self.act, "prev");
            let next = fire(&self.act, "next");
            bar = bar.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .text_size(px(13.))
                    .text_color(p.mute)
                    .child(
                        div()
                            .id("ma-prev")
                            .px(px(4.))
                            .when(cur == 0, |d| d.opacity(0.35))
                            .child("‹")
                            .on_click(move |_, w, cx| prev(w, cx)),
                    )
                    .child(
                        div()
                            .text_color(p.fg)
                            .font_weight(FontWeight::SEMIBOLD)
                            .child((cur + 1).to_string()),
                    )
                    .child(format!("/ {n}"))
                    .child(
                        div()
                            .id("ma-next")
                            .px(px(4.))
                            .when(cur + 1 >= n, |d| d.opacity(0.35))
                            .child("›")
                            .on_click(move |_, w, cx| next(w, cx)),
                    ),
            );
        }
        bar = bar
            .child(icon_btn("up", "▲", self.vote == 1, p, &self.act))
            .child(icon_btn("down", "▼", self.vote == 2, p, &self.act))
            .child(icon_btn(
                "read",
                if self.reading { "▮▮" } else { "♪" },
                self.reading,
                p,
                &self.act,
            ));
        let mut dock = div()
            .flex()
            .flex_col()
            .when(!self.classic, |d| {
                d.border_t_1().border_color(p.line).bg(p.band)
            })
            .when(self.classic, |d| {
                d.mt(px(8.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
            })
            .child(bar);
        if self.feedback {
            let close = fire(&self.act, "close");
            let send = fire(&self.act, "send");
            dock = dock.child(
                div()
                    .px(px(20.))
                    .pb(px(16.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .text_size(px(14.))
                            .child(
                                div()
                                    .flex_1()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child("What went wrong?"),
                            )
                            .child(
                                div()
                                    .id("ma-close")
                                    .cursor_pointer()
                                    .text_color(p.mute)
                                    .child("×")
                                    .on_click(move |_, w, cx| close(w, cx)),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap(px(8.))
                            .children(MESSAGE_REASONS.iter().enumerate().map(|(i, r)| {
                                let f = fire(&self.act, "reason");
                                let on = self.reason == Some(i);
                                div()
                                    .id(SharedString::from(format!("ma-r{i}")))
                                    .h(px(32.))
                                    .px(px(14.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(8.))
                                    .border_1()
                                    .border_color(if on { p.chip } else { p.line })
                                    .when(on && !self.classic, |d| d.bg(p.chip))
                                    .when(on && self.classic, |d| {
                                        d.bg(rgba(0x3B82F633)).text_color(p.blue)
                                    })
                                    .when(self.reason.is_some() && !on, |d| d.opacity(0.5))
                                    .text_size(px(13.))
                                    .child(r.to_string())
                                    .on_click(move |_, w, cx| f(w, cx))
                            }))
                            .child(div().flex_1())
                            .child(if self.sent {
                                div()
                                    .id("ma-send")
                                    .h(px(36.))
                                    .px(px(14.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(8.))
                                    .bg(p.okbg)
                                    .text_color(p.ok)
                                    .child("✓ Thanks!")
                            } else {
                                div()
                                    .id("ma-send")
                                    .h(px(36.))
                                    .px(px(16.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(8.))
                                    .bg(if self.reason.is_some() { p.send } else { p.off })
                                    .text_color(if self.reason.is_some() {
                                        p.send_fg
                                    } else {
                                        p.mute
                                    })
                                    .child("Send")
                                    .on_click(move |_, w, cx| send(w, cx))
                            }),
                    ),
            );
        }
        let msg = div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .px(px(24.))
            .pt(px(20.))
            .pb(px(18.))
            .when(self.classic, |d| {
                d.rounded(px(16.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
            })
            .child(
                div().flex().justify_end().child(
                    div()
                        .max_w(px(420.))
                        .px(px(16.))
                        .py(px(11.))
                        .rounded(px(10.))
                        .bg(p.you)
                        .text_size(px(15.))
                        .child(self.prompt.clone()),
                ),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .text_size(px(13.))
                    .text_color(p.mute)
                    .child(tile("claude", 16., p))
                    .child(self.model.clone()),
            )
            .child(
                div()
                    .text_size(px(16.))
                    .when(self.spinning, |d| d.opacity(0.5))
                    .child(text),
            );
        div()
            .w(px(544.))
            .max_w_full()
            .flex()
            .flex_col()
            .text_color(p.fg)
            .when(!self.classic, |d| {
                d.rounded(px(14.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
                    .overflow_hidden()
            })
            .child(msg)
            .child(dock)
    }
}

// ────────────────────────────────────────────────────────────── Thinking pill ──

/// `thinking-loader` (affectUI Thinking Loader): the host passes the milliseconds since it
/// started; eight rays, the phase label, then a check and "Thought for Ns".
#[derive(IntoElement)]
pub struct ThinkingPillEl {
    phases: Vec<String>,
    interval: u64,
    elapsed: u64,
    looped: bool,
    done: bool,
    small: bool,
    plain: bool,
    classic: bool,
    theme: &'static Theme,
}

impl Default for ThinkingPillEl {
    fn default() -> Self {
        Self {
            phases: LOADER_PHASES.iter().map(|s| s.to_string()).collect(),
            interval: LOADER_INTERVAL,
            elapsed: 0,
            looped: true,
            done: false,
            small: false,
            plain: false,
            classic: false,
            theme: &Theme::DARK,
        }
    }
}

impl ThinkingPillEl {
    /// Labels to walk through.
    pub fn phases(mut self, p: Vec<String>) -> Self {
        self.phases = p;
        self
    }
    /// Milliseconds since it started; `looped` starts over, else it settles on done.
    pub fn elapsed(mut self, ms: u64, looped: bool) -> Self {
        self.elapsed = ms;
        self.looped = looped;
        self
    }
    /// Done.
    pub fn done(mut self, on: bool) -> Self {
        self.done = on;
        self
    }
    /// Small (26 px) and plain (no chip).
    pub fn look(mut self, small: bool, plain: bool) -> Self {
        self.small = small;
        self.plain = plain;
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
}

impl RenderOnce for ThinkingPillEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let (i, past) = loader_phase(self.elapsed, self.interval, self.phases.len(), self.looped);
        let done = self.done || past;
        let secs = ((self.phases.len() as u64 * self.interval) / 1000).max(1);
        let label = if done {
            thought_for(secs)
        } else {
            self.phases.get(i).cloned().unwrap_or_default()
        };
        let ray = if self.classic { p.mute } else { p.blue };
        let lead = ((self.elapsed / 100) % 8) as usize;
        let spinner = div()
            .size(px(14.))
            .flex()
            .flex_wrap()
            .gap(px(1.))
            .children((0..8).map(move |k| {
                let age = (lead + 8 - k) % 8;
                div()
                    .size(px(3.))
                    .rounded_full()
                    .bg(ray.opacity(1.0 - age as f32 * 0.1))
            }));
        div()
            .h(px(if self.small { 26. } else { 32. }))
            .px(px(if self.plain { 0. } else { 11. }))
            .flex()
            .items_center()
            .gap(px(8.))
            .when(!self.plain, |d| {
                d.rounded(px(if self.classic { 999. } else { 10. }))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
            })
            .text_size(px(if self.small { 12. } else { 13. }))
            .text_color(if done { p.fg } else { p.mute })
            .child(if done {
                div().text_color(ray).child("✓").into_any_element()
            } else {
                spinner.into_any_element()
            })
            .child(label)
    }
}

// ─────────────────────────────────────────────────────────────── Thinking panel ──

/// `thinking-state` (affectUI Thinking State): the host passes the milliseconds since it started,
/// whether it is open and whether every source shows; acts: `toggle`, `more`.
#[derive(IntoElement)]
pub struct ThinkingPanelEl {
    steps: Vec<ThinkStep>,
    elapsed: u64,
    open: bool,
    all: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ThinkingPanelEl {
    fn default() -> Self {
        Self {
            steps: demo_think_steps(),
            elapsed: 0,
            open: true,
            all: false,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ThinkingPanelEl {
    /// Milliseconds since it started.
    pub fn elapsed(mut self, ms: u64) -> Self {
        self.elapsed = ms;
        self
    }
    /// Steps shown (open) and every source shown.
    pub fn state(mut self, open: bool, all: bool) -> Self {
        self.open = open;
        self.all = all;
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

impl RenderOnce for ThinkingPanelEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let n = self.steps.len();
        let (shown, done, secs) = think_progress(self.elapsed, THINK_STEP_MS, n);
        let toggle = fire(&self.act, "toggle");
        let dots = div()
            .size(px(13.))
            .flex()
            .flex_wrap()
            .gap(px(1.5))
            .children((0..9).map(|_| div().size(px(3.)).rounded_full().bg(p.blue)));
        let head = div()
            .id("ts-head")
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(28.))
            .text_size(px(14.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(if done { p.fg } else { p.mute })
            .cursor_pointer()
            .child(if self.classic {
                div()
                    .text_color(p.mute)
                    .child(if self.open { "⌄" } else { "›" })
                    .into_any_element()
            } else {
                dots.into_any_element()
            })
            .child(if done {
                format!("Worked for {secs}s")
            } else {
                format!("Thinking {secs}s")
            })
            .when(!self.open && !self.classic, |d| {
                d.child(div().text_color(p.mute).child("›"))
            })
            .on_click(move |_, w, cx| toggle(w, cx));
        let mut root = div()
            .w(px(420.))
            .max_w_full()
            .flex()
            .flex_col()
            .text_color(p.fg)
            .child(head);
        if self.open {
            let mut tl = div()
                .mt(px(10.))
                .ml(px(8.))
                .pl(px(14.))
                .border_l_1()
                .border_color(p.line)
                .flex()
                .flex_col()
                .gap(px(12.));
            for (k, s) in self.steps.iter().take(shown).enumerate() {
                let active = !done && k + 1 == shown;
                let mut col = div().flex().flex_col().gap(px(8.)).child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_size(px(14.))
                        .text_color(if s.sources.is_empty() { p.fg } else { p.mute })
                        .child(div().size(px(5.)).rounded(px(1.5)).bg(if active {
                            p.blue
                        } else {
                            p.mute
                        }))
                        .child(s.label.clone()),
                );
                if !s.sources.is_empty() {
                    let more = fire(&self.act, "more");
                    let list: Vec<_> = s
                        .sources
                        .iter()
                        .chain(if self.all { s.more.iter() } else { [].iter() })
                        .cloned()
                        .collect();
                    col = col.child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap(px(6.))
                            .children(list.into_iter().map(|c| {
                                div()
                                    .h(px(28.))
                                    .px(px(9.))
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .rounded(px(if self.classic { 999. } else { 8. }))
                                    .when(!self.classic, |d| {
                                        d.border_1().border_color(p.line).bg(p.card)
                                    })
                                    .when(self.classic, |d| d.bg(p.cls))
                                    .text_size(px(13.))
                                    .child(tile(&c.brand, 14., p))
                                    .child(c.label)
                            }))
                            .when(!s.more.is_empty(), |d| {
                                d.child(
                                    div()
                                        .id("ts-more")
                                        .h(px(28.))
                                        .px(px(10.))
                                        .flex()
                                        .items_center()
                                        .rounded(px(8.))
                                        .bg(p.chip)
                                        .text_size(px(13.))
                                        .text_color(p.mute)
                                        .cursor_pointer()
                                        .child(if self.all {
                                            "Show less".to_string()
                                        } else {
                                            format!("+{} more", s.more.len())
                                        })
                                        .on_click(move |_, w, cx| more(w, cx)),
                                )
                            }),
                    );
                }
                tl = tl.child(col);
            }
            if done {
                tl = tl.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_size(px(14.))
                        .font_weight(FontWeight::MEDIUM)
                        .child(
                            div()
                                .size(px(15.))
                                .rounded_full()
                                .bg(p.blue)
                                .text_color(gpui::white())
                                .text_size(px(10.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .child("✓"),
                        )
                        .child("Finished thinking"),
                );
            }
            root = root.child(tl);
        }
        root
    }
}

// ──────────────────────────────────────────────────────────── Writing Assistant ──

/// `writing-assistant` (affectUI Writing Assistant). The host keeps the messages (each with the
/// milliseconds since it was sent) and the field; acts: `suggest-0`…`suggest-3`, `send`, `mic`,
/// `new-chat`.
#[derive(IntoElement)]
pub struct WritingAssistantEl {
    name: String,
    tagline: String,
    greeting: String,
    suggestions: Vec<String>,
    messages: Vec<(String, u64)>,
    field: String,
    listening: bool,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for WritingAssistantEl {
    fn default() -> Self {
        Self {
            name: "Wren".into(),
            tagline: "Your writing companion.".into(),
            greeting: "What should we write?".into(),
            suggestions: [
                "Tighten my email",
                "Make it shorter",
                "Soften the tone",
                "Draft a reply",
            ]
            .iter()
            .map(|s| s.to_string())
            .collect(),
            messages: vec![],
            field: String::new(),
            listening: false,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

const SUGGEST_ACTS: [&str; 4] = ["suggest-0", "suggest-1", "suggest-2", "suggest-3"];

impl WritingAssistantEl {
    /// Messages sent, each with the milliseconds since it was sent.
    pub fn messages(mut self, m: Vec<(String, u64)>) -> Self {
        self.messages = m;
        self
    }
    /// The field's text and whether the mic is listening.
    pub fn field(mut self, text: impl Into<String>, listening: bool) -> Self {
        self.field = text.into();
        self.listening = listening;
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

impl RenderOnce for WritingAssistantEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let chatting = !self.messages.is_empty();
        let newc = fire(&self.act, "new-chat");
        let mic = fire(&self.act, "mic");
        let glow = div()
            .absolute()
            .left_0()
            .right_0()
            .bottom_0()
            .h(px(if self.classic { 360. } else { 220. }))
            .opacity(if chatting { 0.35 } else { 1.0 })
            .bg(gpui::linear_gradient(
                0.,
                gpui::linear_color_stop(p.violet.opacity(0.22), 0.),
                gpui::linear_color_stop(p.card.opacity(0.), 1.),
            ));
        let mut body = div()
            .relative()
            .flex_1()
            .flex()
            .flex_col()
            .justify_end()
            .px(px(20.))
            .pt(px(20.))
            .pb(px(12.))
            .when(self.classic, |d| {
                d.rounded(px(16.))
                    .border_1()
                    .border_color(p.line)
                    .overflow_hidden()
            })
            .child(glow);
        if chatting {
            body = body.child(div().flex().flex_col().gap(px(14.)).mb(px(14.)).children(
                self.messages.iter().map(|(m, el)| {
                    let reply = reply_at(&sample_reply(m), *el);
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        .text_size(px(13.5))
                        .child(
                            div().flex().justify_end().child(
                                div()
                                    .px(px(12.))
                                    .py(px(8.))
                                    .rounded(px(11.))
                                    .bg(p.you)
                                    .child(m.clone()),
                            ),
                        )
                        .child(match reply {
                            None => div().text_color(p.mute).child("Writing…"),
                            Some(r) => div().child(r),
                        })
                }),
            ));
        } else {
            body = body.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .mb(px(16.))
                    .child(
                        div()
                            .text_size(px(if self.classic { 23. } else { 17. }))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.greeting.clone()),
                    )
                    .child(div().flex().flex_wrap().gap(px(8.)).children(
                        self.suggestions.iter().enumerate().map(|(i, s)| {
                            let f = fire(&self.act, SUGGEST_ACTS[i.min(3)]);
                            div()
                                .id(SharedString::from(format!("wa-s{i}")))
                                .h(px(32.))
                                .px(px(12.))
                                .flex()
                                .items_center()
                                .rounded(px(if self.classic { 999. } else { 9. }))
                                .when(!self.classic, |d| d.border_1().border_color(p.line))
                                .bg(p.card)
                                .text_size(px(13.))
                                .cursor_pointer()
                                .child(s.clone())
                                .on_click(move |_, w, cx| f(w, cx))
                        }),
                    )),
            );
        }
        body = body.child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(40.))
                .child(
                    div()
                        .flex_1()
                        .text_size(px(13.5))
                        .text_color(if self.field.is_empty() { p.mute } else { p.fg })
                        .when(self.classic, |d| {
                            d.h(px(40.))
                                .px(px(16.))
                                .flex()
                                .items_center()
                                .rounded_full()
                                .bg(p.card)
                        })
                        .child(if self.field.is_empty() {
                            "Ask anything…".to_string()
                        } else {
                            self.field.clone()
                        }),
                )
                .child(
                    div()
                        .id("wa-mic")
                        .size(px(32.))
                        .rounded(px(if self.classic { 999. } else { 9. }))
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(self.listening, |d| {
                            d.bg(p.blue.opacity(0.16)).text_color(p.blue)
                        })
                        .when(!self.listening, |d| d.text_color(p.mute))
                        .cursor_pointer()
                        .child(if self.listening { "▮▮▮" } else { "🎙" })
                        .on_click(move |_, w, cx| mic(w, cx)),
                ),
        );
        div()
            .w(px(380.))
            .max_w_full()
            .min_h(px(430.))
            .flex()
            .flex_col()
            .rounded(px(if self.classic { 20. } else { 14. }))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .text_color(p.fg)
            .overflow_hidden()
            .when(self.classic, |d| d.p(px(6.)))
            .child(
                div()
                    .min_h(px(62.))
                    .px(px(20.))
                    .flex()
                    .items_center()
                    .gap(px(10.))
                    .when(!self.classic, |d| d.border_b_1().border_color(p.line))
                    .when(!self.classic, |d| {
                        d.child(div().text_color(p.violet).child("✦"))
                    })
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(
                                div()
                                    .text_size(px(if self.classic { 19. } else { 14.5 }))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(self.name.clone()),
                            )
                            .child(
                                div()
                                    .text_size(px(12.5))
                                    .text_color(p.mute)
                                    .child(self.tagline.clone()),
                            ),
                    )
                    .when(chatting, |d| {
                        d.child(
                            div()
                                .id("wa-new")
                                .text_color(p.mute)
                                .cursor_pointer()
                                .child("+")
                                .on_click(move |_, w, cx| newc(w, cx)),
                        )
                    }),
            )
            .child(body)
    }
}
