//! Plan #32 (affectUI fidelity pass), overnight batch family A demos: the exact AI Answer, Cited
//! Answer, Message Actions, Thinking Loader, Thinking State and Writing Assistant, each with every
//! affectUI style and state, and Oxyn's earlier component for the same job as an extra.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{
    AnswerCard, CitedAnswerCard, MessageActionBar, ThinkingPanel, ThinkingPill, WritingAssistant,
};

use super::{Code, mk, sel, tog, var};
use crate::{Demo, Vals};

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

fn cstyle(v: &Vals) -> String {
    if v.s("style") == "classic" {
        "classic".into()
    } else {
        String::new()
    }
}

/// An earlier Oxyn demo shown as the "Oxyn extra" of a fidelity page.
#[component]
fn Legacy(slug: String, now: ClockMs) -> Element {
    let demo = use_hook(|| super::legacy_demo(&slug));
    let Some(d) = demo else {
        return rsx! { span { "{slug}" } };
    };
    let v = use_hook(|| d.values(None));
    rsx! { {(d.render)(&v, now)} }
}

fn on_act(mut msg: Signal<String>) -> impl FnMut((String, String)) {
    move |(a, t): (String, String)| {
        msg.set(if t.is_empty() {
            format!("onAction({a:?})")
        } else {
            let t: String = t.chars().take(48).collect();
            format!("onAction({a:?}, {t:?})")
        })
    }
}

// ---------- AI Answer ----------

fn ai_answer(v: &Vals, now: ClockMs) -> Element {
    let msg = use_signal(String::new);
    if v.s("kind") == "oxyn" {
        return rsx! { div { "data-k": "oxyn", style: "width:640px;max-width:100%", Legacy { slug: "ai-answer-oxyn", now } } };
    }
    let key = format!("{}{}{}", v.s("style"), v.s("start"), v.s("vote"));
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:480px;max-width:100%;display:flex;flex-direction:column;gap:8px;padding-top:8px",
            AnswerCard { now, autoplay: v.s("start") == "stream", vote: if v.s("start") == "done" { v.string("vote") } else { String::new() }, style: cstyle(v), on_action: on_act(msg) }
            {status(msg)}
        }
    }
}

fn ai_answer_code(v: &Vals) -> String {
    if v.s("kind") == "oxyn" {
        return Code::new("AiAnswer")
            .raw("answer", "answer()")
            .raw("now", "clock.now()")
            .done();
    }
    let mut c = Code::new("AnswerCard")
        .raw("question", "q.clone()")
        .raw("sources", "sources()")
        .raw("text", "answer_markup()")
        .raw("related", "follow_ups()");
    if v.s("start") != "stream" {
        c = c.b("autoplay", false);
    }
    c.raw("now", "clock.now()")
        .raw("on_action", "move |(what, detail)| handle(what, detail)")
        .done()
}

// ---------- Cited Answer ----------

fn cited_answer(v: &Vals, _: ClockMs) -> Element {
    let msg = use_signal(String::new);
    let lit = match v.s("lit") {
        "none" => None,
        "1" => Some(0),
        "3" => Some(2),
        _ => Some(1),
    };
    let key = format!("{}{}{}", v.s("style"), v.s("lit"), v.b("open"));
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:460px;max-width:100%;display:flex;flex-direction:column;gap:8px;padding-top:8px",
            CitedAnswerCard { lit, open: v.b("open") && lit.is_some(), style: cstyle(v), on_action: on_act(msg) }
            {status(msg)}
        }
    }
}

fn cited_answer_code(v: &Vals) -> String {
    let mut c = Code::new("CitedAnswerCard")
        .raw("header", "\"3 sources · 14 pages read\".into()")
        .raw("text", "markup()")
        .raw("sources", "sources()");
    if v.b("open") {
        c = c.raw("lit", "Some(1)").b("open", true);
    }
    c.raw("on_action", "move |(what, source)| handle(what, source)")
        .done()
}

// ---------- Message Actions ----------

fn message_actions(v: &Vals, now: ClockMs) -> Element {
    let msg = use_signal(String::new);
    if v.s("kind") == "oxyn" {
        return rsx! { div { "data-k": "oxyn", style: "width:520px;max-width:100%", Legacy { slug: "response-feedback", now } } };
    }
    let key = format!("{}{}{}", v.s("style"), v.b("feedback"), v.b("reading"));
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:544px;max-width:100%;display:flex;flex-direction:column;gap:8px;padding-top:28px",
            MessageActionBar { now, feedback_open: v.b("feedback"), reading: v.b("reading"), style: cstyle(v), on_action: on_act(msg) }
            {status(msg)}
        }
    }
}

fn message_actions_code(_: &Vals) -> String {
    Code::new("MessageActionBar")
        .raw("prompt", "prompt.clone()")
        .raw("model", "\"Sonnet 5.5\".into()")
        .raw("versions", "versions()")
        .raw("now", "clock.now()")
        .raw("on_action", "move |(what, detail)| handle(what, detail)")
        .done()
}

// ---------- Thinking Loader ----------

fn thinking_loader(v: &Vals, now: ClockMs) -> Element {
    if v.s("kind") == "oxyn" {
        return rsx! { div { "data-k": "oxyn", Legacy { slug: "thinking-loader-oxyn", now } } };
    }
    let phases: Vec<String> = [
        "Thinking",
        "Searching the web",
        "Reading 9 sources",
        "Comparing plans",
        "Writing response",
    ]
    .iter()
    .map(|s| s.to_string())
    .collect();
    let key = format!(
        "{}{}{}{}",
        v.s("style"),
        v.s("size"),
        v.s("variant"),
        v.s("state")
    );
    let done = v.s("state") == "done";
    let looped = v.s("state") == "loop";
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:420px;max-width:100%;display:flex;flex-direction:column;align-items:flex-start;gap:14px",
            div { style: "align-self:flex-end;max-width:82%;padding:10px 14px;border-radius:10px;background:var(--oxa-you,#F4F4F5);font-size:14.5px;line-height:21px;color:var(--oxa-fg)",
                "Which note-taking app suits a 6-person design team?" }
            ThinkingPill { phases, done, looped, finish: !looped, size: v.string("size"), variant: v.string("variant"), now, style: cstyle(v) }
        }
    }
}

fn thinking_loader_code(v: &Vals) -> String {
    let mut c = Code::new("ThinkingPill").raw(
        "phases",
        "vec![\"Thinking\".into(), \"Searching the web\".into(), \"Writing response\".into()]",
    );
    if v.s("size") == "sm" {
        c = c.raw("size", "\"sm\".into()");
    }
    if v.s("variant") == "plain" {
        c = c.raw("variant", "\"plain\".into()");
    }
    if v.s("state") == "done" {
        c = c.b("done", true);
    }
    c.raw("now", "clock.now()").done()
}

// ---------- Thinking State ----------

fn thinking_state(v: &Vals, now: ClockMs) -> Element {
    if v.s("kind") == "oxyn" {
        return rsx! { div { "data-k": "oxyn", style: "width:520px;max-width:100%", Legacy { slug: "reasoning-block", now } } };
    }
    let st = v.s("state");
    let key = format!("{}{st}{}", v.s("style"), v.b("all"));
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:420px;max-width:100%;padding-top:8px",
            ThinkingPanel { now, looped: st == "running", finished: st == "done" || st == "collapsed", collapsed: st == "collapsed", show_all: v.b("all"), style: cstyle(v) }
        }
    }
}

fn thinking_state_code(v: &Vals) -> String {
    let mut c = Code::new("ThinkingPanel").raw("steps", "steps()");
    if v.s("state") == "running" {
        c = c.b("looped", true);
    }
    c.raw("now", "clock.now()").done()
}

// ---------- Writing Assistant ----------

fn writing_assistant(v: &Vals, now: ClockMs) -> Element {
    let msg = use_signal(String::new);
    let st = v.s("state");
    let key = format!("{}{st}", v.s("style"));
    rsx! {
        div { "data-k": "{key}", key: "{key}", style: "width:380px;max-width:100%;display:flex;flex-direction:column;gap:8px",
            WritingAssistant { now, asked: if st == "chat" { "Tighten my email".to_string() } else { String::new() }, dictating: st == "dictating", style: cstyle(v), on_action: on_act(msg) }
            {status(msg)}
        }
    }
}

fn writing_assistant_code(_: &Vals) -> String {
    Code::new("WritingAssistant")
        .raw("name", "\"Wren\".into()")
        .raw(
            "suggestions",
            "vec![\"Tighten my email\".into(), \"Make it shorter\".into()]",
        )
        .raw("now", "clock.now()")
        .raw("on_action", "move |(what, text)| handle(what, text)")
        .done()
}

/// Family A demos.
pub(crate) fn demos() -> Vec<Demo> {
    let style = || sel("style", "Style", &["clean", "classic"], "clean");
    vec![
        Demo {
            controls: vec![
                sel("kind", "Component", &["standard", "oxyn"], "standard"),
                style(),
                sel("start", "Start", &["stream", "done"], "stream"),
                sel("vote", "Feedback", &["none", "up", "down"], "none"),
            ],
            variants: vec![
                var(
                    "default",
                    "The sources pop in (\"Reading 8 sources\"), the answer streams with a Stop button, then RELATED and the footer arrive.",
                    &[],
                ),
                var(
                    "finished",
                    "Finished: point at a citation chip to preview every source it cites.",
                    &[("start", "done")],
                ),
                var(
                    "feedback",
                    "Thumbs down: \"What went wrong?\" and the reason chips.",
                    &[("start", "done"), ("vote", "down")],
                ),
                var(
                    "classic",
                    "Classic style: the source cards grouped in a grey band, a blue agent badge, arrows and a grey footer.",
                    &[("style", "classic"), ("start", "done")],
                ),
                var(
                    "oxyn-ai-answer",
                    "Extended: the answer with search, provenance and cited cards.",
                    &[("kind", "oxyn")],
                ),
            ],
            render: ai_answer,
            code: ai_answer_code,
            keyboard: vec![
                (
                    "Tab",
                    "Source cards, citation chips, follow-ups, then the footer actions",
                ),
                ("Focus a chip", "Opens its source preview"),
                ("Esc", "Closes the preview"),
            ],
            a11y: vec![
                "The answer is a polite live region, busy while it streams; the status line reads Reading, Writing or Answer.",
                "Citation chips are buttons named \"Source: <titles>\" and describe their tooltip (role=tooltip) while it is open.",
                "Thumbs are toggle buttons (aria-pressed); thumbs down is aria-expanded over the reason chips; Copied and the thanks are announced.",
            ],
            responsive: vec![
                "Fills its container up to 640 px (480 in the demo); three source cards fit the row with a \"+N\" tile, two below 480 px, one below 340 px.",
            ],
            height: 640,
            live: true,
            ..mk(
                "ai-answer",
                "AI Answer",
                "chat",
                "Streamed answer with source cards, citation chips that preview their source, copy, regenerate, feedback and follow-ups.",
                "AnswerCard",
                "AnswerCardEl",
                "ai-answer",
                "parity/answers.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel("lit", "Lit source", &["2", "1", "3", "none"], "2"),
                tog("open", "Quote open", false),
            ],
            variants: vec![
                var(
                    "default",
                    "The second source is lit – its sentence tinted, the rest faded. Point at a marker or a chip to light another.",
                    &[],
                ),
                var(
                    "quote",
                    "Quote unfolded: the passage with the matching words marked, Open source and Copy quote.",
                    &[("open", "true")],
                ),
                var(
                    "none-lit",
                    "Nothing lit: every sentence at full strength and \"5 of 6 sentences backed by a source\".",
                    &[("lit", "none")],
                ),
                var(
                    "classic",
                    "Classic style: the text on a white card over a grey tray, the quote above the chips.",
                    &[("style", "classic"), ("open", "true")],
                ),
            ],
            render: cited_answer,
            code: cited_answer_code,
            keyboard: vec![
                (
                    "Tab",
                    "Markers in reading order, then the source chips and Follow up",
                ),
                ("Enter / Space", "Unfold or fold that source's quote"),
            ],
            a11y: vec![
                "Markers and chips are buttons named \"Source 2: Cache Notes – Keeping build caches warm\", aria-expanded and aria-controls on the quote.",
                "The quote is a region named \"Quote from <publication>\"; the last row is a polite live region (\"Quote copied\").",
                "The lit sentence changes weight as well as colour, so it reads without colour.",
            ],
            responsive: vec![
                "Fills its container up to 460 px and works from 300 px; below 400 px it tightens, below 360 px the number tags drop, below 280 px Copy quote keeps only its icon.",
            ],
            height: 560,
            ..mk(
                "cited-answer",
                "Cited Answer",
                "chat",
                "An AI answer wired to its sources – point at a citation to light up its sentence and source, click to unfold the quoted passage with the matching words marked.",
                "CitedAnswerCard",
                "CitedAnswerEl",
                "cited-answer",
                "parity/answers.rs",
            )
        },
        Demo {
            controls: vec![
                sel("kind", "Component", &["standard", "oxyn"], "standard"),
                style(),
                tog("feedback", "Reasons open", false),
                tog("reading", "Reading aloud", false),
            ],
            variants: vec![
                var(
                    "default",
                    "Copy turns into a check, regenerate spins and grows the 2 / 2 switcher, thumbs pop.",
                    &[],
                ),
                var(
                    "feedback",
                    "Thumbs down: \"What went wrong?\", reason chips and Send.",
                    &[("feedback", "true")],
                ),
                var(
                    "read-aloud",
                    "Read aloud: a blue four-bar equalizer, the tooltip says Stop.",
                    &[("reading", "true")],
                ),
                var(
                    "classic",
                    "Classic style: the bar and the reasons in their own card, the chosen reason blue.",
                    &[("style", "classic"), ("feedback", "true")],
                ),
                var(
                    "oxyn-response-feedback",
                    "Extended: the response feedback with a note field and a retry.",
                    &[("kind", "oxyn")],
                ),
            ],
            render: message_actions,
            code: message_actions_code,
            keyboard: vec![
                (
                    "Tab",
                    "Copy, Regenerate, the version arrows, thumbs, Read aloud",
                ),
                ("Enter / Space", "Press the focused action"),
            ],
            a11y: vec![
                "The bar is a toolbar \"Message actions\"; thumbs and Read aloud are aria-pressed toggles, thumbs down is aria-expanded.",
                "Versions read \"Version 2 of 3\"; a status region announces Copied, Regenerating, the vote and the thanks.",
                "Tooltips are decoration (the buttons carry their names); the bar is never faded on touch screens.",
            ],
            responsive: vec![
                "Fills its container up to 544 px; touch screens get 36 px buttons, below 320 px Send wraps onto its own line.",
            ],
            height: 470,
            live: true,
            ..mk(
                "message-actions",
                "Message Actions",
                "chat",
                "The quiet bar under an AI message – copy turns into a check, regenerate spins and unfolds a version switcher, thumbs pop and ask why, read aloud becomes a tiny equalizer.",
                "MessageActionBar",
                "MessageActionsEl",
                "message-actions",
                "parity/answers.rs",
            )
        },
        Demo {
            controls: vec![
                sel("kind", "Component", &["standard", "oxyn"], "standard"),
                style(),
                sel("state", "Run", &["loop", "settle", "done"], "loop"),
                sel("size", "Size", &["md", "sm"], "md"),
                sel("variant", "Variant", &["pill", "plain"], "pill"),
            ],
            variants: vec![
                var(
                    "default",
                    "The rays turn without rotating, the label rolls through the phases.",
                    &[],
                ),
                var(
                    "settle",
                    "Settles: after the last phase the check draws itself and the label reads \"Thought for 11s\".",
                    &[("state", "settle")],
                ),
                var("done", "Done.", &[("state", "done")]),
                var(
                    "small",
                    "Small: 26 px tall, 12 px label.",
                    &[("size", "sm")],
                ),
                var(
                    "plain",
                    "Plain: just the spinner and the label.",
                    &[("variant", "plain")],
                ),
                var(
                    "classic",
                    "Classic style: a capsule with grey rays.",
                    &[("style", "classic")],
                ),
                var(
                    "oxyn-thinking-loader",
                    "Extended: the loader with its own phases and timer.",
                    &[("kind", "oxyn")],
                ),
            ],
            render: thinking_loader,
            code: thinking_loader_code,
            keyboard: vec![("—", "Not interactive")],
            a11y: vec![
                "role=status with aria-live=polite: each phase and the done label are read once.",
                "The spinner, the check and the label rolling out are aria-hidden.",
                "Reduced motion switches labels instantly and holds the spinner and shimmer still.",
            ],
            responsive: vec!["Sizes to its label on one line; keep phases to two to four words."],
            height: 220,
            live: true,
            ..mk(
                "thinking-loader",
                "Thinking Loader",
                "chat",
                "A small working pill for AI replies – an eight-ray spinner and a shimmering label that rolls through phases and settles on a check.",
                "ThinkingPill",
                "ThinkingPillEl",
                "thinking-loader",
                "parity/answers.rs",
            )
        },
        Demo {
            controls: vec![
                sel("kind", "Component", &["standard", "oxyn"], "standard"),
                style(),
                sel(
                    "state",
                    "State",
                    &["running", "done", "collapsed"],
                    "running",
                ),
                tog("all", "All sources shown", false),
            ],
            variants: vec![
                var(
                    "default",
                    "The timer ticks, steps fade up along the dashed line, then \"Worked for 7s\" and a blue check.",
                    &[],
                ),
                var("finished", "Finished.", &[("state", "done")]),
                var(
                    "more-sources",
                    "\"+3 more\" opened: every source and Show less.",
                    &[("state", "done"), ("all", "true")],
                ),
                var(
                    "collapsed",
                    "Folded: just \"Worked for 7s\" and a chevron.",
                    &[("state", "collapsed")],
                ),
                var(
                    "classic",
                    "Classic style: a chevron on the left and flat grey chips.",
                    &[("style", "classic"), ("state", "done")],
                ),
                var(
                    "oxyn-reasoning-block",
                    "Extended: the reasoning block with summaries and levels.",
                    &[("kind", "oxyn")],
                ),
            ],
            render: thinking_state,
            code: thinking_state_code,
            keyboard: vec![
                (
                    "Enter / Space",
                    "On the header: fold or unfold the steps; on \"+N more\": show every source",
                ),
                ("Tab", "Header, source chips, +N more"),
            ],
            a11y: vec![
                "The header is one button with aria-expanded; the steps are an ordered list.",
                "Source chips are links with their address as a tooltip; \"+N more\" is aria-expanded.",
                "Reduced motion shows every step without animation.",
            ],
            responsive: vec!["Fills its container up to 420 px; chips wrap."],
            height: 380,
            live: true,
            ..mk(
                "thinking-state",
                "Thinking State",
                "chat",
                "AI working panel with a live timer, searched sources and thinking steps that appear one by one.",
                "ThinkingPanel",
                "ThinkingPanelEl",
                "thinking-state",
                "parity/answers.rs",
            )
        },
        Demo {
            controls: vec![
                style(),
                sel(
                    "state",
                    "Start",
                    &["welcome", "chat", "dictating"],
                    "welcome",
                ),
            ],
            variants: vec![
                var(
                    "default",
                    "The glow drifts behind the welcome line; a chip sends itself and the reply streams.",
                    &[],
                ),
                var(
                    "chat",
                    "After a message: the bubble, the reply, the glow quieter and New chat in the header.",
                    &[("state", "chat")],
                ),
                var(
                    "dictating",
                    "Dictating: the mic is a pale-blue tile with level bars and the words land in the field, then send.",
                    &[("state", "dictating")],
                ),
                var(
                    "classic",
                    "Classic style: a stronger glow on an inset panel, capsule chips, a white pill field and a round mic.",
                    &[("style", "classic")],
                ),
            ],
            render: writing_assistant,
            code: writing_assistant_code,
            keyboard: vec![
                ("Tab", "Suggestions, the field, the mic, Send and New chat"),
                ("Enter", "Send the field"),
            ],
            a11y: vec![
                "New messages are announced politely; \"Writing…\" carries a label.",
                "The mic is a toggle named Dictate / Stop dictation (aria-pressed).",
            ],
            responsive: vec!["Fills its container up to 380 px."],
            height: 500,
            live: true,
            ..mk(
                "writing-assistant",
                "Writing Assistant",
                "chat",
                "AI chat card with a soft gradient glow, suggestions, streaming replies and live voice dictation.",
                "WritingAssistant",
                "WritingAssistantEl",
                "writing-assistant",
                "parity/answers.rs",
            )
        },
    ]
}
