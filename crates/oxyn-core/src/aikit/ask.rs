//! Ask your data: a question about a database becomes a chart. The SQL a model writes streams
//! into a foldable panel, the query runs with a rolling row count, and the result builds as
//! stacked bars, lines or a table with a one-line reading of it underneath.

use super::{hash, norm_key, nth, rove, stream_ms, thousands, words_at};

/// The database the questions run against: name and dialect.
pub const SOURCE: (&str, &str) = ("analytics", "PostgreSQL");
/// Questions offered when the field is empty or nothing matched.
pub const SUGGESTIONS: [&str; 3] = [
    "Weekly signups by plan since August",
    "Monthly recurring revenue by plan this year",
    "Churn rate by plan, last 12 weeks",
];
/// Views.
pub const CHARTS: [&str; 3] = ["Bars", "Lines", "Table"];
/// A beat before the SQL starts, ms.
pub const THINK_MS: u64 = 450;
/// Query time, ms.
pub const RUN_MS: u64 = 1_300;
/// Bars rise / lines draw, ms.
pub const BUILD_MS: u64 = 460;

/// One chartable result.
#[derive(Clone, Debug, PartialEq)]
pub struct Dataset {
    /// Chart title.
    pub title: &'static str,
    /// Quiet line under it.
    pub subtitle: &'static str,
    /// The SQL the model writes.
    pub sql: &'static str,
    /// Category axis label.
    pub category: &'static str,
    /// Categories (x axis).
    pub cats: Vec<&'static str>,
    /// Series names and hues.
    pub series: Vec<(&'static str, &'static str)>,
    /// Values by category, then series.
    pub values: Vec<Vec<f64>>,
    /// Series add up.
    pub stacked: bool,
    /// Legend shows the series total (`sum`) or its latest value (`last`).
    pub legend: &'static str,
    /// Number format: `count`, `kusd` or `pct`.
    pub fmt: &'static str,
    /// The one-line reading.
    pub insight: &'static str,
    /// Rows scanned.
    pub scanned: u64,
    /// Query time, ms.
    pub took: u64,
    /// The view it opens on.
    pub chart: usize,
}

fn jitter(seed: &str, i: usize, amp: f64) -> f64 {
    let h = hash(&format!("{seed}{i}"));
    ((h % 1000) as f64 / 1000.0 - 0.5) * 2.0 * amp
}

/// The result for suggestion `i`.
pub fn dataset(i: usize) -> Dataset {
    match i {
        1 => {
            let cats = vec![
                "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct",
            ];
            let values = (0..cats.len())
                .map(|m| {
                    let t = m as f64;
                    vec![
                        118.0 + t * 9.6 + jitter("pro", m, 3.0),
                        76.0 + t * 7.4 + jitter("team", m, 3.0),
                        38.0 + t * 6.1 + jitter("ent", m, 2.0),
                    ]
                })
                .collect();
            Dataset {
                title: "Monthly recurring revenue by plan",
                subtitle: "Jan to Oct 2026 · end of month",
                sql: "SELECT date_trunc('month', day) AS month, plan,\n       round(sum(mrr_cents) / 100000.0, 1) AS mrr_k\nFROM revenue_daily\nWHERE day >= '2026-01-01' AND day = (date_trunc('month', day) + interval '1 month - 1 day')::date\nGROUP BY 1, 2\nORDER BY 1, 2;",
                category: "Month",
                cats,
                series: vec![("Pro", "blue"), ("Team", "violet"), ("Enterprise", "teal")],
                values,
                stacked: false,
                legend: "last",
                fmt: "kusd",
                insight: "Every plan grew each month; Enterprise more than doubled since January and now brings in about a fifth of revenue.",
                scanned: 912_400,
                took: 386,
                chart: 1,
            }
        }
        2 => {
            let cats = vec![
                "Aug 3", "Aug 10", "Aug 17", "Aug 24", "Aug 31", "Sep 7", "Sep 14", "Sep 21",
                "Sep 28", "Oct 5", "Oct 12", "Oct 19",
            ];
            let values = (0..cats.len())
                .map(|w| {
                    let t = w as f64;
                    vec![
                        4.8 - t * 0.08 + jitter("cp", w, 0.3),
                        2.6 - t * 0.05 + jitter("ct", w, 0.2),
                        0.9 + jitter("ce", w, 0.15),
                    ]
                })
                .collect();
            Dataset {
                title: "Churn rate by plan",
                subtitle: "Last 12 weeks · share of paying workspaces",
                sql: "SELECT date_trunc('week', cancelled_on) AS week, plan,\n       round(100.0 * count(*) / max(active_at_start), 1) AS churn_pct\nFROM cancellations c\nJOIN weekly_active w USING (plan, week)\nWHERE cancelled_on >= current_date - interval '12 weeks'\nGROUP BY 1, 2\nORDER BY 1, 2;",
                category: "Week",
                cats,
                series: vec![("Pro", "blue"), ("Team", "violet"), ("Enterprise", "teal")],
                values,
                stacked: false,
                legend: "none",
                fmt: "pct",
                insight: "Pro churn eased from about 4.8% to under 4%, while Enterprise stayed flat near 1% a week.",
                scanned: 48_210,
                took: 214,
                chart: 1,
            }
        }
        _ => {
            let cats = vec![
                "Aug 3", "Aug 10", "Aug 17", "Aug 24", "Aug 31", "Sep 7", "Sep 14", "Sep 21",
                "Sep 28", "Oct 5", "Oct 12", "Oct 19",
            ];
            let values = (0..cats.len())
                .map(|w| {
                    let t = w as f64;
                    let launch = if w == 7 { 22.0 } else { 0.0 };
                    vec![
                        (52.0 + t * 2.4 + launch + jitter("f", w, 5.0)).round(),
                        (21.0 + t * 1.1 + launch * 0.4 + jitter("p", w, 3.0)).round(),
                        (8.0 + t * 0.5 + jitter("t", w, 2.0)).round(),
                        (2.0 + (t * 0.15).round() + jitter("e", w, 1.0))
                            .round()
                            .max(0.0),
                    ]
                })
                .collect();
            Dataset {
                title: "Weekly signups by plan",
                subtitle: "Aug 3 to Oct 19 · new workspaces",
                sql: "SELECT date_trunc('week', created_at) AS week, plan,\n       count(*) AS signups\nFROM workspaces\nWHERE created_at >= '2026-08-03'\nGROUP BY 1, 2\nORDER BY 1, 2;",
                category: "Week",
                cats,
                series: vec![
                    ("Free", "neutral"),
                    ("Pro", "blue"),
                    ("Team", "violet"),
                    ("Enterprise", "teal"),
                ],
                values,
                stacked: true,
                legend: "sum",
                fmt: "count",
                insight: "Signups climbed steadily and peaked the week of Sep 21, when the launch post pulled in the most Free and Pro workspaces.",
                scanned: 128_904,
                took: 248,
                chart: 0,
            }
        }
    }
}

/// Which suggestion a question matches, if any.
pub fn match_question(q: &str) -> Option<usize> {
    let q = q.to_lowercase();
    if q.contains("signup") || q.contains("sign up") || q.contains("sign-up") {
        Some(0)
    } else if q.contains("revenue") || q.contains("mrr") {
        Some(1)
    } else if q.contains("churn") || q.contains("cancel") {
        Some(2)
    } else {
        None
    }
}

impl Dataset {
    /// A value in the dataset's format (`1,284`, `$258k`, `3.9%`).
    pub fn fmt_v(&self, v: f64) -> String {
        match self.fmt {
            "kusd" => format!("${}k", v.round() as u64),
            "pct" => format!("{v:.1}%"),
            _ => thousands(v.round().max(0.0) as u64),
        }
    }
    /// Total of a category.
    pub fn total(&self, c: usize) -> f64 {
        self.values[c].iter().sum()
    }
    /// Largest value the y axis needs.
    pub fn max(&self) -> f64 {
        (0..self.cats.len())
            .map(|c| {
                if self.stacked {
                    self.total(c)
                } else {
                    self.values[c].iter().cloned().fold(0.0, f64::max)
                }
            })
            .fold(0.0, f64::max)
    }
    /// What follows each legend label.
    pub fn legend_value(&self, s: usize) -> Option<String> {
        match self.legend {
            "sum" => Some(self.fmt_v(self.values.iter().map(|r| r[s]).sum())),
            "last" => self.values.last().map(|r| self.fmt_v(r[s])),
            _ => None,
        }
    }
    /// The readout of one category (`<week>: <plan> <n>, …, total <n>`).
    pub fn readout(&self, c: usize) -> String {
        let parts: Vec<String> = self
            .series
            .iter()
            .enumerate()
            .map(|(i, s)| format!("{} {}", s.0, self.fmt_v(self.values[c][i])))
            .collect();
        let mut s = format!("{}: {}", self.cats[c], parts.join(", "));
        if self.stacked {
            s.push_str(&format!(", total {}", self.fmt_v(self.total(c))));
        }
        s
    }
    /// Rows the query returned (one per category and series).
    pub fn rows(&self) -> usize {
        self.cats.len() * self.series.len()
    }
}

/// Where the card is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Nothing asked yet.
    Idle,
    /// The model is writing SQL.
    Writing,
    /// The query is running.
    Running,
    /// The chart is shown.
    Done,
    /// The question matched no tables.
    NoMatch,
    /// Writing or running failed.
    Error,
    /// The person stopped the run.
    Stopped,
}

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct Ask {
    /// The question in the field.
    pub q: String,
    /// The dataset being answered and when the run started.
    pub run: Option<(usize, u64)>,
    /// The last question matched nothing.
    pub no_match: bool,
    /// Stopped at this clock.
    pub stopped: Option<u64>,
    /// The run fails while running (the error scenario).
    pub fail: bool,
    /// View picked by the person.
    pub chart: Option<usize>,
    /// The person opened or closed the SQL panel.
    pub sql_open: Option<bool>,
    /// Keep the SQL open after the query starts.
    pub keep_sql: bool,
    /// Category under the readout.
    pub cursor: Option<usize>,
    /// SQL copied at.
    pub copied: Option<u64>,
}

impl Ask {
    /// A card with `q` in the field, asked at `now` when `auto`.
    pub fn new(q: &str, auto: bool, fail: bool, keep_sql: bool, now: u64) -> Self {
        let mut a = Self {
            q: q.to_string(),
            run: None,
            no_match: false,
            stopped: None,
            fail,
            chart: None,
            sql_open: None,
            keep_sql,
            cursor: None,
            copied: None,
        };
        if auto {
            a.ask(now);
        }
        a
    }

    fn ask(&mut self, now: u64) -> Option<String> {
        self.stopped = None;
        self.sql_open = None;
        self.cursor = None;
        self.chart = None;
        match match_question(&self.q) {
            Some(i) => {
                self.no_match = false;
                self.run = Some((i, now));
                Some("Writing SQL".into())
            }
            None => {
                self.run = None;
                self.no_match = true;
                Some("No tables match that question. Try one of the suggestions.".into())
            }
        }
    }

    /// The dataset being answered.
    pub fn data(&self) -> Option<Dataset> {
        self.run.map(|r| dataset(r.0))
    }

    fn timing(&self) -> Option<(u64, u64, u64)> {
        let (i, t) = self.run?;
        let d = dataset(i);
        let w = t + THINK_MS + stream_ms(d.sql, 4);
        Some((t, w, w + RUN_MS))
    }

    /// The phase at `now`.
    pub fn phase(&self, now: u64) -> Phase {
        if self.no_match {
            return Phase::NoMatch;
        }
        let Some((_, w, r)) = self.timing() else {
            return Phase::Idle;
        };
        let at = self.stopped.unwrap_or(now);
        let p = if at < w {
            Phase::Writing
        } else if self.fail && at >= w + RUN_MS / 2 {
            Phase::Error
        } else if at < r {
            Phase::Running
        } else {
            Phase::Done
        };
        if self.stopped.is_some() && matches!(p, Phase::Writing | Phase::Running) {
            Phase::Stopped
        } else {
            p
        }
    }

    /// The run is writing or running.
    pub fn busy(&self, now: u64) -> bool {
        matches!(self.phase(now), Phase::Writing | Phase::Running)
    }

    /// Words of SQL visible.
    pub fn sql_words(&self, now: u64) -> usize {
        let Some((t, _, _)) = self.timing() else {
            return 0;
        };
        let at = self.stopped.unwrap_or(now);
        let el = at.saturating_sub(t);
        if el < THINK_MS {
            return 0;
        }
        words_at(self.data().map(|d| d.sql).unwrap_or(""), el - THINK_MS, 4)
    }

    /// Rows scanned so far.
    pub fn scanned(&self, now: u64) -> u64 {
        let (Some((_, w, _)), Some(d)) = (self.timing(), self.data()) else {
            return 0;
        };
        let at = self.stopped.unwrap_or(now);
        let p = (at.saturating_sub(w) as f64 / RUN_MS as f64).clamp(0.0, 1.0);
        let p = 1.0 - (1.0 - p).powi(2);
        (d.scanned as f64 * p) as u64
    }

    /// Chart build progress 0..1.
    pub fn built(&self, now: u64) -> f64 {
        match self.timing() {
            Some((_, _, r)) if self.phase(now) == Phase::Done => {
                (now.saturating_sub(r) as f64 / BUILD_MS as f64).min(1.0)
            }
            _ => 0.0,
        }
    }

    /// Words of the insight visible.
    pub fn insight_words(&self, now: u64) -> usize {
        match (self.timing(), self.data()) {
            (Some((_, _, r)), Some(d)) if self.phase(now) == Phase::Done => {
                let el = now.saturating_sub(r + BUILD_MS);
                if now < r + BUILD_MS {
                    0
                } else {
                    words_at(d.insight, el, 3)
                }
            }
            _ => 0,
        }
    }

    /// The SQL panel is open.
    pub fn sql_shown(&self, now: u64) -> bool {
        self.sql_open
            .unwrap_or(self.keep_sql || self.phase(now) == Phase::Writing)
    }

    /// The view on screen.
    pub fn view(&self) -> usize {
        self.chart
            .unwrap_or_else(|| self.data().map(|d| d.chart).unwrap_or(0))
    }

    /// The SQL row's meta (`PostgreSQL · 48 rows · 248ms`).
    pub fn meta(&self, now: u64) -> String {
        match (self.phase(now), self.data()) {
            (Phase::Done, Some(d)) => format!("{} · {} rows · {}ms", SOURCE.1, d.rows(), d.took),
            (Phase::Running, _) => format!("{} · running", SOURCE.1),
            (Phase::Writing, _) => format!("{} · writing", SOURCE.1),
            (Phase::Error, _) => format!("{} · failed", SOURCE.1),
            (Phase::Stopped, _) => format!("{} · stopped", SOURCE.1),
            _ => SOURCE.1.to_string(),
        }
    }

    /// Replaces the question text.
    pub fn set_q(&mut self, q: &str) {
        self.q = q.to_string();
    }

    /// Presses: `ay-ask` (stops while busy), `ay-stop`, `ay-sql`, `ay-copy`, `ay-chart-<i>`,
    /// `ay-sug-<i>`, `ay-retry`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "ay-chart-") {
            let i = i.min(2);
            if i == self.view() {
                return None;
            }
            self.chart = Some(i);
            return Some(format!("{} view", CHARTS[i]));
        }
        if let Some(i) = nth(id, "ay-sug-") {
            self.q = SUGGESTIONS.get(i)?.to_string();
            return self.ask(now);
        }
        match id {
            "ay-ask" if self.busy(now) => self.press("ay-stop", now),
            "ay-ask" | "ay-retry" => {
                if id == "ay-retry" {
                    self.fail = false;
                }
                if self.q.trim().is_empty() {
                    return Some("Type a question first".into());
                }
                self.ask(now)
            }
            "ay-stop" if self.busy(now) => {
                self.stopped = Some(now);
                Some("Stopped. Nothing else will run.".into())
            }
            "ay-sql" => {
                let open = !self.sql_shown(now);
                self.sql_open = Some(open);
                Some(if open { "SQL shown" } else { "SQL hidden" }.into())
            }
            "ay-copy" => {
                self.copied = Some(now);
                Some("SQL copied".into())
            }
            _ => None,
        }
    }

    /// Keys: `field` (Enter asks or stops, Escape stops), `chart` (arrows, Home, End move the
    /// readout, Escape clears it), `view` (arrows switch the view).
    pub fn key(&mut self, part: &str, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        match (part, key) {
            ("field", "Enter") => self.press("ay-ask", now),
            ("field", "Escape") => self.press("ay-stop", now),
            ("chart", "Escape") => {
                self.cursor.take()?;
                Some("Readout cleared".into())
            }
            ("chart", _) => {
                let d = self.data()?;
                if self.phase(now) != Phase::Done {
                    return None;
                }
                let n = d.cats.len();
                let c = match self.cursor {
                    None => match key {
                        "End" | "ArrowLeft" => n - 1,
                        _ => 0,
                    },
                    Some(c) => rove(c, n, key),
                };
                self.cursor = Some(c);
                Some(d.readout(c))
            }
            ("view", _) => {
                let i = rove(self.view(), 3, key);
                self.press(&format!("ay-chart-{i}"), now)
            }
            _ => None,
        }
    }

    /// The polite summary once charted.
    pub fn summary(&self) -> Option<String> {
        let d = self.data()?;
        Some(format!("{}, {} rows. {}", d.title, d.rows(), d.insight))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn asks_streams_runs_and_charts() {
        let mut a = Ask::new(SUGGESTIONS[0], true, false, false, 0);
        assert_eq!(a.phase(100), Phase::Writing);
        assert!(a.sql_shown(100));
        let (_, w, r) = a.timing().unwrap();
        assert_eq!(a.phase(w + 10), Phase::Running);
        assert!(!a.sql_shown(w + 10));
        assert!(a.scanned(w + 600) > 0);
        assert_eq!(a.phase(r + 1), Phase::Done);
        assert!(a.insight_words(r + 5_000) > 10);
        a.key("chart", "ArrowRight", r + 100);
        assert_eq!(a.cursor, Some(0));
        assert!(a.key("chart", "End", r + 100).unwrap().contains("total"));
        a.key("view", "ArrowRight", r + 100);
        assert_eq!(a.view(), 1);
        a.set_q("favourite colour of the CEO");
        a.press("ay-ask", r + 200);
        assert_eq!(a.phase(r + 300), Phase::NoMatch);
        a.press("ay-sug-1", r + 400);
        assert_eq!(a.data().unwrap().fmt, "kusd");
        a.key("field", "Escape", r + 500);
        assert_eq!(a.phase(r + 9_000), Phase::Stopped);
        let mut f = Ask::new(SUGGESTIONS[2], true, true, true, 0);
        let (_, w, _) = f.timing().unwrap();
        assert_eq!(f.phase(w + RUN_MS), Phase::Error);
        f.press("ay-retry", w + RUN_MS);
        assert_eq!(f.phase(w + RUN_MS + 200), Phase::Writing);
        assert!(dataset(1).fmt_v(258.4) == "$258k" && dataset(2).fmt_v(3.94) == "3.9%");
    }
}
