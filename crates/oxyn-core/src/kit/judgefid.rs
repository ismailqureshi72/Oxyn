//! affectUI fidelity pass, family B part 9 (plan #32): the **Judge Bracket** model. Contenders
//! (2, 4, 8 or 16, in seed order, neighbours meet first) go through a single-elimination bracket;
//! a judge model compares each match's two answers and records a winner, a reason and scores.
//! Matches are judged round by round; the champion's way through is the winner path. Behaviour
//! only: the views live in `oxyn-dioxus` / `oxyn-gpui` (`kit/judgefid.rs`).

/// One contender.
#[derive(Clone, Debug, PartialEq)]
pub struct JbContender {
    /// Stable id.
    pub id: String,
    /// Short name of the answer ("Idempotency key").
    pub name: String,
    /// Brand id of the model that wrote it.
    pub brand: String,
    /// Meta line: model, lines changed, files, tests.
    pub meta: String,
    /// The answer in a sentence.
    pub output: String,
    /// One line of its code.
    pub code: String,
}

/// A judged match.
#[derive(Clone, Debug, PartialEq)]
pub struct JbVerdict {
    /// Round (0 = first).
    pub round: usize,
    /// Match index in the round.
    pub index: usize,
    /// 0 = the upper slot won, 1 = the lower.
    pub winner: usize,
    /// The judge's reason.
    pub reason: String,
    /// Scores (upper, lower), 0–10.
    pub scores: (f32, f32),
}

/// Number of rounds for `n` contenders (n a power of two).
pub fn jb_rounds(n: usize) -> usize {
    n.max(2).next_power_of_two().trailing_zeros() as usize
}

/// Matches in round `r`.
pub fn jb_matches(n: usize, r: usize) -> usize {
    (n.max(2).next_power_of_two() >> (r + 1)).max(1)
}

/// The verdict of a match, if judged.
pub fn jb_verdict(v: &[JbVerdict], r: usize, m: usize) -> Option<&JbVerdict> {
    v.iter().find(|x| x.round == r && x.index == m)
}

/// The contenders (indices) in a match; `None` where the feeding match is undecided.
pub fn jb_slots(n: usize, v: &[JbVerdict], r: usize, m: usize) -> [Option<usize>; 2] {
    if r == 0 {
        let a = 2 * m;
        return [(a < n).then_some(a), (a + 1 < n).then_some(a + 1)];
    }
    [
        jb_winner(n, v, r - 1, 2 * m),
        jb_winner(n, v, r - 1, 2 * m + 1),
    ]
}

/// The winner (contender index) of a match.
pub fn jb_winner(n: usize, v: &[JbVerdict], r: usize, m: usize) -> Option<usize> {
    let w = jb_verdict(v, r, m)?.winner;
    jb_slots(n, v, r, m)[w.min(1)]
}

/// Every match in judging order: round by round, top to bottom.
pub fn jb_order(n: usize) -> Vec<(usize, usize)> {
    (0..jb_rounds(n))
        .flat_map(|r| (0..jb_matches(n, r)).map(move |m| (r, m)))
        .collect()
}

/// The match being judged now: the first undecided one whose two contenders are known.
pub fn jb_current(n: usize, v: &[JbVerdict]) -> Option<(usize, usize)> {
    jb_order(n).into_iter().find(|&(r, m)| {
        jb_verdict(v, r, m).is_none() && jb_slots(n, v, r, m).iter().all(Option::is_some)
    })
}

/// The champion, once the final is decided.
pub fn jb_champion(n: usize, v: &[JbVerdict]) -> Option<usize> {
    let r = jb_rounds(n);
    jb_winner(n, v, r - 1, 0)
}

/// Is contender `c` still on the winner path through match (r, m)? (true for every match the
/// champion won).
pub fn jb_on_path(n: usize, v: &[JbVerdict], r: usize, m: usize) -> bool {
    jb_champion(n, v).is_some_and(|c| jb_winner(n, v, r, m) == Some(c))
}

/// "Quarterfinal", "Semifinal", "Final", "Round of 16".
pub fn jb_round_name(n: usize, r: usize) -> String {
    match jb_rounds(n) - r {
        1 => "Final".into(),
        2 => "Semifinal".into(),
        3 => "Quarterfinal".into(),
        k => format!("Round of {}", 1usize << k),
    }
}

/// "Semifinal 1", "Final".
pub fn jb_match_name(n: usize, r: usize, m: usize) -> String {
    let base = jb_round_name(n, r);
    if jb_matches(n, r) == 1 {
        base
    } else {
        format!("{base} {}", m + 1)
    }
}

/// Header progress: "Quarterfinal · 3/7".
pub fn jb_progress(n: usize, v: &[JbVerdict]) -> String {
    let total = n.max(2).next_power_of_two() - 1;
    let done = v.len().min(total);
    match jb_current(n, v) {
        Some((r, _)) => format!("{} · {}/{total}", jb_round_name(n, r), done + 1),
        None => format!("Decided · {done}/{total}"),
    }
}

fn ct(id: &str, name: &str, brand: &str, meta: &str, output: &str, code: &str) -> JbContender {
    JbContender {
        id: id.into(),
        name: name.into(),
        brand: brand.into(),
        meta: meta.into(),
        output: output.into(),
        code: code.into(),
    }
}

/// The demo field: eight fixes for a double-charge bug.
pub fn jb_demo() -> Vec<JbContender> {
    vec![
        ct(
            "a",
            "Idempotency key",
            "claude",
            "Opus · 18 added, 4 removed · 3 files · tests 42 of 42",
            "Every charge carries a key derived from the order; a retry with the same key returns the first result.",
            "charge(order, key: hash(order.id, attempt_group))",
        ),
        ct(
            "b",
            "Webhook dedupe",
            "openai",
            "GPT-5 · 31 added, 9 removed · 4 files · tests 40 of 42",
            "Drops repeated provider webhooks by event id before they reach the ledger.",
            "if seen(event.id) { return Ok(()) }",
        ),
        ct(
            "c",
            "Row lock",
            "gemini",
            "Gemini · 12 added, 2 removed · 2 files · tests 42 of 42",
            "Locks the order row while charging so a second attempt waits and then sees it paid.",
            "SELECT … FROM orders WHERE id = $1 FOR UPDATE",
        ),
        ct(
            "d",
            "Retry cap",
            "grok",
            "Grok · 6 added, 1 removed · 1 file · tests 38 of 42",
            "Stops the client after one retry; fewer doubles, but timeouts now fail the order.",
            "retries: 1",
        ),
        ct(
            "e",
            "Unique charge",
            "mistral",
            "Mistral · 9 added · 2 files · tests 42 of 42",
            "A unique index on (order, attempt group) makes the second insert fail cleanly.",
            "CREATE UNIQUE INDEX ON charges(order_id, group_id)",
        ),
        ct(
            "f",
            "Outbox",
            "deepseek",
            "DeepSeek · 64 added, 12 removed · 7 files · tests 41 of 42",
            "Writes the charge intent to an outbox and lets one worker send it.",
            "outbox.push(ChargeIntent { order, amount })",
        ),
        ct(
            "g",
            "Client token",
            "qwen",
            "Qwen · 22 added, 5 removed · 3 files · tests 42 of 42",
            "The checkout page mints a token once and the server refuses a reused one.",
            "require_fresh(token)?",
        ),
        ct(
            "h",
            "Saga refund",
            "kimi",
            "Kimi · 40 added, 3 removed · 5 files · tests 39 of 42",
            "Lets doubles happen and refunds the second charge within a minute.",
            "on_duplicate(|c| refund(c))",
        ),
    ]
}

fn vd(round: usize, index: usize, winner: usize, scores: (f32, f32), reason: &str) -> JbVerdict {
    JbVerdict {
        round,
        index,
        winner,
        reason: reason.into(),
        scores,
    }
}

/// The demo verdicts in judging order (7 for 8 contenders).
pub fn jb_demo_verdicts() -> Vec<JbVerdict> {
    vec![
        vd(
            0,
            0,
            0,
            (9.1, 7.4),
            "Fixes the cause at the point of charge; the dedupe only covers webhooks.",
        ),
        vd(
            0,
            1,
            0,
            (8.0, 5.2),
            "Both stop the double, but the cap turns slow networks into failed orders.",
        ),
        vd(
            0,
            2,
            0,
            (8.6, 7.1),
            "Same guarantee with a fraction of the moving parts.",
        ),
        vd(
            0,
            3,
            0,
            (8.3, 6.0),
            "Prevents the double instead of cleaning up after it.",
        ),
        vd(
            1,
            0,
            0,
            (9.2, 8.0),
            "Works across instances without holding a lock during the provider call.",
        ),
        vd(
            1,
            1,
            1,
            (7.9, 8.4),
            "The token also covers double clicks before the server sees anything.",
        ),
        vd(
            2,
            0,
            0,
            (9.4, 8.1),
            "Smallest change that closes every path to a second charge, with full tests.",
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bracket_walks() {
        let n = 8;
        assert_eq!(jb_rounds(n), 3);
        assert_eq!(jb_matches(n, 0), 4);
        assert_eq!(jb_matches(n, 2), 1);
        assert_eq!(jb_order(n).len(), 7);
        let all = jb_demo_verdicts();
        let v = &all[..2];
        assert_eq!(jb_slots(n, v, 1, 0), [Some(0), Some(2)]);
        assert_eq!(jb_slots(n, v, 1, 1), [None, None]);
        assert_eq!(jb_current(n, v), Some((0, 2)));
        assert_eq!(jb_progress(n, v), "Quarterfinal · 3/7");
        assert_eq!(jb_champion(n, v), None);
        assert_eq!(jb_winner(n, &all, 1, 1), Some(6));
        assert_eq!(jb_champion(n, &all), Some(0));
        assert!(jb_on_path(n, &all, 0, 0));
        assert!(!jb_on_path(n, &all, 0, 1));
        assert_eq!(jb_current(n, &all), None);
        assert_eq!(jb_progress(n, &all), "Decided · 7/7");
        assert_eq!(jb_match_name(n, 1, 0), "Semifinal 1");
        assert_eq!(jb_match_name(n, 2, 0), "Final");
        assert_eq!(jb_round_name(16, 0), "Round of 16");
        assert_eq!(jb_rounds(2), 1);
    }
}
