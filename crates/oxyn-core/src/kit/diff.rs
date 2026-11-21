//! Diff and review models (plan #30): line and word diffs, hunks with per-hunk decisions, the
//! review state behind the diff viewer, change summaries, row diffs for tables, file revision
//! timelines, code suggestions, AI inline edits, image compare, comment threads, the review
//! center queue and the agent review card.
//!
//! Behaviour-only rebuilds of the affectUI Review Center and the Arc UI comment thread, image
//! compare, AI inline edit and code suggestion; nothing is copied from those libraries. The diff
//! itself is a plain longest-common-subsequence over lines and then over words, trimmed of the
//! common prefix and suffix first, which is plenty for agent edits (a few hundred lines).

use std::collections::{BTreeMap, BTreeSet};

use crate::ClockMs;

// ---------------------------------------------------------------------------------------------
// Line and word diff
// ---------------------------------------------------------------------------------------------

/// What one diff line is.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub enum LineKind {
    /// Unchanged context.
    Ctx,
    /// Added in the new text.
    Add,
    /// Removed from the old text.
    Del,
}

impl LineKind {
    /// Short id (`ctx` · `add` · `del`).
    pub fn id(self) -> &'static str {
        match self {
            Self::Ctx => "ctx",
            Self::Add => "add",
            Self::Del => "del",
        }
    }
    /// Gutter sign.
    pub fn sign(self) -> &'static str {
        match self {
            Self::Ctx => " ",
            Self::Add => "+",
            Self::Del => "−",
        }
    }
    /// Words for screen readers.
    pub fn spoken(self) -> &'static str {
        match self {
            Self::Ctx => "",
            Self::Add => "Added: ",
            Self::Del => "Removed: ",
        }
    }
}

/// One run of text inside a changed line; `changed` marks the words that differ.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Seg {
    /// Text.
    pub text: String,
    /// Differs from the other side.
    pub changed: bool,
}

/// One line of a diff with its old and new line numbers (1-based).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DLine {
    /// Kind.
    pub kind: LineKind,
    /// Old line number.
    pub old: Option<usize>,
    /// New line number.
    pub new: Option<usize>,
    /// Text without the newline.
    pub text: String,
    /// Word segments when this line is paired with a line on the other side (empty otherwise).
    pub words: Vec<Seg>,
}

/// Edit script between two sequences: `(kind, index in a, index in b)`; for `Add` the `a` index
/// is where it lands, for `Del` the `b` index is where it was.
pub fn lcs_ops<T: PartialEq>(a: &[T], b: &[T]) -> Vec<(LineKind, usize, usize)> {
    let mut pre = 0;
    while pre < a.len() && pre < b.len() && a[pre] == b[pre] {
        pre += 1;
    }
    let mut suf = 0;
    while suf < a.len() - pre && suf < b.len() - pre && a[a.len() - 1 - suf] == b[b.len() - 1 - suf]
    {
        suf += 1;
    }
    let (am, bm) = (&a[pre..a.len() - suf], &b[pre..b.len() - suf]);
    let (n, m) = (am.len(), bm.len());
    let mut out: Vec<(LineKind, usize, usize)> = (0..pre).map(|i| (LineKind::Ctx, i, i)).collect();
    if n * m > 4_000_000 {
        // Too big for the table: everything in the middle is replaced.
        out.extend((0..n).map(|i| (LineKind::Del, pre + i, pre)));
        out.extend((0..m).map(|j| (LineKind::Add, pre + n, pre + j)));
    } else {
        let w = m + 1;
        let mut t = vec![0u32; (n + 1) * w];
        for i in (0..n).rev() {
            for j in (0..m).rev() {
                t[i * w + j] = if am[i] == bm[j] {
                    t[(i + 1) * w + j + 1] + 1
                } else {
                    t[(i + 1) * w + j].max(t[i * w + j + 1])
                };
            }
        }
        let (mut i, mut j) = (0, 0);
        while i < n || j < m {
            if i < n && j < m && am[i] == bm[j] {
                out.push((LineKind::Ctx, pre + i, pre + j));
                i += 1;
                j += 1;
            } else if j == m || (i < n && t[(i + 1) * w + j] >= t[i * w + j + 1]) {
                out.push((LineKind::Del, pre + i, pre + j));
                i += 1;
            } else {
                out.push((LineKind::Add, pre + i, pre + j));
                j += 1;
            }
        }
    }
    let (an, bn) = (a.len(), b.len());
    out.extend((0..suf).map(|k| (LineKind::Ctx, an - suf + k, bn - suf + k)));
    out
}

/// Splits text into words, runs of spaces and single punctuation marks.
pub fn tokens(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let mut start = 0;
    let mut kind = 0u8; // 1 word, 2 space
    for (i, ch) in s.char_indices() {
        let k = if ch.is_alphanumeric() || ch == '_' {
            1
        } else if ch.is_whitespace() {
            2
        } else {
            3
        };
        if i > start && (k != kind || k == 3) {
            out.push(&s[start..i]);
            start = i;
        }
        kind = k;
    }
    if start < s.len() {
        out.push(&s[start..]);
    }
    out
}

fn push_seg(v: &mut Vec<Seg>, text: &str, changed: bool) {
    if let Some(last) = v.last_mut()
        && last.changed == changed
    {
        last.text.push_str(text);
        return;
    }
    v.push(Seg {
        text: text.to_string(),
        changed,
    });
}

/// Word diff of two lines: segments for the old line and for the new line.
pub fn word_diff(old: &str, new: &str) -> (Vec<Seg>, Vec<Seg>) {
    let (a, b) = (tokens(old), tokens(new));
    let mut l = Vec::new();
    let mut r = Vec::new();
    for (k, i, j) in lcs_ops(&a, &b) {
        match k {
            LineKind::Ctx => {
                push_seg(&mut l, a[i], false);
                push_seg(&mut r, b[j], false);
            }
            LineKind::Del => push_seg(&mut l, a[i], true),
            LineKind::Add => push_seg(&mut r, b[j], true),
        }
    }
    (l, r)
}

/// Share of the text marked changed, 0..1.
pub fn changed_share(segs: &[Seg]) -> f32 {
    let total: usize = segs.iter().map(|s| s.text.len()).sum();
    if total == 0 {
        return 0.0;
    }
    let ch: usize = segs
        .iter()
        .filter(|s| s.changed)
        .map(|s| s.text.len())
        .sum();
    ch as f32 / total as f32
}

/// Full line diff of two texts, with word segments on paired changed lines (pairs that changed
/// more than 70 % read as whole-line changes and carry no word marks).
pub fn diff_lines(old: &str, new: &str) -> Vec<DLine> {
    let a: Vec<&str> = old.lines().collect();
    let b: Vec<&str> = new.lines().collect();
    let mut out: Vec<DLine> = lcs_ops(&a, &b)
        .into_iter()
        .map(|(k, i, j)| DLine {
            kind: k,
            old: (k != LineKind::Add).then_some(i + 1),
            new: (k != LineKind::Del).then_some(j + 1),
            text: if k == LineKind::Add { b[j] } else { a[i] }.to_string(),
            words: Vec::new(),
        })
        .collect();
    // Pair each block of deletions with the additions that follow it.
    let mut i = 0;
    while i < out.len() {
        if out[i].kind != LineKind::Del {
            i += 1;
            continue;
        }
        let ds = i;
        while i < out.len() && out[i].kind == LineKind::Del {
            i += 1;
        }
        let as_ = i;
        while i < out.len() && out[i].kind == LineKind::Add {
            i += 1;
        }
        let pairs = (as_ - ds).min(i - as_);
        for p in 0..pairs {
            let (l, r) = word_diff(&out[ds + p].text, &out[as_ + p].text);
            if changed_share(&l).max(changed_share(&r)) <= 0.7 {
                out[ds + p].words = l;
                out[as_ + p].words = r;
            }
        }
    }
    out
}

/// A run of changes with its context lines.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Hunk {
    /// Index in its file.
    pub id: usize,
    /// First old line (1-based).
    pub old_start: usize,
    /// Old line count.
    pub old_len: usize,
    /// First new line.
    pub new_start: usize,
    /// New line count.
    pub new_len: usize,
    /// The nearest enclosing function or type above the hunk ("fn login"), if any.
    pub header: String,
    /// Lines, context included.
    pub lines: Vec<DLine>,
    /// Unchanged lines hidden between the previous hunk (or the top) and this one.
    pub gap_before: Vec<DLine>,
    /// Lines added.
    pub added: usize,
    /// Lines removed.
    pub removed: usize,
}

impl Hunk {
    /// `@@ -40,7 +40,8 @@`.
    pub fn range(&self) -> String {
        format!(
            "@@ -{},{} +{},{} @@",
            self.old_start, self.old_len, self.new_start, self.new_len
        )
    }
    /// Short label for buttons and announcements ("fn login" or "lines 40–47").
    pub fn label(&self) -> String {
        if self.header.is_empty() {
            format!(
                "lines {}–{}",
                self.new_start,
                self.new_start + self.new_len.saturating_sub(1)
            )
        } else {
            self.header.clone()
        }
    }
    /// Rows for the split layout: (old side, new side); deletions pair with the additions after
    /// them, context fills both sides.
    pub fn split_rows(&self) -> Vec<(Option<DLine>, Option<DLine>)> {
        split_rows(&self.lines)
    }
}

/// Rows for a split layout from diff lines.
pub fn split_rows(lines: &[DLine]) -> Vec<(Option<DLine>, Option<DLine>)> {
    let mut rows = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        match lines[i].kind {
            LineKind::Ctx => {
                rows.push((Some(lines[i].clone()), Some(lines[i].clone())));
                i += 1;
            }
            _ => {
                let mut dels = Vec::new();
                let mut adds = Vec::new();
                while i < lines.len() && lines[i].kind == LineKind::Del {
                    dels.push(lines[i].clone());
                    i += 1;
                }
                while i < lines.len() && lines[i].kind == LineKind::Add {
                    adds.push(lines[i].clone());
                    i += 1;
                }
                let n = dels.len().max(adds.len());
                let mut d = dels.into_iter();
                let mut a = adds.into_iter();
                for _ in 0..n {
                    rows.push((d.next(), a.next()));
                }
            }
        }
    }
    rows
}

fn header_of(line: &str) -> Option<String> {
    let t = line.trim_start();
    let keys = [
        "pub fn ",
        "fn ",
        "pub async fn ",
        "async fn ",
        "export async function ",
        "export function ",
        "function ",
        "def ",
        "class ",
        "impl ",
        "struct ",
        "pub struct ",
        "export const ",
        "const ",
    ];
    for k in keys {
        if let Some(rest) = t.strip_prefix(k) {
            let name: String = rest
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            if !name.is_empty() {
                let kw = k.trim().rsplit(' ').next().unwrap_or("fn");
                return Some(format!("{kw} {name}"));
            }
        }
    }
    None
}

/// Groups diff lines into hunks with `context` unchanged lines around each change.
pub fn hunks(lines: &[DLine], context: usize) -> Vec<Hunk> {
    let changed: Vec<usize> = lines
        .iter()
        .enumerate()
        .filter(|(_, l)| l.kind != LineKind::Ctx)
        .map(|(i, _)| i)
        .collect();
    let mut spans: Vec<(usize, usize)> = Vec::new();
    for &c in &changed {
        let s = c.saturating_sub(context);
        let e = (c + context + 1).min(lines.len());
        match spans.last_mut() {
            Some(last) if s <= last.1 => last.1 = last.1.max(e),
            _ => spans.push((s, e)),
        }
    }
    let mut out = Vec::new();
    let mut prev_end = 0;
    for (id, (s, e)) in spans.into_iter().enumerate() {
        let body = &lines[s..e];
        let header = lines[..s]
            .iter()
            .rev()
            .chain(body.iter().filter(|l| l.kind == LineKind::Ctx).take(1))
            .find_map(|l| header_of(&l.text))
            .unwrap_or_default();
        let old_start = body.iter().find_map(|l| l.old).unwrap_or(0);
        let new_start = body.iter().find_map(|l| l.new).unwrap_or(0);
        out.push(Hunk {
            id,
            old_start,
            old_len: body.iter().filter(|l| l.kind != LineKind::Add).count(),
            new_start,
            new_len: body.iter().filter(|l| l.kind != LineKind::Del).count(),
            header,
            gap_before: lines[prev_end..s].to_vec(),
            added: body.iter().filter(|l| l.kind == LineKind::Add).count(),
            removed: body.iter().filter(|l| l.kind == LineKind::Del).count(),
            lines: body.to_vec(),
        });
        prev_end = e;
    }
    out
}

/// How a file changed.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FileStatus {
    /// New file.
    Added,
    /// Changed in place.
    Modified,
    /// Removed.
    Deleted,
    /// Moved from the path given.
    Renamed(String),
}

impl FileStatus {
    /// One letter (A · M · D · R).
    pub fn letter(&self) -> &'static str {
        match self {
            Self::Added => "A",
            Self::Modified => "M",
            Self::Deleted => "D",
            Self::Renamed(_) => "R",
        }
    }
    /// Word.
    pub fn word(&self) -> &'static str {
        match self {
            Self::Added => "Added",
            Self::Modified => "Modified",
            Self::Deleted => "Deleted",
            Self::Renamed(_) => "Renamed",
        }
    }
    /// Short id.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Modified => "modified",
            Self::Deleted => "deleted",
            Self::Renamed(_) => "renamed",
        }
    }
}

/// Language id from a file name (`auth.rs` → `rust`).
pub fn lang_of(path: &str) -> &'static str {
    match path.rsplit('.').next().unwrap_or("") {
        "rs" => "rust",
        "ts" => "ts",
        "tsx" => "tsx",
        "js" | "mjs" => "ts",
        "py" => "py",
        "sh" => "sh",
        "json" => "json",
        "toml" => "toml",
        _ => "text",
    }
}

/// One file of a change set.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct FileDiff {
    /// Path.
    pub path: String,
    /// Status.
    pub status: FileStatus,
    /// Language id for highlighting.
    pub lang: &'static str,
    /// Every line of the diff.
    pub full: Vec<DLine>,
    /// Hunks (3 lines of context by default).
    pub hunks: Vec<Hunk>,
    /// Unchanged lines after the last hunk.
    pub tail: Vec<DLine>,
    /// Lines added.
    pub added: usize,
    /// Lines removed.
    pub removed: usize,
}

impl FileDiff {
    /// Diffs `old` against `new` with three lines of context.
    pub fn new(path: &str, old: &str, new: &str) -> Self {
        Self::with_context(path, old, new, 3)
    }
    /// Diffs with `context` lines around each change.
    pub fn with_context(path: &str, old: &str, new: &str, context: usize) -> Self {
        let full = diff_lines(old, new);
        let hs = hunks(&full, context);
        let covered = hs
            .last()
            .map(|h| {
                h.gap_before.len()
                    + h.lines.len()
                    + hs[..hs.len() - 1]
                        .iter()
                        .map(|x| x.gap_before.len() + x.lines.len())
                        .sum::<usize>()
            })
            .unwrap_or(0);
        let tail = if hs.is_empty() {
            Vec::new()
        } else {
            full[covered..].to_vec()
        };
        let status = if old.is_empty() && !new.is_empty() {
            FileStatus::Added
        } else if new.is_empty() && !old.is_empty() {
            FileStatus::Deleted
        } else {
            FileStatus::Modified
        };
        Self {
            path: path.to_string(),
            status,
            lang: lang_of(path),
            added: full.iter().filter(|l| l.kind == LineKind::Add).count(),
            removed: full.iter().filter(|l| l.kind == LineKind::Del).count(),
            full,
            hunks: hs,
            tail,
        }
    }
    /// Marks the file as moved from `from`.
    pub fn renamed(mut self, from: &str) -> Self {
        self.status = FileStatus::Renamed(from.to_string());
        self
    }
    /// File name without folders.
    pub fn name(&self) -> &str {
        self.path.rsplit('/').next().unwrap_or(&self.path)
    }
    /// Folder part ("src/auth"), empty at the root.
    pub fn dir(&self) -> &str {
        self.path
            .rsplit_once('/')
            .map(|(d, _)| d)
            .unwrap_or_default()
    }
    /// The file with only the hunks `keep` says yes to applied; the rest stays as before.
    pub fn apply(&self, keep: impl Fn(usize) -> bool) -> String {
        let mut out: Vec<&str> = Vec::new();
        let mut idx = 0;
        let mut owner: Vec<Option<usize>> = vec![None; self.full.len()];
        for h in &self.hunks {
            idx += h.gap_before.len();
            for k in 0..h.lines.len() {
                owner[idx + k] = Some(h.id);
            }
            idx += h.lines.len();
        }
        for (i, l) in self.full.iter().enumerate() {
            let yes = owner[i].map(&keep).unwrap_or(false);
            match l.kind {
                LineKind::Ctx => out.push(&l.text),
                LineKind::Add if yes => out.push(&l.text),
                LineKind::Del if !yes => out.push(&l.text),
                _ => {}
            }
        }
        out.join("\n")
    }
}

/// `+12 −3` (only the parts that are not zero; `0` for none).
pub fn stat_label(added: usize, removed: usize) -> String {
    match (added, removed) {
        (0, 0) => "0".into(),
        (a, 0) => format!("+{a}"),
        (0, r) => format!("−{r}"),
        (a, r) => format!("+{a} −{r}"),
    }
}

/// GitHub-style blocks for a stat: (added, removed, neutral), `n` in total.
pub fn stat_blocks(added: usize, removed: usize, n: usize) -> (usize, usize, usize) {
    let total = added + removed;
    if total == 0 {
        return (0, 0, n);
    }
    let filled = n.min(total.max(1));
    let mut a = ((added as f32 / total as f32) * filled as f32).round() as usize;
    if added > 0 && a == 0 {
        a = 1;
    }
    if removed > 0 && a == filled {
        a = filled.saturating_sub(1);
    }
    let r = filled - a;
    (a, r, n - filled)
}

// ---------------------------------------------------------------------------------------------
// Review state
// ---------------------------------------------------------------------------------------------

/// Whether the edits are still a proposal or already on disk; it decides the words.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum ReviewMode {
    /// Nothing written yet: Accept / Reject.
    #[default]
    Proposed,
    /// Already applied: Keep / Undo.
    Applied,
}

impl ReviewMode {
    /// From an id (`applied`, anything else is proposed).
    pub fn from_id(s: &str) -> Self {
        if s == "applied" {
            Self::Applied
        } else {
            Self::Proposed
        }
    }
    /// The yes word.
    pub fn yes(self) -> &'static str {
        match self {
            Self::Proposed => "Accept",
            Self::Applied => "Keep",
        }
    }
    /// The no word.
    pub fn no(self) -> &'static str {
        match self {
            Self::Proposed => "Reject",
            Self::Applied => "Undo",
        }
    }
    /// The settled yes word.
    pub fn yes_done(self) -> &'static str {
        match self {
            Self::Proposed => "Accepted",
            Self::Applied => "Kept",
        }
    }
    /// The settled no word.
    pub fn no_done(self) -> &'static str {
        match self {
            Self::Proposed => "Rejected",
            Self::Applied => "Undone",
        }
    }
}

/// A hunk's decision.
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Decision {
    /// Not decided.
    #[default]
    Pending,
    /// Accepted / kept.
    Yes,
    /// Rejected / undone.
    No,
}

impl Decision {
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Yes => "yes",
            Self::No => "no",
        }
    }
}

/// A comment on one line, waiting to be sent to the agent.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct LineComment {
    /// File index.
    pub file: usize,
    /// Hunk index.
    pub hunk: usize,
    /// New line number (old line number for a deleted line).
    pub line: usize,
    /// Text.
    pub text: String,
}

/// A hunk address: (file, hunk).
pub type HunkRef = (usize, usize);

/// The review behind the diff viewer: decisions per hunk with undo, viewed files, draft line
/// comments and the cursor that `j` / `k` and `n` / `p` move.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Review {
    /// Files in first-touch order.
    pub files: Vec<FileDiff>,
    /// Proposal or applied.
    pub mode: ReviewMode,
    /// Decisions (missing = pending).
    pub decisions: BTreeMap<HunkRef, Decision>,
    /// Files marked viewed.
    pub viewed: BTreeSet<usize>,
    /// Draft comments.
    pub comments: Vec<LineComment>,
    /// Current hunk.
    pub cursor: HunkRef,
    history: Vec<Vec<(HunkRef, Decision)>>,
}

impl Review {
    /// A review over `files`.
    pub fn new(files: Vec<FileDiff>, mode: ReviewMode) -> Self {
        Self {
            files,
            mode,
            ..Self::default()
        }
    }
    /// Every hunk in order.
    pub fn hunk_refs(&self) -> Vec<HunkRef> {
        self.files
            .iter()
            .enumerate()
            .flat_map(|(f, d)| d.hunks.iter().map(move |h| (f, h.id)))
            .collect()
    }
    /// A hunk's decision.
    pub fn decision(&self, r: HunkRef) -> Decision {
        self.decisions.get(&r).copied().unwrap_or_default()
    }
    fn record(&mut self, refs: &[HunkRef], d: Decision) -> usize {
        let mut step = Vec::new();
        for &r in refs {
            let before = self.decision(r);
            if before != d {
                step.push((r, before));
                self.decisions.insert(r, d);
            }
        }
        let n = step.len();
        if n > 0 {
            self.history.push(step);
        }
        n
    }
    /// Decides one hunk and moves the cursor to the next pending one. Returns true if it changed.
    pub fn decide(&mut self, r: HunkRef, d: Decision) -> bool {
        let changed = self.record(&[r], d) > 0;
        if changed && d != Decision::Pending {
            self.cursor = r;
            if let Some(n) = self.next_pending(r) {
                self.cursor = n;
            }
        }
        changed
    }
    /// Decides every pending hunk of a file.
    pub fn decide_file(&mut self, f: usize, d: Decision) -> usize {
        let refs: Vec<HunkRef> = self
            .hunk_refs()
            .into_iter()
            .filter(|r| r.0 == f && self.decision(*r) == Decision::Pending)
            .collect();
        self.record(&refs, d)
    }
    /// Decides every pending hunk.
    pub fn decide_all(&mut self, d: Decision) -> usize {
        let refs: Vec<HunkRef> = self
            .hunk_refs()
            .into_iter()
            .filter(|r| self.decision(*r) == Decision::Pending)
            .collect();
        self.record(&refs, d)
    }
    /// Takes back the last decision (one hunk, a file or all). Returns the hunks restored.
    pub fn undo(&mut self) -> usize {
        let Some(step) = self.history.pop() else {
            return 0;
        };
        for (r, d) in &step {
            if *d == Decision::Pending {
                self.decisions.remove(r);
            } else {
                self.decisions.insert(*r, *d);
            }
        }
        if let Some((r, _)) = step.first() {
            self.cursor = *r;
        }
        step.len()
    }
    /// Whether there is anything to undo.
    pub fn can_undo(&self) -> bool {
        !self.history.is_empty()
    }
    /// The next pending hunk after `r`, wrapping.
    pub fn next_pending(&self, r: HunkRef) -> Option<HunkRef> {
        let all = self.hunk_refs();
        let at = all.iter().position(|x| *x == r).unwrap_or(0);
        (1..=all.len())
            .map(|k| all[(at + k) % all.len()])
            .find(|x| self.decision(*x) == Decision::Pending)
    }
    /// Moves the cursor by `delta` hunks (clamped).
    pub fn step(&mut self, delta: isize) {
        let all = self.hunk_refs();
        if all.is_empty() {
            return;
        }
        let at = all.iter().position(|x| *x == self.cursor).unwrap_or(0) as isize;
        let to = (at + delta).clamp(0, all.len() as isize - 1) as usize;
        self.cursor = all[to];
    }
    /// Moves the cursor to the first hunk of the next / previous file (clamped).
    pub fn step_file(&mut self, delta: isize) {
        let n = self.files.len() as isize;
        let mut f = self.cursor.0 as isize;
        loop {
            f += delta;
            if f < 0 || f >= n {
                return;
            }
            if !self.files[f as usize].hunks.is_empty() {
                self.cursor = (f as usize, 0);
                return;
            }
        }
    }
    /// 1-based position of the cursor and the hunk count.
    pub fn position(&self) -> (usize, usize) {
        let all = self.hunk_refs();
        let at = all.iter().position(|x| *x == self.cursor).unwrap_or(0);
        (if all.is_empty() { 0 } else { at + 1 }, all.len())
    }
    /// (pending, yes, no) counts.
    pub fn counts(&self) -> (usize, usize, usize) {
        let mut c = (0, 0, 0);
        for r in self.hunk_refs() {
            match self.decision(r) {
                Decision::Pending => c.0 += 1,
                Decision::Yes => c.1 += 1,
                Decision::No => c.2 += 1,
            }
        }
        c
    }
    /// Lines added / removed across hunks that are not rejected (the count updates as you
    /// decide).
    pub fn live_stat(&self) -> (usize, usize) {
        let mut s = (0, 0);
        for (f, d) in self.files.iter().enumerate() {
            for h in &d.hunks {
                if self.decision((f, h.id)) != Decision::No {
                    s.0 += h.added;
                    s.1 += h.removed;
                }
            }
        }
        s
    }
    /// Per-file (pending, yes, no).
    pub fn file_counts(&self, f: usize) -> (usize, usize, usize) {
        let mut c = (0, 0, 0);
        if let Some(d) = self.files.get(f) {
            for h in &d.hunks {
                match self.decision((f, h.id)) {
                    Decision::Pending => c.0 += 1,
                    Decision::Yes => c.1 += 1,
                    Decision::No => c.2 += 1,
                }
            }
        }
        c
    }
    /// Toggles a file's Viewed mark.
    pub fn toggle_viewed(&mut self, f: usize) {
        if !self.viewed.remove(&f) {
            self.viewed.insert(f);
        }
    }
    /// Adds a draft comment.
    pub fn comment(&mut self, file: usize, hunk: usize, line: usize, text: &str) {
        let text = text.trim();
        if !text.is_empty() {
            self.comments.push(LineComment {
                file,
                hunk,
                line,
                text: text.to_string(),
            });
        }
    }
    /// Takes every draft comment (Send to agent).
    pub fn send_comments(&mut self) -> Vec<LineComment> {
        std::mem::take(&mut self.comments)
    }
    /// A file as it stands after the decisions (applied mode keeps pending hunks; proposed mode
    /// writes only accepted ones).
    pub fn result(&self, f: usize) -> String {
        let pending_yes = self.mode == ReviewMode::Applied;
        self.files[f].apply(|h| match self.decision((f, h)) {
            Decision::Yes => true,
            Decision::No => false,
            Decision::Pending => pending_yes,
        })
    }
    /// One line for a polite live region after a decision.
    pub fn announce(&self, r: HunkRef, d: Decision) -> String {
        let label = self
            .files
            .get(r.0)
            .and_then(|f| f.hunks.get(r.1))
            .map(|h| h.label())
            .unwrap_or_default();
        let word = match d {
            Decision::Yes => self.mode.yes_done(),
            Decision::No => self.mode.no_done(),
            Decision::Pending => "Restored",
        };
        let left = self.counts().0;
        format!(
            "{word}: {label}. {left} {} to review",
            if left == 1 { "change" } else { "changes" }
        )
    }
}

/// Totals for a change set: (files, added, removed).
pub fn change_totals(files: &[FileDiff]) -> (usize, usize, usize) {
    (
        files.len(),
        files.iter().map(|f| f.added).sum(),
        files.iter().map(|f| f.removed).sum(),
    )
}

/// "4 files changed" headline.
pub fn files_changed(n: usize) -> String {
    format!("{n} {} changed", if n == 1 { "file" } else { "files" })
}

// ---------------------------------------------------------------------------------------------
// Diff table (row-level changes)
// ---------------------------------------------------------------------------------------------

/// How a row changed.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RowKind {
    /// New key.
    Added,
    /// Key gone.
    Removed,
    /// Value changed.
    Changed,
    /// Same.
    Same,
}

impl RowKind {
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Added => "added",
            Self::Removed => "removed",
            Self::Changed => "changed",
            Self::Same => "same",
        }
    }
    /// Sign.
    pub fn sign(self) -> &'static str {
        match self {
            Self::Added => "+",
            Self::Removed => "−",
            Self::Changed => "~",
            Self::Same => "",
        }
    }
}

/// One row of a table diff.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct RowChange {
    /// Key.
    pub key: String,
    /// Old value.
    pub before: Option<String>,
    /// New value.
    pub after: Option<String>,
    /// Kind.
    pub kind: RowKind,
}

/// Diffs two keyed tables; order follows `after`, with removed rows at their old place.
pub fn table_diff(before: &[(&str, &str)], after: &[(&str, &str)]) -> Vec<RowChange> {
    let bk: Vec<&str> = before.iter().map(|x| x.0).collect();
    let ak: Vec<&str> = after.iter().map(|x| x.0).collect();
    fn find<'a>(rows: &[(&'a str, &'a str)], k: &str) -> Option<&'a str> {
        rows.iter().find(|x| x.0 == k).map(|x| x.1)
    }
    let mut out = Vec::new();
    for (kind, i, j) in lcs_ops(&bk, &ak) {
        match kind {
            LineKind::Del => {
                let k = bk[i];
                if find(after, k).is_none() {
                    out.push(RowChange {
                        key: k.into(),
                        before: Some(before[i].1.into()),
                        after: None,
                        kind: RowKind::Removed,
                    });
                }
            }
            LineKind::Add | LineKind::Ctx => {
                let k = ak[j];
                let old = find(before, k);
                let new = after[j].1;
                let rk = match old {
                    None => RowKind::Added,
                    Some(o) if o == new => RowKind::Same,
                    Some(_) => RowKind::Changed,
                };
                out.push(RowChange {
                    key: k.into(),
                    before: old.map(Into::into),
                    after: Some(new.into()),
                    kind: rk,
                });
            }
        }
    }
    out
}

// ---------------------------------------------------------------------------------------------
// Diff timeline (one file across revisions)
// ---------------------------------------------------------------------------------------------

/// One saved version of a file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Revision {
    /// Id.
    pub id: String,
    /// What changed, in a few words.
    pub label: String,
    /// Who wrote it ("Agent", "You").
    pub who: String,
    /// When.
    pub at: ClockMs,
    /// Full text.
    pub text: String,
}

/// (added, removed) of each revision against the one before (the first against empty).
pub fn revision_stats(revs: &[Revision]) -> Vec<(usize, usize)> {
    revs.iter()
        .enumerate()
        .map(|(i, r)| {
            let prev = if i == 0 {
                ""
            } else {
                revs[i - 1].text.as_str()
            };
            let d = diff_lines(prev, &r.text);
            (
                d.iter().filter(|l| l.kind == LineKind::Add).count(),
                d.iter().filter(|l| l.kind == LineKind::Del).count(),
            )
        })
        .collect()
}

/// The diff between two revisions (`from` < `to` is not required).
pub fn revision_diff(path: &str, revs: &[Revision], from: usize, to: usize) -> FileDiff {
    let a = revs.get(from).map(|r| r.text.as_str()).unwrap_or("");
    let b = revs.get(to).map(|r| r.text.as_str()).unwrap_or("");
    FileDiff::with_context(path, a, b, 2)
}

// ---------------------------------------------------------------------------------------------
// Code suggestion
// ---------------------------------------------------------------------------------------------

/// One proposed change: replace `remove` lines from the 0-based `start` with `insert`.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct SugHunk {
    /// Id.
    pub id: String,
    /// First line replaced (0-based).
    pub start: usize,
    /// Lines removed.
    pub remove: usize,
    /// Lines inserted.
    pub insert: Vec<String>,
    /// Two to six words.
    pub summary: String,
}

/// Where a suggestion is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum SugStatus {
    /// Reading the file.
    Thinking,
    /// Hunks streaming in.
    Streaming,
    /// Waiting for decisions.
    Ready,
    /// The model failed.
    Error,
}

impl SugStatus {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "thinking" => Self::Thinking,
            "streaming" => Self::Streaming,
            "error" => Self::Error,
            _ => Self::Ready,
        }
    }
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Thinking => "thinking",
            Self::Streaming => "streaming",
            Self::Ready => "ready",
            Self::Error => "error",
        }
    }
}

/// Thinking time before the first hunk, ms.
pub const SUG_THINK_MS: u64 = 800;
/// One inserted line every, ms.
pub const SUG_LINE_MS: u64 = 38;

/// One row of a suggestion's editor panel.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum SugRow {
    /// A file line (1-based number).
    Line(usize, String),
    /// A removed line of a hunk, with its old number.
    Del(String, usize, String),
    /// An added line of a hunk.
    Add(String, String),
    /// The Accept / Reject bar under a pending hunk.
    Bar(String),
}

/// A code suggestion over one file.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Suggestion {
    /// Original lines.
    pub code: Vec<String>,
    /// Hunks, sorted, overlaps dropped.
    pub hunks: Vec<SugHunk>,
    /// Decisions by hunk id (true = accepted).
    pub decisions: BTreeMap<String, bool>,
}

impl Suggestion {
    /// A suggestion; hunks are sorted and any hunk overlapping an earlier one is ignored.
    pub fn new(code: &str, mut hunks: Vec<SugHunk>) -> Self {
        hunks.sort_by_key(|h| h.start);
        let mut kept: Vec<SugHunk> = Vec::new();
        let mut end = 0;
        for h in hunks {
            if !kept.is_empty() && h.start < end {
                continue;
            }
            end = h.start + h.remove.max(if h.remove == 0 { 0 } else { 1 });
            kept.push(h);
        }
        Self {
            code: code.lines().map(String::from).collect(),
            hunks: kept,
            decisions: BTreeMap::new(),
        }
    }
    /// Lines that stream in total.
    pub fn insert_lines(&self) -> usize {
        self.hunks.iter().map(|h| h.insert.len()).sum()
    }
    /// The status and how many inserted lines show `elapsed` ms after the request.
    pub fn play(&self, elapsed: u64) -> (SugStatus, usize) {
        if elapsed < SUG_THINK_MS {
            return (SugStatus::Thinking, 0);
        }
        let n = ((elapsed - SUG_THINK_MS) / SUG_LINE_MS) as usize;
        if n >= self.insert_lines() {
            (SugStatus::Ready, self.insert_lines())
        } else {
            (SugStatus::Streaming, n)
        }
    }
    /// Decision for a hunk.
    pub fn decision(&self, id: &str) -> Option<bool> {
        self.decisions.get(id).copied()
    }
    /// Decides one hunk.
    pub fn decide(&mut self, id: &str, accept: bool) {
        self.decisions.insert(id.to_string(), accept);
    }
    /// Decides every pending hunk; returns their ids.
    pub fn decide_all(&mut self, accept: bool) -> Vec<String> {
        let ids: Vec<String> = self
            .hunks
            .iter()
            .filter(|h| !self.decisions.contains_key(&h.id))
            .map(|h| h.id.clone())
            .collect();
        for id in &ids {
            self.decisions.insert(id.clone(), accept);
        }
        ids
    }
    /// Pending hunks.
    pub fn pending(&self) -> usize {
        self.hunks
            .iter()
            .filter(|h| !self.decisions.contains_key(&h.id))
            .count()
    }
    /// Every hunk decided.
    pub fn resolved(&self) -> bool {
        !self.hunks.is_empty() && self.pending() == 0
    }
    /// Running count for the file tab: lines added / removed by hunks not rejected.
    pub fn stat(&self) -> (usize, usize) {
        self.hunks
            .iter()
            .filter(|h| self.decision(&h.id) != Some(false))
            .fold((0, 0), |s, h| (s.0 + h.insert.len(), s.1 + h.remove))
    }
    /// Rows to draw with `shown` inserted lines revealed (pass `usize::MAX` for all). Decided
    /// hunks fold: accepted shows only the new lines as file lines, rejected the old ones.
    pub fn rows(&self, shown: usize, show_bars: bool) -> Vec<SugRow> {
        let mut out = Vec::new();
        let mut left = shown;
        let mut ln = 0; // 0-based original line
        let mut no = 1; // displayed number
        for h in &self.hunks {
            while ln < h.start.min(self.code.len()) {
                out.push(SugRow::Line(no, self.code[ln].clone()));
                ln += 1;
                no += 1;
            }
            let old: Vec<String> =
                self.code[ln.min(self.code.len())..(ln + h.remove).min(self.code.len())].to_vec();
            match self.decision(&h.id) {
                Some(true) => {
                    for s in &h.insert {
                        out.push(SugRow::Line(no, s.clone()));
                        no += 1;
                    }
                }
                Some(false) => {
                    for s in old {
                        out.push(SugRow::Line(no, s));
                        no += 1;
                    }
                }
                None => {
                    for (k, s) in old.into_iter().enumerate() {
                        out.push(SugRow::Del(h.id.clone(), ln + k + 1, s));
                    }
                    let take = h.insert.len().min(left);
                    left -= take;
                    for s in h.insert.iter().take(take) {
                        out.push(SugRow::Add(h.id.clone(), s.clone()));
                    }
                    no += h.remove;
                    if show_bars {
                        out.push(SugRow::Bar(h.id.clone()));
                    }
                }
            }
            ln += h.remove;
        }
        while ln < self.code.len() {
            out.push(SugRow::Line(no, self.code[ln].clone()));
            ln += 1;
            no += 1;
        }
        out
    }
    /// The file as it stands: accepted hunks applied, the rest original.
    pub fn apply(&self) -> String {
        let mut out: Vec<String> = Vec::new();
        let mut ln = 0;
        for h in &self.hunks {
            while ln < h.start.min(self.code.len()) {
                out.push(self.code[ln].clone());
                ln += 1;
            }
            if self.decision(&h.id) == Some(true) {
                out.extend(h.insert.iter().cloned());
            } else {
                out.extend(
                    self.code[ln.min(self.code.len())..(ln + h.remove).min(self.code.len())]
                        .iter()
                        .cloned(),
                );
            }
            ln += h.remove;
        }
        out.extend(self.code[ln.min(self.code.len())..].iter().cloned());
        out.join("\n")
    }
    /// "3 changes to review" / "All changes applied" / "Kept 1 of 3".
    pub fn status_line(&self) -> String {
        let p = self.pending();
        if p > 0 {
            return format!(
                "{p} {} to review",
                if p == 1 { "change" } else { "changes" }
            );
        }
        let acc = self.decisions.values().filter(|x| **x).count();
        match (acc, self.hunks.len()) {
            (0, _) => "Suggestion rejected".into(),
            (a, n) if a == n => "All changes applied".into(),
            (a, n) => format!("Applied {a} of {n} changes"),
        }
    }
}

// ---------------------------------------------------------------------------------------------
// AI inline edit (word diff of a rewrite)
// ---------------------------------------------------------------------------------------------

/// A piece of an inline rewrite.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Chunk {
    /// Text both versions share.
    Same(String),
    /// A change: what goes and what comes.
    Change {
        /// Id (order).
        id: usize,
        /// Removed text.
        del: String,
        /// Inserted text.
        ins: String,
    },
}

/// Word chunks from `old` to `new`; neighbouring changes separated by one space merge so a
/// rewrite reads as a few phrases, not many single words.
pub fn rewrite_chunks(old: &str, new: &str) -> Vec<Chunk> {
    let (a, b) = (tokens(old), tokens(new));
    let mut raw: Vec<(bool, String, String)> = Vec::new(); // (same, del, ins)
    for (k, i, j) in lcs_ops(&a, &b) {
        let (same, d, n) = match k {
            LineKind::Ctx => (true, a[i], b[j]),
            LineKind::Del => (false, a[i], ""),
            LineKind::Add => (false, "", b[j]),
        };
        match raw.last_mut() {
            Some(last) if last.0 == same => {
                last.1.push_str(d);
                if !same {
                    last.2.push_str(n);
                }
            }
            _ => raw.push((
                same,
                d.to_string(),
                if same { String::new() } else { n.into() },
            )),
        }
    }
    // Merge change · single-space same · change.
    let mut merged: Vec<(bool, String, String)> = Vec::new();
    let mut i = 0;
    while i < raw.len() {
        if !raw[i].0
            && i + 2 < raw.len()
            && raw[i + 1].0
            && raw[i + 1].1.trim().is_empty()
            && raw[i + 1].1.len() <= 1
            && !raw[i + 2].0
        {
            let sp = raw[i + 1].1.clone();
            let d = format!("{}{sp}{}", raw[i].1, raw[i + 2].1);
            let n = format!("{}{sp}{}", raw[i].2, raw[i + 2].2);
            raw[i + 2] = (false, d, n);
            i += 2;
            continue;
        }
        merged.push(raw[i].clone());
        i += 1;
    }
    let mut id = 0;
    merged
        .into_iter()
        .map(|(same, d, n)| {
            if same {
                Chunk::Same(d)
            } else {
                id += 1;
                Chunk::Change {
                    id: id - 1,
                    del: d,
                    ins: n,
                }
            }
        })
        .collect()
}

/// The text after review: an accepted change (`Some(true)`) takes its new words; a rejected or
/// undecided one keeps the old.
pub fn rewrite_apply(chunks: &[Chunk], decisions: &[Option<bool>]) -> String {
    chunks
        .iter()
        .map(|c| match c {
            Chunk::Same(t) => t.as_str(),
            Chunk::Change { id, del, ins } => {
                if decisions.get(*id).copied().flatten() == Some(true) {
                    ins.as_str()
                } else {
                    del.as_str()
                }
            }
        })
        .collect()
}

/// The next undecided change after `from` (wrapping), if any.
pub fn rewrite_next(decisions: &[Option<bool>], from: usize) -> Option<usize> {
    let n = decisions.len();
    (1..=n)
        .map(|k| (from + k) % n.max(1))
        .find(|&i| decisions.get(i).is_some_and(|d| d.is_none()))
}

/// Number of change chunks.
pub fn change_count(chunks: &[Chunk]) -> usize {
    chunks
        .iter()
        .filter(|c| matches!(c, Chunk::Change { .. }))
        .count()
}

/// Text with each change accepted (`Some(true)`), rejected (`Some(false)`) or, when undecided,
/// as `pending_accept` says.
pub fn apply_chunks(
    chunks: &[Chunk],
    decisions: &BTreeMap<usize, bool>,
    pending_accept: bool,
) -> String {
    let mut s = String::new();
    for c in chunks {
        match c {
            Chunk::Same(t) => s.push_str(t),
            Chunk::Change { id, del, ins } => {
                if decisions.get(id).copied().unwrap_or(pending_accept) {
                    s.push_str(ins);
                } else {
                    s.push_str(del);
                }
            }
        }
    }
    s
}

/// One chunk of streamed words every, ms.
pub const EDIT_CHUNK_MS: u64 = 34;

/// How many words of the rewrite are on screen `elapsed` ms into the stream (three a chunk).
pub fn words_shown(elapsed: u64, speed: f32) -> usize {
    ((elapsed as f32 * speed.max(0.1) / EDIT_CHUNK_MS as f32) as usize) * 3
}

/// For each chunk, whether the stream has reached it when `words` words of the new text are
/// out: a change is visible once the stream passes its start.
pub fn chunks_reached(chunks: &[Chunk], words: usize) -> Vec<bool> {
    let mut seen = 0usize;
    chunks
        .iter()
        .map(|c| {
            let text = match c {
                Chunk::Same(t) => t,
                Chunk::Change { ins, .. } => ins,
            };
            let reached = seen < words || (seen == 0 && words > 0);
            seen += text.split_whitespace().count();
            reached
        })
        .collect()
}

/// A quick action in the inline-edit prompt.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct QuickAction {
    /// Id.
    pub id: &'static str,
    /// Label.
    pub label: &'static str,
    /// Instruction sent to the model.
    pub instruction: &'static str,
}

/// The default quick actions.
pub const QUICK_ACTIONS: [QuickAction; 5] = [
    QuickAction {
        id: "shorten",
        label: "Shorten",
        instruction: "Make this shorter",
    },
    QuickAction {
        id: "clearer",
        label: "Make clearer",
        instruction: "Make this easier to follow",
    },
    QuickAction {
        id: "friendlier",
        label: "Friendlier tone",
        instruction: "Make this warmer",
    },
    QuickAction {
        id: "formal",
        label: "More formal",
        instruction: "Make this more formal",
    },
    QuickAction {
        id: "grammar",
        label: "Fix grammar",
        instruction: "Fix spelling and grammar",
    },
];

/// Where an inline edit is.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum EditPhase {
    /// Reading.
    Idle,
    /// Prompt open under the selection.
    Prompt,
    /// Rewrite streaming in.
    Streaming,
    /// Changes waiting for decisions.
    Review,
    /// Applied receipt.
    Applied,
    /// The model failed.
    Error,
}

impl EditPhase {
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Prompt => "prompt",
            Self::Streaming => "streaming",
            Self::Review => "review",
            Self::Applied => "applied",
            Self::Error => "error",
        }
    }
}

fn swap_words(text: &str, pairs: &[(&str, &str)]) -> String {
    let mut s = text.to_string();
    for (a, b) in pairs {
        s = s.replace(a, b);
    }
    s
}

fn tidy(s: &str) -> String {
    let mut out = s.split_whitespace().collect::<Vec<_>>().join(" ");
    out = out.replace(" ,", ",").replace(" .", ".");
    let mut c = out.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => out,
    }
}

/// A deterministic sample rewrite for a quick action (the preview has no model). Free
/// instructions (any other id) make the text clearer and shorter.
pub fn scripted_rewrite(action: &str, text: &str) -> String {
    let filler = [
        ("really ", ""),
        ("very ", ""),
        ("just ", ""),
        ("actually ", ""),
        ("basically ", ""),
        ("in order to ", "to "),
        ("at this point in time", "now"),
        ("a lot of ", "many "),
        ("due to the fact that", "because"),
    ];
    let typos = [
        ("teh ", "the "),
        ("alot", "a lot"),
        ("recieve", "receive"),
        ("seperate", "separate"),
        (" i ", " I "),
        ("its ready", "it's ready"),
        ("dont ", "don't "),
    ];
    let out = match action {
        "shorten" => swap_words(text, &filler),
        "clearer" => swap_words(
            text,
            &[
                ("utilize", "use"),
                ("leverage", "use"),
                ("in the event that", "if"),
                ("prior to", "before"),
                ("subsequent to", "after"),
                ("a lot of ", "many "),
                ("due to the fact that", "because"),
            ],
        ),
        "friendlier" => {
            let t = swap_words(
                text,
                &[
                    ("Please note that ", ""),
                    ("must ", "can "),
                    ("required", "helpful"),
                    ("Users", "Everyone"),
                    ("users", "everyone"),
                ],
            );
            format!("Good news: {}", t.trim_start())
        }
        "formal" => swap_words(
            text,
            &[
                ("we'll", "we will"),
                ("We'll", "We will"),
                ("don't", "do not"),
                ("can't", "cannot"),
                ("it's", "it is"),
                ("It's", "It is"),
                ("a lot of ", "a significant number of "),
                ("get ", "obtain "),
                ("kids", "children"),
            ],
        ),
        "grammar" => swap_words(text, &typos),
        _ => swap_words(&swap_words(text, &filler), &typos),
    };
    let mut t = tidy(&out);
    if !t.is_empty() && !t.ends_with(['.', '!', '?']) {
        t.push('.');
    }
    t
}

/// A short continuation for the end of a paragraph (ghost text), or `None`.
pub fn scripted_completion(paragraph: &str) -> Option<String> {
    let p = paragraph.trim_end();
    if p.is_empty() {
        return None;
    }
    let lower = p.to_lowercase();
    let s = if lower.contains("launch") {
        " We will share the date once the beta feedback is in."
    } else if lower.contains("data") || lower.contains("privacy") {
        " Nothing leaves your workspace without a review."
    } else {
        " Questions go to the team channel."
    };
    Some(s.to_string())
}

// ---------------------------------------------------------------------------------------------
// Image compare
// ---------------------------------------------------------------------------------------------

/// How two images are compared.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CompareMode {
    /// Divider reveal.
    Slider,
    /// Side by side.
    Side,
    /// After over before at an opacity.
    Onion,
    /// Difference blend: what changed lights up.
    Difference,
}

impl CompareMode {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "side" => Self::Side,
            "onion" => Self::Onion,
            "difference" => Self::Difference,
            _ => Self::Slider,
        }
    }
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Slider => "slider",
            Self::Side => "side",
            Self::Onion => "onion",
            Self::Difference => "difference",
        }
    }
}

/// Divider position after a key: arrows 1 % (10 with Shift), Page Up / Down 10 %, Home / End
/// the ends. `None` for keys it does not handle.
pub fn compare_key(pos: f32, key: &str, shift: bool) -> Option<f32> {
    let step = if shift { 10.0 } else { 1.0 };
    let p = match key {
        "ArrowLeft" | "ArrowUp" => pos - step,
        "ArrowRight" | "ArrowDown" => pos + step,
        "PageUp" => pos - 10.0,
        "PageDown" => pos + 10.0,
        "Home" => 0.0,
        "End" => 100.0,
        _ => return None,
    };
    Some(p.clamp(0.0, 100.0))
}

/// `aria-valuetext`: "40% after".
pub fn compare_valuetext(pos: f32) -> String {
    format!("{}% after", (100.0 - pos).round() as i32)
}

/// Caption opacity: a caption fades out as the divider reaches it (`before` sits at 0 %).
pub fn caption_opacity(pos: f32, before: bool) -> f32 {
    let d = if before { pos } else { 100.0 - pos };
    (d / 14.0).clamp(0.0, 1.0)
}

// ---------------------------------------------------------------------------------------------
// Comment thread
// ---------------------------------------------------------------------------------------------

/// Someone who comments.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Person {
    /// Id.
    pub id: String,
    /// Display name.
    pub name: String,
    /// An agent (draws a spark instead of initials).
    pub agent: bool,
}

impl Person {
    /// A person.
    pub fn new(id: &str, name: &str) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            agent: false,
        }
    }
    /// An agent.
    pub fn agent(id: &str, name: &str) -> Self {
        Self {
            agent: true,
            ..Self::new(id, name)
        }
    }
    /// Up to two initials.
    pub fn initials(&self) -> String {
        self.name
            .split_whitespace()
            .filter_map(|w| w.chars().next())
            .take(2)
            .collect::<String>()
            .to_uppercase()
    }
}

/// One comment with its replies.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Comment {
    /// Id.
    pub id: String,
    /// Author.
    pub author: Person,
    /// Plain text; `@Full Name` mentions of known people are marked.
    pub body: String,
    /// Time label ("2h").
    pub at: String,
    /// Edited mark.
    pub edited: bool,
    /// Deleted but kept as a placeholder because it has replies.
    pub deleted: bool,
    /// Reactions: emoji and who gave them.
    pub reactions: Vec<(String, Vec<String>)>,
    /// Replies.
    pub replies: Vec<Comment>,
}

impl Comment {
    /// A comment.
    pub fn new(id: &str, author: Person, body: &str, at: &str) -> Self {
        Self {
            id: id.into(),
            author,
            body: body.into(),
            at: at.into(),
            edited: false,
            deleted: false,
            reactions: Vec::new(),
            replies: Vec::new(),
        }
    }
    /// With a reaction.
    pub fn react(mut self, emoji: &str, users: &[&str]) -> Self {
        self.reactions
            .push((emoji.into(), users.iter().map(|u| u.to_string()).collect()));
        self
    }
    /// With replies.
    pub fn replies(mut self, r: Vec<Comment>) -> Self {
        self.replies = r;
        self
    }
}

/// What a change to a thread was.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum ThreadEvent {
    /// A reply (or a new top comment) with its id.
    Reply(String),
    /// Edited.
    Edit(String),
    /// Deleted.
    Delete(String),
    /// Reaction toggled.
    React(String, String),
}

/// A comment thread.
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct Thread {
    /// Top-level comments.
    pub comments: Vec<Comment>,
    /// Folded into the resolved chip.
    pub resolved: bool,
    next: usize,
}

fn walk_mut<'a>(list: &'a mut [Comment], id: &str) -> Option<&'a mut Comment> {
    for c in list.iter_mut() {
        if c.id == id {
            return Some(c);
        }
        if let Some(x) = walk_mut(&mut c.replies, id) {
            return Some(x);
        }
    }
    None
}

fn depth_of(list: &[Comment], id: &str, d: usize) -> Option<usize> {
    for c in list {
        if c.id == id {
            return Some(d);
        }
        if let Some(x) = depth_of(&c.replies, id, d + 1) {
            return Some(x);
        }
    }
    None
}

fn parent_of(list: &[Comment], id: &str, parent: Option<String>) -> Option<Option<String>> {
    for c in list {
        if c.id == id {
            return Some(parent.clone());
        }
        if let Some(x) = parent_of(&c.replies, id, Some(c.id.clone())) {
            return Some(x);
        }
    }
    None
}

impl Thread {
    /// A thread over `comments`.
    pub fn new(comments: Vec<Comment>) -> Self {
        Self {
            comments,
            resolved: false,
            next: 1,
        }
    }
    /// Comments that are not deleted.
    pub fn count(&self) -> usize {
        fn n(l: &[Comment]) -> usize {
            l.iter()
                .map(|c| usize::from(!c.deleted) + n(&c.replies))
                .sum()
        }
        n(&self.comments)
    }
    /// Everyone who wrote in the thread, first appearance first.
    pub fn people(&self) -> Vec<Person> {
        fn go(l: &[Comment], out: &mut Vec<Person>) {
            for c in l {
                if !out.iter().any(|p| p.id == c.author.id) {
                    out.push(c.author.clone());
                }
                go(&c.replies, out);
            }
        }
        let mut v = Vec::new();
        go(&self.comments, &mut v);
        v
    }
    /// Finds a comment.
    pub fn get(&self, id: &str) -> Option<&Comment> {
        fn go<'a>(l: &'a [Comment], id: &str) -> Option<&'a Comment> {
            for c in l {
                if c.id == id {
                    return Some(c);
                }
                if let Some(x) = go(&c.replies, id) {
                    return Some(x);
                }
            }
            None
        }
        go(&self.comments, id)
    }
    /// Adds a reply to `parent` (or a top comment for `None`); a reply deeper than `max_depth`
    /// attaches to the deepest allowed ancestor. Returns the new id.
    pub fn reply(
        &mut self,
        parent: Option<&str>,
        me: &Person,
        body: &str,
        max_depth: usize,
    ) -> Option<String> {
        let body = body.trim();
        if body.is_empty() {
            return None;
        }
        self.next += 1;
        let id = format!("c-new-{}", self.next);
        let c = Comment::new(&id, me.clone(), body, "now");
        let mut target = parent.map(String::from);
        while let Some(t) = target.clone() {
            let d = depth_of(&self.comments, &t, 1).unwrap_or(1);
            if d <= max_depth {
                break;
            }
            target = parent_of(&self.comments, &t, None).flatten();
        }
        match target.and_then(|t| walk_mut(&mut self.comments, &t)) {
            Some(p) => p.replies.push(c),
            None => self.comments.push(c),
        }
        Some(id)
    }
    /// Edits a comment's text.
    pub fn edit(&mut self, id: &str, body: &str) -> bool {
        let body = body.trim();
        match walk_mut(&mut self.comments, id) {
            Some(c) if !body.is_empty() && c.body != body => {
                c.body = body.to_string();
                c.edited = true;
                true
            }
            _ => false,
        }
    }
    /// Deletes a comment: one with replies stays as a placeholder.
    pub fn delete(&mut self, id: &str) -> bool {
        fn go(l: &mut Vec<Comment>, id: &str) -> bool {
            if let Some(i) = l.iter().position(|c| c.id == id) {
                if l[i].replies.is_empty() {
                    l.remove(i);
                } else {
                    l[i].deleted = true;
                    l[i].body.clear();
                    l[i].reactions.clear();
                }
                return true;
            }
            l.iter_mut().any(|c| go(&mut c.replies, id))
        }
        go(&mut self.comments, id)
    }
    /// Toggles `me`'s reaction; empty reactions disappear.
    pub fn react(&mut self, id: &str, emoji: &str, me: &str) {
        if let Some(c) = walk_mut(&mut self.comments, id) {
            match c.reactions.iter().position(|r| r.0 == emoji) {
                Some(i) => {
                    let users = &mut c.reactions[i].1;
                    if let Some(u) = users.iter().position(|u| u == me) {
                        users.remove(u);
                    } else {
                        users.push(me.to_string());
                    }
                    if c.reactions[i].1.is_empty() {
                        c.reactions.remove(i);
                    }
                }
                None => c.reactions.push((emoji.into(), vec![me.into()])),
            }
        }
    }
}

/// Splits a body into (text, is_mention) for the known people.
pub fn mention_spans(body: &str, people: &[Person]) -> Vec<(String, bool)> {
    let mut out: Vec<(String, bool)> = Vec::new();
    let mut rest = body;
    while let Some(i) = rest.find('@') {
        let after = &rest[i + 1..];
        let hit = people
            .iter()
            .filter(|p| after.starts_with(&p.name))
            .max_by_key(|p| p.name.len());
        match hit {
            Some(p) => {
                if i > 0 {
                    out.push((rest[..i].to_string(), false));
                }
                out.push((format!("@{}", p.name), true));
                rest = &after[p.name.len()..];
            }
            None => {
                out.push((rest[..=i].to_string(), false));
                rest = after;
            }
        }
    }
    if !rest.is_empty() {
        out.push((rest.to_string(), false));
    }
    // Merge neighbouring plain runs.
    let mut merged: Vec<(String, bool)> = Vec::new();
    for (t, m) in out {
        match merged.last_mut() {
            Some(last) if !last.1 && !m => last.0.push_str(&t),
            _ => merged.push((t, m)),
        }
    }
    merged
}

/// The mention being typed at the end of `text` (`"hi @Al"` → `Some("Al")`).
pub fn mention_query(text: &str) -> Option<&str> {
    let i = text.rfind('@')?;
    let q = &text[i + 1..];
    let before_ok = i == 0 || text[..i].ends_with(char::is_whitespace);
    (before_ok && !q.contains('\n') && q.chars().filter(|c| *c == ' ').count() <= 1).then_some(q)
}

/// People whose name matches a mention query (case-insensitive prefix of any word).
pub fn mention_matches<'a>(q: &str, people: &'a [Person]) -> Vec<&'a Person> {
    let q = q.to_lowercase();
    people
        .iter()
        .filter(|p| {
            let n = p.name.to_lowercase();
            n.starts_with(&q) || n.split_whitespace().any(|w| w.starts_with(&q))
        })
        .take(5)
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Review center
// ---------------------------------------------------------------------------------------------

/// Risk of a review item.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Risk {
    /// High.
    High,
    /// Medium.
    Medium,
    /// Low.
    Low,
}

impl Risk {
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::High => "High risk",
            Self::Medium => "Medium risk",
            Self::Low => "Low risk",
        }
    }
}

/// A piece of proposal text: plain, or a flagged span with the agent's note.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Span {
    /// Plain text.
    Text(String),
    /// Flagged span.
    Flag {
        /// Text.
        text: String,
        /// The agent's note.
        note: String,
        /// Where it came from.
        source: Option<String>,
    },
}

impl Span {
    /// Plain.
    pub fn t(s: &str) -> Self {
        Self::Text(s.into())
    }
    /// Flagged.
    pub fn flag(s: &str, note: &str, source: Option<&str>) -> Self {
        Self::Flag {
            text: s.into(),
            note: note.into(),
            source: source.map(Into::into),
        }
    }
    /// The text.
    pub fn text(&self) -> &str {
        match self {
            Self::Text(t) | Self::Flag { text: t, .. } => t,
        }
    }
}

/// Plain text of spans.
pub fn spans_text(s: &[Span]) -> String {
    s.iter().map(Span::text).collect()
}

/// A file in a review item's code change.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ReviewFile {
    /// Path.
    pub path: String,
    /// Diff of the file.
    pub diff: FileDiff,
}

/// A step of a plan under review.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ReviewStep {
    /// Title.
    pub title: String,
    /// Tool.
    pub tool: Option<String>,
    /// Flagged by the agent.
    pub flagged: bool,
}

/// Something an agent wants to do, waiting for a person.
#[derive(Clone, PartialEq, Debug)]
pub struct ReviewItem {
    /// Id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Agent name.
    pub agent: String,
    /// Model.
    pub model: Option<String>,
    /// Type ("Refund", "Code change").
    pub kind: String,
    /// Risk.
    pub risk: Risk,
    /// Confidence 0..1.
    pub confidence: f32,
    /// When it arrived.
    pub created: ClockMs,
    /// Why the agent wants it.
    pub reasoning: Option<String>,
    /// Labelled fields.
    pub fields: Vec<(String, Vec<Span>)>,
    /// Body label.
    pub body_label: Option<String>,
    /// Body.
    pub body: Vec<Span>,
    /// Code change.
    pub files: Vec<ReviewFile>,
    /// Plan steps.
    pub steps: Vec<ReviewStep>,
}

impl ReviewItem {
    /// An item.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: &str,
        title: &str,
        agent: &str,
        kind: &str,
        risk: Risk,
        confidence: f32,
        created: ClockMs,
    ) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            agent: agent.into(),
            model: None,
            kind: kind.into(),
            risk,
            confidence,
            created,
            reasoning: None,
            fields: Vec::new(),
            body_label: None,
            body: Vec::new(),
            files: Vec::new(),
            steps: Vec::new(),
        }
    }
    /// Flagged spans in order (fields first, then body).
    pub fn flags(&self) -> Vec<(String, String)> {
        self.fields
            .iter()
            .flat_map(|f| f.1.iter())
            .chain(self.body.iter())
            .filter_map(|s| match s {
                Span::Flag { text, note, .. } => Some((text.clone(), note.clone())),
                Span::Text(_) => None,
            })
            .collect()
    }
}

/// Queue order.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Order {
    /// High risk first, then oldest.
    Risk,
    /// Oldest first.
    Oldest,
    /// As passed.
    Given,
}

impl Order {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "oldest" => Self::Oldest,
            "given" => Self::Given,
            _ => Self::Risk,
        }
    }
}

/// Queue filter.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Filter {
    /// Everything.
    All,
    /// High risk only.
    High,
    /// Below the confidence bar.
    Unsure,
}

impl Filter {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "high" => Self::High,
            "unsure" => Self::Unsure,
            _ => Self::All,
        }
    }
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::All => "all",
            Self::High => "high",
            Self::Unsure => "unsure",
        }
    }
}

/// A decision in the review center.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Verdict {
    /// Approved, with edited text when the reviewer changed it.
    Approved(Option<String>),
    /// Rejected with a reason.
    Rejected(String),
}

/// How late an item is against its service level.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Sla {
    /// Fine.
    Ok,
    /// Past 75 %.
    Warn,
    /// Past the limit.
    Late,
}

impl Sla {
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Ok => "ok",
            Self::Warn => "warn",
            Self::Late => "late",
        }
    }
}

/// The default rejection reasons (1–4).
pub const REJECT_REASONS: [&str; 4] = ["Wrong info", "Tone", "Not allowed", "Other"];

/// The review center queue: sorted and filtered items, one current item, decisions with undo.
#[derive(Clone, PartialEq, Debug)]
pub struct Center {
    /// Every item passed in.
    pub items: Vec<ReviewItem>,
    /// Decisions by id, in order.
    pub decided: Vec<(String, Verdict)>,
    /// Skipped ids.
    pub skipped: BTreeSet<String>,
    /// Current item.
    pub current: Option<String>,
    /// Order.
    pub order: Order,
    /// Filter.
    pub filter: Filter,
    /// Below this confidence an item is Unsure.
    pub low_confidence: f32,
    /// Reviewed before (the header counts on from here).
    pub reviewed_before: usize,
}

impl Center {
    /// A queue over `items`, current = first in order.
    pub fn new(items: Vec<ReviewItem>, order: Order) -> Self {
        let mut c = Self {
            items,
            decided: Vec::new(),
            skipped: BTreeSet::new(),
            current: None,
            order,
            filter: Filter::All,
            low_confidence: 0.7,
            reviewed_before: 0,
        };
        c.current = c.queue().first().map(|i| i.id.clone());
        c
    }
    /// Whether an item counts as Unsure.
    pub fn unsure(&self, i: &ReviewItem) -> bool {
        i.confidence < self.low_confidence
    }
    /// Undecided items, sorted and filtered.
    pub fn queue(&self) -> Vec<&ReviewItem> {
        let mut v: Vec<&ReviewItem> = self
            .items
            .iter()
            .filter(|i| !self.decided.iter().any(|d| d.0 == i.id))
            .filter(|i| match self.filter {
                Filter::All => true,
                Filter::High => i.risk == Risk::High,
                Filter::Unsure => self.unsure(i),
            })
            .collect();
        match self.order {
            Order::Risk => v.sort_by_key(|i| (i.risk, i.created)),
            Order::Oldest => v.sort_by_key(|i| i.created),
            Order::Given => {}
        }
        v
    }
    /// Count for a filter tab.
    pub fn filter_count(&self, f: Filter) -> usize {
        let mut c = self.clone();
        c.filter = f;
        c.queue().len()
    }
    /// The current item.
    pub fn current_item(&self) -> Option<&ReviewItem> {
        let id = self.current.as_ref()?;
        self.items.iter().find(|i| &i.id == id)
    }
    /// Selects an item by id.
    pub fn select(&mut self, id: &str) {
        if self.items.iter().any(|i| i.id == id) {
            self.current = Some(id.to_string());
        }
    }
    /// Sets the filter and keeps the current item when it still shows.
    pub fn set_filter(&mut self, f: Filter) {
        self.filter = f;
        let q: Vec<String> = self.queue().iter().map(|i| i.id.clone()).collect();
        if !self.current.as_ref().is_some_and(|c| q.contains(c)) {
            self.current = q.first().cloned();
        }
    }
    /// Moves to the next / previous item in the queue (clamped).
    pub fn step(&mut self, delta: isize) {
        let q: Vec<String> = self.queue().iter().map(|i| i.id.clone()).collect();
        if q.is_empty() {
            self.current = None;
            return;
        }
        let at = self
            .current
            .as_ref()
            .and_then(|c| q.iter().position(|x| x == c))
            .unwrap_or(0) as isize;
        let to = (at + delta).clamp(0, q.len() as isize - 1) as usize;
        self.current = Some(q[to].clone());
    }
    fn after_leave(&mut self, left: &str, q_before: &[String]) {
        let at = q_before.iter().position(|x| x == left).unwrap_or(0);
        let q: Vec<String> = self.queue().iter().map(|i| i.id.clone()).collect();
        self.current = q
            .get(at)
            .or_else(|| q.get(at.saturating_sub(1)))
            .or(q.first())
            .cloned();
    }
    /// Approves the current item (with edited text if any) and moves on.
    pub fn approve(&mut self, edited: Option<String>) -> Option<String> {
        self.decide(Verdict::Approved(edited))
    }
    /// Rejects the current item.
    pub fn reject(&mut self, reason: &str) -> Option<String> {
        self.decide(Verdict::Rejected(reason.to_string()))
    }
    fn decide(&mut self, v: Verdict) -> Option<String> {
        let id = self.current.clone()?;
        let q: Vec<String> = self.queue().iter().map(|i| i.id.clone()).collect();
        self.skipped.remove(&id);
        self.decided.push((id.clone(), v));
        self.after_leave(&id, &q);
        Some(id)
    }
    /// Skips the current item (marked skipped, stays in the queue).
    pub fn skip(&mut self) -> Option<String> {
        let id = self.current.clone()?;
        self.skipped.insert(id.clone());
        let q: Vec<String> = self.queue().iter().map(|i| i.id.clone()).collect();
        let at = q.iter().position(|x| *x == id).unwrap_or(0);
        self.current = q.get(at + 1).or(q.first()).cloned();
        Some(id)
    }
    /// Takes back the last decision; the item becomes current again.
    pub fn undo(&mut self) -> Option<(String, Verdict)> {
        let last = self.decided.pop()?;
        self.current = Some(last.0.clone());
        Some(last)
    }
    /// (reviewed, total) for the header.
    pub fn progress(&self) -> (usize, usize) {
        let done = self.reviewed_before + self.decided.len();
        let waiting = self
            .items
            .iter()
            .filter(|i| !self.decided.iter().any(|d| d.0 == i.id))
            .count();
        (done, done + waiting)
    }
    /// Service level of an item at `now`.
    pub fn sla(&self, i: &ReviewItem, now: ClockMs, sla_ms: u64) -> Sla {
        let waited = now.0.saturating_sub(i.created.0);
        if waited > sla_ms {
            Sla::Late
        } else if waited * 4 >= sla_ms * 3 {
            Sla::Warn
        } else {
            Sla::Ok
        }
    }
}

/// Waiting time: "now", "26m", "2h", "3d".
pub fn waited_label(since: ClockMs, now: ClockMs) -> String {
    let s = now.0.saturating_sub(since.0) / 1000;
    match s {
        0..=59 => "now".into(),
        60..=3599 => format!("{}m", s / 60),
        3600..=86_399 => format!("{}h", s / 3600),
        _ => format!("{}d", s / 86_400),
    }
}

// ---------------------------------------------------------------------------------------------
// Agent review card
// ---------------------------------------------------------------------------------------------

/// Severity of a review finding.
#[derive(Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Severity {
    /// Must fix before merge.
    Blocker,
    /// Should fix.
    Major,
    /// Small.
    Minor,
    /// Style only.
    Nit,
}

impl Severity {
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Blocker => "blocker",
            Self::Major => "major",
            Self::Minor => "minor",
            Self::Nit => "nit",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Blocker => "Blocker",
            Self::Major => "Major",
            Self::Minor => "Minor",
            Self::Nit => "Nit",
        }
    }
}

/// Where a finding stands.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum FindingState {
    /// Open.
    Open,
    /// The suggested fix was applied.
    Applied,
    /// Dismissed.
    Dismissed,
}

/// One finding from an agent's code review.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Finding {
    /// Id.
    pub id: String,
    /// Severity.
    pub severity: Severity,
    /// File.
    pub file: String,
    /// Line.
    pub line: usize,
    /// One line.
    pub title: String,
    /// Detail.
    pub body: String,
    /// Suggested fix: (old line, new line).
    pub fix: Option<(String, String)>,
    /// State.
    pub state: FindingState,
}

impl Finding {
    /// A finding.
    pub fn new(id: &str, severity: Severity, file: &str, line: usize, title: &str) -> Self {
        Self {
            id: id.into(),
            severity,
            file: file.into(),
            line,
            title: title.into(),
            body: String::new(),
            fix: None,
            state: FindingState::Open,
        }
    }
    /// With detail.
    pub fn body(mut self, b: &str) -> Self {
        self.body = b.into();
        self
    }
    /// With a suggested fix.
    pub fn fix(mut self, old: &str, new: &str) -> Self {
        self.fix = Some((old.into(), new.into()));
        self
    }
}

/// The review's overall call.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ReviewVerdict {
    /// Nothing open above Nit.
    Approve,
    /// Open Major or Minor findings.
    Comment,
    /// An open Blocker.
    RequestChanges,
}

impl ReviewVerdict {
    /// Short id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Comment => "comment",
            Self::RequestChanges => "changes",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Self::Approve => "Looks good",
            Self::Comment => "Worth a look",
            Self::RequestChanges => "Changes requested",
        }
    }
}

/// Verdict from the open findings.
pub fn review_verdict(f: &[Finding]) -> ReviewVerdict {
    let open = f.iter().filter(|x| x.state == FindingState::Open);
    let mut worst = None;
    for x in open {
        worst = Some(worst.map_or(x.severity, |w: Severity| w.min(x.severity)));
    }
    match worst {
        Some(Severity::Blocker) => ReviewVerdict::RequestChanges,
        Some(Severity::Major | Severity::Minor) => ReviewVerdict::Comment,
        _ => ReviewVerdict::Approve,
    }
}

/// Open findings per severity, in severity order (only non-zero).
pub fn severity_counts(f: &[Finding]) -> Vec<(Severity, usize)> {
    [
        Severity::Blocker,
        Severity::Major,
        Severity::Minor,
        Severity::Nit,
    ]
    .into_iter()
    .map(|s| {
        (
            s,
            f.iter()
                .filter(|x| x.severity == s && x.state == FindingState::Open)
                .count(),
        )
    })
    .filter(|x| x.1 > 0)
    .collect()
}

#[cfg(test)]
mod tests {
    #[test]
    fn rewrite_review_apply() {
        use super::*;
        let ch = rewrite_chunks(
            "We will ship the new page on Friday.",
            "We ship the page Thursday.",
        );
        let n = change_count(&ch);
        assert!(n >= 2);
        let none = vec![None; n];
        assert_eq!(
            rewrite_apply(&ch, &none),
            "We will ship the new page on Friday."
        );
        let all = vec![Some(true); n];
        assert_eq!(rewrite_apply(&ch, &all), "We ship the page Thursday.");
        let mut d = vec![None; n];
        d[0] = Some(true);
        assert_eq!(rewrite_next(&d, 0), Some(1));
        let done = vec![Some(false); n];
        assert_eq!(rewrite_next(&done, 0), None);
    }

    use super::*;

    const OLD: &str = "fn login(id: u32) {\n    let user = find(id)?;\n    let ok = check(pw);\n    if !ok { return Err(Denied) }\n    Ok(user)\n}\n";
    const NEW: &str = "fn login(id: u32) {\n    let user = find(id)?;\n    let ok = verify(pw, &user.salt)?;\n    if !ok { return Err(Denied) }\n    audit(id);\n    Ok(user)\n}\n";

    #[test]
    fn line_and_word_diff() {
        let d = diff_lines(OLD, NEW);
        assert_eq!(d.iter().filter(|l| l.kind == LineKind::Add).count(), 2);
        assert_eq!(d.iter().filter(|l| l.kind == LineKind::Del).count(), 1);
        let del = d.iter().find(|l| l.kind == LineKind::Del).unwrap();
        assert!(
            del.words
                .iter()
                .any(|s| s.changed && s.text.contains("check"))
        );
        assert_eq!(del.old, Some(3));
        let f = FileDiff::new("src/auth.rs", OLD, NEW);
        assert_eq!(f.hunks.len(), 1);
        assert_eq!(f.hunks[0].header, "fn login");
        assert_eq!(f.lang, "rust");
        assert_eq!(stat_label(f.added, f.removed), "+2 −1");
        assert_eq!(f.apply(|_| true), NEW.trim_end());
        assert_eq!(f.apply(|_| false), OLD.trim_end());
    }

    #[test]
    fn hunks_split_and_gap() {
        let old: String = (1..=30).map(|i| format!("line {i}\n")).collect();
        let new = old
            .replace("line 3\n", "line three\n")
            .replace("line 25\n", "line 25\nextra\n");
        let f = FileDiff::new("a.txt", &old, &new);
        assert_eq!(f.hunks.len(), 2);
        assert!(!f.hunks[1].gap_before.is_empty());
        let rows = f.hunks[0].split_rows();
        assert!(rows.iter().any(|r| r.0.is_some()
            && r.1.is_some()
            && r.0.as_ref().unwrap().kind == LineKind::Del));
        let mut rv = Review::new(vec![f.clone()], ReviewMode::Applied);
        assert!(rv.decide((0, 0), Decision::No));
        assert_eq!(rv.cursor, (0, 1));
        assert_eq!(rv.counts(), (1, 0, 1));
        assert!(rv.result(0).contains("line 3\n"));
        assert!(rv.result(0).contains("extra"));
        rv.decide_all(Decision::Yes);
        assert_eq!(rv.counts(), (0, 1, 1));
        assert_eq!(rv.undo(), 1);
        assert_eq!(rv.counts(), (1, 0, 1));
        rv.comment(0, 1, 26, "  why?  ");
        assert_eq!(rv.send_comments().len(), 1);
        assert!(rv.comments.is_empty());
        assert_eq!(stat_blocks(10, 0, 5), (5, 0, 0));
        assert_eq!(stat_blocks(1, 1, 5), (1, 1, 3));
    }

    #[test]
    fn table_and_revisions() {
        let t = table_diff(
            &[("model", "gpt"), ("retries", "2"), ("region", "eu")],
            &[("model", "gpt"), ("retries", "5"), ("timeout", "10s")],
        );
        let kinds: Vec<RowKind> = t.iter().map(|r| r.kind).collect();
        assert!(kinds.contains(&RowKind::Changed));
        assert!(kinds.contains(&RowKind::Removed));
        assert!(kinds.contains(&RowKind::Added));
        let revs = vec![
            Revision {
                id: "r1".into(),
                label: "a".into(),
                who: "Agent".into(),
                at: ClockMs(0),
                text: "a\nb".into(),
            },
            Revision {
                id: "r2".into(),
                label: "b".into(),
                who: "You".into(),
                at: ClockMs(1),
                text: "a\nc\nd".into(),
            },
        ];
        assert_eq!(revision_stats(&revs), vec![(2, 0), (2, 1)]);
        assert_eq!(revision_diff("x", &revs, 0, 1).added, 2);
    }

    #[test]
    fn suggestion_flow() {
        let code = "a\nb\nc\nd";
        let mut s = Suggestion::new(
            code,
            vec![
                SugHunk {
                    id: "h2".into(),
                    start: 3,
                    remove: 1,
                    insert: vec!["D".into()],
                    summary: "d".into(),
                },
                SugHunk {
                    id: "h1".into(),
                    start: 1,
                    remove: 1,
                    insert: vec!["B1".into(), "B2".into()],
                    summary: "b".into(),
                },
                SugHunk {
                    id: "bad".into(),
                    start: 1,
                    remove: 1,
                    insert: vec![],
                    summary: "x".into(),
                },
            ],
        );
        assert_eq!(s.hunks.len(), 2);
        assert_eq!(s.play(100).0, SugStatus::Thinking);
        assert_eq!(s.play(SUG_THINK_MS + SUG_LINE_MS).1, 1);
        assert_eq!(s.play(10_000), (SugStatus::Ready, 3));
        assert_eq!(s.stat(), (3, 2));
        s.decide("h1", true);
        assert_eq!(s.pending(), 1);
        assert_eq!(s.apply(), "a\nB1\nB2\nc\nd");
        s.decide_all(false);
        assert!(s.resolved());
        assert_eq!(s.status_line(), "Applied 1 of 2 changes");
        assert!(matches!(s.rows(usize::MAX, true)[1], SugRow::Line(2, _)));
    }

    #[test]
    fn inline_edit_chunks() {
        let c = rewrite_chunks(
            "We are very careful with data.",
            "We are careful with your data.",
        );
        assert_eq!(change_count(&c), 2);
        let mut d = BTreeMap::new();
        assert_eq!(apply_chunks(&c, &d, true), "We are careful with your data.");
        d.insert(0, false);
        assert_eq!(
            apply_chunks(&c, &d, true),
            "We are very careful with your data."
        );
        let r = chunks_reached(&c, 0);
        assert!(r.iter().all(|x| !x));
        assert!(words_shown(340, 1.0) >= 30);
        assert_eq!(
            scripted_rewrite("shorten", "We are really very careful."),
            "We are careful."
        );
        assert_eq!(
            scripted_rewrite("grammar", "teh plan is ready"),
            "The plan is ready."
        );
        assert!(scripted_completion("The launch is close").is_some());
    }

    #[test]
    fn compare_keys() {
        assert_eq!(compare_key(50.0, "ArrowRight", false), Some(51.0));
        assert_eq!(compare_key(95.0, "ArrowRight", true), Some(100.0));
        assert_eq!(compare_key(30.0, "Home", false), Some(0.0));
        assert_eq!(compare_key(30.0, "x", false), None);
        assert_eq!(compare_valuetext(60.0), "40% after");
        assert!(caption_opacity(3.0, true) < 0.3);
    }

    #[test]
    fn thread_ops() {
        let you = Person::new("you", "You");
        let mira = Person::new("m", "Mira Sol");
        let mut t = Thread::new(vec![
            Comment::new("c1", mira.clone(), "Shorter? @You", "2h").replies(vec![
                Comment::new("c2", you.clone(), "Sure", "1h").replies(vec![Comment::new(
                    "c3",
                    mira.clone(),
                    "Thanks",
                    "1h",
                )]),
            ]),
        ]);
        assert_eq!(t.count(), 3);
        // c3 is at depth 3; with max depth 2 the reply attaches to c2.
        let id = t.reply(Some("c3"), &you, "ok", 2).unwrap();
        assert!(t.get("c2").unwrap().replies.iter().any(|c| c.id == id));
        assert!(t.edit("c2", "Sure thing"));
        assert!(t.get("c2").unwrap().edited);
        t.react("c1", "👍", "you");
        assert_eq!(t.get("c1").unwrap().reactions.len(), 1);
        t.react("c1", "👍", "you");
        assert!(t.get("c1").unwrap().reactions.is_empty());
        assert!(t.delete("c2"));
        assert!(t.get("c2").unwrap().deleted);
        let spans = mention_spans("ping @Mira Sol now", &t.people());
        assert_eq!(spans[1], ("@Mira Sol".to_string(), true));
        assert_eq!(mention_query("hi @Mi"), Some("Mi"));
        assert_eq!(mention_matches("so", &t.people()).len(), 1);
    }

    #[test]
    fn center_flow() {
        let items = vec![
            ReviewItem::new(
                "a",
                "Low one",
                "Ops",
                "Plan",
                Risk::Low,
                0.9,
                ClockMs(5_000),
            ),
            ReviewItem::new(
                "b",
                "High one",
                "Support",
                "Refund",
                Risk::High,
                0.6,
                ClockMs(9_000),
            ),
            ReviewItem::new(
                "c",
                "Mid one",
                "Code",
                "Code change",
                Risk::Medium,
                0.8,
                ClockMs(1_000),
            ),
        ];
        let mut c = Center::new(items, Order::Risk);
        assert_eq!(c.current.as_deref(), Some("b"));
        assert_eq!(c.filter_count(Filter::Unsure), 1);
        c.approve(None);
        assert_eq!(c.current.as_deref(), Some("c"));
        assert_eq!(c.progress(), (1, 3));
        c.skip();
        assert!(c.skipped.contains("c"));
        assert_eq!(c.current.as_deref(), Some("a"));
        c.reject("Tone");
        assert_eq!(c.undo().unwrap().0, "a");
        assert_eq!(c.current.as_deref(), Some("a"));
        let it = c.items[0].clone();
        assert_eq!(c.sla(&it, ClockMs(5_000 + 1_500), 2_000), Sla::Warn);
        assert_eq!(waited_label(ClockMs(0), ClockMs(26 * 60_000)), "26m");
        let f = vec![
            Finding::new("1", Severity::Major, "a.rs", 3, "x"),
            Finding::new("2", Severity::Nit, "a.rs", 9, "y"),
        ];
        assert_eq!(review_verdict(&f), ReviewVerdict::Comment);
        assert_eq!(severity_counts(&f).len(), 2);
    }
}
