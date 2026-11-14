//! Dialog, side sheet, sheet stack and bottom sheet (`dialog.md`, `sheet.md`, `sheet-stack.md`,
//! `bottom-sheet.md`).
//!
//! - **Dialog** (affectUI Dialog, Arc UI Dialog): a modal window on a dimmed backdrop. Escape and a
//!   press on the backdrop close it (each can be turned off), focus is trapped inside and handed
//!   back to the trigger. Sizes `sm`, `md`, `lg`, `full`. Oxyn plus: a busy state that holds the
//!   dialog open while a confirm runs, nested dialogs (Escape closes the top one only) and a
//!   non-modal floating variant.
//! - **Sheet** (Oxyn plus, affectUI's "side sheet" size): the same surface docked to an edge.
//! - **Sheet stack** (Arc UI Sheet stack): nested sheets with depth. The one below scales back 5 %,
//!   dims and peeks above; a drag or fling past 35 % / 450 px/s pops the top. `auto` mode turns
//!   them into stacked dialogs at or above a breakpoint (640 px).
//! - **Bottom sheet** (Arc UI Bottom sheet): rests at detents (fractions of the viewport height),
//!   follows the finger 1:1, rubber-bands past the tallest detent and projects the release
//!   velocity to pick a detent or dismiss. The backdrop dim is a function of the sheet's height.
//!
//! Rebuilt from the public docs of both kits; no source used.

use std::time::Duration;

use crate::ClockMs;

/// Timings.
pub mod timing {
    use std::time::Duration;
    /// Dialog enter: backdrop fade, card rises 8 px and scales from 0.96 (Arc) / 0.97 (affectUI).
    pub const DIALOG_IN: Duration = Duration::from_millis(240);
    /// Dialog exit: quicker, shrinks slightly.
    pub const DIALOG_OUT: Duration = Duration::from_millis(160);
    /// Rise distance px.
    pub const RISE: f32 = 8.0;
    /// Start scale.
    pub const SCALE_FROM: f32 = 0.96;
    /// Sheet slide in.
    pub const SHEET_IN: Duration = Duration::from_millis(320);
    /// Sheet slide out.
    pub const SHEET_OUT: Duration = Duration::from_millis(220);
    /// Stack: the sheet below scales back by this much per level.
    pub const STACK_SCALE: f32 = 0.05;
    /// Stack: peek per level (sheets).
    pub const PEEK_SHEET: f32 = 10.0;
    /// Stack: peek per level (dialogs).
    pub const PEEK_DIALOG: f32 = 14.0;
    /// Fling distance (fraction of the sheet height) that pops the top sheet.
    pub const FLING_FRACTION: f32 = 0.35;
    /// Fling velocity px/s that pops the top sheet.
    pub const FLING_VELOCITY: f32 = 450.0;
    /// How far ahead a release velocity is projected (s).
    pub const PROJECT: f32 = 0.2;
    /// Rubber band constant (iOS 0.55).
    pub const RUBBER: f32 = 0.55;
    /// Backdrop dim at the tallest detent.
    pub const DIM_MAX: f32 = 0.45;
}

/// Dialog sizes (affectUI `size`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DialogSize {
    /// 400 px (AlertDialog).
    Sm,
    /// 520 px.
    #[default]
    Md,
    /// 680 px.
    Lg,
    /// Oxyn plus: fills the viewport minus a gutter.
    Full,
}

impl DialogSize {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "sm" => Self::Sm,
            "lg" => Self::Lg,
            "full" => Self::Full,
            _ => Self::Md,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
            Self::Full => "full",
        }
    }
    /// Width in px (`None` = fill).
    pub fn width(self) -> Option<f32> {
        match self {
            Self::Sm => Some(400.0),
            Self::Md => Some(520.0),
            Self::Lg => Some(680.0),
            Self::Full => None,
        }
    }
}

/// Why a surface asked to close.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum CloseReason {
    /// Escape.
    Escape,
    /// Press on the backdrop / scrim.
    Backdrop,
    /// The × button.
    CloseButton,
    /// A footer action (Cancel, Confirm).
    Action,
    /// Drag or fling past the threshold.
    Drag,
    /// The host.
    Programmatic,
}

/// Where a surface is in its life.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Phase {
    /// Not shown.
    Closed,
    /// Entering.
    Opening,
    /// Shown and settled.
    Open,
    /// Leaving.
    Closing,
}

/// A dialog (also drives the side sheet).
#[derive(Clone, Debug, PartialEq)]
pub struct Dialog {
    /// Size.
    pub size: DialogSize,
    /// A press on the backdrop closes it (default on).
    pub close_on_backdrop: bool,
    /// Escape closes it (default on).
    pub close_on_escape: bool,
    /// Modal: traps focus and blocks the page (Oxyn plus: off = floating, non-modal).
    pub modal: bool,
    /// Oxyn plus: a confirm is running; the dialog can't be dismissed until it settles.
    pub busy: bool,
    open: bool,
    opened_at: Option<ClockMs>,
    closed_at: Option<ClockMs>,
    /// Why it last closed.
    pub last_reason: Option<CloseReason>,
}

impl Default for Dialog {
    fn default() -> Self {
        Self {
            size: DialogSize::Md,
            close_on_backdrop: true,
            close_on_escape: true,
            modal: true,
            busy: false,
            open: false,
            opened_at: None,
            closed_at: None,
            last_reason: None,
        }
    }
}

impl Dialog {
    /// Closed dialog.
    pub fn new() -> Self {
        Self::default()
    }
    /// Is it open (or opening)?
    pub fn is_open(&self) -> bool {
        self.open
    }
    /// Opens it.
    pub fn show(&mut self, now: ClockMs) {
        if !self.open {
            self.open = true;
            self.opened_at = Some(now);
            self.closed_at = None;
            self.last_reason = None;
        }
    }
    /// Asks to close; returns whether it closed. Backdrop and Escape respect their switches, and
    /// nothing but the host closes a busy dialog.
    pub fn request_close(&mut self, reason: CloseReason, now: ClockMs) -> bool {
        if !self.open {
            return false;
        }
        let allowed = match reason {
            CloseReason::Programmatic => true,
            _ if self.busy => false,
            CloseReason::Backdrop => self.close_on_backdrop && self.modal,
            CloseReason::Escape => self.close_on_escape,
            _ => true,
        };
        if allowed {
            self.open = false;
            self.closed_at = Some(now);
            self.last_reason = Some(reason);
        }
        allowed
    }
    /// Phase at `now`.
    pub fn phase(&self, now: ClockMs, reduced: bool) -> Phase {
        let (i, o) = if reduced {
            (Duration::from_millis(120), Duration::from_millis(120))
        } else {
            (timing::DIALOG_IN, timing::DIALOG_OUT)
        };
        match (self.open, self.opened_at, self.closed_at) {
            (true, Some(t), _) if now.since(t) < i => Phase::Opening,
            (true, _, _) => Phase::Open,
            (false, _, Some(t)) if now.since(t) < o => Phase::Closing,
            _ => Phase::Closed,
        }
    }
    /// Focus returns to the trigger after this close.
    pub fn restores_focus(&self) -> bool {
        !matches!(self.last_reason, Some(CloseReason::Backdrop)) || self.modal
    }
}

/// Focus trap: the next focusable index from `current` among `n` (Tab / Shift+Tab wrap).
pub fn trap(n: usize, current: Option<usize>, shift: bool) -> Option<usize> {
    if n == 0 {
        return None;
    }
    Some(match (current, shift) {
        (None, false) => 0,
        (None, true) => n - 1,
        (Some(i), false) => (i + 1) % n,
        (Some(i), true) => (i + n - 1) % n,
    })
}

/// Nested modal surfaces: only the top one is modal; the rest are inert (Oxyn plus for dialogs,
/// Arc's rule for sheet stacks).
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ModalStack {
    ids: Vec<String>,
}

impl ModalStack {
    /// Pushes `id` on top (moving it there when already open).
    pub fn push(&mut self, id: &str) {
        self.ids.retain(|x| x != id);
        self.ids.push(id.into());
    }
    /// Closes the top one (Escape).
    pub fn pop(&mut self) -> Option<String> {
        self.ids.pop()
    }
    /// Closes everything above `id` (keeps `id`).
    pub fn pop_to(&mut self, id: &str) -> Vec<String> {
        match self.ids.iter().position(|x| x == id) {
            Some(i) => self.ids.split_off(i + 1),
            None => vec![],
        }
    }
    /// Closes all.
    pub fn clear(&mut self) -> Vec<String> {
        std::mem::take(&mut self.ids)
    }
    /// Top id.
    pub fn top(&self) -> Option<&str> {
        self.ids.last().map(String::as_str)
    }
    /// Ids, bottom first.
    pub fn ids(&self) -> &[String] {
        &self.ids
    }
    /// How many are open.
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    /// Nothing open.
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }
    /// `id` is below the top (inert, aria-hidden).
    pub fn is_inert(&self, id: &str) -> bool {
        self.ids.iter().any(|x| x == id) && self.top() != Some(id)
    }
    /// Levels from the top (0 = top).
    pub fn depth_of(&self, id: &str) -> Option<usize> {
        let n = self.ids.len();
        self.ids.iter().position(|x| x == id).map(|i| n - 1 - i)
    }
}

/// Which edge a sheet docks to.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum SheetSide {
    /// Right edge (default for inspectors).
    #[default]
    Right,
    /// Left edge.
    Left,
    /// Top edge.
    Top,
    /// Bottom edge.
    Bottom,
}

impl SheetSide {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "left" => Self::Left,
            "top" => Self::Top,
            "bottom" => Self::Bottom,
            _ => Self::Right,
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Right => "right",
            Self::Left => "left",
            Self::Top => "top",
            Self::Bottom => "bottom",
        }
    }
    /// Horizontal edge.
    pub fn horizontal(self) -> bool {
        matches!(self, Self::Left | Self::Right)
    }
    /// The drag that closes it, as a sign on the main axis (+1 = toward +x / +y).
    pub fn close_sign(self) -> f32 {
        match self {
            Self::Right | Self::Bottom => 1.0,
            Self::Left | Self::Top => -1.0,
        }
    }
}

/// Rubber band: how far content moves for `overshoot` px of drag past an edge, with `limit` the
/// dimension it resists against.
pub fn rubber(overshoot: f32, limit: f32) -> f32 {
    if overshoot <= 0.0 || limit <= 0.0 {
        return overshoot.max(0.0);
    }
    let c = timing::RUBBER;
    (1.0 - 1.0 / (overshoot * c / limit + 1.0)) * limit
}

/// Whether a drag of `travel` px toward closing (with `velocity` px/s, positive toward closing)
/// dismisses a surface of `size` px.
pub fn flings(travel: f32, velocity: f32, size: f32) -> bool {
    travel > size * timing::FLING_FRACTION || velocity > timing::FLING_VELOCITY
}

/// Side sheet drag (Oxyn plus): offset along the main axis, positive toward closing.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct EdgeDrag {
    start: f32,
    /// Current offset toward closing (px); negative values are rubber-banded.
    pub offset: f32,
    /// Dragging.
    pub active: bool,
}

impl EdgeDrag {
    /// Pointer down at `pos` on the main axis.
    pub fn begin(&mut self, pos: f32) {
        self.start = pos;
        self.offset = 0.0;
        self.active = true;
    }
    /// Pointer moved to `pos`; `sign` is [`SheetSide::close_sign`], `size` the sheet size.
    pub fn moved(&mut self, pos: f32, sign: f32, size: f32) {
        if !self.active {
            return;
        }
        let d = (pos - self.start) * sign;
        self.offset = if d >= 0.0 { d } else { -rubber(-d, size) };
    }
    /// Pointer up; returns whether it dismisses.
    pub fn release(&mut self, velocity: f32, size: f32) -> bool {
        let out = self.active && flings(self.offset, velocity, size);
        self.active = false;
        if !out {
            self.offset = 0.0;
        }
        out
    }
}

/// Sheet stack presentation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StackMode {
    /// Sheets below the breakpoint, dialogs at or above it.
    #[default]
    Auto,
    /// Always bottom sheets.
    Sheet,
    /// Always centred dialogs.
    Dialog,
    /// Oxyn plus: side sheets that stack sideways (desktop inspectors).
    Side,
}

impl StackMode {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "sheet" => Self::Sheet,
            "dialog" => Self::Dialog,
            "side" => Self::Side,
            _ => Self::Auto,
        }
    }
    /// What `auto` becomes at `width` px.
    pub fn resolve(self, width: f32, breakpoint: f32) -> Self {
        match self {
            Self::Auto if width >= breakpoint => Self::Dialog,
            Self::Auto => Self::Sheet,
            m => m,
        }
    }
}

/// One level as drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Level {
    /// Scale (1 for the top).
    pub scale: f32,
    /// Lift above the one in front, px (the peek).
    pub lift: f32,
    /// Dim over it (0 for the top).
    pub dim: f32,
}

/// A sheet in the stack.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StackSheet {
    /// Id.
    pub id: String,
    /// Title (heading, accessible name, and the back label of the sheet above).
    pub title: String,
    /// Back label override.
    pub back_label: Option<String>,
    /// Drag and fling dismiss.
    pub dismissible: bool,
}

impl StackSheet {
    /// A sheet.
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            back_label: None,
            dismissible: true,
        }
    }
}

/// Sheet stack (Arc UI Sheet stack).
#[derive(Clone, Debug, PartialEq)]
pub struct SheetStack {
    /// Declared sheets.
    pub sheets: Vec<StackSheet>,
    /// Open ids, bottom first.
    pub stack: ModalStack,
    /// Mode.
    pub mode: StackMode,
    /// Auto breakpoint px (640).
    pub breakpoint: f32,
    /// Oxyn plus: most levels kept open; deeper pushes drop the oldest into the breadcrumb.
    pub max_depth: usize,
    /// Top sheet drag.
    pub drag: EdgeDrag,
}

impl SheetStack {
    /// From declared sheets.
    pub fn new(sheets: Vec<StackSheet>) -> Self {
        Self {
            sheets,
            stack: ModalStack::default(),
            mode: StackMode::Auto,
            breakpoint: 640.0,
            max_depth: 6,
            drag: EdgeDrag::default(),
        }
    }
    fn sheet(&self, id: &str) -> Option<&StackSheet> {
        self.sheets.iter().find(|s| s.id == id)
    }
    /// Pushes a declared sheet.
    pub fn push(&mut self, id: &str) -> bool {
        if self.sheet(id).is_none() {
            return false;
        }
        self.stack.push(id);
        true
    }
    /// Pops the top (Escape, back, fling).
    pub fn pop(&mut self) -> Option<String> {
        self.drag = EdgeDrag::default();
        self.stack.pop()
    }
    /// Pops down to `id`.
    pub fn pop_to(&mut self, id: &str) -> Vec<String> {
        self.stack.pop_to(id)
    }
    /// Closes everything.
    pub fn close(&mut self) -> Vec<String> {
        self.stack.clear()
    }
    /// Open sheets, bottom first.
    pub fn open(&self) -> Vec<&StackSheet> {
        self.stack
            .ids()
            .iter()
            .filter_map(|id| self.sheet(id))
            .collect()
    }
    /// The back button on the top sheet names the sheet below.
    pub fn back_label(&self) -> Option<String> {
        let ids = self.stack.ids();
        if ids.len() < 2 {
            return None;
        }
        let below = self.sheet(&ids[ids.len() - 2])?;
        Some(below.back_label.clone().unwrap_or(below.title.clone()))
    }
    /// The close button reads "Close all" when more than one sheet is open.
    pub fn close_label(&self) -> &'static str {
        if self.stack.len() > 1 {
            "Close all"
        } else {
            "Close"
        }
    }
    /// How level `depth` (0 = top) is drawn, with the top sheet's drag progress `follow`
    /// (0 → 1 as it is dragged away, the levels below ease back up).
    pub fn level(&self, depth: usize, resolved: StackMode, follow: f32) -> Level {
        if depth == 0 {
            return Level {
                scale: 1.0,
                lift: 0.0,
                dim: 0.0,
            };
        }
        let d = (depth as f32 - follow.clamp(0.0, 1.0)).max(0.0);
        let peek = match resolved {
            StackMode::Dialog => timing::PEEK_DIALOG,
            _ => timing::PEEK_SHEET,
        };
        Level {
            scale: 1.0 - timing::STACK_SCALE * d,
            lift: peek * d,
            dim: (0.18 * d).min(0.5),
        }
    }
    /// Breadcrumb trail (Oxyn plus): titles bottom first.
    pub fn trail(&self) -> Vec<String> {
        self.open().iter().map(|s| s.title.clone()).collect()
    }
    /// Top sheet can be dragged.
    pub fn top_dismissible(&self) -> bool {
        self.stack
            .top()
            .and_then(|id| self.sheet(id))
            .is_some_and(|s| s.dismissible)
    }
}

/// What a bottom sheet release does.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SheetRelease {
    /// Settles on this detent.
    Detent(usize),
    /// Closes.
    Dismiss,
}

/// Detent keys on the grabber.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DetentKey {
    /// One detent up.
    Up,
    /// One detent down.
    Down,
    /// Tallest.
    Home,
    /// Smallest.
    End,
    /// Toggle smallest / tallest.
    Toggle,
}

/// Bottom sheet (Arc UI Bottom sheet).
#[derive(Clone, Debug, PartialEq)]
pub struct BottomSheet {
    detents: Vec<f32>,
    /// Current detent index (into the sorted detents).
    pub index: usize,
    /// Viewport height px.
    pub viewport: f32,
    /// Height px while dragging (`None` = resting at the detent).
    pub drag_height: Option<f32>,
    drag_from: (f32, f32),
}

impl BottomSheet {
    /// Detents as fractions of the viewport (Arc default `[0.45, 0.92]`).
    pub fn new(detents: &[f32], initial: usize, viewport: f32) -> Self {
        let mut d: Vec<f32> = detents
            .iter()
            .copied()
            .filter(|x| *x > 0.0 && *x <= 1.0)
            .collect();
        if d.is_empty() {
            d = vec![0.45, 0.92];
        }
        d.sort_by(f32::total_cmp);
        d.dedup();
        let index = initial.min(d.len() - 1);
        Self {
            detents: d,
            index,
            viewport,
            drag_height: None,
            drag_from: (0.0, 0.0),
        }
    }
    /// Sorted detents.
    pub fn detents(&self) -> &[f32] {
        &self.detents
    }
    /// Height px of detent `i`.
    pub fn detent_px(&self, i: usize) -> f32 {
        self.detents[i.min(self.detents.len() - 1)] * self.viewport
    }
    /// Current height px.
    pub fn height(&self) -> f32 {
        self.drag_height
            .unwrap_or_else(|| self.detent_px(self.index))
    }
    /// At the tallest detent.
    pub fn expanded(&self) -> bool {
        self.index + 1 == self.detents.len()
    }
    /// Pointer down at client `y`.
    pub fn begin(&mut self, y: f32) {
        self.drag_from = (y, self.height());
        self.drag_height = Some(self.height());
    }
    /// Pointer moved: 1:1, rubber-banded past the tallest detent.
    pub fn moved(&mut self, y: f32) {
        if self.drag_height.is_none() {
            return;
        }
        let max = self.detent_px(self.detents.len() - 1);
        let raw = self.drag_from.1 + (self.drag_from.0 - y);
        let h = if raw > max {
            max + rubber(raw - max, self.viewport * 0.25)
        } else {
            raw.max(0.0)
        };
        self.drag_height = Some(h);
    }
    /// Pointer up with vertical velocity px/s (positive = downward). The release is projected
    /// ahead and lands on the nearest detent, or dismisses below half the smallest.
    pub fn release(&mut self, velocity_down: f32) -> SheetRelease {
        let h = self.height();
        self.drag_height = None;
        let projected = h - velocity_down * timing::PROJECT;
        let min = self.detent_px(0);
        if projected < min * 0.5
            || (velocity_down > timing::FLING_VELOCITY * 2.0 && self.index == 0)
        {
            return SheetRelease::Dismiss;
        }
        let i = (0..self.detents.len())
            .min_by(|&a, &b| {
                (self.detent_px(a) - projected)
                    .abs()
                    .total_cmp(&(self.detent_px(b) - projected).abs())
            })
            .unwrap_or(0);
        self.index = i;
        SheetRelease::Detent(i)
    }
    /// Grabber keys.
    pub fn key(&mut self, k: DetentKey) -> usize {
        let last = self.detents.len() - 1;
        self.index = match k {
            DetentKey::Up => (self.index + 1).min(last),
            DetentKey::Down => self.index.saturating_sub(1),
            DetentKey::Home => last,
            DetentKey::End => 0,
            DetentKey::Toggle => {
                if self.index == last {
                    0
                } else {
                    last
                }
            }
        };
        self.index
    }
    /// Backdrop dim for the current height: tracks the drag instead of a timer.
    pub fn dim(&self) -> f32 {
        let max = self.detent_px(self.detents.len() - 1).max(1.0);
        (self.height() / max).clamp(0.0, 1.0) * timing::DIM_MAX
    }
    /// Polite announcement after settling.
    pub fn announce(&self) -> String {
        if self.expanded() {
            "Sheet expanded".into()
        } else {
            format!(
                "Sheet at {} percent",
                (self.detents[self.index] * 100.0).round()
            )
        }
    }
    /// Grabber label.
    pub fn grabber_label(&self) -> &'static str {
        if self.expanded() {
            "Collapse sheet"
        } else {
            "Expand sheet"
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_close_rules() {
        let mut d = Dialog::new();
        d.show(ClockMs(0));
        assert_eq!(d.phase(ClockMs(100), false), Phase::Opening);
        assert_eq!(d.phase(ClockMs(400), false), Phase::Open);
        d.close_on_backdrop = false;
        assert!(!d.request_close(CloseReason::Backdrop, ClockMs(500)));
        d.busy = true;
        assert!(!d.request_close(CloseReason::Escape, ClockMs(500)));
        assert!(!d.request_close(CloseReason::CloseButton, ClockMs(500)));
        d.busy = false;
        assert!(d.request_close(CloseReason::Escape, ClockMs(500)));
        assert_eq!(d.phase(ClockMs(560), false), Phase::Closing);
        assert_eq!(d.phase(ClockMs(800), false), Phase::Closed);
        assert_eq!(d.last_reason, Some(CloseReason::Escape));
    }

    #[test]
    fn trap_wraps() {
        assert_eq!(trap(3, Some(2), false), Some(0));
        assert_eq!(trap(3, Some(0), true), Some(2));
        assert_eq!(trap(3, None, true), Some(2));
        assert_eq!(trap(0, None, false), None);
    }

    #[test]
    fn modal_stack_escape_closes_top_only() {
        let mut s = ModalStack::default();
        s.push("a");
        s.push("b");
        s.push("c");
        assert!(s.is_inert("a") && !s.is_inert("c"));
        assert_eq!(s.depth_of("a"), Some(2));
        assert_eq!(s.pop().as_deref(), Some("c"));
        assert_eq!(s.pop_to("a"), vec!["b".to_string()]);
        assert_eq!(s.top(), Some("a"));
    }

    #[test]
    fn sheet_stack_levels_and_labels() {
        let mut st = SheetStack::new(vec![
            StackSheet::new("run", "Run settings"),
            StackSheet::new("tools", "Tools"),
            StackSheet::new("shell", "Shell access"),
        ]);
        assert!(!st.push("missing"));
        st.push("run");
        assert_eq!(st.close_label(), "Close");
        st.push("tools");
        st.push("shell");
        assert_eq!(st.back_label().as_deref(), Some("Tools"));
        assert_eq!(st.close_label(), "Close all");
        let l = st.level(1, StackMode::Sheet, 0.0);
        assert!((l.scale - 0.95).abs() < 1e-6 && l.lift == 10.0);
        let half = st.level(1, StackMode::Sheet, 0.5);
        assert!(
            half.scale > l.scale,
            "eases back up while the top is dragged"
        );
        assert_eq!(StackMode::Auto.resolve(800.0, 640.0), StackMode::Dialog);
        assert_eq!(StackMode::Auto.resolve(400.0, 640.0), StackMode::Sheet);
        st.drag.begin(100.0);
        st.drag.moved(300.0, 1.0, 500.0);
        assert!(st.drag.release(0.0, 500.0), "200 px of 500 is past 35 %");
        st.drag.begin(0.0);
        st.drag.moved(-200.0, 1.0, 500.0);
        assert!(
            st.drag.offset > -200.0 && st.drag.offset < 0.0,
            "rubber band"
        );
        assert!(!st.drag.release(0.0, 500.0));
        assert!(flings(10.0, 500.0, 500.0));
    }

    #[test]
    fn bottom_sheet_detents() {
        let mut b = BottomSheet::new(&[0.92, 0.45], 0, 1000.0);
        assert_eq!(b.detents(), &[0.45, 0.92]);
        assert_eq!(b.height(), 450.0);
        b.begin(600.0);
        b.moved(100.0); // up 500 → 950, past the 920 max
        assert!(b.height() > 920.0 && b.height() < 950.0);
        assert_eq!(b.release(0.0), SheetRelease::Detent(1));
        assert!(b.expanded() && b.announce() == "Sheet expanded");
        b.begin(100.0);
        b.moved(500.0); // down 400 → 520
        assert_eq!(b.release(2000.0), SheetRelease::Dismiss);
        b.index = 1;
        b.begin(100.0);
        b.moved(300.0); // 720
        assert_eq!(b.release(1500.0), SheetRelease::Detent(0));
        assert_eq!(b.key(DetentKey::Toggle), 1);
        assert_eq!(b.key(DetentKey::Down), 0);
        assert!((b.dim() - 0.45 * 450.0 / 920.0).abs() < 1e-4);
    }
}
