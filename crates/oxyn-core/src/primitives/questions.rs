//! Question decks: the agent asks, the user answers. One model behind two cards:
//!
//! - **Decision card** (`decision-card.md`, affectUI Decision Card): the open decisions sit in their
//!   own card just above the composer — numbered rows with one Recommended, tiles for a pick,
//!   Approve / Decline with facts for an approval, your own words in a field; `1`–`5` answer at
//!   once, Esc folds the card to "2 decisions waiting · Show". An answered decision leaves at once.
//! - **Approval questions** (`approval-card.md` §questions, Beautiful UI Approval Card, MIT,
//!   © 2026 Shane Levine): paged questions `1 / 3` with an odometer counter, single choice that
//!   auto-advances after 480 ms, multi-select that waits, a custom answer row, Skip, Continue,
//!   Send, "Answers sent", dismiss and Start over.
//!
//! Oxyn plus: a review page before sending, a timeout that picks the recommended answer, and
//! "always answer this way" for a question.

use crate::ClockMs;

/// Question kind.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum QuestionKind {
    /// Pick one row (affectUI `decision`, Beautiful UI `radio`).
    #[default]
    Single,
    /// Pick any number (Beautiful UI `check`).
    Multi,
    /// Pick one tile (affectUI `pick`).
    Pick,
    /// Approve or decline, with facts (affectUI `approval`).
    Approval,
}

impl QuestionKind {
    /// Id.
    pub fn id(self) -> &'static str {
        match self {
            Self::Single => "single",
            Self::Multi => "multi",
            Self::Pick => "pick",
            Self::Approval => "approval",
        }
    }
}

/// One option.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct QuestionOption {
    /// Id.
    pub id: String,
    /// Title.
    pub title: String,
    /// One line under it.
    pub summary: Option<String>,
    /// The agent's pick.
    pub recommended: bool,
}

impl QuestionOption {
    /// An option.
    pub fn new(id: &str, title: &str) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            summary: None,
            recommended: false,
        }
    }
    /// With a summary.
    pub fn summary(mut self, s: &str) -> Self {
        self.summary = Some(s.into());
        self
    }
    /// Marked Recommended.
    pub fn recommended(mut self) -> Self {
        self.recommended = true;
        self
    }
}

/// One question / decision.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Question {
    /// Id.
    pub id: String,
    /// The question.
    pub prompt: String,
    /// The task it belongs to (affectUI `task`).
    pub task: Option<String>,
    /// Kind.
    pub kind: QuestionKind,
    /// Options (three to five for a decision).
    pub options: Vec<QuestionOption>,
    /// Facts for an approval: `(label, value)`.
    pub facts: Vec<(String, String)>,
    /// Offer a free-text answer.
    pub allow_text: bool,
}

impl Question {
    /// A question.
    pub fn new(id: &str, prompt: &str, kind: QuestionKind, options: Vec<QuestionOption>) -> Self {
        Self {
            id: id.into(),
            prompt: prompt.into(),
            task: None,
            kind,
            options,
            facts: vec![],
            allow_text: true,
        }
    }
    /// With a task line.
    pub fn task(mut self, t: &str) -> Self {
        self.task = Some(t.into());
        self
    }
    /// With facts.
    pub fn facts(mut self, f: &[(&str, &str)]) -> Self {
        self.facts = f.iter().map(|(a, b)| ((*a).into(), (*b).into())).collect();
        self
    }
    /// The recommended option index.
    pub fn recommended(&self) -> Option<usize> {
        self.options.iter().position(|o| o.recommended)
    }
    /// Number of focusable rows: options (or Approve / Decline) then the text field.
    pub fn rows(&self) -> usize {
        let base = if self.kind == QuestionKind::Approval {
            2
        } else {
            self.options.len()
        };
        base + usize::from(self.allow_text)
    }
}

/// An answer.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Answer {
    /// Not answered.
    #[default]
    None,
    /// Option indices.
    Options(Vec<usize>),
    /// Own words.
    Text(String),
    /// Approved.
    Approve,
    /// Declined.
    Decline,
    /// Skipped.
    Skipped,
}

impl Answer {
    /// Answered (anything but `None`).
    pub fn answered(&self) -> bool {
        !matches!(self, Self::None)
    }
}

/// Which card the deck drives.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DeckMode {
    /// affectUI Decision Card: answered decisions leave; Later folds.
    #[default]
    Decisions,
    /// Beautiful UI Approval Card: pages, Continue, Send.
    Paged,
}

/// Phase.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum DeckPhase {
    /// Asking.
    #[default]
    Asking,
    /// Oxyn plus: the review page before sending.
    Review,
    /// Sent ("Answers sent") / all decided (the card folds away).
    Done,
    /// Folded to one line (Later) / dismissed.
    Folded,
}

/// Keys.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DeckKey {
    /// A number key 1–9.
    Digit(u8),
    /// ↑ (← on tiles).
    Up,
    /// ↓ (→ on tiles).
    Down,
    /// Home.
    Home,
    /// End.
    End,
    /// Enter / Space on the focused row.
    Activate,
    /// Esc.
    Escape,
    /// ← pager.
    Prev,
    /// → pager.
    Next,
}

/// What happened, for the host's callbacks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DeckEvent {
    /// `on_decide(id, answer)`.
    Decided(String, Answer),
    /// `on_answer_change(index, answer)` (paged).
    Changed(usize, Answer),
    /// `on_later(id)`.
    Later(String),
    /// `on_submitted(answers)` (paged).
    Submitted(Vec<Answer>),
    /// Nothing.
    None,
}

/// Auto-advance delay after a single choice (Beautiful UI).
pub const AUTO_ADVANCE_MS: u64 = 480;
/// Counter roll (Beautiful UI).
pub const ROLL_MS: u64 = 400;

/// The deck.
#[derive(Clone, Debug, PartialEq)]
pub struct QuestionDeck {
    /// Open questions (decisions mode: oldest first; answered ones are removed).
    pub questions: Vec<Question>,
    /// Answers (paged mode), per question.
    pub answers: Vec<Answer>,
    /// Draft own words per question.
    pub text: Vec<String>,
    /// Current question.
    pub index: usize,
    /// Focused row in the current question.
    pub focus: usize,
    /// Mode.
    pub mode: DeckMode,
    /// Phase.
    pub phase: DeckPhase,
    /// Pending auto-advance (paged single choice).
    pub advance_at: Option<ClockMs>,
    /// Counter roll: previous position and when.
    pub roll: Option<(usize, ClockMs)>,
    /// When the card was brought up with `reveal` (pulses grey once).
    pub revealed_at: Option<ClockMs>,
    /// Oxyn plus: show a review page before Send.
    pub review: bool,
    /// Oxyn plus: answer with the recommended option after this long (ms) unanswered.
    pub timeout_ms: Option<u64>,
    /// When the current question was shown (timeout).
    pub shown_at: ClockMs,
    /// Oxyn plus: question ids the user chose to always answer the same way, with the answer.
    pub rules: Vec<(String, Answer)>,
    /// Live-region line.
    pub announce: String,
}

impl QuestionDeck {
    /// A deck.
    pub fn new(questions: Vec<Question>, mode: DeckMode) -> Self {
        let n = questions.len();
        let focus = questions
            .first()
            .and_then(Question::recommended)
            .unwrap_or(0);
        Self {
            questions,
            answers: vec![Answer::None; n],
            text: vec![String::new(); n],
            index: 0,
            focus,
            mode,
            phase: DeckPhase::Asking,
            advance_at: None,
            roll: None,
            revealed_at: None,
            review: false,
            timeout_ms: None,
            shown_at: ClockMs(0),
            rules: vec![],
            announce: String::new(),
        }
    }
    /// The current question.
    pub fn current(&self) -> Option<&Question> {
        self.questions.get(self.index)
    }
    /// `1 / 3` (paged) or `1 of 2` positions.
    pub fn position(&self) -> (usize, usize) {
        (self.index + 1, self.questions.len())
    }
    /// The current answer.
    pub fn answer(&self) -> &Answer {
        self.answers.get(self.index).unwrap_or(&Answer::None)
    }
    /// Continue is enabled (an answer or own words).
    pub fn has_answer(&self) -> bool {
        self.answer().answered()
            || self
                .text
                .get(self.index)
                .is_some_and(|t| !t.trim().is_empty())
    }
    /// Last page.
    pub fn last(&self) -> bool {
        self.index + 1 >= self.questions.len()
    }
    fn reset_focus(&mut self, now: ClockMs) {
        self.focus = self
            .current()
            .and_then(|q| {
                // Paged: focus the first picked option; else the recommended one.
                match self.answers.get(self.index) {
                    Some(Answer::Options(v)) if !v.is_empty() => Some(v[0]),
                    _ => q.recommended(),
                }
            })
            .unwrap_or(0);
        self.shown_at = now;
    }
    fn goto(&mut self, i: usize, now: ClockMs) {
        if self.questions.is_empty() {
            return;
        }
        let i = i.min(self.questions.len() - 1);
        if i != self.index {
            self.roll = Some((self.index + 1, now));
            self.index = i;
            self.advance_at = None;
            self.reset_focus(now);
        }
    }
    /// The text field's row index.
    fn text_row(&self) -> Option<usize> {
        let q = self.current()?;
        q.allow_text.then(|| q.rows() - 1)
    }
    /// Focus is in the own-words field.
    pub fn in_text(&self) -> bool {
        self.text_row() == Some(self.focus)
    }
    /// Choose option `i` (row click, number key).
    pub fn choose(&mut self, i: usize, now: ClockMs) -> DeckEvent {
        let Some(q) = self.current().cloned() else {
            return DeckEvent::None;
        };
        if self.phase != DeckPhase::Asking {
            return DeckEvent::None;
        }
        match self.mode {
            DeckMode::Decisions => {
                let ans = if q.kind == QuestionKind::Approval {
                    match i {
                        0 => Answer::Approve,
                        1 => Answer::Decline,
                        _ => return DeckEvent::None,
                    }
                } else if i < q.options.len() {
                    Answer::Options(vec![i])
                } else {
                    return DeckEvent::None;
                };
                self.decide(ans, now)
            }
            DeckMode::Paged => {
                if i >= q.options.len() {
                    return DeckEvent::None;
                }
                self.focus = i;
                let next = match (&self.answers[self.index], q.kind) {
                    (Answer::Options(v), QuestionKind::Multi) => {
                        let mut v = v.clone();
                        if let Some(p) = v.iter().position(|&x| x == i) {
                            v.remove(p);
                        } else {
                            v.push(i);
                        }
                        Answer::Options(v)
                    }
                    _ => Answer::Options(vec![i]),
                };
                let next = match next {
                    Answer::Options(v) if v.is_empty() => Answer::None,
                    a => a,
                };
                self.answers[self.index] = next.clone();
                if q.kind != QuestionKind::Multi {
                    self.text[self.index].clear();
                    self.advance_at = Some(ClockMs(now.0 + AUTO_ADVANCE_MS));
                }
                DeckEvent::Changed(self.index, next)
            }
        }
    }
    /// Approve / Decline for an approval decision.
    pub fn approve(&mut self, yes: bool, now: ClockMs) -> DeckEvent {
        self.choose(if yes { 0 } else { 1 }, now)
    }
    /// Own words typed.
    pub fn type_text(&mut self, s: &str) {
        if let Some(t) = self.text.get_mut(self.index) {
            *t = s.into();
        }
        if self.mode == DeckMode::Paged && !s.trim().is_empty() {
            // A custom answer replaces a single choice.
            if let Some(QuestionKind::Single) = self.current().map(|q| q.kind) {
                self.answers[self.index] = Answer::None;
            }
            self.advance_at = None;
        }
    }
    /// Send own words (Enter in the field).
    pub fn send_text(&mut self, now: ClockMs) -> DeckEvent {
        let t = self.text.get(self.index).cloned().unwrap_or_default();
        if t.trim().is_empty() {
            return DeckEvent::None;
        }
        match self.mode {
            DeckMode::Decisions => self.decide(Answer::Text(t.trim().into()), now),
            DeckMode::Paged => {
                if !matches!(self.answers[self.index], Answer::Options(_))
                    || self
                        .current()
                        .is_some_and(|q| q.kind == QuestionKind::Single)
                {
                    self.answers[self.index] = Answer::Text(t.trim().into());
                }
                self.advance(now)
            }
        }
    }
    fn decide(&mut self, ans: Answer, now: ClockMs) -> DeckEvent {
        let q = self.questions.remove(self.index);
        self.answers.remove(self.index);
        self.text.remove(self.index);
        let picked = match &ans {
            Answer::Options(v) => v
                .first()
                .and_then(|&i| q.options.get(i))
                .map(|o| o.title.clone())
                .unwrap_or_default(),
            Answer::Text(t) => format!("“{t}”"),
            Answer::Approve => "Approved".into(),
            Answer::Decline => "Declined".into(),
            _ => String::new(),
        };
        let left = self.questions.len();
        self.announce = if left == 0 {
            format!("{picked}. No decisions waiting.")
        } else {
            format!(
                "{picked}. {left} decision{} waiting.",
                if left == 1 { "" } else { "s" }
            )
        };
        if self.questions.is_empty() {
            self.phase = DeckPhase::Done;
            self.index = 0;
        } else {
            self.index = self.index.min(self.questions.len() - 1);
            self.reset_focus(now);
        }
        DeckEvent::Decided(q.id, ans)
    }
    /// Skip (paged): marks the question skipped and moves on.
    pub fn skip(&mut self, now: ClockMs) -> DeckEvent {
        if self.mode != DeckMode::Paged || self.phase != DeckPhase::Asking {
            return DeckEvent::None;
        }
        self.answers[self.index] = Answer::Skipped;
        self.advance(now)
    }
    /// Continue (paged): next page, the review page, or Send on the last one.
    pub fn advance(&mut self, now: ClockMs) -> DeckEvent {
        self.advance_at = None;
        if self.last() {
            if self.review && self.phase == DeckPhase::Asking {
                self.phase = DeckPhase::Review;
                return DeckEvent::None;
            }
            return self.send();
        }
        self.goto(self.index + 1, now);
        DeckEvent::None
    }
    /// Send (paged).
    pub fn send(&mut self) -> DeckEvent {
        self.phase = DeckPhase::Done;
        self.advance_at = None;
        let answered = self.answers.iter().filter(|a| a.answered()).count();
        self.announce = format!("Answers sent ({answered} of {})", self.questions.len());
        DeckEvent::Submitted(self.answers.clone())
    }
    /// Edit an answer from the review page.
    pub fn edit(&mut self, i: usize, now: ClockMs) {
        self.phase = DeckPhase::Asking;
        self.roll = Some((self.index + 1, now));
        self.index = i.min(self.questions.len().saturating_sub(1));
        self.reset_focus(now);
    }
    /// Later / dismiss: folds the card.
    pub fn fold(&mut self) -> DeckEvent {
        if self.phase == DeckPhase::Done {
            return DeckEvent::None;
        }
        self.phase = DeckPhase::Folded;
        self.advance_at = None;
        let n = self.questions.len();
        self.announce = format!("{n} decision{} waiting", if n == 1 { "" } else { "s" });
        match self.current() {
            Some(q) if self.mode == DeckMode::Decisions => DeckEvent::Later(q.id.clone()),
            _ => DeckEvent::None,
        }
    }
    /// Show (unfold).
    pub fn unfold(&mut self, now: ClockMs) {
        if self.phase == DeckPhase::Folded {
            self.phase = DeckPhase::Asking;
            self.reset_focus(now);
        }
    }
    /// `reveal`: unfolds, pulses once and focuses the selected row.
    pub fn reveal(&mut self, now: ClockMs) {
        self.unfold(now);
        self.revealed_at = Some(now);
    }
    /// Start over (paged).
    pub fn reset(&mut self, now: ClockMs) {
        let n = self.questions.len();
        self.answers = vec![Answer::None; n];
        self.text = vec![String::new(); n];
        self.index = 0;
        self.phase = DeckPhase::Asking;
        self.roll = None;
        self.reset_focus(now);
    }
    /// "Always answer this way" (Oxyn plus): remembers the current answer for this question id.
    pub fn always(&mut self) {
        let Some(q) = self.current() else { return };
        let id = q.id.clone();
        let a = self.answer().clone();
        if a.answered() {
            self.rules.retain(|r| r.0 != id);
            self.rules.push((id, a));
        }
    }
    /// Advances time: auto-advance and the timeout default.
    pub fn tick(&mut self, now: ClockMs) -> DeckEvent {
        if let Some(t) = self.advance_at
            && now >= t
        {
            return self.advance(now);
        }
        if let (Some(ms), DeckPhase::Asking) = (self.timeout_ms, self.phase)
            && now.since(self.shown_at).as_millis() >= u128::from(ms)
            && !self.has_answer()
            && let Some(r) = self.current().and_then(Question::recommended)
        {
            return self.choose(r, now);
        }
        DeckEvent::None
    }
    /// Seconds left before the timeout answers (Oxyn plus).
    pub fn timeout_left(&self, now: ClockMs) -> Option<u64> {
        let ms = self.timeout_ms?;
        Some(ms.saturating_sub(now.since(self.shown_at).as_millis() as u64) / 1000)
    }
    /// Keyboard.
    pub fn key(&mut self, k: DeckKey, now: ClockMs) -> DeckEvent {
        let Some(q) = self.current().cloned() else {
            return DeckEvent::None;
        };
        if self.phase == DeckPhase::Folded {
            return DeckEvent::None;
        }
        let rows = q.rows();
        let text_row = self.text_row();
        match k {
            DeckKey::Digit(n) if !self.in_text() => {
                let i = usize::from(n.saturating_sub(1));
                let max = if q.kind == QuestionKind::Approval {
                    2
                } else {
                    q.options.len()
                };
                if i < max {
                    return self.choose(i, now);
                }
                // The next number after the last row goes into the field.
                if let Some(t) = text_row {
                    self.focus = t;
                }
                DeckEvent::None
            }
            DeckKey::Digit(_) => DeckEvent::None,
            DeckKey::Up => {
                let in_text_with_words = self.in_text() && self.has_answer();
                if !in_text_with_words {
                    self.focus = self.focus.saturating_sub(1);
                }
                DeckEvent::None
            }
            DeckKey::Down => {
                self.focus = (self.focus + 1).min(rows.saturating_sub(1));
                DeckEvent::None
            }
            DeckKey::Home => {
                self.focus = 0;
                DeckEvent::None
            }
            DeckKey::End => {
                self.focus = text_row.map_or(rows.saturating_sub(1), |t| t.saturating_sub(1));
                DeckEvent::None
            }
            DeckKey::Activate => {
                if self.in_text() {
                    self.send_text(now)
                } else {
                    self.choose(self.focus, now)
                }
            }
            DeckKey::Escape => self.fold(),
            DeckKey::Prev => {
                if self.index > 0 {
                    self.goto(self.index - 1, now);
                }
                DeckEvent::None
            }
            DeckKey::Next => {
                if self.index + 1 < self.questions.len() {
                    self.goto(self.index + 1, now);
                }
                DeckEvent::None
            }
        }
    }
    /// Folded line: `2 decisions waiting`.
    pub fn waiting(&self) -> String {
        let n = self.questions.len();
        format!("{n} decision{} waiting", if n == 1 { "" } else { "s" })
    }
    /// Review rows (Oxyn plus): `(question, answer text)`.
    pub fn review_rows(&self) -> Vec<(String, String)> {
        self.questions
            .iter()
            .zip(&self.answers)
            .zip(&self.text)
            .map(|((q, a), t)| {
                let s = match a {
                    Answer::Options(v) => v
                        .iter()
                        .filter_map(|&i| q.options.get(i).map(|o| o.title.as_str()))
                        .collect::<Vec<_>>()
                        .join(", "),
                    Answer::Text(t) => t.clone(),
                    Answer::Approve => "Approve".into(),
                    Answer::Decline => "Decline".into(),
                    Answer::Skipped => "Skipped".into(),
                    Answer::None if !t.trim().is_empty() => t.clone(),
                    Answer::None => "—".into(),
                };
                (q.prompt.clone(), s)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn decisions() -> Vec<Question> {
        vec![
            Question::new(
                "db",
                "Which database for the cache?",
                QuestionKind::Single,
                vec![
                    QuestionOption::new("redis", "Redis"),
                    QuestionOption::new("pg", "Postgres").recommended(),
                    QuestionOption::new("none", "No cache"),
                ],
            )
            .task("Speed up the dashboard"),
            Question::new(
                "deploy",
                "Deploy to production?",
                QuestionKind::Approval,
                vec![],
            )
            .facts(&[("Branch", "main"), ("Changes", "+214 −38")]),
        ]
    }

    #[test]
    fn decisions_leave_when_answered() {
        let mut d = QuestionDeck::new(decisions(), DeckMode::Decisions);
        assert_eq!(d.focus, 1, "focus starts on Recommended");
        let ev = d.key(DeckKey::Digit(1), ClockMs(0));
        assert_eq!(
            ev,
            DeckEvent::Decided("db".into(), Answer::Options(vec![0]))
        );
        assert_eq!(d.announce, "Redis. 1 decision waiting.");
        assert_eq!(d.current().unwrap().kind, QuestionKind::Approval);
        assert_eq!(
            d.key(DeckKey::Escape, ClockMs(1)),
            DeckEvent::Later("deploy".into())
        );
        assert_eq!(d.waiting(), "1 decision waiting");
        d.reveal(ClockMs(2));
        assert_eq!(d.phase, DeckPhase::Asking);
        assert_eq!(d.key(DeckKey::Down, ClockMs(3)), DeckEvent::None);
        assert_eq!(
            d.key(DeckKey::Activate, ClockMs(4)),
            DeckEvent::Decided("deploy".into(), Answer::Decline)
        );
        assert_eq!(d.phase, DeckPhase::Done);
    }

    #[test]
    fn own_words_and_field_focus() {
        let mut d = QuestionDeck::new(decisions(), DeckMode::Decisions);
        d.key(DeckKey::Digit(5), ClockMs(0));
        assert!(d.in_text(), "a number past the rows goes into the field");
        d.type_text("Use SQLite");
        assert_eq!(
            d.key(DeckKey::Activate, ClockMs(1)),
            DeckEvent::Decided("db".into(), Answer::Text("Use SQLite".into()))
        );
    }

    fn paged() -> Vec<Question> {
        vec![
            Question::new(
                "n",
                "How many flavors should we launch?",
                QuestionKind::Single,
                vec![
                    QuestionOption::new("3", "Three (core line)"),
                    QuestionOption::new("5", "Five (full case)"),
                    QuestionOption::new("1", "Just one hero"),
                ],
            ),
            Question::new(
                "mix",
                "Which mix-ins should we stock?",
                QuestionKind::Multi,
                vec![
                    QuestionOption::new("c", "Chocolate chips"),
                    QuestionOption::new("w", "Waffle bits"),
                    QuestionOption::new("s", "Sprinkles"),
                ],
            ),
            Question::new(
                "mkt",
                "Which market do we enter first?",
                QuestionKind::Single,
                vec![
                    QuestionOption::new("t", "Food trucks").recommended(),
                    QuestionOption::new("g", "Grocery freezers"),
                ],
            ),
        ]
    }

    #[test]
    fn paged_auto_advances_and_sends() {
        let mut d = QuestionDeck::new(paged(), DeckMode::Paged);
        d.choose(1, ClockMs(0));
        assert_eq!(d.tick(ClockMs(400)), DeckEvent::None);
        d.tick(ClockMs(480));
        assert_eq!(d.index, 1);
        assert_eq!(d.roll, Some((1, ClockMs(480))));
        d.choose(0, ClockMs(500));
        d.choose(2, ClockMs(510));
        d.choose(0, ClockMs(520));
        assert_eq!(d.answer(), &Answer::Options(vec![2]));
        assert_eq!(d.tick(ClockMs(2_000)), DeckEvent::None, "multi waits");
        d.advance(ClockMs(2_100));
        d.key(DeckKey::Prev, ClockMs(2_200));
        assert_eq!(d.index, 1);
        d.key(DeckKey::Next, ClockMs(2_300));
        assert_eq!(d.focus, 0, "focus on Recommended");
        d.type_text("Farmers markets");
        let ev = d.send_text(ClockMs(2_400));
        assert!(
            matches!(ev, DeckEvent::Submitted(ref a) if a[2] == Answer::Text("Farmers markets".into()))
        );
        assert_eq!(d.announce, "Answers sent (3 of 3)");
        d.reset(ClockMs(3_000));
        assert_eq!((d.index, d.phase), (0, DeckPhase::Asking));
    }

    #[test]
    fn review_skip_timeout_rules() {
        let mut d = QuestionDeck::new(paged(), DeckMode::Paged);
        d.review = true;
        d.skip(ClockMs(0));
        d.skip(ClockMs(1));
        d.timeout_ms = Some(10_000);
        assert_eq!(d.timeout_left(ClockMs(4_000)), Some(6));
        d.tick(ClockMs(11_000));
        assert_eq!(
            d.answer(),
            &Answer::Options(vec![0]),
            "timeout picks Recommended"
        );
        d.always();
        assert_eq!(d.rules[0].0, "mkt");
        d.tick(ClockMs(11_480));
        assert_eq!(d.phase, DeckPhase::Review);
        assert_eq!(d.review_rows()[0].1, "Skipped");
        d.edit(0, ClockMs(12_000));
        assert_eq!((d.index, d.phase), (0, DeckPhase::Asking));
        d.fold();
        assert_eq!(d.phase, DeckPhase::Folded);
    }
}
