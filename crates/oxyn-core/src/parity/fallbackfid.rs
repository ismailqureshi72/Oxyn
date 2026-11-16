//! affectUI fidelity pass, family B part 8 (plan #32): the **Fallback Chain** model. One request
//! goes down a list of models in fallback order: each attempt is tried in turn, a failure (rate
//! limit, timeout, error) breaks its branch and hands on to the next, the first that answers runs
//! on to the response, and the ones after it are never tried. Behaviour only: the views live in
//! `oxyn-dioxus` / `oxyn-gpui` (`parity/fallbackfid.rs`).

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
            Self::RateLimited => "429 · rate limited".into(),
            Self::Timeout => "Timed out".into(),
            Self::Error(e) => e.clone(),
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
    FbAttempt {
        model: model.into(),
        brand: brand.into(),
        outcome,
        ms,
    }
}

/// The demo chain: the first model is rate limited, the second answers, the third is never tried.
pub fn fb_demo() -> Vec<FbAttempt> {
    vec![
        at("Opus 4.1", "anthropic", FbOutcome::RateLimited, 900),
        at("GPT-5", "openai", FbOutcome::Ok, 1600),
        at("Gemini 2.5 Pro", "gemini", FbOutcome::Ok, 1400),
        at("Llama on-prem", "ollama", FbOutcome::Ok, 2100),
    ]
}

/// A chain where every model fails.
pub fn fb_demo_all_fail() -> Vec<FbAttempt> {
    vec![
        at("Opus 4.1", "anthropic", FbOutcome::RateLimited, 900),
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
    fn states_walk_the_chain() {
        let a = fb_demo();
        let (s, done) = fb_states(&a, 0, 200);
        assert_eq!(s[0], FbState::Trying);
        assert!(!done);
        // 900 ms try + 200 ms hand-over: the second is being tried at 1.2 s.
        let (s, _) = fb_states(&a, 1200, 200);
        assert_eq!(s[..2], [FbState::Failed, FbState::Trying]);
        let (s, done) = fb_states(&a, 3000, 200);
        assert!(done);
        assert_eq!(
            s,
            vec![
                FbState::Failed,
                FbState::Answered,
                FbState::Skipped,
                FbState::Skipped
            ]
        );
        assert_eq!(fb_final(&a), s);
        assert_eq!(fb_answer(&a), Some(1));
        assert_eq!(fb_total_ms(&a, 200), 2700);
        assert_eq!(fb_headline(&a, &s), "Answered by GPT-5 · 2nd try");
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
        assert_eq!(fb_headline(&a, &fb_states(&a, 10, 0).0), "Trying Opus 4.1");
    }
}
