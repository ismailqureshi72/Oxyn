//! Agent micro-interactions first seen in posts the founders saved (C2a, free): Test ticks,
//! Triage metro, Loop pipeline, Secret card, Split confirm, Agent prompt copy, Proactive card,
//! Write sweep, Done shockwave and Publish popover. Every demo runs on the host clock (`live`).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{
    AgentPromptCopy, DoneShockwave, LoopPipeline, ProactiveCard, PublishPopover, SecretCard,
    SplitConfirm, TestTicks, TriageMetro, WriteSweep,
};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Control, Demo, Vals, Variant};

#[allow(clippy::too_many_arguments)]
fn xp(
    slug: &'static str,
    name: &'static str,
    cat: &'static str,
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
        ..mk(slug, name, cat, summary, dioxus, "XPostView", slug, src)
    }
}

fn clock(name: &str) -> Code {
    Code::new(name)
        .pre("let clock = use_motion_clock(); // the host clock (ClockMs)")
        .raw("now", "clock.now()")
}

fn idx(v: &Vals, k: &str, opts: &[&str]) -> usize {
    opts.iter().position(|o| *o == v.s(k)).unwrap_or(0)
}

// ── Test ticks ──
fn ticks(v: &Vals, now: ClockMs) -> Element {
    let k = format!("{}{}", v.b("fail"), v.f("speed"));
    rsx! { TestTicks { key: "{k}", now, fail: v.b("fail"), speed: v.f("speed"), show_rate: v.b("rate"), compact: v.b("compact") } }
}
fn ticks_code(v: &Vals) -> String {
    let mut c = clock("TestTicks").b("fail", v.b("fail"));
    if (v.f("speed") - 1.0).abs() > f64::EPSILON {
        c = c.raw("speed", format!("{:.1}", v.f("speed")));
    }
    if !v.b("rate") {
        c = c.raw("show_rate", "false");
    }
    c.b("compact", v.b("compact")).done()
}

// ── Triage metro ──
fn metro(v: &Vals, now: ClockMs) -> Element {
    rsx! { TriageMetro { now, looping: v.b("loop"), labels: v.b("labels"), show_counts: v.b("counts") } }
}
fn metro_code(v: &Vals) -> String {
    let mut c = clock("TriageMetro");
    if !v.b("loop") {
        c = c.raw("looping", "false");
    }
    if !v.b("labels") {
        c = c.raw("labels", "false");
    }
    if !v.b("counts") {
        c = c.raw("show_counts", "false");
    }
    c.done()
}

// ── Loop pipeline ──
fn lp(v: &Vals, now: ClockMs) -> Element {
    let target = v.f("target") as u32;
    let max = v.f("max") as usize;
    rsx! { LoopPipeline { key: "{target}-{max}", now, target, max_rounds: max, show_history: v.b("hist") } }
}
fn lp_code(v: &Vals) -> String {
    let mut c = clock("LoopPipeline");
    if v.f("target") as u32 != 85 {
        c = c.raw("target", format!("{}", v.f("target") as u32));
    }
    if v.f("max") as usize != 5 {
        c = c.raw("max_rounds", format!("{}", v.f("max") as usize));
    }
    if !v.b("hist") {
        c = c.raw("show_history", "false");
    }
    c.done()
}

// ── Secret card ──
fn secret(v: &Vals, now: ClockMs) -> Element {
    let k = format!("{}", v.b("revoked"));
    rsx! { SecretCard { key: "{k}", now, revoked: v.b("revoked"), show_chat: v.b("chat"), compact: v.b("compact") } }
}
fn secret_code(v: &Vals) -> String {
    let mut c = clock("SecretCard").b("revoked", v.b("revoked"));
    if !v.b("chat") {
        c = c.raw("show_chat", "false");
    }
    c.b("compact", v.b("compact")).done()
}

// ── Split confirm ──
const SP_LABELS: [&str; 3] = ["approve", "send", "merge"];
fn split(v: &Vals, now: ClockMs) -> Element {
    let label = idx(v, "label", &SP_LABELS);
    let k = format!("{label}{}", v.b("fail"));
    rsx! { SplitConfirm { key: "{k}", now, label, fail: v.b("fail"), size: v.string("size"), context: v.b("ctx") } }
}
fn split_code(v: &Vals) -> String {
    let mut c = clock("SplitConfirm");
    let l = idx(v, "label", &SP_LABELS);
    if l != 0 {
        c = c.raw("label", l.to_string());
    }
    c.b("fail", v.b("fail")).s("size", v.s("size"), "md").done()
}

// ── Agent prompt copy ──
const GA_AGENTS: [&str; 5] = ["claude-code", "codex", "cursor", "copilot", "windsurf"];
const GA_FORMATS: [&str; 3] = ["prompt", "skill", "command"];
fn ga(v: &Vals, now: ClockMs) -> Element {
    let agent = idx(v, "agent", &GA_AGENTS);
    let format = idx(v, "format", &GA_FORMATS);
    let k = format!("{agent}-{format}{}", v.b("fail"));
    rsx! { AgentPromptCopy { key: "{k}", now, agent, format, preview: v.b("preview"), fail: v.b("fail") } }
}
fn ga_code(v: &Vals) -> String {
    let mut c = clock("AgentPromptCopy");
    let a = idx(v, "agent", &GA_AGENTS);
    if a != 0 {
        c = c.raw("agent", a.to_string());
    }
    let f = idx(v, "format", &GA_FORMATS);
    if f != 0 {
        c = c.raw("format", f.to_string());
    }
    if !v.b("preview") {
        c = c.raw("preview", "false");
    }
    c.b("fail", v.b("fail")).done()
}

// ── Proactive card ──
fn pc(v: &Vals, now: ClockMs) -> Element {
    let k = format!("{}", v.b("fail"));
    rsx! { ProactiveCard { key: "{k}", now, fail: v.b("fail"), in_feed: v.b("feed") } }
}
fn pc_code(v: &Vals) -> String {
    let mut c = clock("ProactiveCard").b("fail", v.b("fail"));
    if !v.b("feed") {
        c = c.raw("in_feed", "false");
    }
    c.done()
}

// ── Write sweep ──
fn ws(v: &Vals, now: ClockMs) -> Element {
    let ms = v.f("ms") as u64;
    let k = format!("{ms}{}", v.b("auto"));
    rsx! { WriteSweep { key: "{k}", now, duration_ms: ms, autoplay: v.b("auto"), tone: v.string("tone"), show_before: v.b("before") } }
}
fn ws_code(v: &Vals) -> String {
    let mut c = clock("WriteSweep");
    if v.f("ms") as u64 != 1_800 {
        c = c.raw("duration_ms", format!("{}", v.f("ms") as u64));
    }
    if !v.b("auto") {
        c = c.raw("autoplay", "false");
    }
    c.s("tone", v.s("tone"), "aurora")
        .b("show_before", v.b("before"))
        .done()
}

// ── Done shockwave ──
fn sw(v: &Vals, now: ClockMs) -> Element {
    let k = format!("{}{}{}", v.b("fail"), v.f("speed"), v.b("auto"));
    rsx! { DoneShockwave { key: "{k}", now, fail: v.b("fail"), strength: v.f("strength"), speed: v.f("speed"), autoplay: v.b("auto"), reduced: v.b("reduced") } }
}
fn sw_code(v: &Vals) -> String {
    let mut c = clock("DoneShockwave").b("fail", v.b("fail"));
    if (v.f("strength") - 1.0).abs() > f64::EPSILON {
        c = c.raw("strength", format!("{:.1}", v.f("strength")));
    }
    if !v.b("auto") {
        c = c.raw("autoplay", "false");
    }
    c.b("reduced", v.b("reduced")).done()
}

// ── Publish popover ──
fn pp(v: &Vals, now: ClockMs) -> Element {
    let k = format!("{}{}", v.b("open"), v.b("taken"));
    rsx! { PublishPopover { key: "{k}", now, open: v.b("open"), taken: v.b("taken") } }
}
fn pp_code(v: &Vals) -> String {
    let mut c = clock("PublishPopover");
    if !v.b("open") {
        c = c.raw("open", "false");
    }
    c.b("taken", v.b("taken")).done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        xp(
            "test-ticks",
            "Test ticks",
            "tools",
            "TestTicks",
            "xpost/run.rs",
            "A test run as one small tick per test, a row per file: ticks fill as results arrive, the timer, pass and fail counters and the rate roll with the run, and a failing run dims the passes and opens a fix card with a diff that re-runs only what failed.",
            vec![
                tog("fail", "Two tests fail", true),
                rng("speed", "Speed", 0.5, 3.0, 0.5, "1.0"),
                tog("rate", "Tests per second", true),
                tog("compact", "Compact grid", false),
            ],
            vec![
                var(
                    "failing",
                    "Two failures, then the fix card over the failing file.",
                    &[],
                ),
                var(
                    "all-pass",
                    "A clean run that ends green.",
                    &[("fail", "false")],
                ),
                var(
                    "compact",
                    "Grid only, no file names, fast (Oxyn extra).",
                    &[("compact", "true"), ("speed", "2.0"), ("fail", "false")],
                ),
            ],
            ticks,
            ticks_code,
            vec![
                ("← → ↑ ↓ · Home / End", "Move between tests in the grid"),
                ("Enter", "Apply the fix while the fix card is open"),
                ("Escape", "Close the fix card, then clear the selection"),
            ],
            vec![
                "The grid is focusable and announces each test as you move: name, result and time.",
                "Results never rely on colour alone: counters and the status line carry the numbers, failures are slightly taller.",
                "The grid is marked busy while the run is going; the fix card is a labelled dialog.",
            ],
            vec![
                "Under 520 px the file name sits above its ticks and ticks wrap.",
                "The fix card narrows to the grid width.",
            ],
            380,
        ),
        xp(
            "triage-metro",
            "Triage metro",
            "tools",
            "TriageMetro",
            "xpost/run.rs",
            "Security findings ride a metro map from Scan to Triage and branch to Patched, False positive or Needs you; terminal counters roll as dots land, and a finding waiting on you can be patched or dismissed from its card.",
            vec![
                tog("loop", "Loop when done", true),
                tog("labels", "Station labels", true),
                tog("counts", "Counters", true),
            ],
            vec![
                var("map", "The full map with labels and counters.", &[]),
                var(
                    "bare",
                    "Lines and dots only, for a dense dashboard tile (Oxyn extra).",
                    &[("labels", "false"), ("counts", "false")],
                ),
                var(
                    "once",
                    "Runs once and stays, so you can work the Needs you stop.",
                    &[("loop", "false")],
                ),
            ],
            metro,
            metro_code,
            vec![
                ("← → ↑ ↓ · Home / End", "Move between findings on the map"),
                ("Space", "Pause or resume"),
                ("Escape", "Close the open finding"),
            ],
            vec![
                "Every dot has a button with the finding, its file, severity and where it ended up.",
                "Terminals are named in text, not just colour; counters sit under the map.",
                "Moves are announced politely; Pause stops all motion.",
            ],
            vec![
                "The map scales with the card; at 390 px terminal names shrink and sit above their stop.",
                "The finding card wraps its file path and buttons.",
            ],
            420,
        ),
        xp(
            "loop-pipeline",
            "Loop pipeline",
            "tools",
            "LoopPipeline",
            "xpost/run.rs",
            "An agent working in rounds: a marker runs Work, Check and Improve round a loop, each round leaves a scored chip, and the loop exits to Done at the target or stops at the round limit with the best score and One more round.",
            vec![
                rng("target", "Target %", 60.0, 98.0, 1.0, "85"),
                rng("max", "Most rounds", 2.0, 6.0, 1.0, "5"),
                tog("hist", "Round chips", true),
            ],
            vec![
                var("passes", "Exits on round 4 at 88%.", &[]),
                var(
                    "runs-out",
                    "Target out of reach: stops after 3 rounds and offers one more.",
                    &[("target", "95"), ("max", "3")],
                ),
                var(
                    "first-try",
                    "Passes on the first round.",
                    &[("target", "60")],
                ),
            ],
            lp,
            lp_code,
            vec![
                ("Space", "Pause or resume"),
                ("Escape", "Stop the loop"),
                ("← → · Home / End", "Move between round chips"),
            ],
            vec![
                "The status line always names the round, the station and the score; the ring is decorative.",
                "Round chips are a list of buttons that speak their score.",
                "Pause holds the clock so nothing moves.",
            ],
            vec!["The loop and the score panel stack under 520 px."],
            330,
        ),
        xp(
            "secret-card",
            "Secret card",
            "approvals",
            "SecretCard",
            "xpost/trust.rs",
            "A key the agent can use but never sees: masked to its last four characters, the chat only gets a handle, Show needs a confirm and hides itself after five seconds, Rotate swaps it with a progress bar and Revoke pauses the routines that used it, with Undo.",
            vec![
                tog("revoked", "Start revoked", false),
                tog("chat", "Chat line", true),
                tog("compact", "Compact", false),
            ],
            vec![
                var("active", "An active key with its scopes and routines.", &[]),
                var(
                    "compact",
                    "Key row and actions only (Oxyn extra).",
                    &[("compact", "true"), ("chat", "false")],
                ),
                var(
                    "revoked",
                    "Revoked, routines paused, with Restore.",
                    &[("revoked", "true")],
                ),
            ],
            secret,
            secret_code,
            vec![
                ("Enter", "Confirm the pending action"),
                ("Escape", "Cancel a confirm, or hide the shown key"),
            ],
            vec![
                "The masked value is labelled as hidden except its last four characters.",
                "Every confirm is an alert dialog with its consequence spelled out.",
                "State changes (shown, rotating, revoked, restored) are announced.",
            ],
            vec!["Actions wrap under the status line; the key value truncates inside its box."],
            400,
        ),
        xp(
            "split-confirm",
            "Split confirm",
            "buttons",
            "SplitConfirm",
            "xpost/trust.rs",
            "A pill that confirms by splitting: it stretches with a small overshoot, a check bubble pinches off its end and the label turns to the past tense; the bubble is Undo for four seconds, and a failure shakes once with a cross and Try again.",
            vec![
                sel("label", "Label", &SP_LABELS, "approve"),
                tog("fail", "Fails", false),
                sel("size", "Size", &["md", "lg"], "md"),
                tog("ctx", "Context line", true),
            ],
            vec![
                var("approve", "Approve → Approved.", &[]),
                var(
                    "send-large",
                    "Send at the large size (Oxyn extra).",
                    &[("label", "send"), ("size", "lg")],
                ),
                var(
                    "fails",
                    "Merge that fails and offers Try again.",
                    &[("label", "merge"), ("fail", "true")],
                ),
            ],
            split,
            split_code,
            vec![
                ("Enter / Space", "Confirm, or try again after a failure"),
                ("Backspace · Cmd/Ctrl+Z", "Undo while the check is showing"),
                ("Escape", "Reset"),
            ],
            vec![
                "The pill is a real button; the bubble is a labelled Undo button while it can undo.",
                "Result text changes in place and is announced; failure reads 'Didn't go through'.",
                "Reduced motion drops the stretch and shake but keeps the bubble.",
            ],
            vec!["Fixed size; it never wraps."],
            200,
        ),
        xp(
            "agent-prompt-copy",
            "Agent prompt copy",
            "buttons",
            "AgentPromptCopy",
            "xpost/trust.rs",
            "A Give to your agent split button: the main part copies an agent-ready hand-off for the chosen coding agent, the chevron picks the agent (official marks) and the format (prompt, skill file, install command), and the preview shows exactly what is copied.",
            vec![
                sel("agent", "Agent", &GA_AGENTS, "claude-code"),
                sel("format", "Format", &GA_FORMATS, "prompt"),
                tog("preview", "Preview", true),
                tog("fail", "Clipboard blocked", false),
            ],
            vec![
                var("prompt", "A prompt for the first agent.", &[]),
                var(
                    "skill",
                    "A skill file for Cursor.",
                    &[("agent", "cursor"), ("format", "skill")],
                ),
                var(
                    "button-only",
                    "Just the button (Oxyn extra).",
                    &[("preview", "false"), ("format", "command")],
                ),
            ],
            ga,
            ga_code,
            vec![
                (
                    "Enter / Space",
                    "Copy (menu closed) or pick the row (menu open)",
                ),
                ("↓", "Open the menu"),
                ("↑ ↓ · Home / End", "Move in the menu"),
                ("Escape", "Close the menu"),
            ],
            vec![
                "The menu uses radio items for agent and format, each with its checked state.",
                "Copy results are announced, including when the clipboard is blocked.",
                "The preview is focusable so the text can be selected by keyboard.",
            ],
            vec![
                "The button drops under the title when space runs out; the preview wraps long lines.",
            ],
            380,
        ),
        xp(
            "proactive-card",
            "Proactive card",
            "chat",
            "ProactiveCard",
            "xpost/moments.rs",
            "The assistant drops an offer into a feed when it notices something: accept runs the steps in place and resolves into a result with Review, Not now opens a snooze menu that folds the card into one line with Undo, and the cross removes it.",
            vec![
                tog("fail", "A step fails", false),
                tog("feed", "Between feed posts", true),
            ],
            vec![
                var("offer", "The offer between two posts.", &[]),
                var(
                    "alone",
                    "The card on its own (Oxyn extra).",
                    &[("feed", "false")],
                ),
                var(
                    "step-fails",
                    "The second step fails and offers Try again.",
                    &[("fail", "true")],
                ),
            ],
            pc,
            pc_code,
            vec![
                ("↑ ↓ · Home / End", "Move in the snooze menu"),
                ("Enter", "Pick a snooze time"),
                ("Escape", "Close the snooze menu"),
            ],
            vec![
                "The card is an article named by its title; the steps list is marked busy while it runs.",
                "Every outcome (snoozed, removed, failed, done) is announced and can be undone where it makes sense.",
            ],
            vec!["Actions wrap; feed posts wrap their text."],
            440,
        ),
        xp(
            "write-sweep",
            "Write sweep",
            "status-text",
            "WriteSweep",
            "xpost/moments.rs",
            "An 'agent is writing here' pass: a soft band of light sweeps down the region being changed, each row it crosses blurs out its old value and settles into the new one, and the footer counts the changes with Undo.",
            vec![
                rng("ms", "Sweep ms", 800.0, 4000.0, 200.0, "1800"),
                tog("auto", "Autoplay", true),
                sel("tone", "Band", &["aurora", "mono"], "aurora"),
                tog("before", "Show old values", false),
            ],
            vec![
                var("aurora", "A soft colour band.", &[]),
                var(
                    "mono",
                    "A grey band for quiet UIs (Oxyn extra).",
                    &[("tone", "mono")],
                ),
                var(
                    "compare",
                    "Keeps the old value struck through.",
                    &[("before", "true"), ("ms", "2600")],
                ),
            ],
            ws,
            ws_code,
            vec![
                ("Enter / Space", "Run the update"),
                ("Cmd/Ctrl+Z", "Undo the changes"),
                ("Escape", "Skip the sweep and apply at once"),
            ],
            vec![
                "The region is marked busy while the band moves; the status line counts changes.",
                "The band is decorative and hidden for reduced motion; values still change.",
            ],
            vec!["Rows wrap the old value under the new one on narrow cards."],
            380,
        ),
        xp(
            "done-shockwave",
            "Done shockwave",
            "feedback",
            "DoneShockwave",
            "xpost/moments.rs",
            "When a long task finishes, two rings leave its status mark and roll across the panel, the mark becomes a check and a line says how long it took; a failure sends one short red pulse, and reduced motion swaps the rings for a still halo.",
            vec![
                tog("fail", "Fails", false),
                rng("strength", "Ring strength", 0.2, 1.0, 0.1, "1.0"),
                rng("speed", "Speed", 0.5, 3.0, 0.5, "1.0"),
                tog("auto", "Autoplay", true),
                tog("reduced", "Reduced motion", false),
            ],
            vec![
                var("done", "A deploy that lands with two rings.", &[]),
                var(
                    "soft",
                    "Faint rings for a busy page (Oxyn extra).",
                    &[("strength", "0.4")],
                ),
                var(
                    "fails",
                    "Fails with one short red pulse and Retry.",
                    &[("fail", "true")],
                ),
                var(
                    "still",
                    "Reduced motion: a still halo.",
                    &[("reduced", "true")],
                ),
            ],
            sw,
            sw_code,
            vec![
                ("Enter / Space", "Deploy, or retry after a failure"),
                ("Escape", "Reset"),
            ],
            vec![
                "Rings are decorative; the status line says what happened and the failure is an alert.",
                "Steps list their state in text.",
            ],
            vec!["Rings are clipped to the card; the card wraps its buttons under the title."],
            260,
        ),
        xp(
            "publish-popover",
            "Publish popover",
            "overlays",
            "PublishPopover",
            "xpost/moments.rs",
            "Publish in a document toolbar opens a popover that slides out of the button: page address, who can open it, a host with its official mark and two switches; publishing walks through upload, build and live, then shows the link with Copy, views and Unpublish.",
            vec![
                tog("open", "Start open", true),
                tog("taken", "Address taken", false),
            ],
            vec![
                var("form", "The popover open on the form.", &[]),
                var(
                    "taken",
                    "The address is taken; offers a free one.",
                    &[("taken", "true")],
                ),
                var(
                    "closed",
                    "Just the toolbar button (Oxyn extra).",
                    &[("open", "false")],
                ),
            ],
            pp,
            pp_code,
            vec![
                (
                    "Enter",
                    "Publish from the address field; copy the link when live",
                ),
                ("← → · Home / End", "Pick who can open it or the host"),
                ("Escape", "Close the popover"),
            ],
            vec![
                "The button reports the popover as an expanded dialog; the address field is labelled and flags a taken address.",
                "Radio groups for access and host rove with the arrows; switches report on or off.",
                "Publishing steps are announced; the busy area is marked busy.",
            ],
            vec!["Under 520 px the popover narrows to 300 px and stays inside the card."],
            520,
        ),
    ]
}
