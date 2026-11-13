//! Status text: text shimmer, verb cycler, elapsed timer, text reveal.

use std::time::Duration;

use dioxus::prelude::*;
use oxyn_core::{
    ActivityKind, AgentEvent, AgentMachine, AgentState, ClockMs, ToolRef, text::RunMeter,
};
use oxyn_dioxus::{ElapsedTimer, TextReveal, TextShimmer, VerbCycler};

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, BASE, Demo, Vals, data};

/// The scripted run loops every 14 s: submit, think, read a file, think, stream, finish.
const LOOP: u64 = 14_000;

const ANSWER: &str = "The token refresh races the request queue. I moved the refresh behind a \
single-flight lock in auth.rs, so concurrent calls now wait for one refresh instead of firing \
three. Tests pass locally.";

fn t_of(now: ClockMs) -> u64 {
    now.0.saturating_sub(BASE.0) % LOOP
}

/// The agent state and working time `t` ms into the scripted run.
fn run_at(t: u64) -> (AgentState, Duration) {
    let start = 2_000_000u64;
    let mut m = AgentMachine::default();
    m.apply(AgentEvent::Submit, ClockMs(start));
    m.apply(AgentEvent::Thinking { header: None }, ClockMs(start + 400));
    let read = ToolRef {
        id: "t1".into(),
        name: "read".into(),
        activity: ActivityKind::Read,
        target: Some("auth.rs".into()),
    };
    let mut events: Vec<(u64, AgentEvent)> = vec![
        (1_800, AgentEvent::ToolStarted(read)),
        (3_400, AgentEvent::ToolFinished { id: "t1".into() }),
        (3_500, AgentEvent::Thinking { header: None }),
    ];
    for k in 0..30 {
        events.push((5_200 + k * 160, AgentEvent::TextDelta));
    }
    events.push((10_200, AgentEvent::Finished));
    for (at, e) in events {
        if at <= t {
            m.apply(e, ClockMs(start + at));
        }
    }
    let now = ClockMs(start + t);
    let s = m.tick(now).clone();
    (s, m.elapsed(now))
}

fn shimmer(v: &Vals, now: ClockMs) -> Element {
    rsx! {
        TextShimmer {
            text: v.string("text"),
            state: data::state(v.s("state")),
            now,
            variant: v.string("variant"),
            size: v.f("size") as f32,
            reduced: v.b("reduced"),
        }
    }
}

fn shimmer_code(v: &Vals) -> String {
    Code::new("TextShimmer")
        .raw("text", q(v.s("text")))
        .raw("state", "agent.state() // stops on done, dims when stalled")
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "sweep")
        .raw("size", format!("{}.0", v.s("size")))
        .b("reduced", v.b("reduced"))
        .done()
}

fn cycler(v: &Vals, now: ClockMs) -> Element {
    let state = if v.s("state") == "script" {
        run_at(t_of(now)).0
    } else {
        data::state(v.s("state"))
    };
    rsx! {
        div { class: "oxd-line",
            VerbCycler {
                state,
                now,
                variant: v.string("variant"),
                pack: v.string("pack"),
                seed: 7,
                size: v.f("size") as f32,
                reduced: v.b("reduced"),
            }
        }
    }
}

fn cycler_code(v: &Vals) -> String {
    Code::new("VerbCycler")
        .raw("state", "agent.state()")
        .raw("now", "clock.now()")
        .s("variant", v.s("variant"), "fade")
        .s("pack", v.s("pack"), "neutral")
        .raw("seed", "run_id_hash // same run, same verb order")
        .raw("size", format!("{}.0", v.s("size")))
        .b("reduced", v.b("reduced"))
        .done()
}

fn timer(v: &Vals, now: ClockMs) -> Element {
    let t = t_of(now);
    let (state, elapsed) = run_at(t);
    let meter = RunMeter {
        tokens: Some((t / 5 + 14_000, false)),
        tok_per_s: Some(41.6),
        cost_usd: Some((0.07 + t as f64 / 400_000.0, false)),
    };
    rsx! {
        div { class: "oxd-line",
            VerbCycler { state: state.clone(), now, pack: v.string("pack"), seed: 7, size: v.f("size") as f32 }
            span { class: "oxd-dot", "·" }
            ElapsedTimer { state, elapsed, variant: v.string("variant"), pack: v.string("pack"), meter, size: v.f("size") as f32 }
        }
    }
}

fn timer_code(v: &Vals) -> String {
    let mut c = Code::new("ElapsedTimer")
        .raw("state", "machine.state().clone()")
        .raw(
            "elapsed",
            "machine.elapsed(clock.now()) // excludes time waiting for you",
        )
        .s("variant", v.s("variant"), "units")
        .s("pack", v.s("pack"), "neutral");
    if v.s("variant") == "meter" {
        c = c.raw(
            "meter",
            "RunMeter { tokens: Some((14_200, false)), tok_per_s: Some(41.6), cost_usd: Some((0.07, false)) }",
        );
    }
    c.raw("size", format!("{}.0", v.s("size"))).done()
}

fn reveal(v: &Vals, now: ClockMs) -> Element {
    let t = t_of(now);
    let cps = v.f("speed");
    let n = ((t.saturating_sub(800)) as f64 * cps / 1000.0) as usize;
    let text: String = ANSWER.chars().take(n).collect();
    let ended = n >= ANSWER.chars().count();
    let caret = if v.s("caret") == "none" {
        String::new()
    } else {
        v.string("caret")
    };
    rsx! {
        div { class: "oxd-msgbox",
            div { class: "oxd-you", "Why does login fail when two tabs refresh at once?" }
            div { class: "oxd-bot",
                TextReveal { text, now, ended, interrupted: v.b("interrupted") && !ended, variant: v.string("variant"), caret, reduced: v.b("reduced") }
            }
        }
    }
}

fn reveal_code(v: &Vals) -> String {
    let caret = if v.s("caret") == "none" {
        ""
    } else {
        v.s("caret")
    };
    Code::new("TextReveal")
        .raw(
            "text",
            "answer() // everything received so far, append-only",
        )
        .raw("now", "clock.now()")
        .raw("ended", "stream.is_done()")
        .s("variant", v.s("variant"), "fade-token")
        .s("caret", caret, "bar")
        .b("reduced", v.b("reduced"))
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
            slug: "text-shimmer",
            name: "Text shimmer",
            category: "status-text",
            summary: "A label with light passing over it while the agent works: a sweep, a calm band, a pulse or an ASCII scan; it stops when the run ends and dims when it stalls.",
            dioxus: "TextShimmer",
            gpui: "TextShimmerEl",
            spec: "text-shimmer",
            source: "status_text.rs",
            controls: vec![
                txt("text", "Text", "Thinking through the auth flow"),
                sel(
                    "variant",
                    "Variant",
                    &["sweep", "calm", "pulse", "ascii"],
                    "sweep",
                ),
                sel("state", "Agent state", &data::STATES, "thinking"),
                rng("size", "Size", 11.0, 28.0, 1.0, "18"),
                tog("reduced", "Reduced motion", false),
            ],
            variants: vec![
                var("sweep", "A bright band sweeps across.", &[]),
                var("calm", "A slow, wide band.", &[("variant", "calm")]),
                var(
                    "pulse",
                    "The whole label breathes.",
                    &[("variant", "pulse")],
                ),
                var(
                    "ascii",
                    "Characters scan through weights.",
                    &[("variant", "ascii")],
                ),
            ],
            render: shimmer,
            code: shimmer_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "The shimmer is purely visual; the text node is real text and is the accessible name.",
                "Nothing is announced by the shimmer; reduced draws plain text at full colour.",
            ],
            responsive: vec![
                "line_px tells it the measured width; otherwise it estimates from the text.",
            ],
            height: 140,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "verb-cycler",
            name: "Verb cycler",
            category: "status-text",
            summary: "The verb that says what the agent is doing, rotating through a verb pack while it works and settling into the past tense when it finishes.",
            dioxus: "VerbCycler",
            gpui: "VerbCyclerEl",
            spec: "verb-cycler",
            source: "status_text.rs",
            controls: vec![
                sel(
                    "state",
                    "Agent state",
                    &[
                        "script",
                        "thinking",
                        "tool-running",
                        "streaming",
                        "stalled",
                        "done",
                        "error",
                    ],
                    "script",
                ),
                sel(
                    "variant",
                    "Variant",
                    &["fade", "slide", "blur-swap", "ellipsis", "static"],
                    "fade",
                ),
                sel(
                    "pack",
                    "Verb pack",
                    &["neutral", "coder", "terse", "crm"],
                    "coder",
                ),
                rng("size", "Size", 11.0, 28.0, 1.0, "18"),
                tog("reduced", "Reduced motion", false),
            ],
            variants: vec![
                var("fade", "Crossfade between verbs.", &[]),
                var("slide", "Verbs slide up.", &[("variant", "slide")]),
                var(
                    "blur-swap",
                    "Blur out, blur in.",
                    &[("variant", "blur-swap"), ("pack", "neutral")],
                ),
                var(
                    "ellipsis",
                    "Terse with growing dots.",
                    &[("variant", "ellipsis"), ("pack", "terse")],
                ),
            ],
            render: cycler,
            code: cycler_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "Flavour verbs are never announced; only activity changes reach the live region.",
                "Noise glyphs and peeks are aria-hidden; reduced uses 80 ms crossfades and doubles the dwell.",
            ],
            responsive: vec![
                "Verb packs hold a fixed set of verbs per state, so the width it needs is predictable.",
            ],
            height: 140,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "elapsed-timer",
            name: "Elapsed timer",
            category: "status-text",
            summary: "How long the agent has worked, without the time it spent waiting for you: units, a clock, a past-tense summary or a meter with tokens, speed and cost.",
            dioxus: "ElapsedTimer",
            gpui: "ElapsedTimerEl",
            spec: "elapsed-timer",
            source: "status_text.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["units", "clock", "summary", "meter"],
                    "units",
                ),
                sel(
                    "pack",
                    "Verb pack",
                    &["neutral", "coder", "terse", "crm"],
                    "neutral",
                ),
                rng("size", "Size", 11.0, 24.0, 1.0, "15"),
            ],
            variants: vec![
                var("units", "12s, 1m 04s.", &[]),
                var("clock", "0:12.", &[("variant", "clock")]),
                var("summary", "Worked for 12s.", &[("variant", "summary")]),
                var("meter", "Tokens, speed and cost.", &[("variant", "meter")]),
            ],
            render: timer,
            code: timer_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "Never announced per second; the label reads in spelled units (\"Elapsed 1 minute 12 seconds\").",
                "Time spent waiting for you is left out, and said so (\"paused while waiting for you\").",
            ],
            responsive: vec![
                "units and clock fit a status row; meter needs room for three counters.",
            ],
            height: 140,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "text-reveal",
            name: "Text reveal",
            category: "status-text",
            summary: "Streamed text arriving: tokens fade in, words un-blur, a typewriter or instant, with a bar, block or underscore caret that leaves when the stream ends.",
            dioxus: "TextReveal",
            gpui: "TextRevealEl",
            spec: "text-reveal",
            source: "status_text.rs",
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &[
                        "fade-token",
                        "word-blur",
                        "typewriter",
                        "line-commit",
                        "instant",
                    ],
                    "fade-token",
                ),
                sel(
                    "caret",
                    "Caret",
                    &["bar", "block", "underscore", "none"],
                    "bar",
                ),
                rng("speed", "Characters / s", 20.0, 200.0, 10.0, "70"),
                tog("interrupted", "Stopped by the user", false),
                tog("reduced", "Reduced motion", false),
            ],
            variants: vec![
                var("fade-token", "Tokens fade in.", &[]),
                var("word-blur", "Words un-blur.", &[("variant", "word-blur")]),
                var(
                    "typewriter",
                    "Character by character, block caret.",
                    &[("variant", "typewriter"), ("caret", "block")],
                ),
                var(
                    "instant",
                    "No animation.",
                    &[("variant", "instant"), ("caret", "underscore")],
                ),
            ],
            render: reveal,
            code: reveal_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "Real text nodes; screen readers are never fed tokens (no aria-live on the message).",
                "The caret and effect spans are aria-hidden; reduced motion keeps the caret static.",
            ],
            responsive: vec!["Wraps like body text; the caret follows the last line."],
            height: 220,
            align: Align::Center,
            live: true,
        },
    ]
}
