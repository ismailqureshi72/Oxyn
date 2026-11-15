//! Core primitives for GPUI (plan #23), drawn from the same `oxyn_core::primitives` models as the
//! Dioxus components. Elements are `RenderOnce`: the host owns the model, feeds it events and the
//! shared clock time, and re-renders. Buttons, icon buttons, switches, checkboxes and chips take
//! an `on_click` listener; the rest are driven entirely by their model.
//!
//! Gaps against Dioxus (listed in the PR): keyboard handling and focus rings are the host's job
//! (no `FocusHandle` per element yet); overlays (tooltip, popover, menu, select list) render in
//! flow below their trigger instead of in a deferred, anchored layer with flip; text field shows
//! its value but editing and IME arrive with the shared input handler; no blur effects.

use std::rc::Rc;

use gpui::{
    AnyElement, App, ClickEvent, ElementId, FontWeight, Hsla, IntoElement, ParentElement,
    RenderOnce, Role, SharedString, Styled, Window, div, prelude::*, px,
};
use oxyn_core::{
    ClockMs,
    primitives::{
        button::{
            ActionButton, ActionPhase, ButtonSize, ButtonVariant, ExpandingGroup, GroupItem,
            SplitButton, hidden_dividers, press_scale,
        },
        display::{
            AgentRing, Avatar, AvatarShape, Badge, Chip, ChipKind, Delta, EmptyLayout, EmptyState,
            EntityChip, Fill, Monogram, Platform, Separator, Spinner, Tone, agent_cell_opacity,
            agent_mark, monogram_roles, parse_shortcut,
        },
        menu::{ItemKind, Menu},
        segmented::Segmented,
        select::{ListRow, Select},
        slider::Slider,
        text_field::{CountLevel, TextField},
        ticker::{Column, CountRule, NumberTicker, Reaction},
        toggle::{Check, RadioGroup, Switch},
    },
};
use oxyn_tokens::{ColorRole, Theme};

use crate::hsla;

type Click = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

fn c(t: &Theme, r: ColorRole) -> Hsla {
    hsla(t.color(r))
}

fn alpha(mut h: Hsla, a: f32) -> Hsla {
    h.a *= a;
    h
}

/// Glyphs stand in for the outlined icon set until GPUI SVG icons land (gap).
fn glyph(name: &str) -> &'static str {
    match name {
        "check" => "✓",
        "x" => "×",
        "plus" => "+",
        "copy" => "⧉",
        "trash" => "⌫",
        "chevron" => "⌄",
        "chevron-right" => "›",
        "search" => "⌕",
        "retry" => "↻",
        "up" => "↑",
        "play" => "▶",
        "heart" => "♥",
        "comment" => "◌",
        "repost" => "⇄",
        "share" => "⇪",
        "more" => "⋯",
        "spark" => "✦",
        "star" => "☆",
        "branch" => "⑂",
        "folder" => "▭",
        "file" => "▯",
        "pencil" => "✎",
        "link" => "⛓",
        "eye" => "◉",
        "info" => "ⓘ",
        _ => "•",
    }
}

fn kbd(t: &'static Theme, s: &str) -> impl IntoElement + use<> {
    let chords = parse_shortcut(s, Platform::Mac);
    div()
        .flex()
        .gap(px(2.))
        .children(chords.into_iter().flatten().map(move |k| {
            div()
                .px(px(4.))
                .h(px(16.))
                .flex()
                .items_center()
                .rounded(px(4.))
                .text_size(px(10.5))
                .bg(alpha(c(t, ColorRole::FgDefault), 0.1))
                .child(k.glyph)
        }))
}

// ---------------------------------------------------------------------------------------------
// Buttons

/// `button`: variants, sizes, loading (spinner + progress), disabled with a reason, hold fill,
/// shortcut chip.
#[derive(IntoElement)]
pub struct ButtonEl {
    id: ElementId,
    label: SharedString,
    variant: ButtonVariant,
    size: ButtonSize,
    loading: Option<f32>,
    disabled: Option<SharedString>,
    icon: Option<&'static str>,
    shortcut: Option<String>,
    hold: Option<f32>,
    pressed: bool,
    now: ClockMs,
    on_click: Option<Click>,
    theme: &'static Theme,
}

impl ButtonEl {
    /// A button.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: ButtonVariant::Primary,
            size: ButtonSize::Md,
            loading: None,
            disabled: None,
            icon: None,
            shortcut: None,
            hold: None,
            pressed: false,
            now: ClockMs(0),
            on_click: None,
            theme: &Theme::DARK,
        }
    }
    /// Variant.
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    /// Size.
    pub fn size(mut self, s: ButtonSize) -> Self {
        self.size = s;
        self
    }
    /// Loading; `progress` 0..1 fills the button (progress style), `None` shows the spinner.
    pub fn loading(mut self, progress: Option<f32>, now: ClockMs) -> Self {
        self.loading = Some(progress.unwrap_or(-1.0));
        self.now = now;
        self
    }
    /// Disabled with a reason (used as the accessible description).
    pub fn disabled(mut self, reason: impl Into<SharedString>) -> Self {
        self.disabled = Some(reason.into());
        self
    }
    /// Leading icon.
    pub fn icon(mut self, name: &str) -> Self {
        self.icon = Some(glyph(name));
        self
    }
    /// Shortcut chip.
    pub fn shortcut(mut self, s: &str) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    /// Hold-to-confirm fill (0..1) from [`oxyn_core::primitives::button::HoldConfirm`].
    pub fn hold(mut self, fill: f32) -> Self {
        self.hold = Some(fill);
        self
    }
    /// Draw pressed (scale 0.97).
    pub fn pressed(mut self, p: bool) -> Self {
        self.pressed = p;
        self
    }
    /// Theme.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Click listener (ignored while loading or disabled).
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

fn variant_colors(t: &'static Theme, v: ButtonVariant) -> (Hsla, Hsla, Option<Hsla>) {
    let none = alpha(c(t, ColorRole::BgBase), 0.0);
    match v {
        ButtonVariant::Primary => (
            c(t, ColorRole::AccentSolid),
            c(t, ColorRole::FgOnAccent),
            None,
        ),
        ButtonVariant::Secondary => (
            c(t, ColorRole::SurfaceRaised),
            c(t, ColorRole::FgDefault),
            Some(c(t, ColorRole::BorderDefault)),
        ),
        ButtonVariant::Ghost | ButtonVariant::Link => (none, c(t, ColorRole::FgDefault), None),
        ButtonVariant::Quiet => (none, c(t, ColorRole::FgMuted), None),
        ButtonVariant::Outline => (
            none,
            c(t, ColorRole::FgDefault),
            Some(c(t, ColorRole::BorderStrong)),
        ),
        ButtonVariant::Destructive => (
            c(t, ColorRole::StatusDangerFg),
            c(t, ColorRole::FgOnAccent),
            None,
        ),
        ButtonVariant::Success => (
            c(t, ColorRole::StatusSuccessBg),
            c(t, ColorRole::StatusSuccessFg),
            Some(alpha(c(t, ColorRole::StatusSuccessFg), 0.3)),
        ),
    }
}

impl RenderOnce for ButtonEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let s = self.size;
        let (bg, fg, border) = variant_colors(t, self.variant);
        let inert = self.loading.is_some() || self.disabled.is_some();
        let scale = press_scale(false, self.pressed, false);
        let h = s.height() * scale;
        let spinner = self.loading.filter(|p| *p < 0.0).map(|_| {
            SpinnerEl::new(Spinner {
                size: s.icon(),
                ..Default::default()
            })
            .now(self.now)
            .color(fg)
        });
        let progress = self.loading.filter(|p| *p >= 0.0);
        let label = self.label.clone();
        let on_click = self.on_click.clone();
        div()
            .id(self.id)
            .role(Role::Button)
            .aria_label(label.clone())
            .relative()
            .overflow_hidden()
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(6.))
            .h(px(h))
            .px(px(s.pad_x()))
            .rounded(px(s.radius()))
            .bg(bg)
            .text_color(fg)
            .text_size(px(s.font()))
            .font_weight(FontWeight::MEDIUM)
            .when_some(border, |d, b| d.border_1().border_color(b))
            .when(self.disabled.is_some(), |d| d.opacity(0.45))
            .when(!inert, |d| d.cursor_pointer().hover(|st| st.opacity(0.92)))
            .when_some(progress.or(self.hold), |d, p| {
                d.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .h_full()
                        .w(gpui::relative(p.clamp(0.0, 1.0)))
                        .bg(alpha(fg, 0.2)),
                )
            })
            .children(spinner)
            .when(self.loading.is_none(), |d| {
                d.when_some(self.icon, |d, i| d.child(i))
            })
            .child(label)
            .when_some(self.shortcut.as_deref(), |d, k| d.child(kbd(t, k)))
            .when_some(on_click.filter(|_| !inert), |d, f| {
                d.on_click(move |e, w, cx| f(e, w, cx))
            })
    }
}

/// `icon-button`: ghost / secondary / primary, a toggle state and a dot / count badge.
#[derive(IntoElement)]
pub struct IconButtonEl {
    id: ElementId,
    icon: &'static str,
    label: SharedString,
    variant: ButtonVariant,
    size: f32,
    pressed: Option<bool>,
    badge: Option<u64>,
    on_click: Option<Click>,
    theme: &'static Theme,
}

impl IconButtonEl {
    /// An icon button (`label` is its accessible name and tooltip text).
    pub fn new(id: impl Into<ElementId>, icon: &str, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            icon: glyph(icon),
            label: label.into(),
            variant: ButtonVariant::Ghost,
            size: 32.0,
            pressed: None,
            badge: None,
            on_click: None,
            theme: &Theme::DARK,
        }
    }
    /// Variant (ghost, secondary, primary).
    pub fn variant(mut self, v: ButtonVariant) -> Self {
        self.variant = v;
        self
    }
    /// Size px (16–40).
    pub fn size(mut self, s: f32) -> Self {
        self.size = s;
        self
    }
    /// Toggle state.
    pub fn pressed(mut self, p: bool) -> Self {
        self.pressed = Some(p);
        self
    }
    /// Badge: 0 = dot.
    pub fn badge(mut self, n: u64) -> Self {
        self.badge = Some(n);
        self
    }
    /// Click listener.
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for IconButtonEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let (bg, fg, border) = variant_colors(t, self.variant);
        let on = self.pressed.unwrap_or(false);
        let bg = if on {
            c(t, ColorRole::SurfaceSelected)
        } else {
            bg
        };
        let fg = if self.variant == ButtonVariant::Ghost && !on {
            c(t, ColorRole::FgMuted)
        } else {
            fg
        };
        let f = self.on_click.clone();
        div()
            .id(self.id)
            .role(Role::Button)
            .aria_label(self.label.clone())
            .relative()
            .size(px(self.size))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(self.size * 0.25))
            .bg(bg)
            .text_color(fg)
            .text_size(px((self.size * 0.5).max(10.0)))
            .cursor_pointer()
            .hover(|s| s.bg(c(t, ColorRole::SurfaceHover)))
            .when_some(border, |d, b| d.border_1().border_color(b))
            .child(self.icon)
            .when_some(self.badge, |d, n| {
                let dot = n == 0;
                d.child(
                    div()
                        .absolute()
                        .top(px(if dot { 3. } else { -3. }))
                        .right(px(if dot { 3. } else { -3. }))
                        .h(px(if dot { 7. } else { 14. }))
                        .min_w(px(if dot { 7. } else { 14. }))
                        .px(px(if dot { 0. } else { 3. }))
                        .rounded_full()
                        .bg(if dot {
                            c(t, ColorRole::StatusInfoFg)
                        } else {
                            c(t, ColorRole::FgDefault)
                        })
                        .text_color(c(t, ColorRole::BgBase))
                        .text_size(px(9.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .when(!dot, |d| d.child(n.to_string())),
                )
            })
            .when_some(f, |d, f| d.on_click(move |e, w, cx| f(e, w, cx)))
    }
}

/// `action-button`: pending / success / error with retry back-off, from the core model.
#[derive(IntoElement)]
pub struct ActionButtonEl {
    model: ActionButton,
    now: ClockMs,
    theme: &'static Theme,
}

impl ActionButtonEl {
    /// From the model at `now`.
    pub fn new(model: ActionButton, now: ClockMs) -> Self {
        Self {
            model,
            now,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for ActionButtonEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let phase = self.model.phase(self.now);
        let (fg, lead) = match &phase {
            ActionPhase::Idle => (c(t, ColorRole::FgDefault), Some("↑")),
            ActionPhase::Pending => (c(t, ColorRole::FgMuted), None),
            ActionPhase::Success => (c(t, ColorRole::StatusSuccessFg), Some("✓")),
            ActionPhase::Error { .. } => (c(t, ColorRole::StatusDangerFg), Some("↻")),
        };
        let widest = self.model.widest().chars().count() as f32;
        div()
            .flex()
            .items_center()
            .gap(px(6.))
            .h(px(28.))
            .px(px(10.))
            .rounded(px(7.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_color(fg)
            .text_size(px(13.))
            .when(phase == ActionPhase::Pending, |d| {
                d.child(
                    SpinnerEl::new(Spinner {
                        size: 13.0,
                        ..Default::default()
                    })
                    .now(self.now)
                    .color(fg),
                )
            })
            .when_some(lead, |d, g| d.child(g))
            // Reserve the widest label so the width never jumps.
            .child(
                div()
                    .min_w(px(widest * 7.0))
                    .child(self.model.text(self.now)),
            )
    }
}

/// `split-button`: main half + menu half; the menu renders below when open.
#[derive(IntoElement)]
pub struct SplitButtonEl {
    model: SplitButton,
    open: bool,
    primary: bool,
    theme: &'static Theme,
}

impl SplitButtonEl {
    /// From the model.
    pub fn new(model: SplitButton, open: bool) -> Self {
        Self {
            model,
            open,
            primary: true,
            theme: &Theme::DARK,
        }
    }
    /// Secondary look.
    pub fn secondary(mut self) -> Self {
        self.primary = false;
        self
    }
}

impl RenderOnce for SplitButtonEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let (bg, fg, border) = variant_colors(
            t,
            if self.primary {
                ButtonVariant::Primary
            } else {
                ButtonVariant::Secondary
            },
        );
        let (_, main) = self.model.main();
        let menu: Vec<String> = self
            .model
            .menu()
            .iter()
            .map(|(_, l)| l.to_string())
            .collect();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .flex()
                    .h(px(32.))
                    .rounded(px(8.))
                    .bg(bg)
                    .text_color(fg)
                    .text_size(px(13.))
                    .when_some(border, |d, b| d.border_1().border_color(b))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .px(px(12.))
                            .child(main.to_string()),
                    )
                    .child(div().w(px(1.)).h_full().bg(alpha(fg, 0.25)))
                    .child(div().flex().items_center().px(px(8.)).child("⌄")),
            )
            .when(self.open, |d| {
                d.child(menu_rows(
                    t,
                    menu.into_iter().map(|l| (l, false, false)).collect(),
                    None,
                ))
            })
    }
}

fn menu_rows(
    t: &'static Theme,
    rows: Vec<(String, bool, bool)>,
    hl: Option<usize>,
) -> impl IntoElement + use<> {
    div()
        .flex()
        .flex_col()
        .p(px(4.))
        .min_w(px(200.))
        .rounded(px(10.))
        .bg(c(t, ColorRole::SurfaceOverlay))
        .border_1()
        .border_color(c(t, ColorRole::BorderDefault))
        .shadow_lg()
        .text_size(px(13.))
        .children(
            rows.into_iter()
                .enumerate()
                .map(move |(i, (l, danger, disabled))| {
                    let on = hl == Some(i);
                    div()
                        .h(px(32.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .rounded(px(6.))
                        .text_color(if danger {
                            c(t, ColorRole::StatusDangerFg)
                        } else if disabled {
                            c(t, ColorRole::FgDisabled)
                        } else {
                            c(t, ColorRole::FgDefault)
                        })
                        .when(on, |d| {
                            d.bg(if danger {
                                c(t, ColorRole::StatusDangerBg)
                            } else {
                                c(t, ColorRole::SurfaceHover)
                            })
                        })
                        .child(l)
                }),
        )
}

/// `button-group` (joined) and `floating-button-group` (`floating(true)`): segments with one
/// highlight; dividers next to it fade out.
#[derive(IntoElement)]
pub struct ButtonGroupEl {
    items: Vec<GroupItem>,
    highlight: Option<usize>,
    selected: Option<String>,
    vertical: bool,
    floating: bool,
    theme: &'static Theme,
}

impl ButtonGroupEl {
    /// A joined group.
    pub fn new(items: Vec<GroupItem>) -> Self {
        Self {
            items,
            highlight: None,
            selected: None,
            vertical: false,
            floating: false,
            theme: &Theme::DARK,
        }
    }
    /// Highlighted segment (pointer / keyboard).
    pub fn highlight(mut self, i: Option<usize>) -> Self {
        self.highlight = i;
        self
    }
    /// Selected segment id.
    pub fn selected(mut self, id: &str) -> Self {
        self.selected = Some(id.into());
        self
    }
    /// Vertical.
    pub fn vertical(mut self) -> Self {
        self.vertical = true;
        self
    }
    /// Floating tray look (separate soft buttons, no dividers).
    pub fn floating(mut self) -> Self {
        self.floating = true;
        self
    }
}

impl RenderOnce for ButtonGroupEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let hidden = hidden_dividers(self.highlight, self.items.len());
        let floating = self.floating;
        let mut kids: Vec<AnyElement> = vec![];
        for (i, it) in self.items.iter().enumerate() {
            if i > 0 && !floating {
                let off = hidden[i - 1];
                let hair = if self.vertical {
                    div().h(px(1.)).mx(px(5.))
                } else {
                    div().w(px(1.)).my(px(5.))
                };
                kids.push(
                    hair.bg(c(t, ColorRole::BorderDefault))
                        .opacity(if off { 0.0 } else { 1.0 })
                        .into_any_element(),
                );
            }
            let on = self.highlight == Some(i);
            let sel = self.selected.as_deref() == Some(it.id.as_str());
            kids.push(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(if floating { 32. } else { 26. }))
                    .px(px(if floating { 8. } else { 10. }))
                    .rounded(px(if floating { 8. } else { 6. }))
                    .text_color(if on || sel || !floating {
                        c(t, ColorRole::FgDefault)
                    } else {
                        c(t, ColorRole::FgMuted)
                    })
                    .when(on, |d| d.bg(c(t, ColorRole::SurfaceHover)))
                    .when(sel, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                    .when(it.disabled, |d| d.opacity(0.4))
                    .when_some(it.icon.as_deref(), |d, ic| d.child(glyph(ic)))
                    .when(!floating || it.icon.is_none(), |d| {
                        d.child(it.label.clone())
                    })
                    .when_some(it.count, |d, n| {
                        d.child(
                            div()
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(n.to_string()),
                        )
                    })
                    .into_any_element(),
            );
        }
        div()
            .flex()
            .when(self.vertical, |d| d.flex_col())
            .gap(px(if floating { 4. } else { 0. }))
            .p(px(if floating { 4. } else { 2. }))
            .rounded(px(if floating { 12. } else { 8. }))
            .text_size(px(13.))
            .when(floating, |d| {
                d.bg(c(t, ColorRole::SurfaceOverlay))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .shadow_lg()
            })
            .when(!floating, |d| {
                d.bg(c(t, ColorRole::SurfaceRaised))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
            })
            .children(kids)
    }
}

/// `expanding-button-group`: the open item shows its label (or its done label).
#[derive(IntoElement)]
pub struct ExpandingGroupEl {
    model: ExpandingGroup,
    now: ClockMs,
    theme: &'static Theme,
}

impl ExpandingGroupEl {
    /// From the model at `now`.
    pub fn new(model: ExpandingGroup, now: ClockMs) -> Self {
        Self {
            model,
            now,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for ExpandingGroupEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.model;
        div()
            .flex()
            .gap(px(4.))
            .p(px(3.))
            .rounded_full()
            .bg(c(t, ColorRole::SurfaceSunken))
            .border_1()
            .border_color(c(t, ColorRole::BorderSubtle))
            .text_size(px(13.))
            .children(m.items.iter().enumerate().map(|(i, (_, label, done))| {
                let open = m.shows_label(i, self.now);
                let is_done = m.done(i, self.now);
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(32.))
                    .px(px(9.))
                    .rounded_full()
                    .text_color(if is_done {
                        c(t, ColorRole::StatusSuccessFg)
                    } else if open {
                        c(t, ColorRole::FgDefault)
                    } else {
                        c(t, ColorRole::FgMuted)
                    })
                    .when(open, |d| d.bg(c(t, ColorRole::SurfaceRaised)))
                    .child(if is_done { "✓" } else { "•" })
                    .when(open, |d| {
                        d.child(div().min_w(px(m.slot(i) as f32 * 7.0)).child(if is_done {
                            done.clone()
                        } else {
                            label.clone()
                        }))
                    })
            }))
    }
}

// ---------------------------------------------------------------------------------------------
// Display

/// `badge`.
#[derive(IntoElement)]
pub struct BadgeEl {
    badge: Badge,
    phase: f32,
    size: BadgeSize,
    roll: f32,
    theme: &'static Theme,
}

/// Badge size: 20, 24 or 28 px tall with 11.5, 12 or 14 px text.
#[derive(Clone, Copy, Default, PartialEq, Eq)]
pub enum BadgeSize {
    /// 20 px.
    #[default]
    Sm,
    /// 24 px.
    Md,
    /// 28 px.
    Lg,
}

impl BadgeSize {
    /// `sm` · `md` · `lg`; anything else is `sm`.
    pub fn from_id(id: &str) -> Self {
        match id {
            "md" => Self::Md,
            "lg" => Self::Lg,
            _ => Self::Sm,
        }
    }
    /// (height, horizontal padding, text size, corner radius) in px.
    pub fn metrics(self) -> (f32, f32, f32, f32) {
        match self {
            Self::Sm => (20., 7., 11.5, 5.),
            Self::Md => (24., 9., 12., 6.),
            Self::Lg => (28., 11., 14., 7.),
        }
    }
}

impl BadgeEl {
    /// A badge; `phase` 0..1 drives the live halo (from `display::live_halo`).
    pub fn new(badge: Badge) -> Self {
        Self {
            badge,
            phase: 0.0,
            size: BadgeSize::Sm,
            roll: 1.0,
            theme: &Theme::DARK,
        }
    }
    /// Size (`sm` default).
    pub fn size(mut self, size: BadgeSize) -> Self {
        self.size = size;
        self
    }
    /// Label swap progress 0..1: a new label rises in (4 px, fading) as this goes to 1.
    pub fn roll(mut self, t: f32) -> Self {
        self.roll = t.clamp(0.0, 1.0);
        self
    }
    /// Live halo opacity this frame.
    pub fn halo(mut self, opacity: f32) -> Self {
        self.phase = opacity;
        self
    }
}

impl RenderOnce for BadgeEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let b = &self.badge;
        let (fg, bg) = b.tone.roles();
        let (fg, bg) = (c(t, fg), c(t, bg));
        let (bg, fgc, border) = match b.fill {
            Fill::Soft => (bg, fg, None),
            Fill::Outline => (alpha(bg, 0.0), fg, Some(alpha(fg, 0.45))),
            Fill::Solid => (fg, c(t, ColorRole::BgBase), None),
            Fill::Mono => (alpha(bg, 0.0), fg, Some(c(t, ColorRole::BorderDefault))),
        };
        let (h, pad, size, radius) = self.size.metrics();
        let roll = self.roll;
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(5.))
            .h(px(h))
            .px(px(pad))
            .rounded(px(if b.pill || b.count.is_some() {
                999.
            } else {
                radius
            }))
            .bg(bg)
            .text_color(fgc)
            .text_size(px(size))
            .when_some(border, |d, x| d.border_1().border_color(x))
            .when(b.dot || b.live, |d| {
                d.child(div().size(px(6.)).rounded_full().bg(fgc).when(b.live, |d| {
                    d.border_2()
                        .border_color(alpha(fgc, self.phase.clamp(0.0, 1.0)))
                }))
            })
            .child(
                div()
                    .relative()
                    .top(px((1.0 - roll) * 4.0))
                    .opacity(roll)
                    .child(b.text()),
            )
    }
}

/// `chip`: static, toggle or removable.
#[derive(IntoElement)]
pub struct ChipEl {
    id: ElementId,
    chip: Chip,
    on_click: Option<Click>,
    theme: &'static Theme,
}

impl ChipEl {
    /// A chip.
    pub fn new(id: impl Into<ElementId>, chip: Chip) -> Self {
        Self {
            id: id.into(),
            chip,
            on_click: None,
            theme: &Theme::DARK,
        }
    }
    /// Click (toggle) or × (removable).
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for ChipEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let ch = &self.chip;
        let on = ch.kind == ChipKind::Toggle && ch.selected;
        let f = self.on_click.clone();
        div()
            .id(self.id)
            .flex()
            .flex_none()
            .items_center()
            .gap(px(6.))
            .h(px(28.))
            .px(px(10.))
            .rounded_full()
            .text_size(px(12.5))
            .border_1()
            .border_color(if on {
                alpha(c(t, ColorRole::BorderDefault), 0.0)
            } else {
                c(t, ColorRole::BorderDefault)
            })
            .bg(if on {
                c(t, ColorRole::FgDefault)
            } else {
                c(t, ColorRole::SurfaceRaised)
            })
            .text_color(if on {
                c(t, ColorRole::BgBase)
            } else {
                c(t, ColorRole::FgDefault)
            })
            .when(ch.disabled, |d| d.opacity(0.45))
            .when(on, |d| d.child("✓"))
            .when_some(ch.dot, |d, tone| {
                d.child(div().size(px(7.)).rounded_full().bg(c(t, tone.roles().0)))
            })
            .child(ch.label.clone())
            .when_some(ch.count, |d, n| {
                d.child(div().opacity(0.6).child(n.to_string()))
            })
            .when(ch.kind == ChipKind::Removable, |d| {
                d.child(div().text_color(c(t, ColorRole::FgSubtle)).child("×"))
            })
            .when_some(f.filter(|_| !ch.disabled), |d, f| {
                d.cursor_pointer().on_click(move |e, w, cx| f(e, w, cx))
            })
    }
}

/// `value-pill` (Beautiful UI): tone or a delta arrow.
#[derive(IntoElement)]
pub struct ValuePillEl {
    value: SharedString,
    tone: Tone,
    delta: Option<Delta>,
    theme: &'static Theme,
}

impl ValuePillEl {
    /// A pill.
    pub fn new(value: impl Into<SharedString>, tone: Tone) -> Self {
        Self {
            value: value.into(),
            tone,
            delta: None,
            theme: &Theme::DARK,
        }
    }
    /// Delta arrow; the tone follows `good_up`.
    pub fn delta(mut self, d: Delta, good_up: bool) -> Self {
        self.delta = Some(d);
        self.tone = d.tone(good_up);
        self
    }
}

impl RenderOnce for ValuePillEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let (fg, bg) = self.tone.roles();
        div()
            .flex()
            .flex_none()
            .items_center()
            .gap(px(3.))
            .h(px(20.))
            .px(px(7.))
            .rounded(px(6.))
            .bg(c(t, bg))
            .text_color(c(t, fg))
            .text_size(px(12.))
            .font_family("JetBrains Mono")
            .when_some(self.delta, |d, x| {
                d.child(div().text_size(px(8.)).child(x.glyph()))
            })
            .child(self.value)
    }
}

fn monogram(t: &'static Theme, m: &Monogram, size: f32, square: bool) -> impl IntoElement + use<> {
    let (bg, fg) = monogram_roles(m.colour);
    div()
        .size(px(size))
        .flex()
        .flex_none()
        .items_center()
        .justify_center()
        .rounded(px(if square { size * 0.25 } else { size / 2.0 }))
        .bg(c(t, bg))
        .text_color(c(t, fg))
        .text_size(px(size * 0.45))
        .font_weight(FontWeight::SEMIBOLD)
        .child(m.letters.clone())
}

/// `monogram`.
#[derive(IntoElement)]
pub struct MonogramEl {
    monogram: Monogram,
    size: f32,
    square: bool,
    theme: &'static Theme,
}

impl MonogramEl {
    /// A monogram.
    pub fn new(monogram: Monogram, size: f32) -> Self {
        Self {
            monogram,
            size,
            square: false,
            theme: &Theme::DARK,
        }
    }
    /// Rounded square.
    pub fn square(mut self) -> Self {
        self.square = true;
        self
    }
}

impl RenderOnce for MonogramEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        monogram(self.theme, &self.monogram, self.size, self.square)
    }
}

/// `entity-chip` (Beautiful UI) with status dot; the hover card renders below when `card_open`.
#[derive(IntoElement)]
pub struct EntityChipEl {
    entity: EntityChip,
    card_open: bool,
    theme: &'static Theme,
}

impl EntityChipEl {
    /// An entity chip.
    pub fn new(entity: EntityChip) -> Self {
        Self {
            entity,
            card_open: false,
            theme: &Theme::DARK,
        }
    }
    /// Show the hover card.
    pub fn card(mut self, open: bool) -> Self {
        self.card_open = open;
        self
    }
}

impl RenderOnce for EntityChipEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let e = &self.entity;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(5.))
                    .h(px(22.))
                    .pl(px(3.))
                    .pr(px(7.))
                    .rounded_full()
                    .bg(c(t, ColorRole::SurfaceHover))
                    .text_size(px(12.5))
                    .text_color(c(t, ColorRole::FgDefault))
                    .child(monogram(t, &e.monogram, 16.0, false))
                    .child(e.name.clone())
                    .when_some(e.status, |d, p| {
                        d.child(div().size(px(6.)).rounded_full().bg(c(t, p.role())))
                    }),
            )
            .when(self.card_open && !e.card.is_empty(), |d| {
                d.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .p(px(10.))
                        .rounded(px(10.))
                        .bg(c(t, ColorRole::SurfaceOverlay))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .text_size(px(12.))
                        .children(e.card.iter().map(|(k, v)| {
                            div()
                                .flex()
                                .gap(px(10.))
                                .child(
                                    div()
                                        .w(px(56.))
                                        .text_color(c(t, ColorRole::FgSubtle))
                                        .child(k.clone()),
                                )
                                .child(v.clone())
                        })),
                )
            })
    }
}

/// `avatar` with agent ring: initials or the agent's pixel mark (twinkling while it works), a
/// presence dot, the state ring (or a progress ring drawn as a bottom-up fill bar, gap) and a
/// model badge. Images are the host's (GPUI `img`), not drawn here (gap).
#[derive(IntoElement)]
pub struct AvatarEl {
    avatar: Avatar,
    phase: f32,
    theme: &'static Theme,
}

impl AvatarEl {
    /// An avatar; `phase` 0..1 is the 1.2 s working wave (`MotionClock::phase`).
    pub fn new(avatar: Avatar, phase: f32) -> Self {
        Self {
            avatar,
            phase,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for AvatarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let a = &self.avatar;
        let s = a.size;
        let m = a.monogram();
        let (bg, fg) = monogram_roles(m.colour);
        let pad = (s * 0.12).clamp(2.0, 5.0);
        let face = div()
            .size(px(s))
            .flex()
            .items_center()
            .justify_center()
            .rounded(px(a.radius()))
            .bg(c(t, bg))
            .text_color(c(t, fg))
            .overflow_hidden();
        let face = if a.agent {
            let grid = agent_mark(a.seed.as_deref().unwrap_or(&a.name));
            let cell = (s * 0.62 / 5.0).floor().max(1.0);
            face.child(
                div()
                    .flex()
                    .flex_col()
                    .children(grid.iter().enumerate().map(|(r, row)| {
                        div()
                            .flex()
                            .children(row.iter().enumerate().map(move |(ci, on)| {
                                div().size(px(cell)).when(*on, |d| {
                                    d.bg(alpha(
                                        c(t, fg),
                                        agent_cell_opacity(r, ci, self.phase, a.working, false),
                                    ))
                                })
                            }))
                    })),
            )
        } else {
            face.text_size(px(a.font()))
                .font_weight(FontWeight::SEMIBOLD)
                .child(m.letters.clone())
        };
        let ring = a.ring;
        let outer = s + pad * 2.0;
        div()
            .relative()
            .size(px(outer))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .when(ring != AgentRing::None, |d| {
                let col = c(t, ring.role());
                let col = if ring == AgentRing::Breathe {
                    alpha(
                        col,
                        0.35 + 0.65 * (self.phase * std::f32::consts::TAU).cos().abs(),
                    )
                } else {
                    col
                };
                d.child(
                    div()
                        .absolute()
                        .top_0()
                        .left_0()
                        .size(px(outer))
                        .rounded(px(match a.shape {
                            AvatarShape::Circle => outer / 2.0,
                            AvatarShape::Square => a.radius() + pad,
                        }))
                        .border_2()
                        .border_color(col)
                        .when(matches!(ring, AgentRing::Dashed), |d| d.border_dashed()),
                )
            })
            .child(face)
            .when_some(a.status, |d, p| {
                d.child(
                    div()
                        .absolute()
                        .right(px(pad - 1.0))
                        .bottom(px(pad - 1.0))
                        .size(px(a.dot()))
                        .rounded_full()
                        .border_2()
                        .border_color(c(t, ColorRole::BgBase))
                        .bg(c(t, p.role())),
                )
            })
            .when_some(a.model.clone(), |d, md| {
                d.child(
                    div()
                        .absolute()
                        .top_0()
                        .right_0()
                        .px(px(3.))
                        .h(px(14.))
                        .rounded(px(4.))
                        .bg(c(t, ColorRole::FgDefault))
                        .text_color(c(t, ColorRole::BgBase))
                        .text_size(px(8.5))
                        .child(md),
                )
            })
    }
}

/// `kbd`: keycaps for a shortcut.
#[derive(IntoElement)]
pub struct KbdEl {
    keys: String,
    platform: Platform,
    theme: &'static Theme,
}

impl KbdEl {
    /// Keycaps for `"Mod+Shift+K"`.
    pub fn new(keys: &str, platform: Platform) -> Self {
        Self {
            keys: keys.into(),
            platform,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for KbdEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let chords = parse_shortcut(&self.keys, self.platform);
        div()
            .flex()
            .items_center()
            .gap(px(3.))
            .children(chords.into_iter().enumerate().map(move |(n, ch)| {
                div()
                    .flex()
                    .gap(px(3.))
                    .when(n > 0, |d| {
                        d.child(
                            div()
                                .text_size(px(11.))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child("then"),
                        )
                    })
                    .children(ch.into_iter().map(move |k| {
                        div()
                            .min_w(px(20.))
                            .h(px(20.))
                            .px(px(5.))
                            .flex()
                            .items_center()
                            .justify_center()
                            .rounded(px(5.))
                            .border_1()
                            .border_b_2()
                            .border_color(c(t, ColorRole::BorderDefault))
                            .bg(c(t, ColorRole::SurfaceRaised))
                            .text_size(px(11.5))
                            .text_color(c(t, ColorRole::FgDefault))
                            .child(k.glyph)
                    }))
            }))
    }
}

/// `separator`.
#[derive(IntoElement)]
pub struct SeparatorEl {
    sep: Separator,
    theme: &'static Theme,
}

impl SeparatorEl {
    /// A separator.
    pub fn new(sep: Separator) -> Self {
        Self {
            sep,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for SeparatorEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let s = &self.sep;
        let line = c(t, ColorRole::BorderSubtle);
        let role = if s.decorative {
            Role::GenericContainer
        } else {
            Role::Splitter
        };
        if s.vertical {
            return div()
                .id("oxyn-a11y")
                .role(role)
                .w(px(1.))
                .h_full()
                .min_h(px(16.))
                .bg(line)
                .into_any_element();
        }
        match &s.label {
            Some(l) => div()
                .id("oxyn-a11y")
                .role(role)
                .flex()
                .items_center()
                .gap(px(10.))
                .w_full()
                .text_size(px(11.5))
                .text_color(c(t, ColorRole::FgSubtle))
                .child(div().flex_1().h(px(1.)).bg(line))
                .child(l.clone())
                .child(div().flex_1().h(px(1.)).bg(line))
                .into_any_element(),
            None => div()
                .id("oxyn-a11y")
                .role(role)
                .h(px(1.))
                .ml(px(s.inset))
                .w_full()
                .bg(line)
                .when(s.dashed, |d| d.border_dashed())
                .into_any_element(),
        }
    }
}

/// `spinner`: drawn as segments whose opacity runs round with the clock (all four styles share
/// it in GPUI; true arcs need path drawing, gap).
#[derive(IntoElement)]
pub struct SpinnerEl {
    spinner: Spinner,
    now: ClockMs,
    color: Option<Hsla>,
    theme: &'static Theme,
}

impl SpinnerEl {
    /// A spinner.
    pub fn new(spinner: Spinner) -> Self {
        Self {
            spinner,
            now: ClockMs(0),
            color: None,
            theme: &Theme::DARK,
        }
    }
    /// Time.
    pub fn now(mut self, now: ClockMs) -> Self {
        self.now = now;
        self
    }
    /// Colour (defaults to the text colour role).
    pub fn color(mut self, c: Hsla) -> Self {
        self.color = Some(c);
        self
    }
}

impl RenderOnce for SpinnerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let s = self.spinner.size;
        let col = self.color.unwrap_or(c(self.theme, ColorRole::FgDefault));
        let n = 8;
        let seg = self.spinner.segments(n, self.now, false);
        let r = s / 2.0 - self.spinner.stroke();
        let dot = (self.spinner.stroke() * 1.2).max(2.0);
        let label = self.spinner.label.clone();
        div()
            .id("oxyn-spinner")
            .relative()
            .size(px(s))
            .flex_none()
            .when_some(label, |d, l| d.role(Role::Status).aria_label(l))
            .children((0..n).map(move |i| {
                let a = i as f32 / n as f32 * std::f32::consts::TAU;
                let x = s / 2.0 + r * a.sin() - dot / 2.0;
                let y = s / 2.0 - r * a.cos() - dot / 2.0;
                div()
                    .absolute()
                    .left(px(x))
                    .top(px(y))
                    .size(px(dot))
                    .rounded_full()
                    .bg(alpha(col, seg[i]))
            }))
    }
}

/// `empty-state`.
#[derive(IntoElement)]
pub struct EmptyStateEl {
    state: EmptyState,
    theme: &'static Theme,
}

impl EmptyStateEl {
    /// An empty state.
    pub fn new(state: EmptyState) -> Self {
        Self {
            state,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for EmptyStateEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let s = self.state;
        if s.layout == EmptyLayout::Inline {
            return div()
                .flex()
                .items_center()
                .gap(px(8.))
                .p(px(10.))
                .text_size(px(12.5))
                .text_color(c(t, ColorRole::FgMuted))
                .child(div().text_color(c(t, ColorRole::FgDefault)).child(s.title))
                .child(s.text)
                .into_any_element();
        }
        div()
            .flex()
            .flex_col()
            .items_center()
            .gap(px(6.))
            .p(px(if s.tall { 64. } else { 28. }))
            .when(s.layout == EmptyLayout::Framed, |d| {
                d.border_1()
                    .border_dashed()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .rounded(px(12.))
            })
            .child(
                div()
                    .size(px(40.))
                    .rounded(px(10.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .bg(c(t, ColorRole::SurfaceHover))
                    .text_color(c(t, ColorRole::FgMuted))
                    .child(glyph(&s.icon)),
            )
            .child(
                div()
                    .text_size(px(14.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(c(t, ColorRole::FgDefault))
                    .child(s.title),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .text_color(c(t, ColorRole::FgMuted))
                    .child(s.text),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .mt(px(10.))
                    .when_some(s.action, |d, a| {
                        d.child(ButtonEl::new("es-a", a).size(ButtonSize::Sm).icon("plus"))
                    })
                    .when_some(s.secondary, |d, b| {
                        d.child(
                            ButtonEl::new("es-b", b)
                                .size(ButtonSize::Sm)
                                .variant(ButtonVariant::Ghost),
                        )
                    }),
            )
            .when(!s.prompts.is_empty(), |d| {
                d.child(
                    div().flex().flex_wrap().gap(px(6.)).mt(px(8.)).children(
                        s.prompts.into_iter().enumerate().map(|(i, p)| {
                            ChipEl::new(("es-p", i), Chip::new(&p, ChipKind::Toggle))
                        }),
                    ),
                )
            })
            .into_any_element()
    }
}

/// `reaction-bar`.
#[derive(IntoElement)]
pub struct ReactionBarEl {
    reactions: Vec<Reaction>,
    rule: CountRule,
    now: ClockMs,
    theme: &'static Theme,
}

impl ReactionBarEl {
    /// A bar at `now` (the pop scale reads it).
    pub fn new(reactions: Vec<Reaction>, now: ClockMs) -> Self {
        Self {
            reactions,
            rule: CountRule::default(),
            now,
            theme: &Theme::DARK,
        }
    }
    /// Count rule.
    pub fn rule(mut self, r: CountRule) -> Self {
        self.rule = r;
        self
    }
}

impl RenderOnce for ReactionBarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let rule = self.rule;
        let now = self.now;
        div()
            .flex()
            .items_center()
            .gap(px(2.))
            .p(px(3.))
            .rounded_full()
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_size(px(12.5))
            .children(self.reactions.into_iter().map(move |r| {
                let n = rule.show(r.count);
                let fg = if r.active {
                    c(t, r.tint)
                } else {
                    c(t, ColorRole::FgMuted)
                };
                div()
                    .id("oxyn-a11y")
                    .role(Role::Button)
                    .aria_label(SharedString::from(r.aria(&rule)))
                    .flex()
                    .items_center()
                    .gap(px(5.))
                    .h(px(28.))
                    .px(px(10.))
                    .rounded_full()
                    .text_color(fg)
                    .child(
                        div()
                            .text_size(px(16. * r.pop(now, false)))
                            .child(glyph(&r.icon)),
                    )
                    .when(!n.is_empty(), |d| d.child(n))
            }))
    }
}

/// `number-ticker`: odometer columns from the core model.
#[derive(IntoElement)]
pub struct NumberTickerEl {
    ticker: NumberTicker,
    now: ClockMs,
    size: f32,
    theme: &'static Theme,
}

impl NumberTickerEl {
    /// A ticker at `now`.
    pub fn new(ticker: NumberTicker, now: ClockMs, size: f32) -> Self {
        Self {
            ticker,
            now,
            size,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for NumberTickerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let fs = self.size;
        let cw = fs * 0.62;
        div()
            .flex()
            .id("oxyn-a11y")
            .role(Role::Label)
            .aria_label(SharedString::from(self.ticker.text()))
            .text_size(px(fs))
            .line_height(px(fs))
            .font_weight(FontWeight::SEMIBOLD)
            .text_color(c(t, ColorRole::FgDefault))
            .children(
                self.ticker
                    .columns(self.now, false)
                    .into_iter()
                    .map(move |col| match col {
                        Column::Digit { pos, .. } => {
                            let base = pos.floor() as u32 % 10;
                            div()
                                .relative()
                                .w(px(cw))
                                .h(px(fs))
                                .overflow_hidden()
                                .child(
                                    div()
                                        .absolute()
                                        .top(px(-pos.fract() * fs))
                                        .flex()
                                        .flex_col()
                                        .child(div().h(px(fs)).child(base.to_string()))
                                        .child(
                                            div().h(px(fs)).child(((base + 1) % 10).to_string()),
                                        ),
                                )
                                .into_any_element()
                        }
                        Column::Enter { ch, k } | Column::Exit { ch, k } => div()
                            .w(px(cw * k))
                            .h(px(fs))
                            .overflow_hidden()
                            .opacity(k)
                            .child(ch.to_string())
                            .into_any_element(),
                        Column::Static(ch) => div().child(ch.to_string()).into_any_element(),
                    }),
            )
    }
}

// ---------------------------------------------------------------------------------------------
// Form controls

/// `switch`: thumb geometry (stretch and travel) from the core model.
#[derive(IntoElement)]
pub struct SwitchEl {
    id: ElementId,
    label: SharedString,
    model: Switch,
    now: ClockMs,
    on_click: Option<Click>,
    theme: &'static Theme,
}

impl SwitchEl {
    /// A switch.
    pub fn new(
        id: impl Into<ElementId>,
        label: impl Into<SharedString>,
        model: Switch,
        now: ClockMs,
    ) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            model,
            now,
            on_click: None,
            theme: &Theme::DARK,
        }
    }
    /// Click (the host calls `Switch::toggle`).
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for SwitchEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let track = (36.0, 20.0);
        let (x, w) = self.model.thumb(track, 2.0, self.now, false);
        let on = self.model.checked;
        let f = self.on_click.clone();
        div()
            .id(self.id)
            .role(Role::Switch)
            .aria_label(self.label.clone())
            .flex()
            .items_center()
            .gap(px(10.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .when(self.model.disabled, |d| d.opacity(0.45))
            .child(
                div()
                    .relative()
                    .w(px(track.0))
                    .h(px(track.1))
                    .rounded_full()
                    .bg(if on {
                        c(t, ColorRole::FgDefault)
                    } else {
                        c(t, ColorRole::BorderStrong)
                    })
                    .child(
                        div()
                            .absolute()
                            .top(px(2.))
                            .left(px(x))
                            .w(px(w))
                            .h(px(16.))
                            .rounded_full()
                            .bg(c(t, ColorRole::BgBase))
                            .when(self.model.pending, |d| {
                                d.flex().items_center().justify_center().child(
                                    SpinnerEl::new(Spinner {
                                        size: 10.0,
                                        ..Default::default()
                                    })
                                    .now(self.now),
                                )
                            }),
                    ),
            )
            .child(self.label)
            .when_some(f.filter(|_| !self.model.disabled), |d, f| {
                d.cursor_pointer().on_click(move |e, w, cx| f(e, w, cx))
            })
    }
}

/// `checkbox`: box + label; check, indeterminate dash, error.
#[derive(IntoElement)]
pub struct CheckboxEl {
    id: ElementId,
    label: SharedString,
    value: Check,
    error: Option<SharedString>,
    card: bool,
    on_click: Option<Click>,
    theme: &'static Theme,
}

impl CheckboxEl {
    /// A checkbox.
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>, value: Check) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value,
            error: None,
            card: false,
            on_click: None,
            theme: &Theme::DARK,
        }
    }
    /// Error line.
    pub fn error(mut self, e: impl Into<SharedString>) -> Self {
        self.error = Some(e.into());
        self
    }
    /// Card row.
    pub fn card(mut self) -> Self {
        self.card = true;
        self
    }
    /// Click (the host calls `Check::next`).
    pub fn on_click(mut self, f: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static) -> Self {
        self.on_click = Some(Rc::new(f));
        self
    }
}

impl RenderOnce for CheckboxEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let filled = self.value != Check::Unchecked;
        let f = self.on_click.clone();
        div()
            .id(self.id)
            .role(Role::CheckBox)
            .aria_label(self.label.clone())
            .flex()
            .items_start()
            .gap(px(10.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .when(self.card, |d| {
                d.p(px(12.))
                    .rounded(px(10.))
                    .border_1()
                    .border_color(if filled {
                        c(t, ColorRole::FgDefault)
                    } else {
                        c(t, ColorRole::BorderDefault)
                    })
            })
            .child(
                div()
                    .size(px(16.))
                    .flex()
                    .flex_none()
                    .items_center()
                    .justify_center()
                    .rounded(px(4.))
                    .border_1()
                    .border_color(if self.error.is_some() {
                        c(t, ColorRole::StatusDangerFg)
                    } else if filled {
                        c(t, ColorRole::FgDefault)
                    } else {
                        c(t, ColorRole::BorderStrong)
                    })
                    .when(filled, |d| d.bg(c(t, ColorRole::FgDefault)))
                    .text_color(c(t, ColorRole::BgBase))
                    .text_size(px(11.))
                    .child(match self.value {
                        Check::Checked => "✓",
                        Check::Indeterminate => "–",
                        Check::Unchecked => "",
                    }),
            )
            .child(
                div()
                    .flex()
                    .flex_col()
                    .child(self.label)
                    .when_some(self.error, |d, e| {
                        d.child(
                            div()
                                .text_size(px(12.))
                                .text_color(c(t, ColorRole::StatusDangerFg))
                                .child(e),
                        )
                    }),
            )
            .when_some(f, |d, f| {
                d.cursor_pointer().on_click(move |e, w, cx| f(e, w, cx))
            })
    }
}

/// `radio-group`: rows with a dot; the picked row carries the highlight.
#[derive(IntoElement)]
pub struct RadioGroupEl {
    model: RadioGroup,
    cards: bool,
    theme: &'static Theme,
}

impl RadioGroupEl {
    /// From the model.
    pub fn new(model: RadioGroup) -> Self {
        Self {
            model,
            cards: false,
            theme: &Theme::DARK,
        }
    }
    /// Card layout.
    pub fn cards(mut self) -> Self {
        self.cards = true;
        self
    }
}

impl RenderOnce for RadioGroupEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let sel = self.model.selected();
        let cards = self.cards;
        div()
            .id("oxyn-a11y")
            .role(Role::RadioGroup)
            .flex()
            .flex_col()
            .gap(px(if cards { 8. } else { 2. }))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .children(
                self.model
                    .options
                    .into_iter()
                    .enumerate()
                    .map(move |(i, o)| {
                        let on = sel == Some(i);
                        div()
                            .id("oxyn-a11y")
                            .role(Role::RadioButton)
                            .aria_label(SharedString::from(o.label.clone()))
                            .flex()
                            .items_start()
                            .gap(px(10.))
                            .p(px(if cards { 12. } else { 8. }))
                            .rounded(px(8.))
                            .when(on && !cards, |d| d.bg(c(t, ColorRole::SurfaceHover)))
                            .when(cards, |d| {
                                d.border_1().border_color(if on {
                                    c(t, ColorRole::FgDefault)
                                } else {
                                    c(t, ColorRole::BorderDefault)
                                })
                            })
                            .when(o.disabled.is_some(), |d| d.opacity(0.45))
                            .child(
                                div()
                                    .size(px(16.))
                                    .flex()
                                    .flex_none()
                                    .items_center()
                                    .justify_center()
                                    .rounded_full()
                                    .border_1()
                                    .border_color(if on {
                                        c(t, ColorRole::FgDefault)
                                    } else {
                                        c(t, ColorRole::BorderStrong)
                                    })
                                    .when(on, |d| {
                                        d.child(
                                            div()
                                                .size(px(8.))
                                                .rounded_full()
                                                .bg(c(t, ColorRole::FgDefault)),
                                        )
                                    }),
                            )
                            .child(div().flex().flex_col().child(o.label).when_some(
                                o.description,
                                |d, x| {
                                    d.child(
                                        div()
                                            .text_size(px(12.))
                                            .text_color(c(t, ColorRole::FgMuted))
                                            .child(x),
                                    )
                                },
                            ))
                    }),
            )
    }
}

/// `segmented-control`: tray pill or underline; the indicator geometry (leading-edge stretch)
/// comes from the core model given each option's width.
#[derive(IntoElement)]
pub struct SegmentedEl {
    model: Segmented,
    widths: Vec<f32>,
    underline: bool,
    now: ClockMs,
    theme: &'static Theme,
}

impl SegmentedEl {
    /// From the model; `widths` are the measured option widths (px).
    pub fn new(model: Segmented, widths: Vec<f32>, now: ClockMs) -> Self {
        Self {
            model,
            widths,
            underline: false,
            now,
            theme: &Theme::DARK,
        }
    }
    /// Underline variant.
    pub fn underline(mut self) -> Self {
        self.underline = true;
        self
    }
}

impl RenderOnce for SegmentedEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let mut x = 0.0;
        let rects: Vec<(f32, f32)> = self
            .widths
            .iter()
            .map(|w| {
                let r = (x, *w);
                x += w;
                r
            })
            .collect();
        let ind = self.model.indicator(&rects, self.now, false);
        let sel = self.model.selected();
        let under = self.underline;
        div()
            .id("oxyn-a11y")
            .role(
                if self.model.kind == oxyn_core::primitives::segmented::SegmentedKind::Tabs {
                    Role::TabList
                } else {
                    Role::RadioGroup
                },
            )
            .relative()
            .flex()
            .p(px(if under { 0. } else { 2. }))
            .rounded(px(9.))
            .when(!under, |d| d.bg(c(t, ColorRole::SurfaceSunken)))
            .text_size(px(13.))
            .when_some(ind, |d, (s, w)| {
                d.child(
                    div()
                        .absolute()
                        .left(px(s + if under { 0. } else { 2. }))
                        .w(px(w))
                        .when(under, |d| {
                            d.bottom_0().h(px(2.)).bg(c(t, ColorRole::FgDefault))
                        })
                        .when(!under, |d| {
                            d.top(px(2.))
                                .h(px(28.))
                                .rounded(px(7.))
                                .bg(c(t, ColorRole::SurfaceRaised))
                                .border_1()
                                .border_color(c(t, ColorRole::BorderSubtle))
                        }),
                )
            })
            .children(self.model.options.iter().enumerate().map(|(i, o)| {
                let on = sel == Some(i);
                div()
                    .w(px(*self.widths.get(i).unwrap_or(&60.0)))
                    .h(px(28.))
                    .flex()
                    .items_center()
                    .justify_center()
                    .gap(px(6.))
                    .text_color(if on {
                        c(t, ColorRole::FgDefault)
                    } else {
                        c(t, ColorRole::FgMuted)
                    })
                    .when(on, |d| d.font_weight(FontWeight::SEMIBOLD))
                    .when(o.disabled, |d| d.opacity(0.4))
                    .child(o.label.clone())
                    .when_some(o.count, |d, n| {
                        d.child(
                            div()
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(n.to_string()),
                        )
                    })
            }))
    }
}

/// `select`: the trigger and, when open, the list (in flow below it; gap: no flip layer).
#[derive(IntoElement)]
pub struct SelectEl {
    model: Select,
    label: Option<SharedString>,
    theme: &'static Theme,
}

impl SelectEl {
    /// From the model.
    pub fn new(model: Select) -> Self {
        Self {
            model,
            label: None,
            theme: &Theme::DARK,
        }
    }
    /// Label.
    pub fn label(mut self, l: impl Into<SharedString>) -> Self {
        self.label = Some(l.into());
        self
    }
}

impl RenderOnce for SelectEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.model;
        let empty = m.value.is_none();
        let hl = m.highlighted();
        let rows = m.rows();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .min_w(px(220.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .when_some(self.label.clone(), |d, l| d.child(l))
            .child(
                div()
                    .id("oxyn-a11y")
                    .role(Role::ComboBox)
                    .flex()
                    .items_center()
                    .h(px(34.))
                    .px(px(10.))
                    .rounded(px(8.))
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .border_1()
                    .border_color(if m.open {
                        c(t, ColorRole::FgMuted)
                    } else {
                        c(t, ColorRole::BorderDefault)
                    })
                    .child(
                        div()
                            .flex_1()
                            .text_color(if empty {
                                c(t, ColorRole::FgSubtle)
                            } else {
                                c(t, ColorRole::FgDefault)
                            })
                            .child(m.display().to_string()),
                    )
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
                        .children(rows.into_iter().map(|r| {
                            match r {
                                ListRow::Group(g) => div()
                                    .px(px(8.))
                                    .pt(px(6.))
                                    .text_size(px(11.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child(g.to_uppercase())
                                    .into_any_element(),
                                ListRow::Skeleton => div()
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
                                ListRow::Empty => div()
                                    .p(px(12.))
                                    .text_color(c(t, ColorRole::FgSubtle))
                                    .child("No matches")
                                    .into_any_element(),
                                ListRow::Option(i) => {
                                    let o = &m.options[i];
                                    let sel = m.value.as_deref() == Some(o.value.as_str());
                                    div()
                                        .id("oxyn-a11y")
                                        .role(Role::ListBoxOption)
                                        .flex()
                                        .items_center()
                                        .gap(px(8.))
                                        .min_h(px(32.))
                                        .px(px(8.))
                                        .rounded(px(6.))
                                        .when(hl == Some(i), |d| {
                                            d.bg(c(t, ColorRole::SurfaceHover))
                                        })
                                        .when(o.disabled, |d| {
                                            d.text_color(c(t, ColorRole::FgDisabled))
                                        })
                                        .child(
                                            div()
                                                .flex_1()
                                                .flex()
                                                .flex_col()
                                                .child(o.label.clone())
                                                .when_some(o.description.clone(), |d, x| {
                                                    d.child(
                                                        div()
                                                            .text_size(px(11.5))
                                                            .text_color(c(t, ColorRole::FgSubtle))
                                                            .child(x),
                                                    )
                                                }),
                                        )
                                        .when(sel, |d| d.child("✓"))
                                        .into_any_element()
                                }
                            }
                        })),
                )
            })
    }
}

/// `slider`: rail, fill, marks and thumbs at the model's fractions over `width` px.
#[derive(IntoElement)]
pub struct SliderEl {
    model: Slider,
    width: f32,
    label: SharedString,
    theme: &'static Theme,
}

impl SliderEl {
    /// From the model.
    pub fn new(model: Slider, label: impl Into<SharedString>, width: f32) -> Self {
        Self {
            model,
            width,
            label: label.into(),
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for SliderEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.model;
        let w = self.width;
        let (a, b) = m.fill();
        let readout = (0..m.values.len())
            .map(|i| m.value_text(i, |v| format!("{v}")))
            .collect::<Vec<_>>()
            .join(" – ");
        div()
            .flex()
            .flex_col()
            .gap(px(10.))
            .w(px(w))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .when(m.disabled, |d| d.opacity(0.45))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .child(self.label.clone())
                    .child(div().text_color(c(t, ColorRole::FgMuted)).child(readout)),
            )
            .child(
                div()
                    .relative()
                    .h(px(20.))
                    .w(px(w))
                    .child(
                        div()
                            .absolute()
                            .top(px(8.))
                            .left_0()
                            .w(px(w))
                            .h(px(4.))
                            .rounded(px(2.))
                            .bg(c(t, ColorRole::BorderDefault)),
                    )
                    .child(
                        div()
                            .absolute()
                            .top(px(8.))
                            .left(px(a as f32 * w))
                            .w(px((b - a) as f32 * w))
                            .h(px(4.))
                            .rounded(px(2.))
                            .bg(c(t, ColorRole::FgDefault)),
                    )
                    .children(m.marks.iter().map(|mk| {
                        let s = if mk.detent { 6. } else { 2. };
                        div()
                            .absolute()
                            .top(px(7.))
                            .left(px(m.fraction(mk.value) as f32 * w - s / 2.0))
                            .w(px(s))
                            .h(px(6.))
                            .rounded(px(3.))
                            .bg(c(t, ColorRole::BorderStrong))
                    }))
                    .children(m.values.iter().map(|v| {
                        div()
                            .id("oxyn-a11y")
                            .role(Role::Slider)
                            .absolute()
                            .top(px(2.))
                            .left(px(m.fraction(*v) as f32 * w - 8.0))
                            .size(px(16.))
                            .rounded_full()
                            .border_2()
                            .border_color(c(t, ColorRole::FgDefault))
                            .bg(c(t, ColorRole::BgBase))
                    })),
            )
    }
}

/// `text-field`: label, control (prefix, value or placeholder, suffix), message slot and counter.
/// Display only: editing comes with the shared GPUI input handler (gap).
#[derive(IntoElement)]
pub struct TextFieldEl {
    model: TextField,
    label: SharedString,
    placeholder: Option<SharedString>,
    hint: Option<SharedString>,
    prefix: Option<SharedString>,
    suffix: Option<SharedString>,
    now: ClockMs,
    theme: &'static Theme,
}

impl TextFieldEl {
    /// From the model.
    pub fn new(model: TextField, label: impl Into<SharedString>, now: ClockMs) -> Self {
        Self {
            model,
            label: label.into(),
            placeholder: None,
            hint: None,
            prefix: None,
            suffix: None,
            now,
            theme: &Theme::DARK,
        }
    }
    /// Placeholder.
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = Some(p.into());
        self
    }
    /// Hint.
    pub fn hint(mut self, h: impl Into<SharedString>) -> Self {
        self.hint = Some(h.into());
        self
    }
    /// Prefix.
    pub fn prefix(mut self, p: impl Into<SharedString>) -> Self {
        self.prefix = Some(p.into());
        self
    }
    /// Suffix.
    pub fn suffix(mut self, s: impl Into<SharedString>) -> Self {
        self.suffix = Some(s.into());
        self
    }
}

impl RenderOnce for TextFieldEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.model;
        let err = m.error().map(str::to_string);
        let shown =
            if m.kind == oxyn_core::primitives::text_field::FieldType::Password && !m.revealed {
                "•".repeat(m.value.chars().count())
            } else {
                m.value.clone()
            };
        let empty = m.value.is_empty();
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .min_w(px(220.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(self.label.clone())
            .child(
                div()
                    .id("oxyn-a11y")
                    .role(Role::TextInput)
                    .aria_label(self.label)
                    .relative()
                    .left(px(m.nudge(self.now, false)))
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .h(px(34.))
                    .px(px(10.))
                    .rounded(px(8.))
                    .bg(c(t, ColorRole::SurfaceRaised))
                    .border_1()
                    .border_color(if err.is_some() {
                        c(t, ColorRole::StatusDangerFg)
                    } else if m.focused {
                        c(t, ColorRole::FgMuted)
                    } else {
                        c(t, ColorRole::BorderDefault)
                    })
                    .when_some(self.prefix, |d, p| {
                        d.child(div().text_color(c(t, ColorRole::FgSubtle)).child(p))
                    })
                    .child(
                        div()
                            .flex_1()
                            .text_color(if empty {
                                c(t, ColorRole::FgSubtle)
                            } else {
                                c(t, ColorRole::FgDefault)
                            })
                            .child(if empty {
                                self.placeholder.map(|p| p.to_string()).unwrap_or_default()
                            } else {
                                shown
                            }),
                    )
                    .when_some(self.suffix, |d, s| {
                        d.child(div().text_color(c(t, ColorRole::FgSubtle)).child(s))
                    })
                    .when(m.can_clear(), |d| {
                        d.child(div().text_color(c(t, ColorRole::FgSubtle)).child("×"))
                    })
                    .when(m.showing_copied(self.now), |d| {
                        d.child(
                            div()
                                .text_color(c(t, ColorRole::StatusSuccessFg))
                                .child("✓"),
                        )
                    }),
            )
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .text_size(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .text_color(if err.is_some() {
                                c(t, ColorRole::StatusDangerFg)
                            } else {
                                c(t, ColorRole::FgMuted)
                            })
                            .child(err.or(self.hint.map(|h| h.to_string())).unwrap_or_default()),
                    )
                    .when_some(m.counter(), |d, (txt, lvl)| {
                        d.child(
                            div()
                                .text_color(match lvl {
                                    CountLevel::Ok => c(t, ColorRole::FgSubtle),
                                    CountLevel::Near => c(t, ColorRole::StatusWarningFg),
                                    CountLevel::Over => c(t, ColorRole::StatusDangerFg),
                                })
                                .child(txt),
                        )
                    }),
            )
    }
}

// ---------------------------------------------------------------------------------------------
// Overlays

/// `tooltip` surface (place it with `placement::place`; GPUI's own `.tooltip()` can host it).
#[derive(IntoElement)]
pub struct TooltipEl {
    text: SharedString,
    shortcut: Option<String>,
    title: Option<SharedString>,
    theme: &'static Theme,
}

impl TooltipEl {
    /// A tooltip.
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            shortcut: None,
            title: None,
            theme: &Theme::DARK,
        }
    }
    /// Shortcut inside.
    pub fn shortcut(mut self, s: &str) -> Self {
        self.shortcut = Some(s.into());
        self
    }
    /// Rich title.
    pub fn title(mut self, s: impl Into<SharedString>) -> Self {
        self.title = Some(s.into());
        self
    }
}

impl RenderOnce for TooltipEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .id("oxyn-a11y")
            .role(Role::Tooltip)
            .flex()
            .when(self.title.is_some(), |d| d.flex_col())
            .items_start()
            .gap(px(6.))
            .px(px(8.))
            .py(px(5.))
            .rounded(px(6.))
            .bg(c(t, ColorRole::FgDefault))
            .text_color(c(t, ColorRole::BgBase))
            .text_size(px(12.))
            .when_some(self.title, |d, x| {
                d.child(div().font_weight(FontWeight::SEMIBOLD).child(x))
            })
            .child(self.text)
            .when_some(self.shortcut, |d, s| d.child(kbd(t, &s)))
    }
}

/// `popover` card (title, description, body).
#[derive(IntoElement)]
pub struct PopoverEl {
    title: Option<SharedString>,
    description: Option<SharedString>,
    body: Vec<AnyElement>,
    width: f32,
    theme: &'static Theme,
}

impl PopoverEl {
    /// A popover card.
    pub fn new(width: f32) -> Self {
        Self {
            title: None,
            description: None,
            body: vec![],
            width,
            theme: &Theme::DARK,
        }
    }
    /// Title.
    pub fn title(mut self, s: impl Into<SharedString>) -> Self {
        self.title = Some(s.into());
        self
    }
    /// Description.
    pub fn description(mut self, s: impl Into<SharedString>) -> Self {
        self.description = Some(s.into());
        self
    }
    /// Body element.
    pub fn child(mut self, e: impl IntoElement) -> Self {
        self.body.push(e.into_any_element());
        self
    }
}

impl RenderOnce for PopoverEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .id("oxyn-a11y")
            .role(Role::Dialog)
            .flex()
            .flex_col()
            .gap(px(10.))
            .w(px(self.width))
            .p(px(12.))
            .rounded(px(12.))
            .bg(c(t, ColorRole::SurfaceOverlay))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .shadow_lg()
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .when_some(self.title, |d, x| {
                d.child(div().font_weight(FontWeight::SEMIBOLD).child(x))
            })
            .when_some(self.description, |d, x| {
                d.child(
                    div()
                        .text_size(px(12.5))
                        .text_color(c(t, ColorRole::FgMuted))
                        .child(x),
                )
            })
            .children(self.body)
    }
}

/// `dropdown-menu`: the open menu (and open submenus side by side) from the core model.
#[derive(IntoElement)]
pub struct MenuEl {
    menu: Menu,
    theme: &'static Theme,
}

impl MenuEl {
    /// From the model.
    pub fn new(menu: Menu) -> Self {
        Self {
            menu,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for MenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = self.menu;
        if !m.open {
            return div().into_any_element();
        }
        div()
            .flex()
            .items_start()
            .gap(px(4.))
            .children((0..=m.depth()).map(|l| {
                let hl = m.levels.get(l).and_then(|g| g.target);
                div()
                    .id("oxyn-a11y")
                    .role(Role::Menu)
                    .flex()
                    .flex_col()
                    .p(px(4.))
                    .min_w(px(200.))
                    .rounded(px(10.))
                    .bg(c(t, ColorRole::SurfaceOverlay))
                    .border_1()
                    .border_color(c(t, ColorRole::BorderDefault))
                    .shadow_lg()
                    .text_size(px(13.))
                    .when(m.rows(l).is_empty(), |d| {
                        d.child(
                            div()
                                .p(px(12.))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(m.empty.clone()),
                        )
                    })
                    .children(m.rows(l).into_iter().enumerate().map(move |(i, it)| {
                        match it.kind {
                            ItemKind::Divider => div()
                                .h(px(1.))
                                .my(px(4.))
                                .mx(px(6.))
                                .bg(c(t, ColorRole::BorderSubtle))
                                .into_any_element(),
                            ItemKind::Label => div()
                                .px(px(8.))
                                .pt(px(6.))
                                .pb(px(4.))
                                .text_size(px(11.))
                                .text_color(c(t, ColorRole::FgSubtle))
                                .child(it.label.to_uppercase())
                                .into_any_element(),
                            _ => {
                                let on = hl == Some(i);
                                let sub = matches!(it.kind, ItemKind::Submenu(_));
                                div()
                                    .id("oxyn-a11y")
                                    .role(Role::MenuItem)
                                    .flex()
                                    .items_center()
                                    .gap(px(8.))
                                    .h(px(m.density.row()))
                                    .px(px(8.))
                                    .rounded(px(6.))
                                    .text_color(if it.danger {
                                        c(t, ColorRole::StatusDangerFg)
                                    } else if it.disabled {
                                        c(t, ColorRole::FgDisabled)
                                    } else {
                                        c(t, ColorRole::FgDefault)
                                    })
                                    .when(on, |d| {
                                        d.bg(if it.danger {
                                            c(t, ColorRole::StatusDangerBg)
                                        } else {
                                            c(t, ColorRole::SurfaceHover)
                                        })
                                    })
                                    .child(div().w(px(14.)).child(match it.checked() {
                                        Some(true) => "✓",
                                        _ => it.icon.as_deref().map(glyph).unwrap_or(""),
                                    }))
                                    .child(div().flex_1().child(it.label.clone()))
                                    .when_some(it.shortcut.clone(), |d, s| d.child(kbd(t, &s)))
                                    .when(sub, |d| d.child("›"))
                                    .into_any_element()
                            }
                        }
                    }))
            }))
            .into_any_element()
    }
}
