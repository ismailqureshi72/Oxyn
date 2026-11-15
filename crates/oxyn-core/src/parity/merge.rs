//! Merge conflicts (plan #31): the 3-way merge-conflict editor model and Conflict Radar.
//!
//! - [`parse_markers`] reads a file with git conflict markers (`<<<<<<<`, optional `|||||||` base,
//!   `=======`, `>>>>>>>`) into [`Seg`]ments; [`MergeFile`] holds one file, [`Merge`] the whole
//!   session (files, picks, manual edits, focus, undo, marked files) and builds the result text.
//! - [`merge_key`] maps keys to [`MergeCmd`]s so the Dioxus and GPUI views share one keymap.
//! - [`Radar`] is the Conflict Radar model: agents working on their own branches, the files they
//!   touch, overlapping line ranges as conflicts, the way picked per conflicting file, per-agent
//!   numbers, a suggested merge order and when Merge becomes possible.
//!
//! The component never runs git; hosts map their merge state onto these types and route the
//! callbacks back. Behaviour-only rebuild of the affectUI Conflict Radar; nothing copied.

use std::collections::{BTreeMap, BTreeSet};

// ---------------------------------------------------------------------------------------------
// Parsing
// ---------------------------------------------------------------------------------------------

/// One input of a 3-way merge.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Side {
    /// The checked-out side (`HEAD`): git's "ours" in a merge.
    Ours,
    /// The merge base (common ancestor).
    Base,
    /// The side being merged in: git's "theirs".
    Theirs,
}

impl Side {
    /// Stable id (`ours` · `base` · `theirs`).
    pub fn id(self) -> &'static str {
        match self {
            Self::Ours => "ours",
            Self::Base => "base",
            Self::Theirs => "theirs",
        }
    }
}

/// An agent's proposed resolution for one conflict.
#[derive(Clone, Debug, PartialEq)]
pub struct Suggestion {
    /// Proposed lines.
    pub lines: Vec<String>,
    /// One or two sentences on why, shown verbatim.
    pub rationale: String,
    /// `high` · `medium` · `low`, only when the agent supplies it.
    pub confidence: Option<&'static str>,
}

/// One conflicting region.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Conflict {
    /// Lines on our side.
    pub ours: Vec<String>,
    /// Lines in the merge base (`None` when the file had no `|||||||` section).
    pub base: Option<Vec<String>>,
    /// Lines on their side.
    pub theirs: Vec<String>,
    /// Optional agent suggestion.
    pub suggestion: Option<Suggestion>,
}

/// A run of lines that merged cleanly, or a conflict.
#[derive(Clone, Debug, PartialEq)]
pub enum Seg {
    /// Lines identical in the result.
    Same(Vec<String>),
    /// A conflicting region.
    Conflict(Conflict),
}

/// What went wrong while reading markers.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseError {
    /// A `=======` or `>>>>>>>` without an opening marker, at this 1-based line.
    Stray(usize),
    /// A conflict opened at this 1-based line never closed.
    Unclosed(usize),
}

fn marker(line: &str, ch: char) -> bool {
    let n = line.chars().take_while(|&c| c == ch).count();
    n == 7 && (line.len() == 7 || line[7..].starts_with(' '))
}

/// Reads a file with git conflict markers into segments. Labels after the markers are ignored
/// (the host supplies branch names); a `|||||||` section becomes the conflict's base.
pub fn parse_markers(text: &str) -> Result<Vec<Seg>, ParseError> {
    #[derive(PartialEq)]
    enum At {
        Out,
        Ours,
        Base,
        Theirs,
    }
    let mut out = Vec::new();
    let mut same: Vec<String> = Vec::new();
    let mut cur = Conflict::default();
    let mut at = At::Out;
    let mut opened = 0;
    for (i, line) in text.lines().enumerate() {
        if marker(line, '<') {
            if at != At::Out {
                return Err(ParseError::Unclosed(opened));
            }
            if !same.is_empty() {
                out.push(Seg::Same(std::mem::take(&mut same)));
            }
            at = At::Ours;
            opened = i + 1;
        } else if marker(line, '|') && at == At::Ours {
            at = At::Base;
            cur.base = Some(Vec::new());
        } else if marker(line, '=') && at != At::Out {
            if at == At::Theirs {
                return Err(ParseError::Stray(i + 1));
            }
            at = At::Theirs;
        } else if marker(line, '>') {
            if at != At::Theirs {
                return Err(ParseError::Stray(i + 1));
            }
            out.push(Seg::Conflict(std::mem::take(&mut cur)));
            at = At::Out;
        } else if (marker(line, '=') || marker(line, '|')) && at == At::Out {
            return Err(ParseError::Stray(i + 1));
        } else {
            let l = line.to_string();
            match at {
                At::Out => same.push(l),
                At::Ours => cur.ours.push(l),
                At::Base => cur.base.get_or_insert_with(Vec::new).push(l),
                At::Theirs => cur.theirs.push(l),
            }
        }
    }
    if at != At::Out {
        return Err(ParseError::Unclosed(opened));
    }
    if !same.is_empty() {
        out.push(Seg::Same(same));
    }
    Ok(out)
}

/// Number of marker lines (`<<<<<<<`, `|||||||`, `=======`, `>>>>>>>`) in `text`.
pub fn count_markers(text: &str) -> usize {
    text.lines()
        .filter(|l| marker(l, '<') || marker(l, '|') || marker(l, '=') || marker(l, '>'))
        .count()
}

// ---------------------------------------------------------------------------------------------
// Picks and the result
// ---------------------------------------------------------------------------------------------

/// How a conflict was settled.
#[derive(Clone, Debug, PartialEq)]
pub enum Pick {
    /// Take our lines.
    Ours,
    /// Take their lines.
    Theirs,
    /// Ours then theirs (`theirs_first` swaps the order).
    Both {
        /// Their lines go first.
        theirs_first: bool,
    },
    /// Keep the base (drop both changes).
    Base,
    /// The agent's suggestion.
    Agent,
    /// Text typed by hand.
    Manual(Vec<String>),
}

impl Pick {
    /// Short tag shown on a resolved region.
    pub fn tag(&self) -> &'static str {
        match self {
            Self::Ours => "ours",
            Self::Theirs => "theirs",
            Self::Both { .. } => "both",
            Self::Base => "base",
            Self::Agent => "agent",
            Self::Manual(_) => "edited",
        }
    }
}

/// The lines a pick puts into the result.
pub fn pick_lines(c: &Conflict, p: &Pick) -> Vec<String> {
    match p {
        Pick::Ours => c.ours.clone(),
        Pick::Theirs => c.theirs.clone(),
        Pick::Both { theirs_first } => {
            let (a, b) = if *theirs_first {
                (&c.theirs, &c.ours)
            } else {
                (&c.ours, &c.theirs)
            };
            let mut v = a.clone();
            for l in b {
                // identical lines on both sides are written once
                if !a.contains(l) {
                    v.push(l.clone());
                }
            }
            v
        }
        Pick::Base => c.base.clone().unwrap_or_default(),
        Pick::Agent => c
            .suggestion
            .as_ref()
            .map(|s| s.lines.clone())
            .unwrap_or_default(),
        Pick::Manual(l) => l.clone(),
    }
}

/// The marker block an unresolved conflict leaves in the result.
pub fn marker_block(c: &Conflict, ours: &str, theirs: &str) -> Vec<String> {
    let mut v = vec![format!("<<<<<<< {ours}")];
    v.extend(c.ours.iter().cloned());
    if let Some(b) = &c.base {
        v.push("||||||| base".to_string());
        v.extend(b.iter().cloned());
    }
    v.push("=======".to_string());
    v.extend(c.theirs.iter().cloned());
    v.push(format!(">>>>>>> {theirs}"));
    v
}

/// One file in conflict.
#[derive(Clone, Debug, PartialEq)]
pub struct MergeFile {
    /// Repository path.
    pub path: String,
    /// Segments in file order.
    pub segs: Vec<Seg>,
}

impl MergeFile {
    /// A file from segments.
    pub fn new(path: &str, segs: Vec<Seg>) -> Self {
        Self {
            path: path.to_string(),
            segs,
        }
    }

    /// A file read from marker text (a malformed file becomes one clean segment).
    pub fn from_markers(path: &str, text: &str) -> Self {
        let segs = parse_markers(text)
            .unwrap_or_else(|_| vec![Seg::Same(text.lines().map(str::to_string).collect())]);
        Self::new(path, segs)
    }

    /// Conflicts in order.
    pub fn conflicts(&self) -> Vec<&Conflict> {
        self.segs
            .iter()
            .filter_map(|s| match s {
                Seg::Conflict(c) => Some(c),
                Seg::Same(_) => None,
            })
            .collect()
    }

    /// Segment index of conflict `n`.
    pub fn seg_of(&self, n: usize) -> Option<usize> {
        self.segs
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s, Seg::Conflict(_)))
            .nth(n)
            .map(|(i, _)| i)
    }

    /// 1-based result line where each conflict starts, given the picks so far.
    pub fn conflict_lines(&self, picks: &BTreeMap<usize, Pick>) -> Vec<usize> {
        let mut line = 1;
        let mut out = Vec::new();
        let mut n = 0;
        for s in &self.segs {
            match s {
                Seg::Same(l) => line += l.len(),
                Seg::Conflict(c) => {
                    out.push(line);
                    line += match picks.get(&n) {
                        Some(p) => pick_lines(c, p).len(),
                        None => marker_block(c, "", "").len(),
                    };
                    n += 1;
                }
            }
        }
        out
    }
}

/// Where a result line came from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Origin {
    /// Merged cleanly.
    Same,
    /// From a settled conflict.
    Picked,
    /// A marker or side line of an unresolved conflict.
    Marker,
}

/// One line of the result buffer.
#[derive(Clone, Debug, PartialEq)]
pub struct ResultLine {
    /// Text.
    pub text: String,
    /// Origin.
    pub origin: Origin,
    /// The conflict it belongs to.
    pub conflict: Option<usize>,
}

/// What a key does in the editor.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MergeCmd {
    /// Next unresolved conflict (wraps).
    Next,
    /// Previous unresolved conflict (wraps).
    Prev,
    /// Settle the focused conflict.
    Take(TakeKind),
    /// Reopen the focused conflict.
    Reopen,
    /// Edit the focused conflict by hand.
    Edit,
    /// Undo the last change.
    Undo,
    /// Mark the current file resolved.
    Mark,
}

/// The one-key picks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TakeKind {
    /// Ours.
    Ours,
    /// Theirs.
    Theirs,
    /// Both, ours first.
    Both,
    /// Base.
    Base,
    /// The agent's suggestion.
    Agent,
}

impl TakeKind {
    /// As a [`Pick`].
    pub fn pick(self) -> Pick {
        match self {
            Self::Ours => Pick::Ours,
            Self::Theirs => Pick::Theirs,
            Self::Both => Pick::Both {
                theirs_first: false,
            },
            Self::Base => Pick::Base,
            Self::Agent => Pick::Agent,
        }
    }
}

/// Maps a key (`code` is the physical key, e.g. `Digit1`, so Option+1 works on macOS) to a
/// command. `cmd` is ⌘ on macOS and Ctrl elsewhere.
pub fn merge_key(key: &str, code: &str, alt: bool, cmd: bool, shift: bool) -> Option<MergeCmd> {
    if cmd && shift && key == "Enter" {
        return Some(MergeCmd::Mark);
    }
    if cmd && !shift && (key == "z" || key == "Z") {
        return Some(MergeCmd::Undo);
    }
    if cmd {
        return None;
    }
    if alt {
        return match code {
            "ArrowDown" => Some(MergeCmd::Next),
            "ArrowUp" => Some(MergeCmd::Prev),
            "Digit1" => Some(MergeCmd::Take(TakeKind::Ours)),
            "Digit2" => Some(MergeCmd::Take(TakeKind::Theirs)),
            "Digit3" => Some(MergeCmd::Take(TakeKind::Both)),
            "Digit4" => Some(MergeCmd::Take(TakeKind::Base)),
            "Digit5" => Some(MergeCmd::Take(TakeKind::Agent)),
            "KeyR" => Some(MergeCmd::Reopen),
            _ => None,
        };
    }
    match key {
        "]" => Some(MergeCmd::Next),
        "[" => Some(MergeCmd::Prev),
        "e" | "E" => Some(MergeCmd::Edit),
        _ => None,
    }
}

/// How the session ends.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// Resolving.
    Open,
    /// Abort asked; waiting for the confirm.
    ConfirmAbort,
    /// Every file marked and Complete pressed.
    Completed,
    /// Aborted: files restored.
    Aborted,
}

#[derive(Clone, Debug, PartialEq)]
enum Undo {
    Pick(usize, usize, Option<Pick>),
    Mark(usize),
}

/// A merge session: one or more conflicted files and everything the user decided.
#[derive(Clone, Debug, PartialEq)]
pub struct Merge {
    /// Files.
    pub files: Vec<MergeFile>,
    /// Picks per file, keyed by conflict index.
    pub picks: Vec<BTreeMap<usize, Pick>>,
    /// Files marked resolved (staged).
    pub marked: BTreeSet<usize>,
    /// Current file.
    pub file: usize,
    /// Focused conflict in the current file.
    pub focus: usize,
    /// Lifecycle.
    pub phase: Phase,
    undo: Vec<Undo>,
}

impl Merge {
    /// A session over `files`, focused on the first conflict.
    pub fn new(files: Vec<MergeFile>) -> Self {
        let n = files.len();
        Self {
            files,
            picks: vec![BTreeMap::new(); n],
            marked: BTreeSet::new(),
            file: 0,
            focus: 0,
            phase: Phase::Open,
            undo: Vec::new(),
        }
    }

    /// The current file.
    pub fn cur(&self) -> &MergeFile {
        &self.files[self.file]
    }

    /// Conflicts in file `f`.
    pub fn total(&self, f: usize) -> usize {
        self.files.get(f).map_or(0, |x| x.conflicts().len())
    }

    /// Settled conflicts in file `f` (a manual edit that still has markers does not count).
    pub fn resolved(&self, f: usize) -> usize {
        (0..self.total(f))
            .filter(|&n| self.is_resolved(f, n))
            .count()
    }

    /// Conflicts across every file.
    pub fn total_all(&self) -> usize {
        (0..self.files.len()).map(|f| self.total(f)).sum()
    }

    /// Settled conflicts across every file.
    pub fn resolved_all(&self) -> usize {
        (0..self.files.len()).map(|f| self.resolved(f)).sum()
    }

    /// Whether conflict `n` of file `f` is settled.
    pub fn is_resolved(&self, f: usize, n: usize) -> bool {
        match self.picks.get(f).and_then(|p| p.get(&n)) {
            None => false,
            Some(Pick::Manual(l)) => l.iter().all(|x| count_markers(x) == 0),
            Some(_) => true,
        }
    }

    /// The pick for conflict `n` of the current file.
    pub fn pick(&self, n: usize) -> Option<&Pick> {
        self.picks[self.file].get(&n)
    }

    /// Settles conflict `n` of the current file. A pick for a missing suggestion or base is
    /// ignored (returns `false`). With `advance`, focus moves to the next unresolved conflict.
    pub fn take(&mut self, n: usize, p: Pick, advance: bool) -> bool {
        if self.phase != Phase::Open || self.marked.contains(&self.file) {
            return false;
        }
        let Some(c) = self.cur().conflicts().get(n).map(|c| (*c).clone()) else {
            return false;
        };
        if (p == Pick::Agent && c.suggestion.is_none()) || (p == Pick::Base && c.base.is_none()) {
            return false;
        }
        let was = self.picks[self.file].insert(n, p);
        self.undo.push(Undo::Pick(self.file, n, was));
        self.focus = n;
        if advance && self.is_resolved(self.file, n) {
            self.go(true);
        }
        true
    }

    /// Reopens conflict `n` of the current file.
    pub fn reopen(&mut self, n: usize) {
        if self.marked.contains(&self.file) || self.phase != Phase::Open {
            return;
        }
        if let Some(was) = self.picks[self.file].remove(&n) {
            self.undo.push(Undo::Pick(self.file, n, Some(was)));
        }
        self.focus = n;
    }

    /// Settles every unresolved conflict that has a suggestion. Returns how many.
    pub fn accept_all_suggestions(&mut self) -> usize {
        let mut k = 0;
        for n in 0..self.total(self.file) {
            if !self.is_resolved(self.file, n)
                && self.cur().conflicts()[n].suggestion.is_some()
                && self.take(n, Pick::Agent, false)
            {
                k += 1;
            }
        }
        k
    }

    /// Takes one side for every unresolved conflict in the current file.
    pub fn take_all(&mut self, p: Pick) -> usize {
        let mut k = 0;
        for n in 0..self.total(self.file) {
            if !self.is_resolved(self.file, n) && self.take(n, p.clone(), false) {
                k += 1;
            }
        }
        k
    }

    /// Undoes the last pick, reopen or mark. Returns `false` when there is nothing to undo.
    pub fn undo(&mut self) -> bool {
        match self.undo.pop() {
            Some(Undo::Pick(f, n, was)) => {
                match was {
                    Some(p) => {
                        self.picks[f].insert(n, p);
                    }
                    None => {
                        self.picks[f].remove(&n);
                    }
                }
                self.file = f;
                self.focus = n;
                true
            }
            Some(Undo::Mark(f)) => {
                self.marked.remove(&f);
                self.file = f;
                true
            }
            None => false,
        }
    }

    /// Unresolved conflicts of the current file, in order.
    pub fn unresolved(&self) -> Vec<usize> {
        (0..self.total(self.file))
            .filter(|&n| !self.is_resolved(self.file, n))
            .collect()
    }

    /// Moves focus to the next (or previous) unresolved conflict, wrapping; stays when none.
    pub fn go(&mut self, forward: bool) {
        let open = self.unresolved();
        if open.is_empty() {
            return;
        }
        let f = self.focus;
        self.focus = if forward {
            open.iter().copied().find(|&n| n > f).unwrap_or(open[0])
        } else {
            open.iter()
                .rev()
                .copied()
                .find(|&n| n < f)
                .unwrap_or(open[open.len() - 1])
        };
    }

    /// Moves to any conflict by index, regardless of state.
    pub fn focus_on(&mut self, n: usize) {
        if n < self.total(self.file) {
            self.focus = n;
        }
    }

    /// Switches file and focuses its first unresolved conflict.
    pub fn open_file(&mut self, f: usize) {
        if f < self.files.len() {
            self.file = f;
            self.focus = self.unresolved().first().copied().unwrap_or(0);
        }
    }

    /// Marker lines still in the current file's result.
    pub fn markers_left(&self) -> usize {
        self.result()
            .iter()
            .filter(|l| l.origin == Origin::Marker && count_markers(&l.text) > 0)
            .count()
            + self.picks[self.file]
                .values()
                .map(|p| match p {
                    Pick::Manual(l) => l.iter().map(|x| count_markers(x)).sum(),
                    _ => 0,
                })
                .sum::<usize>()
    }

    /// Why the current file cannot be marked resolved yet (`None` = it can).
    pub fn mark_blocker(&self) -> Option<String> {
        if self.marked.contains(&self.file) {
            return Some("Already marked resolved".to_string());
        }
        let left = self.total(self.file) - self.resolved(self.file);
        if left > 0 {
            return Some(format!(
                "{left} conflict{} left",
                if left == 1 { "" } else { "s" }
            ));
        }
        let m = self.markers_left();
        if m > 0 {
            return Some(format!("{m} marker{} left", if m == 1 { "" } else { "s" }));
        }
        None
    }

    /// Marks the current file resolved. Returns `false` when blocked.
    pub fn mark(&mut self) -> bool {
        if self.mark_blocker().is_some() || self.phase != Phase::Open {
            return false;
        }
        self.marked.insert(self.file);
        self.undo.push(Undo::Mark(self.file));
        // move on to the next file that still needs work
        if let Some(f) = (0..self.files.len()).find(|f| !self.marked.contains(f)) {
            self.open_file(f);
        }
        true
    }

    /// Why the merge cannot complete yet (`None` = it can).
    pub fn complete_blocker(&self) -> Option<String> {
        let left = self.files.len() - self.marked.len();
        if left > 0 {
            Some(format!(
                "{left} file{} not marked resolved",
                if left == 1 { "" } else { "s" }
            ))
        } else {
            None
        }
    }

    /// Completes the merge when every file is marked.
    pub fn complete(&mut self) -> bool {
        if self.complete_blocker().is_some() || self.phase != Phase::Open {
            return false;
        }
        self.phase = Phase::Completed;
        true
    }

    /// Asks to abort; `confirm` then aborts, `cancel` returns to resolving.
    pub fn abort(&mut self, step: &str) {
        self.phase = match (self.phase, step) {
            (Phase::Open, "ask") => Phase::ConfirmAbort,
            (Phase::ConfirmAbort, "confirm") => Phase::Aborted,
            (Phase::ConfirmAbort, "cancel") => Phase::Open,
            (p, _) => p,
        };
    }

    /// The current file's result buffer, line by line.
    pub fn result(&self) -> Vec<ResultLine> {
        self.result_with("ours", "theirs")
    }

    /// The result buffer with marker labels for unresolved conflicts.
    pub fn result_with(&self, ours: &str, theirs: &str) -> Vec<ResultLine> {
        let mut out = Vec::new();
        let mut n = 0;
        for s in &self.cur().segs {
            match s {
                Seg::Same(l) => out.extend(l.iter().map(|t| ResultLine {
                    text: t.clone(),
                    origin: Origin::Same,
                    conflict: None,
                })),
                Seg::Conflict(c) => {
                    let (lines, origin) = match self.picks[self.file].get(&n) {
                        Some(p) => (pick_lines(c, p), Origin::Picked),
                        None => (marker_block(c, ours, theirs), Origin::Marker),
                    };
                    out.extend(lines.into_iter().map(|t| ResultLine {
                        text: t,
                        origin,
                        conflict: Some(n),
                    }));
                    n += 1;
                }
            }
        }
        out
    }

    /// The current file's result as text.
    pub fn result_text(&self) -> String {
        let mut s = self
            .result()
            .into_iter()
            .map(|l| l.text)
            .collect::<Vec<_>>()
            .join("\n");
        s.push('\n');
        s
    }

    /// Polite announcement for the focused conflict.
    pub fn announce(&self) -> String {
        let total = self.total(self.file);
        if total == 0 {
            return "No conflicts".to_string();
        }
        let left = total - self.resolved(self.file);
        let state = match self.pick(self.focus) {
            Some(p) if self.is_resolved(self.file, self.focus) => {
                format!("resolved, took {}", p.tag())
            }
            Some(_) => "edited, markers left".to_string(),
            None => "unresolved".to_string(),
        };
        format!(
            "Conflict {} of {total} in {}, {state}. {left} left.",
            self.focus + 1,
            self.cur().path
        )
    }
}

// ---------------------------------------------------------------------------------------------
// Conflict Radar
// ---------------------------------------------------------------------------------------------

/// One agent working on its own branch.
#[derive(Clone, Debug, PartialEq)]
pub struct RadarAgent {
    /// Stable id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Model or branch.
    pub model: String,
    /// Still working.
    pub running: bool,
}

impl RadarAgent {
    /// A finished agent.
    pub fn new(id: &str, name: &str, model: &str) -> Self {
        Self {
            id: id.to_string(),
            name: name.to_string(),
            model: model.to_string(),
            running: false,
        }
    }

    /// Marks it as still working.
    pub fn running(mut self, r: bool) -> Self {
        self.running = r;
        self
    }

    /// Monogram (first letter of each of the first two words).
    pub fn monogram(&self) -> String {
        self.name
            .split_whitespace()
            .take(2)
            .filter_map(|w| w.chars().next())
            .collect::<String>()
            .to_uppercase()
    }
}

/// One agent's edit to one file.
#[derive(Clone, Debug, PartialEq)]
pub struct RadarEdit {
    /// The agent's id.
    pub agent: String,
    /// First changed line (1-based, inclusive).
    pub from: u32,
    /// Last changed line (inclusive).
    pub to: u32,
    /// Its way, in one sentence (shown when the file conflicts).
    pub summary: Option<String>,
    /// Still writing.
    pub active: bool,
}

impl RadarEdit {
    /// An edit to lines `from..=to`.
    pub fn new(agent: &str, from: u32, to: u32) -> Self {
        Self {
            agent: agent.to_string(),
            from: from.min(to),
            to: to.max(from),
            summary: None,
            active: false,
        }
    }

    /// Its way in one sentence.
    pub fn way(mut self, s: &str) -> Self {
        self.summary = Some(s.to_string());
        self
    }

    /// Still writing.
    pub fn active(mut self, a: bool) -> Self {
        self.active = a;
        self
    }

    /// Lines changed.
    pub fn len(&self) -> u32 {
        self.to - self.from + 1
    }

    /// Always `false` (an edit covers at least one line).
    pub fn is_empty(&self) -> bool {
        false
    }

    /// Lines shared with `o`.
    pub fn overlap(&self, o: &RadarEdit) -> u32 {
        let a = self.from.max(o.from);
        let b = self.to.min(o.to);
        if a <= b { b - a + 1 } else { 0 }
    }
}

/// One file with every agent's edit.
#[derive(Clone, Debug, PartialEq)]
pub struct RadarFile {
    /// Repository path.
    pub path: String,
    /// Edits.
    pub edits: Vec<RadarEdit>,
}

impl RadarFile {
    /// A file.
    pub fn new(path: &str, edits: Vec<RadarEdit>) -> Self {
        Self {
            path: path.to_string(),
            edits,
        }
    }

    /// Folder and name (`("src/auth/", "session.rs")`).
    pub fn split(&self) -> (String, String) {
        match self.path.rfind('/') {
            Some(i) => (self.path[..=i].to_string(), self.path[i + 1..].to_string()),
            None => (String::new(), self.path.clone()),
        }
    }

    /// The edit of agent `id`.
    pub fn edit(&self, id: &str) -> Option<&RadarEdit> {
        self.edits.iter().find(|e| e.agent == id)
    }

    /// Pairs of agents whose line ranges overlap, with the shared line count.
    pub fn clashes(&self) -> Vec<(String, String, u32)> {
        let mut v = Vec::new();
        for (i, a) in self.edits.iter().enumerate() {
            for b in &self.edits[i + 1..] {
                let o = a.overlap(b);
                if o > 0 {
                    v.push((a.agent.clone(), b.agent.clone(), o));
                }
            }
        }
        v
    }

    /// Whether two or more agents changed the same lines.
    pub fn conflicting(&self) -> bool {
        !self.clashes().is_empty()
    }

    /// Agents in a clash, in edit order.
    pub fn clash_agents(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for (a, b, _) in self.clashes() {
            for x in [a, b] {
                if !v.contains(&x) {
                    v.push(x);
                }
            }
        }
        v
    }

    /// Lines in any overlap (heat).
    pub fn overlap_lines(&self) -> u32 {
        self.clashes().iter().map(|c| c.2).max().unwrap_or(0)
    }
}

/// Per-agent numbers shown when pointing at an agent.
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct AgentNumbers {
    /// Files touched.
    pub files: usize,
    /// Conflicts involving the agent that are not settled.
    pub open: usize,
    /// Conflicts settled in its favour.
    pub kept: usize,
    /// Lines changed.
    pub lines: u32,
}

/// The Conflict Radar model.
#[derive(Clone, Debug, PartialEq)]
pub struct Radar {
    /// Columns.
    pub agents: Vec<RadarAgent>,
    /// Rows.
    pub files: Vec<RadarFile>,
    /// The agent picked per conflicting file.
    pub resolved: BTreeMap<String, String>,
}

impl Radar {
    /// A radar.
    pub fn new(agents: Vec<RadarAgent>, files: Vec<RadarFile>) -> Self {
        Self {
            agents,
            files,
            resolved: BTreeMap::new(),
        }
    }

    /// Conflicting file paths.
    pub fn conflicts(&self) -> Vec<&RadarFile> {
        self.files.iter().filter(|f| f.conflicting()).collect()
    }

    /// Conflicting files not yet settled.
    pub fn open(&self) -> Vec<&RadarFile> {
        self.conflicts()
            .into_iter()
            .filter(|f| !self.resolved.contains_key(&f.path))
            .collect()
    }

    /// Picks agent `agent`'s way for `path`. Ignored for a file that does not conflict or an
    /// agent not in the clash.
    pub fn resolve(&mut self, path: &str, agent: &str) -> bool {
        let Some(f) = self.files.iter().find(|f| f.path == path) else {
            return false;
        };
        if !f.clash_agents().iter().any(|a| a == agent) {
            return false;
        }
        self.resolved.insert(path.to_string(), agent.to_string());
        true
    }

    /// Whether a cell's edit was kept: the file is not conflicting, or this agent was picked.
    pub fn kept(&self, path: &str, agent: &str) -> Option<bool> {
        let f = self.files.iter().find(|f| f.path == path)?;
        f.edit(agent)?;
        if !f.clash_agents().iter().any(|a| a == agent) {
            return Some(true);
        }
        self.resolved.get(path).map(|a| a == agent)
    }

    /// Agents still working.
    pub fn running(&self) -> usize {
        self.agents.iter().filter(|a| a.running).count()
    }

    /// Why Merge is disabled (`None` = enabled).
    pub fn merge_blocker(&self) -> Option<String> {
        let open = self.open().len();
        let run = self.running();
        match (open, run) {
            (0, 0) => None,
            (o, 0) => Some(format!(
                "{o} conflict{} to settle",
                if o == 1 { "" } else { "s" }
            )),
            (0, r) => Some(format!(
                "{r} agent{} still working",
                if r == 1 { "" } else { "s" }
            )),
            (o, r) => Some(format!(
                "{o} conflict{} · {r} working",
                if o == 1 { "" } else { "s" }
            )),
        }
    }

    /// An agent's numbers.
    pub fn numbers(&self, id: &str) -> AgentNumbers {
        let mut n = AgentNumbers::default();
        for f in &self.files {
            let Some(e) = f.edit(id) else { continue };
            n.files += 1;
            n.lines += e.len();
            if f.clash_agents().iter().any(|a| a == id) {
                match self.resolved.get(&f.path) {
                    None => n.open += 1,
                    Some(a) if a == id => n.kept += 1,
                    Some(_) => {}
                }
            }
        }
        n
    }

    /// Suggested merge order (Oxyn plus): agents with no clashes first, then by fewest open
    /// conflicts and most lines changed, so the least contested branch lands first.
    pub fn merge_order(&self) -> Vec<String> {
        let mut v: Vec<(usize, u32, usize, String)> = self
            .agents
            .iter()
            .enumerate()
            .map(|(i, a)| {
                let n = self.numbers(&a.id);
                (n.open, u32::MAX - n.lines, i, a.id.clone())
            })
            .collect();
        v.sort();
        v.into_iter().map(|x| x.3).collect()
    }

    /// Accessible text of one cell.
    pub fn cell_label(&self, path: &str, agent: &RadarAgent) -> String {
        let Some(f) = self.files.iter().find(|f| f.path == path) else {
            return String::new();
        };
        match f.edit(&agent.id) {
            None => format!("{} did not change {}", agent.name, f.path),
            Some(e) => {
                let state = match self.kept(path, &agent.id) {
                    Some(true) if f.conflicting() => ", kept",
                    Some(false) => ", not kept",
                    None => ", conflict",
                    _ => "",
                };
                let w = if e.active { ", still writing" } else { "" };
                format!(
                    "{} changed lines {} to {}{w}{state}",
                    agent.name, e.from, e.to
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FILE: &str = "use std::time::Duration;\n<<<<<<< main\nconst TTL: u64 = 300;\n||||||| base\nconst TTL: u64 = 600;\n=======\nconst TTL: u64 = cfg::ttl();\n>>>>>>> feature/refresh\nfn a() {}\n<<<<<<< main\nlog(\"a\");\n=======\nlog(\"b\");\n>>>>>>> feature/refresh\n";

    fn session() -> Merge {
        let mut f = MergeFile::from_markers("src/session.rs", FILE);
        if let Seg::Conflict(c) = &mut f.segs[1] {
            c.suggestion = Some(Suggestion {
                lines: vec!["const TTL: u64 = cfg::ttl().max(300);".into()],
                rationale: "Keep the config value but never below ours.".into(),
                confidence: Some("medium"),
            });
        }
        Merge::new(vec![
            f,
            MergeFile::from_markers("a.toml", "<<<<<<<\nx = 1\n=======\nx = 2\n>>>>>>>\n"),
        ])
    }

    #[test]
    fn parses_markers_with_base() {
        let s = parse_markers(FILE).unwrap();
        assert_eq!(s.len(), 4);
        let Seg::Conflict(c) = &s[1] else { panic!() };
        assert_eq!(c.ours, vec!["const TTL: u64 = 300;"]);
        assert_eq!(
            c.base.as_deref(),
            Some(&["const TTL: u64 = 600;".to_string()][..])
        );
        let Seg::Conflict(c2) = &s[3] else { panic!() };
        assert!(c2.base.is_none());
        assert_eq!(count_markers(FILE), 7);
    }

    #[test]
    fn parse_errors() {
        assert_eq!(parse_markers("a\n=======\n"), Err(ParseError::Stray(2)));
        assert_eq!(
            parse_markers("<<<<<<< x\na\n"),
            Err(ParseError::Unclosed(1))
        );
        assert_eq!(parse_markers("a\n>>>>>>> x\n"), Err(ParseError::Stray(2)));
        // eight '=' is content, not a marker
        assert!(parse_markers("========\n").is_ok());
    }

    #[test]
    fn picks_build_the_result() {
        let mut m = session();
        assert_eq!(m.total(0), 2);
        assert!(m.markers_left() > 0);
        assert!(m.take(0, Pick::Theirs, true));
        assert_eq!(m.focus, 1, "advanced to the next unresolved");
        assert!(m.take(
            1,
            Pick::Both {
                theirs_first: false
            },
            true
        ));
        assert_eq!(m.resolved(0), 2);
        assert_eq!(m.markers_left(), 0);
        let t = m.result_text();
        assert!(t.contains("cfg::ttl()"));
        assert!(t.contains("log(\"a\");\nlog(\"b\");"));
        assert!(m.mark_blocker().is_none());
    }

    #[test]
    fn base_agent_and_manual() {
        let mut m = session();
        assert!(m.take(0, Pick::Agent, false));
        assert!(m.result_text().contains(".max(300)"));
        assert!(!m.take(1, Pick::Base, false), "second conflict has no base");
        assert!(!m.take(1, Pick::Agent, false), "and no suggestion");
        m.take(
            1,
            Pick::Manual(vec!["<<<<<<< x".into(), "log(\"c\");".into()]),
            false,
        );
        assert!(!m.is_resolved(0, 1), "markers left keep it open");
        assert_eq!(m.mark_blocker().as_deref(), Some("1 conflict left"));
        m.take(1, Pick::Manual(vec!["log(\"c\");".into()]), false);
        assert!(m.is_resolved(0, 1));
    }

    #[test]
    fn navigation_wraps_and_skips_resolved() {
        let mut m = session();
        m.go(true);
        assert_eq!(m.focus, 1);
        m.go(true);
        assert_eq!(m.focus, 0);
        m.take(0, Pick::Ours, false);
        m.go(true);
        assert_eq!(m.focus, 1);
        m.go(false);
        assert_eq!(m.focus, 1, "only one left");
    }

    #[test]
    fn undo_mark_complete_abort() {
        let mut m = session();
        m.take(0, Pick::Ours, false);
        m.reopen(0);
        assert!(!m.is_resolved(0, 0));
        assert!(m.undo());
        assert!(m.is_resolved(0, 0));
        m.take(1, Pick::Theirs, false);
        assert!(m.mark());
        assert_eq!(m.file, 1, "moved on to the next file");
        assert_eq!(
            m.complete_blocker().as_deref(),
            Some("1 file not marked resolved")
        );
        m.take_all(Pick::Theirs);
        assert!(m.mark());
        assert!(m.complete());
        assert_eq!(m.phase, Phase::Completed);
        let mut a = session();
        a.abort("ask");
        assert_eq!(a.phase, Phase::ConfirmAbort);
        a.abort("cancel");
        assert_eq!(a.phase, Phase::Open);
        a.abort("ask");
        a.abort("confirm");
        assert_eq!(a.phase, Phase::Aborted);
    }

    #[test]
    fn keymap() {
        assert_eq!(
            merge_key("¡", "Digit1", true, false, false),
            Some(MergeCmd::Take(TakeKind::Ours))
        );
        assert_eq!(
            merge_key("ArrowDown", "ArrowDown", true, false, false),
            Some(MergeCmd::Next)
        );
        assert_eq!(
            merge_key("]", "BracketRight", false, false, false),
            Some(MergeCmd::Next)
        );
        assert_eq!(
            merge_key("z", "KeyZ", false, true, false),
            Some(MergeCmd::Undo)
        );
        assert_eq!(
            merge_key("Enter", "Enter", false, true, true),
            Some(MergeCmd::Mark)
        );
        assert_eq!(merge_key("a", "KeyA", false, false, false), None);
    }

    fn radar() -> Radar {
        Radar::new(
            vec![
                RadarAgent::new("a", "Atlas", "agent/atlas"),
                RadarAgent::new("b", "Birch", "agent/birch"),
                RadarAgent::new("c", "Cove", "agent/cove").running(true),
            ],
            vec![
                RadarFile::new(
                    "src/auth/session.rs",
                    vec![
                        RadarEdit::new("a", 10, 40).way("Refresh tokens in middleware"),
                        RadarEdit::new("b", 30, 52).way("Refresh tokens on demand"),
                    ],
                ),
                RadarFile::new(
                    "src/ui/nav.rs",
                    vec![RadarEdit::new("c", 1, 9).active(true)],
                ),
                RadarFile::new(
                    "README.md",
                    vec![RadarEdit::new("a", 1, 3), RadarEdit::new("c", 20, 22)],
                ),
            ],
        )
    }

    #[test]
    fn radar_conflicts_and_merge() {
        let mut r = radar();
        assert_eq!(r.conflicts().len(), 1);
        assert_eq!(r.files[0].overlap_lines(), 11);
        assert_eq!(r.merge_blocker().as_deref(), Some("1 conflict · 1 working"));
        assert!(!r.resolve("src/auth/session.rs", "c"), "not in the clash");
        assert!(!r.resolve("README.md", "a"), "not conflicting");
        assert!(r.resolve("src/auth/session.rs", "b"));
        assert_eq!(r.kept("src/auth/session.rs", "a"), Some(false));
        assert_eq!(r.kept("README.md", "a"), Some(true));
        assert_eq!(
            r.numbers("b"),
            AgentNumbers {
                files: 1,
                open: 0,
                kept: 1,
                lines: 23
            }
        );
        assert_eq!(r.merge_blocker().as_deref(), Some("1 agent still working"));
        r.agents[2].running = false;
        assert!(r.merge_blocker().is_none());
        assert_eq!(r.agents[0].monogram(), "A");
        assert!(
            r.cell_label("src/auth/session.rs", &r.agents[0])
                .contains("not kept")
        );
    }

    #[test]
    fn radar_merge_order() {
        let r = radar();
        let o = r.merge_order();
        assert_eq!(o[0], "c", "no clashes first");
    }
}
