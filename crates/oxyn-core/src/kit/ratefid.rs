//! affectUI fidelity pass, family B part 9 (plan #32): the **Rate Limit Planner** model. Each
//! planned agent run draws tokens per minute: it climbs linearly to its peak, then halves at a
//! steady rate. The day's load is the sum of every run plus a small base; the planner reports
//! the peak against the API's per-minute limit, spreads movable runs out to lower the peak, and
//! finds the nearest start time at which a run that causes an overrun fits. Behaviour only: the
//! views live in `oxyn-dioxus` / `oxyn-gpui` (`kit/ratefid.rs`).

/// Minutes in a day.
pub const RL_DAY: u32 = 1440;
/// The chart starts at 06:00 and runs for a day.
pub const RL_AXIS0: u32 = 360;
/// Sampling step, minutes.
pub const RL_STEP: u32 = 5;

/// One planned run.
#[derive(Clone, Debug, PartialEq)]
pub struct RlRun {
    /// Stable id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Start, minutes on the chart (0 = 06:00; may pass 1440 − 360 into the next morning).
    pub start: u32,
    /// Peak, thousand tokens per minute.
    pub peak: u32,
    /// Minutes to climb to the peak.
    pub climb: u32,
    /// Minutes for the load to halve after the peak.
    pub half: u32,
    /// Colour slot (0–3).
    pub tone: u8,
    /// Already started: it cannot be moved.
    pub started: bool,
}

/// Where the day stands against the limit.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RlState {
    /// Peak under 90 % of the limit.
    Under,
    /// Peak between 90 % and 100 %.
    Close,
    /// Peak above the limit.
    Over,
}

impl RlState {
    /// `under` / `close` / `over`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Under => "under",
            Self::Close => "close",
            Self::Over => "over",
        }
    }
}

/// One run's load at chart minute `t`, thousand tokens / minute.
pub fn rl_run_at(r: &RlRun, t: u32) -> f64 {
    if t < r.start {
        return 0.0;
    }
    let d = (t - r.start) as f64;
    let c = r.climb.max(1) as f64;
    let p = r.peak as f64;
    if d < c {
        return p * d / c;
    }
    let v = p * 0.5f64.powf((d - c) / r.half.max(1) as f64);
    if v < p * 0.02 { 0.0 } else { v }
}

/// The whole load at `t` (runs + `base`).
pub fn rl_at(runs: &[RlRun], base: f64, t: u32) -> f64 {
    base + runs.iter().map(|r| rl_run_at(r, t)).sum::<f64>()
}

/// The load every [`RL_STEP`] minutes over the chart day: (minute, load).
pub fn rl_curve(runs: &[RlRun], base: f64) -> Vec<(u32, f64)> {
    (0..=RL_DAY)
        .step_by(RL_STEP as usize)
        .map(|t| (t, rl_at(runs, base, t)))
        .collect()
}

/// The peak: (minute, load).
pub fn rl_peak(runs: &[RlRun], base: f64) -> (u32, f64) {
    rl_curve(runs, base)
        .into_iter()
        .fold((0, f64::MIN), |a, b| if b.1 > a.1 { b } else { a })
}

/// Under / close / over for a peak against `limit`.
pub fn rl_state(peak: f64, limit: f64) -> RlState {
    if peak > limit {
        RlState::Over
    } else if peak >= limit * 0.9 {
        RlState::Close
    } else {
        RlState::Under
    }
}

/// The first stretch over the limit: (from, to) minutes.
pub fn rl_over(runs: &[RlRun], base: f64, limit: f64) -> Option<(u32, u32)> {
    let c = rl_curve(runs, base);
    let i = c.iter().position(|p| p.1 > limit)?;
    let j = c[i..]
        .iter()
        .position(|p| p.1 <= limit)
        .map(|k| i + k)
        .unwrap_or(c.len() - 1);
    Some((c[i].0, c[j].0))
}

/// Tokens a run spends, thousands (area under its curve).
pub fn rl_tokens(r: &RlRun) -> f64 {
    (r.start..r.start + RL_DAY)
        .step_by(RL_STEP as usize)
        .map(|t| rl_run_at(r, t) * RL_STEP as f64)
        .sum()
}

/// "41M" / "880k".
pub fn rl_fmt_tokens(k: f64) -> String {
    if k >= 1000.0 {
        format!("{:.0}M", k / 1000.0)
    } else {
        format!("{k:.0}k")
    }
}

/// Chart minute → "16:05" (wraps past midnight).
pub fn rl_clock(t: u32) -> String {
    let m = (t + RL_AXIS0) % RL_DAY;
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// "16:05" → chart minute.
pub fn rl_parse(s: &str) -> Option<u32> {
    let (h, m) = s.trim().split_once(':')?;
    let (h, m): (u32, u32) = (h.parse().ok()?, m.parse().ok()?);
    (h < 24 && m < 60).then(|| (h * 60 + m + RL_DAY - RL_AXIS0) % RL_DAY)
}

/// Latest start a run may take: it must start before the chart ends.
pub const RL_LAST: u32 = RL_DAY - 60;

/// Spread the movable runs to lower the peak: biggest first, each run tries every start from
/// `now` to [`RL_LAST`] (5-min steps) and keeps the one with the lowest peak, the nearest to
/// its old start on a tie. Started runs stay.
pub fn rl_spread(runs: &[RlRun], base: f64, now: u32) -> Vec<RlRun> {
    let mut out = runs.to_vec();
    let mut order: Vec<usize> = (0..out.len()).filter(|&i| !out[i].started).collect();
    order.sort_by(|&a, &b| out[b].peak.cmp(&out[a].peak).then(a.cmp(&b)));
    for i in order {
        let old = out[i].start;
        let mut best = (f64::MAX, u32::MAX, old);
        let first = now.div_ceil(RL_STEP) * RL_STEP;
        for s in (first..=RL_LAST).step_by(RL_STEP as usize) {
            out[i].start = s;
            let p = rl_peak(&out, base).1;
            let dist = s.abs_diff(old);
            if p < best.0 - 0.05 || ((p - best.0).abs() <= 0.05 && dist < best.1) {
                best = (p, dist, s);
            }
        }
        out[i].start = best.2;
    }
    out
}

/// The nearest start (≥ `now`) at which run `i` keeps the day under `limit`, when it is not
/// there already.
pub fn rl_fit(runs: &[RlRun], base: f64, limit: f64, i: usize, now: u32) -> Option<u32> {
    let r = runs.get(i)?;
    if r.started {
        return None;
    }
    let mut t = runs.to_vec();
    let first = now.div_ceil(RL_STEP) * RL_STEP;
    let mut cands: Vec<u32> = (first..=RL_LAST).step_by(RL_STEP as usize).collect();
    cands.sort_by_key(|s| (s.abs_diff(r.start), *s));
    cands.into_iter().find(|&s| {
        t[i].start = s;
        rl_peak(&t, base).1 <= limit
    })
}

/// The run that adds most to the load at minute `t`.
pub fn rl_blame(runs: &[RlRun], t: u32) -> Option<usize> {
    (0..runs.len())
        .filter(|&i| rl_run_at(&runs[i], t) > 0.0)
        .max_by(|&a, &b| rl_run_at(&runs[a], t).total_cmp(&rl_run_at(&runs[b], t)))
}

fn run(id: &str, name: &str, at: &str, peak: u32, climb: u32, half: u32, tone: u8) -> RlRun {
    RlRun {
        id: id.into(),
        name: name.into(),
        start: rl_parse(at).unwrap_or(0),
        peak,
        climb,
        half,
        tone,
        started: false,
    }
}

/// A new run as "Add run" makes it.
pub fn rl_new(id: &str, now: u32) -> RlRun {
    RlRun {
        id: id.into(),
        name: "New run".into(),
        start: now,
        peak: 74,
        climb: 40,
        half: 70,
        tone: 3,
        started: false,
    }
}

/// The demo limit, thousand tokens per minute.
pub const RL_LIMIT: f64 = 120.0;
/// The demo base load.
pub const RL_BASE: f64 = 6.0;

/// The demo "now": 10:30.
pub fn rl_now() -> u32 {
    rl_parse("10:30").unwrap_or(0)
}

/// The demo day: one run under way since the morning, two afternoon runs that crowd each other
/// close to the limit, and a nightly one.
pub fn rl_demo() -> Vec<RlRun> {
    let mut v = vec![
        run("triage", "Sort the support queue", "09:40", 46, 20, 45, 1),
        run("research", "Market scan", "15:00", 82, 30, 60, 0),
        run("docs", "Refresh the docs site", "16:10", 70, 25, 50, 2),
        run("index", "Rebuild search index", "23:00", 64, 40, 70, 3),
    ];
    v[0].started = true;
    v
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn day_peak_spread_fit() {
        let r = rl_demo();
        assert_eq!(rl_clock(0), "06:00");
        assert_eq!(rl_clock(r[1].start), "15:00");
        assert_eq!(rl_clock(rl_parse("02:15").unwrap()), "02:15");
        let (pt, pv) = rl_peak(&r, RL_BASE);
        assert_eq!(rl_clock(pt), "16:35");
        assert!((pv - 114.7).abs() < 0.2, "{pv}");
        assert_eq!(rl_state(pv, RL_LIMIT), RlState::Close);
        assert_eq!(rl_over(&r, RL_BASE, RL_LIMIT), None);
        // Spread out lowers the peak; the started run stays put.
        let s = rl_spread(&r, RL_BASE, rl_now());
        assert_eq!(s[0].start, r[0].start);
        let sp = rl_peak(&s, RL_BASE).1;
        assert!(sp < pv - 20.0, "{sp}");
        assert_eq!(rl_state(sp, RL_LIMIT), RlState::Under);
        // Push the docs run onto the research peak: over, and a fit is found.
        let mut o = r.clone();
        o[2].start = rl_parse("15:30").unwrap();
        let (a, b) = rl_over(&o, RL_BASE, RL_LIMIT).unwrap();
        assert!(a < b);
        assert_eq!(rl_state(rl_peak(&o, RL_BASE).1, RL_LIMIT), RlState::Over);
        let f = rl_fit(&o, RL_BASE, RL_LIMIT, 2, rl_now()).unwrap();
        o[2].start = f;
        assert!(rl_peak(&o, RL_BASE).1 <= RL_LIMIT);
        assert_eq!(rl_fit(&r, RL_BASE, RL_LIMIT, 0, rl_now()), None);
        assert!(rl_tokens(&r[1]) > 5000.0);
        assert_eq!(rl_fmt_tokens(41_200.0), "41M");
        assert_eq!(rl_fmt_tokens(880.0), "880k");
        assert_eq!(rl_blame(&r, pt), Some(2));
    }
}
