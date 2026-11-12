//! Approvals (`approvals.md`, `approval-card.md`): requests with a risk tier, the card's input
//! rules (input guard, arming for High, press-and-hold for Critical, pausing countdown), the
//! queue with paging and batch answers, and "always allow" pattern suggestions.

use std::time::Duration;

use crate::ClockMs;

/// Risk tier. Always shown with a glyph and a word, never colour alone.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Risk {
    /// Reads, reversible.
    #[default]
    Low,
    /// Writes inside the workspace.
    Medium,
    /// Hard to undo.
    High,
    /// Money, credentials, production, deletion outside the workspace.
    Critical,
}

impl Risk {
    /// Every tier.
    pub const ALL: [Risk; 4] = [Self::Low, Self::Medium, Self::High, Self::Critical];

    /// Glyph (○ ◇ ▲ ⬣).
    pub fn glyph(self) -> &'static str {
        match self {
            Self::Low => "○",
            Self::Medium => "◇",
            Self::High => "▲",
            Self::Critical => "⬣",
        }
    }

    /// "Low risk".
    pub fn label(self) -> &'static str {
        match self {
            Self::Low => "Low risk",
            Self::Medium => "Medium risk",
            Self::High => "High risk",
            Self::Critical => "Critical",
        }
    }

    /// Id (`low`, `medium`, `high`, `critical`).
    pub fn id(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
            Self::Critical => "critical",
        }
    }

    /// Enter maps to the safe default: Allow once for Low / Medium, Deny for High / Critical.
    pub fn enter_allows(self) -> bool {
        self <= Self::Medium
    }

    /// "Always allow" is offered only up to Medium (apps may widen it).
    pub fn always_allowed(self) -> bool {
        self <= Self::Medium
    }
}

/// What the user answered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Decision {
    /// Allow this call.
    AllowOnce,
    /// Allow for this session.
    AllowSession,
    /// Allow calls matching `pattern` from now on.
    AllowAlways {
        /// Pattern (`cargo test *`).
        pattern: String,
    },
    /// Allow an edited version.
    AllowEdited {
        /// The edited command or arguments.
        edited: String,
    },
    /// Deny, optionally telling the agent why.
    Deny {
        /// The user's note.
        reason: Option<String>,
    },
    /// Denied by the countdown.
    Expired,
}

impl Decision {
    /// Allowed in some form.
    pub fn allows(&self) -> bool {
        matches!(
            self,
            Self::AllowOnce
                | Self::AllowSession
                | Self::AllowAlways { .. }
                | Self::AllowEdited { .. }
        )
    }

    /// The resolved row's text.
    pub fn row(&self) -> String {
        match self {
            Self::AllowOnce => "Allowed once".into(),
            Self::AllowSession => "Allowed for this session".into(),
            Self::AllowAlways { pattern } => format!("Always allowed `{pattern}`"),
            Self::AllowEdited { .. } => "Allowed with your edit".into(),
            Self::Deny { reason: Some(r) } => format!("Denied · \u{201c}{r}\u{201d}"),
            Self::Deny { reason: None } => "Denied".into(),
            Self::Expired => "Timed out · denied".into(),
        }
    }
}

/// One open request.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ApprovalRequest {
    /// Id.
    pub id: String,
    /// The action ("Run cargo test", "Send campaign email"); the card's accessible name.
    pub title: String,
    /// One line of context.
    pub description: Option<String>,
    /// Tray label ("Step 4 · gmail.send").
    pub label: Option<String>,
    /// Tier.
    pub risk: Risk,
    /// Why it is that risky ("Deletes files outside git").
    pub reasons: Vec<String>,
    /// Key-value rows.
    pub details: Vec<(String, String)>,
    /// The exact command, when there is one (shown mono; editable).
    pub command: Option<String>,
    /// Amount for spend requests ("$420.00").
    pub amount: Option<String>,
    /// Opened.
    pub opened_at: ClockMs,
    /// Expires after this (countdown pauses while hovered or focused).
    pub expires_in: Option<Duration>,
    /// Offer "Always allow".
    pub allow_always: bool,
    /// Offer "For this session".
    pub allow_session: bool,
}

impl ApprovalRequest {
    /// A request.
    pub fn new(id: &str, title: &str, risk: Risk, at: ClockMs) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            description: None,
            label: None,
            risk,
            reasons: vec![],
            details: vec![],
            command: None,
            amount: None,
            opened_at: at,
            expires_in: None,
            allow_always: risk.always_allowed(),
            allow_session: risk <= Risk::High,
        }
    }
}

/// Card timings (`approval-card.md` §5).
pub mod timing {
    use std::time::Duration;

    /// Ignore decisions this soon after the card appears (stray key presses).
    pub const INPUT_GUARD: Duration = Duration::from_millis(400);
    /// High: Allow fills before it can be pressed.
    pub const ARMING: Duration = Duration::from_millis(600);
    /// Critical: hold to allow.
    pub const HOLD: Duration = Duration::from_millis(1200);
    /// A released hold drains.
    pub const DRAIN: Duration = Duration::from_millis(200);
    /// Countdown turns warning in the last 10 s.
    pub const COUNTDOWN_WARN: Duration = Duration::from_secs(10);
    /// Card rises in.
    pub const ENTER: Duration = Duration::from_millis(200);
    /// Card morphs into its resolved row.
    pub const RESOLVE: Duration = Duration::from_millis(220);
    /// A spinner shows on the pressed button only after this.
    pub const SPINNER_AFTER: Duration = Duration::from_millis(300);
}

/// The card's phase.
#[derive(Clone, Debug, PartialEq)]
pub enum CardPhase {
    /// Rising in; decisions ignored.
    Entering,
    /// High: Allow is filling (`p` 0..1).
    Arming(f32),
    /// Ready.
    Open,
    /// Critical: Allow held (`p` 0..1).
    Holding(f32),
    /// Critical: released early; draining from `p`.
    Draining(f32),
    /// Sent; waiting for the agent.
    Submitting(Decision),
    /// Sending failed; the call stays blocked.
    SubmitFailed(Decision),
    /// Answered (morph progress `0..1`).
    Resolved {
        /// What was decided.
        decision: Decision,
        /// After how long the user answered.
        after: Duration,
        /// Collapse progress.
        morph: f32,
    },
    /// The run stopped; no longer needed.
    Stale,
}

/// Why a press was ignored.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// Inside the input guard.
    TooSoon,
    /// High risk, still arming ("Available in a moment").
    Arming,
    /// Critical: must hold.
    MustHold,
    /// Not offered at this tier.
    NotOffered,
    /// Already decided or stale.
    Closed,
}

/// The card's input rules and countdown, driven by events and `now`.
#[derive(Clone, Debug, PartialEq)]
pub struct ApprovalCard {
    /// The request.
    pub request: ApprovalRequest,
    paused: Duration,
    hover_since: Option<ClockMs>,
    hold_since: Option<ClockMs>,
    released: Option<(ClockMs, f32)>,
    sent: Option<(Decision, ClockMs)>,
    resolved: Option<(Decision, ClockMs)>,
    failed: bool,
    stale: bool,
}

impl ApprovalCard {
    /// A card for `request`.
    pub fn new(request: ApprovalRequest) -> Self {
        Self {
            request,
            paused: Duration::ZERO,
            hover_since: None,
            hold_since: None,
            released: None,
            sent: None,
            resolved: None,
            failed: false,
            stale: false,
        }
    }

    fn age(&self, now: ClockMs) -> Duration {
        now.since(self.request.opened_at)
    }

    /// Hover or focus entered / left (pauses the countdown).
    pub fn set_hover(&mut self, on: bool, now: ClockMs) {
        match (on, self.hover_since) {
            (true, None) => self.hover_since = Some(now),
            (false, Some(s)) => {
                self.paused += now.since(s);
                self.hover_since = None;
            }
            _ => {}
        }
    }

    /// Time left on the countdown, if any.
    pub fn remaining(&self, now: ClockMs) -> Option<Duration> {
        let total = self.request.expires_in?;
        let paused = self.paused + self.hover_since.map(|s| now.since(s)).unwrap_or_default();
        Some(total.saturating_sub(self.age(now).saturating_sub(paused)))
    }

    /// "Expires in 0:42", with whether it is in the warning window.
    pub fn countdown(&self, now: ClockMs) -> Option<(String, bool)> {
        if self.is_closed() {
            return None;
        }
        let r = self.remaining(now)?;
        let s = r.as_secs_f32().ceil() as u64;
        Some((
            format!("Expires in {}:{:02}", s / 60, s % 60),
            r <= timing::COUNTDOWN_WARN,
        ))
    }

    fn is_closed(&self) -> bool {
        self.resolved.is_some() || self.stale || (self.sent.is_some() && !self.failed)
    }

    /// Press a decision. Allow on Critical needs [`hold_start`](Self::hold_start).
    pub fn press(&mut self, d: Decision, now: ClockMs) -> Result<(), Refusal> {
        if self.is_closed() {
            return Err(Refusal::Closed);
        }
        if self.age(now) < timing::INPUT_GUARD {
            return Err(Refusal::TooSoon);
        }
        let r = self.request.risk;
        match &d {
            Decision::AllowAlways { .. } if !self.request.allow_always => {
                return Err(Refusal::NotOffered);
            }
            Decision::AllowSession if !self.request.allow_session => {
                return Err(Refusal::NotOffered);
            }
            d if d.allows() && r == Risk::Critical => return Err(Refusal::MustHold),
            d if d.allows()
                && r == Risk::High
                && self.age(now) < timing::INPUT_GUARD + timing::ARMING =>
            {
                return Err(Refusal::Arming);
            }
            _ => {}
        }
        self.failed = false;
        self.sent = Some((d, now));
        Ok(())
    }

    /// The Enter key: the tier's safe default.
    pub fn press_enter(&mut self, now: ClockMs) -> Result<(), Refusal> {
        if self.request.risk.enter_allows() {
            self.press(Decision::AllowOnce, now)
        } else {
            self.press(Decision::Deny { reason: None }, now)
        }
    }

    /// Critical: hold started (pointer, Y or Space).
    pub fn hold_start(&mut self, now: ClockMs) -> Result<(), Refusal> {
        if self.is_closed() {
            return Err(Refusal::Closed);
        }
        if self.age(now) < timing::INPUT_GUARD {
            return Err(Refusal::TooSoon);
        }
        self.hold_since = Some(now);
        self.released = None;
        Ok(())
    }

    /// Critical: hold released. Sends Allow once if it was held long enough.
    pub fn hold_end(&mut self, now: ClockMs) {
        if let Some(s) = self.hold_since.take() {
            let p = (now.since(s).as_secs_f32() / timing::HOLD.as_secs_f32()).min(1.0);
            if p >= 1.0 {
                self.sent = Some((Decision::AllowOnce, now));
            } else {
                self.released = Some((now, p));
            }
        }
    }

    /// The agent acknowledged the decision.
    pub fn ack(&mut self, now: ClockMs) {
        if let Some((d, _)) = self.sent.take() {
            self.resolved = Some((d, now));
        }
    }

    /// Sending failed.
    pub fn fail(&mut self) {
        self.failed = true;
    }

    /// The run stopped.
    pub fn stale(&mut self) {
        self.stale = true;
    }

    /// Advances time-driven transitions: a finished hold sends, an expired countdown denies.
    pub fn tick(&mut self, now: ClockMs) {
        if let Some(s) = self.hold_since
            && now.since(s) >= timing::HOLD
        {
            self.hold_since = None;
            self.sent = Some((Decision::AllowOnce, now));
        }
        if !self.is_closed() && self.remaining(now) == Some(Duration::ZERO) {
            self.sent = None;
            self.resolved = Some((Decision::Expired, now));
        }
    }

    /// The frame at `now`.
    pub fn phase(&self, now: ClockMs) -> CardPhase {
        if self.stale {
            return CardPhase::Stale;
        }
        if let Some((d, at)) = &self.resolved {
            let morph = (now.since(*at).as_secs_f32() / timing::RESOLVE.as_secs_f32()).min(1.0);
            return CardPhase::Resolved {
                decision: d.clone(),
                after: at.since(self.request.opened_at),
                morph,
            };
        }
        if let Some((d, _)) = &self.sent {
            return if self.failed {
                CardPhase::SubmitFailed(d.clone())
            } else {
                CardPhase::Submitting(d.clone())
            };
        }
        if let Some(s) = self.hold_since {
            return CardPhase::Holding(
                (now.since(s).as_secs_f32() / timing::HOLD.as_secs_f32()).min(1.0),
            );
        }
        if let Some((at, p)) = self.released {
            let left = p - now.since(at).as_secs_f32() / timing::DRAIN.as_secs_f32();
            if left > 0.0 {
                return CardPhase::Draining(left);
            }
        }
        let age = self.age(now);
        if age < timing::INPUT_GUARD {
            return CardPhase::Entering;
        }
        if self.request.risk == Risk::High && age < timing::INPUT_GUARD + timing::ARMING {
            return CardPhase::Arming(
                (age - timing::INPUT_GUARD).as_secs_f32() / timing::ARMING.as_secs_f32(),
            );
        }
        CardPhase::Open
    }

    /// Whether a spinner shows on the pressed button.
    pub fn spinner(&self, now: ClockMs) -> bool {
        matches!(&self.sent, Some((_, at)) if !self.failed && now.since(*at) >= timing::SPINNER_AFTER)
    }

    /// The resolved row: "Allowed once · 0:07" (the renderer adds ✓ or ⊘).
    pub fn resolved_row(&self) -> Option<String> {
        let (d, at) = self.resolved.as_ref()?;
        let s = at.since(self.request.opened_at).as_secs();
        Some(format!("{} · {}:{:02}", d.row(), s / 60, s % 60))
    }

    /// Polite announcement after a decision ("Approved: Send campaign email").
    pub fn announcement(&self) -> Option<String> {
        let (d, _) = self.resolved.as_ref()?;
        Some(format!(
            "{}: {}",
            if d.allows() { "Approved" } else { "Denied" },
            self.request.title
        ))
    }
}

/// Pattern suggestions for "Always allow", narrowest first: `cargo test -p core` →
/// `cargo test -p core`, `cargo test *`, `cargo *`.
pub fn pattern_ladder(command: &str) -> Vec<String> {
    let words: Vec<&str> = command.split_whitespace().collect();
    let mut out = vec![words.join(" ")];
    for keep in (1..words.len().min(3)).rev() {
        let p = format!("{} *", words[..keep].join(" "));
        if !out.contains(&p) {
            out.push(p);
        }
    }
    out
}

/// Whether `command` matches an allow `pattern` (`*` matches the rest).
pub fn pattern_matches(pattern: &str, command: &str) -> bool {
    match pattern.strip_suffix(" *") {
        Some(prefix) => command == prefix || command.starts_with(&format!("{prefix} ")),
        None => pattern == command,
    }
}

/// Open requests as one paged card.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ApprovalQueue {
    /// Requests in arrival order.
    pub requests: Vec<ApprovalRequest>,
    /// Answers by id.
    pub answers: Vec<(String, Decision)>,
}

impl ApprovalQueue {
    /// Unanswered requests.
    pub fn open(&self) -> Vec<&ApprovalRequest> {
        self.requests
            .iter()
            .filter(|r| !self.answers.iter().any(|(id, _)| *id == r.id))
            .collect()
    }

    /// "1 of 3".
    pub fn pager(&self, page: usize) -> Option<String> {
        let n = self.open().len();
        (n > 1).then(|| format!("{} of {n}", page.min(n - 1) + 1))
    }

    /// Batch row offered: two or more open requests, all Medium or lower.
    pub fn batch(&self) -> Option<String> {
        let open = self.open();
        (open.len() >= 2 && open.iter().all(|r| r.risk <= Risk::Medium))
            .then(|| format!("Allow all {} once", open.len()))
    }

    /// Answer one request.
    pub fn answer(&mut self, id: &str, d: Decision) {
        self.answers.push((id.into(), d));
    }

    /// Answer every open request (batch).
    pub fn answer_all(&mut self, d: Decision) {
        let ids: Vec<String> = self.open().iter().map(|r| r.id.clone()).collect();
        for id in ids {
            self.answer(&id, d.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(r: Risk) -> ApprovalCard {
        let mut q = ApprovalRequest::new("1", "Run cargo test", r, ClockMs(0));
        q.expires_in = Some(Duration::from_secs(60));
        ApprovalCard::new(q)
    }

    #[test]
    fn guard_arming_and_enter_default() {
        let mut c = card(Risk::Low);
        assert_eq!(
            c.press(Decision::AllowOnce, ClockMs(100)),
            Err(Refusal::TooSoon)
        );
        assert_eq!(c.phase(ClockMs(100)), CardPhase::Entering);
        assert!(c.press_enter(ClockMs(500)).is_ok());
        assert_eq!(
            c.phase(ClockMs(500)),
            CardPhase::Submitting(Decision::AllowOnce)
        );
        assert!(!c.spinner(ClockMs(700)) && c.spinner(ClockMs(900)));
        c.ack(ClockMs(7_000));
        assert_eq!(c.resolved_row().as_deref(), Some("Allowed once · 0:07"));
        assert_eq!(
            c.announcement().as_deref(),
            Some("Approved: Run cargo test")
        );

        let mut h = card(Risk::High);
        assert!(matches!(h.phase(ClockMs(700)), CardPhase::Arming(_)));
        assert_eq!(
            h.press(Decision::AllowOnce, ClockMs(700)),
            Err(Refusal::Arming)
        );
        assert_eq!(
            h.press(
                Decision::AllowAlways {
                    pattern: "x".into()
                },
                ClockMs(2_000)
            ),
            Err(Refusal::NotOffered)
        );
        assert!(h.press_enter(ClockMs(2_000)).is_ok());
        assert_eq!(
            h.phase(ClockMs(2_000)),
            CardPhase::Submitting(Decision::Deny { reason: None })
        );
    }

    #[test]
    fn critical_hold_and_drain() {
        let mut c = card(Risk::Critical);
        assert_eq!(
            c.press(Decision::AllowOnce, ClockMs(1_000)),
            Err(Refusal::MustHold)
        );
        c.hold_start(ClockMs(1_000)).unwrap();
        assert_eq!(c.phase(ClockMs(1_600)), CardPhase::Holding(0.5));
        c.hold_end(ClockMs(1_600));
        assert!(matches!(c.phase(ClockMs(1_650)), CardPhase::Draining(p) if p > 0.2 && p < 0.3));
        assert_eq!(c.phase(ClockMs(1_900)), CardPhase::Open);
        c.hold_start(ClockMs(2_000)).unwrap();
        c.tick(ClockMs(3_200));
        assert_eq!(
            c.phase(ClockMs(3_200)),
            CardPhase::Submitting(Decision::AllowOnce)
        );
    }

    #[test]
    fn countdown_pauses_and_expires() {
        let mut c = card(Risk::Medium);
        assert_eq!(
            c.countdown(ClockMs(18_000)),
            Some(("Expires in 0:42".into(), false))
        );
        c.set_hover(true, ClockMs(20_000));
        assert_eq!(c.countdown(ClockMs(50_000)).unwrap().0, "Expires in 0:40");
        c.set_hover(false, ClockMs(50_000));
        assert_eq!(
            c.countdown(ClockMs(81_000)),
            Some(("Expires in 0:09".into(), true))
        );
        c.tick(ClockMs(90_000));
        assert!(matches!(
            c.phase(ClockMs(90_000)),
            CardPhase::Resolved {
                decision: Decision::Expired,
                ..
            }
        ));
    }

    #[test]
    fn patterns_and_queue() {
        assert_eq!(
            pattern_ladder("cargo test -p core"),
            vec!["cargo test -p core", "cargo test *", "cargo *"]
        );
        assert!(pattern_matches("cargo test *", "cargo test -p x"));
        assert!(!pattern_matches("cargo test *", "cargo testing"));
        let mut q = ApprovalQueue::default();
        for (i, r) in [Risk::Low, Risk::Medium, Risk::Low].into_iter().enumerate() {
            q.requests
                .push(ApprovalRequest::new(&i.to_string(), "x", r, ClockMs(0)));
        }
        assert_eq!(q.pager(0).as_deref(), Some("1 of 3"));
        assert_eq!(q.batch().as_deref(), Some("Allow all 3 once"));
        q.answer("1", Decision::AllowOnce);
        assert_eq!(q.pager(5).as_deref(), Some("2 of 2"));
        q.answer_all(Decision::AllowOnce);
        assert!(q.open().is_empty());
    }
}
