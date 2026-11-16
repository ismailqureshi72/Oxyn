//! affectUI fidelity pass, family B part 8 (plan #32): the **Decision Tree** model. An agent stuck at
//! a fork lays its options out as a tree – question, two to four branches, the consequences of each
//! (some flagged with the agent's note), its own pick marked, an optional one-level follow-up per
//! branch – and a person picks a path or writes an instruction ("Other…"). Behaviour only: the
//! views live in `oxyn-dioxus` / `oxyn-gpui` (`parity/dtreefid.rs`).

/// How risky a decision is; the two upper levels get a coloured tag beside the agent.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DtRisk {
    /// Red tag.
    High,
    /// Amber tag.
    Medium,
    /// No tag.
    Low,
}

impl DtRisk {
    /// The tag text, `None` for low.
    pub fn tag(self) -> Option<&'static str> {
        match self {
            Self::High => Some("High risk"),
            Self::Medium => Some("Medium risk"),
            Self::Low => None,
        }
    }
    /// `high` / `medium` / `low`.
    pub fn id(self) -> &'static str {
        match self {
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// One run of text inside a consequence line; with a `note` it is a flag the agent explains.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DtSeg {
    /// The words.
    pub text: String,
    /// The agent's note – non-empty makes this run a flag.
    pub note: String,
    /// Where the note comes from (may be empty).
    pub source: String,
}

impl DtSeg {
    /// Plain text.
    pub fn plain(t: &str) -> Self {
        Self {
            text: t.into(),
            ..Self::default()
        }
    }
    /// A flagged run with the agent's note and an optional source.
    pub fn flag(t: &str, note: &str, source: &str) -> Self {
        Self {
            text: t.into(),
            note: note.into(),
            source: source.into(),
        }
    }
    /// Whether the run is flagged.
    pub fn flagged(&self) -> bool {
        !self.note.is_empty()
    }
}

/// One branch.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DtOption {
    /// What to do.
    pub label: String,
    /// The tool it runs with (brand id or glyph id: `revert`, `flag`, `pause`, `limit`, `page`, `mail`, `status`).
    pub icon: String,
    /// Its consequences: one entry per line, each line made of one or more runs.
    pub effects: Vec<Vec<DtSeg>>,
    /// Systems it touches (brand / glyph ids), drawn after the effects.
    pub apps: Vec<String>,
    /// The agent's own preference.
    pub pick: bool,
    /// A second question asked only on this branch (one level deep).
    pub next: Option<DtFollow>,
}

/// A follow-up question and the two to four answers it offers.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DtFollow {
    /// The question.
    pub question: String,
    /// Its options (their own `next` is ignored).
    pub options: Vec<DtOption>,
}

/// Everything waiting for one call.
#[derive(Clone, Debug, PartialEq)]
pub struct DtDecision {
    /// Stable id.
    pub id: String,
    /// One-sentence question.
    pub question: String,
    /// Context line under it.
    pub context: String,
    /// The agent asking.
    pub agent: String,
    /// Its model.
    pub model: String,
    /// The agent's glyph (`burst`, `coin`, `cloud`, brand id …).
    pub icon: String,
    /// Risk.
    pub risk: DtRisk,
    /// Two to four branches.
    pub options: Vec<DtOption>,
}

/// What a key does in the tree.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DtKey {
    /// Pick the n-th branch (of the follow-up when it is awake).
    Pick(usize),
    /// Open "Other…".
    Other,
    /// Next decision.
    Next,
    /// Previous decision.
    Prev,
    /// Show the next flagged note.
    Flag,
    /// Close "Other…" / a note.
    Close,
}

/// Map a key to an action: digits 1..=n pick, n+1 opens "Other…" (when allowed), arrows / J K page,
/// F flags, Escape closes. `n` is the number of options at the active level; `other` is only offered
/// at the first level.
pub fn dt_key(key: &str, n: usize, other: bool) -> Option<DtKey> {
    match key {
        "ArrowRight" | "j" | "J" => Some(DtKey::Next),
        "ArrowLeft" | "k" | "K" => Some(DtKey::Prev),
        "f" | "F" => Some(DtKey::Flag),
        "Escape" => Some(DtKey::Close),
        _ => {
            let d: usize = key.parse().ok()?;
            if d >= 1 && d <= n {
                Some(DtKey::Pick(d - 1))
            } else if other && d == n + 1 {
                Some(DtKey::Other)
            } else {
                None
            }
        }
    }
}

/// The agent's pick among options, if one is marked.
pub fn dt_pick(opts: &[DtOption]) -> Option<usize> {
    opts.iter().position(|o| o.pick)
}

/// "3 waiting for you" / "1 waiting for you" / "Nothing waiting".
pub fn dt_waiting(n: usize) -> String {
    if n == 0 {
        "Nothing waiting".into()
    } else {
        format!("{n} waiting for you")
    }
}

/// The key hint at the active level: "1 – 4" (with Other) or "1 – 2".
pub fn dt_range(n: usize, other: bool) -> String {
    let top = if other { n + 1 } else { n };
    if top <= 1 {
        "1".into()
    } else {
        format!("1 – {top}")
    }
}

/// Every flagged run of a decision in reading order: (option, follow-up option or None, line, run).
pub fn dt_flags(d: &DtDecision) -> Vec<(usize, Option<usize>, usize, usize)> {
    let mut out = vec![];
    for (oi, o) in d.options.iter().enumerate() {
        push_flags(&o.effects, oi, None, &mut out);
        if let Some(f) = &o.next {
            for (fi, fo) in f.options.iter().enumerate() {
                push_flags(&fo.effects, oi, Some(fi), &mut out);
            }
        }
    }
    out
}

fn push_flags(
    eff: &[Vec<DtSeg>],
    oi: usize,
    fi: Option<usize>,
    out: &mut Vec<(usize, Option<usize>, usize, usize)>,
) {
    for (li, line) in eff.iter().enumerate() {
        for (si, s) in line.iter().enumerate() {
            if s.flagged() {
                out.push((oi, fi, li, si));
            }
        }
    }
}

/// The next flag after `cur` (wrapping), `None` when the decision has none.
pub fn dt_next_flag(d: &DtDecision, cur: Option<usize>) -> Option<usize> {
    let n = dt_flags(d).len();
    (n > 0).then(|| cur.map_or(0, |c| (c + 1) % n))
}

/// Where a decision stands.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct DtState {
    /// The picked branch.
    pub branch: Option<usize>,
    /// The picked follow-up option.
    pub then: Option<usize>,
    /// The instruction written in "Other…", once sent.
    pub other: Option<String>,
}

impl DtState {
    /// Whether the decision is settled: a leaf branch picked, a follow-up answered, or an instruction sent.
    pub fn done(&self, d: &DtDecision) -> bool {
        if self.other.is_some() {
            return true;
        }
        match self.branch {
            None => false,
            Some(b) => d
                .options
                .get(b)
                .is_none_or(|o| o.next.is_none() || self.then.is_some()),
        }
    }
    /// Pick at the active level; returns false when the pick does not apply (already settled).
    pub fn pick(&mut self, d: &DtDecision, i: usize) -> bool {
        if self.done(d) {
            return false;
        }
        match self.branch {
            None if i < d.options.len() => {
                self.branch = Some(i);
                true
            }
            Some(b) => {
                let n = d.options[b].next.as_ref().map_or(0, |f| f.options.len());
                if i < n {
                    self.then = Some(i);
                    true
                } else {
                    false
                }
            }
            _ => false,
        }
    }
    /// The number of options at the active level (follow-up once a branch with one is picked).
    pub fn active_len(&self, d: &DtDecision) -> usize {
        match self.branch {
            Some(b) => d.options[b].next.as_ref().map_or(0, |f| f.options.len()),
            None => d.options.len(),
        }
    }
}

/// The live-region line for a pick: "Revert to build 1206 – picked. Deploy agent goes ahead."
pub fn dt_announce(d: &DtDecision, s: &DtState) -> String {
    if let Some(o) = &s.other {
        return format!("Instruction sent to the {}: {o}", d.agent.to_lowercase());
    }
    let Some(b) = s.branch else {
        return String::new();
    };
    let opt = &d.options[b];
    let label = match (s.then, &opt.next) {
        (Some(t), Some(f)) => f.options[t].label.clone(),
        _ => opt.label.clone(),
    };
    if s.done(d) {
        format!("{label} – picked. {} goes ahead.", d.agent)
    } else {
        format!(
            "{label} – picked. Then: {}",
            opt.next.as_ref().map_or("", |f| f.question.as_str())
        )
    }
}

/// The agent's short answer to a written instruction (demo stand-in for `onChoose`).
pub fn dt_reply(d: &DtDecision) -> String {
    format!(
        "Got it – the {} will do that instead.",
        d.agent.to_lowercase()
    )
}

fn eff(lines: &[&str]) -> Vec<Vec<DtSeg>> {
    lines.iter().map(|l| vec![DtSeg::plain(l)]).collect()
}

fn opt(label: &str, icon: &str, lines: &[&str], apps: &[&str], pick: bool) -> DtOption {
    DtOption {
        label: label.into(),
        icon: icon.into(),
        effects: eff(lines),
        apps: apps.iter().map(|a| (*a).to_string()).collect(),
        pick,
        next: None,
    }
}

/// The demo queue: three decisions, invented, no real names.
pub fn dt_demo() -> Vec<DtDecision> {
    let mut revert = opt(
        "Revert to build 1206",
        "revert",
        &["Sign-ins recover in ~3 min", "Drops the new SSO screen"],
        &[],
        true,
    );
    revert.next = Some(DtFollow {
        question: "Post a status update?".into(),
        options: vec![
            opt(
                "Update the status page",
                "status",
                &["Public · refreshed every 10 min"],
                &["status", "slack"],
                true,
            ),
            opt(
                "Team channel only",
                "slack",
                &["#ops-alerts · nothing public"],
                &["slack"],
                false,
            ),
        ],
    });
    let mut patch = opt(
        "Patch forward",
        "github",
        &["Fix is 2 lines · ~20 min to ship"],
        &["github"],
        false,
    );
    let mut skip = opt("Skip the test suite", "github", &[], &["github"], false);
    skip.effects = vec![vec![
        DtSeg::plain("Ships in 4 min · "),
        DtSeg::flag(
            "Bypasses 912 tests",
            "Last time the suite was skipped, a second fix was needed within the hour.",
            "Deploy log · last quarter",
        ),
    ]];
    patch.next = Some(DtFollow {
        question: "Wait for CI?".into(),
        options: vec![
            opt(
                "Wait for CI",
                "github",
                &["~11 min more downtime"],
                &["github"],
                true,
            ),
            skip,
        ],
    });
    let flag = opt(
        "Turn off the sso flag",
        "flag",
        &["Instant · old sign-in for everyone"],
        &[],
        false,
    );
    let d1 = DtDecision {
        id: "signin".into(),
        question: "Sign-in errors rose to 9% after build 1207 – how should I proceed?".into(),
        context: "1,860 failed sign-ins in 15 minutes · support tickets doubling.".into(),
        agent: "Deploy agent".into(),
        model: "Opus 4.1".into(),
        icon: "burst".into(),
        risk: DtRisk::High,
        options: vec![revert, patch, flag],
    };

    let mut counter = opt(
        "Counter with 12%",
        "hubspot",
        &["Keeps ~$41k a year in margin"],
        &["hubspot"],
        true,
    );
    counter.next = Some(DtFollow {
        question: "Ask for a longer term?".into(),
        options: vec![
            opt(
                "Two-year term",
                "hubspot",
                &["Locks the price until 2028"],
                &["hubspot"],
                true,
            ),
            opt(
                "Keep it yearly",
                "hubspot",
                &["No change to terms"],
                &["hubspot"],
                false,
            ),
        ],
    });
    let mut accept = opt("Accept 30%", "stripe", &[], &["stripe"], false);
    accept.effects = vec![vec![DtSeg::flag(
        "$58k a year · needs approval",
        "Discounts over 20% need a finance sign-off before they are offered.",
        "Pricing policy",
    )]];
    let mut escalate = opt(
        "Ask the account lead",
        "mail",
        &["Reply expected within a day"],
        &["gmail"],
        false,
    );
    escalate.next = Some(DtFollow {
        question: "Attach the usage report?".into(),
        options: vec![
            opt("Attach it", "mail", &["Shows 3× seat growth"], &[], true),
            opt("Send the ask alone", "mail", &["Shorter email"], &[], false),
        ],
    });
    let d2 = DtDecision {
        id: "renewal".into(),
        question: "A customer asks for 30% off to renew – counter or accept?".into(),
        context: "Renewal is due Friday · 140 seats on the annual plan.".into(),
        agent: "Billing agent".into(),
        model: "GPT-5".into(),
        icon: "coin".into(),
        risk: DtRisk::Medium,
        options: vec![counter, accept, escalate],
    };

    let mut stop = opt(
        "Stop the backfill",
        "pause",
        &["Spend stops now"],
        &["aws"],
        true,
    );
    stop.next = Some(DtFollow {
        question: "Keep the rows written so far?".into(),
        options: vec![
            opt(
                "Roll them back",
                "revert",
                &["Table back to last night"],
                &["postgresql"],
                true,
            ),
            opt(
                "Keep and dedupe later",
                "postgresql",
                &["~4.1M duplicate rows stay"],
                &["postgresql"],
                false,
            ),
        ],
    });
    let d3 = DtDecision {
        id: "backfill".into(),
        question: "The nightly backfill is writing twice the expected rows – stop it?".into(),
        context: "Compute cost up 260% since 02:00 · still running.".into(),
        agent: "Data agent".into(),
        model: "Gemini 2.5 Pro".into(),
        icon: "cloud".into(),
        risk: DtRisk::High,
        options: vec![
            stop,
            opt(
                "Cap the spend",
                "limit",
                &["Pauses at $400 · job keeps going"],
                &["aws"],
                false,
            ),
            opt(
                "Page the on-call",
                "page",
                &["Someone looks in ~10 min"],
                &["slack"],
                false,
            ),
        ],
    };
    vec![d1, d2, d3]
}

/// One more decision, used by Classic and by the "a new one joins" demo.
pub fn dt_extra() -> DtDecision {
    DtDecision {
        id: "refund".into(),
        question: "A duplicate charge was found on 37 orders – refund them all now?".into(),
        context: "$1,184 in total · customers not told yet.".into(),
        agent: "Support agent".into(),
        model: "Sonnet 4.5".into(),
        icon: "stripe".into(),
        risk: DtRisk::Medium,
        options: vec![
            opt(
                "Refund all 37",
                "stripe",
                &["$1,184 · not reversible"],
                &["stripe"],
                true,
            ),
            opt(
                "Refund on request",
                "mail",
                &["Only customers who write in"],
                &["gmail"],
                false,
            ),
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_map_to_branches_other_and_paging() {
        assert_eq!(dt_key("1", 3, true), Some(DtKey::Pick(0)));
        assert_eq!(dt_key("3", 3, true), Some(DtKey::Pick(2)));
        assert_eq!(dt_key("4", 3, true), Some(DtKey::Other));
        assert_eq!(dt_key("4", 3, false), None);
        assert_eq!(dt_key("0", 3, true), None);
        assert_eq!(dt_key("J", 3, true), Some(DtKey::Next));
        assert_eq!(dt_key("ArrowLeft", 3, true), Some(DtKey::Prev));
        assert_eq!(dt_key("f", 2, true), Some(DtKey::Flag));
        assert_eq!(dt_range(3, true), "1 – 4");
        assert_eq!(dt_range(2, false), "1 – 2");
        assert_eq!(dt_waiting(3), "3 waiting for you");
    }

    #[test]
    fn two_level_pick_settles_and_announces() {
        let d = &dt_demo()[0];
        assert_eq!(dt_pick(&d.options), Some(0));
        let mut s = DtState::default();
        assert!(!s.done(d));
        assert!(s.pick(d, 0));
        assert!(!s.done(d), "a branch with a follow-up waits for its answer");
        assert_eq!(s.active_len(d), 2);
        assert!(dt_announce(d, &s).contains("Then: Post a status update?"));
        assert!(!s.pick(d, 5));
        assert!(s.pick(d, 0));
        assert!(s.done(d));
        assert_eq!(
            dt_announce(d, &s),
            "Update the status page – picked. Deploy agent goes ahead."
        );
        assert!(!s.pick(d, 1), "settled decisions take no more picks");
        // A leaf branch settles at once.
        let mut l = DtState::default();
        assert!(l.pick(d, 2));
        assert!(l.done(d));
    }

    #[test]
    fn flags_are_listed_in_reading_order_and_wrap() {
        let d = &dt_demo()[0];
        let f = dt_flags(d);
        assert_eq!(f, vec![(1, Some(1), 0, 1)]);
        assert_eq!(dt_next_flag(d, None), Some(0));
        assert_eq!(dt_next_flag(d, Some(0)), Some(0));
        let mut x = dt_extra();
        x.options[0].effects.clear();
        assert_eq!(dt_next_flag(&x, None), None);
        let o = DtState {
            other: Some("Revert, then page the owner".into()),
            ..DtState::default()
        };
        assert!(o.done(d));
        assert!(dt_announce(d, &o).starts_with("Instruction sent to the deploy agent"));
    }

    #[test]
    fn demo_shape() {
        for d in dt_demo().iter().chain([dt_extra()].iter()) {
            assert!((2..=4).contains(&d.options.len()));
            assert!(d.options.iter().filter(|o| o.pick).count() <= 1);
            for o in &d.options {
                if let Some(f) = &o.next {
                    assert!((2..=4).contains(&f.options.len()));
                }
            }
        }
    }
}
