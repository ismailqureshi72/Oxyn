//! affectUI fidelity pass (plan #32, overnight batch, family B part 4): the model behind the exact
//! **Agent Ledger** – everything an agent changed in your systems, newest first, each change with
//! an undo. Pointing at Undo shows what builds on it: what would be undone too and what can't be
//! taken back. Behaviour only; times are milliseconds on a clock the caller passes in.

/// One change an agent made.
#[derive(Clone, Debug, PartialEq)]
pub struct LedgerEntry {
    /// Id.
    pub id: String,
    /// What it did ("Changed price"), shown in bold.
    pub action: String,
    /// What it did it to ("Team plan"), muted after the action.
    pub target: String,
    /// The system ("Stripe"), after the time.
    pub system: String,
    /// Brand mark for the tile (the system's, lowercased, when empty).
    pub brand: String,
    /// When (ms on the clock).
    pub at: u64,
    /// Before → after; empty `before` shows `detail` instead.
    pub change: (String, String),
    /// A single detail line when there is no before / after.
    pub detail: String,
    /// `false` for what can't be taken back (a sent email): no Undo.
    pub reversible: bool,
    /// The changes this one builds on.
    pub depends_on: Vec<String>,
}

impl LedgerEntry {
    /// The brand mark to draw.
    pub fn mark(&self) -> String {
        if self.brand.is_empty() {
            self.system.to_lowercase()
        } else {
            self.brand.clone()
        }
    }
    /// Whether it shows before → after.
    pub fn has_change(&self) -> bool {
        !self.change.0.is_empty()
    }
}

/// "just now" · "18s ago" · "2m ago" · "1h ago" · "2d ago".
pub fn ledger_ago(now: u64, at: u64) -> String {
    let s = now.saturating_sub(at) / 1000;
    if s < 10 {
        "just now".into()
    } else if s < 60 {
        format!("{s}s ago")
    } else if s < 3600 {
        format!("{}m ago", s / 60)
    } else if s < 86_400 {
        format!("{}h ago", s / 3600)
    } else {
        format!("{}d ago", s / 86_400)
    }
}

/// Everything that builds on `id`, directly or not, in list order (newest first).
pub fn ledger_dependents(entries: &[LedgerEntry], id: &str) -> Vec<String> {
    let mut found: Vec<String> = vec![id.to_string()];
    loop {
        let before = found.len();
        for e in entries {
            if !found.contains(&e.id) && e.depends_on.iter().any(|d| found.contains(d)) {
                found.push(e.id.clone());
            }
        }
        if found.len() == before {
            break;
        }
    }
    let mut out: Vec<String> = ledger_sorted(entries)
        .into_iter()
        .map(|e| e.id.clone())
        .filter(|x| x != id && found.contains(x))
        .collect();
    out.dedup();
    out
}

/// What pointing at `id`'s Undo means: (ids that would be undone with it – itself first – and
/// ids that build on it but can't be undone). Already undone changes are left out.
pub fn ledger_undo_plan(
    entries: &[LedgerEntry],
    id: &str,
    undone: &[String],
) -> (Vec<String>, Vec<String>) {
    let mut go = vec![id.to_string()];
    let mut kept = vec![];
    for d in ledger_dependents(entries, id) {
        if undone.contains(&d) {
            continue;
        }
        match entries.iter().find(|e| e.id == d) {
            Some(e) if e.reversible => go.push(d),
            Some(_) => kept.push(d),
            None => {}
        }
    }
    (go, kept)
}

/// The entries newest first (ties keep their order).
pub fn ledger_sorted(entries: &[LedgerEntry]) -> Vec<&LedgerEntry> {
    let mut v: Vec<&LedgerEntry> = entries.iter().collect();
    v.sort_by_key(|e| std::cmp::Reverse(e.at));
    v
}

/// Undo's accessible name: "Undo: Changed price Team plan – and 2 that build on it".
pub fn undo_label(e: &LedgerEntry, also: usize) -> String {
    let t = if e.target.is_empty() {
        e.action.clone()
    } else {
        format!("{} {}", e.action, e.target)
    };
    match also {
        0 => format!("Undo: {t}"),
        n => format!("Undo: {t} – and {n} that build on it"),
    }
}

/// "8 changes" / "1 change".
pub fn changes_said(n: usize) -> String {
    format!("{n} change{}", if n == 1 { "" } else { "s" })
}

/// What the demo does at one moment.
#[derive(Clone, Debug, PartialEq)]
pub struct LedgerScene {
    /// Entries shown so far.
    pub entries: Vec<LedgerEntry>,
    /// Undone ids.
    pub undone: Vec<String>,
    /// The Undo being pointed at.
    pub point: Option<String>,
    /// The clock (ms).
    pub now: u64,
}

/// One pass of the demo (real ms) before it starts over.
pub const LEDGER_LOOP_MS: u64 = 22_000;

#[allow(clippy::too_many_arguments)]
fn entry(
    id: &str,
    action: &str,
    target: &str,
    system: &str,
    at_s: u64,
    change: (&str, &str),
    detail: &str,
    deps: &[&str],
) -> LedgerEntry {
    LedgerEntry {
        id: id.into(),
        action: action.into(),
        target: target.into(),
        system: system.into(),
        brand: String::new(),
        at: at_s * 1000,
        change: (change.0.into(), change.1.into()),
        detail: detail.into(),
        reversible: true,
        depends_on: deps.iter().map(|s| s.to_string()).collect(),
    }
}

/// The demo's eight changes, oldest first; the n-th lands `n` seconds into a pass (the clock
/// itself runs 20× faster so the times read in minutes).
pub fn ledger_demo_entries() -> Vec<LedgerEntry> {
    let mut v = vec![
        entry(
            "pr",
            "Merged PR #212",
            "Plans page v2",
            "GitHub",
            0,
            ("plans-v2", "main"),
            "",
            &[],
        ),
        entry(
            "deploy",
            "Deployed",
            "shop.example",
            "Vercel",
            20,
            ("#90", "#92 in production"),
            "",
            &["pr"],
        ),
        entry(
            "price",
            "Changed price",
            "Team plan",
            "Stripe",
            40,
            ("$19 / month", "$24 / month"),
            "",
            &["deploy"],
        ),
        entry(
            "coupon",
            "Created coupon",
            "SPRING15",
            "Stripe",
            60,
            ("", ""),
            "15% off the first year · ends in 7 days",
            &["price"],
        ),
        entry(
            "faq",
            "Updated page",
            "Plans FAQ",
            "Notion",
            80,
            ("4 answers", "4 answers rewritten"),
            "",
            &[],
        ),
        entry(
            "deals",
            "Updated 140 deals",
            "Renewals in Q1",
            "HubSpot",
            100,
            ("$19 per seat", "$24 per seat"),
            "",
            &["price"],
        ),
        entry(
            "post",
            "Posted in #launch",
            "The new plans are out",
            "Slack",
            120,
            ("", ""),
            "With a link to the plans page",
            &["deploy"],
        ),
        entry(
            "email",
            "Sent email",
            "2,460 customers",
            "HubSpot",
            140,
            ("", ""),
            "The new price and the code SPRING15",
            &["deals", "coupon"],
        ),
    ];
    v[7].reversible = false;
    v
}

/// Demo clock speed: 20 clock-seconds per real second.
pub const LEDGER_SPEED: u64 = 20;

/// The demo `t` real ms into a pass: a change lands every second; at 11 s the Undo of the price
/// change is pointed at, at 13 s it is undone (with the coupon and the deals), at 18 s the
/// pointer leaves.
pub fn ledger_at(t: u64) -> LedgerScene {
    let all = ledger_demo_entries();
    let n = ((t / 1000) as usize + 1).min(all.len());
    let entries: Vec<LedgerEntry> = all.into_iter().take(n).collect();
    // The clock starts as the last change lands; earlier ones sit further back.
    let now = 140_000 + t * LEDGER_SPEED;
    let (undone, point) = if t >= 13_000 {
        let (go, _) = ledger_undo_plan(&entries, "price", &[]);
        (go, (t < 18_000).then(|| "price".to_string()))
    } else if t >= 11_000 {
        (vec![], Some("price".to_string()))
    } else {
        (vec![], None)
    };
    LedgerScene {
        entries,
        undone,
        point,
        now,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_takes_the_chain_and_keeps_what_cannot_go() {
        let e = ledger_demo_entries();
        assert_eq!(ledger_dependents(&e, "price"), ["email", "deals", "coupon"]);
        let (go, kept) = ledger_undo_plan(&e, "price", &[]);
        assert_eq!(go, ["price", "deals", "coupon"]);
        assert_eq!(kept, ["email"]);
        assert_eq!(
            undo_label(&e[2], go.len() - 1),
            "Undo: Changed price Team plan – and 2 that build on it"
        );
        assert_eq!(ledger_undo_plan(&e, "faq", &[]).0, ["faq"]);
        assert_eq!(changes_said(1), "1 change");
    }

    #[test]
    fn the_demo_streams_then_undoes() {
        assert_eq!(ledger_at(0).entries.len(), 1);
        assert_eq!(ledger_at(9_000).entries.len(), 8);
        assert_eq!(ledger_at(11_500).point.as_deref(), Some("price"));
        let s = ledger_at(14_000);
        assert_eq!(s.undone.len(), 3);
        assert_eq!(ledger_sorted(&s.entries)[0].id, "email");
        assert_eq!(ledger_ago(s.now, 140_000), "4m ago");
        assert_eq!(ledger_ago(5_000, 0), "just now");
    }
}
