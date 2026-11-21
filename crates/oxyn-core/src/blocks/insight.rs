//! AI insight card: a metric tile that can say why it moved. Explain streams a short reason and
//! ranks three drivers with contribution bars; as the answer reaches each driver, a numbered
//! marker lights up on the sparkline at the point it explains. Follow-up chips ask the next
//! question in place; the chart reads point by point with the arrow keys.

use super::{first_words, nth, rove, words_at};

/// The metric: label, period, value, previous, unit suffix.
pub const METRIC: (&str, &str, f64, f64) =
    ("Weekly active teams", "Last 12 weeks", 1_284.0, 1_102.0);
/// This period's series.
pub const SERIES: [f64; 12] = [
    1_090.0, 1_104.0, 1_098.0, 1_121.0, 1_117.0, 1_140.0, 1_162.0, 1_158.0, 1_201.0, 1_236.0,
    1_247.0, 1_284.0,
];
/// The previous period's series.
pub const PREV: [f64; 12] = [
    1_041.0, 1_052.0, 1_060.0, 1_049.0, 1_071.0, 1_078.0, 1_069.0, 1_083.0, 1_090.0, 1_088.0,
    1_097.0, 1_102.0,
];
/// Point labels.
pub const POINTS: [&str; 12] = [
    "Sep 1", "Sep 8", "Sep 15", "Sep 22", "Sep 29", "Oct 6", "Oct 13", "Oct 20", "Oct 27", "Nov 3",
    "Nov 10", "Nov 17",
];

/// An answer: text segments (with the driver each one lights up) and drivers (label, signed
/// contribution in teams, point index it marks, brand mark if a product is named).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Answer {
    /// Segments: text and driver index.
    pub segments: &'static [(&'static str, Option<usize>)],
    /// Drivers: label, contribution, point, brand.
    pub drivers: &'static [(&'static str, i32, usize, &'static str)],
}

/// The main answer and the follow-ups (label, answer).
pub const ANSWERS: [(&str, Answer); 3] = [
    (
        "Why did it change?",
        Answer {
            segments: &[
                ("Active teams rose 16.5% on the previous 12 weeks.", None),
                (
                    "Most of it came from the Slack integration launch on Oct 27, which brought back teams that had gone quiet.",
                    Some(0),
                ),
                (
                    "The new onboarding checklist lifted week-one activation from Oct 6.",
                    Some(1),
                ),
                (
                    "A small loss came from the end of the education pilot in mid October.",
                    Some(2),
                ),
            ],
            drivers: &[
                ("Slack integration launch", 118, 8, "slack"),
                ("Onboarding checklist", 87, 5, ""),
                ("Education pilot ended", -23, 6, ""),
            ],
        },
    ),
    (
        "Which segment grew most?",
        Answer {
            segments: &[
                (
                    "Teams of 10 to 50 people grew fastest, adding 96 active teams.",
                    Some(0),
                ),
                (
                    "Teams under 10 added 61, mostly through the new templates.",
                    Some(1),
                ),
                (
                    "Enterprise teams were flat apart from one large workspace that paused.",
                    Some(2),
                ),
            ],
            drivers: &[
                ("Teams of 10 to 50", 96, 9, ""),
                ("Teams under 10", 61, 7, ""),
                ("Enterprise", -4, 10, ""),
            ],
        },
    ),
    (
        "Will it keep going?",
        Answer {
            segments: &[
                ("The last four weeks grew about 30 teams a week.", Some(0)),
                (
                    "If that holds, the next 12 weeks end near 1,600 active teams.",
                    Some(1),
                ),
                (
                    "Holidays in late December usually cut activity by around 8% for two weeks.",
                    Some(2),
                ),
            ],
            drivers: &[
                ("Recent weekly growth", 30, 11, ""),
                ("Projected trend", 316, 11, ""),
                ("Holiday dip", -100, 11, ""),
            ],
        },
    ),
];

/// Streaming speed: words per tick.
pub const PER: usize = 2;

/// The live state of the card.
#[derive(Clone, Debug, PartialEq)]
pub struct Insight {
    /// The answer shown (index into [`ANSWERS`]) and when it started streaming.
    pub answer: Option<(usize, u64)>,
    /// Stopped at word n.
    pub stopped: Option<usize>,
    /// The chart readout (point index).
    pub point: Option<usize>,
    /// The first answer fails (the error scenario).
    pub fail: bool,
    /// Failed at (shows the error row).
    pub error: bool,
}

impl Insight {
    /// A card, optionally already explaining at `now`.
    pub fn new(auto: bool, fail: bool, now: u64) -> Self {
        let mut s = Self {
            answer: None,
            stopped: None,
            point: None,
            fail,
            error: false,
        };
        if auto {
            s.press("ai-explain", now);
        }
        s
    }

    /// The change in percent.
    pub fn delta(&self) -> f64 {
        (METRIC.2 - METRIC.3) / METRIC.3 * 100.0
    }

    /// The full text of answer `a`.
    pub fn full(a: usize) -> String {
        ANSWERS[a]
            .1
            .segments
            .iter()
            .map(|s| s.0)
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Words visible at `now`.
    pub fn words(&self, now: u64) -> usize {
        let Some((a, t)) = self.answer else { return 0 };
        let full = Self::full(a);
        let n = words_at(&full, now.saturating_sub(t), PER);
        self.stopped.map_or(n, |s| s.min(n))
    }

    /// Streaming now.
    pub fn streaming(&self, now: u64) -> bool {
        self.answer.is_some_and(|(a, _)| {
            self.stopped.is_none()
                && !self.error
                && self.words(now) < Self::full(a).split_whitespace().count()
        })
    }

    /// Segments as shown at `now`: visible text and driver.
    pub fn segments(&self, now: u64) -> Vec<(String, Option<usize>)> {
        let Some((a, _)) = self.answer else {
            return Vec::new();
        };
        let mut left = self.words(now);
        let mut out = Vec::new();
        for &(text, d) in ANSWERS[a].1.segments {
            if left == 0 {
                break;
            }
            let n = text.split_whitespace().count();
            out.push((first_words(text, left.min(n)), d));
            left = left.saturating_sub(n);
        }
        out
    }

    /// Drivers reached at `now` (the answer has named them).
    pub fn reached(&self, now: u64) -> usize {
        let Some((a, _)) = self.answer else { return 0 };
        let mut seen = 0;
        let segs = self.segments(now);
        for (i, &(text, d)) in ANSWERS[a].1.segments.iter().enumerate() {
            if let (Some(s), Some(d)) = (segs.get(i), d)
                && !s.0.is_empty()
                && s.0.split_whitespace().count() * 2 >= text.split_whitespace().count()
            {
                seen = seen.max(d + 1);
            }
        }
        if self.answer.is_some() && !self.streaming(now) && self.stopped.is_none() && !self.error {
            ANSWERS[a].1.drivers.len()
        } else {
            seen
        }
    }

    /// Presses: `ai-explain` (Explain, Stop or Hide), `ai-follow-<i>`, `ai-retry`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if id == "ai-explain" {
            if self.streaming(now) {
                self.stopped = Some(self.words(now));
                return Some("Stopped".into());
            }
            if self.answer.is_some() || self.error {
                self.answer = None;
                self.stopped = None;
                self.error = false;
                return Some("Explanation hidden".into());
            }
            if self.fail {
                self.fail = false;
                self.error = true;
                return Some("Couldn't explain the change. Retry".into());
            }
            self.answer = Some((0, now));
            return Some("Explaining. The answer is simulated from sample data".into());
        }
        if id == "ai-retry" && self.error {
            self.error = false;
            self.answer = Some((0, now));
            return Some("Explaining".into());
        }
        if let Some(i) = nth(id, "ai-follow-") {
            if self.streaming(now) || i == 0 || i >= ANSWERS.len() {
                return None;
            }
            self.answer = Some((i, now));
            self.stopped = None;
            return Some(ANSWERS[i].0.to_string());
        }
        None
    }

    /// Keys on the chart: Left, Right, Home, End read points; Escape clears.
    pub fn key(&mut self, key: &str, _now: u64) -> Option<String> {
        let k = super::norm_key(key);
        if k == "Escape" {
            self.point = None;
            return None;
        }
        if !matches!(k, "ArrowLeft" | "ArrowRight" | "Home" | "End") {
            return None;
        }
        let i = rove(self.point.unwrap_or(SERIES.len() - 1), SERIES.len(), k);
        self.point = Some(i);
        Some(format!(
            "{}: {} active teams, {} the period before",
            POINTS[i], SERIES[i], PREV[i]
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn explain_stream_follow_up() {
        let mut c = Insight::new(false, true, 0);
        c.press("ai-explain", 0);
        assert!(c.error);
        c.press("ai-retry", 100);
        assert!(c.streaming(200));
        assert_eq!(c.reached(200), 0);
        assert!(!c.streaming(5_000));
        assert_eq!(c.reached(5_000), 3);
        c.press("ai-follow-2", 5_000);
        assert_eq!(c.answer.map(|a| a.0), Some(2));
        c.press("ai-explain", 5_100);
        assert!(c.stopped.is_some());
        c.press("ai-explain", 5_200);
        assert!(c.answer.is_none());
        c.key("ArrowLeft", 0);
        assert_eq!(c.point, Some(10));
        assert!((c.delta() - 16.5).abs() < 0.1);
    }
}
