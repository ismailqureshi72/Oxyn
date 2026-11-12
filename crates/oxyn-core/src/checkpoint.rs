//! Checkpoints (`checkpoint-timeline.md`, `checkpoints-git.md`): checkpoints as a tree of
//! attempts (rewinding branches a new attempt and the old one fades but stays), the head path,
//! restore scopes with honest labels, and the restore flow with its undo window.

use std::time::Duration;

use crate::ClockMs;

/// How a checkpoint ended up.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckpointStatus {
    /// The agent is working toward it (shimmers).
    Running,
    /// Taken.
    #[default]
    Done,
    /// Ended in a failure (badge).
    Failed,
    /// You rewound away while the agent worked toward it.
    Stopped,
}

/// Who made it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum CheckpointKind {
    /// Before a turn that wrote files.
    #[default]
    Auto,
    /// Saved by the user, maybe named.
    Manual,
    /// A milestone (tests green, deploy).
    Milestone,
}

/// One checkpoint.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Checkpoint {
    /// Id.
    pub id: String,
    /// The one it follows. A second child of the same parent starts a new attempt.
    pub parent: Option<String>,
    /// Title ("Built the pricing table") or the prompt it precedes.
    pub title: String,
    /// Status.
    pub status: CheckpointStatus,
    /// Kind.
    pub kind: CheckpointKind,
    /// Run time when taken, from the run start.
    pub at: Duration,
    /// Files changed since the parent.
    pub files: u32,
    /// Lines `(+, −)`.
    pub lines: (u32, u32),
    /// Cost so far in cents.
    pub cost_cents: Option<u32>,
    /// No file changes: chat-only marker (◇).
    pub chat_only: bool,
}

impl Checkpoint {
    /// A done checkpoint.
    pub fn new(id: &str, parent: Option<&str>, title: &str, at_secs: u64) -> Self {
        Self {
            id: id.into(),
            parent: parent.map(str::to_string),
            title: title.into(),
            status: CheckpointStatus::Done,
            kind: CheckpointKind::Auto,
            at: Duration::from_secs(at_secs),
            files: 0,
            lines: (0, 0),
            cost_cents: None,
            chat_only: false,
        }
    }

    /// Builder: status.
    pub fn with(mut self, s: CheckpointStatus) -> Self {
        self.status = s;
        self
    }

    /// Builder: files and lines.
    pub fn files(mut self, n: u32, add: u32, del: u32) -> Self {
        self.files = n;
        self.lines = (add, del);
        self
    }
}

/// One drawn row of the lane graph.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CheckpointRow {
    /// Index into the checkpoints slice.
    pub index: usize,
    /// Lane (0 = first attempt); also the attempt number − 1.
    pub lane: usize,
    /// Lane of the parent (for the branch curve), when it differs.
    pub branch_from: Option<usize>,
    /// On the way to the head (accent), else faded.
    pub on_path: bool,
    /// The head ("You are here").
    pub current: bool,
    /// Time ("1:05").
    pub time: String,
    /// Accessible name.
    pub aria: String,
}

/// `0:42`, `1:05`, `1:02:33`.
pub fn format_at(d: Duration) -> String {
    let s = d.as_secs();
    if s >= 3_600 {
        format!("{}:{:02}:{:02}", s / 3_600, s / 60 % 60, s % 60)
    } else {
        format!("{}:{:02}", s / 60, s % 60)
    }
}

/// Lays checkpoints out in lanes. A checkpoint whose parent already has an earlier child starts
/// a new lane (a new attempt); everything off the head's ancestry is faded.
pub fn lanes(cps: &[Checkpoint], head: Option<&str>) -> Vec<CheckpointRow> {
    let head = head
        .map(str::to_string)
        .or_else(|| cps.last().map(|c| c.id.clone()));
    let mut path: Vec<String> = vec![];
    let mut cur = head.clone();
    while let Some(id) = cur {
        cur = cps
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.parent.clone());
        path.push(id);
    }
    let mut lane_of: Vec<(String, usize)> = vec![];
    let mut next_lane = 0usize;
    let mut out = vec![];
    for (i, c) in cps.iter().enumerate() {
        let parent_lane = c
            .parent
            .as_ref()
            .and_then(|p| lane_of.iter().find(|(id, _)| id == p).map(|(_, l)| *l));
        let sibling_before = c.parent.is_some() && cps[..i].iter().any(|o| o.parent == c.parent);
        let lane = match parent_lane {
            Some(l) if !sibling_before => l,
            _ if i == 0 => {
                next_lane = 1;
                0
            }
            _ => {
                let l = next_lane;
                next_lane += 1;
                l
            }
        };
        lane_of.push((c.id.clone(), lane));
        let on_path = path.contains(&c.id);
        let current = head.as_deref() == Some(c.id.as_str());
        let status = match c.status {
            CheckpointStatus::Running => "running",
            CheckpointStatus::Done => "done",
            CheckpointStatus::Failed => "failed",
            CheckpointStatus::Stopped => "stopped",
        };
        out.push(CheckpointRow {
            index: i,
            lane,
            branch_from: parent_lane.filter(|p| *p != lane),
            on_path,
            current,
            time: format_at(c.at),
            aria: format!(
                "{}, attempt {}, {status}{}",
                c.title,
                lane + 1,
                if current {
                    ", current"
                } else if on_path {
                    ""
                } else {
                    ", left behind"
                }
            ),
        });
    }
    out
}

/// What a restore brings back.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreAction {
    /// Files only.
    Files,
    /// Chat only.
    Chat,
    /// Both.
    Both,
    /// New thread from here; nothing here changes.
    Fork,
}

impl RestoreAction {
    /// Menu order and shortcut letters.
    pub const ALL: [(RestoreAction, char); 4] = [
        (Self::Files, 'R'),
        (Self::Chat, 'C'),
        (Self::Both, 'B'),
        (Self::Fork, 'F'),
    ];

    /// Menu label.
    pub fn menu(self) -> &'static str {
        match self {
            Self::Files => "Restore files",
            Self::Chat => "Rewind chat",
            Self::Both => "Restore files and chat",
            Self::Fork => "Fork from here",
        }
    }

    /// Confirm label with counts: labels say what moves.
    pub fn confirm(self, files: u32, turns: u32) -> String {
        let f = format!("{files} {}", if files == 1 { "file" } else { "files" });
        let t = format!("{turns} {}", if turns == 1 { "turn" } else { "turns" });
        match self {
            Self::Files => format!("Restore {f}"),
            Self::Chat => format!("Rewind {t}"),
            Self::Both => format!("Restore {f} and rewind {t}"),
            Self::Fork => "Fork from here".into(),
        }
    }

    /// Offered for this checkpoint (chat-only markers have no file actions).
    pub fn offered(self, c: &Checkpoint) -> bool {
        !(c.chat_only && matches!(self, Self::Files | Self::Both))
    }
}

/// Undo window after a restore (10 s, proposed in the spec).
pub const UNDO_WINDOW: Duration = Duration::from_secs(10);

/// A restore in flight or done.
#[derive(Clone, Debug, PartialEq)]
pub enum RestoreFlow {
    /// Nothing.
    Idle,
    /// Writing files (`done` of `total`).
    Restoring {
        /// Files written.
        done: u32,
        /// Files to write.
        total: u32,
    },
    /// Done; undo available until `until`.
    Restored {
        /// Toast text.
        label: String,
        /// Undo deadline.
        until: ClockMs,
    },
    /// Some files failed.
    Partial {
        /// Toast text.
        label: String,
    },
    /// Nothing changed.
    Failed {
        /// Reason.
        reason: String,
    },
    /// Undone.
    Undone,
}

impl RestoreFlow {
    /// The toast at `now`: text, action, and the undo hairline left (`0..1`).
    pub fn toast(&self, now: ClockMs) -> Option<(String, Option<&'static str>, f32)> {
        match self {
            Self::Idle => None,
            Self::Restoring { done, total } => {
                Some((format!("Restoring {done} of {total} files"), None, 1.0))
            }
            Self::Restored { label, until } => {
                let left = until.0.saturating_sub(now.0) as f32 / UNDO_WINDOW.as_millis() as f32;
                (left > 0.0).then(|| (label.clone(), Some("Undo"), left.clamp(0.0, 1.0)))
            }
            Self::Partial { label } => Some((label.clone(), Some("Retry failed"), 1.0)),
            Self::Failed { reason } => Some((
                format!("Couldn't restore · nothing changed · {reason}"),
                Some("Retry"),
                1.0,
            )),
            Self::Undone => Some(("Restore undone".into(), None, 0.0)),
        }
    }
}

/// Header: "Checkpoints · 5 · 2 attempts · $0.42".
pub fn header(cps: &[Checkpoint]) -> String {
    let attempts = lanes(cps, None)
        .iter()
        .map(|r| r.lane)
        .max()
        .map_or(0, |m| m + 1);
    let mut s = format!("{} checkpoints", cps.len());
    if attempts > 1 {
        s.push_str(&format!(" · {attempts} attempts"));
    }
    if let Some(c) = cps.iter().filter_map(|c| c.cost_cents).max() {
        s.push_str(&format!(" · ${}.{:02}", c / 100, c % 100));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run() -> Vec<Checkpoint> {
        vec![
            Checkpoint::new("c1", None, "Read the brief", 9),
            Checkpoint::new("c2", Some("c1"), "Built the pricing table", 64).files(3, 120, 4),
            Checkpoint::new("c3", Some("c2"), "Tests failing – 4 of 12", 131)
                .with(CheckpointStatus::Failed),
            Checkpoint::new("c4", Some("c2"), "Switched to Stripe Elements", 169)
                .with(CheckpointStatus::Running),
        ]
    }

    #[test]
    fn lanes_and_path() {
        let r = lanes(&run(), Some("c4"));
        assert_eq!(
            r.iter().map(|x| x.lane).collect::<Vec<_>>(),
            vec![0, 0, 0, 1]
        );
        assert_eq!(r[3].branch_from, Some(0));
        assert!(!r[2].on_path && r[3].current && r[1].on_path);
        assert_eq!(
            r[2].aria,
            "Tests failing – 4 of 12, attempt 1, failed, left behind"
        );
        assert_eq!(r[3].time, "2:49");
        assert_eq!(header(&run()), "4 checkpoints · 2 attempts");
    }

    #[test]
    fn labels_and_undo() {
        assert_eq!(
            RestoreAction::Both.confirm(3, 2),
            "Restore 3 files and rewind 2 turns"
        );
        let mut c = Checkpoint::new("x", None, "x", 0);
        c.chat_only = true;
        assert!(!RestoreAction::Files.offered(&c) && RestoreAction::Chat.offered(&c));
        let f = RestoreFlow::Restored {
            label: "Restored 3 files to 1:04".into(),
            until: ClockMs(10_000),
        };
        assert_eq!(f.toast(ClockMs(5_000)).unwrap().2, 0.5);
        assert!(f.toast(ClockMs(10_000)).is_none());
    }
}
