//! affectUI fidelity pass, family B part 9 (plan #32): the **Diff Timeline** run model. An agent
//! edits files over a run; each edit lands at a moment (ms from the start) with the lines it
//! added and removed. At any moment the run has running totals (added, removed, net, files
//! touched), a cumulative series for the chart, one row per touched file (in first-touch order,
//! its edits as pieces of a bar) and a "file being edited now". Behaviour only: the views live in
//! `oxyn-dioxus` / `oxyn-gpui` (`kit/difftlfid.rs`).

/// One edit of the run.
#[derive(Clone, Debug, PartialEq)]
pub struct DtEdit {
    /// When it landed, ms from the run start.
    pub at: u32,
    /// The file path (`folder/name.ext`).
    pub file: String,
    /// Lines added.
    pub added: u32,
    /// Lines removed.
    pub removed: u32,
    /// A binary file (counts are not lines).
    pub binary: bool,
    /// A few changed lines, each starting with `+` or `-`.
    pub lines: Vec<String>,
}

impl DtEdit {
    /// A text edit.
    pub fn new(at: u32, file: &str, added: u32, removed: u32, lines: &[&str]) -> Self {
        Self {
            at,
            file: file.into(),
            added,
            removed,
            binary: false,
            lines: lines.iter().map(|s| s.to_string()).collect(),
        }
    }
    /// A binary edit (an image, a font).
    pub fn binary(at: u32, file: &str) -> Self {
        Self {
            at,
            file: file.into(),
            added: 0,
            removed: 0,
            binary: true,
            lines: vec![],
        }
    }
}

/// Splits a path into (folder with trailing slash, file name).
pub fn dt_split(path: &str) -> (String, String) {
    match path.rfind('/') {
        Some(i) => (path[..=i].to_string(), path[i + 1..].to_string()),
        None => (String::new(), path.to_string()),
    }
}

/// The file's extension, lowercase (`tsx`, `png`), or empty.
pub fn dt_ext(path: &str) -> String {
    let (_, n) = dt_split(path);
    n.rfind('.')
        .map(|i| n[i + 1..].to_ascii_lowercase())
        .unwrap_or_default()
}

/// One touched file at a moment.
#[derive(Clone, Debug, PartialEq)]
pub struct DtFile {
    /// The path.
    pub file: String,
    /// Lines added so far.
    pub added: u32,
    /// Lines removed so far.
    pub removed: u32,
    /// Binary.
    pub binary: bool,
    /// Each edit so far as (index into the edits, added, removed), in time order.
    pub pieces: Vec<(usize, u32, u32)>,
    /// When it was first touched.
    pub first: u32,
}

/// Edits that have landed by `now`, in time order (indices into `edits`).
pub fn dt_landed(edits: &[DtEdit], now: u32) -> Vec<usize> {
    let mut v: Vec<usize> = (0..edits.len()).filter(|&i| edits[i].at <= now).collect();
    v.sort_by_key(|&i| edits[i].at);
    v
}

/// The touched files at `now`, in first-touch order.
pub fn dt_files(edits: &[DtEdit], now: u32) -> Vec<DtFile> {
    let mut out: Vec<DtFile> = vec![];
    for i in dt_landed(edits, now) {
        let e = &edits[i];
        match out.iter_mut().find(|f| f.file == e.file) {
            Some(f) => {
                f.added += e.added;
                f.removed += e.removed;
                f.binary |= e.binary;
                f.pieces.push((i, e.added, e.removed));
            }
            None => out.push(DtFile {
                file: e.file.clone(),
                added: e.added,
                removed: e.removed,
                binary: e.binary,
                pieces: vec![(i, e.added, e.removed)],
                first: e.at,
            }),
        }
    }
    out
}

/// The run's totals at a moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DtTotals {
    /// Lines added.
    pub added: u32,
    /// Lines removed.
    pub removed: u32,
    /// Files touched.
    pub files: usize,
    /// Edits landed.
    pub edits: usize,
}

impl DtTotals {
    /// Added minus removed.
    pub fn net(&self) -> i64 {
        self.added as i64 - self.removed as i64
    }
}

/// Totals at `now`.
pub fn dt_totals(edits: &[DtEdit], now: u32) -> DtTotals {
    let l = dt_landed(edits, now);
    DtTotals {
        added: l.iter().map(|&i| edits[i].added).sum(),
        removed: l.iter().map(|&i| edits[i].removed).sum(),
        files: dt_files(edits, now).len(),
        edits: l.len(),
    }
}

/// The cumulative series at `now`: (at, added so far, removed so far) per landed edit, starting
/// with (0, 0, 0) and ending at `now` (a flat run to the present while live).
pub fn dt_series(edits: &[DtEdit], now: u32) -> Vec<(u32, u32, u32)> {
    let mut v = vec![(0, 0, 0)];
    let (mut a, mut r) = (0, 0);
    for i in dt_landed(edits, now) {
        a += edits[i].added;
        r += edits[i].removed;
        v.push((edits[i].at, a, r));
    }
    let last = *v.last().unwrap_or(&(0, 0, 0));
    if now > last.0 {
        v.push((now, last.1, last.2));
    }
    v
}

/// The edit whose file is being worked on at `now` (the last landed), if the run is still going.
pub fn dt_current(edits: &[DtEdit], now: u32, end: u32) -> Option<usize> {
    if now >= end {
        return None;
    }
    dt_landed(edits, now).last().copied()
}

/// The run's end: the last edit (or 0).
pub fn dt_end(edits: &[DtEdit]) -> u32 {
    edits.iter().map(|e| e.at).max().unwrap_or(0)
}

/// The x axis: the run rounded up to a whole minute (at least one), and the tick step in ms.
pub fn dt_x_axis(end: u32) -> (u32, u32) {
    let m = end.div_ceil(60_000).max(1);
    let step = if m <= 6 { 1 } else { m.div_ceil(5) };
    (m.div_ceil(step) * step * 60_000, step * 60_000)
}

/// The y axis for a peak line count: (max, step) on a 1-2-5 ladder, four or five steps.
pub fn dt_y_axis(peak: u32) -> (u32, u32) {
    let peak = peak.max(1);
    let mut step = 1u32;
    for s in [
        1, 2, 5, 10, 20, 25, 50, 100, 200, 250, 500, 1000, 2000, 5000, 10000,
    ] {
        step = s;
        if peak.div_ceil(s) <= 4 {
            break;
        }
    }
    (peak.div_ceil(step).max(1) * step, step)
}

/// `m:ss` for ms.
pub fn dt_clock(ms: u32) -> String {
    let s = ms / 1000;
    format!("{}:{:02}", s / 60, s % 60)
}

/// A count, short past 999 (`4.8k`) when `short`.
pub fn dt_count(n: u32, short: bool) -> String {
    if short && n >= 1000 {
        let k = n as f64 / 1000.0;
        if k >= 10.0 {
            format!("{}k", k.round() as u32)
        } else {
            format!("{:.1}k", k)
        }
    } else {
        n.to_string()
    }
}

/// The status line: "Editing <file>" while live, "Done · n edits in m:ss" at the end, "Starting"
/// before the first edit.
pub fn dt_status(edits: &[DtEdit], now: u32) -> String {
    let end = dt_end(edits);
    let l = dt_landed(edits, now);
    if now >= end && !edits.is_empty() {
        format!(
            "Done · {} edit{} in {}",
            l.len(),
            if l.len() == 1 { "" } else { "s" },
            dt_clock(end)
        )
    } else if let Some(&i) = l.last() {
        format!("Editing {}", edits[i].file)
    } else {
        "Starting".to_string()
    }
}

/// Stepping between landed edits: the next position from `cur` (`None` = none open) for a key
/// (`next`, `prev`, `first`, `last`), over `n` landed edits.
pub fn dt_step(cur: Option<usize>, key: &str, n: usize) -> Option<usize> {
    if n == 0 {
        return None;
    }
    Some(match (key, cur) {
        ("first", _) => 0,
        ("last", _) => n - 1,
        ("next", None) => 0,
        ("prev", None) => n - 1,
        ("next", Some(c)) => (c + 1).min(n - 1),
        ("prev", Some(c)) => c.saturating_sub(1),
        (_, c) => return c,
    })
}

/// A 5×5 symmetric pixel mark for an agent id (cells on, row-major), so each agent has its own.
pub fn dt_mark(id: &str) -> Vec<bool> {
    let mut h: u32 = 2166136261;
    for b in id.bytes() {
        h = (h ^ b as u32).wrapping_mul(16777619);
    }
    let mut v = vec![false; 25];
    for r in 0..5 {
        for c in 0..3 {
            let on = (h >> (r * 3 + c)) & 1 == 1 || (r == 2 && c == 2);
            v[r * 5 + c] = on;
            v[r * 5 + (4 - c)] = on;
        }
    }
    v
}

/// The demo run: a UI agent reworking a billing page, fifteen edits over 3:22.
pub fn dt_demo() -> Vec<DtEdit> {
    vec![
        DtEdit::new(
            6_000,
            "lib/plans.ts",
            18,
            4,
            &[
                "+export const PLANS = [starter, team, scale];",
                "-export const plans = legacy;",
            ],
        ),
        DtEdit::new(
            19_000,
            "app/billing/page.tsx",
            34,
            12,
            &["+<PlanPicker plans={PLANS} />", "-<LegacyPlans />"],
        ),
        DtEdit::new(
            31_000,
            "components/plan-card.tsx",
            41,
            0,
            &[
                "+export function PlanCard({ plan }: Props) {",
                "+  return <article className={styles.card}>",
            ],
        ),
        DtEdit::new(
            44_000,
            "app/billing/page.tsx",
            12,
            9,
            &["+<UsageMeter used={usage} />", "-<p>{usage} used</p>"],
        ),
        DtEdit::binary(58_000, "public/plan-badge.png"),
        DtEdit::new(
            71_000,
            "styles/billing.css",
            27,
            15,
            &[".plan-grid { gap: 16px; }", "-.plans { margin: 0 auto; }"],
        ),
        DtEdit::new(
            86_000,
            "components/usage-meter.tsx",
            29,
            0,
            &["+export function UsageMeter({ used, cap }: Props) {"],
        ),
        DtEdit::new(
            98_000,
            "components/plan-card.tsx",
            8,
            6,
            &[
                "+  {plan.badge && <img src=\"/plan-badge.png\" alt=\"\" />}",
                "-  <span>{plan.tag}</span>",
            ],
        ),
        DtEdit::new(
            111_000,
            "lib/plans.ts",
            11,
            3,
            &["+  seats: { min: 3, max: 50 },", "-  seats: 5,"],
        ),
        DtEdit::new(
            124_000,
            "app/billing/page.tsx",
            22,
            7,
            &["+<InvoiceList invoices={invoices} />", "+<PlanFaq />"],
        ),
        DtEdit::new(
            139_000,
            "tests/billing.test.ts",
            38,
            0,
            &[
                "+it(\"shows three plans\", async () => {",
                "+  expect(cards).toHaveLength(3);",
            ],
        ),
        DtEdit::new(
            152_000,
            "styles/billing.css",
            9,
            11,
            &[
                "+.plan-card[data-pick] { outline: 2px solid; }",
                "-.plan-card.active { border: 2px solid; }",
            ],
        ),
        DtEdit::new(
            166_000,
            "components/usage-meter.tsx",
            6,
            2,
            &[
                "+  const pct = Math.min(used / cap, 1);",
                "-  const pct = used / cap;",
            ],
        ),
        DtEdit::new(
            181_000,
            "tests/billing.test.ts",
            7,
            1,
            &["+it(\"caps the meter at 100%\", () => {"],
        ),
        DtEdit::new(
            202_000,
            "app/billing/page.tsx",
            5,
            3,
            &[
                "+export const metadata = { title: \"Billing\" };",
                "-export const title = \"Plans\";",
            ],
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn totals_files_and_series() {
        let d = dt_demo();
        let end = dt_end(&d);
        assert_eq!(end, 202_000);
        let t = dt_totals(&d, end);
        assert_eq!(t.edits, 15);
        assert_eq!(t.files, 7);
        assert_eq!(t.added, d.iter().map(|e| e.added).sum::<u32>());
        let f = dt_files(&d, end);
        assert_eq!(f[0].file, "lib/plans.ts");
        assert_eq!(f[1].pieces.len(), 4);
        assert!(f.iter().any(|x| x.binary));
        let s = dt_series(&d, 50_000);
        assert_eq!(s.first(), Some(&(0, 0, 0)));
        assert_eq!(s.last().unwrap().0, 50_000);
        assert_eq!(s.last().unwrap().1, 18 + 34 + 41 + 12);
        assert_eq!(dt_totals(&d, 0).edits, 0);
    }

    #[test]
    fn status_axes_and_steps() {
        let d = dt_demo();
        assert_eq!(dt_status(&d, 0), "Starting");
        assert_eq!(dt_status(&d, 20_000), "Editing app/billing/page.tsx");
        assert_eq!(dt_status(&d, 300_000), "Done · 15 edits in 3:22");
        assert_eq!(dt_current(&d, 20_000, 202_000), Some(1));
        assert_eq!(dt_current(&d, 202_000, 202_000), None);
        assert_eq!(dt_x_axis(202_000), (240_000, 60_000));
        assert_eq!(dt_x_axis(0), (60_000, 60_000));
        assert_eq!(dt_y_axis(290), (300, 100));
        assert_eq!(dt_y_axis(7), (8, 2));
        assert_eq!(dt_count(4_812, true), "4.8k");
        assert_eq!(dt_count(4_812, false), "4812");
        assert_eq!(dt_step(None, "next", 3), Some(0));
        assert_eq!(dt_step(Some(2), "next", 3), Some(2));
        assert_eq!(dt_step(Some(0), "prev", 3), Some(0));
        assert_eq!(dt_step(None, "last", 3), Some(2));
        assert_eq!(dt_step(None, "next", 0), None);
        assert_eq!(dt_split("a/b/c.ts"), ("a/b/".into(), "c.ts".into()));
        assert_eq!(dt_ext("public/x.PNG"), "png");
        let m = dt_mark("ui-agent");
        assert_eq!(m.len(), 25);
        assert!(m[12]);
        assert_eq!(m[0], m[4]);
    }
}
