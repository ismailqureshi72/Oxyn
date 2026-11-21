//! The assistant characters (AI mascot, Ghost mascot, Cube mascot, Pixel mascot) and the AI
//! spark (Dioxus). Every frame is drawn from `oxyn_core::aikit::mascot` shapes, so the GPUI view
//! draws the same character from the same numbers.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::mascot::{self, Ink, Kind, MOODS, Mascot, Mood, Shape};
use oxyn_core::aikit::{pixel, spark};

use super::{cls, live};

fn ink(k: Ink) -> &'static str {
    match k {
        Ink::Body => "body",
        Ink::Light => "light",
        Ink::Shade => "shade",
        Ink::Face => "face",
        Ink::Mark => "mark",
        Ink::Ground => "ground",
        Ink::Danger => "danger",
        Ink::Blush => "blush",
    }
}

/// Shapes as one SVG image in the 100 × 100 box.
fn svg(shapes: &[Shape], label: &str, size: f32) -> Element {
    rsx! {
        svg { class: "oxaik-msvg", width: "{size}", height: "{size}", view_box: "0 0 100 100", role: "img", "aria-label": "{label}",
            for (i, s) in shapes.iter().enumerate() {
                path { key: "{i}", d: "{s.d()}", "data-ink": ink(s.ink), "fill-opacity": "{s.alpha:.2}" }
            }
        }
    }
}

/// The status picker under a demo character (Oxyn extra): radio chips plus Loop.
fn picker(cur: Option<Mood>, moods: &[Mood], on: EventHandler<String>) -> Element {
    let moods = moods.to_vec();
    rsx! {
        div { class: "oxaik-row oxaik-msc-pick", role: "radiogroup", "aria-label": "Status",
            button { r#type: "button", role: "radio", class: cls("oxaik-chip", cur.is_none()), "aria-checked": "{cur.is_none()}", onclick: move |_| on.call("ms-auto".into()), "Loop" }
            for m in moods {
                button { r#type: "button", role: "radio", class: cls("oxaik-chip", cur == Some(m)), "aria-checked": "{cur == Some(m)}",
                    onclick: move |_| on.call(format!("ms-mood-{}", m.index())),
                    "{m.name()}"
                }
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Who {
    Char(Kind),
    Pixel,
}

#[allow(clippy::too_many_arguments)]
fn character(
    who: Who,
    now: ClockMs,
    status: String,
    size: f32,
    tone: String,
    ground: bool,
    paused: bool,
    name: String,
    announce: bool,
    pick: bool,
) -> Element {
    let t0 = now.0;
    let cycle: Vec<Mood> = match who {
        Who::Pixel => pixel::CYCLE.to_vec(),
        Who::Char(Kind::Cube) => vec![
            Mood::Idle,
            Mood::Thinking,
            Mood::Writing,
            Mood::Done,
            Mood::Error,
        ],
        Who::Char(_) => MOODS.to_vec(),
    };
    let fixed = (!status.is_empty()).then(|| Mood::parse(&status));
    let cyc = cycle.clone();
    let mut m = use_signal(move || {
        let mut x = Mascot::new(fixed, &cyc, t0);
        if paused {
            x.paused = Some(t0);
        }
        x
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let s = m.read().clone();
    let label = s.label(&name, n);
    let compact = size < 56.0;
    let off = (size as u64) * 37;
    let body = match who {
        Who::Char(k) => {
            let p = s.pose(n, off);
            let el = s.paused.unwrap_or(n);
            svg(&mascot::draw(k, &p, el, compact, ground), &label, size)
        }
        Who::Pixel => {
            let cells = pixel::cells(&s, n);
            let runs = pixel::runs(&cells);
            rsx! {
                svg { class: "oxaik-msvg oxaik-px", width: "{size}", height: "{size}", view_box: "0 0 24 24", role: "img", "aria-label": "{label}", "shape-rendering": "crispEdges",
                    for (i, (x, y, l, k)) in runs.into_iter().enumerate() {
                        rect { key: "{i}", x: "{x}", y: "{y}", width: "{l}", height: "1", "data-ink": ink(k) }
                    }
                }
            }
        }
    };
    let mood = s.mood(n);
    let cur = s.fixed;
    let mut act = move |id: String| {
        let x = m.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let kind = match who {
        Who::Char(Kind::Orb) => "orb",
        Who::Char(Kind::Ghost) => "ghost",
        Who::Char(Kind::Cube) => "cube",
        Who::Pixel => "pixel",
    };
    rsx! {
        div { class: "oxaik-msc", "data-tone": "{tone}", "data-kind": kind, "data-mood": mood.name(),
            span { class: "oxaik-msc-fig", style: "width:{size}px;height:{size}px", {body} }
            if announce { div { class: "oxaik-sr", role: "status", "aria-live": "polite", "{label}" } }
            if pick {
                {picker(cur, &cycle, EventHandler::new(act))}
                div { class: "oxaik-row oxaik-mute oxaik-msc-cap",
                    span { "{label}" }
                    button { class: "oxaik-btn sm", r#type: "button", "aria-pressed": "{s.paused.is_some()}", onclick: move |_| act("ms-pause".into()),
                        if s.paused.is_some() { "Resume" } else { "Pause" }
                    }
                }
                {live(&said())}
            }
        }
    }
}

/// AI mascot: Oxyn's assistant, a rounded head with an antenna orb, showing what the AI is
/// doing. Idle breathes and glances; listening leans in with wide eyes while the orb pulses;
/// thinking looks up as the orb circles the head (passing behind it); writing reads along the
/// line it draws; done squints happily with a sparkle; an error shakes once with worried brows
/// and a red orb. Left without a status it plays every status in turn, blending between poses.
/// The drawing is a role img labelled with the name and status; it takes no focus.
#[component]
pub fn AiMascot(
    /// Host clock.
    now: ClockMs,
    /// `idle`, `listening`, `thinking`, `writing`, `done`, `error`; empty plays the loop.
    #[props(default)]
    status: String,
    /// Width and height, px (24 to 160; under 56 draws the compact framing).
    #[props(default = 96.0)]
    size: f32,
    /// `ink`, `soft` or `violet`.
    #[props(default = "ink".to_string())]
    tone: String,
    /// The contact shadow and writing line (56 px and up).
    #[props(default = true)]
    ground: bool,
    /// Freeze the current frame.
    #[props(default)]
    paused: bool,
    /// Name in the label (`Atlas, thinking`).
    #[props(default = "Assistant".to_string())]
    name: String,
    /// Speak each status change (politely) for screen readers.
    #[props(default)]
    announce: bool,
    /// Status chips and Pause under it (Oxyn extra).
    #[props(default)]
    picker: bool,
) -> Element {
    character(
        Who::Char(Kind::Orb),
        now,
        status,
        size,
        tone,
        ground,
        paused,
        name,
        announce,
        picker,
    )
}

/// Ghost mascot: a hovering ghost whose wavy hem keeps moving. Resting it blinks; to listen it
/// drifts closer with its mouth open; thinking it floats up, goes see-through and a spiral turns
/// above it; writing it draws a line with a little pen; finished it smiles under a sparkle; on
/// failure it rocks once with knitted brows. Status model, label and blending match the AI mascot.
#[component]
pub fn GhostMascot(
    /// Host clock.
    now: ClockMs,
    /// Status; empty plays the loop.
    #[props(default)]
    status: String,
    /// Size, px.
    #[props(default = 96.0)]
    size: f32,
    /// `ink`, `soft` or `violet`.
    #[props(default = "soft".to_string())]
    tone: String,
    /// The floating shadow.
    #[props(default = true)]
    ground: bool,
    /// Freeze.
    #[props(default)]
    paused: bool,
    /// Name in the label.
    #[props(default = "Assistant".to_string())]
    name: String,
    /// Announce status changes.
    #[props(default)]
    announce: bool,
    /// Status chips and Pause (Oxyn extra).
    #[props(default)]
    picker: bool,
) -> Element {
    character(
        Who::Char(Kind::Ghost),
        now,
        status,
        size,
        tone,
        ground,
        paused,
        name,
        announce,
        picker,
    )
}

/// Cube mascot: an isometric block with three shaded sides; the eyes live on whichever side faces
/// you. Resting it tips slightly and blinks; thinking it rotates ninety degrees at a time and the
/// eyes hop to the side now in front; writing it lays lines down on the ground beneath it; finished
/// it calms down under a sparkle; on failure it jolts.
#[component]
pub fn CubeMascot(
    /// Host clock.
    now: ClockMs,
    /// Status; empty plays the loop.
    #[props(default)]
    status: String,
    /// Size, px.
    #[props(default = 96.0)]
    size: f32,
    /// `ink`, `soft` or `violet`.
    #[props(default = "violet".to_string())]
    tone: String,
    /// The contact shadow and printed lines.
    #[props(default = true)]
    ground: bool,
    /// Freeze.
    #[props(default)]
    paused: bool,
    /// Name in the label.
    #[props(default = "Assistant".to_string())]
    name: String,
    /// Announce status changes.
    #[props(default)]
    announce: bool,
    /// Status chips and Pause (Oxyn extra).
    #[props(default)]
    picker: bool,
) -> Element {
    character(
        Who::Char(Kind::Cube),
        now,
        status,
        size,
        tone,
        ground,
        paused,
        name,
        announce,
        picker,
    )
}

/// Pixel mascot: Oxyn's pixel robot on a 24 × 24 grid, stepped on a fixed clock in whole
/// pixels: a bob and blink at rest, a thought bubble while thinking, a keyboard while writing,
/// raised arms and stars when done, crossed eyes and a shake on an error. `scale` is a whole
/// number of CSS pixels per sprite pixel, so edges stay crisp.
#[component]
pub fn PixelMascot(
    /// Host clock.
    now: ClockMs,
    /// Status; empty plays the loop.
    #[props(default)]
    status: String,
    /// Whole pixels per sprite pixel (1 to 8).
    #[props(default = 4)]
    scale: u32,
    /// `ink`, `soft` or `violet`.
    #[props(default = "ink".to_string())]
    tone: String,
    /// Freeze.
    #[props(default)]
    paused: bool,
    /// Name in the label.
    #[props(default = "Assistant".to_string())]
    name: String,
    /// Announce status changes.
    #[props(default)]
    announce: bool,
    /// Status chips and Pause (Oxyn extra).
    #[props(default)]
    picker: bool,
) -> Element {
    let size = (scale.clamp(1, 8) * 24) as f32;
    character(
        Who::Pixel,
        now,
        status,
        size,
        tone,
        false,
        paused,
        name,
        announce,
        picker,
    )
}

/// AI spark: a small star glyph with four long points and four short paler ones. Resting it swells
/// slowly, thinking sends a wave around it, writing pulses, finishing pulls the points in, and
/// failure leaves short faded points in red; each change eases from the pose on screen. `variant`
/// `mark` draws it bare, `pill` adds a word for the state, `avatar` puts it in a round tile. It
/// never changes its footprint between states and is a polite status region (busy while working).
#[component]
pub fn AiSpark(
    /// Host clock.
    now: ClockMs,
    /// `idle`, `thinking`, `writing`, `done`, `error`; empty plays the loop.
    #[props(default)]
    status: String,
    /// `mark`, `pill` or `avatar`.
    #[props(default = "mark".to_string())]
    variant: String,
    /// Size in px; 0 picks a default per variant (mark 24, pill 16, avatar 20).
    #[props(default)]
    size: f32,
    /// `ink`, `muted` or `violet`.
    #[props(default = "violet".to_string())]
    tone: String,
    /// Pill text instead of the status word.
    #[props(default)]
    label: String,
    /// Speed multiplier.
    #[props(default = 1.0)]
    speed: f32,
    /// Freeze.
    #[props(default)]
    paused: bool,
    /// Announce status changes (off for decorative copies).
    #[props(default = true)]
    announce: bool,
    /// Status chips (Oxyn extra).
    #[props(default)]
    picker: bool,
) -> Element {
    let t0 = now.0;
    let fixed = (!status.is_empty()).then(|| Mood::parse(&status));
    let mut m = use_signal(move || {
        let mut x = Mascot::new(fixed, &spark::CYCLE, t0);
        if paused {
            x.paused = Some(t0);
        }
        x
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let s = m.read().clone();
    let mood = s.mood(n);
    let p = spark::pose_at(&s, n, speed);
    let shapes = spark::draw(&p, mood == Mood::Error);
    let px = if size > 0.0 {
        size
    } else {
        match variant.as_str() {
            "pill" => 16.0,
            "avatar" => 20.0,
            _ => 24.0,
        }
    };
    let word = if label.is_empty() {
        spark::label(mood).to_string()
    } else {
        label.clone()
    };
    let busy = matches!(mood, Mood::Thinking | Mood::Writing);
    let spoken = if variant == "pill" {
        word.clone()
    } else {
        spark::spoken(mood).to_string()
    };
    let act = move |id: String| {
        let x = m.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let cur = s.fixed;
    let mark = rsx! {
        svg { class: "oxaik-msvg", width: "{px}", height: "{px}", view_box: "0 0 100 100", "aria-hidden": "true",
            for (i, sh) in shapes.iter().enumerate() {
                path { key: "{i}", d: "{sh.d()}", "data-ink": ink(sh.ink), "fill-opacity": "{sh.alpha:.2}" }
            }
        }
    };
    rsx! {
        div { class: "oxaik-spk-wrap",
            span { class: "oxaik-spk", "data-variant": "{variant}", "data-tone": "{tone}", "data-mood": mood.name(),
                role: if announce { "status" } else { "img" }, "aria-live": if announce { "polite" } else { "off" },
                "aria-busy": "{busy}", "aria-label": "{spoken}",
                {mark}
                if variant == "pill" {
                    span { class: "oxaik-spk-l", key: "{word}", "{word}" }
                }
            }
            if picker {
                {picker_fn(cur, EventHandler::new(act))}
                {live(&said())}
            }
        }
    }
}

fn picker_fn(cur: Option<Mood>, on: EventHandler<String>) -> Element {
    picker(
        cur,
        &spark::CYCLE
            .iter()
            .copied()
            .chain([Mood::Error])
            .collect::<Vec<_>>(),
        on,
    )
}
