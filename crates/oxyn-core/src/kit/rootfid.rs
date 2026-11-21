//! affectUI fidelity pass, family B part 9 (plan #32): the **Root Cause Tree** model. Why a run
//! failed, as a fault tree: the failure on top, hypotheses under it joined by an OR or AND gate,
//! some with their own sub-hypotheses. The agent checks them one after another – each is ruled out
//! or confirmed with a verdict and its evidence (a tool call or a diff) – and the confirmed route
//! to the failure is highlighted. Behaviour only: the views live in `oxyn-dioxus` / `oxyn-gpui`
//! (`kit/rootfid.rs`).

/// How children combine.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RcGate {
    /// Any one child explains the parent.
    Or,
    /// All children together explain it.
    And,
}

/// Where a hypothesis stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RcStatus {
    /// Not checked yet.
    Open,
    /// Being checked.
    Checking,
    /// Checked and not the cause.
    RuledOut,
    /// Checked and true.
    Confirmed,
}

impl RcStatus {
    /// `open` / `checking` / `ruled-out` / `confirmed`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Checking => "checking",
            Self::RuledOut => "ruled-out",
            Self::Confirmed => "confirmed",
        }
    }
}

/// What backs a verdict.
#[derive(Clone, Debug, PartialEq)]
pub enum RcEvidence {
    /// A tool call and its output lines.
    Tool(String, Vec<String>),
    /// A file and its diff lines ("-…" / "+…" / " …").
    Diff(String, Vec<String>),
}

/// One node of the tree.
#[derive(Clone, Debug, PartialEq)]
pub struct RcNode {
    /// Stable id.
    pub id: String,
    /// Title.
    pub title: String,
    /// Brand id for its mark.
    pub brand: String,
    /// Files involved.
    pub files: Vec<String>,
    /// How its children combine.
    pub gate: RcGate,
    /// The verdict once checked.
    pub verdict: String,
    /// The evidence once checked.
    pub evidence: Option<RcEvidence>,
    /// How the check ends.
    pub outcome: RcStatus,
    /// Run time (s) when the check starts and when it ends; `None` = never checked.
    pub check: Option<(u32, u32)>,
    /// Sub-hypotheses.
    pub children: Vec<RcNode>,
}

/// A node's status at run time `t`.
pub fn rc_status(n: &RcNode, t: u32) -> RcStatus {
    match n.check {
        Some((_, b)) if t >= b => n.outcome,
        Some((a, _)) if t >= a => RcStatus::Checking,
        _ => RcStatus::Open,
    }
}

/// Every hypothesis (not the failure itself) depth-first, with its depth (1 = under the failure).
pub fn rc_flat(failure: &RcNode) -> Vec<(&RcNode, usize)> {
    fn walk<'a>(n: &'a RcNode, d: usize, out: &mut Vec<(&'a RcNode, usize)>) {
        for c in &n.children {
            out.push((c, d));
            walk(c, d + 1, out);
        }
    }
    let mut v = vec![];
    walk(failure, 1, &mut v);
    v
}

/// (hypotheses, ruled out, confirmed) at `t`.
pub fn rc_counts(failure: &RcNode, t: u32) -> (usize, usize, usize) {
    let f = rc_flat(failure);
    let r = f
        .iter()
        .filter(|(n, _)| rc_status(n, t) == RcStatus::RuledOut)
        .count();
    let c = f
        .iter()
        .filter(|(n, _)| rc_status(n, t) == RcStatus::Confirmed)
        .count();
    (f.len(), r, c)
}

/// Is the cause found at `t` (a confirmed node directly under the failure)?
pub fn rc_found(failure: &RcNode, t: u32) -> bool {
    failure
        .children
        .iter()
        .any(|c| rc_status(c, t) == RcStatus::Confirmed)
}

/// The root causes at `t`: confirmed nodes with no confirmed children.
pub fn rc_roots(failure: &RcNode, t: u32) -> Vec<String> {
    rc_flat(failure)
        .into_iter()
        .filter(|(n, _)| {
            rc_status(n, t) == RcStatus::Confirmed
                && !n
                    .children
                    .iter()
                    .any(|c| rc_status(c, t) == RcStatus::Confirmed)
        })
        .map(|(n, _)| n.id.clone())
        .collect()
}

/// Ids on the lit path at `t`: every confirmed node (once the cause is found).
pub fn rc_path(failure: &RcNode, t: u32) -> Vec<String> {
    if !rc_found(failure, t) {
        return vec![];
    }
    rc_flat(failure)
        .into_iter()
        .filter(|(n, _)| rc_status(n, t) == RcStatus::Confirmed)
        .map(|(n, _)| n.id.clone())
        .collect()
}

/// When the whole check ends (the last check's end).
pub fn rc_len(failure: &RcNode) -> u32 {
    rc_flat(failure)
        .iter()
        .filter_map(|(n, _)| n.check.map(|c| c.1))
        .max()
        .unwrap_or(0)
}

/// "2:57".
pub fn rc_fmt(s: u32) -> String {
    format!("{}:{:02}", s / 60, s % 60)
}

#[allow(clippy::too_many_arguments)]
fn node(
    id: &str,
    title: &str,
    brand: &str,
    files: &[&str],
    outcome: RcStatus,
    check: Option<(u32, u32)>,
    verdict: &str,
    evidence: Option<RcEvidence>,
) -> RcNode {
    RcNode {
        id: id.into(),
        title: title.into(),
        brand: brand.into(),
        files: files.iter().map(|f| f.to_string()).collect(),
        gate: RcGate::Or,
        verdict: verdict.into(),
        evidence,
        outcome,
        check,
        children: vec![],
    }
}

fn lines(v: &[&str]) -> Vec<String> {
    v.iter().map(|s| s.to_string()).collect()
}

/// The demo investigation: a failing checkout test; two hypotheses ruled out, a pair of changes
/// confirmed together under an AND gate, one never needed.
pub fn rc_demo() -> RcNode {
    let mut coupon = node(
        "coupon",
        "Discount maths changed",
        "github",
        &["discount.ts", "cart.ts"],
        RcStatus::Confirmed,
        Some((150, 205)),
        "Both changes below land in the same release and together give the off-by-a-cent totals.",
        None,
    );
    coupon.gate = RcGate::And;
    coupon.children = vec![
        node(
            "round",
            "Rounding moved earlier",
            "typescript",
            &["discount.ts"],
            RcStatus::Confirmed,
            Some((160, 185)),
            "Rounding now happens before tax, so 4 of the 12 baskets come out one cent low.",
            Some(RcEvidence::Diff(
                "src/discount.ts".into(),
                lines(&[
                    "- const net = round(subtotal * (1 - rate))",
                    "+ const net = round(subtotal) * (1 - rate)",
                    "  return net + tax(net)",
                ]),
            )),
        ),
        node(
            "fixture",
            "Fixture prices not updated",
            "postgresql",
            &["baskets.json"],
            RcStatus::Confirmed,
            Some((185, 200)),
            "The expected totals were written before the change and never regenerated.",
            Some(RcEvidence::Tool(
                "git.log baskets.json".into(),
                lines(&[
                    "last change: 41 days ago",
                    "discount.ts changed: 2 days ago",
                ]),
            )),
        ),
    ];
    let mut f = node(
        "fail",
        "Checkout test failed",
        "vercel",
        &["checkout.test.ts"],
        RcStatus::Confirmed,
        None,
        "4 of 12 basket totals are one cent off.",
        None,
    );
    f.children = vec![
        node(
            "payments",
            "Payment API errors",
            "stripe",
            &["payments.ts"],
            RcStatus::RuledOut,
            Some((20, 70)),
            "No failed calls in the window – every charge request came back 200.",
            Some(RcEvidence::Tool(
                "stripe.requests.list".into(),
                lines(&["window: 14:02 – 14:17", "requests: 48 · errors: 0"]),
            )),
        ),
        coupon,
        node(
            "network",
            "Flaky network",
            "cloudflare",
            &["ci.yml"],
            RcStatus::RuledOut,
            Some((70, 150)),
            "The same 4 baskets fail on every rerun, so it is not timing.",
            Some(RcEvidence::Tool(
                "ci.rerun ×3".into(),
                lines(&["run 1: 4 failed", "run 2: 4 failed", "run 3: 4 failed"]),
            )),
        ),
        node(
            "cache",
            "Stale price cache",
            "redis",
            &["prices.ts"],
            RcStatus::Open,
            None,
            "",
            None,
        ),
    ];
    f
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_walks() {
        let f = rc_demo();
        assert_eq!(rc_flat(&f).len(), 6);
        assert_eq!(rc_counts(&f, 0), (6, 0, 0));
        assert_eq!(rc_status(&f.children[0], 30), RcStatus::Checking);
        assert_eq!(rc_counts(&f, 151), (6, 2, 0));
        assert!(!rc_found(&f, 190));
        assert!(rc_path(&f, 190).is_empty());
        let end = rc_len(&f);
        assert_eq!(end, 205);
        assert!(rc_found(&f, end));
        assert_eq!(rc_counts(&f, end), (6, 2, 3));
        assert_eq!(
            rc_roots(&f, end),
            vec!["round".to_string(), "fixture".to_string()]
        );
        assert_eq!(rc_path(&f, end).len(), 3);
        assert_eq!(rc_status(&f.children[3], end), RcStatus::Open);
        assert_eq!(rc_fmt(177), "2:57");
    }
}
