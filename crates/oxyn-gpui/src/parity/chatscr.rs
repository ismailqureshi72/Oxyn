//! AI Chat parity (S1) for GPUI over `oxyn_core::parity::screensfid`: the exact composer (the Add
//! menu with its two groups, Request approval, Auto ⌄ with coloured official model marks, mic and a
//! round send, plus the Work in a folder / Link apps strip) and static snapshots of the AI Chat
//! screens on the 1440 × 900 stage: [`ChatStartEl`] (centred 672 column) and [`ChatOverviewEl`]
//! (centred 880 column with the numbers). Each takes the data state, the screen's own extra state
//! (`.extra("attached")`, `"dictating"`, `"limit"`, or `"plus"` / `"model"` / `"approval"` to draw a
//! menu open), the frame and the theme; the host owns any live state. Rows carry `Act` ids:
//! `plus-<key>`, `model-<key>`, `suggest-<key>`.

use gpui::{
    AnyElement, App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::parity::screensfid::{
    AGENTS_WORK_W, PERSON, PROGRESS_COL_W, PROGRESS_FOLDED, RAIL_W, RAIL_WORK_W, RUN_WORK_W,
    STAGE_H, STAGE_W, StepState, angles, crew, progress_outputs, progress_steps, progress_tools,
    run_line, run_marks, run_plan, step_state, tool_mark,
};
use oxyn_core::parity::screensfid::{
    APPROVAL_MODES, COMPOSER_APPS, CREDITS_CAP, MODELS, OVERVIEW_COL_W, OVERVIEW_WEEKS, PLUS_ADD,
    PLUS_PLUGINS, START_COL_W, ScreenState, ShellLayout, ago, attach_label, connected_words,
    credits_used, delta, greeting, heat, made_files, near_limit, overview_counts, overview_line,
    start_suggestions, thousands, threads, used_pct,
};
use oxyn_tokens::Theme;

use super::Act;
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{frame, page, sidebar, skel};
use crate::brand::{brand_mark, tone_color};

macro_rules! chat_el {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(IntoElement)]
        pub struct $name {
            state: ScreenState,
            extra: &'static str,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, extra: "", layout: ShellLayout::Inset, act: None, theme: &Theme::DARK }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
                self
            }
            /// The screen's own state (a key from its `*_STATES` row past the first three), or a
            /// menu to draw open (`plus`, `model`, `approval`).
            pub fn extra(mut self, k: &'static str) -> Self {
                self.extra = k;
                self
            }
            /// Inset or Flat.
            pub fn layout(mut self, l: ShellLayout) -> Self {
                self.layout = l;
                self
            }
            /// Light or dark chrome.
            pub fn theme(mut self, t: &'static Theme) -> Self {
                self.theme = t;
                self
            }
            /// `nav-<key>`, `plus-<key>`, `model-<key>`, `suggest-<key>`.
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
    };
}

/// A coloured official mark, or nothing for an unknown name.
pub(super) fn mark(p: &ThPal, name: &str, size: f32) -> Option<AnyElement> {
    if name.is_empty() {
        None
    } else {
        brand_mark(name, size, p.fg)
    }
}

/// One line of text that ellipsises in a flex row (rule R5).
pub(super) fn clip(t: impl Into<SharedString>) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .overflow_hidden()
        .whitespace_nowrap()
        .text_ellipsis()
        .child(t.into())
}

fn quiet(p: &ThPal, t: impl Into<SharedString>) -> Div {
    div()
        .flex_none()
        .whitespace_nowrap()
        .text_size(px(12.))
        .text_color(p.mute)
        .child(t.into())
}

fn tone(p: &ThPal, t: IconTone) -> Hsla {
    tone_color(t, p.fg.l > 0.5)
}

/// A file glyph: a small tile in the file's semantic tone.
pub(super) fn file_tile(p: &ThPal, name: &str) -> Div {
    div()
        .flex_none()
        .w(px(12.))
        .h(px(14.))
        .rounded(px(2.5))
        .bg(tone(p, IconTone::for_file(name)))
}

fn menu_box(p: &ThPal, w: f32) -> Div {
    div()
        .w(px(w))
        .p(px(5.))
        .rounded(px(12.))
        .bg(p.card)
        .border_1()
        .border_color(p.line)
        .shadow_lg()
        .flex()
        .flex_col()
}

fn menu_row(p: &ThPal, lead: Option<AnyElement>, l: &str, hint: String, hi: bool) -> Div {
    let mut r = div()
        .h(px(32.))
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(9.))
        .rounded(px(8.))
        .text_color(p.fg2)
        .when(hi, |d| d.bg(p.sel).text_color(p.fg));
    if let Some(e) = lead {
        r = r.child(e);
    }
    r.child(div().flex_none().whitespace_nowrap().child(l.to_string()))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .text_right()
                .text_size(px(12.))
                .text_color(p.mute)
                .child(hint),
        )
}

/// The Add (+) menu, drawn open: Add (files, attach app ›, folder, connect apps, goal, record a
/// skill, draw), then Plugins (documents, PDF, spreadsheets).
pub(super) fn plus_menu(p: &ThPal, connected: usize) -> Div {
    let glyph = |k: &str| -> &'static str {
        match k {
            "files" => "⌕",
            "attach" => "▦",
            "folder" => "▭",
            "connect" => "⏚",
            "goal" => "◎",
            "skill" => "◉",
            "draw" => "✎",
            _ => "▯",
        }
    };
    let mut m = menu_box(p, 300.);
    for (i, it) in PLUS_ADD.iter().enumerate() {
        let hint = if it.key == "connect" {
            connected_words(connected)
        } else if it.more {
            "›".to_string()
        } else {
            it.hint.to_string()
        };
        let col = if it.key == "folder" {
            tone(p, IconTone::Folder)
        } else {
            p.mute
        };
        m = m.child(menu_row(
            p,
            Some(
                div()
                    .w(px(16.))
                    .text_color(col)
                    .child(glyph(it.key))
                    .into_any_element(),
            ),
            it.label,
            hint,
            i == 0,
        ));
    }
    m = m
        .child(div().h(px(1.)).my(px(5.)).mx(px(4.)).bg(p.line))
        .child(
            div()
                .px(px(8.))
                .py(px(2.))
                .text_size(px(11.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(p.mute)
                .child("Plugins"),
        );
    for it in PLUS_PLUGINS {
        let file = match it.key {
            "pdf" => "x.pdf",
            "sheets" => "x.csv",
            _ => "x.docx",
        };
        m = m.child(menu_row(
            p,
            Some(file_tile(p, file).into_any_element()),
            it.label,
            it.hint.to_string(),
            false,
        ));
    }
    m
}

/// The model picker, drawn open: Auto, then the models with their coloured official marks.
pub(super) fn model_menu(p: &ThPal) -> Div {
    let mut m = menu_box(p, 260.);
    for (i, md) in MODELS.iter().enumerate() {
        let lead = mark(p, md.mark, 15.).unwrap_or_else(|| {
            div()
                .w(px(15.))
                .text_color(p.mute)
                .child("✦")
                .into_any_element()
        });
        m = m.child(menu_row(
            p,
            Some(lead),
            md.name,
            md.hint.to_string(),
            i == 0,
        ));
    }
    m
}

/// The approval menu, drawn open: Request approval, then Oxyn's two steps beyond it.
pub(super) fn approval_menu(p: &ThPal) -> Div {
    let mut m = menu_box(p, 300.);
    for (i, (_, l, h, _)) in APPROVAL_MODES.iter().enumerate() {
        m = m.child(menu_row(p, None, l, h.to_string(), i == 0));
    }
    m
}

/// How a composer is drawn.
#[derive(Clone, Copy, Default)]
pub(super) struct Cmp {
    /// The Work in a folder / Link apps strip.
    pub foot: bool,
    /// Attached files (a leading `@` marks an app: `@notion`).
    pub files: &'static [&'static str],
    /// Taking dictation.
    pub dict: bool,
    /// Placeholder.
    pub placeholder: &'static str,
    /// A menu drawn open: `plus`, `model` or `approval`.
    pub open: &'static str,
    /// Width (0: 680).
    pub w: f32,
}

/// The composer (see the module docs).
pub(super) fn composer(p: &ThPal, c: Cmp) -> Div {
    let w = if c.w > 0. { c.w } else { 680. };
    let ph = if c.placeholder.is_empty() {
        "Ask anything, or @ a file or app"
    } else {
        c.placeholder
    };
    let ghost = |t: &'static str| {
        div()
            .h(px(30.))
            .px(px(8.))
            .flex()
            .items_center()
            .gap(px(6.))
            .rounded(px(9.))
            .text_color(p.fg2)
            .child(t)
    };
    let mut boxed = div()
        .w(px(w))
        .pt(px(12.))
        .pb(px(8.))
        .pl(px(14.))
        .pr(px(10.))
        .rounded(px(16.))
        .bg(p.card)
        .border_1()
        .border_color(p.line)
        .flex()
        .flex_col()
        .gap(px(6.));
    if !c.files.is_empty() {
        let mut chips = div().flex().flex_wrap().gap(px(6.));
        for f in c.files {
            let (lead, l): (AnyElement, String) = match f.strip_prefix('@') {
                Some(a) => (
                    mark(p, a, 14.).unwrap_or_else(|| div().into_any_element()),
                    attach_label(a).to_string(),
                ),
                None => (file_tile(p, f).into_any_element(), f.to_string()),
            };
            chips = chips.child(
                div()
                    .h(px(28.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(p.line)
                    .text_size(px(12.5))
                    .text_color(p.fg2)
                    .child(lead)
                    .child(l)
                    .child(div().text_color(p.mute).child("×")),
            );
        }
        boxed = boxed.child(chips);
    }
    if c.dict {
        let mut wave = div().flex().items_center().gap(px(2.)).h(px(24.));
        for i in 0..24u32 {
            let h = 5. + ((i * 37) % 19) as f32;
            wave = wave.child(div().w(px(3.)).h(px(h)).rounded(px(2.)).bg(p.fg2));
        }
        boxed = boxed.child(
            div()
                .h(px(44.))
                .flex()
                .items_center()
                .gap(px(12.))
                .child(wave)
                .child(quiet(p, "Listening… speak, then press the mic to stop")),
        );
    } else {
        boxed = boxed.child(div().h(px(44.)).text_color(p.mute).child(ph));
    }
    let mdl = MODELS[0];
    boxed = boxed.child(
        div()
            .flex()
            .items_center()
            .gap(px(2.))
            .ml(px(-6.))
            .child(
                div()
                    .id("plus-open")
                    .w(px(32.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.))
                    .when(c.open == "plus", |d| d.bg(p.sel))
                    .text_color(p.fg2)
                    .text_size(px(18.))
                    .child("+"),
            )
            .child(ghost("✋ Request approval").when(c.open == "approval", |d| d.bg(p.sel)))
            .child(div().flex_1())
            .child(
                ghost(mdl.name)
                    .child(div().text_color(p.mute).child("⌄"))
                    .when(c.open == "model", |d| d.bg(p.sel)),
            )
            .child(
                div()
                    .w(px(32.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(8.))
                    .when(c.dict, |d| d.bg(p.sel))
                    .text_color(p.fg2)
                    .child("🎙"),
            )
            .child(
                div()
                    .w(px(30.))
                    .h(px(30.))
                    .ml(px(2.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .rounded(px(15.))
                    .bg(p.sel)
                    .text_color(p.mute)
                    .child("↑"),
            ),
    );
    let mut outer = div().relative().w(px(w)).flex().flex_col();
    if c.foot {
        let mut marks = div().flex().items_center().gap(px(3.));
        for (k, _) in COMPOSER_APPS.iter().take(3) {
            if let Some(e) = mark(p, k, 14.) {
                marks = marks.child(e);
            }
        }
        outer = outer
            .rounded(px(16.))
            .bg(p.inset)
            .border_1()
            .border_color(p.line)
            .child(boxed)
            .child(
                div()
                    .h(px(38.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(px(12.5))
                    .text_color(p.mute)
                    .child(div().text_color(tone(p, IconTone::Folder)).child("▭"))
                    .child("Work in a folder")
                    .child(div().flex_1())
                    .child("Link apps")
                    .child(marks),
            );
    } else {
        outer = outer.child(boxed);
    }
    let pop = match c.open {
        "plus" => Some((plus_menu(p, 3), 4.)),
        "approval" => Some((approval_menu(p), 34.)),
        "model" => Some((model_menu(p), w - 340.)),
        _ => None,
    };
    if let Some((m, x)) = pop {
        outer = outer.child(
            div()
                .absolute()
                .top(px(if c.foot { 128. } else { 112. }))
                .left(px(x))
                .child(m),
        );
    }
    outer
}

impl RenderOnce for ChatStartEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let ld = self.state == ScreenState::Loading;
        let files: &'static [&'static str] = if self.extra == "attached" {
            &["brief.pdf", "budget-by-team.csv", "@notion"]
        } else {
            &[]
        };
        let mut sugg = div()
            .flex()
            .flex_wrap()
            .justify_center()
            .gap(px(10.))
            .w(px(640.))
            .mt(px(8.));
        for (k, l, _, _) in start_suggestions() {
            let f = fire(&self.act, "suggest");
            sugg = sugg.child(
                div()
                    .id(SharedString::from(format!("suggest-{k}")))
                    .h(px(32.))
                    .px(px(10.))
                    .flex()
                    .items_center()
                    .rounded(px(9.))
                    .text_color(p.fg2)
                    .when(ld, |d| d.bg(p.sel).text_color(p.sel))
                    .child(l)
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        let hi = format!("{}, Rowan. What are we working on?", greeting(19));
        let body = div()
            .flex_1()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .gap(px(14.))
            .pb(px(120.))
            .child(if ld {
                div()
                    .w(px(420.))
                    .h(px(30.))
                    .mb(px(16.))
                    .rounded(px(8.))
                    .bg(p.sel)
            } else {
                div()
                    .mb(px(16.))
                    .text_size(px(30.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(hi)
            })
            .child(composer(
                &p,
                Cmp {
                    foot: true,
                    files,
                    dict: self.extra == "dictating",
                    placeholder: "Help me think through…",
                    open: self.extra,
                    w: START_COL_W as f32,
                },
            ))
            .child(sugg);
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "", body),
            None,
        )
    }
}

impl RenderOnce for ChatOverviewEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let ld = self.state == ScreenState::Loading;
        let blank = self.state == ScreenState::Empty;
        let filled = self.state == ScreenState::Sample;
        let lv = if filled {
            heat(OVERVIEW_WEEKS)
        } else {
            vec![0; OVERVIEW_WEEKS * 7]
        };
        let blue = tone(&p, IconTone::Doc);
        let mut grid = div().flex().gap(px(2.));
        for w in 0..OVERVIEW_WEEKS {
            let mut col = div().flex().flex_col().gap(px(2.));
            for d in 0..7 {
                let l = lv[w * 7 + d] as f32;
                col = col.child(div().w(px(9.)).h(px(9.)).rounded(px(2.)).bg(if l == 0. {
                    p.sel
                } else {
                    blue.opacity(0.2 + l * 0.2)
                }));
            }
            grid = grid.child(col);
        }
        let card = |ic: &'static str, t: &'static str, right: String, b: Div, w: f32| {
            div()
                .w(px(w))
                .flex_none()
                .p(px(12.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .bg(p.card)
                .flex()
                .flex_col()
                .gap(px(6.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(7.))
                        .child(div().text_color(p.mute).child(ic))
                        .child(clip(t))
                        .child(quiet(&p, right)),
                )
                .child(if ld { skel(&p, 2) } else { b })
        };
        let counts = overview_counts(filled);
        let act = div().flex().flex_col().gap(px(6.)).child(grid).child(
            div()
                .flex()
                .text_size(px(12.))
                .text_color(p.mute)
                .child(clip(if blank {
                    "Nothing yet – your first chat lights up today's square.".to_string()
                } else {
                    overview_line(counts)
                }))
                .child("Feb – Nov ⌄"),
        );
        let mut recent = div().flex().flex_col();
        if blank {
            recent = recent.child(quiet(&p, "No chats yet. Ask anything above to start one."));
        }
        if filled {
            for t in threads().into_iter().take(3) {
                recent = recent.child(
                    div()
                        .h(px(26.))
                        .flex()
                        .items_center()
                        .gap(px(9.))
                        .child(div().w(px(18.)).h(px(18.)).rounded(px(9.)).bg(p.sel))
                        .child(clip(t.title))
                        .child(quiet(&p, ago(t.mins))),
                );
            }
        }
        let used = if filled { credits_used(self.extra) } else { 0 };
        let warn = near_limit(used, CREDITS_CAP);
        let pct = used_pct(used, CREDITS_CAP) as f32 / 100.;
        let bar_c = if warn {
            tone(&p, IconTone::Approval)
        } else {
            blue
        };
        let usage = div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(4.))
                    .child(
                        div()
                            .text_size(px(19.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(thousands(used)),
                    )
                    .child(quiet(&p, format!("/ {} credits", thousands(CREDITS_CAP)))),
            )
            .child(
                div()
                    .h(px(4.))
                    .w_full()
                    .rounded(px(2.))
                    .bg(p.sel)
                    .child(div().h_full().w(px(188. * pct)).rounded(px(2.)).bg(bar_c)),
            )
            .child(quiet(
                &p,
                if warn {
                    "93% used · Upgrade"
                } else {
                    "Resets 1 Dec"
                },
            ));
        let mut files = div().flex().flex_col();
        if blank {
            files = files.child(quiet(&p, "Docs and sheets it makes for you land here."));
        }
        if filled {
            for (f, _, a) in made_files().into_iter().take(3) {
                files = files.child(
                    div()
                        .h(px(26.))
                        .flex()
                        .items_center()
                        .gap(px(9.))
                        .child(file_tile(&p, f))
                        .child(clip(f))
                        .child(quiet(&p, a)),
                );
            }
        }
        let num = |l: &'static str, v: &'static str, d: i32| {
            let c = if d >= 0 {
                tone(&p, IconTone::Done)
            } else {
                tone(&p, IconTone::Failed)
            };
            div().flex_1().flex().flex_col().child(quiet(&p, l)).child(
                div()
                    .flex()
                    .items_baseline()
                    .gap(px(6.))
                    .child(
                        div()
                            .text_size(px(19.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(v),
                    )
                    .when(!blank, |x| {
                        x.child(div().text_size(px(12.)).text_color(c).child(delta(d)))
                    }),
            )
        };
        let month = div()
            .flex()
            .mt(px(14.))
            .child(num("Chats", if blank { "0" } else { "36" }, 12))
            .child(num("Files made", if blank { "0" } else { "14" }, -3));
        let half = (OVERVIEW_COL_W as f32 - 8.) / 2.;
        let q = (OVERVIEW_COL_W as f32 - 24.) / 4.;
        let more = if filled { "6 more" } else { "" };
        let body = div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(16.))
            .pt(px(120.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .mb(px(10.))
                    .text_size(px(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(div().w(px(26.)).h(px(26.)).rounded(px(8.)).bg(p.fg))
                    .child("Kestrel"),
            )
            .child(composer(
                &p,
                Cmp {
                    foot: true,
                    placeholder: "Help me think through…",
                    open: self.extra,
                    w: START_COL_W as f32,
                    ..Cmp::default()
                },
            ))
            .child(
                div()
                    .w(px(OVERVIEW_COL_W as f32))
                    .mt(px(12.))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(card("◷", "Activity", String::new(), act, half))
                            .child(card("◌", "Recent chats", more.into(), recent, half)),
                    )
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(card("∿", "Usage", String::new(), usage, q))
                            .child(card(
                                "▯",
                                "Files made for you",
                                if filled {
                                    "2 more".into()
                                } else {
                                    String::new()
                                },
                                files,
                                half,
                            ))
                            .child(card("▦", "This month", String::new(), month, q)),
                    ),
            );
        frame(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page(&p, "", body),
            None,
        )
    }
}

chat_el!(
    /// AI Chat / Start: greeting, the composer with its strip, two rows of suggestions.
    ChatStartEl
);
chat_el!(
    /// AI Chat / Overview: the mark, the composer and the numbers in a centred 880 column.
    ChatOverviewEl
);

/// The app frame with the work area as its own card (Inset) or behind a hairline (Flat), at the
/// screen's own width.
pub(super) fn frame_w(
    theme: &Theme,
    lay: ShellLayout,
    left: Div,
    main: Div,
    work: Option<(Div, f32)>,
) -> Div {
    let p = th_pal(theme);
    let card = |d: Div| match lay {
        ShellLayout::Inset => d
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .overflow_hidden(),
        ShellLayout::Flat => d.border_l_1().border_color(p.line).bg(p.card),
    };
    let mut root = div()
        .w(px(STAGE_W as f32))
        .h(px(STAGE_H as f32))
        .flex()
        .rounded(px(16.))
        .border_1()
        .border_color(p.line)
        .bg(p.inset)
        .text_color(p.fg)
        .text_size(px(13.))
        .overflow_hidden()
        .child(left)
        .child(card(div().flex_1().min_w_0().flex().child(main)));
    if let Some((w, ww)) = work {
        root = root.child(
            card(div().w(px(ww)).flex_none().flex().flex_col().child(w))
                .when(lay == ShellLayout::Inset, |d| d.ml(px(6.))),
        );
    }
    match lay {
        ShellLayout::Inset => root.pt(px(8.)).pb(px(8.)).pr(px(8.)),
        ShellLayout::Flat => root,
    }
}

/// A page with a crumb ("Home / title"), an optional pill beside it and actions on the right.
fn page_crumb(
    p: &ThPal,
    crumb: &'static str,
    title: &'static str,
    pill: Option<Div>,
    right: Div,
    body: Div,
) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .child(
            div()
                .h(px(48.))
                .flex_none()
                .px(px(14.))
                .flex()
                .items_center()
                .gap(px(8.))
                .border_b_1()
                .border_color(p.line)
                .when(!crumb.is_empty(), |d| {
                    d.child(div().text_color(p.mute).child(crumb))
                        .child(div().text_color(p.line).child("/"))
                })
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .whitespace_nowrap()
                        .child(title),
                )
                .when_some(pill, |d, x| d.child(x))
                .child(div().flex_1())
                .child(right),
        )
        .child(body)
}

/// A page whose top bar is bare (no hairline, no title).
fn page_bare(body: Div) -> Div {
    div()
        .flex_1()
        .min_w_0()
        .flex()
        .flex_col()
        .child(div().h(px(48.)).flex_none())
        .child(body)
}

/// A tab strip at the top of the work area (the open tabs, then +).
fn work_tabs(p: &ThPal, tabs: &[(&'static str, bool)]) -> Div {
    let mut d = div()
        .h(px(48.))
        .flex_none()
        .px(px(8.))
        .flex()
        .items_center()
        .gap(px(4.))
        .border_b_1()
        .border_color(p.line);
    for &(t, on) in tabs {
        d = d.child(
            div()
                .h(px(30.))
                .max_w(px(170.))
                .px(px(10.))
                .flex()
                .items_center()
                .gap(px(6.))
                .rounded(px(9.))
                .when(on, |x| x.bg(p.card).border_1().border_color(p.line))
                .text_color(if on { p.fg } else { p.mute })
                .child(clip(t))
                .child(div().text_color(p.mute).child("×")),
        );
    }
    d.child(div().px(px(8.)).text_color(p.mute).child("+"))
}

fn pill(_p: &ThPal, t: impl Into<SharedString>, c: Hsla) -> Div {
    div()
        .h(px(24.))
        .px(px(9.))
        .flex()
        .items_center()
        .gap(px(5.))
        .rounded(px(12.))
        .bg(c.opacity(0.12))
        .text_size(px(12.))
        .text_color(c)
        .child(div().w(px(6.)).h(px(6.)).rounded(px(3.)).bg(c))
        .child(t.into())
}

fn chip(p: &ThPal, lead: Option<AnyElement>, t: impl Into<SharedString>, on: bool) -> Div {
    let mut d = div()
        .h(px(30.))
        .px(px(12.))
        .flex()
        .items_center()
        .gap(px(7.))
        .rounded(px(15.))
        .border_1()
        .border_color(if on { p.sel } else { p.line })
        .when(on, |x| x.bg(p.sel))
        .text_color(if on { p.fg } else { p.fg2 });
    if let Some(e) = lead {
        d = d.child(e);
    }
    d.child(div().whitespace_nowrap().child(t.into()))
}

fn bubble(p: &ThPal, t: &'static str) -> Div {
    div().flex().justify_end().child(
        div()
            .max_w(px(520.))
            .px(px(14.))
            .py(px(9.))
            .rounded(px(16.))
            .bg(p.sel)
            .child(t),
    )
}

fn para(t: &'static str) -> Div {
    div().text_size(px(14.)).line_height(px(22.)).child(t)
}

fn switch(p: &ThPal, on: bool) -> Div {
    div()
        .w(px(34.))
        .h(px(20.))
        .p(px(2.))
        .rounded(px(10.))
        .bg(if on { p.fg } else { p.off })
        .flex()
        .when(on, |d| d.justify_end())
        .child(
            div()
                .w(px(16.))
                .h(px(16.))
                .rounded(px(8.))
                .bg(if on { p.card } else { p.knob }),
        )
}

impl RenderOnce for ChatAgentsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let body = match self.state {
            ScreenState::Loading => skel(&p, 8).p(px(24.)),
            ScreenState::Empty => div()
                .flex_1()
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Start with a goal"),
                ),
            ScreenState::Sample => {
                let mut crew_row = div().flex().flex_wrap().gap(px(8.));
                for (i, m) in crew().into_iter().enumerate() {
                    crew_row = crew_row.child(chip(&p, None, m.name, i == 0));
                }
                div()
                    .flex_1()
                    .px(px(32.))
                    .pt(px(24.))
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .child(bubble(&p, "Lead with how it feels"))
                    .child(para("I'll pull your top seller first, then line up the ad frames."))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .items_center()
                            .text_size(px(12.5))
                            .child(div().text_color(tone(&p, IconTone::Agent)).child("⠿"))
                            .child("Worked for 3m 38s"),
                    )
                    .child(para("Before the ads go out I need one call I can't make for you, because it shapes every headline: which angle should lead?"))
                    .child(quiet(&p, "Agents working"))
                    .child(crew_row)
            }
        };
        let main = div().flex_1().flex().flex_col().child(body).child(
            div()
                .px(px(32.))
                .pb(px(16.))
                .flex()
                .flex_col()
                .gap(px(10.))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().w(px(20.)).h(px(20.)).rounded(px(10.)).bg(p.sel))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Orchestrator"),
                        )
                        .child(quiet(&p, "Routes each request")),
                )
                .child(composer(
                    &p,
                    Cmp {
                        w: 680.,
                        open: self.extra,
                        ..Cmp::default()
                    },
                )),
        );
        let filled = self.state == ScreenState::Sample;
        let mut brief = div().flex().flex_col().gap(px(8.)).p(px(14.));
        if filled {
            let mut angles_c = div()
                .p(px(4.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .flex()
                .flex_col();
            for (i, a) in angles().into_iter().enumerate() {
                let mut bars = div().flex().items_end().gap(px(2.));
                for b in 0..3u8 {
                    bars = bars.child(
                        div()
                            .w(px(3.))
                            .h(px(5. + b as f32 * 3.5))
                            .rounded(px(1.))
                            .bg(if b < a.signal {
                                tone(&p, IconTone::Done)
                            } else {
                                p.line
                            }),
                    );
                }
                angles_c = angles_c.child(
                    div()
                        .px(px(10.))
                        .py(px(7.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .rounded(px(9.))
                        .when(i == 0, |d| d.bg(p.sel))
                        .child(
                            div()
                                .w(px(14.))
                                .h(px(14.))
                                .rounded(px(7.))
                                .border_1()
                                .border_color(if i == 0 { p.fg } else { p.line }),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(clip(a.label))
                                .child(quiet(&p, a.why)),
                        )
                        .child(bars),
                );
            }
            let mut chans = div()
                .p(px(4.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .flex()
                .flex_col();
            for (l, on) in oxyn_core::parity::screensfid::channels() {
                chans = chans.child(
                    div()
                        .h(px(44.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(clip(l))
                        .child(switch(&p, on)),
                );
            }
            let made = self.extra == "making";
            brief = brief
                .child(
                    div()
                        .p(px(12.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .child(
                            div().w(px(40.)).h(px(40.)).rounded(px(9.)).bg(tone(
                                &p,
                                IconTone::Data,
                            )
                            .opacity(0.16)),
                        )
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(clip("Field Overshirt"))
                                .child(quiet(&p, "$118 · 640 sold this month")),
                        )
                        .child(pill(&p, "Top seller", tone(&p, IconTone::Done))),
                )
                .child(quiet(&p, "Selling angle"))
                .child(angles_c)
                .child(quiet(&p, "Channels"))
                .child(chans)
                .child(div().flex_1())
                .child(
                    div()
                        .p(px(8.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(quiet(
                            &p,
                            if made {
                                "Making 3 of 6…"
                            } else {
                                "6 ads · ~2 min"
                            },
                        ))
                        .child(div().flex_1())
                        .child(
                            div()
                                .h(px(28.))
                                .px(px(12.))
                                .flex()
                                .items_center()
                                .rounded(px(9.))
                                .bg(p.fg)
                                .text_color(p.card)
                                .child("Generate ads"),
                        ),
                );
        } else {
            brief = brief.child(skel(&p, 9));
        }
        let work = div()
            .flex_1()
            .flex()
            .flex_col()
            .child(work_tabs(&p, &[("Overshirt launch", true)]))
            .child(brief);
        frame_w(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            page_crumb(
                &p,
                "Home",
                "Ads for the overshirt",
                None,
                div().text_color(p.mute).child("⋯"),
                main,
            ),
            Some((work, AGENTS_WORK_W as f32)),
        )
    }
}

impl RenderOnce for ChatRailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let filled = self.state == ScreenState::Sample;
        let mut rail = div()
            .w(px(RAIL_W as f32))
            .flex_none()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.))
            .child(
                div()
                    .w(px(32.))
                    .h(px(32.))
                    .mb(px(10.))
                    .rounded(px(16.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
                    .flex()
                    .items_center()
                    .justify_center()
                    .child("K"),
            );
        for (i, g) in ["◌", "☑", "▷", "▭", "⏚"].into_iter().enumerate() {
            rail = rail.child(
                div()
                    .w(px(34.))
                    .h(px(34.))
                    .rounded(px(10.))
                    .when(i == 0, |d| d.bg(p.sel))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_color(p.fg2)
                    .child(g),
            );
        }
        rail = rail
            .child(div().flex_1())
            .child(div().text_color(p.fg2).child("🔔"))
            .child(div().text_color(p.fg2).child("⚙"))
            .child(
                div()
                    .w(px(26.))
                    .h(px(26.))
                    .rounded(px(13.))
                    .bg(p.sel)
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(10.))
                    .child("RV"),
            );
        let mut col = div()
            .w(px(272.))
            .flex_none()
            .p(px(8.))
            .flex()
            .flex_col()
            .bg(p.card2)
            .border_r_1()
            .border_color(p.line)
            .child(
                div()
                    .h(px(34.))
                    .px(px(6.))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .text_size(px(14.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("All projects")
                    .child(div().text_color(p.mute).child("⌄"))
                    .child(div().flex_1())
                    .child(div().text_color(p.mute).child("⌕")),
            )
            .child(
                div()
                    .h(px(30.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .child("✎  New chat"),
            );
        if filled {
            col = col.child(super::screensfid::gh(&p, "Projects"));
            for (i, pr) in oxyn_core::parity::screensfid::projects()
                .into_iter()
                .enumerate()
            {
                col = col.child(super::screensfid::row_ic(
                    &p,
                    IconTone::Folder,
                    pr.label.to_string(),
                    String::new(),
                    false,
                ));
                if i == 3 {
                    col = col.child(
                        div()
                            .h(px(30.))
                            .pl(px(32.))
                            .flex()
                            .items_center()
                            .rounded(px(8.))
                            .bg(p.sel)
                            .child(clip("Fabric weights for the overshirt")),
                    );
                }
            }
            col = col.child(super::screensfid::gh(&p, "Chats"));
            for t in threads() {
                col = col.child(super::screensfid::row(&p, t.title, String::new(), false));
            }
        } else if self.state == ScreenState::Loading {
            col = col.child(skel(&p, 7));
        } else {
            col = col.child(
                div()
                    .mt(px(24.))
                    .text_color(p.mute)
                    .child("No projects or chats yet"),
            );
        }
        let thread = if filled {
            div()
                .flex_1()
                .px(px(24.))
                .flex()
                .flex_col()
                .gap(px(14.))
                .child(bubble(&p, "Compare five fabrics for the overshirt: weight, price per metre and lead time."))
                .child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .items_center()
                        .child(div().text_color(tone(&p, IconTone::Skills)).child("⠿"))
                        .child(div().font_weight(FontWeight::SEMIBOLD).child("Kestrel"))
                        .child(quiet(&p, "7:22 PM")),
                )
                .child(
                    div()
                        .flex()
                        .gap(px(7.))
                        .items_center()
                        .child(div().text_color(tone(&p, IconTone::Done)).child("✓"))
                        .child(quiet(&p, "Ran 3 searches · 4.1s")),
                )
                .child(para("Washed linen and hemp twill are the sturdiest; organic cotton is the cheapest and the quickest to arrive. Tencel drapes best but needs the gentlest wash."))
                .child(
                    div()
                        .flex()
                        .gap(px(6.))
                        .child(chip(&p, Some(file_tile(&p, "x.csv").into_any_element()), "fabric-compare.csv", true))
                        .child(chip(&p, Some(file_tile(&p, "x.md").into_any_element()), "sources.md", true)),
                )
        } else if self.state == ScreenState::Loading {
            skel(&p, 8).px(px(24.))
        } else {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child("Ask with the web")
        };
        let main = page_bare(div().flex_1().flex().flex_col().child(thread).child(
            div().px(px(24.)).pb(px(16.)).child(composer(
                &p,
                Cmp {
                    w: 646.,
                    placeholder: "Ask anything; it can search the web",
                    open: self.extra,
                    ..Cmp::default()
                },
            )),
        ));
        let mut grid = div().flex().flex_col().border_t_1().border_color(p.line);
        if filled {
            let sh = oxyn_core::parity::screensfid::sheet();
            let mut hd = div().flex().bg(p.inset).child(div().w(px(30.)));
            for c in ["A", "B", "C", "D"] {
                hd = hd.child(
                    div()
                        .flex_1()
                        .h(px(24.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(11.))
                        .text_color(p.mute)
                        .child(c),
                );
            }
            grid = grid.child(hd);
            for (ri, r) in sh.iter().enumerate() {
                let mut tr = div()
                    .flex()
                    .h(px(26.))
                    .border_b_1()
                    .border_color(p.line)
                    .child(
                        div()
                            .w(px(30.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(p.inset)
                            .text_size(px(11.))
                            .text_color(p.mute)
                            .child((ri + 1).to_string()),
                    );
                for (ci, v) in r.iter().take(4).enumerate() {
                    tr = tr.child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .px(px(8.))
                            .flex()
                            .items_center()
                            .border_l_1()
                            .border_color(p.line)
                            .when(ri == 1 && ci == 0, |d| d.border_2().border_color(p.fg2))
                            .child(clip(v.clone())),
                    );
                }
                grid = grid.child(tr);
            }
        }
        let work = div()
            .flex_1()
            .flex()
            .flex_col()
            .child(work_tabs(
                &p,
                &[("fabric-compare.csv", true), ("sources.md", false)],
            ))
            .when(filled, |d| {
                d.child(
                    div()
                        .p(px(16.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(file_tile(&p, "x.csv"))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child("fabric-compare.csv")
                                .child(quiet(&p, "Just now")),
                        ),
                )
                .child(
                    div()
                        .mx(px(12.))
                        .mb(px(10.))
                        .h(px(28.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(p.line)
                        .flex()
                        .items_center()
                        .px(px(8.))
                        .gap(px(10.))
                        .child(quiet(&p, "B2"))
                        .child("185"),
                )
                .child(grid)
            });
        let folded = self.extra == "folded";
        let mut card = div()
            .flex_1()
            .min_w_0()
            .flex()
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .overflow_hidden();
        if !folded {
            card = card.child(col);
        }
        card = card.child(main);
        div()
            .w(px(STAGE_W as f32))
            .h(px(STAGE_H as f32))
            .flex()
            .p(px(8.))
            .pl(px(0.))
            .rounded(px(16.))
            .border_1()
            .border_color(p.line)
            .bg(p.inset)
            .text_color(p.fg)
            .text_size(px(13.))
            .overflow_hidden()
            .child(rail)
            .child(card)
            .child(
                div()
                    .w(px(RAIL_WORK_W as f32))
                    .flex_none()
                    .ml(px(6.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(p.line)
                    .bg(p.card)
                    .overflow_hidden()
                    .flex()
                    .flex_col()
                    .child(work),
            )
    }
}

impl RenderOnce for ChatProgressEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let filled = self.state == ScreenState::Sample;
        let steps = progress_steps();
        let done = if self.extra == "working" {
            3
        } else if filled {
            steps.len()
        } else {
            0
        };
        let tool = |t: &'static str, d: &'static str, s: f32| {
            let lead = mark(&p, tool_mark(t), 15.).unwrap_or_else(|| {
                div()
                    .w(px(15.))
                    .text_color(p.fg2)
                    .child("◍")
                    .into_any_element()
            });
            div()
                .h(px(42.))
                .px(px(12.))
                .flex()
                .items_center()
                .gap(px(10.))
                .rounded(px(12.))
                .border_1()
                .border_color(p.line)
                .child(lead)
                .child(
                    div()
                        .flex_none()
                        .font_family("monospace")
                        .text_size(px(12.5))
                        .child(t),
                )
                .child(
                    div()
                        .flex_1()
                        .min_w_0()
                        .overflow_hidden()
                        .whitespace_nowrap()
                        .text_ellipsis()
                        .text_color(p.mute)
                        .child(d),
                )
                .child(
                    div()
                        .w(px(15.))
                        .h(px(15.))
                        .rounded(px(8.))
                        .bg(tone(&p, IconTone::Running))
                        .text_color(p.card)
                        .text_size(px(10.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child("✓"),
                )
                .child(quiet(&p, format!("{s:.1}s")))
                .child(div().text_color(p.mute).child("⌄"))
        };
        let chat = if filled {
            let tl = progress_tools();
            div()
                .flex_1()
                .px(px(32.))
                .flex()
                .flex_col()
                .gap(px(14.))
                .child(bubble(&p, "Three people start Monday. Plan their first week and pair each with a buddy – the intake form is in Drive, the wiki in Notion."))
                .child(tool(tl[0].0, tl[0].1, tl[0].2))
                .child(para("I read the first-week pages and last spring's notes. The weeks that went well all had the same three parts: accounts on day one, one buddy, and a small task shipped by Friday."))
                .child(tool(tl[1].0, tl[1].1, tl[1].2))
                .child(div().font_weight(FontWeight::SEMIBOLD).child("Here is the week now:"))
                .child(para("• Monday: laptops, accounts and lunch with their buddy"))
                .child(para("• Tuesday to Thursday: one shadow day per team"))
                .child(para("• Friday: a small first task, shipped together"))
        } else if self.state == ScreenState::Loading {
            skel(&p, 9).px(px(32.))
        } else {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child("Ask for something that takes a few steps")
        };
        let mut col = div()
            .w(px(PROGRESS_COL_W as f32))
            .flex_none()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(6.))
            .border_l_1()
            .border_color(p.line)
            .child(
                div()
                    .flex()
                    .child(
                        div()
                            .flex_1()
                            .font_weight(FontWeight::MEDIUM)
                            .child("Progress ⌄"),
                    )
                    .child(div().text_color(p.mute).child("×")),
            );
        if filled || self.extra == "working" {
            col = col.child(quiet(&p, format!("{} earlier steps", PROGRESS_FOLDED)));
            for (i, l) in steps.iter().enumerate().skip(PROGRESS_FOLDED) {
                let k = step_state(i, done);
                col = col.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .min_h(px(26.))
                        .child(
                            div()
                                .w(px(17.))
                                .h(px(17.))
                                .rounded(px(9.))
                                .border_1()
                                .border_color(p.line)
                                .when(k == StepState::Done, |d| d.bg(p.mute)),
                        )
                        .child(
                            div()
                                .text_color(if k == StepState::Pending {
                                    p.mute
                                } else {
                                    p.fg2
                                })
                                .child(*l),
                        ),
                );
            }
            col = col
                .child(div().h(px(1.)).my(px(8.)).bg(p.line))
                .child(div().font_weight(FontWeight::MEDIUM).child("Outputs ⌄"));
            for (n, k, f) in progress_outputs() {
                col = col.child(
                    div()
                        .h(px(32.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(file_tile(&p, f))
                        .child(clip(n))
                        .child(quiet(&p, k)),
                );
            }
            let mut apps = div().flex().gap(px(5.));
            for a in ["notion", "googledrive", "slack"] {
                if let Some(e) = mark(&p, a, 14.) {
                    apps = apps.child(e);
                }
            }
            col = col
                .child(div().h(px(1.)).my(px(8.)).bg(p.line))
                .child(
                    div()
                        .font_weight(FontWeight::MEDIUM)
                        .child("Used in this session ⌄"),
                )
                .child(
                    div()
                        .h(px(32.))
                        .flex()
                        .items_center()
                        .child(clip("Uploads"))
                        .child(quiet(&p, "intake-form.pdf +1")),
                )
                .child(
                    div()
                        .h(px(32.))
                        .flex()
                        .items_center()
                        .child(clip("Connectors"))
                        .child(apps),
                )
                .child(
                    div()
                        .h(px(32.))
                        .flex()
                        .items_center()
                        .child(clip("Web search"))
                        .child(quiet(&p, "6 pages")),
                )
                .child(
                    div()
                        .h(px(32.))
                        .flex()
                        .items_center()
                        .child(clip("Skills"))
                        .child(quiet(&p, "onboarding-plan")),
                );
        } else if self.state == ScreenState::Loading {
            col = col.child(skel(&p, 10));
        }
        let main = div()
            .flex_1()
            .min_w_0()
            .flex()
            .child(page_bare(
                div().flex_1().flex().flex_col().child(chat).child(
                    div().px(px(32.)).pb(px(16.)).child(composer(
                        &p,
                        Cmp {
                            w: 780.,
                            placeholder: "Reply…",
                            open: self.extra,
                            ..Cmp::default()
                        },
                    )),
                ),
            ))
            .child(col);
        frame_w(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, false),
            main,
            None,
        )
    }
}

impl RenderOnce for ChatRunEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let filled = self.state == ScreenState::Sample;
        let plan = run_plan();
        let n = plan.len();
        let (done, status, c) = match self.extra {
            "input" => (3, "Needs you", tone(&p, IconTone::Approval)),
            "done" => (n, "Done", tone(&p, IconTone::Done)),
            _ => (1, "Running", tone(&p, IconTone::Running)),
        };
        let body = if filled {
            let mut uses = div()
                .flex()
                .items_center()
                .gap(px(6.))
                .child(quiet(&p, "Uses"));
            for m in run_marks(&plan) {
                if let Some(e) = mark(&p, m, 13.) {
                    uses = uses.child(
                        div()
                            .w(px(22.))
                            .h(px(22.))
                            .rounded(px(6.))
                            .border_1()
                            .border_color(p.line)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(e),
                    );
                }
            }
            let stat = |l: &'static str, v: String| {
                div()
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .child(quiet(&p, l))
                    .child(div().text_size(px(15.)).child(v))
            };
            let mut steps = div()
                .p(px(14.))
                .rounded(px(14.))
                .bg(p.inset)
                .flex()
                .flex_col()
                .child(
                    div()
                        .h(px(30.))
                        .flex()
                        .items_center()
                        .child(div().flex_1().font_weight(FontWeight::MEDIUM).child("Plan"))
                        .child(quiet(&p, format!("{n} steps"))),
                );
            for (i, s) in plan.iter().enumerate() {
                let k = if self.extra == "input" && i == 3 {
                    "wait"
                } else {
                    step_state(i, done).key()
                };
                let dot = div()
                    .w(px(18.))
                    .h(px(18.))
                    .rounded(px(9.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(10.));
                let dot = match k {
                    "done" => dot
                        .bg(tone(&p, IconTone::Running))
                        .text_color(p.card)
                        .child("✓"),
                    "wait" => dot
                        .bg(tone(&p, IconTone::Approval).opacity(0.22))
                        .text_color(tone(&p, IconTone::Approval))
                        .child("✋"),
                    "active" => dot.border_2().border_color(tone(&p, IconTone::Running)),
                    _ => dot
                        .border_1()
                        .border_color(p.line)
                        .text_color(p.mute)
                        .child((i + 1).to_string()),
                };
                steps = steps.child(
                    div()
                        .min_h(px(52.))
                        .py(px(6.))
                        .flex()
                        .gap(px(10.))
                        .child(dot)
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .gap(px(3.))
                                .child(clip(s.title.clone()))
                                .child(
                                    div().flex().gap(px(8.)).child(quiet(&p, s.why)).child(
                                        div()
                                            .px(px(6.))
                                            .rounded(px(5.))
                                            .bg(p.sel)
                                            .text_size(px(11.5))
                                            .child(s.tool),
                                    ),
                                ),
                        )
                        .child(if k == "wait" {
                            pill(&p, "Needs you", tone(&p, IconTone::Approval))
                        } else {
                            quiet(
                                &p,
                                if k == "done" {
                                    format!("{}.0s", s.secs)
                                } else {
                                    String::new()
                                },
                            )
                        }),
                );
            }
            steps = steps.child(
                div()
                    .mt(px(6.))
                    .pt(px(8.))
                    .border_t_1()
                    .border_color(p.line)
                    .child(quiet(&p, run_line(status, done, n))),
            );
            div()
                .flex_1()
                .w(px(680.))
                .mx_auto()
                .pt(px(22.))
                .flex()
                .flex_col()
                .gap(px(14.))
                .child(quiet(
                    &p,
                    format!("Member update · started by {PERSON} at 7:48 PM"),
                ))
                .child(
                    div()
                        .text_size(px(22.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Write the November member update"),
                )
                .child(
                    div().h(px(4.)).rounded(px(2.)).bg(p.sel).child(
                        div()
                            .h_full()
                            .w(px(680. * done as f32 / n as f32))
                            .rounded(px(2.))
                            .bg(tone(&p, IconTone::Running)),
                    ),
                )
                .child(uses)
                .child(
                    div()
                        .flex()
                        .child(stat("Steps", format!("{done} / {n}")))
                        .child(stat("Tool calls", (done * 2).to_string()))
                        .child(stat("Time", "0:07".into()))
                        .child(stat("Cost", "$0.04".into())),
                )
                .child(steps)
        } else if self.state == ScreenState::Loading {
            skel(&p, 10).p(px(24.))
        } else {
            div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .child("No runs yet")
        };
        let mut side = div().flex().flex_col().gap(px(8.)).p(px(14.));
        if self.extra == "input" && filled {
            side = side.child(
                div()
                    .p(px(14.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(tone(&p, IconTone::Approval).opacity(0.55))
                    .bg(tone(&p, IconTone::Approval).opacity(0.07))
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Send the member update?"),
                    )
                    .child(quiet(&p, "212 members get it by email."))
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(6.))
                            .child(
                                div()
                                    .h(px(30.))
                                    .px(px(10.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(9.))
                                    .border_1()
                                    .border_color(p.line)
                                    .child("Hold"),
                            )
                            .child(
                                div()
                                    .h(px(30.))
                                    .px(px(12.))
                                    .flex()
                                    .items_center()
                                    .rounded(px(9.))
                                    .bg(p.fg)
                                    .text_color(p.card)
                                    .child("Send it"),
                            ),
                    ),
            );
        } else {
            side = side.child(
                div()
                    .py(px(24.))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.))
                    .child(div().w(px(40.)).h(px(40.)).rounded(px(12.)).bg(p.sel))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Nothing needs you yet"),
                    )
                    .child(quiet(&p, "It stops here and asks first.")),
            );
        }
        if filled {
            side = side
                .child(quiet(&p, "Draft"))
                .child(
                    div()
                        .p(px(12.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                        .flex()
                        .flex_col()
                        .gap(px(10.))
                        .child(quiet(&p, "Member update – November"))
                        .child(para("In October membership grew 18% · we expect · by 12 December · 212 members · on Friday"))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(div().px(px(6.)).rounded(px(6.)).bg(p.sel).child("5"))
                                .child(div().flex_1())
                                .child(div().h(px(28.)).px(px(10.)).flex().items_center().rounded(px(9.)).border_1().border_color(p.line).child("Reject all"))
                                .child(div().h(px(28.)).px(px(10.)).flex().items_center().rounded(px(9.)).bg(p.fg).text_color(p.card).child("Accept all")),
                        ),
                );
        }
        let work = div()
            .flex_1()
            .flex()
            .flex_col()
            .child(work_tabs(&p, &[("Needs your input", true)]))
            .child(side);
        frame_w(
            self.theme,
            self.layout,
            sidebar(&p, self.state, &self.act, true),
            page_crumb(
                &p,
                "Runs",
                "Write the November member update",
                if filled {
                    Some(pill(&p, status, c))
                } else {
                    None
                },
                div()
                    .flex()
                    .gap(px(8.))
                    .child(
                        div()
                            .h(px(30.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .rounded(px(9.))
                            .border_1()
                            .border_color(p.line)
                            .child("Stop run"),
                    )
                    .child(div().text_color(p.mute).child("⋯")),
                body,
            ),
            Some((work, RUN_WORK_W as f32)),
        )
    }
}

chat_el!(
    /// AI Chat / Agents: the chat with the crew and the Orchestrator line; the brief in a 368 work area.
    ChatAgentsEl
);
chat_el!(
    /// AI Chat / Double Sidebar: a 52 rail, a 272 chats column, the thread and a 408 artifacts area.
    ChatRailEl
);
chat_el!(
    /// AI Chat / Progress: the chat beside an in-page 300 px Progress column.
    ChatProgressEl
);
chat_el!(
    /// AI Chat / Run: the run with its plan; Needs your input in a 348 work area.
    ChatRunEl
);
