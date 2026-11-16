//! affectUI fidelity pass, family B part 9 (plan #32): the **Plan vs Actual** model. An agent's
//! plan – steps with planned durations and dependencies – laid against what really happens: a
//! step starts when everything it depends on has ended, a running step that passes its plan keeps
//! growing, the steps after it move later, and the expected finish (ETA) moves away from the
//! planned one. Behaviour only: the views live in `oxyn-dioxus` / `oxyn-gpui`
//! (`parity/planfid.rs`).

/// One step of the plan.
#[derive(Clone, Debug, PartialEq)]
pub struct PvStep {
    /// Stable id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Planned duration, seconds.
    pub plan: u32,
    /// Steps it waits for (ids).
    pub deps: Vec<String>,
    /// How long it really takes, seconds (the script; unknown to the view until it ends).
    pub actual: u32,
    /// What it costs once done, dollars.
    pub cost: f64,
    /// Planned cost, dollars.
    pub plan_cost: f64,
}

/// Where a step stands at a moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PvState {
    /// Not started.
    Waiting,
    /// Working.
    Running,
    /// Finished.
    Done,
}

impl PvState {
    /// `waiting` / `running` / `done`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Running => "running",
            Self::Done => "done",
        }
    }
}

/// A step as drawn at time `now`.
#[derive(Clone, Debug, PartialEq)]
pub struct PvRow {
    /// State.
    pub state: PvState,
    /// Planned start and end, seconds.
    pub plan_start: u32,
    /// Planned end.
    pub plan_end: u32,
    /// Start (real, or expected for a waiting step).
    pub start: u32,
    /// End (real, or expected: start + max(plan, elapsed)).
    pub end: u32,
    /// Seconds over (+) or under (−) plan so far (done: final; running: only once past plan).
    pub delta: i64,
}

fn idx(steps: &[PvStep], id: &str) -> Option<usize> {
    steps.iter().position(|s| s.id == id)
}

/// Planned (start, end) of every step: each starts when its dependencies' planned ends are past.
pub fn pv_planned(steps: &[PvStep]) -> Vec<(u32, u32)> {
    let mut out: Vec<(u32, u32)> = Vec::with_capacity(steps.len());
    for s in steps {
        let st = s
            .deps
            .iter()
            .filter_map(|d| idx(steps, d))
            .filter(|&j| j < out.len())
            .map(|j| out[j].1)
            .max()
            .unwrap_or(0);
        out.push((st, st + s.plan));
    }
    out
}

/// Every step at `now` seconds (steps must list dependencies before dependants).
pub fn pv_rows(steps: &[PvStep], now: u32) -> Vec<PvRow> {
    let plan = pv_planned(steps);
    let mut out: Vec<PvRow> = Vec::with_capacity(steps.len());
    for (i, s) in steps.iter().enumerate() {
        let deps: Vec<&PvRow> = s
            .deps
            .iter()
            .filter_map(|d| idx(steps, d))
            .filter(|&j| j < out.len())
            .map(|j| &out[j])
            .collect();
        let ready = deps.iter().all(|r| r.state == PvState::Done);
        let start = deps.iter().map(|r| r.end).max().unwrap_or(0);
        let (state, end, delta) = if ready && now >= start + s.actual {
            (
                PvState::Done,
                start + s.actual,
                s.actual as i64 - s.plan as i64,
            )
        } else if ready && now >= start {
            let el = now - start;
            let d = (el as i64 - s.plan as i64).max(0);
            (PvState::Running, start + el.max(s.plan), d)
        } else {
            (PvState::Waiting, start + s.plan, 0)
        };
        out.push(PvRow {
            state,
            plan_start: plan[i].0,
            plan_end: plan[i].1,
            start,
            end,
            delta,
        });
    }
    out
}

/// The planned finish.
pub fn pv_plan_total(steps: &[PvStep]) -> u32 {
    pv_planned(steps).iter().map(|p| p.1).max().unwrap_or(0)
}

/// The expected finish at `now`.
pub fn pv_eta(rows: &[PvRow]) -> u32 {
    rows.iter().map(|r| r.end).max().unwrap_or(0)
}

/// When the whole run really ends.
pub fn pv_actual_total(steps: &[PvStep]) -> u32 {
    let r = pv_rows(steps, u32::MAX / 4);
    pv_eta(&r)
}

/// Money: (spent so far, planned total). A running step counts pro rata of its real cost.
pub fn pv_cost(steps: &[PvStep], rows: &[PvRow], now: u32) -> (f64, f64) {
    let spent = steps
        .iter()
        .zip(rows)
        .map(|(s, r)| match r.state {
            PvState::Done => s.cost,
            PvState::Running => s.cost * ((now - r.start) as f64 / s.actual.max(1) as f64).min(1.0),
            PvState::Waiting => 0.0,
        })
        .sum();
    (spent, steps.iter().map(|s| s.plan_cost).sum())
}

/// "2:04".
pub fn pv_fmt(s: u32) -> String {
    format!("{}:{:02}", s / 60, s % 60)
}

/// "+28s" / "−4s" / "+1:05".
pub fn pv_delta(d: i64) -> String {
    let sign = if d < 0 { "−" } else { "+" };
    let a = d.unsigned_abs() as u32;
    if a >= 60 {
        format!("{sign}{}", pv_fmt(a))
    } else {
        format!("{sign}{a}s")
    }
}

/// Axis end: the later of plan and ETA plus a margin, rounded up to 30 s.
pub fn pv_axis(plan: u32, eta: u32) -> u32 {
    let m = plan.max(eta) + 15;
    m.div_ceil(30) * 30
}

fn st(
    id: &str,
    title: &str,
    plan: u32,
    deps: &[&str],
    actual: u32,
    cost: f64,
    plan_cost: f64,
) -> PvStep {
    PvStep {
        id: id.into(),
        title: title.into(),
        plan,
        deps: deps.iter().map(|d| d.to_string()).collect(),
        actual,
        cost,
        plan_cost,
    }
}

/// The demo plan: launch a pricing page – two steps overrun, one finishes early.
pub fn pv_demo() -> Vec<PvStep> {
    vec![
        st("brief", "Read the brief", 20, &[], 16, 0.04, 0.05),
        st("copy", "Write the copy", 30, &["brief"], 58, 0.31, 0.16),
        st("layout", "Build the layout", 45, &["brief"], 41, 0.27, 0.29),
        st(
            "checkout",
            "Hook up checkout",
            35,
            &["layout"],
            47,
            0.33,
            0.24,
        ),
        st(
            "tests",
            "Add tests",
            25,
            &["copy", "checkout"],
            25,
            0.12,
            0.12,
        ),
        st("review", "Self-review", 15, &["tests"], 13, 0.08, 0.09),
        st("ship", "Deploy", 10, &["review"], 10, 0.06, 0.10),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plan_and_run() {
        let s = pv_demo();
        let p = pv_planned(&s);
        assert_eq!(p[0], (0, 20));
        assert_eq!(p[3], (65, 100));
        assert_eq!(p[4], (100, 125));
        assert_eq!(pv_plan_total(&s), 150);
        let r = pv_rows(&s, 10);
        assert_eq!(r[0].state, PvState::Running);
        assert_eq!(r[1].state, PvState::Waiting);
        assert_eq!(pv_eta(&r), 150);
        // At 60 s: copy (started 16) is 44 s in – 14 s over its 30 s plan.
        let r = pv_rows(&s, 60);
        assert_eq!(r[0].delta, -4);
        assert_eq!(r[1].state, PvState::Running);
        assert_eq!(r[1].delta, 14);
        assert_eq!(r[1].end, 60);
        let total = pv_actual_total(&s);
        // brief 16 → layout 57 → checkout 104 → tests (copy ended 74) 129 → review 142 → ship 152.
        assert_eq!(total, 152);
        let done = pv_rows(&s, total);
        assert!(done.iter().all(|x| x.state == PvState::Done));
        let (spent, plan) = pv_cost(&s, &done, total);
        assert!((spent - 1.21).abs() < 1e-9);
        assert!((plan - 1.05).abs() < 1e-9);
        assert_eq!(pv_fmt(124), "2:04");
        assert_eq!(pv_delta(28), "+28s");
        assert_eq!(pv_delta(-4), "−4s");
        assert_eq!(pv_delta(65), "+1:05");
        assert_eq!(pv_axis(150, 152), 180);
    }
}
