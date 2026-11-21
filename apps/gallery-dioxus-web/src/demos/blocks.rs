//! Blocks (Oxyn Pro): larger compositions for AI and agent products. Agents: Live session,
//! Mission control, Mission timeline, Run inspector, Background agents, Agent routines, MCP
//! servers, Knowledge base, Eval suite, Agent builder and Agent gallery. AI: AI insight card.
//! Every demo runs on the host clock (`live`).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{
    AgentBuilder, AgentGallery, AgentRoutines, AiInsightCard, BackgroundAgents, EvalSuite,
    KnowledgeBase, LiveSession, McpServers, MissionControl, MissionTimeline, RunInspector,
};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Control, Demo, Vals, Variant};

/// Category of the agent blocks.
pub(crate) const CAT_AGENTS: &str = "blocks-agents";
/// Category of the AI blocks.
pub(crate) const CAT_AI: &str = "blocks-ai";

#[allow(clippy::too_many_arguments)]
fn blk(
    slug: &'static str,
    name: &'static str,
    cat: &'static str,
    dioxus: &'static str,
    summary: &'static str,
    controls: Vec<Control>,
    variants: Vec<Variant>,
    render: fn(&Vals, ClockMs) -> Element,
    code: fn(&Vals) -> String,
    keyboard: Vec<(&'static str, &'static str)>,
    a11y: Vec<&'static str>,
    responsive: Vec<&'static str>,
    height: u32,
) -> Demo {
    Demo {
        controls,
        variants,
        render,
        code,
        keyboard,
        a11y,
        responsive,
        height,
        align: Align::Center,
        live: true,
        ..mk(
            slug,
            name,
            cat,
            summary,
            dioxus,
            "BlocksView",
            slug,
            "blocks/",
        )
    }
}

fn clock(name: &str) -> Code {
    Code::new(name)
        .pre("let clock = use_motion_clock(); // the host clock (ClockMs)")
        .raw("now", "clock.now()")
}

fn pick(v: &Vals, k: &str, opts: &[&str]) -> usize {
    opts.iter().position(|o| *o == v.s(k)).unwrap_or(0)
}

const FILTERS: [&str; 4] = ["all", "running", "waiting", "failed"];
const AGENTS: [&str; 4] = ["everyone", "researcher", "writer", "designer"];

fn session(v: &Vals, now: ClockMs) -> Element {
    let focus = match pick(v, "focus", &AGENTS) {
        0 => None,
        i => Some(i),
    };
    rsx! { LiveSession { now, skip: (v.f("skip") * 1000.0) as u64, context: v.b("context"), focus, paused: v.b("paused") } }
}
fn session_code(v: &Vals) -> String {
    let mut c = clock("LiveSession");
    if v.f("skip") > 0.0 {
        c = c.raw("skip", format!("{}", (v.f("skip") * 1000.0) as u64));
    }
    if !v.b("context") {
        c = c.raw("context", "false");
    }
    c.b("paused", v.b("paused")).done()
}

fn control(v: &Vals, now: ClockMs) -> Element {
    rsx! { MissionControl { now, filter: pick(v, "filter", &FILTERS), fail_retry: v.b("fail") } }
}
fn control_code(v: &Vals) -> String {
    clock("MissionControl")
        .raw("filter", pick(v, "filter", &FILTERS).to_string())
        .b("fail_retry", v.b("fail"))
        .done()
}

fn timeline(v: &Vals, now: ClockMs) -> Element {
    rsx! { MissionTimeline { now, filter: pick(v, "filter", &FILTERS), fail_retry: v.b("fail"), ticks: v.b("ticks") } }
}
fn timeline_code(v: &Vals) -> String {
    clock("MissionTimeline")
        .raw("filter", pick(v, "filter", &FILTERS).to_string())
        .b("fail_retry", v.b("fail"))
        .done()
}

fn inspector(v: &Vals, now: ClockMs) -> Element {
    let call = v.b("open").then_some(9);
    rsx! { RunInspector { now, skip: if v.b("finished") { 24_000 } else { 0 }, speed: (v.f("speed") * 100.0) as u64, supervisor: v.b("supervisor"), call } }
}
fn inspector_code(v: &Vals) -> String {
    let mut c = clock("RunInspector");
    if v.b("finished") {
        c = c.raw("skip", "24_000");
    }
    if !v.b("supervisor") {
        c = c.raw("supervisor", "false");
    }
    c.done()
}

const BA_FILTERS: [&str; 4] = ["all", "running", "ready", "merged"];
fn bgagents(v: &Vals, now: ClockMs) -> Element {
    let open = v.b("open").then_some(2);
    rsx! { BackgroundAgents { now, filter: pick(v, "filter", &BA_FILTERS), fail_next: v.b("fail"), open } }
}
fn bgagents_code(v: &Vals) -> String {
    clock("BackgroundAgents")
        .raw("filter", pick(v, "filter", &BA_FILTERS).to_string())
        .b("fail_next", v.b("fail"))
        .done()
}

fn routines(v: &Vals, now: ClockMs) -> Element {
    let open = v.b("open").then_some(0);
    rsx! { AgentRoutines { now, fail: v.b("fail"), open, ticks: v.b("ticks") } }
}
fn routines_code(v: &Vals) -> String {
    clock("AgentRoutines").b("fail", v.b("fail")).done()
}

fn mcp(v: &Vals, now: ClockMs) -> Element {
    let open = v.b("open").then_some(0);
    rsx! { McpServers { now, open, adding: v.b("adding"), meter: v.b("meter") } }
}
fn mcp_code(v: &Vals) -> String {
    clock("McpServers").b("adding", v.b("adding")).done()
}

const ASKS: [&str; 4] = ["none", "refunds", "invoices", "seats"];
fn kb(v: &Vals, now: ClockMs) -> Element {
    let ask = match pick(v, "ask", &ASKS) {
        0 => None,
        i => Some(i - 1),
    };
    rsx! { KnowledgeBase { now, ask, settled: v.b("settled"), rerank: v.b("rerank") } }
}
fn kb_code(v: &Vals) -> String {
    let mut c = clock("KnowledgeBase");
    if let Some(i) = pick(v, "ask", &ASKS).checked_sub(1) {
        c = c.raw("ask", format!("Some({i})"));
    }
    c.b("settled", v.b("settled"))
        .b("rerank", v.b("rerank"))
        .done()
}

const EV_FILTERS: [&str; 4] = ["all", "failed", "regressions", "improved"];
fn evals(v: &Vals, now: ClockMs) -> Element {
    let open = v.b("open").then_some(3);
    rsx! { EvalSuite { now, done: v.b("done"), fail: v.b("fail"), filter: pick(v, "filter", &EV_FILTERS), open } }
}
fn evals_code(v: &Vals) -> String {
    clock("EvalSuite")
        .b("done", v.b("done"))
        .b("fail", v.b("fail"))
        .done()
}

fn builder(v: &Vals, now: ClockMs) -> Element {
    rsx! { AgentBuilder { now, testing: v.s("view") == "test", fail: v.b("fail"), unsaved: v.b("unsaved") } }
}
fn builder_code(v: &Vals) -> String {
    clock("AgentBuilder")
        .b("testing", v.s("view") == "test")
        .b("fail", v.b("fail"))
        .done()
}

const TEAMS: [&str; 5] = ["sales", "support", "engineering", "marketing", "ops"];
fn gallery(v: &Vals, now: ClockMs) -> Element {
    rsx! { AgentGallery { now, team: pick(v, "team", &TEAMS), fail: v.b("fail"), query: v.string("query") } }
}
fn gallery_code(v: &Vals) -> String {
    clock("AgentGallery")
        .raw("team", pick(v, "team", &TEAMS).to_string())
        .b("fail", v.b("fail"))
        .done()
}

fn insight(v: &Vals, now: ClockMs) -> Element {
    rsx! { AiInsightCard { now, auto_play: v.b("auto"), fail: v.b("fail"), previous: v.b("previous") } }
}
fn insight_code(v: &Vals) -> String {
    let mut c = clock("AiInsightCard");
    if !v.b("auto") {
        c = c.raw("auto_play", "false");
    }
    c.b("fail", v.b("fail")).done()
}

fn filter_ctl() -> Control {
    sel("filter", "Filter", &FILTERS, "all")
}

/// The block demos, in gallery order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        blk(
            "live-session",
            "Live session",
            CAT_AGENTS,
            "LiveSession",
            "Sit in on a crew of agents mid-run: the chat between them fills the centre, their shared plan checks itself off on the right and a bar tracks the context window used so far. Choose one agent to spotlight its lines and steps; when one asks you something, the plan holds until you reply.",
            vec![
                sel("focus", "Narrow to", &AGENTS, "everyone"),
                rng("skip", "Start at (s)", 0.0, 14.0, 1.0, "0"),
                tog("context", "Shared context", true),
                tog("paused", "Start paused", false),
            ],
            vec![
                var("live", "The session plays from the first message.", &[]),
                var(
                    "your-turn",
                    "Opens on the question that waits for you (Oxyn extra).",
                    &[("skip", "13")],
                ),
                var(
                    "writer",
                    "Narrowed to the Writer, plan only (Oxyn extra).",
                    &[("focus", "writer"), ("context", "false"), ("skip", "8")],
                ),
            ],
            session,
            session_code,
            vec![
                (
                    "Tab",
                    "Pause, the agent picker, each message, then the message field",
                ),
                ("← / → / Home / End", "Move between agents in the tabs"),
                (
                    "↑ / ↓",
                    "Step through the thread; Enter shows when and to whom a line went",
                ),
                (
                    "@",
                    "In the message field, lists agents to address; arrows and Enter choose",
                ),
                (
                    "Enter",
                    "Sends the message; Esc clears the spotlight and the open line",
                ),
            ],
            vec![
                "A region titled with the goal; steps form an ordered list and the active one carries aria-current=step.",
                "Each line's accessible name gives sender, recipient and time; arrivals are read politely.",
                "Window use is exposed as role meter, its text giving tokens used of the limit and a percentage.",
            ],
            vec![
                "Side by side from 720 px of block width; below that the plan and context go under the stream.",
                "Tabs scroll sideways inside their row; long messages wrap.",
            ],
            620,
        ),
        blk(
            "mission-control",
            "Mission control",
            CAT_AGENTS,
            "MissionControl",
            "One overview for a whole fleet: a stacked event chart of the last eight hours, lead agents with their helpers, and a side queue of asks and failures, all narrowed by a single status switch.",
            vec![filter_ctl(), tog("fail", "Retry fails again", false)],
            vec![
                var("all", "Every agent, three leads and their sub-agents.", &[]),
                var(
                    "waiting",
                    "Only the agents that wait on you (Oxyn extra).",
                    &[("filter", "waiting")],
                ),
                var(
                    "failing-retry",
                    "Failed only, and a retry that fails again (Oxyn extra).",
                    &[("filter", "failed"), ("fail", "true")],
                ),
            ],
            control,
            control_code,
            vec![
                (
                    "Tab",
                    "The filter, the leads, the sub-agents and the inbox cards",
                ),
                (
                    "← / → / Home / End",
                    "In the filter, pick the previous or next status",
                ),
                (
                    "Enter / Space",
                    "Open an inbox card with its answer buttons",
                ),
                ("Esc", "Close the open card"),
            ],
            vec![
                "The status switch is a radiogroup; every option speaks its count.",
                "The event chart is a single image described by its totals.",
                "Queue items toggle open (aria-expanded) to reveal real answer buttons; outcomes are read aloud.",
            ],
            vec![
                "Under 860 px of its own width the queue drops below; under 620 px helpers sit beneath their lead.",
            ],
            640,
        ),
        blk(
            "mission-timeline",
            "Mission timeline",
            CAT_AGENTS,
            "MissionTimeline",
            "The fleet overview laid out against time: one row per agent across the last half hour, the same event chart above and the same queue of asks beside it.",
            vec![
                filter_ctl(),
                tog("fail", "Retry fails again", false),
                tog("ticks", "Axis ticks", true),
            ],
            vec![
                var("all", "Every agent on its lane.", &[]),
                var(
                    "running",
                    "Only running agents (Oxyn extra).",
                    &[("filter", "running")],
                ),
                var(
                    "bare",
                    "No axis ticks, failed only (Oxyn extra).",
                    &[("filter", "failed"), ("ticks", "false")],
                ),
            ],
            timeline,
            timeline_code,
            vec![
                (
                    "Tab",
                    "The filter, the lane blocks that need you and the inbox cards",
                ),
                (
                    "← / → / Home / End",
                    "In the filter, pick the previous or next status",
                ),
                (
                    "Enter / Space",
                    "On a waiting or failed block, open its inbox card",
                ),
                ("Esc", "Close the open card"),
            ],
            vec![
                "Rows are list items whose names give agent, state, job and elapsed time.",
                "State is spelled out with an icon at the end of each row, never colour alone.",
                "Status switch: a radiogroup that reads its counts.",
            ],
            vec![
                "Agent names hold a fixed column while bars share one time scale.",
                "Below 620 px the status column hides (it stays in each lane's label).",
            ],
            640,
        ),
        blk(
            "run-inspector",
            "Run inspector",
            CAT_AGENTS,
            "RunInspector",
            "Debug a single run of several agents: a growing tree of who spawned whom, each card listing its model and tool calls as timing bars, totals across the top, and any call expandable in place to show its input and output or failure.",
            vec![
                tog("finished", "Open finished", false),
                rng("speed", "Speed", 0.5, 3.0, 0.5, "1"),
                tog("supervisor", "Supervisor", true),
                tog("open", "Open the failed call", false),
            ],
            vec![
                var("live", "The run plays from the start.", &[]),
                var(
                    "failed-call",
                    "Finished, with the failed test run open (Oxyn extra).",
                    &[("finished", "true"), ("open", "true")],
                ),
                var(
                    "fast",
                    "Three times speed, no supervisor (Oxyn extra).",
                    &[("speed", "3"), ("supervisor", "false")],
                ),
            ],
            inspector,
            inspector_code,
            vec![
                ("Tab", "Agent cards first, then the call rows inside each"),
                (
                    "Enter / Space",
                    "Card: focus the totals on that agent (press again to clear); call row: expand or collapse",
                ),
                ("Esc", "Close the open call"),
            ],
            vec![
                "Call rows are buttons inside a list, named with tool, time taken and any failure, linked to their details.",
                "Totals use a dl whose label names the scope (whole run or one agent).",
                "The title line with the agent count is a polite live region.",
            ],
            vec![
                "Under 640 px the six totals form a 3 × 2 grid, the watcher card moves below and timing bars give way to durations.",
            ],
            700,
        ),
        blk(
            "background-agents",
            "Background agents",
            CAT_AGENTS,
            "BackgroundAgents",
            "Hand coding jobs to agents that run off-screen: describe the change, choose repo, branch and model, then follow each job through queue, setup and its steps until a diff is waiting for you to turn into a pull request and merge.",
            vec![
                sel("filter", "Filter", &BA_FILTERS, "all"),
                tog("fail", "Next task fails", false),
                tog("open", "Open a task", false),
            ],
            vec![
                var("all", "Running, ready, merged and failed tasks.", &[]),
                var(
                    "review",
                    "Ready for review, details open (Oxyn extra).",
                    &[("filter", "ready"), ("open", "true")],
                ),
                var(
                    "failing",
                    "The next task you start fails its tests (Oxyn extra).",
                    &[("fail", "true")],
                ),
            ],
            bgagents,
            bgagents_code,
            vec![
                (
                    "Tab",
                    "The job field, the three pickers, Start, the tabs, then every job",
                ),
                ("Cmd / Ctrl + Enter", "Inside the job field: start it"),
                (
                    "Enter / Space",
                    "Picker: show the choices; job title: show steps and files",
                ),
                (
                    "← / → / Home / End",
                    "On the filter tabs, move the selection",
                ),
                ("Esc", "Closes an open chip menu"),
            ],
            vec![
                "Job titles toggle their detail panel and report whether it is open.",
                "Each state shows a glyph plus text; durations and line counts are given full spoken wording.",
                "Every action is read out, and the demo says plainly that the work is simulated.",
            ],
            vec![
                "Works from 320 to 760 px; pickers flow onto a new line and the shortcut hint drops under 520 px.",
                "Steps and files share a row from 600 px, otherwise one above the other.",
            ],
            640,
        ),
        blk(
            "agent-routines",
            "Agent routines",
            CAT_AGENTS,
            "AgentRoutines",
            "Agents on a timer, listed like plain sentences: when each fires and how long until the next time, which apps it reads, which channel gets the result and which model writes it, plus a strip of recent outcomes. Trigger one early or expand it to change how often it runs.",
            vec![
                tog("fail", "Error triage fails", false),
                tog("open", "Open the first", false),
                tog("ticks", "Run ticks", true),
            ],
            vec![
                var(
                    "list",
                    "Four timers; the morning digest fires about a minute in.",
                    &[],
                ),
                var(
                    "open",
                    "The first routine open with its report (Oxyn extra).",
                    &[("open", "true")],
                ),
                var(
                    "failing",
                    "Run Error triage now to see it fail (Oxyn extra).",
                    &[("fail", "true"), ("ticks", "false")],
                ),
            ],
            routines,
            routines_code,
            vec![
                (
                    "Tab",
                    "New routine, then name, run button and on/off for each row",
                ),
                ("↑ / ↓ / Home / End", "Jump from row to row"),
                (
                    "Enter / Space",
                    "Expand a row; on the run button, start or stop",
                ),
                ("← / →", "Inside an expanded row, cycle how often it fires"),
            ],
            vec![
                "Names toggle their row; the app logos are summarised in a single spoken line per routine.",
                "The outcome strip is one group that reads its totals; each mark carries a word too.",
                "An active run is a progressbar naming its current step; the result is read out.",
            ],
            vec![
                "Max 680 px; under 540 px the latest outcome moves out of the schedule line and under 420 px the run button shrinks to an icon.",
            ],
            560,
        ),
        blk(
            "mcp-servers",
            "MCP servers",
            CAT_AGENTS,
            "McpServers",
            "Look after the Model Context Protocol connections behind an agent: a token budget bar split by server, health per connection with sign in and retry, per-tool switches with their cost, and a catalogue to plug in new ones.",
            vec![
                tog("open", "Open GitHub", false),
                tog("adding", "Add server sheet", false),
                tog("meter", "Context bar", true),
            ],
            vec![
                var("list", "Five connections in three health states.", &[]),
                var(
                    "tools",
                    "GitHub open with its tools (Oxyn extra).",
                    &[("open", "true")],
                ),
                var(
                    "add",
                    "The Add server sheet open (Oxyn extra).",
                    &[("adding", "true")],
                ),
            ],
            mcp,
            mcp_code,
            vec![
                ("↑ / ↓ / Home / End", "Step through the connections"),
                ("Enter / Space", "Expand a connection; flip a tool"),
                (
                    "Cmd / Ctrl + Z",
                    "Bring back the connection you just removed",
                ),
                ("Esc", "Dismiss the catalogue"),
            ],
            vec![
                "Connection rows read their name, health, local or remote, enabled tools and token cost.",
                "The budget bar is role meter; its text says by how much the budget is exceeded.",
                "Health is a glyph with a word; each tool switch speaks the tool, read or write and tokens.",
            ],
            vec![
                "Max 720 px; tools lose their inset under 520 px and the access label turns into an icon under 400 px.",
            ],
            620,
        ),
        blk(
            "knowledge-base",
            "Knowledge base",
            CAT_AGENTS,
            "KnowledgeBase",
            "Build and probe a search index for an assistant: documents go through four ingest stages with live counts, then a test query returns scored passages, highlights what matched and shows what a reranker would change.",
            vec![
                sel("ask", "Ask", &ASKS, "none"),
                tog("settled", "All indexed", false),
                tog("rerank", "Rerank", false),
            ],
            vec![
                var(
                    "sources",
                    "Documents mid-ingest, one stuck at the embedding stage.",
                    &[],
                ),
                var(
                    "refunds",
                    "Test retrieval on a refund question (Oxyn extra).",
                    &[("ask", "refunds")],
                ),
                var(
                    "reranked",
                    "The same results reranked (Oxyn extra).",
                    &[("ask", "refunds"), ("rerank", "true")],
                ),
            ],
            kb,
            kb_code,
            vec![
                ("← / →", "Switch between the documents and the query tester"),
                ("Enter", "Query field: search; passage: show its neighbours"),
                ("Esc", "Empty the query"),
            ],
            vec![
                "Every ingest row is a progressbar that names its stage, e.g. Embed, step 3 of 4.",
                "Passages are buttons that speak score and movement after reranking; hits use mark.",
                "Progress, errors and query results share one polite announcer.",
            ],
            vec![
                "Single column at every width; under 560 px the stage bar sits beneath the document name without labels.",
            ],
            560,
        ),
        blk(
            "eval-suite",
            "Eval suite",
            CAT_AGENTS,
            "EvalSuite",
            "Score a release candidate of a model-backed feature against the version in production: a grid of test cases fills as the judge works, the pass rate climbs beside last release's, and anything that got worse or better links to the exact case.",
            vec![
                tog("done", "Open finished", false),
                sel("filter", "Cases", &EV_FILTERS, "all"),
                tog("fail", "Judge fails", false),
                tog("open", "Open a regression", false),
            ],
            vec![
                var("ready", "Idle, 48 cases queued.", &[]),
                var(
                    "regressions",
                    "Finished, regressions only, one open (Oxyn extra).",
                    &[
                        ("done", "true"),
                        ("filter", "regressions"),
                        ("open", "true"),
                    ],
                ),
                var(
                    "judge-fails",
                    "The judge stops halfway (Oxyn extra).",
                    &[("fail", "true")],
                ),
            ],
            evals,
            evals_code,
            vec![
                (
                    "Enter / Space",
                    "Start or halt; expand a case; accept it or file it",
                ),
                ("← / → / Home / End", "Change the case list view"),
            ],
            vec![
                "Every category grid is exposed as one image with a written tally rather than one stop per cell.",
                "Outcomes use distinct marks (✓, ×, !) and newly broken cases are filled in.",
                "Case rows toggle their detail and say so; scores and triage choices are read out.",
            ],
            vec![
                "Max 760 px; the grid moves under its label below 680 px and the two answers stack below 560 px.",
            ],
            680,
        ),
        blk(
            "agent-builder",
            "Agent builder",
            CAT_AGENTS,
            "AgentBuilder",
            "Shape a single assistant before shipping it: rename it in place, choose its model, write its brief, switch integrations on, attach reference files, try it in a sandbox chat, and push the result live.",
            vec![
                sel("view", "View", &["configure", "test"], "configure"),
                tog("fail", "Publish fails", false),
                tog("unsaved", "Unsaved changes", false),
            ],
            vec![
                var("configure", "A billing helper, live and saved.", &[]),
                var("test", "The Test view (Oxyn extra).", &[("view", "test")]),
                var(
                    "unsaved",
                    "Unsaved changes whose first publish fails (Oxyn extra).",
                    &[("unsaved", "true"), ("fail", "true")],
                ),
            ],
            builder,
            builder_code,
            vec![
                (
                    "Tab",
                    "Through the title, the mode switch, each field and switch, then Publish",
                ),
                ("← / →", "Swap between setup and sandbox"),
                (
                    "↑ / ↓",
                    "While choosing a model: step through, Esc dismisses",
                ),
                ("Enter", "Title field: confirm (Esc reverts); sandbox: send"),
            ],
            vec![
                "All inputs have names; an integration's description is folded into its switch label.",
                "Files uploading expose progress; a failed one states why beside an icon.",
                "Saves, uploads and sandbox replies are announced, noting nothing is really deployed.",
            ],
            vec![
                "Max 640 px; long labels end in an ellipsis and the sandbox chat scrolls under a fixed input.",
            ],
            700,
        ),
        blk(
            "agent-gallery",
            "Agent gallery",
            CAT_AGENTS,
            "AgentGallery",
            "Browse ready-made assistants for each department, search by job or integration, watch a short example of one at work and add it to a workspace without leaving the card while each integration connects in turn.",
            vec![
                sel("team", "Team", &TEAMS, "engineering"),
                tog("fail", "An app fails to connect", false),
            ],
            vec![
                var("engineering", "Engineering agents.", &[]),
                var(
                    "support",
                    "Support agents (Oxyn extra).",
                    &[("team", "support")],
                ),
                var(
                    "failing",
                    "Ops, where the second app fails to connect (Oxyn extra).",
                    &[("team", "ops"), ("fail", "true")],
                ),
            ],
            gallery,
            gallery_code,
            vec![
                (
                    "Tab",
                    "Search, the department switch, then each template's buttons",
                ),
                (
                    "← / → / Home / End",
                    "Change department, or choose a workspace while adding",
                ),
                ("Esc", "Empty the search, or back out of the current step"),
            ],
            vec![
                "Departments and workspaces are radiogroups with a single tab stop.",
                "Templates are articles; logos collapse into one spoken list and stars into words.",
                "Counts, connection steps and the final result are read out, noting the demo connects nothing.",
            ],
            vec![
                "Two template columns from 600 px, a single one below; the search box drops under the heading below 560 px.",
            ],
            620,
        ),
        blk(
            "ai-insight-card",
            "AI insight card",
            CAT_AI,
            "AiInsightCard",
            "A metric tile that can tell you why it moved: ask it and a short write-up appears word by word, the main causes line up with signed bars, numbered pins drop onto the trend line as each cause is mentioned, and suggested next questions swap the answer in place.",
            vec![
                tog("auto", "Explain at once", true),
                tog("fail", "First answer fails", false),
                tog("previous", "Previous period", true),
            ],
            vec![
                var("explain", "Explains on open.", &[]),
                var(
                    "idle",
                    "Waiting for Explain, no previous line (Oxyn extra).",
                    &[("auto", "false"), ("previous", "false")],
                ),
                var(
                    "error",
                    "The first answer fails, with Retry (Oxyn extra).",
                    &[("auto", "false"), ("fail", "true")],
                ),
            ],
            insight,
            insight_code,
            vec![
                (
                    "Tab",
                    "The ask button, the trend line, then the suggested questions",
                ),
                ("← / → / Home / End", "On the trend line, hear week by week"),
                ("Esc", "Hide the week readout"),
            ],
            vec![
                "The ask button reports whether the write-up is shown; start, stop and failure are read out.",
                "Changes show + or − with the colour; causes not yet reached are skipped by screen readers.",
                "Suggested questions are toggle buttons, inactive while text is still arriving.",
            ],
            vec!["Up to 600 px; below 460 px the legend hides and the bars shorten."],
            560,
        ),
    ]
}
