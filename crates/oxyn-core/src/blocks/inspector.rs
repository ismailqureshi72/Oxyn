//! Run inspector: one multi-agent run on one screen. The orchestrator and the sub-agents it
//! started form a live tree of cards; each card shows its own calls (model and tool) as small bars
//! on the agent's time axis. Picking an agent fades the others and the numbers on top sum up its
//! part; opening a call shows its arguments, result or error right under its row. The run plays on
//! its own clock derived from `now`; a supervisor watches beside the orchestrator.

use super::{dur, nth, usd};

/// What a span is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// An agent (orchestrator or sub-agent).
    Agent,
    /// A model call.
    Llm,
    /// A tool call.
    Tool,
}

/// One span: parent, kind, name, model brand and label, start and end on the run clock (ms),
/// tokens in and out, cost in tenths of a cent, the argument line, and the result or error.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Span {
    /// Parent span.
    pub parent: Option<usize>,
    /// Kind.
    pub kind: Kind,
    /// Name (agent name, call summary or tool id).
    pub name: &'static str,
    /// Brand id of its model (agents and model calls).
    pub brand: &'static str,
    /// Model label.
    pub model: &'static str,
    /// Start, run ms.
    pub start: u64,
    /// End, run ms.
    pub end: u64,
    /// Tokens in.
    pub tin: u64,
    /// Tokens out.
    pub tout: u64,
    /// Cost in tenths of a cent.
    pub cost: u64,
    /// Arguments or the agent's task.
    pub args: &'static str,
    /// Result.
    pub result: &'static str,
    /// Error (a failed call).
    pub error: &'static str,
}

#[allow(clippy::too_many_arguments)]
const fn s(
    parent: Option<usize>,
    kind: Kind,
    name: &'static str,
    brand: &'static str,
    model: &'static str,
    start: u64,
    end: u64,
    tin: u64,
    tout: u64,
    cost: u64,
    args: &'static str,
    result: &'static str,
    error: &'static str,
) -> Span {
    Span {
        parent,
        kind,
        name,
        brand,
        model,
        start,
        end,
        tin,
        tout,
        cost,
        args,
        result,
        error,
    }
}

use Kind::{Agent as A, Llm as L, Tool as T};

/// The sample run (fictional product and teams).
pub const SPANS: [Span; 13] = [
    s(
        None,
        A,
        "Orchestrator",
        "claude",
        "Claude Opus 4.1",
        0,
        24_000,
        0,
        0,
        0,
        "Launch the new pricing page by Friday",
        "Page shipped behind a flag",
        "",
    ),
    s(
        Some(0),
        L,
        "Plan the launch",
        "claude",
        "Claude Opus 4.1",
        200,
        3_100,
        6_200,
        900,
        300,
        "3 sub-tasks, 2 agents",
        "Research prices, build the page, write the summary",
        "",
    ),
    s(
        Some(0),
        A,
        "Researcher",
        "gemini",
        "Gemini 2.5 Pro",
        3_300,
        14_000,
        0,
        0,
        0,
        "Collect per-seat prices from five tools",
        "Median $14 per seat",
        "",
    ),
    s(
        Some(2),
        T,
        "web_search",
        "",
        "",
        3_600,
        6_200,
        0,
        0,
        10,
        "query: \"per-seat pricing project tools\"",
        "12 results",
        "",
    ),
    s(
        Some(2),
        T,
        "fetch_page",
        "",
        "",
        6_400,
        9_800,
        0,
        0,
        10,
        "url: five pricing pages",
        "5 pages, 38 KB of text",
        "",
    ),
    s(
        Some(2),
        L,
        "Summarise the prices",
        "gemini",
        "Gemini 2.5 Pro",
        10_000,
        13_800,
        14_800,
        1_100,
        210,
        "38 KB of pricing text",
        "Table of 5 tools, median $14",
        "",
    ),
    s(
        Some(0),
        A,
        "Coder",
        "openai",
        "GPT-5",
        3_400,
        22_000,
        0,
        0,
        0,
        "Build the page",
        "Page and checkout tests green",
        "",
    ),
    s(
        Some(6),
        T,
        "read_file",
        "",
        "",
        3_800,
        5_000,
        0,
        0,
        0,
        "path: app/pricing/page.tsx",
        "212 lines",
        "",
    ),
    s(
        Some(6),
        T,
        "edit_file",
        "",
        "",
        5_200,
        11_800,
        0,
        0,
        0,
        "path: app/pricing/page.tsx",
        "+148 −36",
        "",
    ),
    s(
        Some(6),
        T,
        "run_tests",
        "",
        "",
        12_000,
        17_400,
        0,
        0,
        0,
        "command: test checkout",
        "",
        "checkout suite: 2 failures, yearly total rounded down",
    ),
    s(
        Some(6),
        T,
        "edit_file",
        "",
        "",
        17_600,
        20_500,
        0,
        0,
        0,
        "path: lib/price.ts",
        "+4 −2: round yearly totals up",
        "",
    ),
    s(
        Some(6),
        T,
        "run_tests",
        "",
        "",
        20_700,
        21_900,
        0,
        0,
        0,
        "command: test checkout",
        "14 of 14 pass",
        "",
    ),
    s(
        Some(0),
        L,
        "Write the summary",
        "claude",
        "Claude Opus 4.1",
        22_100,
        23_800,
        9_400,
        700,
        190,
        "Results of both agents",
        "Summary for the team",
        "",
    ),
];
/// The supervisor beside the orchestrator: name, brand, model, what it watches.
pub const SUPERVISOR: (&str, &str, &str, &str) = (
    "Reviewer",
    "claude",
    "Claude Haiku 4.5",
    "Watches every edit and test run",
);
/// The whole run, ms.
pub const TOTAL: u64 = 24_000;
/// Model tokens are billed in these on agents that make calls (tenths of a cent per call of code).
const TOOL_TOKENS: u64 = 400;

/// A span's state on the run clock.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpanSt {
    /// Not started yet (hidden).
    Waiting,
    /// Running.
    Running,
    /// Finished.
    Done,
    /// Failed.
    Failed,
}

/// The numbers on top.
#[derive(Clone, Debug, PartialEq)]
pub struct Numbers {
    /// What they sum up (`Whole run` or the agent's name).
    pub of: String,
    /// Time.
    pub time: String,
    /// Agents.
    pub agents: usize,
    /// Calls.
    pub calls: usize,
    /// Tokens in and out.
    pub tokens: (u64, u64),
    /// Cost.
    pub cost: String,
    /// Failed calls.
    pub errors: usize,
}

/// The live state of the inspector.
#[derive(Clone, Debug, PartialEq)]
pub struct Inspector {
    /// Clock start, ms.
    pub t0: u64,
    /// Run time to start from (open a finished run with `TOTAL`).
    pub skip: u64,
    /// Speed multiplier ×100.
    pub speed: u64,
    /// The picked agent.
    pub selected: Option<usize>,
    /// The open call.
    pub call: Option<usize>,
    /// Show the supervisor.
    pub supervisor: bool,
}

impl Inspector {
    /// An inspector at `now`, `skip` ms into the run, at `speed` (100 = real time).
    pub fn new(skip: u64, speed: u64, supervisor: bool, now: u64) -> Self {
        Self {
            t0: now,
            skip,
            speed: speed.max(10),
            selected: None,
            call: None,
            supervisor,
        }
    }

    /// Run clock at `now`, ms.
    pub fn t(&self, now: u64) -> u64 {
        (self.skip + now.saturating_sub(self.t0) * self.speed / 100).min(TOTAL)
    }

    /// State of span `i` at `now`.
    pub fn st(&self, i: usize, now: u64) -> SpanSt {
        let t = self.t(now);
        let sp = &SPANS[i];
        if t < sp.start {
            SpanSt::Waiting
        } else if t < sp.end {
            SpanSt::Running
        } else if sp.error.is_empty() {
            SpanSt::Done
        } else {
            SpanSt::Failed
        }
    }

    /// Agents (orchestrator first) that have started at `now`.
    pub fn agents(&self, now: u64) -> Vec<usize> {
        (0..SPANS.len())
            .filter(|&i| SPANS[i].kind == Kind::Agent && self.st(i, now) != SpanSt::Waiting)
            .collect()
    }

    /// Calls (model and tool) agent `a` made that have started at `now`.
    pub fn calls(&self, a: usize, now: u64) -> Vec<usize> {
        (0..SPANS.len())
            .filter(|&i| {
                SPANS[i].parent == Some(a)
                    && SPANS[i].kind != Kind::Agent
                    && self.st(i, now) != SpanSt::Waiting
            })
            .collect()
    }

    /// Depth of span `i` in the tree.
    pub fn depth(i: usize) -> usize {
        let mut d = 0;
        let mut p = SPANS[i].parent;
        while let Some(x) = p {
            d += 1;
            p = SPANS[x].parent;
        }
        d
    }

    /// Is `i` inside the subtree of `a`?
    fn under(i: usize, a: usize) -> bool {
        let mut p = Some(i);
        while let Some(x) = p {
            if x == a {
                return true;
            }
            p = SPANS[x].parent;
        }
        false
    }

    /// A call bar on its agent's card: left and width as fractions of the agent's span.
    pub fn bar(&self, i: usize, now: u64) -> (f64, f64) {
        let sp = &SPANS[i];
        let ag = &SPANS[sp.parent.unwrap_or(0)];
        let len = (ag.end - ag.start).max(1) as f64;
        let end = sp.end.min(self.t(now).max(sp.start));
        (
            (sp.start - ag.start) as f64 / len,
            (end - sp.start) as f64 / len,
        )
    }

    /// Duration of span `i` so far.
    pub fn time(&self, i: usize, now: u64) -> String {
        let sp = &SPANS[i];
        dur(self.t(now).min(sp.end).saturating_sub(sp.start))
    }

    /// The agent's status as a word with its hue, failed calls counted in.
    pub fn agent_st(&self, a: usize, now: u64) -> (&'static str, &'static str) {
        let failed = self
            .calls(a, now)
            .iter()
            .any(|&c| self.st(c, now) == SpanSt::Failed);
        match self.st(a, now) {
            SpanSt::Running if failed => ("Running · 1 failed call", "amber"),
            SpanSt::Running => ("Running", "blue"),
            SpanSt::Done => ("Done", "green"),
            SpanSt::Failed => ("Failed", "red"),
            SpanSt::Waiting => ("Waiting", "neutral"),
        }
    }

    /// Tokens a call billed (tool calls count a small fixed overhead).
    fn tokens(i: usize) -> (u64, u64) {
        let sp = &SPANS[i];
        if sp.kind == Kind::Tool {
            (TOOL_TOKENS, 0)
        } else {
            (sp.tin, sp.tout)
        }
    }

    /// Totals at `now` for the run, or for the selected agent's subtree.
    pub fn numbers(&self, now: u64) -> Numbers {
        let root = self.selected.unwrap_or(0);
        let done: Vec<usize> = (0..SPANS.len())
            .filter(|&i| Self::under(i, root) && self.st(i, now) != SpanSt::Waiting)
            .collect();
        let calls: Vec<usize> = done
            .iter()
            .copied()
            .filter(|&i| SPANS[i].kind != Kind::Agent)
            .collect();
        let fin = |i: &usize| matches!(self.st(*i, now), SpanSt::Done | SpanSt::Failed);
        let tokens = calls.iter().filter(|i| fin(i)).fold((0, 0), |a, &i| {
            let t = Self::tokens(i);
            (a.0 + t.0, a.1 + t.1)
        });
        let cost: u64 = calls
            .iter()
            .filter(|i| fin(i))
            .map(|&i| SPANS[i].cost)
            .sum();
        Numbers {
            of: if root == 0 {
                "Whole run".into()
            } else {
                SPANS[root].name.into()
            },
            time: self.time(root, now),
            agents: done
                .iter()
                .filter(|&&i| SPANS[i].kind == Kind::Agent)
                .count(),
            calls: calls.len(),
            tokens,
            cost: usd(cost as f64 / 1000.0),
            errors: calls
                .iter()
                .filter(|&&i| self.st(i, now) == SpanSt::Failed)
                .count(),
        }
    }

    /// The title line (`3 agents · running for 12s`).
    pub fn title(&self, now: u64) -> String {
        let n = self.agents(now).len();
        let t = self.t(now);
        if t >= TOTAL {
            format!("{n} agents · finished in {}", dur(TOTAL))
        } else {
            format!("{n} agents · running for {}", dur(t))
        }
    }

    /// Accessible label of call row `i`.
    pub fn call_label(&self, i: usize, now: u64) -> String {
        let sp = &SPANS[i];
        let f = if self.st(i, now) == SpanSt::Failed {
            ", failed"
        } else {
            ""
        };
        format!("{}, {}{}", sp.name, self.time(i, now), f)
    }

    /// Presses: `ri-agent-<i>`, `ri-call-<i>`, `ri-replay`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "ri-agent-") {
            if i >= SPANS.len() || SPANS[i].kind != Kind::Agent {
                return None;
            }
            self.selected = if self.selected == Some(i) || i == 0 {
                None
            } else {
                Some(i)
            };
            let n = self.numbers(now);
            return Some(format!("{}: {} calls, {}", n.of, n.calls, n.cost));
        }
        if let Some(i) = nth(id, "ri-call-") {
            if i >= SPANS.len() || SPANS[i].kind == Kind::Agent {
                return None;
            }
            self.call = if self.call == Some(i) { None } else { Some(i) };
            return self.call.map(|i| {
                let sp = &SPANS[i];
                if self.st(i, now) == SpanSt::Failed {
                    format!("{} failed: {}", sp.name, sp.error)
                } else {
                    format!("{} opened", sp.name)
                }
            });
        }
        if id == "ri-replay" {
            self.t0 = now;
            self.skip = 0;
            self.call = None;
            return Some("Replaying the run".into());
        }
        None
    }

    /// Escape closes the open call.
    pub fn key(&mut self, key: &str, _now: u64) -> Option<String> {
        (super::norm_key(key) == "Escape")
            .then(|| self.call.take().map(|_| "Call closed".to_string()))
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plays_and_sums() {
        let mut r = Inspector::new(0, 100, true, 0);
        assert_eq!(r.agents(0), vec![0]);
        assert_eq!(r.agents(5_000).len(), 3);
        assert_eq!(r.st(9, 13_000), SpanSt::Running);
        assert_eq!(r.st(9, 18_000), SpanSt::Failed);
        assert_eq!(r.agent_st(6, 18_000).1, "amber");
        let all = r.numbers(30_000);
        assert_eq!(all.calls, 10);
        assert_eq!(all.errors, 1);
        r.press("ri-agent-2", 30_000);
        let n = r.numbers(30_000);
        assert_eq!((n.of.as_str(), n.calls), ("Researcher", 3));
        assert!(r.title(30_000).contains("finished"));
        r.press("ri-call-9", 30_000);
        assert_eq!(r.call, Some(9));
        r.key("Escape", 30_000);
        assert_eq!(r.call, None);
        assert!(r.press("ri-call-2", 0).is_none());
        let (l, w) = r.bar(9, 30_000);
        assert!(l > 0.0 && w > 0.0 && l + w <= 1.0);
        r.press("ri-replay", 40_000);
        assert_eq!(r.t(40_000), 0);
    }
}
