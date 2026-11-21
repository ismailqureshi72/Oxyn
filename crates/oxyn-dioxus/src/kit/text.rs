//! Loaders and moving text (plan #28, chunk 5): `ThinkingLoader`, `LoadingState`, `WordReveal`,
//! `ScrollTitle`, `TextMorph`, `StatefulActionButton`, `SweepText`, `SlotReels` and
//! `StreamingText`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::text::{
        LetterMove, Phases, Piece, SlotText, StatefulAction, Stream, elapsed_label, morph_letters,
        pixel_frame,
    },
};

use crate::{Spinner, composer::icons::Icon};

/// Thinking loader: an eight-ray spinner and a shimmering label that rolls through phases (each
/// new label rises in) and settles on a check with "Thought for 4s". Pill or plain, sm or md.
#[component]
pub fn ThinkingLoader(
    /// Phases (list, interval, loop, started, done).
    phases: Phases,
    /// Shared clock.
    now: ClockMs,
    /// `sm` (28 px) · `md` (34 px).
    #[props(default = "md".to_string())]
    size: String,
    /// `pill` · `plain`.
    #[props(default = "pill".to_string())]
    variant: String,
) -> Element {
    let label = phases.label(now);
    let done = phases.done.is_some();
    let i = phases.index(now);
    rsx! {
        span { class: "oxx-think {size} {variant}", "data-done": "{done}", role: "status", "aria-live": "polite",
            if done {
                span { class: "oxx-check", Icon { name: "check", size: 12.0, stroke: 2.2 } }
            } else {
                Spinner { size: 14.0, style: "ray".to_string(), speed_ms: 900 }
            }
            // A keyed one-item list remounts the label on each phase so it rolls in.
            for k in [format!("{done}-{i}")] {
                span { class: "oxx-lab", key: "{k}",
                    span { class: if done { "" } else { "oxr-shim oxx-sh" }, "{label}" }
                }
            }
        }
    }
}

/// Loading state: a 4×4 pixel-grid loader (drive, dots, orbit, surfer), a shimmering verb and
/// the elapsed time with one decimal.
#[component]
pub fn LoadingState(
    /// Verb ("Churning").
    label: String,
    /// `drive` · `dots` · `orbit` · `surfer`.
    #[props(default = "drive".to_string())]
    variant: String,
    /// Started.
    started: ClockMs,
    /// Shared clock.
    now: ClockMs,
    /// Show the elapsed time.
    #[props(default = true)]
    elapsed: bool,
    /// Reduced motion: one still frame, no shimmer.
    #[props(default)]
    reduced: bool,
) -> Element {
    let g = pixel_frame(&variant, if reduced { ClockMs(0) } else { now });
    let t = elapsed_label(started, now);
    rsx! {
        span { class: "oxx-load", role: "status", "aria-label": "{label}",
            span { class: "oxx-grid", "aria-hidden": "true",
                for (k, c) in g.iter().enumerate() { i { key: "{k}", "data-l": "{c}" } }
            }
            span { class: if reduced { "oxx-verb" } else { "oxx-verb oxr-shim" }, "aria-hidden": "true", "{label}" }
            if elapsed { span { class: "oxx-el", "aria-hidden": "true", "{t}" } }
        }
    }
}

/// Text that arrives word by word or letter by letter, each piece rising out of a clip and a
/// soft blur on a stagger (capped so long text never drags). Plays when `shown` turns true;
/// with `once` it stays revealed.
#[component]
pub fn WordReveal(
    /// Text.
    text: String,
    /// `word` · `char`.
    #[props(default = "word".to_string())]
    by: String,
    /// `h1` · `h2` · `h3` · `p` · `span`.
    #[props(default = "p".to_string())]
    tag: String,
    /// Between pieces, ms (total capped at 900 ms).
    #[props(default = 40)]
    stagger_ms: u32,
    /// Before the first piece, ms.
    #[props(default)]
    delay_ms: u32,
    /// Each piece, ms.
    #[props(default = 700)]
    duration_ms: u32,
    /// In view (host-observed).
    #[props(default = true)]
    shown: bool,
    /// Reveal only the first time.
    #[props(default = true)]
    once: bool,
) -> Element {
    let mut seen = use_signal(|| shown);
    if shown && !seen() {
        seen.set(true);
    }
    let on = if once { seen() || shown } else { shown };
    let parts: Vec<String> = if by == "char" {
        // A space rides on the letter before it, so a wrapped line never starts indented.
        let mut v: Vec<String> = Vec::new();
        for c in text.chars() {
            match v.last_mut() {
                Some(last) if c == ' ' => last.push(c),
                _ => v.push(c.to_string()),
            }
        }
        v
    } else {
        text.split_inclusive(' ').map(|w| w.to_string()).collect()
    };
    let n = parts.len().max(1) as u32;
    let step = stagger_ms.min(900 / n);
    let soft = if tag == "p" || tag == "span" {
        "soft"
    } else {
        ""
    };
    let body = rsx! {
        for (i, p) in parts.iter().enumerate() {
            span { key: "{i}", class: "oxx-w", style: "--i:{i};--st:{step}ms;--dl:{delay_ms}ms;--du:{duration_ms}ms", span { "{p}" } }
        }
    };
    let cls = format!("oxx-reveal {soft}");
    rsx! {
        match tag.as_str() {
            "h1" => rsx! { h1 { class: "{cls}", "data-on": "{on}", "aria-label": "{text}", {body} } },
            "h2" => rsx! { h2 { class: "{cls}", "data-on": "{on}", "aria-label": "{text}", {body} } },
            "h3" => rsx! { h3 { class: "{cls}", "data-on": "{on}", "aria-label": "{text}", {body} } },
            "span" => rsx! { span { class: "{cls}", "data-on": "{on}", "aria-label": "{text}", {body} } },
            _ => rsx! { p { class: "{cls}", "data-on": "{on}", "aria-label": "{text}", {body} } },
        }
    }
}

/// Scroll title: a section heading that comes in as it scrolls into view: `blur` (default),
/// `word` and `line` rise from a clip, `tracking` opens the letter spacing, `wipe` uncovers left
/// to right. Opacity leads, blur clears next, the transform settles last; hiding (with
/// `once = false`) is a quick fade.
#[component]
pub fn ScrollTitle(
    /// Title.
    text: String,
    /// `blur` · `word` · `line` · `tracking` · `wipe`.
    #[props(default = "blur".to_string())]
    variant: String,
    /// `h1` · `h2` · `h3`.
    #[props(default = "h2".to_string())]
    tag: String,
    /// Explicit lines for `line`.
    #[props(default)]
    lines: Vec<String>,
    /// In view (host-observed, e.g. 45 % visible).
    shown: bool,
    /// Reveal only the first time.
    #[props(default = true)]
    once: bool,
) -> Element {
    let mut seen = use_signal(|| shown);
    if shown && !seen() {
        seen.set(true);
    }
    let on = if once { seen() || shown } else { shown };
    let parts: Vec<String> = match variant.as_str() {
        "word" => text.split_inclusive(' ').map(String::from).collect(),
        "line" if !lines.is_empty() => lines.clone(),
        _ => vec![text.clone()],
    };
    let body = rsx! {
        for (i, p) in parts.iter().enumerate() {
            span { key: "{i}", class: "oxx-tp", style: "--i:{i}", span { "{p}" } }
        }
    };
    let cls = format!("oxx-ivt {variant}");
    rsx! {
        match tag.as_str() {
            "h1" => rsx! { h1 { class: "{cls}", "data-on": "{on}", "aria-label": "{text}", {body} } },
            "h3" => rsx! { h3 { class: "{cls}", "data-on": "{on}", "aria-label": "{text}", {body} } },
            _ => rsx! { h2 { class: "{cls}", "data-on": "{on}", "aria-label": "{text}", {body} } },
        }
    }
}

/// Text morph: a label morphs into its next state letter by letter: letters matched by character
/// and occurrence stay, new letters stagger in from a blur, removed letters fade out.
#[component]
pub fn TextMorph(
    /// The label.
    text: String,
) -> Element {
    let mut prev = use_signal(|| text.clone());
    let mut cur = use_signal(|| text.clone());
    if cur() != text {
        prev.set(cur());
        cur.set(text.clone());
    }
    let letters = morph_letters(&prev(), &text);
    rsx! {
        span { class: "oxx-morph", "aria-label": "{text}", role: "text",
            span { class: "oxx-mnew", "aria-hidden": "true",
                {
                    let mut e = 0usize;
                    rsx! {
                        for l in letters.iter().filter(|l| l.mv != LetterMove::Leave) {
                            {
                                let d = if l.mv == LetterMove::Enter { e += 1; e } else { 0 };
                                rsx! { span { key: "{text}-{l.key}", class: if l.mv == LetterMove::Enter { "in" } else { "keep" }, style: "--e:{d}", if l.ch == ' ' { "\u{a0}" } else { "{l.ch}" } } }
                            }
                        }
                    }
                }
            }
            // Keyed one-item list: the leaving letters remount (and fade again) on every change.
            for k in [text.clone()] {
                span { class: "oxx-mold", "aria-hidden": "true", key: "{k}",
                    for l in letters.iter().filter(|l| l.mv == LetterMove::Leave) { span { class: "out", "{l.ch}" } }
                }
            }
        }
    }
}

/// Action states: a button whose label swaps through states (Follow → Following) keeping its
/// place; a pending phase shows the next state's pending label or folds into a round spinner;
/// a failure shows the failure label for 1.8 s; a state can hold and advance on its own.
#[component]
pub fn StatefulActionButton(
    /// Model.
    model: StatefulAction,
    /// `sm` · `md` · `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// Pressed.
    on_press: EventHandler<()>,
) -> Element {
    let (label, tone, icon) = model.view();
    let busy = matches!(model.phase, oxyn_core::kit::text::SwapPhase::Working(_));
    let round = label.is_none();
    rsx! {
        button { class: "oxx-swap {size}", "data-tone": "{tone}", "data-round": "{round}", r#type: "button",
            "aria-busy": "{busy}", "aria-live": "polite",
            onclick: move |_| on_press.call(()),
            if round {
                span { class: "oxp-spin", "aria-label": "Working" }
            } else {
                if let Some(i) = icon { span { class: "oxx-si", key: "{i}", Icon { name: i, size: 14.0, stroke: 2.0 } } }
                TextMorph { text: label.unwrap_or_default() }
            }
        }
    }
}

/// Text shimmer (Arc behaviour): a calm band of the current text colour sweeping over a muted
/// base while `active`, one pass every `duration_ms` with a pause between passes; a paused band
/// resumes where it stopped; a changed label rises in.
#[component]
pub fn SweepText(
    /// Label.
    text: String,
    /// Sweep while work goes on.
    #[props(default = true)]
    active: bool,
    /// One pass, ms.
    #[props(default = 1800)]
    duration_ms: u32,
) -> Element {
    rsx! {
        span { class: "oxx-shim", "data-active": "{active}", style: "--du:{duration_ms}ms", key: "{text}", "{text}" }
    }
}

/// Slot text: text and numbers spin into their new value on reels that stop left to right with a
/// fast start and a soft landing; blur follows each reel's speed; new characters grow in and
/// removed ones collapse; separators and leading symbols stay on their own reels.
#[component]
pub fn SlotReels(
    /// Model.
    model: SlotText,
    /// Shared clock.
    now: ClockMs,
    /// Announce new values politely.
    #[props(default)]
    announce: bool,
    /// Reduced motion: values swap instantly.
    #[props(default)]
    reduced: bool,
) -> Element {
    let reels = model.reels();
    let now = if reduced { ClockMs(u64::MAX / 2) } else { now };
    let done = model.done(now);
    rsx! {
        span { class: "oxx-slot", "aria-label": "{model.to}", role: if announce { "status" } else { "text" }, "aria-live": if announce { "polite" } else { "off" },
            for (i, r) in reels.iter().enumerate() {
                {
                    let p = model.pos(r, now);
                    let p2 = model.pos(r, ClockMs(now.0 + 16));
                    let blur = ((p2 - p).abs() * 1.6).min(2.5);
                    let gone = r.grow == Some(false) && done;
                    rsx! {
                        span { key: "{i}", class: "oxx-reel", "data-grow": match r.grow { Some(true) => "in", Some(false) => "out", None => "" }, "data-gone": "{gone}", "aria-hidden": "true",
                            span { class: "oxx-strip", style: "transform:translateY(-{p:.3}em);filter:blur({blur:.2}px)",
                                for (k, c) in r.strip.iter().enumerate() { span { key: "{k}", "{c}" } }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// One source of a [`StreamingText`] answer.
#[derive(Clone, Debug, PartialEq)]
pub struct Source {
    /// Name ("Scoop Data").
    pub name: String,
    /// Host ("scoopdata.io").
    pub host: String,
    /// Link.
    pub url: String,
}

/// Streaming text (Beautiful UI): a streamed answer whose words fade in with inline citation
/// chips, a stacked "10 sources" button that lists them, actions once done (copy, good, bad,
/// retry) and follow-up questions.
#[component]
pub fn StreamingText(
    /// The stream.
    stream: Stream,
    /// Shared clock.
    now: ClockMs,
    /// Sources (1-based in `[n]` markers).
    sources: Vec<Source>,
    /// Total sources to report (defaults to `sources.len()`).
    #[props(default)]
    source_count: Option<usize>,
    /// Follow-up questions.
    #[props(default)]
    followups: Vec<String>,
    /// A follow-up picked.
    #[props(default)]
    on_follow: EventHandler<String>,
    /// Retry pressed.
    #[props(default)]
    on_retry: EventHandler<()>,
) -> Element {
    let pieces = stream.pieces();
    let shown = stream.shown(now);
    let done = stream.done(now);
    let mut list = use_signal(|| false);
    let mut vote = use_signal(|| 0i8);
    let mut copied = use_signal(|| false);
    let count = source_count.unwrap_or(sources.len());
    rsx! {
        div { class: "oxx-stream", "aria-busy": "{!done}",
            p { class: "oxx-ans",
                for (i, p) in pieces.iter().take(shown).enumerate() {
                    match p {
                        Piece::Word(w) => rsx! { span { key: "{i}", class: "oxx-tok", "{w}" } },
                        Piece::Cite(n) => {
                            let s = sources.get(n.saturating_sub(1)).cloned();
                            rsx! { a { key: "{i}", class: "oxx-cite", href: s.as_ref().map(|s| s.url.clone()).unwrap_or_default(), target: "_blank", rel: "noopener", title: s.as_ref().map(|s| s.name.clone()).unwrap_or_default(), "{s.as_ref().map(|s| s.host.clone()).unwrap_or_else(|| n.to_string())}" } " " }
                        }
                    }
                }
                if !done { span { class: "oxyn-caret bar", "aria-hidden": "true" } }
            }
            if done {
                div { class: "oxx-srow",
                    button { class: "oxx-srcs", r#type: "button", "aria-expanded": "{list()}", onclick: move |_| list.toggle(),
                        span { class: "oxx-favs", for s in sources.iter().take(3) { span { class: "oxx-fav", "{s.name.chars().next().unwrap_or('·')}" } } }
                        "{count} sources"
                    }
                    span { class: "oxt-sp" }
                    button { class: "oxx-act", r#type: "button", "aria-label": "Copy answer", "data-copy": "{stream.text}", onclick: move |_| copied.set(true), Icon { name: if copied() { "check" } else { "copy" }, size: 13.0 } }
                    button { class: "oxx-act", r#type: "button", "aria-label": "Good answer", "aria-pressed": "{vote() == 1}", onclick: move |_| vote.set(if vote() == 1 { 0 } else { 1 }), Icon { name: "heart", size: 13.0 } }
                    button { class: "oxx-act", r#type: "button", "aria-label": "Bad answer", "aria-pressed": "{vote() == -1}", onclick: move |_| vote.set(if vote() == -1 { 0 } else { -1 }), Icon { name: "ban", size: 13.0 } }
                    button { class: "oxx-act", r#type: "button", "aria-label": "Retry", onclick: move |_| on_retry.call(()), Icon { name: "retry", size: 13.0 } }
                }
                if list() {
                    ul { class: "oxx-slist",
                        for s in sources.iter() { li { a { href: "{s.url}", target: "_blank", rel: "noopener", span { class: "oxx-fav", "{s.name.chars().next().unwrap_or('·')}" } b { "{s.name}" } span { class: "oxr-dim", "{s.host}" } } } }
                    }
                }
                if !followups.is_empty() {
                    div { class: "oxx-fu",
                        div { class: "oxt-sh", "Follow-ups" }
                        for f in followups.iter().cloned() {
                            button { class: "oxx-fub", r#type: "button", onclick: { let f = f.clone(); move |_| on_follow.call(f.clone()) }, Icon { name: "chevron-right", size: 12.0 } "{f}" }
                        }
                    }
                }
            }
        }
    }
}
