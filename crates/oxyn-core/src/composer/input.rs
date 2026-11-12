//! Input shell model (`docs/specs/composer-input-shell.md`): variants, row growth, placeholders by
//! state, large-paste chips, limits and prompt history. Text editing itself (caret, selection,
//! IME) belongs to the renderer; it hands the text back here.

use unicode_segmentation::UnicodeSegmentation;

use super::{BusyAction, DisabledReason, DraftSummary};
use crate::{AgentState, StallLevel, text::format_tokens};

/// Input shell variants (`composer-input-shell.md` §6, free set).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum InputVariant {
    /// 2→8 rows, 12 px radius, toolbar under the text.
    #[default]
    RoundedBox,
    /// One-row capsule that grows into a box when it wraps.
    Pill,
    /// Actions beside a one-line draft, below it once it wraps.
    Adaptive,
    /// Full-width bar flush with the window bottom.
    DockedBar,
    /// Detached card above the bottom edge with a shadow.
    Floating,
    /// Centered greeting composer that docks on first send.
    HeroDock,
    /// Dense rows for IDE panels; icon toolbar.
    IdeInline,
    /// `›` prompt, no border, tinted row.
    TerminalPrompt,
    /// Underline only.
    Minimal,
}

impl InputVariant {
    /// Every free variant.
    pub const ALL: [InputVariant; 9] = [
        Self::RoundedBox,
        Self::Pill,
        Self::Adaptive,
        Self::DockedBar,
        Self::Floating,
        Self::HeroDock,
        Self::IdeInline,
        Self::TerminalPrompt,
        Self::Minimal,
    ];

    /// Short id (`rounded-box`); the registry id is `composer-input/<id>`.
    pub fn id(self) -> &'static str {
        match self {
            Self::RoundedBox => "rounded-box",
            Self::Pill => "pill",
            Self::Adaptive => "adaptive",
            Self::DockedBar => "docked-bar",
            Self::Floating => "floating",
            Self::HeroDock => "hero-dock",
            Self::IdeInline => "ide-inline",
            Self::TerminalPrompt => "terminal-prompt",
            Self::Minimal => "minimal",
        }
    }

    /// Parses `pill` or `composer-input/pill`.
    pub fn from_id(id: &str) -> Option<Self> {
        let id = id.strip_prefix("composer-input/").unwrap_or(id);
        Self::ALL.into_iter().find(|v| v.id() == id)
    }

    /// Default row range.
    pub fn rows(self) -> RowRange {
        match self {
            Self::RoundedBox | Self::Floating | Self::HeroDock => RowRange { min: 2, max: 8 },
            Self::IdeInline => RowRange { min: 1, max: 6 },
            Self::TerminalPrompt | Self::Minimal | Self::Pill | Self::Adaptive => {
                RowRange { min: 1, max: 8 }
            }
            Self::DockedBar => RowRange { min: 1, max: 6 },
        }
    }

    /// Footer actions sit inline with a one-line draft (pill, adaptive, terminal, minimal).
    pub fn inline_actions(self, rows: u16) -> bool {
        match self {
            Self::Pill | Self::Adaptive => rows <= 1,
            Self::TerminalPrompt | Self::Minimal => true,
            _ => false,
        }
    }
}

/// Visible rows before the shell scrolls internally.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowRange {
    /// Rows when empty.
    pub min: u16,
    /// Rows before internal scroll.
    pub max: u16,
}

/// Row layout of a draft at a given width in columns (average character advances).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RowFit {
    /// Wrapped rows the text needs.
    pub content: u16,
    /// Rows the shell shows (clamped to the range).
    pub visible: u16,
    /// The draft scrolls inside the shell (fade mask on the clipped edge).
    pub scrolling: bool,
}

impl RowRange {
    /// Rows for `text` wrapped at `cols` columns. Wide (CJK) graphemes count as two columns.
    /// Renderers with real text layout use their own line count with [`RowRange::fit_lines`].
    pub fn fit(self, text: &str, cols: u16) -> RowFit {
        self.fit_lines(wrapped_lines(text, cols.max(1)))
    }

    /// Rows for a known line count.
    pub fn fit_lines(self, lines: u16) -> RowFit {
        RowFit {
            content: lines,
            visible: lines.clamp(self.min, self.max),
            scrolling: lines > self.max,
        }
    }
}

/// Soft-wrapped line count, wrapping at word boundaries where possible.
pub fn wrapped_lines(text: &str, cols: u16) -> u16 {
    let cols = usize::from(cols.max(1));
    let mut total = 0usize;
    for line in text.split('\n') {
        let mut rows = 1usize;
        let mut col = 0usize;
        for word in line.split_word_bounds() {
            let w: usize = word.graphemes(true).map(grapheme_cols).sum();
            if col + w > cols && col > 0 && !word.trim().is_empty() {
                rows += 1;
                col = 0;
            }
            if w > cols {
                // A word longer than a line breaks anywhere.
                rows += (col + w - 1) / cols;
                col = (col + w) % cols;
            } else {
                col += w;
            }
        }
        total += rows;
    }
    total.clamp(1, usize::from(u16::MAX)) as u16
}

fn grapheme_cols(g: &str) -> usize {
    let c = g.chars().next().unwrap_or(' ');
    if matches!(c as u32, 0x1100..=0x115F | 0x2E80..=0xA4CF | 0xAC00..=0xD7A3 | 0xF900..=0xFAFF | 0xFE30..=0xFE4F | 0xFF00..=0xFF60 | 0xFFE0..=0xFFE6 | 0x1F300..=0x1FAFF)
    {
        2
    } else {
        1
    }
}

/// When a paste becomes a chip (`composer-input-shell.md` §4, after Codex CLI).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PastePolicy {
    /// Pastes longer than this many characters fold into a chip.
    pub chip_over_chars: usize,
    /// Pastes with more lines than this fold into a chip.
    pub chip_over_lines: usize,
    /// Pastes beyond this are refused with a reason.
    pub hard_cap: usize,
}

impl Default for PastePolicy {
    fn default() -> Self {
        Self {
            chip_over_chars: 1_000,
            chip_over_lines: 20,
            hard_cap: 100_000,
        }
    }
}

/// What a paste turns into.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PasteOutcome {
    /// Insert the text at the caret.
    Inline,
    /// Fold into a "Pasted text · N lines" chip.
    Chip {
        /// Line count.
        lines: usize,
        /// Character count.
        chars: usize,
    },
    /// Too large; the reason is shown and announced.
    Refused(String),
}

impl PastePolicy {
    /// Classifies a paste.
    pub fn classify(&self, text: &str) -> PasteOutcome {
        let chars = text.chars().count();
        let lines = text.lines().count().max(1);
        if chars > self.hard_cap {
            PasteOutcome::Refused(format!(
                "Paste is {} characters · attach it as a file instead",
                group(chars)
            ))
        } else if chars > self.chip_over_chars || lines > self.chip_over_lines {
            PasteOutcome::Chip { lines, chars }
        } else {
            PasteOutcome::Inline
        }
    }
}

fn group(n: usize) -> String {
    let s = n.to_string();
    let mut out = String::new();
    for (i, c) in s.chars().enumerate() {
        if i > 0 && (s.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A pasted-text chip in the draft.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PasteChip {
    /// Stable id (1-based, per draft).
    pub id: u32,
    /// The pasted text.
    pub text: String,
}

impl PasteChip {
    /// `Pasted text · 142 lines`.
    pub fn label(&self) -> String {
        let n = self.text.lines().count().max(1);
        format!("Pasted text · {n} line{}", if n == 1 { "" } else { "s" })
    }

    /// First line, for the hover preview.
    pub fn preview(&self) -> &str {
        self.text.lines().next().unwrap_or("")
    }
}

/// The structured draft: text plus folded pastes. Attachments live in their own list.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Draft {
    /// Typed text.
    pub text: String,
    /// Folded pastes, in paste order.
    pub pastes: Vec<PasteChip>,
    next_id: u32,
}

impl Draft {
    /// A draft holding `text`.
    pub fn new(text: impl Into<String>) -> Self {
        Self {
            text: text.into(),
            ..Self::default()
        }
    }

    /// Applies a paste per `policy`; inline pastes are appended at the end (renderers that know
    /// the caret insert the text themselves when this returns [`PasteOutcome::Inline`]).
    pub fn paste(&mut self, text: &str, policy: &PastePolicy) -> PasteOutcome {
        let out = policy.classify(text);
        match out {
            PasteOutcome::Inline => self.text.push_str(text),
            PasteOutcome::Chip { .. } => {
                self.next_id += 1;
                self.pastes.push(PasteChip {
                    id: self.next_id,
                    text: text.to_string(),
                });
            }
            PasteOutcome::Refused(_) => {}
        }
        out
    }

    /// Expands a chip back into raw text (appended), removing the chip.
    pub fn expand(&mut self, id: u32) {
        if let Some(i) = self.pastes.iter().position(|p| p.id == id) {
            let chip = self.pastes.remove(i);
            if !self.text.is_empty() && !self.text.ends_with('\n') {
                self.text.push('\n');
            }
            self.text.push_str(&chip.text);
        }
    }

    /// Removes a chip.
    pub fn remove_paste(&mut self, id: u32) {
        self.pastes.retain(|p| p.id != id);
    }

    /// No text (whitespace counts as empty) and no chips.
    pub fn is_empty(&self) -> bool {
        self.text.trim().is_empty() && self.pastes.is_empty()
    }

    /// The text that will be sent: typed text followed by each paste.
    pub fn full_text(&self) -> String {
        let mut s = self.text.clone();
        for p in &self.pastes {
            if !s.is_empty() {
                s.push_str("\n\n");
            }
            s.push_str(&p.text);
        }
        s
    }

    /// Clears the draft after a send.
    pub fn clear(&mut self) {
        self.text.clear();
        self.pastes.clear();
    }

    /// Summary for [`super::ComposerState`].
    pub fn summary(&self, attachments: usize, uploading: usize) -> DraftSummary {
        DraftSummary {
            is_empty: self.is_empty() && attachments == 0,
            chars: self.full_text().graphemes(true).count(),
            attachments,
            uploading,
            ime_composing: false,
            over_limit: false,
        }
    }
}

/// A length limit on the draft.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputLimit {
    /// Graphemes.
    Chars(usize),
    /// Tokens, estimated at 4 characters per token unless the host supplies a count.
    Tokens(usize),
}

/// Where the draft stands against its limit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LimitState {
    /// Under 80 %: no counter.
    Fine,
    /// 80–100 %: counter in the hint row.
    Near {
        /// Counter text, `1.8k / 2k tokens`.
        label: String,
    },
    /// Over: send blocked with a reason.
    Over {
        /// Counter text.
        label: String,
        /// `212 tokens over · attach as file instead`.
        reason: String,
    },
}

impl LimitState {
    /// Sending is blocked.
    pub fn blocks(&self) -> bool {
        matches!(self, Self::Over { .. })
    }
}

/// Rough token estimate (chars / 4, rounded up), for limits while no tokenizer is wired.
pub fn estimate_tokens(text: &str) -> usize {
    text.chars().count().div_ceil(4)
}

impl InputLimit {
    /// Checks `text`. `tokens` overrides the estimate when the host has a real count.
    pub fn check(&self, text: &str, tokens: Option<usize>) -> LimitState {
        let (used, max, unit) = match *self {
            Self::Chars(max) => (text.graphemes(true).count(), max, "characters"),
            Self::Tokens(max) => (
                tokens.unwrap_or_else(|| estimate_tokens(text)),
                max,
                "tokens",
            ),
        };
        let label = format!(
            "{} / {} {unit}",
            format_tokens(used as u64, false),
            format_tokens(max as u64, false)
        );
        if used > max {
            LimitState::Over {
                label,
                reason: format!("{} {unit} over · attach as file instead", used - max),
            }
        } else if used * 10 >= max * 8 {
            LimitState::Near { label }
        } else {
            LimitState::Fine
        }
    }
}

/// Prompt history recall (↑ on the first line, ↓ on the last), stashing the live draft.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct History {
    entries: Vec<String>,
    cursor: Option<usize>,
    stash: Option<String>,
}

impl History {
    /// History with `entries`, oldest first.
    pub fn new(entries: Vec<String>) -> Self {
        Self {
            entries,
            ..Self::default()
        }
    }

    /// Records a sent prompt (skips repeats of the newest entry) and leaves recall.
    pub fn push(&mut self, prompt: &str) {
        if self.entries.last().map(String::as_str) != Some(prompt) && !prompt.trim().is_empty() {
            self.entries.push(prompt.to_string());
        }
        self.cursor = None;
        self.stash = None;
    }

    /// ↑: an older prompt, stashing `current` on the first step. `None` at the oldest.
    pub fn up(&mut self, current: &str) -> Option<&str> {
        let next = match self.cursor {
            None if self.entries.is_empty() => return None,
            None => {
                self.stash = Some(current.to_string());
                self.entries.len() - 1
            }
            Some(0) => return None,
            Some(i) => i - 1,
        };
        self.cursor = Some(next);
        Some(&self.entries[next])
    }

    /// ↓: a newer prompt, or the stashed draft past the newest.
    pub fn down(&mut self) -> Option<String> {
        let i = self.cursor?;
        if i + 1 < self.entries.len() {
            self.cursor = Some(i + 1);
            Some(self.entries[i + 1].clone())
        } else {
            self.cursor = None;
            Some(self.stash.take().unwrap_or_default())
        }
    }

    /// `History 3/12` while recalling.
    pub fn hint(&self) -> Option<String> {
        self.cursor
            .map(|i| format!("History {}/{}", i + 1, self.entries.len()))
    }
}

/// Placeholder copy by state (`composer-input-shell.md` §3). Hosts hold it with
/// [`super::Held`] for 600 ms so a fast agent doesn't make it flicker.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Placeholder {
    /// Idle copy; mention only the triggers the app registered.
    pub idle: String,
}

impl Default for Placeholder {
    fn default() -> Self {
        Self {
            idle: "Ask anything · @ files · / commands".into(),
        }
    }
}

impl Placeholder {
    /// The placeholder for `agent`.
    pub fn text(
        &self,
        agent: &AgentState,
        busy: BusyAction,
        disabled: Option<&DisabledReason>,
    ) -> String {
        if let Some(d) = disabled {
            return d.label();
        }
        match agent {
            AgentState::Idle
            | AgentState::Composing
            | AgentState::Done { .. }
            | AgentState::Cancelled { .. } => self.idle.clone(),
            AgentState::Submitted
            | AgentState::Thinking { .. }
            | AgentState::Streaming
            | AgentState::ToolRunning { .. } => match busy {
                BusyAction::Steer => "Steer the agent…".into(),
                _ => "Queue a follow-up…".into(),
            },
            AgentState::Queued { count } => format!("Add to the queue ({count} waiting)…"),
            AgentState::Steering => "Steering…".into(),
            AgentState::AwaitingApproval { .. } => "Approve above, or type why not…".into(),
            AgentState::Stalled { level, .. } => match level {
                StallLevel::Soft => "Still working · type to nudge, Esc to stop".into(),
                StallLevel::Hard => "No reply · type to nudge, Esc to stop".into(),
            },
            AgentState::Error { .. } => "Edit and try again".into(),
        }
    }
}

/// Hint row under a busy draft (`Busy(Draft)`).
pub fn busy_hint(busy: BusyAction) -> &'static str {
    match busy {
        BusyAction::Queue => "Enter to queue · ⌘↵ to steer",
        BusyAction::Steer => "Enter to steer · ⌥↵ to queue",
        BusyAction::Interrupt => "Enter stops the run and sends",
        BusyAction::Ask => "Enter to choose: steer or queue",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_grow_and_scroll() {
        let r = RowRange { min: 2, max: 4 };
        assert_eq!(r.fit("", 40).visible, 2);
        assert_eq!(r.fit("one\ntwo\nthree", 40).visible, 3);
        let long = "word ".repeat(60);
        let f = r.fit(&long, 40);
        assert!(f.scrolling && f.visible == 4 && f.content > 4);
        assert_eq!(wrapped_lines("abcdefghij", 4), 3);
        assert_eq!(wrapped_lines("漢字漢字", 4), 2);
    }

    #[test]
    fn paste_folds_into_chip() {
        let p = PastePolicy::default();
        let mut d = Draft::new("look at this:");
        assert_eq!(d.paste("short", &p), PasteOutcome::Inline);
        let big = "line\n".repeat(142);
        assert!(matches!(
            d.paste(&big, &p),
            PasteOutcome::Chip { lines: 142, .. }
        ));
        assert_eq!(d.pastes[0].label(), "Pasted text · 142 lines");
        assert!(d.full_text().ends_with("line\n"));
        d.expand(1);
        assert!(d.pastes.is_empty() && d.text.contains("line\nline"));
        assert!(matches!(
            p.classify(&"x".repeat(100_001)),
            PasteOutcome::Refused(r) if r.contains("100,001")
        ));
    }

    #[test]
    fn limits() {
        let l = InputLimit::Tokens(2_000);
        assert_eq!(l.check("hi", None), LimitState::Fine);
        assert_eq!(
            l.check("", Some(1_800)),
            LimitState::Near {
                label: "1.8k / 2k tokens".into()
            }
        );
        let over = l.check("", Some(2_212));
        assert!(
            over.blocks()
                && matches!(&over, LimitState::Over { reason, .. } if reason == "212 tokens over · attach as file instead")
        );
    }

    #[test]
    fn history_recall_restores_stash() {
        let mut h = History::new(vec!["first".into(), "second".into()]);
        assert_eq!(h.up("my draft"), Some("second"));
        assert_eq!(h.hint().as_deref(), Some("History 2/2"));
        assert_eq!(h.up(""), Some("first"));
        assert_eq!(h.up(""), None);
        assert_eq!(h.down().as_deref(), Some("second"));
        assert_eq!(h.down().as_deref(), Some("my draft"));
        assert_eq!(h.down(), None);
    }

    #[test]
    fn placeholders() {
        let p = Placeholder::default();
        assert_eq!(
            p.text(&AgentState::Streaming, BusyAction::Steer, None),
            "Steer the agent…"
        );
        assert_eq!(
            p.text(&AgentState::Queued { count: 2 }, BusyAction::Queue, None),
            "Add to the queue (2 waiting)…"
        );
        assert_eq!(
            p.text(
                &AgentState::Idle,
                BusyAction::Queue,
                Some(&DisabledReason::Quota {
                    resets: "14:00".into()
                })
            ),
            "Usage limit reached · resets 14:00"
        );
        for v in InputVariant::ALL {
            assert_eq!(InputVariant::from_id(v.id()), Some(v));
        }
    }
}
