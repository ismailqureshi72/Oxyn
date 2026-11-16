//! affectUI fidelity pass, family B part 8 (plan #32): the **Delegate Board** model. A person hands
//! work to a specific agent by hand: unassigned tasks wait in a tray, each agent shows its skills
//! and how busy it is, the agents that can take the selected task light up (the least busy one is
//! the best fit), and handed-over tasks run and finish in that agent's column. Behaviour only.

/// One agent that can take work.
#[derive(Clone, Debug, PartialEq)]
pub struct DbAgent {
    /// Stable id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Model.
    pub model: String,
    /// Brand id for its mark.
    pub brand: String,
    /// What it can do.
    pub skills: Vec<String>,
    /// How many tasks it runs at once.
    pub capacity: usize,
}

/// One task.
#[derive(Clone, Debug, PartialEq)]
pub struct DbTask {
    /// Stable id.
    pub id: String,
    /// What to do.
    pub title: String,
    /// The skill it needs.
    pub skill: String,
    /// Rough run time, s (the demo plays it faster).
    pub est_s: u32,
}

/// Where a handed-over task stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbStatus {
    /// Waiting for a free slot.
    Queued,
    /// Running.
    Running,
    /// Finished.
    Done,
}

impl DbStatus {
    /// `queued` / `running` / `done`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Running => "running",
            Self::Done => "done",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Queued => "Queued",
            Self::Running => "Running",
            Self::Done => "Done",
        }
    }
}

/// Whether an agent has the skill a task needs.
pub fn db_fits(a: &DbAgent, t: &DbTask) -> bool {
    a.skills.iter().any(|s| s == &t.skill)
}

/// Open (not done) tasks an agent holds, given `(task id, agent id, status)` rows.
pub fn db_load(agent: &str, rows: &[(String, String, DbStatus)]) -> usize {
    rows.iter()
        .filter(|(_, a, s)| a == agent && *s != DbStatus::Done)
        .count()
}

/// The best fit for a task: an agent with the skill and the most free slots (first on a tie).
pub fn db_best(
    agents: &[DbAgent],
    t: &DbTask,
    rows: &[(String, String, DbStatus)],
) -> Option<usize> {
    agents
        .iter()
        .enumerate()
        .filter(|(_, a)| db_fits(a, t))
        .max_by(|(i, a), (j, b)| {
            let fa = a.capacity as i64 - db_load(&a.id, rows) as i64;
            let fb = b.capacity as i64 - db_load(&b.id, rows) as i64;
            fa.cmp(&fb).then(j.cmp(i))
        })
        .map(|(i, _)| i)
}

/// Status of each task an agent holds, from when it was handed over (`ms_ago`) and a time scale
/// (ms per estimated second): tasks run in hand-over order, `capacity` at a time.
pub fn db_statuses(a: &DbAgent, held: &[(DbTask, u64)], ms_per_s: u64) -> Vec<DbStatus> {
    // Simulate slots: each task starts when handed over or when a slot frees, whichever is later.
    let mut slots: Vec<u64> = vec![0; a.capacity.max(1)];
    let mut out = vec![];
    // Hand-over time measured from the oldest hand-over.
    let oldest = held.iter().map(|(_, ago)| *ago).max().unwrap_or(0);
    for (t, ago) in held {
        let given = oldest - ago;
        let (si, free) = slots
            .iter()
            .enumerate()
            .min_by_key(|(_, f)| **f)
            .map(|(i, f)| (i, *f))
            .unwrap_or((0, 0));
        let start = given.max(free);
        let end = start + t.est_s as u64 * ms_per_s;
        slots[si] = end;
        let now = oldest;
        out.push(if now < start {
            DbStatus::Queued
        } else if now < end {
            DbStatus::Running
        } else {
            DbStatus::Done
        });
    }
    out
}

fn ag(id: &str, name: &str, model: &str, brand: &str, skills: &[&str], capacity: usize) -> DbAgent {
    DbAgent {
        id: id.into(),
        name: name.into(),
        model: model.into(),
        brand: brand.into(),
        skills: skills.iter().map(|s| (*s).to_string()).collect(),
        capacity,
    }
}

fn tk(id: &str, title: &str, skill: &str, est_s: u32) -> DbTask {
    DbTask {
        id: id.into(),
        title: title.into(),
        skill: skill.into(),
        est_s,
    }
}

/// Demo agents (invented).
pub fn db_agents() -> Vec<DbAgent> {
    vec![
        ag(
            "code",
            "Code agent",
            "Sonnet 4.5",
            "anthropic",
            &["code", "tests"],
            2,
        ),
        ag(
            "research",
            "Research agent",
            "GPT-5",
            "openai",
            &["research", "writing"],
            2,
        ),
        ag(
            "ops",
            "Ops agent",
            "Gemini 2.5 Pro",
            "gemini",
            &["infra", "code"],
            1,
        ),
    ]
}

/// Demo tasks (invented).
pub fn db_tasks() -> Vec<DbTask> {
    vec![
        tk("t1", "Fix the flaky export test", "tests", 6),
        tk("t2", "Compare three vector databases", "research", 8),
        tk("t3", "Rotate the staging API keys", "infra", 4),
        tk("t4", "Draft the changelog for 2.4", "writing", 5),
        tk("t5", "Add retries to the webhook sender", "code", 7),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fit_load_and_best() {
        let a = db_agents();
        let t = db_tasks();
        assert!(db_fits(&a[0], &t[0]));
        assert!(!db_fits(&a[1], &t[0]));
        // Code work: the code agent (2 slots) beats ops (1 slot).
        assert_eq!(db_best(&a, &t[4], &[]), Some(0));
        let rows = vec![
            ("x".to_string(), "code".to_string(), DbStatus::Running),
            ("y".to_string(), "code".to_string(), DbStatus::Running),
        ];
        assert_eq!(db_load("code", &rows), 2);
        // Code agent full: ops (1 free) is the best fit now.
        assert_eq!(db_best(&a, &t[4], &rows), Some(2));
        assert_eq!(db_best(&a, &tk("z", "Paint", "art", 1), &rows), None);
    }

    #[test]
    fn statuses_respect_capacity() {
        let ops = &db_agents()[2]; // one slot
        let t = db_tasks();
        // Two tasks handed over together, 4 s and 7 s at 100 ms/s; 300 ms later.
        let held = vec![(t[2].clone(), 300), (t[4].clone(), 300)];
        let s = db_statuses(ops, &held, 100);
        assert_eq!(s, vec![DbStatus::Running, DbStatus::Queued]);
        // The first, handed over long ago, is done; the second runs since it arrived.
        let held = vec![(t[2].clone(), 5000), (t[4].clone(), 100)];
        let s = db_statuses(ops, &held, 100);
        assert_eq!(s, vec![DbStatus::Done, DbStatus::Running]);
        assert_eq!(DbStatus::Queued.label(), "Queued");
    }
}
