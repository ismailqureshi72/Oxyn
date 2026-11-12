//! Context meter model (`docs/specs/composer-context-meter.md`): usage by category, levels with
//! hysteresis, estimate vs exact, compaction, the digit roll, breakdown rows and the cell grid.

use std::time::Duration;

use oxyn_tokens::ColorRole;

use crate::{ClockMs, text::format_tokens};

/// Context meter variants (`composer-context-meter.md` §6, free set).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ContextVariant {
    /// 16 px ring in the composer row.
    #[default]
    Ring,
    /// Ring split into category arcs.
    RingSegmented,
    /// Thin bar with a `56%` label.
    Bar,
    /// Cell grid (`/context` map).
    Grid,
    /// Footer text, `44% context left`.
    Text,
    /// Session-rail card with breakdown and actions.
    Card,
}

impl ContextVariant {
    /// Every free variant.
    pub const ALL: [ContextVariant; 6] = [
        Self::Ring,
        Self::RingSegmented,
        Self::Bar,
        Self::Grid,
        Self::Text,
        Self::Card,
    ];

    /// Short id; registry id is `context-meter/<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Ring => "ring",
            Self::RingSegmented => "ring-segmented",
            Self::Bar => "bar",
            Self::Grid => "grid",
            Self::Text => "text",
            Self::Card => "card",
        }
    }

    /// Parses `bar` or `context-meter/bar`.
    pub fn from_id(id: &str) -> Option<Self> {
        let id = id.strip_prefix("context-meter/").unwrap_or(id);
        Self::ALL.into_iter().find(|v| v.id() == id)
    }
}

/// What fills the context window, in fixed order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ContextCategory {
    /// System prompt.
    System,
    /// Tool definitions (MCP servers).
    Tools,
    /// Memory and rules files.
    Memory,
    /// Skills.
    Skills,
    /// Files and attachments.
    Files,
    /// Conversation history.
    History,
    /// Reasoning carried between turns.
    Reasoning,
    /// Reserved output and autocompact buffer (drawn hatched).
    Reserved,
    /// Rows under 0.5 % folded together.
    Other,
    /// Free space.
    Free,
}

impl ContextCategory {
    /// Row label.
    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Tools => "Tools",
            Self::Memory => "Memory & rules",
            Self::Skills => "Skills",
            Self::Files => "Files & attachments",
            Self::History => "History",
            Self::Reasoning => "Reasoning carried",
            Self::Reserved => "Reserved",
            Self::Other => "Other",
            Self::Free => "Free",
        }
    }

    /// Swatch colour. Uses the agent palette until dedicated `ctx.*` tokens land in the token
    /// rename pass.
    pub fn color(self) -> ColorRole {
        match self {
            Self::System => ColorRole::AgentBlue,
            Self::Tools => ColorRole::AgentViolet,
            Self::Memory => ColorRole::AgentCyan,
            Self::Skills => ColorRole::AgentPink,
            Self::Files => ColorRole::AgentGreen,
            Self::History => ColorRole::AgentOrange,
            Self::Reasoning => ColorRole::AgentYellow,
            Self::Reserved => ColorRole::FgSubtle,
            Self::Other => ColorRole::FgMuted,
            Self::Free => ColorRole::LoaderTrack,
        }
    }
}

/// Tokens in one category.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CategoryUsage {
    /// Category.
    pub category: ContextCategory,
    /// Tokens.
    pub tokens: u32,
}

/// The host's usage report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ContextUsage {
    /// Window size; `None` when the provider doesn't say.
    pub window: Option<u32>,
    /// Usage by category (Reserved included).
    pub categories: Vec<CategoryUsage>,
    /// Exact provider count (false while estimating mid-stream).
    pub exact: bool,
    /// When the report was made.
    pub at: ClockMs,
}

impl ContextUsage {
    /// Total tokens used, reserved buffer included.
    pub fn used(&self) -> u32 {
        self.categories.iter().map(|c| c.tokens).sum()
    }

    /// `used / window`.
    pub fn ratio(&self) -> Option<f32> {
        self.window.map(|w| self.used() as f32 / w.max(1) as f32)
    }

    /// Breakdown rows: categories in order, rows under 0.5 % folded into Other, then Free.
    pub fn rows(&self) -> Vec<(ContextCategory, u32, f32)> {
        let Some(w) = self.window else {
            return Vec::new();
        };
        let w = w.max(1);
        let mut rows: Vec<_> = Vec::new();
        let mut other = 0u32;
        let mut cats = self.categories.clone();
        cats.sort_by_key(|c| c.category);
        for c in cats {
            if c.tokens == 0 {
                continue;
            }
            if (c.tokens as f32 / w as f32) < 0.005 && c.category != ContextCategory::Reserved {
                other += c.tokens;
            } else {
                rows.push((c.category, c.tokens, c.tokens as f32 / w as f32));
            }
        }
        if other > 0 {
            rows.push((ContextCategory::Other, other, other as f32 / w as f32));
        }
        let free = w.saturating_sub(self.used());
        rows.push((ContextCategory::Free, free, free as f32 / w as f32));
        rows
    }

    /// Grid cells (row-major), each the category that fills it. Cells are assigned in category
    /// order with largest-remainder rounding; the rest are Free.
    pub fn grid(&self, cols: usize, rows: usize) -> Vec<ContextCategory> {
        let n = cols * rows;
        let parts: Vec<_> = self
            .rows()
            .into_iter()
            .filter(|r| r.0 != ContextCategory::Free)
            .collect();
        let mut counts: Vec<(ContextCategory, usize, f32)> = parts
            .iter()
            .map(|(c, _, p)| {
                let exact = p * n as f32;
                (*c, exact.floor() as usize, exact.fract())
            })
            .collect();
        let used_exact: f32 = parts
            .iter()
            .map(|p| p.2 * n as f32)
            .sum::<f32>()
            .min(n as f32);
        let mut left =
            (used_exact.round() as usize).saturating_sub(counts.iter().map(|c| c.1).sum());
        let mut order: Vec<usize> = (0..counts.len()).collect();
        order.sort_by(|a, b| counts[*b].2.total_cmp(&counts[*a].2));
        for i in order {
            if left == 0 {
                break;
            }
            counts[i].1 += 1;
            left -= 1;
        }
        let mut cells = Vec::with_capacity(n);
        for (c, k, _) in counts {
            cells.extend(std::iter::repeat_n(c, k));
        }
        cells.truncate(n);
        cells.resize(n, ContextCategory::Free);
        cells
    }
}

/// Level thresholds (proposed defaults).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Thresholds {
    /// Amber from here.
    pub warn_at: f32,
    /// Red from here; the ring breathes.
    pub critical_at: f32,
    /// The compact prompt appears from here.
    pub compact_prompt_at: f32,
    /// A level drops only below its threshold minus this.
    pub hysteresis: f32,
}

impl Default for Thresholds {
    fn default() -> Self {
        Self {
            warn_at: 0.70,
            critical_at: 0.85,
            compact_prompt_at: 0.90,
            hysteresis: 0.03,
        }
    }
}

/// How full the window is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum ContextLevel {
    /// No window size reported.
    #[default]
    Unknown,
    /// Neutral fill.
    Normal,
    /// Amber.
    Warning,
    /// Red, breathing.
    Critical,
    /// Over 100 %: the next request will fail.
    Overflow,
}

impl ContextLevel {
    /// Fill colour.
    pub fn color(self) -> ColorRole {
        match self {
            Self::Unknown | Self::Normal => ColorRole::FgMuted,
            Self::Warning => ColorRole::StatusWarningFg,
            Self::Critical | Self::Overflow => ColorRole::StatusDangerFg,
        }
    }

    fn from_ratio(r: f32, t: &Thresholds, prev: Self) -> Self {
        if r > 1.0 {
            return Self::Overflow;
        }
        let raw = if r >= t.critical_at {
            Self::Critical
        } else if r >= t.warn_at {
            Self::Warning
        } else {
            Self::Normal
        };
        if raw >= prev || prev == Self::Unknown || prev == Self::Overflow {
            return raw;
        }
        // Dropping: only below the previous level's threshold minus hysteresis.
        let floor = match prev {
            Self::Critical => t.critical_at,
            Self::Warning => t.warn_at,
            _ => 0.0,
        };
        if r < floor - t.hysteresis { raw } else { prev }
    }
}

/// How the label reads.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum LabelMode {
    /// No label.
    None,
    /// `41%`.
    #[default]
    Percent,
    /// `82K / 200K`.
    Tokens,
    /// `59% context left`.
    Remaining,
}

/// Compaction, driven by the host's events.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Compaction {
    /// Nothing happening.
    #[default]
    Idle,
    /// Compacting now.
    Running,
    /// Done: `delta` tokens freed at `at`.
    Done {
        /// Tokens freed.
        delta: u32,
        /// When.
        at: ClockMs,
    },
    /// Failed.
    Failed(String),
}

/// Digit roll and fill easing duration.
pub const ROLL: Duration = Duration::from_millis(320);
/// Critical breathing period.
pub const BREATHE: Duration = Duration::from_millis(2400);
/// The `−84K` float.
pub const FLOAT: Duration = Duration::from_millis(1200);
/// A report older than this is drawn stale.
pub const STALE_AFTER: Duration = Duration::from_secs(120);

/// What the meter draws this frame.
#[derive(Clone, Debug, PartialEq)]
pub struct ContextView {
    /// Level (colour).
    pub level: ContextLevel,
    /// Fill fraction, eased (`0..=1`, capped).
    pub fill: f32,
    /// Tokens shown, rolling toward the report.
    pub used: u32,
    /// Window.
    pub window: Option<u32>,
    /// Count is an estimate (`≈`).
    pub estimate: bool,
    /// Opacity multiplier for the critical breathe.
    pub alpha: f32,
    /// Compact prompt text, when it should show.
    pub prompt: Option<String>,
    /// Compaction is running (indeterminate sweep).
    pub compacting: bool,
    /// `−84K` float: text and progress.
    pub float: Option<(String, f32)>,
    /// The report is stale (50 % fill opacity).
    pub stale: bool,
}

impl ContextView {
    /// Label per mode.
    pub fn label(&self, mode: LabelMode) -> String {
        let pre = if self.estimate { "≈ " } else { "" };
        let Some(w) = self.window else {
            return String::new();
        };
        let pct = (self.used as f32 / w.max(1) as f32 * 100.0).round() as u32;
        match mode {
            LabelMode::None => String::new(),
            LabelMode::Percent if self.level == ContextLevel::Overflow => {
                format!("Over by {}", k(self.used.saturating_sub(w)))
            }
            LabelMode::Percent => format!("{pre}{pct}%"),
            LabelMode::Tokens => format!("{pre}{} / {}", k(self.used), k(w)),
            LabelMode::Remaining => format!("{pre}{}% context left", 100u32.saturating_sub(pct)),
        }
    }

    /// Tooltip: `82.4K of 200K tokens · 41% · 118K free`.
    pub fn tooltip(&self) -> String {
        match self.window {
            None => "Context size not reported".into(),
            Some(w) => format!(
                "{}{} of {} tokens · {}% · {} free",
                if self.estimate { "≈ " } else { "" },
                k(self.used),
                k(w),
                (self.used as f32 / w.max(1) as f32 * 100.0).round() as u32,
                k(w.saturating_sub(self.used))
            ),
        }
    }
}

/// Tokens in the meter's `K` style (`82.4K`, `200K`, `1.2M`).
pub fn k(n: u32) -> String {
    format_tokens(u64::from(n), false).to_uppercase()
}

/// The meter's state: level with hysteresis, the roll, snooze and compaction.
#[derive(Clone, Debug, Default)]
pub struct ContextMeter {
    seen: bool,
    level: ContextLevel,
    from: u32,
    target: u32,
    since: ClockMs,
    snoozed_below: Option<f32>,
    /// Thresholds.
    pub thresholds: Thresholds,
    /// Host compaction state.
    pub compaction: Compaction,
    /// Reduced motion: no roll, no breathe.
    pub reduced: bool,
}

impl ContextMeter {
    /// Dismisses the compact prompt until usage grows another 5 points.
    pub fn snooze(&mut self, ratio: f32) {
        self.snoozed_below = Some(ratio + 0.05);
    }

    /// Updates from the latest report and returns the frame. Hosts mark the report inexact
    /// (`exact: false`) while streaming; the label then reads `≈`.
    pub fn view(&mut self, usage: &ContextUsage, now: ClockMs) -> ContextView {
        let used = usage.used();
        if !self.seen {
            // The first report shows as-is; only later changes roll.
            self.seen = true;
            self.from = used;
            self.target = used;
            self.since = now;
        }
        if used != self.target {
            self.from = self.shown(now);
            self.target = used;
            self.since = now;
        }
        let shown = if self.reduced { used } else { self.shown(now) };
        let ratio = usage.ratio();
        self.level = match ratio {
            None => ContextLevel::Unknown,
            Some(r) => ContextLevel::from_ratio(r, &self.thresholds, self.level),
        };
        let fill = usage
            .window
            .map(|w| (shown as f32 / w.max(1) as f32).min(1.0))
            .unwrap_or(0.0);
        let alpha = if self.level == ContextLevel::Critical && !self.reduced {
            let p = (now.0 % BREATHE.as_millis() as u64) as f32 / BREATHE.as_millis() as f32;
            0.875 + 0.125 * (p * std::f32::consts::TAU).cos()
        } else {
            1.0
        };
        let snoozed = matches!((self.snoozed_below, ratio), (Some(s), Some(r)) if r < s);
        let prompt = match (ratio, &self.compaction) {
            (_, Compaction::Failed(_)) => Some("Couldn't compact · Retry".into()),
            (Some(r), Compaction::Idle | Compaction::Done { .. })
                if r >= self.thresholds.compact_prompt_at && !snoozed =>
            {
                Some(format!(
                    "Context {}% full · Compact now",
                    (r * 100.0).round() as u32
                ))
            }
            _ => None,
        };
        let float = match &self.compaction {
            Compaction::Done { delta, at } if now.since(*at) < FLOAT => Some((
                format!("−{}", k(*delta)),
                now.since(*at).as_secs_f32() / FLOAT.as_secs_f32(),
            )),
            _ => None,
        };
        let estimate = !usage.exact;
        ContextView {
            level: self.level,
            fill,
            used: shown,
            window: usage.window,
            estimate,
            alpha,
            prompt,
            compacting: self.compaction == Compaction::Running,
            float,
            stale: now.since(usage.at) >= STALE_AFTER,
        }
    }

    fn shown(&self, now: ClockMs) -> u32 {
        let t = (now.since(self.since).as_secs_f32() / ROLL.as_secs_f32()).min(1.0);
        let e = 1.0 - (1.0 - t).powi(3);
        (self.from as f32 + (self.target as f32 - self.from as f32) * e).round() as u32
    }

    /// Frames are needed.
    pub fn animating(&self, now: ClockMs) -> bool {
        self.shown(now) != self.target
            || self.level == ContextLevel::Critical
            || self.compaction == Compaction::Running
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ContextCategory as C;

    fn usage(parts: &[(ContextCategory, u32)], window: u32) -> ContextUsage {
        ContextUsage {
            window: Some(window),
            categories: parts
                .iter()
                .map(|(c, t)| CategoryUsage {
                    category: *c,
                    tokens: *t,
                })
                .collect(),
            exact: true,
            at: ClockMs(0),
        }
    }

    #[test]
    fn levels_with_hysteresis() {
        let t = Thresholds::default();
        let mut l = ContextLevel::Unknown;
        for (r, want) in [
            (0.5, ContextLevel::Normal),
            (0.71, ContextLevel::Warning),
            (0.69, ContextLevel::Warning),
            (0.66, ContextLevel::Normal),
            (0.9, ContextLevel::Critical),
            (1.05, ContextLevel::Overflow),
            (0.5, ContextLevel::Normal),
        ] {
            l = ContextLevel::from_ratio(r, &t, l);
            assert_eq!(l, want, "at {r}");
        }
    }

    #[test]
    fn rows_fold_small_and_add_free() {
        let u = usage(
            &[
                (C::System, 12_000),
                (C::Skills, 400),
                (C::History, 70_000),
                (C::Reserved, 20_000),
            ],
            200_000,
        );
        let rows = u.rows();
        assert_eq!(rows[0].0, C::System);
        assert!(rows.iter().any(|r| r.0 == C::Other && r.1 == 400));
        assert_eq!(rows.last().unwrap().0, C::Free);
        assert_eq!(rows.last().unwrap().1, 97_600);
        let g = u.grid(10, 10);
        assert_eq!(g.len(), 100);
        assert_eq!(g.iter().filter(|c| **c == C::History).count(), 35);
        assert_eq!(g.iter().filter(|c| **c == C::Free).count(), 49);
    }

    #[test]
    fn labels_roll_and_prompt() {
        let mut m = ContextMeter::default();
        let u = usage(&[(C::History, 82_400)], 200_000);
        let v = m.view(&u, ClockMs(0));
        assert_eq!(v.used, 82_400, "first report shows as-is");
        m.view(&usage(&[(C::History, 92_400)], 200_000), ClockMs(100));
        let v2 = m.view(&usage(&[(C::History, 92_400)], 200_000), ClockMs(200));
        assert!(v2.used > 82_400 && v2.used < 92_400, "later changes roll");
        let v = ContextMeter::default().view(&u, ClockMs(1_400));
        assert_eq!(v.label(LabelMode::Percent), "41%");
        assert_eq!(v.label(LabelMode::Tokens), "82.4K / 200K");
        assert_eq!(v.label(LabelMode::Remaining), "59% context left");
        assert_eq!(v.tooltip(), "82.4K of 200K tokens · 41% · 117K free");
        let full = usage(&[(C::History, 182_000)], 200_000);
        let v = m.view(&full, ClockMs(1_000));
        assert_eq!(v.prompt.as_deref(), Some("Context 91% full · Compact now"));
        m.snooze(0.91);
        assert_eq!(m.view(&full, ClockMs(1_100)).prompt, None);
        m.compaction = Compaction::Done {
            delta: 84_000,
            at: ClockMs(2_000),
        };
        let v = m.view(&usage(&[(C::History, 98_000)], 200_000), ClockMs(2_100));
        assert_eq!(v.float.as_ref().map(|f| f.0.as_str()), Some("−84K"));
        for v in ContextVariant::ALL {
            assert_eq!(ContextVariant::from_id(v.id()), Some(v));
        }
    }
}
