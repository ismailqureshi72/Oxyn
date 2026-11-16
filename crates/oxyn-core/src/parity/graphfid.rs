//! affectUI fidelity pass (plan #32, overnight batch, family B part 3): the model behind the exact
//! **Agent Graph** – agents and the tools they call as a living network: the lead in the middle,
//! agents on an inner orbit, tools and you on the outer one, a pulse along the link for every
//! call and its answer, agents that breathe while they work and turn amber when they need you.
//! Behaviour only; time is passed in as elapsed milliseconds.

/// What a node is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    /// An agent (inner orbit; the lead in the middle).
    Agent,
    /// A tool or data source (outer orbit).
    Tool,
    /// You (outer orbit, a dark tile).
    User,
}

/// How a node is doing.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeStatus {
    /// Resting.
    Idle,
    /// Working: breathes.
    Working,
    /// Needs you: amber.
    Waiting,
    /// Failed: red.
    Error,
    /// Finished: a green check.
    Done,
}

impl NodeStatus {
    /// "Idle" · "Working" · "Needs you" · "Failed" · "Done".
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Working => "Working",
            Self::Waiting => "Needs you",
            Self::Error => "Failed",
            Self::Done => "Done",
        }
    }
    /// `idle` · `working` · `waiting` · `error` · `done`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Working => "working",
            Self::Waiting => "waiting",
            Self::Error => "error",
            Self::Done => "done",
        }
    }
}

/// One agent, tool or user.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphNode {
    /// Id (keeps the node in place).
    pub id: String,
    /// Name under the tile.
    pub name: String,
    /// A shorter name for narrow graphs.
    pub short: String,
    /// Kind.
    pub kind: NodeKind,
    /// Brand mark in the tile (empty: initials; a person for the user).
    pub brand: String,
    /// The model, or what a tool is.
    pub detail: String,
    /// Status.
    pub status: NodeStatus,
    /// What it does right now (in its card).
    pub doing: String,
    /// What it has cost so far (dollars).
    pub cost: f64,
    /// The agent in the middle.
    pub lead: bool,
}

impl GraphNode {
    fn new(id: &str, name: &str, kind: NodeKind, brand: &str, detail: &str) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            short: String::new(),
            kind,
            brand: brand.into(),
            detail: detail.into(),
            status: NodeStatus::Idle,
            doing: String::new(),
            cost: 0.0,
            lead: false,
        }
    }
    /// The label for a narrow graph.
    pub fn short_name(&self) -> &str {
        if self.short.is_empty() {
            &self.name
        } else {
            &self.short
        }
    }
}

/// A call's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CallStatus {
    /// Waiting for the answer.
    Pending,
    /// Answered (a green pulse travels back).
    Ok,
    /// Failed (red, the link flashes).
    Error,
}

/// One call from a node to another.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphCall {
    /// Id (one pulse per id).
    pub id: String,
    /// Caller.
    pub from: String,
    /// Callee.
    pub to: String,
    /// When it was made (ms on the graph's clock).
    pub at: u64,
    /// State.
    pub status: CallStatus,
    /// What it asked.
    pub label: String,
    /// How long the answer took.
    pub ms: u64,
}

/// "$0.42".
pub fn graph_cost(v: f64) -> String {
    format!("${:.2}", v.max(0.0))
}

/// "6.0s" / "840ms".
pub fn call_duration(ms: u64) -> String {
    if ms >= 1000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{ms}ms")
    }
}

/// Calls per minute from the calls of the last `window` ms.
pub fn calls_per_min(calls: &[GraphCall], now: u64, window: u64) -> u32 {
    let w = window.max(1);
    let n = calls
        .iter()
        .filter(|c| c.at <= now && now - c.at < w)
        .count() as f64;
    (n * 60_000.0 / w as f64).round() as u32
}

/// The links: every edge plus every pair that has called, each once (in first-seen order).
pub fn graph_links(edges: &[(String, String)], calls: &[GraphCall]) -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = vec![];
    let mut push = |a: &str, b: &str| {
        if a == b {
            return;
        }
        if !out
            .iter()
            .any(|(x, y)| (x == a && y == b) || (x == b && y == a))
        {
            out.push((a.into(), b.into()));
        }
    };
    for (a, b) in edges {
        push(a, b);
    }
    for c in calls {
        push(&c.from, &c.to);
    }
    out
}

/// Where every node sits: (id, x, y) in a `w` × `h` box, the lead in the middle, the other
/// agents on the corners of an inner box, tools and you in line with the agent they talk to
/// most (beside it or above / below it), so every link needs at most one turn.
pub fn graph_layout(
    nodes: &[GraphNode],
    links: &[(String, String)],
    w: f32,
    h: f32,
) -> Vec<(String, f32, f32)> {
    let narrow = w < 520.0;
    let (cx, cy) = (w / 2.0, h / 2.0);
    let (ax, ay) = if narrow {
        (w * 0.24, h * 0.17)
    } else {
        (w * 0.127, h * 0.156)
    };
    let (ox, oy) = if narrow {
        (w * 0.40, h * 0.40)
    } else {
        (w * 0.37, h * 0.37)
    };
    let lead = nodes
        .iter()
        .position(|n| n.lead)
        .or_else(|| nodes.iter().position(|n| n.kind == NodeKind::Agent));
    let mut out: Vec<(String, f32, f32)> = vec![];
    let mut corners: Vec<(String, f32, f32)> = vec![];
    // Agents: up-left, up-right, down-left, down-right, then around.
    const SIGNS: [(f32, f32); 8] = [
        (-1.0, -1.0),
        (1.0, -1.0),
        (-1.0, 1.0),
        (1.0, 1.0),
        (0.0, -1.6),
        (0.0, 1.6),
        (-1.9, 0.0),
        (1.9, 0.0),
    ];
    let mut k = 0;
    for (i, n) in nodes.iter().enumerate() {
        if Some(i) == lead {
            out.push((n.id.clone(), cx, cy));
        } else if n.kind == NodeKind::Agent {
            let (sx, sy) = SIGNS[k % SIGNS.len()];
            k += 1;
            let p = (n.id.clone(), cx + sx * ax, cy + sy * ay);
            corners.push(p.clone());
            out.push(p);
        }
    }
    // Outer slots: for each agent, beside it (outwards) and above / below it.
    let mut slots: Vec<(String, f32, f32, bool)> = vec![];
    for (id, x, y) in corners.iter() {
        let sx = if *x < cx { -1.0 } else { 1.0 };
        let sy = if *y < cy { -1.0 } else { 1.0 };
        slots.push((id.clone(), cx + sx * ox, *y, false));
        slots.push((id.clone(), *x, cy + sy * oy, false));
    }
    let lead_id = lead.map(|i| nodes[i].id.clone()).unwrap_or_default();
    for n in nodes.iter().filter(|n| n.kind != NodeKind::Agent) {
        let partners: Vec<&String> = links
            .iter()
            .filter_map(|(a, b)| {
                if a == &n.id {
                    Some(b)
                } else if b == &n.id {
                    Some(a)
                } else {
                    None
                }
            })
            .collect();
        // First a free slot of a partner agent, then the free slot nearest to one.
        let want: Vec<(f32, f32)> = partners
            .iter()
            .filter(|p| **p != &lead_id)
            .filter_map(|p| corners.iter().find(|c| &c.0 == *p).map(|c| (c.1, c.2)))
            .collect();
        let pick = slots
            .iter()
            .position(|s| !s.3 && partners.contains(&&s.0))
            .or_else(|| {
                let (tx, ty) = want.first().copied().unwrap_or((cx, cy - oy));
                slots
                    .iter()
                    .enumerate()
                    .filter(|(_, s)| !s.3)
                    .min_by(|a, b| {
                        let da = (a.1.1 - tx).powi(2) + (a.1.2 - ty).powi(2);
                        let db = (b.1.1 - tx).powi(2) + (b.1.2 - ty).powi(2);
                        da.total_cmp(&db)
                    })
                    .map(|x| x.0)
            });
        match pick {
            Some(i) => {
                slots[i].3 = true;
                out.push((n.id.clone(), slots[i].1, slots[i].2));
            }
            None => out.push((n.id.clone(), cx, cy - oy)),
        }
    }
    out
}

/// A link's path: straight when the two nodes line up, otherwise one rounded right-angled turn
/// (`r` px). `lane` shifts it sideways (links sharing a side run 8 px apart). Starts at `a`.
pub fn link_path(a: (f32, f32), b: (f32, f32), r: f32, lane: f32, vertical_first: bool) -> String {
    let (x0, y0) = a;
    let (x1, y1) = b;
    if (x0 - x1).abs() < 1.0 || (y0 - y1).abs() < 1.0 {
        let (dx, dy) = if (x0 - x1).abs() < 1.0 {
            (lane, 0.0)
        } else {
            (0.0, lane)
        };
        return format!(
            "M{:.1} {:.1}L{:.1} {:.1}",
            x0 + dx,
            y0 + dy,
            x1 + dx,
            y1 + dy
        );
    }
    let sx = (x1 - x0).signum();
    let sy = (y1 - y0).signum();
    let r = r
        .min((x1 - x0).abs() / 2.0)
        .min((y1 - y0).abs() / 2.0)
        .max(0.0);
    if vertical_first {
        let x = x0 + lane;
        format!(
            "M{x:.1} {y0:.1}L{x:.1} {:.1}Q{x:.1} {y1:.1} {:.1} {y1:.1}L{x1:.1} {y1:.1}",
            y1 - sy * r,
            x + sx * r
        )
    } else {
        let y = y0 + lane;
        format!(
            "M{x0:.1} {y:.1}L{:.1} {y:.1}Q{x1:.1} {y:.1} {x1:.1} {:.1}L{x1:.1} {y1:.1}",
            x1 - sx * r,
            y + sy * r
        )
    }
}

/// Which way a link turns: `true` = vertical first. Picks the corner that sits clear of every
/// other node (`others`, centres), so a link never runs through a node.
pub fn link_turn(a: (f32, f32), b: (f32, f32), others: &[(f32, f32)]) -> bool {
    let clear = |p: (f32, f32)| {
        others
            .iter()
            .filter(|o| (o.0 - a.0).abs() + (o.1 - a.1).abs() > 1.0)
            .filter(|o| (o.0 - b.0).abs() + (o.1 - b.1).abs() > 1.0)
            .map(|o| (o.0 - p.0).abs().max((o.1 - p.1).abs()))
            .fold(f32::MAX, f32::min)
    };
    clear((a.0, b.1)) >= clear((b.0, a.1))
}

/// The whole graph at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct GraphScene {
    /// Nodes.
    pub nodes: Vec<GraphNode>,
    /// Links that are always there.
    pub edges: Vec<(String, String)>,
    /// Recent calls, newest last.
    pub calls: Vec<GraphCall>,
}

/// How long one pass of the demo lasts before it starts over (ms).
pub const GRAPH_LOOP_MS: u64 = 24_000;

/// Every node the demo ever shows (its layout keeps a place for the ones that come and go).
pub fn graph_demo_nodes() -> Vec<GraphNode> {
    base_nodes()
}

fn base_nodes() -> Vec<GraphNode> {
    use NodeKind::{Agent, Tool, User};
    let mut v = vec![
        GraphNode::new("lead", "Orchestrator", Agent, "claude", "Opus 4.1"),
        GraphNode::new("writer", "Writer", Agent, "claude", "Sonnet 4.5"),
        GraphNode::new("research", "Researcher", Agent, "claude", "Sonnet 4.5"),
        GraphNode::new("coder", "Coder", Agent, "openai", "GPT-5"),
        GraphNode::new("analyst", "Analyst", Agent, "gemini", "Gemini 2.5 Pro"),
        GraphNode::new("slack", "Slack", Tool, "slack", "#release-room"),
        GraphNode::new("you", "You", User, "", "Release owner"),
        GraphNode::new("web", "Web search", Tool, "", "Live results"),
        GraphNode::new("notion", "Notion", Tool, "notion", "Release notes"),
        GraphNode::new("stripe", "Stripe", Tool, "stripe", "Sandbox payments"),
        GraphNode::new("github", "GitHub", Tool, "github", "team/app"),
        GraphNode::new("hubspot", "HubSpot", Tool, "hubspot", "Contacts"),
        GraphNode::new("db", "Database", Tool, "postgresql", "Postgres, read-only"),
    ];
    v[0].lead = true;
    for n in v.iter_mut() {
        match n.id.as_str() {
            "web" => n.short = "Search".into(),
            "db" => n.short = "DB".into(),
            "lead" => n.short = "Lead".into(),
            _ => {}
        }
    }
    v
}

/// Repeating calls: (from, to, every ms, first at, answer ms, label, from, until).
type Beat = (
    &'static str,
    &'static str,
    u64,
    u64,
    u64,
    &'static str,
    u64,
    u64,
);

const BEATS: [Beat; 13] = [
    (
        "lead",
        "research",
        6_000,
        300,
        1_400,
        "Gather what shipped",
        0,
        9_000,
    ),
    (
        "lead",
        "coder",
        6_500,
        700,
        1_600,
        "Fix the release script",
        0,
        16_000,
    ),
    (
        "lead",
        "analyst",
        7_000,
        1_100,
        1_500,
        "Size the audience",
        0,
        11_000,
    ),
    (
        "lead",
        "writer",
        7_500,
        1_500,
        1_800,
        "Draft the announcement",
        0,
        20_000,
    ),
    (
        "research",
        "web",
        2_400,
        1_900,
        900,
        "Search recent changes",
        0,
        9_000,
    ),
    (
        "research",
        "notion",
        3_100,
        2_600,
        700,
        "Read the release notes",
        0,
        9_000,
    ),
    (
        "analyst",
        "db",
        2_200,
        2_100,
        600,
        "Count active teams",
        0,
        11_000,
    ),
    (
        "analyst",
        "hubspot",
        3_300,
        3_200,
        800,
        "Pull the beta list",
        0,
        11_000,
    ),
    (
        "coder",
        "github",
        2_600,
        2_400,
        1_100,
        "Open the release branch",
        0,
        13_000,
    ),
    (
        "coder",
        "stripe",
        2_900,
        6_400,
        900,
        "Check the sandbox plan",
        6_000,
        13_000,
    ),
    (
        "writer",
        "notion",
        3_400,
        3_000,
        800,
        "Save the draft",
        0,
        14_000,
    ),
    (
        "writer",
        "slack",
        4_200,
        4_000,
        700,
        "Post a preview",
        0,
        14_000,
    ),
    (
        "lead",
        "slack",
        9_000,
        5_200,
        600,
        "Update the channel",
        0,
        24_000,
    ),
];

/// The demo graph `t` ms into a pass.
pub fn graph_at(t: u64) -> GraphScene {
    let mut nodes = base_nodes();
    let set = |nodes: &mut Vec<GraphNode>, id: &str, st: NodeStatus, doing: &str| {
        if let Some(n) = nodes.iter_mut().find(|n| n.id == id) {
            n.status = st;
            n.doing = doing.into();
        }
    };
    use NodeStatus::*;
    set(
        &mut nodes,
        "lead",
        Working,
        if t < 4_000 {
            "Dividing the release into four jobs"
        } else {
            "Waiting on four agents"
        },
    );
    set(
        &mut nodes,
        "research",
        if t < 9_000 { Working } else { Done },
        if t < 9_000 {
            "Collecting what shipped"
        } else {
            "Handed over 9 changes"
        },
    );
    set(
        &mut nodes,
        "analyst",
        if t < 11_000 { Working } else { Done },
        if t < 11_000 {
            "Sizing the audience"
        } else {
            "1,180 teams to reach"
        },
    );
    let (cs, cd) = if t < 13_000 {
        (Working, "Patching the release script")
    } else if t < 16_000 {
        (Error, "The sandbox check timed out")
    } else {
        (Working, "Retrying the sandbox check")
    };
    set(&mut nodes, "coder", cs, cd);
    let (ws, wd) = if t < 14_000 {
        (Working, "Drafting the announcement")
    } else {
        (Waiting, "Can I post the draft to #release-room?")
    };
    set(&mut nodes, "writer", ws, wd);
    set(&mut nodes, "you", Idle, "");
    for id in [
        "notion", "slack", "web", "stripe", "hubspot", "github", "db",
    ] {
        set(&mut nodes, id, Idle, "");
    }
    // Stripe slides in while the coder needs it.
    if !(6_000..15_000).contains(&t) {
        nodes.retain(|n| n.id != "stripe");
    }
    let mut calls: Vec<GraphCall> = vec![];
    for (b, (from, to, every, first, ms, label, on, off)) in BEATS.iter().enumerate() {
        let mut at = *first.max(on);
        let mut k = 0;
        while at <= t && at < *off {
            let fail = *to == "stripe" && at >= 12_000;
            let status = if t < at + ms {
                CallStatus::Pending
            } else if fail {
                CallStatus::Error
            } else {
                CallStatus::Ok
            };
            if t - at < 60_000 {
                calls.push(GraphCall {
                    id: format!("c{b}-{k}"),
                    from: (*from).into(),
                    to: (*to).into(),
                    at,
                    status,
                    label: (*label).into(),
                    ms: *ms,
                });
            }
            at += every;
            k += 1;
        }
    }
    // The writer asks you once it waits.
    if t >= 14_000 {
        calls.push(GraphCall {
            id: "ask-you".into(),
            from: "writer".into(),
            to: "you".into(),
            at: 14_000,
            status: CallStatus::Pending,
            label: "Post the draft?".into(),
            ms: 0,
        });
    }
    calls.sort_by_key(|c| c.at);
    if !nodes.iter().any(|n| n.id == "stripe") {
        calls.retain(|c| c.to != "stripe");
    }
    // Costs from the calls made.
    for n in nodes.iter_mut() {
        let made = calls.iter().filter(|c| c.from == n.id).count() as f64;
        let got = calls.iter().filter(|c| c.to == n.id).count() as f64;
        n.cost = match n.kind {
            NodeKind::Agent => made * 0.011 + got * 0.006 + if n.lead { 0.02 } else { 0.0 },
            _ => 0.0,
        };
    }
    GraphScene {
        nodes,
        edges: vec![
            ("you".into(), "lead".into()),
            ("lead".into(), "research".into()),
            ("lead".into(), "coder".into()),
            ("lead".into(), "analyst".into()),
            ("lead".into(), "writer".into()),
        ],
        calls,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layout_puts_tools_in_line_with_their_agent() {
        let s = graph_at(8_000);
        let links = graph_links(&s.edges, &s.calls);
        let pos = graph_layout(&s.nodes, &links, 756.0, 454.0);
        let at = |id: &str| pos.iter().find(|p| p.0 == id).map(|p| (p.1, p.2)).unwrap();
        assert_eq!(at("lead"), (378.0, 227.0));
        let (wx, wy) = at("research");
        let (tx, ty) = at("web");
        assert!((wx - tx).abs() < 0.5 || (wy - ty).abs() < 0.5);
        assert_eq!(pos.len(), s.nodes.len());
    }

    #[test]
    fn paths_and_counts() {
        assert_eq!(
            link_path((0.0, 0.0), (0.0, 50.0), 8.0, 0.0, true),
            "M0.0 0.0L0.0 50.0"
        );
        assert!(link_path((0.0, 0.0), (40.0, 50.0), 8.0, 0.0, true).contains('Q'));
        assert!(link_turn((0.0, 0.0), (100.0, 100.0), &[(100.0, 0.0)]));
        assert_eq!(call_duration(6_000), "6.0s");
        assert_eq!(call_duration(840), "840ms");
        let s = graph_at(15_000);
        let w = s.nodes.iter().find(|n| n.id == "writer").unwrap();
        assert_eq!(w.status, NodeStatus::Waiting);
        assert!(calls_per_min(&s.calls, 15_000, 10_000) > 20);
        assert_eq!(graph_cost(0.4199), "$0.42");
    }
}
