//! Fleet overview and Fleet timeline: every agent at a glance. On top, what happened since you
//! last looked (a curve of done, needs-you and failed events); in the middle the fleet (leads with
//! their sub-agents) or one lane per agent on a shared clock; on the side an inbox of what needs
//! you. One filter (all, running, waiting, failed) applies to all three. Answers, retries and
//! dismissals change an agent's status in place and are derived from `now`.

use super::{dur, norm_key, nth, rove};

/// Where an agent stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum St {
    /// Working.
    Running,
    /// Waiting on you (it asked something).
    Waiting,
    /// Failed.
    Failed,
    /// Finished.
    Done,
}

impl St {
    /// Word.
    pub fn word(self) -> &'static str {
        match self {
            St::Running => "Running",
            St::Waiting => "Needs you",
            St::Failed => "Failed",
            St::Done => "Done",
        }
    }
    /// Semantic hue.
    pub fn hue(self) -> &'static str {
        match self {
            St::Running => "blue",
            St::Waiting => "amber",
            St::Failed => "red",
            St::Done => "green",
        }
    }
}

/// An event kind on the curve.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ev {
    /// Something finished.
    Done,
    /// An agent asked.
    Needs,
    /// Something failed.
    Failed,
}

/// One agent: name, model mark and label, base status, task, ask or error, parent lead, when it
/// started (minutes before the clock started), when it ended, cost in cents.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Agent {
    /// Name.
    pub name: &'static str,
    /// Brand id of its model.
    pub brand: &'static str,
    /// Model label.
    pub model: &'static str,
    /// Status before anyone answered.
    pub st: St,
    /// What it works on.
    pub task: &'static str,
    /// Its question (waiting) or error (failed).
    pub note: &'static str,
    /// Index of its lead, if it is a sub-agent.
    pub parent: Option<usize>,
    /// Started, minutes before the clock's start.
    pub start_min: u64,
    /// Ended, minutes before the clock's start (finished and failed agents).
    pub end_min: Option<u64>,
    /// Cost so far, cents.
    pub cents: u64,
}

#[allow(clippy::too_many_arguments)]
const fn a(
    name: &'static str,
    brand: &'static str,
    model: &'static str,
    st: St,
    task: &'static str,
    note: &'static str,
    parent: Option<usize>,
    start_min: u64,
    end_min: Option<u64>,
    cents: u64,
) -> Agent {
    Agent {
        name,
        brand,
        model,
        st,
        task,
        note,
        parent,
        start_min,
        end_min,
        cents,
    }
}

/// The sample fleet (fictional customers and teams).
pub const AGENTS: [Agent; 8] = [
    a(
        "Support lead",
        "claude",
        "Claude Sonnet 4.5",
        St::Running,
        "Work through the open support queue",
        "",
        None,
        29,
        None,
        184,
    ),
    a(
        "Billing",
        "claude",
        "Claude Haiku 4.5",
        St::Failed,
        "Refund $240 to Northwind Supply",
        "The card on file has expired",
        Some(0),
        22,
        Some(17),
        21,
    ),
    a(
        "Triage",
        "claude",
        "Claude Haiku 4.5",
        St::Running,
        "Tag the 38 new tickets",
        "",
        Some(0),
        14,
        None,
        33,
    ),
    a(
        "Release lead",
        "openai",
        "GPT-5",
        St::Waiting,
        "Ship version 2.4 to production",
        "Deploy 2.4 to production now? Smoke tests passed 41 of 41.",
        None,
        26,
        None,
        142,
    ),
    a(
        "Changelog",
        "openai",
        "GPT-5 mini",
        St::Done,
        "Write the release notes",
        "",
        Some(3),
        25,
        Some(16),
        12,
    ),
    a(
        "QA",
        "openai",
        "GPT-5 mini",
        St::Running,
        "Run the smoke suite on staging",
        "",
        Some(3),
        15,
        None,
        27,
    ),
    a(
        "Research lead",
        "gemini",
        "Gemini 2.5 Pro",
        St::Running,
        "Compare three single sign-on vendors",
        "",
        None,
        19,
        None,
        96,
    ),
    a(
        "Pricing scout",
        "gemini",
        "Gemini 2.5 Flash",
        St::Waiting,
        "Price the three vendors' team plans",
        "Start a 14-day trial with the team card to see real prices?",
        Some(6),
        12,
        None,
        9,
    ),
];

/// What happened since you looked: minutes before the clock's start, kind, agent, title.
pub const EVENTS: [(u64, Ev, usize, &str); 10] = [
    (452, Ev::Done, 4, "Release notes for 2.3 published"),
    (388, Ev::Done, 2, "Tagged 21 tickets"),
    (301, Ev::Failed, 5, "Smoke suite timed out on staging"),
    (246, Ev::Done, 5, "Smoke suite passed on a rerun"),
    (190, Ev::Done, 0, "Answered 14 tickets"),
    (17, Ev::Failed, 1, "Refund to Northwind Supply"),
    (64, Ev::Done, 6, "Shortlisted three vendors"),
    (11, Ev::Needs, 7, "Trial sign-up for pricing"),
    (9, Ev::Done, 2, "Closed 9 duplicate tickets"),
    (4, Ev::Needs, 3, "Deploy 2.4 to production"),
];

/// Filter names, in order.
pub const FILTERS: [&str; 4] = ["All", "Running", "Waiting", "Failed"];
/// How far back the curve reaches, minutes.
pub const SINCE_MIN: u64 = 480;
/// Curve buckets.
pub const BUCKETS: usize = 16;
/// Width of the lane window, minutes.
pub const SPAN_MIN: u64 = 30;
/// An approved or retried agent finishes this long after the answer, ms.
pub const SETTLE_MS: u64 = 6_000;

/// An answer given to an agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Act {
    /// Approved: runs again, then finishes.
    Approve,
    /// Declined: done without doing it.
    Reject,
    /// Retried: runs again, then finishes.
    Retry,
    /// Dismissed: counts as done.
    Dismiss,
}

/// The live state of Fleet overview or Fleet timeline.
#[derive(Clone, Debug, PartialEq)]
pub struct Mission {
    /// Clock start, ms.
    pub t0: u64,
    /// Filter index into [`FILTERS`].
    pub filter: usize,
    /// The inbox card that is open.
    pub open: Option<usize>,
    /// The lead whose sub-agents are shown (control) or the lane picked (timeline).
    pub lead: usize,
    /// Answers by agent, with when.
    pub acts: Vec<Option<(Act, u64)>>,
    /// The retried agent fails again (the error scenario).
    pub fail_retry: bool,
}

impl Mission {
    /// A fresh view at `now`, filtered by `filter`.
    pub fn new(filter: usize, fail_retry: bool, now: u64) -> Self {
        Self {
            t0: now,
            filter: filter.min(FILTERS.len() - 1),
            open: None,
            lead: 0,
            acts: vec![None; AGENTS.len()],
            fail_retry,
        }
    }

    /// Status of agent `i` at `now`, answers applied.
    pub fn st(&self, i: usize, now: u64) -> St {
        match self.acts.get(i).copied().flatten() {
            None => AGENTS[i].st,
            Some((Act::Reject | Act::Dismiss, _)) => St::Done,
            Some((Act::Retry, t)) if now >= t + SETTLE_MS && self.fail_retry => St::Failed,
            Some((_, t)) if now >= t + SETTLE_MS => St::Done,
            Some(_) => St::Running,
        }
    }

    /// The note under an agent at `now` (its ask, error, or what an answer did).
    pub fn note(&self, i: usize, now: u64) -> String {
        match (self.acts[i], self.st(i, now)) {
            (Some((Act::Reject, _)), _) => "Declined, nothing was changed".into(),
            (Some((Act::Dismiss, _)), _) => "Dismissed".into(),
            (Some((Act::Retry, _)), St::Failed) => format!("{} (again)", AGENTS[i].note),
            (Some(_), St::Running) => "Back at work on your answer".into(),
            (Some(_), _) => "Finished after your answer".into(),
            (None, St::Waiting | St::Failed) => AGENTS[i].note.into(),
            _ => String::new(),
        }
    }

    /// Running time of agent `i` at `now` (live while it runs).
    pub fn running(&self, i: usize, now: u64) -> u64 {
        let ag = &AGENTS[i];
        let el = now.saturating_sub(self.t0);
        let start = ag.start_min * 60_000;
        match (self.st(i, now), self.acts[i], ag.end_min) {
            (St::Done | St::Failed, Some((_, t)), _) => {
                start + t.saturating_sub(self.t0) + SETTLE_MS.min(now.saturating_sub(t))
            }
            (St::Done | St::Failed, None, Some(e)) => (ag.start_min - e) * 60_000,
            _ => start + el,
        }
    }

    /// Does agent `i` pass the filter at `now`?
    pub fn passes(&self, i: usize, now: u64) -> bool {
        let s = self.st(i, now);
        match self.filter {
            1 => s == St::Running,
            2 => s == St::Waiting,
            3 => s == St::Failed,
            _ => true,
        }
    }

    /// Leads, sub-agents and whether a row shows: a lead shows when it or one of its sub-agents
    /// passes the filter.
    pub fn shows(&self, i: usize, now: u64) -> bool {
        self.passes(i, now)
            || (AGENTS[i].parent.is_none()
                && self.children(i).into_iter().any(|c| self.passes(c, now)))
    }

    /// Lead indices.
    pub fn leads(&self) -> Vec<usize> {
        (0..AGENTS.len())
            .filter(|&i| AGENTS[i].parent.is_none())
            .collect()
    }

    /// Sub-agents of `lead`.
    pub fn children(&self, lead: usize) -> Vec<usize> {
        (0..AGENTS.len())
            .filter(|&i| AGENTS[i].parent == Some(lead))
            .collect()
    }

    /// Counts per filter at `now` (all, running, waiting, failed).
    pub fn counts(&self, now: u64) -> [usize; 4] {
        let mut c = [AGENTS.len(), 0, 0, 0];
        for i in 0..AGENTS.len() {
            match self.st(i, now) {
                St::Running => c[1] += 1,
                St::Waiting => c[2] += 1,
                St::Failed => c[3] += 1,
                St::Done => {}
            }
        }
        c
    }

    /// The inbox at `now`: agents that ask or failed and pass the filter, failures first.
    pub fn inbox(&self, now: u64) -> Vec<usize> {
        let mut v: Vec<usize> = (0..AGENTS.len())
            .filter(|&i| matches!(self.st(i, now), St::Waiting | St::Failed) && self.passes(i, now))
            .collect();
        v.sort_by_key(|&i| (self.st(i, now) != St::Failed, i));
        v
    }

    /// Events on the curve that pass the filter: minutes before now, kind, agent, title. Answers
    /// given here join the list as they settle.
    pub fn events(&self, now: u64) -> Vec<(u64, Ev, usize, String)> {
        let el = now.saturating_sub(self.t0) / 60_000;
        let mut v: Vec<(u64, Ev, usize, String)> = EVENTS
            .iter()
            .map(|&(m, k, i, t)| (m + el, k, i, t.to_string()))
            .collect();
        for (i, a) in self.acts.iter().enumerate() {
            if let Some((_, t)) = a
                && now >= t + SETTLE_MS
            {
                let st = self.st(i, now);
                let k = if st == St::Failed {
                    Ev::Failed
                } else {
                    Ev::Done
                };
                let m = now.saturating_sub(t + SETTLE_MS) / 60_000;
                v.push((m, k, i, format!("{}: {}", AGENTS[i].name, st.word())));
            }
        }
        v.retain(|e| {
            let s = self.st(e.2, now);
            match self.filter {
                1 => s == St::Running,
                2 => s == St::Waiting,
                3 => s == St::Failed,
                _ => true,
            }
        });
        v.sort_by_key(|e| e.0);
        v
    }

    /// The curve: per bucket (oldest first), counts of done, needs and failed.
    pub fn curve(&self, now: u64) -> Vec<[u32; 3]> {
        let mut b = vec![[0u32; 3]; BUCKETS];
        let w = SINCE_MIN as f64 / BUCKETS as f64;
        for (m, k, _, _) in self.events(now) {
            if m >= SINCE_MIN {
                continue;
            }
            let i = BUCKETS - 1 - ((m as f64 / w) as usize).min(BUCKETS - 1);
            b[i][match k {
                Ev::Done => 0,
                Ev::Needs => 1,
                Ev::Failed => 2,
            }] += 1;
        }
        b
    }

    /// The summary line (`7 done · 2 need you · 1 failed since 8 h ago`).
    pub fn summary(&self, now: u64) -> String {
        let ev = self.events(now);
        let n = |k: Ev| ev.iter().filter(|e| e.1 == k).count();
        format!(
            "{} done · {} need you · {} failed in the last {} hours",
            n(Ev::Done),
            n(Ev::Needs),
            n(Ev::Failed),
            SINCE_MIN / 60
        )
    }

    /// A lane block for agent `i`: start and end as fractions of the span (0 = 30 min ago,
    /// 1 = now), or `None` when it is outside the span.
    pub fn lane(&self, i: usize, now: u64) -> Option<(f64, f64)> {
        let span = (SPAN_MIN * 60_000) as f64;
        let el = now.saturating_sub(self.t0);
        let start_ago = AGENTS[i].start_min * 60_000 + el;
        let run = self.running(i, now);
        let end_ago = start_ago.saturating_sub(run);
        let s = 1.0 - (start_ago as f64 / span);
        let e = 1.0 - (end_ago as f64 / span);
        (e > 0.0).then_some((s.max(0.0), e.clamp(0.0, 1.0)))
    }

    /// Fleet cost in dollars.
    pub fn cost(&self) -> f64 {
        AGENTS.iter().map(|a| a.cents).sum::<u64>() as f64 / 100.0
    }

    /// Presses: `mi-f-<i>`, `mi-open-<i>`, `mi-ok-<i>`, `mi-no-<i>`, `mi-retry-<i>`,
    /// `mi-dismiss-<i>`, `mi-lead-<i>`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "mi-f-") {
            self.filter = i.min(FILTERS.len() - 1);
            self.open = None;
            let c = self.counts(now)[self.filter];
            return Some(format!(
                "Showing {}: {c} agents",
                FILTERS[self.filter].to_lowercase()
            ));
        }
        if let Some(i) = nth(id, "mi-open-") {
            self.open = if self.open == Some(i) { None } else { Some(i) };
            return self
                .open
                .map(|i| format!("{}: {}", AGENTS[i].name, self.note(i, now)));
        }
        if let Some(i) = nth(id, "mi-lead-") {
            self.lead = i.min(AGENTS.len() - 1);
            return Some(format!(
                "{}, {} sub-agents",
                AGENTS[self.lead].name,
                self.children(self.lead).len()
            ));
        }
        let (act, i) = [
            ("mi-ok-", Act::Approve),
            ("mi-no-", Act::Reject),
            ("mi-retry-", Act::Retry),
            ("mi-dismiss-", Act::Dismiss),
        ]
        .iter()
        .find_map(|(p, a)| nth(id, p).map(|i| (*a, i)))?;
        if i >= AGENTS.len() {
            return None;
        }
        let st = self.st(i, now);
        let ok = matches!(
            (act, st),
            (Act::Approve | Act::Reject, St::Waiting) | (Act::Retry | Act::Dismiss, St::Failed)
        );
        if !ok {
            return None;
        }
        self.acts[i] = Some((act, now));
        if self.open == Some(i) {
            self.open = None;
        }
        Some(match act {
            Act::Approve => format!("Approved. {} is back at work", AGENTS[i].name),
            Act::Reject => format!("Declined. {} will not do it", AGENTS[i].name),
            Act::Retry => format!("Retrying {}", AGENTS[i].name),
            Act::Dismiss => format!("{} dismissed", AGENTS[i].name),
        })
    }

    /// Keys on the filter (Left, Right, Home, End) and Escape to close the open card.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        match norm_key(key) {
            "Escape" => self.open.take().map(|_| "Closed".to_string()),
            k @ ("ArrowLeft" | "ArrowRight" | "Home" | "End") => {
                let i = rove(self.filter, FILTERS.len(), k);
                (i != self.filter)
                    .then(|| self.press(&format!("mi-f-{i}"), now))
                    .flatten()
            }
            _ => None,
        }
    }

    /// The running time of agent `i` as text.
    pub fn time(&self, i: usize, now: u64) -> String {
        dur(self.running(i, now))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_inbox_and_answers() {
        let mut m = Mission::new(0, false, 1_000);
        assert_eq!(m.counts(1_000), [8, 4, 2, 1]);
        assert_eq!(m.inbox(1_000), vec![1, 3, 7]);
        m.key("ArrowRight", 1_000);
        assert_eq!(m.filter, 1);
        assert!(m.shows(0, 1_000) && !m.passes(1, 1_000));
        m.press("mi-f-0", 1_000);
        m.press("mi-open-3", 1_000);
        assert_eq!(m.open, Some(3));
        assert!(m.press("mi-retry-3", 1_000).is_none());
        m.press("mi-ok-3", 2_000);
        assert_eq!(m.open, None);
        assert_eq!(m.st(3, 3_000), St::Running);
        assert_eq!(m.st(3, 9_000), St::Done);
        m.press("mi-retry-1", 2_000);
        assert_eq!(m.st(1, 9_000), St::Done);
        assert_eq!(m.inbox(9_000), vec![7]);
        assert!(m.summary(9_000).contains("done"));
        assert_eq!(m.curve(9_000).len(), BUCKETS);
    }

    #[test]
    fn lanes_and_failing_retry() {
        let mut m = Mission::new(3, true, 0);
        assert_eq!(m.inbox(0), vec![1]);
        m.press("mi-retry-1", 0);
        assert_eq!(m.st(1, 7_000), St::Failed);
        let (s, e) = m.lane(0, 0).unwrap();
        assert!(s > 0.0 && (e - 1.0).abs() < 1e-9);
        assert!(m.lane(4, 0).is_some());
        assert_eq!(m.time(2, 60_000), "15m 00s");
    }
}
