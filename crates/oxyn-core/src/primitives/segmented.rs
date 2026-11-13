//! Segmented control (`segmented-control.md`): 2–6 options (Oxyn plus: up to 8), a raised pill
//! (tray) or underline that glides with its leading edge first, drag-to-switch, radio or tabs
//! keyboard semantics, counts and badges, horizontal or vertical (Oxyn plus).

use std::time::Duration;

use super::nav::{NavKey, index_at, step};
use crate::{ClockMs, ease::Ease};

/// Look.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SegmentedVariant {
    /// Raised pill in a grey tray.
    #[default]
    Tray,
    /// A line under the picked option.
    Underline,
}

impl SegmentedVariant {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Tray => "tray",
            Self::Underline => "underline",
        }
    }
}

/// Keyboard semantics.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SegmentedKind {
    /// `radiogroup`: arrows move and select.
    #[default]
    Radio,
    /// `tablist`: arrows move focus; Enter / Space selects (manual activation).
    Tabs,
}

impl SegmentedKind {
    /// Container role.
    pub fn role(self) -> &'static str {
        match self {
            Self::Radio => "radiogroup",
            Self::Tabs => "tablist",
        }
    }
    /// Option role.
    pub fn item_role(self) -> &'static str {
        match self {
            Self::Radio => "radio",
            Self::Tabs => "tab",
        }
    }
    /// The selected-state attribute name.
    pub fn selected_attr(self) -> &'static str {
        match self {
            Self::Radio => "aria-checked",
            Self::Tabs => "aria-selected",
        }
    }
}

/// One option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Segment {
    /// Value.
    pub value: String,
    /// Label (may be empty for icon-only).
    pub label: String,
    /// Icon name.
    pub icon: Option<String>,
    /// Count after the label.
    pub count: Option<u64>,
    /// Badge text ("New").
    pub badge: Option<String>,
    /// Disabled.
    pub disabled: bool,
}

impl Segment {
    /// An option.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            icon: None,
            count: None,
            badge: None,
            disabled: false,
        }
    }
}

/// Timings.
pub mod timing {
    use std::time::Duration;
    /// Indicator glide (calm, no bounce).
    pub const GLIDE: Duration = Duration::from_millis(280);
    /// The leading edge arrives this much earlier than the trailing edge (the stretch).
    pub const LEAD: f32 = 0.35;
    /// Press squeezes the pill.
    pub const PRESS_SCALE: f32 = 0.97;
    /// Label weight crossfade.
    pub const WEIGHT: Duration = oxyn_tokens::motion::dur::BASE;
}

/// The control's state.
#[derive(Clone, Debug, PartialEq)]
pub struct Segmented {
    /// Options.
    pub options: Vec<Segment>,
    /// Selected value.
    pub value: String,
    /// Kind.
    pub kind: SegmentedKind,
    /// Focused option (tabs: may differ from the selection).
    pub focus: Option<usize>,
    /// Whole control disabled.
    pub disabled: bool,
    /// Oxyn plus: drag the pill across.
    pub draggable: bool,
    from: Option<usize>,
    at: ClockMs,
    drag: Option<f32>,
}

/// Oxyn plus: maximum options (affectUI says 2–6).
pub const MAX_OPTIONS: usize = 8;

impl Segmented {
    /// A control; `value` defaults to the first enabled option.
    pub fn new(options: Vec<Segment>, value: Option<&str>) -> Self {
        let value = value
            .map(str::to_string)
            .or_else(|| {
                options
                    .iter()
                    .find(|o| !o.disabled)
                    .map(|o| o.value.clone())
            })
            .unwrap_or_default();
        Self {
            options,
            value,
            kind: SegmentedKind::Radio,
            focus: None,
            disabled: false,
            draggable: true,
            from: None,
            at: ClockMs(0),
            drag: None,
        }
    }
    /// Selected index.
    pub fn selected(&self) -> Option<usize> {
        self.options.iter().position(|o| o.value == self.value)
    }
    fn enabled(&self, i: usize) -> bool {
        !self.disabled && self.options.get(i).is_some_and(|o| !o.disabled)
    }
    /// Select `i`; returns the new value when it changed.
    pub fn select(&mut self, i: usize, now: ClockMs) -> Option<String> {
        if !self.enabled(i) || self.selected() == Some(i) {
            return None;
        }
        self.from = self.selected();
        self.value = self.options[i].value.clone();
        self.at = now;
        self.focus = Some(i);
        Some(self.value.clone())
    }
    /// Arrow / Home / End. Radio selects as it moves; tabs only move focus.
    pub fn key(&mut self, k: NavKey, now: ClockMs) -> Option<String> {
        let from = self.focus.or(self.selected());
        let to = step(from, k, self.options.len(), true, |i| self.enabled(i))?;
        self.focus = Some(to);
        match self.kind {
            SegmentedKind::Radio => self.select(to, now),
            SegmentedKind::Tabs => None,
        }
    }
    /// Enter / Space (tabs: activates the focused tab).
    pub fn activate(&mut self, now: ClockMs) -> Option<String> {
        let f = self.focus?;
        self.select(f, now)
    }
    /// Drag started / moved at `x` along the track.
    pub fn drag(&mut self, x: f32) {
        if self.draggable && !self.disabled {
            self.drag = Some(x);
        }
    }
    /// Drag released: selects the option under the pointer.
    pub fn drop_at(&mut self, edges: &[f32], now: ClockMs) -> Option<String> {
        let x = self.drag.take()?;
        let i = index_at(edges, x)?;
        self.select(i, now)
    }
    /// Dragging now.
    pub fn dragging(&self) -> Option<f32> {
        self.drag
    }
    /// Indicator `(start, len)` over `rects` (offset, length) at `now`. The edge facing the target
    /// leads, so it stretches a little and settles. While dragging the pill follows the pointer.
    pub fn indicator(
        &self,
        rects: &[(f32, f32)],
        now: ClockMs,
        reduced: bool,
    ) -> Option<(f32, f32)> {
        let to = *rects.get(self.selected()?)?;
        if let Some(x) = self.drag {
            let lo = rects.first()?.0;
            let hi = rects.last().map(|r| r.0 + r.1)?;
            let s = (x - to.1 / 2.0).clamp(lo, hi - to.1);
            return Some((s, to.1));
        }
        let Some(from) = self.from.and_then(|f| rects.get(f).copied()) else {
            return Some(to);
        };
        if reduced {
            return Some(to);
        }
        let d = timing::GLIDE.as_secs_f32();
        let p = (now.since(self.at).as_secs_f32() / d).clamp(0.0, 1.0);
        let ease = oxyn_tokens::motion::ease::OUT;
        let lead = ease.ease((p / (1.0 - timing::LEAD)).min(1.0));
        let trail = ease.ease(((p - timing::LEAD) / (1.0 - timing::LEAD)).max(0.0));
        let right = to.0 >= from.0;
        let (a0, b0) = (from.0, from.0 + from.1);
        let (a1, b1) = (to.0, to.0 + to.1);
        let (a, b) = if right {
            (a0 + (a1 - a0) * trail, b0 + (b1 - b0) * lead)
        } else {
            (a0 + (a1 - a0) * lead, b0 + (b1 - b0) * trail)
        };
        Some((a, b - a))
    }
    /// Still gliding.
    pub fn moving(&self, now: ClockMs) -> bool {
        self.from.is_some() && now.since(self.at) < timing::GLIDE + Duration::from_millis(16)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn seg() -> Segmented {
        let mut o = vec![
            Segment::new("ask", "Ask"),
            Segment::new("plan", "Plan"),
            Segment::new("agent", "Agent"),
            Segment::new("debug", "Debug"),
        ];
        o[3].disabled = true;
        Segmented::new(o, None)
    }

    #[test]
    fn default_and_radio_keys() {
        let mut s = seg();
        assert_eq!(s.value, "ask");
        assert_eq!(s.key(NavKey::Next, ClockMs(0)).as_deref(), Some("plan"));
        assert_eq!(
            s.key(NavKey::Last, ClockMs(1)).as_deref(),
            Some("agent"),
            "skips disabled"
        );
        assert_eq!(
            s.key(NavKey::Next, ClockMs(2)).as_deref(),
            Some("ask"),
            "wraps"
        );
    }

    #[test]
    fn tabs_move_focus_then_enter() {
        let mut s = seg();
        s.kind = SegmentedKind::Tabs;
        assert_eq!(s.key(NavKey::Next, ClockMs(0)), None);
        assert_eq!(s.value, "ask");
        assert_eq!(s.activate(ClockMs(1)).as_deref(), Some("plan"));
        assert_eq!(s.kind.role(), "tablist");
    }

    #[test]
    fn indicator_stretches_and_settles() {
        let rects = [(0.0, 40.0), (40.0, 40.0), (80.0, 50.0)];
        let mut s = seg();
        assert_eq!(s.indicator(&rects, ClockMs(0), false), Some((0.0, 40.0)));
        s.select(2, ClockMs(0));
        let (_, w) = s.indicator(&rects, ClockMs(120), false).unwrap();
        assert!(w > 50.0, "stretched mid-glide: {w}");
        assert_eq!(
            s.indicator(&rects, ClockMs(1_000), false),
            Some((80.0, 50.0))
        );
        assert_eq!(s.indicator(&rects, ClockMs(10), true), Some((80.0, 50.0)));
    }

    #[test]
    fn drag_to_switch() {
        let edges = [0.0, 40.0, 80.0, 130.0];
        let mut s = seg();
        s.drag(55.0);
        let (x, _) = s
            .indicator(
                &[(0.0, 40.0), (40.0, 40.0), (80.0, 50.0)],
                ClockMs(0),
                false,
            )
            .unwrap();
        assert_eq!(x, 35.0);
        assert_eq!(s.drop_at(&edges, ClockMs(0)).as_deref(), Some("plan"));
        s.drag(120.0);
        assert_eq!(s.drop_at(&edges, ClockMs(0)).as_deref(), Some("agent"));
    }
}
