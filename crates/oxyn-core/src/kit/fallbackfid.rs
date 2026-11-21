//! affectUI fidelity pass, family B part 8 (plan #32): the **Fallback Chain** model. One request
//! goes down a list of models in fallback order: each attempt is tried in turn, a failure (rate
//! limit, timeout, error) breaks its branch and hands on to the next, the first that answers runs
//! on to the response, and the ones after it are never tried. Behaviour only: the views live in
//! `oxyn-dioxus` / `oxyn-gpui` (`kit/fallbackfid.rs`).

/// How an attempt ends when it is tried.
#[derive(Clone, Debug, PartialEq)]
pub enum FbOutcome {
    /// It answers.
    Ok,
    /// HTTP 429 or a quota.
    RateLimited,
    /// No answer in time.
    Timeout,
    /// Any other error, with its short message.
    Error(String),
}

impl FbOutcome {
    /// The short reason drawn on a broken branch.
    pub fn reason(&self) -> String {
        match self {
            Self::Ok => "Answered".into(),
            Self::RateLimited => "Rate limited · 429".into(),
            Self::Timeout => "Timed out".into(),
            Self::Error(e) => e.clone(),
        }
    }

    /// The status code line of the details card: "429 – too many requests".
    pub fn code(&self) -> String {
        match self {
            Self::Ok => "200 – ok".into(),
            Self::RateLimited => "429 – too many requests".into(),
            Self::Timeout => "504 – no reply before the budget ran out".into(),
            Self::Error(e) => e.replace(" · ", " – "),
        }
    }
}

/// One model in the chain.
#[derive(Clone, Debug, PartialEq)]
pub struct FbAttempt {
    /// Model name.
    pub model: String,
    /// Provider brand id (drawn as its coloured mark).
    pub brand: String,
    /// How it ends if tried.
    pub outcome: FbOutcome,
    /// How long the try takes, ms.
    pub ms: u64,
    /// Where it is served from ("Bedrock · us-east-1").
    pub provider: String,
    /// What the try costs, US dollars (0 for a failure that was not billed).
    pub cost: f64,
    /// Tokens in the answer (an answering attempt).
    pub tokens: u32,
    /// The list price, "$3 / $15 per 1M".
    pub price: String,
    /// The time budget for one try, ms.
    pub budget_ms: u64,
}

/// Where an attempt stands at a moment.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FbState {
    /// Not reached yet.
    Waiting,
    /// Being tried now.
    Trying,
    /// Tried and failed.
    Failed,
    /// Tried and answered.
    Answered,
    /// Never tried – an earlier one answered.
    Skipped,
}

impl FbState {
    /// `waiting` / `trying` / `failed` / `answered` / `skipped`.
    pub fn id(self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::Trying => "trying",
            Self::Failed => "failed",
            Self::Answered => "answered",
            Self::Skipped => "skipped",
        }
    }
}

/// The index of the attempt that answers, if any.
pub fn fb_answer(a: &[FbAttempt]) -> Option<usize> {
    a.iter().position(|x| x.outcome == FbOutcome::Ok)
}

/// Every attempt's state `el` ms after the request left (`gap` ms of hand-over between tries).
/// Returns the states and whether the chain has finished.
pub fn fb_states(a: &[FbAttempt], el: u64, gap: u64) -> (Vec<FbState>, bool) {
    let mut out = vec![FbState::Waiting; a.len()];
    let mut t = 0u64;
    let mut done = false;
    for (i, x) in a.iter().enumerate() {
        if done {
            out[i] = FbState::Skipped;
            continue;
        }
        if el < t {
            break;
        }
        if el < t + x.ms {
            out[i] = FbState::Trying;
            return (out, false);
        }
        if x.outcome == FbOutcome::Ok {
            out[i] = FbState::Answered;
            done = true;
        } else {
            out[i] = FbState::Failed;
        }
        t += x.ms + gap;
    }
    let finished = done || out.iter().all(|s| *s == FbState::Failed);
    (out, finished)
}

/// How far the trying attempt is, 0–1, at `el` ms (`None` when none is trying).
pub fn fb_progress(a: &[FbAttempt], el: u64, gap: u64) -> Option<(usize, f64)> {
    let mut t = 0u64;
    for (i, x) in a.iter().enumerate() {
        if el < t {
            return None;
        }
        if el < t + x.ms {
            return Some((i, (el - t) as f64 / x.ms.max(1) as f64));
        }
        if x.outcome == FbOutcome::Ok {
            return None;
        }
        t += x.ms + gap;
    }
    None
}

/// Time on the clock at `el` (stops when the chain finishes) and the money spent so far:
/// finished tries in full, the one being tried pro rata.
pub fn fb_meter(a: &[FbAttempt], el: u64, gap: u64) -> (u64, f64) {
    let total = fb_total_ms(a, gap);
    let (s, _) = fb_states(a, el, gap);
    let mut cost: f64 = a
        .iter()
        .zip(&s)
        .filter(|(_, s)| matches!(s, FbState::Failed | FbState::Answered))
        .map(|(x, _)| x.cost)
        .sum();
    if let Some((i, p)) = fb_progress(a, el, gap) {
        cost += a[i].cost * p;
    }
    (el.min(total), cost)
}

/// "$0" / "$0.0027" / "$0.31".
pub fn fb_usd(v: f64) -> String {
    if v <= 0.0 {
        "$0".into()
    } else if v < 0.01 {
        format!("${v:.4}")
    } else {
        format!("${v:.2}")
    }
}

/// Seconds, compact: "1.6s", "12s".
pub fn fb_s(ms: u64) -> String {
    if ms < 10_000 {
        format!("{:.1}s", ms as f64 / 1000.0)
    } else {
        format!("{}s", ms / 1000)
    }
}

/// The header status: "Attempt 2 of 4", "Answered", "All failed".
pub fn fb_status(s: &[FbState]) -> (&'static str, String) {
    if let Some(i) = s.iter().position(|x| *x == FbState::Trying) {
        return ("live", format!("Attempt {} of {}", i + 1, s.len()));
    }
    if s.contains(&FbState::Answered) {
        return ("done", "Answered".into());
    }
    if !s.is_empty() && s.iter().all(|x| *x == FbState::Failed) {
        return ("failed", "All failed".into());
    }
    ("live", "Sending".into())
}

/// The settled states (as if all the time in the world had passed).
pub fn fb_final(a: &[FbAttempt]) -> Vec<FbState> {
    fb_states(a, u64::MAX / 4, 0).0
}

/// Total time until the answer (or the last failure), including hand-overs.
pub fn fb_total_ms(a: &[FbAttempt], gap: u64) -> u64 {
    let stop = fb_answer(a).map_or(a.len(), |i| i + 1);
    a.iter().take(stop).map(|x| x.ms).sum::<u64>() + gap * stop.saturating_sub(1) as u64
}

/// "820 ms" / "1.4 s" / "31 s".
pub fn fb_ms(ms: u64) -> String {
    if ms < 1000 {
        format!("{ms} ms")
    } else if ms < 10_000 {
        format!("{:.1} s", ms as f64 / 1000.0)
    } else {
        format!("{} s", ms / 1000)
    }
}

/// The headline: "Trying Sonnet 4.5", "Answered by Gemini 2.5 Flash · 2nd try", "Every model failed".
pub fn fb_headline(a: &[FbAttempt], s: &[FbState]) -> String {
    if let Some(i) = s.iter().position(|x| *x == FbState::Trying) {
        return format!("Trying {}", a[i].model);
    }
    if let Some(i) = s.iter().position(|x| *x == FbState::Answered) {
        let nth = match i + 1 {
            1 => "first try".to_string(),
            2 => "2nd try".to_string(),
            3 => "3rd try".to_string(),
            n => format!("{n}th try"),
        };
        return format!("Answered by {} · {nth}", a[i].model);
    }
    if !s.is_empty() && s.iter().all(|x| *x == FbState::Failed) {
        return "Every model failed".into();
    }
    "Waiting".into()
}

fn at(model: &str, brand: &str, outcome: FbOutcome, ms: u64) -> FbAttempt {
    let (provider, price, per_tok) = match brand {
        "claude" | "anthropic" => ("Bedrock · us-east-1", "$3 / $15 per 1M", 15.0),
        "openai" => ("Azure · eastus2", "$1.25 / $10 per 1M", 10.0),
        "gemini" => ("Vertex · europe-west4", "$1.25 / $10 per 1M", 10.0),
        "xai" | "grok" => ("xAI · direct", "$3 / $15 per 1M", 15.0),
        _ => ("Self-hosted · rack 2", "$0 / $0 per 1M", 0.0),
    };
    let tokens = if outcome == FbOutcome::Ok { 412 } else { 0 };
    // Input is billed on every try that got an answer; failures here are not billed.
    let cost = if outcome == FbOutcome::Ok {
        (tokens as f64 * per_tok + 600.0 * per_tok / 8.0) / 1_000_000.0
    } else {
        0.0
    };
    FbAttempt {
        model: model.into(),
        brand: brand.into(),
        outcome,
        ms,
        provider: provider.into(),
        cost,
        tokens,
        price: price.into(),
        budget_ms: 5000,
    }
}

/// The demo chain: the first model is rate limited, the second runs out of time, the third
/// answers and the fourth is never tried.
pub fn fb_demo() -> Vec<FbAttempt> {
    vec![
        at("Opus 4.1", "claude", FbOutcome::RateLimited, 1600),
        at("GPT-5", "openai", FbOutcome::Timeout, 2400),
        at("Gemini 2.5 Pro", "gemini", FbOutcome::Ok, 3100),
        at("Grok 4", "grok", FbOutcome::Ok, 2100),
    ]
}

/// A chain where every model fails.
pub fn fb_demo_all_fail() -> Vec<FbAttempt> {
    vec![
        at("Opus 4.1", "claude", FbOutcome::RateLimited, 900),
        at("GPT-5", "openai", FbOutcome::Timeout, 1800),
        at(
            "Gemini 2.5 Pro",
            "gemini",
            FbOutcome::Error("503 · overloaded".into()),
            700,
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meter_and_progress() {
        let a = fb_demo();
        assert_eq!(fb_progress(&a, 800, 200), Some((0, 0.5)));
        assert_eq!(fb_progress(&a, 1700, 200), None);
        let (t, c) = fb_meter(&a, 500, 200);
        assert_eq!((t, c), (500, 0.0));
        let (t, c) = fb_meter(&a, 60_000, 200);
        assert_eq!(t, 7500);
        assert!((c - a[2].cost).abs() < 1e-12 && c > 0.0);
        assert_eq!(fb_usd(0.0), "$0");
        assert_eq!(fb_usd(0.00271), "$0.0027");
        assert_eq!(fb_usd(0.314), "$0.31");
        assert_eq!(fb_s(1600), "1.6s");
        assert_eq!(fb_s(12_400), "12s");
        let (st, _) = fb_states(&a, 2000, 200);
        assert_eq!(fb_status(&st), ("live", "Attempt 2 of 4".to_string()));
        assert_eq!(fb_status(&fb_final(&a)).1, "Answered");
    }

    #[test]
    fn states_walk_the_chain() {
        let a = fb_demo();
        let (s, done) = fb_states(&a, 0, 200);
        assert_eq!(s[0], FbState::Trying);
        assert!(!done);
        // 1.6 s try + 200 ms hand-over: the second is being tried at 2 s.
        let (s, _) = fb_states(&a, 2000, 200);
        assert_eq!(s[..2], [FbState::Failed, FbState::Trying]);
        let (s, done) = fb_states(&a, 9000, 200);
        assert!(done);
        assert_eq!(
            s,
            vec![
                FbState::Failed,
                FbState::Failed,
                FbState::Answered,
                FbState::Skipped
            ]
        );
        assert_eq!(fb_final(&a), s);
        assert_eq!(fb_answer(&a), Some(2));
        assert_eq!(fb_total_ms(&a, 200), 7500);
        assert_eq!(fb_headline(&a, &s), "Answered by Gemini 2.5 Pro · 3rd try");
    }

    #[test]
    fn all_fail_and_formats() {
        let a = fb_demo_all_fail();
        let s = fb_final(&a);
        assert!(s.iter().all(|x| *x == FbState::Failed));
        assert_eq!(fb_answer(&a), None);
        assert_eq!(fb_headline(&a, &s), "Every model failed");
        assert_eq!(fb_total_ms(&a, 0), 3400);
        assert_eq!(fb_ms(820), "820 ms");
        assert_eq!(fb_ms(1440), "1.4 s");
        assert_eq!(fb_ms(31_000), "31 s");
        assert_eq!(a[2].outcome.reason(), "503 · overloaded");
        assert_eq!(a[2].outcome.code(), "503 – overloaded");
        assert_eq!(fb_status(&s), ("failed", "All failed".to_string()));
        assert_eq!(fb_headline(&a, &fb_states(&a, 10, 0).0), "Trying Opus 4.1");
    }
}
