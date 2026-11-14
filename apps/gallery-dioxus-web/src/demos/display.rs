//! Data display: badge, chip, value pill, entity chip, avatar, kbd, separator, number ticker,
//! reaction bar, empty state.

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
use oxyn_dioxus::{
    Avatar, AvatarGroup, Badge, Chip, EmptyState, EntityChip, Kbd, Monogram, NumberTicker,
    ReactionBar, Separator, ValuePill,
};
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

fn badge(v: &Vals, _: ClockMs) -> Element {
    let tone = Tone::from_id(v.s("tone")).unwrap_or(Tone::Neutral);
    let mut b = BadgeModel::new(v.s("label"), tone);
    b.fill = fill(v.s("fill"));
    b.live = v.b("live");
    b.dot = v.b("dot");
    if v.b("count") {
        b.count = Some(128);
    }
    rsx! { Badge { badge: b, size: v.string("size"), icon: v.opt("icon") } }
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
    c.raw("badge", "badge")
        .s("size", v.s("size"), "sm")
        .opt("icon", v.s("icon"))
        .done()
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

fn pill(v: &Vals, _: ClockMs) -> Element {
    let delta = v.b("delta").then_some((410.0, 320.0));
    rsx! {
        p { class: "oxd-prose",
            "p95 latency is " ValuePill { value: v.string("value"), tone: v.string("tone"), delta, good_up: v.b("good_up") } " after the cache change."
        }
    }
}

fn pill_code(v: &Vals) -> String {
    let mut c =
        Code::new("ValuePill")
            .raw("value", q(v.s("value")))
            .s("tone", v.s("tone"), "neutral");
    if v.b("delta") {
        c = c.raw("delta", "Some((410.0, 320.0)) // previous, current");
    }
    if !v.b("good_up") {
        c = c.raw("good_up", "false // lower is better (latency, cost)");
    }
    c.done()
}

fn entity(v: &Vals, _: ClockMs) -> Element {
    let mut e = EntityModel::new(v.s("name"));
    if v.b("status") {
        e.status = Some(Presence::Online);
    }
    e.card = vec![
        ("Type".into(), "Supplier".into()),
        ("Since".into(), "March 2024".into()),
        ("Orders".into(), "1,204".into()),
    ];
    let open = v.b("card");
    rsx! {
        div { class: "oxd-w480 oxd-room-card",
            p { class: "oxd-prose",
                "Restock " EntityChip { entity: e, card_open: open } " in " ValuePill { value: "7 days" } " — it is cheaper than "
                EntityChip { entity: EntityModel::new("Northwind Labs") } "."
            }
            div { class: "oxd-line",
                for i in 0..8usize {
                    Monogram { monogram: MonogramModel::of("x", Some(i), Some(["AK","BR","CK","DL","EM","FN","GO","HP"][i])), size: v.f("size") as f32, square: v.b("square") }
                }
            }
        }
    }
}

fn entity_code(v: &Vals) -> String {
    let mut c = Code::new("EntityChip").pre(format!(
        "let mut supplier = EntityModel::new({});",
        q(v.s("name"))
    ));
    if v.b("status") {
        c = c.pre("supplier.status = Some(Presence::Online);");
    }
    c.pre("supplier.card = vec![(\"Type\".into(), \"Supplier\".into()), (\"Orders\".into(), \"1,204\".into())];")
        .raw("entity", "supplier")
        .done()
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

fn kbd(v: &Vals, _: ClockMs) -> Element {
    rsx! {
        div { class: "oxd-line",
            Kbd { keys: v.string("keys"), other: v.s("platform") == "windows", style: v.string("style"), pressed: v.b("pressed") }
            span { class: "oxd-note", "to open the command palette" }
        }
    }
}

fn kbd_code(v: &Vals) -> String {
    Code::new("Kbd")
        .raw("keys", q(v.s("keys")))
        .b("other", v.s("platform") == "windows")
        .s("style", v.s("style"), "keycap")
        .b("pressed", v.b("pressed"))
        .done()
}

fn separator(v: &Vals, _: ClockMs) -> Element {
    let vertical = v.b("vertical");
    let label = (!vertical).then(|| v.opt("label")).flatten();
    rsx! {
        if vertical {
            div { class: "oxd-line oxd-h24",
                span { "Files" }
                Separator { vertical: true, dashed: v.b("dashed") }
                span { "Edits" }
                Separator { vertical: true, dashed: v.b("dashed") }
                span { "Runs" }
            }
        } else {
            div { class: "oxd-w420 oxd-col oxd-stretch",
                p { class: "oxd-msg", "Fixed the refresh-token race." }
                Separator { label: label.clone(), decorative: label.is_none(), dashed: v.b("dashed"), fade: v.b("fade"), inset: v.f("inset") as f32 }
                p { class: "oxd-msg", "Now add rate limiting to the login route." }
            }
        }
    }
}

fn separator_code(v: &Vals) -> String {
    let mut c = Code::new("Separator").b("vertical", v.b("vertical"));
    if !v.b("vertical") && v.opt("label").is_some() {
        c = c
            .raw("label", q(v.s("label")))
            .raw("decorative", "false // a real separator for screen readers");
    }
    c = c.b("dashed", v.b("dashed")).b("fade", v.b("fade"));
    if v.f("inset") > 0.0 {
        c = c.raw("inset", format!("{}.0", v.s("inset")));
    }
    c.done()
}

fn ticker_values(k: u64, kind: &str) -> f64 {
    let seq: [f64; 6] = match kind {
        "currency" => [0.42, 0.87, 1.31, 1.87, 2.44, 3.12],
        "compact" => [
            980_000.0,
            1_200_000.0,
            1_450_000.0,
            1_900_000.0,
            2_300_000.0,
            3_400_000.0,
        ],
        "percent" => [0.912, 0.934, 0.951, 0.968, 0.979, 0.986],
        _ => [9_998.0, 10_240.0, 10_391.0, 10_877.0, 11_204.0, 12_480.0],
    };
    seq[(k % 6) as usize]
}

fn ticker(v: &Vals, now: ClockMs) -> Element {
    let kind = v.s("format");
    let fmt = match kind {
        "currency" => NumberFormat::Currency("$".into(), 2),
        "compact" => NumberFormat::Compact,
        "percent" => NumberFormat::Percent(1),
        _ => NumberFormat::Integer,
    };
    let k = now.0.saturating_sub(crate::BASE.0) / 3_000;
    let mut t = TickerModel::new(ticker_values(k.max(1) - 1, kind), fmt);
    if v.s("mode") == "count-up" {
        t.mode = TickerMode::CountUp;
    }
    t.set(
        ticker_values(k + 1, kind),
        ClockMs(crate::BASE.0 + k * 3_000),
    );
    rsx! { NumberTicker { ticker: t, now, size: v.f("size") as f32 } }
}

fn ticker_code(v: &Vals) -> String {
    let fmt = match v.s("format") {
        "currency" => "NumberFormat::Currency(\"$\".into(), 2)",
        "compact" => "NumberFormat::Compact",
        "percent" => "NumberFormat::Percent(1)",
        _ => "NumberFormat::Integer",
    };
    let mut c = Code::new("NumberTicker").pre(format!(
        "let mut runs = use_signal(|| NumberTicker::new(9_998.0, {fmt}));"
    ));
    if v.s("mode") == "count-up" {
        c = c.pre("runs.write().mode = TickerMode::CountUp;");
    }
    c.pre("// later: roll to the new value from the current frame")
        .pre("runs.write().set(10_240.0, clock.now());")
        .raw("ticker", "runs()")
        .raw("now", "clock.now()")
        .raw("size", format!("{}.0", v.s("size")))
        .done()
}

fn reactions() -> Vec<Reaction> {
    vec![
        Reaction::new("like", "Like", "heart", 1_203, ColorRole::FgDefault),
        Reaction::action("comment", "Comment", "comment", 48),
        Reaction::new("repost", "Repost", "repost", 12_480, ColorRole::FgDefault),
        Reaction::action("share", "Share", "share", 0),
    ]
}

fn reaction(v: &Vals, now: ClockMs) -> Element {
    let mut r = reactions();
    r[0].active = v.b("liked");
    let rule = CountRule {
        from: v.f("abbreviate") as u64,
        hide_zero: v.b("hide_zero"),
        ..Default::default()
    };
    rsx! { ReactionBar { reactions: r, compact: v.b("compact"), rule, now } }
}

fn reaction_code(v: &Vals) -> String {
    Code::new("ReactionBar")
        .pre("let reactions = vec![")
        .pre("    Reaction::new(\"like\", \"Like\", \"heart\", 1_203, ColorRole::FgDefault),")
        .pre("    Reaction::action(\"comment\", \"Comment\", \"comment\", 48),")
        .pre("    Reaction::new(\"repost\", \"Repost\", \"repost\", 12_480, ColorRole::FgDefault),")
        .pre("];")
        .raw("reactions", "reactions")
        .b("compact", v.b("compact"))
        .raw(
            "rule",
            format!(
                "CountRule {{ from: {}, hide_zero: {}, ..Default::default() }}",
                v.s("abbreviate"),
                v.b("hide_zero")
            ),
        )
        .raw("now", "clock.now()")
        .raw("on_react", "move |(id, on): (String, bool)| react(id, on)")
        .done()
}

fn empty(v: &Vals, _: ClockMs) -> Element {
    let layout = match v.s("layout") {
        "bare" => EmptyLayout::Bare,
        "inline" => EmptyLayout::Inline,
        _ => EmptyLayout::Framed,
    };
    let mut e = EmptyModel {
        title: "No runs yet".into(),
        text: "Start an agent on this repo and its runs show up here.".into(),
        action: v.b("action").then(|| "New run".into()),
        secondary: v.b("secondary").then(|| "Read the guide".into()),
        icon: "agents".into(),
        layout,
        tall: v.b("tall"),
        ..Default::default()
    };
    if v.b("prompts") {
        e.prompts = vec![
            "Summarise this repo".into(),
            "Find flaky tests".into(),
            "Draft release notes".into(),
        ];
    }
    rsx! { div { class: "oxd-w520", EmptyState { state: e } } }
}

fn empty_code(v: &Vals) -> String {
    let layout = match v.s("layout") {
        "bare" => "EmptyLayout::Bare",
        "inline" => "EmptyLayout::Inline",
        _ => "EmptyLayout::Framed",
    };
    let mut lines = vec![
        "let empty = EmptyModel {".to_string(),
        "    title: \"No runs yet\".into(),".into(),
        "    text: \"Start an agent on this repo and its runs show up here.\".into(),".into(),
    ];
    if v.b("action") {
        lines.push("    action: Some(\"New run\".into()),".into());
    }
    if v.b("secondary") {
        lines.push("    secondary: Some(\"Read the guide\".into()),".into());
    }
    lines.push("    icon: \"agents\".into(),".into());
    lines.push(format!("    layout: {layout},"));
    if v.b("tall") {
        lines.push("    tall: true,".into());
    }
    if v.b("prompts") {
        lines.push(
            "    prompts: vec![\"Summarise this repo\".into(), \"Find flaky tests\".into()],"
                .into(),
        );
    }
    lines.push("    ..Default::default()".into());
    lines.push("};".into());
    let mut c = Code::new("EmptyState");
    for l in lines {
        c = c.pre(l);
    }
    c.raw("state", "empty")
        .raw("on_action", "move |_| new_run()")
        .raw("on_prompt", "move |p: String| start(p)")
        .done()
}

pub(crate) fn demos() -> Vec<Demo> {
    vec![
        Demo {
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
            ],
            variants: vec![
                var("live", "Pulses while something runs.", &[]),
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
            live: false,
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
            slug: "value-pill",
            name: "Value pill",
            category: "data-display",
            summary: "An inline value in a sentence (\"7 days\") with a tone and an optional delta arrow that knows whether up is good.",
            dioxus: "ValuePill",
            gpui: "ValuePillEl",
            spec: "badges-chips",
            source: "primitives/display.rs",
            controls: vec![
                txt("value", "Value", "320 ms"),
                sel(
                    "tone",
                    "Tone",
                    &["neutral", "green", "orange", "red", "accent"],
                    "neutral",
                ),
                tog("delta", "Delta arrow", true),
                tog("good_up", "Up is good", false),
            ],
            variants: vec![
                var("delta", "Latency went down: good.", &[]),
                var(
                    "tone",
                    "A plain toned value.",
                    &[("value", "Healthy"), ("tone", "green"), ("delta", "false")],
                ),
            ],
            render: pill,
            code: pill_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "The accessible name carries the value and the direction word, not only the arrow colour.",
            ],
            responsive: vec!["Sits on the text baseline and wraps with the sentence."],
            height: 140,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "entity-chip",
            name: "Entity chip",
            category: "data-display",
            summary: "A named thing inside a sentence, with a monogram on a colour of its own, a status dot and a hover card.",
            dioxus: "EntityChip, Monogram",
            gpui: "EntityChipEl, MonogramEl",
            spec: "badges-chips",
            source: "primitives/display.rs",
            controls: vec![
                txt("name", "Name", "Cone King"),
                tog("status", "Status dot", true),
                tog("card", "Hover card open", false),
                rng("size", "Monogram size", 16.0, 32.0, 2.0, "22"),
                tog("square", "Square monograms", false),
            ],
            variants: vec![
                var("inline", "In a sentence.", &[]),
                var("card", "Hover card open.", &[("card", "true")]),
            ],
            render: entity,
            code: entity_code,
            keyboard: vec![("Esc", "Close the hover card")],
            a11y: vec!["The monogram is decorative; the chip is named by the entity."],
            responsive: vec!["Sits inline and wraps with the sentence."],
            height: 260,
            align: Align::Start,
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
                    "Arc size",
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
        Demo {
            slug: "kbd",
            name: "Kbd",
            category: "data-display",
            summary: "A key chip for shortcuts: ⌘ on a Mac and Ctrl elsewhere from one Mod key, sequences, keycap, flat and inline styles and a pressed state.",
            dioxus: "Kbd",
            gpui: "KbdEl",
            spec: "display-atoms",
            source: "primitives/display.rs",
            controls: vec![
                sel(
                    "keys",
                    "Keys",
                    &[
                        "Mod+K",
                        "Mod+Shift+P",
                        "G I",
                        "Mod+Enter",
                        "Esc",
                        "Alt+Shift+Down",
                    ],
                    "Mod+K",
                ),
                sel("platform", "Platform", &["mac", "windows"], "mac"),
                sel("style", "Style", &["keycap", "flat", "inline"], "keycap"),
                tog("pressed", "Pressed", false),
            ],
            variants: vec![
                var("keycap", "Raised keycaps.", &[]),
                var(
                    "other",
                    "Ctrl on Windows and Linux.",
                    &[("platform", "windows")],
                ),
                var("sequence", "A two-key sequence.", &[("keys", "G I")]),
                var("flat", "Flat style.", &[("style", "flat"), ("keys", "Esc")]),
            ],
            render: kbd,
            code: kbd_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "Spelled out for screen readers (\"Command K\").",
                "Shortcuts are also exposed as aria-keyshortcuts on the control they belong to.",
            ],
            responsive: vec!["inline style sits in running text without changing line height."],
            height: 140,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "separator",
            name: "Separator",
            category: "data-display",
            summary: "A thin line between groups: across or upright, a word in the middle, dashed, faded ends or inset.",
            dioxus: "Separator",
            gpui: "SeparatorEl",
            spec: "display-atoms",
            source: "primitives/display.rs",
            controls: vec![
                tog("vertical", "Vertical", false),
                txt("label", "Label", "Earlier"),
                tog("dashed", "Dashed", false),
                tog("fade", "Faded ends", false),
                rng("inset", "Inset", 0.0, 48.0, 8.0, "0"),
            ],
            variants: vec![
                var("labelled", "A word in the middle.", &[]),
                var("plain", "Decorative line.", &[("label", "")]),
                var("dashed", "Dashed.", &[("label", ""), ("dashed", "true")]),
                var(
                    "vertical",
                    "Upright between tools.",
                    &[("vertical", "true")],
                ),
            ],
            render: separator,
            code: separator_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "Decorative by default.",
                "A labelled separator is a real role=separator for screen readers.",
            ],
            responsive: vec!["Fills its container; inset keeps it off the edges."],
            height: 180,
            align: Align::Center,
            live: false,
        },
        Demo {
            slug: "number-ticker",
            name: "Number ticker",
            category: "data-display",
            summary: "Numbers that roll digit by digit like an odometer, or count up through the values in between; new columns slide in.",
            dioxus: "NumberTicker",
            gpui: "NumberTickerEl",
            spec: "number-ticker",
            source: "primitives/display.rs",
            controls: vec![
                sel(
                    "format",
                    "Format",
                    &["integer", "currency", "compact", "percent"],
                    "integer",
                ),
                sel("mode", "Mode", &["roll", "count-up"], "roll"),
                rng("size", "Size", 14.0, 48.0, 2.0, "36"),
            ],
            variants: vec![
                var("roll", "Odometer columns.", &[]),
                var(
                    "currency",
                    "Cost so far.",
                    &[("format", "currency"), ("size", "28")],
                ),
                var(
                    "count-up",
                    "Counts through the values.",
                    &[("format", "compact"), ("mode", "count-up"), ("size", "28")],
                ),
            ],
            render: ticker,
            code: ticker_code,
            keyboard: vec![("—", "Not focusable")],
            a11y: vec![
                "The final value is the accessible text (role=text); rolling digit columns are aria-hidden.",
            ],
            responsive: vec!["New digit columns slide in; the rest keep their place."],
            height: 180,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "reaction-bar",
            name: "Reaction bar",
            category: "data-display",
            summary: "Like, comment, repost and share in one pill: icons pop and fill, counts roll in digit by digit.",
            dioxus: "ReactionBar",
            gpui: "ReactionBarEl",
            spec: "number-ticker",
            source: "primitives/display.rs",
            controls: vec![
                tog("liked", "Liked", false),
                tog("compact", "Compact", false),
                rng(
                    "abbreviate",
                    "Abbreviate from",
                    0.0,
                    20000.0,
                    1000.0,
                    "10000",
                ),
                tog("hide_zero", "Hide zero", true),
            ],
            variants: vec![
                var("default", "Counts and labels.", &[]),
                var("liked", "Liked.", &[("liked", "true")]),
                var("compact", "Icon only.", &[("compact", "true")]),
            ],
            render: reaction,
            code: reaction_code,
            keyboard: vec![
                ("Tab", "Each reaction is a button"),
                ("Enter / Space", "React"),
            ],
            a11y: vec![
                "A role=group named Reactions; toggles expose aria-pressed and the count is part of each name.",
            ],
            responsive: vec!["compact drops the labels for narrow cards."],
            height: 160,
            align: Align::Center,
            live: true,
        },
        Demo {
            slug: "empty-state",
            name: "Empty state",
            category: "data-display",
            summary: "What a list or panel shows while it has nothing yet: an icon, a title, one line and a first step, framed, bare or inline, with starter prompts for agent apps.",
            dioxus: "EmptyState",
            gpui: "EmptyStateEl",
            spec: "display-atoms",
            source: "primitives/display.rs",
            controls: vec![
                sel("layout", "Layout", &["framed", "bare", "inline"], "framed"),
                tog("action", "Action", true),
                tog("secondary", "Secondary link", true),
                tog("prompts", "Starter prompts", true),
                tog("tall", "Tall", false),
            ],
            variants: vec![
                var("framed", "Dashed frame and prompts.", &[]),
                var(
                    "bare",
                    "No frame.",
                    &[
                        ("layout", "bare"),
                        ("prompts", "false"),
                        ("secondary", "false"),
                    ],
                ),
                var(
                    "inline",
                    "One line in a table.",
                    &[
                        ("layout", "inline"),
                        ("prompts", "false"),
                        ("secondary", "false"),
                    ],
                ),
            ],
            render: empty,
            code: empty_code,
            keyboard: vec![("Tab", "To the action, the link and each prompt")],
            a11y: vec![
                "Title and text are plain text in reading order; the action and prompts are real buttons.",
            ],
            responsive: vec![
                "Framed and bare centre in their container; inline fits inside a table row.",
            ],
            height: 300,
            align: Align::Center,
            live: false,
        },
    ]
}
