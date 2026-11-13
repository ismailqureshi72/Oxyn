//! Tooltip and popover (`tooltip.md`, `popover.md`).
//!
//! Tooltips share one delay group per window: hover waits `delay` (500 ms), keyboard focus shows
//! at once, and once one is open a neighbour within 300 ms opens instantly without the fade. They
//! hide at once. Popovers keep a stack so nested popovers close inner-first, close on a press
//! outside or Escape, and hand the focus back to their trigger.

use std::time::Duration;

use crate::ClockMs;

/// Tooltip timings.
pub mod timing {
    use std::time::Duration;
    /// Hover delay (affectUI `delay` default).
    pub const DELAY: Duration = Duration::from_millis(500);
    /// Neighbours within this window open instantly (affectUI and Arc: 300 ms).
    pub const SKIP: Duration = Duration::from_millis(300);
    /// Fade-in.
    pub const FADE: Duration = oxyn_tokens::motion::dur::FAST;
    /// Drift from the trigger's side.
    pub const DRIFT: f32 = 3.0;
    /// Popover: settle distance toward the trigger (Arc 5 px).
    pub const POPOVER_DRIFT: f32 = 5.0;
    /// Popover open / close.
    pub const POPOVER_OPEN: Duration = oxyn_tokens::motion::dur::popover::OPEN;
    /// Popover leave (Arc 140 ms; token 120 ms).
    pub const POPOVER_CLOSE: Duration = oxyn_tokens::motion::dur::popover::CLOSE;
    /// Oxyn plus: hover-intent open delay for popovers.
    pub const INTENT_OPEN: Duration = oxyn_tokens::motion::delay::HOVER;
    /// Oxyn plus: hover-intent close grace.
    pub const INTENT_CLOSE: Duration = oxyn_tokens::motion::delay::LEAVE;
}

/// How a tooltip appears this time.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Entrance {
    /// Fade + 3 px drift.
    Fade,
    /// Instantly (a neighbour, or keyboard focus with reduced motion).
    Instant,
}

/// One delay group (per window).
#[derive(Clone, Debug, PartialEq)]
pub struct TooltipGroup {
    /// Hover delay (Oxyn plus: configurable).
    pub delay: Duration,
    /// Skip window.
    pub skip: Duration,
    open: Option<(String, Entrance)>,
    pending: Option<(String, ClockMs)>,
    closed_at: Option<ClockMs>,
    disabled: Vec<String>,
}

impl Default for TooltipGroup {
    fn default() -> Self {
        Self {
            delay: timing::DELAY,
            skip: timing::SKIP,
            open: None,
            pending: None,
            closed_at: None,
            disabled: vec![],
        }
    }
}

impl TooltipGroup {
    /// Marks a tooltip disabled (it never opens).
    pub fn set_disabled(&mut self, id: &str, disabled: bool) {
        self.disabled.retain(|d| d != id);
        if disabled {
            self.disabled.push(id.into());
            if self.open_id() == Some(id) {
                self.open = None;
            }
        }
    }
    fn warm(&self, now: ClockMs) -> bool {
        self.open.is_some() || self.closed_at.is_some_and(|t| now.since(t) <= self.skip)
    }
    /// Pointer entered `id`'s trigger.
    pub fn enter(&mut self, id: &str, now: ClockMs) {
        if self.disabled.iter().any(|d| d == id) {
            return;
        }
        if self.warm(now) {
            self.open = Some((id.into(), Entrance::Instant));
            self.pending = None;
        } else {
            self.pending = Some((id.into(), now));
        }
    }
    /// Keyboard focus reached `id`'s trigger: shows at once.
    pub fn focus(&mut self, id: &str, now: ClockMs) {
        if self.disabled.iter().any(|d| d == id) {
            return;
        }
        let e = if self.warm(now) {
            Entrance::Instant
        } else {
            Entrance::Fade
        };
        self.open = Some((id.into(), e));
        self.pending = None;
    }
    /// Pointer left / focus left / the trigger was pressed: hide at once.
    pub fn leave(&mut self, id: &str, now: ClockMs) {
        if self.pending.as_ref().is_some_and(|(p, _)| p == id) {
            self.pending = None;
        }
        if self.open_id() == Some(id) {
            self.open = None;
            self.closed_at = Some(now);
        }
    }
    /// Escape hides the open tooltip (focus stays on the trigger).
    pub fn escape(&mut self, now: ClockMs) -> bool {
        if self.open.take().is_some() {
            self.closed_at = Some(now);
            true
        } else {
            false
        }
    }
    /// Advances the hover delay. Returns the open tooltip and how it entered.
    pub fn tick(&mut self, now: ClockMs) -> Option<(&str, Entrance)> {
        if let Some((id, t)) = &self.pending
            && now.since(*t) >= self.delay
        {
            self.open = Some((id.clone(), Entrance::Fade));
            self.pending = None;
        }
        self.open.as_ref().map(|(i, e)| (i.as_str(), *e))
    }
    /// Open tooltip id.
    pub fn open_id(&self) -> Option<&str> {
        self.open.as_ref().map(|(i, _)| i.as_str())
    }
}

/// Tooltip content (Oxyn plus: rich title + body).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TooltipContent {
    /// The label (the trigger's accessible description).
    pub text: String,
    /// Shortcut shown inside ("Mod+K").
    pub shortcut: Option<String>,
    /// Rich: a bold title line above `text`.
    pub title: Option<String>,
}

// ---------------------------------------------------------------------------------------------
// Popover

/// How a popover opens (Oxyn plus: hover intent).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum PopoverTrigger {
    /// Click / Enter / Space.
    #[default]
    Press,
    /// Opens after resting on the trigger, closes after leaving trigger and surface.
    HoverIntent,
}

/// What closed a popover.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseReason {
    /// Escape.
    Escape,
    /// A press outside.
    Outside,
    /// The trigger was pressed again.
    Trigger,
    /// Hover intent ran out.
    Leave,
    /// The app closed it.
    Programmatic,
}

impl CloseReason {
    /// Focus goes back to the trigger (not for outside presses, which move focus themselves).
    pub fn restores_focus(self) -> bool {
        !matches!(self, Self::Outside | Self::Leave)
    }
}

/// Open popovers, innermost last.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PopoverStack {
    stack: Vec<(String, ClockMs)>,
    hover_enter: Option<(String, ClockMs)>,
    hover_leave: Option<(String, ClockMs)>,
}

impl PopoverStack {
    /// Open ids, outermost first.
    pub fn open(&self) -> Vec<&str> {
        self.stack.iter().map(|(i, _)| i.as_str()).collect()
    }
    /// Whether `id` is open.
    pub fn is_open(&self, id: &str) -> bool {
        self.stack.iter().any(|(i, _)| i == id)
    }
    /// When `id` opened.
    pub fn opened_at(&self, id: &str) -> Option<ClockMs> {
        self.stack.iter().find(|(i, _)| i == id).map(|(_, t)| *t)
    }
    /// Opens `id` inside `parent` (`None` = top level: closes other top-level popovers).
    pub fn show(&mut self, id: &str, parent: Option<&str>, now: ClockMs) {
        let keep = parent
            .and_then(|p| self.stack.iter().position(|(i, _)| i == p))
            .map(|k| k + 1)
            .unwrap_or(0);
        self.stack.truncate(keep);
        self.stack.push((id.into(), now));
    }
    /// Closes `id` and everything nested in it. Returns what closed, innermost first.
    pub fn close(&mut self, id: &str) -> Vec<String> {
        let Some(k) = self.stack.iter().position(|(i, _)| i == id) else {
            return vec![];
        };
        self.stack.drain(k..).rev().map(|(i, _)| i).collect()
    }
    /// Trigger pressed: toggles.
    pub fn toggle(&mut self, id: &str, parent: Option<&str>, now: ClockMs) -> bool {
        if self.is_open(id) {
            self.close(id);
            false
        } else {
            self.show(id, parent, now);
            true
        }
    }
    /// Escape closes the innermost only.
    pub fn escape(&mut self) -> Option<String> {
        self.stack.pop().map(|(i, _)| i)
    }
    /// A press landed inside `inside` (`None` = outside every popover): closes everything nested
    /// deeper than it.
    pub fn press(&mut self, inside: Option<&str>) -> Vec<String> {
        let keep = inside
            .and_then(|p| self.stack.iter().position(|(i, _)| i == p))
            .map(|k| k + 1)
            .unwrap_or(0);
        self.stack.drain(keep..).rev().map(|(i, _)| i).collect()
    }
    /// Hover intent: pointer entered the trigger or the surface of `id`.
    pub fn hover_in(&mut self, id: &str, now: ClockMs) {
        if self.hover_leave.as_ref().is_some_and(|(i, _)| i == id) {
            self.hover_leave = None;
        }
        if !self.is_open(id) {
            self.hover_enter = Some((id.into(), now));
        }
    }
    /// Hover intent: pointer left both.
    pub fn hover_out(&mut self, id: &str, now: ClockMs) {
        if self.hover_enter.as_ref().is_some_and(|(i, _)| i == id) {
            self.hover_enter = None;
        }
        if self.is_open(id) {
            self.hover_leave = Some((id.into(), now));
        }
    }
    /// Advances hover intent.
    pub fn tick(&mut self, now: ClockMs) {
        if let Some((id, t)) = self.hover_enter.clone()
            && now.since(t) >= timing::INTENT_OPEN
        {
            self.show(&id, None, now);
            self.hover_enter = None;
        }
        if let Some((id, t)) = self.hover_leave.clone()
            && now.since(t) >= timing::INTENT_CLOSE
        {
            self.close(&id);
            self.hover_leave = None;
        }
    }
}

/// Enter progress (0..1) for the open animation.
pub fn enter_progress(opened: ClockMs, now: ClockMs, d: Duration, reduced: bool) -> f32 {
    if reduced {
        return 1.0;
    }
    crate::ease::Ease::ease(
        &oxyn_tokens::motion::ease::OUT,
        (now.since(opened).as_secs_f32() / d.as_secs_f32()).clamp(0.0, 1.0),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hover_delay_skip_and_focus() {
        let mut g = TooltipGroup::default();
        g.enter("copy", ClockMs(0));
        assert_eq!(g.tick(ClockMs(400)), None);
        assert_eq!(g.tick(ClockMs(500)), Some(("copy", Entrance::Fade)));
        g.leave("copy", ClockMs(600));
        assert_eq!(g.tick(ClockMs(600)), None, "hides at once");
        g.enter("paste", ClockMs(800));
        assert_eq!(
            g.tick(ClockMs(800)),
            Some(("paste", Entrance::Instant)),
            "neighbour within 300 ms"
        );
        g.leave("paste", ClockMs(900));
        g.enter("cut", ClockMs(1_300));
        assert_eq!(g.tick(ClockMs(1_300)), None, "window passed: waits again");
        g.leave("cut", ClockMs(1_350));
        g.focus("cut", ClockMs(2_000));
        assert_eq!(
            g.tick(ClockMs(2_000)),
            Some(("cut", Entrance::Fade)),
            "keyboard: no delay"
        );
        assert!(g.escape(ClockMs(2_100)));
        g.set_disabled("save", true);
        g.focus("save", ClockMs(3_000));
        assert_eq!(g.tick(ClockMs(3_000)), None);
    }

    #[test]
    fn popover_stack() {
        let mut s = PopoverStack::default();
        s.show("filters", None, ClockMs(0));
        s.show("date", Some("filters"), ClockMs(1));
        assert_eq!(s.open(), vec!["filters", "date"]);
        assert_eq!(s.escape().as_deref(), Some("date"));
        s.show("date", Some("filters"), ClockMs(2));
        assert_eq!(s.press(Some("filters")), vec!["date".to_string()]);
        assert_eq!(s.open(), vec!["filters"]);
        s.show("date", Some("filters"), ClockMs(3));
        assert_eq!(
            s.press(None),
            vec!["date".to_string(), "filters".to_string()]
        );
        assert!(!s.toggle("x", None, ClockMs(4)) || s.is_open("x"));
        s.show("other", None, ClockMs(5));
        assert_eq!(
            s.open(),
            vec!["other"],
            "top-level popovers replace each other"
        );
        assert!(CloseReason::Escape.restores_focus() && !CloseReason::Outside.restores_focus());
    }

    #[test]
    fn hover_intent() {
        let mut s = PopoverStack::default();
        s.hover_in("card", ClockMs(0));
        s.tick(ClockMs(100));
        assert!(!s.is_open("card"));
        s.tick(ClockMs(150));
        assert!(s.is_open("card"));
        s.hover_out("card", ClockMs(200));
        s.hover_in("card", ClockMs(300));
        s.tick(ClockMs(600));
        assert!(s.is_open("card"), "came back within the grace");
        s.hover_out("card", ClockMs(700));
        s.tick(ClockMs(1_000));
        assert!(!s.is_open("card"));
        assert_eq!(
            enter_progress(ClockMs(0), ClockMs(0), timing::POPOVER_OPEN, true),
            1.0
        );
    }
}
