//! Meters (plan #28, chunk 4b): the behaviour behind the gauge, the usage meter, the split bar
//! and both context windows.
//!
//! - [`gauge`]: a value against a range with threshold bands; the highest band reached sets the
//!   tone and the caption.
//! - [`usage`]: what fills an allowance, with free space, an overage past the limit and the
//!   "Almost full" / "Over limit" badge.
//! - [`split`]: a 100 % bar cut into shares, where the highlighted share widens by about 4.5 % (or
//!   enough to fit its label) and the others give way proportionally.
//! - [`CellField`]: the affectUI Context Window field of cells (parts fill it in order; removing
//!   items and compacting free cells).
//! - [`ContextGrid`]: the Arc UI context window (200 cells grouped by category, a threshold row,
//!   send, attach and compact with automatic compaction at the threshold).
//!
//! Behaviour-only rebuilds; nothing is copied. Semantic tones only (no brand hues).

use super::text::group;

/// Semantic tone of a meter.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tone {
    /// The foreground (monochrome default).
    Accent,
    /// Success.
    Success,
    /// Warning.
    Warning,
    /// Danger.
    Danger,
}

impl Tone {
    /// From an id (`accent`, `success`, `warning`, `danger`).
    pub fn from_id(s: &str) -> Self {
        match s {
            "success" => Tone::Success,
            "warning" => Tone::Warning,
            "danger" => Tone::Danger,
            _ => Tone::Accent,
        }
    }
    /// The id.
    pub fn id(self) -> &'static str {
        match self {
            Tone::Accent => "accent",
            Tone::Success => "success",
            Tone::Warning => "warning",
            Tone::Danger => "danger",
        }
    }
}

/// A threshold band: from this value up, this tone and caption.
#[derive(Clone, Debug, PartialEq)]
pub struct Band {
    /// Starting value.
    pub from: f64,
    /// Tone.
    pub tone: Tone,
    /// Caption.
    pub label: String,
}

impl Band {
    /// A band.
    pub fn new(from: f64, tone: Tone, label: &str) -> Self {
        Self {
            from,
            tone,
            label: label.into(),
        }
    }
}

/// What a gauge shows.
#[derive(Clone, Debug, PartialEq)]
pub struct GaugeView {
    /// Fill, 0..1.
    pub frac: f32,
    /// Whole percent of the range.
    pub pct: u32,
    /// Tone.
    pub tone: Tone,
    /// Caption of the reached band.
    pub band: Option<String>,
}

/// A gauge: `value` between `min` and `max`; the highest band at or below the value wins.
pub fn gauge(value: f64, min: f64, max: f64, tone: Tone, bands: &[Band]) -> GaugeView {
    let span = (max - min).max(f64::EPSILON);
    let frac = ((value - min) / span).clamp(0.0, 1.0) as f32;
    let hit = bands
        .iter()
        .filter(|b| value >= b.from)
        .max_by(|a, b| a.from.total_cmp(&b.from));
    GaugeView {
        frac,
        pct: (frac * 100.0).round() as u32,
        tone: hit.map_or(tone, |b| b.tone),
        band: hit.map(|b| b.label.clone()),
    }
}

/// An arc's dash for a ring of radius `r` drawn over `sweep` degrees: (visible, gap) lengths for
/// `frac` of it, plus the full arc length.
pub fn arc_dash(frac: f32, r: f32, sweep: f32) -> (f32, f32, f32) {
    let full = std::f32::consts::TAU * r;
    let arc = full * sweep / 360.0;
    let on = arc * frac.clamp(0.0, 1.0);
    (on, full - on, arc)
}

/// One category of a usage meter or split bar.
#[derive(Clone, Debug, PartialEq)]
pub struct Share {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Amount.
    pub value: f64,
}

impl Share {
    /// A share.
    pub fn new(id: &str, label: &str, value: f64) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            value,
        }
    }
}

/// Usage meter state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UsageState {
    /// Under the warning line.
    Ok,
    /// At or past `warn_at` of the limit.
    Almost,
    /// Past the limit.
    Over,
}

/// What a usage meter draws.
#[derive(Clone, Debug, PartialEq)]
pub struct UsageView {
    /// `(start, width)` of each share on the bar, 0..1 of the bar.
    pub segs: Vec<(f32, f32)>,
    /// Total used.
    pub used: f64,
    /// Space left (0 when over).
    pub free: f64,
    /// Amount past the limit.
    pub over: f64,
    /// Where the limit marker sits on the bar (1.0 unless over).
    pub limit_at: f32,
    /// State.
    pub state: UsageState,
}

/// A usage meter over `limit`. Past the limit the bar is rescaled to the total used, the limit
/// marker moves inside and the overage is the part right of it.
pub fn usage(shares: &[Share], limit: f64, warn_at: f64) -> UsageView {
    let used: f64 = shares.iter().map(|s| s.value.max(0.0)).sum();
    let scale = used.max(limit).max(f64::EPSILON);
    let mut x = 0.0f32;
    let segs = shares
        .iter()
        .map(|s| {
            let w = (s.value.max(0.0) / scale) as f32;
            let r = (x, w);
            x += w;
            r
        })
        .collect();
    let state = if used > limit {
        UsageState::Over
    } else if used >= limit * warn_at {
        UsageState::Almost
    } else {
        UsageState::Ok
    };
    UsageView {
        segs,
        used,
        free: (limit - used).max(0.0),
        over: (used - limit).max(0.0),
        limit_at: (limit / scale) as f32,
        state,
    }
}

/// `12.4 GB` with `decimals` places.
pub fn amount(v: f64, decimals: usize, unit: &str) -> String {
    let s = format!("{v:.decimals$}");
    if unit.is_empty() {
        s
    } else {
        format!("{s} {unit}")
    }
}

/// Split bar layout: `(start, width, share)` per visible segment (hidden ones get width 0). The
/// focused segment widens by `grow` of the bar, or to `min_fit` when it is smaller than that,
/// and the others give way in proportion so the bar stays at 100 %.
pub fn split(
    shares: &[Share],
    hidden: &[String],
    focus: Option<usize>,
    grow: f32,
    min_fit: f32,
) -> Vec<(f32, f32, f32)> {
    let vis: Vec<bool> = shares
        .iter()
        .map(|s| s.value > 0.0 && !hidden.contains(&s.id))
        .collect();
    let total: f64 = shares
        .iter()
        .zip(&vis)
        .filter(|(_, v)| **v)
        .map(|(s, _)| s.value)
        .sum();
    let share: Vec<f32> = shares
        .iter()
        .zip(&vis)
        .map(|(s, v)| {
            if *v && total > 0.0 {
                (s.value / total) as f32
            } else {
                0.0
            }
        })
        .collect();
    let mut width = share.clone();
    if let Some(f) = focus.filter(|f| vis.get(*f) == Some(&true)) {
        let target = (share[f] + grow).max(min_fit).min(0.92);
        let rest: f32 = 1.0 - share[f];
        if rest > 0.0 {
            let k = (1.0 - target) / rest;
            for (i, w) in width.iter_mut().enumerate() {
                *w = if i == f { target } else { share[i] * k };
            }
        }
    }
    let mut x = 0.0;
    width
        .iter()
        .zip(share)
        .map(|(w, s)| {
            let r = (x, *w, s);
            x += w;
            r
        })
        .collect()
}

/// Next visible segment from `from` in `dir`, wrapping (arrow keys on the split bar).
pub fn next_visible(
    shares: &[Share],
    hidden: &[String],
    from: Option<usize>,
    dir: i32,
) -> Option<usize> {
    let vis: Vec<usize> = (0..shares.len())
        .filter(|i| shares[*i].value > 0.0 && !hidden.contains(&shares[*i].id))
        .collect();
    if vis.is_empty() {
        return None;
    }
    let pos = from.and_then(|f| vis.iter().position(|v| *v == f));
    let n = vis.len() as i32;
    let p = match pos {
        Some(p) => (p as i32 + dir).rem_euclid(n),
        None if dir >= 0 => 0,
        None => n - 1,
    };
    Some(vis[p as usize])
}

/// `12.5%` share text, one decimal.
pub fn pct1(f: f32) -> String {
    format!("{:.1}%", f * 100.0)
}

/// Compact token count: `1.2M`, `84K`, `950`.
pub fn tokens(n: u64) -> String {
    if n >= 1_000_000 {
        let v = n as f64 / 1_000_000.0;
        if v >= 10.0 || (v.fract() * 10.0).round() == 0.0 {
            format!("{:.0}M", v)
        } else {
            format!("{v:.1}M")
        }
    } else if n >= 1000 {
        let v = n as f64 / 1000.0;
        if v >= 100.0 || (v.fract() * 10.0).round() == 0.0 {
            format!("{:.0}K", v)
        } else {
            format!("{v:.1}K")
        }
    } else {
        group(n as i64)
    }
}

/// One removable item inside a context part (a file, a tool).
#[derive(Clone, Debug, PartialEq)]
pub struct PartItem {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Tokens.
    pub tokens: u64,
}

/// A part of the context (system prompt, tools, files, messages) in the cell field.
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Tokens when it has no items.
    pub tokens: u64,
    /// Removable items (their tokens add up to the part).
    pub items: Vec<PartItem>,
    /// Cannot be removed or compacted.
    pub locked: bool,
    /// Share of its size kept after Compact (e.g. 0.2), if it can be compacted.
    pub compact: Option<f64>,
}

impl Part {
    /// A part with a flat token count.
    pub fn new(id: &str, label: &str, tokens: u64) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            tokens,
            items: vec![],
            locked: false,
            compact: None,
        }
    }
    /// Its size.
    pub fn size(&self) -> u64 {
        if self.items.is_empty() {
            self.tokens
        } else {
            self.items.iter().map(|i| i.tokens).sum()
        }
    }
}

/// Level of a context window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Fill {
    /// Under `warn_at`.
    Ok,
    /// From `warn_at`: the percentage turns amber and the tray says what is left.
    Warn,
    /// From 95 %: the window counts as full.
    Full,
}

/// affectUI Context Window behaviour: parts fill a field of cells in order.
#[derive(Clone, Debug, PartialEq)]
pub struct CellField {
    /// Parts, in fill order.
    pub parts: Vec<Part>,
    /// Window size in tokens.
    pub limit: u64,
    /// Warning share.
    pub warn_at: f64,
}

impl CellField {
    /// Tokens in use.
    pub fn used(&self) -> u64 {
        self.parts.iter().map(Part::size).sum()
    }
    /// Share in use.
    pub fn share(&self) -> f64 {
        self.used() as f64 / self.limit.max(1) as f64
    }
    /// Level.
    pub fn fill(&self) -> Fill {
        let s = self.share();
        if s >= 0.95 {
            Fill::Full
        } else if s >= self.warn_at {
            Fill::Warn
        } else {
            Fill::Ok
        }
    }
    /// Which part owns each of `cells` cells (`None` = free). Cumulative rounding, so each part
    /// gets at least one cell when it has any tokens.
    pub fn cells(&self, cells: usize) -> Vec<Option<usize>> {
        let mut out = vec![None; cells];
        let per = self.limit.max(1) as f64 / cells as f64;
        let mut at = 0usize;
        let mut acc = 0f64;
        for (pi, p) in self.parts.iter().enumerate() {
            let s = p.size();
            if s == 0 {
                continue;
            }
            acc += s as f64;
            let end = ((acc / per).round() as usize).max(at + 1).min(cells);
            for c in out.iter_mut().take(end).skip(at) {
                *c = Some(pi);
            }
            at = end;
        }
        out
    }
    /// Removes an item; false when the part is locked or the item is unknown.
    pub fn remove(&mut self, part: &str, item: &str) -> bool {
        let Some(p) = self.parts.iter_mut().find(|p| p.id == part && !p.locked) else {
            return false;
        };
        let n = p.items.len();
        p.items.retain(|i| i.id != item);
        p.items.len() != n
    }
    /// Shrinks a part to its compact share; returns tokens freed.
    pub fn compact(&mut self, part: &str) -> u64 {
        let Some(p) = self.parts.iter_mut().find(|p| p.id == part) else {
            return 0;
        };
        let Some(k) = p.compact else { return 0 };
        let before = p.size();
        let after = (before as f64 * k).round() as u64;
        p.items.clear();
        p.tokens = after;
        p.compact = None;
        before - after
    }
    /// The tray line: "38K left" when warning, "Full — compact or start a new chat" when full.
    pub fn tray(&self) -> String {
        let left = self.limit.saturating_sub(self.used());
        match self.fill() {
            Fill::Ok => format!("{} of {}", tokens(self.used()), tokens(self.limit)),
            Fill::Warn => format!("{} left", tokens(left)),
            Fill::Full => "Full · compact or start a new chat".into(),
        }
    }
}

/// Category of an item in the Arc context grid, in fill order.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Cat {
    /// System prompt.
    System,
    /// Tool definitions.
    Tools,
    /// Attached files.
    Files,
    /// Summary of compacted messages.
    Summary,
    /// Messages.
    Messages,
}

impl Cat {
    /// All, in order.
    pub const ALL: [Cat; 5] = [
        Cat::System,
        Cat::Tools,
        Cat::Files,
        Cat::Summary,
        Cat::Messages,
    ];
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Cat::System => "System prompt",
            Cat::Tools => "Tools",
            Cat::Files => "Files",
            Cat::Summary => "Summary",
            Cat::Messages => "Messages",
        }
    }
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Cat::System => "system",
            Cat::Tools => "tools",
            Cat::Files => "files",
            Cat::Summary => "summary",
            Cat::Messages => "messages",
        }
    }
}

/// An item in the Arc context grid.
#[derive(Clone, Debug, PartialEq)]
pub struct GridItem {
    /// Category.
    pub cat: Cat,
    /// Label.
    pub label: String,
    /// Tokens.
    pub tokens: u64,
}

/// What a compaction did.
#[derive(Clone, Debug, PartialEq)]
pub struct Compaction {
    /// Messages folded.
    pub folded: usize,
    /// Summary size.
    pub summary: u64,
    /// Used before.
    pub before: u64,
    /// Used after.
    pub after: u64,
    /// Ran on its own at the threshold.
    pub automatic: bool,
}

/// Summary size for folded messages: about 6 %, between 4K and 24K.
pub fn summarize(folded: u64) -> u64 {
    ((folded as f64 * 0.06).round() as u64).clamp(4_000, 24_000)
}

/// Arc context window behaviour.
#[derive(Clone, Debug, PartialEq)]
pub struct ContextGrid {
    /// Items.
    pub items: Vec<GridItem>,
    /// Window in tokens.
    pub window: u64,
    /// Tokens reserved for the reply (never filled).
    pub reserve: u64,
    /// Compaction runs on its own past this share.
    pub threshold: f64,
    /// The last compaction.
    pub last: Option<Compaction>,
}

impl ContextGrid {
    /// Tokens in use.
    pub fn used(&self) -> u64 {
        self.items.iter().map(|i| i.tokens).sum()
    }
    /// Room left before the reserve.
    pub fn free(&self) -> u64 {
        self.window.saturating_sub(self.reserve + self.used())
    }
    /// Tokens per category.
    pub fn by_cat(&self, c: Cat) -> u64 {
        self.items
            .iter()
            .filter(|i| i.cat == c)
            .map(|i| i.tokens)
            .sum()
    }
    /// Category of each of `cells` cells, grouped in category order; the reserve fills the tail
    /// as `Some(None)`-like marker: returned as `(cat, reserve)`.
    pub fn cells(&self, cells: usize) -> Vec<(Option<Cat>, bool)> {
        let per = self.window.max(1) as f64 / cells as f64;
        let mut out = vec![(None, false); cells];
        let mut acc = 0f64;
        let mut at = 0usize;
        for c in Cat::ALL {
            let t = self.by_cat(c);
            if t == 0 {
                continue;
            }
            acc += t as f64;
            let end = ((acc / per).round() as usize).max(at + 1).min(cells);
            for cell in out.iter_mut().take(end).skip(at) {
                cell.0 = Some(c);
            }
            at = end;
        }
        let rs = ((self.reserve as f64 / per).round() as usize).min(cells);
        for cell in out.iter_mut().skip(cells - rs) {
            if cell.0.is_none() {
                cell.1 = true;
            }
        }
        out
    }
    /// Cell index where compaction starts.
    pub fn threshold_cell(&self, cells: usize) -> usize {
        ((cells as f64) * self.threshold).round() as usize
    }
    /// Over the threshold.
    pub fn over(&self) -> bool {
        self.used() as f64 >= self.window as f64 * self.threshold
    }
    /// Adds an item; compacts on its own when it crosses the threshold.
    pub fn add(&mut self, cat: Cat, label: &str, tokens: u64) -> Option<Compaction> {
        self.items.push(GridItem {
            cat,
            label: label.into(),
            tokens,
        });
        if self.over() {
            self.compact(true)
        } else {
            None
        }
    }
    /// Folds every message into one summary placed after the files.
    pub fn compact(&mut self, automatic: bool) -> Option<Compaction> {
        let before = self.used();
        let folded: Vec<GridItem> = self
            .items
            .iter()
            .filter(|i| i.cat == Cat::Messages)
            .cloned()
            .collect();
        if folded.is_empty() {
            return None;
        }
        let ft: u64 = folded.iter().map(|i| i.tokens).sum();
        let s = summarize(ft);
        self.items.retain(|i| i.cat != Cat::Messages);
        if let Some(prev) = self.items.iter_mut().find(|i| i.cat == Cat::Summary) {
            prev.tokens += s;
        } else {
            self.items.push(GridItem {
                cat: Cat::Summary,
                label: "Summary".into(),
                tokens: s,
            });
        }
        let c = Compaction {
            folded: folded.len(),
            summary: s,
            before,
            after: self.used(),
            automatic,
        };
        self.last = Some(c.clone());
        Some(c)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gauge_bands() {
        let b = [
            Band::new(0.0, Tone::Success, "Healthy"),
            Band::new(70.0, Tone::Warning, "Busy"),
            Band::new(90.0, Tone::Danger, "Critical"),
        ];
        let g = gauge(72.0, 0.0, 100.0, Tone::Accent, &b);
        assert_eq!((g.pct, g.tone), (72, Tone::Warning));
        assert_eq!(g.band.as_deref(), Some("Busy"));
        assert_eq!(
            gauge(50.0, 0.0, 100.0, Tone::Accent, &[]).tone,
            Tone::Accent
        );
        let (on, _, arc) = arc_dash(0.5, 10.0, 270.0);
        assert!((on * 2.0 - arc).abs() < 0.01);
    }

    #[test]
    fn usage_over_and_almost() {
        let s = [Share::new("a", "Docs", 6.0), Share::new("b", "Media", 3.5)];
        let u = usage(&s, 10.0, 0.9);
        assert_eq!(u.state, UsageState::Almost);
        assert_eq!(u.limit_at, 1.0);
        let s2 = [Share::new("a", "Docs", 8.0), Share::new("b", "Media", 4.0)];
        let o = usage(&s2, 10.0, 0.9);
        assert_eq!(o.state, UsageState::Over);
        assert!((o.over - 2.0).abs() < 1e-9);
        assert!((o.limit_at - 10.0 / 12.0).abs() < 1e-6);
    }

    #[test]
    fn split_widens_focus_and_keeps_100() {
        let s = [
            Share::new("a", "Search", 50.0),
            Share::new("b", "Direct", 30.0),
            Share::new("c", "Other", 20.0),
        ];
        let l = split(&s, &[], Some(2), 0.045, 0.0);
        let sum: f32 = l.iter().map(|x| x.1).sum();
        assert!((sum - 1.0).abs() < 1e-5);
        assert!((l[2].1 - 0.245).abs() < 1e-5);
        let h = split(&s, &["a".into()], None, 0.045, 0.0);
        assert_eq!(h[0].1, 0.0);
        assert!((h[1].2 - 0.6).abs() < 1e-5);
        assert_eq!(next_visible(&s, &["b".into()], Some(0), 1), Some(2));
        assert_eq!(next_visible(&s, &[], None, -1), Some(2));
    }

    #[test]
    fn cell_field_fills_removes_and_compacts() {
        let mut f = CellField {
            parts: vec![
                Part::new("sys", "System", 10_000),
                Part {
                    items: vec![
                        PartItem {
                            id: "a".into(),
                            label: "a.rs".into(),
                            tokens: 20_000,
                        },
                        PartItem {
                            id: "b".into(),
                            label: "b.rs".into(),
                            tokens: 10_000,
                        },
                    ],
                    ..Part::new("files", "Files", 0)
                },
                Part {
                    compact: Some(0.2),
                    ..Part::new("msgs", "Messages", 120_000)
                },
            ],
            limit: 200_000,
            warn_at: 0.85,
        };
        assert_eq!(f.fill(), Fill::Ok);
        let c = f.cells(200);
        assert_eq!(c.iter().filter(|x| **x == Some(0)).count(), 10);
        assert!(f.remove("files", "a"));
        assert_eq!(f.used(), 140_000);
        assert_eq!(f.compact("msgs"), 96_000);
        assert_eq!(f.used(), 44_000);
        assert_eq!(tokens(1_000_000), "1M");
        assert_eq!(tokens(84_200), "84.2K");
    }

    #[test]
    fn grid_auto_compacts() {
        let mut g = ContextGrid {
            items: vec![
                GridItem {
                    cat: Cat::System,
                    label: "System".into(),
                    tokens: 10_000,
                },
                GridItem {
                    cat: Cat::Messages,
                    label: "Turn".into(),
                    tokens: 600_000,
                },
            ],
            window: 1_000_000,
            reserve: 64_000,
            threshold: 0.8,
            last: None,
        };
        assert!(g.add(Cat::Files, "spec.pdf", 50_000).is_none());
        let c = g.add(Cat::Messages, "Turn 2", 200_000).expect("compacts");
        assert!(c.automatic);
        assert_eq!(c.folded, 2);
        assert_eq!(c.summary, 24_000);
        assert_eq!(g.used(), 84_000);
        let cells = g.cells(200);
        assert_eq!(cells[0].0, Some(Cat::System));
        assert!(cells[199].1, "reserve at the tail");
        assert_eq!(g.threshold_cell(200), 160);
    }
}
