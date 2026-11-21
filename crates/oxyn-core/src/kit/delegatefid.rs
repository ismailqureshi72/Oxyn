//! affectUI fidelity pass, family B part 8 (plan #32): the **Delegate Board** model. A pile of
//! unassigned tasks and a column of agents, each with a model, a per-minute rate, the job it is on
//! and a queue. Handing a task over appends it to that agent's queue, pushes its "busy until" time
//! and adds to the money waiting in queues; Auto mode hands each task to whoever is free first.
//! Behaviour only: the views live in `oxyn-dioxus` / `oxyn-gpui` (`kit/delegatefid.rs`).

/// One agent.
#[derive(Clone, Debug, PartialEq)]
pub struct DbAgent {
    /// Stable id.
    pub id: String,
    /// Display name ("Coder").
    pub name: String,
    /// Model line ("Opus 4.1").
    pub model: String,
    /// Brand id for the avatar mark.
    pub brand: String,
    /// Dollars per working minute.
    pub rate: f64,
    /// The job it is on now and the minutes left on it.
    pub current: Option<(String, u32)>,
}

/// One task.
#[derive(Clone, Debug, PartialEq)]
pub struct DbTask {
    /// Stable id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Estimate, minutes.
    pub est: u32,
    /// Brand id of the tool it mostly needs.
    pub tool: String,
}

/// Minutes of work an agent holds: what is left of its current job plus every queued task.
pub fn db_load_min(a: &DbAgent, tasks: &[DbTask], given: &[(String, String)]) -> u32 {
    a.current.as_ref().map_or(0, |c| c.1)
        + given
            .iter()
            .filter(|(_, ai)| *ai == a.id)
            .filter_map(|(ti, _)| tasks.iter().find(|t| t.id == *ti))
            .map(|t| t.est)
            .sum::<u32>()
}

/// "10:41" for minutes after midnight (wraps at 24 h).
pub fn db_clock(min: u32) -> String {
    let m = min % (24 * 60);
    format!("{}:{:02}", m / 60, m % 60)
}

/// Money waiting in queues: every agent's held minutes at its rate.
pub fn db_cost(agents: &[DbAgent], tasks: &[DbTask], given: &[(String, String)]) -> f64 {
    agents
        .iter()
        .map(|a| db_load_min(a, tasks, given) as f64 * a.rate)
        .sum()
}

/// "$2.16".
pub fn db_usd(v: f64) -> String {
    format!("${v:.2}")
}

/// The agent that is free first (fewest held minutes; the earlier one on a tie).
pub fn db_free_first(
    agents: &[DbAgent],
    tasks: &[DbTask],
    given: &[(String, String)],
) -> Option<usize> {
    agents
        .iter()
        .enumerate()
        .min_by_key(|(i, a)| (db_load_min(a, tasks, given), *i))
        .map(|(i, _)| i)
}

/// Auto mode's plan: the tasks still in the pile handed one by one, each to whoever is free first
/// at that moment. Returns (task id, agent id) in order.
pub fn db_auto_plan(
    agents: &[DbAgent],
    tasks: &[DbTask],
    given: &[(String, String)],
) -> Vec<(String, String)> {
    let mut g = given.to_vec();
    let mut out = vec![];
    for t in tasks {
        if g.iter().any(|(ti, _)| *ti == t.id) {
            continue;
        }
        let Some(i) = db_free_first(agents, tasks, &g) else {
            break;
        };
        let pair = (t.id.clone(), agents[i].id.clone());
        g.push(pair.clone());
        out.push(pair);
    }
    out
}

fn ag(
    id: &str,
    name: &str,
    model: &str,
    brand: &str,
    rate: f64,
    cur: Option<(&str, u32)>,
) -> DbAgent {
    DbAgent {
        id: id.into(),
        name: name.into(),
        model: model.into(),
        brand: brand.into(),
        rate,
        current: cur.map(|(t, m)| (t.to_string(), m)),
    }
}

fn tk(id: &str, title: &str, est: u32, tool: &str) -> DbTask {
    DbTask {
        id: id.into(),
        title: title.into(),
        est,
        tool: tool.into(),
    }
}

/// The demo agents: two busy, one free.
pub fn db_agents() -> Vec<DbAgent> {
    vec![
        ag(
            "coder",
            "Coder",
            "Opus 4.1",
            "claude",
            0.08,
            Some(("Fix the flaky login test", 12)),
        ),
        ag(
            "research",
            "Researcher",
            "GPT-5",
            "openai",
            0.05,
            Some(("Compare three vendors", 20)),
        ),
        ag("ops", "Ops", "Gemini 2.5 Pro", "gemini", 0.04, None),
    ]
}

/// The demo pile.
pub fn db_tasks() -> Vec<DbTask> {
    vec![
        tk("t1", "Merge duplicate CRM contacts", 15, "hubspot"),
        tk("t2", "Draft the release notes", 10, "notion"),
        tk("t3", "Rotate the staging keys", 8, "github"),
        tk("t4", "Tag last week's tickets", 20, "linear"),
    ]
}

/// The demo's starting clock, minutes after midnight (10:21).
pub const DB_START: u32 = 10 * 60 + 21;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_cost_and_clock() {
        let a = db_agents();
        let t = db_tasks();
        assert_eq!(db_load_min(&a[0], &t, &[]), 12);
        assert_eq!(db_clock(DB_START + 12), "10:33");
        assert_eq!(db_clock(24 * 60 + 5), "0:05");
        // 12 × 0.08 + 20 × 0.05 = 1.96
        assert_eq!(db_usd(db_cost(&a, &t, &[])), "$1.96");
        let g = vec![("t1".to_string(), "coder".to_string())];
        assert_eq!(db_load_min(&a[0], &t, &g), 27);
        assert_eq!(db_usd(db_cost(&a, &t, &g)), "$3.16");
    }

    #[test]
    fn auto_hands_to_free_first() {
        let a = db_agents();
        let t = db_tasks();
        assert_eq!(db_free_first(&a, &t, &[]), Some(2));
        let plan = db_auto_plan(&a, &t, &[]);
        let who: Vec<&str> = plan.iter().map(|(_, a)| a.as_str()).collect();
        // ops 0 → t1 (15); coder 12 → t2 (22); ops 15 → t3 (23); research 20 → t4.
        assert_eq!(who, ["ops", "coder", "ops", "research"]);
        let done = db_auto_plan(&a, &t, &plan);
        assert!(done.is_empty());
    }
}
