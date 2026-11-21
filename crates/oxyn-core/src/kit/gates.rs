//! Run-level approvals.
//!
//! - [`Gates`]: every approval request of a run as a stack (open, the one to decide on top) and a
//!   Decided list; approve, edit-then-approve or reject, and each open gate holds the steps
//!   behind it.
//! - [`PermissionRequest`]: one agent asks for one right; allow once, for a while or always, then
//!   it folds to a line with the time left and Revoke.
//! - [`AgentApprovals`]: a queue of tool calls with editable arguments, always-allow rules (per
//!   agent or workspace) that skip later asks, and a decision log.

use std::time::Duration;

use crate::ClockMs;

/// What a gate asks about.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateKind {
    /// A tool call with arguments.
    Tool,
    /// A file change with a diff.
    File,
    /// Handing work to another agent.
    Agent,
    /// Sending a message.
    Message,
}

impl GateKind {
    /// Kebab id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Tool => "tool",
            Self::File => "file",
            Self::Agent => "agent",
            Self::Message => "message",
        }
    }
    /// Icon name.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Tool => "tool",
            Self::File => "file",
            Self::Agent => "agents",
            Self::Message => "mail",
        }
    }
}

/// Where a gate stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GateStatus {
    /// Waiting.
    Open,
    /// Approved as asked.
    Approved,
    /// Approved after a change.
    Edited,
    /// Rejected.
    Rejected,
}

impl GateStatus {
    /// Kebab id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Approved => "approved",
            Self::Edited => "edited",
            Self::Rejected => "rejected",
        }
    }
    /// Past-tense label for the Decided list.
    pub fn label(self) -> &'static str {
        match self {
            Self::Open => "Waiting",
            Self::Approved => "Approved",
            Self::Edited => "Edited",
            Self::Rejected => "Rejected",
        }
    }
}

/// One approval request of a run.
#[derive(Clone, Debug, PartialEq)]
pub struct Gate {
    /// Id.
    pub id: String,
    /// Agent asking.
    pub agent: String,
    /// Kind.
    pub kind: GateKind,
    /// What exactly ("Send 42 emails").
    pub action: String,
    /// Why, in the agent's words.
    pub reason: String,
    /// Status.
    pub status: GateStatus,
    /// Input: arguments or prompt.
    pub input: Option<String>,
    /// Diff lines (`+`/`-`/` ` prefixed) for file gates.
    pub diff: Vec<String>,
    /// Steps of the run held behind this gate.
    pub holds: usize,
    /// Note left with the decision (the edit or the rejection reason).
    pub note: Option<String>,
    /// Opened at.
    pub at: ClockMs,
}

impl Gate {
    /// An open gate.
    pub fn new(id: &str, agent: &str, kind: GateKind, action: &str, reason: &str) -> Self {
        Self {
            id: id.into(),
            agent: agent.into(),
            kind,
            action: action.into(),
            reason: reason.into(),
            status: GateStatus::Open,
            input: None,
            diff: Vec::new(),
            holds: 0,
            note: None,
            at: ClockMs(0),
        }
    }
    /// With input.
    pub fn input(mut self, s: &str) -> Self {
        self.input = Some(s.into());
        self
    }
    /// With a diff.
    pub fn diff(mut self, lines: &[&str]) -> Self {
        self.diff = lines.iter().map(|s| s.to_string()).collect();
        self
    }
    /// Holding `n` steps.
    pub fn holds(mut self, n: usize) -> Self {
        self.holds = n;
        self
    }
    /// With a status.
    pub fn status(mut self, s: GateStatus) -> Self {
        self.status = s;
        self
    }
    /// `+a −d` for the diff.
    pub fn diff_stat(&self) -> Option<String> {
        if self.diff.is_empty() {
            return None;
        }
        let a = self.diff.iter().filter(|l| l.starts_with('+')).count();
        let d = self.diff.iter().filter(|l| l.starts_with('-')).count();
        Some(format!("+{a} −{d}"))
    }
}

/// A decision on a gate.
#[derive(Clone, Debug, PartialEq)]
pub enum GateDecision {
    /// Approve as asked.
    Approve,
    /// Approve with a changed input.
    Edit(String),
    /// Reject with an optional reason.
    Reject(Option<String>),
}

/// The approval gates of a run.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Gates {
    /// What the run is about.
    pub task: String,
    /// Run still going.
    pub live: bool,
    /// Every gate in the order they came up.
    pub gates: Vec<Gate>,
}

impl Gates {
    /// Open gates, oldest first (the first one is on top of the stack).
    pub fn open(&self) -> Vec<&Gate> {
        self.gates
            .iter()
            .filter(|g| g.status == GateStatus::Open)
            .collect()
    }
    /// Decided gates, newest decision first.
    pub fn decided(&self) -> Vec<&Gate> {
        let mut v: Vec<&Gate> = self
            .gates
            .iter()
            .filter(|g| g.status != GateStatus::Open)
            .collect();
        v.reverse();
        v
    }
    /// Header status: "2 waiting · holds 5 steps", "Running", "All decided".
    pub fn status_line(&self) -> String {
        let open = self.open();
        if open.is_empty() {
            return if self.live { "Running" } else { "All decided" }.into();
        }
        let holds: usize = open.iter().map(|g| g.holds).sum();
        let mut s = format!("{} waiting", open.len());
        if holds > 0 {
            s.push_str(&format!(
                " · holds {holds} step{}",
                if holds == 1 { "" } else { "s" }
            ));
        }
        s
    }
    /// Decides gate `id`. Returns `false` when it is not open.
    pub fn decide(&mut self, id: &str, d: GateDecision) -> bool {
        let Some(g) = self
            .gates
            .iter_mut()
            .find(|g| g.id == id && g.status == GateStatus::Open)
        else {
            return false;
        };
        match d {
            GateDecision::Approve => g.status = GateStatus::Approved,
            GateDecision::Edit(v) => {
                g.status = GateStatus::Edited;
                g.note = Some(v);
            }
            GateDecision::Reject(r) => {
                g.status = GateStatus::Rejected;
                g.note = r;
            }
        }
        true
    }
    /// A new request slides in behind the stack.
    pub fn push(&mut self, g: Gate) {
        self.gates.push(g);
    }
}

/// How dangerous a permission is.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PermRisk {
    /// No tag.
    Safe,
    /// "Risky" tag.
    #[default]
    Risky,
    /// "Destructive" tag, a red Allow and no Always.
    Destructive,
}

impl PermRisk {
    /// From a kebab id.
    pub fn parse(s: &str) -> Self {
        match s {
            "safe" => Self::Safe,
            "destructive" => Self::Destructive,
            _ => Self::Risky,
        }
    }
    /// Kebab id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Risky => "risky",
            Self::Destructive => "destructive",
        }
    }
}

/// How long a grant holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// This one time.
    Once,
    /// For `duration` minutes.
    Timed,
    /// From now on.
    Always,
}

impl Scope {
    /// Label.
    pub fn label(self, minutes: u64) -> String {
        match self {
            Self::Once => "Once".into(),
            Self::Timed if minutes.is_multiple_of(60) => format!("For {} h", minutes / 60),
            Self::Timed => format!("For {minutes} min"),
            Self::Always => "Always".into(),
        }
    }
    /// Kebab id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Once => "once",
            Self::Timed => "timed",
            Self::Always => "always",
        }
    }
}

/// A recorded permission decision.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PermDecision {
    /// Allowed (else denied).
    pub allow: bool,
    /// Scope.
    pub scope: Scope,
    /// When.
    pub at: ClockMs,
    /// Expiry for a timed grant.
    pub until: Option<ClockMs>,
}

/// One row of "What will happen".
#[derive(Clone, Debug, PartialEq)]
pub struct PreviewRow {
    /// Label (file, recipient, amount).
    pub label: String,
    /// Right-aligned meta ("+12 −4").
    pub meta: Option<String>,
    /// Icon name.
    pub icon: &'static str,
}

/// A permission request.
#[derive(Clone, Debug, PartialEq)]
pub struct PermissionRequest {
    /// Agent name.
    pub agent: String,
    /// Model, shown muted.
    pub model: Option<String>,
    /// What it wants to do, read after "wants to".
    pub action: String,
    /// Where.
    pub target: String,
    /// Risk.
    pub risk: PermRisk,
    /// The agent's reason.
    pub reason: String,
    /// What will happen.
    pub preview: Vec<PreviewRow>,
    /// Rows shown before "N more".
    pub preview_limit: usize,
    /// Choices of how long.
    pub scopes: Vec<Scope>,
    /// Chosen scope.
    pub scope: Scope,
    /// Minutes a timed grant holds.
    pub duration_min: u64,
    /// The decision, `None` while it waits.
    pub decision: Option<PermDecision>,
    /// Preview expanded past the limit.
    pub expanded: bool,
}

impl PermissionRequest {
    /// A request.
    pub fn new(agent: &str, action: &str, target: &str) -> Self {
        Self {
            agent: agent.into(),
            model: None,
            action: action.into(),
            target: target.into(),
            risk: PermRisk::Risky,
            reason: String::new(),
            preview: Vec::new(),
            preview_limit: 3,
            scopes: vec![Scope::Once, Scope::Timed, Scope::Always],
            scope: Scope::Once,
            duration_min: 60,
            decision: None,
            expanded: false,
        }
    }
    /// Scopes on offer: destructive drops Always.
    pub fn offered(&self) -> Vec<Scope> {
        self.scopes
            .iter()
            .copied()
            .filter(|s| !(self.risk == PermRisk::Destructive && *s == Scope::Always))
            .collect()
    }
    /// Visible preview rows and the "N more" count.
    pub fn visible_preview(&self) -> (&[PreviewRow], usize) {
        if self.expanded || self.preview.len() <= self.preview_limit {
            (&self.preview, 0)
        } else {
            (
                &self.preview[..self.preview_limit],
                self.preview.len() - self.preview_limit,
            )
        }
    }
    /// Allow with the chosen scope.
    pub fn allow(&mut self, now: ClockMs) {
        let scope = if self.offered().contains(&self.scope) {
            self.scope
        } else {
            Scope::Once
        };
        let until =
            (scope == Scope::Timed).then(|| now.plus(Duration::from_secs(self.duration_min * 60)));
        self.decision = Some(PermDecision {
            allow: true,
            scope,
            at: now,
            until,
        });
    }
    /// Deny.
    pub fn deny(&mut self, now: ClockMs) {
        self.decision = Some(PermDecision {
            allow: false,
            scope: Scope::Once,
            at: now,
            until: None,
        });
    }
    /// Revoke, Change or Ask again: back to waiting.
    pub fn revoke(&mut self) {
        self.decision = None;
    }
    /// Is a timed grant still running.
    pub fn active(&self, now: ClockMs) -> bool {
        match self.decision {
            Some(PermDecision {
                allow: true,
                until: Some(u),
                ..
            }) => now < u,
            Some(PermDecision { allow: true, .. }) => true,
            _ => false,
        }
    }
    /// The folded line: "Allowed for 58 min", "Allowed always", "Allowed once", "Denied",
    /// "Expired".
    pub fn folded(&self, now: ClockMs) -> Option<String> {
        let d = self.decision?;
        Some(if !d.allow {
            "Denied".into()
        } else {
            match (d.scope, d.until) {
                (Scope::Timed, Some(u)) if now >= u => "Expired".into(),
                (Scope::Timed, Some(u)) => {
                    let left = u.0.saturating_sub(now.0) / 1000;
                    if left >= 60 {
                        format!("Allowed · {} min left", left.div_ceil(60))
                    } else {
                        format!("Allowed · {left} s left")
                    }
                }
                (Scope::Always, _) => "Allowed always".into(),
                _ => "Allowed once".into(),
            }
        })
    }
}

/// Outcome of an agent approval.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    /// Approved.
    Approved,
    /// Denied.
    Denied,
    /// Covered by an always-allow rule.
    Allowed,
}

impl Outcome {
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Approved => "Approved",
            Self::Denied => "Denied",
            Self::Allowed => "Auto-allowed",
        }
    }
}

/// One argument of a tool call.
#[derive(Clone, Debug, PartialEq)]
pub struct Arg {
    /// Key.
    pub key: String,
    /// Value.
    pub value: String,
    /// Can be edited before approving.
    pub editable: bool,
}

/// A pending tool call.
#[derive(Clone, Debug, PartialEq)]
pub struct ToolAsk {
    /// Id.
    pub id: String,
    /// Agent name.
    pub agent: String,
    /// Model.
    pub model: String,
    /// Tool id the rules match on (`vercel.deploy`).
    pub tool: String,
    /// Permission name in the rules list ("Deploy to production").
    pub permission: String,
    /// Title, with backticks for code.
    pub title: String,
    /// Short label for the queue and log.
    pub summary: String,
    /// Reason.
    pub reason: String,
    /// Arguments.
    pub args: Vec<Arg>,
    /// Risk id (`low` · `medium` · `high`).
    pub risk: &'static str,
}

/// Always-allow rule scope.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RuleScope {
    /// Only for this agent.
    Agent(String),
    /// For every agent in the workspace.
    Workspace,
}

/// An always-allow rule.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    /// Tool id.
    pub tool: String,
    /// Permission name.
    pub permission: String,
    /// Scope.
    pub scope: RuleScope,
}

impl Rule {
    /// Covers a request.
    pub fn covers(&self, a: &ToolAsk) -> bool {
        self.tool == a.tool
            && match &self.scope {
                RuleScope::Workspace => true,
                RuleScope::Agent(n) => *n == a.agent,
            }
    }
}

/// A log entry.
#[derive(Clone, Debug, PartialEq)]
pub struct LogEntry {
    /// Agent.
    pub agent: String,
    /// Summary.
    pub summary: String,
    /// Outcome.
    pub outcome: Outcome,
    /// Arguments were edited.
    pub edited: bool,
    /// Who decided.
    pub by: String,
}

/// The agent approvals block.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct AgentApprovals {
    /// Pending, in queue order.
    pub queue: Vec<ToolAsk>,
    /// Always-allow rules.
    pub rules: Vec<Rule>,
    /// Decisions, newest first.
    pub log: Vec<LogEntry>,
    /// Reviewer name.
    pub reviewer: String,
    /// Workspace name.
    pub workspace: String,
}

impl AgentApprovals {
    /// The request on top.
    pub fn current(&self) -> Option<&ToolAsk> {
        self.queue.first()
    }
    fn record(&mut self, a: &ToolAsk, outcome: Outcome, edited: bool) {
        self.log.insert(
            0,
            LogEntry {
                agent: a.agent.clone(),
                summary: a.summary.clone(),
                outcome,
                edited,
                by: if outcome == Outcome::Allowed {
                    "rule".into()
                } else {
                    self.reviewer.clone()
                },
            },
        );
    }
    /// Edits an argument of the top request.
    pub fn edit_arg(&mut self, key: &str, value: &str) -> bool {
        let Some(a) = self.queue.first_mut() else {
            return false;
        };
        match a.args.iter_mut().find(|x| x.key == key && x.editable) {
            Some(x) => {
                x.value = value.into();
                true
            }
            None => false,
        }
    }
    /// Approves or denies the top request.
    pub fn decide(&mut self, approve: bool, edited: bool) {
        if self.queue.is_empty() {
            return;
        }
        let a = self.queue.remove(0);
        self.record(
            &a,
            if approve {
                Outcome::Approved
            } else {
                Outcome::Denied
            },
            edited,
        );
    }
    /// Always allow: approves the top request, adds a rule, and allows every queued request the
    /// rule covers. Returns how many others it covered.
    pub fn always(&mut self, scope: RuleScope) -> usize {
        let Some(a) = self.queue.first().cloned() else {
            return 0;
        };
        let rule = Rule {
            tool: a.tool.clone(),
            permission: a.permission.clone(),
            scope,
        };
        self.decide(true, false);
        let (covered, rest): (Vec<ToolAsk>, Vec<ToolAsk>) =
            self.queue.drain(..).partition(|x| rule.covers(x));
        self.queue = rest;
        for c in &covered {
            self.record(c, Outcome::Allowed, false);
        }
        self.rules.push(rule);
        covered.len()
    }
    /// Removes rule `i`.
    pub fn remove_rule(&mut self, i: usize) {
        if i < self.rules.len() {
            self.rules.remove(i);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gates_stack_and_status() {
        let mut g = Gates {
            task: "Ship".into(),
            live: true,
            gates: vec![
                Gate::new("a", "Writer", GateKind::Message, "Send", "why").holds(2),
                Gate::new("b", "Coder", GateKind::File, "Edit", "why")
                    .diff(&["+a", "-b", "+c"])
                    .holds(1),
            ],
        };
        assert_eq!(g.status_line(), "2 waiting · holds 3 steps");
        assert_eq!(g.gates[1].diff_stat().as_deref(), Some("+2 −1"));
        assert!(g.decide("a", GateDecision::Edit("to: team".into())));
        assert!(!g.decide("a", GateDecision::Approve));
        g.decide("b", GateDecision::Reject(None));
        assert_eq!(g.status_line(), "Running");
        assert_eq!(g.decided()[0].id, "b");
    }

    #[test]
    fn permission_timed_and_destructive() {
        let mut p = PermissionRequest::new("Coder", "push to main", "acme/web");
        p.scope = Scope::Timed;
        p.allow(ClockMs(0));
        assert_eq!(p.folded(ClockMs(60_000)).unwrap(), "Allowed · 59 min left");
        assert!(p.active(ClockMs(60_000)));
        assert_eq!(p.folded(ClockMs(3_600_000)).unwrap(), "Expired");
        p.revoke();
        p.risk = PermRisk::Destructive;
        p.scope = Scope::Always;
        assert!(!p.offered().contains(&Scope::Always));
        p.allow(ClockMs(0));
        assert_eq!(p.decision.unwrap().scope, Scope::Once);
    }

    #[test]
    fn always_rule_covers_queue() {
        let ask = |id: &str, agent: &str, tool: &str| ToolAsk {
            id: id.into(),
            agent: agent.into(),
            model: "m".into(),
            tool: tool.into(),
            permission: "p".into(),
            title: "t".into(),
            summary: id.into(),
            reason: "r".into(),
            args: vec![Arg {
                key: "env".into(),
                value: "prod".into(),
                editable: true,
            }],
            risk: "high",
        };
        let mut a = AgentApprovals {
            queue: vec![
                ask("1", "A", "deploy"),
                ask("2", "A", "deploy"),
                ask("3", "B", "deploy"),
            ],
            reviewer: "Emma".into(),
            ..Default::default()
        };
        assert!(a.edit_arg("env", "preview"));
        assert_eq!(a.always(RuleScope::Agent("A".into())), 1);
        assert_eq!(a.queue.len(), 1);
        assert_eq!(a.log[0].outcome, Outcome::Allowed);
        a.decide(false, false);
        assert_eq!(a.log[0].outcome, Outcome::Denied);
    }
}
