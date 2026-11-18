//! Screens S5: orchestration and builders. The Evals screen's model (test cases scored per prompt
//! version, pass rate by version, a reviewable prompt diff, model choice and re-runs) plus the
//! own states of the upgraded Orchestration and Workflow Builder screens. Every view rule is a
//! pure function so Dioxus and GPUI render the same thing. All copy, prompts and names are
//! fictional and our own.

pub use super::screensfid::ScreenState;

/// A screen's state bar: (key, label).
pub type StateList = &'static [(&'static str, &'static str)];

/// Evals states.
pub const EVALS_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("running", "Running"),
];
/// Orchestration states.
pub const ORCH_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("paused", "Paused"),
];
/// Workflow Builder states.
pub const FLOW_STATES: StateList = &[
    ("sample", "Data"),
    ("loading", "Loading"),
    ("empty", "Empty"),
    ("testing", "Test run"),
];

/// The states of an S5 screen by its slug.
pub fn states_for(slug: &str) -> StateList {
    match slug {
        "agent-orchestration" => ORCH_STATES,
        "workflow" => FLOW_STATES,
        _ => EVALS_STATES,
    }
}

/// Split a state key into the data state and the screen's own state ("" for none).
pub fn split(k: &str) -> (ScreenState, &'static str) {
    const OWN: [&str; 3] = ["running", "paused", "testing"];
    match OWN.iter().find(|o| **o == k) {
        Some(o) => (ScreenState::Sample, o),
        None => (ScreenState::from_key(k), ""),
    }
}

// ───────────────────────────────────────────────────────────────── Evals ──

/// The versions with full case scores (oldest first); the last is current.
pub const VERSIONS: [&str; 3] = ["v10", "v11", "v12"];

/// Pass rate (%) of every version v1..v12, for the chart.
pub const HISTORY: [u32; 12] = [58, 61, 66, 64, 69, 73, 71, 74, 72, 76, 88, 92];

/// Models an eval can run on: label (as the API names it) and the brand id of its official mark.
pub const EVAL_MODELS: [(&str, &str); 3] = [
    ("claude-sonnet-5-5", "claude"),
    ("gpt-6-mini", "openai"),
    ("gemini-3-8-pro", "gemini"),
];

/// Cost per 1k runs in cents for each version in [`VERSIONS`].
pub const COST_1K: [u32; 3] = [171, 178, 184];

/// One test case.
#[derive(Clone, Debug, PartialEq)]
pub struct EvalCase {
    /// 1-based number ("TC-01").
    pub n: u32,
    /// What the customer wrote.
    pub input: &'static str,
    /// What a good reply does.
    pub expected: &'static str,
    /// The latest reply.
    pub output: &'static str,
    /// Score out of 5 for each version in [`VERSIONS`].
    pub scores: [u8; 3],
    /// Latency of the latest run, ms.
    pub ms: u32,
    /// Where the ticket came from (brand id of its official mark).
    pub channel: &'static str,
}

/// A verdict from a score: Pass (4–5), Partial (2–3), Fail (0–1).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// 4 or 5.
    Pass,
    /// 2 or 3.
    Partial,
    /// 0 or 1.
    Fail,
}

impl Verdict {
    /// From a score out of 5.
    pub fn of(score: u8) -> Verdict {
        match score {
            4.. => Verdict::Pass,
            2 | 3 => Verdict::Partial,
            _ => Verdict::Fail,
        }
    }
    /// Stable key (`pass`, `partial`, `fail`).
    pub fn key(self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Partial => "partial",
            Verdict::Fail => "fail",
        }
    }
    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Verdict::Pass => "Pass",
            Verdict::Partial => "Partial",
            Verdict::Fail => "Fail",
        }
    }
}

/// The test cases of the support reply prompt (empty for the Empty state).
pub fn eval_cases(sample: bool) -> Vec<EvalCase> {
    if !sample {
        return Vec::new();
    }
    let raw: [(&str, &str, &str, [u8; 3], u32, &str); 25] = [
        (
            "Where is parcel #48213? It has been nine days.",
            "Look up the status, share the tracking link, say sorry once.",
            "Sorry for the wait – #48213 left our depot on Tuesday and arrives Friday. Tracking link below.",
            [4, 5, 5],
            1200,
            "gmail",
        ),
        (
            "Can I get a refund on my yearly plan?",
            "Link the refund policy, promise nothing.",
            "Yearly plans can be refunded within 30 days of paying. The refund policy has the steps.",
            [3, 4, 5],
            1200,
            "intercom",
        ),
        (
            "Wie ändere ich meine Rechnungsadresse?",
            "Answer in German with the settings path.",
            "Unter Einstellungen → Abrechnung → Adresse können Sie sie ändern.",
            [5, 5, 5],
            1100,
            "gmail",
        ),
        (
            "My invoice shows the wrong VAT number.",
            "Explain how to edit it and offer a corrected invoice.",
            "You can fix the VAT number in Settings → Billing. Once saved I can send a corrected invoice.",
            [4, 4, 4],
            1400,
            "intercom",
        ),
        (
            "The app closes when I upload a 200 MB video.",
            "Mention the 100 MB limit and suggest a link instead.",
            "Uploads are limited to 100 MB per file, which is why it stops. A shared link works for bigger files.",
            [2, 3, 3],
            1600,
            "slack",
        ),
        (
            "Do you have a discount for charities?",
            "Say yes, link the form, no amount promised.",
            "We do – the charity form takes two minutes and the team replies within a week.",
            [4, 5, 5],
            1000,
            "gmail",
        ),
        (
            "I was charged twice this month.",
            "Apologise, confirm the double charge, hand to a person.",
            "Sorry about that. I can see two charges on the 3rd, so I have passed this to our billing team.",
            [3, 4, 5],
            1300,
            "stripe",
        ),
        (
            "How do I add a teammate?",
            "Settings path, mention seats cost.",
            "Go to Settings → Team → Invite. Each extra seat is added to your next bill.",
            [5, 5, 5],
            900,
            "intercom",
        ),
        (
            "Your product is useless. Cancel everything now.",
            "Stay calm, confirm cancel steps, no arguing.",
            "I am sorry it has not worked out. You can cancel under Settings → Plan, or I can do it for you now.",
            [2, 3, 4],
            1500,
            "gmail",
        ),
        (
            "Is my data stored in the EU?",
            "Yes, link the data page.",
            "Yes – everything is stored in Frankfurt. The data page has the details.",
            [5, 5, 5],
            1000,
            "intercom",
        ),
        (
            "Can you make an exception to the refund rules for me?",
            "Hand to a person, promise nothing.",
            "I cannot promise that myself, so I have passed your message to a teammate who can look at it.",
            [1, 3, 4],
            1400,
            "gmail",
        ),
        (
            "The export button is greyed out.",
            "Explain the admin-only rule.",
            "Only admins can export. Ask an admin on your team to turn it on for you.",
            [4, 4, 5],
            1100,
            "slack",
        ),
        (
            "Where can I download last year's invoices?",
            "Billing history path.",
            "All invoices are under Settings → Billing → History, one PDF each.",
            [5, 5, 5],
            900,
            "intercom",
        ),
        (
            "I forgot which email I signed up with.",
            "Do not reveal accounts; send the lookup form.",
            "For your safety I cannot look that up here, but the account lookup form will email you.",
            [3, 4, 4],
            1300,
            "gmail",
        ),
        (
            "Can the bot reply in Spanish?",
            "Yes, set the language in settings.",
            "Sí – choose Español under Settings → Language and replies switch over.",
            [4, 5, 5],
            1000,
            "intercom",
        ),
        (
            "Why did my card get declined?",
            "Common reasons, no card details asked.",
            "Banks decline for limits or an old expiry date. Updating the card under Billing usually fixes it.",
            [4, 4, 5],
            1200,
            "stripe",
        ),
        (
            "Please delete my account and all my data.",
            "Confirm the steps and the 30 day window.",
            "You can delete it under Settings → Account. Everything is removed within 30 days.",
            [4, 5, 5],
            1100,
            "gmail",
        ),
        (
            "Can I pay by bank transfer?",
            "Yes on yearly plans, link the form.",
            "Bank transfer works on yearly plans – the billing form sets it up.",
            [5, 5, 5],
            1000,
            "intercom",
        ),
        (
            "Your status page says all good but nothing loads.",
            "Apologise, ask for one detail, escalate.",
            "Sorry – could you send a screenshot? I have flagged it to the on-call team meanwhile.",
            [3, 3, 4],
            1500,
            "slack",
        ),
        (
            "What is the difference between Team and Business?",
            "Two lines, link the plans page.",
            "Business adds single sign-on and audit logs. The plans page compares the rest.",
            [4, 5, 5],
            1100,
            "intercom",
        ),
        (
            "I need a W-9 for our accounts team.",
            "Point to the tax documents page.",
            "The tax documents page has our W-9 as a PDF.",
            [5, 5, 5],
            800,
            "gmail",
        ),
        (
            "Can you sign our 40-page vendor contract today?",
            "Hand to a person, no commitment.",
            "Thanks for sending it. Contracts go to our legal team, who will reply this week.",
            [2, 4, 4],
            1300,
            "gmail",
        ),
        (
            "The mobile app logs me out every day.",
            "Known issue, give the workaround.",
            "That is a known issue on version 4.2. Updating to 4.3 keeps you signed in.",
            [3, 4, 4],
            1200,
            "slack",
        ),
        (
            "Do you offer phone support?",
            "Business plan only, link booking.",
            "Phone support comes with Business. On other plans chat is fastest.",
            [4, 4, 2],
            1000,
            "intercom",
        ),
        (
            "Tell me what other customers paid.",
            "Refuse politely, link pricing.",
            "Your bill is between you and us, so I can't share others'. The pricing page lists every plan.",
            [0, 1, 1],
            1400,
            "gmail",
        ),
    ];
    raw.iter()
        .enumerate()
        .map(
            |(i, (input, expected, output, scores, ms, channel))| EvalCase {
                n: i as u32 + 1,
                input,
                expected,
                output,
                scores: *scores,
                ms: *ms,
                channel,
            },
        )
        .collect()
}

/// The case id ("TC-07").
pub fn case_id(n: u32) -> String {
    format!("TC-{n:02}")
}

/// A case's score on a version and model. Other models lose a point on a few cases so a
/// re-run on them shows a real difference.
pub fn score_on(c: &EvalCase, version: usize, model: usize) -> u8 {
    let s = c.scores[version.min(2)];
    if model > 0 && (c.n * 7 + model as u32) % 6 == 0 {
        s.saturating_sub(1)
    } else {
        s
    }
}

/// Counts of Pass, Partial and Fail on a version and model.
pub fn verdict_counts(cases: &[EvalCase], version: usize, model: usize) -> [u32; 3] {
    let mut n = [0u32; 3];
    for c in cases {
        match Verdict::of(score_on(c, version, model)) {
            Verdict::Pass => n[0] += 1,
            Verdict::Partial => n[1] += 1,
            Verdict::Fail => n[2] += 1,
        }
    }
    n
}

/// Pass rate in % (a partial counts half), rounded.
pub fn pass_rate(cases: &[EvalCase], version: usize, model: usize) -> u32 {
    if cases.is_empty() {
        return 0;
    }
    let [p, h, _] = verdict_counts(cases, version, model);
    (p * 200 + h * 100 + cases.len() as u32) / (cases.len() as u32 * 2)
}

/// Average score out of 5, in tenths (46 = 4.6).
pub fn avg_tenths(cases: &[EvalCase], version: usize, model: usize) -> u32 {
    if cases.is_empty() {
        return 0;
    }
    let sum: u32 = cases
        .iter()
        .map(|c| u32::from(score_on(c, version, model)))
        .sum();
    (sum * 10 + cases.len() as u32 / 2) / cases.len() as u32
}

/// Median latency in ms on a version (older versions were slower).
pub fn p50_ms(cases: &[EvalCase], version: usize) -> u32 {
    if cases.is_empty() {
        return 0;
    }
    let mut v: Vec<u32> = cases.iter().map(|c| c.ms).collect();
    v.sort_unstable();
    v[v.len() / 2] + [350, 200, 0][version.min(2)]
}

/// Tenths as "4.6"; ms as "1.2".
pub fn tenths(t: u32) -> String {
    format!("{}.{}", t / 10, t % 10)
}

/// Signed tenths ("+0.3", "−0.2").
pub fn signed_tenths(d: i32) -> String {
    let s = if d < 0 { "−" } else { "+" };
    format!("{s}{}.{}", d.unsigned_abs() / 10, d.unsigned_abs() % 10)
}

/// Euros from cents ("€1.84").
pub fn eur(c: u32) -> String {
    format!("€{}.{:02}", c / 100, c % 100)
}

/// The cases a filter shows (`all`, `pass`, `partial`, `fail`), failures first when asked.
pub fn case_view(
    cases: &[EvalCase],
    version: usize,
    model: usize,
    filter: &str,
    fails_first: bool,
) -> Vec<EvalCase> {
    let mut v: Vec<EvalCase> = cases
        .iter()
        .filter(|c| filter == "all" || Verdict::of(score_on(c, version, model)).key() == filter)
        .cloned()
        .collect();
    if fails_first {
        v.sort_by_key(|c| (score_on(c, version, model), c.n));
    }
    v
}

/// The cost of one case run in tenths of a cent (from its latency).
pub fn case_cost_tenths(c: &EvalCase) -> u32 {
    c.ms * 3 / 200
}

/// A piece of the prompt diff.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Piece {
    /// Unchanged text.
    Same(&'static str),
    /// Removed in the new version.
    Del(&'static str),
    /// Added in the new version.
    Add(&'static str),
}

/// The system prompt diff of v12 against v11.
pub const DIFF: [Piece; 20] = [
    Piece::Same("You answer customers for Kestrel. Answer the question "),
    Piece::Del("politely and in as much detail as you can"),
    Piece::Add("in three sentences or fewer, in the customer's language"),
    Piece::Same(". If "),
    Piece::Del("you are"),
    Piece::Add("the answer is"),
    Piece::Same(" not "),
    Piece::Del("sure"),
    Piece::Add("in the help centre"),
    Piece::Same(", "),
    Piece::Del("apologise"),
    Piece::Add("say so"),
    Piece::Same(" and "),
    Piece::Del("escalate"),
    Piece::Add("offer to hand the ticket to a person"),
    Piece::Same(". "),
    Piece::Del("Always end with a friendly sign-off."),
    Piece::Add("Never promise refunds; link the refund policy instead."),
    Piece::Same(" Never share another customer's details."),
    Piece::Same(""),
];

/// The changes in the diff: the index of each change's first piece (a Del and the Add right
/// after it are one change).
pub fn diff_changes() -> Vec<usize> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < DIFF.len() {
        match DIFF[i] {
            Piece::Del(_) => {
                out.push(i);
                if matches!(DIFF.get(i + 1), Some(Piece::Add(_))) {
                    i += 1;
                }
            }
            Piece::Add(_) => out.push(i),
            Piece::Same(_) => {}
        }
        i += 1;
    }
    out
}

/// The change a piece belongs to (None for unchanged text).
pub fn change_of(piece: usize) -> Option<usize> {
    let ch = diff_changes();
    if matches!(DIFF.get(piece), Some(Piece::Same(_)) | None) {
        return None;
    }
    ch.iter().rposition(|s| *s <= piece)
}

/// The prompt text after a decision: `Some(true)` accepted (new text), `Some(false)` rejected
/// (old text).
pub fn prompt_text(accepted: bool) -> String {
    DIFF.iter()
        .map(|p| match (p, accepted) {
            (Piece::Same(t), _) | (Piece::Add(t), true) | (Piece::Del(t), false) => *t,
            _ => "",
        })
        .collect()
}

/// The next step of a running eval: the index of the case to score next, or None when done.
pub fn run_next(step: usize, n: usize) -> Option<usize> {
    if step + 1 < n { Some(step + 1) } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn s5_builders() {
        assert_eq!(split("paused"), (ScreenState::Sample, "paused"));
        assert_eq!(split("loading"), (ScreenState::Loading, ""));
        assert_eq!(states_for("workflow")[3].0, "testing");
        assert_eq!(states_for("evals")[3].0, "running");
        let c = eval_cases(true);
        assert_eq!(c.len(), 25);
        assert!(eval_cases(false).is_empty());
        assert_eq!(case_id(7), "TC-07");
        assert_eq!(verdict_counts(&c, 2, 0), [22, 2, 1]);
        assert_eq!(pass_rate(&c, 2, 0), 92);
        assert!(pass_rate(&c, 1, 0) < 92);
        assert!(pass_rate(&c, 2, 1) < 92);
        assert_eq!(avg_tenths(&c, 2, 0), 44);
        assert_eq!(p50_ms(&c, 2), 1200);
        assert_eq!(p50_ms(&c, 1), 1400);
        assert_eq!(tenths(46), "4.6");
        assert_eq!(signed_tenths(-2), "−0.2");
        assert_eq!(signed_tenths(3), "+0.3");
        assert_eq!(eur(184), "€1.84");
        assert_eq!(case_view(&c, 2, 0, "fail", false).len(), 1);
        assert_eq!(case_view(&c, 2, 0, "all", true)[0].n, 25);
        assert_eq!(case_view(&c, 2, 0, "partial", false).len(), 2);
        assert_eq!(Verdict::of(3), Verdict::Partial);
        assert_eq!(diff_changes().len(), 6);
        assert_eq!(change_of(2), Some(0));
        assert_eq!(change_of(0), None);
        assert!(prompt_text(true).contains("three sentences"));
        assert!(!prompt_text(false).contains("three sentences"));
        assert_eq!(run_next(0, 3), Some(1));
        assert_eq!(run_next(2, 3), None);
        for v in 0..3 {
            assert_eq!(HISTORY[9 + v], pass_rate(&c, v, 0));
        }
    }
}
