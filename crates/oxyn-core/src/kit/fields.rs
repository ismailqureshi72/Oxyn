//! Number entry and choice (plan #28, chunk 4b): the behaviour behind the number field, the
//! Figma-style scrub input, the rubber slider and the unfold select.
//!
//! - [`NumberField`]: a bounded number with steppers, hold-to-repeat that speeds up, a typed
//!   draft that may pass a limit (and springs back on commit), a strain toward a refused press
//!   and a "Max 10 seats" note for about 1.5 s.
//! - [`eval`] / [`parse_value`] / [`Scrub`]: typed arithmetic with units, and dragging a label
//!   sideways to change a value with fine and coarse steps, wrapping and rubber-band resistance.
//! - [`rubber`] / [`detent`] / [`tag_tilt`]: the rubber slider's stretch past its ends, the
//!   detents a dragged thumb clicks into, and the swaying value tag.
//! - [`UnfoldSelect`]: a select with a gliding highlight, type-ahead, paging and an automatic
//!   search field above eight options.
//!
//! Behaviour-only rebuilds of the Arc UI Number field, Scrub input, Elastic slider and Morph
//! select; nothing is copied. Time is passed in as [`ClockMs`].

use crate::ClockMs;

use super::text::group;

/// The limit a value met.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Edge {
    /// The lower bound.
    Min,
    /// The upper bound.
    Max,
}

/// Decimal places of a step (`0.25` → 2, `5` → 0).
pub fn decimals_of(step: f64) -> usize {
    let s = format!("{step}");
    s.split_once('.').map_or(0, |(_, f)| f.len().min(6))
}

/// Rounds to `places` decimals.
pub fn round_to(v: f64, places: usize) -> f64 {
    let m = 10f64.powi(places as i32);
    (v * m).round() / m
}

/// `1,234.50` with grouping, fixed decimals.
pub fn format_number(v: f64, places: usize, grouping: bool) -> String {
    let s = format!("{:.*}", places, v.abs());
    let (int, frac) = s.split_once('.').map_or((s.as_str(), ""), |(a, b)| (a, b));
    let int = if grouping {
        group(int.parse::<i64>().unwrap_or(0))
    } else {
        int.to_string()
    };
    let sign = if v < 0.0 && v.abs() >= 0.5 * 10f64.powi(-(places as i32)) {
        "-"
    } else {
        ""
    };
    if frac.is_empty() {
        format!("{sign}{int}")
    } else {
        format!("{sign}{int}.{frac}")
    }
}

/// Steps taken by a stepper held for `held_ms`: one on press, then after 400 ms a repeat that
/// ramps from 150 ms to 40 ms per step.
pub fn held_steps(held_ms: u64) -> u32 {
    if held_ms < 400 {
        return 1;
    }
    let mut t = 400u64;
    let mut n = 1u32;
    let mut gap = 150f64;
    while t + gap as u64 <= held_ms {
        t += gap as u64;
        n += 1;
        gap = (gap * 0.82).max(40.0);
        if n > 10_000 {
            break;
        }
    }
    n
}

/// A bounded number with steppers (Arc Number field behaviour).
#[derive(Clone, Debug, PartialEq)]
pub struct NumberField {
    /// Committed value.
    pub value: f64,
    /// Lower bound.
    pub min: f64,
    /// Upper bound.
    pub max: f64,
    /// Increment.
    pub step: f64,
    /// PageUp/PageDown and Shift+arrow distance (default `step * 10`).
    pub large_step: Option<f64>,
    /// Text being typed, before Enter or blur commits it.
    pub draft: Option<String>,
    /// Refused presses in a row at a limit (the strain grows with them).
    pub pushes: u32,
    /// The last refused press: which edge and when (drives the note and the shake).
    pub refused: Option<(Edge, ClockMs)>,
}

impl NumberField {
    /// A field at `value` between `min` and `max`.
    pub fn new(value: f64, min: f64, max: f64, step: f64) -> Self {
        Self {
            value: value.clamp(min, max),
            min,
            max,
            step,
            large_step: None,
            draft: None,
            pushes: 0,
            refused: None,
        }
    }

    /// Decimal places shown (from the step).
    pub fn places(&self) -> usize {
        decimals_of(self.step)
    }

    fn large(&self) -> f64 {
        self.large_step.unwrap_or(self.step * 10.0)
    }

    /// Steps by `dir` (+1 or −1), `large` for PageUp/Shift. At a limit the press is refused:
    /// the value stays, `pushes` grows and the edge comes back.
    pub fn step_by(&mut self, dir: i32, large: bool, now: ClockMs) -> Result<f64, Edge> {
        self.draft = None;
        let d = if large { self.large() } else { self.step };
        let edge = if dir > 0 { Edge::Max } else { Edge::Min };
        let at = match edge {
            Edge::Max => self.value >= self.max,
            Edge::Min => self.value <= self.min,
        };
        if at {
            self.pushes = (self.pushes + 1).min(6);
            self.refused = Some((edge, now));
            return Err(edge);
        }
        self.pushes = 0;
        self.value = round_to(
            (self.value + d * f64::from(dir.signum())).clamp(self.min, self.max),
            self.places(),
        );
        Ok(self.value)
    }

    /// Home / End: jumps to a bound.
    pub fn jump(&mut self, edge: Edge) -> f64 {
        self.draft = None;
        self.value = match edge {
            Edge::Min => self.min,
            Edge::Max => self.max,
        };
        self.value
    }

    /// Typing: keeps the draft as text.
    pub fn type_draft(&mut self, text: &str) {
        self.draft = Some(text.to_string());
    }

    /// The typed draft as a number, ignoring grouping commas and affixes.
    pub fn draft_value(&self) -> Option<f64> {
        let d = self.draft.as_ref()?;
        let clean: String = d
            .chars()
            .filter(|c| c.is_ascii_digit() || *c == '.' || *c == '-')
            .collect();
        clean.parse().ok()
    }

    /// The edge a typed draft has passed (the shell tints until it commits).
    pub fn draft_over(&self) -> Option<Edge> {
        let v = self.draft_value()?;
        if v > self.max {
            Some(Edge::Max)
        } else if v < self.min {
            Some(Edge::Min)
        } else {
            None
        }
    }

    /// Enter or blur: commits the draft, clamped and rounded. An unparseable draft reverts.
    pub fn commit(&mut self) -> f64 {
        if let Some(v) = self.draft_value() {
            self.value = round_to(v.clamp(self.min, self.max), self.places());
        }
        self.draft = None;
        self.value
    }

    /// Escape: drops the draft.
    pub fn revert(&mut self) {
        self.draft = None;
    }

    /// Pixels the value strains toward a refused press (grows with pushes in a row, capped).
    pub fn strain_px(&self, now: ClockMs) -> f32 {
        match self.refused {
            Some((_, at)) if now.since(at).as_millis() < 260 => (2.0 + self.pushes as f32).min(8.0),
            _ => 0.0,
        }
    }

    /// The note beside the label for about 1.5 s after a refused press: "Max 10 seats".
    pub fn limit_note(&self, now: ClockMs, prefix: &str, suffix: &str) -> Option<String> {
        let (edge, at) = self.refused?;
        if now.since(at).as_millis() >= 1500 {
            return None;
        }
        let (w, v) = match edge {
            Edge::Min => ("Min", self.min),
            Edge::Max => ("Max", self.max),
        };
        Some(format!(
            "{w} {prefix}{}{suffix}",
            format_number(v, self.places(), true)
        ))
    }

    /// Whether a stepper can still move that way.
    pub fn can(&self, dir: i32) -> bool {
        if dir > 0 {
            self.value < self.max
        } else {
            self.value > self.min
        }
    }
}

/// Plural-aware suffix: `" seat"` for 1, `" seats"` otherwise.
pub fn plural(n: f64, one: &str, many: &str) -> String {
    if (n - 1.0).abs() < f64::EPSILON {
        one.to_string()
    } else {
        many.to_string()
    }
}

/// Evaluates arithmetic typed into a number field: `+ - * /`, parentheses, unary minus and
/// decimals. `None` for anything else.
pub fn eval(expr: &str) -> Option<f64> {
    let toks: Vec<char> = expr.chars().filter(|c| !c.is_whitespace()).collect();
    if toks.is_empty() {
        return None;
    }
    let mut p = 0usize;
    let v = sum(&toks, &mut p)?;
    (p == toks.len() && v.is_finite()).then_some(v)
}

fn sum(t: &[char], p: &mut usize) -> Option<f64> {
    let mut v = product(t, p)?;
    while *p < t.len() && (t[*p] == '+' || t[*p] == '-') {
        let op = t[*p];
        *p += 1;
        let r = product(t, p)?;
        v = if op == '+' { v + r } else { v - r };
    }
    Some(v)
}

fn product(t: &[char], p: &mut usize) -> Option<f64> {
    let mut v = unary(t, p)?;
    while *p < t.len() && (t[*p] == '*' || t[*p] == '/' || t[*p] == '×') {
        let op = t[*p];
        *p += 1;
        let r = unary(t, p)?;
        v = if op == '/' { v / r } else { v * r };
    }
    Some(v)
}

fn unary(t: &[char], p: &mut usize) -> Option<f64> {
    if *p < t.len() && t[*p] == '-' {
        *p += 1;
        return unary(t, p).map(|v| -v);
    }
    if *p < t.len() && t[*p] == '+' {
        *p += 1;
        return unary(t, p);
    }
    if *p < t.len() && t[*p] == '(' {
        *p += 1;
        let v = sum(t, p)?;
        if *p < t.len() && t[*p] == ')' {
            *p += 1;
            return Some(v);
        }
        return None;
    }
    let start = *p;
    while *p < t.len() && (t[*p].is_ascii_digit() || t[*p] == '.') {
        *p += 1;
    }
    t[start..*p].iter().collect::<String>().parse().ok()
}

/// Parses typed text with an optional trailing unit from `units` ("12px", "50 %", "12*2").
/// Returns the value and the unit when one was typed.
pub fn parse_value(text: &str, units: &[&str]) -> Option<(f64, Option<String>)> {
    let t = text.trim();
    let mut sorted: Vec<&&str> = units.iter().collect();
    sorted.sort_by_key(|u| std::cmp::Reverse(u.len()));
    for u in sorted {
        if !u.is_empty()
            && let Some(head) = t.strip_suffix(*u)
        {
            return eval(head).map(|v| (v, Some((*u).to_string())));
        }
    }
    eval(t).map(|v| (v, None))
}

/// Step size held while scrubbing or pressing an arrow.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Gear {
    /// Alt: ten times finer.
    Fine,
    /// No modifier.
    Normal,
    /// Shift: ten times coarser, snapped to the coarse grid.
    Coarse,
}

impl Gear {
    /// From the modifier keys.
    pub fn from_mods(shift: bool, alt: bool) -> Self {
        if shift {
            Gear::Coarse
        } else if alt {
            Gear::Fine
        } else {
            Gear::Normal
        }
    }
    /// The multiplier shown while dragging ("×10", "×0.1").
    pub fn badge(self) -> Option<&'static str> {
        match self {
            Gear::Fine => Some("×0.1"),
            Gear::Normal => None,
            Gear::Coarse => Some("×10"),
        }
    }
}

/// iOS-style rubber band: how far a surface follows a pull of `over` px, never more than `max`.
pub fn rubber(over: f32, max: f32) -> f32 {
    if max <= 0.0 {
        return 0.0;
    }
    let s = over.signum();
    let o = over.abs();
    s * max * (1.0 - 1.0 / (o / max * 0.55 + 1.0))
}

/// A Figma-style scrub input (Arc Scrub input behaviour).
#[derive(Clone, Debug, PartialEq)]
pub struct Scrub {
    /// Value.
    pub value: f64,
    /// Lower limit (`f64::NEG_INFINITY` for none).
    pub min: f64,
    /// Upper limit.
    pub max: f64,
    /// One step.
    pub step: f64,
    /// Pointer travel for one step.
    pub px_per_step: f32,
    /// Wrap from max to min (rotation). Needs both limits.
    pub wrap: bool,
    /// Decimal places kept.
    pub precision: usize,
    /// A drag in progress: the value and pointer x it started at, and the gear.
    pub drag: Option<(f64, f32, Gear)>,
    /// Pixels past a limit during the drag (the field strains).
    pub over_px: f32,
    /// Value when the drag began (Escape returns here, even after a gear change).
    pub origin: f64,
    /// Last change was refused at a limit (the field kicks).
    pub kicked: Option<ClockMs>,
}

impl Scrub {
    /// A scrub input at `value`.
    pub fn new(value: f64, min: f64, max: f64, step: f64) -> Self {
        let precision = decimals_of(step / 10.0).min(decimals_of(step) + 1);
        Self {
            value,
            min,
            max,
            step,
            px_per_step: 2.0,
            wrap: false,
            precision: precision.max(decimals_of(step)),
            drag: None,
            over_px: 0.0,
            origin: value,
            kicked: None,
        }
    }

    /// Both limits are finite (the quiet track fill shows).
    pub fn bounded(&self) -> bool {
        self.min.is_finite() && self.max.is_finite()
    }

    /// Where the value sits between the limits.
    pub fn fraction(&self) -> f32 {
        if !self.bounded() || self.max <= self.min {
            return 0.0;
        }
        ((self.value - self.min) / (self.max - self.min)).clamp(0.0, 1.0) as f32
    }

    fn step_for(&self, g: Gear) -> f64 {
        match g {
            Gear::Fine => self.step / 10.0,
            Gear::Normal => self.step,
            Gear::Coarse => self.step * 10.0,
        }
    }

    fn settle(&self, v: f64, g: Gear) -> (f64, f32) {
        let mut v = v;
        if g == Gear::Coarse {
            let c = self.step_for(g);
            v = (v / c).round() * c;
        }
        if self.wrap && self.bounded() {
            let span = self.max - self.min;
            v = self.min + (v - self.min).rem_euclid(span);
            return (round_to(v, self.precision), 0.0);
        }
        let over = if v > self.max {
            ((v - self.max) / self.step_for(g)) as f32 * self.px_per_step
        } else if v < self.min {
            ((v - self.min) / self.step_for(g)) as f32 * self.px_per_step
        } else {
            0.0
        };
        (round_to(v.clamp(self.min, self.max), self.precision), over)
    }

    /// Pointer down on the label at `x`.
    pub fn begin(&mut self, x: f32, g: Gear) {
        self.drag = Some((self.value, x, g));
        self.origin = self.value;
        self.over_px = 0.0;
    }

    /// Pointer moved to `x` with `g` held. Changing gear mid-drag re-anchors so the value does
    /// not jump. Returns the new value.
    pub fn drag_to(&mut self, x: f32, g: Gear) -> f64 {
        let Some((v0, x0, g0)) = self.drag else {
            return self.value;
        };
        if g != g0 {
            self.drag = Some((self.value, x, g));
            return self.value;
        }
        let steps = ((x - x0) / self.px_per_step).trunc() as f64;
        let (v, over) = self.settle(v0 + steps * self.step_for(g), g);
        self.value = v;
        self.over_px = rubber(over, 12.0);
        v
    }

    /// Pointer up: the strain springs back.
    pub fn end(&mut self) -> f64 {
        self.drag = None;
        self.over_px = 0.0;
        self.value
    }

    /// Escape during a drag: back to where it started.
    pub fn cancel(&mut self) -> f64 {
        if self.drag.take().is_some() {
            self.value = self.origin;
        }
        self.over_px = 0.0;
        self.value
    }

    /// Arrow key: one step in the gear. Refused at a limit (kicks).
    pub fn key(&mut self, dir: i32, g: Gear, now: ClockMs) -> f64 {
        let want = self.value + self.step_for(g) * f64::from(dir.signum());
        let (v, over) = self.settle(want, g);
        if over != 0.0 || (v == self.value && !self.wrap) {
            self.kicked = Some(now);
        }
        self.value = v;
        v
    }

    /// Typed text committed with Enter or blur. `None` (and a kick) when it does not parse.
    pub fn commit_text(
        &mut self,
        text: &str,
        units: &[&str],
        now: ClockMs,
    ) -> Option<(f64, Option<String>)> {
        match parse_value(text, units) {
            Some((v, u)) => {
                let (v, _) = self.settle(v, Gear::Normal);
                self.value = v;
                Some((v, u))
            }
            None => {
                self.kicked = Some(now);
                None
            }
        }
    }

    /// Whether the field is mid-kick.
    pub fn kicking(&self, now: ClockMs) -> bool {
        self.kicked.is_some_and(|k| now.since(k).as_millis() < 320)
    }

    /// The value as text.
    pub fn text(&self) -> String {
        let s = format!("{:.*}", self.precision, self.value);
        if s.contains('.') {
            s.trim_end_matches('0').trim_end_matches('.').to_string()
        } else {
            s
        }
    }
}

/// The next unit in a cycle (the unit button).
pub fn next_unit(units: &[&str], current: &str) -> String {
    let i = units.iter().position(|u| *u == current).unwrap_or(0);
    units
        .get((i + 1) % units.len().max(1))
        .map_or(String::new(), |u| (*u).to_string())
}

/// Rubber slider detents: where a dragged thumb at `px` lands, given the detent positions in px.
/// Within `hold` px of a detent it clicks in and stays; returns the landing px and the detent hit.
pub fn detent(px: f32, detents: &[f32], hold: f32) -> (f32, Option<usize>) {
    for (i, d) in detents.iter().enumerate() {
        if (px - d).abs() <= hold {
            return (*d, Some(i));
        }
    }
    (px, None)
}

/// The value tag's tilt in degrees for a thumb moving at `vx` px per frame (sways and settles).
pub fn tag_tilt(vx: f32) -> f32 {
    (vx * 1.4).clamp(-14.0, 14.0)
}

/// The elastic stretch of a track pulled `over` px past an end: (track px stretch, icon push).
pub fn stretch(over: f32) -> (f32, f32) {
    let s = rubber(over, 22.0);
    (s, s * 0.6)
}

/// How a unfold select opens.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OpenAt {
    /// On the selected option (or the first).
    Selected,
    /// On the first enabled option.
    First,
    /// On the last enabled option.
    Last,
}

/// One option of a unfold select.
#[derive(Clone, Debug, PartialEq)]
pub struct Choice {
    /// Value.
    pub value: String,
    /// Label.
    pub label: String,
    /// Quiet second line.
    pub hint: String,
    /// Not choosable.
    pub disabled: bool,
}

impl Choice {
    /// An enabled option.
    pub fn new(value: &str, label: &str) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
            hint: String::new(),
            disabled: false,
        }
    }
    /// With a second line.
    pub fn hint(mut self, h: &str) -> Self {
        self.hint = h.into();
        self
    }
    /// Disabled.
    pub fn off(mut self) -> Self {
        self.disabled = true;
        self
    }
}

/// A select whose trigger grows into its list (Arc Morph select behaviour).
#[derive(Clone, Debug, PartialEq)]
pub struct UnfoldSelect {
    /// Options in order.
    pub options: Vec<Choice>,
    /// Selected index.
    pub selected: Option<usize>,
    /// Open.
    pub open: bool,
    /// Highlighted option (index into `options`).
    pub active: usize,
    /// Search text (search mode).
    pub query: String,
    /// Type-ahead buffer and when it was last typed.
    pub typed: (String, ClockMs),
    /// Direction the trigger value last rolled: +1 later option, −1 earlier.
    pub roll: i32,
}

impl UnfoldSelect {
    /// A closed select.
    pub fn new(options: Vec<Choice>, selected: Option<usize>) -> Self {
        Self {
            options,
            selected,
            open: false,
            active: selected.unwrap_or(0),
            query: String::new(),
            typed: (String::new(), ClockMs(0)),
            roll: 0,
        }
    }

    /// Search shows in place of the trigger: `"auto"` turns on above eight options.
    pub fn searchable(&self, mode: &str) -> bool {
        match mode {
            "on" | "true" => true,
            "off" | "false" => false,
            _ => self.options.len() > 8,
        }
    }

    /// Options matching the search, in order.
    pub fn visible(&self) -> Vec<usize> {
        let q = self.query.to_lowercase();
        (0..self.options.len())
            .filter(|i| q.is_empty() || self.options[*i].label.to_lowercase().contains(&q))
            .collect()
    }

    fn enabled(&self) -> Vec<usize> {
        self.visible()
            .into_iter()
            .filter(|i| !self.options[*i].disabled)
            .collect()
    }

    /// Opens on the selected, first or last option.
    pub fn open_at(&mut self, at: OpenAt) {
        self.open = true;
        self.query.clear();
        let en = self.enabled();
        self.active = match at {
            OpenAt::Selected => self
                .selected
                .filter(|s| en.contains(s))
                .or(en.first().copied()),
            OpenAt::First => en.first().copied(),
            OpenAt::Last => en.last().copied(),
        }
        .unwrap_or(0);
    }

    /// Closes without picking.
    pub fn close(&mut self) {
        self.open = false;
        self.query.clear();
    }

    /// Moves the highlight `delta` enabled options (PageDown is 8), clamped at the ends.
    pub fn move_by(&mut self, delta: i32) {
        let en = self.enabled();
        if en.is_empty() {
            return;
        }
        let pos = en.iter().position(|i| *i == self.active).unwrap_or(0) as i32;
        let n = (pos + delta).clamp(0, en.len() as i32 - 1);
        self.active = en[n as usize];
    }

    /// Search text changed: the highlight goes to the first match.
    pub fn search(&mut self, q: &str) {
        self.query = q.to_string();
        if let Some(f) = self.enabled().first() {
            self.active = *f;
        }
    }

    /// Type-ahead: letters typed within 700 ms of each other build a prefix; the highlight jumps
    /// to the next label that starts with it (one repeated letter cycles).
    pub fn type_ahead(&mut self, ch: char, now: ClockMs) {
        let (buf, at) = &self.typed;
        let mut b = if now.since(*at).as_millis() < 700 {
            buf.clone()
        } else {
            String::new()
        };
        b.push(ch.to_ascii_lowercase());
        let first = b.chars().next();
        let cycle = b.len() > 1 && b.chars().all(|c| Some(c) == first);
        let prefix = if cycle { b[..1].to_string() } else { b.clone() };
        let en = self.enabled();
        let start = en.iter().position(|i| *i == self.active).unwrap_or(0);
        let skip = usize::from(cycle || b.len() == 1);
        let hit = (0..en.len())
            .map(|k| en[(start + skip + k) % en.len()])
            .find(|i| self.options[*i].label.to_lowercase().starts_with(&prefix));
        if let Some(h) = hit {
            self.active = h;
            if !self.open {
                self.choose(h);
            }
        }
        self.typed = (b, now);
    }

    /// Picks an option: the roll direction follows its place relative to the old one.
    pub fn choose(&mut self, i: usize) -> Option<&Choice> {
        if self.options.get(i).is_none_or(|o| o.disabled) {
            return None;
        }
        self.roll = match self.selected {
            Some(s) if i > s => 1,
            Some(s) if i < s => -1,
            _ => 0,
        };
        self.selected = Some(i);
        self.active = i;
        self.open = false;
        self.query.clear();
        self.options.get(i)
    }

    /// Enter / Space / Alt+Up while open: picks the highlight.
    pub fn pick_active(&mut self) -> Option<&Choice> {
        self.choose(self.active)
    }

    /// The trigger label.
    pub fn label(&self, placeholder: &str) -> String {
        self.selected
            .and_then(|i| self.options.get(i))
            .map_or(placeholder.to_string(), |o| o.label.clone())
    }

    /// Position of the highlight among visible rows (the glide target).
    pub fn active_row(&self) -> Option<usize> {
        self.visible().iter().position(|i| *i == self.active)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_field_steps_refuses_and_commits() {
        let mut f = NumberField::new(9.0, 1.0, 10.0, 1.0);
        assert_eq!(f.step_by(1, false, ClockMs(0)), Ok(10.0));
        assert_eq!(f.step_by(1, false, ClockMs(10)), Err(Edge::Max));
        assert_eq!(f.step_by(1, false, ClockMs(20)), Err(Edge::Max));
        assert_eq!(f.pushes, 2);
        assert_eq!(
            f.limit_note(ClockMs(100), "", " seats").as_deref(),
            Some("Max 10 seats")
        );
        assert_eq!(f.limit_note(ClockMs(2000), "", ""), None);
        assert!(f.strain_px(ClockMs(30)) > 0.0);
        f.type_draft("1,200");
        assert_eq!(f.draft_over(), Some(Edge::Max));
        assert_eq!(f.commit(), 10.0);
        f.type_draft("abc");
        assert_eq!(f.commit(), 10.0);
        assert_eq!(f.jump(Edge::Min), 1.0);
        let mut m = NumberField::new(0.25, 0.0, 1.0, 0.05);
        m.step_by(1, false, ClockMs(0)).ok();
        assert_eq!(m.value, 0.3);
        assert_eq!(format_number(1234.5, 2, true), "1,234.50");
    }

    #[test]
    fn hold_repeat_ramps() {
        assert_eq!(held_steps(0), 1);
        assert_eq!(held_steps(399), 1);
        assert_eq!(held_steps(550), 2);
        assert!(held_steps(2000) > held_steps(1000) + 10);
    }

    #[test]
    fn arithmetic_and_units() {
        assert_eq!(eval("12*2"), Some(24.0));
        assert_eq!(eval("(1+2)*-3"), Some(-9.0));
        assert_eq!(eval("2+"), None);
        assert_eq!(
            parse_value("12px", &["px", "%"]),
            Some((12.0, Some("px".into())))
        );
        assert_eq!(parse_value(" 50 ", &["px"]), Some((50.0, None)));
        assert_eq!(next_unit(&["px", "%", "em"], "em"), "px");
    }

    #[test]
    fn scrub_gears_wrap_and_rubber() {
        let mut s = Scrub::new(10.0, 0.0, 100.0, 1.0);
        s.begin(0.0, Gear::Normal);
        assert_eq!(s.drag_to(20.0, Gear::Normal), 20.0);
        s.drag_to(20.0, Gear::Fine);
        assert_eq!(s.drag_to(40.0, Gear::Fine), 21.0);
        assert_eq!(s.cancel(), 10.0);
        s.begin(0.0, Gear::Normal);
        s.drag_to(400.0, Gear::Normal);
        assert_eq!(s.value, 100.0);
        assert!(s.over_px > 0.0 && s.over_px <= 12.0);
        s.end();
        assert_eq!(s.over_px, 0.0);
        let mut r = Scrub::new(350.0, 0.0, 360.0, 1.0);
        r.wrap = true;
        r.key(1, Gear::Coarse, ClockMs(0));
        assert_eq!(r.value, 0.0);
        assert!(s.commit_text("nope", &[], ClockMs(5)).is_none());
        assert!(s.kicking(ClockMs(10)));
        assert!(rubber(1000.0, 12.0) < 12.0);
    }

    #[test]
    fn detents_hold() {
        assert_eq!(detent(103.0, &[100.0, 200.0], 7.0), (100.0, Some(0)));
        assert_eq!(detent(150.0, &[100.0, 200.0], 7.0), (150.0, None));
        assert!(tag_tilt(100.0) <= 14.0);
    }

    #[test]
    fn morph_select_keys() {
        let opts = vec![
            Choice::new("a", "Apple"),
            Choice::new("b", "Banana").off(),
            Choice::new("c", "Cherry"),
            Choice::new("d", "Date"),
        ];
        let mut m = UnfoldSelect::new(opts, Some(2));
        m.open_at(OpenAt::Selected);
        assert_eq!(m.active, 2);
        m.move_by(-1);
        assert_eq!(m.active, 0, "skips the disabled option");
        m.move_by(8);
        assert_eq!(m.active, 3);
        m.type_ahead('c', ClockMs(0));
        assert_eq!(m.active, 2);
        assert_eq!(m.pick_active().map(|c| c.value.clone()), Some("c".into()));
        assert_eq!(m.roll, 0);
        m.choose(0);
        assert_eq!(m.roll, -1);
        assert!(!m.searchable("auto"));
        m.search("ch");
        assert_eq!(m.visible(), vec![2]);
    }
}
