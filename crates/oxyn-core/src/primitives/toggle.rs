//! Switch, checkbox and radio group (`switch.md`, `checkbox.md`, `radio.md`).

use std::time::Duration;

use super::nav::{NavKey, step};
use crate::ClockMs;

/// Shared sizes for toggles. `Sm` and `Md` are the source sizes; `Lg` is Oxyn plus.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ToggleSize {
    /// Small.
    Sm,
    /// Medium.
    #[default]
    Md,
    /// Oxyn plus: large (touch).
    Lg,
}

impl ToggleSize {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Sm => "sm",
            Self::Md => "md",
            Self::Lg => "lg",
        }
    }
    /// Switch track `(w, h)`.
    pub fn track(self) -> (f32, f32) {
        match self {
            Self::Sm => (28.0, 16.0),
            Self::Md => (36.0, 20.0),
            Self::Lg => (44.0, 24.0),
        }
    }
    /// Checkbox / radio box size.
    pub fn box_size(self) -> f32 {
        match self {
            Self::Sm => 14.0,
            Self::Md => 16.0,
            Self::Lg => 20.0,
        }
    }
}

/// Toggle timings.
pub mod timing {
    use std::time::Duration;
    /// Thumb travel (`spring.snap`).
    pub const THUMB: Duration = oxyn_tokens::motion::spring::SNAP;
    /// Track colour fade.
    pub const TRACK: Duration = oxyn_tokens::motion::dur::BASE;
    /// The check draws itself.
    pub const CHECK: Duration = oxyn_tokens::motion::dur::check::DRAW;
    /// Press nudge.
    pub const PRESS: Duration = oxyn_tokens::motion::dur::INSTANT;
    /// Press nudge scale for the box.
    pub const PRESS_SCALE: f32 = 0.92;
    /// Pressed thumb stretches by this fraction of its width toward the other side.
    pub const STRETCH: f32 = 0.28;
    /// Radio: the highlight glides to the picked row.
    pub const GLIDE: Duration = oxyn_tokens::motion::spring::SNAP;
}

// ---------------------------------------------------------------------------------------------
// Switch

/// Switch state.
#[derive(Clone, Debug, PartialEq)]
pub struct Switch {
    /// On.
    pub checked: bool,
    /// Disabled.
    pub disabled: bool,
    /// Oxyn plus: applying (async). The thumb shows a spinner and input is ignored; the switch
    /// shows the *target* position and rolls back on failure.
    pub pending: bool,
    /// Oxyn plus: "requires restart" note shown after a change.
    pub restart_note: bool,
    pressed_at: Option<ClockMs>,
    changed_at: Option<ClockMs>,
    before: bool,
}

impl Switch {
    /// A switch.
    pub fn new(checked: bool) -> Self {
        Self {
            checked,
            disabled: false,
            pending: false,
            restart_note: false,
            pressed_at: None,
            changed_at: None,
            before: checked,
        }
    }
    /// Pointer / Space down: the thumb stretches toward where it is going.
    pub fn press(&mut self, now: ClockMs) {
        if !self.disabled && !self.pending {
            self.pressed_at = Some(now);
        }
    }
    /// Pointer leaves while pressed: cancel without toggling.
    pub fn cancel(&mut self) {
        self.pressed_at = None;
    }
    /// Release / click / Space / Enter: toggle. Returns the new value, or `None` if ignored.
    pub fn toggle(&mut self, now: ClockMs) -> Option<bool> {
        self.pressed_at = None;
        if self.disabled || self.pending {
            return None;
        }
        self.before = self.checked;
        self.checked = !self.checked;
        self.changed_at = Some(now);
        Some(self.checked)
    }
    /// Toggle as an async apply: shows the target with a spinner until [`Switch::settle`].
    pub fn toggle_async(&mut self, now: ClockMs) -> Option<bool> {
        let v = self.toggle(now)?;
        self.pending = true;
        Some(v)
    }
    /// The apply finished; `false` rolls back to where it was.
    pub fn settle(&mut self, ok: bool, now: ClockMs) {
        self.pending = false;
        if !ok {
            self.checked = self.before;
            self.changed_at = Some(now);
        }
    }
    /// Thumb geometry at `now` for a `track` and thumb inset `pad`: `(x, width)` in px.
    pub fn thumb(&self, track: (f32, f32), pad: f32, now: ClockMs, reduced: bool) -> (f32, f32) {
        let d = track.1 - pad * 2.0;
        let off = pad;
        let on = track.0 - pad - d;
        let target = if self.checked { on } else { off };
        let from = if self.checked { off } else { on };
        let x = match (self.changed_at, reduced) {
            (Some(t), false) => {
                let p = (now.since(t).as_secs_f32() / timing::THUMB.as_secs_f32()).clamp(0.0, 1.0);
                let k = crate::ease::Ease::ease(&oxyn_tokens::motion::ease::OUT, p);
                from + (target - from) * k
            }
            _ => target,
        };
        if self.pressed_at.is_some() && !reduced {
            // Stretch toward the side it would go to; keep the anchored edge still.
            let grow = d * timing::STRETCH;
            if self.checked {
                (x - grow, d + grow)
            } else {
                (x, d + grow)
            }
        } else {
            (x, d)
        }
    }
    /// `aria-checked`.
    pub fn aria_checked(&self) -> &'static str {
        if self.checked { "true" } else { "false" }
    }
}

// ---------------------------------------------------------------------------------------------
// Checkbox

/// Checkbox value.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Check {
    /// Off.
    #[default]
    Unchecked,
    /// On.
    Checked,
    /// Some of them (Select all).
    Indeterminate,
}

impl Check {
    /// `aria-checked`.
    pub fn aria(self) -> &'static str {
        match self {
            Self::Unchecked => "false",
            Self::Checked => "true",
            Self::Indeterminate => "mixed",
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Unchecked => "unchecked",
            Self::Checked => "checked",
            Self::Indeterminate => "indeterminate",
        }
    }
    /// Next value on activation. Indeterminate goes to checked (native behaviour). With
    /// `tri_state` (Oxyn plus) the cycle is unchecked → checked → indeterminate → unchecked.
    pub fn next(self, tri_state: bool) -> Self {
        match (self, tri_state) {
            (Self::Unchecked, _) => Self::Checked,
            (Self::Checked, false) => Self::Unchecked,
            (Self::Checked, true) => Self::Indeterminate,
            (Self::Indeterminate, false) => Self::Checked,
            (Self::Indeterminate, true) => Self::Unchecked,
        }
    }
    /// The Select-all value for children.
    pub fn of_all(children: &[bool]) -> Self {
        match children.iter().filter(|&&c| c).count() {
            0 => Self::Unchecked,
            n if n == children.len() => Self::Checked,
            _ => Self::Indeterminate,
        }
    }
}

/// Checkbox look (affectUI `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum CheckboxVariant {
    /// Box + label.
    #[default]
    Plain,
    /// A bordered row that is the hit target.
    Card,
}

/// How far the check mark has drawn (0..1) `since` a change, and the dash's growth from the
/// middle. Oxyn plus: `speed` multiplies the draw time.
pub fn check_draw(since: Duration, speed: f32, reduced: bool) -> f32 {
    if reduced {
        return 1.0;
    }
    let d = timing::CHECK.as_secs_f32() * speed.max(0.1);
    crate::ease::Ease::ease(
        &oxyn_tokens::motion::ease::OUT,
        (since.as_secs_f32() / d).clamp(0.0, 1.0),
    )
}

/// Select-all toggle: checks every child unless all are checked.
pub fn select_all(children: &mut [bool]) {
    let all = children.iter().all(|&c| c);
    children.iter_mut().for_each(|c| *c = !all);
}

// ---------------------------------------------------------------------------------------------
// Radio group

/// Radio renderer (Oxyn plus: Arc ships a list only).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum RadioLayout {
    /// Rows with a dot.
    #[default]
    List,
    /// Bordered cards with a description.
    Cards,
    /// Inline row of dots.
    Inline,
}

impl RadioLayout {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::List => "list",
            Self::Cards => "cards",
            Self::Inline => "inline",
        }
    }
}

/// One option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RadioOption {
    /// Value.
    pub value: String,
    /// Label.
    pub label: String,
    /// Second line.
    pub description: Option<String>,
    /// Disabled, with the reason as a tooltip (Oxyn plus).
    pub disabled: Option<String>,
}

impl RadioOption {
    /// An option.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            description: None,
            disabled: None,
        }
    }
}

/// Radio group: one tab stop; arrows move *and select* (native radio behaviour), skipping
/// disabled options and wrapping.
#[derive(Clone, Debug, PartialEq)]
pub struct RadioGroup {
    /// Options.
    pub options: Vec<RadioOption>,
    /// Selected value.
    pub value: Option<String>,
    /// Focused index.
    pub focus: Option<usize>,
    /// When the selection last changed (the highlight glide and the dot spring).
    pub changed_at: Option<ClockMs>,
    /// Previous selection index (where the highlight glides from).
    pub prev: Option<usize>,
}

impl RadioGroup {
    /// A group.
    pub fn new(options: Vec<RadioOption>, value: Option<&str>) -> Self {
        Self {
            options,
            value: value.map(str::to_string),
            focus: None,
            changed_at: None,
            prev: None,
        }
    }
    /// Index of the selected option.
    pub fn selected(&self) -> Option<usize> {
        let v = self.value.as_deref()?;
        self.options.iter().position(|o| o.value == v)
    }
    /// The tab stop: the selected option, else the first enabled one.
    pub fn tab_stop(&self) -> Option<usize> {
        self.selected()
            .or_else(|| self.options.iter().position(|o| o.disabled.is_none()))
    }
    /// Picks `i`. Returns the value when it changed.
    pub fn pick(&mut self, i: usize, now: ClockMs) -> Option<String> {
        let o = self.options.get(i)?;
        if o.disabled.is_some() {
            return None;
        }
        self.focus = Some(i);
        if self.value.as_deref() == Some(&o.value) {
            return None;
        }
        self.prev = self.selected();
        self.value = Some(o.value.clone());
        self.changed_at = Some(now);
        self.value.clone()
    }
    /// Arrow / Home / End: move and select.
    pub fn key(&mut self, k: NavKey, now: ClockMs) -> Option<String> {
        let from = self.focus.or(self.tab_stop());
        let opts = &self.options;
        let to = step(from, k, opts.len(), true, |i| opts[i].disabled.is_none())?;
        self.pick(to, now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn switch_toggles_stretches_and_rolls_back() {
        let mut s = Switch::new(false);
        s.press(ClockMs(0));
        let (x, w) = s.thumb((36.0, 20.0), 2.0, ClockMs(0), false);
        assert_eq!(x, 2.0);
        assert!(w > 16.0, "pressed thumb stretches");
        assert_eq!(s.toggle(ClockMs(10)), Some(true));
        let (x_end, w_end) = s.thumb((36.0, 20.0), 2.0, ClockMs(1_000), false);
        assert_eq!((x_end, w_end), (18.0, 16.0));
        assert_eq!(
            s.thumb((36.0, 20.0), 2.0, ClockMs(10), true).0,
            18.0,
            "reduced: instant"
        );
        s.toggle_async(ClockMs(2_000));
        assert!(s.pending && !s.checked);
        assert_eq!(s.toggle(ClockMs(2_100)), None, "pending ignores input");
        s.settle(false, ClockMs(2_500));
        assert!(s.checked, "rolled back");
        s.disabled = true;
        assert_eq!(s.toggle(ClockMs(3_000)), None);
    }

    #[test]
    fn checkbox_cycles() {
        assert_eq!(Check::Indeterminate.next(false), Check::Checked);
        assert_eq!(Check::Checked.next(true), Check::Indeterminate);
        assert_eq!(Check::Indeterminate.next(true), Check::Unchecked);
        assert_eq!(Check::of_all(&[true, false]), Check::Indeterminate);
        assert_eq!(Check::of_all(&[true, true]).aria(), "true");
        let mut c = [true, false, true];
        select_all(&mut c);
        assert_eq!(c, [true, true, true]);
        select_all(&mut c);
        assert_eq!(c, [false, false, false]);
        assert_eq!(check_draw(Duration::from_millis(240), 1.0, false), 1.0);
        assert!(check_draw(Duration::from_millis(240), 2.0, false) < 1.0);
    }

    #[test]
    fn radio_arrows_select_and_skip_disabled() {
        let mut opts = vec![
            RadioOption::new("a", "Auto"),
            RadioOption::new("b", "Balanced"),
            RadioOption::new("c", "Careful"),
        ];
        opts[1].disabled = Some("Needs a paid plan".into());
        let mut g = RadioGroup::new(opts, Some("a"));
        assert_eq!(g.tab_stop(), Some(0));
        assert_eq!(g.key(NavKey::Next, ClockMs(0)).as_deref(), Some("c"));
        assert_eq!(g.prev, Some(0));
        assert_eq!(
            g.key(NavKey::Next, ClockMs(1)).as_deref(),
            Some("a"),
            "wraps"
        );
        assert_eq!(g.pick(1, ClockMs(2)), None);
        assert_eq!(g.key(NavKey::Last, ClockMs(3)).as_deref(), Some("c"));
    }
}
