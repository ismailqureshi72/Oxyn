//! affectUI fidelity pass, family B part 9 (plan #32): the **Running Now** model. Background
//! agent tasks summarised as one quiet pill ("3 running · 2 need you"); each task has steps, a
//! state (running, paused, needs you, done) and the step it is on. Running tasks advance a step at
//! a time; pausing freezes them where they are. Behaviour only: the views live in `oxyn-dioxus` /
//! `oxyn-gpui` (`kit/runningfid.rs`).

/// A task's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RnState {
    /// Working.
    Running,
    /// Stopped by you.
    Paused,
    /// Waiting for your answer.
    NeedsYou,
    /// Finished.
    Done,
}

impl RnState {
    /// `running` / `paused` / `needs-you` / `done`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::NeedsYou => "needs-you",
            Self::Done => "done",
        }
    }
}

/// One background task.
#[derive(Clone, Debug, PartialEq)]
pub struct RnTask {
    /// Stable id.
    pub id: String,
    /// Title.
    pub title: String,
    /// The steps in order.
    pub steps: Vec<String>,
    /// Steps finished.
    pub done: u32,
    /// State.
    pub state: RnState,
}

impl RnTask {
    /// Total steps.
    pub fn total(&self) -> u32 {
        self.steps.len() as u32
    }
    /// The line under the title: the current step, "Paused · <step>", "Waiting for you · …",
    /// or "Finished".
    pub fn line(&self) -> String {
        let cur = self
            .steps
            .get((self.done as usize).min(self.steps.len().saturating_sub(1)))
            .cloned()
            .unwrap_or_default();
        match self.state {
            RnState::Running => cur,
            RnState::Paused => format!("Paused · {cur}"),
            RnState::NeedsYou => format!("Waiting for you · {cur}"),
            RnState::Done => "Finished".into(),
        }
    }
}

/// Steps done for a running task that had `base` done at `since_ms` ago and makes one step per
/// `step_ms` (capped at `total`).
pub fn rn_done_at(base: u32, total: u32, elapsed_ms: u64, step_ms: u64) -> u32 {
    (base + (elapsed_ms / step_ms.max(1)) as u32).min(total)
}

/// (running, needs you).
pub fn rn_counts(t: &[RnTask]) -> (usize, usize) {
    (
        t.iter().filter(|x| x.state == RnState::Running).count(),
        t.iter().filter(|x| x.state == RnState::NeedsYou).count(),
    )
}

/// The pill's words: "3 running · 2 need you", "1 needs you", "All done".
pub fn rn_label(t: &[RnTask]) -> String {
    let (r, n) = rn_counts(t);
    let mut parts = vec![];
    if r > 0 {
        parts.push(format!("{r} running"));
    }
    if n > 0 {
        parts.push(format!(
            "{n} {}",
            if n == 1 { "needs you" } else { "need you" }
        ));
    }
    if parts.is_empty() {
        if t.iter().any(|x| x.state == RnState::Paused) {
            return "Paused".into();
        }
        return "All done".into();
    }
    parts.join(" · ")
}

fn tk(id: &str, title: &str, steps: &[&str], done: u32, state: RnState) -> RnTask {
    RnTask {
        id: id.into(),
        title: title.into(),
        steps: steps.iter().map(|s| s.to_string()).collect(),
        done,
        state,
    }
}

/// The demo tasks: three running, one waiting for you, one paused.
pub fn rn_demo() -> Vec<RnTask> {
    vec![
        tk(
            "t1",
            "Nightly import",
            &[
                "Fetching files",
                "Parsing rows",
                "Checking totals",
                "Writing tables",
            ],
            1,
            RnState::Running,
        ),
        tk(
            "t2",
            "Refund review",
            &[
                "Reading the claim",
                "Matching the order",
                "Drafting a reply",
            ],
            2,
            RnState::NeedsYou,
        ),
        tk(
            "t3",
            "Docs translation",
            &[
                "Splitting pages",
                "Translating",
                "Checking terms",
                "Publishing",
                "Notifying",
            ],
            2,
            RnState::Running,
        ),
        tk(
            "t4",
            "Lead scoring",
            &["Pulling contacts", "Scoring", "Syncing back"],
            1,
            RnState::Paused,
        ),
        tk(
            "t5",
            "Weekly report",
            &[
                "Collecting numbers",
                "Making charts",
                "Writing the summary",
                "Sending",
            ],
            0,
            RnState::Running,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_label_lines() {
        let t = rn_demo();
        assert_eq!(rn_counts(&t), (3, 1));
        assert_eq!(rn_label(&t), "3 running · 1 needs you");
        let mut u = t.clone();
        u[1].state = RnState::Running;
        u[0].state = RnState::NeedsYou;
        u[2].state = RnState::NeedsYou;
        assert_eq!(rn_label(&u), "2 running · 2 need you");
        assert_eq!(t[3].line(), "Paused · Scoring");
        assert_eq!(t[0].line(), "Parsing rows");
        assert_eq!(t[1].line(), "Waiting for you · Drafting a reply");
        assert_eq!(rn_done_at(1, 4, 2500, 1000), 3);
        assert_eq!(rn_done_at(1, 4, 9000, 1000), 4);
        let done: Vec<RnTask> = t
            .iter()
            .cloned()
            .map(|mut x| {
                x.state = RnState::Done;
                x
            })
            .collect();
        assert_eq!(rn_label(&done), "All done");
        assert_eq!(done[0].line(), "Finished");
    }
}
