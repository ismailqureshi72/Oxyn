//! Plan #28 parity demos, loaders and moving text: thinking loader, loading state, word reveal,
//! in-view title, text morph, action swap, shimmer label, slot text and streaming text.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::text::{ActionSwap, Phases, SlotText, Stream, SwapState, group},
};
use oxyn_dioxus::{
    ActionSwapButton, InViewTitle, LoadingState, SlotReels, Source, StreamingText, SweepText,
    TextMorph, ThinkingLoader, WordReveal,
};

use super::{Code, mk, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals};

/// Restarts a demo when its key changes, remembering the clock time it started at.
macro_rules! keyed {
    ($sig:ident, $key:expr, $build:expr) => {
        let key = $key;
        let mut $sig = use_signal(|| $build);
        let mut k = use_signal(|| key.clone());
        if k() != key {
            k.set(key.clone());
            $sig.set($build);
        }
    };
}

// ---------- thinking loader ----------

fn think(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveThink { size: v.string("size"), variant: v.string("variant"), looping: v.b("loop"), done: v.b("done"), interval: v.f("interval") as u64, now } }
}

#[component]
fn LiveThink(
    size: String,
    variant: String,
    looping: bool,
    done: bool,
    interval: u64,
    now: ClockMs,
) -> Element {
    keyed!(p, (looping, done, interval), {
        let mut p = Phases::standard(ClockMs(now.0.saturating_sub(if done { 4_200 } else { 0 })));
        p.looping = looping;
        p.interval_ms = interval;
        if done {
            p.done = Some(now);
        }
        p
    });
    rsx! {
        div { class: "oxd-w420", style: "display:flex;flex-direction:column;gap:12px;align-items:flex-start",
            ThinkingLoader { phases: p(), now, size, variant }
            div { style: "display:flex;gap:8px",
                button { class: "oxr-btn", r#type: "button", disabled: p().done.is_some(), onclick: move |_| p.write().done = Some(now), "Finish" }
                button { class: "oxr-btn", r#type: "button", onclick: move |_| { let mut w = p.write(); w.started = now; w.done = None; }, "Start over" }
            }
        }
    }
}

fn think_code(v: &Vals) -> String {
    Code::new("ThinkingLoader")
        .pre("let mut phases = use_signal(|| Phases::standard(clock.now())); // phases.write().done = Some(now) settles it")
        .raw("phases", "phases()")
        .raw("now", "clock.now()")
        .s("size", &v.string("size"), "md")
        .s("variant", &v.string("variant"), "pill")
        .done()
}

// ---------- loading state ----------

fn loading(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveLoading { variant: v.string("variant"), label: v.string("label"), elapsed: v.b("elapsed"), reduced: v.b("reduced"), now } }
}

#[component]
fn LiveLoading(
    variant: String,
    label: String,
    elapsed: bool,
    reduced: bool,
    now: ClockMs,
) -> Element {
    keyed!(t0, variant.clone(), now);
    rsx! { LoadingState { label, variant, started: t0(), now, elapsed, reduced } }
}

fn loading_code(v: &Vals) -> String {
    Code::new("LoadingState")
        .raw("label", q(&v.string("label")))
        .s("variant", &v.string("variant"), "drive")
        .raw("started", "started")
        .raw("now", "clock.now()")
        .done()
}

// ---------- word reveal ----------

fn reveal(v: &Vals, _: ClockMs) -> Element {
    let text = if v.string("tag") == "p" {
        "Agents finish the work you start, then hand it back with every step written down so you can check it in a minute."
    } else {
        "Ship the work, keep the trail"
    };
    rsx! {
        LiveReveal { text: text.to_string(), by: v.string("by"), tag: v.string("tag"), stagger: v.f("stagger") as u32, delay: v.f("delay") as u32, once: v.b("once") }
    }
}

#[component]
fn LiveReveal(
    text: String,
    by: String,
    tag: String,
    stagger: u32,
    delay: u32,
    once: bool,
) -> Element {
    let mut shown = use_signal(|| true);
    let mut n = use_signal(|| 0u32);
    rsx! {
        div { class: "oxd-w520", style: "display:flex;flex-direction:column;gap:14px;align-items:flex-start",
            div { key: "{n}-{by}-{tag}-{stagger}-{delay}", WordReveal { text: text.clone(), by: by.clone(), tag: tag.clone(), stagger_ms: stagger, delay_ms: delay, shown: shown(), once } }
            div { style: "display:flex;gap:8px",
                button { class: "oxr-btn", r#type: "button", onclick: move |_| n += 1, "Replay" }
                button { class: "oxr-btn", r#type: "button", "aria-pressed": "{shown()}", onclick: move |_| shown.toggle(), if shown() { "Scroll out" } else { "Scroll in" } }
            }
        }
    }
}

fn reveal_code(v: &Vals) -> String {
    Code::new("WordReveal")
        .raw("text", q("Ship the work, keep the trail"))
        .s("by", &v.string("by"), "word")
        .s("tag", &v.string("tag"), "p")
        .raw("stagger_ms", format!("{}", v.f("stagger") as u32))
        .raw("shown", "in_view()")
        .done()
}

// ---------- in-view title ----------

fn title(v: &Vals, _: ClockMs) -> Element {
    rsx! { LiveTitle { variant: v.string("variant"), tag: v.string("tag"), once: v.b("once") } }
}

#[component]
fn LiveTitle(variant: String, tag: String, once: bool) -> Element {
    let mut shown = use_signal(|| true);
    let mut n = use_signal(|| 0u32);
    rsx! {
        div { class: "oxd-w520", style: "display:flex;flex-direction:column;gap:14px;align-items:flex-start",
            div { key: "{n}-{variant}-{tag}",
                InViewTitle { text: "Every run, start to finish".to_string(), variant: variant.clone(), tag: tag.clone(), lines: vec!["Every run,".to_string(), "start to finish".to_string()], shown: shown(), once }
            }
            div { style: "display:flex;gap:8px",
                button { class: "oxr-btn", r#type: "button", onclick: move |_| n += 1, "Replay" }
                button { class: "oxr-btn", r#type: "button", "aria-pressed": "{shown()}", onclick: move |_| shown.toggle(), if shown() { "Scroll out" } else { "Scroll in" } }
            }
        }
    }
}

fn title_code(v: &Vals) -> String {
    Code::new("InViewTitle")
        .raw("text", q("Every run, start to finish"))
        .s("variant", &v.string("variant"), "blur")
        .s("tag", &v.string("tag"), "h2")
        .raw("shown", "in_view() // e.g. 45 % visible")
        .raw("once", format!("{}", v.b("once")))
        .done()
}

// ---------- text morph ----------

const MORPHS: [&[&str]; 3] = [
    &["Follow", "Following"],
    &["Copy", "Copied", "Done"],
    &["Save draft", "Saving…", "Saved"],
];

fn morph(v: &Vals, now: ClockMs) -> Element {
    let set = match v.string("set").as_str() {
        "copy" => 1,
        "save" => 2,
        _ => 0,
    };
    rsx! { LiveMorph { set, auto: v.b("auto"), size: v.f("size") as u32, now } }
}

#[component]
fn LiveMorph(set: usize, auto: bool, size: u32, now: ClockMs) -> Element {
    let mut i = use_signal(|| 0usize);
    let words = MORPHS[set];
    let k = if auto {
        (now.0 / 1600) as usize % words.len()
    } else {
        i() % words.len()
    };
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:14px;align-items:center",
            span { style: "font-size:{size}px;font-weight:600", TextMorph { text: words[k].to_string() } }
            button { class: "oxr-btn", r#type: "button", disabled: auto, onclick: move |_| i += 1, "Next label" }
        }
    }
}

fn morph_code(_: &Vals) -> String {
    Code::new("TextMorph").raw("text", "label()").done()
}

// ---------- action swap ----------

fn swap_states(kind: &str) -> Vec<SwapState> {
    match kind {
        "copy" => vec![
            SwapState {
                icon: Some("copy".into()),
                ..SwapState::new("copy", "Copy link")
            },
            SwapState {
                icon: Some("check".into()),
                tone: "success".into(),
                hold_ms: Some(1500),
                ..SwapState::new("copied", "Copied")
            },
        ],
        "deploy" => vec![
            SwapState {
                tone: "accent".into(),
                icon: Some("bolt".into()),
                ..SwapState::new("deploy", "Deploy")
            },
            SwapState {
                tone: "success".into(),
                icon: Some("check".into()),
                pending: Some("Deploying…".into()),
                ..SwapState::new("live", "Live")
            },
            SwapState {
                tone: "quiet".into(),
                icon: Some("undo".into()),
                pending: Some("Rolling back…".into()),
                ..SwapState::new("rollback", "Roll back")
            },
        ],
        _ => vec![
            SwapState {
                tone: "accent".into(),
                icon: Some("plus".into()),
                ..SwapState::new("follow", "Follow")
            },
            SwapState {
                icon: Some("check".into()),
                ..SwapState::new("following", "Following")
            },
        ],
    }
}

fn swap(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveSwap { kind: v.string("kind"), size: v.string("size"), fail: v.b("fail"), asynch: v.b("async"), now } }
}

#[component]
fn LiveSwap(kind: String, size: String, fail: bool, asynch: bool, now: ClockMs) -> Element {
    keyed!(m, kind.clone(), ActionSwap::new(swap_states(&kind)));
    // The simulated action settles 900 ms after the press.
    let mut cur = m();
    if let oxyn_core::parity::text::SwapPhase::Working(s) = cur.phase
        && now.0.saturating_sub(s.0) >= 900
    {
        cur.settle(!fail, now);
    }
    cur.tick(now);
    let shown = cur.clone();
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:10px;align-items:center",
            ActionSwapButton { model: shown, size, on_press: move |_| { let mut c = cur.clone(); c.press(asynch, now); m.set(c) } }
            span { class: "oxr-dim oxr-small", "Enter or Space advances" }
        }
    }
}

fn swap_code(v: &Vals) -> String {
    Code::new("ActionSwapButton")
        .pre("let mut swap = use_signal(|| ActionSwap::new(states)); // tick(now) each frame; settle(ok, now) when the action ends")
        .raw("model", "swap()")
        .s("size", &v.string("size"), "md")
        .raw("on_press", "move |_| { swap.write().press(true, now); spawn(run_action()) }")
        .done()
}

// ---------- shimmer label ----------

fn shimmer(v: &Vals, now: ClockMs) -> Element {
    let labels = [
        "Reading the repository",
        "Planning the change",
        "Writing tests",
    ];
    let text = if v.b("rotate") {
        labels[(now.0 / 3000) as usize % 3]
    } else {
        labels[0]
    };
    let size = v.f("size") as u32;
    rsx! {
        span { style: "font-size:{size}px",
            SweepText { text: text.to_string(), active: v.b("active"), duration_ms: v.f("duration") as u32 }
        }
    }
}

fn shimmer_code(v: &Vals) -> String {
    Code::new("SweepText")
        .raw("text", q("Reading the repository"))
        .raw("active", format!("{}", v.b("active")))
        .raw("duration_ms", format!("{}", v.f("duration") as u32))
        .done()
}

// ---------- slot text ----------

fn slot(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveSlot { kind: v.string("kind"), spins: v.f("spins") as usize, stagger: v.f("stagger") as u64, size: v.f("size") as u32, announce: v.b("announce"), now } }
}

#[component]
fn LiveSlot(
    kind: String,
    spins: usize,
    stagger: u64,
    size: u32,
    announce: bool,
    now: ClockMs,
) -> Element {
    let start = match kind.as_str() {
        "money" => "$999".to_string(),
        "word" => "QUEUED".to_string(),
        _ => group(12_480),
    };
    keyed!(m, (kind.clone(), spins, stagger), {
        let mut s = SlotText::new(&start);
        s.spins = spins;
        s.stagger_ms = stagger;
        s.align_end = kind != "word";
        s
    });
    let mut n = use_signal(|| 0usize);
    let next = move |i: usize, kind: &str| -> String {
        match kind {
            "money" => ["$1,000", "$1,284", "$987", "$999"][i % 4].to_string(),
            "word" => ["RUNNING", "PASSED", "QUEUED"][i % 3].to_string(),
            _ => group([12_481, 13_007, 9_998, 12_480][i % 4]),
        }
    };
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:14px;align-items:center",
            span { style: "font-size:{size}px;font-weight:600;letter-spacing:-.01em", SlotReels { model: m(), now, announce } }
            button { class: "oxr-btn", r#type: "button", onclick: move |_| { let t = next(n(), &kind); n += 1; m.write().set(&t, now) }, "Change value" }
        }
    }
}

fn slot_code(v: &Vals) -> String {
    Code::new("SlotReels")
        .pre(format!(
            "let mut slot = use_signal(|| SlotText::new(&group(12_480))); // spins = {}",
            v.f("spins") as usize
        ))
        .raw("model", "slot()")
        .raw("now", "clock.now()")
        .b("announce", v.b("announce"))
        .done()
}

// ---------- streaming text ----------

const ANSWER: &str = "Mint chip is up 12% against last summer [1] with stronger weekend peaks, while search interest for gelato keeps climbing into July [2] . Basket data shows mint chip sells best next to waffle cones [3] , so a bundle is worth testing.";

fn stream(v: &Vals, now: ClockMs) -> Element {
    rsx! { LiveStream { wps: v.string("speed").parse().unwrap_or(12.0), follow: v.b("follow"), done: v.b("done"), now } }
}

#[component]
fn LiveStream(wps: f32, follow: bool, done: bool, now: ClockMs) -> Element {
    keyed!(
        s,
        (wps.to_bits(), done),
        Stream {
            text: ANSWER.into(),
            started: ClockMs(now.0.saturating_sub(if done { 600_000 } else { 0 })),
            wps
        }
    );
    let mut picked = use_signal(|| None::<String>);
    let sources = vec![
        Source {
            name: "Scoop Data".into(),
            host: "scoopdata.io".into(),
            url: "https://scoopdata.io/".into(),
        },
        Source {
            name: "Trends Index".into(),
            host: "trends.google.com".into(),
            url: "https://trends.google.com/trends/".into(),
        },
        Source {
            name: "Market Basket".into(),
            host: "marketbasket.io".into(),
            url: "https://marketbasket.io/".into(),
        },
    ];
    let followups = if follow {
        vec![
            "Which flavors sell best in winter".to_string(),
            "Compare gelato and soft serve margins".to_string(),
        ]
    } else {
        vec![]
    };
    rsx! {
        div { class: "oxd-w520",
            StreamingText {
                stream: s(),
                now,
                sources,
                source_count: Some(10),
                followups,
                on_follow: move |f: String| picked.set(Some(f)),
                on_retry: move |_| s.write().started = now,
            }
            if let Some(f) = picked() { p { class: "oxr-dim oxr-small", role: "status", "Asked: {f}" } }
        }
    }
}

fn stream_code(_: &Vals) -> String {
    Code::new("StreamingText")
        .pre("// append tokens to stream.text as they arrive; [n] markers become citation chips")
        .raw("stream", "stream()")
        .raw("now", "clock.now()")
        .raw("sources", "sources()")
        .raw("followups", "followups()")
        .raw("on_follow", "move |q| ask(q)")
        .done()
}

pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel("size", "Size", &["sm", "md"], "md"),
                sel("variant", "Variant", &["pill", "plain"], "pill"),
                tog("loop", "Loop", false),
                tog("done", "Done", false),
                rng("interval", "Interval (ms)", 800.0, 4000.0, 200.0, "2200"),
            ],
            variants: vec![
                var(
                    "phases",
                    "Walks through four phases, then holds the last.",
                    &[],
                ),
                var("done", "Settled: Thought for 4s.", &[("done", "true")]),
                var(
                    "plain-sm",
                    "Plain and small, looping fast.",
                    &[
                        ("size", "sm"),
                        ("variant", "plain"),
                        ("loop", "true"),
                        ("interval", "1200"),
                    ],
                ),
            ],
            render: think,
            code: think_code,
            keyboard: vec![("—", "Not interactive (the buttons belong to the demo)")],
            a11y: vec![
                "role=status with aria-live=polite: each phase and the done label are announced.",
            ],
            responsive: vec!["The label truncates inside the pill."],
            height: 200,
            align: Align::Center,
            live: true,
            ..mk(
                "thinking-loader-oxyn",
                "Thinking loader (Oxyn)",
                "loaders",
                "A small working pill for AI replies: an eight-ray spinner and a shimmering label that rolls through phases and settles on a check with “Thought for 4s”.",
                "ThinkingLoader",
                "ThinkingLoaderEl",
                "thinking-loader",
                "parity/text.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "variant",
                    "Grid",
                    &["drive", "dots", "orbit", "surfer"],
                    "drive",
                ),
                txt("label", "Verb", "Churning"),
                tog("elapsed", "Elapsed time", true),
                tog("reduced", "Reduced motion", false),
            ],
            variants: vec![
                var("drive", "A lit column driving across.", &[]),
                var(
                    "orbit",
                    "A dot orbiting the edge.",
                    &[("variant", "orbit"), ("label", "Thinking")],
                ),
                var(
                    "surfer",
                    "A wave rolling through.",
                    &[("variant", "surfer"), ("label", "Searching")],
                ),
                var(
                    "dots",
                    "Four dots, no timer.",
                    &[
                        ("variant", "dots"),
                        ("elapsed", "false"),
                        ("label", "Loading"),
                    ],
                ),
            ],
            render: loading,
            code: loading_code,
            keyboard: vec![("—", "Not interactive")],
            a11y: vec!["role=status named by the verb; the grid and timer are aria-hidden."],
            responsive: vec!["Inline; the timer keeps four characters of width."],
            height: 180,
            align: Align::Center,
            live: true,
            ..mk(
                "loading-state",
                "Loading state",
                "loaders",
                "A pixel-grid loader with a shimmering verb and the elapsed time: drive, dots, orbit or surfer.",
                "LoadingState",
                "LoadingStateEl",
                "loading-state",
                "parity/text.rs",
            )
        },
        Demo {
            controls: vec![
                sel("by", "By", &["word", "char"], "word"),
                sel("tag", "Element", &["h1", "h2", "h3", "p"], "h2"),
                rng("stagger", "Stagger (ms)", 10.0, 120.0, 5.0, "40"),
                rng("delay", "Delay (ms)", 0.0, 800.0, 50.0, "0"),
                tog("once", "Once", true),
            ],
            variants: vec![
                var("heading", "Words rise out of a clip and a blur.", &[]),
                var(
                    "paragraph",
                    "A paragraph with a softer blur; total stagger capped.",
                    &[("tag", "p"), ("stagger", "30")],
                ),
                var(
                    "chars",
                    "Letter by letter.",
                    &[("by", "char"), ("tag", "h1"), ("stagger", "25")],
                ),
                var("replay", "Plays again on each entry.", &[("once", "false")]),
            ],
            render: reveal,
            code: reveal_code,
            keyboard: vec![("—", "Not interactive")],
            a11y: vec![
                "The element carries the whole text as its aria-label, so pieces are read as one.",
            ],
            responsive: vec!["Wraps normally; each word is its own box."],
            height: 260,
            align: Align::Start,
            live: false,
            ..mk(
                "word-reveal",
                "Text reveal",
                "status-text",
                "Text that arrives word by word or letter by letter, each piece rising out of a clip and a soft blur as it comes into view, on a capped stagger.",
                "WordReveal",
                "",
                "word-reveal",
                "parity/text.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "variant",
                    "Variant",
                    &["blur", "word", "line", "tracking", "wipe"],
                    "blur",
                ),
                sel("tag", "Heading", &["h1", "h2", "h3"], "h2"),
                tog("once", "Once", true),
            ],
            variants: vec![
                var("blur", "Opacity, then blur, then transform.", &[]),
                var("word", "Words rise from a clip.", &[("variant", "word")]),
                var(
                    "line",
                    "Explicit lines rise.",
                    &[("variant", "line"), ("tag", "h1")],
                ),
                var(
                    "tracking",
                    "Letter spacing opens.",
                    &[("variant", "tracking")],
                ),
                var(
                    "wipe",
                    "Uncovers left to right.",
                    &[("variant", "wipe"), ("once", "false")],
                ),
            ],
            render: title,
            code: title_code,
            keyboard: vec![("—", "Not interactive")],
            a11y: vec!["A real heading element with the full title as its label."],
            responsive: vec!["Lines wrap; the line variant keeps explicit breaks."],
            height: 240,
            align: Align::Start,
            live: false,
            ..mk(
                "in-view-title",
                "In-view title",
                "status-text",
                "A section title that comes in as it scrolls into view: blur, word, line, tracking or wipe; once, or again on every entry with a quick fade out.",
                "InViewTitle",
                "",
                "in-view-title",
                "parity/text.rs",
            )
        },
        Demo {
            controls: vec![
                sel("set", "Labels", &["follow", "copy", "save"], "follow"),
                tog("auto", "Auto", true),
                rng("size", "Size", 13.0, 40.0, 1.0, "24"),
            ],
            variants: vec![
                var("follow", "Follow → Following: six letters stay.", &[]),
                var("copy", "Copy → Copied → Done.", &[("set", "copy")]),
                var(
                    "manual",
                    "Step by hand.",
                    &[("set", "save"), ("auto", "false"), ("size", "16")],
                ),
            ],
            render: morph,
            code: morph_code,
            keyboard: vec![("—", "Not interactive (the button belongs to the demo)")],
            a11y: vec!["The label is the accessible name; letter spans are aria-hidden."],
            responsive: vec!["Inline; never wraps."],
            height: 200,
            align: Align::Center,
            live: true,
            ..mk(
                "text-morph",
                "Text morph",
                "status-text",
                "A label morphing into its next state letter by letter: letters matched by character and occurrence stay, new ones stagger in from a blur, removed ones fade out.",
                "TextMorph",
                "",
                "text-morph",
                "parity/text.rs",
            )
        },
        Demo {
            controls: vec![
                sel("kind", "States", &["follow", "copy", "deploy"], "follow"),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                tog("async", "Async action", true),
                tog("fail", "Action fails", false),
            ],
            variants: vec![
                var("follow", "Folds into a round spinner while it works.", &[]),
                var(
                    "copy",
                    "Copied holds 1.5 s, then returns.",
                    &[("kind", "copy"), ("async", "false")],
                ),
                var(
                    "deploy",
                    "Pending labels and tones.",
                    &[("kind", "deploy"), ("size", "lg")],
                ),
                var(
                    "fails",
                    "A failed action shows Try again for 1.8 s.",
                    &[("fail", "true")],
                ),
            ],
            render: swap,
            code: swap_code,
            keyboard: vec![("Enter / Space", "Advance to the next state")],
            a11y: vec!["A real button; aria-busy while working; the label change is polite."],
            responsive: vec!["Width follows the label."],
            height: 200,
            align: Align::Center,
            live: true,
            ..mk(
                "action-swap",
                "Action swap",
                "buttons",
                "An action label that swaps to its next state while keeping its place: a working phase (pending label or a round spinner), a failure label, tones, and holds that advance on their own.",
                "ActionSwapButton",
                "",
                "action-swap",
                "parity/text.rs",
            )
        },
        Demo {
            controls: vec![
                tog("active", "Active", true),
                tog("rotate", "Change label", false),
                rng("duration", "Pass (ms)", 800.0, 4000.0, 100.0, "1800"),
                rng("size", "Size", 12.0, 32.0, 1.0, "18"),
            ],
            variants: vec![
                var("sweep", "A calm band across the words.", &[]),
                var("labels", "A changed label rises in.", &[("rotate", "true")]),
                var(
                    "paused",
                    "Inactive: muted text, no sweep.",
                    &[("active", "false")],
                ),
            ],
            render: shimmer,
            code: shimmer_code,
            keyboard: vec![("—", "Not interactive")],
            a11y: vec!["Plain text; the sweep is decoration."],
            responsive: vec!["Inline."],
            height: 160,
            align: Align::Center,
            live: true,
            ..mk(
                "shimmer-label",
                "Text shimmer",
                "status-text",
                "Ongoing work shown as a calm band of light across the words while active, one pass per duration with a pause between passes; a changed label rises in.",
                "SweepText",
                "ShimmerEl",
                "shimmer-label",
                "parity/text.rs",
            )
        },
        Demo {
            controls: vec![
                sel("kind", "Value", &["number", "money", "word"], "number"),
                rng("spins", "Spins", 0.0, 3.0, 1.0, "1"),
                rng("stagger", "Stagger (ms)", 0.0, 200.0, 10.0, "70"),
                rng("size", "Size", 16.0, 56.0, 2.0, "40"),
                tog("announce", "Announce", false),
            ],
            variants: vec![
                var(
                    "number",
                    "Grouped numbers, reels matched from the end.",
                    &[],
                ),
                var(
                    "money",
                    "$999 → $1,000 only spins digits.",
                    &[("kind", "money")],
                ),
                var(
                    "status",
                    "Words flip from the start.",
                    &[("kind", "word"), ("spins", "0"), ("size", "28")],
                ),
            ],
            render: slot,
            code: slot_code,
            keyboard: vec![("—", "Not interactive (the button belongs to the demo)")],
            a11y: vec!["The value is the accessible name; with announce it is a polite status."],
            responsive: vec!["Font size and colour come from the parent."],
            height: 200,
            align: Align::Center,
            live: true,
            ..mk(
                "slot-text",
                "Slot text",
                "status-text",
                "Text and numbers that spin into their new value on reels stopping left to right, with a soft landing, speed blur, and reels that grow in or collapse when the length changes.",
                "SlotReels",
                "SlotTextEl",
                "slot-text",
                "parity/text.rs",
            )
        },
        Demo {
            controls: vec![
                sel("speed", "Words / s", &["6", "12", "30"], "12"),
                tog("follow", "Follow-ups", true),
                tog("done", "Finished", false),
            ],
            variants: vec![
                var("streaming", "Words fade in; citations pop in place.", &[]),
                var(
                    "done",
                    "Sources, actions and follow-ups.",
                    &[("done", "true")],
                ),
                var(
                    "bare",
                    "Fast, no follow-ups.",
                    &[("speed", "30"), ("follow", "false")],
                ),
            ],
            render: stream,
            code: stream_code,
            keyboard: vec![
                (
                    "Tab",
                    "Citations, sources, copy, good, bad, retry, follow-ups",
                ),
                ("Enter", "Open sources / ask a follow-up"),
            ],
            a11y: vec![
                "aria-busy while streaming; actions are named buttons; votes use aria-pressed.",
            ],
            responsive: vec!["Fills its container; citation chips wrap with the text."],
            height: 360,
            align: Align::Start,
            live: true,
            ..mk(
                "streaming-text",
                "Streaming text",
                "chat",
                "A streamed answer with inline sources, a stacked source list, actions and follow-ups.",
                "StreamingText",
                "",
                "streaming-text",
                "parity/text.rs",
            )
        },
    ]
}
