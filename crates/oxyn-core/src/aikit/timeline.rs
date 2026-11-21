//! Agent timeline: one orchestrated run on a shared time axis. The orchestrator sits on top and
//! its subagents below; spans are thinking (a tint of the agent colour), tool calls (solid) and
//! waiting (hatched); handoffs connect rows; a playhead replays the run while the header counts
//! time, tokens and cost and settles into a summary.

use super::{norm_key, secs, thousands, usd};

/// What a span is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanKind {
    /// The model thinking.
    Thinking,
    /// A tool call.
    Tool,
    /// Blocked on other agents.
    Waiting,
}

impl SpanKind {
    /// Lowercase id (`thinking`, `tool`, `waiting`).
    pub fn id(self) -> &'static str {
        match self {
            Self::Thinking => "thinking",
            Self::Tool => "tool",
            Self::Waiting => "waiting",
        }
    }
    /// Label (`Thinking`, `Tool call`, `Waiting`).
    pub fn label(self) -> &'static str {
        match self {
            Self::Thinking => "Thinking",
            Self::Tool => "Tool call",
            Self::Waiting => "Waiting",
        }
    }
}

/// One agent row.
#[derive(Clone, Debug, PartialEq)]
pub struct Agent {
    /// Role name.
    pub name: &'static str,
    /// Model name (its mark is resolved from it).
    pub model: &'static str,
    /// Hue key for the row colour (`neutral`, `blue`, `violet`, `teal`, `amber`).
    pub hue: &'static str,
    /// Blended USD per million tokens.
    pub price: f64,
}

/// One span of work.
#[derive(Clone, Debug, PartialEq)]
pub struct Span {
    /// Row index.
    pub agent: usize,
    /// Kind.
    pub kind: SpanKind,
    /// Start, ms from run start.
    pub start: u64,
    /// End, ms from run start.
    pub end: u64,
    /// What it did.
    pub label: &'static str,
    /// Tool name for tool calls (a brand when it has a mark).
    pub tool: &'static str,
    /// Tokens used.
    pub tokens: u64,
    /// The call failed.
    pub failed: bool,
    /// The call retried a failed one.
    pub retry: bool,
    /// Detail (failure message).
    pub detail: &'static str,
}

/// Work handed between two rows.
#[derive(Clone, Debug, PartialEq)]
pub struct Handoff {
    /// From row.
    pub from: usize,
    /// To row.
    pub to: usize,
    /// Time, ms.
    pub at: u64,
    /// What was handed over.
    pub label: &'static str,
}

/// Where a span is at the playhead.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SpanAt {
    /// Not started yet.
    Future,
    /// Running, with the drawn fraction of its own length.
    Active(f64),
    /// Finished.
    Done,
    /// Finished and failed.
    Failed,
}

#[allow(clippy::too_many_arguments)]
fn sp(
    agent: usize,
    kind: SpanKind,
    start: u64,
    end: u64,
    label: &'static str,
    tool: &'static str,
    tokens: u64,
) -> Span {
    Span {
        agent,
        kind,
        start,
        end,
        label,
        tool,
        tokens,
        failed: false,
        retry: false,
        detail: "",
    }
}

/// The sample run: five agents digging into a drop in checkout conversion, with one query that
/// fails and is retried.
pub fn sample() -> (Vec<Agent>, Vec<Span>, Vec<Handoff>) {
    use SpanKind::*;
    let agents = vec![
        Agent {
            name: "Orchestrator",
            model: "Claude Opus 4.1",
            hue: "neutral",
            price: 18.0,
        },
        Agent {
            name: "Researcher",
            model: "Claude Sonnet 4.5",
            hue: "blue",
            price: 6.0,
        },
        Agent {
            name: "Analyst",
            model: "GPT-5 mini",
            hue: "violet",
            price: 1.0,
        },
        Agent {
            name: "Checker",
            model: "Gemini 2.5 Flash",
            hue: "teal",
            price: 0.6,
        },
        Agent {
            name: "Writer",
            model: "Mistral Medium",
            hue: "amber",
            price: 1.5,
        },
    ];
    let mut failed = sp(
        2,
        Tool,
        2_300,
        5_100,
        "Queried checkout events",
        "PostgreSQL",
        900,
    );
    failed.failed = true;
    failed.detail = "Timed out after 2.8s";
    let mut retry = sp(
        2,
        Tool,
        5_300,
        9_800,
        "Pulled conversion by step",
        "PostgreSQL",
        4_400,
    );
    retry.retry = true;
    retry.detail = "Retried with a narrower date range";
    let spans = vec![
        sp(
            0,
            Thinking,
            0,
            1_800,
            "Planned the investigation",
            "",
            2_100,
        ),
        sp(
            0,
            Waiting,
            1_800,
            15_800,
            "Waiting on the researcher and analyst",
            "",
            0,
        ),
        sp(
            1,
            Tool,
            2_000,
            6_400,
            "Read six release notes",
            "Web search",
            5_200,
        ),
        sp(
            1,
            Thinking,
            6_400,
            9_200,
            "Summarised what changed",
            "",
            3_100,
        ),
        failed,
        retry,
        sp(
            2,
            Thinking,
            9_800,
            12_600,
            "Found the drop at the payment step",
            "",
            2_600,
        ),
        sp(
            3,
            Tool,
            12_900,
            15_600,
            "Matched the drop to a deploy",
            "GitHub",
            3_800,
        ),
        sp(
            0,
            Thinking,
            15_800,
            18_400,
            "Combined the findings",
            "",
            4_200,
        ),
        sp(0, Waiting, 18_400, 24_800, "Waiting on the writer", "", 0),
        sp(
            4,
            Thinking,
            18_600,
            24_600,
            "Drafted the incident summary",
            "",
            9_800,
        ),
        sp(
            0,
            Thinking,
            24_800,
            26_400,
            "Reviewed and sent it",
            "",
            1_900,
        ),
    ];
    let handoffs = vec![
        Handoff {
            from: 0,
            to: 1,
            at: 1_900,
            label: "Release notes since Oct 10",
        },
        Handoff {
            from: 0,
            to: 2,
            at: 2_100,
            label: "Conversion by funnel step",
        },
        Handoff {
            from: 1,
            to: 0,
            at: 9_300,
            label: "The payment SDK changed on Oct 14",
        },
        Handoff {
            from: 2,
            to: 3,
            at: 12_700,
            label: "The drop starts at 14:05 UTC",
        },
        Handoff {
            from: 3,
            to: 0,
            at: 15_700,
            label: "Deploy 4f2a1c went out at 14:02",
        },
        Handoff {
            from: 0,
            to: 4,
            at: 18_500,
            label: "Write the incident summary",
        },
        Handoff {
            from: 4,
            to: 0,
            at: 24_700,
            label: "Draft ready for review",
        },
    ];
    (agents, spans, handoffs)
}

/// The live state of an agent timeline.
#[derive(Clone, Debug, PartialEq)]
pub struct Timeline {
    /// Rows, orchestrator first.
    pub agents: Vec<Agent>,
    /// Spans.
    pub spans: Vec<Span>,
    /// Handoffs.
    pub handoffs: Vec<Handoff>,
    /// Run length, ms.
    pub duration: u64,
    /// Playhead when playback last started or paused, ms.
    pub base: u64,
    /// `now` when playback started; `None` while paused.
    pub started: Option<u64>,
    /// Playback speed (1 or 2).
    pub speed: u64,
    /// Pinned span.
    pub pinned: Option<usize>,
    /// Focused span (roving tab stop).
    pub focus: usize,
    /// Follow a run in progress (no playback controls).
    pub live: bool,
}

impl Timeline {
    /// The sample run; `autoplay` starts replaying at `now`, otherwise it renders finished.
    pub fn new(autoplay: bool, live: bool, now: u64) -> Self {
        let (agents, spans, handoffs) = sample();
        let duration = spans.iter().map(|s| s.end).max().unwrap_or(0);
        Self {
            agents,
            spans,
            handoffs,
            duration,
            base: if autoplay || live { 0 } else { duration },
            started: (autoplay || live).then_some(now),
            speed: 1,
            pinned: None,
            focus: 0,
            live,
        }
    }

    /// Playhead, ms.
    pub fn t(&self, now: u64) -> u64 {
        match self.started {
            Some(s) => (self.base + now.saturating_sub(s) * self.speed).min(self.duration),
            None => self.base.min(self.duration),
        }
    }

    /// True while the playhead moves.
    pub fn playing(&self, now: u64) -> bool {
        self.started.is_some() && self.t(now) < self.duration
    }

    /// True once the playhead reached the end.
    pub fn finished(&self, now: u64) -> bool {
        self.t(now) >= self.duration
    }

    /// Where span `i` is at `now`.
    pub fn span_at(&self, i: usize, now: u64) -> SpanAt {
        let s = &self.spans[i];
        let t = self.t(now);
        if t < s.start {
            SpanAt::Future
        } else if t < s.end {
            SpanAt::Active((t - s.start) as f64 / (s.end - s.start).max(1) as f64)
        } else if s.failed {
            SpanAt::Failed
        } else {
            SpanAt::Done
        }
    }

    /// Left edge and drawn width of span `i` as fractions of the axis.
    pub fn geom(&self, i: usize, now: u64) -> (f64, f64) {
        let s = &self.spans[i];
        let d = self.duration.max(1) as f64;
        let t = self.t(now).max(s.start);
        let end = t.min(s.end);
        (s.start as f64 / d, (end - s.start) as f64 / d)
    }

    /// Spans of row `a`, by start.
    pub fn row(&self, a: usize) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.spans.len())
            .filter(|&i| self.spans[i].agent == a)
            .collect();
        v.sort_by_key(|&i| self.spans[i].start);
        v
    }

    /// Every span by start (the waterfall order).
    pub fn waterfall(&self) -> Vec<usize> {
        let mut v: Vec<usize> = (0..self.spans.len()).collect();
        v.sort_by_key(|&i| (self.spans[i].start, self.spans[i].agent));
        v
    }

    /// Tokens used up to the playhead.
    pub fn tokens(&self, now: u64) -> u64 {
        (0..self.spans.len())
            .map(|i| match self.span_at(i, now) {
                SpanAt::Future => 0,
                SpanAt::Active(f) => (self.spans[i].tokens as f64 * f) as u64,
                _ => self.spans[i].tokens,
            })
            .sum()
    }

    /// Cost of span `i` in USD.
    pub fn span_cost(&self, i: usize) -> f64 {
        let s = &self.spans[i];
        s.tokens as f64 * self.agents[s.agent].price / 1e6
    }

    /// Cost up to the playhead, USD.
    pub fn cost(&self, now: u64) -> f64 {
        (0..self.spans.len())
            .map(|i| match self.span_at(i, now) {
                SpanAt::Future => 0.0,
                SpanAt::Active(f) => self.span_cost(i) * f,
                _ => self.span_cost(i),
            })
            .sum()
    }

    /// Agents with a running non-waiting span.
    pub fn working(&self, now: u64) -> usize {
        let mut v: Vec<usize> = (0..self.spans.len())
            .filter(|&i| {
                self.spans[i].kind != SpanKind::Waiting
                    && matches!(self.span_at(i, now), SpanAt::Active(_))
            })
            .map(|i| self.spans[i].agent)
            .collect();
        v.dedup();
        v.len()
    }

    /// Real work over wall time: how much faster than running every span one after another.
    pub fn speedup(&self) -> f64 {
        let work: u64 = self
            .spans
            .iter()
            .filter(|s| s.kind != SpanKind::Waiting)
            .map(|s| s.end - s.start)
            .sum();
        work as f64 / self.duration.max(1) as f64
    }

    /// The status line while running (`12.4s · 21,300 tokens · $0.08 · 2 agents working`).
    pub fn status(&self, now: u64) -> String {
        let w = self.working(now);
        format!(
            "{} · {} tokens · {} · {} agent{} working",
            secs(self.t(now)),
            thousands(self.tokens(now)),
            usd(self.cost(now)),
            w,
            if w == 1 { "" } else { "s" }
        )
    }

    /// The finished summary (`5 agents · 26.4s · 1.5x faster than sequential · $0.16`).
    pub fn summary(&self) -> String {
        format!(
            "{} agents · {} · {:.1}x faster than sequential · {}",
            self.agents.len(),
            secs(self.duration),
            self.speedup(),
            usd(self.cost(u64::MAX / 4))
        )
    }

    /// Handoff `h` has happened at the playhead.
    pub fn handoff_shown(&self, h: usize, now: u64) -> bool {
        self.t(now) >= self.handoffs[h].at
    }

    /// Handoffs on the pinned span's path (they touch its row within its time).
    pub fn on_path(&self, h: usize) -> bool {
        let Some(p) = self.pinned else { return true };
        let s = &self.spans[p];
        let x = &self.handoffs[h];
        (x.from == s.agent || x.to == s.agent) && x.at + 400 >= s.start && x.at <= s.end + 400
    }

    /// Span `i` is on the pinned path (or nothing is pinned).
    pub fn span_on_path(&self, i: usize) -> bool {
        let Some(p) = self.pinned else { return true };
        if p == i {
            return true;
        }
        let s = &self.spans[i];
        (0..self.handoffs.len()).any(|h| {
            self.on_path(h)
                && (self.handoffs[h].from == s.agent || self.handoffs[h].to == s.agent)
                && self.handoffs[h].at + 400 >= s.start
                && self.handoffs[h].at <= s.end + 400
        })
    }

    /// The accessible name of span `i`.
    pub fn span_name(&self, i: usize) -> String {
        let s = &self.spans[i];
        let a = &self.agents[s.agent];
        let tool = if s.tool.is_empty() {
            String::new()
        } else {
            format!(", {}", s.tool)
        };
        let fail = if s.failed { ", failed" } else { "" };
        let retry = if s.retry { ", retry" } else { "" };
        format!(
            "{}, {}{}{}{}: {}, {} to {}, {} tokens",
            a.name,
            s.kind.label(),
            tool,
            fail,
            retry,
            s.label,
            secs(s.start),
            secs(s.end),
            thousands(s.tokens)
        )
    }

    fn rebase(&mut self, now: u64) {
        self.base = self.t(now);
        if self.started.is_some() {
            self.started = Some(now);
        }
    }

    /// Presses: `tl-play` (play / pause / replay), `tl-restart`, `tl-speed`, `tl-span-<i>` (pin
    /// or unpin), `tl-seek-<ms>`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        match id {
            "tl-play" => {
                if self.playing(now) {
                    self.base = self.t(now);
                    self.started = None;
                    Some(format!("Paused at {}", secs(self.base)))
                } else {
                    if self.finished(now) {
                        self.base = 0;
                    } else {
                        self.base = self.t(now);
                    }
                    self.started = Some(now);
                    Some("Replaying the run".into())
                }
            }
            "tl-restart" => {
                self.base = 0;
                self.started = Some(now);
                Some("Replaying from the start".into())
            }
            "tl-speed" => {
                self.rebase(now);
                self.speed = if self.speed == 1 { 2 } else { 1 };
                Some(format!("Speed {}x", self.speed))
            }
            _ => {
                if let Some(i) = super::nth(id, "tl-span-") {
                    if i >= self.spans.len() {
                        return None;
                    }
                    self.focus = i;
                    if self.pinned == Some(i) {
                        self.pinned = None;
                        return Some("Unpinned".into());
                    }
                    self.pinned = Some(i);
                    return Some(format!("Pinned {}", self.span_name(i)));
                }
                if let Some(ms) = id
                    .strip_prefix("tl-seek-")
                    .and_then(|x| x.parse::<u64>().ok())
                {
                    self.base = ms.min(self.duration);
                    if self.started.is_some() {
                        self.started = Some(now);
                    }
                    return Some(format!("{} of {}", secs(self.base), secs(self.duration)));
                }
                None
            }
        }
    }

    /// Keys on the spans: ←/→ previous or next in the row, ↑/↓ the closest span one row up or
    /// down, Home/End first or last in the row, Enter/Space pin, Escape unpin.
    pub fn key_span(&mut self, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        let cur = &self.spans[self.focus.min(self.spans.len() - 1)];
        let row = self.row(cur.agent);
        let pos = row.iter().position(|&i| i == self.focus).unwrap_or(0);
        match key {
            "ArrowLeft" | "ArrowRight" | "Home" | "End" => {
                self.focus = row[super::rove(pos, row.len(), key)];
                None
            }
            "ArrowUp" | "ArrowDown" => {
                let a = cur.agent;
                let to = if key == "ArrowUp" {
                    a.checked_sub(1)
                } else {
                    (a + 1 < self.agents.len()).then_some(a + 1)
                };
                if let Some(to) = to {
                    let mid = (cur.start + cur.end) / 2;
                    if let Some(&j) = self.row(to).iter().min_by_key(|&&j| {
                        let s = &self.spans[j];
                        ((s.start + s.end) / 2).abs_diff(mid)
                    }) {
                        self.focus = j;
                    }
                }
                None
            }
            "Enter" | "Space" => self.press(&format!("tl-span-{}", self.focus), now),
            "Escape" => self.pinned.take().map(|_| "Unpinned".to_string()),
            _ => None,
        }
    }

    /// Keys on the scrubber: ←/→ half a second (Shift: five), PageUp/PageDown five seconds,
    /// Home/End the ends, Space play or pause.
    pub fn key_scrub(&mut self, key: &str, shift: bool, now: u64) -> Option<String> {
        let key = norm_key(key);
        let t = self.t(now) as i64;
        let step = if shift { 5_000 } else { 500 };
        let to = match key {
            "ArrowLeft" | "ArrowDown" => t - step,
            "ArrowRight" | "ArrowUp" => t + step,
            "PageDown" => t - 5_000,
            "PageUp" => t + 5_000,
            "Home" => 0,
            "End" => self.duration as i64,
            "Space" | "Enter" => return self.press("tl-play", now),
            _ => return None,
        };
        let to = to.clamp(0, self.duration as i64) as u64;
        self.press(&format!("tl-seek-{to}"), now)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timeline_plays_pins_and_settles() {
        let mut tl = Timeline::new(true, false, 0);
        assert_eq!(tl.t(1_000), 1_000);
        assert!(tl.playing(1_000));
        assert_eq!(tl.span_at(4, 3_000), SpanAt::Active(0.25));
        assert_eq!(tl.span_at(4, 6_000), SpanAt::Failed);
        assert!(tl.press("tl-play", 2_000).unwrap().starts_with("Paused"));
        assert_eq!(tl.t(9_000), 2_000);
        tl.press("tl-speed", 9_000);
        tl.press("tl-play", 9_000);
        assert_eq!(tl.t(10_000), 4_000);
        assert!(tl.finished(100_000));
        assert!(tl.summary().starts_with("5 agents · 26.4s"));
        assert!(tl.speedup() > 1.0);
        // Pin, path, keys.
        tl.press("tl-span-4", 0);
        assert_eq!(tl.pinned, Some(4));
        assert!(tl.span_on_path(4));
        tl.key_span("ArrowRight", 0);
        assert_eq!(tl.focus, 5);
        tl.key_span("ArrowUp", 0);
        assert_eq!(tl.spans[tl.focus].agent, 1);
        tl.key_span("Escape", 0);
        assert_eq!(tl.pinned, None);
        tl.key_scrub("Home", false, 100_000);
        assert_eq!(tl.t(100_000), 0);
        tl.key_scrub("ArrowRight", true, 100_000);
        assert_eq!(tl.t(100_000), 5_000);
        let finished = Timeline::new(false, false, 0);
        assert!(finished.finished(0));
        assert_eq!(finished.working(0), 0);
    }
}
