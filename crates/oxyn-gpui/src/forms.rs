//! Forms, pickers and flows for GPUI (plan #25), drawn from the same `oxyn_core::primitives`
//! models as the Dioxus components. Elements are `RenderOnce`: the host owns the model, routes
//! key events to the model's `key` methods, feeds the shared clock and re-renders. Rows, chips,
//! days and options take an index (or date) listener so clicks reach the host.
//!
//! Gaps against Dioxus (listed in the PR): text entry is display-only until the shared input
//! handler lands (the host forwards characters to the model); popovers render in flow under their
//! trigger; no per-element `FocusHandle`; the time dial is built from positioned dots rather
//! than SVG arcs; reel entries fade and offset but do not scale their glyphs.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use std::rc::Rc;

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, Role, SharedString,
    Styled, Window, div, prelude::*, px,
};
use oxyn_core::{
    ClockMs,
    primitives::{
        combobox::{ComboRow, Combobox},
        date::{
            Calendar, Date, DatePicker, DayCell, HourCycle, RangePicker, TimeOfDay, time_options,
        },
        dial::{DateWheel, TimeDial},
        filter_menu::{FilterMenu, FilterRow},
        inline_edit::{EditPhase, InlineEdit},
        otp::{Otp, OtpStatus},
        password::{Level, Rule, evaluate},
        phone::{COUNTRIES, CountryRow, PhoneInput, PhoneStatus},
        questions::{Answer, DeckMode, DeckPhase, QuestionDeck, QuestionKind},
        smart_fill::{FieldMark, SmartFill},
        stepper::{FieldKind, Stepper, StepperPhase},
        tags::TagInput,
    },
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

type OnIndex = Rc<dyn Fn(usize, &mut Window, &mut App)>;
type OnDate = Rc<dyn Fn(Date, &mut Window, &mut App)>;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

fn label_row(t: &'static Theme, s: impl Into<SharedString>) -> impl IntoElement {
    div()
        .text_size(px(12.5))
        .font_weight(FontWeight::MEDIUM)
        .text_color(c(t, ColorRole::FgDefault))
        .child(s.into())
}

fn shell(t: &'static Theme, focused: bool, invalid: bool) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(6.))
        .min_h(px(34.))
        .px(px(10.))
        .rounded(px(8.))
        .bg(c(t, ColorRole::SurfaceRaised))
        .border_1()
        .border_color(if invalid {
            c(t, ColorRole::StatusDangerFg)
        } else if focused {
            c(t, ColorRole::FgMuted)
        } else {
            c(t, ColorRole::BorderDefault)
        })
}

fn chip(t: &'static Theme, text: impl Into<SharedString>, marked: bool) -> gpui::Div {
    div()
        .flex()
        .items_center()
        .gap(px(4.))
        .h(px(24.))
        .pl(px(8.))
        .pr(px(4.))
        .rounded(px(6.))
        .bg(c(t, ColorRole::SurfaceHover))
        .text_size(px(12.5))
        .when(marked, |d| {
            d.border_1().border_color(c(t, ColorRole::Focus))
        })
        .child(text.into())
}

fn message(t: &'static Theme, err: Option<String>, hint: Option<String>) -> Option<AnyElement> {
    let (txt, role) = match (err, hint) {
        (Some(e), _) => (e, ColorRole::StatusDangerFg),
        (None, Some(h)) => (h, ColorRole::FgMuted),
        _ => return None,
    };
    Some(
        div()
            .text_size(px(12.))
            .text_color(c(t, role))
            .child(txt)
            .into_any_element(),
    )
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

// ---------------------------------------------------------------------------------------------
// Tag input

/// `tag-input`: chips before the draft, a counter and the message.
#[derive(IntoElement)]
pub struct TagInputEl {
    label: SharedString,
    model: TagInput,
    placeholder: SharedString,
    on_remove: Option<OnIndex>,
    theme: &'static Theme,
}

impl TagInputEl {
    /// From the model.
    pub fn new(label: impl Into<SharedString>, model: TagInput) -> Self {
        Self {
            label: label.into(),
            model,
            placeholder: "Add…".into(),
            on_remove: None,
            theme: &Theme::DARK,
        }
    }
    /// A chip's × was clicked (the host calls `TagInput::remove`).
    pub fn on_remove(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_remove = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for TagInputEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let (shown, extra) = m.shown();
        let rm = self.on_remove.clone();
        let draft = if m.draft.is_empty() {
            div()
                .text_color(c(t, ColorRole::FgSubtle))
                .child(m.placeholder(&self.placeholder).to_string())
        } else {
            div().child(m.draft.clone())
        };
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(label_row(t, self.label.clone()))
            .child(
                shell(t, m.picked.is_some(), m.message.is_some())
                    .id("oxyn-a11y")
                    .role(Role::TextInput)
                    .aria_label(self.label)
                    .flex_wrap()
                    .py(px(4.))
                    .children(m.tags.iter().enumerate().take(shown).map(|(i, tag)| {
                        let rm = rm.clone();
                        chip(t, tag.text.clone(), m.picked == Some(i))
                            .when(m.flash.is_some_and(|f| f.0 == i), |d| {
                                d.bg(c(t, ColorRole::SurfaceSelected))
                            })
                            .child(
                                div()
                                    .id(("tag-rm", i))
                                    .role(Role::Button)
                                    .aria_label(SharedString::from(format!("Remove {}", tag.text)))
                                    .px(px(3.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child("×")
                                    .when_some(rm, |d, f| {
                                        d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                                    }),
                            )
                    }))
                    .when(extra > 0, |d| d.child(chip(t, format!("+{extra}"), false)))
                    .child(draft),
            )
            .child(
                div()
                    .flex()
                    .justify_between()
                    .children(message(t, m.message.clone(), None))
                    .children(m.counter().map(|s| {
                        div()
                            .text_size(px(11.5))
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(s)
                    })),
            )
    }
}

// ---------------------------------------------------------------------------------------------
// Combobox / multi-select

/// `combobox` / `multi-select`: chips, the query and the grouped list.
#[derive(IntoElement)]
pub struct ComboboxEl {
    label: Option<SharedString>,
    model: Combobox,
    placeholder: SharedString,
    on_pick: Option<OnIndex>,
    theme: &'static Theme,
}

impl ComboboxEl {
    /// From the model.
    pub fn new(model: Combobox) -> Self {
        Self {
            label: None,
            model,
            placeholder: "Search…".into(),
            on_pick: None,
            theme: &Theme::DARK,
        }
    }
    /// Label.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
    /// Placeholder.
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = p.into();
        self
    }
    /// A row was clicked: the index into [`Combobox::rows`] (the host calls `hover` + `key(Enter)`).
    pub fn on_pick(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for ComboboxEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let (chips, extra) = m.chips();
        let rows = m.rows();
        let pick = self.on_pick.clone();
        let input = if m.query.is_empty() {
            let ph = if !m.multiple {
                m.selected()
                    .first()
                    .map(|o| o.label.clone())
                    .unwrap_or(self.placeholder.to_string())
            } else if m.value.is_empty() {
                self.placeholder.to_string()
            } else {
                String::new()
            };
            div().text_color(c(t, ColorRole::FgSubtle)).child(ph)
        } else {
            div().child(m.query.clone())
        };
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .min_w(px(260.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .when_some(self.label, |d, l| d.child(label_row(t, l)))
            .child(
                shell(t, m.open, false)
                    .id("oxyn-a11y")
                    .role(Role::ComboBox)
                    .aria_expanded(m.open)
                    .flex_wrap()
                    .py(px(4.))
                    .when(m.multiple, |d| {
                        d.children(chips.iter().enumerate().map(|(i, o)| {
                            chip(t, o.label.clone(), m.chip_marked == Some(i)).child(
                                div()
                                    .px(px(3.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child("×"),
                            )
                        }))
                        .when(extra > 0, |d| d.child(chip(t, format!("+{extra}"), false)))
                    })
                    .child(input.flex_1())
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(if m.open { "⌃" } else { "⌄" }),
                    ),
            )
            .when(m.open, |d| {
                d.child(
                    div()
                        .id("oxyn-a11y")
                        .role(Role::ListBox)
                        .flex()
                        .flex_col()
                        .p(px(4.))
                        .rounded(px(10.))
                        .bg(c(t, ColorRole::SurfaceOverlay))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .children(rows.into_iter().enumerate().map(|(ri, r)| {
                            let active = m.active == Some(ri);
                            let base = || {
                                div()
                                    .id(("combo-row", ri))
                                    .role(Role::ListBoxOption)
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .min_h(px(32.))
                                    .px(px(8.))
                                    .rounded(px(6.))
                                    .when(active, |d| d.bg(c(t, ColorRole::SurfaceHover)))
                            };
                            let clickable = |d: gpui::Stateful<gpui::Div>| match pick.clone() {
                                Some(f) => {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(ri, w, cx))
                                }
                                None => d,
                            };
                            match r {
                                ComboRow::Group(g) => div()
                                    .px(px(8.))
                                    .pt(px(6.))
                                    .text_size(px(11.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(g.to_uppercase())
                                    .into_any_element(),
                                ComboRow::Loading => div()
                                    .h(px(32.))
                                    .px(px(8.))
                                    .flex()
                                    .items_center()
                                    .child(
                                        div()
                                            .w(px(120.))
                                            .h(px(10.))
                                            .rounded(px(4.))
                                            .bg(c(t, ColorRole::SurfaceHover)),
                                    )
                                    .into_any_element(),
                                ComboRow::Empty => div()
                                    .p(px(12.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(m.empty_text.clone())
                                    .into_any_element(),
                                ComboRow::Create(q) => clickable(
                                    base()
                                        .text_color(c(t, ColorRole::FgMuted))
                                        .child("+")
                                        .child(m.create_text(&q)),
                                )
                                .into_any_element(),
                                ComboRow::SelectAll(n) => {
                                    clickable(base().child(format!("Select all {n}")))
                                        .into_any_element()
                                }
                                ComboRow::Option(i, _) => {
                                    let o = &m.options[i];
                                    let on = m.is_selected(i);
                                    clickable(
                                        base()
                                            .aria_selected(on)
                                            .when(o.disabled, |d| {
                                                d.text_color(c(t, ColorRole::FgDisabled))
                                            })
                                            .when(m.multiple, |d| {
                                                d.child(
                                                    div()
                                                        .size(px(15.))
                                                        .rounded(px(4.))
                                                        .border_1()
                                                        .border_color(if on {
                                                            c(t, ColorRole::FgDefault)
                                                        } else {
                                                            c(t, ColorRole::BorderStrong)
                                                        })
                                                        .when(on, |d| {
                                                            d.bg(c(t, ColorRole::FgDefault))
                                                        })
                                                        .text_color(c(t, ColorRole::BgBase))
                                                        .text_size(px(10.))
                                                        .flex()
                                                        .items_center()
                                                        .justify_center()
                                                        .child(if on { "✓" } else { "" }),
                                                )
                                            })
                                            .child(
                                                div()
                                                    .flex_1()
                                                    .flex()
                                                    .flex_col()
                                                    .child(o.label.clone())
                                                    .when_some(o.hint.clone(), |d, h| {
                                                        d.child(
                                                            div()
                                                                .text_size(px(11.5))
                                                                .text_color(c(
                                                                    t,
                                                                    ColorRole::FgSubtle,
                                                                ))
                                                                .child(h),
                                                        )
                                                    }),
                                            )
                                            .when(!m.multiple && on, |d| d.child("✓")),
                                    )
                                    .into_any_element()
                                }
                            }
                        })),
                )
            })
    }
}

// ---------------------------------------------------------------------------------------------
// OTP

/// `otp-input`: boxes with the active ring, shake offset, the Verified pill and resend.
#[derive(IntoElement)]
pub struct OtpEl {
    model: Otp,
    now: ClockMs,
    focused: bool,
    label: SharedString,
    theme: &'static Theme,
}

impl OtpEl {
    /// From the model.
    pub fn new(model: Otp, now: ClockMs) -> Self {
        Self {
            model,
            now,
            focused: true,
            label: "Verification code".into(),
            theme: &Theme::DARK,
        }
    }
    /// Whether the field has focus (draws the ring).
    pub fn focused(mut self, f: bool) -> Self {
        self.focused = f;
        self
    }
    themed!();
}

impl RenderOnce for OtpEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let s = m.status;
        let sep = m.separator_at();
        let dx = m.shake(self.now, false);
        let boxes: AnyElement = if s == OtpStatus::Success {
            div()
                .h(px(36.))
                .px(px(14.))
                .flex()
                .items_center()
                .gap(px(6.))
                .rounded_full()
                .bg(c(t, ColorRole::StatusSuccessBg))
                .text_color(c(t, ColorRole::StatusSuccessFg))
                .font_weight(FontWeight::SEMIBOLD)
                .child("✓ Verified")
                .into_any_element()
        } else {
            div()
                .flex()
                .gap(px(6.))
                .items_center()
                .ml(px(dx))
                .children((0..m.length).flat_map(|i| {
                    let mut v: Vec<AnyElement> = vec![];
                    if sep == Some(i) {
                        v.push(
                            div()
                                .w(px(10.))
                                .h(px(2.))
                                .bg(c(t, ColorRole::BorderStrong))
                                .into_any_element(),
                        );
                    }
                    let active = self.focused && m.active == i && s == OtpStatus::Idle;
                    v.push(
                        div()
                            .w(px(40.))
                            .h(px(46.))
                            .rounded(px(9.))
                            .border_1()
                            .border_color(if s == OtpStatus::Error {
                                c(t, ColorRole::StatusDangerFg)
                            } else if active {
                                c(t, ColorRole::FgDefault)
                            } else if m.cells[i].is_some() {
                                c(t, ColorRole::BorderStrong)
                            } else {
                                c(t, ColorRole::BorderDefault)
                            })
                            .bg(c(t, ColorRole::SurfaceRaised))
                            .when(s == OtpStatus::Verifying, |d| {
                                d.text_color(c(t, ColorRole::FgMuted))
                            })
                            .flex()
                            .items_center()
                            .justify_center()
                            .text_size(px(20.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(m.glyph(i).map(String::from).unwrap_or_default())
                            .into_any_element(),
                    );
                    v
                }))
                .into_any_element()
        };
        let below = match s {
            OtpStatus::Error => Some((
                "That code didn't work. Try again.".to_string(),
                ColorRole::StatusDangerFg,
            )),
            OtpStatus::Success => None,
            _ => m.resend_left(self.now).map(|l| {
                if l > 0 {
                    (
                        format!("Didn't get a code? Resend in 0:{l:02}"),
                        ColorRole::FgMuted,
                    )
                } else {
                    (
                        "Didn't get a code? Resend".to_string(),
                        ColorRole::FgDefault,
                    )
                }
            }),
        };
        div()
            .id("oxyn-a11y")
            .role(Role::TextInput)
            .aria_label(SharedString::from(m.label(&self.label)))
            .flex()
            .flex_col()
            .gap(px(8.))
            .text_color(c(t, ColorRole::FgDefault))
            .when(m.disabled, |d| d.opacity(0.5))
            .child(boxes)
            .children(
                below.map(|(txt, r)| div().text_size(px(12.5)).text_color(c(t, r)).child(txt)),
            )
    }
}

/// `password-strength`: four segments, the word and the checklist.
#[derive(IntoElement)]
pub struct PasswordStrengthEl {
    value: String,
    rules: Vec<Rule>,
    short_length: usize,
    checklist: bool,
    theme: &'static Theme,
}

impl PasswordStrengthEl {
    /// For `value` against `rules`.
    pub fn new(value: &str, rules: Vec<Rule>) -> Self {
        Self {
            value: value.into(),
            rules,
            short_length: 8,
            checklist: true,
            theme: &Theme::DARK,
        }
    }
    /// Meter only.
    pub fn meter_only(mut self) -> Self {
        self.checklist = false;
        self
    }
    themed!();
}

fn level_role(l: Level) -> ColorRole {
    match l {
        Level::TooShort | Level::Weak => ColorRole::StatusDangerFg,
        Level::Fair => ColorRole::StatusWarningFg,
        Level::Good => ColorRole::StatusInfoFg,
        Level::Strong => ColorRole::StatusSuccessFg,
    }
}

impl RenderOnce for PasswordStrengthEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = evaluate(&self.value, &self.rules, self.short_length);
        let lit = r.level.segments().max(u8::from(!self.value.is_empty()));
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .text_size(px(12.5))
            .text_color(c(t, ColorRole::FgMuted))
            .child(
                div()
                    .id("oxyn-a11y")
                    .role(Role::Meter)
                    .aria_label("Password strength")
                    .aria_numeric_value(f64::from(r.level.segments()))
                    .aria_min_numeric_value(0.0)
                    .aria_max_numeric_value(4.0)
                    .aria_value(r.level.word())
                    .flex()
                    .items_center()
                    .gap(px(4.))
                    .children((0..4u8).map(|i| {
                        div().flex_1().h(px(4.)).rounded(px(2.)).bg(if i < lit {
                            c(t, level_role(r.level))
                        } else {
                            c(t, ColorRole::BorderDefault)
                        })
                    }))
                    .child(div().w(px(64.)).child(if self.value.is_empty() {
                        ""
                    } else {
                        r.level.word()
                    })),
            )
            .when(self.checklist, |d| {
                d.children(self.rules.iter().enumerate().map(|(i, rule)| {
                    let ok = r.met[i];
                    div()
                        .flex()
                        .gap(px(6.))
                        .when(ok, |d| d.text_color(c(t, ColorRole::FgDefault)))
                        .child(
                            div()
                                .text_color(if ok {
                                    c(t, ColorRole::StatusSuccessFg)
                                } else {
                                    c(t, ColorRole::FgSubtle)
                                })
                                .child(if ok { "●" } else { "○" }),
                        )
                        .child(rule.label.clone())
                        .children(r.remaining[i].map(|n| {
                            div()
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(format!("· {n} more"))
                        }))
                }))
            })
    }
}

// ---------------------------------------------------------------------------------------------
// Inline edit

/// `inline-edit`: the text as a button, or the draft in a field; saving, saved, failed.
#[derive(IntoElement)]
pub struct InlineEditEl {
    label: SharedString,
    model: InlineEdit,
    on_start: Option<OnIndex>,
    theme: &'static Theme,
}

impl InlineEditEl {
    /// From the model.
    pub fn new(label: impl Into<SharedString>, model: InlineEdit) -> Self {
        Self {
            label: label.into(),
            model,
            on_start: None,
            theme: &Theme::DARK,
        }
    }
    /// The text was clicked (the host calls `start`).
    pub fn on_start(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_start = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for InlineEditEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let title = m.variant == oxyn_core::primitives::inline_edit::EditVariant::Title;
        let f = self.on_start.clone();
        let size = if title { 18. } else { 13. };
        let body: AnyElement = if m.editing() {
            div()
                .px(px(6.))
                .rounded(px(6.))
                .border_1()
                .border_color(c(t, ColorRole::FgMuted))
                .bg(c(t, ColorRole::SurfaceRaised))
                .child(m.draft.clone())
                .into_any_element()
        } else {
            div()
                .id("ie-text")
                .role(Role::Button)
                .aria_label(SharedString::from(m.label(&self.label)))
                .flex()
                .items_center()
                .gap(px(8.))
                .child(m.value.clone())
                .child(
                    div()
                        .text_size(px(12.))
                        .text_color(match m.phase {
                            EditPhase::Saved(_) => c(t, ColorRole::StatusSuccessFg),
                            EditPhase::Failed(_) => c(t, ColorRole::StatusDangerFg),
                            _ => c(t, ColorRole::FgSubtle),
                        })
                        .child(match &m.phase {
                            EditPhase::Saving => "…".to_string(),
                            EditPhase::Saved(_) => "✓".to_string(),
                            EditPhase::Failed(_) => "↻ Retry".to_string(),
                            _ => String::new(),
                        }),
                )
                .when_some(f, |d, f| {
                    d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                })
                .into_any_element()
        };
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .text_size(px(size))
            .when(title, |d| d.font_weight(FontWeight::SEMIBOLD))
            .text_color(c(t, ColorRole::FgDefault))
            .child(body)
            .children(message(t, m.error.clone(), None))
            .when_some(
                match &m.phase {
                    EditPhase::Conflict(_) | EditPhase::Failed(_) => Some(m.status()),
                    _ => None,
                },
                |d, s| {
                    d.child(
                        div()
                            .text_size(px(12.))
                            .text_color(c(t, ColorRole::StatusWarningFg))
                            .child(s),
                    )
                },
            )
    }
}

// ---------------------------------------------------------------------------------------------
// Phone input

/// `phone-input`: the country button, the formatted number and the picker.
#[derive(IntoElement)]
pub struct PhoneInputEl {
    label: SharedString,
    model: PhoneInput,
    on_pick: Option<OnIndex>,
    theme: &'static Theme,
}

impl PhoneInputEl {
    /// From the model.
    pub fn new(label: impl Into<SharedString>, model: PhoneInput) -> Self {
        Self {
            label: label.into(),
            model,
            on_pick: None,
            theme: &Theme::DARK,
        }
    }
    /// A picker row was clicked (the host calls `pick(Some(row))`).
    pub fn on_pick(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for PhoneInputEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let pick = self.on_pick.clone();
        let valid = m.status() == PhoneStatus::Valid;
        let num = if m.digits.is_empty() {
            div().flex_1().text_color(c(t, ColorRole::FgSubtle)).child(
                oxyn_core::primitives::phone::format_national(m.country, m.country.example),
            )
        } else {
            div().flex_1().child(if m.masked {
                m.masked_text()
            } else {
                m.formatted()
            })
        };
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .min_w(px(260.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(label_row(t, self.label.clone()))
            .child(
                shell(t, m.open, m.error().is_some())
                    .id("oxyn-a11y")
                    .role(Role::PhoneNumberInput)
                    .aria_label(self.label)
                    .child(
                        div()
                            .id("ph-cc")
                            .role(Role::Button)
                            .aria_label(SharedString::from(m.button_label()))
                            .aria_expanded(m.open)
                            .flex()
                            .gap(px(4.))
                            .pr(px(8.))
                            .border_r_1()
                            .border_color(c(t, ColorRole::BorderDefault))
                            .child(m.country.flag())
                            .child(format!("+{}", m.country.code))
                            .child(div().text_color(c(t, ColorRole::FgSubtle)).child("⌄")),
                    )
                    .child(num)
                    .when(valid, |d| {
                        d.child(
                            div()
                                .text_color(c(t, ColorRole::StatusSuccessFg))
                                .child("✓"),
                        )
                    }),
            )
            .children(message(t, m.error(), None))
            .when(m.open, |d| {
                d.child(
                    div()
                        .id("oxyn-a11y")
                        .role(Role::ListBox)
                        .flex()
                        .flex_col()
                        .p(px(4.))
                        .max_h(px(240.))
                        .overflow_y_scroll()
                        .rounded(px(12.))
                        .bg(c(t, ColorRole::SurfaceOverlay))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .children(m.rows().into_iter().enumerate().map(|(ri, r)| {
                            match r {
                                CountryRow::Group(g) => div()
                                    .px(px(8.))
                                    .pt(px(6.))
                                    .text_size(px(11.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(g.to_uppercase())
                                    .into_any_element(),
                                CountryRow::Empty => div()
                                    .p(px(12.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child("No countries match")
                                    .into_any_element(),
                                CountryRow::Country(ci) => {
                                    let co = &COUNTRIES[ci];
                                    let p = pick.clone();
                                    div()
                                        .id(("ph-row", ri))
                                        .role(Role::ListBoxOption)
                                        .aria_selected(co.iso == m.country.iso)
                                        .flex()
                                        .gap(px(8.))
                                        .h(px(32.))
                                        .items_center()
                                        .px(px(8.))
                                        .rounded(px(6.))
                                        .when(m.active == Some(ri), |d| {
                                            d.bg(c(t, ColorRole::SurfaceHover))
                                        })
                                        .child(co.flag())
                                        .child(div().flex_1().child(co.name))
                                        .child(
                                            div()
                                                .text_color(c(t, ColorRole::FgSubtle))
                                                .child(format!("+{}", co.code)),
                                        )
                                        .when_some(p, |d, f| {
                                            d.cursor_pointer()
                                                .on_click(move |_, w, cx| f(ri, w, cx))
                                        })
                                        .into_any_element()
                                }
                            }
                        })),
                )
            })
    }
}

// ---------------------------------------------------------------------------------------------
// Calendar / pickers

fn month_grid(
    t: &'static Theme,
    cal: &Calendar,
    band: Option<&RangePicker>,
    pick: Option<OnDate>,
) -> impl IntoElement + use<> {
    let rows: Vec<[DayCell; 7]> = cal.grid();
    let band = band.cloned();
    div()
        .id("oxyn-a11y")
        .role(Role::Grid)
        .aria_label(SharedString::from(cal.title()))
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(
            div()
                .flex()
                .children(cal.weekdays().into_iter().map(|(s, _)| {
                    div()
                        .w(px(36.))
                        .text_center()
                        .text_size(px(11.))
                        .text_color(c(t, ColorRole::FgSubtle))
                        .child(s)
                })),
        )
        .children(rows.into_iter().enumerate().map(move |(ri, row)| {
            let band = band.clone();
            let pick = pick.clone();
            div()
                .flex()
                .children(row.into_iter().enumerate().map(move |(ci, d)| {
                    let b = band.as_ref().filter(|_| d.in_month).map(|r| r.band(d.date));
                    let ends = b.is_some_and(|b| (b.start || b.end) && !b.preview);
                    let inside = b.is_some_and(|b| b.inside);
                    let cmp = b.is_some_and(|b| b.compare && !b.inside);
                    let dark = d.selected || ends;
                    let dt = d.date;
                    let p = pick.clone();
                    div()
                        .id(("day", ri * 7 + ci))
                        .role(Role::GridCell)
                        .aria_label(SharedString::from(dt.long()))
                        .aria_selected(d.selected || inside)
                        .w(px(36.))
                        .h(px(34.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(if inside && !ends { 0. } else { 8. }))
                        .text_size(px(13.))
                        .when(!d.in_month, |x| x.opacity(0.45))
                        .when(d.disabled, |x| x.text_color(c(t, ColorRole::FgDisabled)))
                        .when(inside && !dark, |x| x.bg(c(t, ColorRole::SurfaceSelected)))
                        .when(cmp, |x| x.bg(c(t, ColorRole::SurfaceHover)))
                        .when(dark, |x| {
                            x.bg(c(t, ColorRole::FgDefault))
                                .text_color(c(t, ColorRole::BgBase))
                        })
                        .when(d.focused, |x| {
                            x.border_1().border_color(c(t, ColorRole::Focus))
                        })
                        .when(d.today, |x| x.font_weight(FontWeight::BOLD))
                        .child(dt.d.to_string())
                        .when_some(p.filter(|_| !d.disabled), |x, f| {
                            x.cursor_pointer().on_click(move |_, w, cx| f(dt, w, cx))
                        })
                }))
        }))
}

/// `calendar`: the title, pagers and the month grid.
#[derive(IntoElement)]
pub struct CalendarEl {
    model: Calendar,
    on_pick: Option<OnDate>,
    theme: &'static Theme,
}

impl CalendarEl {
    /// From the model.
    pub fn new(model: Calendar) -> Self {
        Self {
            model,
            on_pick: None,
            theme: &Theme::DARK,
        }
    }
    /// A day was clicked (the host calls `select`).
    pub fn on_pick(mut self, f: impl Fn(Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
    themed!();
}

fn cal_head(t: &'static Theme, cal: &Calendar) -> impl IntoElement + use<> {
    div()
        .text_color(c(t, ColorRole::FgDefault))
        .flex()
        .items_center()
        .gap(px(4.))
        .child(
            div()
                .flex_1()
                .font_weight(FontWeight::SEMIBOLD)
                .child(cal.title()),
        )
        .child(
            div()
                .px(px(6.))
                .when(!cal.can_prev(), |d| d.opacity(0.35))
                .child("‹"),
        )
        .child(
            div()
                .px(px(6.))
                .when(!cal.can_next(), |d| d.opacity(0.35))
                .child("›"),
        )
}

impl RenderOnce for CalendarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(cal_head(t, &self.model))
            .child(month_grid(t, &self.model, None, self.on_pick))
    }
}

/// `date-reel`: the summary, each reel's entries projected on the cylinder, quick picks.
#[derive(IntoElement)]
pub struct DateWheelEl {
    model: DateWheel,
    on_preset: Option<OnIndex>,
    theme: &'static Theme,
}

impl DateWheelEl {
    /// From the model (the host ticks it every frame).
    pub fn new(model: DateWheel) -> Self {
        Self {
            model,
            on_preset: None,
            theme: &Theme::DARK,
        }
    }
    /// A quick pick was clicked (the host calls `preset`).
    pub fn on_preset(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_preset = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for DateWheelEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let row = 34.0f32;
        let hgt = row * 5.0;
        let f = self.on_preset.clone();
        let active = m.active_preset();
        div()
            .id("oxyn-a11y")
            .role(Role::Group)
            .aria_label("Date and time")
            .flex()
            .flex_col()
            .gap(px(12.))
            .w(px(380.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .items_end()
                    .child(
                        div()
                            .text_size(px(17.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(m.summary()),
                    )
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(c(t, ColorRole::FgSubtle))
                            .child(m.relative()),
                    ),
            )
            .child(
                div()
                    .relative()
                    .h(px(hgt))
                    .rounded(px(12.))
                    .bg(c(t, ColorRole::SurfaceSunken))
                    .flex()
                    .justify_center()
                    .gap(px(4.))
                    .overflow_hidden()
                    .child(
                        div()
                            .absolute()
                            .left(px(6.))
                            .right(px(6.))
                            .top(px(hgt / 2. - row / 2.))
                            .h(px(row))
                            .rounded(px(8.))
                            .bg(c(t, ColorRole::SurfaceRaised)),
                    )
                    .children(m.reels.iter().enumerate().map(|(ri, reel)| {
                        let w = if ri == 0 && reel.entries.len() > 40 {
                            120.
                        } else {
                            56.
                        };
                        div()
                            .id(("reel", ri))
                            .role(Role::SpinButton)
                            .aria_label(reel.kind.name())
                            .aria_numeric_value(reel.index() as f64)
                            .aria_value(SharedString::from(m.valuetext(ri)))
                            .relative()
                            .w(px(w))
                            .h_full()
                            .children(reel.entries.iter().enumerate().filter_map(|(i, e)| {
                                let (y, _, o) = reel.project(i, 2.6)?;
                                let cur = i == reel.index();
                                Some(
                                    div()
                                        .absolute()
                                        .left_0()
                                        .right_0()
                                        .top(px(hgt / 2. - row / 2. + y * row))
                                        .h(px(row))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .opacity(o)
                                        .when(!reel.valid[i], |d| {
                                            d.text_color(c(t, ColorRole::FgDisabled))
                                        })
                                        .when(cur, |d| d.font_weight(FontWeight::SEMIBOLD))
                                        .text_size(px(15.))
                                        .child(e.clone()),
                                )
                            }))
                    })),
            )
            .child(
                div()
                    .flex()
                    .gap(px(4.))
                    .flex_wrap()
                    .children(m.presets.iter().enumerate().map(|(i, p)| {
                        let on = active == Some(i);
                        let f = f.clone();
                        div()
                            .id(("reel-pre", i))
                            .role(Role::Button)
                            .h(px(26.))
                            .px(px(10.))
                            .flex()
                            .items_center()
                            .rounded_full()
                            .border_1()
                            .border_color(c(t, ColorRole::BorderDefault))
                            .text_size(px(12.))
                            .when(on, |d| {
                                d.bg(c(t, ColorRole::FgDefault))
                                    .text_color(c(t, ColorRole::BgBase))
                            })
                            .child(p.label.clone())
                            .when_some(f, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                            })
                    })),
            )
    }
}

#[cfg(test)]
#[cfg_attr(not(oxyn_pro), allow(unused_imports, dead_code))]
mod tests {
    use super::*;
    use oxyn_core::primitives::{
        password::Policy, questions::Question, smart_fill::contact_fields,
    };

    #[cfg(oxyn_pro)]
    #[test]
    fn elements_build() {
        let _ = TagInputEl::new("Labels", TagInput::new(&["a", "b"])).on_remove(|_, _, _| {});
        let _ = ComboboxEl::new(Combobox::new(vec![], &[])).label("Assign");
        let _ = OtpEl::new(Otp::new(6), ClockMs(0));
        let _ = PasswordFieldEl::new("Password", "x", false).caps(true);
        let _ = PasswordStrengthEl::new("abc", Policy::Basic.rules()).meter_only();
        let _ = InlineEditEl::new("Name", InlineEdit::new("Atlas"));
        let _ = PhoneInputEl::new("Phone", PhoneInput::new("GB"));
        let today = Date::new(2026, 11, 13).unwrap();
        let _ = CalendarEl::new(Calendar::new(Some(today), today));
        let _ = DatePickerEl::new("Due", DatePicker::new(None, today));
        let _ = RangePickerEl::new(RangePicker::new(None, today));
        let _ = TimePickerEl::new("At", Some(TimeOfDay::new(9, 30)), 15).h24();
        let _ = TimeDialEl::new(TimeDial::new(vec![], 600));
        let _ = DateWheelEl::new(DateWheel::new(
            Default::default(),
            (today, TimeOfDay::new(9, 0)),
            today,
        ));
        let _ = FilterMenuEl::new(FilterMenu::new(vec![]));
        let _ = StepperEl::new(Stepper::new(vec![
            oxyn_core::primitives::stepper::Step::review("Review"),
        ]));
        let deck = QuestionDeck::new(
            vec![Question::new("q", "Ok?", QuestionKind::Approval, vec![])],
            DeckMode::Decisions,
        );
        let _ = DecisionCardEl::new(deck.clone());
        let _ = ApprovalQuestionsEl::new(deck);
        let _ = SmartFillEl::new(SmartFill::new(contact_fields()), ClockMs(0));
        assert_eq!(level_role(Level::Strong), ColorRole::StatusSuccessFg);
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
