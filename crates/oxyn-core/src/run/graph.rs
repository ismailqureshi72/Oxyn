//! Sub-agent graph (`subagent-tree.md`): per-node state derived from each node's `AgentState`
//! plus the graph, roll-up of the most urgent descendant, and the flattened rows a tree draws.

use std::collections::BTreeSet;
use std::time::Duration;

use super::{Tone, format_call_duration};
use crate::{AgentState, ClockMs, VerbPack, verbs};

/// Tree variants (§6, free).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TreeVariant {
    /// Indented rows with chevrons.
    #[default]
    Indented,
    /// Connector lines (├ └) with result travel.
    Rail,
    /// Live children only under the composer; finished ones fold into "3 done".
    Dock,
    /// One collapsible tool row in the parent transcript.
    InlineWrapper,
    /// Box-drawing tree with braille spinners.
    Terminal,
}

impl TreeVariant {
    /// Every free variant.
    pub const ALL: [Self; 5] = [
        Self::Indented,
        Self::Rail,
        Self::Dock,
        Self::InlineWrapper,
        Self::Terminal,
    ];

    /// Registry id `subagent-tree/<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Indented => "indented",
            Self::Rail => "rail",
            Self::Dock => "dock",
            Self::InlineWrapper => "inline-wrapper",
            Self::Terminal => "terminal",
        }
    }
}

/// One agent in the run.
#[derive(Clone, Debug, PartialEq)]
pub struct AgentNode {
    /// Id.
    pub id: String,
    /// Parent id (`None` for the root).
    pub parent: Option<String>,
    /// Name ("Researcher").
    pub name: String,
    /// Identity colour index `0..8` (`color.agent.*`); never carries state.
    pub colour: u8,
    /// This node's state.
    pub state: AgentState,
    /// Started.
    pub started: Option<ClockMs>,
    /// Ended.
    pub ended: Option<ClockMs>,
    /// Time waiting on the user.
    pub waited: Duration,
    /// Cheap status line from the adapter ("Reading 14 files").
    pub summary: Option<String>,
    /// One-line result ("2 findings").
    pub result: Option<String>,
    /// The approval or question it waits on ("run `cargo test`").
    pub request: Option<String>,
    /// Survives its parent.
    pub detached: bool,
    /// Not started: depth limit.
    pub refused: Option<u8>,
    /// Parked by the app.
    pub hibernating: bool,
}

impl AgentNode {
    /// A node.
    pub fn new(id: &str, parent: Option<&str>, name: &str, colour: u8, state: AgentState) -> Self {
        Self {
            id: id.into(),
            parent: parent.map(str::to_string),
            name: name.into(),
            colour,
            state,
            started: None,
            ended: None,
            waited: Duration::ZERO,
            summary: None,
            result: None,
            request: None,
            detached: false,
            refused: None,
            hibernating: false,
        }
    }
}

/// A node's displayed state (§2).
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum NodeState {
    /// Queued by the parent.
    Pending,
    /// Just started (one pulse).
    Starting,
    /// Working, with a verb.
    Working(String),
    /// Running a tool.
    ToolRunning(String),
    /// Waiting on n children.
    WaitingOnChildren(usize),
    /// Needs the user.
    AwaitingYou(String),
    /// No events.
    Stalled,
    /// Done with a result line.
    Done(String),
    /// Failed.
    Failed(String),
    /// Stopped.
    Interrupted(Duration),
    /// Parent ended first; still running.
    Orphaned(String),
    /// Not started: depth limit.
    Refused(u8),
    /// Parked.
    Hibernating,
}

impl NodeState {
    /// Roll-up priority: higher wins (§1 rule 2).
    pub fn urgency(&self) -> u8 {
        match self {
            Self::AwaitingYou(_) => 5,
            Self::Failed(_) => 4,
            Self::Stalled => 3,
            Self::Working(_)
            | Self::ToolRunning(_)
            | Self::Starting
            | Self::WaitingOnChildren(_)
            | Self::Orphaned(_) => 2,
            Self::Done(_) => 1,
            _ => 0,
        }
    }

    /// Live (animates, counts as running).
    pub fn is_live(&self) -> bool {
        matches!(
            self,
            Self::Starting
                | Self::Working(_)
                | Self::ToolRunning(_)
                | Self::WaitingOnChildren(_)
                | Self::Orphaned(_)
                | Self::Stalled
        )
    }

    /// The dot glyph.
    pub fn glyph(&self) -> &'static str {
        match self {
            Self::Pending | Self::Refused(_) => "○",
            Self::AwaitingYou(_) => "⚠",
            Self::Done(_) => "✓",
            Self::Failed(_) => "✕",
            Self::Interrupted(_) => "■",
            Self::Hibernating => "☾",
            _ => "●",
        }
    }

    /// Tone (status colours; identity colour is separate).
    pub fn tone(&self) -> Tone {
        match self {
            Self::AwaitingYou(_) => Tone::Waiting,
            Self::Failed(_) => Tone::Danger,
            Self::Stalled => Tone::Warn,
            Self::Done(_) => Tone::Success,
            Self::Working(_) | Self::ToolRunning(_) | Self::Starting | Self::Orphaned(_) => {
                Tone::Default
            }
            _ => Tone::Muted,
        }
    }

    /// The row's text.
    pub fn line(&self) -> String {
        match self {
            Self::Pending => "Queued".into(),
            Self::Starting => "Starting".into(),
            Self::Working(v) | Self::ToolRunning(v) | Self::Orphaned(v) => v.clone(),
            Self::WaitingOnChildren(n) => {
                format!(
                    "Waiting on {n} {}",
                    if *n == 1 { "agent" } else { "agents" }
                )
            }
            Self::AwaitingYou(r) => format!("Needs approval: {r}"),
            Self::Stalled => "No response".into(),
            Self::Done(r) => format!("Returned · {r}"),
            Self::Failed(k) => format!("Failed · {k}"),
            Self::Interrupted(d) => format!("Interrupted after {}", verbs::format_summary(*d)),
            Self::Refused(n) => format!("Not started · depth limit {n}"),
            Self::Hibernating => "Hibernating".into(),
        }
    }
}

/// The run's agents, in spawn order.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RunGraph {
    /// Nodes.
    pub nodes: Vec<AgentNode>,
}

/// One row of a drawn tree.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeRow {
    /// Node id (empty for a truncated row).
    pub id: String,
    /// Depth (0 = root).
    pub depth: usize,
    /// Connector prefix for rail / terminal (`│ ├─`).
    pub rail: String,
    /// Name.
    pub name: String,
    /// Identity colour index.
    pub colour: u8,
    /// State.
    pub state: NodeState,
    /// Elapsed text.
    pub elapsed: Option<String>,
    /// Timer paused (waiting on you).
    pub paused: bool,
    /// Children hidden under this row.
    pub hidden_children: usize,
    /// Most urgent hidden descendant state (roll-up badge).
    pub rollup: Option<NodeState>,
    /// Expanded.
    pub expanded: bool,
    /// Animates (at most 8 rows do).
    pub animate: bool,
    /// Parent ended first.
    pub detached: bool,
    /// "+ 4 nested agents" placeholder.
    pub truncated: Option<String>,
    /// Accessible name.
    pub aria: String,
}

/// A whole tree frame.
#[derive(Clone, Debug, PartialEq)]
pub struct TreeView {
    /// Header ("Agents · 2 running · 1 needs you").
    pub header: String,
    /// Rows.
    pub rows: Vec<TreeRow>,
    /// Dock only: finished agents folded ("3 done").
    pub folded_done: usize,
}

/// Most rows animating at once.
pub const MAX_ANIMATING: usize = 8;

impl RunGraph {
    fn children<'a>(&'a self, id: &'a str) -> impl Iterator<Item = &'a AgentNode> + 'a {
        self.nodes
            .iter()
            .filter(move |n| n.parent.as_deref() == Some(id))
    }

    fn get(&self, id: &str) -> Option<&AgentNode> {
        self.nodes.iter().find(|n| n.id == id)
    }

    /// A node's displayed state (§2 mapping, plus graph-derived states).
    pub fn node_state(&self, n: &AgentNode, pack: &VerbPack) -> NodeState {
        if let Some(d) = n.refused {
            return NodeState::Refused(d);
        }
        if n.hibernating {
            return NodeState::Hibernating;
        }
        let live_kids = self
            .children(&n.id)
            .filter(|c| self.node_state(c, pack).is_live())
            .count();
        let parent_done = n
            .parent
            .as_deref()
            .and_then(|p| self.get(p))
            .is_some_and(|p| {
                matches!(
                    p.state,
                    AgentState::Done { .. }
                        | AgentState::Cancelled { .. }
                        | AgentState::Error { .. }
                )
            });
        let verb = |s: &AgentState| -> String {
            n.summary.clone().unwrap_or_else(|| match s {
                AgentState::ToolRunning { tool, .. } => {
                    let v = pack.activity(tool.activity).present;
                    match &tool.target {
                        Some(t) => format!("{v} {t}"),
                        None => v,
                    }
                }
                AgentState::Thinking { header: Some(h) } => h.clone(),
                AgentState::Thinking { .. } => pack.activity(crate::ActivityKind::Think).present,
                _ => "Working".into(),
            })
        };
        let s = match &n.state {
            AgentState::Idle | AgentState::Composing => NodeState::Pending,
            AgentState::Submitted => NodeState::Starting,
            AgentState::ToolRunning { .. } => NodeState::ToolRunning(verb(&n.state)),
            AgentState::Thinking { .. }
            | AgentState::Streaming
            | AgentState::Queued { .. }
            | AgentState::Steering => {
                if live_kids > 0 {
                    NodeState::WaitingOnChildren(live_kids)
                } else {
                    NodeState::Working(verb(&n.state))
                }
            }
            AgentState::AwaitingApproval { .. } => {
                NodeState::AwaitingYou(n.request.clone().unwrap_or_else(|| "review".into()))
            }
            AgentState::Stalled { .. } => {
                if live_kids > 0 {
                    NodeState::WaitingOnChildren(live_kids)
                } else {
                    NodeState::Stalled
                }
            }
            AgentState::Done { .. } => {
                NodeState::Done(n.result.clone().unwrap_or_else(|| "done".into()))
            }
            AgentState::Error { kind } => NodeState::Failed(verbs::error_label(kind).to_string()),
            AgentState::Cancelled { after } => NodeState::Interrupted(*after),
        };
        if parent_done && s.is_live() && n.detached {
            NodeState::Orphaned(s.line())
        } else {
            s
        }
    }

    fn descendants_urgent(&self, id: &str, pack: &VerbPack) -> Option<NodeState> {
        let mut best: Option<NodeState> = None;
        for c in self.children(id) {
            for s in [
                Some(self.node_state(c, pack)),
                self.descendants_urgent(&c.id, pack),
            ]
            .into_iter()
            .flatten()
            {
                if best.as_ref().is_none_or(|b| s.urgency() > b.urgency()) {
                    best = Some(s);
                }
            }
        }
        best
    }

    fn count_desc(&self, id: &str) -> usize {
        self.children(id).map(|c| 1 + self.count_desc(&c.id)).sum()
    }

    /// Flattens the tree for drawing. `collapsed` holds ids the user folded; levels deeper than
    /// `display_depth` fold into "+ n nested agents"; more than `max_children` siblings into
    /// "+ n more".
    pub fn view(
        &self,
        variant: TreeVariant,
        collapsed: &BTreeSet<String>,
        display_depth: usize,
        max_children: usize,
        pack: &VerbPack,
        now: ClockMs,
    ) -> TreeView {
        let states: Vec<NodeState> = self
            .nodes
            .iter()
            .map(|n| self.node_state(n, pack))
            .collect();
        let running = states.iter().filter(|s| s.is_live()).count();
        let needs = states
            .iter()
            .filter(|s| matches!(s, NodeState::AwaitingYou(_)))
            .count();
        let failed = states
            .iter()
            .filter(|s| matches!(s, NodeState::Failed(_)))
            .count();
        let mut header = String::from("Agents");
        if running > 0 {
            header.push_str(&format!(" · {running} running"));
        }
        if needs > 0 {
            header.push_str(&format!(" · {needs} needs you"));
        }
        if failed > 0 {
            header.push_str(&format!(" · {failed} failed"));
        }
        let mut rows = vec![];
        let mut folded_done = 0;
        let roots: Vec<&AgentNode> = self.nodes.iter().filter(|n| n.parent.is_none()).collect();
        let mut animating = 0usize;
        let n_roots = roots.len();
        for (i, r) in roots.into_iter().enumerate() {
            self.walk(
                r,
                0,
                String::new(),
                i + 1 == n_roots,
                variant,
                collapsed,
                display_depth,
                max_children,
                pack,
                now,
                &mut rows,
                &mut animating,
                &mut folded_done,
            );
        }
        TreeView {
            header,
            rows,
            folded_done,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn walk(
        &self,
        n: &AgentNode,
        depth: usize,
        prefix: String,
        last: bool,
        variant: TreeVariant,
        collapsed: &BTreeSet<String>,
        display_depth: usize,
        max_children: usize,
        pack: &VerbPack,
        now: ClockMs,
        rows: &mut Vec<TreeRow>,
        animating: &mut usize,
        folded_done: &mut usize,
    ) {
        let state = self.node_state(n, pack);
        let kids: Vec<&AgentNode> = self.children(&n.id).collect();
        if variant == TreeVariant::Dock
            && depth > 0
            && !state.is_live()
            && !matches!(state, NodeState::AwaitingYou(_) | NodeState::Failed(_))
        {
            *folded_done += 1 + self.count_desc(&n.id);
            return;
        }
        let expanded = !collapsed.contains(&n.id) && depth + 1 < display_depth;
        let rail = if depth == 0 {
            String::new()
        } else {
            format!("{prefix}{}", if last { "└─" } else { "├─" })
        };
        let animate = state.is_live() && *animating < MAX_ANIMATING;
        if animate {
            *animating += 1;
        }
        let paused = matches!(state, NodeState::AwaitingYou(_));
        let elapsed = n.started.map(|s| {
            let d = n.ended.unwrap_or(now).since(s).saturating_sub(n.waited);
            format_call_duration(d)
        });
        let rollup = (!expanded && !kids.is_empty())
            .then(|| self.descendants_urgent(&n.id, pack))
            .flatten()
            .filter(|s| {
                s.urgency() > state.urgency().min(2)
                    || matches!(s, NodeState::AwaitingYou(_) | NodeState::Failed(_))
            });
        rows.push(TreeRow {
            id: n.id.clone(),
            depth,
            rail,
            name: n.name.clone(),
            colour: n.colour % 8,
            aria: format!(
                "{}, {}{}",
                n.name,
                state.line(),
                elapsed
                    .as_ref()
                    .map(|e| format!(", {e}"))
                    .unwrap_or_default()
            ),
            state,
            elapsed,
            paused,
            hidden_children: if expanded { 0 } else { self.count_desc(&n.id) },
            rollup,
            expanded,
            animate,
            detached: n.detached,
            truncated: None,
        });
        if !expanded {
            return;
        }
        let child_prefix = if depth == 0 {
            String::new()
        } else {
            format!("{prefix}{}", if last { "  " } else { "│ " })
        };
        let shown = kids.len().min(max_children);
        for (i, c) in kids.iter().take(shown).enumerate() {
            self.walk(
                c,
                depth + 1,
                child_prefix.clone(),
                i + 1 == kids.len().min(max_children) && kids.len() <= max_children,
                variant,
                collapsed,
                display_depth,
                max_children,
                pack,
                now,
                rows,
                animating,
                folded_done,
            );
        }
        if kids.len() > max_children {
            let more = kids.len() - max_children;
            rows.push(TreeRow {
                id: String::new(),
                depth: depth + 1,
                rail: format!("{child_prefix}└─"),
                name: String::new(),
                colour: 0,
                state: NodeState::Pending,
                elapsed: None,
                paused: false,
                hidden_children: more,
                rollup: None,
                expanded: false,
                animate: false,
                detached: false,
                truncated: Some(format!("+ {more} more")),
                aria: format!("{more} more agents"),
            });
        }
    }
}

/// Builds the "+ n nested agents" text for a row folded by depth.
pub fn nested_label(n: usize) -> String {
    format!("+ {n} nested {}", if n == 1 { "agent" } else { "agents" })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActivityKind, RequestRef, ToolRef};

    pub(crate) fn graph() -> RunGraph {
        let mut root = AgentNode::new(
            "o",
            None,
            "Orchestrator",
            4,
            AgentState::Thinking { header: None },
        );
        root.started = Some(ClockMs(0));
        let mut r = AgentNode::new(
            "r",
            Some("o"),
            "Researcher",
            2,
            AgentState::ToolRunning {
                tool: ToolRef {
                    id: "t".into(),
                    name: "read".into(),
                    activity: ActivityKind::Read,
                    target: Some("14 files".into()),
                },
                concurrent: 1,
            },
        );
        r.started = Some(ClockMs(1_000));
        let mut t = AgentNode::new(
            "t",
            Some("o"),
            "Test runner",
            3,
            AgentState::AwaitingApproval {
                request: RequestRef {
                    id: "q".into(),
                    tool: None,
                },
            },
        );
        t.request = Some("run `cargo test`".into());
        let mut f = AgentNode::new(
            "f",
            Some("t"),
            "Fixture gen",
            5,
            AgentState::Done {
                elapsed: Duration::from_secs(12),
                waited: Duration::ZERO,
            },
        );
        f.result = Some("3 fixtures".into());
        RunGraph {
            nodes: vec![root, r, t, f],
        }
    }

    #[test]
    fn states_and_rows() {
        let g = graph();
        let p = VerbPack::neutral();
        let v = g.view(
            TreeVariant::Rail,
            &BTreeSet::new(),
            3,
            8,
            p,
            ClockMs(60_000),
        );
        assert_eq!(v.header, "Agents · 2 running · 1 needs you");
        assert_eq!(v.rows.len(), 4);
        assert_eq!(v.rows[0].state, NodeState::WaitingOnChildren(1));
        assert_eq!(v.rows[1].state.line(), "Reading 14 files");
        assert_eq!(v.rows[1].rail, "├─");
        assert_eq!(v.rows[2].state.line(), "Needs approval: run `cargo test`");
        assert_eq!(v.rows[3].rail, "  └─");
        assert_eq!(v.rows[3].state.line(), "Returned · 3 fixtures");

        let mut c = BTreeSet::new();
        c.insert("o".to_string());
        let v = g.view(TreeVariant::Indented, &c, 3, 8, p, ClockMs(60_000));
        assert_eq!(v.rows.len(), 1);
        assert_eq!(v.rows[0].hidden_children, 3);
        assert!(matches!(v.rows[0].rollup, Some(NodeState::AwaitingYou(_))));

        let v = g.view(
            TreeVariant::Dock,
            &BTreeSet::new(),
            3,
            8,
            p,
            ClockMs(60_000),
        );
        assert_eq!(v.folded_done, 1);
    }
}
