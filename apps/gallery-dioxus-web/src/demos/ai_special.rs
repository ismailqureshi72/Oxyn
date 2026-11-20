//! AI special components (Oxyn Pro, Arc "Special components · AI"): Agent timeline, Model card,
//! AI workflow, Image generation, Speech player, Meeting summary, Reply draft, Human handoff,
//! PII redaction, LLM cost calculator, Provider keys, Generation stats, AI translate, TL;DR
//! toggle, Connector setup, Enrich column and Ticket router. Every demo runs on the host clock
//! (`live`), so streams, playheads and syncs move on their own.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{
    AgentTimeline, AiTranslate, AiWorkflow, ConnectorSetup, EnrichColumn, GenerationStats,
    HumanHandoff, ImageGeneration, LlmCostCalculator, MeetingSummary, ModelCard, PiiRedaction,
    ProviderKeys, ReplyDraft, SpeechPlayer, TicketRouter, TldrToggle,
};

use super::{Code, mk, q, rng, sel, tog, var};
use crate::{Align, Control, Demo, Vals, Variant};

/// The category slug of every AI special component.
pub(crate) const CAT: &str = "ai-special";

#[allow(clippy::too_many_arguments)]
fn ai(
    slug: &'static str,
    name: &'static str,
    dioxus: &'static str,
    src: &'static str,
    summary: &'static str,
    controls: Vec<Control>,
    variants: Vec<Variant>,
    render: fn(&Vals, ClockMs) -> Element,
    code: fn(&Vals) -> String,
    keyboard: Vec<(&'static str, &'static str)>,
    a11y: Vec<&'static str>,
    responsive: Vec<&'static str>,
    height: u32,
) -> Demo {
    Demo {
        controls,
        variants,
        render,
        code,
        keyboard,
        a11y,
        responsive,
        height,
        align: Align::Center,
        live: true,
        ..mk(slug, name, CAT, summary, dioxus, "AiKitView", slug, src)
    }
}

fn idx(v: &Vals, k: &str, opts: &[&str]) -> usize {
    opts.iter().position(|o| *o == v.s(k)).unwrap_or(0)
}

fn clock(name: &str) -> Code {
    Code::new(name)
        .pre("let clock = use_motion_clock(); // the host clock (ClockMs)")
        .raw("now", "clock.now()")
}

// ── Agent timeline ──
fn timeline(v: &Vals, now: ClockMs) -> Element {
    rsx! { AgentTimeline { now, view: v.string("view"), autoplay: v.b("autoplay"), live_run: v.b("live"), show_cost: v.b("cost"), speed: v.f("speed") as u64 } }
}
fn timeline_code(v: &Vals) -> String {
    let mut c = clock("AgentTimeline").s("view", v.s("view"), "lanes");
    if !v.b("autoplay") {
        c = c.raw("autoplay", "false");
    }
    c.b("live_run", v.b("live"))
        .b("show_cost", v.b("cost"))
        .done()
}

// ── Model card ──
const MODELS: [&str; 5] = [
    "Claude Opus 4.1",
    "Claude Sonnet 4.5",
    "GPT-5",
    "Gemini 2.5 Pro",
    "Grok 4",
];
fn model_card(v: &Vals, now: ClockMs) -> Element {
    rsx! { ModelCard { now, model: idx(v, "model", &MODELS), view: v.string("view"), volume: v.u("volume"), loading: v.b("loading"), error: v.b("error"), selectable: v.b("select"), tabs: v.b("tabs") } }
}
fn model_card_code(v: &Vals) -> String {
    let mut c = clock("ModelCard").raw("model", idx(v, "model", &MODELS).to_string());
    c = c.s("view", v.s("view"), "details");
    if v.u("volume") != 5 {
        c = c.raw("volume", v.u("volume").to_string());
    }
    c.b("loading", v.b("loading"))
        .b("error", v.b("error"))
        .b("selectable", v.b("select"))
        .done()
}

// ── AI workflow ──
fn workflow(v: &Vals, now: ClockMs) -> Element {
    rsx! { AiWorkflow { now, scenario: v.string("scenario"), autoplay: v.b("autoplay"), editable: v.b("editable"), gate: v.b("gate") } }
}
fn workflow_code(v: &Vals) -> String {
    clock("AiWorkflow")
        .s("scenario", v.s("scenario"), "passes")
        .b("gate", v.b("gate"))
        .done()
}

// ── Image generation ──
fn imagegen(v: &Vals, now: ClockMs) -> Element {
    rsx! { ImageGeneration { now, autoplay: v.b("autoplay"), fail: v.b("fail"), aspect: idx(v, "aspect", &["1:1", "4:5", "16:9"]), placeholder: v.string("placeholder"), grid: v.b("grid") } }
}
fn imagegen_code(v: &Vals) -> String {
    let mut c = clock("ImageGeneration");
    let a = idx(v, "aspect", &["1:1", "4:5", "16:9"]);
    if a != 0 {
        c = c.raw("aspect", a.to_string());
    }
    c.s("placeholder", v.s("placeholder"), "dots")
        .b("grid", v.b("grid"))
        .b("fail", v.b("fail"))
        .done()
}

// ── Speech player ──
fn speech(v: &Vals, now: ClockMs) -> Element {
    rsx! { SpeechPlayer { now, autoplay: v.b("autoplay"), variant: v.string("variant"), fail: v.b("fail"), karaoke: v.b("karaoke"), voice: idx(v, "voice", &["Ava", "Leo", "Maya", "Noah"]) } }
}
fn speech_code(v: &Vals) -> String {
    clock("SpeechPlayer")
        .s("variant", v.s("variant"), "card")
        .b("fail", v.b("fail"))
        .done()
}

// ── Meeting summary ──
fn meeting(v: &Vals, now: ClockMs) -> Element {
    rsx! { MeetingSummary { now, scenario: v.string("scenario"), send: v.b("send"), fail_send: v.b("fail"), brief: v.b("brief") } }
}
fn meeting_code(v: &Vals) -> String {
    clock("MeetingSummary")
        .s("scenario", v.s("scenario"), "play")
        .b("brief", v.b("brief"))
        .b("fail_send", v.b("fail"))
        .done()
}

// ── Reply draft ──
fn reply(v: &Vals, now: ClockMs) -> Element {
    rsx! { ReplyDraft { now, autostart: v.b("autostart"), tone: idx(v, "tone", &["friendly", "concise", "formal"]), length: idx(v, "length", &["short", "medium", "long"]), history: v.b("history") } }
}
fn reply_code(v: &Vals) -> String {
    let mut c = clock("ReplyDraft");
    let t = idx(v, "tone", &["friendly", "concise", "formal"]);
    if t != 0 {
        c = c.raw("tone", t.to_string());
    }
    c.b("history", v.b("history")).done()
}

// ── Human handoff ──
fn handoff(v: &Vals, now: ClockMs) -> Element {
    rsx! { HumanHandoff { now, scenario: v.string("scenario"), edit_summary: v.b("edit"), timeline: v.b("timeline") } }
}
fn handoff_code(v: &Vals) -> String {
    clock("HumanHandoff")
        .s("scenario", v.s("scenario"), "handoff")
        .b("edit_summary", v.b("edit"))
        .done()
}

// ── PII redaction ──
fn pii(v: &Vals, now: ClockMs) -> Element {
    rsx! { PiiRedaction { now, autoplay: v.b("autoplay"), editable: v.b("editable"), categories: v.b("categories") } }
}
fn pii_code(v: &Vals) -> String {
    clock("PiiRedaction")
        .b("categories", v.b("categories"))
        .done()
}

// ── LLM cost calculator ──
fn cost(v: &Vals, _now: ClockMs) -> Element {
    rsx! { LlmCostCalculator { preset: idx(v, "preset", &["support", "rag", "agent"]), batch: v.b("batch"), loading: v.b("loading"), error: v.b("error") } }
}
fn cost_code(v: &Vals) -> String {
    let p = idx(v, "preset", &["support", "rag", "agent"]);
    let mut c = Code::new("LlmCostCalculator");
    if p != 0 {
        c = c.raw("preset", p.to_string());
    }
    c.b("batch", v.b("batch"))
        .b("loading", v.b("loading"))
        .b("error", v.b("error"))
        .done()
}

// ── Provider keys ──
fn keys(v: &Vals, now: ClockMs) -> Element {
    rsx! { ProviderKeys { now, empty: v.b("empty"), usage: v.b("usage"), adding: v.b("adding") } }
}
fn keys_code(v: &Vals) -> String {
    clock("ProviderKeys")
        .b("empty", v.b("empty"))
        .b("adding", v.b("adding"))
        .done()
}

// ── Generation stats ──
fn genstats(v: &Vals, now: ClockMs) -> Element {
    rsx! { GenerationStats { now, scenario: v.string("scenario"), variant: v.string("variant"), model: idx(v, "model", &["Claude Sonnet 4.5", "GPT-5", "Gemini 2.5 Flash"]), looping: v.b("loop"), session: v.b("session") } }
}
fn genstats_code(v: &Vals) -> String {
    clock("GenerationStats")
        .s("scenario", v.s("scenario"), "stream")
        .s("variant", v.s("variant"), "card")
        .b("looping", v.b("loop"))
        .b("session", v.b("session"))
        .done()
}

// ── AI translate ──
const LANGS: [&str; 6] = [
    "original",
    "German",
    "French",
    "Spanish",
    "Japanese",
    "Portuguese",
];
fn translate(v: &Vals, now: ClockMs) -> Element {
    rsx! { AiTranslate { now, lang: idx(v, "lang", &LANGS), fail: v.b("fail"), side: v.b("side") } }
}
fn translate_code(v: &Vals) -> String {
    clock("AiTranslate")
        .raw("lang", idx(v, "lang", &LANGS).to_string())
        .b("side", v.b("side"))
        .done()
}

// ── TL;DR toggle ──
fn tldr(v: &Vals, now: ClockMs) -> Element {
    rsx! { TldrToggle { now, mode: idx(v, "mode", &["full", "summary", "points"]), length: idx(v, "length", &["short", "medium", "long"]), fail: v.b("fail"), saved: v.b("saved") } }
}
fn tldr_code(v: &Vals) -> String {
    clock("TldrToggle")
        .raw(
            "mode",
            idx(v, "mode", &["full", "summary", "points"]).to_string(),
        )
        .raw(
            "length",
            idx(v, "length", &["short", "medium", "long"]).to_string(),
        )
        .done()
}

// ── Connector setup ──
fn connector(v: &Vals, now: ClockMs) -> Element {
    rsx! { ConnectorSetup { now, deny: v.b("deny"), searchable: v.b("search"), autoplay: v.b("autoplay") } }
}
fn connector_code(v: &Vals) -> String {
    clock("ConnectorSetup")
        .raw("assistant", q("Corvel Assistant"))
        .b("deny", v.b("deny"))
        .done()
}

// ── Enrich column ──
fn enrich(v: &Vals, now: ClockMs) -> Element {
    rsx! { EnrichColumn { now, column: idx(v, "column", &["industry", "employees", "city"]), concurrency: v.u("concurrency").max(1), autoplay: v.b("autoplay") } }
}
fn enrich_code(v: &Vals) -> String {
    clock("EnrichColumn")
        .raw(
            "column",
            idx(v, "column", &["industry", "employees", "city"]).to_string(),
        )
        .raw("concurrency", v.u("concurrency").max(1).to_string())
        .done()
}

// ── Ticket router ──
fn router(v: &Vals, now: ClockMs) -> Element {
    rsx! { TicketRouter { now, paused: v.b("paused"), threshold: v.u("threshold") as u32, tuning: v.b("tuning") } }
}
fn router_code(v: &Vals) -> String {
    clock("TicketRouter")
        .raw("threshold", v.u("threshold").to_string())
        .b("paused", v.b("paused"))
        .done()
}

/// The AI special demos, in Arc's order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        ai(
            "agent-timeline",
            "Agent timeline",
            "AgentTimeline",
            "aikit/timeline.rs",
            "One orchestrated agent run on a shared time axis: the orchestrator and its subagents in rows, handoffs drawn between them, and a playhead that replays thinking, tool calls and waiting while the header counts time, tokens and cost.",
            vec![
                sel("view", "View", &["lanes", "waterfall"], "lanes"),
                tog("autoplay", "Replay on load", true),
                tog("live", "Live run (no controls)", false),
                tog("cost", "Cost per span", false),
                sel("speed", "Speed", &["1", "2"], "1"),
            ],
            vec![
                var("replay", "The run replays from zero.", &[]),
                var(
                    "finished",
                    "A finished run, ready to replay, with cost per span (Oxyn extra).",
                    &[("autoplay", "false"), ("cost", "true")],
                ),
                var(
                    "waterfall",
                    "One row per span in start order (Oxyn extra).",
                    &[("view", "waterfall"), ("autoplay", "false")],
                ),
                var(
                    "live",
                    "Following a run in progress: no playback controls.",
                    &[("live", "true")],
                ),
            ],
            timeline,
            timeline_code,
            vec![
                (
                    "Tab",
                    "The spans (one tab stop), the scrubber, then play, restart and speed",
                ),
                (
                    "← / →",
                    "Previous or next span in the row; on the scrubber half a second (Shift: five)",
                ),
                ("↑ / ↓", "Nearest span in the row above or below"),
                (
                    "Home / End",
                    "First or last span in the row; on the scrubber the start or end",
                ),
                ("Page up / down", "On the scrubber, five seconds"),
                (
                    "Enter / Space",
                    "Pin the span and highlight its handoff path; again to unpin",
                ),
                ("Escape", "Unpin"),
            ],
            vec![
                "The root is a labelled section; spans are buttons named by agent, kind, tool, failure, what it did, time range and tokens; aria-pressed marks the pinned span.",
                "Spans use a roving tab index, so the chart is one tab stop.",
                "The scrubber is a slider with a value text like 12.4s of 26.4s.",
                "Failure is a cross icon and the word Failed, never colour alone; the finished state shows a check with the summary.",
                "Replay, pause and the summary are announced politely; the counting status line is not live.",
            ],
            vec![
                "Labels take 180 px; under 560 px they narrow to dot and name and the model moves into the span card.",
                "The plot keeps a 300 px minimum and scrolls inside its own frame (thin scrollbar) with labels pinned.",
                "Counters use tabular numerals, so the header never shifts while it counts.",
            ],
            430,
        ),
        ai(
            "model-card",
            "Model card",
            "ModelCard",
            "aikit/model_card.rs",
            "What a model can do and what it costs: switching models swaps every value in place, a volume slider estimates the monthly cost, benchmark rows show score and rank, and Compare puts a rival side by side with the better value checked.",
            vec![
                sel("model", "Model", &MODELS, "Claude Sonnet 4.5"),
                sel("view", "View", &["details", "compare"], "details"),
                rng("volume", "Volume stop", 0.0, 8.0, 1.0, "5"),
                tog("select", "Use model button", false),
                tog("tabs", "Model tabs", true),
                tog("loading", "Loading", false),
                tog("error", "Error", false),
            ],
            vec![
                var(
                    "details",
                    "Details of Claude Sonnet 4.5 at 100M tokens a month.",
                    &[],
                ),
                var(
                    "compare",
                    "GPT-5 against a rival with the better values checked.",
                    &[("view", "compare"), ("model", "GPT-5")],
                ),
                var(
                    "picker-pane",
                    "The detail pane of a model picker with Use model (Oxyn extra).",
                    &[
                        ("select", "true"),
                        ("tabs", "false"),
                        ("model", "Gemini 2.5 Pro"),
                    ],
                ),
                var(
                    "error",
                    "Couldn't load, with Retry.",
                    &[("error", "true"), ("tabs", "false")],
                ),
            ],
            model_card,
            model_card_code,
            vec![
                (
                    "Tab",
                    "Model tabs, Compare, the volume slider, benchmark switcher and rows, then Copy and Use model",
                ),
                ("← / →", "Model tabs and the benchmark switcher"),
                (
                    "Arrows · Page up / down · Home / End",
                    "Step the monthly volume",
                ),
                (
                    "Enter / ↓",
                    "Open the rival menu; ↑/↓ move, Enter picks, Escape closes",
                ),
                (
                    "Space / Enter",
                    "Toggle Compare, copy the id, select the model",
                ),
            ],
            vec![
                "The card is an article labelled by the model; specs and prices are description lists.",
                "The compare view is a table with row and column headers; the better value has a check and the words better by, so it never relies on colour.",
                "Benchmark rows are focusable and read the full readout (score, rank, gap to the leader); the bars are decorative.",
                "Unsupported modalities are dashed as well as muted and read not supported.",
                "Loading sets aria-busy; errors use role alert; the polite region says what changed.",
            ],
            vec![
                "Details, compare, loading and error share one grid cell, so the card never changes size.",
                "Under 560 px specs become label and value rows, the cost moves under the slider and each bar moves under its name.",
                "The compare table scrolls inside its own frame rather than the page.",
            ],
            560,
        ),
        ai(
            "ai-workflow",
            "AI workflow",
            "AiWorkflow",
            "aikit/workflow.rs",
            "An automation card where a Gmail trigger, a Claude step, a branch and Slack and Linear actions sit on one hairline rail, and a test run glides down it, opening each step's output inline. Steps come from a compact command menu with app marks.",
            vec![
                sel(
                    "scenario",
                    "Scenario",
                    &["passes", "fails", "empty", "loading"],
                    "passes",
                ),
                tog("autoplay", "Test run on load", true),
                tog("editable", "Editable", true),
                tog("gate", "Approval before actions", false),
            ],
            vec![
                var(
                    "passes",
                    "A test run that passes; the untaken path is skipped.",
                    &[],
                ),
                var(
                    "fails",
                    "The Linear step fails; Retry resumes from it.",
                    &[("scenario", "fails")],
                ),
                var(
                    "approval-gate",
                    "The run waits for your approval before the first action (Oxyn extra).",
                    &[("gate", "true")],
                ),
                var(
                    "empty",
                    "No steps yet: pick a trigger.",
                    &[("scenario", "empty"), ("autoplay", "false")],
                ),
            ],
            workflow,
            workflow_code,
            vec![
                (
                    "Tab",
                    "Test run (or Stop), each remove button, the branch tabs, Retry, Add step",
                ),
                ("Enter / Space", "Run or stop, remove, retry, open the menu"),
                ("← / → · Home / End", "Move between branch paths"),
                ("Type", "Filter the add menu"),
                ("↑ / ↓ · Enter", "Move in the menu and add"),
                ("1–9", "With an empty search, add that step"),
                ("Escape", "Close the menu"),
            ],
            vec![
                "A labelled section with aria-busy while running; steps are an ordered list and each branch path its own list.",
                "Each step's name carries its app and status; status is an icon plus a word.",
                "The add menu is a combobox with aria-activedescendant over a listbox; branch paths are a radio group.",
                "Remove buttons are named per step and appear on hover or focus (always on touch).",
                "Run start, results, failures and the outcome (simulated, nothing was sent) are announced politely.",
            ],
            vec![
                "Rows are 36 px with the detail inline; under 520 px the detail drops under the title and the status slot narrows.",
                "Status labels sit in a fixed 72 px slot so titles never shift.",
                "The add menu opens above the button and caps its height with a thin scrollbar.",
            ],
            470,
        ),
        ai(
            "image-generation",
            "Image generation",
            "ImageGeneration",
            "aikit/imagegen.rs",
            "An image studio where each image forms on a dot matrix: a soft wave, then the colours of the coming image as a halftone, then a blur into the sharp image. A filmstrip picks one of four, with Upscale, Vary and Download.",
            vec![
                tog("autoplay", "Generate on load", true),
                sel("aspect", "Aspect", &["1:1", "4:5", "16:9"], "1:1"),
                sel(
                    "placeholder",
                    "Placeholder",
                    &["dots", "blur", "scan"],
                    "dots",
                ),
                tog("grid", "Batch grid", false),
                tog("fail", "One variation fails", false),
            ],
            vec![
                var(
                    "dot-matrix",
                    "Four variations forming on the dot matrix.",
                    &[],
                ),
                var(
                    "blur-up",
                    "Blur-up placeholder in 16:9 (Oxyn extra).",
                    &[("placeholder", "blur"), ("aspect", "16:9")],
                ),
                var(
                    "batch-grid",
                    "All four in a grid with scan lines (Oxyn extra).",
                    &[("grid", "true"), ("placeholder", "scan")],
                ),
                var(
                    "failed",
                    "The third variation fails; the others finish.",
                    &[("fail", "true"), ("aspect", "4:5")],
                ),
            ],
            imagegen,
            imagegen_code,
            vec![
                ("Cmd/Ctrl + Enter", "Generate from the prompt"),
                (
                    "← / →",
                    "Model chips, aspect chips or variations in the filmstrip",
                ),
                ("Escape", "Stop a running batch; finished images are kept"),
                (
                    "Tab",
                    "Variations, Upscale, Vary, Download, then prompt, model, aspect, Generate",
                ),
            ],
            vec![
                "Model and aspect chips and the filmstrip are radio groups with a roving tab stop.",
                "The canvas is labelled with the image or the variation's state; thumbnails are named Variation 1 to 4.",
                "Starts, finishes, failures, picks, upscales and downloads are announced; results say they are simulated.",
                "Failures pair an alert icon with the word Failed; the chosen thumbnail has an outline as well as aria-checked.",
            ],
            vec![
                "Fills up to 600 px; the canvas keeps the batch's ratio and caps its height at 440 px.",
                "Under 500 px the actions drop their labels to icons so four thumbnails always fit beside them.",
                "Under 440 px the chips wrap onto two rows and Generate follows them.",
            ],
            620,
        ),
        ai(
            "speech-player",
            "Speech player",
            "SpeechPlayer",
            "aikit/speech.rs",
            "Read aloud for an AI reply: Read aloud turns the action row into a compact player with play, a scrubbable bar, speed and a voice picker, and a highlight follows the spoken word through the reply.",
            vec![
                tog("autoplay", "Read on load", true),
                sel("variant", "Variant", &["card", "plain"], "card"),
                sel("voice", "Voice", &["Ava", "Leo", "Maya", "Noah"], "Ava"),
                tog("karaoke", "Dim upcoming words", true),
                tog("fail", "First prepare fails", false),
            ],
            vec![
                var("reading", "Reading with Ava, word by word.", &[]),
                var(
                    "plain",
                    "Inside a chat column, full-contrast text (Oxyn extra).",
                    &[("variant", "plain"), ("karaoke", "false"), ("voice", "Leo")],
                ),
                var(
                    "idle",
                    "The action row before Read aloud.",
                    &[("autoplay", "false")],
                ),
                var(
                    "failed",
                    "Couldn't generate audio, with Retry.",
                    &[("fail", "true")],
                ),
            ],
            speech,
            speech_code,
            vec![
                (
                    "Tab",
                    "Read aloud and actions, or play, the bar, speed, voice and Stop reading",
                ),
                (
                    "Enter / Space",
                    "Read aloud starts reading; play pauses and resumes",
                ),
                ("← / →", "On the bar, two seconds back or forward"),
                ("Page up / down · Home / End", "Ten seconds, or the ends"),
                ("Space / K", "On the bar, play or pause"),
                ("↑ / ↓ · Enter · Escape", "In the voice menu"),
            ],
            vec![
                "The player is a labelled group; the play button's name follows its state (Play, Pause, Play again, Retry, Preparing audio).",
                "The bar is a slider with a value text like 0:04 of 0:22.",
                "The polite region says what changed and that the sample is simulated.",
                "Dimmed upcoming words keep enough contrast; the error pairs an icon with words.",
                "The hidden layer of the dock is out of the tab order.",
            ],
            vec![
                "Read aloud and the player share one 32 px slot, so the card never changes height.",
                "Under 440 px the total time steps aside so the bar keeps room.",
                "On touch screens the row grows to 36 px.",
            ],
            360,
        ),
        ai(
            "meeting-summary",
            "Meeting summary",
            "MeetingSummary",
            "aikit/meeting.rs",
            "AI notes for one call: Summary, Decisions and Action items stream in after a short summarizing pass; each line opens its transcript quote and each action item can be checked off or sent to Linear or Notion.",
            vec![
                sel(
                    "scenario",
                    "Scenario",
                    &["play", "ready", "error", "empty"],
                    "play",
                ),
                tog("send", "Send buttons", true),
                tog("fail", "First Notion send fails", false),
                tog("brief", "Brief notes", false),
            ],
            vec![
                var("streaming", "Summarizing, then the notes stream in.", &[]),
                var(
                    "brief",
                    "Brief notes: summary and action items (Oxyn extra).",
                    &[("brief", "true"), ("scenario", "ready")],
                ),
                var(
                    "send-retry",
                    "A failed Notion send with Retry.",
                    &[("scenario", "ready"), ("fail", "true")],
                ),
                var(
                    "error",
                    "Couldn't summarize, with Try again.",
                    &[("scenario", "error")],
                ),
            ],
            meeting,
            meeting_code,
            vec![
                (
                    "Tab",
                    "Attendees, each timestamp, checkbox and send button, then Brief, Copy, Regenerate",
                ),
                (
                    "Enter / Space",
                    "Open or pin a quote, toggle a checkbox, send",
                ),
                ("Escape", "Close the quote"),
            ],
            vec![
                "An article named after the meeting with h2 and h3 headings.",
                "Each time chip toggles its transcript quote (aria-expanded) and points to it with aria-describedby.",
                "Checkboxes are role checkbox with aria-checked and the item text as the name; the hit area is 32 px.",
                "A failed send shows an alert icon with Retry; a sent item shows the target mark and its reference.",
                "Summarizing, the finished notes, checks and sends are announced.",
            ],
            vec![
                "Holds from 360 px to 640 px; under 460 px the attendees move under the date.",
                "Quotes open below their line inside the card.",
                "Owner, date and send controls reserve their space, so lines never rewrap as they finish.",
            ],
            560,
        ),
        ai(
            "reply-draft",
            "Reply draft",
            "ReplyDraft",
            "aikit/reply.rs",
            "An AI reply drafted under a received email: quick replies steer a fresh draft, tone and length rewrite it with one crossfade, and the finished draft can be edited before Insert.",
            vec![
                tog("autostart", "Draft on load", true),
                sel(
                    "tone",
                    "Tone",
                    &["friendly", "concise", "formal"],
                    "friendly",
                ),
                sel("length", "Length", &["short", "medium", "long"], "medium"),
                tog("history", "Draft history", false),
            ],
            vec![
                var("drafting", "The first draft streams in.", &[]),
                var(
                    "formal-long",
                    "A long, formal draft with earlier drafts to restore (Oxyn extra).",
                    &[("tone", "formal"), ("length", "long"), ("history", "true")],
                ),
                var(
                    "empty",
                    "No draft yet: Draft a reply.",
                    &[("autostart", "false")],
                ),
            ],
            reply,
            reply_code,
            vec![
                (
                    "Tab",
                    "Quick replies, the draft, tone, length, Regenerate, Insert",
                ),
                ("Arrows · Home / End", "In tone or length, move and rewrite"),
                ("Enter / Space", "Activate a quick reply or button"),
                (
                    "Cmd/Ctrl + Enter",
                    "Insert the finished draft from anywhere in the card",
                ),
            ],
            vec![
                "An article labelled with the sender; the reply area is a labelled section with aria-busy while drafting.",
                "Quick replies are toggle buttons; tone and length are radio groups with a roving tab stop.",
                "The finished draft is an editable text box; Insert takes the edits.",
                "Drafting, rewriting, ready and inserted are announced; simulated actions say nothing was sent.",
            ],
            vec![
                "Fills up to 600 px; under 480 px the padding drops and the body loses its avatar indent.",
                "Quick replies and the tone and length groups wrap; under 460 px the divider hides.",
                "The draft box reserves its height, so a rewrite never jumps.",
            ],
            600,
        ),
        ai(
            "human-handoff",
            "Human handoff",
            "HumanHandoff",
            "aikit/handoff.rs",
            "A support chat where an AI assistant hands over to a person: a transfer card shows the summary it passes on and the typical wait, then the person joins and takes over, with a thin timeline of each stage.",
            vec![
                sel(
                    "scenario",
                    "Stage",
                    &["handoff", "connected", "unavailable", "resolved"],
                    "handoff",
                ),
                tog("edit", "Edit summary first", false),
                tog("timeline", "Stage timeline", true),
            ],
            vec![
                var(
                    "assistant",
                    "The assistant answers; Talk to a person hands over.",
                    &[],
                ),
                var(
                    "connected",
                    "The person has joined and taken over.",
                    &[("scenario", "connected")],
                ),
                var(
                    "editable-summary",
                    "Edit the summary before the transfer, with a queue place (Oxyn extra).",
                    &[("edit", "true")],
                ),
                var(
                    "unavailable",
                    "Nobody online: try again or keep chatting.",
                    &[("scenario", "unavailable")],
                ),
            ],
            handoff,
            handoff_code,
            vec![
                (
                    "Tab",
                    "Suggested replies, Talk to a person, Cancel transfer, the composer",
                ),
                ("Enter", "Send a message"),
                ("Enter / Space", "Activate a button"),
            ],
            vec![
                "The thread is a log, so new messages are read in order; typing is announced as text, not only dots.",
                "The stage timeline is an ordered list with aria-current on the current stage.",
                "The unavailable card uses role alert; every action is said back politely.",
                "Assistant and person are told apart by name and placement, not colour alone.",
            ],
            vec![
                "The thread has a fixed 300 px height and scrolls inside (thin scrollbar).",
                "Under 480 px the stage timeline runs across the top instead of down the side.",
                "Messages wrap long words instead of overflowing.",
            ],
            520,
        ),
        ai(
            "pii-redaction",
            "PII redaction",
            "PiiRedaction",
            "aikit/pii.rs",
            "A guardrail card that redacts personal data before a prompt reaches the model: Scan marks each name, email, phone number and card, Redact swaps them for placeholders like [EMAIL_1], and the reply comes back restored.",
            vec![
                tog("autoplay", "Scan on load", true),
                tog("editable", "Editable message", true),
                tog("categories", "Category toggles", false),
            ],
            vec![
                var("scan", "Scanning the sample message.", &[]),
                var(
                    "categories",
                    "Choose what to redact (Oxyn extra).",
                    &[("categories", "true")],
                ),
                var(
                    "edit",
                    "Edit the message, then Scan.",
                    &[("autoplay", "false")],
                ),
            ],
            pii,
            pii_code,
            vec![
                ("Tab", "Show original, the message, then the next step"),
                ("Cmd/Ctrl + Enter", "Scan, Redact, Send or edit again"),
                ("Space / Enter", "Toggle Show original or a category"),
            ],
            vec![
                "Each found item reads its value and type, or its type and redacted once swapped.",
                "The type counts read Checking while scanning, then the count.",
                "Show original is a switch; categories are toggle buttons.",
                "The swap, the send and the reply are announced; the sample says it is simulated.",
            ],
            vec![
                "Placeholders and originals share one inline grid cell, so the swap never rewraps the message.",
                "Under 480 px the type counts become two columns.",
                "The safe prompt line ellipses rather than wrapping.",
            ],
            460,
        ),
        ai(
            "llm-cost-calculator",
            "LLM cost calculator",
            "LlmCostCalculator",
            "aikit/cost.rs",
            "A pricing calculator for LLM workloads: set tokens per request, requests per day and prompt caching, and eight models re-rank by monthly cost while bars glide to their share of the most expensive.",
            vec![
                sel("preset", "Preset", &["support", "rag", "agent"], "support"),
                tog("batch", "Batch pricing switch", false),
                tog("loading", "Loading", false),
                tog("error", "Error", false),
            ],
            vec![
                var("support-chat", "A support chat workload.", &[]),
                var(
                    "coding-agent",
                    "A coding agent with heavy caching and the batch switch (Oxyn extra).",
                    &[("preset", "agent"), ("batch", "true")],
                ),
                var("loading", "Loading prices.", &[("loading", "true")]),
            ],
            cost,
            cost_code,
            vec![
                ("Tab", "Presets, the four sliders, the model list"),
                ("← / →", "One step (Shift: ten)"),
                ("Page up / down · Home / End", "Ten steps, or the ends"),
                ("↑ / ↓", "On the list, select the next model by rank"),
            ],
            vec![
                "Sliders have a value text with their unit; the list is a listbox with aria-selected.",
                "Rows move by transform only, so screen readers keep the reading order of the list.",
                "The summary (cost a month, per request, how many times the cheapest) is announced when the selection changes.",
            ],
            vec![
                "Sliders sit beside the ranking; under 600 px they stack and the bars hide, keeping name, rank and cost.",
                "Costs use tabular numerals in a fixed column, so re-ranking never shifts text.",
            ],
            430,
        ),
        ai(
            "provider-keys",
            "Provider keys",
            "ProviderKeys",
            "aikit/keys.rs",
            "A bring-your-own-key settings card: each provider row shows its mark, the masked key, a status and when it was last used; reveal for five seconds, copy, rotate or remove with Undo, and add a key that is tested first.",
            vec![
                tog("usage", "Spend and rate limits", true),
                tog("adding", "Add form open", false),
                tog("empty", "No keys", false),
            ],
            vec![
                var(
                    "keys",
                    "Three keys: active, near the limit and invalid.",
                    &[],
                ),
                var(
                    "add",
                    "Adding a key, tested before save (Oxyn extra spend readout off).",
                    &[("adding", "true"), ("usage", "false")],
                ),
                var("empty", "No keys yet.", &[("empty", "true")]),
            ],
            keys,
            keys_code,
            vec![
                (
                    "Tab",
                    "Add key, then each row's Reveal, Copy, Rotate, Remove",
                ),
                ("Enter", "Save the form"),
                ("Escape", "Close the form"),
            ],
            vec![
                "Each key reads its provider and whether it is shown or hidden; Reveal is a toggle button with a countdown.",
                "Status is a word with an icon or dot, never colour alone.",
                "Removing announces Undo; the test result is announced.",
                "The key input is a password field with aria-invalid after a failed test.",
            ],
            vec![
                "Under 480 px the actions move under the key.",
                "Long keys and usage lines ellipse instead of wrapping.",
            ],
            440,
        ),
        ai(
            "generation-stats",
            "Generation stats",
            "GenerationStats",
            "aikit/genstats.rs",
            "A live stats strip for a streaming reply: time to first token, speed with a sparkline, output tokens and cost count up beside the model's mark, then settle and open into a latency breakdown.",
            vec![
                sel(
                    "scenario",
                    "Scenario",
                    &["stream", "done", "idle", "error"],
                    "stream",
                ),
                sel("variant", "Variant", &["card", "inline"], "card"),
                sel(
                    "model",
                    "Model",
                    &["Claude Sonnet 4.5", "GPT-5", "Gemini 2.5 Flash"],
                    "Claude Sonnet 4.5",
                ),
                tog("loop", "Loop", false),
                tog("session", "Session totals", false),
            ],
            vec![
                var("streaming", "Counting up while the reply streams.", &[]),
                var(
                    "breakdown",
                    "Finished, with the latency breakdown and session totals (Oxyn extra).",
                    &[("scenario", "done"), ("session", "true")],
                ),
                var(
                    "inline",
                    "A compact chip under a message (Oxyn extra).",
                    &[("variant", "inline"), ("model", "GPT-5"), ("loop", "true")],
                ),
                var(
                    "error",
                    "Failed part way; what arrived is kept.",
                    &[("scenario", "error")],
                ),
            ],
            genstats,
            genstats_code,
            vec![
                ("Tab", "Stop or Run, then the breakdown toggle"),
                (
                    "Enter / Space",
                    "Stop, run again, retry or open the breakdown",
                ),
            ],
            vec![
                "Each value is a labelled term; numbers are tabular in reserved widths, so updates never shift.",
                "The status is a word with an icon; the error uses role alert.",
                "The breakdown toggle has aria-expanded; the sparkline is decorative.",
            ],
            vec![
                "The strip wraps under 520 px; Stop and the toggle follow the stats.",
                "Inline drops labels and the sparkline to fit under a message.",
            ],
            300,
        ),
        ai(
            "ai-translate",
            "AI translate",
            "AiTranslate",
            "aikit/translate.rs",
            "A message card where picking a language streams the translation in place, sentence by sentence over the dimmed original, with a detected-language chip, Show original and a status line credited to the model.",
            vec![
                sel("lang", "Language", &LANGS, "German"),
                tog("side", "Side by side", false),
                tog("fail", "First translation fails", false),
            ],
            vec![
                var("german", "Translating to German.", &[]),
                var(
                    "side-by-side",
                    "Japanese beside the original (Oxyn extra).",
                    &[("lang", "Japanese"), ("side", "true")],
                ),
                var(
                    "failed",
                    "Couldn't translate, with Try again.",
                    &[("fail", "true"), ("lang", "French")],
                ),
            ],
            translate,
            translate_code,
            vec![
                ("Enter / Space / ↓", "Open the language menu"),
                ("↑ / ↓ · Home / End", "Move in the menu"),
                ("Enter", "Pick a language"),
                ("Escape", "Close the menu"),
            ],
            vec![
                "The menu is a listbox with aria-selected; the text carries a lang attribute for the language shown.",
                "Translating, done and errors are announced; the error uses role alert.",
                "Show original and side by side are toggle buttons.",
            ],
            vec![
                "Each finished sentence replaces one dimmed sentence, so the card does not jump.",
                "Side by side stacks under 520 px.",
            ],
            320,
        ),
        ai(
            "tldr-toggle",
            "TL;DR toggle",
            "TldrToggle",
            "aikit/tldr.rs",
            "An article card with a Full, Summary and Key points switch and a length control: switching streams an AI summary in place, and each key point jumps back to the paragraph it came from.",
            vec![
                sel("mode", "View", &["full", "summary", "points"], "summary"),
                sel("length", "Length", &["short", "medium", "long"], "medium"),
                tog("fail", "Summary fails", false),
                tog("saved", "Words saved", true),
            ],
            vec![
                var("summary", "A medium summary streams in.", &[]),
                var(
                    "key-points",
                    "Five key points that jump to their paragraph.",
                    &[("mode", "points"), ("length", "long")],
                ),
                var("full", "The full article.", &[("mode", "full")]),
                var(
                    "failed",
                    "Couldn't summarize, with Try again.",
                    &[("fail", "true")],
                ),
            ],
            tldr,
            tldr_code,
            vec![
                (
                    "Tab",
                    "The view switch, the length control, key point links",
                ),
                ("Arrows · Home / End", "Move in either group"),
                ("Enter / Space", "Jump to a key point's paragraph"),
            ],
            vec![
                "Both controls are radio groups with a roving tab stop.",
                "The body has aria-busy while summarizing; the jumped-to paragraph is highlighted and announced.",
                "Errors use role alert with Try again.",
            ],
            vec![
                "The body follows its content's height; the controls wrap under 480 px.",
                "Key points wrap; their jump buttons stay at the end of the line.",
            ],
            460,
        ),
        ai(
            "connector-setup",
            "Connector setup",
            "ConnectorSetup",
            "aikit/connector.rs",
            "A Connect your tools card for an AI assistant: each app connects inline, authorizes and runs a first sync with a progress hairline, then lists the scopes it was granted and offers Disconnect with an inline confirm.",
            vec![
                tog("autoplay", "Connect Notion on load", true),
                tog("deny", "HubSpot grant denied", false),
                tog("search", "Search", true),
            ],
            vec![
                var("connecting", "Notion connects and syncs.", &[]),
                var("denied", "A denied grant with Retry.", &[("deny", "true")]),
                var(
                    "compact",
                    "No search, nothing connecting (Oxyn extra).",
                    &[("search", "false"), ("autoplay", "false")],
                ),
            ],
            connector,
            connector_code,
            vec![
                (
                    "Tab",
                    "Search, then each app's Connect, access toggle, Disconnect",
                ),
                ("Enter / Space", "Connect, retry, confirm"),
                ("Escape", "Cancel a Disconnect confirm"),
            ],
            vec![
                "Status lines are text (Opening Notion, Syncing 120 of 642 pages, 642 pages synced); progress is decorative.",
                "Connected and denied are words with icons; denial uses role alert.",
                "Disconnect asks inline before it acts.",
            ],
            vec![
                "Status and actions share fixed slots, so rows never reflow; counts use tabular numerals.",
                "Under 480 px the actions drop under the name.",
            ],
            520,
        ),
        ai(
            "enrich-column",
            "Enrich column",
            "EnrichColumn",
            "aikit/enrich.rs",
            "An AI enrichment column for a data table: one prompt runs down every row, each cell shimmers while it works, then settles into a value with its source or a failure you can retry.",
            vec![
                sel(
                    "column",
                    "Column",
                    &["industry", "employees", "city"],
                    "industry",
                ),
                rng("concurrency", "At a time", 1.0, 6.0, 1.0, "3"),
                tog("autoplay", "Run on load", true),
            ],
            vec![
                var("industry", "Industry, three cells at a time.", &[]),
                var(
                    "one-at-a-time",
                    "Employee counts one by one (Oxyn extra).",
                    &[("column", "employees"), ("concurrency", "1")],
                ),
                var(
                    "idle",
                    "HQ city, not run yet.",
                    &[("column", "city"), ("autoplay", "false")],
                ),
            ],
            enrich,
            enrich_code,
            vec![
                ("Tab", "Run, the column menu, retry buttons, concurrency"),
                ("Cmd/Ctrl + Enter", "Run the column"),
                ("Escape", "Stop, or close the menu"),
            ],
            vec![
                "A real table with column headers; the AI column's menu is a listbox.",
                "Cells read Queued, Looking up, the value with its source, or Not found with a named Retry.",
                "Runs, stops and retries are announced.",
            ],
            vec![
                "The table keeps a 480 px minimum and scrolls inside its frame (thin scrollbar).",
                "Cells swap content in place, so nothing changes size.",
            ],
            420,
        ),
        ai(
            "ticket-router",
            "Ticket router",
            "TicketRouter",
            "aikit/router.rs",
            "A support ticket router: messages from Gmail, Intercom, Slack and GitHub come in one at a time, an AI classifier labels each with intent, sentiment and confidence, and the ticket moves into a team lane; low confidence waits for review.",
            vec![
                rng("threshold", "Review below %", 40.0, 95.0, 5.0, "70"),
                tog("paused", "Start paused", false),
                tog("tuning", "Threshold control", true),
            ],
            vec![
                var("routing", "Tickets routed live.", &[]),
                var(
                    "strict",
                    "A stricter threshold sends more to review (Oxyn extra).",
                    &[("threshold", "90")],
                ),
                var(
                    "paused",
                    "Paused before the first ticket.",
                    &[("paused", "true"), ("tuning", "false")],
                ),
            ],
            router,
            router_code,
            vec![
                (
                    "Tab",
                    "Pause, restart, speed, review buttons, the threshold",
                ),
                (
                    "Enter / Space",
                    "Pause or resume, move a ticket out of review",
                ),
                ("← / →", "On the threshold slider"),
            ],
            vec![
                "Labels are words (intent, sentiment, confidence), never colour alone; low confidence is also amber.",
                "Lanes show a count and the latest tickets; review buttons are named with the ticket and lane.",
                "Pause, resume, reviews and threshold changes are announced.",
            ],
            vec![
                "Four lanes in a row; under 600 px two by two.",
                "Ticket subjects ellipse; counts use tabular numerals.",
            ],
            470,
        ),
    ]
}
