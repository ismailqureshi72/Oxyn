//! Stepper form (`stepper-form.md`): a multi-step flow for onboarding and setup. Steps fill and
//! check off, slide in from the side you're heading to, validate inline (the first invalid field
//! shakes and takes focus), keep values across Back and Edit, and end in a review and a success
//! state.
//!
//! One model drives both sources: the affectUI Stepper Form (numbered steps, chip fields, plan
//! cards, review with Edit links, success) and the Arc UI Multi-step form (minimal header with a
//! rolling step count, Next / Submit labels). Rebuilt from the public docs, no source used. Oxyn
//! plus: vertical layout, save-draft-and-resume, and steps that are skipped by a condition.

use std::collections::BTreeMap;

use super::tags::is_email;
use crate::ClockMs;

/// What a field holds and how it is drawn.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldKind {
    /// A text field with optional fixed text in front (`acme.app/`).
    Text {
        /// Fixed prefix.
        prefix: Option<String>,
    },
    /// An email address.
    Email,
    /// Chips (emails by default).
    Tags {
        /// Max chips.
        max: Option<usize>,
    },
    /// Selectable cards (a radio group with a gliding ring): `(value, title, aside, description)`.
    Choices(Vec<(String, String, String, String)>),
}

/// A field.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StepField {
    /// Name (the key in the values and errors).
    pub name: String,
    /// Label.
    pub label: String,
    /// Hint.
    pub hint: Option<String>,
    /// Optional (no required check).
    pub optional: bool,
    /// Kind.
    pub kind: FieldKind,
    /// Minimum length for text (validation).
    pub min_len: usize,
}

impl StepField {
    /// A required text field.
    pub fn text(name: &str, label: &str) -> Self {
        Self {
            name: name.into(),
            label: label.into(),
            hint: None,
            optional: false,
            kind: FieldKind::Text { prefix: None },
            min_len: 1,
        }
    }
    /// With a prefix.
    pub fn prefix(mut self, p: &str) -> Self {
        self.kind = FieldKind::Text {
            prefix: Some(p.into()),
        };
        self
    }
    /// With a hint.
    pub fn hint(mut self, h: &str) -> Self {
        self.hint = Some(h.into());
        self
    }
    /// Optional.
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }
    /// With a kind.
    pub fn kind(mut self, k: FieldKind) -> Self {
        self.kind = k;
        self
    }
}

/// A field's value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldValue {
    /// Text.
    Text(String),
    /// Chips.
    List(Vec<String>),
}

impl FieldValue {
    /// As shown in the review.
    pub fn display(&self) -> String {
        match self {
            Self::Text(t) => t.clone(),
            Self::List(v) if v.is_empty() => "—".into(),
            Self::List(v) => v.join(", "),
        }
    }
    fn is_empty(&self) -> bool {
        match self {
            Self::Text(t) => t.trim().is_empty(),
            Self::List(v) => v.is_empty(),
        }
    }
}

/// One step.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Step {
    /// Id.
    pub id: String,
    /// Title in the step list.
    pub title: String,
    /// Heading in the card.
    pub heading: String,
    /// One line under the heading.
    pub description: Option<String>,
    /// Fields (empty for the review step).
    pub fields: Vec<StepField>,
    /// This step is the review (summary rows with Edit links).
    pub review: bool,
    /// Oxyn plus: skip this step unless the field holds this value: `(field, value)`.
    pub only_if: Option<(String, String)>,
}

impl Step {
    /// A step.
    pub fn new(id: &str, title: &str, heading: &str, fields: Vec<StepField>) -> Self {
        Self {
            id: id.into(),
            title: title.into(),
            heading: heading.into(),
            description: None,
            fields,
            review: false,
            only_if: None,
        }
    }
    /// The review step.
    pub fn review(title: &str) -> Self {
        Self {
            review: true,
            ..Self::new("review", title, "Check everything", vec![])
        }
    }
    /// With a description.
    pub fn describe(mut self, d: &str) -> Self {
        self.description = Some(d.into());
        self
    }
}

/// Phase.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum StepperPhase {
    /// Filling steps.
    #[default]
    Editing,
    /// `on_complete` is running ("Creating…").
    Submitting,
    /// Done: the steps are replaced by the success view.
    Success,
    /// The submit failed; the message shows next to the button.
    Failed(String),
}

/// Which source the look follows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum StepperStyle {
    /// affectUI: numbered step list, Back / Continue / Finish.
    #[default]
    Stepper,
    /// Arc: minimal header with a rolling `Step 2 of 3`, Next / Submit.
    Minimal,
    /// Oxyn plus: a vertical step rail beside the card.
    Vertical,
}

impl StepperStyle {
    /// From an id.
    pub fn from_id(s: &str) -> Self {
        match s {
            "minimal" => Self::Minimal,
            "vertical" => Self::Vertical,
            _ => Self::Stepper,
        }
    }
}

/// The stepper.
#[derive(Clone, Debug, PartialEq)]
pub struct Stepper {
    /// Steps.
    pub steps: Vec<Step>,
    /// Shared values.
    pub values: BTreeMap<String, FieldValue>,
    /// Current step.
    pub current: usize,
    /// Finished steps.
    pub done: Vec<bool>,
    /// Messages by field.
    pub errors: BTreeMap<String, String>,
    /// Direction of the last move (`true` = forward) and when (slide).
    pub moved: Option<(bool, ClockMs)>,
    /// When the last failed Continue shook the first invalid field.
    pub shake_at: Option<(String, ClockMs)>,
    /// Phase.
    pub phase: StepperPhase,
    /// Look.
    pub style: StepperStyle,
    /// Last step's button.
    pub submit_label: String,
    /// While submitting.
    pub submitting_label: String,
    /// Oxyn plus: a saved draft (step and values).
    pub draft: Option<(usize, BTreeMap<String, FieldValue>)>,
}

impl Stepper {
    /// A stepper over `steps`.
    pub fn new(steps: Vec<Step>) -> Self {
        let n = steps.len();
        Self {
            steps,
            values: BTreeMap::new(),
            current: 0,
            done: vec![false; n],
            errors: BTreeMap::new(),
            moved: None,
            shake_at: None,
            phase: StepperPhase::Editing,
            style: StepperStyle::Stepper,
            submit_label: "Finish".into(),
            submitting_label: "Creating…".into(),
            draft: None,
        }
    }
    /// Sets a value.
    pub fn set(&mut self, name: &str, v: FieldValue) {
        self.errors.remove(name);
        self.values.insert(name.into(), v);
    }
    /// Text value.
    pub fn text(&self, name: &str) -> String {
        match self.values.get(name) {
            Some(FieldValue::Text(t)) => t.clone(),
            Some(v) => v.display(),
            None => String::new(),
        }
    }
    /// List value.
    pub fn list(&self, name: &str) -> Vec<String> {
        match self.values.get(name) {
            Some(FieldValue::List(v)) => v.clone(),
            _ => vec![],
        }
    }
    /// Whether step `i` applies (Oxyn plus conditions).
    pub fn applies(&self, i: usize) -> bool {
        self.steps
            .get(i)
            .and_then(|s| s.only_if.as_ref())
            .is_none_or(|(f, v)| self.text(f) == *v)
    }
    /// Steps that apply, as indices.
    pub fn visible(&self) -> Vec<usize> {
        (0..self.steps.len()).filter(|&i| self.applies(i)).collect()
    }
    /// `(position, total)` among visible steps, 1-based.
    pub fn progress(&self) -> (usize, usize) {
        let v = self.visible();
        let p = v.iter().position(|&i| i == self.current).unwrap_or(0) + 1;
        (p, v.len())
    }
    /// `Step 2 of 4: Team`.
    pub fn progress_label(&self) -> String {
        let (p, n) = self.progress();
        format!("Step {p} of {n}: {}", self.steps[self.current].title)
    }
    /// On the last visible step.
    pub fn last(&self) -> bool {
        self.visible().last() == Some(&self.current)
    }
    /// Validates the current step; returns messages by field.
    pub fn validate(&self) -> BTreeMap<String, String> {
        let mut out = BTreeMap::new();
        for f in &self.steps[self.current].fields {
            let v = self.values.get(&f.name);
            let empty = v.is_none_or(FieldValue::is_empty);
            if empty {
                if !f.optional {
                    out.insert(f.name.clone(), format!("{} is required", f.label));
                }
                continue;
            }
            match (&f.kind, v) {
                (FieldKind::Email, Some(FieldValue::Text(t))) if !is_email(t.trim()) => {
                    out.insert(f.name.clone(), "Enter an email address".into());
                }
                (FieldKind::Text { .. }, Some(FieldValue::Text(t)))
                    if t.trim().chars().count() < f.min_len =>
                {
                    out.insert(f.name.clone(), format!("At least {} characters", f.min_len));
                }
                (FieldKind::Tags { max: Some(m) }, Some(FieldValue::List(l))) if l.len() > *m => {
                    out.insert(f.name.clone(), format!("Up to {m}"));
                }
                _ => {}
            }
        }
        out
    }
    /// The step is valid (Continue looks dimmed otherwise but still answers).
    pub fn valid(&self) -> bool {
        self.validate().is_empty()
    }
    fn go(&mut self, to: usize, now: ClockMs) {
        if to != self.current {
            self.moved = Some((to > self.current, now));
            self.current = to;
        }
    }
    /// Continue: validates; on failure the first invalid field shakes (returns its name for
    /// focus); on the last step it submits.
    pub fn next(&mut self, now: ClockMs) -> Result<bool, String> {
        if self.phase == StepperPhase::Submitting {
            return Ok(false);
        }
        let errs = self.validate();
        if let Some(first) = self.steps[self.current]
            .fields
            .iter()
            .find(|f| errs.contains_key(&f.name))
        {
            let name = first.name.clone();
            self.errors = errs;
            self.shake_at = Some((name.clone(), now));
            return Err(name);
        }
        self.errors.clear();
        self.done[self.current] = true;
        if self.last() {
            self.phase = StepperPhase::Submitting;
            return Ok(true);
        }
        let v = self.visible();
        if let Some(&n) = v.iter().find(|&&i| i > self.current) {
            self.go(n, now);
        }
        Ok(false)
    }
    /// Back.
    pub fn back(&mut self, now: ClockMs) {
        if self.phase == StepperPhase::Submitting {
            return;
        }
        if let Some(&p) = self.visible().iter().rev().find(|&&i| i < self.current) {
            self.errors.clear();
            self.go(p, now);
        }
    }
    /// Jump to a finished step (step list buttons, review Edit links).
    pub fn go_to(&mut self, i: usize, now: ClockMs) -> bool {
        if i < self.steps.len()
            && self.applies(i)
            && (self.done[i] || i < self.current)
            && self.phase != StepperPhase::Submitting
        {
            self.errors.clear();
            self.go(i, now);
            return true;
        }
        false
    }
    /// The host finished `on_complete`.
    pub fn finish(&mut self, result: Result<(), String>) {
        self.phase = match result {
            Ok(()) => StepperPhase::Success,
            Err(m) => StepperPhase::Failed(m),
        };
    }
    /// Start over (the success view's reset).
    pub fn reset(&mut self, now: ClockMs) {
        self.values.clear();
        self.errors.clear();
        self.done = vec![false; self.steps.len()];
        self.phase = StepperPhase::Editing;
        self.go(0, now);
    }
    /// The primary button's label (the button glides to its new width).
    pub fn primary_label(&self) -> &str {
        match (&self.phase, self.last(), self.style) {
            (StepperPhase::Submitting, _, _) => &self.submitting_label,
            (_, true, _) => &self.submit_label,
            (_, false, StepperStyle::Minimal) => "Next",
            _ => "Continue",
        }
    }
    /// Review rows: `(label, value, step)` for every field on earlier steps.
    pub fn review_rows(&self) -> Vec<(String, String, usize)> {
        let mut out = vec![];
        for i in self.visible() {
            let s = &self.steps[i];
            if s.review {
                continue;
            }
            for f in &s.fields {
                let v = self
                    .values
                    .get(&f.name)
                    .map(FieldValue::display)
                    .unwrap_or_else(|| "—".into());
                let v = match (&f.kind, &v) {
                    (FieldKind::Text { prefix: Some(p) }, v) if v != "—" => format!("{p}{v}"),
                    (FieldKind::Choices(c), v) => c
                        .iter()
                        .find(|x| x.0 == *v)
                        .map(|x| x.1.clone())
                        .unwrap_or(v.clone()),
                    _ => v,
                };
                out.push((f.label.clone(), v, i));
            }
        }
        out
    }
    /// Save draft (Oxyn plus).
    pub fn save_draft(&mut self) {
        self.draft = Some((self.current, self.values.clone()));
    }
    /// Resume a saved draft.
    pub fn resume(&mut self, now: ClockMs) -> bool {
        let Some((step, vals)) = self.draft.clone() else {
            return false;
        };
        self.values = vals;
        for i in 0..step {
            self.done[i] = true;
        }
        self.go(step, now);
        true
    }
    /// The live-region line.
    pub fn announce(&self) -> String {
        match &self.phase {
            StepperPhase::Submitting => self.submitting_label.clone(),
            StepperPhase::Success => "All set".into(),
            StepperPhase::Failed(m) => m.clone(),
            StepperPhase::Editing if !self.errors.is_empty() => format!(
                "{} field{} need{} attention",
                self.errors.len(),
                if self.errors.len() == 1 { "" } else { "s" },
                if self.errors.len() == 1 { "s" } else { "" }
            ),
            StepperPhase::Editing => self.progress_label(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn flow() -> Stepper {
        Stepper::new(vec![
            Step::new(
                "ws",
                "Workspace",
                "Name your workspace",
                vec![
                    StepField::text("name", "Workspace name"),
                    StepField::text("url", "URL").prefix("acme.app/"),
                ],
            ),
            Step::new(
                "team",
                "Team",
                "Invite your team",
                vec![
                    StepField::text("invites", "Invites")
                        .kind(FieldKind::Tags { max: Some(3) })
                        .optional(),
                ],
            ),
            Step::new(
                "plan",
                "Plan",
                "Pick a plan",
                vec![
                    StepField::text("plan", "Plan").kind(FieldKind::Choices(vec![
                        (
                            "free".into(),
                            "Free".into(),
                            "$0".into(),
                            "For trying it".into(),
                        ),
                        (
                            "team".into(),
                            "Team".into(),
                            "$12".into(),
                            "Per seat".into(),
                        ),
                    ])),
                ],
            ),
            Step::review("Review"),
        ])
    }

    #[test]
    fn validates_moves_and_submits() {
        let mut s = flow();
        assert_eq!(s.next(ClockMs(0)), Err("name".into()));
        assert_eq!(s.errors.len(), 2);
        assert_eq!(s.announce(), "2 fields need attention");
        s.set("name", FieldValue::Text("Acme".into()));
        s.set("url", FieldValue::Text("acme".into()));
        assert_eq!(s.next(ClockMs(10)), Ok(false));
        assert_eq!((s.current, s.moved.map(|m| m.0)), (1, Some(true)));
        assert_eq!(s.progress_label(), "Step 2 of 4: Team");
        s.next(ClockMs(20)).unwrap();
        s.set("plan", FieldValue::Text("team".into()));
        s.next(ClockMs(30)).unwrap();
        assert!(s.last());
        assert_eq!(s.primary_label(), "Finish");
        let rows = s.review_rows();
        assert_eq!(rows[1], ("URL".into(), "acme.app/acme".into(), 0));
        assert_eq!(rows[3].1, "Team");
        assert!(s.go_to(0, ClockMs(40)));
        assert_eq!(s.moved.map(|m| m.0), Some(false));
        assert!(!s.go_to(3, ClockMs(41)) || s.done[3]);
        s.go_to(2, ClockMs(42));
        s.next(ClockMs(43)).unwrap();
        assert_eq!(s.next(ClockMs(44)), Ok(true));
        assert_eq!(s.primary_label(), "Creating…");
        s.back(ClockMs(45));
        assert_eq!(s.current, 3, "locked while submitting");
        s.finish(Err("Name taken".into()));
        assert_eq!(s.announce(), "Name taken");
        s.phase = StepperPhase::Editing;
        s.next(ClockMs(46)).unwrap();
        s.finish(Ok(()));
        assert_eq!(s.phase, StepperPhase::Success);
        s.reset(ClockMs(50));
        assert_eq!(s.current, 0);
    }

    #[test]
    fn minimal_labels_conditions_drafts() {
        let mut s = flow();
        s.style = StepperStyle::Minimal;
        assert_eq!(s.primary_label(), "Next");
        s.steps[1].only_if = Some(("plan".into(), "team".into()));
        assert_eq!(s.progress(), (1, 3));
        s.set("name", FieldValue::Text("A".into()));
        s.set("url", FieldValue::Text("a".into()));
        s.next(ClockMs(0)).unwrap();
        assert_eq!(s.current, 2, "team step skipped");
        s.save_draft();
        let mut t = flow();
        t.draft = s.draft.clone();
        assert!(t.resume(ClockMs(1)));
        assert_eq!(t.current, 2);
        assert_eq!(t.text("name"), "A");
        t.set("invites", FieldValue::List(vec!["a@b.co".into(); 4]));
        t.current = 1;
        assert_eq!(
            t.validate().get("invites").map(String::as_str),
            Some("Up to 3")
        );
    }
}
