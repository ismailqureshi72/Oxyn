//! Control center (`control-center.md`).
//!
//! Arc UI Control center: workspace quick settings. Tiles toggle (Focus, Notifications,
//! Presenting) and their chevrons open a detail as a modal over the grid; the focus detail has
//! focus modes (a radiogroup) and a session dial that counts minutes as the knob travels, keeps
//! counting across twelve o'clock, resists past 5 and 120 minutes and snaps to 5-minute steps on
//! release. Meters (alert volume 0–100 %, text size 85–130 %) follow the pointer 1:1 from where
//! they were grabbed with rubber-banded edges; a tap without a drag jumps there. Every change is
//! reported as a full state snapshot.
//!
//! Oxyn plus: agent tiles (effort, permission mode, voice), a "Do not disturb while agents run"
//! rule, and user-arrangeable tile order. Rebuilt from the docs; no source used.

use super::modal::rubber;

/// A focus mode.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FocusMode {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Icon name.
    pub icon: String,
    /// One line.
    pub description: String,
    /// Seeds the dial.
    pub default_minutes: u32,
}

impl FocusMode {
    /// A mode.
    pub fn new(id: &str, label: &str, icon: &str, description: &str, minutes: u32) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.into(),
            description: description.into(),
            default_minutes: minutes,
        }
    }
}

/// A notification channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Channel {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// One line.
    pub description: Option<String>,
    /// On.
    pub on: bool,
}

impl Channel {
    /// A channel.
    pub fn new(id: &str, label: &str, on: bool) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            description: None,
            on,
        }
    }
}

/// A running focus session.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    /// Mode id.
    pub mode: String,
    /// Length.
    pub minutes: u32,
    /// Ends at, minutes after midnight.
    pub until: u32,
}

/// Which detail is open.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Detail {
    /// Focus modes and the session dial.
    Focus,
    /// Notification channels.
    Notifications,
    /// Oxyn plus: agent settings.
    Agent,
}

/// Which meter.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Meter {
    /// Alert volume, 0–100, step 1.
    Volume,
    /// Text size, 85–130, step 5.
    TextSize,
}

impl Meter {
    /// (min, max, step).
    pub fn range(self) -> (f32, f32, f32) {
        match self {
            Self::Volume => (0.0, 100.0, 1.0),
            Self::TextSize => (85.0, 130.0, 5.0),
        }
    }
}

/// Keys on a meter or the dial.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKey {
    /// → / ↑.
    Inc,
    /// ← / ↓.
    Dec,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
    /// Home.
    Home,
    /// End.
    End,
}

/// Session dial limits.
pub mod dial {
    /// Fewest minutes.
    pub const MIN: f32 = 5.0;
    /// Most minutes.
    pub const MAX: f32 = 120.0;
    /// Snap.
    pub const SNAP: f32 = 5.0;
    /// One full turn is an hour.
    pub const TURN: f32 = 60.0;
}

/// Snapshot passed to `on_state_change`.
#[derive(Clone, Debug, PartialEq)]
pub struct CcState {
    /// Focus session.
    pub focus: Option<Session>,
    /// Any channel on.
    pub notifications: bool,
    /// Channels.
    pub channels: Vec<(String, bool)>,
    /// Presenting.
    pub presenting: bool,
    /// Volume.
    pub volume: f32,
    /// Text size %.
    pub text_size: f32,
}

/// Oxyn plus: agent quick settings.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AgentTiles {
    /// Effort (low / medium / high).
    pub effort: String,
    /// Permission mode (ask / auto-edit / full).
    pub permissions: String,
    /// Voice replies.
    pub voice: bool,
    /// Silence notifications while agents run.
    pub dnd_while_running: bool,
}

impl Default for AgentTiles {
    fn default() -> Self {
        Self {
            effort: "medium".into(),
            permissions: "ask".into(),
            voice: false,
            dnd_while_running: true,
        }
    }
}

/// Control center.
#[derive(Clone, Debug, PartialEq)]
pub struct ControlCenter {
    /// Focus modes.
    pub modes: Vec<FocusMode>,
    /// Channels.
    pub channels: Vec<Channel>,
    /// People who can still reach you (names).
    pub people: Vec<String>,
    /// Running session.
    pub focus: Option<Session>,
    /// Presenting.
    pub presenting: bool,
    /// Alert volume.
    pub volume: f32,
    /// Text size %.
    pub text_size: f32,
    /// Open detail.
    pub detail: Option<Detail>,
    /// Draft mode in the focus detail.
    pub draft_mode: usize,
    /// Draft minutes on the dial (unsnapped while turning).
    pub draft_minutes: f32,
    /// Now, minutes after midnight (for "Until 3:40 PM").
    pub now_min: u32,
    /// Oxyn plus: agent tiles.
    pub agent: AgentTiles,
    /// Oxyn plus: tile order (ids).
    pub order: Vec<String>,
    drag: Option<(Meter, f32, f32)>,
    dial_angle: Option<f32>,
}

impl ControlCenter {
    /// A panel.
    pub fn new(modes: Vec<FocusMode>, channels: Vec<Channel>, now_min: u32) -> Self {
        let draft = modes.first().map(|m| m.default_minutes).unwrap_or(25) as f32;
        Self {
            modes,
            channels,
            people: vec![],
            focus: None,
            presenting: false,
            volume: 60.0,
            text_size: 100.0,
            detail: None,
            draft_mode: 0,
            draft_minutes: draft,
            now_min,
            agent: AgentTiles::default(),
            order: ["focus", "notifications", "presenting", "agent"]
                .map(String::from)
                .to_vec(),
            drag: None,
            dial_angle: None,
        }
    }
    /// The snapshot.
    pub fn state(&self) -> CcState {
        CcState {
            focus: self.focus.clone(),
            notifications: self.notifications(),
            channels: self.channels.iter().map(|c| (c.id.clone(), c.on)).collect(),
            presenting: self.presenting,
            volume: self.volume,
            text_size: self.text_size,
        }
    }
    /// Any channel on.
    pub fn notifications(&self) -> bool {
        self.channels.iter().any(|c| c.on)
    }
    /// "2 of 3 channels" / "Off".
    pub fn notifications_label(&self) -> String {
        let on = self.channels.iter().filter(|c| c.on).count();
        if on == 0 {
            "Off".into()
        } else if on == self.channels.len() {
            "All channels".into()
        } else {
            format!("{on} of {} channels", self.channels.len())
        }
    }
    /// Focus tile line.
    pub fn focus_label(&self) -> String {
        match &self.focus {
            None => "Off".into(),
            Some(s) => {
                let m = self
                    .modes
                    .iter()
                    .find(|m| m.id == s.mode)
                    .map(|m| m.label.as_str())
                    .unwrap_or("Focus");
                format!("{m} · until {}", clock(s.until))
            }
        }
    }
    /// The Focus tile: with no session starts the first mode at its default length; with one,
    /// ends it.
    pub fn toggle_focus(&mut self) {
        if self.focus.is_some() {
            self.focus = None;
        } else if let Some(m) = self.modes.first() {
            self.focus = Some(Session {
                mode: m.id.clone(),
                minutes: m.default_minutes,
                until: (self.now_min + m.default_minutes) % 1440,
            });
        }
    }
    /// The Notifications tile: all off, or back to all on.
    pub fn toggle_notifications(&mut self) {
        let on = !self.notifications();
        for c in &mut self.channels {
            c.on = on;
        }
    }
    /// A channel switch.
    pub fn toggle_channel(&mut self, id: &str) {
        if let Some(c) = self.channels.iter_mut().find(|c| c.id == id) {
            c.on = !c.on;
        }
    }
    /// The Presenting tile.
    pub fn toggle_presenting(&mut self) {
        self.presenting = !self.presenting;
    }
    /// A chevron: opens a detail (seeding the focus draft).
    pub fn open(&mut self, d: Detail) {
        if d == Detail::Focus {
            match &self.focus {
                Some(s) => {
                    self.draft_mode = self.modes.iter().position(|m| m.id == s.mode).unwrap_or(0);
                    self.draft_minutes = s.minutes as f32;
                }
                None => {
                    self.draft_minutes = self
                        .modes
                        .get(self.draft_mode)
                        .map(|m| m.default_minutes)
                        .unwrap_or(25) as f32
                }
            }
        }
        self.detail = Some(d);
    }
    /// Escape / scrim.
    pub fn close(&mut self) {
        self.detail = None;
        self.dial_angle = None;
    }
    /// Picks a focus mode in the detail (seeds the dial with its default).
    pub fn pick_mode(&mut self, i: usize) {
        if let Some(m) = self.modes.get(i) {
            self.draft_mode = i;
            self.draft_minutes = m.default_minutes as f32;
        }
    }
    /// Start / Update session: applies the draft.
    pub fn apply_session(&mut self) {
        let Some(m) = self.modes.get(self.draft_mode) else {
            return;
        };
        let minutes = snap(self.draft_minutes) as u32;
        self.focus = Some(Session {
            mode: m.id.clone(),
            minutes,
            until: (self.now_min + minutes) % 1440,
        });
        self.detail = None;
    }
    /// The detail's primary button.
    pub fn session_button(&self) -> &'static str {
        if self.focus.is_some() {
            "Update session"
        } else {
            "Start session"
        }
    }
    /// Dial pointer at `angle` degrees (0 = twelve o'clock, clockwise). Keeps counting across
    /// twelve; past 5 and 120 minutes it resists.
    pub fn dial_to(&mut self, angle: f32) {
        let prev = self.dial_angle.unwrap_or(angle);
        let mut d = angle - prev;
        if d > 180.0 {
            d -= 360.0;
        } else if d < -180.0 {
            d += 360.0;
        }
        self.dial_angle = Some(angle);
        let raw = self.draft_minutes + d / 360.0 * dial::TURN;
        self.draft_minutes = if raw > dial::MAX {
            dial::MAX + rubber(raw - dial::MAX, 15.0) * 0.5
        } else if raw < dial::MIN {
            dial::MIN - rubber(dial::MIN - raw, 15.0) * 0.5
        } else {
            raw
        };
    }
    /// Dial released: snaps to 5 minutes inside the limits.
    pub fn dial_release(&mut self) -> u32 {
        self.dial_angle = None;
        self.draft_minutes = snap(self.draft_minutes);
        self.draft_minutes as u32
    }
    /// Dial keys: ±5, Page ±15, Home 5, End 120.
    pub fn dial_key(&mut self, k: StepKey) -> u32 {
        let m = snap(self.draft_minutes);
        self.draft_minutes = match k {
            StepKey::Inc => m + 5.0,
            StepKey::Dec => m - 5.0,
            StepKey::PageUp => m + 15.0,
            StepKey::PageDown => m - 15.0,
            StepKey::Home => dial::MIN,
            StepKey::End => dial::MAX,
        }
        .clamp(dial::MIN, dial::MAX);
        self.draft_minutes as u32
    }
    /// Knob angle for the draft (degrees, can exceed 360 past an hour).
    pub fn knob_angle(&self) -> f32 {
        self.draft_minutes / dial::TURN * 360.0
    }
    /// "Until 3:40 PM" for the draft.
    pub fn draft_until(&self) -> String {
        clock((self.now_min + snap(self.draft_minutes) as u32) % 1440)
    }
    fn meter_mut(&mut self, m: Meter) -> &mut f32 {
        match m {
            Meter::Volume => &mut self.volume,
            Meter::TextSize => &mut self.text_size,
        }
    }
    /// Meter value.
    pub fn meter(&self, m: Meter) -> f32 {
        match m {
            Meter::Volume => self.volume,
            Meter::TextSize => self.text_size,
        }
    }
    /// Pointer down on a meter at `x` px of `width`.
    pub fn grab(&mut self, m: Meter, x: f32) {
        let v = self.meter(m);
        self.drag = Some((m, x, v));
    }
    /// Pointer moved: follows 1:1 from where it was grabbed, rubber-banded at the ends. Returns
    /// the shown value (may sit past the range while held).
    pub fn drag_to(&mut self, x: f32, width: f32) -> f32 {
        let Some((m, x0, v0)) = self.drag else {
            return 0.0;
        };
        let (lo, hi, _) = m.range();
        let raw = v0 + (x - x0) / width.max(1.0) * (hi - lo);
        let span = (hi - lo) * 0.08;
        let shown = if raw > hi {
            hi + rubber(raw - hi, span)
        } else if raw < lo {
            lo - rubber(lo - raw, span)
        } else {
            raw
        };
        *self.meter_mut(m) = shown.clamp(lo, hi);
        shown
    }
    /// Pointer up: a tap without a drag jumps to the tapped spot (`x` of `width`).
    pub fn release(&mut self, x: f32, width: f32) {
        if let Some((m, x0, _)) = self.drag.take()
            && (x - x0).abs() < 3.0
        {
            let (lo, hi, step) = m.range();
            let v = lo + (x / width.max(1.0)).clamp(0.0, 1.0) * (hi - lo);
            *self.meter_mut(m) = (v / step).round() * step;
        }
        for m in [Meter::Volume, Meter::TextSize] {
            let (_, _, step) = m.range();
            let v = self.meter(m);
            *self.meter_mut(m) = (v / step).round() * step;
        }
    }
    /// A meter is being held (the track swells).
    pub fn held(&self) -> Option<Meter> {
        self.drag.map(|d| d.0)
    }
    /// Meter keys: one step, Page = four steps, Home / End = limits.
    pub fn meter_key(&mut self, m: Meter, k: StepKey) -> f32 {
        let (lo, hi, step) = m.range();
        let v = self.meter(m);
        let nv = match k {
            StepKey::Inc => v + step,
            StepKey::Dec => v - step,
            StepKey::PageUp => v + step * 4.0,
            StepKey::PageDown => v - step * 4.0,
            StepKey::Home => lo,
            StepKey::End => hi,
        }
        .clamp(lo, hi);
        *self.meter_mut(m) = nv;
        nv
    }
    /// Value text for a meter.
    pub fn meter_text(&self, m: Meter) -> String {
        format!("{}%", self.meter(m).round())
    }
    /// Oxyn plus: moves tile `from` to `to`.
    pub fn move_tile(&mut self, from: usize, to: usize) {
        if from < self.order.len() && to < self.order.len() {
            let t = self.order.remove(from);
            self.order.insert(to, t);
        }
    }
}

fn snap(m: f32) -> f32 {
    ((m / dial::SNAP).round() * dial::SNAP).clamp(dial::MIN, dial::MAX)
}

/// 12-hour clock for minutes after midnight ("3:40 PM").
pub fn clock(min: u32) -> String {
    let h = (min / 60) % 24;
    let m = min % 60;
    let (h12, ap) = match h {
        0 => (12, "AM"),
        1..=11 => (h, "AM"),
        12 => (12, "PM"),
        _ => (h - 12, "PM"),
    };
    format!("{h12}:{m:02} {ap}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cc() -> ControlCenter {
        ControlCenter::new(
            vec![
                FocusMode::new(
                    "deep",
                    "Deep work",
                    "brain",
                    "Silences everything but mentions",
                    50,
                ),
                FocusMode::new("rest", "Rest", "moon", "Silences all alerts", 30),
            ],
            vec![
                Channel::new("mentions", "Mentions", true),
                Channel::new("deploys", "Deploys", true),
                Channel::new("agents", "Agent runs", false),
            ],
            14 * 60 + 50,
        )
    }

    #[test]
    fn tiles_and_labels() {
        let mut c = cc();
        assert_eq!(c.notifications_label(), "2 of 3 channels");
        c.toggle_focus();
        assert_eq!(c.focus_label(), "Deep work · until 3:40 PM");
        c.toggle_focus();
        assert_eq!(c.focus_label(), "Off");
        c.toggle_notifications();
        assert_eq!(c.notifications_label(), "Off");
        c.toggle_notifications();
        assert_eq!(c.notifications_label(), "All channels");
        c.toggle_channel("deploys");
        assert_eq!(c.state().channels[1], ("deploys".to_string(), false));
        assert_eq!(clock(0), "12:00 AM");
    }

    #[test]
    fn dial_counts_across_twelve_and_resists() {
        let mut c = cc();
        c.open(Detail::Focus);
        assert_eq!(c.draft_minutes, 50.0);
        // From 300° to 30°: crosses twelve, +90° = +15 minutes.
        c.dial_to(300.0);
        c.dial_to(330.0);
        c.dial_to(0.0);
        c.dial_to(30.0);
        assert!((c.draft_minutes - 65.0).abs() < 1e-3);
        for a in (40..=1400).step_by(10) {
            c.dial_to((a % 360) as f32);
        }
        assert!(
            c.draft_minutes > 120.0 && c.draft_minutes < 128.0,
            "resists past 120"
        );
        assert_eq!(c.dial_release(), 120);
        assert_eq!(c.dial_key(StepKey::PageDown), 105);
        assert_eq!(c.dial_key(StepKey::Home), 5);
        c.pick_mode(1);
        assert_eq!(c.draft_minutes, 30.0);
        assert_eq!(c.session_button(), "Start session");
        c.apply_session();
        assert_eq!(c.focus.as_ref().unwrap().mode, "rest");
        assert!(c.detail.is_none());
    }

    #[test]
    fn meters_follow_and_rubber_band() {
        let mut c = cc();
        c.grab(Meter::Volume, 100.0);
        assert_eq!(c.held(), Some(Meter::Volume));
        let shown = c.drag_to(300.0, 200.0);
        assert!(shown > 100.0 && shown < 110.0);
        assert_eq!(c.volume, 100.0);
        c.release(300.0, 200.0);
        assert_eq!(c.volume, 100.0);
        c.grab(Meter::TextSize, 50.0);
        c.release(51.0, 200.0);
        assert_eq!(
            c.text_size, 95.0,
            "tap jumps (85 + 0.255 × 45 → 96.5 → step 5)"
        );
        assert_eq!(c.meter_key(Meter::TextSize, StepKey::PageUp), 115.0);
        assert_eq!(c.meter_key(Meter::Volume, StepKey::Home), 0.0);
        assert_eq!(c.meter_text(Meter::TextSize), "115%");
        c.move_tile(3, 0);
        assert_eq!(c.order[0], "agent");
    }
}
