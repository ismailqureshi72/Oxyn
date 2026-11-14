//! Inline edit (`inline-edit.md`): rename in place. The text becomes a field with the same
//! metrics, saves optimistically, shows a drawn check when the save lands and rolls back with a
//! retry when it fails. Enter saves (Shift+Enter breaks a line in multiline), Esc cancels, Tab
//! saves like a rename.
//!
//! Behaviour follows the Arc UI Inline edit docs (rebuilt, no source used). Oxyn plus: agent
//! rename suggestions, and a conflict state when the agent changes the value while you edit.

use crate::ClockMs;

/// Phase.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum EditPhase {
    /// Showing the text (a button).
    #[default]
    Idle,
    /// Editing the draft.
    Editing,
    /// Saving (optimistic: the new text shows).
    Saving,
    /// Saved; the check draws.
    Saved(ClockMs),
    /// The save failed; the old text rolled back. Retry sends the draft again.
    Failed(String),
    /// Oxyn plus: the value changed underneath while editing.
    Conflict(String),
}

impl EditPhase {
    /// Id.
    pub fn id(&self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Editing => "editing",
            Self::Saving => "saving",
            Self::Saved(_) => "saved",
            Self::Failed(_) => "failed",
            Self::Conflict(_) => "conflict",
        }
    }
}

/// Type style (Arc `variant`).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum EditVariant {
    /// Names and headings.
    #[default]
    Title,
    /// Descriptions.
    Body,
}

/// Keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EditKey {
    /// Enter (`shift` for a line break in multiline).
    Enter {
        /// Shift held.
        shift: bool,
    },
    /// Esc.
    Escape,
    /// Tab / blur.
    Tab,
}

/// What the host should do.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EditAction {
    /// Persist this value (then call `saved` / `failed`).
    Save(String),
    /// Nothing.
    None,
}

/// The inline editor.
#[derive(Clone, Debug, PartialEq)]
pub struct InlineEdit {
    /// The saved value.
    pub value: String,
    /// The draft while editing.
    pub draft: String,
    /// Phase.
    pub phase: EditPhase,
    /// Validation message (blocks saving).
    pub error: Option<String>,
    /// Wraps and grows.
    pub multiline: bool,
    /// Type style.
    pub variant: EditVariant,
    /// Max characters (validation).
    pub max: Option<usize>,
    /// Empty not allowed.
    pub required: bool,
    /// The value before the optimistic save (rollback target).
    pub before: String,
    /// When the text last changed (roll).
    pub changed_at: ClockMs,
    /// Oxyn plus: rename suggestions.
    pub suggestions: Vec<String>,
}

/// How long the saved check stays.
pub const SAVED_MS: u64 = 1_400;

impl InlineEdit {
    /// An editor for `value`.
    pub fn new(value: &str) -> Self {
        Self {
            value: value.into(),
            draft: value.into(),
            phase: EditPhase::Idle,
            error: None,
            multiline: false,
            variant: EditVariant::Title,
            max: None,
            required: true,
            before: value.into(),
            changed_at: ClockMs(0),
            suggestions: vec![],
        }
    }
    /// Editing (or resolving a conflict).
    pub fn editing(&self) -> bool {
        matches!(self.phase, EditPhase::Editing | EditPhase::Conflict(_))
    }
    /// Click / Enter / Space on the text.
    pub fn start(&mut self) {
        if matches!(self.phase, EditPhase::Saving) {
            return;
        }
        self.draft = self.value.clone();
        self.error = None;
        self.phase = EditPhase::Editing;
    }
    /// Typing.
    pub fn input(&mut self, s: &str) {
        self.draft = s.into();
        self.error = None;
    }
    /// The check for the draft.
    pub fn validate(&self, s: &str) -> Option<String> {
        if self.required && s.trim().is_empty() {
            return Some("Name can't be empty".into());
        }
        if let Some(m) = self.max
            && s.chars().count() > m
        {
            return Some(format!("Keep it under {m} characters"));
        }
        None
    }
    /// Commit the draft: validates, then saves optimistically.
    pub fn commit(&mut self, now: ClockMs) -> EditAction {
        if !self.editing() {
            return EditAction::None;
        }
        let v = if self.multiline {
            self.draft.trim_end().to_string()
        } else {
            self.draft.trim().to_string()
        };
        if let Some(e) = self.validate(&v) {
            self.error = Some(e);
            return EditAction::None;
        }
        if v == self.value {
            self.phase = EditPhase::Idle;
            return EditAction::None;
        }
        self.before = self.value.clone();
        self.value = v.clone();
        self.changed_at = now;
        self.phase = EditPhase::Saving;
        EditAction::Save(v)
    }
    /// The save landed.
    pub fn saved(&mut self, now: ClockMs) {
        if self.phase == EditPhase::Saving {
            self.phase = EditPhase::Saved(now);
        }
    }
    /// The save failed: roll back, keep the draft for Retry.
    pub fn failed(&mut self, msg: &str, now: ClockMs) {
        if self.phase == EditPhase::Saving {
            self.draft = self.value.clone();
            self.value = self.before.clone();
            self.changed_at = now;
            self.phase = EditPhase::Failed(msg.into());
        }
    }
    /// Retry after a failure.
    pub fn retry(&mut self, now: ClockMs) -> EditAction {
        if matches!(self.phase, EditPhase::Failed(_)) {
            self.phase = EditPhase::Editing;
            return self.commit(now);
        }
        EditAction::None
    }
    /// Esc: cancel and roll the text back.
    pub fn cancel(&mut self) {
        if self.editing() || matches!(self.phase, EditPhase::Failed(_)) {
            self.draft = self.value.clone();
            self.error = None;
            self.phase = EditPhase::Idle;
        }
    }
    /// Keyboard while editing. Returns what to save.
    pub fn key(&mut self, k: EditKey, now: ClockMs) -> EditAction {
        match k {
            EditKey::Enter { shift: true } if self.multiline => EditAction::None,
            EditKey::Enter { .. } | EditKey::Tab => self.commit(now),
            EditKey::Escape => {
                self.cancel();
                EditAction::None
            }
        }
    }
    /// Advances time: the saved check fades back to idle.
    pub fn tick(&mut self, now: ClockMs) {
        if let EditPhase::Saved(t) = self.phase
            && now.since(t).as_millis() >= u128::from(SAVED_MS)
        {
            self.phase = EditPhase::Idle;
        }
    }
    /// An outside change (Arc: replaces the text while not editing). Oxyn plus: while editing it
    /// raises a conflict instead of clobbering the draft.
    pub fn external(&mut self, v: &str, now: ClockMs) {
        if v == self.value {
            return;
        }
        if self.editing() {
            self.phase = EditPhase::Conflict(v.into());
        } else {
            self.value = v.into();
            self.draft = v.into();
            self.changed_at = now;
        }
    }
    /// Resolve a conflict: keep mine (`true`) or take theirs.
    pub fn resolve(&mut self, keep_mine: bool, now: ClockMs) -> EditAction {
        let EditPhase::Conflict(theirs) = self.phase.clone() else {
            return EditAction::None;
        };
        if keep_mine {
            self.value = theirs;
            self.phase = EditPhase::Editing;
            self.commit(now)
        } else {
            self.value = theirs.clone();
            self.draft = theirs;
            self.changed_at = now;
            self.phase = EditPhase::Idle;
            EditAction::None
        }
    }
    /// Use a suggestion as the draft.
    pub fn suggest(&mut self, i: usize) {
        if let Some(s) = self.suggestions.get(i).cloned() {
            if !self.editing() {
                self.start();
            }
            self.draft = s;
        }
    }
    /// The button's accessible name: `Project name: Atlas`.
    pub fn label(&self, label: &str) -> String {
        format!("{label}: {}", self.value)
    }
    /// The status line.
    pub fn status(&self) -> String {
        match &self.phase {
            EditPhase::Saving => "Saving…".into(),
            EditPhase::Saved(_) => "Saved".into(),
            EditPhase::Failed(m) => format!("Couldn't save: {m}"),
            EditPhase::Conflict(v) => format!("Changed to “{v}” while you were editing"),
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn edit_save_and_rollback() {
        let mut e = InlineEdit::new("Atlas");
        e.start();
        e.input("  Atlas v2 ");
        assert_eq!(
            e.key(EditKey::Enter { shift: false }, ClockMs(0)),
            EditAction::Save("Atlas v2".into())
        );
        assert_eq!(e.value, "Atlas v2", "optimistic");
        e.failed("Network error", ClockMs(10));
        assert_eq!(e.value, "Atlas");
        assert_eq!(e.status(), "Couldn't save: Network error");
        assert_eq!(e.retry(ClockMs(20)), EditAction::Save("Atlas v2".into()));
        e.saved(ClockMs(30));
        assert_eq!(e.phase.id(), "saved");
        e.tick(ClockMs(2_000));
        assert_eq!(e.phase, EditPhase::Idle);
        assert_eq!(e.label("Project name"), "Project name: Atlas v2");
    }

    #[test]
    fn validation_cancel_multiline() {
        let mut e = InlineEdit::new("Notes");
        e.max = Some(5);
        e.start();
        e.input("   ");
        assert_eq!(e.commit(ClockMs(0)), EditAction::None);
        assert!(e.error.is_some());
        e.input("Too long");
        e.commit(ClockMs(1));
        assert_eq!(e.error.as_deref(), Some("Keep it under 5 characters"));
        e.key(EditKey::Escape, ClockMs(2));
        assert_eq!(
            (e.phase.clone(), e.draft.as_str()),
            (EditPhase::Idle, "Notes")
        );
        e.multiline = true;
        e.start();
        assert_eq!(
            e.key(EditKey::Enter { shift: true }, ClockMs(3)),
            EditAction::None
        );
        e.input("Notes");
        assert_eq!(
            e.key(EditKey::Tab, ClockMs(4)),
            EditAction::None,
            "unchanged"
        );
        assert_eq!(e.phase, EditPhase::Idle);
    }

    #[test]
    fn conflicts_and_suggestions() {
        let mut e = InlineEdit::new("Draft");
        e.external("Draft 2", ClockMs(0));
        assert_eq!(e.value, "Draft 2");
        e.start();
        e.input("Mine");
        e.external("Agent's", ClockMs(1));
        assert!(matches!(e.phase, EditPhase::Conflict(_)));
        assert_eq!(e.resolve(true, ClockMs(2)), EditAction::Save("Mine".into()));
        e.saved(ClockMs(3));
        e.suggestions = vec!["Q4 launch plan".into()];
        e.suggest(0);
        assert!(e.editing());
        assert_eq!(e.draft, "Q4 launch plan");
    }
}
