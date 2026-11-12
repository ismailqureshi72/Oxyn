//! `StatusIndicator`: a dot and a short label for the agent's state
//! (`docs/specs/status-indicator.md`). Same model as the GPUI component (`oxyn_core::indicator`).

use dioxus::prelude::*;
use oxyn_core::{
    AgentState, ClockMs, VerbPack,
    indicator::{DotMotion, PULSE_MIN_OPACITY, indicator},
};

use crate::color_var;

/// How much chrome the indicator has.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IndicatorVariant {
    /// Dot, label, rounded surface and hairline border. The default.
    #[default]
    Pill,
    /// Dot and label, no surface.
    Inline,
    /// Dot only; the label becomes the accessible name.
    Dot,
}

/// Dot and text size.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum IndicatorSize {
    /// 6 px dot, 12 px text.
    Small,
    /// 8 px dot, 13 px text.
    #[default]
    Medium,
}

/// Stylesheet for the indicator. Loops are phase-locked to the shared clock with a negative
/// `animation-delay`; reduced motion removes them.
pub const STATUS_INDICATOR_CSS: &str = r#"
.oxyn-si{display:inline-flex;align-items:center;gap:8px;color:var(--oxyn-color-fg-default);
  font-family:var(--oxyn-typography-status-font-family);font-size:13px;line-height:1.35;
  font-weight:500;white-space:nowrap}
.oxyn-si[data-size=sm]{gap:6px;font-size:12px}
.oxyn-si[data-variant=pill]{padding:6px 12px;border-radius:999px;
  background:var(--oxyn-color-surface-raised);border:1px solid var(--oxyn-color-border-subtle)}
.oxyn-si[data-variant=pill][data-size=sm]{padding:3px 8px}
.oxyn-si__dot{flex:none;width:8px;height:8px;border-radius:50%;box-sizing:border-box;
  transition:background-color var(--oxyn-dur-stall-recover) ease-out}
.oxyn-si[data-size=sm] .oxyn-si__dot{width:6px;height:6px}
.oxyn-si__dot[data-motion=pulse]{animation:oxyn-si-pulse var(--oxyn-si-period) ease-in-out infinite}
.oxyn-si__dot[data-motion=breathe]{animation:oxyn-si-breathe var(--oxyn-si-period) ease-in-out infinite}
.oxyn-si__dot[data-motion=ramp]{transition:background-color var(--oxyn-dur-stall-ramp) linear}
@keyframes oxyn-si-pulse{0%,100%{opacity:1}50%{opacity:var(--oxyn-si-min)}}
@keyframes oxyn-si-breathe{0%,100%{opacity:1}50%{opacity:.7}}
@media (prefers-reduced-motion:reduce){.oxyn-si__dot{animation:none!important;transition:none!important}}
.oxyn-sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}
"#;

/// A status dot and label driven by an [`AgentState`].
#[component]
pub fn StatusIndicator(
    /// The agent state.
    state: AgentState,
    /// Shared clock time (for loop phase and stall durations).
    #[props(default)]
    now: ClockMs,
    /// Verb pack id (`neutral`, `coder`, `terse`, `crm`).
    #[props(default = "neutral".to_string())]
    pack: String,
    /// Variant.
    #[props(default)]
    variant: IndicatorVariant,
    /// Size.
    #[props(default)]
    size: IndicatorSize,
) -> Element {
    let pack = VerbPack::builtin(&pack).unwrap_or_else(VerbPack::neutral);
    let view = indicator(&state, pack, now);
    let colour = color_var(view.color);
    let (motion, period) = match view.motion {
        DotMotion::Still => ("still", None),
        DotMotion::Pulse { period, .. } => ("pulse", Some(period)),
        DotMotion::Breathe { period } => ("breathe", Some(period)),
        DotMotion::Ramp { .. } => ("ramp", None),
    };
    let mut dot_style = if view.hollow {
        format!("border:1.5px solid {colour};")
    } else {
        format!("background-color:{colour};")
    };
    if let Some(p) = period {
        let ms = p.as_millis() as u64;
        dot_style.push_str(&format!(
            "--oxyn-si-period:{ms}ms;--oxyn-si-min:{};animation-delay:-{}ms;",
            f32::from(PULSE_MIN_OPACITY) / 100.0,
            now.0 % ms.max(1)
        ));
    }
    let variant_attr = match variant {
        IndicatorVariant::Pill => "pill",
        IndicatorVariant::Inline => "inline",
        IndicatorVariant::Dot => "dot",
    };
    let size_attr = if size == IndicatorSize::Small {
        "sm"
    } else {
        "md"
    };
    let label = view.label.clone();
    // The live region is a hidden copy of the label without ticking numbers, so screen readers
    // hear each state once, not every second of a hard stall.
    let announce = if view.announce_key == "stalled-hard" {
        "No response".to_string()
    } else {
        label.clone()
    };
    rsx! {
        span {
            class: "oxyn-si",
            "data-variant": variant_attr,
            "data-size": size_attr,
            "data-state": view.announce_key,
            span { class: "oxyn-si__dot", "aria-hidden": "true", "data-motion": motion, style: "{dot_style}" }
            if variant != IndicatorVariant::Dot {
                span { class: "oxyn-si__label", "aria-hidden": "true", "{label}" }
            }
            span { class: "oxyn-sr", role: "status", "aria-live": "polite", "{announce}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    fn html(state: AgentState, variant: IndicatorVariant) -> String {
        dioxus_ssr::render_element(rsx! { StatusIndicator { state, variant, now: ClockMs(1_500) } })
    }

    #[test]
    fn renders_label_role_and_token_colour() {
        let h = html(
            AgentState::Thinking { header: None },
            IndicatorVariant::Pill,
        );
        assert!(h.contains(r#"role="status""#), "{h}");
        assert!(h.contains("Thinking"));
        assert!(h.contains("var(--oxyn-color-state-thinking)"));
        assert!(h.contains(r#"data-motion="pulse""#));
        assert!(h.contains("animation-delay:-1500ms"));
    }

    #[test]
    fn dot_variant_keeps_an_accessible_label() {
        let h = html(
            AgentState::Done {
                elapsed: Duration::from_secs(12),
                waited: Duration::ZERO,
            },
            IndicatorVariant::Dot,
        );
        assert!(
            h.contains(r#"class="oxyn-sr""#) && h.contains("Worked for 12s"),
            "{h}"
        );
    }

    #[test]
    fn idle_is_hollow_and_still() {
        let h = html(AgentState::Idle, IndicatorVariant::Inline);
        assert!(
            h.contains("border:1.5px solid var(--oxyn-color-state-idle)"),
            "{h}"
        );
        assert!(h.contains(r#"data-motion="still""#));
    }
}
