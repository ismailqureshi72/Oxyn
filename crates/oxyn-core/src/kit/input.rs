//! Inputs (plan #28, chunk 4a): the behaviour behind the text area, the labelled input and
//! textarea rows, and inline ghost-text autocomplete.
//!
//! - [`grow_rows`] / [`counter`]: a text area that grows with its text between a minimum and a
//!   maximum, and the "120/2,000" counter that turns red over the limit.
//! - [`word_changes`]: which words of a helper or error row changed, so only those rise in.
//! - [`Ghost`]: a suggestion that continues the text at the caret; typing that matches consumes
//!   it, Tab takes it all, Cmd/Alt+Right takes one word, Escape dismisses it.
//!
//! Behaviour-only rebuilds of the affectUI Text Area and the Arc UI Input, Textarea and Ghost
//! text; nothing is copied. Time is passed in as [`ClockMs`].

use crate::ClockMs;

use super::text::group;

/// Visible rows for `text` in a field `cols` characters wide: wrapped lines, at least `min`, at
/// most `max`. The flag says the field scrolls inside (the text needs more than `max`).
pub fn grow_rows(text: &str, cols: usize, min: usize, max: usize) -> (usize, bool) {
    let cols = cols.max(1);
    let lines: usize = text
        .split('\n')
        .map(|l| l.chars().count().div_ceil(cols).max(1))
        .sum();
    let max = max.max(min);
    (lines.clamp(min, max), lines > max)
}

/// The counter under a field: "120/2,000", and whether the text is over the limit.
pub fn counter(len: usize, max: usize) -> (String, bool) {
    (
        format!("{}/{}", group(len as i64), group(max as i64)),
        len > max,
    )
}

/// Words of `new` with a flag for the ones that differ from the word at the same place in
/// `old` (those rise in; the rest stay still).
pub fn word_changes(old: &str, new: &str) -> Vec<(String, bool)> {
    let o: Vec<&str> = old.split_whitespace().collect();
    new.split_whitespace()
        .enumerate()
        .map(|(i, w)| (w.to_string(), o.get(i) != Some(&w)))
        .collect()
}

/// Characters of `new` with a flag for the ones that changed against `old`, matched from the
/// end (so "98" → "102" rolls the last two places and grows the first): counts roll only the
/// digits that changed.
pub fn digit_changes(old: &str, new: &str) -> Vec<(char, bool)> {
    let o: Vec<char> = old.chars().collect();
    let n: Vec<char> = new.chars().collect();
    n.iter()
        .enumerate()
        .map(|(i, c)| {
            let from_end = n.len() - i;
            let oc = o.len().checked_sub(from_end).map(|j| o[j]);
            (*c, oc != Some(*c))
        })
        .collect()
}

/// Where a ghost-text suggestion stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GhostStatus {
    /// Nothing asked.
    #[default]
    Idle,
    /// Waiting on the model.
    Loading,
    /// A suggestion shows.
    Ready,
    /// The last ask failed.
    Error,
}

impl GhostStatus {
    /// Chip label for the model named `model`.
    pub fn chip(self, model: &str) -> String {
        match self {
            Self::Idle => model.to_string(),
            Self::Loading => "Thinking".into(),
            Self::Ready => "Tab to accept".into(),
            Self::Error => "No suggestion".into(),
        }
    }
}

/// Inline autocomplete state for one field.
#[derive(Clone, Debug, PartialEq)]
pub struct Ghost {
    /// The continuation showing after the text.
    pub suggestion: Option<String>,
    /// Status.
    pub status: GhostStatus,
    /// Last edit (the pause before asking runs from here).
    pub edited: ClockMs,
    /// The text the pending ask was made for.
    pub asked_for: Option<String>,
    /// Pause before asking, ms.
    pub delay_ms: u64,
    /// Trimmed characters needed before asking.
    pub min_len: usize,
    /// Text just accepted (eases from muted to solid), with when.
    pub accepted: Option<(String, ClockMs)>,
}

impl Default for Ghost {
    fn default() -> Self {
        Self {
            suggestion: None,
            status: GhostStatus::Idle,
            edited: ClockMs(0),
            asked_for: None,
            delay_ms: 450,
            min_len: 3,
            accepted: None,
        }
    }
}

/// End of the next word of `s`: leading spaces, then the word, as Cmd+Right takes it.
pub fn next_word_end(s: &str) -> usize {
    let lead = s.len() - s.trim_start().len();
    let rest = &s[lead..];
    lead + rest
        .char_indices()
        .find(|(_, c)| c.is_whitespace())
        .map(|(i, _)| i)
        .unwrap_or(rest.len())
}

impl Ghost {
    /// The text changed from `old` to `new` at `now`; `caret_end` says the caret sits at the end.
    /// Typing that matches the start of the ghost consumes it in place; anything else clears it.
    pub fn edit(&mut self, old: &str, new: &str, caret_end: bool, now: ClockMs) {
        self.edited = now;
        self.accepted = None;
        let keep = caret_end
            && new.len() > old.len()
            && new.starts_with(old)
            && self.suggestion.as_deref().is_some_and(|g| {
                g.starts_with(&new[old.len()..]) && g.len() > new.len() - old.len()
            });
        if keep {
            let typed = new.len() - old.len();
            if let Some(g) = self.suggestion.as_mut() {
                g.drain(..typed);
            }
            return;
        }
        self.suggestion = None;
        self.asked_for = None;
        self.status = GhostStatus::Idle;
    }
    /// The caret left the end of the text (arrow keys, a click): the suggestion goes.
    pub fn caret_moved(&mut self) {
        if self.suggestion.take().is_some() {
            self.status = GhostStatus::Idle;
        }
    }
    /// Time to ask for `text` at `now`: paused long enough, long enough, nothing showing or
    /// pending.
    pub fn due(&self, text: &str, now: ClockMs) -> bool {
        self.suggestion.is_none()
            && self.asked_for.is_none()
            && self.status != GhostStatus::Error
            && text.trim().chars().count() >= self.min_len
            && now.0.saturating_sub(self.edited.0) >= self.delay_ms
    }
    /// Starts an ask for `text`.
    pub fn ask(&mut self, text: &str) {
        self.asked_for = Some(text.to_string());
        self.status = GhostStatus::Loading;
    }
    /// The answer for `text`; a stale answer (the text moved on) is dropped.
    pub fn answer(&mut self, text: &str, s: Option<String>) {
        if self.asked_for.as_deref() != Some(text) {
            return;
        }
        match s.filter(|s| !s.is_empty()) {
            Some(s) => {
                self.suggestion = Some(s);
                self.status = GhostStatus::Ready;
            }
            None => self.status = GhostStatus::Idle,
        }
    }
    /// Runs the ask on the clock: once `text` has paused for `delay_ms` it is asked for, and
    /// `latency_ms` later `f` answers (`Err` marks the ask failed). Pure in time, so a view can
    /// settle a copy each frame and an event handler settles the real one before acting.
    pub fn settle(
        &mut self,
        text: &str,
        now: ClockMs,
        latency_ms: u64,
        f: impl Fn(&str) -> Result<Option<String>, ()>,
    ) {
        if self.due(text, now) {
            self.ask(text);
        }
        if self.status == GhostStatus::Loading
            && now.0.saturating_sub(self.edited.0) >= self.delay_ms + latency_ms
        {
            match f(text) {
                Ok(s) => self.answer(text, s),
                Err(()) => self.fail(),
            }
        }
    }
    /// The ask failed.
    pub fn fail(&mut self) {
        self.status = GhostStatus::Error;
    }
    /// Tab: the whole suggestion, to append.
    pub fn accept_all(&mut self, now: ClockMs) -> Option<String> {
        let g = self.suggestion.take()?;
        self.status = GhostStatus::Idle;
        self.asked_for = None;
        self.edited = now;
        self.accepted = Some((g.clone(), now));
        Some(g)
    }
    /// Cmd/Alt+Right: the next word, to append; the rest stays.
    pub fn accept_word(&mut self, now: ClockMs) -> Option<String> {
        let g = self.suggestion.as_mut()?;
        let n = next_word_end(g);
        let w: String = g.drain(..n).collect();
        if g.trim().is_empty() {
            self.suggestion = None;
            self.status = GhostStatus::Idle;
            self.asked_for = None;
        }
        self.edited = now;
        self.accepted = Some((w.clone(), now));
        Some(w)
    }
    /// Escape: the dismissed suggestion (None leaves Escape to the page).
    pub fn dismiss(&mut self) -> Option<String> {
        let g = self.suggestion.take()?;
        self.status = GhostStatus::Idle;
        Some(g)
    }
}

/// Canned continuations for demos: the first entry whose key ends the trimmed text.
pub fn demo_suggestion(text: &str) -> Option<String> {
    const TABLE: [(&str, &str); 8] = [
        ("thanks for", " the quick turnaround on the review."),
        ("let me know", " if you have any questions."),
        ("looking forward", " to hearing from you."),
        ("i'll send", " the updated draft by Friday."),
        ("the build", " passes locally; CI should be green soon."),
        ("can we", " move the sync to Thursday afternoon?"),
        ("hi", " team, a quick update on the launch:"),
        ("please", " review the attached notes before the call."),
    ];
    let t = text.trim_end().to_lowercase();
    TABLE
        .iter()
        .find(|(k, _)| t.ends_with(k))
        .map(|(_, v)| v.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rows_and_counter() {
        assert_eq!(grow_rows("", 40, 3, 10), (3, false));
        assert_eq!(grow_rows(&"x".repeat(200), 40, 3, 10), (5, false));
        assert_eq!(grow_rows(&"a\n".repeat(14), 40, 3, 10), (10, true));
        assert_eq!(counter(120, 2000), ("120/2,000".into(), false));
        assert!(counter(2001, 2000).1);
    }

    #[test]
    fn changed_words() {
        let w = word_changes("Use 8 characters", "Use 12 characters");
        assert_eq!(
            w.iter().map(|x| x.1).collect::<Vec<_>>(),
            [false, true, false]
        );
    }

    #[test]
    fn changed_digits() {
        let d = digit_changes("98", "102");
        assert_eq!(
            d.iter().map(|x| x.1).collect::<Vec<_>>(),
            [true, true, true]
        );
        let d = digit_changes("120", "121");
        assert_eq!(
            d.iter().map(|x| x.1).collect::<Vec<_>>(),
            [false, false, true]
        );
    }

    #[test]
    fn ghost_flow() {
        let mut g = Ghost::default();
        g.edit("", "Thanks for", true, ClockMs(0));
        assert!(!g.due("Thanks for", ClockMs(100)));
        assert!(g.due("Thanks for", ClockMs(500)));
        g.ask("Thanks for");
        g.answer("Thanks for", demo_suggestion("Thanks for"));
        assert_eq!(g.status, GhostStatus::Ready);
        // Matching typing consumes the ghost in place.
        g.edit("Thanks for", "Thanks for t", true, ClockMs(600));
        assert_eq!(
            g.suggestion.as_deref(),
            Some("he quick turnaround on the review.")
        );
        assert_eq!(g.accept_word(ClockMs(700)).as_deref(), Some("he"));
        assert_eq!(g.accept_word(ClockMs(800)).as_deref(), Some(" quick"));
        assert_eq!(
            g.accept_all(ClockMs(900)).as_deref(),
            Some(" turnaround on the review.")
        );
        assert!(g.suggestion.is_none());
        // A different character clears it; a stale answer is dropped.
        g.ask("Can we");
        g.edit("Can we", "Can wex", true, ClockMs(1000));
        g.answer("Can we", Some(" move".into()));
        assert!(g.suggestion.is_none());
        assert_eq!(g.dismiss(), None);
        // On the clock: asked after the pause, answered after the latency.
        let mut g = Ghost::default();
        g.edit("", "Let me know", true, ClockMs(1000));
        let f = |t: &str| Ok(demo_suggestion(t));
        g.settle("Let me know", ClockMs(1500), 300, f);
        assert_eq!(g.status, GhostStatus::Loading);
        g.settle("Let me know", ClockMs(1800), 300, f);
        assert_eq!(g.suggestion.as_deref(), Some(" if you have any questions."));
        assert_eq!(g.dismiss().as_deref(), Some(" if you have any questions."));
        g.settle("Let me know", ClockMs(5000), 300, f);
        assert!(
            g.suggestion.is_none(),
            "a dismissed text is not asked again"
        );
    }
}
