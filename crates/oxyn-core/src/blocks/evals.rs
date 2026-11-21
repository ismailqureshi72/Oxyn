//! Eval suite: score a release candidate of a model-backed feature against the shipped version.
//! Results stream in case by case: one row of small cells per category fills (pass, fail, error,
//! each with its own glyph), the score counts up beside the baseline with the change in points,
//! and every regression and fix is called out with the case behind it. The case list filters to
//! all, failed, regressions or improved; opening a case shows input, expected behaviour, both
//! outputs and the judge's scores, and lets you accept the change or file it as a bug.

use super::{dur, norm_key, nth, rove, usd};

/// A case outcome.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Out {
    /// Passed.
    Pass,
    /// Failed.
    Fail,
    /// Errored (timeout, refusal to answer in format).
    Error,
}

impl Out {
    /// Word.
    pub fn word(self) -> &'static str {
        match self {
            Out::Pass => "pass",
            Out::Fail => "fail",
            Out::Error => "error",
        }
    }
}

/// Categories and their baseline pass rate (percent).
pub const CATEGORIES: [(&str, u32); 4] = [
    ("Refund questions", 88),
    ("Plan changes", 79),
    ("Tone and policy", 92),
    ("Escalations", 71),
];
/// Cases per category.
pub const PER: usize = 12;
/// One case scored, ms.
pub const CASE_MS: u64 = 90;
/// Candidate and baseline: version, brand, model.
pub const CANDIDATE: (&str, &str, &str) = ("v12", "claude", "Claude Sonnet 4.5");
/// The baseline.
pub const BASELINE: (&str, &str, &str) = ("v11", "openai", "GPT-5 mini");
/// The judge.
pub const JUDGE: (&str, &str) = ("openai", "GPT-5");
/// The feature under test.
pub const FEATURE: &str = "Billing support replies";
/// Earlier runs: version and score.
pub const HISTORY: [(&str, u32); 5] =
    [("v8", 71), ("v9", 76), ("v10", 80), ("v11", 83), ("v12", 0)];
/// Filters.
pub const FILTERS: [&str; 4] = ["All", "Failed", "Regressions", "Improved"];
/// Rubric criteria.
pub const RUBRIC: [&str; 3] = ["Correct policy", "Answers the question", "Tone"];

/// One case: category, title, baseline outcome, new outcome.
#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    /// Category index.
    pub cat: usize,
    /// Title.
    pub title: String,
    /// Baseline outcome.
    pub base: Out,
    /// New outcome.
    pub new: Out,
}

const TITLES: [&str; 6] = [
    "Annual plan refund after 40 days",
    "Refund to an expired card",
    "Downgrade mid-cycle with seats",
    "Customer asks to speak to a person",
    "Angry message about a double charge",
    "Discount for a non-profit",
];

/// The suite's cases, deterministic.
pub fn cases() -> Vec<Case> {
    let mut v = Vec::new();
    for (c, &(_, base_pct)) in CATEGORIES.iter().enumerate() {
        for k in 0..PER {
            let base_n = base_pct as usize * PER / 100;
            let new_n = (base_pct + [6, 9, 2, 13][c]).min(100) as usize * PER / 100;
            let base = if (k * 7) % PER < base_n {
                Out::Pass
            } else {
                Out::Fail
            };
            let mut new = if (k * 7 + [3, 5, 2, 7][c]) % PER < new_n {
                Out::Pass
            } else {
                Out::Fail
            };
            if c == 3 && k == 7 {
                new = Out::Error;
            }
            let title = format!("{} #{}", TITLES[(c + k) % TITLES.len()], k + 1);
            v.push(Case {
                cat: c,
                title,
                base,
                new,
            });
        }
    }
    // One clear regression to talk about.
    v[3].base = Out::Pass;
    v[3].new = Out::Fail;
    v
}

/// The live state of the suite.
#[derive(Clone, Debug, PartialEq)]
pub struct Evals {
    /// Cases.
    pub cases: Vec<Case>,
    /// Run started at (host ms).
    pub run: Option<u64>,
    /// Stopped at case n.
    pub stopped: Option<usize>,
    /// Filter.
    pub filter: usize,
    /// Open case.
    pub open: Option<usize>,
    /// Triage by case: 1 marked as expected, 2 issue opened.
    pub triage: Vec<u8>,
    /// Run history readout index.
    pub hist: usize,
    /// The judge errors halfway (the error scenario).
    pub fail: bool,
}

impl Evals {
    /// A suite at `now`; `done` opens it with the run finished.
    pub fn new(done: bool, fail: bool, now: u64) -> Self {
        let cases = cases();
        let n = cases.len();
        Self {
            cases,
            run: done.then(|| now.saturating_sub(n as u64 * CASE_MS + 1)),
            stopped: None,
            filter: 0,
            open: None,
            triage: vec![0; n],
            hist: HISTORY.len() - 1,
            fail,
        }
    }

    /// Cases scored at `now`.
    pub fn scored(&self, now: u64) -> usize {
        let n = self.cases.len();
        let Some(t) = self.run else { return 0 };
        let k = ((now.saturating_sub(t)) / CASE_MS) as usize;
        let k = k.min(n);
        let k = if self.fail { k.min(n / 2) } else { k };
        self.stopped.map_or(k, |s| s.min(k))
    }

    /// The run's state word.
    pub fn state(&self, now: u64) -> &'static str {
        let s = self.scored(now);
        match self.run {
            None => "Not run",
            Some(_) if self.stopped.is_some() => "Stopped",
            Some(_) if self.fail && s == self.cases.len() / 2 => "Judge failed",
            Some(_) if s == self.cases.len() => "Finished",
            _ => "Running",
        }
    }

    /// Outcome of case `i` at `now` (`None` while not yet scored). Cases are scored round
    /// robin across categories so every row fills together.
    pub fn out(&self, i: usize, now: u64) -> Option<Out> {
        let c = &self.cases[i];
        let order = (i % PER) * CATEGORIES.len() + c.cat;
        (order < self.scored(now)).then_some(c.new)
    }

    /// Category `c` at `now`: passed, failed, errors, pending, percent of scored.
    pub fn cat(&self, c: usize, now: u64) -> (usize, usize, usize, usize, u32) {
        let mut r = (0, 0, 0, 0);
        for i in (0..self.cases.len()).filter(|&i| self.cases[i].cat == c) {
            match self.out(i, now) {
                Some(Out::Pass) => r.0 += 1,
                Some(Out::Fail) => r.1 += 1,
                Some(Out::Error) => r.2 += 1,
                None => r.3 += 1,
            }
        }
        let done = r.0 + r.1 + r.2;
        let pct = (r.0 * 100_usize).checked_div(done).unwrap_or(0) as u32;
        (r.0, r.1, r.2, r.3, pct)
    }

    /// The candidate's score so far and the baseline's, percent.
    pub fn score(&self, now: u64) -> (u32, u32) {
        let s = self.scored(now);
        let n = self.cases.len();
        let pass = (0..n)
            .filter(|&i| self.out(i, now) == Some(Out::Pass))
            .count();
        let base = (0..n).filter(|&i| self.cases[i].base == Out::Pass).count();
        (
            (pass * 100).checked_div(s).unwrap_or(0) as u32,
            (base * 100 / n) as u32,
        )
    }

    /// A category's summary for assistive technology.
    pub fn cat_label(&self, c: usize, now: u64) -> String {
        let (p, f, e, w, pct) = self.cat(c, now);
        let d = pct as i64 - CATEGORIES[c].1 as i64;
        format!(
            "{}: {p} passed, {f} failed, {e} errors, {w} not yet scored, {pct}%, {d:+} points against the baseline",
            CATEGORIES[c].0
        )
    }

    /// Regression (passed before, not now) and improvement at `now`.
    pub fn change(&self, i: usize, now: u64) -> Option<bool> {
        let o = self.out(i, now)?;
        let c = &self.cases[i];
        match (c.base, o) {
            (Out::Pass, Out::Fail | Out::Error) => Some(false),
            (Out::Fail, Out::Pass) => Some(true),
            _ => None,
        }
    }

    /// Cases in the list at `now`.
    pub fn list(&self, now: u64) -> Vec<usize> {
        (0..self.cases.len())
            .filter(|&i| {
                let o = self.out(i, now);
                o.is_some()
                    && match self.filter {
                        1 => matches!(o, Some(Out::Fail | Out::Error)),
                        2 => self.change(i, now) == Some(false),
                        3 => self.change(i, now) == Some(true),
                        _ => true,
                    }
            })
            .collect()
    }

    /// Counts per filter at `now`.
    pub fn counts(&self, now: u64) -> [usize; 4] {
        let mut s = self.clone();
        let mut c = [0; 4];
        for (f, x) in c.iter_mut().enumerate() {
            s.filter = f;
            *x = s.list(now).len();
        }
        c
    }

    /// The summary: cost and time of the run so far.
    pub fn cost_time(&self, now: u64) -> (String, String) {
        let s = self.scored(now) as f64;
        (usd(s * 0.0125), dur(self.scored(now) as u64 * 1_900))
    }

    /// Outputs of case `i`: input, expected, baseline output, new output, judge scores (1–5)
    /// for baseline and new, rationale.
    pub fn detail(
        &self,
        i: usize,
    ) -> (String, &'static str, String, String, [(u8, u8); 3], String) {
        let c = &self.cases[i];
        let input = format!(
            "{}: \"I paid for a year in March and want my money back now.\"",
            c.title
        );
        let expected = "Explain the 30-day rule, offer a refund of the unused months and link the billing page.";
        let base =
            "Annual plans can't be refunded. You can cancel so it doesn't renew.".to_string();
        let new = match c.new {
            Out::Pass => "Annual plans are refundable in full within 30 days. After that we refund the unused months; I can start that for you.".to_string(),
            Out::Fail => "Annual plans can't be refunded after 30 days. You can cancel so it doesn't renew next year.".to_string(),
            Out::Error => "(no answer: the request timed out after 30 s)".to_string(),
        };
        let sc = |o: Out| if o == Out::Pass { 4 } else { 2 };
        let scores = [
            (sc(c.base), sc(c.new)),
            (4, if c.new == Out::Pass { 5 } else { 3 }),
            (4, 4),
        ];
        let why = match c.new {
            Out::Pass => "States the policy correctly and offers the refund.".to_string(),
            Out::Fail => "Misses the refund of the unused months the policy allows.".to_string(),
            Out::Error => "No answer to judge.".to_string(),
        };
        (input, expected, base, new, scores, why)
    }

    /// Presses: `ev-run`, `ev-f-<i>`, `ev-open-<i>`, `ev-expected-<i>`, `ev-issue-<i>`,
    /// `ev-undo-<i>`, `ev-hist-<i>`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if id == "ev-run" {
            let st = self.state(now);
            if st == "Running" {
                self.stopped = Some(self.scored(now));
                return Some(format!("Stopped after {} cases", self.scored(now)));
            }
            self.run = Some(now);
            self.stopped = None;
            self.open = None;
            return Some(format!(
                "Running {} cases on {}. Simulated in the preview",
                self.cases.len(),
                CANDIDATE.0
            ));
        }
        if let Some(i) = nth(id, "ev-f-") {
            self.filter = i.min(FILTERS.len() - 1);
            return Some(format!(
                "{}: {} cases",
                FILTERS[self.filter],
                self.list(now).len()
            ));
        }
        if let Some(i) = nth(id, "ev-open-") {
            self.open = if self.open == Some(i) { None } else { Some(i) };
            return None;
        }
        if let Some(i) = nth(id, "ev-hist-") {
            self.hist = i.min(HISTORY.len() - 1);
            let (v, s) = HISTORY[self.hist];
            let s = if s == 0 { self.score(now).0 } else { s };
            return Some(format!("{v}: {s}%"));
        }
        for (pre, val, word) in [
            ("ev-expected-", 1u8, "Marked as expected"),
            ("ev-issue-", 2, "Issue opened (simulated)"),
            ("ev-undo-", 0, "Undone"),
        ] {
            if let Some(i) = nth(id, pre) {
                let t = self.triage.get_mut(i)?;
                if (val == 0) == (*t == 0) {
                    return None;
                }
                *t = val;
                return Some(word.to_string());
            }
        }
        None
    }

    /// Keys on the filter (Left, Right, Home, End).
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = norm_key(key);
        let i = rove(self.filter, FILTERS.len(), k);
        (i != self.filter)
            .then(|| self.press(&format!("ev-f-{i}"), now))
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_filters_and_triage() {
        let mut e = Evals::new(false, false, 0);
        assert_eq!(e.state(0), "Not run");
        e.press("ev-run", 0);
        assert_eq!(e.state(10 * CASE_MS), "Running");
        assert_eq!(e.scored(10 * CASE_MS), 10);
        let all = (48 * CASE_MS) + 10;
        assert_eq!(e.state(all), "Finished");
        let c = e.counts(all);
        assert_eq!(c[0], 48);
        assert!(c[2] >= 1);
        assert!(e.list(all).len() == 48);
        e.press("ev-f-2", all);
        assert!(e.list(all).contains(&3));
        e.press("ev-expected-3", all);
        assert!(e.press("ev-issue-3", all).is_none());
        e.press("ev-undo-3", all);
        assert_eq!(e.triage[3], 0);
        let (s, b) = e.score(all);
        assert!(s > b);
        assert!(e.cat_label(0, all).contains("points"));
        let mut f = Evals::new(false, true, 0);
        f.press("ev-run", 0);
        assert_eq!(f.state(all), "Judge failed");
        e.key("End", all);
        assert_eq!(e.filter, 3);
    }
}
