//! Toaster and notification stack (`toaster.md`, `notification-stack.md`).
//!
//! - **Toaster** (affectUI Toaster): toasts from anywhere into one stack in a corner. At most
//!   `visible` show (3); older ones wait with a paused timer. Collapsed, the ones behind move back,
//!   shrink and peek out; hover, focus or `expand` fans the stack into a list. Each toast has a
//!   timer (4 s; `None` keeps it open) that pauses while the pointer or focus is in the stack and
//!   while the window is hidden; a hairline runs down with it. Promise toasts start as `Loading`
//!   and morph into success or error; updating a toast by id restarts its timer and crossfades the
//!   text. Swipe sideways or down, Escape, or the close button dismiss it. F8 focuses the newest.
//! - **Notification stack** (affectUI Notification Stack): notice cards stacked behind each
//!   other; dismissing the front one reveals the next. Only the front card is focusable.
//!
//! Oxyn plus: agent-run toasts that follow a run's state (`from_agent`), an undo for the last
//! dismissed notice, priority ordering and a stack depth setting. Rebuilt from the docs only.

use std::time::Duration;

use crate::{AgentState, ClockMs};

/// Timings.
pub mod timing {
    use std::time::Duration;
    /// Default time on screen.
    pub const DURATION: Duration = Duration::from_millis(4000);
    /// Lift in from the edge.
    pub const ENTER: Duration = Duration::from_millis(320);
    /// Fade and sink a few px.
    pub const EXIT: Duration = Duration::from_millis(200);
    /// Promise morph: arc closes into a ring, mark draws.
    pub const MORPH: Duration = Duration::from_millis(360);
    /// Collapsed peek per level px.
    pub const PEEK: f32 = 14.0;
    /// Collapsed scale step per level.
    pub const SCALE_STEP: f32 = 0.05;
    /// Swipe distance (fraction of the width) that dismisses.
    pub const SWIPE: f32 = 0.45;
    /// Swipe velocity px/s that dismisses.
    pub const SWIPE_VELOCITY: f32 = 600.0;
}

/// What a toast says it is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToastKind {
    /// Plain.
    #[default]
    Default,
    /// Green check.
    Success,
    /// Red cross; read in an alert region.
    Error,
    /// Info mark.
    Info,
    /// Oxyn plus: amber warning.
    Warning,
    /// Spinning arc (promise pending).
    Loading,
}

impl ToastKind {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Default => "default",
            Self::Success => "success",
            Self::Error => "error",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Loading => "loading",
        }
    }
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "success" => Self::Success,
            "error" => Self::Error,
            "info" => Self::Info,
            "warning" => Self::Warning,
            "loading" => Self::Loading,
            _ => Self::Default,
        }
    }
    /// Live region role: errors interrupt, the rest are polite.
    pub fn live(self) -> &'static str {
        if self == Self::Error {
            "alert"
        } else {
            "status"
        }
    }
}

/// Where the stack sits.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ToastPosition {
    /// Top left.
    TopLeft,
    /// Top centre.
    TopCenter,
    /// Top right.
    TopRight,
    /// Bottom left.
    BottomLeft,
    /// Bottom centre.
    BottomCenter,
    /// Bottom right.
    #[default]
    BottomRight,
}

impl ToastPosition {
    /// From an id (`bottom-right`).
    pub fn from_id(s: &str) -> Self {
        match s {
            "top-left" => Self::TopLeft,
            "top-center" => Self::TopCenter,
            "top-right" => Self::TopRight,
            "bottom-left" => Self::BottomLeft,
            "bottom-center" => Self::BottomCenter,
            _ => Self::BottomRight,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::TopLeft => "top-left",
            Self::TopCenter => "top-center",
            Self::TopRight => "top-right",
            Self::BottomLeft => "bottom-left",
            Self::BottomCenter => "bottom-center",
            Self::BottomRight => "bottom-right",
        }
    }
    /// New toasts come in from the top edge.
    pub fn top(self) -> bool {
        matches!(self, Self::TopLeft | Self::TopCenter | Self::TopRight)
    }
}

/// One toast.
#[derive(Clone, Debug, PartialEq)]
pub struct Toast {
    /// Id.
    pub id: u64,
    /// Title.
    pub title: String,
    /// Quiet line.
    pub description: Option<String>,
    /// Kind.
    pub kind: ToastKind,
    /// Action label (Undo, Retry, View).
    pub action: Option<String>,
    /// Time on screen (`None` = until dismissed).
    pub duration: Option<Duration>,
    /// Text version: bumps on update so the renderer crossfades.
    pub version: u32,
    /// Created.
    pub created: ClockMs,
    remaining: Option<Duration>,
    running_since: Option<ClockMs>,
    /// Set when it started leaving.
    pub closing_at: Option<ClockMs>,
    /// Horizontal swipe offset px.
    pub swipe: f32,
}

impl Toast {
    /// Time left on its timer at `now` (None = no timer).
    pub fn left(&self, now: ClockMs) -> Option<Duration> {
        let r = self.remaining?;
        Some(match self.running_since {
            Some(t) => r.saturating_sub(now.since(t)),
            None => r,
        })
    }
    /// Fraction of the timer left (for the hairline).
    pub fn fraction_left(&self, now: ClockMs) -> f32 {
        match (self.left(now), self.duration) {
            (Some(l), Some(d)) if !d.is_zero() => l.as_secs_f32() / d.as_secs_f32(),
            _ => 1.0,
        }
    }
    /// The timer is running.
    pub fn running(&self) -> bool {
        self.running_since.is_some()
    }
}

/// Options for a new toast.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ToastOptions {
    /// Quiet line.
    pub description: Option<String>,
    /// Kind.
    pub kind: ToastKind,
    /// Action label.
    pub action: Option<String>,
    /// Override the toaster duration (`Some(None)` = until dismissed).
    pub duration: Option<Option<Duration>>,
}

/// One slot as drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Slot {
    /// Offset from the edge px (toward the centre of the screen).
    pub y: f32,
    /// Scale.
    pub scale: f32,
    /// Opacity.
    pub opacity: f32,
    /// Shown at all (beyond `visible` it waits).
    pub shown: bool,
}

/// The toaster.
#[derive(Clone, Debug, PartialEq)]
pub struct Toaster {
    toasts: Vec<Toast>,
    /// Position.
    pub position: ToastPosition,
    /// Toasts visible at once (3).
    pub visible: usize,
    /// Default duration.
    pub duration: Option<Duration>,
    /// Always fanned out.
    pub expand: bool,
    /// Gap when fanned out px (10).
    pub gap: f32,
    /// Distance from the edges px (24; 20 inline).
    pub offset: f32,
    /// Hotkey that focuses the newest toast.
    pub hotkey: Option<String>,
    /// Region label.
    pub label: String,
    hovered: bool,
    hidden: bool,
    next: u64,
}

impl Default for Toaster {
    fn default() -> Self {
        Self {
            toasts: vec![],
            position: ToastPosition::BottomRight,
            visible: 3,
            duration: Some(timing::DURATION),
            expand: false,
            gap: 10.0,
            offset: 24.0,
            hotkey: Some("F8".into()),
            label: "Notifications".into(),
            hovered: false,
            hidden: false,
            next: 1,
        }
    }
}

impl Toaster {
    /// Empty toaster.
    pub fn new() -> Self {
        Self::default()
    }
    /// Region name for screen readers, with the hotkey ("Notifications (F8)").
    pub fn region_label(&self) -> String {
        match &self.hotkey {
            Some(k) => format!("{} ({k})", self.label),
            None => self.label.clone(),
        }
    }
    /// All toasts still alive (newest last), including those leaving.
    pub fn toasts(&self) -> &[Toast] {
        &self.toasts
    }
    /// Toasts not leaving, newest last.
    pub fn live(&self) -> Vec<&Toast> {
        self.toasts
            .iter()
            .filter(|t| t.closing_at.is_none())
            .collect()
    }
    fn paused(&self) -> bool {
        self.hovered || self.hidden
    }
    /// Shows a toast; returns its id.
    pub fn add(&mut self, title: &str, opts: ToastOptions, now: ClockMs) -> u64 {
        let id = self.next;
        self.next += 1;
        let duration = match (opts.duration, opts.kind) {
            (_, ToastKind::Loading) => None,
            (Some(d), _) => d,
            (None, _) => self.duration,
        };
        self.toasts.push(Toast {
            id,
            title: title.into(),
            description: opts.description,
            kind: opts.kind,
            action: opts.action,
            duration,
            version: 0,
            created: now,
            remaining: duration,
            running_since: None,
            closing_at: None,
            swipe: 0.0,
        });
        self.schedule(now);
        id
    }
    /// A loading toast for a promise; settle it with [`Toaster::resolve`].
    pub fn promise(&mut self, title: &str, description: Option<&str>, now: ClockMs) -> u64 {
        self.add(
            title,
            ToastOptions {
                description: description.map(Into::into),
                kind: ToastKind::Loading,
                ..Default::default()
            },
            now,
        )
    }
    /// Updates a toast in place (the timer restarts, the text crossfades).
    pub fn update(
        &mut self,
        id: u64,
        title: &str,
        description: Option<&str>,
        kind: ToastKind,
        now: ClockMs,
    ) -> bool {
        let d = self.duration;
        let paused = self.paused();
        let Some(t) = self
            .toasts
            .iter_mut()
            .find(|t| t.id == id && t.closing_at.is_none())
        else {
            return false;
        };
        t.title = title.into();
        t.description = description.map(Into::into);
        t.kind = kind;
        t.version += 1;
        t.duration = if kind == ToastKind::Loading { None } else { d };
        t.remaining = t.duration;
        t.running_since = (!paused && t.remaining.is_some()).then_some(now);
        self.schedule(now);
        true
    }
    /// Settles a promise toast.
    pub fn resolve(
        &mut self,
        id: u64,
        ok: bool,
        title: &str,
        description: Option<&str>,
        now: ClockMs,
    ) -> bool {
        let k = if ok {
            ToastKind::Success
        } else {
            ToastKind::Error
        };
        self.update(id, title, description, k, now)
    }
    /// Closes one toast (it fades and sinks; the rest move up).
    pub fn dismiss(&mut self, id: u64, now: ClockMs) -> bool {
        let hit = match self
            .toasts
            .iter_mut()
            .find(|t| t.id == id && t.closing_at.is_none())
        {
            Some(t) => {
                t.closing_at = Some(now);
                t.running_since = None;
                true
            }
            None => false,
        };
        self.schedule(now);
        hit
    }
    /// Removes a toast whose exit finished (renderers that time exits themselves).
    pub fn remove(&mut self, id: u64) {
        self.toasts.retain(|t| t.id != id);
    }
    /// Closes all.
    pub fn dismiss_all(&mut self, now: ClockMs) {
        let ids: Vec<u64> = self.live().iter().map(|t| t.id).collect();
        for id in ids {
            self.dismiss(id, now);
        }
    }
    /// Pointer or focus entered the stack: timers pause, the stack fans out.
    pub fn set_hovered(&mut self, on: bool, now: ClockMs) {
        if self.hovered != on {
            self.freeze(now);
            self.hovered = on;
            self.schedule(now);
        }
    }
    /// The window was hidden or shown.
    pub fn set_hidden(&mut self, on: bool, now: ClockMs) {
        if self.hidden != on {
            self.freeze(now);
            self.hidden = on;
            self.schedule(now);
        }
    }
    /// Fanned out now.
    pub fn expanded(&self) -> bool {
        self.expand || self.hovered
    }
    fn freeze(&mut self, now: ClockMs) {
        for t in &mut self.toasts {
            if let (Some(s), Some(r)) = (t.running_since, t.remaining) {
                t.remaining = Some(r.saturating_sub(now.since(s)));
                t.running_since = None;
            }
        }
    }
    /// Only the newest `visible` run their timers; older ones wait paused.
    fn schedule(&mut self, now: ClockMs) {
        let paused = self.paused();
        let live: Vec<u64> = self.live().iter().map(|t| t.id).collect();
        let cut = live.len().saturating_sub(self.visible);
        for t in &mut self.toasts {
            if t.closing_at.is_some() {
                continue;
            }
            let shown = live
                .iter()
                .position(|x| *x == t.id)
                .is_some_and(|i| i >= cut);
            let should = shown && !paused && t.remaining.is_some();
            match (should, t.running_since) {
                (true, None) => t.running_since = Some(now),
                (false, Some(s)) => {
                    t.remaining = t.remaining.map(|r| r.saturating_sub(now.since(s)));
                    t.running_since = None;
                }
                _ => {}
            }
        }
    }
    /// Advances timers: expired toasts start leaving; finished exits are removed. Returns the ids
    /// that timed out this tick.
    pub fn tick(&mut self, now: ClockMs) -> Vec<u64> {
        let expired: Vec<u64> = self
            .toasts
            .iter()
            .filter(|t| t.closing_at.is_none() && t.left(now) == Some(Duration::ZERO))
            .map(|t| t.id)
            .collect();
        for id in &expired {
            self.dismiss(*id, now);
        }
        self.toasts
            .retain(|t| t.closing_at.is_none_or(|c| now.since(c) < timing::EXIT));
        expired
    }
    /// Swipe offset while dragging.
    pub fn swipe(&mut self, id: u64, dx: f32) {
        if let Some(t) = self.toasts.iter_mut().find(|t| t.id == id) {
            t.swipe = dx;
        }
    }
    /// Swipe released: dismisses past 45 % of `width` or a fast flick; otherwise snaps back.
    pub fn swipe_end(&mut self, id: u64, velocity: f32, width: f32, now: ClockMs) -> bool {
        let Some(t) = self.toasts.iter_mut().find(|t| t.id == id) else {
            return false;
        };
        let out = t.swipe.abs() > width * timing::SWIPE || velocity.abs() > timing::SWIPE_VELOCITY;
        if !out {
            t.swipe = 0.0;
            return false;
        }
        self.dismiss(id, now)
    }
    /// Layout of the live toasts (newest last) given their measured `heights`.
    pub fn layout(&self, heights: &[f32]) -> Vec<Slot> {
        let n = heights.len();
        let expanded = self.expanded();
        let mut out = vec![
            Slot {
                y: 0.0,
                scale: 1.0,
                opacity: 1.0,
                shown: true,
            };
            n
        ];
        let mut acc = 0.0;
        for k in 0..n {
            // k = 0 is the newest (front).
            let i = n - 1 - k;
            let shown = k < self.visible;
            out[i] = if expanded {
                let y = acc;
                acc += heights[i] + self.gap;
                Slot {
                    y,
                    scale: 1.0,
                    opacity: if shown { 1.0 } else { 0.0 },
                    shown,
                }
            } else {
                Slot {
                    y: timing::PEEK * k as f32,
                    scale: 1.0 - timing::SCALE_STEP * k as f32,
                    opacity: if shown { 1.0 - 0.12 * k as f32 } else { 0.0 },
                    shown,
                }
            };
        }
        out
    }
    /// Height of the stack's hover area.
    pub fn extent(&self, heights: &[f32]) -> f32 {
        let shown: Vec<f32> = heights.iter().rev().take(self.visible).copied().collect();
        if self.expanded() {
            shown.iter().sum::<f32>() + self.gap * shown.len().saturating_sub(1) as f32
        } else {
            shown.first().copied().unwrap_or(0.0)
                + timing::PEEK * shown.len().saturating_sub(1) as f32
        }
    }
    /// Escape on toast `id`: closes it and returns the toast focus moves to (the next newest),
    /// or `None` when it was the last (focus goes back where it was).
    pub fn escape(&mut self, id: Option<u64>, now: ClockMs) -> Option<u64> {
        let live: Vec<u64> = self.live().iter().map(|t| t.id).collect();
        let id = id.or(live.last().copied())?;
        let pos = live.iter().position(|x| *x == id)?;
        self.dismiss(id, now);
        if pos > 0 {
            Some(live[pos - 1])
        } else {
            live.get(1).copied()
        }
    }
    /// Newest toast (the hotkey's target).
    pub fn newest(&self) -> Option<u64> {
        self.live().last().map(|t| t.id)
    }
    /// Oxyn plus: a toast that follows an agent run.
    pub fn from_agent(
        &mut self,
        id: Option<u64>,
        run: &str,
        state: &AgentState,
        now: ClockMs,
    ) -> u64 {
        let (title, kind) = match state {
            AgentState::Done { .. } | AgentState::Idle => {
                (format!("{run} finished"), ToastKind::Success)
            }
            AgentState::Error { .. } => (format!("{run} failed"), ToastKind::Error),
            AgentState::Cancelled { .. } => (format!("{run} stopped"), ToastKind::Default),
            AgentState::AwaitingApproval { .. } => {
                (format!("{run} needs your approval"), ToastKind::Warning)
            }
            _ => (format!("{run} running…"), ToastKind::Loading),
        };
        match id {
            Some(i) if self.update(i, &title, None, kind, now) => i,
            _ => self.add(
                &title,
                ToastOptions {
                    kind,
                    ..Default::default()
                },
                now,
            ),
        }
    }
}

/// One notice.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Notice {
    /// Id.
    pub id: String,
    /// Badge ("Notice", "New").
    pub badge: Option<String>,
    /// Title.
    pub title: String,
    /// Text.
    pub text: String,
    /// Action label.
    pub action: Option<String>,
    /// Action opens elsewhere.
    pub external: bool,
    /// Background pattern (`grid`, `dots`, `lines`).
    pub pattern: Option<String>,
    /// Oxyn plus: higher comes first when ordering by priority.
    pub priority: u8,
}

impl Notice {
    /// A notice.
    pub fn new(id: &str, title: &str, text: &str) -> Self {
        Self {
            id: id.into(),
            badge: None,
            title: title.into(),
            text: text.into(),
            action: None,
            external: false,
            pattern: None,
            priority: 0,
        }
    }
    /// With a badge.
    pub fn badge(mut self, b: &str) -> Self {
        self.badge = Some(b.into());
        self
    }
    /// With an action.
    pub fn action(mut self, a: &str, external: bool) -> Self {
        self.action = Some(a.into());
        self.external = external;
        self
    }
    /// With a pattern.
    pub fn pattern(mut self, p: &str) -> Self {
        self.pattern = Some(p.into());
        self
    }
    /// With a priority.
    pub fn priority(mut self, p: u8) -> Self {
        self.priority = p;
        self
    }
}

/// Notification stack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NotificationStack {
    notices: Vec<Notice>,
    dismissed: Vec<Notice>,
    /// Cards drawn behind the front one + 1 (Oxyn plus; 3).
    pub depth: usize,
    /// Oxyn plus: order by priority instead of arrival.
    pub by_priority: bool,
}

impl NotificationStack {
    /// From notices (first = front).
    pub fn new(notices: Vec<Notice>) -> Self {
        Self {
            notices,
            dismissed: vec![],
            depth: 3,
            by_priority: false,
        }
    }
    /// Remaining notices in display order.
    pub fn ordered(&self) -> Vec<&Notice> {
        let mut v: Vec<&Notice> = self.notices.iter().collect();
        if self.by_priority {
            v.sort_by_key(|a| std::cmp::Reverse(a.priority));
        }
        v
    }
    /// The front notice.
    pub fn front(&self) -> Option<&Notice> {
        self.ordered().first().copied()
    }
    /// Cards drawn (front first), at most `depth`.
    pub fn shown(&self) -> Vec<&Notice> {
        self.ordered().into_iter().take(self.depth.max(1)).collect()
    }
    /// Dismisses the front notice.
    pub fn dismiss_front(&mut self) -> Option<Notice> {
        let id = self.front()?.id.clone();
        let i = self.notices.iter().position(|n| n.id == id)?;
        let n = self.notices.remove(i);
        self.dismissed.push(n.clone());
        Some(n)
    }
    /// Oxyn plus: brings back the last dismissed notice to the front.
    pub fn undo(&mut self) -> Option<&Notice> {
        let n = self.dismissed.pop()?;
        self.notices.insert(0, n);
        self.notices.first()
    }
    /// Can undo.
    pub fn can_undo(&self) -> bool {
        !self.dismissed.is_empty()
    }
    /// Count left.
    pub fn len(&self) -> usize {
        self.notices.len()
    }
    /// Nothing left.
    pub fn is_empty(&self) -> bool {
        self.notices.is_empty()
    }
    /// "1 of 3".
    pub fn position_label(&self) -> String {
        format!("1 of {}", self.notices.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn t(ms: u64) -> ClockMs {
        ClockMs(ms)
    }

    #[test]
    fn timers_pause_and_queue() {
        let mut ts = Toaster::new();
        ts.visible = 2;
        let a = ts.add("Saved", ToastOptions::default(), t(0));
        let b = ts.add("Copied", ToastOptions::default(), t(100));
        let c = ts.add("Sent", ToastOptions::default(), t(200));
        // Only the newest two run; `a` waits.
        assert!(!ts.toasts()[0].running() && ts.toasts()[2].running());
        assert_eq!(
            ts.toasts()[0].left(t(3000)),
            Some(Duration::from_millis(3800)),
            "ran 200 ms before it was queued"
        );
        ts.set_hovered(true, t(1200));
        assert!(ts.expanded());
        assert!(ts.tick(t(9000)).is_empty(), "paused while hovered");
        ts.set_hovered(false, t(9000));
        // c had 3000 left at 1200 → expires at 12000.
        assert_eq!(ts.tick(t(12_100)), vec![b, c]);
        assert!(ts.toasts()[0].running(), "a moves up and starts its timer");
        let _ = a;
        assert!(ts.region_label().contains("(F8)"));
    }

    #[test]
    fn promise_morphs_and_updates_restart() {
        let mut ts = Toaster::new();
        let p = ts.promise("Uploading report…", Some("q3.pdf"), t(0));
        assert_eq!(ts.toasts()[0].left(t(60_000)), None, "loading stays");
        assert!(ts.resolve(
            p,
            true,
            "Report uploaded",
            Some("Shared with 4 people"),
            t(5000)
        ));
        let tt = &ts.toasts()[0];
        assert_eq!((tt.kind, tt.version), (ToastKind::Success, 1));
        assert_eq!(tt.left(t(6000)), Some(Duration::from_millis(3000)));
        assert_eq!(ToastKind::Error.live(), "alert");
    }

    #[test]
    fn layout_collapsed_and_fanned() {
        let mut ts = Toaster::new();
        for i in 0..4 {
            ts.add(&format!("t{i}"), ToastOptions::default(), t(0));
        }
        let h = [60.0, 60.0, 72.0, 60.0];
        let l = ts.layout(&h);
        assert_eq!(l[3].y, 0.0);
        assert!((l[2].scale - 0.95).abs() < 1e-6 && l[2].y == 14.0);
        assert!(!l[0].shown && l[0].opacity == 0.0);
        ts.expand = true;
        let l = ts.layout(&h);
        assert_eq!(l[2].y, 70.0);
        assert_eq!(l[1].y, 70.0 + 82.0);
        assert_eq!(ts.extent(&h), 60.0 + 72.0 + 60.0 + 20.0);
    }

    #[test]
    fn swipe_escape_and_exit() {
        let mut ts = Toaster::new();
        let a = ts.add("a", ToastOptions::default(), t(0));
        let b = ts.add("b", ToastOptions::default(), t(0));
        ts.swipe(b, 100.0);
        assert!(!ts.swipe_end(b, 0.0, 356.0, t(10)));
        ts.swipe(b, 200.0);
        assert!(ts.swipe_end(b, 0.0, 356.0, t(10)));
        assert_eq!(ts.live().len(), 1);
        assert_eq!(ts.escape(None, t(20)), None);
        assert!(ts.live().is_empty());
        ts.tick(t(500));
        assert!(ts.toasts().is_empty(), "exits finish and are removed");
        let _ = a;
    }

    #[test]
    fn agent_toast_follows_run() {
        let mut ts = Toaster::new();
        let id = ts.from_agent(None, "Refactor", &AgentState::Streaming, t(0));
        assert_eq!(ts.toasts()[0].kind, ToastKind::Loading);
        let id2 = ts.from_agent(
            Some(id),
            "Refactor",
            &AgentState::Done {
                elapsed: Duration::from_secs(3),
                waited: Duration::ZERO,
            },
            t(10),
        );
        assert_eq!(id, id2);
        assert_eq!(ts.toasts()[0].title, "Refactor finished");
    }

    #[test]
    fn notification_stack() {
        let mut s = NotificationStack::new(vec![
            Notice::new("a", "Maintenance", "Sunday 02:00").priority(1),
            Notice::new("b", "Invite", "Join Acme").priority(5),
            Notice::new("c", "Release", "v2.4 is out"),
        ]);
        assert_eq!(s.front().unwrap().id, "a");
        s.by_priority = true;
        assert_eq!(s.front().unwrap().id, "b");
        assert_eq!(s.dismiss_front().unwrap().id, "b");
        assert_eq!(s.position_label(), "1 of 2");
        assert_eq!(s.undo().unwrap().id, "b");
        s.depth = 2;
        assert_eq!(s.shown().len(), 2);
    }
}
