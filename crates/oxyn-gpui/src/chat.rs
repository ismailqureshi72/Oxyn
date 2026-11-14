//! Chat elements for GPUI (plan #27), drawn from the same `oxyn_core::chat` models as the Dioxus
//! components: `MarkdownEl`, `ReasoningEl`, `TraceEl`, `SourceCardsEl`, `AnswerEl`, `MessageEl`,
//! `FeedbackEl`, `ChatThreadEl` and `StartersEl`.
//!
//! Elements are `RenderOnce`: the host owns the model (and the clock), feeds `elapsed` / `now`
//! values in, routes presses back through `on_*` callbacks and re-renders.
//!
//! Gaps against Dioxus (listed in the PR): no chunk fade, shimmer or caret blink (text is drawn
//! as revealed), tables draw as plain rows, the source preview tooltip and the emoji picker are
//! not drawn, the feedback note is display-only (the host owns text entry), and the support
//! conversation, research run, document extraction, context and recommendation cards are
//! Dioxus-only for now. Behaviour follows Arc UI and affectUI (rebuilt; no source used) and
//! Beautiful UI's MIT Thinking and Chat (© 2026 Shane Levine).

use std::{rc::Rc, time::Duration};

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    Styled, Window, div, prelude::*, px,
};
use oxyn_core::{
    ClockMs,
    chat::{
        markdown::{Block, Inline, parse, revealed_at},
        reasoning::{Reasoning, ReasoningStatus, RowStatus, Trace, preview, steps},
        sources::{Claim, Provenance, SearchStatus, Slot, SourceCards, grounding},
        thread::{
            ChatMessage, Delivery, Feedback, FeedbackPhase, NEGATIVE, POSITIVE, Starter,
            ThreadItem, Thumb, clock, group,
        },
    },
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

type OnIndex = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type OnText = Rc<dyn Fn(String, &mut Window, &mut App)>;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

macro_rules! themed {
    () => {
        /// Theme.
        pub fn theme(mut self, t: &'static Theme) -> Self {
            self.theme = t;
            self
        }
    };
}

macro_rules! on_index {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
            self.$name = Some(Rc::new(f));
            self
        }
    };
}

macro_rules! on_text {
    ($name:ident, $doc:literal) => {
        #[doc = $doc]
        pub fn $name(mut self, f: impl Fn(String, &mut Window, &mut App) + 'static) -> Self {
            self.$name = Some(Rc::new(f));
            self
        }
    };
}

fn chip(t: &'static Theme, label: impl Into<SharedString>) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .h(px(22.))
        .px(px(8.))
        .rounded(px(99.))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .text_size(px(11.5))
        .text_color(c(t, ColorRole::FgMuted))
        .child(label.into())
}

// ─────────────────────────────────────── Markdown ──

fn inline_el(
    t: &'static Theme,
    runs: &[Inline],
    on_cite: Option<OnIndex>,
    key: usize,
) -> AnyElement {
    div()
        .flex()
        .flex_wrap()
        .items_baseline()
        .children(runs.iter().enumerate().map(|(i, r)| {
            match r {
                Inline::Text(s) => div().child(s.clone()).into_any_element(),
                Inline::Bold(s) => div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(s.clone())
                    .into_any_element(),
                Inline::Italic(s) => div().italic().child(s.clone()).into_any_element(),
                Inline::Code(s) => div()
                    .px(px(4.))
                    .rounded(px(4.))
                    .bg(c(t, ColorRole::SurfaceSunken))
                    .font_family("monospace")
                    .text_size(px(12.5))
                    .child(s.clone())
                    .into_any_element(),
                Inline::Link(s, _) => div()
                    .underline()
                    .text_color(c(t, ColorRole::Link))
                    .child(s.clone())
                    .into_any_element(),
                Inline::Cite(n) => {
                    let n = *n as usize;
                    let f = on_cite.clone();
                    div()
                        .id(("cite", key * 1000 + i))
                        .role(Role::Button)
                        .aria_label(SharedString::from(format!("Source {n}")))
                        .mx(px(2.))
                        .px(px(5.))
                        .rounded(px(5.))
                        .bg(c(t, ColorRole::SurfaceSelected))
                        .text_size(px(10.5))
                        .child(n.to_string())
                        .when_some(f, |d, f| {
                            d.cursor_pointer().on_click(move |_, w, cx| f(n, w, cx))
                        })
                        .into_any_element()
                }
            }
        }))
        .into_any_element()
}

fn blocks_el(t: &'static Theme, blocks: Vec<Block>, on_cite: Option<OnIndex>) -> gpui::Div {
    div()
        .flex()
        .flex_col()
        .gap(px(8.))
        .children(blocks.into_iter().enumerate().map(|(k, b)| {
            match b {
                Block::Heading(l, runs) => div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_size(px(match l {
                        1 => 18.,
                        2 => 16.,
                        _ => 14.,
                    }))
                    .child(inline_el(t, &runs, on_cite.clone(), k))
                    .into_any_element(),
                Block::Para(runs) => inline_el(t, &runs, on_cite.clone(), k),
                Block::Quote(runs) => div()
                    .pl(px(10.))
                    .border_l_2()
                    .border_color(c(t, ColorRole::BorderStrong))
                    .text_color(c(t, ColorRole::FgMuted))
                    .child(inline_el(t, &runs, on_cite.clone(), k))
                    .into_any_element(),
                Block::List(ordered, items) => div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .children(items.into_iter().enumerate().map(|(i, it)| {
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(div().w(px(16.)).text_color(c(t, ColorRole::FgMuted)).child(
                                if ordered {
                                    format!("{}.", i + 1)
                                } else {
                                    "•".into()
                                },
                            ))
                            .child(inline_el(t, &it, on_cite.clone(), k * 100 + i))
                    }))
                    .into_any_element(),
                Block::Code { lang, text, open } => div()
                    .flex()
                    .flex_col()
                    .rounded(px(8.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .bg(c(t, ColorRole::SurfaceSunken))
                    .child(
                        div()
                            .px(px(10.))
                            .py(px(4.))
                            .text_size(px(11.))
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(if lang.is_empty() {
                                "code".to_string()
                            } else {
                                lang
                            }),
                    )
                    .child(
                        div()
                            .px(px(10.))
                            .pb(px(8.))
                            .font_family("monospace")
                            .text_size(px(12.5))
                            .children(
                                text.lines()
                                    .map(|l| div().child(l.to_string()))
                                    .collect::<Vec<_>>(),
                            )
                            .when(open, |d| {
                                d.child(div().text_color(c(t, ColorRole::FgMuted)).child("▍"))
                            }),
                    )
                    .into_any_element(),
                Block::Rule => div()
                    .h(px(1.))
                    .bg(c(t, ColorRole::BorderSubtle))
                    .into_any_element(),
                Block::Table(head, rows) => div()
                    .flex()
                    .flex_col()
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .rounded(px(8.))
                    .child(
                        div()
                            .flex()
                            .font_weight(FontWeight::SEMIBOLD)
                            .children(head.into_iter().map(|h| div().flex_1().p(px(6.)).child(h))),
                    )
                    .children(rows.into_iter().map(|r| {
                        div()
                            .flex()
                            .border_t_1()
                            .border_color(c(t, ColorRole::BorderSubtle))
                            .children(r.into_iter().map(|x| div().flex_1().p(px(6.)).child(x)))
                    }))
                    .into_any_element(),
            }
        }))
}

/// Streaming markdown: reveals `text` at `cps` characters per second `elapsed` after it started.
#[derive(IntoElement)]
pub struct MarkdownEl {
    text: String,
    elapsed: Option<Duration>,
    cps: f32,
    on_cite: Option<OnIndex>,
    theme: &'static Theme,
}

impl MarkdownEl {
    /// Finished markdown.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            elapsed: None,
            cps: 90.0,
            on_cite: None,
            theme: &Theme::DARK,
        }
    }
    /// Streaming: reveal what has arrived `elapsed` after the start at `cps`.
    pub fn streaming(mut self, elapsed: Duration, cps: f32) -> Self {
        self.elapsed = Some(elapsed);
        self.cps = cps;
        self
    }
    on_index!(on_cite, "A citation chip was pressed (1-based source).");
    themed!();
}

impl RenderOnce for MarkdownEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let total = self.text.chars().count();
        let (src, busy) = match self.elapsed {
            Some(e) => {
                let n = revealed_at(total, e, self.cps);
                (self.text.chars().take(n).collect::<String>(), n < total)
            }
            None => (self.text, false),
        };
        div()
            .id("oxyn-md")
            .aria_label(if busy {
                "Response, streaming"
            } else {
                "Response"
            })
            .text_size(px(14.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(blocks_el(t, parse(&src), self.on_cite))
            .when(busy, |d| {
                d.child(div().text_color(c(t, ColorRole::FgMuted)).child("▍"))
            })
    }
}

// ────────────────────────────────────── Reasoning ──

/// Reasoning block: header (“Thinking” / “Thought for 7s”), preview while thinking, steps when open.
#[derive(IntoElement)]
pub struct ReasoningEl {
    model: Reasoning,
    now: ClockMs,
    on_toggle: Option<OnIndex>,
    on_stop: Option<OnIndex>,
    on_retry: Option<OnIndex>,
    theme: &'static Theme,
}

impl ReasoningEl {
    /// From the model at `now`.
    pub fn new(model: Reasoning, now: ClockMs) -> Self {
        Self {
            model,
            now,
            on_toggle: None,
            on_stop: None,
            on_retry: None,
            theme: &Theme::DARK,
        }
    }
    on_index!(on_toggle, "The header was pressed.");
    on_index!(on_stop, "Stop was pressed while thinking.");
    on_index!(on_retry, "Retry was pressed after an error.");
    themed!();
}

impl RenderOnce for ReasoningEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let thinking = m.status == ReasoningStatus::Thinking;
        let label = m.label(self.now);
        let secs = if thinking { m.secs(self.now) } else { None };
        let ft = self.on_toggle.clone();
        let fs = self.on_stop.clone();
        let fr = self.on_retry.clone();
        let header = div()
            .id("oxyn-rs-h")
            .role(Role::Button)
            .aria_expanded(m.open)
            .aria_label(SharedString::from(m.announce(self.now)))
            .flex()
            .items_center()
            .gap(px(8.))
            .h(px(44.))
            .px(px(12.))
            .child(div().text_color(c(t, ColorRole::FgMuted)).child(if m.open {
                "⌄"
            } else {
                "›"
            }))
            .child(
                div()
                    .flex_1()
                    .font_weight(FontWeight::MEDIUM)
                    .text_color(c(
                        t,
                        if m.status == ReasoningStatus::Error {
                            ColorRole::StatusDangerFg
                        } else {
                            ColorRole::FgDefault
                        },
                    ))
                    .child(label),
            )
            .when_some(secs, |d, s| {
                d.child(
                    div()
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child(format!("{s}s")),
                )
            })
            .when_some(ft, |d, f| {
                d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
            });
        let action = |id: &'static str, label: &'static str, f: Option<OnIndex>| {
            div()
                .id(id)
                .role(Role::Button)
                .px(px(8.))
                .h(px(24.))
                .flex()
                .items_center()
                .rounded(px(6.))
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault))
                .text_size(px(12.))
                .child(label)
                .when_some(f, |d, f| {
                    d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                })
        };
        div()
            .flex()
            .flex_col()
            .rounded(px(10.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(header)
            .when(m.shows_preview(), |d| {
                d.child(
                    div()
                        .px(px(12.))
                        .pb(px(10.))
                        .text_color(c(t, ColorRole::FgSubtle))
                        .children(preview(&m.text, 2).into_iter().map(|l| div().child(l))),
                )
            })
            .when(m.open, |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .px(px(12.))
                        .pb(px(12.))
                        .text_color(c(t, ColorRole::FgMuted))
                        .children(steps(&m.text).into_iter().map(|s| {
                            div()
                                .flex()
                                .flex_col()
                                .when_some(s.title, |d, ti| {
                                    d.child(
                                        div()
                                            .font_weight(FontWeight::MEDIUM)
                                            .text_color(c(t, ColorRole::FgDefault))
                                            .child(ti),
                                    )
                                })
                                .child(s.text)
                        })),
                )
            })
            .when(thinking, |d| {
                d.child(
                    div()
                        .px(px(12.))
                        .pb(px(10.))
                        .child(action("oxyn-rs-stop", "Stop", fs)),
                )
            })
            .when(m.status == ReasoningStatus::Error, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .px(px(12.))
                        .pb(px(10.))
                        .child(
                            div()
                                .flex_1()
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(m.error.clone().unwrap_or_else(|| "Connection lost".into())),
                        )
                        .child(action("oxyn-rs-retry", "Retry", fr)),
                )
            })
    }
}

/// Beautiful UI thinking trace: a header summary and rows that move pending → active → done.
#[derive(IntoElement)]
pub struct TraceEl {
    model: Trace,
    elapsed: Option<(Duration, Duration)>,
    on_toggle: Option<OnIndex>,
    theme: &'static Theme,
}

impl TraceEl {
    /// From the model (rows as given).
    pub fn new(model: Trace) -> Self {
        Self {
            model,
            elapsed: None,
            on_toggle: None,
            theme: &Theme::DARK,
        }
    }
    /// Plays the rows back: one per `step`, `elapsed` after the start.
    pub fn playback(mut self, elapsed: Duration, step: Duration) -> Self {
        self.elapsed = Some((elapsed, step));
        self
    }
    on_index!(on_toggle, "The header was pressed.");
    themed!();
}

impl RenderOnce for TraceEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let rows = match self.elapsed {
            Some((e, s)) => self.model.playback(e, s),
            None => self.model.rows.clone(),
        };
        let summary = self.model.summary(&rows);
        let open = self.model.open;
        let f = self.on_toggle.clone();
        div()
            .flex()
            .flex_col()
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .id("oxyn-trace-h")
                    .role(Role::Button)
                    .aria_expanded(open)
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(32.))
                    .child(div().text_color(c(t, ColorRole::FgMuted)).child(if open {
                        "⌄"
                    } else {
                        "›"
                    }))
                    .child(summary)
                    .when_some(f, |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                    }),
            )
            .when(open, |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .pl(px(8.))
                        .border_l_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .children(rows.into_iter().map(|r| {
                            let (mark, role) = match r.status {
                                RowStatus::Done => ("✓", ColorRole::StatusSuccessFg),
                                RowStatus::Active => ("◌", ColorRole::FgDefault),
                                RowStatus::Failed => ("×", ColorRole::StatusDangerFg),
                                RowStatus::Pending => ("·", ColorRole::FgSubtle),
                            };
                            div()
                                .flex()
                                .gap(px(8.))
                                .child(div().w(px(14.)).text_color(c(t, role)).child(mark))
                                .child(div().flex().flex_col().child(r.label).when_some(
                                    r.detail,
                                    |d, x| {
                                        d.child(
                                            div()
                                                .font_family("monospace")
                                                .text_size(px(11.5))
                                                .text_color(c(t, ColorRole::FgSubtle))
                                                .child(x),
                                        )
                                    },
                                ))
                        })),
                )
            })
    }
}

// ──────────────────────────────────────── Sources ──

/// Source cards with skeletons while searching, a favicon stack after `max`, and highlight.
#[derive(IntoElement)]
pub struct SourceCardsEl {
    model: SourceCards,
    now: ClockMs,
    duration_ms: Option<u64>,
    on_open: Option<OnIndex>,
    on_expand: Option<OnIndex>,
    on_retry: Option<OnIndex>,
    theme: &'static Theme,
}

impl SourceCardsEl {
    /// From the model at `now`.
    pub fn new(model: SourceCards, now: ClockMs) -> Self {
        Self {
            model,
            now,
            duration_ms: None,
            on_open: None,
            on_expand: None,
            on_retry: None,
            theme: &Theme::DARK,
        }
    }
    /// Search duration for the status line.
    pub fn duration_ms(mut self, d: u64) -> Self {
        self.duration_ms = Some(d);
        self
    }
    on_index!(
        on_open,
        "A card was pressed (0-based source); the host opens its URL."
    );
    on_index!(on_expand, "The stack was pressed.");
    on_index!(on_retry, "Retry was pressed after a failed search.");
    themed!();
}

impl RenderOnce for SourceCardsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let hl = m.highlight_at(self.now);
        let status = m.status_line(self.duration_ms);
        let fo = self.on_open.clone();
        let fe = self.on_expand.clone();
        let fr = self.on_retry.clone();
        let card = |d: gpui::Stateful<gpui::Div>| {
            d.flex()
                .flex_col()
                .gap(px(4.))
                .w(px(168.))
                .h(px(76.))
                .p(px(10.))
                .rounded(px(10.))
                .border_1()
                .bg(c(t, ColorRole::SurfaceRaised))
        };
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .text_size(px(12.5))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .text_color(c(
                        t,
                        if m.status == SearchStatus::Error {
                            ColorRole::StatusDangerFg
                        } else {
                            ColorRole::FgMuted
                        },
                    ))
                    .child(status)
                    .when(m.status == SearchStatus::Error, |d| {
                        d.child(
                            div()
                                .id("oxyn-src-retry")
                                .role(Role::Button)
                                .underline()
                                .child("Retry")
                                .when_some(fr, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                                }),
                        )
                    }),
            )
            .child(
                div()
                    .id("oxyn-src")
                    .role(Role::List)
                    .flex()
                    .flex_wrap()
                    .gap(px(8.))
                    .children(m.slots().into_iter().enumerate().map(|(k, s)| {
                        match s {
                            Slot::Card(i) => {
                                let src = &m.sources[i];
                                let on = hl == Some(i + 1);
                                let f = fo.clone();
                                card(div().id(("src", i)))
                                    .role(Role::ListItem)
                                    .aria_label(SharedString::from(src.card_label(i + 1)))
                                    .border_color(c(
                                        t,
                                        if on {
                                            ColorRole::FgDefault
                                        } else {
                                            ColorRole::BorderDefault
                                        },
                                    ))
                                    .child(
                                        div()
                                            .text_size(px(11.))
                                            .text_color(c(t, ColorRole::FgSubtle))
                                            .child(format!("{} · {}", i + 1, src.domain())),
                                    )
                                    .child(
                                        div()
                                            .font_weight(FontWeight::MEDIUM)
                                            .line_clamp(2)
                                            .child(src.title.clone()),
                                    )
                                    .when_some(f, |d, f| {
                                        d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                                    })
                                    .into_any_element()
                            }
                            Slot::Skeleton => card(div().id(("src-sk", k)))
                                .border_color(c(t, ColorRole::BorderSubtle))
                                .child(
                                    div()
                                        .h(px(8.))
                                        .w(px(60.))
                                        .rounded(px(4.))
                                        .bg(c(t, ColorRole::SurfaceSunken)),
                                )
                                .child(
                                    div()
                                        .h(px(10.))
                                        .w(px(130.))
                                        .rounded(px(4.))
                                        .bg(c(t, ColorRole::SurfaceSunken)),
                                )
                                .into_any_element(),
                            Slot::Stack(from) => {
                                let f = fe.clone();
                                let rest = m.sources.len() - from;
                                card(div().id("src-stack"))
                                    .role(Role::Button)
                                    .aria_label(SharedString::from(format!(
                                        "Show {rest} more sources"
                                    )))
                                    .border_color(c(t, ColorRole::BorderDefault))
                                    .justify_center()
                                    .child(div().flex().gap(px(2.)).children(
                                        m.sources[from..].iter().take(4).map(|s| {
                                            div()
                                                .size(px(18.))
                                                .rounded(px(9.))
                                                .bg(c(t, ColorRole::SurfaceSelected))
                                                .flex()
                                                .items_center()
                                                .justify_center()
                                                .text_size(px(9.))
                                                .child(
                                                    s.domain()
                                                        .chars()
                                                        .next()
                                                        .unwrap_or('·')
                                                        .to_uppercase()
                                                        .to_string(),
                                                )
                                        }),
                                    ))
                                    .child(
                                        div()
                                            .text_color(c(t, ColorRole::FgMuted))
                                            .child(format!("+{rest} more")),
                                    )
                                    .when_some(f, |d, f| {
                                        d.cursor_pointer().on_click(move |_, w, cx| f(from, w, cx))
                                    })
                                    .into_any_element()
                            }
                        }
                    })),
            )
    }
}

/// AI answer (question, answer markdown, follow-ups) and the cited variant with claim provenance
/// and a grounding meter.
#[derive(IntoElement)]
pub struct AnswerEl {
    question: SharedString,
    answer: String,
    claims: Vec<Claim>,
    follow_ups: Vec<SharedString>,
    on_follow_up: Option<OnText>,
    on_cite: Option<OnIndex>,
    theme: &'static Theme,
}

impl AnswerEl {
    /// An answer.
    pub fn new(question: impl Into<SharedString>, answer: impl Into<String>) -> Self {
        Self {
            question: question.into(),
            answer: answer.into(),
            claims: vec![],
            follow_ups: vec![],
            on_follow_up: None,
            on_cite: None,
            theme: &Theme::DARK,
        }
    }
    /// Cited variant: claims with provenance replace the prose.
    pub fn claims(mut self, c: Vec<Claim>) -> Self {
        self.claims = c;
        self
    }
    /// Follow-up questions.
    pub fn follow_ups(mut self, v: Vec<SharedString>) -> Self {
        self.follow_ups = v;
        self
    }
    on_text!(on_follow_up, "A follow-up was pressed.");
    on_index!(on_cite, "A citation was pressed (1-based).");
    themed!();
}

impl RenderOnce for AnswerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let cited = !self.claims.is_empty();
        let g = grounding(&self.claims);
        let fc = self.on_cite.clone();
        let ff = self.on_follow_up.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .text_size(px(14.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .text_size(px(17.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(self.question),
            )
            .when(!cited, |d| {
                d.child(blocks_el(t, parse(&self.answer), fc.clone()))
            })
            .when(cited, |d| {
                d.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_size(px(12.))
                        .text_color(c(t, ColorRole::FgMuted))
                        .child(format!("Grounded {}%", (g * 100.0).round() as u32))
                        .child(
                            div()
                                .w(px(120.))
                                .h(px(4.))
                                .rounded(px(2.))
                                .bg(c(t, ColorRole::SurfaceSunken))
                                .child(
                                    div()
                                        .h_full()
                                        .w(px(120. * g))
                                        .rounded(px(2.))
                                        .bg(c(t, ColorRole::FgDefault)),
                                ),
                        ),
                )
                .children(self.claims.iter().enumerate().map(|(i, cl)| {
                    let role = match cl.provenance {
                        Provenance::Sourced(_) => ColorRole::FgMuted,
                        Provenance::Inferred => ColorRole::StatusInfoFg,
                        Provenance::Unverified => ColorRole::StatusWarningFg,
                    };
                    let cites = match &cl.provenance {
                        Provenance::Sourced(v) => v.clone(),
                        _ => vec![],
                    };
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(6.))
                        .child(cl.text.clone())
                        .children(cites.into_iter().map(|n| {
                            let f = fc.clone();
                            div()
                                .id(("claim-cite", i * 100 + n))
                                .role(Role::Button)
                                .aria_label(SharedString::from(format!("Source {n}")))
                                .px(px(5.))
                                .rounded(px(5.))
                                .bg(c(t, ColorRole::SurfaceSelected))
                                .text_size(px(10.5))
                                .child(n.to_string())
                                .when_some(f, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(n, w, cx))
                                })
                        }))
                        .child(chip(t, cl.provenance.label()).text_color(c(t, role)))
                }))
            })
            .when(!self.follow_ups.is_empty(), |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .border_t_1()
                        .border_color(c(t, ColorRole::BorderSubtle))
                        .children(self.follow_ups.into_iter().enumerate().map(|(i, q)| {
                            let f = ff.clone();
                            let qs = q.to_string();
                            div()
                                .id(("follow", i))
                                .role(Role::Button)
                                .flex()
                                .justify_between()
                                .py(px(8.))
                                .border_b_1()
                                .border_color(c(t, ColorRole::BorderSubtle))
                                .child(q)
                                .child(div().text_color(c(t, ColorRole::FgSubtle)).child("+"))
                                .when_some(f, |d, f| {
                                    d.cursor_pointer()
                                        .on_click(move |_, w, cx| f(qs.clone(), w, cx))
                                })
                        })),
                )
            })
    }
}

// ─────────────────────────────── Messages / feedback ──

/// One message: user bubble, assistant prose, system note or tool row; `failed` adds Retry.
#[derive(IntoElement)]
pub struct MessageEl {
    role: &'static str,
    text: String,
    author: Option<SharedString>,
    flat: bool,
    failed: bool,
    on_retry: Option<OnIndex>,
    theme: &'static Theme,
}

impl MessageEl {
    /// A message with role `user` · `assistant` · `system` · `tool`.
    pub fn new(role: &'static str, text: impl Into<String>) -> Self {
        Self {
            role,
            text: text.into(),
            author: None,
            flat: false,
            failed: false,
            on_retry: None,
            theme: &Theme::DARK,
        }
    }
    /// Author name.
    pub fn author(mut self, a: impl Into<SharedString>) -> Self {
        self.author = Some(a.into());
        self
    }
    /// Flat transcript variant (no bubble).
    pub fn flat(mut self) -> Self {
        self.flat = true;
        self
    }
    /// Send failed.
    pub fn failed(mut self) -> Self {
        self.failed = true;
        self
    }
    on_index!(on_retry, "Retry was pressed on a failed send.");
    themed!();
}

impl RenderOnce for MessageEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let user = self.role == "user";
        let f = self.on_retry.clone();
        let body: AnyElement = match self.role {
            "system" => div()
                .w_full()
                .flex()
                .justify_center()
                .text_size(px(12.))
                .text_color(c(t, ColorRole::FgSubtle))
                .child(self.text)
                .into_any_element(),
            "tool" => div()
                .font_family("monospace")
                .text_size(px(12.))
                .text_color(c(t, ColorRole::FgMuted))
                .child(format!("› {}", self.text))
                .into_any_element(),
            _ if user && !self.flat => div()
                .max_w(px(420.))
                .px(px(12.))
                .py(px(8.))
                .rounded(px(14.))
                .bg(c(t, ColorRole::SurfaceSelected))
                .child(self.text)
                .into_any_element(),
            _ => blocks_el(t, parse(&self.text), None).into_any_element(),
        };
        div()
            .id("oxyn-msg")
            .role(Role::Article)
            .flex()
            .flex_col()
            .gap(px(4.))
            .text_size(px(14.))
            .text_color(c(t, ColorRole::FgDefault))
            .when(user && !self.flat, |d| d.items_end())
            .when_some(self.author, |d, a| {
                d.child(
                    div()
                        .text_size(px(12.))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(c(t, ColorRole::FgMuted))
                        .child(a),
                )
            })
            .child(body)
            .when(self.failed, |d| {
                d.child(
                    div()
                        .flex()
                        .gap(px(6.))
                        .text_size(px(12.))
                        .text_color(c(t, ColorRole::StatusDangerFg))
                        .child("Not delivered ·")
                        .child(
                            div()
                                .id("oxyn-msg-retry")
                                .role(Role::Button)
                                .underline()
                                .child("Retry")
                                .when_some(f, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                                }),
                        ),
                )
            })
    }
}

/// Response feedback: thumbs, reason chips after a rating, Submit, thanks / error.
#[derive(IntoElement)]
pub struct FeedbackEl {
    model: Feedback,
    positive: bool,
    on_thumb: Option<OnIndex>,
    on_reason: Option<OnText>,
    on_submit: Option<OnIndex>,
    theme: &'static Theme,
}

impl FeedbackEl {
    /// From the model.
    pub fn new(model: Feedback) -> Self {
        Self {
            model,
            positive: true,
            on_thumb: None,
            on_reason: None,
            on_submit: None,
            theme: &Theme::DARK,
        }
    }
    /// Hide the “What was good?” reasons after a thumbs up.
    pub fn no_positive(mut self) -> Self {
        self.positive = false;
        self
    }
    on_index!(
        on_thumb,
        "A thumb was pressed: 0 = up, 1 = down (feed to `Feedback::press`)."
    );
    on_text!(on_reason, "A reason chip was pressed.");
    on_index!(on_submit, "Submit (or Try again) was pressed.");
    themed!();
}

impl RenderOnce for FeedbackEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let thumb =
            |i: usize, glyph: &'static str, label: &'static str, on: bool, f: Option<OnIndex>| {
                div()
                    .id(("thumb", i))
                    .role(Role::Button)
                    .aria_label(label)
                    .size(px(28.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(7.))
                    .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                    .text_color(c(
                        t,
                        if on {
                            ColorRole::FgDefault
                        } else {
                            ColorRole::FgMuted
                        },
                    ))
                    .child(glyph)
                    .when_some(f, |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                    })
            };
        let reasons: Vec<&str> = match m.rating {
            Some(Thumb::Down) => NEGATIVE.to_vec(),
            Some(Thumb::Up) if self.positive => POSITIVE.to_vec(),
            _ => vec![],
        };
        let fr = self.on_reason.clone();
        let fs = self.on_submit.clone();
        let form = matches!(
            m.phase,
            FeedbackPhase::Form | FeedbackPhase::Sending | FeedbackPhase::Error
        );
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .text_size(px(12.5))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .flex()
                    .gap(px(2.))
                    .child(thumb(
                        0,
                        "👍",
                        "Good response",
                        m.rating == Some(Thumb::Up),
                        self.on_thumb.clone(),
                    ))
                    .child(thumb(
                        1,
                        "👎",
                        "Bad response",
                        m.rating == Some(Thumb::Down),
                        self.on_thumb.clone(),
                    )),
            )
            .when(form && !reasons.is_empty(), |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(8.))
                        .p(px(10.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .child(div().font_weight(FontWeight::MEDIUM).child(
                            if m.rating == Some(Thumb::Up) {
                                "What was good?"
                            } else {
                                "What went wrong?"
                            },
                        ))
                        .child(div().flex().flex_wrap().gap(px(6.)).children(
                            reasons.into_iter().enumerate().map(|(i, r)| {
                                let on = m.reasons.iter().any(|x| x == r);
                                let f = fr.clone();
                                let rs = r.to_string();
                                div()
                                    .id(("reason", i))
                                    .role(Role::Button)
                                    .px(px(10.))
                                    .h(px(26.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(99.))
                                    .border_1()
                                    .border_color(c(t, ColorRole::BorderDefault))
                                    .when(on, |d| {
                                        d.bg(c(t, ColorRole::FgDefault))
                                            .text_color(c(t, ColorRole::FgInverse))
                                    })
                                    .child(r)
                                    .when_some(f, |d, f| {
                                        d.cursor_pointer()
                                            .on_click(move |_, w, cx| f(rs.clone(), w, cx))
                                    })
                            }),
                        ))
                        .when(!m.comment.is_empty(), |d| {
                            d.child(
                                div()
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child(m.comment.clone()),
                            )
                        })
                        .when(m.phase == FeedbackPhase::Error, |d| {
                            d.child(
                                div()
                                    .text_color(c(t, ColorRole::StatusDangerFg))
                                    .child("Couldn’t send feedback."),
                            )
                        })
                        .child(
                            div()
                                .id("oxyn-fb-submit")
                                .role(Role::Button)
                                .self_end()
                                .px(px(12.))
                                .h(px(28.))
                                .flex()
                                .items_center()
                                .rounded(px(8.))
                                .bg(c(t, ColorRole::AccentSolid))
                                .text_color(c(t, ColorRole::FgOnAccent))
                                .child(match m.phase {
                                    FeedbackPhase::Sending => "Sending…",
                                    FeedbackPhase::Error => "Try again",
                                    _ => "Submit",
                                })
                                .when_some(fs, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                                }),
                        ),
                )
            })
            .when(m.phase == FeedbackPhase::Thanks, |d| {
                d.child(
                    div()
                        .text_color(c(t, ColorRole::StatusSuccessFg))
                        .child("✓ Thanks for the feedback"),
                )
            })
    }
}

// ───────────────────────────────────── Chat thread ──

/// A team chat thread: day separators, messages grouped by author, receipts and reactions.
#[derive(IntoElement)]
pub struct ChatThreadEl {
    messages: Vec<ChatMessage>,
    me: String,
    now: u64,
    window: u64,
    typing: Option<SharedString>,
    on_react: Option<OnText>,
    theme: &'static Theme,
}

impl ChatThreadEl {
    /// Messages (oldest first), your author id and the host clock (ms).
    pub fn new(messages: Vec<ChatMessage>, me: impl Into<String>, now: u64) -> Self {
        Self {
            messages,
            me: me.into(),
            now,
            window: 5 * 60_000,
            typing: None,
            on_react: None,
            theme: &Theme::DARK,
        }
    }
    /// Typing line (from `typing_label`).
    pub fn typing(mut self, s: impl Into<SharedString>) -> Self {
        let s = s.into();
        self.typing = (!s.is_empty()).then_some(s);
        self
    }
    on_text!(on_react, "A reaction chip was pressed: “message-id:emoji”.");
    themed!();
}

impl RenderOnce for ChatThreadEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let items = group(&self.messages, self.window, self.now);
        let msgs = self.messages;
        let me = self.me;
        let fr = self.on_react.clone();
        div()
            .id("oxyn-thread")
            .role(Role::Log)
            .flex()
            .flex_col()
            .gap(px(10.))
            .text_size(px(13.5))
            .text_color(c(t, ColorRole::FgDefault))
            .children(items.into_iter().map(|it| {
                match it {
                    ThreadItem::Day(l) => div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .text_size(px(11.5))
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child(div().flex_1().h(px(1.)).bg(c(t, ColorRole::BorderSubtle)))
                        .child(l)
                        .child(div().flex_1().h(px(1.)).bg(c(t, ColorRole::BorderSubtle)))
                        .into_any_element(),
                    ThreadItem::Group { author, messages } => {
                        let mine = author == me;
                        let first = &msgs[messages[0]];
                        div()
                            .flex()
                            .flex_col()
                            .gap(px(3.))
                            .when(mine, |d| d.items_end())
                            .child(
                                div()
                                    .flex()
                                    .gap(px(6.))
                                    .text_size(px(11.5))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .when(!mine, |d| {
                                        d.child(
                                            div()
                                                .font_weight(FontWeight::MEDIUM)
                                                .text_color(c(t, ColorRole::FgMuted))
                                                .child(author.clone()),
                                        )
                                    })
                                    .child(clock(first.at)),
                            )
                            .children(messages.into_iter().map(|i| {
                                let m = &msgs[i];
                                let f = fr.clone();
                                div()
                                    .flex()
                                    .flex_col()
                                    .gap(px(3.))
                                    .when(mine, |d| d.items_end())
                                    .child(
                                        div()
                                            .max_w(px(420.))
                                            .px(px(11.))
                                            .py(px(7.))
                                            .rounded(px(12.))
                                            .bg(c(
                                                t,
                                                if mine {
                                                    ColorRole::SurfaceSelected
                                                } else {
                                                    ColorRole::SurfaceRaised
                                                },
                                            ))
                                            .border_1()
                                            .border_color(c(
                                                t,
                                                if m.status == Delivery::Failed {
                                                    ColorRole::StatusDangerBorder
                                                } else {
                                                    ColorRole::BorderSubtle
                                                },
                                            ))
                                            .child(m.text.clone()),
                                    )
                                    .when(!m.reactions.is_empty(), |d| {
                                        d.child(div().flex().gap(px(4.)).children(
                                            m.reactions.iter().enumerate().map(
                                                |(k, (e, n, on))| {
                                                    let f = f.clone();
                                                    let key = format!("{}:{e}", m.id);
                                                    div()
                                                        .id(SharedString::from(format!(
                                                            "re-{}-{k}",
                                                            m.id
                                                        )))
                                                        .role(Role::Button)
                                                        .px(px(6.))
                                                        .h(px(22.))
                                                        .flex()
                                                        .items_center()
                                                        .gap(px(3.))
                                                        .rounded(px(99.))
                                                        .border_1()
                                                        .border_color(c(
                                                            t,
                                                            if *on {
                                                                ColorRole::BorderStrong
                                                            } else {
                                                                ColorRole::BorderSubtle
                                                            },
                                                        ))
                                                        .text_size(px(11.5))
                                                        .child(format!("{e} {n}"))
                                                        .when_some(f, |d, f| {
                                                            d.cursor_pointer().on_click(
                                                                move |_, w, cx| {
                                                                    f(key.clone(), w, cx)
                                                                },
                                                            )
                                                        })
                                                },
                                            ),
                                        ))
                                    })
                                    .when(mine, |d| {
                                        d.child(
                                            div()
                                                .text_size(px(11.))
                                                .text_color(c(
                                                    t,
                                                    if m.status == Delivery::Failed {
                                                        ColorRole::StatusDangerFg
                                                    } else {
                                                        ColorRole::FgSubtle
                                                    },
                                                ))
                                                .child(m.status.label()),
                                        )
                                    })
                            }))
                            .into_any_element()
                    }
                }
            }))
            .when_some(self.typing, |d, s| {
                d.child(
                    div()
                        .text_size(px(12.))
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child(s),
                )
            })
    }
}

/// Starter prompts: category tabs over a grid of cards and a Shuffle button.
#[derive(IntoElement)]
pub struct StartersEl {
    starters: Vec<Starter>,
    categories: Vec<String>,
    category: String,
    on_category: Option<OnText>,
    on_pick: Option<OnText>,
    on_shuffle: Option<OnIndex>,
    theme: &'static Theme,
}

impl StartersEl {
    /// The starters to show (from `starters_for`), category ids and the selected one.
    pub fn new(
        starters: Vec<Starter>,
        categories: Vec<String>,
        category: impl Into<String>,
    ) -> Self {
        Self {
            starters,
            categories,
            category: category.into(),
            on_category: None,
            on_pick: None,
            on_shuffle: None,
            theme: &Theme::DARK,
        }
    }
    on_text!(on_category, "A category tab was pressed.");
    on_text!(on_pick, "A card was pressed: its full prompt.");
    on_index!(on_shuffle, "Shuffle was pressed.");
    themed!();
}

impl RenderOnce for StartersEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let fc = self.on_category.clone();
        let fp = self.on_pick.clone();
        let fs = self.on_shuffle.clone();
        let cur = self.category;
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .id("st-tabs")
                    .role(Role::TabList)
                    .children(self.categories.into_iter().enumerate().map(|(i, k)| {
                        let on = k == cur;
                        let f = fc.clone();
                        let ks = k.clone();
                        let mut label = k.clone();
                        if let Some(ch) = label.get_mut(0..1) {
                            ch.make_ascii_uppercase();
                        }
                        div()
                            .id(("st-cat", i))
                            .role(Role::Tab)
                            .px(px(10.))
                            .h(px(28.))
                            .flex()
                            .items_center()
                            .rounded(px(99.))
                            .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                            .text_color(c(
                                t,
                                if on {
                                    ColorRole::FgDefault
                                } else {
                                    ColorRole::FgMuted
                                },
                            ))
                            .child(label)
                            .when_some(f, |d, f| {
                                d.cursor_pointer()
                                    .on_click(move |_, w, cx| f(ks.clone(), w, cx))
                            })
                    }))
                    .child(div().flex_1())
                    .child(
                        div()
                            .id("st-shuffle")
                            .role(Role::Button)
                            .px(px(10.))
                            .h(px(28.))
                            .flex()
                            .items_center()
                            .rounded(px(8.))
                            .border_1()
                            .border_color(c(t, ColorRole::BorderDefault))
                            .child("↻ Shuffle")
                            .when_some(fs, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                            }),
                    ),
            )
            .child(div().flex().flex_wrap().gap(px(8.)).children(
                self.starters.into_iter().enumerate().map(|(i, s)| {
                    let f = fp.clone();
                    let p = s.prompt.clone();
                    div()
                        .id(("st", i))
                        .role(Role::Button)
                        .aria_label(SharedString::from(s.title.clone()))
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .w(px(220.))
                        .p(px(12.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .child(div().font_weight(FontWeight::MEDIUM).child(s.title))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(c(t, ColorRole::FgMuted))
                                .line_clamp(2)
                                .child(s.prompt),
                        )
                        .when_some(s.app, |d, a| d.child(chip(t, a)))
                        .when_some(f, |d, f| {
                            d.cursor_pointer()
                                .on_click(move |_, w, cx| f(p.clone(), w, cx))
                        })
                }),
            ))
    }
}
