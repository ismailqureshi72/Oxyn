//! affectUI fidelity pass (plan #32, overnight batch, family B): the models behind the exact
//! **Findings feed** (a swarm of checking agents, their progress and a feed of findings) and
//! **Task inbox** (background tasks on a board or a list, with live clocks and Approve, Reject,
//! Retry). Behaviour only; time is passed in as elapsed milliseconds.

// ─────────────────────────────────────────────────────────────── Findings feed ──

/// How bad a finding is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindingSeverity {
    /// Must fix (red).
    Critical,
    /// Should look at (amber).
    Warning,
    /// Worth knowing (blue-grey).
    Note,
}

impl FindingSeverity {
    /// "Critical" · "Warning" · "Note".
    pub fn label(self) -> &'static str {
        match self {
            Self::Critical => "Critical",
            Self::Warning => "Warning",
            Self::Note => "Note",
        }
    }
    /// `critical` · `warning` · `note`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Critical => "critical",
            Self::Warning => "warning",
            Self::Note => "note",
        }
    }
}

/// One checking agent: a row with its progress.
#[derive(Clone, Debug, PartialEq)]
pub struct FindingAgent {
    /// Name ("Auditor").
    pub name: String,
    /// Model, shown in the row's title ("Opus 5.5").
    pub model: String,
    /// Brand for the coloured mark.
    pub brand: String,
    /// Steps it walks through, in turn.
    pub steps: Vec<String>,
    /// Checks in all.
    pub total: u32,
    /// Milliseconds per check.
    pub ms_per_check: u64,
}

impl FindingAgent {
    /// Checks done `elapsed` ms in.
    pub fn done_at(&self, elapsed: u64) -> u32 {
        ((elapsed / self.ms_per_check.max(1)) as u32).min(self.total)
    }
    /// The step shown at `done` checks ("Done" at the end).
    pub fn step_at(&self, done: u32) -> String {
        if done >= self.total {
            return "Done".into();
        }
        let n = self.steps.len().max(1);
        let i = (done as usize * n) / self.total.max(1) as usize;
        self.steps.get(i.min(n - 1)).cloned().unwrap_or_default()
    }
}

/// One finding.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentFinding {
    /// Id.
    pub id: String,
    /// Severity.
    pub severity: FindingSeverity,
    /// What was found.
    pub title: String,
    /// The agent (index into the agents).
    pub agent: usize,
    /// Where ("Vault.sol:142").
    pub location: String,
    /// When it was found, ms after the start.
    pub at_ms: u64,
}

/// Findings found by `elapsed`, newest first.
pub fn findings_at(all: &[AgentFinding], elapsed: u64) -> Vec<AgentFinding> {
    let mut v: Vec<AgentFinding> = all.iter().filter(|f| f.at_ms <= elapsed).cloned().collect();
    v.sort_by_key(|f| std::cmp::Reverse(f.at_ms));
    v
}

/// (critical, warning) counts among `found`, for `agent` or all agents.
pub fn severity_counts(found: &[AgentFinding], agent: Option<usize>) -> (usize, usize) {
    let mine = |f: &&AgentFinding| agent.is_none_or(|a| f.agent == a);
    (
        found
            .iter()
            .filter(mine)
            .filter(|f| f.severity == FindingSeverity::Critical)
            .count(),
        found
            .iter()
            .filter(mine)
            .filter(|f| f.severity == FindingSeverity::Warning)
            .count(),
    )
}

/// "now", "5s", "43s", "2m", "1h" for an age in ms.
pub fn age(ms: u64) -> String {
    let s = ms / 1000;
    if s < 3 {
        "now".into()
    } else if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m", s / 60)
    } else {
        format!("{}h", s / 3600)
    }
}

/// The demo: a code review swarm (agents, findings, title).
pub fn demo_findings() -> (Vec<FindingAgent>, Vec<AgentFinding>) {
    let a =
        |name: &str, model: &str, brand: &str, steps: &[&str], total: u32, ms: u64| FindingAgent {
            name: name.into(),
            model: model.into(),
            brand: brand.into(),
            steps: steps.iter().map(|s| s.to_string()).collect(),
            total,
            ms_per_check: ms,
        };
    let f = |id: &str, sev: FindingSeverity, title: &str, agent: usize, loc: &str, at: u64| {
        AgentFinding {
            id: id.into(),
            severity: sev,
            title: title.into(),
            agent,
            location: loc.into(),
            at_ms: at,
        }
    };
    use FindingSeverity::*;
    (
        vec![
            a(
                "Auditor",
                "Opus 5.5",
                "claude",
                &[
                    "Tracing calls in Ledger.settle",
                    "Checking roles on refund",
                    "Reading migration paths",
                ],
                24,
                900,
            ),
            a(
                "Fuzzer",
                "GPT-6",
                "openai",
                &[
                    "Fuzz-testing the payout path, 12k cases",
                    "Replaying failed sequences",
                    "Shrinking a failing case",
                ],
                18,
                1300,
            ),
            a(
                "Prover",
                "Gemini 3.8 Pro",
                "gemini",
                &[
                    "Proving the balance invariant",
                    "Checking that refunds conserve money",
                ],
                12,
                2100,
            ),
            a(
                "Perf review",
                "Sonnet 5.5",
                "claude",
                &[
                    "Profiling the hot queries",
                    "Packing cache keys",
                    "Checking loop costs",
                ],
                10,
                2600,
            ),
        ],
        vec![
            f(
                "f1",
                Warning,
                "Refund can run twice on a retry",
                0,
                "ledger.rs:142",
                2500,
            ),
            f(
                "f2",
                Note,
                "Unused index on payments.created_at",
                3,
                "0042_payments.sql:9",
                5200,
            ),
            f(
                "f3",
                Warning,
                "Return value of charge() not checked",
                1,
                "billing.rs:88",
                7800,
            ),
            f(
                "f4",
                Critical,
                "Settle can double-spend under a race",
                0,
                "ledger.rs:201",
                10400,
            ),
            f(
                "f5",
                Warning,
                "Exchange rate used without a staleness check",
                1,
                "fx.rs:57",
                13600,
            ),
            f(
                "f6",
                Critical,
                "Total can differ from the sum of entries",
                2,
                "ledger.rs:318",
                17200,
            ),
        ],
    )
}

// ────────────────────────────────────────────────────────────────── Task inbox ──

/// Where a background task is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskState {
    /// Waits for a person (Needs you).
    Waiting,
    /// Working.
    Running,
    /// Finished.
    Done,
    /// Failed.
    Failed,
}

impl TaskState {
    /// `waiting` · `running` · `done` · `failed`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Running => "running",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
    /// The board column: 0 Needs you (waiting and failed), 1 Running, 2 Done.
    pub fn column(self) -> usize {
        match self {
            Self::Waiting | Self::Failed => 0,
            Self::Running => 1,
            Self::Done => 2,
        }
    }
}

/// The three board columns.
pub const INBOX_COLUMNS: [&str; 3] = ["Needs you", "Running", "Done"];

/// A background task.
#[derive(Clone, Debug, PartialEq)]
pub struct InboxTask {
    /// Id.
    pub id: String,
    /// Title.
    pub title: String,
    /// What it does.
    pub description: String,
    /// The model running it.
    pub agent: String,
    /// State at the start.
    pub state: TaskState,
    /// Seconds already on its clock at the start.
    pub elapsed_s: u64,
    /// Running: seconds until it finishes by itself (`None` keeps running).
    pub finishes_after_s: Option<u64>,
    /// Tools used.
    pub tools: u32,
    /// Sources read.
    pub sources: u32,
    /// Last steps, oldest first: (label, when).
    pub steps: Vec<(String, String)>,
}

/// "6m 14s", "48s", "1h 02m".
pub fn clock(s: u64) -> String {
    if s < 60 {
        format!("{s}s")
    } else if s < 3600 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{}h {:02}m", s / 3600, (s % 3600) / 60)
    }
}

/// The live sentence under the board ("2 need you, 2 running").
pub fn inbox_summary(states: &[TaskState]) -> String {
    let need = states.iter().filter(|s| s.column() == 0).count();
    let run = states.iter().filter(|s| **s == TaskState::Running).count();
    let mut parts = Vec::new();
    if need > 0 {
        parts.push(format!(
            "{need} need{} you",
            if need == 1 { "s" } else { "" }
        ));
    }
    if run > 0 {
        parts.push(format!("{run} running"));
    }
    if parts.is_empty() {
        "All done".into()
    } else {
        parts.join(", ")
    }
}

/// The demo tasks.
pub fn demo_inbox() -> Vec<InboxTask> {
    let st = |v: &[(&str, &str)]| -> Vec<(String, String)> {
        v.iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect()
    };
    vec![
        InboxTask {
            id: "t1".into(),
            title: "Migrate the orders table".into(),
            description: "Adds a region column and backfills 48k rows. Ready to run on production once you approve.".into(),
            agent: "Opus 5.5".into(),
            state: TaskState::Waiting,
            elapsed_s: 374,
            finishes_after_s: None,
            tools: 6,
            sources: 2,
            steps: st(&[
                ("Read the schema and 3 migrations", "6m ago"),
                ("Wrote the migration and a rollback", "4m ago"),
                ("Dry run on staging: 48,112 rows in 41s", "2m ago"),
                ("Waiting for your approval to run on production", "now"),
            ]),
        },
        InboxTask {
            id: "t2".into(),
            title: "Dedupe the customer list".into(),
            description: "Merges contacts that share an email or phone. Stopped at a rate limit from the CRM.".into(),
            agent: "Haiku 5.5".into(),
            state: TaskState::Failed,
            elapsed_s: 131,
            finishes_after_s: None,
            tools: 3,
            sources: 1,
            steps: st(&[
                ("Exported 9,840 contacts", "3m ago"),
                ("Found 1,204 likely duplicates", "2m ago"),
                ("The CRM answered 429 – too many requests", "1m ago"),
            ]),
        },
        InboxTask {
            id: "t3".into(),
            title: "Write the release notes".into(),
            description: "Groups 37 merged pull requests into features, fixes and upgrade notes for 3.1.".into(),
            agent: "Sonnet 5.5".into(),
            state: TaskState::Running,
            elapsed_s: 228,
            finishes_after_s: Some(9),
            tools: 4,
            sources: 37,
            steps: st(&[
                ("Read 37 merged pull requests", "3m ago"),
                ("Grouped them into 3 sections", "1m ago"),
                ("Drafting the upgrade notes", "now"),
            ]),
        },
        InboxTask {
            id: "t4".into(),
            title: "Summarise churn interviews".into(),
            description: "Reads 12 call transcripts and pulls out the reasons people left, with quotes.".into(),
            agent: "Opus 5.5".into(),
            state: TaskState::Running,
            elapsed_s: 95,
            finishes_after_s: None,
            tools: 2,
            sources: 12,
            steps: st(&[
                ("Transcribed 12 calls", "1m ago"),
                ("Tagging reasons in call 8 of 12", "now"),
            ]),
        },
        InboxTask {
            id: "t5".into(),
            title: "Check the pricing page links".into(),
            description: "Crawled every link on the pricing and plans pages; all 64 answer.".into(),
            agent: "Haiku 5.5".into(),
            state: TaskState::Done,
            elapsed_s: 252,
            finishes_after_s: None,
            tools: 1,
            sources: 64,
            steps: st(&[
                ("Crawled 64 links", "5m ago"),
                ("All links answer with 200", "4m ago"),
            ]),
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn findings_progress_and_feed() {
        let (agents, all) = demo_findings();
        assert_eq!(agents[0].done_at(0), 0);
        assert_eq!(agents[0].done_at(1_000_000), 24);
        assert_eq!(agents[0].step_at(24), "Done");
        assert_eq!(agents[0].step_at(0), "Tracing calls in Ledger.settle");
        let f = findings_at(&all, 11_000);
        assert_eq!(f.len(), 4);
        assert_eq!(f[0].id, "f4");
        assert_eq!(severity_counts(&f, None), (1, 2));
        assert_eq!(severity_counts(&f, Some(1)), (0, 1));
        assert_eq!(age(1000), "now");
        assert_eq!(age(43_000), "43s");
        assert_eq!(age(130_000), "2m");
    }

    #[test]
    fn inbox_clock_and_summary() {
        assert_eq!(clock(374), "6m 14s");
        assert_eq!(clock(48), "48s");
        let s: Vec<TaskState> = demo_inbox().iter().map(|t| t.state).collect();
        assert_eq!(inbox_summary(&s), "2 need you, 2 running");
        assert_eq!(TaskState::Failed.column(), 0);
    }
}
