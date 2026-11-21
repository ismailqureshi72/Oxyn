//! Background agents: coding jobs handed to agents that run off-screen. The job field takes the
//! task plus the repo, branch and model to use; Start puts a card on top that moves from queued to
//! setting up to running (the live step shimmers) until it is waiting on review with lines added and removed,
//! the changed files and the checks. Create pull request turns into the PR number; Merge finishes
//! it. Filters split the list; cards expand to their step log and files; running tasks cancel,
//! failed or cancelled ones retry, finished ones archive with Undo. Phases derive from `now`.

use super::{dur, dur_words, norm_key, nth, rove};

/// Repositories: name and branches (the first is the default).
pub const REPOS: [(&str, &[&str]); 3] = [
    ("northwind/web", &["main", "develop"]),
    ("northwind/api", &["main", "release/3.2"]),
    ("northwind/mobile", &["main"]),
];
/// Models: brand id and label.
pub const MODELS: [(&str, &str); 3] = [
    ("claude", "Claude Sonnet 4.5"),
    ("openai", "GPT-5 Codex"),
    ("gemini", "Gemini 2.5 Pro"),
];
/// The steps every task runs after setup.
pub const STEPS: [&str; 5] = [
    "Reading the codebase",
    "Planning the change",
    "Editing files",
    "Running the tests",
    "Writing the summary",
];
/// Queued, ms.
pub const QUEUE_MS: u64 = 1_400;
/// Setting up the environment, ms.
pub const SETUP_MS: u64 = 2_000;
/// One step, ms.
pub const STEP_MS: u64 = 1_700;
/// Creating a pull request, ms.
pub const PR_MS: u64 = 1_200;
/// Undo window after archiving, ms.
pub const UNDO_MS: u64 = 6_000;
/// Filters.
pub const FILTERS: [&str; 4] = ["All", "Running", "Ready for review", "Merged"];

/// Where a task is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Waiting for a machine.
    Queued,
    /// Setting up the environment.
    Setup,
    /// Running step n.
    Running(usize),
    /// Ready for review.
    Ready,
    /// Failed at step n.
    Failed(usize),
    /// Cancelled.
    Cancelled,
    /// Merged.
    Merged,
}

impl Phase {
    /// Label and hue.
    pub fn label(self) -> (&'static str, &'static str) {
        match self {
            Phase::Queued => ("Queued", "neutral"),
            Phase::Setup => ("Setting up environment", "blue"),
            Phase::Running(_) => ("Running", "blue"),
            Phase::Ready => ("Ready for review", "green"),
            Phase::Failed(_) => ("Failed", "red"),
            Phase::Cancelled => ("Cancelled", "neutral"),
            Phase::Merged => ("Merged", "violet"),
        }
    }
    /// Still working.
    pub fn busy(self) -> bool {
        matches!(self, Phase::Queued | Phase::Setup | Phase::Running(_))
    }
}

/// One task.
#[derive(Clone, Debug, PartialEq)]
pub struct Task {
    /// Id.
    pub id: usize,
    /// What it does.
    pub title: String,
    /// Repository index.
    pub repo: usize,
    /// Branch index.
    pub branch: usize,
    /// Model index.
    pub model: usize,
    /// Started at, host ms.
    pub at: u64,
    /// Fails at this step.
    pub fail_at: Option<usize>,
    /// Cancelled at.
    pub cancelled: Option<u64>,
    /// Pull request: number and when it was asked for.
    pub pr: Option<(u32, u64)>,
    /// Merged.
    pub merged: bool,
    /// Archived at.
    pub archived: Option<u64>,
    /// Lines added, removed, files changed.
    pub diff: (u32, u32, usize),
}

/// Changed files of a task (path, lines added, removed).
pub fn files(t: &Task) -> Vec<(String, u32, u32)> {
    let base = [
        "src/pricing/plan_table",
        "src/pricing/yearly_toggle",
        "tests/pricing",
    ];
    let ext = if REPOS[t.repo].0.ends_with("api") {
        ".rs"
    } else {
        ".tsx"
    };
    (0..t.diff.2)
        .map(|i| {
            let k = i as u32 + 1;
            (
                format!("{}{}", base[i % 3], ext),
                t.diff.0 / k / 2 + 3,
                t.diff.1 / k / 2,
            )
        })
        .collect()
}

/// The live state of the block.
#[derive(Clone, Debug, PartialEq)]
pub struct BgAgents {
    /// Tasks, newest first.
    pub tasks: Vec<Task>,
    /// Filter index.
    pub filter: usize,
    /// Expanded task id.
    pub open: Option<usize>,
    /// Composer: repository, branch, model.
    pub pick: (usize, usize, usize),
    /// Next id.
    pub next: usize,
    /// The next started task fails (the error scenario).
    pub fail_next: bool,
}

impl BgAgents {
    /// The sample list at `now`: one running, one ready, one merged, one failed.
    pub fn new(fail_next: bool, now: u64) -> Self {
        let t = |id, title: &str, repo, model, ago: u64, fail_at, pr, merged, diff| Task {
            id,
            title: title.into(),
            repo,
            branch: 0,
            model,
            at: now.saturating_sub(ago),
            fail_at,
            cancelled: None,
            pr,
            merged,
            archived: None,
            diff,
        };
        let done = QUEUE_MS + SETUP_MS + STEP_MS * STEPS.len() as u64;
        Self {
            tasks: vec![
                t(
                    3,
                    "Add a yearly toggle to the pricing table",
                    0,
                    0,
                    4_200,
                    None,
                    None,
                    false,
                    (148, 36, 3),
                ),
                t(
                    2,
                    "Return 429 with Retry-After from the rate limiter",
                    1,
                    1,
                    done + 50_000,
                    None,
                    None,
                    false,
                    (64, 12, 2),
                ),
                t(
                    1,
                    "Fix the flaky checkout test on Safari",
                    0,
                    0,
                    done + 900_000,
                    None,
                    Some((412, 0)),
                    true,
                    (22, 9, 1),
                ),
                t(
                    0,
                    "Upgrade the push notification library",
                    2,
                    2,
                    done + 400_000,
                    Some(3),
                    None,
                    false,
                    (0, 0, 0),
                ),
            ],
            filter: 0,
            open: None,
            pick: (0, 0, 0),
            next: 4,
            fail_next,
        }
    }

    /// Phase of `t` at `now`.
    pub fn phase(t: &Task, now: u64) -> Phase {
        if t.merged {
            return Phase::Merged;
        }
        let end = t.cancelled.unwrap_or(u64::MAX).min(now);
        let el = end.saturating_sub(t.at);
        let ph = if el < QUEUE_MS {
            Phase::Queued
        } else if el < QUEUE_MS + SETUP_MS {
            Phase::Setup
        } else {
            let k = ((el - QUEUE_MS - SETUP_MS) / STEP_MS) as usize;
            match t.fail_at {
                Some(f) if k >= f => Phase::Failed(f),
                _ if k >= STEPS.len() => Phase::Ready,
                _ => Phase::Running(k),
            }
        };
        if t.cancelled.is_some() && ph.busy() {
            Phase::Cancelled
        } else {
            ph
        }
    }

    /// Elapsed time of `t` (frozen once it stops).
    pub fn elapsed(t: &Task, now: u64) -> u64 {
        let total = QUEUE_MS + SETUP_MS + STEP_MS * STEPS.len() as u64;
        let fail = t.fail_at.map(|f| QUEUE_MS + SETUP_MS + STEP_MS * f as u64);
        let el = t.cancelled.unwrap_or(now).saturating_sub(t.at);
        el.min(fail.unwrap_or(total)).min(total)
    }

    /// Elapsed text and its spoken form.
    pub fn time(t: &Task, now: u64) -> (String, String) {
        let e = Self::elapsed(t, now);
        let busy = Self::phase(t, now).busy();
        (
            dur(e),
            format!(
                "{} for {}",
                if busy { "Running" } else { "Ran" },
                dur_words(e)
            ),
        )
    }

    /// The live line under a busy task.
    pub fn step_line(t: &Task, now: u64) -> String {
        match Self::phase(t, now) {
            Phase::Queued => "Waiting for a machine".into(),
            Phase::Setup => format!("Cloning {} and installing", REPOS[t.repo].0),
            Phase::Running(k) => STEPS[k].into(),
            Phase::Failed(k) => format!("{}: 3 of 52 tests fail after the upgrade", STEPS[k]),
            Phase::Ready => format!("{} files changed · checks passed", t.diff.2),
            Phase::Cancelled => "Stopped before it finished".into(),
            Phase::Merged => "Merged into main".into(),
        }
    }

    /// PR state: `None`, creating, or the number.
    pub fn pr(t: &Task, now: u64) -> Option<Result<u32, ()>> {
        t.pr.map(|(n, at)| if now >= at + PR_MS { Ok(n) } else { Err(()) })
    }

    fn passes(&self, t: &Task, now: u64) -> bool {
        let ph = Self::phase(t, now);
        t.archived.is_none_or(|a| now < a + UNDO_MS)
            && match self.filter {
                1 => ph.busy(),
                2 => ph == Phase::Ready,
                3 => ph == Phase::Merged,
                _ => true,
            }
    }

    /// Visible tasks at `now`.
    pub fn visible(&self, now: u64) -> Vec<&Task> {
        self.tasks.iter().filter(|t| self.passes(t, now)).collect()
    }

    /// Counts per filter at `now`.
    pub fn counts(&self, now: u64) -> [usize; 4] {
        let live: Vec<&Task> = self.tasks.iter().filter(|t| t.archived.is_none()).collect();
        let n = |f: &dyn Fn(Phase) -> bool| live.iter().filter(|t| f(Self::phase(t, now))).count();
        [
            live.len(),
            n(&|p| p.busy()),
            n(&|p| p == Phase::Ready),
            n(&|p| p == Phase::Merged),
        ]
    }

    fn find(&mut self, id: usize) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// Starts `title` with the composer's picks at `now`.
    pub fn start(&mut self, title: &str, now: u64) -> Option<String> {
        let title = title.trim();
        if title.is_empty() {
            return None;
        }
        let (repo, branch, model) = self.pick;
        let fail_at = self.fail_next.then_some(3);
        self.fail_next = false;
        let diff = (
            40 + (title.len() as u32 * 3) % 120,
            6 + title.len() as u32 % 30,
            2 + title.len() % 3,
        );
        self.tasks.insert(
            0,
            Task {
                id: self.next,
                title: title.into(),
                repo,
                branch,
                model,
                at: now,
                fail_at,
                cancelled: None,
                pr: None,
                merged: false,
                archived: None,
                diff,
            },
        );
        self.next += 1;
        if self.filter > 1 {
            self.filter = 0;
        }
        Some(format!(
            "Started on {} with {}. The preview simulates the run",
            REPOS[repo].0, MODELS[model].1
        ))
    }

    /// Presses: `ba-repo-<i>`, `ba-branch-<i>`, `ba-model-<i>`, `ba-f-<i>`, `ba-open-<id>`,
    /// `ba-cancel-<id>`, `ba-retry-<id>`, `ba-pr-<id>`, `ba-merge-<id>`, `ba-archive-<id>`,
    /// `ba-undo-<id>`.
    pub fn press(&mut self, id: &str, now: u64) -> Option<String> {
        if let Some(i) = nth(id, "ba-repo-") {
            self.pick = (i.min(REPOS.len() - 1), 0, self.pick.2);
            return Some(format!(
                "Repository {}, branch {}",
                REPOS[self.pick.0].0, REPOS[self.pick.0].1[0]
            ));
        }
        if let Some(i) = nth(id, "ba-branch-") {
            self.pick.1 = i.min(REPOS[self.pick.0].1.len() - 1);
            return Some(format!("Branch {}", REPOS[self.pick.0].1[self.pick.1]));
        }
        if let Some(i) = nth(id, "ba-model-") {
            self.pick.2 = i.min(MODELS.len() - 1);
            return Some(format!("Model {}", MODELS[self.pick.2].1));
        }
        if let Some(i) = nth(id, "ba-f-") {
            self.filter = i.min(FILTERS.len() - 1);
            return Some(format!(
                "{}: {} tasks",
                FILTERS[self.filter],
                self.visible(now).len()
            ));
        }
        if let Some(i) = nth(id, "ba-open-") {
            self.open = if self.open == Some(i) { None } else { Some(i) };
            return None;
        }
        let (verb, i) = ["cancel", "retry", "pr", "merge", "archive", "undo"]
            .iter()
            .find_map(|v| nth(id, &format!("ba-{v}-")).map(|i| (*v, i)))?;
        let fail_next = self.fail_next;
        let t = self.find(i)?;
        let ph = Self::phase(t, now);
        let out = match verb {
            "cancel" if ph.busy() => {
                t.cancelled = Some(now);
                format!("Cancelled {}", t.title)
            }
            "retry" if matches!(ph, Phase::Failed(_) | Phase::Cancelled) => {
                t.at = now;
                t.cancelled = None;
                t.fail_at = fail_next.then_some(3);
                if t.diff.2 == 0 {
                    t.diff = (57, 41, 2);
                }
                format!("Retrying {}", t.title)
            }
            "pr" if ph == Phase::Ready && t.pr.is_none() => {
                t.pr = Some((418 + t.id as u32, now));
                "Creating a pull request. Simulated in the preview".into()
            }
            "merge" if ph == Phase::Ready && Self::pr(t, now) == Some(Ok(418 + t.id as u32)) => {
                t.merged = true;
                format!("Merged pull request #{}", 418 + t.id)
            }
            "archive" if !ph.busy() && t.archived.is_none() => {
                t.archived = Some(now);
                format!("Archived {}. Undo for a few seconds", t.title)
            }
            "undo" if t.archived.is_some_and(|a| now < a + UNDO_MS) => {
                t.archived = None;
                format!("Restored {}", t.title)
            }
            _ => return None,
        };
        if verb == "archive" && self.open == Some(i) {
            self.open = None;
        }
        Some(out)
    }

    /// Keys on the filter tabs.
    pub fn key(&mut self, key: &str, now: u64) -> Option<String> {
        let k = norm_key(key);
        let i = rove(self.filter, FILTERS.len(), k);
        (i != self.filter)
            .then(|| self.press(&format!("ba-f-{i}"), now))
            .flatten()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle() {
        let mut b = BgAgents::new(false, 100_000);
        let n = 100_000;
        assert_eq!(BgAgents::phase(&b.tasks[0], n), Phase::Running(0));
        assert_eq!(BgAgents::phase(&b.tasks[1], n), Phase::Ready);
        assert_eq!(BgAgents::phase(&b.tasks[2], n), Phase::Merged);
        assert_eq!(BgAgents::phase(&b.tasks[3], n), Phase::Failed(3));
        assert_eq!(b.counts(n), [4, 1, 1, 1]);
        b.press("ba-repo-1", n);
        b.press("ba-model-2", n);
        b.start("Add request ids to every log line", n);
        assert_eq!(b.tasks[0].repo, 1);
        assert_eq!(BgAgents::phase(&b.tasks[0], n + 500), Phase::Queued);
        assert_eq!(BgAgents::phase(&b.tasks[0], n + 2_000), Phase::Setup);
        assert_eq!(BgAgents::phase(&b.tasks[0], n + 20_000), Phase::Ready);
        b.press("ba-pr-4", n + 20_000);
        assert_eq!(BgAgents::pr(&b.tasks[0], n + 20_500), Some(Err(())));
        b.press("ba-merge-4", n + 22_000);
        assert_eq!(BgAgents::phase(&b.tasks[0], n + 22_000), Phase::Merged);
        b.press("ba-cancel-3", n);
        assert_eq!(BgAgents::phase(&b.tasks[1], n + 9_000), Phase::Cancelled);
        b.press("ba-retry-0", n);
        assert_eq!(BgAgents::phase(&b.tasks[4], n + 30_000), Phase::Ready);
        b.press("ba-archive-1", n);
        assert_eq!(b.visible(n).len(), 5);
        assert_eq!(b.visible(n + 7_000).len(), 4);
        b.key("End", n);
        assert_eq!(b.filter, 3);
    }
}
