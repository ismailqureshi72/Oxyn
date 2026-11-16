//! affectUI fidelity pass (plan #32, overnight batch, family B part 6): the model behind the exact
//! **Autonomy Dial** – how much an agent may do on its own: three levels (Always ask, If risky,
//! Full auto), the agent's real actions split between "Runs on its own" and "Asks you first", and
//! a lock per action that always asks. Behaviour only.

/// How risky an action is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AutonomyRisk {
    /// Runs at "If risky"; carries no tag.
    Safe,
    /// Asks at "If risky"; an amber tag.
    Risky,
    /// Asks at "If risky"; a red tag.
    Destructive,
}

impl AutonomyRisk {
    /// `safe` · `risky` · `destructive`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Safe => "safe",
            Self::Risky => "risky",
            Self::Destructive => "destructive",
        }
    }
    /// The tag's word (empty for safe actions).
    pub fn tag(self) -> &'static str {
        match self {
            Self::Safe => "",
            Self::Risky => "Risky",
            Self::Destructive => "Destructive",
        }
    }
}

/// One thing the agent can do.
#[derive(Clone, Debug, PartialEq)]
pub struct AutonomyAction {
    /// Id.
    pub id: String,
    /// What it does.
    pub label: String,
    /// One optional line under it (left out under 340 px).
    pub detail: String,
    /// Risk.
    pub risk: AutonomyRisk,
    /// Brand mark (empty for none).
    pub brand: String,
}

/// The three stops: 0 Always ask, 1 If risky, 2 Full auto.
pub const AUTONOMY_LEVELS: [&str; 3] = ["Always ask", "If risky", "Full auto"];

/// What a level means, in one sentence (the radio group's description).
pub fn autonomy_meaning(level: u8) -> &'static str {
    match level {
        0 => "Nothing happens without your OK",
        1 => "Safe actions run, risky ones wait for you",
        _ => "Everything runs, except what you locked",
    }
}

/// The rule the running agent uses too: a lock always asks; level 0 asks for everything; level 1
/// runs only safe actions; level 2 runs everything not locked.
pub fn asks_first(a: &AutonomyAction, level: u8, pinned: &[String]) -> bool {
    if pinned.iter().any(|p| p == &a.id) {
        return true;
    }
    match level {
        0 => true,
        1 => a.risk != AutonomyRisk::Safe,
        _ => false,
    }
}

/// (runs on its own, asks you first) – each in the actions' own order.
pub fn autonomy_split(
    actions: &[AutonomyAction],
    level: u8,
    pinned: &[String],
) -> (Vec<AutonomyAction>, Vec<AutonomyAction>) {
    actions
        .iter()
        .cloned()
        .partition(|a| !asks_first(a, level, pinned))
}

/// The polite announcement: "If risky: 2 on its own, 4 ask first".
pub fn autonomy_status(actions: &[AutonomyAction], level: u8, pinned: &[String]) -> String {
    let (own, ask) = autonomy_split(actions, level, pinned);
    format!(
        "{}: {} on its own, {} {} first",
        AUTONOMY_LEVELS[level.min(2) as usize],
        own.len(),
        ask.len(),
        if ask.len() == 1 { "asks" } else { "ask" }
    )
}

/// Sets or removes a lock.
pub fn autonomy_toggle(pinned: &[String], id: &str) -> Vec<String> {
    if pinned.iter().any(|p| p == id) {
        pinned.iter().filter(|p| *p != id).cloned().collect()
    } else {
        let mut v = pinned.to_vec();
        v.push(id.to_string());
        v
    }
}

/// The next level for an arrow key (`delta` −1 or +1), clamped like a radio group that wraps.
pub fn autonomy_step(level: u8, delta: i8) -> u8 {
    ((level as i8 + delta).rem_euclid(3)) as u8
}

/// The demo agent's actions (our own copy).
pub fn autonomy_demo() -> Vec<AutonomyAction> {
    let a = |id: &str, label: &str, risk, brand: &str| AutonomyAction {
        id: id.into(),
        label: label.into(),
        detail: String::new(),
        risk,
        brand: brand.into(),
    };
    use AutonomyRisk::*;
    vec![
        a("read", "Read the help center", Safe, "notion"),
        a("draft", "Draft replies", Safe, "gmail"),
        a("tickets", "Update ticket fields", Risky, "hubspot"),
        a("send", "Send replies", Risky, "gmail"),
        a("purge", "Remove attachments", Destructive, "googledrive"),
        a("refund", "Issue refunds", Destructive, "stripe"),
    ]
}

/// The demo's starting locks.
pub fn autonomy_demo_pinned() -> Vec<String> {
    vec!["purge".into(), "refund".into()]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_rule_and_the_split() {
        let a = autonomy_demo();
        let p = autonomy_demo_pinned();
        let n = |l| {
            let (o, k) = autonomy_split(&a, l, &p);
            (o.len(), k.len())
        };
        assert_eq!(n(0), (0, 6));
        assert_eq!(n(1), (2, 4));
        assert_eq!(n(2), (4, 2));
        assert_eq!(
            autonomy_status(&a, 1, &p),
            "If risky: 2 on its own, 4 ask first"
        );
        let p2 = autonomy_toggle(&p, "read");
        assert_eq!(n(1), (2, 4));
        assert_eq!(autonomy_split(&a, 1, &p2).0.len(), 1);
        assert_eq!(autonomy_toggle(&p2, "read"), p);
        assert!(asks_first(&a[5], 2, &p));
        assert!(!asks_first(&a[3], 2, &p));
        assert_eq!(autonomy_step(0, -1), 2);
        assert_eq!(autonomy_step(2, 1), 0);
        assert_eq!(autonomy_step(1, 1), 2);
        assert_eq!(AutonomyRisk::Risky.tag(), "Risky");
    }
}
