//! affectUI fidelity pass (plan #32, overnight batch, family B part 3): the model behind the exact
//! **Agent Handoff** – a task passed from agent to agent like a relay: agent cards in relay
//! order, and between each pair a column of what was handed over – what went along and what got
//! lost on the way. Behaviour only; time is passed in as elapsed milliseconds.

/// An agent in the relay.
#[derive(Clone, Debug, PartialEq)]
pub struct HandoffAgent {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Model under the name.
    pub model: String,
    /// Brand mark (`claude`, `openai`, `gemini`, …).
    pub brand: String,
    /// What it is for (two lines).
    pub description: String,
    /// What it has cost so far (dollars).
    pub cost: f64,
}

/// What kind of thing went along.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ItemKind {
    /// A file.
    File,
    /// Sources.
    Source,
    /// A fact.
    Fact,
    /// A decision.
    Decision,
    /// A note.
    Note,
}

/// One thing handed over.
#[derive(Clone, Debug, PartialEq)]
pub struct HandoffItem {
    /// Id.
    pub id: String,
    /// Label.
    pub label: String,
    /// Kind (its icon).
    pub kind: ItemKind,
    /// It did not make it (struck through).
    pub lost: bool,
    /// Why it got lost.
    pub why: String,
}

/// A handoff: a card in the column between `from` and `to`.
#[derive(Clone, Debug, PartialEq)]
pub struct Handoff {
    /// Id (a new one sends light along its lines).
    pub id: String,
    /// From agent.
    pub from: String,
    /// To agent.
    pub to: String,
    /// The task's title.
    pub task: String,
    /// What went along.
    pub items: Vec<HandoffItem>,
    /// When it landed (ms; 0 = there from the start).
    pub at: u64,
}

/// Where a task is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TaskStatus {
    /// Waiting.
    Todo,
    /// With an agent.
    Working,
    /// Finished and fine.
    Done,
    /// Finished with something to check.
    Warning,
}

/// A task.
#[derive(Clone, Debug, PartialEq)]
pub struct HandoffTask {
    /// Id.
    pub id: String,
    /// Title (handoffs refer to it by title).
    pub title: String,
    /// Brand mark of where it comes from.
    pub logo: String,
    /// Status.
    pub status: TaskStatus,
    /// The agent it is with.
    pub agent: String,
    /// What the agent does with it ("Reading sources"), or the warning ("2 claims unchecked").
    pub note: String,
}

/// The status line of a handoff card: (`kind`, text) – `spin` "With Writer" / "Handing on",
/// `ok` "Looks good", `warn` the note.
pub fn handoff_status(
    h: &Handoff,
    tasks: &[HandoffTask],
    agents: &[HandoffAgent],
    now: u64,
) -> (&'static str, String) {
    let Some(t) = tasks.iter().find(|t| t.title == h.task) else {
        return ("ok", "Looks good".into());
    };
    match t.status {
        TaskStatus::Working | TaskStatus::Todo => {
            if h.at > 0 && now.saturating_sub(h.at) < 1_200 && t.agent == h.to {
                ("spin", "Handing on".into())
            } else {
                let who = agents
                    .iter()
                    .find(|a| a.id == t.agent)
                    .map(|a| a.name.clone())
                    .unwrap_or_else(|| t.agent.clone());
                ("spin", format!("With {who}"))
            }
        }
        TaskStatus::Done => ("ok", "Looks good".into()),
        TaskStatus::Warning => (
            "warn",
            if t.note.is_empty() {
                "Check before it ships".into()
            } else {
                t.note.clone()
            },
        ),
    }
}

/// The one-line summary of what went along: the labels, comma separated.
pub fn items_line(items: &[HandoffItem]) -> String {
    items
        .iter()
        .filter(|i| !i.lost)
        .map(|i| i.label.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

/// "$1.42".
pub fn handoff_cost(v: f64) -> String {
    format!("${:.2}", v.max(0.0))
}

/// The whole relay at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct HandoffScene {
    /// Agents in relay order.
    pub agents: Vec<HandoffAgent>,
    /// Handoffs, oldest first.
    pub handoffs: Vec<Handoff>,
    /// Tasks.
    pub tasks: Vec<HandoffTask>,
}

impl HandoffScene {
    /// The handoffs between `from` and `to`, latest on top.
    pub fn column(&self, from: &str, to: &str) -> Vec<Handoff> {
        let mut v: Vec<Handoff> = self
            .handoffs
            .iter()
            .filter(|h| h.from == from && h.to == to)
            .cloned()
            .collect();
        v.reverse();
        v
    }
    /// What an agent shows: its working task, or (first agent) the next todo one with how many
    /// more wait.
    pub fn agent_task(&self, agent: usize) -> Option<(HandoffTask, bool, usize)> {
        let id = &self.agents.get(agent)?.id;
        if let Some(t) = self
            .tasks
            .iter()
            .find(|t| &t.agent == id && t.status == TaskStatus::Working)
        {
            return Some((t.clone(), false, 0));
        }
        if agent == 0 {
            let todo: Vec<&HandoffTask> = self
                .tasks
                .iter()
                .filter(|t| t.status == TaskStatus::Todo)
                .collect();
            if let Some(first) = todo.first() {
                return Some(((*first).clone(), true, todo.len() - 1));
            }
        }
        None
    }
    /// Gives a task to an agent (a card dropped on an agent to the right of its column).
    pub fn assign(&mut self, task: &str, agent: &str, doing: &str) {
        if let Some(t) = self.tasks.iter_mut().find(|t| t.title == task) {
            t.status = TaskStatus::Working;
            t.agent = agent.into();
            t.note = doing.into();
        }
    }
    /// Whether `agent` is to the right of the column that ends at `to` (cards only move on down
    /// the relay, never back).
    pub fn can_take(&self, to: &str, agent: &str) -> bool {
        let i = self.agents.iter().position(|a| a.id == to);
        let j = self.agents.iter().position(|a| a.id == agent);
        matches!((i, j), (Some(i), Some(j)) if j >= i)
    }
}

fn item(id: &str, label: &str, kind: ItemKind) -> HandoffItem {
    HandoffItem {
        id: id.into(),
        label: label.into(),
        kind,
        lost: false,
        why: String::new(),
    }
}

fn lost(id: &str, label: &str, why: &str) -> HandoffItem {
    HandoffItem {
        id: id.into(),
        label: label.into(),
        kind: ItemKind::Note,
        lost: true,
        why: why.into(),
    }
}

fn task(
    id: &str,
    title: &str,
    logo: &str,
    status: TaskStatus,
    agent: &str,
    note: &str,
) -> HandoffTask {
    HandoffTask {
        id: id.into(),
        title: title.into(),
        logo: logo.into(),
        status,
        agent: agent.into(),
        note: note.into(),
    }
}

/// How long one pass of the demo lasts before it starts over (ms).
pub const HANDOFF_LOOP_MS: u64 = 22_000;

/// The demo relay `t` ms into a pass.
pub fn handoff_at(t: u64) -> HandoffScene {
    use TaskStatus::*;
    let agents = vec![
        HandoffAgent {
            id: "research".into(),
            name: "Researcher".into(),
            model: "Opus 4.1".into(),
            brand: "claude".into(),
            description: "Gathers the sources, checks the facts and picks the angle.".into(),
            cost: 1.18,
        },
        HandoffAgent {
            id: "writer".into(),
            name: "Writer".into(),
            model: "GPT-5".into(),
            brand: "openai".into(),
            description: "Shapes the brief into a first draft in the house voice.".into(),
            cost: 0.54,
        },
        HandoffAgent {
            id: "review".into(),
            name: "Reviewer".into(),
            model: "Gemini 2.5 Pro".into(),
            brand: "gemini".into(),
            description: "Tests each claim against its source before release.".into(),
            cost: 0.27,
        },
    ];
    let lost_quotes = || {
        lost(
            "quotes",
            "4 customer quotes",
            "Cut to fit the Writer's 32k context budget before the handoff.",
        )
    };
    let mut handoffs = vec![
        Handoff {
            id: "h-welcome-1".into(),
            from: "research".into(),
            to: "writer".into(),
            task: "Welcome email".into(),
            items: vec![
                item("a", "onboarding.md", ItemKind::File),
                item("b", "Where teams stall", ItemKind::Fact),
                item("c", "One step at a time", ItemKind::Decision),
            ],
            at: 0,
        },
        Handoff {
            id: "h-welcome-2".into(),
            from: "writer".into(),
            to: "review".into(),
            task: "Welcome email".into(),
            items: vec![
                item("a", "Email draft", ItemKind::File),
                item("b", "Tone: warm", ItemKind::Note),
            ],
            at: 0,
        },
        Handoff {
            id: "h-docs-1".into(),
            from: "research".into(),
            to: "writer".into(),
            task: "Integrator docs".into(),
            items: vec![
                item("a", "api.yaml", ItemKind::File),
                item("b", "3 examples", ItemKind::Source),
                item("c", "Write for builders", ItemKind::Decision),
            ],
            at: 0,
        },
        Handoff {
            id: "h-docs-2".into(),
            from: "writer".into(),
            to: "review".into(),
            task: "Integrator docs".into(),
            items: vec![
                item("a", "Draft 2", ItemKind::File),
                item("b", "3 examples", ItemKind::Source),
            ],
            at: 0,
        },
        Handoff {
            id: "h-refund-1".into(),
            from: "research".into(),
            to: "writer".into(),
            task: "Refund macro".into(),
            items: vec![
                item("a", "40 sample replies", ItemKind::Source),
                item("b", "refunds.md", ItemKind::File),
            ],
            at: 0,
        },
        Handoff {
            id: "h-refund-2".into(),
            from: "writer".into(),
            to: "review".into(),
            task: "Refund macro".into(),
            items: vec![
                item("a", "Macro draft", ItemKind::File),
                item("b", "Tone: kind", ItemKind::Note),
            ],
            at: 0,
        },
        Handoff {
            id: "h-winback-1".into(),
            from: "research".into(),
            to: "writer".into(),
            task: "Win-back email".into(),
            items: vec![
                item("a", "winback-brief.md", ItemKind::File),
                item("b", "9 sources", ItemKind::Source),
                item("c", "Open with the fix", ItemKind::Decision),
                lost_quotes(),
            ],
            at: 0,
        },
        Handoff {
            id: "h-winback-2".into(),
            from: "writer".into(),
            to: "review".into(),
            task: "Win-back email".into(),
            items: vec![
                item("a", "Draft 1", ItemKind::File),
                item("b", "Tone: direct", ItemKind::Note),
                item("c", "p95 2.1s", ItemKind::Fact),
            ],
            at: 0,
        },
    ];
    let mut tasks = vec![
        task("t-welcome", "Welcome email", "notion", Done, "", ""),
        task("t-docs", "Integrator docs", "github", Done, "", ""),
        task("t-refund", "Refund macro", "slack", Done, "", ""),
        task(
            "t-winback",
            "Win-back email",
            "slack",
            Warning,
            "",
            "2 claims unchecked",
        ),
        task("t-launch", "Launch article", "notion", Todo, "", ""),
        task("t-checklist", "Release checklist", "github", Todo, "", ""),
        task("t-pricing", "Pricing page copy", "notion", Todo, "", ""),
    ];
    let mut costs = [1.18, 0.54, 0.27];
    let launch = tasks.iter().position(|t| t.id == "t-launch").unwrap_or(0);
    // The Researcher picks up the launch article…
    if t >= 1_500 {
        tasks[launch].status = Working;
        tasks[launch].agent = "research".into();
        tasks[launch].note = "Reading sources".into();
        costs[0] += (t.min(6_000) - 1_500) as f64 / 1000.0 * 0.05;
    }
    // …hands it to the Writer…
    if t >= 6_000 {
        handoffs.push(Handoff {
            id: "h-launch-1".into(),
            from: "research".into(),
            to: "writer".into(),
            task: "Launch article".into(),
            items: vec![
                item("a", "launch-brief.md", ItemKind::File),
                item("b", "11 sources", ItemKind::Source),
                item("c", "Open with the fix", ItemKind::Decision),
                lost_quotes(),
            ],
            at: 6_000,
        });
        tasks[launch].agent = "writer".into();
        tasks[launch].note = "Drafting".into();
        costs[1] += (t.min(11_500) - 6_000) as f64 / 1000.0 * 0.03;
    }
    // …who hands it to the Reviewer, who flags two claims.
    if t >= 11_500 {
        handoffs.push(Handoff {
            id: "h-launch-2".into(),
            from: "writer".into(),
            to: "review".into(),
            task: "Launch article".into(),
            items: vec![
                item("a", "Draft 1", ItemKind::File),
                item("b", "Tone: bold", ItemKind::Note),
                item("c", "p95 2.1s", ItemKind::Fact),
            ],
            at: 11_500,
        });
        tasks[launch].agent = "review".into();
        tasks[launch].note = "Checking claims".into();
        costs[2] += (t.min(16_000) - 11_500) as f64 / 1000.0 * 0.02;
    }
    if t >= 16_000 {
        tasks[launch].status = Warning;
        tasks[launch].agent.clear();
        tasks[launch].note = "2 claims unchecked".into();
    }
    let mut agents = agents;
    for (a, c) in agents.iter_mut().zip(costs) {
        a.cost = c;
    }
    HandoffScene {
        agents,
        handoffs,
        tasks,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_launch_article_travels_down_the_relay() {
        let s = handoff_at(0);
        let (t, up_next, more) = s.agent_task(0).unwrap();
        assert!(up_next);
        assert_eq!(t.title, "Launch article");
        assert_eq!(more, 2);
        assert_eq!(s.column("research", "writer").len(), 4);
        let s = handoff_at(7_000);
        let col = s.column("research", "writer");
        assert_eq!(col[0].task, "Launch article");
        assert_eq!(
            handoff_status(&col[0], &s.tasks, &s.agents, 7_000),
            ("spin", "Handing on".to_string())
        );
        assert_eq!(
            handoff_status(&col[0], &s.tasks, &s.agents, 9_000),
            ("spin", "With Writer".to_string())
        );
        let s = handoff_at(17_000);
        let col = s.column("writer", "review");
        assert_eq!(
            handoff_status(&col[0], &s.tasks, &s.agents, 17_000).0,
            "warn"
        );
        assert_eq!(items_line(&col[0].items), "Draft 1, Tone: bold, p95 2.1s");
    }

    #[test]
    fn cards_only_move_down_the_relay() {
        let mut s = handoff_at(0);
        assert!(s.can_take("writer", "review"));
        assert!(!s.can_take("writer", "research"));
        assert!(s.can_take("review", "review"));
        s.assign("Pricing page copy", "review", "Checking claims");
        let t = s
            .tasks
            .iter()
            .find(|t| t.title == "Pricing page copy")
            .unwrap();
        assert_eq!(t.status, TaskStatus::Working);
        assert_eq!(handoff_cost(1.4199), "$1.42");
    }
}
