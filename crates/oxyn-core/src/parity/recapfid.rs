//! affectUI fidelity pass (plan #32, overnight batch, family B part 4): the model behind the exact
//! **Agent Recap** – what the agents did while you were away: the night as a small chart with a
//! peak per event, then a board of what needs you, what failed and what's done, acted on right on
//! the card. Behaviour only; times are minutes after the person left.

/// Which lane an event sits in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecapKind {
    /// Needs a decision.
    Needs,
    /// Failed.
    Failed,
    /// Done by an agent.
    Done,
}

impl RecapKind {
    /// `needs` · `failed` · `done`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Needs => "needs",
            Self::Failed => "failed",
            Self::Done => "done",
        }
    }
    /// The lane's heading.
    pub fn lane(self) -> &'static str {
        match self {
            Self::Needs => "Needs you",
            Self::Failed => "Failed",
            Self::Done => "Done",
        }
    }
}

/// One thing that happened.
#[derive(Clone, Debug, PartialEq)]
pub struct RecapEvent {
    /// Id.
    pub id: String,
    /// Minutes after the person left.
    pub at: u32,
    /// Lane.
    pub kind: RecapKind,
    /// Title.
    pub title: String,
    /// One line under it.
    pub detail: String,
    /// Brand mark.
    pub brand: String,
    /// How many (a pill; 0 hides it) – also the peak's height.
    pub count: u32,
    /// The main action ("Approve", "Merge", "Retry"); empty for done cards.
    pub action: String,
    /// The tag once acted on ("Approved", "Merged", "Fixed").
    pub done_label: String,
}

/// Where a card is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecapCardState {
    /// As it came.
    Open,
    /// Its action is working.
    Busy,
    /// Acted on (needs / failed) – shows the done label and Undo.
    Acted,
    /// A done card undone – shows Undone and Redo.
    Undone,
}

/// "7:20 PM" for `minutes` after `since` (minutes after midnight).
pub fn recap_time(since: u32, minutes: u32) -> String {
    let m = (since + minutes) % (24 * 60);
    let (h, mm) = (m / 60, m % 60);
    let h12 = if h % 12 == 0 { 12 } else { h % 12 };
    format!("{h12}:{mm:02} {}", if h < 12 { "AM" } else { "PM" })
}

/// "13h 30m".
pub fn recap_span(minutes: u32) -> String {
    match (minutes / 60, minutes % 60) {
        (0, m) => format!("{m}m"),
        (h, 0) => format!("{h}h"),
        (h, m) => format!("{h}h {m}m"),
    }
}

/// The curve: `n` points (x 0–1, y 0–1 with 1 = top) – a soft peak per event, its height by count.
pub fn recap_curve(events: &[RecapEvent], span: u32, n: usize) -> Vec<(f32, f32)> {
    let span = span.max(1) as f32;
    let max = events.iter().map(|e| e.count.max(1)).max().unwrap_or(1) as f32;
    let w = 0.028f32;
    (0..n)
        .map(|i| {
            let x = i as f32 / (n - 1).max(1) as f32;
            let y = events
                .iter()
                .map(|e| {
                    let h = 0.55 + 0.45 * (e.count.max(1) as f32 / max).sqrt();
                    let d = (x - e.at as f32 / span) / w;
                    h * (-d * d).exp()
                })
                .fold(0.0f32, f32::max);
            (x, y.min(1.0))
        })
        .collect()
}

/// The peak's height (0–1) for an event, on the same curve.
pub fn recap_peak(events: &[RecapEvent], e: &RecapEvent, span: u32) -> f32 {
    let x = e.at as f32 / span.max(1) as f32;
    let c = recap_curve(events, span, 401);
    c.iter()
        .min_by(|a, b| (a.0 - x).abs().total_cmp(&(b.0 - x).abs()))
        .map_or(0.0, |p| p.1)
}

/// (done, need you, failed) given each card's state: acting on a needs or failed card makes it
/// done; undoing a done card takes it off.
pub fn recap_counts(
    events: &[RecapEvent],
    state: &dyn Fn(&str) -> RecapCardState,
) -> (u32, u32, u32) {
    let (mut d, mut n, mut f) = (0, 0, 0);
    for e in events {
        match (e.kind, state(&e.id)) {
            (RecapKind::Done, RecapCardState::Undone) => {}
            (RecapKind::Done, _) => d += 1,
            (_, RecapCardState::Acted) => d += 1,
            (RecapKind::Needs, _) => n += 1,
            (RecapKind::Failed, _) => f += 1,
        }
    }
    (d, n, f)
}

/// The demo night: since 6:40 PM, 13 h 30 m.
pub const RECAP_SINCE: u32 = 18 * 60 + 40;
/// Minutes the demo night lasts.
pub const RECAP_SPAN: u32 = 13 * 60 + 30;
/// How long the night replays (ms).
pub const RECAP_REPLAY_MS: u64 = 1_800;
/// How long an action works before it settles (ms).
pub const RECAP_BUSY_MS: u64 = 1_000;

#[allow(clippy::too_many_arguments)]
fn ev(
    id: &str,
    at: u32,
    kind: RecapKind,
    title: &str,
    detail: &str,
    brand: &str,
    count: u32,
    action: &str,
    done: &str,
) -> RecapEvent {
    RecapEvent {
        id: id.into(),
        at,
        kind,
        title: title.into(),
        detail: detail.into(),
        brand: brand.into(),
        count,
        action: action.into(),
        done_label: done.into(),
    }
}

/// The demo's six events.
pub fn recap_demo() -> Vec<RecapEvent> {
    use RecapKind::*;
    vec![
        ev(
            "tickets",
            40,
            Done,
            "Replied to support tickets",
            "first answer within 3 min",
            "slack",
            41,
            "",
            "",
        ),
        ev(
            "prs",
            145,
            Done,
            "Closed stale pull requests",
            "none touched in 60 days",
            "github",
            7,
            "",
            "",
        ),
        ev(
            "issues",
            270,
            Done,
            "Sorted the new issues",
            "5 flagged urgent",
            "linear",
            19,
            "",
            "",
        ),
        ev(
            "refund",
            430,
            Needs,
            "Refund $180 to Northwind",
            "Support agent · charged twice in March",
            "slack",
            0,
            "Approve",
            "Approved",
        ),
        ev(
            "sync",
            480,
            Failed,
            "Calendar sync to the CRM",
            "Token expired after 1,480 of 2,000 contacts",
            "googledrive",
            0,
            "Retry",
            "Fixed",
        ),
        ev(
            "merge",
            575,
            Needs,
            "Merge the login fix",
            "All 186 tests pass · needs a reviewer",
            "github",
            0,
            "Merge",
            "Merged",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times_counts_and_the_curve() {
        assert_eq!(recap_time(RECAP_SINCE, 0), "6:40 PM");
        assert_eq!(recap_time(RECAP_SINCE, RECAP_SPAN), "8:10 AM");
        assert_eq!(recap_time(RECAP_SINCE, 430), "1:50 AM");
        assert_eq!(recap_span(RECAP_SPAN), "13h 30m");
        let e = recap_demo();
        assert_eq!(recap_counts(&e, &|_| RecapCardState::Open), (3, 2, 1));
        assert_eq!(
            recap_counts(&e, &|id| if id == "refund" {
                RecapCardState::Acted
            } else {
                RecapCardState::Open
            }),
            (4, 1, 1)
        );
        assert_eq!(
            recap_counts(&e, &|id| if id == "prs" {
                RecapCardState::Undone
            } else {
                RecapCardState::Open
            }),
            (2, 2, 1)
        );
        let c = recap_curve(&e, RECAP_SPAN, 101);
        assert_eq!(c.len(), 101);
        assert!(recap_peak(&e, &e[0], RECAP_SPAN) > 0.95);
        assert!(recap_peak(&e, &e[1], RECAP_SPAN) < recap_peak(&e, &e[0], RECAP_SPAN));
    }
}
