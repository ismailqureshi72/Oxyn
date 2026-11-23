//! Data display: badge, chip, value pill, entity chip, avatar, kbd, separator, number ticker,
//! reaction bar, empty state.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports, unused_variables))]

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        display::{
            AgentRing, Avatar as AvatarModel, AvatarShape, Badge as BadgeModel, EmptyLayout,
            EmptyState as EmptyModel, EntityChip as EntityModel, Fill, Monogram as MonogramModel,
            Presence, Tone,
        },
        ticker::{CountRule, NumberFormat, NumberTicker as TickerModel, Reaction, TickerMode},
    },
};
use oxyn_dioxus::{Avatar, AvatarGroup, Badge, Chip};
#[cfg(oxyn_pro)]
use oxyn_dioxus::{EmptyState, EntityChip, Kbd, Monogram, NumberTicker, ReactionBar, Separator, ValuePill};
use oxyn_tokens::ColorRole;

use super::{Code, q, rng, sel, tog, txt, var};
use crate::{Align, Demo, Vals, data};

const TONES: [&str; 10] = [
    "neutral", "info", "success", "warning", "danger", "purple", "working", "waiting", "queued",
    "stalled",
];

fn fill(id: &str) -> Fill {
    match id {
        "outline" => Fill::Outline,
        "solid" => Fill::Solid,
        "mono" => Fill::Mono,
        _ => Fill::Soft,
    }
}

fn fill_code(id: &str) -> &'static str {
    match id {
        "outline" => "Fill::Outline",
        "solid" => "Fill::Solid",
        "mono" => "Fill::Mono",
        _ => "Fill::Soft",
    }
}

fn tone_code(id: &str) -> String {
    let mut c = id.chars();
    match c.next() {
        Some(f) => format!("Tone::{}{}", f.to_ascii_uppercase(), c.as_str()),
        None => "Tone::Neutral".into(),
    }
}

fn badge(v: &Vals, now: ClockMs) -> Element {
    let roll = v.b("roll");
    let (label, tone) = if roll {
        // Cycle the label so the roll is visible: queued, running, passed.
        [
            ("Queued", Tone::Queued),
            ("Running", Tone::Working),
            ("Passed", Tone::Success),
        ][(now.0 / 1600 % 3) as usize]
    } else {
        (
            v.s("label"),
            Tone::from_id(v.s("tone")).unwrap_or(Tone::Neutral),
        )
    };
    let mut b = BadgeModel::new(label, tone);
    b.fill = fill(v.s("fill"));
    b.live = v.b("live");
    b.dot = v.b("dot");
    if v.b("count") {
        b.count = Some(128);
    }
    rsx! { Badge { badge: b, size: v.string("size"), icon: v.opt("icon"), roll } }
}

fn badge_code(v: &Vals) -> String {
    let mut c = Code::new("Badge").pre(format!(
        "let mut badge = BadgeModel::new({}, {});",
        q(v.s("label")),
        tone_code(v.s("tone"))
    ));
    if v.s("fill") != "soft" {
        c = c.pre(format!("badge.fill = {};", fill_code(v.s("fill"))));
    }
    if v.b("live") {
        c = c.pre("badge.live = true; // pulses every 1.8 s");
    }
    if v.b("dot") {
        c = c.pre("badge.dot = true;");
    }
    if v.b("count") {
        c = c.pre("badge.count = Some(128); // shows 99+");
    }
    let c = c
        .raw("badge", "badge")
        .s("size", v.s("size"), "sm")
        .opt("icon", v.s("icon"));
    if v.b("roll") {
        c.raw("roll", "true // a new label rises in").done()
    } else {
        c.done()
    }
}

fn chip(v: &Vals, _: ClockMs) -> Element {
    let count = v.b("count").then_some(12);
    rsx! {
        Chip {
            label: v.string("label"),
            kind: v.string("kind"),
            selected: v.b("selected"),
            count,
            dot: v.opt("dot"),
            disabled: v.b("disabled"),
        }
    }
}

fn chip_code(v: &Vals) -> String {
    let mut c = Code::new("Chip")
        .raw("label", q(v.s("label")))
        .s("kind", v.s("kind"), "static")
        .b("selected", v.b("selected"));
    if v.b("count") {
        c = c.raw("count", "Some(12)");
    }
    c = c.opt("dot", v.s("dot")).b("disabled", v.b("disabled"));
    match v.s("kind") {
        "toggle" => c.raw("on_toggle", "move |on: bool| filter.set(on)").done(),
        "removable" => c.raw("on_remove", "move |_| remove()").done(),
        _ => c.done(),
    }
}

fn agent_av(name: &str, size: f32, state: &str) -> AvatarModel {
    let mut a = AvatarModel::agent(name, &data::state(state));
    a.size = size;
    a
}

fn person(name: &str, size: f32) -> AvatarModel {
    let mut a = AvatarModel::person(name);
    a.size = size;
    a
}

fn avatar(v: &Vals, _: ClockMs) -> Element {
    let size = AvatarModel::preset_size(v.s("preset")).unwrap_or(v.f("size") as f32);
    let mut a = if v.s("kind") == "agent" {
        agent_av(v.s("name"), size, v.s("state"))
    } else {
        person(v.s("name"), size)
    };
    if v.s("kind") == "person" {
        a.status = match v.s("presence") {
            "online" => Some(Presence::Online),
            "away" => Some(Presence::Away),
            "busy" => Some(Presence::Busy),
            "offline" => Some(Presence::Offline),
            _ => None,
        };
    }
    if v.b("square") {
        a.shape = AvatarShape::Square;
    } else if v.s("kind") == "person" {
        a.shape = AvatarShape::Circle;
    }
    if v.b("progress") {
        a.ring = AgentRing::Progress(0.64);
    }
    if v.b("model") {
        a.model = Some("S".into());
    }
    if v.b("broken_src") && v.s("kind") == "person" {
        // A photo that fails to load falls back to initials (Arc UI behaviour).
        a.src = Some("/missing-avatar.png".into());
    }
    let gs = (size * 0.7).clamp(24.0, 40.0);
    let group = vec![
        person("Ada Lovelace", gs),
        agent_av("Coder", gs, "streaming"),
        person("Grace Hopper", gs),
        person("Alan Turing", gs),
        person("Katherine Johnson", gs),
        person("Linus T", gs),
    ];
    rsx! {
        div { class: "oxd-col",
            Avatar { avatar: a }
            if v.b("group") {
                AvatarGroup { avatars: group, max: v.u("max") }
            }
        }
    }
}

fn avatar_code(v: &Vals) -> String {
    let size = v.s("size");
    let mut c = if v.s("kind") == "agent" {
        Code::new("Avatar").pre(format!(
            "let mut av = AvatarModel::agent({}, &agent_state); // ring follows the state",
            q(v.s("name"))
        ))
    } else {
        Code::new("Avatar").pre(format!(
            "let mut av = AvatarModel::person({});",
            q(v.s("name"))
        ))
    };
    c = c.pre(format!("av.size = {size}.0;"));
    if v.s("kind") == "person" && v.s("presence") != "none" {
        let p = v.s("presence");
        c = c.pre(format!(
            "av.status = Some(Presence::{}{});",
            p[..1].to_uppercase(),
            &p[1..]
        ));
    }
    if v.b("square") {
        c = c.pre("av.shape = AvatarShape::Square;");
    }
    if v.b("progress") {
        c = c.pre("av.ring = AgentRing::Progress(0.64);");
    }
    if v.b("model") {
        c = c.pre("av.model = Some(\"S\".into());");
    }
    let mut out = c.raw("avatar", "av").done();
    if v.b("group") {
        out.push_str(&format!(
            "\n\n// Overlapping group with a +N tile\nrsx! {{ AvatarGroup {{ avatars: team, max: {} }} }}",
            v.s("max")
        ));
    }
    out
}

pub(crate) fn demos() -> Vec<Demo> {
    #[allow(unused_mut)]
    let mut v = vec![        Demo {
            slug: "badge",
            name: "Badge",
            category: "data-display",
            summary: "A small status label that carries the colour: six tones plus four agent-state tones, soft, outline, solid or mono, a dot, a live dot that pulses and a count up to 99+.",
            dioxus: "Badge",
            gpui: "BadgeEl",
            spec: "badges-chips",
            source: "primitives/display.rs",
            controls: vec![
                txt("label", "Label", "Running"),
                sel("tone", "Tone", &TONES, "working"),
                sel(
                    "fill",
                    "Fill",
                    &["soft", "outline", "solid", "mono"],
                    "soft",
                ),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                sel(
                    "icon",
                    "Icon",
                    &["none", "pencil", "check", "clock"],
                    "none",
                ),
                tog("live", "Live dot", true),
                tog("dot", "Dot", false),
                tog("count", "Count", false),
                tog("roll", "Roll label changes", false),
            ],
            variants: vec![
                var("live", "Pulses while something runs.", &[]),
                var(
                    "roll-lg",
                    "Large, the label rolls as the status changes.",
                    &[
                        ("size", "lg"),
                        ("roll", "true"),
                        ("live", "false"),
                        ("dot", "true"),
                    ],
                ),
                var(
                    "dot-success",
                    "Leading dot.",
                    &[
                        ("label", "Passed"),
                        ("tone", "success"),
                        ("live", "false"),
                        ("dot", "true"),
                    ],
                ),
                var(
                    "outline-warning",
                    "Border only.",
                    &[
                        ("label", "Needs review"),
                        ("tone", "warning"),
                        ("live", "false"),
                        ("fill", "outline"),
                    ],
                ),
                var(
                    "count",
                    "Round count, 99+.",
                    &[
                        ("label", "unread"),
                        ("tone", "neutral"),
                        ("live", "false"),
                        ("fill", "solid"),
                        ("count", "true"),
                    ],
                ),
                var(
                    "mono",
                    "Monospace tag, no fill.",
                    &[
                        ("label", "v0.4.2"),
                        ("tone", "neutral"),
                        ("live", "false"),
                        ("fill", "mono"),
                    ],
                ),
            ],
            render: badge,
            code: badge_code,
            keyboard: vec![("—", "Not focusable; it labels the thing next to it")],
            a11y: vec![
                "Status is never colour alone: every tone carries its text label.",
                "The live dot is decorative and stops under reduced motion.",
                "Counts over 99 are read as \"More than 99 unread\"; live badges add \"(in progress)\".",
            ],
            responsive: vec![
                "Inline-sized: sits in table cells, list rows and headers; three sizes.",
            ],
            height: 160,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "chip",
            name: "Chip",
            category: "data-display",
            summary: "A static, toggle or removable chip with a count and a status dot.",
            dioxus: "Chip",
            gpui: "ChipEl",
            spec: "badges-chips",
            source: "primitives/display.rs",
            controls: vec![
                txt("label", "Label", "Running"),
                sel("kind", "Kind", &["static", "toggle", "removable"], "toggle"),
                tog("selected", "Selected", true),
                tog("count", "Count", true),
                sel(
                    "dot",
                    "Dot",
                    &["none", "working", "success", "danger"],
                    "working",
                ),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var("toggle", "A filter chip.", &[]),
                var(
                    "removable",
                    "A context chip you can remove.",
                    &[
                        ("label", "src/lib.rs"),
                        ("kind", "removable"),
                        ("count", "false"),
                        ("dot", "none"),
                    ],
                ),
                var(
                    "static",
                    "A tag.",
                    &[
                        ("label", "Rust"),
                        ("kind", "static"),
                        ("count", "false"),
                        ("dot", "none"),
                    ],
                ),
            ],
            render: chip,
            code: chip_code,
            keyboard: vec![
                ("Enter / Space", "Toggle"),
                ("Backspace / Delete", "Remove a focused removable chip"),
            ],
            a11y: vec![
                "Toggle chips are buttons with aria-pressed; the check and dot are aria-hidden.",
            ],
            responsive: vec!["Chips wrap with their row."],
            height: 160,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "avatar",
            name: "Avatar",
            category: "data-display",
            summary: "Initials on a colour of their own, a pixel mark for agents with a ring that follows the agent state, presence, a model badge, a progress ring and an overlapping group with +N.",
            dioxus: "Avatar, AvatarGroup",
            gpui: "AvatarEl",
            spec: "avatar",
            source: "primitives/display.rs",
            controls: vec![
                sel("kind", "Kind", &["person", "agent"], "agent"),
                txt("name", "Name", "Coder"),
                sel("state", "Agent state", &data::STATES, "thinking"),
                sel(
                    "presence",
                    "Presence",
                    &["none", "online", "away", "busy", "offline"],
                    "online",
                ),
                rng("size", "Size", 12.0, 96.0, 4.0, "48"),
                sel(
                    "preset",
                    "Size preset",
                    &["custom", "sm", "md", "lg", "xl"],
                    "custom",
                ),
                tog("broken_src", "Photo fails to load", false),
                tog("square", "Square", false),
                tog("progress", "Progress ring", false),
                tog("model", "Model badge", false),
                tog("group", "Show group", true),
                rng("max", "Group max", 2.0, 6.0, 1.0, "4"),
            ],
            variants: vec![
                var("agent-ring", "Agent ring follows the state.", &[]),
                var(
                    "person",
                    "Initials and presence.",
                    &[
                        ("kind", "person"),
                        ("name", "Ada Lovelace"),
                        ("group", "false"),
                    ],
                ),
                var(
                    "progress",
                    "Progress ring and model badge.",
                    &[("progress", "true"), ("model", "true"), ("group", "false")],
                ),
                var(
                    "workspace",
                    "Rounded square for a workspace.",
                    &[
                        ("kind", "person"),
                        ("name", "Northwind Labs"),
                        ("square", "true"),
                        ("presence", "none"),
                        ("group", "false"),
                    ],
                ),
            ],
            render: avatar,
            code: avatar_code,
            keyboard: vec![("—", "Not focusable on its own")],
            a11y: vec![
                "role=img with a full name (\"Coder, thinking\", \"Ada Lovelace, online\").",
                "The group is a role=group named by its members (\"… and 2 more\").",
                "The ring and the twinkling mark stop under prefers-reduced-motion.",
            ],
            responsive: vec![
                "Seven sizes from 12 to 96 px; presence and the model badge scale with it.",
                "In a group, overlapped initials stay centred in the part that is visible.",
            ],
            height: 220,
            align: Align::Center,
            live: false,
        },
    ];
    #[cfg(oxyn_pro)]
    v.extend(pro::demos());
    super::in_order(&mut v, &ORDER);
    v
}

/// Page order of this file's demos (free and Pro).
const ORDER: [&str; 10] = ["badge", "chip", "value-pill", "entity-chip", "avatar", "kbd", "separator", "number-ticker", "reaction-bar", "empty-state"];

// Oxyn Pro demos of this file (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
