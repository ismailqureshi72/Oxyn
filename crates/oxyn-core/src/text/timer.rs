//! `elapsed-timer` (`docs/specs/elapsed-timer.md`): formats and the per-state view.

use std::time::Duration;

use crate::{
    VerbPack,
    state::AgentState,
    verbs::{self, format_units},
};

/// The four free timer variants.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TimerVariant {
    /// `42s · 1m 05s · 1h 02m 33s`.
    #[default]
    Units,
    /// `0:42 · 12:05 · 1:02:33` in a reserved slot.
    Clock,
    /// Only the completion phrase, no live ticking.
    Summary,
    /// `1:12 · 18.2k tokens · 42 tok/s · $0.07`, each segment optional.
    Meter,
}

impl TimerVariant {
    /// Parses a variant id (`units` or `elapsed-timer/units`).
    pub fn from_id(id: &str) -> Option<Self> {
        Some(match id.strip_prefix("elapsed-timer/").unwrap_or(id) {
            "units" => Self::Units,
            "clock" => Self::Clock,
            "summary" => Self::Summary,
            "meter" => Self::Meter,
            _ => return None,
        })
    }
}

/// Time formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TimerFormat {
    /// `42s`, `1m 05s`, `1h 02m 33s`, `1d 03h`.
    Units,
    /// `0:42`, `12:05`, `1:02:33`, `27:02:33`.
    Clock,
}

/// `0:42 · 12:05 · 1:02:33 · 27:02:33`.
pub fn format_clock(d: Duration) -> String {
    let s = d.as_secs();
    let (h, m, sec) = (s / 3600, s / 60 % 60, s % 60);
    if h > 0 {
        format!("{h}:{m:02}:{sec:02}")
    } else {
        format!("{m}:{sec:02}")
    }
}

/// `842 · 18.2k · 182k · 1.24M`, `~` when estimated.
pub fn format_tokens(n: u64, estimated: bool) -> String {
    let body = match n {
        0..1_000 => n.to_string(),
        1_000..100_000 => trim(format!("{:.1}", n as f64 / 1e3)) + "k",
        100_000..1_000_000 => format!("{}k", n / 1_000),
        _ => trim(format!("{:.2}", n as f64 / 1e6)) + "M",
    };
    if estimated { format!("~{body}") } else { body }
}

fn trim(s: String) -> String {
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// `7.4 tok/s` under 10, `42 tok/s` above.
pub fn format_rate(tok_per_s: f32) -> String {
    if tok_per_s < 10.0 {
        format!("{tok_per_s:.1} tok/s")
    } else {
        format!("{} tok/s", tok_per_s.round() as u64)
    }
}

/// `$0.004` under a cent, else `$0.46`, `$12.30`; `est.` prefix when estimated. USD only here;
/// locale currency formatting is the host's job. Oxyn never ships prices.
pub fn format_cost(usd: f64, estimated: bool) -> String {
    let body = if usd < 0.01 {
        format!("${usd:.3}")
    } else {
        format!("${usd:.2}")
    };
    if estimated {
        format!("est. {body}")
    } else {
        body
    }
}

/// Counters beside the timer, all optional (the app supplies them).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RunMeter {
    /// Tokens so far, and whether the count is an estimate.
    pub tokens: Option<(u64, bool)>,
    /// Output tokens per second (shown only while streaming).
    pub tok_per_s: Option<f32>,
    /// Cost in USD from the app's price table, and whether it is an estimate.
    pub cost_usd: Option<(f64, bool)>,
}

/// What the timer shows.
#[derive(Clone, Debug, PartialEq)]
pub struct TimerView {
    /// The text, or empty when hidden (under 1 s, or idle).
    pub text: String,
    /// A template of the widest value of the current magnitude (`0:00`, `00:00`, `0:00:00`) for
    /// `Width::ReserveMagnitude`; tabular digits make its char count the slot width.
    pub reserve: String,
    /// Waiting on the user: digits frozen, ⏸ shown, secondary tone.
    pub paused: bool,
    /// The run has ended: this is the summary phrase.
    pub settled: bool,
}

fn magnitude_template(d: Duration, format: TimerFormat) -> String {
    let s = d.as_secs();
    match format {
        TimerFormat::Clock => match s {
            0..600 => "0:00".into(),
            600..3_600 => "00:00".into(),
            3_600..36_000 => "0:00:00".into(),
            _ => "00:00:00".into(),
        },
        TimerFormat::Units => match s {
            0..10 => "0s".into(),
            10..60 => "00s".into(),
            60..600 => "0m 00s".into(),
            600..3_600 => "00m 00s".into(),
            3_600..86_400 => "00h 00m 00s".into(),
            _ => "0d 00h".into(),
        },
    }
}

/// The timer for `state`, given the run's working time and the meter.
pub fn timer_view(
    variant: TimerVariant,
    state: &AgentState,
    elapsed: Duration,
    meter: &RunMeter,
    pack: &VerbPack,
) -> TimerView {
    let hidden = TimerView {
        text: String::new(),
        reserve: String::new(),
        paused: false,
        settled: false,
    };
    if let Some(s) = verbs::summary(state, pack) {
        let mut text = match state {
            AgentState::Error { .. } if elapsed >= Duration::from_secs(1) => {
                format!("Stopped after {}", verbs::format_summary(elapsed))
            }
            _ => s,
        };
        if let AgentState::Done { waited, .. } = state
            && *waited >= Duration::from_secs(5)
        {
            text.push_str(&format!(" · waited {}", verbs::format_summary(*waited)));
        }
        if variant == TimerVariant::Meter {
            for seg in meter_segments(meter, false) {
                text.push_str(" · ");
                text.push_str(&seg);
            }
        }
        return TimerView {
            text,
            reserve: String::new(),
            paused: false,
            settled: true,
        };
    }
    let live = state.is_busy() || matches!(state, AgentState::AwaitingApproval { .. });
    if !live || elapsed < Duration::from_secs(1) || variant == TimerVariant::Summary {
        return hidden;
    }
    let format = if variant == TimerVariant::Units {
        TimerFormat::Units
    } else {
        TimerFormat::Clock
    };
    let mut text = match format {
        TimerFormat::Units => format_units(elapsed),
        TimerFormat::Clock => format_clock(elapsed),
    };
    let reserve = magnitude_template(elapsed, format);
    if variant == TimerVariant::Meter {
        let streaming = matches!(state, AgentState::Streaming);
        for seg in meter_segments(meter, streaming) {
            text.push_str(" · ");
            text.push_str(&seg);
        }
    }
    TimerView {
        text,
        reserve,
        paused: matches!(state, AgentState::AwaitingApproval { .. }),
        settled: false,
    }
}

fn meter_segments(m: &RunMeter, streaming: bool) -> Vec<String> {
    let mut out = Vec::new();
    if let Some((n, est)) = m.tokens {
        out.push(format!("{} tokens", format_tokens(n, est)));
    }
    if streaming && let Some(r) = m.tok_per_s {
        out.push(format_rate(r));
    }
    if let Some((usd, est)) = m.cost_usd {
        out.push(format_cost(usd, est));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(format_clock(Duration::from_secs(42)), "0:42");
        assert_eq!(format_clock(Duration::from_secs(725)), "12:05");
        assert_eq!(format_clock(Duration::from_secs(3753)), "1:02:33");
        assert_eq!(format_clock(Duration::from_secs(97_353)), "27:02:33");
        assert_eq!(format_tokens(842, false), "842");
        assert_eq!(format_tokens(18_200, false), "18.2k");
        assert_eq!(format_tokens(182_000, true), "~182k");
        assert_eq!(format_tokens(1_240_000, false), "1.24M");
        assert_eq!(format_rate(7.42), "7.4 tok/s");
        assert_eq!(format_rate(42.3), "42 tok/s");
        assert_eq!(format_cost(0.004, false), "$0.004");
        assert_eq!(format_cost(12.3, true), "est. $12.30");
    }

    #[test]
    fn hidden_under_a_second_and_paused_while_waiting() {
        let p = VerbPack::neutral();
        let th = AgentState::Thinking { header: None };
        let m = RunMeter::default();
        assert!(
            timer_view(TimerVariant::Units, &th, Duration::from_millis(900), &m, p)
                .text
                .is_empty()
        );
        let v = timer_view(TimerVariant::Clock, &th, Duration::from_secs(42), &m, p);
        assert_eq!((v.text.as_str(), v.reserve.as_str()), ("0:42", "0:00"));
        let wait = AgentState::AwaitingApproval {
            request: crate::RequestRef {
                id: "r".into(),
                tool: None,
            },
        };
        assert!(timer_view(TimerVariant::Units, &wait, Duration::from_secs(5), &m, p).paused);
    }

    #[test]
    fn summary_and_meter() {
        let p = VerbPack::neutral();
        let done = AgentState::Done {
            elapsed: Duration::from_secs(72),
            waited: Duration::from_secs(64),
        };
        let m = RunMeter {
            tokens: Some((18_200, false)),
            tok_per_s: Some(42.0),
            cost_usd: Some((0.07, false)),
        };
        let v = timer_view(TimerVariant::Meter, &done, Duration::from_secs(72), &m, p);
        assert_eq!(
            v.text,
            "Worked for 1m 12s · waited 1m 04s · 18.2k tokens · $0.07"
        );
        assert!(v.settled);
        let th = AgentState::Streaming;
        let v = timer_view(TimerVariant::Meter, &th, Duration::from_secs(72), &m, p);
        assert_eq!(v.text, "1:12 · 18.2k tokens · 42 tok/s · $0.07");
        assert!(
            timer_view(TimerVariant::Summary, &th, Duration::from_secs(72), &m, p)
                .text
                .is_empty()
        );
    }
}
