//! Buttons (`button.md`, `button-groups.md`): the button and icon button, the action button with
//! pending / success / error and retry back-off, hold-to-confirm, the split button's remembered
//! choice, and the three groups (joined, floating tray, expanding) with their shared highlight.

use std::time::Duration;

use crate::{AgentState, ClockMs};

/// Visual weight. Chrome is monochrome; only `Destructive` and `Success` carry status colour.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ButtonVariant {
    /// Solid, highest emphasis (white on dark, black on light).
    #[default]
    Primary,
    /// Raised surface with a hairline border.
    Secondary,
    /// No surface until hover.
    Ghost,
    /// Danger (affectUI `destructive`, Arc `danger`).
    Destructive,
    /// Oxyn plus: border only, transparent fill.
    Outline,
    /// Oxyn plus: text button with an underline on hover.
    Link,
    /// Beautiful UI `quiet`: muted text, no border.
    Quiet,
    /// Beautiful UI `success`.
    Success,
}

impl ButtonVariant {
    /// Every variant.
    pub const ALL: [Self; 8] = [
        Self::Primary,
        Self::Secondary,
        Self::Ghost,
        Self::Destructive,
        Self::Outline,
        Self::Link,
        Self::Quiet,
        Self::Success,
    ];
    /// Class / id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Primary => "primary",
            Self::Secondary => "secondary",
            Self::Ghost => "ghost",
            Self::Destructive => "destructive",
            Self::Outline => "outline",
            Self::Link => "link",
            Self::Quiet => "quiet",
            Self::Success => "success",
        }
    }
    /// Parses an id; `danger` is accepted for Arc parity.
    pub fn from_id(s: &str) -> Option<Self> {
        Some(match s {
            "primary" => Self::Primary,
            "secondary" => Self::Secondary,
            "ghost" => Self::Ghost,
            "destructive" | "danger" => Self::Destructive,
            "outline" => Self::Outline,
            "link" => Self::Link,
            "quiet" => Self::Quiet,
            "success" => Self::Success,
            _ => return None,
        })
    }
}

/// Size. `Sm`, `Md`, `Lg` are the source sizes; `Xs` and `Xl` are Oxyn plus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ButtonSize {
    /// 24 px.
    Xs,
    /// 28 px.
    Sm,
    /// 32 px.
    #[default]
    Md,
    /// 40 px.
    Lg,
    /// 48 px.
    Xl,
}

impl ButtonSize {
    /// Every size.
    pub const ALL: [Self; 5] = [Self::Xs, Self::Sm, Self::Md, Self::Lg, Self::Xl];
    /// Height in px.
    pub fn height(self) -> f32 {
        match self {
            Self::Xs => 24.0,
            Self::Sm => 28.0,
            Self::Md => 32.0,
            Self::Lg => 40.0,
            Self::Xl => 48.0,
        }
    }
    /// Horizontal padding.
    pub fn pad_x(self) -> f32 {
        match self {
            Self::Xs => 8.0,
            Self::Sm => 10.0,
            Self::Md => 12.0,
            Self::Lg => 16.0,
            Self::Xl => 20.0,
        }
    }
    /// Label size.
    pub fn font(self) -> f32 {
        match self {
            Self::Xs => 12.0,
            Self::Sm | Self::Md => 13.0,
            Self::Lg => 14.0,
            Self::Xl => 15.0,
        }
    }
    /// Icon size.
    pub fn icon(self) -> f32 {
        match self {
            Self::Xs => 12.0,
            Self::Sm => 14.0,
            Self::Md => 16.0,
            Self::Lg => 18.0,
            Self::Xl => 20.0,
        }
    }
    /// Corner radius.
    pub fn radius(self) -> f32 {
        match self {
            Self::Xs => 6.0,
            Self::Sm => 7.0,
            Self::Md => 8.0,
            Self::Lg => 10.0,
            Self::Xl => 12.0,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Xs => "xs",
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Xl => "xl",
        }
    }
    /// Parses an id.
    pub fn from_id(s: &str) -> Option<Self> {
        Some(match s {
            "xs" => Self::Xs,
            "sm" => Self::Sm,
            "md" => Self::Md,
            "lg" => Self::Lg,
            "xl" => Self::Xl,
            _ => return None,
        })
    }
}

/// How `loading` looks (Oxyn plus: the source has spinner + shimmer only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LoadingStyle {
    /// affectUI: the left icon becomes a spinner and a band sweeps the label.
    #[default]
    SpinnerShimmer,
    /// Spinner only.
    Spinner,
    /// A fill grows behind the label with `progress`.
    Progress,
}

impl LoadingStyle {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::SpinnerShimmer => "shimmer",
            Self::Spinner => "spinner",
            Self::Progress => "progress",
        }
    }
}

/// Button timings (`button.md` §5).
pub mod timing {
    use std::time::Duration;
    /// Press scale (affectUI 97 %, Arc ≈ 0.97).
    pub const PRESS_SCALE: f32 = 0.97;
    /// Icon-only buttons press slightly deeper (Arc).
    pub const ICON_PRESS_SCALE: f32 = 0.94;
    /// Press-down.
    pub const PRESS: Duration = oxyn_tokens::motion::dur::INSTANT;
    /// Colour / shadow fade on hover.
    pub const HOVER: Duration = oxyn_tokens::motion::dur::FAST;
    /// Width glide when the label changes (`spring.snap`).
    pub const WIDTH: Duration = oxyn_tokens::motion::spring::SNAP;
    /// Label crossfade from a soft blur.
    pub const LABEL: Duration = oxyn_tokens::motion::dur::BASE;
    /// Shimmer band period while loading.
    pub const SHIMMER: Duration = oxyn_tokens::motion::dur::shimmer::SWEEP;
    /// Action button: success label holds this long by default (Arc `resetAfterMs`).
    pub const RESET_AFTER: Duration = Duration::from_millis(1600);
    /// Hold-to-confirm fill time.
    pub const HOLD: Duration = oxyn_tokens::motion::dur::approval::HOLD;
    /// Released hold drains in.
    pub const DRAIN: Duration = oxyn_tokens::motion::dur::approval::DRAIN;
    /// Expanding group: confirmation holds (Arc `confirmDuration`).
    pub const CONFIRM: Duration = Duration::from_millis(1400);
    /// Floating tray highlight grows in from this scale when it appears from nothing.
    pub const APPEAR_SCALE: f32 = 0.94;
}

/// Why a button is disabled, shown as its tooltip (Oxyn plus: disabled-with-reason).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisabledReason(pub String);

/// The state a button draws in.
#[derive(Clone, Debug, PartialEq)]
pub enum ButtonState {
    /// Resting.
    Idle,
    /// Pointer over (fine pointers only).
    Hover,
    /// Held down.
    Pressed,
    /// Busy; `Some(p)` when the loading style is a progress fill.
    Loading(Option<f32>),
    /// Not available, with the reason.
    Disabled(Option<DisabledReason>),
}

impl ButtonState {
    /// Id.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Hover => "hover",
            Self::Pressed => "pressed",
            Self::Loading(_) => "loading",
            Self::Disabled(_) => "disabled",
        }
    }
    /// Activations are ignored (loading keeps its width and focus, but does nothing).
    pub fn inert(&self) -> bool {
        matches!(self, Self::Loading(_) | Self::Disabled(_))
    }
    /// `aria-busy`.
    pub fn busy(&self) -> bool {
        matches!(self, Self::Loading(_))
    }
}

/// Oxyn plus (Arc row): bind a button to the agent. While the agent works the button shows its
/// loading state; while it waits for approval it is disabled with that reason.
pub fn agent_bound(agent: &AgentState) -> ButtonState {
    match agent {
        AgentState::Submitted
        | AgentState::Thinking { .. }
        | AgentState::Streaming
        | AgentState::Steering
        | AgentState::ToolRunning { .. } => ButtonState::Loading(None),
        AgentState::AwaitingApproval { .. } => {
            ButtonState::Disabled(Some(DisabledReason("Waiting for your approval".into())))
        }
        AgentState::Stalled { .. } => ButtonState::Loading(None),
        _ => ButtonState::Idle,
    }
}

/// Scale while pressed (reduced motion: none).
pub fn press_scale(icon_only: bool, pressed: bool, reduced: bool) -> f32 {
    match (pressed, reduced, icon_only) {
        (false, _, _) | (_, true, _) => 1.0,
        (true, false, true) => timing::ICON_PRESS_SCALE,
        (true, false, false) => timing::PRESS_SCALE,
    }
}

/// Action button phase (Arc `action-button`).
#[derive(Clone, Debug, PartialEq)]
pub enum ActionPhase {
    /// Ready.
    Idle,
    /// Running; shows `pending_label`.
    Pending,
    /// Done; shows `success_label` with a drawn check until the reset.
    Success,
    /// Failed. Oxyn plus: retry is offered after a back-off countdown (`Some(secs)` left).
    Error {
        /// Seconds until retry is allowed; `None` when it is allowed now.
        retry_in: Option<u64>,
    },
}

/// Action button with pending / success / error and retry back-off.
#[derive(Clone, Debug, PartialEq)]
pub struct ActionButton {
    /// Resting label ("Deploy").
    pub label: String,
    /// "Deploying…".
    pub pending_label: String,
    /// "Deployed".
    pub success_label: String,
    /// Success holds this long, then the label returns (Oxyn plus: user-adjustable).
    pub reset_after: Duration,
    /// First retry wait; doubles per failure up to 30 s (Oxyn plus).
    pub backoff: Duration,
    started: Option<ClockMs>,
    succeeded: Option<ClockMs>,
    failed: Option<ClockMs>,
    failures: u32,
}

impl ActionButton {
    /// An action button.
    pub fn new(label: &str, pending: &str, success: &str) -> Self {
        Self {
            label: label.into(),
            pending_label: pending.into(),
            success_label: success.into(),
            reset_after: timing::RESET_AFTER,
            backoff: Duration::ZERO,
            started: None,
            succeeded: None,
            failed: None,
            failures: 0,
        }
    }
    /// Pressed. Returns false when the press is ignored (pending, or inside the back-off).
    pub fn press(&mut self, now: ClockMs) -> bool {
        match self.phase(now) {
            ActionPhase::Pending | ActionPhase::Error { retry_in: Some(_) } => false,
            _ => {
                self.started = Some(now);
                self.succeeded = None;
                self.failed = None;
                true
            }
        }
    }
    /// The action finished.
    pub fn resolve(&mut self, ok: bool, now: ClockMs) {
        self.started = None;
        if ok {
            self.succeeded = Some(now);
            self.failures = 0;
        } else {
            self.failed = Some(now);
            self.failures += 1;
        }
    }
    fn wait(&self) -> Duration {
        if self.backoff.is_zero() || self.failures == 0 {
            return Duration::ZERO;
        }
        let ms = self.backoff.as_millis() as u64 * (1u64 << (self.failures - 1).min(8));
        Duration::from_millis(ms.min(30_000))
    }
    /// Phase at `now`.
    pub fn phase(&self, now: ClockMs) -> ActionPhase {
        if self.started.is_some() {
            return ActionPhase::Pending;
        }
        if let Some(t) = self.succeeded
            && now.since(t) < self.reset_after
        {
            return ActionPhase::Success;
        }
        if let Some(t) = self.failed {
            let w = self.wait();
            let gone = now.since(t);
            return ActionPhase::Error {
                retry_in: (gone < w).then(|| (w - gone).as_millis().div_ceil(1000) as u64),
            };
        }
        ActionPhase::Idle
    }
    /// The label at `now`.
    pub fn text(&self, now: ClockMs) -> String {
        match self.phase(now) {
            ActionPhase::Idle => self.label.clone(),
            ActionPhase::Pending => self.pending_label.clone(),
            ActionPhase::Success => self.success_label.clone(),
            ActionPhase::Error { retry_in: Some(s) } => format!("Retry in {s}s"),
            ActionPhase::Error { retry_in: None } => "Retry".into(),
        }
    }
    /// The longest label, so the width can be reserved and never jumps.
    pub fn widest(&self) -> &str {
        [&self.label, &self.pending_label, &self.success_label]
            .into_iter()
            .max_by_key(|s| s.chars().count())
            .map(String::as_str)
            .unwrap_or("")
    }
}

/// Oxyn plus: press-and-hold to confirm a destructive action.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HoldConfirm {
    /// Fill time.
    pub hold: Duration,
    down: Option<ClockMs>,
    released: Option<(ClockMs, f32)>,
    done: bool,
}

impl HoldConfirm {
    /// With the default 1.2 s hold.
    pub fn new() -> Self {
        Self {
            hold: timing::HOLD,
            ..Self::default()
        }
    }
    /// Pointer / Space down.
    pub fn down(&mut self, now: ClockMs) {
        if !self.done {
            self.down = Some(now);
            self.released = None;
        }
    }
    /// Pointer / Space up; the fill drains back.
    pub fn up(&mut self, now: ClockMs) {
        if let Some(d) = self.down.take() {
            let p = (now.since(d).as_secs_f32() / self.hold.as_secs_f32()).min(1.0);
            self.released = Some((now, p));
        }
    }
    /// Fill 0..1 at `now`; reaching 1 confirms (returns `(fill, confirmed)`).
    pub fn tick(&mut self, now: ClockMs) -> (f32, bool) {
        if self.done {
            return (1.0, true);
        }
        if let Some(d) = self.down {
            let p = now.since(d).as_secs_f32() / self.hold.as_secs_f32();
            if p >= 1.0 {
                self.done = true;
                self.down = None;
                return (1.0, true);
            }
            return (p, false);
        }
        if let Some((t, p)) = self.released {
            let k = now.since(t).as_secs_f32() / timing::DRAIN.as_secs_f32();
            return ((p * (1.0 - k)).max(0.0), false);
        }
        (0.0, false)
    }
    /// Back to the start.
    pub fn reset(&mut self) {
        *self = Self {
            hold: self.hold,
            ..Self::default()
        };
    }
}

/// Split button: a main action and a menu of alternatives. Oxyn plus: remembers the last choice
/// and promotes it to the main half.
#[derive(Clone, Debug, PartialEq)]
pub struct SplitButton {
    /// `(id, label)` for every action; the first is the default main action.
    pub actions: Vec<(String, String)>,
    /// Promote the last chosen action to the main half.
    pub remember: bool,
    main: usize,
}

impl SplitButton {
    /// From `(id, label)` pairs.
    pub fn new(actions: &[(&str, &str)]) -> Self {
        Self {
            actions: actions
                .iter()
                .map(|(a, b)| (a.to_string(), b.to_string()))
                .collect(),
            remember: false,
            main: 0,
        }
    }
    /// The main half's `(id, label)`.
    pub fn main(&self) -> (&str, &str) {
        let (a, b) = &self.actions[self.main.min(self.actions.len().saturating_sub(1))];
        (a, b)
    }
    /// The menu: every action except the main one.
    pub fn menu(&self) -> Vec<(&str, &str)> {
        self.actions
            .iter()
            .enumerate()
            .filter(|(i, _)| *i != self.main)
            .map(|(_, (a, b))| (a.as_str(), b.as_str()))
            .collect()
    }
    /// A menu item was chosen; returns its id.
    pub fn choose(&mut self, id: &str) -> Option<String> {
        let i = self.actions.iter().position(|(a, _)| a == id)?;
        if self.remember {
            self.main = i;
        }
        Some(self.actions[i].0.clone())
    }
}

/// Group layout.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Orientation {
    /// Row.
    #[default]
    Horizontal,
    /// Column.
    Vertical,
}

impl Orientation {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Horizontal => "horizontal",
            Self::Vertical => "vertical",
        }
    }
}

/// The highlight shared by every segment of a button group, floating tray or menu: one shape that
/// glides between items instead of each item lighting up on its own.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Glide {
    /// Item under the pointer or keyboard focus.
    pub target: Option<usize>,
    /// Item it travelled from (`None` = it appeared from nothing).
    pub from: Option<usize>,
    /// When the move started.
    pub at: ClockMs,
    /// Keyboard moves jump (Arc dropdown); pointer moves glide.
    pub jump: bool,
}

impl Glide {
    /// Moves to `to` at `now`. `keyboard` makes the move instant.
    pub fn go(&mut self, to: Option<usize>, now: ClockMs, keyboard: bool) {
        if to == self.target {
            return;
        }
        self.from = self.target;
        self.target = to;
        self.at = now;
        self.jump = keyboard;
    }
    /// Position + size of the highlight over `rects` (offset, length along the axis) at `now`.
    /// Returns `(start, len, opacity, scale)`.
    pub fn frame(
        &self,
        rects: &[(f32, f32)],
        now: ClockMs,
        reduced: bool,
    ) -> Option<(f32, f32, f32, f32)> {
        let to = rects.get(self.target?)?;
        let d = timing::WIDTH.as_secs_f32();
        let p = (now.since(self.at).as_secs_f32() / d).clamp(0.0, 1.0);
        let k = crate::ease::Ease::ease(&oxyn_tokens::motion::ease::OUT, p);
        match self.from.and_then(|f| rects.get(f)) {
            Some(from) if !self.jump && !reduced => Some((
                from.0 + (to.0 - from.0) * k,
                from.1 + (to.1 - from.1) * k,
                1.0,
                1.0,
            )),
            Some(_) => Some((to.0, to.1, 1.0, 1.0)),
            // Appearing from nothing: land in place, grow from 94 %, fade in.
            None if reduced => Some((to.0, to.1, 1.0, 1.0)),
            None => Some((
                to.0,
                to.1,
                k,
                timing::APPEAR_SCALE + (1.0 - timing::APPEAR_SCALE) * k,
            )),
        }
    }
}

/// One segment of a button group, floating tray or expanding group.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GroupItem {
    /// Id passed back on activation.
    pub id: String,
    /// Label (tooltip and accessible name when `icon_only`).
    pub label: String,
    /// Icon name.
    pub icon: Option<String>,
    /// Oxyn plus: per-segment badge count.
    pub count: Option<u64>,
    /// Label after activation (expanding group: "Copied").
    pub done_label: Option<String>,
    /// Disabled.
    pub disabled: bool,
}

impl GroupItem {
    /// A segment.
    pub fn new(id: &str, label: &str, icon: Option<&str>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            icon: icon.map(str::to_string),
            count: None,
            done_label: None,
            disabled: false,
        }
    }
    /// With a done label.
    pub fn done(mut self, d: &str) -> Self {
        self.done_label = Some(d.into());
        self
    }
    /// With a count.
    pub fn count(mut self, n: u64) -> Self {
        self.count = Some(n);
        self
    }
}

/// Joined button group (Arc `button-group`): hairline dividers next to the highlight fade out.
pub fn hidden_dividers(highlight: Option<usize>, len: usize) -> Vec<bool> {
    // Divider i sits between item i and i + 1.
    (0..len.saturating_sub(1))
        .map(|i| highlight.is_some_and(|h| h == i || h == i + 1))
        .collect()
}

/// Oxyn plus: how many segments fit in `available` px; the rest go to an overflow menu.
pub fn fit(widths: &[f32], available: f32, overflow_button: f32) -> usize {
    let total: f32 = widths.iter().sum();
    if total <= available {
        return widths.len();
    }
    let mut used = overflow_button;
    let mut n = 0;
    for w in widths {
        if used + w > available {
            break;
        }
        used += w;
        n += 1;
    }
    n
}

/// Expanding button group (Arc): the hovered / focused button grows to show its label.
#[derive(Clone, Debug, PartialEq)]
pub struct ExpandingGroup {
    /// `(id, label, done_label)`.
    pub items: Vec<(String, String, String)>,
    /// Expanded on load (`defaultExpanded`).
    pub expanded: Option<usize>,
    /// Oxyn plus: labels always shown (accessibility toggle).
    pub always: bool,
    /// Oxyn plus: wait this long on hover before expanding.
    pub delay: Duration,
    /// How long "done" holds (`confirmDuration`).
    pub confirm: Duration,
    hover_at: Option<(usize, ClockMs)>,
    confirmed: Option<(usize, ClockMs)>,
}

impl ExpandingGroup {
    /// From `(id, label, done_label)`.
    pub fn new(items: &[(&str, &str, &str)]) -> Self {
        Self {
            items: items
                .iter()
                .map(|(a, b, c)| (a.to_string(), b.to_string(), c.to_string()))
                .collect(),
            expanded: None,
            always: false,
            delay: Duration::ZERO,
            confirm: timing::CONFIRM,
            hover_at: None,
            confirmed: None,
        }
    }
    /// Pointer entered / focus moved to `i` (`None` = left the group).
    pub fn point(&mut self, i: Option<usize>, now: ClockMs) {
        self.hover_at = i.map(|i| (i, now));
        if i.is_none() {
            self.expanded = None;
        }
    }
    /// Activated `i`; it confirms in place.
    pub fn act(&mut self, i: usize, now: ClockMs) -> Option<String> {
        self.confirmed = Some((i, now));
        self.items.get(i).map(|x| x.0.clone())
    }
    /// Which item is open at `now`.
    pub fn open(&self, now: ClockMs) -> Option<usize> {
        if let Some((i, t)) = self.hover_at
            && now.since(t) >= self.delay
        {
            return Some(i);
        }
        self.expanded
    }
    /// Whether `i` is showing its done label.
    pub fn done(&self, i: usize, now: ClockMs) -> bool {
        self.confirmed
            .is_some_and(|(c, t)| c == i && now.since(t) < self.confirm)
    }
    /// Label slot chars to reserve for `i`: the longer of label and done label.
    pub fn slot(&self, i: usize) -> usize {
        self.items
            .get(i)
            .map(|(_, l, d)| l.chars().count().max(d.chars().count()))
            .unwrap_or(0)
    }
    /// Whether `i` shows its label at `now`.
    pub fn shows_label(&self, i: usize, now: ClockMs) -> bool {
        self.always || self.open(now) == Some(i) || self.done(i, now)
    }
}

/// Oxyn plus (floating tray): auto-hide after idle.
pub fn tray_visible(last_activity: ClockMs, now: ClockMs, auto_hide: Option<Duration>) -> bool {
    auto_hide.is_none_or(|d| now.since(last_activity) < d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn variants_sizes_and_press() {
        assert_eq!(
            ButtonVariant::from_id("danger"),
            Some(ButtonVariant::Destructive)
        );
        assert_eq!(
            ButtonSize::ALL.map(|s| s.height()),
            [24.0, 28.0, 32.0, 40.0, 48.0]
        );
        assert_eq!(press_scale(false, true, false), 0.97);
        assert_eq!(press_scale(true, true, false), 0.94);
        assert_eq!(press_scale(true, true, true), 1.0);
        assert!(ButtonState::Loading(None).inert() && ButtonState::Loading(None).busy());
    }

    #[test]
    fn agent_binding() {
        assert_eq!(agent_bound(&AgentState::Idle), ButtonState::Idle);
        assert!(agent_bound(&AgentState::Streaming).busy());
    }

    #[test]
    fn action_button_cycle_and_backoff() {
        let mut a = ActionButton::new("Deploy", "Deploying…", "Deployed");
        assert!(a.press(ClockMs(0)));
        assert!(!a.press(ClockMs(10)), "pending ignores presses");
        assert_eq!(a.text(ClockMs(10)), "Deploying…");
        a.resolve(true, ClockMs(500));
        assert_eq!(a.phase(ClockMs(600)), ActionPhase::Success);
        assert_eq!(a.text(ClockMs(2_200)), "Deploy");
        assert_eq!(a.widest(), "Deploying…");
        a.backoff = Duration::from_secs(2);
        a.press(ClockMs(3_000));
        a.resolve(false, ClockMs(3_100));
        assert_eq!(a.text(ClockMs(3_200)), "Retry in 2s");
        assert!(!a.press(ClockMs(3_500)));
        assert_eq!(
            a.phase(ClockMs(5_200)),
            ActionPhase::Error { retry_in: None }
        );
        assert!(a.press(ClockMs(5_200)));
        a.resolve(false, ClockMs(5_300));
        assert_eq!(a.text(ClockMs(5_300)), "Retry in 4s", "doubles");
    }

    #[test]
    fn hold_to_confirm() {
        let mut h = HoldConfirm::new();
        h.down(ClockMs(0));
        let (p, done) = h.tick(ClockMs(600));
        assert!((p - 0.5).abs() < 1e-3 && !done);
        h.up(ClockMs(600));
        assert!(h.tick(ClockMs(700)).0 < 0.5);
        assert_eq!(h.tick(ClockMs(900)).0, 0.0);
        h.down(ClockMs(1_000));
        assert!(h.tick(ClockMs(2_300)).1);
    }

    #[test]
    fn split_button_remembers() {
        let mut s = SplitButton::new(&[
            ("merge", "Merge"),
            ("squash", "Squash and merge"),
            ("rebase", "Rebase"),
        ]);
        assert_eq!(s.menu().len(), 2);
        s.choose("squash");
        assert_eq!(s.main().0, "merge", "off by default");
        s.remember = true;
        s.choose("squash");
        assert_eq!(s.main(), ("squash", "Squash and merge"));
        assert!(s.menu().iter().any(|(a, _)| *a == "merge"));
    }

    #[test]
    fn glide_and_dividers() {
        let rects = [(0.0, 40.0), (40.0, 60.0), (100.0, 50.0)];
        let mut g = Glide::default();
        g.go(Some(0), ClockMs(0), false);
        let (_, _, o, s) = g.frame(&rects, ClockMs(0), false).unwrap();
        assert_eq!((o, s), (0.0, 0.94), "appears from nothing");
        g.go(Some(2), ClockMs(1_000), false);
        let (x, w, ..) = g.frame(&rects, ClockMs(1_000), false).unwrap();
        assert_eq!((x, w), (0.0, 40.0));
        let (x, ..) = g.frame(&rects, ClockMs(2_000), false).unwrap();
        assert_eq!(x, 100.0);
        g.go(Some(1), ClockMs(3_000), true);
        assert_eq!(
            g.frame(&rects, ClockMs(3_000), false).unwrap().0,
            40.0,
            "keyboard jumps"
        );
        assert_eq!(hidden_dividers(Some(1), 3), vec![true, true]);
        assert_eq!(hidden_dividers(Some(0), 3), vec![true, false]);
        assert_eq!(fit(&[50.0, 50.0, 50.0, 50.0], 160.0, 30.0), 2);
        assert_eq!(fit(&[50.0, 50.0], 160.0, 30.0), 2);
    }

    #[test]
    fn expanding_group() {
        let mut g =
            ExpandingGroup::new(&[("copy", "Copy", "Copied"), ("share", "Share", "Shared")]);
        g.delay = Duration::from_millis(100);
        g.point(Some(1), ClockMs(0));
        assert_eq!(g.open(ClockMs(50)), None);
        assert_eq!(g.open(ClockMs(120)), Some(1));
        g.act(0, ClockMs(200));
        assert!(g.done(0, ClockMs(300)) && g.shows_label(0, ClockMs(300)));
        assert!(!g.done(0, ClockMs(2_000)));
        assert_eq!(g.slot(0), 6);
        g.point(None, ClockMs(2_000));
        assert_eq!(g.open(ClockMs(2_000)), None);
        assert!(tray_visible(
            ClockMs(0),
            ClockMs(100),
            Some(Duration::from_secs(3))
        ));
        assert!(!tray_visible(
            ClockMs(0),
            ClockMs(4_000),
            Some(Duration::from_secs(3))
        ));
    }
}
