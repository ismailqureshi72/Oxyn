//! AI special components, part two (Oxyn Pro, Arc "Special components · AI"): Ask your data,
//! Agent doc cursor, Provider status, Training run, Tone tuner, AI mascot, Image reveal, AI
//! spark, Pixel mascot, Ghost mascot, Cube mascot, Upgrade prompt, Referral card, Trial banner,
//! Live visitors and Social proof toast. Every demo runs on the host clock (`live`).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{
    AgentDocCursor, AiMascot, AiSpark, AskYourData, CubeMascot, GhostMascot, ImageReveal,
    LiveVisitors, PixelMascot, ProviderStatus, ReferralCard, SocialProofToast, ToneTuner,
    TrainingRun, TrialBanner, UpgradePrompt,
};

use super::ai_special::{ai, clock, idx};
use super::{rng, sel, tog, var};
use crate::{Demo, Vals};

const SUGS: [&str; 3] = ["signups", "revenue", "churn"];
const QS: [&str; 3] = [
    "Weekly signups by plan since August",
    "Monthly recurring revenue by plan this year",
    "Churn rate by plan, last 12 weeks",
];

fn ask(v: &Vals, now: ClockMs) -> Element {
    let q = if v.s("question") == "unmatched" {
        "Favourite colour of each customer".to_string()
    } else {
        QS[idx(v, "question", &SUGS)].to_string()
    };
    rsx! { AskYourData { now, question: q, auto_run: v.b("auto"), keep_sql_open: v.b("keep"), fail: v.b("fail") } }
}
fn ask_code(v: &Vals) -> String {
    clock("AskYourData")
        .raw("question", format!("{:?}", QS[idx(v, "question", &SUGS)]))
        .b("keep_sql_open", v.b("keep"))
        .done()
}

fn doc(v: &Vals, now: ClockMs) -> Element {
    rsx! { AgentDocCursor { now, auto_play: v.b("auto"), looping: v.b("loop"), speed: v.f("speed") as u64, fail: v.b("fail"), teammate: v.b("mate") } }
}
fn doc_code(v: &Vals) -> String {
    let mut c = clock("AgentDocCursor");
    if !v.b("loop") {
        c = c.raw("looping", "false");
    }
    c.done()
}

fn prov(v: &Vals, now: ClockMs) -> Element {
    let start = idx(v, "start", &["story", "failed", "clear"]);
    rsx! { ProviderStatus { now, interval_s: v.f("interval") as u64, start_check: [0, 9, 10][start], uptime_bars: v.b("bars"), open: if v.b("open") { Some(0) } else { None } } }
}
fn prov_code(v: &Vals) -> String {
    clock("ProviderStatus")
        .raw("interval_s", format!("{}", v.f("interval") as u64))
        .done()
}

fn train(v: &Vals, now: ClockMs) -> Element {
    rsx! { TrainingRun { now, oom: v.b("oom"), speed: v.f("speed") as u64, events: v.b("events") } }
}
fn train_code(v: &Vals) -> String {
    clock("TrainingRun").b("oom", v.b("oom")).done()
}

fn tone(v: &Vals, now: ClockMs) -> Element {
    rsx! { ToneTuner { now, formality: idx(v, "formality", &["casual", "neutral", "formal"]), detail: idx(v, "detail", &["brief", "standard", "detailed"]), show_changes: v.b("changes"), tour: v.b("tour"), fail: v.b("fail") } }
}
fn tone_code(v: &Vals) -> String {
    clock("ToneTuner")
        .raw(
            "formality",
            idx(v, "formality", &["casual", "neutral", "formal"]).to_string(),
        )
        .raw(
            "detail",
            idx(v, "detail", &["brief", "standard", "detailed"]).to_string(),
        )
        .done()
}

const MOODS: [&str; 7] = [
    "loop",
    "idle",
    "listening",
    "thinking",
    "writing",
    "done",
    "error",
];
fn mood(v: &Vals) -> String {
    match v.s("status") {
        "loop" | "" => String::new(),
        s => s.to_string(),
    }
}
fn mascot_code(name: &str, v: &Vals) -> String {
    let m = mood(v);
    let mut c = clock(name);
    if !m.is_empty() {
        c = c.raw("status", format!("{m:?}"));
    }
    c.raw("size", format!("{:.0}.0", v.f("size"))).done()
}

fn orb(v: &Vals, now: ClockMs) -> Element {
    rsx! { div { class: "oxaik-msc-row",
        AiMascot { now, status: mood(v), size: v.f("size") as f32, tone: v.string("tone"), ground: v.b("ground"), name: "Atlas".to_string(), announce: true, picker: v.b("picker") }
        if v.b("ladder") { for s in [24.0f32, 40.0, 64.0] { AiMascot { now, status: mood(v), size: s, tone: v.string("tone"), name: "Atlas".to_string() } } }
    } }
}
fn orb_code(v: &Vals) -> String {
    mascot_code("AiMascot", v)
}
fn ghost(v: &Vals, now: ClockMs) -> Element {
    rsx! { div { class: "oxaik-msc-row",
        GhostMascot { now, status: mood(v), size: v.f("size") as f32, tone: v.string("tone"), ground: v.b("ground"), name: "Atlas".to_string(), announce: true, picker: v.b("picker") }
        if v.b("ladder") { for s in [24.0f32, 40.0, 64.0] { GhostMascot { now, status: mood(v), size: s, tone: v.string("tone"), name: "Atlas".to_string() } } }
    } }
}
fn ghost_code(v: &Vals) -> String {
    mascot_code("GhostMascot", v)
}
fn cube(v: &Vals, now: ClockMs) -> Element {
    rsx! { div { class: "oxaik-msc-row",
        CubeMascot { now, status: mood(v), size: v.f("size") as f32, tone: v.string("tone"), ground: v.b("ground"), name: "Atlas".to_string(), announce: true, picker: v.b("picker") }
        if v.b("ladder") { for s in [24.0f32, 40.0, 64.0] { CubeMascot { now, status: mood(v), size: s, tone: v.string("tone"), name: "Atlas".to_string() } } }
    } }
}
fn cube_code(v: &Vals) -> String {
    mascot_code("CubeMascot", v)
}
fn pixel(v: &Vals, now: ClockMs) -> Element {
    rsx! { div { class: "oxaik-msc-row",
        PixelMascot { now, status: mood(v), scale: v.f("scale") as u32, tone: v.string("tone"), name: "Atlas".to_string(), announce: true, picker: v.b("picker") }
        if v.b("ladder") { for s in [1u32, 2, 3] { PixelMascot { now, status: mood(v), scale: s, tone: v.string("tone"), name: "Atlas".to_string() } } }
    } }
}
fn pixel_code(v: &Vals) -> String {
    let m = mood(v);
    let mut c = clock("PixelMascot");
    if !m.is_empty() {
        c = c.raw("status", format!("{m:?}"));
    }
    c.raw("scale", format!("{}", v.f("scale") as u32)).done()
}

const SPARK_MOODS: [&str; 6] = ["loop", "idle", "thinking", "writing", "done", "error"];
fn spark(v: &Vals, now: ClockMs) -> Element {
    let st = mood(v);
    rsx! { div { class: "oxaik-msc-row",
        AiSpark { now, status: st.clone(), variant: v.string("variant"), size: if v.string("variant") == "mark" { v.f("size") as f32 } else { 0.0 }, tone: v.string("tone"), speed: v.f("speed") as f32, picker: v.b("picker") }
        if v.b("all") {
            AiSpark { now, status: st.clone(), variant: "pill".to_string(), tone: v.string("tone"), announce: false }
            AiSpark { now, status: st.clone(), variant: "avatar".to_string(), tone: v.string("tone"), announce: false }
        }
    } }
}
fn spark_code(v: &Vals) -> String {
    let m = mood(v);
    let mut c = clock("AiSpark").s("variant", v.s("variant"), "mark");
    if !m.is_empty() {
        c = c.raw("status", format!("{m:?}"));
    }
    c.done()
}

fn reveal(v: &Vals, now: ClockMs) -> Element {
    let aspect = match v.s("aspect") {
        "portrait" => "4 / 5",
        "wide" => "16 / 9",
        _ => "1",
    };
    if v.b("grid") {
        return rsx! { div { class: "oxaik-ir-grid",
            for i in 0..4u64 { ImageReveal { now, size: "sm".to_string(), aspect: "1".to_string(), delay: i * 350, seed: i, duration: v.f("duration") as u64 } }
        } };
    }
    rsx! { ImageReveal { now, size: v.string("size"), aspect: aspect.to_string(), duration: v.f("duration") as u64, fail: v.b("fail"), again: v.b("again"), looping: !v.b("again") } }
}
fn reveal_code(v: &Vals) -> String {
    clock("ImageReveal")
        .s("size", v.s("size"), "md")
        .b("fail", v.b("fail"))
        .done()
}

fn upgrade(v: &Vals, now: ClockMs) -> Element {
    let (used, limit, unit) = match v.s("limit") {
        "under" => (38, 50, "AI credits"),
        "seats" => (3, 3, "seats"),
        _ => (50, 50, "AI credits"),
    };
    rsx! { UpgradePrompt { now, used, limit, unit, yearly: v.s("interval") == "yearly", fail: v.b("fail") } }
}
fn upgrade_code(v: &Vals) -> String {
    clock("UpgradePrompt").b("fail", v.b("fail")).done()
}

fn referral(v: &Vals, now: ClockMs) -> Element {
    rsx! { ReferralCard { now, simulate: v.b("simulate"), fail: v.b("fail"), list: v.b("list") } }
}
fn referral_code(v: &Vals) -> String {
    let mut c = clock("ReferralCard");
    if !v.b("simulate") {
        c = c.raw("simulate", "false");
    }
    c.done()
}

fn trial(v: &Vals, now: ClockMs) -> Element {
    rsx! { TrialBanner { now, variant: v.string("variant"), days_left: v.f("days") as i64, fail: v.b("fail") } }
}
fn trial_code(v: &Vals) -> String {
    clock("TrialBanner")
        .s("variant", v.s("variant"), "banner")
        .raw("days_left", format!("{}", v.f("days") as i64))
        .done()
}

fn visitors(v: &Vals, now: ClockMs) -> Element {
    let c: &'static str = match v.s("connection") {
        "connecting" => "connecting",
        "reconnecting" => "reconnecting",
        _ => "live",
    };
    rsx! { LiveVisitors { now, connection: c, paused: v.b("paused"), map: v.b("map"), max_pages: v.f("pages") as usize } }
}
fn visitors_code(v: &Vals) -> String {
    clock("LiveVisitors").b("paused", v.b("paused")).done()
}

fn proof(v: &Vals, now: ClockMs) -> Element {
    rsx! { SocialProofToast { now, interval: (v.f("interval") * 1000.0) as u64, max_visible: v.f("max") as usize, prefill: if v.b("prefill") { 3 } else { 0 }, expand_on_hover: v.b("fan"), header: v.b("header") } }
}
fn proof_code(v: &Vals) -> String {
    clock("SocialProofToast")
        .raw("interval", format!("{}", (v.f("interval") * 1000.0) as u64))
        .done()
}

fn mascot_controls(tone_default: &'static str) -> Vec<crate::Control> {
    vec![
        sel("status", "Status", &MOODS, "loop"),
        rng("size", "Size", 24.0, 160.0, 4.0, "112"),
        sel("tone", "Tone", &["ink", "soft", "violet"], tone_default),
        tog("ground", "Ground", true),
        tog("picker", "Status chips", true),
        tog("ladder", "Size ladder", false),
    ]
}

fn mascot_a11y(what: &'static str) -> Vec<&'static str> {
    vec![
        what,
        "With announce on, status changes are read in a polite live region outside the image; use it on one character per view.",
        "Each status also changes the expression (eyes widen, lift, drop, squint with joy, brows knit), so meaning never rests on movement or colour.",
        "Reduced motion keeps each status's face and drops the loops.",
    ]
}

/// The second batch of AI special component demos.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        ai(
            "ask-your-data",
            "Ask your data",
            "AskYourData",
            "aikit/ask.rs",
            "Ask a database a question in plain words: the query the model writes types itself into a panel you can fold, rows are counted while it runs, and the result appears as a bar, line or table view plus one sentence explaining it.",
            vec![
                sel(
                    "question",
                    "Question",
                    &["signups", "revenue", "churn", "unmatched"],
                    "signups",
                ),
                tog("auto", "Ask on load", true),
                tog("keep", "Keep SQL open", false),
                tog("fail", "Query fails", false),
            ],
            vec![
                var("signups", "Weekly signups by plan as stacked bars.", &[]),
                var(
                    "revenue",
                    "MRR by plan as lines, latest values in the legend (Oxyn extra).",
                    &[("question", "revenue"), ("keep", "true")],
                ),
                var(
                    "no-match",
                    "A question that matches no table offers suggestions.",
                    &[("question", "unmatched")],
                ),
                var(
                    "failed",
                    "The query times out, with Try again.",
                    &[("question", "churn"), ("fail", "true")],
                ),
            ],
            ask,
            ask_code,
            vec![
                (
                    "Enter",
                    "Ask in the field; while busy the same button stops",
                ),
                (
                    "Escape",
                    "Stop a run from the field; clear the chart readout",
                ),
                ("Enter / Space", "Open or fold the Generated SQL row"),
                (
                    "← → · Home / End",
                    "Move the readout on the chart, or change the view",
                ),
            ],
            vec![
                "Screen readers hear each step: writing the query, running it, then what the chart shows and how many rows it used.",
                "Progress in the query header and the result area always pairs a glyph with text.",
                "The query header toggles its panel and reports expanded or collapsed; the plot takes focus and speaks the values under the guide.",
                "The result area is marked busy until the chart lands.",
            ],
            vec![
                "Under 520 px the source keeps only its mark and the view switch drops under the title.",
                "Narrow cards show every second date label; long query lines and the table scroll inside their own boxes with slim scrollbars.",
            ],
            520,
        ),
        ai(
            "agent-doc-cursor",
            "Agent doc cursor",
            "AgentDocCursor",
            "aikit/doccursor.rs",
            "Watch an agent work on a shared doc the way a coworker would: its labelled pointer marks a paragraph and retypes it, adds a new section and corrects a misspelling while a colleague reads along, and every edit stays highlighted until you keep or revert it.",
            vec![
                tog("auto", "Run on load", true),
                tog("loop", "Loop", true),
                rng("speed", "Speed %", 50.0, 300.0, 25.0, "100"),
                tog("mate", "Teammate cursor", true),
                tog("fail", "Agent unreachable", false),
            ],
            vec![
                var(
                    "editing",
                    "A looping run with a teammate reading along.",
                    &[],
                ),
                var(
                    "review",
                    "No loop: the changes wait in the review bar.",
                    &[("loop", "false"), ("speed", "200")],
                ),
                var(
                    "ask",
                    "Idle with an Ask button, no teammate (Oxyn extra).",
                    &[("auto", "false"), ("mate", "false")],
                ),
                var(
                    "error",
                    "Couldn't reach the agent, with Try again.",
                    &[("fail", "true")],
                ),
            ],
            doc,
            doc_code,
            vec![
                ("Tab", "Stop or Run again, then Undo and Accept changes"),
                ("Enter / Space", "Press the focused button"),
            ],
            vec![
                "Each step is announced (reading, the current edit, changes ready, accepted, undone, errors).",
                "Pointers and highlights are hidden from assistive tech; the brief itself is ordinary headings and text, and unreviewed edits are wrapped in mark.",
                "Hover or focus inside the card holds a looping run in review so the bar never moves away mid reach.",
            ],
            vec![
                "Grows to 680 px; on phones the page margins shrink and the review strip loses its credit line.",
                "Cursors sit inline with the text they act on, so they wrap with it at any width.",
            ],
            470,
        ),
        ai(
            "provider-status",
            "Provider status",
            "ProviderStatus",
            "aikit/provstatus.rs",
            "Are the model APIs you rely on up? One row per vendor with its logo, a health word and glyph, median response time and a month of daily ticks; rows with an open problem expand into its timeline, and a ring counts down to the next poll.",
            vec![
                sel("start", "Start at", &["story", "failed", "clear"], "story"),
                rng("interval", "Check every (s)", 6.0, 60.0, 2.0, "12"),
                tog("bars", "Uptime bars", true),
                tog("open", "OpenAI incident open", true),
            ],
            vec![
                var("incident", "An OpenAI outage resolving over checks.", &[]),
                var(
                    "failed-check",
                    "A failed check keeps the last results.",
                    &[("start", "failed"), ("open", "false")],
                ),
                var(
                    "compact",
                    "All clear, no uptime bars (Oxyn extra).",
                    &[("start", "clear"), ("bars", "false"), ("open", "false")],
                ),
            ],
            prov,
            prov_code,
            vec![
                ("Tab", "Check now, each incident row, each uptime bar"),
                ("Enter / Space", "Open or close an incident"),
                ("← → · Home / End", "Move the day readout on an uptime bar"),
                ("Escape", "Hide the day readout"),
            ],
            vec![
                "A labelled section with a list; incident rows are full-row buttons with aria-expanded.",
                "Health is an icon and a word; each day's health is in the readout and the bar's label sums up the 30 days.",
                "Checks, failures and simulated refreshes are announced politely.",
            ],
            vec![
                "From 560 px a row is one line; below it (a container query) the uptime bar takes its own line.",
                "Long names and model lists end in an ellipsis; latency uses tabular numerals.",
            ],
            470,
        ),
        ai(
            "training-run",
            "Training run",
            "TrainingRun",
            "aikit/training.rs",
            "Follow a fine-tune from queue to endpoint: loss for training and held-out data is plotted against the full step range (which also shows epoch progress), alongside learning rate, time spent and time left, the latest log lines, Pause, a Cancel that asks twice, and Deploy at the end.",
            vec![
                rng("speed", "Speed %", 50.0, 400.0, 25.0, "150"),
                tog("oom", "Out of memory at 940", false),
                tog("events", "Event log", true),
            ],
            vec![
                var("training", "A LoRA run from the queue to Deploy.", &[]),
                var(
                    "oom",
                    "Fails at step 940; Retry from checkpoint 800.",
                    &[("oom", "true"), ("speed", "300")],
                ),
                var(
                    "quiet",
                    "No event log, quarter speed (Oxyn extra).",
                    &[("events", "false"), ("speed", "50")],
                ),
            ],
            train,
            train_code,
            vec![
                ("Tab", "The loss chart, then the footer actions"),
                (
                    "← / →",
                    "Step through samples on the chart (Shift jumps ten)",
                ),
                ("Home / End", "First or latest sample"),
                ("Escape", "Back to the latest sample"),
                (
                    "Enter / Space",
                    "Press an action; Cancel needs Confirm cancel",
                ),
            ],
            vec![
                "Status changes are announced (Queued, Training, Paused, Completed, Failed), never every step.",
                "The job state is shown as a glyph with a label; why it failed is written out at the bottom and in the log.",
                "The plot is a named group that speaks the step under the guide and both loss values; the figures beneath are a dl.",
            ],
            vec![
                "Grows to 640 px; on narrow cards the figures form a 2 × 2 grid and the key abbreviates to Val.",
                "The axis spans the whole run and the y range is fixed, so nothing rescales mid-run.",
            ],
            540,
        ),
        ai(
            "tone-tuner",
            "Tone tuner",
            "ToneTuner",
            "aikit/tone.rs",
            "Adjust how a customer reply sounds: two sliders (how formal, how detailed) and three voice chips rework the message; only sentences that changed are swapped, words that are new compared with the draft can be highlighted, and Copy grabs whatever is showing.",
            vec![
                sel(
                    "formality",
                    "Formality",
                    &["casual", "neutral", "formal"],
                    "neutral",
                ),
                sel(
                    "detail",
                    "Detail",
                    &["brief", "standard", "detailed"],
                    "standard",
                ),
                tog("changes", "Mark changes", true),
                tog("tour", "Tour voices", true),
                tog("fail", "Rewrite fails", false),
            ],
            vec![
                var("tour", "Tours the voices until you touch it.", &[]),
                var(
                    "formal-detailed",
                    "Formal and detailed, no tour (Oxyn extra).",
                    &[
                        ("formality", "formal"),
                        ("detail", "detailed"),
                        ("tour", "false"),
                    ],
                ),
                var(
                    "failed",
                    "A failed rewrite keeps the last version.",
                    &[("tour", "false"), ("fail", "true")],
                ),
            ],
            tone,
            tone_code,
            vec![
                ("Tab", "Voice chips, both sliders, Changes, Copy"),
                (
                    "← → · Home / End",
                    "Step a focused slider through its three levels",
                ),
                ("Enter / Space", "Press a chip, flip Changes, copy"),
            ],
            vec![
                "The three voices are pressable chips inside a group labelled Voice.",
                "Sliders are named and speak their level name (Casual, Neutral, Formal), not a number.",
                "New words use mark elements and the header counts them, so changes never rely on colour.",
            ],
            vec![
                "Up to 640 px; under 520 px the sliders stack and the chips share one row.",
                "Each sentence keeps its place while it swaps, so the text never jumps.",
            ],
            480,
        ),
        ai(
            "ai-mascot",
            "AI mascot",
            "AiMascot",
            "aikit/mascots.rs",
            "Oxyn's own assistant: a soft square head with a glowing bulb on an antenna. Resting it bobs and looks around; it grows a little to hear you, gazes upward while the bulb orbits as it thinks, follows its own pen line as it writes, beams when finished and jolts with a red bulb when something breaks.",
            mascot_controls("ink"),
            vec![
                var("loop", "Plays every status in turn.", &[]),
                var(
                    "thinking-violet",
                    "Thinking, violet (Oxyn extra).",
                    &[("status", "thinking"), ("tone", "violet")],
                ),
                var(
                    "sizes",
                    "Done at 112 px with a 24 / 40 / 64 ladder (Oxyn extra).",
                    &[("status", "done"), ("ladder", "true"), ("picker", "false")],
                ),
            ],
            orb,
            orb_code,
            vec![(
                "—",
                "A status image: it takes no focus. The demo's chips and Pause are ordinary buttons.",
            )],
            mascot_a11y(
                "The drawing is role img with a label from name and status (Atlas, thinking) that follows every change.",
            ),
            vec![
                "Always a square of the given size, so nothing around it moves; vector shapes stay sharp at any zoom.",
                "Under 56 px a tighter framing with larger eyes keeps it readable at 24 px.",
            ],
            300,
        ),
        ai(
            "image-reveal",
            "Image reveal",
            "ImageReveal",
            "aikit/reveal.rs",
            "A picture coming into being like a diffusion sampler: a coarse block grid in the picture's colours sheds its noise in visible steps over a soft guess that slowly focuses, then a sweep from the top lays down the final sharp result.",
            vec![
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                sel(
                    "aspect",
                    "Aspect",
                    &["square", "portrait", "wide"],
                    "square",
                ),
                rng("duration", "Sampling ms", 2000.0, 9000.0, 200.0, "4600"),
                tog("fail", "Fails at 60%", false),
                tog("again", "Generate again", false),
                tog("grid", "Grid of four", false),
            ],
            vec![
                var(
                    "loop",
                    "A looping generation over Oxyn's sample scenes.",
                    &[],
                ),
                var(
                    "grid",
                    "Four small tiles with staggered starts (Oxyn extra).",
                    &[("grid", "true")],
                ),
                var(
                    "wide-again",
                    "Wide, with Generate again (Oxyn extra).",
                    &[("aspect", "wide"), ("again", "true")],
                ),
                var(
                    "error",
                    "Fails at step 19, with Retry.",
                    &[("fail", "true")],
                ),
            ],
            reveal,
            reveal_code,
            vec![
                (
                    "Tab",
                    "Retry after a failure; the picture itself is not a stop",
                ),
                ("Enter / Space", "Retry or Generate again"),
            ],
            vec![
                "The picture is exposed as an image whose name reports progress and switches to a real description at the end.",
                "Waiting, generating, ready and failures are spoken; individual steps stay silent.",
                "During sampling the badge acts as a progress bar reading Step n of total.",
            ],
            vec![
                "Fills its container (up to 560 px) at the given aspect ratio.",
                "The small size keeps only the ring and the step count.",
            ],
            560,
        ),
        ai(
            "ai-spark",
            "AI spark",
            "AiSpark",
            "aikit/mascots.rs",
            "A little star glyph that signals machine work: four long points and four short paler ones swell gently when resting, send a wave round while thinking, pulse while writing and draw together when finished, always easing from wherever they were.",
            vec![
                sel("status", "Status", &SPARK_MOODS, "loop"),
                sel("variant", "Variant", &["mark", "pill", "avatar"], "mark"),
                rng("size", "Mark size", 12.0, 120.0, 4.0, "64"),
                sel("tone", "Tone", &["violet", "ink", "muted"], "violet"),
                rng("speed", "Speed", 0.5, 2.0, 0.25, "1"),
                tog("picker", "Status chips", true),
                tog("all", "All variants", true),
            ],
            vec![
                var("loop", "The mark with its pill and avatar.", &[]),
                var(
                    "pill-thinking",
                    "The pill, thinking (Oxyn extra: ink tone).",
                    &[
                        ("variant", "pill"),
                        ("status", "thinking"),
                        ("tone", "ink"),
                        ("all", "false"),
                    ],
                ),
                var(
                    "error",
                    "Couldn't finish: short rays, dim diagonals, red.",
                    &[("status", "error"), ("all", "false")],
                ),
            ],
            spark,
            spark_code,
            vec![(
                "—",
                "Purely an indicator and never focused; place buttons like Stop next to it.",
            )],
            vec![
                "The root is a polite status region announcing changes once (AI is thinking); aria-busy while working.",
                "Every state has a distinct silhouette as well as movement, failure adds red, and the pill spells the state out.",
                "Decorative copies set announce off so only one spark speaks.",
            ],
            vec![
                "One footprint for every status, so text around it never shifts.",
                "Geometry is in a 100-unit box and stays crisp from 12 px up.",
            ],
            240,
        ),
        ai(
            "pixel-mascot",
            "Pixel mascot",
            "PixelMascot",
            "aikit/mascots.rs",
            "Oxyn's pixel robot: it idles, thinks under a bubble of dots, types on a keyboard, raises its arms under stars and crosses its eyes on an error, in sprite steps on a fixed clock, crisp at every whole-pixel scale.",
            vec![
                sel(
                    "status",
                    "Status",
                    &["loop", "idle", "thinking", "writing", "done", "error"],
                    "loop",
                ),
                rng("scale", "Scale", 1.0, 8.0, 1.0, "5"),
                sel("tone", "Tone", &["ink", "soft", "violet"], "ink"),
                tog("picker", "Status chips", true),
                tog("ladder", "Scale ladder", false),
            ],
            vec![
                var("loop", "Plays every status in turn.", &[]),
                var(
                    "writing-violet",
                    "Writing, violet (Oxyn extra).",
                    &[("status", "writing"), ("tone", "violet")],
                ),
                var(
                    "scales",
                    "Done at 1×, 2×, 3× and 5× (Oxyn extra).",
                    &[("status", "done"), ("ladder", "true"), ("picker", "false")],
                ),
            ],
            pixel,
            pixel_code,
            vec![("—", "A status image: it takes no focus.")],
            mascot_a11y(
                "A single image element named after the character and its state (Atlas, thinking), kept current.",
            ),
            vec![
                "A fixed square of 24 × scale px; crisp edges at every whole scale.",
                "Inline at scales 1 and 2, so it sits on a line of text.",
            ],
            300,
        ),
        ai(
            "ghost-mascot",
            "Ghost mascot",
            "GhostMascot",
            "aikit/mascots.rs",
            "A floating ghost whose scalloped hem ripples as it hovers: it blinks at rest, comes forward to listen, rises and turns translucent under a swirl while thinking, writes with a tiny pen, smiles when done and wobbles on an error.",
            mascot_controls("soft"),
            vec![
                var("loop", "Plays every status in turn.", &[]),
                var(
                    "listening-ink",
                    "Listening, ink (Oxyn extra).",
                    &[("status", "listening"), ("tone", "ink")],
                ),
                var(
                    "sizes",
                    "Thinking with a size ladder (Oxyn extra).",
                    &[
                        ("status", "thinking"),
                        ("ladder", "true"),
                        ("picker", "false"),
                    ],
                ),
            ],
            ghost,
            ghost_code,
            vec![("—", "A status image: it takes no focus.")],
            mascot_a11y(
                "The drawing is role img labelled with the name and status, updated with every change.",
            ),
            vec![
                "A fixed square that never reflows; the shadow shrinks as the ghost rises.",
                "Under 56 px a tighter framing keeps the eyes readable.",
            ],
            300,
        ),
        ai(
            "cube-mascot",
            "Cube mascot",
            "CubeMascot",
            "aikit/mascots.rs",
            "An isometric cube with three shaded sides and a painted face: it tilts and blinks at rest, turns a quarter at a time while thinking with the face moving to the side in front, prints lines on the floor while writing, and sparkles when done.",
            mascot_controls("violet"),
            vec![
                var("loop", "Plays every status in turn.", &[]),
                var(
                    "thinking-ink",
                    "Thinking, ink (Oxyn extra).",
                    &[("status", "thinking"), ("tone", "ink")],
                ),
                var(
                    "sizes",
                    "Writing with a size ladder (Oxyn extra).",
                    &[
                        ("status", "writing"),
                        ("ladder", "true"),
                        ("picker", "false"),
                    ],
                ),
            ],
            cube,
            cube_code,
            vec![("—", "A status image: it takes no focus.")],
            mascot_a11y(
                "The drawing is role img labelled with the name and status (Atlas, writing).",
            ),
            vec![
                "A fixed square; the sides are real 3D corners projected each frame.",
                "Under 56 px the cube is framed tighter with larger eyes.",
            ],
            300,
        ),
        ai(
            "upgrade-prompt",
            "Upgrade prompt",
            "UpgradePrompt",
            "aikit/growth.rs",
            "Shown when an allowance runs out: how much was used, what Pro costs with a monthly or annual toggle, three things it adds (with the logos of the models it mentions), and a choice of Upgrade or Maybe later, every state the same size.",
            vec![
                sel("limit", "Usage", &["full", "under", "seats"], "full"),
                sel("interval", "Billing", &["yearly", "monthly"], "yearly"),
                tog("fail", "Card declined", false),
            ],
            vec![
                var("limit", "50 of 50 AI credits, yearly billing.", &[]),
                var(
                    "under",
                    "38 of 50, monthly (Oxyn extra).",
                    &[("limit", "under"), ("interval", "monthly")],
                ),
                var(
                    "seats",
                    "All 3 seats used, card declined (Oxyn extra).",
                    &[("limit", "seats"), ("fail", "true")],
                ),
            ],
            upgrade,
            upgrade_code,
            vec![
                (
                    "Tab",
                    "The billing switch, Maybe later, Upgrade (or Continue, See Pro again)",
                ),
                ("← / →", "Monthly or yearly on the billing switch"),
                ("Enter / Space", "Press the focused button"),
            ],
            vec![
                "A named region with its own heading, which suits a modal or popover.",
                "The allowance bar is a meter that speaks its value in words, e.g. all 50 credits spent.",
                "Limit reached and credits left are an icon and words; upgrades and snoozes are announced.",
            ],
            vec![
                "Fluid up to 440 px; the billing switch wraps under the price when narrow.",
                "Buttons stay side by side: Maybe later at its width, Upgrade filling the rest.",
            ],
            460,
        ),
        ai(
            "referral-card",
            "Referral card",
            "ReferralCard",
            "aikit/growth.rs",
            "Both sides win: share your personal link or post it, add friends' addresses as chips, then follow each one from invited to joined to paying while your credit climbs and a bonus tier unlocks.",
            vec![
                tog("simulate", "Play the sample program", true),
                tog("list", "Invite list", true),
                tog("fail", "First send fails", false),
            ],
            vec![
                var(
                    "program",
                    "Friends sign up and subscribe; the milestone unlocks.",
                    &[],
                ),
                var(
                    "compact",
                    "No list, no simulation (Oxyn extra).",
                    &[("list", "false"), ("simulate", "false")],
                ),
                var(
                    "send-fails",
                    "The first send fails and keeps the chips.",
                    &[("fail", "true"), ("simulate", "false")],
                ),
            ],
            referral,
            referral_code,
            vec![
                ("Tab", "Copy, the share buttons, the email field, Send"),
                (
                    "Enter · comma · space · Tab",
                    "Make the typed address a chip",
                ),
                ("Enter (empty field)", "Send the chips"),
                ("Backspace (empty field)", "Remove the last chip"),
            ],
            vec![
                "Share buttons are named icon buttons (Post on X, Share in Slack, Share by email).",
                "The helper line below the field describes it and is read when it changes; a bad address flips the field to invalid and adds a warning glyph.",
                "Each invite's progress has a glyph plus a label; the bonus bar is a named image and its goal is also written out.",
            ],
            vec![
                "Grows to 560 px; on narrow cards the post-to buttons move onto a line of their own.",
                "Long addresses are cut with an ellipsis but dates stay readable; extra chips scroll horizontally inside the input.",
            ],
            600,
        ),
        ai(
            "trial-banner",
            "Trial banner",
            "TrialBanner",
            "aikit/growth.rs",
            "Tells a team how long their trial has left: a ring empties as days pass, the text says what they would lose, and paying is one press away; it comes as a thin strip, a side panel or a compact header badge that hiding or postponing shrinks it into.",
            vec![
                sel("variant", "Variant", &["banner", "card", "pill"], "banner"),
                rng("days", "Days left", 0.0, 14.0, 1.0, "9"),
                tog("fail", "Card fails once", false),
            ],
            vec![
                var("banner", "Nine days left as a slim banner.", &[]),
                var(
                    "warning-card",
                    "Two days left as a sidebar card.",
                    &[("variant", "card"), ("days", "2")],
                ),
                var("expired", "Expired.", &[("days", "0")]),
                var(
                    "pill",
                    "The header pill; it opens the card (Oxyn extra).",
                    &[("variant", "pill"), ("days", "5")],
                ),
            ],
            trial,
            trial_code,
            vec![
                ("Tab", "Plans, add a card, postpone, hide"),
                (
                    "Enter / Space",
                    "Run an action, open the snooze menu or the pill's card",
                ),
                (
                    "↑ / ↓ · Enter · Escape",
                    "Move in, pick from or close the snooze menu",
                ),
            ],
            vec![
                "Strip and panel are named regions; the ring is decoration only, and the day count is always written.",
                "Near the end the wording switches to the actual weekday it stops; ended and paid states have their own sentences.",
                "Billing, snooze and dismiss results are announced, including when an action is simulated.",
            ],
            vec![
                "Under 600 px the buttons drop under the copy.",
                "The card fills its column up to 340 px.",
            ],
            300,
        ),
        ai(
            "live-visitors",
            "Live visitors",
            "LiveVisitors",
            "aikit/growth.rs",
            "Who is on your site this second: a headcount whose digits roll, a dot map where each new visitor flashes, the busiest pages ranked with bars, and fresh sign-ups and upgrades listed with their country flag.",
            vec![
                sel(
                    "connection",
                    "Connection",
                    &["live", "connecting", "reconnecting"],
                    "live",
                ),
                rng("pages", "Pages listed", 2.0, 6.0, 1.0, "4"),
                tog("map", "Map", true),
                tog("paused", "Start paused", false),
            ],
            vec![
                var("live", "A simulated marketing site stream.", &[]),
                var(
                    "reconnecting",
                    "Reconnecting keeps the last data.",
                    &[("connection", "reconnecting")],
                ),
                var(
                    "compact",
                    "No map, three pages (Oxyn extra).",
                    &[("map", "false"), ("pages", "3")],
                ),
            ],
            visitors,
            visitors_code,
            vec![
                (
                    "Tab",
                    "Only the pause control is focusable; map and lists are display only",
                ),
                ("Enter / Space", "Pause or resume"),
            ],
            vec![
                "The count is read once as plain text; the rolling digits are hidden from assistive tech.",
                "Pause reports whether it is on and its name says what pressing it does.",
                "Status is a dot next to Live, Paused or Reconnecting, never colour alone.",
            ],
            vec![
                "Caps at 640 px; on narrow cards the two lists sit one above the other and the map shrinks to fit.",
                "Paths and cities end in an ellipsis; counts keep right-aligned tabular columns.",
            ],
            560,
        ),
        ai(
            "social-proof-toast",
            "Social proof toast",
            "SocialProofToast",
            "aikit/growth.rs",
            "Small “someone just joined” cards for marketing pages: new trials, upgrades and checked purchases slide in on a timer, pointing at or tabbing into the pile freezes it and spreads it open, and each card can be closed.",
            vec![
                rng("interval", "Every (s)", 1.5, 8.0, 0.5, "3.5"),
                rng("max", "Stacked", 1.0, 5.0, 1.0, "3"),
                tog("prefill", "Start with three", true),
                tog("fan", "Fan out on hover", true),
                tog("header", "Header line", true),
            ],
            vec![
                var("feed", "A sample SaaS feed.", &[]),
                var(
                    "single",
                    "One at a time, no header (Oxyn extra).",
                    &[("max", "1"), ("header", "false")],
                ),
                var(
                    "deep",
                    "Five stacked, no fan (Oxyn extra).",
                    &[("max", "5"), ("fan", "false")],
                ),
            ],
            proof,
            proof_code,
            vec![
                (
                    "Tab",
                    "Each visible toast's dismiss button; focus pauses and fans the stack",
                ),
                ("Enter / Space", "Dismiss the focused toast"),
                ("Escape", "Dismiss the front toast"),
            ],
            vec![
                "The pile is a named region, but arrivals are deliberately silent so nobody using a screen reader gets cut off.",
                "Each dismiss button names its toast (Dismiss: Sofia from Brightlane started a Pro trial).",
                "Pointing at it or tabbing into it freezes the timer without losing its place.",
            ],
            vec![
                "360 px wide; on narrow screens it spans the width.",
                "One line per row with an ellipsis, so a toast never changes height.",
            ],
            340,
        ),
    ]
}
