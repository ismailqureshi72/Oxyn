//! affectUI fidelity pass, family B part 9 (plan #32): the **Tool Access Map** model. One agent
//! in the middle of every tool the run knows about: each tool is out of reach, reachable but not
//! called yet, called (with its calls), calling right now, or failed on its last call. Several
//! agents share one layout of tools, so switching agents only changes the states. A call can be
//! replayed with the same or an edited input. Behaviour only: the views live in `oxyn-dioxus` /
//! `oxyn-gpui` (`kit/toolmapfid.rs`).

/// Where a call (or an agent) stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TmStatus {
    /// Not started.
    Queued,
    /// In flight.
    Running,
    /// Waiting on a person.
    Waiting,
    /// Finished.
    Done,
    /// Finished with an error.
    Failed,
}

impl TmStatus {
    /// `queued` / `running` / `waiting` / `done` / `failed`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Done => "done",
            Self::Failed => "failed",
        }
    }
    /// Badge words.
    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Waiting => "Needs you",
            Self::Done => "Done",
            Self::Failed => "Failed",
        }
    }
}

/// One tool call.
#[derive(Clone, Debug, PartialEq)]
pub struct TmCall {
    /// Stable id.
    pub id: String,
    /// Where it stands.
    pub status: TmStatus,
    /// The input, as text (JSON in the demo).
    pub input: String,
    /// The output or the error text; empty while running.
    pub output: String,
    /// Run time it started, seconds.
    pub at: u32,
    /// How long it took, ms (0 while running).
    pub ms: u32,
    /// Started as a replay of an earlier call.
    pub replay: bool,
}

/// One tool of the run (the layout order is the slice order; the first half sits left).
#[derive(Clone, Debug, PartialEq)]
pub struct TmTool {
    /// Stable id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Brand id for its mark.
    pub brand: String,
    /// What it points at ("acme/web"), shown in the calls view.
    pub scope: String,
}

/// One agent and the tools it may use.
#[derive(Clone, Debug, PartialEq)]
pub struct TmAgent {
    /// Stable id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Model.
    pub model: String,
    /// Brand id for the model mark.
    pub brand: String,
    /// Where the agent stands.
    pub status: TmStatus,
    /// Its task.
    pub task: String,
    /// The tools it can reach, each with its calls, oldest first. Unlisted tools are out of reach.
    pub tools: Vec<(String, Vec<TmCall>)>,
}

/// What a tool card shows for one agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TmState {
    /// The agent cannot use it.
    NoAccess,
    /// Reachable, never called.
    Unused,
    /// Called; the last call finished.
    Used,
    /// A call is in flight.
    Running,
    /// The last call failed.
    Failed,
}

impl TmState {
    /// `none` / `unused` / `used` / `running` / `failed`.
    pub fn id(self) -> &'static str {
        match self {
            Self::NoAccess => "none",
            Self::Unused => "unused",
            Self::Used => "used",
            Self::Running => "running",
            Self::Failed => "failed",
        }
    }
}

/// The calls an agent made with a tool (`None` = no access).
pub fn tm_calls<'a>(a: &'a TmAgent, tool: &str) -> Option<&'a Vec<TmCall>> {
    a.tools.iter().find(|(t, _)| t == tool).map(|(_, c)| c)
}

/// A tool's state for an agent.
pub fn tm_state(a: &TmAgent, tool: &str) -> TmState {
    match tm_calls(a, tool) {
        None => TmState::NoAccess,
        Some(c) if c.is_empty() => TmState::Unused,
        Some(c) if c.iter().any(|x| x.status == TmStatus::Running) => TmState::Running,
        Some(c) if c.last().is_some_and(|x| x.status == TmStatus::Failed) => TmState::Failed,
        Some(_) => TmState::Used,
    }
}

/// (reachable tools, all tools, calls, calls in flight) for an agent.
pub fn tm_counts(a: &TmAgent, tools: &[TmTool]) -> (usize, usize, usize, usize) {
    let reach = tools
        .iter()
        .filter(|t| tm_calls(a, &t.id).is_some())
        .count();
    let calls: Vec<&TmCall> = a.tools.iter().flat_map(|(_, c)| c.iter()).collect();
    let running = calls
        .iter()
        .filter(|c| c.status == TmStatus::Running)
        .count();
    (reach, tools.len(), calls.len(), running)
}

/// "520ms" under a second, "5.2s" under a minute, "1m 04s" after.
pub fn tm_dur(ms: u32) -> String {
    if ms < 1000 {
        format!("{ms}ms")
    } else if ms < 60_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}m {:02}s", ms / 60_000, (ms / 1000) % 60)
    }
}

/// m:ss for a run time.
pub fn tm_clock(s: u32) -> String {
    format!("{}:{:02}", s / 60, s % 60)
}

/// The first line of an input, squeezed to one line for the call list.
pub fn tm_summary(input: &str, max: usize) -> String {
    let one: String = input
        .chars()
        .filter(|c| *c != '{' && *c != '}')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if one.chars().count() <= max {
        one
    } else {
        format!(
            "{}…",
            one.chars().take(max.saturating_sub(1)).collect::<String>()
        )
    }
}

/// Left and right halves of the layout: the first `ceil(n / 2)` tools sit left.
pub fn tm_sides(n: usize) -> (std::ops::Range<usize>, std::ops::Range<usize>) {
    let l = n.div_ceil(2);
    (0..l, l..n)
}

/// Replay a call: a new running call with `input` goes to the end of that tool's list. Returns
/// the new call's id, or `None` when the agent cannot reach the tool.
pub fn tm_replay(a: &mut TmAgent, tool: &str, input: &str, at: u32) -> Option<String> {
    let calls = a
        .tools
        .iter_mut()
        .find(|(t, _)| t == tool)
        .map(|(_, c)| c)?;
    let id = format!("{tool}-r{}", calls.len() + 1);
    calls.push(TmCall {
        id: id.clone(),
        status: TmStatus::Running,
        input: input.to_string(),
        output: String::new(),
        at,
        ms: 0,
        replay: true,
    });
    Some(id)
}

/// Settle a running call.
pub fn tm_settle(a: &mut TmAgent, call: &str, ok: bool, output: &str, ms: u32) {
    for (_, cs) in a.tools.iter_mut() {
        for c in cs.iter_mut().filter(|c| c.id == call) {
            c.status = if ok { TmStatus::Done } else { TmStatus::Failed };
            c.output = output.to_string();
            c.ms = ms;
        }
    }
}

fn tool(id: &str, name: &str, brand: &str, scope: &str) -> TmTool {
    TmTool {
        id: id.into(),
        name: name.into(),
        brand: brand.into(),
        scope: scope.into(),
    }
}

fn call(id: &str, st: TmStatus, input: &str, output: &str, at: u32, ms: u32) -> TmCall {
    TmCall {
        id: id.into(),
        status: st,
        input: input.into(),
        output: output.into(),
        at,
        ms,
        replay: false,
    }
}

/// The demo's tools.
pub fn tm_demo_tools() -> Vec<TmTool> {
    vec![
        tool("github", "GitHub", "github", "shop/storefront"),
        tool("shell", "Shell", "", "ci runner"),
        tool("stripe", "Stripe", "stripe", "test mode"),
        tool("figma", "Figma", "figma", "Pricing v3"),
        tool("db", "Postgres", "postgresql", "orders (read)"),
        tool("search", "Web search", "", "public web"),
        tool("notion", "Notion", "notion", "Launch notes"),
        tool("slack", "Slack", "slack", "#launch"),
        tool("linear", "Linear", "linear", "WEB team"),
    ]
}

/// The demo's agents (the builder is first).
pub fn tm_demo_agents() -> Vec<TmAgent> {
    use TmStatus::*;
    vec![
        TmAgent {
            id: "builder".into(),
            name: "Builder".into(),
            model: "GPT-5".into(),
            brand: "openai".into(),
            status: Running,
            task: "Ship the new pricing page and hook up the payment step".into(),
            tools: vec![
                (
                    "github".into(),
                    vec![
                        call(
                            "gh1",
                            Done,
                            "{ \"branch\": \"pricing-v3\", \"from\": \"main\" }",
                            "{ \"ref\": \"pricing-v3\" }",
                            12,
                            480,
                        ),
                        call(
                            "gh2",
                            Done,
                            "{ \"commit\": \"Add plan cards\", \"files\": 4 }",
                            "{ \"sha\": \"9c1e7a2\" }",
                            131,
                            610,
                        ),
                        call(
                            "gh3",
                            Done,
                            "{ \"commit\": \"Wire the pay button\", \"files\": 2 }",
                            "{ \"sha\": \"4b07d1f\" }",
                            204,
                            4300,
                        ),
                    ],
                ),
                (
                    "shell".into(),
                    vec![
                        call(
                            "sh1",
                            Done,
                            "{ \"run\": \"npm test\" }",
                            "{ \"passed\": 118, \"failed\": 0 }",
                            150,
                            9800,
                        ),
                        call("sh2", Running, "{ \"run\": \"npm run build\" }", "", 236, 0),
                    ],
                ),
                (
                    "stripe".into(),
                    vec![call(
                        "st1",
                        Done,
                        "{ \"price\": \"pro_monthly\", \"amount\": 1900 }",
                        "{ \"id\": \"price_test_31\" }",
                        170,
                        720,
                    )],
                ),
                (
                    "figma".into(),
                    vec![call(
                        "fg1",
                        Done,
                        "{ \"frame\": \"Pricing / Desktop\" }",
                        "{ \"nodes\": 42 }",
                        40,
                        1300,
                    )],
                ),
                ("db".into(), vec![]),
                ("search".into(), vec![]),
            ],
        },
        TmAgent {
            id: "scout".into(),
            name: "Scout".into(),
            model: "Gemini 2.5 Pro".into(),
            brand: "gemini".into(),
            status: Running,
            task: "Find what people say about our prices in 80 support tickets".into(),
            tools: vec![
                (
                    "db".into(),
                    vec![call(
                        "db1",
                        Running,
                        "{ \"query\": \"tickets where tag = 'price'\" }",
                        "",
                        64,
                        0,
                    )],
                ),
                ("notion".into(), vec![]),
                (
                    "search".into(),
                    vec![call(
                        "ws1",
                        Failed,
                        "{ \"q\": \"pricing page conversion benchmarks\" }",
                        "Rate limited – try again in 30 s",
                        20,
                        2200,
                    )],
                ),
            ],
        },
        TmAgent {
            id: "scribe".into(),
            name: "Scribe".into(),
            model: "Claude Sonnet 4".into(),
            brand: "claude".into(),
            status: Waiting,
            task: "Write the launch note and post it once someone signs it off".into(),
            tools: vec![
                (
                    "notion".into(),
                    vec![call(
                        "no1",
                        Done,
                        "{ \"page\": \"Launch notes\", \"append\": \"draft\" }",
                        "{ \"blocks\": 14 }",
                        30,
                        900,
                    )],
                ),
                ("slack".into(), vec![]),
                ("linear".into(), vec![]),
            ],
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn states_and_counts() {
        let tools = tm_demo_tools();
        let a = tm_demo_agents();
        assert_eq!(tm_state(&a[0], "github"), TmState::Used);
        assert_eq!(tm_state(&a[0], "shell"), TmState::Running);
        assert_eq!(tm_state(&a[0], "db"), TmState::Unused);
        assert_eq!(tm_state(&a[0], "slack"), TmState::NoAccess);
        assert_eq!(tm_state(&a[1], "search"), TmState::Failed);
        assert_eq!(tm_counts(&a[0], &tools), (6, 9, 7, 1));
        assert_eq!(tm_counts(&a[1], &tools), (3, 9, 2, 1));
        assert_eq!(tm_sides(9), (0..5, 5..9));
    }

    #[test]
    fn replay_and_format() {
        let mut a = tm_demo_agents().remove(0);
        let id = tm_replay(&mut a, "github", "{ \"pr\": \"Pricing\" }", 250).unwrap();
        assert_eq!(tm_state(&a, "github"), TmState::Running);
        tm_settle(&mut a, &id, true, "{ \"number\": 17 }", 6400);
        assert_eq!(tm_state(&a, "github"), TmState::Used);
        assert_eq!(tm_calls(&a, "github").unwrap().len(), 4);
        assert!(tm_replay(&mut a, "slack", "{}", 1).is_none());
        assert_eq!(tm_dur(520), "520ms");
        assert_eq!(tm_dur(5200), "5.2s");
        assert_eq!(tm_clock(204), "3:24");
        assert_eq!(
            tm_summary("{ \"run\":  \"npm test\" }", 40),
            "\"run\": \"npm test\""
        );
        assert_eq!(tm_summary("abcdefghij", 5), "abcd…");
    }
}
