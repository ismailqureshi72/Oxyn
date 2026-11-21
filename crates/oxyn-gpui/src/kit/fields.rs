//! Number entry, meters, uploads, composers and model menus for GPUI (plan #28, chunk 4b).
//! Elements are `RenderOnce` views of the `oxyn_core::kit::{fields, meters, upload, compose}`
//! models; the host keeps the model, drives the clock and routes [`Act`] ids (and row picks
//! through [`Pick`]) back to it. Motion that needs per-frame springs (strain, rubber band, wave
//! fills, clip-path morphs) is left to the host's animation loop.

use std::rc::Rc;

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative,
};
use oxyn_core::{
    ClockMs,
    kit::{
        compose::{BOLD, CODE, Doc, Effort, Kind, RingTone, RunSeg, RunStatus},
        fields::{NumberField, Scrub, UnfoldSelect, format_number},
        meters::{
            Band, Cat, CellField, ContextGrid, Share, Tone, UsageState, amount, gauge, pct1, split,
            tokens, usage,
        },
        upload::{Crop, Status, Uploads, ext, progress, size, time_left},
    },
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Weight, alpha, button, c, card, muted, tag};

/// Row pick callback: the row index and the window.
pub type Pick = Rc<dyn Fn(usize, &mut Window, &mut App)>;

fn tone(t: &'static Theme, k: Tone) -> Hsla {
    match k {
        Tone::Accent => c(t, ColorRole::FgDefault),
        Tone::Success => c(t, ColorRole::StatusSuccessFg),
        Tone::Warning => c(t, ColorRole::StatusWarningFg),
        Tone::Danger => c(t, ColorRole::StatusDangerFg),
    }
}

fn ramp(t: &'static Theme, i: usize) -> Hsla {
    let a = [0.88, 0.64, 0.46, 0.32, 0.22, 0.14][i % 6];
    alpha(c(t, ColorRole::FgDefault), a)
}

fn bar(t: &'static Theme, frac: f32, col: Hsla) -> impl IntoElement {
    div()
        .h(px(4.))
        .w_full()
        .rounded(px(999.))
        .bg(alpha(c(t, ColorRole::FgDefault), 0.1))
        .child(
            div()
                .h_full()
                .w(relative(frac.clamp(0.0, 1.0)))
                .rounded(px(999.))
                .bg(col),
        )
}

/// `number-field`: − value + with the limit note (host steps the model on "dec" / "inc").
#[derive(IntoElement)]
pub struct NumberFieldEl {
    f: NumberField,
    label: SharedString,
    prefix: String,
    suffix: String,
    now: ClockMs,
    act: Option<Act>,
    theme: &'static Theme,
}

impl NumberFieldEl {
    /// A field.
    pub fn new(f: NumberField, label: impl Into<SharedString>, now: ClockMs) -> Self {
        Self {
            f,
            label: label.into(),
            prefix: String::new(),
            suffix: String::new(),
            now,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Prefix and suffix.
    pub fn affix(mut self, prefix: &str, suffix: &str) -> Self {
        self.prefix = prefix.into();
        self.suffix = suffix.into();
        self
    }
    /// Button callback ("dec", "inc").
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for NumberFieldEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let note = self.f.limit_note(self.now, &self.prefix, &self.suffix);
        let v = format!(
            "{}{}{}",
            self.prefix,
            format_number(self.f.value, self.f.places(), true),
            self.suffix
        );
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .w(px(196.))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(12.5))
                            .font_weight(FontWeight::MEDIUM)
                            .child(self.label),
                    )
                    .when_some(note, |d, n| {
                        d.child(
                            div()
                                .text_size(px(11.5))
                                .text_color(c(t, ColorRole::StatusWarningFg))
                                .child(n),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(34.))
                    .rounded(px(9.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .child(button(
                        t,
                        "nf-dec",
                        "−",
                        Weight::Quiet,
                        "dec",
                        self.act.clone(),
                    ))
                    .child(
                        div()
                            .flex_1()
                            .flex()
                            .justify_center()
                            .text_size(px(13.5))
                            .child(v),
                    )
                    .child(button(t, "nf-inc", "+", Weight::Quiet, "inc", self.act)),
            )
    }
}

/// `scrub-input`: label, value, unit and the quiet track.
#[derive(IntoElement)]
pub struct ScrubInputEl {
    s: Scrub,
    label: SharedString,
    unit: SharedString,
    theme: &'static Theme,
}

impl ScrubInputEl {
    /// A scrub input.
    pub fn new(s: Scrub, label: impl Into<SharedString>, unit: impl Into<SharedString>) -> Self {
        Self {
            s,
            label: label.into(),
            unit: unit.into(),
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for ScrubInputEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .flex_col()
            .w(px(150.))
            .rounded(px(8.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(28.))
                    .px(px(8.))
                    .child(muted(t, self.label))
                    .child(div().flex_1().text_size(px(12.5)).child(self.s.text()))
                    .child(muted(t, self.unit)),
            )
            .when(self.s.bounded(), |d| {
                d.child(
                    div()
                        .h(px(2.))
                        .w(relative(self.s.fraction()))
                        .bg(alpha(c(t, ColorRole::FgDefault), 0.35)),
                )
            })
    }
}

/// `elastic-slider`: track, detents, thumb and tag; `over` stretches the track (host springs it).
#[derive(IntoElement)]
pub struct RubberSliderEl {
    value: f64,
    min: f64,
    max: f64,
    marks: Vec<(f64, String)>,
    over: f32,
    label: SharedString,
    theme: &'static Theme,
}

impl RubberSliderEl {
    /// A slider.
    pub fn new(label: impl Into<SharedString>, value: f64, min: f64, max: f64) -> Self {
        Self {
            value,
            min,
            max,
            marks: vec![],
            over: 0.0,
            label: label.into(),
            theme: &Theme::DARK,
        }
    }
    /// Detents.
    pub fn marks(mut self, m: Vec<(f64, String)>) -> Self {
        self.marks = m;
        self
    }
    /// Pull past an end, in px.
    pub fn over(mut self, px_: f32) -> Self {
        self.over = px_;
        self
    }
}

impl RenderOnce for RubberSliderEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let span = (self.max - self.min).max(f64::EPSILON);
        let f = ((self.value - self.min) / span) as f32;
        let (st, _) = oxyn_core::kit::fields::stretch(self.over);
        let w = 320.0 + st.abs();
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(
                div()
                    .flex()
                    .child(div().flex_1().text_size(px(12.5)).child(self.label))
                    .child(muted(t, format!("{:.0}", self.value))),
            )
            .child(
                div()
                    .relative()
                    .h(px(22.))
                    .w(px(w))
                    .child(
                        div()
                            .absolute()
                            .top(px(8.))
                            .h(px(6.))
                            .w_full()
                            .rounded(px(999.))
                            .bg(alpha(c(t, ColorRole::FgDefault), 0.1)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(8.))
                            .h(px(6.))
                            .w(relative(f))
                            .rounded(px(999.))
                            .bg(c(t, ColorRole::FgDefault)),
                    )
                    .children(self.marks.iter().map(|(m, _)| {
                        let x = ((m - self.min) / span) as f32 * w;
                        div()
                            .absolute()
                            .top(px(6.))
                            .left(px(x - 1.))
                            .w(px(2.))
                            .h(px(10.))
                            .bg(c(t, ColorRole::BorderStrong))
                    }))
                    .child(
                        div()
                            .absolute()
                            .top(px(3.))
                            .left(px(f * w - 8.))
                            .size(px(16.))
                            .rounded_full()
                            .bg(c(t, ColorRole::SurfaceRaised))
                            .border_1()
                            .border_color(c(t, ColorRole::BorderStrong)),
                    ),
            )
            .child(
                div().flex().justify_between().children(
                    self.marks
                        .iter()
                        .filter(|m| !m.1.is_empty())
                        .map(|m| muted(t, m.1.clone())),
                ),
            )
    }
}

/// `morph-select`: trigger and, when open, the list with the highlight row.
#[derive(IntoElement)]
pub struct UnfoldSelectEl {
    m: UnfoldSelect,
    placeholder: SharedString,
    act: Option<Act>,
    pick: Option<Pick>,
    theme: &'static Theme,
}

impl UnfoldSelectEl {
    /// A select.
    pub fn new(m: UnfoldSelect) -> Self {
        Self {
            m,
            placeholder: "Select".into(),
            act: None,
            pick: None,
            theme: &Theme::DARK,
        }
    }
    /// Trigger callback ("toggle") and row picks.
    pub fn on(mut self, act: Act, pick: Pick) -> Self {
        self.act = Some(act);
        self.pick = Some(pick);
        self
    }
}

impl RenderOnce for UnfoldSelectEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let lab = self.m.label(&self.placeholder);
        let act = self.act.clone();
        let mut root = div()
            .flex()
            .flex_col()
            .w(px(240.))
            .rounded(px(if self.m.open { 12. } else { 9. }))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .overflow_hidden();
        root = root.child(
            div()
                .id("ms-trigger")
                .flex()
                .items_center()
                .h(px(34.))
                .px(px(11.))
                .cursor_pointer()
                .child(div().flex_1().child(lab))
                .child(muted(t, "▾"))
                .when_some(act, |d, f| d.on_click(move |_, w, cx| f("toggle", w, cx))),
        );
        if self.m.open {
            for i in self.m.visible() {
                let o = self.m.options[i].clone();
                let pick = self.pick.clone();
                let hi = self.m.active == i;
                root = root.child(
                    div()
                        .id(("ms-opt", i))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .h(px(34.))
                        .px(px(9.))
                        .mx(px(4.))
                        .rounded(px(8.))
                        .when(hi, |d| d.bg(c(t, ColorRole::SurfaceHover)))
                        .text_color(c(
                            t,
                            if o.disabled {
                                ColorRole::FgSubtle
                            } else {
                                ColorRole::FgDefault
                            },
                        ))
                        .child(div().flex_1().child(o.label.clone()))
                        .child(muted(t, o.hint.clone()))
                        .when(self.m.selected == Some(i), |d| d.child("✓"))
                        .when_some(pick.filter(|_| !o.disabled), move |d, p| {
                            d.on_click(move |_, w, cx| p(i, w, cx))
                        }),
                );
            }
        }
        root
    }
}

/// `gauge`: a 270° ring of dots lit to the value, tone from the bands.
#[derive(IntoElement)]
pub struct GaugeEl {
    value: f64,
    min: f64,
    max: f64,
    bands: Vec<Band>,
    label: SharedString,
    theme: &'static Theme,
}

impl GaugeEl {
    /// A gauge.
    pub fn new(value: f64, min: f64, max: f64, label: impl Into<SharedString>) -> Self {
        Self {
            value,
            min,
            max,
            bands: vec![],
            label: label.into(),
            theme: &Theme::DARK,
        }
    }
    /// Threshold bands.
    pub fn bands(mut self, b: Vec<Band>) -> Self {
        self.bands = b;
        self
    }
}

impl RenderOnce for GaugeEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let g = gauge(self.value, self.min, self.max, Tone::Accent, &self.bands);
        let col = tone(t, g.tone);
        let n = 36;
        let r = 58.0f32;
        div()
            .relative()
            .size(px(140.))
            .children((0..n).map(|i| {
                let f = i as f32 / (n - 1) as f32;
                let a = (135.0 + 270.0 * f).to_radians();
                let (x, y) = (70.0 + r * a.cos() - 3.5, 70.0 + r * a.sin() - 3.5);
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .size(px(7.))
                    .rounded_full()
                    .bg(if f <= g.frac {
                        col
                    } else {
                        alpha(c(t, ColorRole::FgDefault), 0.1)
                    })
            }))
            .child(
                div()
                    .absolute()
                    .top(px(46.))
                    .w_full()
                    .flex()
                    .flex_col()
                    .items_center()
                    .child(
                        div()
                            .text_size(px(26.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("{}%", g.pct)),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(col)
                            .child(g.band.unwrap_or_else(|| self.label.to_string())),
                    ),
            )
    }
}

/// `usage-meter`: segments, overage, badge and legend.
#[derive(IntoElement)]
pub struct UsageMeterEl {
    title: SharedString,
    shares: Vec<Share>,
    limit: f64,
    unit: String,
    theme: &'static Theme,
}

impl UsageMeterEl {
    /// A meter.
    pub fn new(title: impl Into<SharedString>, shares: Vec<Share>, limit: f64, unit: &str) -> Self {
        Self {
            title: title.into(),
            shares,
            limit,
            unit: unit.into(),
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for UsageMeterEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let u = usage(&self.shares, self.limit, 0.9);
        let (badge, role) = match u.state {
            UsageState::Ok => (format!("{:.0}% used", u.used / self.limit * 100.0), None),
            UsageState::Almost => ("Almost full".to_string(), Some(ColorRole::StatusWarningFg)),
            UsageState::Over => (
                format!("Over limit by {}", amount(u.over, 1, &self.unit)),
                Some(ColorRole::StatusDangerFg),
            ),
        };
        let mut b = div()
            .relative()
            .h(px(12.))
            .w(px(420.))
            .rounded(px(999.))
            .overflow_hidden()
            .bg(alpha(c(t, ColorRole::FgDefault), 0.07));
        for (i, (x, w)) in u.segs.iter().enumerate() {
            b = b.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(relative(*x))
                    .w(relative(*w))
                    .bg(ramp(t, i)),
            );
        }
        if u.over > 0.0 {
            b = b.child(
                div()
                    .absolute()
                    .top_0()
                    .bottom_0()
                    .left(relative(u.limit_at))
                    .w(px(2.))
                    .bg(c(t, ColorRole::StatusDangerFg)),
            );
        }
        card(t)
            .child(
                div()
                    .flex()
                    .gap(px(10.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(self.title))
                    .child(muted(
                        t,
                        format!(
                            "{} of {}",
                            amount(u.used, 1, &self.unit),
                            amount(self.limit, 1, &self.unit)
                        ),
                    ))
                    .child(div().flex_1())
                    .child(tag(t, badge, role)),
            )
            .child(b)
            .child(div().flex().flex_wrap().gap(px(10.)).children(
                self.shares.iter().enumerate().map(|(i, s)| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(5.))
                        .child(div().size(px(9.)).rounded(px(3.)).bg(ramp(t, i)))
                        .child(div().text_size(px(12.)).child(s.label.clone()))
                        .child(muted(t, amount(s.value, 1, &self.unit)))
                }),
            ))
    }
}

/// `split-bar`: shares with the focused one widened.
#[derive(IntoElement)]
pub struct SplitBarEl {
    shares: Vec<Share>,
    hidden: Vec<String>,
    focus: Option<usize>,
    theme: &'static Theme,
}

impl SplitBarEl {
    /// A split bar.
    pub fn new(shares: Vec<Share>) -> Self {
        Self {
            shares,
            hidden: vec![],
            focus: None,
            theme: &Theme::DARK,
        }
    }
    /// Hidden ids and the focused segment.
    pub fn state(mut self, hidden: Vec<String>, focus: Option<usize>) -> Self {
        self.hidden = hidden;
        self.focus = focus;
        self
    }
}

impl RenderOnce for SplitBarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let l = split(&self.shares, &self.hidden, self.focus, 0.045, 0.16);
        div()
            .flex()
            .h(px(34.))
            .w(px(460.))
            .rounded(px(9.))
            .overflow_hidden()
            .children(
                l.iter()
                    .enumerate()
                    .filter(|(_, x)| x.1 > 0.0)
                    .map(|(i, (_, w, sh))| {
                        div()
                            .h_full()
                            .w(relative(*w))
                            .bg(ramp(t, i))
                            .flex()
                            .items_center()
                            .px(px(8.))
                            .text_size(px(12.))
                            .text_color(c(
                                t,
                                if i < 3 {
                                    ColorRole::BgBase
                                } else {
                                    ColorRole::FgDefault
                                },
                            ))
                            .when(*w >= 0.13, |d| d.child(pct1(*sh)))
                            .when(self.focus.is_some() && self.focus != Some(i), |d| {
                                d.opacity(0.5)
                            })
                    }),
            )
    }
}

/// `context-field`: the affectUI cell field.
#[derive(IntoElement)]
pub struct ContextFieldEl {
    f: CellField,
    cells: usize,
    columns: usize,
    hover: Option<usize>,
    theme: &'static Theme,
}

impl ContextFieldEl {
    /// A field.
    pub fn new(f: CellField) -> Self {
        Self {
            f,
            cells: 216,
            columns: 36,
            hover: None,
            theme: &Theme::DARK,
        }
    }
    /// Lights one part.
    pub fn hover(mut self, p: Option<usize>) -> Self {
        self.hover = p;
        self
    }
}

impl RenderOnce for ContextFieldEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let g = self.f.cells(self.cells);
        let rows: Vec<Vec<Option<usize>>> = g
            .chunks(self.columns)
            .map(<[Option<usize>]>::to_vec)
            .collect();
        card(t)
            .child(
                div()
                    .flex()
                    .child(
                        div()
                            .flex_1()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Context window"),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("{:.0}%", self.f.share() * 100.0)),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .children(rows.into_iter().map(|r| {
                        div().flex().gap(px(2.)).children(r.into_iter().map(|cell| {
                            let col = match cell {
                                Some(p) => ramp(t, p),
                                None => alpha(c(t, ColorRole::FgDefault), 0.07),
                            };
                            div()
                                .size(px(10.))
                                .rounded(px(2.))
                                .bg(col)
                                .when(self.hover.is_some() && cell != self.hover, |d| {
                                    d.opacity(0.25)
                                })
                        }))
                    })),
            )
            .child(muted(t, self.f.tray()))
    }
}

/// `context-window-grid`: the Arc grid with its threshold row.
#[derive(IntoElement)]
pub struct ContextGridEl {
    g: ContextGrid,
    theme: &'static Theme,
}

impl ContextGridEl {
    /// A grid.
    pub fn new(g: ContextGrid) -> Self {
        Self {
            g,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for ContextGridEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let cells = self.g.cells(200);
        let th = self.g.threshold_cell(200) / 20;
        let shade = |c_: Option<Cat>| -> f32 {
            match c_ {
                Some(Cat::System) => 0.9,
                Some(Cat::Tools) => 0.68,
                Some(Cat::Files) => 0.48,
                Some(Cat::Summary) => 0.4,
                Some(Cat::Messages) => 0.3,
                None => 0.07,
            }
        };
        card(t)
            .child(muted(
                t,
                format!(
                    "{} of {} · {} free",
                    tokens(self.g.used()),
                    tokens(self.g.window),
                    tokens(self.g.free())
                ),
            ))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .children(cells.chunks(20).enumerate().map(|(ri, r)| {
                        div()
                            .flex()
                            .gap(px(2.))
                            .when(ri == th, |d| {
                                d.border_t_1()
                                    .border_color(c(t, ColorRole::StatusWarningFg))
                            })
                            .children(r.iter().map(|(cat, res)| {
                                div().size(px(14.)).rounded(px(2.)).bg(alpha(
                                    c(t, ColorRole::FgDefault),
                                    if *res { 0.03 } else { shade(*cat) },
                                ))
                            }))
                    })),
            )
            .children(self.g.last.as_ref().map(|l| {
                muted(
                    t,
                    format!(
                        "{} {} messages into {}",
                        if l.automatic {
                            "Auto-compacted"
                        } else {
                            "Compacted"
                        },
                        l.folded,
                        tokens(l.summary)
                    ),
                )
            }))
    }
}

/// `file-dropzone`: the target and its rows.
#[derive(IntoElement)]
pub struct FileDropzoneEl {
    u: Uploads,
    now: ClockMs,
    act: Option<Act>,
    theme: &'static Theme,
}

impl FileDropzoneEl {
    /// A dropzone.
    pub fn new(u: Uploads, now: ClockMs) -> Self {
        Self {
            u,
            now,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Browse callback ("browse").
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for FileDropzoneEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let mut u = self.u.clone();
        u.tick(self.now);
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .w(px(420.))
            .child(
                div()
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.))
                    .p(px(22.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderStrong))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Add files"))
                    .child(muted(t, u.hint()))
                    .child(button(
                        t,
                        "dz-browse",
                        "Browse",
                        Weight::Plain,
                        "browse",
                        self.act,
                    )),
            )
            .children(u.rows.iter().map(|r| {
                let p = progress(r, self.now);
                let line = match r.status {
                    Status::Done => format!("{} · Uploaded", size(r.bytes)),
                    Status::Failed => r.error.clone().unwrap_or_default(),
                    _ => time_left(p, self.now.since(r.started).as_millis() as u64)
                        .unwrap_or_else(|| size(r.bytes)),
                };
                let failed = r.status == Status::Failed;
                card(t).child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .child(tag(t, ext(&r.name), None))
                        .child(
                            div()
                                .flex_1()
                                .flex()
                                .flex_col()
                                .gap(px(3.))
                                .child(div().text_size(px(12.5)).child(r.name.clone()))
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(c(
                                            t,
                                            if failed {
                                                ColorRole::StatusDangerFg
                                            } else {
                                                ColorRole::FgMuted
                                            },
                                        ))
                                        .child(line),
                                )
                                .when(r.status == Status::Uploading, |d| {
                                    d.child(bar(t, p, c(t, ColorRole::FgDefault)))
                                }),
                        ),
                )
            }))
    }
}

/// `avatar-upload`: the avatar or crop circle with the progress ring.
#[derive(IntoElement)]
pub struct AvatarUploadEl {
    initials: SharedString,
    crop: Option<Crop>,
    progress: Option<f32>,
    theme: &'static Theme,
}

impl AvatarUploadEl {
    /// An avatar.
    pub fn new(initials: impl Into<SharedString>) -> Self {
        Self {
            initials: initials.into(),
            crop: None,
            progress: None,
            theme: &Theme::DARK,
        }
    }
    /// Cropping (the 184 px circle) and upload progress.
    pub fn state(mut self, crop: Option<Crop>, progress: Option<f32>) -> Self {
        self.crop = crop;
        self.progress = progress;
        self
    }
}

impl RenderOnce for AvatarUploadEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let d = if self.crop.is_some() { 184.0 } else { 88.0 };
        let n = 32;
        div()
            .flex()
            .items_center()
            .gap(px(16.))
            .child(
                div()
                    .relative()
                    .size(px(d + 12.))
                    .child(
                        div()
                            .absolute()
                            .left(px(6.))
                            .top(px(6.))
                            .size(px(d))
                            .rounded_full()
                            .bg(c(t, ColorRole::SurfaceHover))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(22.))
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(self.initials),
                    )
                    .children(self.progress.into_iter().flat_map(|p| {
                        (0..n).map(move |i| {
                            let f = i as f32 / n as f32;
                            let a = (-90.0 + 360.0 * f).to_radians();
                            let r = d / 2.0 + 3.0;
                            let (x, y) = (
                                d / 2.0 + 6.0 + r * a.cos() - 2.0,
                                d / 2.0 + 6.0 + r * a.sin() - 2.0,
                            );
                            div()
                                .absolute()
                                .left(px(x))
                                .top(px(y))
                                .size(px(4.))
                                .rounded_full()
                                .bg(if f < p {
                                    c(t, ColorRole::FgDefault)
                                } else {
                                    alpha(c(t, ColorRole::FgDefault), 0.12)
                                })
                        })
                    })),
            )
            .when(self.crop.is_none(), |x| {
                x.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child("Profile photo"),
                        )
                        .child(muted(t, "PNG, JPG, WEBP up to 5 MB")),
                )
            })
            .when_some(self.crop, |x, c_| {
                x.child(muted(t, format!("Zoom {:.2}×", c_.zoom)))
            })
    }
}

/// `rich-text-editor`: blocks and marks of a [`Doc`] (editing goes through `ImeInput`).
#[derive(IntoElement)]
pub struct RichTextEl {
    doc: Doc,
    theme: &'static Theme,
}

impl RichTextEl {
    /// A document view.
    pub fn new(doc: Doc) -> Self {
        Self {
            doc,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for RichTextEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let mut num = 0;
        card(t).children(self.doc.lines.iter().map(|l| {
            num = if l.kind == Kind::Numbered { num + 1 } else { 0 };
            let (size_, weight) = match l.kind {
                Kind::H1 => (22., FontWeight::BOLD),
                Kind::H2 => (18., FontWeight::SEMIBOLD),
                Kind::H3 => (15., FontWeight::SEMIBOLD),
                _ => (13.5, FontWeight::NORMAL),
            };
            let lead = match l.kind {
                Kind::Bullet => "•".to_string(),
                Kind::Numbered => format!("{num}."),
                Kind::Todo => {
                    if l.checked {
                        "☑".into()
                    } else {
                        "☐".into()
                    }
                }
                Kind::Quote => "▍".into(),
                _ => String::new(),
            };
            div()
                .flex()
                .gap(px(6.))
                .pl(px(l.indent as f32 * 20.))
                .text_size(px(size_))
                .font_weight(weight)
                .when(l.kind == Kind::Code, |d| {
                    d.p(px(8.))
                        .rounded(px(8.))
                        .bg(alpha(c(t, ColorRole::FgDefault), 0.06))
                })
                .when(!lead.is_empty(), |d| d.child(muted(t, lead.clone())))
                .when(l.kind == Kind::Divider, |d| {
                    d.child(div().flex_1().h(px(1.)).bg(c(t, ColorRole::BorderStrong)))
                })
                .children(l.runs().into_iter().map(|(text, m, link)| {
                    div()
                        .when(m & BOLD != 0, |d| d.font_weight(FontWeight::BOLD))
                        .when(m & CODE != 0, |d| {
                            d.px(px(2.)).bg(alpha(c(t, ColorRole::FgDefault), 0.09))
                        })
                        .when(link > 0, |d| d.underline())
                        .child(text)
                }))
        }))
    }
}

/// `ai-composer`: tokens, text, ring counter and the send / stop button.
#[derive(IntoElement)]
pub struct AiComposerEl {
    tokens: Vec<(String, bool)>,
    text: SharedString,
    ring: Option<(f32, RingTone)>,
    streaming: Option<f32>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl AiComposerEl {
    /// A composer: tokens as (label, is_command), the trailing text.
    pub fn new(tokens: Vec<(String, bool)>, text: impl Into<SharedString>) -> Self {
        Self {
            tokens,
            text: text.into(),
            ring: None,
            streaming: None,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Ring counter and stream progress (the button becomes Stop).
    pub fn state(mut self, ring: Option<(f32, RingTone)>, streaming: Option<f32>) -> Self {
        self.ring = ring;
        self.streaming = streaming;
        self
    }
    /// "send" / "stop".
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for AiComposerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let ring = self.ring.map(|(f, k)| {
            let role = match k {
                RingTone::Quiet => ColorRole::FgMuted,
                RingTone::Warn => ColorRole::StatusWarningFg,
                RingTone::Over => ColorRole::StatusDangerFg,
            };
            div()
                .text_size(px(11.))
                .text_color(c(t, role))
                .child(format!("{:.0}%", f * 100.0))
        });
        let (label, action) = match self.streaming {
            Some(p) => (format!("■ {:.0}%", p * 100.0), "stop"),
            None => ("↑".to_string(), "send"),
        };
        card(t)
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.))
                    .children(self.tokens.into_iter().map(|(l, cmd)| {
                        tag(
                            t,
                            if cmd {
                                format!("/{l}")
                            } else {
                                format!("@{l}")
                            },
                            None,
                        )
                    }))
                    .child(div().child(self.text)),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(tag(t, "Model", None))
                    .child(div().flex_1())
                    .children(ring)
                    .child(button(
                        t,
                        "ai-send",
                        label,
                        Weight::Primary,
                        action,
                        self.act,
                    )),
            )
    }
}

/// `run-controls`: status, steered answer, queue, mode, permission and context.
#[derive(IntoElement)]
pub struct RunControlsEl {
    status: RunStatus,
    prompt: SharedString,
    segs: Vec<RunSeg>,
    queue: Vec<String>,
    act_mode: bool,
    full_access: bool,
    context: f32,
    act: Option<Act>,
    theme: &'static Theme,
}

impl RunControlsEl {
    /// A run.
    pub fn new(status: RunStatus, prompt: impl Into<SharedString>, segs: Vec<RunSeg>) -> Self {
        Self {
            status,
            prompt: prompt.into(),
            segs,
            queue: vec![],
            act_mode: true,
            full_access: false,
            context: 0.4,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Queue, Act mode, full access and context share.
    pub fn state(
        mut self,
        queue: Vec<String>,
        act_mode: bool,
        full_access: bool,
        context: f32,
    ) -> Self {
        self.queue = queue;
        self.act_mode = act_mode;
        self.full_access = full_access;
        self.context = context;
        self
    }
    /// The run's buttons: "stop", "resume", "retry", "dismiss", "send".
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for RunControlsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let a = self.act;
        let ctx_role = if self.context >= 0.8 {
            ColorRole::StatusWarningFg
        } else {
            ColorRole::FgMuted
        };
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .w(px(520.))
            .child(
                div()
                    .flex()
                    .child(
                        div()
                            .flex_1()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.prompt),
                    )
                    .child(muted(t, format!("{:?}", self.status))),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(2.))
                    .children(self.segs.into_iter().map(|s| {
                        match s {
                            RunSeg::Text(x) => div().child(x),
                            RunSeg::Steer(x) => div()
                                .px(px(6.))
                                .rounded(px(6.))
                                .border_1()
                                .border_color(c(t, ColorRole::BorderStrong))
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(format!("↳ {x}")),
                        }
                    })),
            )
            .child(
                div().flex().gap(px(6.)).children(
                    self.status
                        .actions()
                        .iter()
                        .map(|x| button(t, *x, *x, Weight::Plain, x, a.clone())),
                ),
            )
            .children(
                self.queue
                    .into_iter()
                    .map(|q| card(t).child(muted(t, format!("Queued · {q}")))),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .child(tag(t, if self.act_mode { "Act" } else { "Chat" }, None))
                    .when(self.act_mode, |d| {
                        d.child(tag(
                            t,
                            if self.full_access {
                                "Full access"
                            } else {
                                "Ask first"
                            },
                            self.full_access.then_some(ColorRole::StatusDangerFg),
                        ))
                    })
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(px(11.5))
                            .text_color(c(t, ctx_role))
                            .child(format!("{:.0}%", self.context * 100.0)),
                    )
                    .child(button(t, "rc-send", "↑", Weight::Primary, "send", a)),
            )
    }
}

/// `model-menu`: rows with strength bars, the thinking switch and effort.
#[derive(IntoElement)]
pub struct ModelMenuEl {
    rows: Vec<(String, String, Option<[u8; 3]>)>,
    hi: usize,
    thinking: Option<Effort>,
    pick: Option<Pick>,
    theme: &'static Theme,
}

impl ModelMenuEl {
    /// Rows: name, description and (speed, quality, cost) or `None` for a router.
    pub fn new(rows: Vec<(String, String, Option<[u8; 3]>)>, hi: usize) -> Self {
        Self {
            rows,
            hi,
            thinking: None,
            pick: None,
            theme: &Theme::DARK,
        }
    }
    /// Thinking on at this effort.
    pub fn thinking(mut self, e: Option<Effort>) -> Self {
        self.thinking = e;
        self
    }
    /// Row picks.
    pub fn on_pick(mut self, p: Pick) -> Self {
        self.pick = Some(p);
        self
    }
}

impl RenderOnce for ModelMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let bars = move |v: u8| {
            div().flex().gap(px(2.)).children((1..=5u8).map(move |i| {
                div().w(px(5.)).h(px(8.)).rounded(px(1.5)).bg(if i <= v {
                    c(t, ColorRole::FgDefault)
                } else {
                    alpha(c(t, ColorRole::FgDefault), 0.14)
                })
            }))
        };
        card(t)
            .w(px(300.))
            .children(self.rows.into_iter().enumerate().map(|(i, (n, d, s))| {
                let p = self.pick.clone();
                div()
                    .id(("mm", i))
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .p(px(8.))
                    .rounded(px(9.))
                    .cursor_pointer()
                    .when(i == self.hi, |x| x.bg(c(t, ColorRole::SurfaceHover)))
                    .child(div().font_weight(FontWeight::MEDIUM).child(n))
                    .child(muted(t, d))
                    .when(i == self.hi, |x| {
                        x.child(match s {
                            Some([sp, q, co]) => div()
                                .flex()
                                .gap(px(10.))
                                .text_size(px(11.))
                                .child(div().flex().gap(px(4.)).child("Speed").child(bars(sp)))
                                .child(div().flex().gap(px(4.)).child("Quality").child(bars(q)))
                                .child(div().flex().gap(px(4.)).child("Cost").child(bars(co))),
                            None => div().text_size(px(11.)).child("Varies"),
                        })
                    })
                    .when_some(p, move |x, p| x.on_click(move |_, w, cx| p(i, w, cx)))
            }))
            .child(muted(
                t,
                match self.thinking {
                    Some(e) => format!("Extended thinking · {}", e.label()),
                    None => "Extended thinking off".into(),
                },
            ))
    }
}

/// `model-pill`: the pill and its summary.
#[derive(IntoElement)]
pub struct ModelPillEl {
    name: SharedString,
    provider: SharedString,
    effort: Effort,
    summary: Option<Vec<(String, String)>>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ModelPillEl {
    /// A pill.
    pub fn new(
        name: impl Into<SharedString>,
        provider: impl Into<SharedString>,
        effort: Effort,
    ) -> Self {
        Self {
            name: name.into(),
            provider: provider.into(),
            effort,
            summary: None,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Summary rows (label, value).
    pub fn summary(mut self, rows: Vec<(String, String)>) -> Self {
        self.summary = Some(rows);
        self
    }
    /// "open".
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ModelPillEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let mark: String = self
            .provider
            .chars()
            .next()
            .map(|c_| c_.to_uppercase().collect())
            .unwrap_or_default();
        let a = self.act;
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .child(
                div()
                    .id("mp-pill")
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .h(px(32.))
                    .pl(px(5.))
                    .pr(px(10.))
                    .rounded(px(999.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .cursor_pointer()
                    .child(
                        div()
                            .size(px(22.))
                            .rounded_full()
                            .bg(c(t, ColorRole::FgDefault))
                            .text_color(c(t, ColorRole::BgBase))
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(10.5))
                            .map(|d| {
                                match crate::brand::brand_icon(
                                    &self.provider,
                                    12.,
                                    c(t, ColorRole::BgBase),
                                    true,
                                ) {
                                    Some(icon) => d.child(icon),
                                    None => d.child(mark),
                                }
                            }),
                    )
                    .child(div().font_weight(FontWeight::MEDIUM).child(self.name))
                    .child(muted(t, self.effort.label()))
                    .when_some(a, |d, f| d.on_click(move |_, w, cx| f("open", w, cx))),
            )
            .when_some(self.summary, |d, rows| {
                d.child(card(t).children(rows.into_iter().map(|(k, v)| {
                    div()
                        .flex()
                        .gap(px(12.))
                        .child(muted(t, k))
                        .child(div().text_size(px(12.)).child(v))
                })))
            })
    }
}
