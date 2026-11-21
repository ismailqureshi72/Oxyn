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
    /// Raw name of the tool that made it (`Edit`, `Bash`, `mcp__github__create_pr`), for the
    /// per-checkpoint icon.
    pub tool: Option<String>,
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
            tool: None,
        }
    }

    /// Builder: the tool that made it.
    pub fn tool(mut self, name: &str) -> Self {
        self.tool = Some(name.into());
        self
    }

    /// Builder: run cost so far, cents.
    pub fn cost(mut self, cents: u32) -> Self {
        self.cost_cents = Some(cents);
        self
    }

    /// Icon name for the tool that made it, through the standard tool-kind map (`edit`,
    /// `prompt`, `search`, …), or `None` when no tool is recorded.
    pub fn tool_icon(&self) -> Option<&'static str> {
        self.tool
            .as_deref()
            .map(|t| crate::kit::tools::ToolKindMap::standard().kind(t).icon())
    }

    /// The app behind the tool, when it is one Oxyn has a logo for (`slack`, `github`,
    /// `notion`, `figma`, `stripe`, `linear`, `vercel`): matched in the tool's name, so
    /// `mcp__github__create_pull_request` is `github`.
    pub fn tool_brand(&self) -> Option<&'static str> {
        let t = self.tool.as_deref()?.to_ascii_lowercase();
        BRANDS.iter().copied().find(|b| t.contains(b))
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

/// Apps with a logo for the tool tile ([`Checkpoint::tool_brand`]).
pub const BRANDS: [&str; 7] = [
    "slack", "github", "notion", "figma", "stripe", "linear", "vercel",
];

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
    /// The lane line above this row's dot (from its parent in the same lane, or the fork curve
    /// that started this lane); `Some(on_path)` when drawn. `None` on the first checkpoint, so
    /// nothing sticks up above a root.
    pub up: Option<bool>,
    /// The lane line below this row's dot (towards its next checkpoint in the same lane);
    /// `None` at the end of a lane, so a failed attempt simply stops.
    pub down: Option<bool>,
    /// Lanes that pass this row on their way from a fork to their first checkpoint, or between
    /// two checkpoints of the same lane: `(lane, on_path)`.
    pub through: Vec<(usize, bool)>,
    /// Lanes that fork from this row's dot (a new attempt starting from this checkpoint):
    /// `(lane, on_path)`. Draw a curve from this dot down into the lane.
    pub forks: Vec<(usize, bool)>,
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

/// Lays checkpoints out in lanes, git-graph style. A checkpoint starts a new lane (a new attempt)
/// when its parent already has an earlier child, or when other checkpoints came between it and
/// its parent (a rewind, even to a leaf); everything off the head's ancestry is faded.
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
        // A rewind: the parent is not the checkpoint just before this one.
        let rewound = i > 0 && c.parent.as_ref().is_some_and(|p| &cps[i - 1].id != p);
        let sibling_before = sibling_before || rewound;
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
            up: None,
            down: None,
            through: vec![],
            forks: vec![],
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
    // Edges: every checkpoint joins its parent, in the parent's lane (a straight segment) or
    // from the parent's dot into a new lane (a fork curve), passing the rows in between.
    for i in 0..out.len() {
        let Some(p) = cps[i]
            .parent
            .as_ref()
            .and_then(|pid| cps[..i].iter().position(|o| &o.id == pid))
        else {
            continue;
        };
        let lane = out[i].lane;
        let on = out[i].on_path;
        if out[p].lane == lane {
            out[p].down = Some(on);
        } else {
            out[p].forks.push((lane, on));
        }
        for row in out.iter_mut().take(i).skip(p + 1) {
            row.through.push((lane, on));
        }
        out[i].up = Some(on);
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

/// Default cost formatter: `$0.42`, `$12.30`, `$1.2k`, and `<$0.01` for a spend under a cent
/// that rounded to zero.
pub fn format_cost(cents: u32) -> String {
    match cents {
        0 => "<$0.01".into(),
        c if c >= 100_000 => {
            let k = c as f64 / 100_000.0;
            if k >= 10.0 {
                format!("${k:.0}k")
            } else {
                format!("${k:.1}k")
            }
        }
        c => format!("${}.{:02}", c / 100, c % 100),
    }
}

/// What a checkpoint's own step cost: its running cost minus its parent's (cents), when both
/// are known.
pub fn step_cost(cps: &[Checkpoint], i: usize) -> Option<u32> {
    let c = cps.get(i)?;
    let own = c.cost_cents?;
    let parent = c
        .parent
        .as_deref()
        .and_then(|p| cps.iter().find(|x| x.id == p))
        .and_then(|p| p.cost_cents)
        .unwrap_or(0);
    Some(own.saturating_sub(parent))
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
        s.push_str(&format!(" · {}", format_cost(c)));
    }
    s
}

// ── Interactive models: keys, rewind, branch tree, film strip, undo ladder, restore preview ──

/// A key the checkpoint views act on, the same for both renderers.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CpKey {
    /// Previous row (↑).
    Up,
    /// Next row (↓).
    Down,
    /// Previous frame, or fold a branch (←).
    Left,
    /// Next frame, or unfold a branch (→).
    Right,
    /// First.
    Home,
    /// Last.
    End,
    /// Act on the focused item (Enter / Space).
    Enter,
    /// Close or cancel (Esc).
    Escape,
    /// Undo (⌘Z / Ctrl+Z).
    Undo,
    /// Redo (⇧⌘Z / Ctrl+Y).
    Redo,
}

impl CpKey {
    /// From a DOM / GPUI key name (`ArrowUp`, `up`, `Home`, `enter`, ` `, `Escape`, …).
    pub fn from_name(k: &str) -> Option<Self> {
        Some(match k {
            "ArrowUp" | "up" => Self::Up,
            "ArrowDown" | "down" => Self::Down,
            "ArrowLeft" | "left" => Self::Left,
            "ArrowRight" | "right" => Self::Right,
            "Home" | "home" => Self::Home,
            "End" | "end" => Self::End,
            "Enter" | "enter" | " " | "space" => Self::Enter,
            "Escape" | "escape" => Self::Escape,
            _ => return None,
        })
    }
}

/// Roving focus over `len` items: ↑ / ← back, ↓ / → on, Home / End to the ends (clamped, no wrap).
pub fn roving(len: usize, at: usize, key: CpKey) -> usize {
    if len == 0 {
        return 0;
    }
    let last = len - 1;
    match key {
        CpKey::Up | CpKey::Left => at.min(last).saturating_sub(1),
        CpKey::Down | CpKey::Right => (at + 1).min(last),
        CpKey::Home => 0,
        CpKey::End => last,
        _ => at.min(last),
    }
}

/// Absolute ISO timestamp of a checkpoint: the run's start (`YYYY-MM-DD`, `HH:MM`) plus `at`,
/// as `2026-11-14 15:31:09`. Rolls over midnight into the next day.
pub fn iso_at(start_day: &str, start_hm: (u32, u32), at: Duration) -> String {
    let day = crate::primitives::date::Date::parse_iso(start_day)
        .unwrap_or_else(|| crate::primitives::date::Date::clamped(2026, 1, 1));
    let secs = u64::from(start_hm.0) * 3_600 + u64::from(start_hm.1) * 60 + at.as_secs();
    let d = day.add_days((secs / 86_400) as i64);
    let s = secs % 86_400;
    format!(
        "{} {:02}:{:02}:{:02}",
        d.iso(),
        s / 3_600,
        s / 60 % 60,
        s % 60
    )
}

/// A run you can rewind: the checkpoints, the head and the focused row. Rewinding to a
/// checkpoint stops whatever is running and starts a new attempt from it; nothing is removed.
#[derive(Clone, Debug, PartialEq)]
pub struct CheckpointRun {
    /// Checkpoints in the order taken.
    pub checkpoints: Vec<Checkpoint>,
    /// Where the agent stands (`None` = the last one).
    pub head: Option<String>,
    /// Focused row (roving tab stop).
    pub focus: usize,
    rewinds: u32,
}

impl CheckpointRun {
    /// A run at `head`, focused on the head.
    pub fn new(checkpoints: Vec<Checkpoint>, head: Option<String>) -> Self {
        let mut r = Self {
            checkpoints,
            head,
            focus: 0,
            rewinds: 0,
        };
        r.focus = r.head_index();
        r
    }

    /// Id of the head.
    pub fn head_id(&self) -> String {
        self.head
            .clone()
            .or_else(|| self.checkpoints.last().map(|c| c.id.clone()))
            .unwrap_or_default()
    }

    /// Index of the head.
    pub fn head_index(&self) -> usize {
        let h = self.head_id();
        self.checkpoints.iter().position(|c| c.id == h).unwrap_or(0)
    }

    /// Can `i` be rewound to (not the head, not still running)?
    pub fn can_rewind(&self, i: usize) -> bool {
        self.checkpoints
            .get(i)
            .is_some_and(|c| c.status != CheckpointStatus::Running && i != self.head_index())
    }

    /// Rewinds to `i`: a running checkpoint becomes `Stopped`, a new running attempt starts from
    /// `i` and becomes the head. Returns the new checkpoint's id, or `None` when `i` can't be
    /// rewound to.
    pub fn rewind(&mut self, i: usize) -> Option<String> {
        let t = self
            .checkpoints
            .get(i)
            .map(|c| format!("Retry from {}", c.title))
            .unwrap_or_default();
        self.rewind_with(i, &t)
    }

    /// [`Self::rewind`] with the new attempt's first step titled `title` (what the agent does
    /// next, `Switched to Stripe Elements`).
    pub fn rewind_with(&mut self, i: usize, title: &str) -> Option<String> {
        if !self.can_rewind(i) {
            return None;
        }
        for c in &mut self.checkpoints {
            if c.status == CheckpointStatus::Running {
                c.status = CheckpointStatus::Stopped;
            }
        }
        self.rewinds += 1;
        let from = self.checkpoints[i].clone();
        let at = self
            .checkpoints
            .iter()
            .map(|c| c.at)
            .max()
            .unwrap_or_default()
            + Duration::from_secs(14);
        let id = format!("r{}", self.rewinds);
        let mut n = Checkpoint::new(&id, Some(&from.id), title, 0).with(CheckpointStatus::Running);
        n.at = at;
        n.tool = Some("Edit".into());
        n.cost_cents = from.cost_cents;
        self.checkpoints.push(n);
        self.head = Some(id.clone());
        self.focus = self.checkpoints.len() - 1;
        Some(id)
    }

    /// Lane rows for the current state.
    pub fn rows(&self) -> Vec<CheckpointRow> {
        lanes(&self.checkpoints, self.head.as_deref())
    }

    /// One key: arrows / Home / End move focus, Enter rewinds to the focused row. Returns the
    /// new head id when a rewind happened.
    pub fn key(&mut self, k: CpKey) -> Option<String> {
        match k {
            CpKey::Enter => self.rewind(self.focus),
            _ => {
                self.focus = roving(self.checkpoints.len(), self.focus, k);
                None
            }
        }
    }
}

/// One row of the branch tree: a checkpoint indented under the one it follows, with tree guides.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TreeRow {
    /// Index into the checkpoints.
    pub index: usize,
    /// Depth in forks (0 = the first attempt; each fork indents one level).
    pub depth: usize,
    /// Starts a fork (its parent already had a child).
    pub fork: bool,
    /// Last child of its parent (draw `└` instead of `├`).
    pub last: bool,
    /// Checkpoints below it in its own subtree (shown as "+N" when folded).
    pub below: usize,
    /// On the way to the head.
    pub on_path: bool,
    /// The head.
    pub current: bool,
    /// Hidden because an ancestor fork is folded.
    pub hidden: bool,
}

/// Lays checkpoints out as a tree of forks: the first child continues the same line, every
/// later child of the same parent opens a fork one level deeper. `folded` holds ids whose
/// subtrees are collapsed.
pub fn branch_tree(cps: &[Checkpoint], head: Option<&str>, folded: &[String]) -> Vec<TreeRow> {
    struct Walk<'a> {
        cps: &'a [Checkpoint],
        rows: Vec<CheckpointRow>,
        folded: &'a [String],
        out: Vec<TreeRow>,
    }
    impl Walk<'_> {
        fn kids(&self, id: &str) -> Vec<usize> {
            (0..self.cps.len())
                .filter(|&i| self.cps[i].parent.as_deref() == Some(id))
                .collect()
        }
        // `(depth, fork, last, hidden)`; returns how many rows its subtree adds below it.
        fn go(&mut self, i: usize, at: (usize, bool, bool, bool)) -> usize {
            let (depth, fork, last, hidden) = at;
            let me = self.out.len();
            self.out.push(TreeRow {
                index: i,
                depth,
                fork,
                last,
                below: 0,
                on_path: self.rows[i].on_path,
                current: self.rows[i].current,
                hidden,
            });
            let ks = self.kids(&self.cps[i].id);
            let hide = hidden || self.folded.contains(&self.cps[i].id);
            let mut n = 0;
            // Later children are forks, one level deeper; the first child continues the line
            // and is drawn after them so the main line reads straight down.
            for (j, &k) in ks.iter().enumerate().skip(1) {
                n += 1 + self.go(k, (depth + 1, true, j + 1 == ks.len(), hide));
            }
            if let Some(&k) = ks.first() {
                n += 1 + self.go(k, (depth, false, true, hide));
            }
            self.out[me].below = n;
            n
        }
    }
    let mut w = Walk {
        cps,
        rows: lanes(cps, head),
        folded,
        out: vec![],
    };
    for (i, c) in cps.iter().enumerate() {
        if c.parent.is_none() {
            w.go(i, (0, false, true, false));
        }
    }
    w.out
}

/// Number of checkpoints a fork row hides when folded, counting forks below it.
pub fn forks_of(cps: &[Checkpoint], id: &str) -> usize {
    cps.iter()
        .filter(|c| c.parent.as_deref() == Some(id))
        .count()
        .saturating_sub(1)
}

/// The film strip: one frame per checkpoint with a scrub position.
#[derive(Clone, Debug, PartialEq)]
pub struct FilmStrip {
    /// Frames (checkpoints in the order taken).
    pub len: usize,
    /// Selected frame.
    pub at: usize,
}

impl FilmStrip {
    /// A strip of `len` frames on frame `at`.
    pub fn new(len: usize, at: usize) -> Self {
        Self {
            len,
            at: at.min(len.saturating_sub(1)),
        }
    }

    /// Scrubs to a fraction `0..=1` of the track (pointer x over the track width).
    pub fn scrub(&mut self, frac: f32) -> usize {
        if self.len > 0 {
            let f = frac.clamp(0.0, 1.0);
            self.at = ((f * (self.len - 1) as f32).round() as usize).min(self.len - 1);
        }
        self.at
    }

    /// Playhead position `0..=1` of the selected frame.
    pub fn playhead(&self) -> f32 {
        if self.len < 2 {
            0.0
        } else {
            self.at as f32 / (self.len - 1) as f32
        }
    }

    /// ← / → / Home / End.
    pub fn key(&mut self, k: CpKey) -> usize {
        self.at = roving(self.len, self.at, k);
        self.at
    }
}

/// A deterministic thumbnail sketch for a frame: `rows` bars of widths `0..=100` (percent), so a
/// checkpoint always draws the same miniature without an image.
pub fn thumb_bars(c: &Checkpoint, rows: usize) -> Vec<(u8, bool)> {
    let mut h: u32 = 2_166_136_261;
    for b in c.id.bytes().chain(c.title.bytes()) {
        h = (h ^ u32::from(b)).wrapping_mul(16_777_619);
    }
    let changed = c.files.min(rows as u32) as usize;
    (0..rows)
        .map(|i| {
            h = h.wrapping_mul(1_103_515_245).wrapping_add(12_345);
            let w = 30 + (h >> 16) % 66;
            (w as u8, i < changed)
        })
        .collect()
}

/// One reversible step on the undo ladder.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndoStep {
    /// What the step did ("Renamed Plan to Tier").
    pub title: String,
    /// What it touched ("2 files", "1 migration").
    pub detail: String,
    /// Can be undone; a step that can't (a sent email, a deploy) stops the ladder there.
    pub reversible: bool,
}

impl UndoStep {
    /// A reversible step.
    pub fn new(title: &str, detail: &str) -> Self {
        Self {
            title: title.into(),
            detail: detail.into(),
            reversible: true,
        }
    }

    /// Builder: can't be undone.
    pub fn fixed(mut self) -> Self {
        self.reversible = false;
        self
    }
}

/// A vertical stepper of steps you can walk back and forward: `applied` steps are in effect,
/// the rest are undone (and can be redone until a new step is pushed).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UndoLadder {
    /// Steps, oldest first.
    pub steps: Vec<UndoStep>,
    /// How many are in effect.
    pub applied: usize,
    /// Focused rung.
    pub focus: usize,
}

impl UndoLadder {
    /// All steps applied, focus on the newest.
    pub fn new(steps: Vec<UndoStep>) -> Self {
        let n = steps.len();
        Self {
            steps,
            applied: n,
            focus: n.saturating_sub(1),
        }
    }

    /// Lowest `applied` reachable: everything after the newest step that can't be undone.
    pub fn floor(&self) -> usize {
        self.steps
            .iter()
            .rposition(|s| !s.reversible)
            .map_or(0, |i| i + 1)
    }

    /// Undo one step; `false` at the floor.
    pub fn undo(&mut self) -> bool {
        if self.applied > self.floor() {
            self.applied -= 1;
            self.focus = self.applied.saturating_sub(1);
            true
        } else {
            false
        }
    }

    /// Redo one step; `false` when everything is applied.
    pub fn redo(&mut self) -> bool {
        if self.applied < self.steps.len() {
            self.applied += 1;
            self.focus = self.applied - 1;
            true
        } else {
            false
        }
    }

    /// Jump so that steps `0..=i` are in effect (clamped to the floor). Returns the steps
    /// undone (negative) or redone (positive).
    pub fn jump(&mut self, i: usize) -> i32 {
        let target = (i + 1).min(self.steps.len()).max(self.floor());
        let d = target as i32 - self.applied as i32;
        self.applied = target;
        self.focus = i.min(self.steps.len().saturating_sub(1));
        d
    }

    /// State of rung `i`: `"applied"`, `"undone"` or `"locked"` (applied and below the floor).
    pub fn state(&self, i: usize) -> &'static str {
        if i >= self.applied {
            "undone"
        } else if i < self.floor() {
            "locked"
        } else {
            "applied"
        }
    }

    /// One key: ↑ / ↓ / Home / End move focus, Enter jumps there, Undo / Redo step.
    pub fn key(&mut self, k: CpKey) {
        match k {
            CpKey::Enter => {
                self.jump(self.focus);
            }
            CpKey::Undo => {
                self.undo();
            }
            CpKey::Redo => {
                self.redo();
            }
            _ => self.focus = roving(self.steps.len(), self.focus, k),
        }
    }

    /// Status line: "4 of 6 applied · 2 undone".
    pub fn summary(&self) -> String {
        let undone = self.steps.len() - self.applied;
        let mut s = format!("{} of {} applied", self.applied, self.steps.len());
        if undone > 0 {
            s.push_str(&format!(" · {undone} undone"));
        }
        s
    }
}

/// How a file changes when a checkpoint is restored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FileChange {
    /// Its content goes back.
    Modified,
    /// It comes back (was deleted since).
    Restored,
    /// It goes away (was created since).
    Removed,
}

impl FileChange {
    /// Short tag (`M`, `A`, `D`), never colour alone.
    pub fn tag(self) -> &'static str {
        match self {
            Self::Modified => "M",
            Self::Restored => "A",
            Self::Removed => "D",
        }
    }
}

/// One file in the restore preview.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PreviewFile {
    /// Path.
    pub path: String,
    /// Change.
    pub change: FileChange,
    /// Lines added and removed by the restore.
    pub lines: (u32, u32),
}

/// Where the restore preview sheet is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PreviewPhase {
    /// Reading the diff summary.
    Review,
    /// Asked to confirm ("Restore 3 files?").
    Confirm,
    /// Restored.
    Done,
    /// Closed without restoring.
    Cancelled,
}

/// The restore preview sheet: what restoring a checkpoint will change, then Restore → confirm.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RestorePreview {
    /// Files that change.
    pub files: Vec<PreviewFile>,
    /// Turns of the conversation that are rewound.
    pub turns: u32,
    /// What is restored.
    pub action: RestoreAction,
    /// Phase.
    pub phase: PreviewPhase,
}

impl RestorePreview {
    /// A sheet in review.
    pub fn new(files: Vec<PreviewFile>, turns: u32, action: RestoreAction) -> Self {
        Self {
            files,
            turns,
            action,
            phase: PreviewPhase::Review,
        }
    }

    /// Totals `(+, −)`.
    pub fn totals(&self) -> (u32, u32) {
        self.files
            .iter()
            .fold((0, 0), |(a, d), f| (a + f.lines.0, d + f.lines.1))
    }

    /// Summary line: "3 files · +42 −17 · 2 turns".
    pub fn summary(&self) -> String {
        let (a, d) = self.totals();
        let n = self.files.len();
        let mut s = String::new();
        if !matches!(self.action, RestoreAction::Chat) {
            s = format!(
                "{n} {} · +{a} \u{2212}{d}",
                if n == 1 { "file" } else { "files" }
            );
        }
        if !matches!(self.action, RestoreAction::Files) && self.turns > 0 {
            if !s.is_empty() {
                s.push_str(" · ");
            }
            s.push_str(&format!(
                "{} {}",
                self.turns,
                if self.turns == 1 { "turn" } else { "turns" }
            ));
        }
        s
    }

    /// Primary button label for the phase.
    pub fn primary(&self) -> String {
        match self.phase {
            PreviewPhase::Review => self.action.confirm(self.files.len() as u32, self.turns),
            PreviewPhase::Confirm => "Yes, restore".into(),
            PreviewPhase::Done | PreviewPhase::Cancelled => "Close".into(),
        }
    }

    /// Primary pressed: Review → Confirm → Done.
    pub fn press(&mut self) {
        self.phase = match self.phase {
            PreviewPhase::Review => PreviewPhase::Confirm,
            PreviewPhase::Confirm => PreviewPhase::Done,
            p => p,
        };
    }

    /// Cancel / Esc: Confirm steps back to Review, Review closes.
    pub fn cancel(&mut self) {
        self.phase = match self.phase {
            PreviewPhase::Confirm => PreviewPhase::Review,
            PreviewPhase::Review => PreviewPhase::Cancelled,
            p => p,
        };
    }

    /// Enter presses, Esc cancels.
    pub fn key(&mut self, k: CpKey) {
        match k {
            CpKey::Enter => self.press(),
            CpKey::Escape => self.cancel(),
            _ => {}
        }
    }
}

// ── Lane rail geometry (C1b, measured against affectUI in C1c): one drawing for the whole
// list, shared by both renderers ──

/// Height of a lane row, in px.
pub const RAIL_ROW_H: f32 = 30.0;
/// X of the first lane's centre, in px (inside the card's border).
pub const RAIL_X0: f32 = 21.0;
/// Distance between two lanes, in px.
pub const RAIL_LANE_W: f32 = 14.0;
/// Height of a fork's bend: the new lane is vertical again this far below the parent dot.
pub const RAIL_BEND_H: f32 = 22.0;
/// Space between the last lane and the row content, in px.
pub const RAIL_GAP: f32 = 15.0;
/// Radius of a dot on the way to the head.
pub const RAIL_DOT_R: f32 = 3.0;
/// Radius of the head's dot.
pub const RAIL_HEAD_R: f32 = 4.0;
/// Outer radius of a left-behind ring (stroke included).
pub const RAIL_RING_R: f32 = 4.25;
/// Width of a rail line and of a ring's stroke.
pub const RAIL_LINE_W: f32 = 1.5;
/// Gap in the card colour around every dot, so no line touches a dot.
pub const RAIL_KNOCKOUT: f32 = 1.3;

/// One stroke of the rail: a straight run in one lane, or a fork that bends out of the parent's
/// dot into a new lane and then runs straight down to the attempt's first checkpoint.
#[derive(Clone, Debug, PartialEq)]
pub struct RailEdge {
    /// The checkpoint this edge leads to (stable key for animation).
    pub child: usize,
    /// Parent dot.
    pub from: (f32, f32),
    /// Child dot.
    pub to: (f32, f32),
    /// Bends into another lane.
    pub fork: bool,
    /// On the way to the head.
    pub on_path: bool,
}

impl RailEdge {
    /// The bend ends [`RAIL_BEND_H`] below the parent (or at the child, when sooner).
    pub fn bend_end(&self) -> f32 {
        (self.from.1 + RAIL_BEND_H).min(self.to.1)
    }

    /// Length of the stroke in px (the bend is measured as a polyline), for a draw that runs at
    /// a constant speed.
    pub fn len(&self) -> f32 {
        let (x1, y1) = self.from;
        let (x2, y2) = self.to;
        if !self.fork {
            return ((x2 - x1).powi(2) + (y2 - y1).powi(2)).sqrt();
        }
        let e = self.bend_end();
        let h = e - y1;
        let p = |t: f32| {
            let u = 1.0 - t;
            let x = u * u * u * x1 + 3.0 * u * u * t * x1 + 3.0 * u * t * t * x2 + t * t * t * x2;
            let y = u * u * u * y1
                + 3.0 * u * u * t * (y1 + h * 12.0 / 22.0)
                + 3.0 * u * t * t * (y1 + h * 10.0 / 22.0)
                + t * t * t * e;
            (x, y)
        };
        let mut l = 0.0;
        let mut a = p(0.0);
        for k in 1..=16 {
            let b = p(k as f32 / 16.0);
            l += ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
            a = b;
        }
        l + (y2 - e).max(0.0)
    }

    /// SVG path data: `M x y L x y` or the fork curve plus its straight tail.
    pub fn svg_d(&self) -> String {
        let (x1, y1) = self.from;
        let (x2, y2) = self.to;
        if !self.fork {
            return format!("M{x1} {y1} L{x2} {y2}");
        }
        let e = self.bend_end();
        let h = e - y1;
        // A symmetric S, traced from affectUI: controls at 12/22 and 10/22 of the bend.
        let mut d = format!(
            "M{x1} {y1} C{x1} {} {x2} {} {x2} {e}",
            y1 + h * 12.0 / 22.0,
            y1 + h * 10.0 / 22.0
        );
        if y2 > e {
            d.push_str(&format!(" L{x2} {y2}"));
        }
        d
    }
}

/// One dot of the rail.
#[derive(Clone, Debug, PartialEq)]
pub struct RailDot {
    /// Index into the checkpoints.
    pub index: usize,
    /// Centre.
    pub at: (f32, f32),
    /// Radius: [`RAIL_DOT_R`], the head [`RAIL_HEAD_R`], a left-behind ring [`RAIL_RING_R`]
    /// (outer edge, stroke included).
    pub r: f32,
    /// On the way to the head (filled), else a hollow ring.
    pub on_path: bool,
    /// The head.
    pub current: bool,
    /// Status of the checkpoint.
    pub status: CheckpointStatus,
}

/// The whole rail of a lane list: its size, edges (left-behind first, so the live path draws on
/// top) and dots.
#[derive(Clone, Debug, PartialEq)]
pub struct Rail {
    /// Gutter width (rows pad their content by this much).
    pub width: f32,
    /// Height of all rows.
    pub height: f32,
    /// Strokes.
    pub edges: Vec<RailEdge>,
    /// Dots, in row order.
    pub dots: Vec<RailDot>,
}

/// Centre of `lane` at row `i`.
pub fn rail_point(lane: usize, i: usize) -> (f32, f32) {
    (
        RAIL_X0 + RAIL_LANE_W * lane as f32,
        RAIL_ROW_H * i as f32 + RAIL_ROW_H / 2.0,
    )
}

/// Lays out the rail for `lanes(cps, head)`.
pub fn rail(cps: &[Checkpoint], rows: &[CheckpointRow]) -> Rail {
    let n_lanes = rows.iter().map(|r| r.lane).max().map_or(1, |m| m + 1);
    let mut edges = vec![];
    for (i, c) in cps.iter().enumerate() {
        let Some(p) = c
            .parent
            .as_ref()
            .and_then(|pid| cps[..i].iter().position(|o| &o.id == pid))
        else {
            continue;
        };
        let (Some(rp), Some(rc)) = (rows.get(p), rows.get(i)) else {
            continue;
        };
        edges.push(RailEdge {
            child: i,
            from: rail_point(rp.lane, p),
            to: rail_point(rc.lane, i),
            fork: rp.lane != rc.lane,
            on_path: rc.on_path,
        });
    }
    edges.sort_by_key(|e| e.on_path);
    let dots = rows
        .iter()
        .map(|r| RailDot {
            index: r.index,
            at: rail_point(r.lane, r.index),
            r: if r.current {
                RAIL_HEAD_R
            } else if r.on_path {
                RAIL_DOT_R
            } else {
                RAIL_RING_R
            },
            on_path: r.on_path,
            current: r.current,
            status: cps
                .get(r.index)
                .map_or(CheckpointStatus::Done, |c| c.status),
        })
        .collect();
    Rail {
        width: RAIL_X0 + RAIL_LANE_W * (n_lanes - 1) as f32 + RAIL_GAP,
        height: RAIL_ROW_H * rows.len() as f32,
        edges,
        dots,
    }
}

/// Splits an absolute label `2026-11-14 15:31:09` into its day and its time of day; any other
/// label is all time.
pub fn split_day(label: &str) -> (Option<&str>, &str) {
    match label.split_once(' ') {
        Some((d, t)) if d.len() == 10 && d.as_bytes()[4] == b'-' && d.as_bytes()[7] == b'-' => {
            (Some(d), t)
        }
        _ => (None, label),
    }
}

/// The day every label shares (shown once in the bar), when they all share one.
pub fn shared_day(labels: &[String]) -> Option<&str> {
    let mut it = labels.iter().map(|l| split_day(l).0);
    let first = it.next()??;
    it.all(|d| d == Some(first)).then_some(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_icons_and_costs() {
        assert_eq!(
            Checkpoint::new("g", None, "PR", 1)
                .tool("mcp__github__create_pull_request")
                .tool_brand(),
            Some("github")
        );
        assert_eq!(
            Checkpoint::new("r", None, "R", 1).tool("Read").tool_brand(),
            None
        );
        let a = Checkpoint::new("a", None, "Read", 1).tool("Read").cost(12);
        let b = Checkpoint::new("b", Some("a"), "Edit", 2)
            .tool("apply_patch")
            .cost(42);
        let n = Checkpoint::new("n", Some("b"), "Chat", 3);
        assert_eq!(a.tool_icon(), Some(crate::run::ToolKind::Read.icon()));
        assert_eq!(b.tool_icon(), Some(crate::run::ToolKind::Edit.icon()));
        assert_eq!(n.tool_icon(), None);
        let cps = vec![a, b, n];
        assert_eq!(step_cost(&cps, 0), Some(12));
        assert_eq!(step_cost(&cps, 1), Some(30));
        assert_eq!(step_cost(&cps, 2), None);
        assert_eq!(format_cost(42), "$0.42");
        assert_eq!(format_cost(1230), "$12.30");
        assert_eq!(format_cost(0), "<$0.01");
        assert_eq!(format_cost(123_456), "$1.2k");
        assert_eq!(format_cost(2_500_000), "$25k");
        assert!(header(&cps).ends_with("$0.42"));
    }

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
        // The new attempt forks from c2's dot and runs past the failed c3; nothing above c1.
        assert_eq!(r[0].up, None);
        assert_eq!(r[1].forks, vec![(1, true)]);
        assert_eq!(r[1].down, Some(false));
        assert_eq!(r[2].through, vec![(1, true)]);
        assert_eq!(r[2].down, None);
        assert_eq!(r[3].up, Some(true));
        assert_eq!(r[3].down, None);
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

    #[test]
    fn c1a_models() {
        assert_eq!(roving(5, 2, CpKey::Up), 1);
        assert_eq!(roving(5, 4, CpKey::Down), 4);
        assert_eq!(roving(5, 2, CpKey::End), 4);
        assert_eq!(roving(0, 3, CpKey::Down), 0);
        assert_eq!(CpKey::from_name("ArrowDown"), Some(CpKey::Down));
        assert_eq!(
            iso_at("2026-11-14", (15, 31), Duration::from_secs(64)),
            "2026-11-14 15:32:04"
        );
        assert_eq!(
            iso_at("2026-11-14", (23, 59), Duration::from_secs(120)),
            "2026-11-15 00:01:00"
        );
        let mut r = CheckpointRun::new(run(), Some("c4".into()));
        assert_eq!(r.focus, 3);
        assert!(r.key(CpKey::Enter).is_none());
        r.key(CpKey::Home);
        r.key(CpKey::Down);
        assert_eq!(r.key(CpKey::Enter).as_deref(), Some("r1"));
        assert_eq!(r.checkpoints[3].status, CheckpointStatus::Stopped);
        assert_eq!(r.head_id(), "r1");
        assert_eq!(r.rows()[4].lane, 2);
        let t = branch_tree(&run(), Some("c4"), &[]);
        assert_eq!(
            t.iter().map(|x| (x.index, x.depth)).collect::<Vec<_>>(),
            vec![(0, 0), (1, 0), (3, 1), (2, 0)]
        );
        assert!(t[2].fork && t[1].below == 2);
        let t = branch_tree(&run(), None, &["c2".into()]);
        assert!(t[2].hidden && t[3].hidden && !t[1].hidden);
        let mut f = FilmStrip::new(6, 0);
        assert_eq!(f.scrub(0.5), 3);
        assert_eq!(f.key(CpKey::End), 5);
        assert_eq!(f.playhead(), 1.0);
        assert_eq!(thumb_bars(&run()[1], 5), thumb_bars(&run()[1], 5));
        let mut l = UndoLadder::new(vec![
            UndoStep::new("a", "1 file"),
            UndoStep::new("b", "sent").fixed(),
            UndoStep::new("c", "2 files"),
            UndoStep::new("d", "1 file"),
        ]);
        assert_eq!(l.floor(), 2);
        assert!(l.undo() && l.undo() && !l.undo());
        assert_eq!(l.summary(), "2 of 4 applied · 2 undone");
        assert_eq!(
            (l.state(0), l.state(1), l.state(3)),
            ("locked", "locked", "undone")
        );
        assert_eq!(l.jump(3), 2);
        assert_eq!(l.jump(0), -2);
        l.key(CpKey::Undo);
        assert_eq!(l.applied, 2);
        let mut p = RestorePreview::new(
            vec![
                PreviewFile {
                    path: "a.rs".into(),
                    change: FileChange::Modified,
                    lines: (3, 1),
                },
                PreviewFile {
                    path: "b.rs".into(),
                    change: FileChange::Removed,
                    lines: (0, 9),
                },
            ],
            2,
            RestoreAction::Both,
        );
        assert_eq!(p.summary(), "2 files · +3 \u{2212}10 · 2 turns");
        assert_eq!(p.primary(), "Restore 2 files and rewind 2 turns");
        p.key(CpKey::Enter);
        assert_eq!(p.phase, PreviewPhase::Confirm);
        p.key(CpKey::Escape);
        assert_eq!(p.phase, PreviewPhase::Review);
        p.press();
        p.press();
        assert_eq!(p.phase, PreviewPhase::Done);
    }

    #[test]
    fn c1c_fork_and_lane_rule() {
        // Rewinding to a leaf (the failed last step of attempt 1) opens attempt 2's lane too.
        let mut run = CheckpointRun::new(
            vec![
                Checkpoint::new("a", None, "A", 1),
                Checkpoint::new("b", Some("a"), "B", 2),
                Checkpoint::new("c", Some("b"), "C", 3).with(CheckpointStatus::Failed),
                Checkpoint::new("d", Some("b"), "D", 4),
            ],
            None,
        );
        let id = run.rewind_with(2, "Fixed the test").unwrap();
        let rows = run.rows();
        assert_eq!(
            rows.iter().map(|r| r.lane).collect::<Vec<_>>(),
            vec![0, 0, 0, 1, 2]
        );
        assert_eq!(rows[4].branch_from, Some(0));
        assert!(rows[4].current && rows[4].aria.contains("attempt 3"));
        assert_eq!(run.checkpoints[4].title, "Fixed the test");
        assert_eq!(run.checkpoints[4].id, id);
        assert_eq!(header(&run.checkpoints), "5 checkpoints · 3 attempts");
        // A step straight after its parent stays in the parent's lane.
        let rows = lanes(
            &[
                Checkpoint::new("a", None, "A", 1),
                Checkpoint::new("b", Some("a"), "B", 2),
            ],
            None,
        );
        assert_eq!(rows[1].lane, 0);
        // The fork bends in 22 px whatever the distance, then runs straight down.
        let e = RailEdge {
            child: 6,
            from: rail_point(0, 3),
            to: rail_point(1, 6),
            fork: true,
            on_path: true,
        };
        assert_eq!(e.bend_end(), e.from.1 + RAIL_BEND_H);
        assert_eq!(e.svg_d(), "M21 105 C21 117 35 115 35 127 L35 195");
        let mut r = CheckpointRun::new(vec![Checkpoint::new("a", None, "A", 1)], None);
        assert_eq!(r.rewind(0), None);
        r.checkpoints.push(Checkpoint::new("b", Some("a"), "B", 2));
        r.rewind(0);
        assert_eq!(r.checkpoints[2].title, "Retry from A");
    }

    #[test]
    fn c1b_rail_geometry() {
        let cps = vec![
            Checkpoint::new("a", None, "A", 1),
            Checkpoint::new("b", Some("a"), "B", 2),
            Checkpoint::new("c", Some("b"), "C", 3).with(CheckpointStatus::Failed),
            Checkpoint::new("d", Some("b"), "D", 4).with(CheckpointStatus::Running),
        ];
        let rows = lanes(&cps, None);
        let r = rail(&cps, &rows);
        assert_eq!(r.height, 4.0 * RAIL_ROW_H);
        assert_eq!(r.width, RAIL_X0 + RAIL_LANE_W + RAIL_GAP);
        assert_eq!(r.width, 50.0);
        assert_eq!(r.edges.len(), 3);
        // left-behind first, so the live path paints over it
        assert!(!r.edges[0].on_path && r.edges[0].child == 2);
        let fork = r.edges.iter().find(|e| e.child == 3).unwrap();
        assert!(fork.fork && fork.on_path);
        assert_eq!(fork.from, rail_point(0, 1));
        assert_eq!(fork.to, rail_point(1, 3));
        assert_eq!(fork.svg_d(), "M21 45 C21 57 35 55 35 67 L35 105");
        assert!((fork.len() - 64.0).abs() < 2.0, "{}", fork.len());
        assert_eq!(r.dots[3].r, RAIL_HEAD_R);
        assert_eq!(r.dots[2].r, RAIL_RING_R);
        assert_eq!(r.dots[1].r, RAIL_DOT_R);
        assert!(!r.dots[2].on_path);
        assert_eq!(
            split_day("2026-11-14 15:31:09"),
            (Some("2026-11-14"), "15:31:09")
        );
        assert_eq!(split_day("1:05"), (None, "1:05"));
        let l = vec![
            "2026-11-14 15:30:09".to_string(),
            "2026-11-14 15:31:09".into(),
        ];
        assert_eq!(shared_day(&l), Some("2026-11-14"));
        let l2 = vec![
            "2026-11-14 23:59:09".to_string(),
            "2026-11-15 00:01:09".into(),
        ];
        assert_eq!(shared_day(&l2), None);
    }
}
