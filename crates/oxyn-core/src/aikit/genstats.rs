//! Generation stats: a live strip for a streaming reply. Time to first token, speed with a
//! sparkline, output tokens and cost count up beside the model's mark, then settle and open into
//! a latency breakdown. Stop freezes the counts; an error keeps what arrived.

use super::{nth, thousands, usd};

/// Time to first token in the sample, ms.
pub const TTFT_MS: u64 = 420;
/// Queue time before the request starts, ms.
pub const QUEUE_MS: u64 = 60;
/// Pause between loops, ms.
pub const LOOP_GAP_MS: u64 = 2_500;

/// Models: name, mark, USD per 1M input, per 1M output, input tokens of the request.
pub const MODELS: [(&str, &str, f64, f64); 3] = [
    ("Claude Sonnet 4.5", "claude", 3.0, 15.0),
    ("GPT-5", "openai", 1.25, 10.0),
    ("Gemini 2.5 Flash", "gemini", 0.3, 2.5),
];

/// Input tokens of the sample request.
pub const INPUT_TOKENS: u64 = 1_840;

/// Status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Not started.
    Idle,
    /// Waiting for the first token.
    Waiting,
    /// Tokens arriving.
    Streaming,
    /// Finished.
    Done,
    /// Stopped by the reader.
    Stopped,
    /// Failed part way.
    Error,
}

impl Status {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Ready",
            Self::Waiting => "Waiting for first token",
            Self::Streaming => "Streaming",
            Self::Done => "Done",
            Self::Stopped => "Stopped",
            Self::Error => "Failed",
        }
    }
}

/// The live state of the stats strip.
#[derive(Clone, Debug, PartialEq)]
pub struct GenStats {
    /// Model.
    pub model: usize,
    /// Run start.
    pub start: Option<u64>,
    /// Frozen at (Stop).
    pub stopped: Option<u64>,
    /// Chunk schedule: ms after start, tokens.
    pub chunks: Vec<(u64, u64)>,
    /// Fail after this many chunks (the error scenario).
    pub fail_after: Option<usize>,
    /// Breakdown open.
    pub expanded: bool,
    /// Replay after finishing.
    pub looping: bool,
    /// Finished runs this session: tokens, cost (Oxyn extra).
    pub session: Vec<(u64, f64)>,
}

impl GenStats {
    /// `scenario`: `stream` (starts at `now`), `done`, `idle`, `error`.
    pub fn new(scenario: &str, model: usize, now: u64) -> Self {
        let speed = [86.0, 112.0, 190.0][model.min(2)];
        let mut chunks = Vec::new();
        let mut t = QUEUE_MS + TTFT_MS;
        let mut k = 0u64;
        let mut total = 0;
        while total < 640 {
            let n = 6 + (k * 7 + model as u64 * 3) % 9;
            chunks.push((t, n));
            total += n;
            t += (n as f64 / speed * 1000.0) as u64 + (k % 3) * 6;
            k += 1;
        }
        let mut g = Self {
            model: model.min(2),
            start: None,
            stopped: None,
            chunks,
            fail_after: (scenario == "error").then_some(30),
            expanded: false,
            looping: false,
            session: vec![(512, 0.0131), (1_204, 0.0236)],
        };
        match scenario {
            "idle" => {}
            "done" => {
                g.start = Some(0);
                g.expanded = true;
            }
            _ => g.start = Some(now),
        }
        g
    }

    fn total_ms(&self) -> u64 {
        self.chunks.last().map(|c| c.0 + 40).unwrap_or(0)
    }

    /// Ms since start at `now` (frozen when stopped, wrapped when looping).
    pub fn el(&self, now: u64) -> Option<u64> {
        let s = self.start?;
        let t = self.stopped.unwrap_or(now).saturating_sub(s);
        if self.looping && self.stopped.is_none() {
            Some(t % (self.total_ms() + LOOP_GAP_MS))
        } else {
            Some(t)
        }
    }

    /// Chunks landed.
    pub fn landed(&self, now: u64) -> usize {
        let Some(t) = self.el(now) else { return 0 };
        let n = self.chunks.iter().take_while(|c| c.0 <= t).count();
        match self.fail_after {
            Some(f) => n.min(f),
            None => n,
        }
    }

    /// Status at `now`.
    pub fn status(&self, now: u64) -> Status {
        let Some(t) = self.el(now) else {
            return Status::Idle;
        };
        let n = self.landed(now);
        if self.fail_after.is_some_and(|f| n >= f) {
            return Status::Error;
        }
        if self.stopped.is_some() && n < self.chunks.len() {
            return Status::Stopped;
        }
        if t < QUEUE_MS + TTFT_MS {
            Status::Waiting
        } else if n < self.chunks.len() {
            Status::Streaming
        } else {
            Status::Done
        }
    }

    /// Output tokens so far.
    pub fn tokens(&self, now: u64) -> u64 {
        self.chunks.iter().take(self.landed(now)).map(|c| c.1).sum()
    }

    /// Speed over the last five chunks, tokens a second (held between chunks).
    pub fn speed(&self, now: u64) -> f64 {
        let n = self.landed(now);
        if n < 2 {
            return 0.0;
        }
        let a = n.saturating_sub(6);
        let toks: u64 = self.chunks[a + 1..n].iter().map(|c| c.1).sum();
        let dt = self.chunks[n - 1].0 - self.chunks[a].0;
        toks as f64 / (dt.max(1) as f64 / 1000.0)
    }

    /// Sparkline samples (speed after each chunk, at most 24).
    pub fn spark(&self, now: u64) -> Vec<f64> {
        let n = self.landed(now);
        let mut v = Vec::new();
        for k in 2..=n {
            let a = k.saturating_sub(6);
            let toks: u64 = self.chunks[a + 1..k].iter().map(|c| c.1).sum();
            let dt = self.chunks[k - 1].0 - self.chunks[a].0;
            v.push(toks as f64 / (dt.max(1) as f64 / 1000.0));
        }
        let skip = v.len().saturating_sub(24);
        v.split_off(skip)
    }

    /// Cost so far (input billed once the request starts), USD.
    pub fn cost(&self, now: u64) -> f64 {
        if self.start.is_none() {
            return 0.0;
        }
        let (_, _, i, o) = MODELS[self.model];
        (INPUT_TOKENS as f64 * i + self.tokens(now) as f64 * o) / 1e6
    }

    /// TTFT text (`420 ms`, or `—` while waiting).
    pub fn ttft(&self, now: u64) -> String {
        if self.landed(now) == 0 {
            "—".into()
        } else {
            format!("{TTFT_MS} ms")
        }
    }

    /// Latency breakdown rows: label, ms, share of total.
    pub fn breakdown(&self, now: u64) -> Vec<(&'static str, u64, f64)> {
        let n = self.landed(now);
        let end = if n == 0 { 0 } else { self.chunks[n - 1].0 };
        let gn = end.saturating_sub(QUEUE_MS + TTFT_MS);
        let total = (QUEUE_MS + TTFT_MS + gn).max(1) as f64;
        vec![
            ("Queue", QUEUE_MS, QUEUE_MS as f64 / total),
            ("Time to first token", TTFT_MS, TTFT_MS as f64 / total),
            ("Generation", gn, gn as f64 / total),
        ]
    }

    /// The settled summary (`640 tokens in 6.4s at 98 tok/s · $0.0151`).
    pub fn summary(&self, now: u64) -> String {
        let n = self.landed(now);
        let end = if n == 0 { 0 } else { self.chunks[n - 1].0 };
        format!(
            "{} tokens in {} · {}",
            thousands(self.tokens(now)),
            super::secs(end),
            usd(self.cost(now))
        )
    }

    /// Presses: `gs-run`, `gs-stop`, `gs-retry`, `gs-expand`, `gs-model-<i>`, `gs-loop`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "gs-model-") {
            let exp = self.expanded;
            *self = GenStats::new("stream", i, now);
            self.expanded = exp;
            return Some(format!("Streaming with {}", MODELS[self.model].0));
        }
        let st = self.status(now);
        match id {
            "gs-run" | "gs-retry" => {
                if matches!(st, Status::Done | Status::Stopped | Status::Error) {
                    self.session.push((self.tokens(now), self.cost(now)));
                }
                self.start = Some(now);
                self.stopped = None;
                self.fail_after = None;
                Some(format!("Streaming with {}", MODELS[self.model].0))
            }
            "gs-stop" if matches!(st, Status::Waiting | Status::Streaming) => {
                self.stopped = Some(now);
                Some(format!("Stopped at {} tokens", self.tokens(now)))
            }
            "gs-expand" => {
                self.expanded = !self.expanded;
                None
            }
            "gs-loop" => {
                self.looping = !self.looping;
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn genstats_counts_stops_and_fails() {
        let mut g = GenStats::new("stream", 0, 0);
        assert_eq!(g.status(100), Status::Waiting);
        assert_eq!(g.ttft(100), "—");
        assert_eq!(g.status(1_000), Status::Streaming);
        assert!(g.tokens(1_000) > 0 && g.speed(2_000) > 40.0);
        assert!(!g.spark(3_000).is_empty());
        assert_eq!(g.status(60_000), Status::Done);
        assert!(g.tokens(60_000) >= 640);
        g.press("gs-run", 60_000);
        assert_eq!(g.session.len(), 3);
        g.press("gs-stop", 61_500);
        assert_eq!(g.status(70_000), Status::Stopped);
        assert_eq!(g.tokens(70_000), g.tokens(61_500));
        let e = GenStats::new("error", 1, 0);
        assert_eq!(e.status(60_000), Status::Error);
        assert_eq!(e.breakdown(60_000).len(), 3);
        let mut l = GenStats::new("stream", 2, 0);
        l.press("gs-loop", 0);
        let t = l.total_ms() + LOOP_GAP_MS + 10;
        assert_eq!(l.status(t), Status::Waiting);
        assert!(
            GenStats::new("done", 0, 0)
                .summary(99_000)
                .contains("tokens in")
        );
    }
}
