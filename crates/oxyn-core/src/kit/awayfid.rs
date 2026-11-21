//! affectUI fidelity pass (plan #32, overnight batch, family B part 6): the model behind the exact
//! **Return brief** – the "While you were away" card for a chat thread: how long you were gone,
//! what needs you, what failed and what's done as a short list with one action per row; Got it
//! folds it into a one-line divider. Behaviour only; times are minutes.

/// What an item is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AwayKind {
    /// Waiting on the person.
    NeedsYou,
    /// Failed.
    Failed,
    /// Done.
    Done,
}

impl AwayKind {
    /// `needs-you` · `failed` · `done`.
    pub fn id(self) -> &'static str {
        match self {
            Self::NeedsYou => "needs-you",
            Self::Failed => "failed",
            Self::Done => "done",
        }
    }
    /// The default action's label.
    pub fn action(self) -> &'static str {
        match self {
            Self::NeedsYou => "Decide",
            Self::Failed => "Retry",
            Self::Done => "Open",
        }
    }
    /// The order rows sit in.
    pub fn rank(self) -> u8 {
        match self {
            Self::NeedsYou => 0,
            Self::Failed => 1,
            Self::Done => 2,
        }
    }
    /// The state in words.
    pub fn word(self) -> &'static str {
        match self {
            Self::NeedsYou => "needs you",
            Self::Failed => "failed",
            Self::Done => "done",
        }
    }
}

/// One thing that happened.
#[derive(Clone, Debug, PartialEq)]
pub struct AwayItem {
    /// Id.
    pub id: String,
    /// The task.
    pub title: String,
    /// What happened, one line.
    pub detail: String,
    /// Kind.
    pub kind: AwayKind,
    /// The button's label: `None` = the kind's default, `Some("")` = no button.
    pub action: Option<String>,
    /// What the row says once it resolves (needs-you answered, failed retried).
    pub resolved: String,
}

impl AwayItem {
    /// The label of its button, if any.
    pub fn action_label(&self) -> Option<String> {
        match &self.action {
            Some(s) if s.is_empty() => None,
            Some(s) => Some(s.clone()),
            None => Some(self.kind.action().to_string()),
        }
    }
}

/// Rows in order: needs you, then failed, then done – stable within a kind.
pub fn away_order(items: &[AwayItem]) -> Vec<AwayItem> {
    let mut v = items.to_vec();
    v.sort_by_key(|i| i.kind.rank());
    v
}

/// (needs you, failed, done).
pub fn away_counts(items: &[AwayItem]) -> (usize, usize, usize) {
    let n = |k| items.iter().filter(|i| i.kind == k).count();
    (
        n(AwayKind::NeedsYou),
        n(AwayKind::Failed),
        n(AwayKind::Done),
    )
}

/// "2h 14m" (or "45m").
pub fn away_duration(minutes: u32) -> String {
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m}m"),
        (h, m) => format!("{h}h {m}m"),
    }
}

/// "since 9:40" for `since` minutes after midnight (24-hour, as affectUI's clock style).
pub fn away_clock(since: u32) -> String {
    let m = since % (24 * 60);
    format!("since {}:{:02}", m / 60, m % 60)
}

/// "2 need you" / "1 needs you".
pub fn away_needs_word(n: usize) -> String {
    format!("{n} {}", if n == 1 { "needs you" } else { "need you" })
}

/// The folded divider's text: "You were away 2h 14m · 8 updates".
pub fn away_divider(head: &str, items: usize) -> String {
    format!(
        "You were away {head} · {items} update{}",
        if items == 1 { "" } else { "s" }
    )
}

/// The row after its action resolved: a needs-you or failed item becomes done with its resolved
/// line; done items stay as they are.
pub fn away_resolve(i: &AwayItem) -> AwayItem {
    if i.kind == AwayKind::Done {
        return i.clone();
    }
    AwayItem {
        kind: AwayKind::Done,
        detail: if i.resolved.is_empty() {
            i.detail.clone()
        } else {
            i.resolved.clone()
        },
        action: None,
        ..i.clone()
    }
}

/// The demo: 2 h 14 m away (our own tasks and copy).
pub const AWAY_MINUTES: u32 = 134;

/// The demo's items.
pub fn away_demo() -> Vec<AwayItem> {
    let it = |id: &str, title: &str, detail: &str, kind, resolved: &str| AwayItem {
        id: id.into(),
        title: title.into(),
        detail: detail.into(),
        kind,
        action: None,
        resolved: resolved.into(),
    };
    use AwayKind::*;
    vec![
        it(
            "export",
            "Export job",
            "Waiting on: CSV or Parquet for the archive?",
            NeedsYou,
            "Answered: Parquet · exporting now",
        ),
        it(
            "digest",
            "Release notes",
            "Waiting on: Post to the changelog or email only?",
            NeedsYou,
            "Answered: changelog · published",
        ),
        it(
            "sync",
            "Invoice sync",
            "Failed: timeout from the billing API after 912 rows",
            Failed,
            "Retried: all 2,040 rows synced",
        ),
        it(
            "cache",
            "Cache layer",
            "Opened PR #317 · on preview",
            Done,
            "",
        ),
        it(
            "search",
            "Search page",
            "Shipped to the beta group",
            Done,
            "",
        ),
        it("docs", "API reference", "Published 22 pages", Done, ""),
        it(
            "perf",
            "Load check",
            "Held at 15k requests a minute",
            Done,
            "",
        ),
        it(
            "flaky",
            "Flaky specs",
            "Quarantined 2 specs in onboarding",
            Done,
            "",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn order_counts_and_words() {
        let mut d = away_demo();
        d.swap(0, 7);
        let o = away_order(&d);
        assert_eq!(o[0].kind, AwayKind::NeedsYou);
        assert_eq!(o[2].kind, AwayKind::Failed);
        assert_eq!(away_counts(&o), (2, 1, 5));
        assert_eq!(away_duration(AWAY_MINUTES), "2h 14m");
        assert_eq!(away_duration(45), "45m");
        assert_eq!(away_clock(9 * 60 + 40), "since 9:40");
        assert_eq!(away_needs_word(1), "1 needs you");
        assert_eq!(
            away_divider("2h 14m", 8),
            "You were away 2h 14m · 8 updates"
        );
        let r = away_resolve(&away_demo()[0]);
        assert_eq!(r.kind, AwayKind::Done);
        assert_eq!(r.action_label().as_deref(), Some("Open"));
        assert!(r.detail.starts_with("Answered"));
        let mut n = away_demo()[3].clone();
        n.action = Some(String::new());
        assert_eq!(n.action_label(), None);
    }
}
