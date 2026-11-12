//! `StatusIndicator`: a dot and a short label for the agent's state
//! (`docs/specs/status-indicator.md`).

use gpui::{
    Animation, AnimationExt, App, ElementId, IntoElement, ParentElement, RenderOnce, Role,
    SharedString, Styled, Window, div, prelude::*, px,
};
use oxyn_core::{
    AgentState, MotionClock, VerbPack,
    ease::easing_fn,
    indicator::{DotMotion, IndicatorView, dot_opacity, indicator},
};
use oxyn_tokens::{ColorRole, Theme, motion};

use crate::{hsla, mix};

/// How much chrome the indicator has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IndicatorVariant {
    /// Dot, label, rounded surface and hairline border. The default.
    #[default]
    Pill,
    /// Dot and label, no surface (inline in rows and headers).
    Inline,
    /// Dot only; the label becomes the accessible name.
    Dot,
}

/// Dot and text size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IndicatorSize {
    /// 6 px dot, 12 px text (dense rows).
    Small,
    /// 8 px dot, 13 px text.
    #[default]
    Medium,
}

/// A status dot and label driven by an [`AgentState`].
#[derive(IntoElement)]
pub struct StatusIndicator {
    id: ElementId,
    view: IndicatorView,
    theme: &'static Theme,
    variant: IndicatorVariant,
    size: IndicatorSize,
    phase: f32,
    reduced: bool,
}

impl StatusIndicator {
    /// An indicator for `state`, timed by `clock`, with the `neutral` verb pack and dark theme.
    pub fn new(id: impl Into<ElementId>, state: &AgentState, clock: &MotionClock) -> Self {
        Self::with_pack(id, state, clock, VerbPack::neutral())
    }

    /// Same as [`new`](Self::new) with a specific verb pack.
    pub fn with_pack(
        id: impl Into<ElementId>,
        state: &AgentState,
        clock: &MotionClock,
        pack: &VerbPack,
    ) -> Self {
        let view = indicator(state, pack, clock.now());
        let phase = match view.motion {
            DotMotion::Pulse { period, .. } | DotMotion::Breathe { period } => clock.phase(period),
            _ => 0.0,
        };
        Self {
            id: id.into(),
            view,
            theme: &Theme::DARK,
            variant: IndicatorVariant::default(),
            size: IndicatorSize::default(),
            phase,
            reduced: clock.reduced(),
        }
    }

    /// From a precomputed view (Studio, tests, posters).
    pub fn from_view(id: impl Into<ElementId>, view: IndicatorView) -> Self {
        Self {
            id: id.into(),
            view,
            theme: &Theme::DARK,
            variant: IndicatorVariant::default(),
            size: IndicatorSize::default(),
            phase: 0.0,
            reduced: false,
        }
    }

    /// Theme (dark by default).
    pub fn theme(mut self, theme: &'static Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Variant.
    pub fn variant(mut self, variant: IndicatorVariant) -> Self {
        self.variant = variant;
        self
    }

    /// Size.
    pub fn size(mut self, size: IndicatorSize) -> Self {
        self.size = size;
        self
    }
}

impl RenderOnce for StatusIndicator {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let t = self.theme;
        let view = self.view;
        let (dot_px, text_px, gap) = match self.size {
            IndicatorSize::Small => (6.0, 12.0, 6.0),
            IndicatorSize::Medium => (8.0, 13.0, 8.0),
        };
        let to = t.color(view.color);
        let opacity = dot_opacity(view.motion, self.phase, self.reduced);
        let mut fill = hsla(to);
        fill.a *= opacity;

        let dot = div().flex_none().size(px(dot_px)).rounded_full();
        let dot = if view.hollow {
            dot.border_1().border_color(fill)
        } else {
            dot.bg(fill)
        };
        // Soft stall: ramp the dot from the stalled-from colour to the stall colour, once. The
        // animation id carries the state class so it restarts only when the state changes.
        let dot = match view.motion {
            DotMotion::Ramp { from, over } if !self.reduced => {
                let from = t.color(from);
                dot.with_animation(
                    SharedString::from(format!("oxyn-ramp-{}", view.announce_key)),
                    Animation::new(over).with_easing(easing_fn(motion::ease::LINEAR)),
                    move |el, p| el.bg(hsla(mix(from, to, p))),
                )
                .into_any_element()
            }
            _ => dot.into_any_element(),
        };

        let root = div()
            .id(self.id)
            .role(Role::Status)
            .aria_label(SharedString::from(view.label.clone()))
            .flex()
            .flex_none()
            .items_center()
            .gap(px(gap))
            .text_size(px(text_px))
            .line_height(px(text_px * 1.35))
            .text_color(hsla(t.color(ColorRole::FgDefault)))
            .child(dot);

        match self.variant {
            IndicatorVariant::Dot => root,
            IndicatorVariant::Inline => root.child(view.label),
            IndicatorVariant::Pill => root
                .px(px(if self.size == IndicatorSize::Small {
                    8.0
                } else {
                    12.0
                }))
                .py(px(if self.size == IndicatorSize::Small {
                    3.0
                } else {
                    6.0
                }))
                .rounded_full()
                .bg(hsla(t.color(ColorRole::SurfaceRaised)))
                .border_1()
                .border_color(hsla(t.color(ColorRole::BorderSubtle)))
                .child(view.label),
        }
    }
}
