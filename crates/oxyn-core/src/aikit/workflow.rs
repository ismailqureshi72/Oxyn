//! AI workflow: a trigger, an AI step, a branch and actions on one hairline rail. A test run
//! glides a dot down each connector, runs each step and opens its output inline; the untaken
//! path is skipped, a failing step offers Retry (which resumes from it), steps are added from a
//! command menu with app marks and removed with a small button.

use super::{norm_key, nth, words_at};

/// Travel time of the run dot down one connector, ms.
pub const TRAVEL_MS: u64 = 240;

/// What a step is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepKind {
    /// Starts the flow.
    Trigger,
    /// A model step.
    Ai,
    /// Picks a path.
    Branch,
    /// Does something in an app.
    Action,
}

impl StepKind {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Trigger => "Trigger",
            Self::Ai => "AI step",
            Self::Branch => "Branch",
            Self::Action => "Action",
        }
    }
}

/// A step's output preview.
#[derive(Clone, Debug, PartialEq)]
pub enum Output {
    /// An email: sender, subject, preview.
    Email(&'static str, &'static str, &'static str),
    /// A label with confidence and a streamed reason.
    Label(&'static str, f64, &'static str),
    /// The path a branch took.
    Route(&'static str),
    /// A posted message: channel, text.
    Message(&'static str, &'static str),
    /// A created record: id, title.
    Record(&'static str, &'static str),
    /// Free text, streamed.
    Text(&'static str),
}

/// One step.
#[derive(Clone, Debug, PartialEq)]
pub struct Step {
    /// Stable id.
    pub id: u32,
    /// Kind.
    pub kind: StepKind,
    /// App (a brand id, or `branch`).
    pub app: &'static str,
    /// Title.
    pub title: &'static str,
    /// Detail line.
    pub detail: &'static str,
    /// Activity line while running.
    pub activity: &'static str,
    /// Output.
    pub output: Output,
    /// Run time, ms.
    pub dur: u64,
    /// Fails on the first attempt (the Fails scenario).
    pub fails: bool,
}

/// A branch path.
#[derive(Clone, Debug, PartialEq)]
pub struct Path {
    /// Label.
    pub label: &'static str,
    /// Steps.
    pub steps: Vec<Step>,
}

/// A step's status at `now`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StepSt {
    /// No run yet.
    Idle,
    /// Waiting its turn.
    Pending,
    /// The dot is on its connector.
    Travel,
    /// Running.
    Running,
    /// Waiting for an approval (gate).
    Approval,
    /// Done in `ms`.
    Done(u64),
    /// Failed.
    Failed,
    /// On the untaken path.
    Skipped,
}

#[allow(clippy::too_many_arguments)]
fn st(
    id: u32,
    kind: StepKind,
    app: &'static str,
    title: &'static str,
    detail: &'static str,
    activity: &'static str,
    output: Output,
    dur: u64,
) -> Step {
    Step {
        id,
        kind,
        app,
        title,
        detail,
        activity,
        output,
        dur,
        fails: false,
    }
}

/// The add-step catalogue: kind, app, title, detail, activity, output, ms.
pub fn catalog() -> Vec<Step> {
    use StepKind::*;
    vec![
        st(
            0,
            Trigger,
            "gmail",
            "New email in Gmail",
            "support@corvel.dev",
            "Fetching the latest email",
            Output::Email(
                "Ines Varga",
                "Charged twice for October",
                "Hi, my card shows two charges of $49 for the same invoice.",
            ),
            500,
        ),
        st(
            0,
            Trigger,
            "slack",
            "New message in Slack",
            "#help channel",
            "Reading the message",
            Output::Message("#help", "Can someone check the export?"),
            450,
        ),
        st(
            0,
            Ai,
            "claude",
            "Summarise with Claude Sonnet 4.5",
            "Two sentences, plain words",
            "Reading the thread",
            Output::Text(
                "The customer was charged twice for one invoice and wants a refund today.",
            ),
            900,
        ),
        st(
            0,
            Ai,
            "openai",
            "Draft a reply with GPT-5",
            "Friendly, under 80 words",
            "Drafting the reply",
            Output::Text(
                "Thanks for flagging this. I can see the duplicate charge and have refunded it.",
            ),
            1000,
        ),
        st(
            0,
            Action,
            "slack",
            "Post to Slack",
            "#support, as Triage bot",
            "Posting the message",
            Output::Message(
                "#support",
                "Billing issue from Ines Varga, refund requested.",
            ),
            600,
        ),
        st(
            0,
            Action,
            "linear",
            "Create Linear issue",
            "Support team",
            "Creating the issue",
            Output::Record("SUP-482", "Charged twice for October"),
            700,
        ),
        st(
            0,
            Action,
            "notion",
            "Add a Notion page",
            "Support log database",
            "Creating the page",
            Output::Record("Support log", "Charged twice for October"),
            650,
        ),
        st(
            0,
            Action,
            "hubspot",
            "Update HubSpot contact",
            "Set lifecycle to Customer",
            "Updating the contact",
            Output::Record("Contact", "Ines Varga, Customer"),
            600,
        ),
        st(
            0,
            Action,
            "github",
            "Open GitHub issue",
            "corvel/billing",
            "Opening the issue",
            Output::Record("#1294", "Duplicate charge on retry"),
            700,
        ),
    ]
}

/// The live state of a workflow card.
#[derive(Clone, Debug, PartialEq)]
pub struct Workflow {
    /// Top-level steps (a branch, when present, is last).
    pub steps: Vec<Step>,
    /// Branch paths.
    pub paths: Vec<Path>,
    /// Path shown in the branch tabs.
    pub tab: usize,
    /// Path the run takes.
    pub taken: usize,
    /// Run start (`now`), when a run is going or finished.
    pub run: Option<u64>,
    /// Frozen run time after Stop.
    pub frozen: Option<u64>,
    /// Attempt of the failing step (0 first, 1 after Retry).
    pub attempt: u32,
    /// Ask before the first action (Oxyn gate variant).
    pub gate: bool,
    /// The gate was approved.
    pub approved: bool,
    /// Add menu open.
    pub menu: bool,
    /// Menu search.
    pub query: String,
    /// Highlighted menu row.
    pub hi: usize,
    /// Next step id.
    pub next_id: u32,
    /// Loading skeleton.
    pub loading: bool,
}

impl Workflow {
    /// `scenario`: `passes`, `fails` (the Linear step fails once), `empty` or `loading`.
    pub fn new(scenario: &str, gate: bool) -> Self {
        use StepKind::*;
        let mut linear = st(
            5,
            Action,
            "linear",
            "Create Linear issue",
            "Support team, priority Urgent",
            "Creating the issue",
            Output::Record("SUP-482", "Charged twice for October"),
            700,
        );
        linear.fails = scenario == "fails";
        let empty = scenario == "empty";
        Self {
            steps: if empty {
                vec![]
            } else {
                vec![
                    st(
                        1,
                        Trigger,
                        "gmail",
                        "New email in Gmail",
                        "support@corvel.dev",
                        "Fetching the latest email",
                        Output::Email(
                            "Ines Varga",
                            "Charged twice for October",
                            "Hi, my card shows two charges of $49 for the same invoice. Can you refund one today?",
                        ),
                        500,
                    ),
                    st(
                        2,
                        Ai,
                        "claude",
                        "Classify with Claude Sonnet 4.5",
                        "Urgent, billing, question or feedback",
                        "Reading the email",
                        Output::Label(
                            "Urgent",
                            0.94,
                            "A duplicate charge the customer wants refunded today, so money is at stake and time matters.",
                        ),
                        900,
                    ),
                    st(
                        3,
                        Branch,
                        "branch",
                        "Route by label",
                        "If label is Urgent",
                        "Checking the rule",
                        Output::Route("Urgent"),
                        300,
                    ),
                ]
            },
            paths: if empty {
                vec![]
            } else {
                vec![
                    Path {
                        label: "Urgent",
                        steps: vec![
                            st(
                                4,
                                Action,
                                "slack",
                                "Post to #support in Slack",
                                "As Triage bot, mentions @oncall",
                                "Posting the message",
                                Output::Message(
                                    "#support",
                                    "@oncall Urgent billing email from Ines Varga: charged twice for October.",
                                ),
                                600,
                            ),
                            linear,
                        ],
                    },
                    Path {
                        label: "Everything else",
                        steps: vec![st(
                            6,
                            Action,
                            "notion",
                            "Add a Notion page",
                            "Support log database",
                            "Creating the page",
                            Output::Record("Support log", "New email logged"),
                            650,
                        )],
                    },
                ]
            },
            tab: 0,
            taken: 0,
            run: None,
            frozen: None,
            attempt: 0,
            gate,
            approved: false,
            menu: false,
            query: String::new(),
            hi: 0,
            next_id: 7,
            loading: scenario == "loading",
        }
    }

    /// The run order: top-level steps, then the taken path.
    pub fn sequence(&self) -> Vec<&Step> {
        let mut v: Vec<&Step> = self.steps.iter().collect();
        if let Some(p) = self.paths.get(self.taken) {
            v.extend(p.steps.iter());
        }
        v
    }

    /// Start offset of each sequence step (travel included).
    fn starts(&self) -> Vec<u64> {
        let mut t = 0;
        self.sequence()
            .iter()
            .map(|s| {
                let at = t;
                t += TRAVEL_MS + s.dur;
                at
            })
            .collect()
    }

    fn gate_index(&self) -> Option<usize> {
        if !self.gate || self.approved {
            return None;
        }
        self.sequence()
            .iter()
            .position(|s| s.kind == StepKind::Action)
    }

    fn fail_index(&self) -> Option<usize> {
        if self.attempt > 0 {
            return None;
        }
        self.sequence().iter().position(|s| s.fails)
    }

    /// Run time at `now`, held at a gate or a failure.
    pub fn rt(&self, now: u64) -> Option<u64> {
        let start = self.run?;
        let mut t = self.frozen.unwrap_or_else(|| now.saturating_sub(start));
        let starts = self.starts();
        let seq = self.sequence();
        if let Some(g) = self.gate_index() {
            t = t.min(starts[g] + TRAVEL_MS);
        }
        if let Some(f) = self.fail_index() {
            t = t.min(starts[f] + TRAVEL_MS + seq[f].dur);
        }
        Some(t)
    }

    /// Total run length, ms.
    pub fn total(&self) -> u64 {
        self.sequence().iter().map(|s| TRAVEL_MS + s.dur).sum()
    }

    /// Status of step `id`.
    pub fn status(&self, id: u32, now: u64) -> StepSt {
        let Some(t) = self.rt(now) else {
            return StepSt::Idle;
        };
        let seq = self.sequence();
        let Some(k) = seq.iter().position(|s| s.id == id) else {
            // Not on the run order: on the untaken path once the branch ran.
            let b = seq.iter().position(|s| s.kind == StepKind::Branch);
            return match b {
                Some(b) if t >= self.starts()[b] + TRAVEL_MS + seq[b].dur => StepSt::Skipped,
                _ => StepSt::Pending,
            };
        };
        let s0 = self.starts()[k];
        let s = seq[k];
        if Some(k) == self.gate_index() && t >= s0 + TRAVEL_MS {
            return StepSt::Approval;
        }
        if t < s0 {
            StepSt::Pending
        } else if t < s0 + TRAVEL_MS {
            if self.frozen.is_some() {
                StepSt::Pending
            } else {
                StepSt::Travel
            }
        } else if t < s0 + TRAVEL_MS + s.dur {
            if self.frozen.is_some() {
                StepSt::Pending
            } else {
                StepSt::Running
            }
        } else if Some(k) == self.fail_index() {
            StepSt::Failed
        } else {
            StepSt::Done(s.dur)
        }
    }

    /// Ms since step `id` finished (for streaming its output), when done.
    pub fn since_done(&self, id: u32, now: u64) -> Option<u64> {
        let t = self.rt(now)?;
        let seq = self.sequence();
        let k = seq.iter().position(|s| s.id == id)?;
        let end = self.starts()[k] + TRAVEL_MS + seq[k].dur;
        (t >= end && self.status(id, now) != StepSt::Failed).then(|| t - end)
    }

    /// Words of a streamed output visible.
    pub fn stream_words(&self, id: u32, text: &str, now: u64) -> usize {
        self.since_done(id, now)
            .map(|e| words_at(text, e, 5))
            .unwrap_or(0)
    }

    /// The connector into step `id` is filled (the dot has passed).
    pub fn wire_fill(&self, id: u32, now: u64) -> f64 {
        let Some(t) = self.rt(now) else { return 0.0 };
        let seq = self.sequence();
        let Some(k) = seq.iter().position(|s| s.id == id) else {
            return 0.0;
        };
        let s0 = self.starts()[k];
        ((t.saturating_sub(s0)) as f64 / TRAVEL_MS as f64).clamp(0.0, 1.0)
    }

    /// True mid-run (not yet finished, failed, stopped or held).
    pub fn running(&self, now: u64) -> bool {
        match self.rt(now) {
            Some(t) => self.frozen.is_none() && t < self.total() && !self.held(now),
            None => false,
        }
    }

    /// The run waits on a gate or stopped on a failure.
    pub fn held(&self, now: u64) -> bool {
        self.sequence()
            .iter()
            .any(|s| matches!(self.status(s.id, now), StepSt::Approval | StepSt::Failed))
    }

    /// The outcome line for the footer.
    pub fn outcome(&self, now: u64) -> String {
        let n = self.steps.len() + self.paths.iter().map(|p| p.steps.len()).sum::<usize>();
        let Some(t) = self.rt(now) else {
            return format!("{n} steps, test runs are simulated");
        };
        if self
            .sequence()
            .iter()
            .any(|s| self.status(s.id, now) == StepSt::Failed)
        {
            return "Run failed, retry resumes from the failed step".into();
        }
        if self
            .sequence()
            .iter()
            .any(|s| self.status(s.id, now) == StepSt::Approval)
        {
            return "Waiting for your approval before the first action".into();
        }
        if self.frozen.is_some() {
            return "Stopped, simulated, nothing was sent".into();
        }
        if t >= self.total() {
            return format!(
                "Passed in {}, simulated, nothing was sent",
                super::secs(self.total())
            );
        }
        "Running the test".into()
    }

    /// Catalogue rows matching the search (triggers only when the flow is empty).
    pub fn menu_rows(&self) -> Vec<Step> {
        let q = self.query.to_lowercase();
        catalog()
            .into_iter()
            .filter(|s| (s.kind == StepKind::Trigger) == self.steps.is_empty())
            .filter(|s| {
                q.is_empty()
                    || s.title.to_lowercase().contains(&q)
                    || s.app.contains(&q)
                    || s.detail.to_lowercase().contains(&q)
            })
            .collect()
    }

    fn add(&mut self, mut s: Step) -> String {
        s.id = self.next_id;
        self.next_id += 1;
        let title = s.title;
        let to = if let Some(p) = self.paths.get_mut(self.tab) {
            p.steps.push(s);
            format!(" to {}", p.label)
        } else {
            self.steps.push(s);
            String::new()
        };
        self.menu = false;
        self.query.clear();
        self.hi = 0;
        self.run = None;
        format!("Added {title}{to}")
    }

    /// Presses: `wf-run` (run or stop), `wf-retry`, `wf-approve`, `wf-add` (menu), `wf-close`,
    /// `wf-pick-<i>` (menu row), `wf-rm-<id>`, `wf-path-<i>`, `wf-q:<text>` (search).
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "wf-pick-") {
            let rows = self.menu_rows();
            let s = rows.get(i)?.clone();
            return Some(self.add(s));
        }
        if let Some(i) = nth(id, "wf-rm-") {
            let i = i as u32;
            let mut title = None;
            if let Some(k) = self
                .steps
                .iter()
                .position(|s| s.id == i && s.kind != StepKind::Trigger)
            {
                let s = self.steps.remove(k);
                if s.kind == StepKind::Branch {
                    self.paths.clear();
                }
                title = Some(s.title);
            }
            for p in &mut self.paths {
                if let Some(k) = p.steps.iter().position(|s| s.id == i) {
                    title = Some(p.steps.remove(k).title);
                }
            }
            self.run = None;
            return title.map(|t| format!("Removed {t}"));
        }
        if let Some(i) = nth(id, "wf-path-") {
            self.tab = i.min(self.paths.len().saturating_sub(1));
            return self
                .paths
                .get(self.tab)
                .map(|p| format!("Path {}", p.label));
        }
        if let Some(q) = id.strip_prefix("wf-q:") {
            self.query = q.to_string();
            self.hi = 0;
            self.menu = true;
            return None;
        }
        match id {
            "wf-run" => {
                if self.loading || self.steps.is_empty() {
                    return None;
                }
                if self.running(now) {
                    self.frozen = self.rt(now);
                    return Some("Run stopped".into());
                }
                self.run = Some(now);
                self.frozen = None;
                self.attempt = 0;
                self.approved = false;
                Some("Test run started, simulated".into())
            }
            "wf-retry" => {
                let f = self.fail_index()?;
                let s0 = self.starts()[f];
                self.attempt = 1;
                self.run = Some(now.saturating_sub(s0));
                self.frozen = None;
                Some(format!("Retrying {}", self.sequence()[f].title))
            }
            "wf-approve" => {
                let g = self.gate_index()?;
                let s0 = self.starts()[g];
                self.approved = true;
                self.run = Some(now.saturating_sub(s0 + TRAVEL_MS));
                Some("Approved, running the actions".into())
            }
            "wf-add" => {
                if self.loading {
                    return None;
                }
                self.menu = !self.menu;
                self.query.clear();
                self.hi = 0;
                None
            }
            "wf-close" => {
                self.menu = false;
                None
            }
            _ => None,
        }
    }

    /// Keys in the open menu: ↑/↓ move, Enter adds, 1–9 add by shortcut (empty search),
    /// Escape closes. In the branch tabs (`tabs`): ←/→, Home, End.
    pub fn key(&mut self, part: &str, key: &str, now: u64) -> Option<String> {
        let key = norm_key(key);
        if part == "tabs" {
            let i = super::rove(self.tab, self.paths.len(), key);
            return self.press(&format!("wf-path-{i}"), now);
        }
        let n = self.menu_rows().len();
        match key {
            "ArrowDown" | "ArrowUp" | "Home" | "End" => {
                self.hi = super::rove(self.hi, n, key);
                None
            }
            "Enter" => self.press(&format!("wf-pick-{}", self.hi), now),
            "Escape" => {
                self.menu = false;
                None
            }
            d if self.query.is_empty() && d.len() == 1 && d.as_bytes()[0].is_ascii_digit() => {
                let i = (d.as_bytes()[0] - b'0') as usize;
                if i >= 1 && i <= n {
                    self.press(&format!("wf-pick-{}", i - 1), now)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn workflow_runs_fails_retries_and_edits() {
        let mut w = Workflow::new("passes", false);
        assert_eq!(w.status(1, 0), StepSt::Idle);
        w.press("wf-run", 0);
        assert_eq!(w.status(1, 100), StepSt::Travel);
        assert_eq!(w.status(1, 300), StepSt::Running);
        assert_eq!(w.status(1, 800), StepSt::Done(500));
        assert!(w.running(800));
        let end = w.total();
        assert_eq!(w.status(6, end), StepSt::Skipped);
        assert!(w.outcome(end).starts_with("Passed"));
        assert!(w.stream_words(2, "a b c d e f g h i j k", end) > 0);

        let mut f = Workflow::new("fails", false);
        f.press("wf-run", 0);
        let t = 100_000;
        assert_eq!(f.status(5, t), StepSt::Failed);
        assert!(f.held(t));
        f.press("wf-retry", t);
        assert_eq!(f.status(4, t), StepSt::Done(600));
        assert_eq!(f.status(5, t + 100), StepSt::Travel);
        assert!(f.outcome(t + 10_000).starts_with("Passed"));

        let mut g = Workflow::new("passes", true);
        g.press("wf-run", 0);
        assert_eq!(g.status(4, 100_000), StepSt::Approval);
        g.press("wf-approve", 100_000);
        assert_eq!(g.status(4, 100_100), StepSt::Running);

        let mut e = Workflow::new("passes", false);
        e.press("wf-add", 0);
        e.press("wf-q:notion", 0);
        assert_eq!(e.menu_rows().len(), 1);
        e.key("menu", "Enter", 0);
        assert_eq!(e.paths[0].steps.len(), 3);
        e.press("wf-add", 0);
        e.key("menu", "2", 0);
        assert_eq!(e.paths[0].steps.len(), 4);
        assert!(e.press("wf-rm-1", 0).is_none());
        assert!(e.press("wf-rm-4", 0).is_some());
        e.key("tabs", "ArrowRight", 0);
        assert_eq!(e.tab, 1);
        let mut z = Workflow::new("empty", false);
        assert!(z.menu_rows().iter().all(|s| s.kind == StepKind::Trigger));
        z.press("wf-pick-0", 0);
        assert_eq!(z.steps.len(), 1);
    }
}
