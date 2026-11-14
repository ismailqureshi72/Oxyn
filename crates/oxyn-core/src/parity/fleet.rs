//! Agents, feeds, tables and counters (plan #28, chunk 2).
//!
//! - [`Crew`]: an orchestrator and its sub-agents as a tree: status, own and branch cost, time,
//!   folded branches, "needs you" and failure windows; also the fleet (several roots side by
//!   side, one open).
//! - [`Orchestration`]: a lead splits a goal into subtasks, fans them out to worker lanes under a
//!   concurrency limit, and merges the results.
//! - [`Feed`]: day-grouped events, newest first: timeline and activity feed (type filter, unread,
//!   held-back new updates, mark all read).
//! - [`TreeTable`]: nested rows, tri-state selection, hierarchy-preserving sort, lazy children.
//! - [`Countdown`] and [`digits_roll`]: launch countdown and rolling counter digits.
//! - [`Readiness`]: a release checklist with prerequisites and a gated final action.

use crate::ClockMs;

use super::plan::{StepState, short_duration};

/// One agent in a crew.
#[derive(Clone, Debug, PartialEq)]
pub struct CrewAgent {
    /// Id.
    pub id: String,
    /// Name.
    pub name: String,
    /// Task.
    pub task: String,
    /// Model.
    pub model: Option<String>,
    /// State (`Waiting` reads "Needs you").
    pub state: StepState,
    /// Own cost in dollars.
    pub cost: f64,
    /// Started.
    pub started: Option<ClockMs>,
    /// Ended.
    pub ended: Option<ClockMs>,
    /// What a waiting agent needs decided.
    pub ask: Option<String>,
    /// What went wrong.
    pub error: Option<String>,
    /// Tool names (overlapping tiles).
    pub tools: Vec<String>,
    /// Sub-agents.
    pub children: Vec<CrewAgent>,
}

impl CrewAgent {
    /// An agent.
    pub fn new(id: &str, name: &str, task: &str, state: StepState) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            task: task.into(),
            model: None,
            state,
            cost: 0.0,
            started: None,
            ended: None,
            ask: None,
            error: None,
            tools: Vec::new(),
            children: Vec::new(),
        }
    }
    /// Branch cost: own plus every descendant.
    pub fn branch_cost(&self) -> f64 {
        self.cost + self.children.iter().map(|c| c.branch_cost()).sum::<f64>()
    }
    /// Time text at `now`.
    pub fn time(&self, now: ClockMs) -> Option<String> {
        let s = self.started?;
        let e = self.ended.unwrap_or(now);
        Some(short_duration(e.0.saturating_sub(s.0)))
    }
    /// Count of descendants by state: (running, waiting, done, failed).
    pub fn tally(&self) -> (usize, usize, usize, usize) {
        let mut t = (0, 0, 0, 0);
        for c in &self.children {
            match c.state {
                StepState::Running => t.0 += 1,
                StepState::Waiting => t.1 += 1,
                StepState::Done => t.2 += 1,
                StepState::Failed => t.3 += 1,
                _ => {}
            }
            let s = c.tally();
            t = (t.0 + s.0, t.1 + s.1, t.2 + s.2, t.3 + s.3);
        }
        t
    }
    /// Finds an agent by id.
    pub fn find_mut(&mut self, id: &str) -> Option<&mut CrewAgent> {
        if self.id == id {
            return Some(self);
        }
        self.children.iter_mut().find_map(|c| c.find_mut(id))
    }
    /// Flattened `(depth, agent)` rows, skipping the children of `folded` ids.
    pub fn rows<'a>(
        &'a self,
        folded: &[String],
        depth: usize,
        out: &mut Vec<(usize, &'a CrewAgent)>,
    ) {
        out.push((depth, self));
        if !folded.contains(&self.id) {
            for c in &self.children {
                c.rows(folded, depth + 1, out);
            }
        }
    }
}

/// An answer to a waiting or failed agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Resolve {
    /// Approve what it asks.
    Approve,
    /// Reject what it asks.
    Reject,
    /// Retry after a failure.
    Retry,
    /// Dismiss a failure.
    Dismiss,
}

impl CrewAgent {
    /// Applies an answer: approve/retry → running, reject → declined, dismiss → stopped.
    pub fn resolve(&mut self, r: Resolve) {
        self.state = match r {
            Resolve::Approve | Resolve::Retry => StepState::Running,
            Resolve::Reject => StepState::Declined,
            Resolve::Dismiss => StepState::Skipped,
        };
        self.ask = None;
        self.error = None;
    }
}

/// `$0.42`.
pub fn dollars(x: f64) -> String {
    format!("${x:.2}")
}

/// Lane state in an orchestration.
#[derive(Clone, Debug, PartialEq)]
pub struct Lane {
    /// Worker name (model).
    pub worker: String,
    /// Subtask title.
    pub title: String,
    /// The lead's instruction.
    pub instruction: String,
    /// State.
    pub state: StepState,
    /// Progress 0..1.
    pub progress: f32,
    /// Result line once done.
    pub result: Option<String>,
}

/// Phase of an orchestration.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OrchPhase {
    /// Lead is planning.
    Planning,
    /// Lanes are working.
    FanOut,
    /// Lead is merging.
    Merging,
    /// Summary ready.
    Done,
}

/// A lead agent fanning a goal out to worker lanes.
#[derive(Clone, Debug, PartialEq)]
pub struct Orchestration {
    /// Goal.
    pub goal: String,
    /// Lead name.
    pub lead: String,
    /// Lanes.
    pub lanes: Vec<Lane>,
    /// Concurrency 1..=4.
    pub concurrency: usize,
    /// Phase.
    pub phase: OrchPhase,
    /// Summary.
    pub summary: Option<String>,
}

impl Orchestration {
    /// Starts queued lanes up to the concurrency limit.
    pub fn dispatch(&mut self) {
        if self.phase == OrchPhase::Planning {
            self.phase = OrchPhase::FanOut;
        }
        let running = self
            .lanes
            .iter()
            .filter(|l| l.state == StepState::Running)
            .count();
        let mut free = self.concurrency.clamp(1, 4).saturating_sub(running);
        for l in self.lanes.iter_mut() {
            if free == 0 {
                break;
            }
            if l.state == StepState::Pending {
                l.state = StepState::Running;
                free -= 1;
            }
        }
    }
    /// Lane `i` finished with a result; dispatches the next and merges when all are done.
    pub fn complete(&mut self, i: usize, result: &str) {
        if let Some(l) = self.lanes.get_mut(i) {
            l.state = StepState::Done;
            l.progress = 1.0;
            l.result = Some(result.into());
        }
        self.dispatch();
        if self.lanes.iter().all(|l| l.state.settled()) {
            self.phase = OrchPhase::Merging;
        }
    }
    /// "2 running · 1 queued · 1 done".
    pub fn status(&self) -> String {
        let n = |s: StepState| self.lanes.iter().filter(|l| l.state == s).count();
        format!(
            "{} running · {} queued · {} done",
            n(StepState::Running),
            n(StepState::Pending),
            n(StepState::Done)
        )
    }
}

/// Kind of feed event.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EventKind {
    /// Code pushed / merged.
    Code,
    /// A comment.
    Comment,
    /// A deploy or release.
    Deploy,
    /// An agent action.
    Agent,
    /// An incident or failure.
    Alert,
}

impl EventKind {
    /// Kebab id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Code => "code",
            Self::Comment => "comment",
            Self::Deploy => "deploy",
            Self::Agent => "agent",
            Self::Alert => "alert",
        }
    }
    /// Icon name.
    pub fn icon(self) -> &'static str {
        match self {
            Self::Code => "branch",
            Self::Comment => "comment",
            Self::Deploy => "bolt",
            Self::Agent => "spark",
            Self::Alert => "alert",
        }
    }
    /// From a kebab id.
    pub fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "code" => Self::Code,
            "comment" => Self::Comment,
            "deploy" => Self::Deploy,
            "agent" => Self::Agent,
            "alert" => Self::Alert,
            _ => return None,
        })
    }
}

/// One feed event.
#[derive(Clone, Debug, PartialEq)]
pub struct FeedEvent {
    /// Id.
    pub id: String,
    /// Kind.
    pub kind: EventKind,
    /// Who.
    pub actor: String,
    /// What they did ("merged").
    pub action: String,
    /// What ("#482 Rate limits").
    pub target: String,
    /// Days ago (0 = today).
    pub day: u32,
    /// Clock time ("14:05").
    pub time: String,
    /// Unread.
    pub unread: bool,
    /// Detail shown when expanded.
    pub detail: Option<String>,
}

/// Day-grouped event feed.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Feed {
    /// Events, newest first.
    pub events: Vec<FeedEvent>,
    /// Arrived but held behind the "new updates" button.
    pub incoming: Vec<FeedEvent>,
    /// Type filter.
    pub filter: Option<EventKind>,
    /// Range in days (1, 7, 30).
    pub range: u32,
    /// Expanded ids.
    pub expanded: Vec<String>,
}

impl Feed {
    /// Day label.
    pub fn day_label(day: u32) -> String {
        match day {
            0 => "Today".into(),
            1 => "Yesterday".into(),
            n => format!("{n} days ago"),
        }
    }
    /// Visible events grouped by day label.
    pub fn groups(&self) -> Vec<(String, Vec<&FeedEvent>)> {
        let mut out: Vec<(String, Vec<&FeedEvent>)> = Vec::new();
        for e in self
            .events
            .iter()
            .filter(|e| e.day < self.range.max(1) && self.filter.is_none_or(|f| f == e.kind))
        {
            let l = Self::day_label(e.day);
            match out.last_mut() {
                Some((d, v)) if *d == l => v.push(e),
                _ => out.push((l, vec![e])),
            }
        }
        out
    }
    /// Unread count.
    pub fn unread(&self) -> usize {
        self.events.iter().filter(|e| e.unread).count()
    }
    /// Shows held-back updates at the top.
    pub fn show_new(&mut self) {
        let mut n = std::mem::take(&mut self.incoming);
        n.append(&mut self.events);
        self.events = n;
    }
    /// Toggles an event open (and marks it read).
    pub fn toggle(&mut self, id: &str) {
        if let Some(i) = self.expanded.iter().position(|x| x == id) {
            self.expanded.remove(i);
        } else {
            self.expanded.push(id.into());
        }
        if let Some(e) = self.events.iter_mut().find(|e| e.id == id) {
            e.unread = false;
        }
    }
    /// Marks everything read.
    pub fn mark_all_read(&mut self) {
        for e in self.events.iter_mut() {
            e.unread = false;
        }
    }
}

/// Tri-state check.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tri {
    /// Unchecked.
    Off,
    /// Some descendants checked.
    Mixed,
    /// Checked.
    On,
}

/// A tree-table row.
#[derive(Clone, Debug, PartialEq)]
pub struct TRow {
    /// Id.
    pub id: String,
    /// Cells (first is the name).
    pub cells: Vec<String>,
    /// Numeric sort key per cell (None sorts as text).
    pub keys: Vec<Option<f64>>,
    /// Children.
    pub children: Vec<TRow>,
    /// Has children that load on first expand.
    pub lazy: bool,
    /// Lazy load failed (retry row).
    pub failed: bool,
}

/// Tree table state.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TreeTable {
    /// Rows.
    pub rows: Vec<TRow>,
    /// Expanded ids.
    pub expanded: Vec<String>,
    /// Selected ids (leaves and parents kept in step).
    pub selected: Vec<String>,
    /// Sort `(column, ascending)`.
    pub sort: Option<(usize, bool)>,
}

fn leaves<'a>(r: &'a TRow, out: &mut Vec<&'a str>) {
    if r.children.is_empty() {
        out.push(&r.id);
    }
    for c in &r.children {
        leaves(c, out);
    }
}

fn find<'a>(rows: &'a [TRow], id: &str) -> Option<&'a TRow> {
    for r in rows {
        if r.id == id {
            return Some(r);
        }
        if let Some(x) = find(&r.children, id) {
            return Some(x);
        }
    }
    None
}

impl TreeTable {
    /// Check state of a row from its leaves.
    pub fn tri(&self, id: &str) -> Tri {
        let Some(r) = find(&self.rows, id) else {
            return Tri::Off;
        };
        let mut l = Vec::new();
        leaves(r, &mut l);
        let n = l
            .iter()
            .filter(|x| self.selected.iter().any(|s| s == *x))
            .count();
        if n == 0 {
            Tri::Off
        } else if n == l.len() {
            Tri::On
        } else {
            Tri::Mixed
        }
    }
    /// Toggles a row: checking a parent checks every leaf under it.
    pub fn toggle_select(&mut self, id: &str) {
        let Some(r) = find(&self.rows, id) else {
            return;
        };
        let mut l = Vec::new();
        leaves(r, &mut l);
        let on = self.tri(id) != Tri::On;
        let l: Vec<String> = l.into_iter().map(String::from).collect();
        self.selected.retain(|s| !l.contains(s));
        if on {
            self.selected.extend(l);
        }
    }
    /// Header click: ascending → descending → off.
    pub fn cycle_sort(&mut self, col: usize) {
        self.sort = match self.sort {
            Some((c, true)) if c == col => Some((col, false)),
            Some((c, false)) if c == col => None,
            _ => Some((col, true)),
        };
    }
    fn sorted(&self, rows: &[TRow]) -> Vec<TRow> {
        let mut v: Vec<TRow> = rows
            .iter()
            .map(|r| TRow {
                children: self.sorted(&r.children),
                ..r.clone()
            })
            .collect();
        if let Some((c, asc)) = self.sort {
            v.sort_by(|a, b| {
                let o = match (
                    a.keys.get(c).copied().flatten(),
                    b.keys.get(c).copied().flatten(),
                ) {
                    (Some(x), Some(y)) => x.total_cmp(&y),
                    _ => a.cells.get(c).cmp(&b.cells.get(c)),
                };
                if asc { o } else { o.reverse() }
            });
        }
        v
    }
    /// Visible rows `(depth, row)`, sorted within each parent.
    pub fn visible(&self) -> Vec<(usize, TRow)> {
        fn walk(t: &TreeTable, rows: &[TRow], d: usize, out: &mut Vec<(usize, TRow)>) {
            for r in rows {
                out.push((d, r.clone()));
                if t.expanded.contains(&r.id) {
                    walk(t, &r.children, d + 1, out);
                }
            }
        }
        let mut out = Vec::new();
        walk(self, &self.sorted(&self.rows), 0, &mut out);
        out
    }
    /// Toggles a row open.
    pub fn toggle_open(&mut self, id: &str) {
        if let Some(i) = self.expanded.iter().position(|x| x == id) {
            self.expanded.remove(i);
        } else {
            self.expanded.push(id.into());
        }
    }
}

/// A launch countdown.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Countdown {
    /// Target time.
    pub target: ClockMs,
}

impl Countdown {
    /// `[days, hours, minutes, seconds]` left; the largest shown unit absorbs the rest
    /// (`units` = how many from the left are hidden).
    pub fn parts(&self, now: ClockMs, drop_days: bool) -> Option<[u64; 4]> {
        if now >= self.target {
            return None;
        }
        let s = (self.target.0 - now.0).div_ceil(1000);
        let (d, h, m, sec) = (s / 86_400, (s % 86_400) / 3600, (s % 3600) / 60, s % 60);
        Some(if drop_days {
            [0, d * 24 + h, m, sec]
        } else {
            [d, h, m, sec]
        })
    }
    /// One compact line: "2d 04:05:09" or "04:05:09".
    pub fn compact(&self, now: ClockMs) -> Option<String> {
        let [d, h, m, s] = self.parts(now, false)?;
        Some(if d > 0 {
            format!("{d}d {h:02}:{m:02}:{s:02}")
        } else {
            format!("{h:02}:{m:02}:{s:02}")
        })
    }
}

/// Per-digit roll for a counter moving `from → to`: each column's offset 0..1 at progress `t`
/// and the direction the whole number moved (`true` = up).
pub fn digits_roll(from: f64, to: f64, decimals: usize, t: f32) -> (Vec<(char, char, f32)>, bool) {
    let a = format!("{from:.decimals$}");
    let b = format!("{to:.decimals$}");
    let w = a.len().max(b.len());
    let pad = |s: &str| format!("{s:>w$}");
    let (a, b) = (pad(&a), pad(&b));
    let t = t.clamp(0.0, 1.0);
    let cols = a
        .chars()
        .zip(b.chars())
        .enumerate()
        .map(|(i, (x, y))| {
            let stagger = (i as f32 * 0.04).min(0.3);
            let k = ((t - stagger) / (1.0 - stagger)).clamp(0.0, 1.0);
            (x, y, if x == y { 1.0 } else { k })
        })
        .collect();
    (cols, to >= from)
}

/// Thousands separators: `1284.5` → `1,284.50` with `decimals = 2`.
pub fn group(x: f64, decimals: usize) -> String {
    let s = format!("{:.decimals$}", x.abs());
    let (int, frac) = match s.split_once('.') {
        Some((i, f)) => (i.to_string(), Some(f.to_string())),
        None => (s, None),
    };
    let mut out = String::new();
    for (i, ch) in int.chars().enumerate() {
        if i > 0 && (int.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(ch);
    }
    if let Some(f) = frac {
        out.push('.');
        out.push_str(&f);
    }
    if x < 0.0 { format!("−{out}") } else { out }
}

/// A release prerequisite.
#[derive(Clone, Debug, PartialEq)]
pub struct Prereq {
    /// Group ("Quality", "Docs").
    pub group: String,
    /// Title.
    pub title: String,
    /// Owner.
    pub owner: String,
    /// Done.
    pub done: bool,
    /// Blocking (release waits for it).
    pub blocking: bool,
}

/// Release readiness.
#[derive(Clone, Debug, PartialEq)]
pub struct Readiness {
    /// Version.
    pub version: String,
    /// Prerequisites.
    pub items: Vec<Prereq>,
    /// Release state.
    pub release: StepState,
    /// Error from the last attempt.
    pub error: Option<String>,
}

impl Readiness {
    /// `(done, total)`.
    pub fn count(&self) -> (usize, usize) {
        (
            self.items.iter().filter(|i| i.done).count(),
            self.items.len(),
        )
    }
    /// Blocking items still open.
    pub fn blockers(&self) -> usize {
        self.items.iter().filter(|i| i.blocking && !i.done).count()
    }
    /// Release enabled.
    pub fn can_release(&self) -> bool {
        self.blockers() == 0
            && self.release != StepState::Running
            && self.release != StepState::Done
    }
    /// Groups in first-seen order.
    pub fn groups(&self) -> Vec<String> {
        let mut g: Vec<String> = Vec::new();
        for i in &self.items {
            if !g.contains(&i.group) {
                g.push(i.group.clone());
            }
        }
        g
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crew_costs_and_rows() {
        let mut root = CrewAgent::new("r", "Lead", "t", StepState::Running);
        root.cost = 0.10;
        let mut a = CrewAgent::new("a", "A", "t", StepState::Waiting);
        a.cost = 0.05;
        a.children
            .push(CrewAgent::new("a1", "A1", "t", StepState::Done));
        root.children.push(a);
        assert!((root.branch_cost() - 0.15).abs() < 1e-9);
        assert_eq!(root.tally(), (0, 1, 1, 0));
        let mut rows = Vec::new();
        root.rows(&["a".into()], 0, &mut rows);
        assert_eq!(rows.len(), 2);
        root.find_mut("a").unwrap().resolve(Resolve::Approve);
        assert_eq!(root.children[0].state, StepState::Running);
    }

    #[test]
    fn orchestration_respects_concurrency() {
        let lane = |w: &str| Lane {
            worker: w.into(),
            title: w.into(),
            instruction: String::new(),
            state: StepState::Pending,
            progress: 0.0,
            result: None,
        };
        let mut o = Orchestration {
            goal: "g".into(),
            lead: "L".into(),
            lanes: vec![lane("a"), lane("b"), lane("c")],
            concurrency: 2,
            phase: OrchPhase::Planning,
            summary: None,
        };
        o.dispatch();
        assert_eq!(o.status(), "2 running · 1 queued · 0 done");
        o.complete(0, "ok");
        o.complete(1, "ok");
        o.complete(2, "ok");
        assert_eq!(o.phase, OrchPhase::Merging);
    }

    #[test]
    fn feed_tree_counter() {
        let ev = |id: &str, day: u32, kind: EventKind| FeedEvent {
            id: id.into(),
            kind,
            actor: "a".into(),
            action: "b".into(),
            target: "c".into(),
            day,
            time: "1".into(),
            unread: true,
            detail: None,
        };
        let mut f = Feed {
            events: vec![ev("1", 0, EventKind::Code), ev("2", 1, EventKind::Deploy)],
            incoming: vec![ev("0", 0, EventKind::Agent)],
            range: 7,
            ..Default::default()
        };
        assert_eq!(f.groups().len(), 2);
        f.filter = Some(EventKind::Deploy);
        assert_eq!(f.groups()[0].0, "Yesterday");
        f.show_new();
        f.toggle("0");
        assert_eq!(f.unread(), 2);
        f.mark_all_read();
        assert_eq!(f.unread(), 0);

        let leaf = |id: &str, n: f64| TRow {
            id: id.into(),
            cells: vec![id.into()],
            keys: vec![Some(n)],
            children: vec![],
            lazy: false,
            failed: false,
        };
        let mut t = TreeTable {
            rows: vec![TRow {
                children: vec![leaf("b", 2.0), leaf("c", 1.0)],
                ..leaf("a", 0.0)
            }],
            expanded: vec!["a".into()],
            ..Default::default()
        };
        t.toggle_select("b");
        assert_eq!(t.tri("a"), Tri::Mixed);
        t.toggle_select("a");
        assert_eq!(t.tri("a"), Tri::On);
        t.cycle_sort(0);
        assert_eq!(t.visible()[1].1.id, "c");

        let c = Countdown {
            target: ClockMs(90_061_000),
        };
        assert_eq!(c.parts(ClockMs(0), false), Some([1, 1, 1, 1]));
        assert_eq!(c.parts(ClockMs(0), true), Some([0, 25, 1, 1]));
        assert_eq!(group(1284.5, 2), "1,284.50");
        let (cols, up) = digits_roll(9.0, 10.0, 0, 1.0);
        assert!(up && cols.len() == 2);
    }
}
