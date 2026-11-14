//! Forms, pickers and flows for GPUI (plan #25), drawn from the same `oxyn_core::primitives`
//! models as the Dioxus components. Elements are `RenderOnce`: the host owns the model, routes
//! key events to the model's `key` methods, feeds the shared clock and re-renders. Rows, chips,
//! days and options take an index (or date) listener so clicks reach the host.
//!
//! Gaps against Dioxus (listed in the PR): text entry is display-only until the shared input
//! handler lands (the host forwards characters to the model); popovers render in flow under their
//! trigger; no per-element `FocusHandle`; the time dial is built from positioned dots rather
//! than SVG arcs; reel entries fade and offset but do not scale their glyphs.

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
        dial::{DateReel, TimeDial},
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

fn alpha(mut h: Hsla, a: f32) -> Hsla {
    h.a *= a;
    h
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

// ---------------------------------------------------------------------------------------------
// Password field / strength

/// `password-field`: masked or revealed value, the eye toggle, Caps Lock warning.
#[derive(IntoElement)]
pub struct PasswordFieldEl {
    label: SharedString,
    value: String,
    revealed: bool,
    caps: bool,
    description: Option<String>,
    on_toggle: Option<OnIndex>,
    theme: &'static Theme,
}

impl PasswordFieldEl {
    /// A field.
    pub fn new(label: impl Into<SharedString>, value: &str, revealed: bool) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
            revealed,
            caps: false,
            description: None,
            on_toggle: None,
            theme: &Theme::DARK,
        }
    }
    /// Caps Lock is on (from `password::caps_lock_from`).
    pub fn caps(mut self, on: bool) -> Self {
        self.caps = on;
        self
    }
    /// Helper copy.
    pub fn description(mut self, d: impl Into<String>) -> Self {
        self.description = Some(d.into());
        self
    }
    /// The eye was clicked.
    pub fn on_toggle(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_toggle = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for PasswordFieldEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let shown = if self.revealed {
            self.value.clone()
        } else {
            "•".repeat(self.value.chars().count())
        };
        let f = self.on_toggle.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(label_row(t, self.label.clone()))
            .child(
                shell(t, false, false)
                    .id("oxyn-a11y")
                    .role(Role::PasswordInput)
                    .aria_label(self.label)
                    .child(div().flex_1().child(shown))
                    .child(
                        div()
                            .id("pw-eye")
                            .role(Role::Button)
                            .aria_label(if self.revealed {
                                "Hide password"
                            } else {
                                "Show password"
                            })
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(if self.revealed { "◉" } else { "⊘" })
                            .when_some(f, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                            }),
                    ),
            )
            .children(if self.caps {
                Some(
                    div()
                        .text_size(px(12.))
                        .text_color(c(t, ColorRole::StatusWarningFg))
                        .child("↑ Caps Lock is on")
                        .into_any_element(),
                )
            } else {
                message(t, None, self.description)
            })
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

/// `date-picker`: the trigger with the formatted date and, when open, the calendar.
#[derive(IntoElement)]
pub struct DatePickerEl {
    label: SharedString,
    model: DatePicker,
    on_pick: Option<OnDate>,
    theme: &'static Theme,
}

impl DatePickerEl {
    /// From the model.
    pub fn new(label: impl Into<SharedString>, model: DatePicker) -> Self {
        Self {
            label: label.into(),
            model,
            on_pick: None,
            theme: &Theme::DARK,
        }
    }
    /// A day was clicked (the host calls `pick`).
    pub fn on_pick(mut self, f: impl Fn(Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for DatePickerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let txt = m.value().map(|v| m.format.apply(v));
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(label_row(t, self.label.clone()))
            .child(
                shell(t, m.open, false)
                    .id("oxyn-a11y")
                    .role(Role::DateInput)
                    .aria_label(self.label)
                    .aria_expanded(m.open)
                    .child("▦")
                    .child(
                        div()
                            .when(txt.is_none(), |d| d.text_color(c(t, ColorRole::FgSubtle)))
                            .child(txt.unwrap_or("Select a date".into())),
                    ),
            )
            .when(m.open, |d| {
                d.child(
                    div()
                        .p(px(12.))
                        .rounded(px(12.))
                        .bg(c(t, ColorRole::SurfaceOverlay))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .flex()
                        .flex_col()
                        .gap(px(6.))
                        .child(cal_head(t, &m.cal))
                        .child(month_grid(t, &m.cal, None, self.on_pick)),
                )
            })
    }
}

/// `date-range-picker`: presets, one or two months with the band and comparison, the footer.
#[derive(IntoElement)]
pub struct RangePickerEl {
    model: RangePicker,
    on_pick: Option<OnDate>,
    on_preset: Option<OnIndex>,
    theme: &'static Theme,
}

impl RangePickerEl {
    /// From the model.
    pub fn new(model: RangePicker) -> Self {
        Self {
            model,
            on_pick: None,
            on_preset: None,
            theme: &Theme::DARK,
        }
    }
    /// A day was clicked (the host calls `pick`).
    pub fn on_pick(mut self, f: impl Fn(Date, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
    /// A preset was clicked: its index in `presets`.
    pub fn on_preset(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_preset = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for RangePickerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let active = m.active_preset();
        let pre = self.on_preset.clone();
        let pick = self.on_pick.clone();
        let trig = div()
            .id("oxyn-a11y")
            .role(Role::Button)
            .aria_expanded(m.open)
            .h(px(34.))
            .px(px(10.))
            .flex()
            .items_center()
            .gap(px(8.))
            .rounded(px(8.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .child("▦")
            .child(m.trigger_label("Select dates"));
        let months: Vec<Calendar> = (0..m.months)
            .map(|i| {
                let mut cal = m.cal.clone();
                cal.month = m.cal.month.add_months(i32::from(i));
                cal
            })
            .collect();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(trig)
            .when(m.open, |d| {
                d.child(
                    div()
                        .id("oxyn-a11y")
                        .role(Role::Dialog)
                        .flex()
                        .rounded(px(14.))
                        .bg(c(t, ColorRole::SurfaceOverlay))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .p(px(8.))
                                .gap(px(1.))
                                .border_r_1()
                                .border_color(c(t, ColorRole::BorderSubtle))
                                .children(m.presets.iter().enumerate().map(|(i, p)| {
                                    let on = active == Some(p.id);
                                    let f = pre.clone();
                                    div()
                                        .id(("drp-pre", i))
                                        .role(Role::ListBoxOption)
                                        .aria_selected(on)
                                        .h(px(28.))
                                        .px(px(10.))
                                        .flex()
                                        .items_center()
                                        .rounded(px(7.))
                                        .text_color(if on {
                                            c(t, ColorRole::FgDefault)
                                        } else {
                                            c(t, ColorRole::FgMuted)
                                        })
                                        .when(on, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                                        .child(p.label)
                                        .when_some(f, |d, f| {
                                            d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                                        })
                                })),
                        )
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(div().flex().gap(px(16.)).p(px(12.)).children(
                                    months.iter().map(|cal| {
                                        div()
                                            .flex()
                                            .flex_col()
                                            .gap(px(6.))
                                            .child(
                                                div()
                                                    .font_weight(FontWeight::SEMIBOLD)
                                                    .text_center()
                                                    .child(cal.title()),
                                            )
                                            .child(month_grid(t, cal, Some(&m), pick.clone()))
                                    }),
                                ))
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(8.))
                                        .p(px(12.))
                                        .border_t_1()
                                        .border_color(c(t, ColorRole::BorderSubtle))
                                        .child(div().flex_1().child(m.announce()))
                                        .child(
                                            div()
                                                .text_color(c(t, ColorRole::FgMuted))
                                                .child("Cancel"),
                                        )
                                        .child(
                                            div()
                                                .px(px(10.))
                                                .rounded(px(6.))
                                                .bg(c(t, ColorRole::FgDefault))
                                                .text_color(c(t, ColorRole::BgBase))
                                                .child("Apply"),
                                        ),
                                ),
                        ),
                )
            })
    }
}

/// `time-picker`: the value and, when open, the list of times.
#[derive(IntoElement)]
pub struct TimePickerEl {
    label: SharedString,
    value: Option<TimeOfDay>,
    step: u32,
    cycle: HourCycle,
    open: bool,
    active: Option<usize>,
    on_pick: Option<OnIndex>,
    theme: &'static Theme,
}

impl TimePickerEl {
    /// A picker.
    pub fn new(label: impl Into<SharedString>, value: Option<TimeOfDay>, step: u32) -> Self {
        Self {
            label: label.into(),
            value,
            step,
            cycle: HourCycle::H12,
            open: false,
            active: None,
            on_pick: None,
            theme: &Theme::DARK,
        }
    }
    /// 24-hour.
    pub fn h24(mut self) -> Self {
        self.cycle = HourCycle::H24;
        self
    }
    /// Open with this row active.
    pub fn open(mut self, active: Option<usize>) -> Self {
        self.open = true;
        self.active = active;
        self
    }
    /// A time was clicked: its index in `time_options`.
    pub fn on_pick(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_pick = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for TimePickerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let opts = time_options(self.step, self.cycle, None);
        let cur = self.value.map(|v| v.hhmm());
        let f = self.on_pick.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(label_row(t, self.label.clone()))
            .child(
                shell(t, self.open, false)
                    .id("oxyn-a11y")
                    .role(Role::TimeInput)
                    .aria_label(self.label)
                    .aria_expanded(self.open)
                    .child("◷")
                    .child(
                        div().flex_1().child(
                            self.value
                                .map(|v| v.label(self.cycle))
                                .unwrap_or("Select a time".into()),
                        ),
                    )
                    .child("⌄"),
            )
            .when(self.open, |d| {
                d.child(
                    div()
                        .id("tp-list")
                        .role(Role::ListBox)
                        .flex()
                        .flex_col()
                        .p(px(4.))
                        .max_h(px(250.))
                        .overflow_y_scroll()
                        .rounded(px(10.))
                        .bg(c(t, ColorRole::SurfaceOverlay))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .children(opts.into_iter().enumerate().map(|(i, (v, l))| {
                            let on = cur.as_deref() == Some(v.as_str());
                            let f = f.clone();
                            div()
                                .id(("tp", i))
                                .role(Role::ListBoxOption)
                                .aria_selected(on)
                                .h(px(30.))
                                .px(px(8.))
                                .flex()
                                .items_center()
                                .rounded(px(6.))
                                .when(self.active == Some(i), |d| {
                                    d.bg(c(t, ColorRole::SurfaceHover))
                                })
                                .child(div().flex_1().child(l))
                                .when(on, |d| d.child("✓"))
                                .when_some(f, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                                })
                        })),
                )
            })
    }
}

/// `time-dial`: a 24-hour face drawn from positioned dots (working band, ticks, cities) with the
/// readout in the middle, and the city list.
#[derive(IntoElement)]
pub struct TimeDialEl {
    model: TimeDial,
    on_home: Option<OnIndex>,
    theme: &'static Theme,
}

impl TimeDialEl {
    /// From the model.
    pub fn new(model: TimeDial) -> Self {
        Self {
            model,
            on_home: None,
            theme: &Theme::DARK,
        }
    }
    /// A city row was clicked (the host calls `set_home`).
    pub fn on_home(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_home = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for TimeDialEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let (size, r) = (240.0f32, 104.0f32);
        let (cx, cy) = (size / 2.0, size / 2.0);
        let home = m.home;
        let (hm, _) = m.local(home);
        // Face in home-local time, home at the top: minute k sits at (k − hm + 720).
        let rel = |k: i32| (k - hm + 720).rem_euclid(1440);
        let mut face: Vec<AnyElement> = vec![];
        for q in 0..96 {
            let k = q * 15;
            let (x, y) = TimeDial::point(rel(k), cx, cy, r);
            let work = k >= m.working.0 && k < m.working.1;
            let s = if k % 360 == 0 {
                5.
            } else if work {
                4.
            } else {
                2.
            };
            face.push(
                div()
                    .absolute()
                    .left(px(x - s / 2.))
                    .top(px(y - s / 2.))
                    .size(px(s))
                    .rounded_full()
                    .bg(if work {
                        c(t, ColorRole::FgMuted)
                    } else {
                        c(t, ColorRole::BorderStrong)
                    })
                    .into_any_element(),
            );
        }
        for (i, city) in m.cities.iter().enumerate() {
            let (lm, _) = m.local(i);
            let (x, y) = TimeDial::point(rel(lm), cx, cy, r - 18.0);
            let s = if i == home { 22. } else { 9. };
            face.push(
                div()
                    .absolute()
                    .left(px(x - s / 2.))
                    .top(px(y - s / 2.))
                    .size(px(s))
                    .rounded_full()
                    .flex()
                    .items_center()
                    .justify_center()
                    .text_size(px(7.5))
                    .bg(if i == home {
                        c(t, ColorRole::FgDefault)
                    } else if m.at_work(i) {
                        c(t, ColorRole::FgMuted)
                    } else {
                        c(t, ColorRole::FgSubtle)
                    })
                    .text_color(c(t, ColorRole::BgBase))
                    .when(i == home, |d| d.child(city.code.clone()))
                    .into_any_element(),
            );
        }
        let f = self.on_home.clone();
        div()
            .flex()
            .gap(px(20.))
            .items_center()
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .id("oxyn-a11y")
                    .role(Role::Slider)
                    .aria_label("Time")
                    .aria_numeric_value(f64::from(hm))
                    .aria_min_numeric_value(0.0)
                    .aria_max_numeric_value(1439.0)
                    .aria_value(SharedString::from(m.valuetext()))
                    .relative()
                    .size(px(size))
                    .rounded_full()
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .children(face)
                    .child(
                        div()
                            .absolute()
                            .inset_0()
                            .flex()
                            .flex_col()
                            .items_center()
                            .justify_center()
                            .child(
                                div()
                                    .text_color(c(t, ColorRole::FgMuted))
                                    .child(if m.daytime(home) { "☀" } else { "☾" }),
                            )
                            .child(
                                div()
                                    .text_size(px(26.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(m.time_label(home)),
                            )
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(format!(
                                        "{} · {}",
                                        m.home_weekday(),
                                        m.cities[home].name
                                    )),
                            ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(2.))
                    .min_w(px(220.))
                    .children(m.cities.iter().enumerate().map(|(i, city)| {
                        let f = f.clone();
                        div()
                            .id(("dial-city", i))
                            .role(Role::Button)
                            .flex()
                            .gap(px(10.))
                            .items_center()
                            .px(px(10.))
                            .py(px(7.))
                            .rounded(px(8.))
                            .when(i == home, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                            .child(div().size(px(8.)).rounded_full().bg(if m.at_work(i) {
                                c(t, ColorRole::StatusSuccessFg)
                            } else {
                                c(t, ColorRole::FgSubtle)
                            }))
                            .child(div().flex_1().child(city.name.clone()))
                            .children(m.day_note(i).map(|n| {
                                div()
                                    .text_size(px(11.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(n)
                            }))
                            .child(m.time_label(i))
                            .when_some(f, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                            })
                    })),
            )
    }
}

/// `date-reel`: the summary, each reel's entries projected on the cylinder, quick picks.
#[derive(IntoElement)]
pub struct DateReelEl {
    model: DateReel,
    on_preset: Option<OnIndex>,
    theme: &'static Theme,
}

impl DateReelEl {
    /// From the model (the host ticks it every frame).
    pub fn new(model: DateReel) -> Self {
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

impl RenderOnce for DateReelEl {
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

// ---------------------------------------------------------------------------------------------
// Flows

/// `filter-menu`: chips, group headers and options with the focus row.
#[derive(IntoElement)]
pub struct FilterMenuEl {
    model: FilterMenu,
    on_row: Option<OnIndex>,
    theme: &'static Theme,
}

impl FilterMenuEl {
    /// From the model.
    pub fn new(model: FilterMenu) -> Self {
        Self {
            model,
            on_row: None,
            theme: &Theme::DARK,
        }
    }
    /// A row was clicked: its index in `rows` (the host sets `focus` and calls `key(Activate)`).
    pub fn on_row(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_row = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for FilterMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let f = self.on_row.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .w(px(380.))
            .p(px(12.))
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceOverlay))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .flex()
                    .gap(px(6.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Filters"))
                    .when(m.count() > 0, |d| {
                        d.child(
                            div()
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(m.count().to_string()),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(4.))
                    .children(m.chips().into_iter().map(|(_, _, s)| {
                        chip(t, s, false).child(
                            div()
                                .px(px(3.))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child("×"),
                        )
                    })),
            )
            .child(
                div()
                    .id("oxyn-a11y")
                    .role(Role::Tree)
                    .flex()
                    .flex_col()
                    .children(m.rows().into_iter().enumerate().map(|(ri, r)| {
                        let f = f.clone();
                        let base = div()
                            .id(("fm", ri))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .min_h(px(32.))
                            .px(px(8.))
                            .rounded(px(7.))
                            .when(m.focus == ri, |d| d.bg(c(t, ColorRole::SurfaceHover)));
                        let el = match r {
                            FilterRow::Header(g) => base
                                .role(Role::TreeItem)
                                .aria_expanded(m.open[g])
                                .font_weight(FontWeight::MEDIUM)
                                .child(if m.open[g] { "⌄" } else { "›" })
                                .child(div().flex_1().child(m.groups[g].label.clone()))
                                .when(m.group_count(g) > 0, |d| {
                                    d.child(
                                        div()
                                            .text_color(c(t, ColorRole::FgSubtle))
                                            .child(m.group_count(g).to_string()),
                                    )
                                }),
                            FilterRow::Option(g, o) => {
                                let on = m.checked(g, o);
                                let opt = &m.groups[g].options[o];
                                base.role(Role::CheckBox)
                                    .pl(px(30.))
                                    .child(
                                        div()
                                            .size(px(15.))
                                            .rounded(px(4.))
                                            .border_1()
                                            .border_color(if on {
                                                c(t, ColorRole::FgDefault)
                                            } else {
                                                c(t, ColorRole::BorderStrong)
                                            })
                                            .when(on, |d| d.bg(c(t, ColorRole::FgDefault)))
                                            .text_color(c(t, ColorRole::BgBase))
                                            .text_size(px(10.))
                                            .flex()
                                            .items_center()
                                            .justify_center()
                                            .child(if on { "✓" } else { "" }),
                                    )
                                    .child(div().flex_1().child(opt.label.clone()))
                                    .children(opt.count.map(|n| {
                                        div()
                                            .text_color(c(t, ColorRole::FgSubtle))
                                            .child(n.to_string())
                                    }))
                            }
                        };
                        el.when_some(f, |d, f| {
                            d.cursor_pointer().on_click(move |_, w, cx| f(ri, w, cx))
                        })
                    })),
            )
    }
}

/// `stepper-form` / `multi-step-form`: the step list (or minimal header), the step's fields as
/// read-only rows, and Back / Continue.
#[derive(IntoElement)]
pub struct StepperEl {
    model: Stepper,
    on_next: Option<OnIndex>,
    on_back: Option<OnIndex>,
    theme: &'static Theme,
}

impl StepperEl {
    /// From the model.
    pub fn new(model: Stepper) -> Self {
        Self {
            model,
            on_next: None,
            on_back: None,
            theme: &Theme::DARK,
        }
    }
    /// Continue pressed (the host calls `next`).
    pub fn on_next(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_next = Some(Rc::new(f));
        self
    }
    /// Back pressed (the host calls `back`).
    pub fn on_back(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_back = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for StepperEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        if m.phase == StepperPhase::Success {
            return div()
                .id("oxyn-a11y")
                .role(Role::Status)
                .flex()
                .flex_col()
                .items_center()
                .gap(px(8.))
                .p(px(28.))
                .rounded(px(14.))
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault))
                .text_color(c(t, ColorRole::FgDefault))
                .child(
                    div()
                        .size(px(48.))
                        .rounded_full()
                        .bg(c(t, ColorRole::StatusSuccessBg))
                        .text_color(c(t, ColorRole::StatusSuccessFg))
                        .flex()
                        .items_center()
                        .justify_center()
                        .child("✓"),
                )
                .child(div().font_weight(FontWeight::SEMIBOLD).child("All set"))
                .into_any_element();
        }
        let step = &m.steps[m.current];
        let (pos, total) = m.progress();
        let vis = m.visible();
        let header: AnyElement = if m.style == oxyn_core::primitives::stepper::StepperStyle::Minimal
        {
            div()
                .text_size(px(12.5))
                .text_color(c(t, ColorRole::FgMuted))
                .child(format!("Step {pos} of {total}"))
                .into_any_element()
        } else {
            div()
                .flex()
                .gap(px(6.))
                .children(vis.iter().enumerate().map(|(k, &i)| {
                    let cur = i == m.current;
                    let done = m.done[i] && !cur;
                    div()
                        .flex_1()
                        .pt(px(8.))
                        .border_t_2()
                        .border_color(if cur {
                            c(t, ColorRole::FgDefault)
                        } else if done {
                            c(t, ColorRole::FgMuted)
                        } else {
                            c(t, ColorRole::BorderDefault)
                        })
                        .text_color(if cur {
                            c(t, ColorRole::FgDefault)
                        } else {
                            c(t, ColorRole::FgSubtle)
                        })
                        .text_size(px(12.5))
                        .child(format!(
                            "{} {}",
                            if done {
                                "✓".to_string()
                            } else {
                                (k + 1).to_string()
                            },
                            m.steps[i].title
                        ))
                }))
                .into_any_element()
        };
        let rows: Vec<AnyElement> = if step.review {
            m.review_rows()
                .into_iter()
                .map(|(l, v, _)| {
                    div()
                        .flex()
                        .gap(px(8.))
                        .py(px(6.))
                        .child(
                            div()
                                .w(px(120.))
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(l),
                        )
                        .child(v)
                        .into_any_element()
                })
                .collect()
        } else {
            step.fields
                .iter()
                .map(|f| {
                    let v = match &f.kind {
                        FieldKind::Tags { .. } => m.list(&f.name).join(", "),
                        _ => m.text(&f.name),
                    };
                    let err = m.errors.get(&f.name).cloned();
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(4.))
                        .child(label_row(t, f.label.clone()))
                        .child(shell(t, false, err.is_some()).child(v))
                        .children(message(t, err, f.hint.clone()))
                        .into_any_element()
                })
                .collect()
        };
        let next = self.on_next.clone();
        let back = self.on_back.clone();
        div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .w(px(480.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(header)
            .child(
                div()
                    .id("oxyn-a11y")
                    .role(Role::Form)
                    .p(px(18.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .flex()
                    .flex_col()
                    .gap(px(12.))
                    .child(
                        div()
                            .text_size(px(16.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(step.heading.clone()),
                    )
                    .children(rows),
            )
            .child(
                div()
                    .flex()
                    .gap(px(6.))
                    .when(pos > 1, |d| {
                        d.child(
                            div()
                                .id("st-back")
                                .role(Role::Button)
                                .px(px(10.))
                                .h(px(28.))
                                .flex()
                                .items_center()
                                .rounded(px(6.))
                                .child("Back")
                                .when_some(back, |d, f| {
                                    d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                                }),
                        )
                    })
                    .child(div().flex_1())
                    .child(
                        div()
                            .id("st-next")
                            .role(Role::Button)
                            .px(px(12.))
                            .h(px(28.))
                            .flex()
                            .items_center()
                            .rounded(px(6.))
                            .bg(c(t, ColorRole::FgDefault))
                            .text_color(c(t, ColorRole::BgBase))
                            .when(!m.valid(), |d| d.opacity(0.55))
                            .child(m.primary_label().to_string())
                            .when_some(next, |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f(0, w, cx))
                            }),
                    ),
            )
            .into_any_element()
    }
}

fn deck_rows(t: &'static Theme, d: &QuestionDeck, choose: Option<OnIndex>) -> Vec<AnyElement> {
    let Some(q) = d.current() else { return vec![] };
    let picked: Vec<usize> = match d.answer() {
        Answer::Options(v) => v.clone(),
        _ => vec![],
    };
    if q.kind == QuestionKind::Approval {
        return ["Approve", "Decline"]
            .into_iter()
            .enumerate()
            .map(|(i, l)| {
                let f = choose.clone();
                div()
                    .id(("dk-ab", i))
                    .role(Role::ListBoxOption)
                    .aria_selected(d.focus == i)
                    .px(px(12.))
                    .h(px(32.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(8.))
                    .border_1()
                    .border_color(if d.focus == i {
                        c(t, ColorRole::Focus)
                    } else {
                        c(t, ColorRole::BorderDefault)
                    })
                    .when(i == 0, |d| {
                        d.bg(c(t, ColorRole::FgDefault))
                            .text_color(c(t, ColorRole::BgBase))
                    })
                    .child(format!("{} {l}", i + 1))
                    .when_some(f, |d, f| {
                        d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                    })
                    .into_any_element()
            })
            .collect();
    }
    q.options
        .iter()
        .enumerate()
        .map(|(i, o)| {
            let f = choose.clone();
            let on = picked.contains(&i);
            div()
                .id(("dk-op", i))
                .role(
                    if d.mode == DeckMode::Paged && q.kind == QuestionKind::Multi {
                        Role::CheckBox
                    } else {
                        Role::ListBoxOption
                    },
                )
                .aria_selected(on || d.focus == i)
                .flex()
                .items_center()
                .gap(px(10.))
                .min_h(px(36.))
                .px(px(10.))
                .rounded(px(8.))
                .when(d.focus == i && d.mode == DeckMode::Decisions, |d| {
                    d.bg(c(t, ColorRole::SurfaceHover))
                })
                .when(d.mode == DeckMode::Paged, |d| {
                    d.border_1().border_color(if on {
                        c(t, ColorRole::FgDefault)
                    } else {
                        c(t, ColorRole::BorderDefault)
                    })
                })
                .child(
                    div()
                        .text_size(px(10.5))
                        .px(px(4.))
                        .rounded(px(4.))
                        .bg(alpha(c(t, ColorRole::FgDefault), 0.1))
                        .child(if d.mode == DeckMode::Paged {
                            if on {
                                "●".to_string()
                            } else {
                                "○".to_string()
                            }
                        } else {
                            (i + 1).to_string()
                        }),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .flex_1()
                        .child(div().flex().gap(px(6.)).child(o.title.clone()).when(
                            o.recommended,
                            |d| {
                                d.child(
                                    div()
                                        .text_size(px(10.5))
                                        .px(px(6.))
                                        .rounded_full()
                                        .bg(c(t, ColorRole::SurfaceSelected))
                                        .text_color(c(t, ColorRole::FgMuted))
                                        .child("Recommended"),
                                )
                            },
                        ))
                        .children(o.summary.clone().map(|s| {
                            div()
                                .text_size(px(12.))
                                .text_color(c(t, ColorRole::FgMuted))
                                .child(s)
                        })),
                )
                .when_some(f, |d, f| {
                    d.cursor_pointer().on_click(move |_, w, cx| f(i, w, cx))
                })
                .into_any_element()
        })
        .collect()
}

/// `decision-card`: the grey head (Needs you · pager · Later) and the white card of rows.
#[derive(IntoElement)]
pub struct DecisionCardEl {
    model: QuestionDeck,
    on_choose: Option<OnIndex>,
    theme: &'static Theme,
}

impl DecisionCardEl {
    /// From a deck in `DeckMode::Decisions`.
    pub fn new(model: QuestionDeck) -> Self {
        Self {
            model,
            on_choose: None,
            theme: &Theme::DARK,
        }
    }
    /// A row was clicked (the host calls `choose`).
    pub fn on_choose(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_choose = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for DecisionCardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let d = self.model;
        if d.phase == DeckPhase::Folded {
            return div()
                .px(px(14.))
                .py(px(8.))
                .rounded_full()
                .bg(c(t, ColorRole::SurfaceSunken))
                .text_color(c(t, ColorRole::FgDefault))
                .text_size(px(13.))
                .child(format!("{} · Show", d.waiting()))
                .into_any_element();
        }
        let Some(q) = d.current().cloned() else {
            return div().into_any_element();
        };
        let (pos, n) = d.position();
        div()
            .id("oxyn-a11y")
            .role(Role::Group)
            .aria_label(SharedString::from(q.prompt.clone()))
            .w(px(560.))
            .rounded(px(14.))
            .bg(c(t, ColorRole::SurfaceSunken))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .px(px(14.))
                    .py(px(8.))
                    .text_size(px(12.))
                    .text_color(c(t, ColorRole::FgMuted))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(c(t, ColorRole::FgDefault))
                            .child("Needs you"),
                    )
                    .children(q.task.clone().map(|x| div().child(format!("· {x}"))))
                    .child(div().flex_1())
                    .when(n > 1, |d| d.child(format!("‹ {pos} of {n} ›")))
                    .child("Later  Esc"),
            )
            .child(
                div()
                    .p(px(14.))
                    .flex()
                    .flex_col()
                    .gap(px(10.))
                    .rounded_t(px(12.))
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .child(
                        div()
                            .text_size(px(14.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(q.prompt.clone()),
                    )
                    .children(q.facts.iter().map(|(k, v)| {
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(div().text_color(c(t, ColorRole::FgSubtle)).child(k.clone()))
                            .child(v.clone())
                    }))
                    .child(
                        div()
                            .id("oxyn-a11y")
                            .role(Role::ListBox)
                            .flex()
                            .when(
                                q.kind == QuestionKind::Pick || q.kind == QuestionKind::Approval,
                                |d| d.flex_row().gap(px(8.)),
                            )
                            .when(
                                q.kind != QuestionKind::Pick && q.kind != QuestionKind::Approval,
                                |d| d.flex_col().gap(px(2.)),
                            )
                            .children(deck_rows(t, &d, self.on_choose)),
                    )
                    .when(q.allow_text, |d2| {
                        d2.child(
                            shell(t, d.in_text(), false).child(
                                div().text_color(c(t, ColorRole::FgSubtle)).child(
                                    d.text
                                        .get(d.index)
                                        .filter(|s| !s.is_empty())
                                        .cloned()
                                        .unwrap_or("Or tell it what to do…".into()),
                                ),
                            ),
                        )
                    }),
            )
            .into_any_element()
    }
}

/// `approval-questions` (after Beautiful UI's Approval Card, MIT, © 2026 Shane Levine): counter,
/// question, options, custom row, Skip / Continue; the sent pill.
#[derive(IntoElement)]
pub struct ApprovalQuestionsEl {
    model: QuestionDeck,
    on_choose: Option<OnIndex>,
    theme: &'static Theme,
}

impl ApprovalQuestionsEl {
    /// From a deck in `DeckMode::Paged`.
    pub fn new(model: QuestionDeck) -> Self {
        Self {
            model,
            on_choose: None,
            theme: &Theme::DARK,
        }
    }
    /// An option was clicked (the host calls `choose` and ticks for the auto-advance).
    pub fn on_choose(mut self, f: impl Fn(usize, &mut Window, &mut App) + 'static) -> Self {
        self.on_choose = Some(Rc::new(f));
        self
    }
    themed!();
}

impl RenderOnce for ApprovalQuestionsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let d = self.model;
        if d.phase == DeckPhase::Done {
            return div()
                .flex()
                .gap(px(8.))
                .px(px(14.))
                .py(px(8.))
                .rounded_full()
                .border_1()
                .border_color(c(t, ColorRole::BorderDefault))
                .text_color(c(t, ColorRole::FgDefault))
                .child(
                    div()
                        .text_color(c(t, ColorRole::StatusSuccessFg))
                        .child("✓"),
                )
                .child("Answers sent")
                .child(
                    div()
                        .text_color(c(t, ColorRole::FgMuted))
                        .child("Start over"),
                )
                .into_any_element();
        }
        let Some(q) = d.current().cloned() else {
            return div().into_any_element();
        };
        let (pos, n) = d.position();
        let primary = if d.last() { "Send ⏎" } else { "Continue ⏎" };
        div()
            .id("oxyn-a11y")
            .role(Role::Group)
            .aria_label(SharedString::from(format!("Question {pos} of {n}")))
            .w(px(440.))
            .p(px(14.))
            .flex()
            .flex_col()
            .gap(px(12.))
            .rounded(px(16.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .flex()
                    .child(
                        div()
                            .flex_1()
                            .text_size(px(12.))
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(format!("{pos} / {n}")),
                    )
                    .child(div().text_color(c(t, ColorRole::FgMuted)).child("‹  ›  ×")),
            )
            .child(
                div()
                    .text_size(px(15.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(q.prompt.clone()),
            )
            .child(
                div()
                    .id("oxyn-a11y")
                    .role(if q.kind == QuestionKind::Multi {
                        Role::Group
                    } else {
                        Role::RadioGroup
                    })
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .children(deck_rows(t, &d, self.on_choose))
                    .child(
                        shell(t, d.in_text(), false).child(
                            div().text_color(c(t, ColorRole::FgSubtle)).child(
                                d.text
                                    .get(d.index)
                                    .filter(|s| !s.is_empty())
                                    .cloned()
                                    .unwrap_or("Something else…".into()),
                            ),
                        ),
                    ),
            )
            .child(
                div()
                    .flex()
                    .gap(px(6.))
                    .child(div().flex_1())
                    .child(
                        div()
                            .px(px(14.))
                            .h(px(32.))
                            .flex()
                            .items_center()
                            .text_color(c(t, ColorRole::FgMuted))
                            .child("Skip"),
                    )
                    .child(
                        div()
                            .px(px(14.))
                            .h(px(32.))
                            .flex()
                            .items_center()
                            .rounded_full()
                            .bg(c(t, ColorRole::FgDefault))
                            .text_color(c(t, ColorRole::BgBase))
                            .when(!d.has_answer(), |d| d.opacity(0.4))
                            .child(primary),
                    ),
            )
            .into_any_element()
    }
}

/// `smart-form-fill`: the pasted text with the source span, fields that fade in with their
/// confidence, review marks and the footer.
#[derive(IntoElement)]
pub struct SmartFillEl {
    model: SmartFill,
    now: ClockMs,
    theme: &'static Theme,
}

impl SmartFillEl {
    /// From the model at `now`.
    pub fn new(model: SmartFill, now: ClockMs) -> Self {
        Self {
            model,
            now,
            theme: &Theme::DARK,
        }
    }
    themed!();
}

impl RenderOnce for SmartFillEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.model;
        let now = self.now;
        let hl = m.highlight(now);
        let text = m.text.clone();
        let src: Vec<AnyElement> = match hl {
            Some((a, b))
                if b <= text.len() && text.is_char_boundary(a) && text.is_char_boundary(b) =>
            {
                vec![
                    div().child(text[..a].to_string()).into_any_element(),
                    div()
                        .bg(alpha(c(t, ColorRole::FgDefault), 0.16))
                        .text_color(c(t, ColorRole::FgDefault))
                        .child(text[a..b].to_string())
                        .into_any_element(),
                    div().child(text[b..].to_string()).into_any_element(),
                ]
            }
            _ => vec![div().child(text.clone()).into_any_element()],
        };
        let review = m.needs_review().len();
        div()
            .w(px(720.))
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(14.))
            .rounded(px(16.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .text_size(px(15.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("New contact"),
            )
            .child(
                div()
                    .flex()
                    .gap(px(14.))
                    .child(
                        div()
                            .w(px(240.))
                            .p(px(10.))
                            .rounded(px(12.))
                            .bg(c(t, ColorRole::SurfaceSunken))
                            .text_size(px(12.5))
                            .text_color(c(t, ColorRole::FgMuted))
                            .children(src),
                    )
                    .child(div().flex_1().flex().flex_wrap().gap(px(10.)).children(
                        m.fields.iter().map(|f| {
                            let (fade, tint) = m.reveal(&f.id, now, false);
                            let mk = m.mark(&f.id);
                            let conf = m.confidence(&f.id);
                            div()
                                .w(px(200.))
                                .flex()
                                .flex_col()
                                .gap(px(4.))
                                .child(
                                    div()
                                        .flex()
                                        .gap(px(6.))
                                        .text_size(px(12.))
                                        .child(f.label.clone())
                                        .child(div().flex_1())
                                        .children(conf.map(|cf| {
                                            div()
                                                .text_size(px(11.))
                                                .text_color(if mk == FieldMark::Review {
                                                    c(t, ColorRole::StatusWarningFg)
                                                } else {
                                                    c(t, ColorRole::FgSubtle)
                                                })
                                                .child(if mk.word().is_empty() {
                                                    format!("● {cf}%")
                                                } else {
                                                    format!("● {}", mk.word())
                                                })
                                        })),
                                )
                                .child(
                                    shell(t, false, false)
                                        .id(SharedString::from(format!("sff-{}", f.id)))
                                        .role(Role::TextInput)
                                        .aria_label(SharedString::from(f.label.clone()))
                                        .when(mk == FieldMark::Review, |d| {
                                            d.border_color(c(t, ColorRole::StatusWarningBorder))
                                        })
                                        .when(tint > 0.0, |d| {
                                            d.bg(alpha(c(t, ColorRole::FgDefault), 0.12 * tint))
                                        })
                                        .child(div().opacity(fade).child(
                                            m.values.get(&f.id).cloned().unwrap_or_default(),
                                        )),
                                )
                        }),
                    )),
            )
            .child(
                div()
                    .flex()
                    .gap(px(6.))
                    .text_size(px(12.))
                    .text_color(c(t, ColorRole::FgMuted))
                    .child(if review > 0 {
                        format!("{review} to review")
                    } else {
                        m.announce.clone()
                    })
                    .child(div().flex_1())
                    .child(
                        div()
                            .px(px(12.))
                            .h(px(28.))
                            .flex()
                            .items_center()
                            .rounded(px(6.))
                            .bg(c(t, ColorRole::FgDefault))
                            .text_color(c(t, ColorRole::BgBase))
                            .child("Save contact"),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use oxyn_core::primitives::{
        password::Policy, questions::Question, smart_fill::contact_fields,
    };

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
        let _ = DateReelEl::new(DateReel::new(
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
