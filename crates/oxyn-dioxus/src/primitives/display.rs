//! Display atoms: `Badge`, `Chip`, `ValuePill`, `Monogram`, `EntityChip`, `Avatar`,
//! `AvatarGroup`, `Kbd`, `Separator`, `Spinner`, `EmptyState`, `ReactionBar`, `NumberTicker`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        display::{
            Avatar as AvatarModel, AvatarShape, Badge as BadgeModel, ChipKind, Delta, EmptyLayout,
            EmptyState as EmptyModel, EntityChip as EntityModel, Fill, Monogram as MonogramModel,
            Platform, Presence, SpinnerStyle, Tone, agent_mark, aria_keyshortcuts, avatar_overflow,
            parse_shortcut,
        },
        ticker::{Column, CountRule, NumberTicker as TickerModel, Reaction},
    },
};

use super::cls;
use crate::{color_var, composer::icons::Icon};

fn mono_vars(i: usize) -> String {
    let (bg, fg) = oxyn_core::primitives::display::monogram_roles(i);
    format!("--mbg:{};--mfg:{}", color_var(bg), color_var(fg))
}

/// Badge: six tones (+ four agent-state tones), soft / outline / solid / mono, a dot or icon, a
/// live dot that pulses every 1.8 s, a round count up to `99+`.
#[component]
pub fn Badge(
    /// Model.
    badge: BadgeModel,
    /// `sm` · `md` · `lg`.
    #[props(default = "sm".to_string())]
    size: String,
    /// Leading icon, sized to the badge (12, 14 or 16 px).
    #[props(default)]
    icon: Option<String>,
    /// Roll the label: a changed label rises in with a short blur (keyed on the text).
    #[props(default)]
    roll: bool,
) -> Element {
    let b = badge;
    let icon_px = match size.as_str() {
        "lg" => 16.0,
        "md" => 14.0,
        _ => 12.0,
    };
    let (fg, bg) = b.tone.roles();
    let style = format!("--fg:{};--bg:{}", color_var(fg), color_var(bg));
    let round = b.count.is_some();
    let class = cls(&[
        "oxyn-badge",
        b.tone.id(),
        b.fill.id(),
        &size,
        if b.pill { "pill" } else { "" },
        if round { "count" } else { "" },
    ]);
    let text = b.text();
    let aria = b.aria();
    rsx! {
        span { class: "{class}", style: "{style}", role: if b.live { "status" } else { "" }, "aria-label": "{aria}",
            if b.dot || b.live { span { class: if b.live { "dot live" } else { "dot" }, "aria-hidden": "true" } }
            if let Some(i) = icon { Icon { name: i, size: icon_px, stroke: 2.0 } }
            if roll {
                for k in std::iter::once(text.clone()) { span { key: "{k}", class: "lbl roll", "aria-hidden": "true", "{k}" } }
            } else {
                span { "aria-hidden": "true", "{text}" }
            }
        }
    }
}

/// Chip: static, toggle (`aria-pressed`, a check grows in) or removable (× and Backspace /
/// Delete). Oxyn plus: count badge and a status dot.
#[component]
pub fn Chip(
    /// Label.
    label: String,
    /// `static` · `toggle` · `removable`.
    #[props(default = "static".to_string())]
    kind: String,
    /// Selected (toggles).
    #[props(default)]
    selected: bool,
    /// Count.
    #[props(default)]
    count: Option<u64>,
    /// Status dot tone.
    #[props(default)]
    dot: Option<String>,
    /// Disabled.
    #[props(default)]
    disabled: bool,
    /// Toggled.
    #[props(default)]
    on_toggle: EventHandler<bool>,
    /// Removed.
    #[props(default)]
    on_remove: EventHandler<()>,
) -> Element {
    let k = match kind.as_str() {
        "toggle" => ChipKind::Toggle,
        "removable" => ChipKind::Removable,
        _ => ChipKind::Static,
    };
    let mut on = use_signal(|| selected);
    let sel = *on.read();
    let dot_style = dot
        .as_deref()
        .and_then(Tone::from_id)
        .map(|t| format!("--dot:{}", color_var(t.roles().0)))
        .unwrap_or_default();
    let body = rsx! {
        if k == ChipKind::Toggle { span { class: "ck", "aria-hidden": "true", Icon { name: "check", size: 11.0, stroke: 2.2 } } }
        if dot.is_some() { span { class: "dot", style: "{dot_style}", "aria-hidden": "true" } }
        span { class: "lb", "{label}" }
        if let Some(c) = count { span { class: "cnt", "{c}" } }
    };
    match k {
        ChipKind::Toggle => rsx! {
            button { class: cls(&["oxyn-chip toggle", if sel { "on" } else { "" }]), r#type: "button", "aria-pressed": "{sel}", disabled,
                onclick: move |_| { let v = !*on.peek(); on.set(v); on_toggle.call(v) },
                {body}
            }
        },
        ChipKind::Removable => rsx! {
            span { class: "oxyn-chip removable", tabindex: "0", role: "group", "aria-label": "{label}",
                onkeydown: move |e: KeyboardEvent| if matches!(e.key(), Key::Backspace | Key::Delete) { e.prevent_default(); on_remove.call(()) },
                {body}
                button { class: "x", r#type: "button", tabindex: "-1", "aria-label": "Remove {label}", disabled, onclick: move |_| on_remove.call(()),
                    Icon { name: "x", size: 10.0, stroke: 2.0 }
                }
            }
        },
        ChipKind::Static => rsx! { span { class: "oxyn-chip", {body} } },
    }
}

/// Value pill (Beautiful UI, MIT © 2026 Shane Levine): a small value badge in neutral, green,
/// orange, red or accent. Oxyn plus: a delta arrow from the previous value (`prev`), and the
/// number rolls when it changes (`ticker`).
#[component]
pub fn ValuePill(
    /// Text ("7 days").
    value: String,
    /// `neutral` · `green` · `orange` · `red` · `accent` (or any badge tone).
    #[props(default = "neutral".to_string())]
    tone: String,
    /// Previous numeric value and the current one, for the delta arrow.
    #[props(default)]
    delta: Option<(f64, f64)>,
    /// Up is good (green) or bad (red, for cost and latency).
    #[props(default = true)]
    good_up: bool,
) -> Element {
    let accent = tone == "accent";
    let d = delta.map(|(a, b)| Delta::of(a, b));
    let t = match d {
        Some(d) => d.tone(good_up),
        None => Tone::from_id(&tone).unwrap_or_default(),
    };
    let (fg, bg) = t.roles();
    let style = if accent {
        "--fg:var(--oxyn-color-fg-on-accent);--bg:var(--oxyn-color-accent-solid)".to_string()
    } else {
        format!("--fg:{};--bg:{}", color_var(fg), color_var(bg))
    };
    let aria = match d {
        Some(d) => format!("{value}, {}", d.word()),
        None => value.clone(),
    };
    rsx! {
        span { class: "oxyn-vpill", style: "{style}", "aria-label": "{aria}",
            if let Some(d) = d { span { class: "dl", "aria-hidden": "true", "{d.glyph()}" } }
            span { "aria-hidden": "true", "{value}" }
        }
    }
}

/// Monogram: letters on a seeded colour pair.
#[component]
pub fn Monogram(
    /// Model.
    monogram: MonogramModel,
    /// Size px.
    #[props(default = 18.0)]
    size: f32,
    /// Square corners.
    #[props(default)]
    square: bool,
) -> Element {
    let st = format!("{};--s:{size}px", mono_vars(monogram.colour));
    rsx! { span { class: if square { "oxyn-mono sq" } else { "oxyn-mono" }, style: "{st}", "aria-hidden": "true", "{monogram.letters}" } }
}

/// Entity chip (Beautiful UI, MIT © 2026 Shane Levine): an inline company or person with a
/// monogram. Oxyn plus: avatar image, status dot and a hover card.
#[component]
pub fn EntityChip(
    /// Model.
    entity: EntityModel,
    /// Review sheets: show the hover card.
    #[props(default)]
    card_open: bool,
) -> Element {
    let e = entity;
    let mut open = use_signal(|| card_open);
    let has_card = !e.card.is_empty();
    let id = format!("ec-{}", e.monogram.letters.to_lowercase());
    rsx! {
        span { class: "oxyn-entity", tabindex: if has_card { "0" } else { "" },
            "aria-describedby": if has_card && *open.read() { "{id}" },
            onpointerenter: move |_| if has_card { open.set(true) },
            onpointerleave: move |_| open.set(false),
            onfocus: move |_| if has_card { open.set(true) },
            onblur: move |_| open.set(false),
            onkeydown: move |k: KeyboardEvent| if k.key() == Key::Escape { open.set(false) },
            if let Some(src) = e.image.clone() {
                img { class: "img", src: "{src}", alt: "" }
            } else {
                Monogram { monogram: e.monogram.clone(), size: 16.0 }
            }
            span { class: "nm", "{e.name}" }
            if let Some(p) = e.status { span { class: "st {p.id()}", "aria-label": "{p.id()}" } }
            if has_card && *open.read() {
                span { class: "card", role: "tooltip", id: "{id}",
                    for (k, v) in e.card.iter() { span { class: "kv", span { class: "k", "{k}" } span { class: "v", "{v}" } } }
                }
            }
        }
    }
}

/// Avatar: picture (fades in over the initials once loaded), initials on a seeded colour, a
/// workspace square, an agent's pixel mark that twinkles while it works, a presence dot and an
/// agent ring for its state (or progress). Oxyn plus: model badge, ring progress, seven sizes.
#[component]
pub fn Avatar(
    /// Model.
    avatar: AvatarModel,
) -> Element {
    let a = avatar;
    let mut loaded = use_signal(|| false);
    let mut failed = use_signal(|| false);
    let m = a.monogram();
    let size = a.size;
    let r = a.radius();
    let ring_id = a.ring.id();
    let ring_col = color_var(a.ring.role());
    let pad = (size * 0.12).clamp(2.0, 5.0);
    let outer = size + pad * 2.0;
    let style = format!(
        "{};--s:{size}px;--r:{r}px;--ring:{ring_col};--o:{outer}px;--fs:{:.1}px",
        mono_vars(m.colour),
        a.font()
    );
    let aria = a.aria();
    let mark = a
        .agent
        .then(|| agent_mark(a.seed.as_deref().unwrap_or(&a.name)));
    let prog = match a.ring {
        oxyn_core::primitives::display::AgentRing::Progress(p) => Some(p.clamp(0.0, 1.0)),
        _ => None,
    };
    let rr = outer / 2.0 - 1.0;
    let circ = std::f32::consts::TAU * rr;
    let square = a.shape == AvatarShape::Square;
    rsx! {
        span { class: cls(&["oxyn-avatar", if square { "sq" } else { "" }, if a.working { "working" } else { "" }, &format!("ring-{ring_id}")]),
            style: "{style}", role: "img", "aria-label": "{aria}",
            if ring_id != "none" {
                svg { class: "ring", width: "{outer}", height: "{outer}", view_box: "0 0 {outer} {outer}", "aria-hidden": "true",
                    if square {
                        rect { x: "1", y: "1", width: "{outer - 2.0}", height: "{outer - 2.0}", rx: "{r + pad}", fill: "none", stroke: "var(--ring)", stroke_width: "1.5" }
                    } else {
                        circle { cx: "{outer / 2.0}", cy: "{outer / 2.0}", r: "{rr}", fill: "none", stroke: "var(--ring)", stroke_width: "1.5",
                            stroke_dasharray: if let Some(p) = prog { "{circ * p:.1} {circ:.1}" } else if ring_id == "orbit" { "{circ * 0.22:.1} {circ:.1}" } else if ring_id == "dashed" { "3 3" },
                            transform: if prog.is_some() { "rotate(-90 {outer / 2.0} {outer / 2.0})" },
                        }
                    }
                }
            }
            span { class: "face",
                if let Some(g) = mark {
                    svg { class: "mark", width: "{size * 0.62}", height: "{size * 0.62}", view_box: "0 0 5 5", "aria-hidden": "true",
                        for (ri, row) in g.iter().enumerate() {
                            for (ci, on) in row.iter().enumerate() {
                                if *on { rect { x: "{ci}", y: "{ri}", width: "1", height: "1", rx: "0.18", style: "--d:{(ri + ci) as f32 * 0.15:.2}s" } }
                            }
                        }
                    }
                } else {
                    span { class: "ini", "aria-hidden": "true", "{m.letters}" }
                }
                if let Some(src) = a.src.clone().filter(|_| !failed()) {
                    img { class: if *loaded.read() { "pic loaded" } else { "pic" }, src: "{src}", alt: "", onload: move |_| loaded.set(true), onerror: move |_| failed.set(true) }
                }
            }
            if let Some(p) = a.status { span { class: "pres {p.id()}", style: "--d:{a.dot()}px", "aria-hidden": "true" } }
            if let Some(md) = a.model.clone() { span { class: "model", "aria-hidden": "true", "{md}" } }
        }
    }
}

/// Overlapping avatars with a `+N` tile.
#[component]
pub fn AvatarGroup(
    /// Avatars.
    avatars: Vec<AvatarModel>,
    /// Shown before the `+N` tile (4).
    #[props(default = 4)]
    max: usize,
    /// Total when the list is a page of a bigger set.
    #[props(default)]
    total: Option<usize>,
) -> Element {
    let (shown, rest) = avatar_overflow(avatars.len(), max, total);
    let size = avatars.first().map(|a| a.size).unwrap_or(32.0);
    let names: Vec<String> = avatars.iter().take(shown).map(|a| a.name.clone()).collect();
    let label = match rest {
        Some(n) => format!("{} and {n} more", names.join(", ")),
        None => names.join(", "),
    };
    rsx! {
        span { class: "oxyn-avgroup", role: "group", "aria-label": "{label}", style: "--s:{size}px",
            for a in avatars.iter().take(shown) { Avatar { avatar: a.clone() } }
            if let Some(n) = rest { span { class: "more", "aria-hidden": "true", "+{n}" } }
        }
    }
}

/// Keycaps for a shortcut. Oxyn plus: platform-aware glyphs (`Mod` = ⌘ or Ctrl), sequences
/// ("G I"), a pressed highlight, flat and inline styles.
#[component]
pub fn Kbd(
    /// Shortcut ("Mod+Shift+K").
    keys: String,
    /// Non-Mac glyphs.
    #[props(default)]
    other: bool,
    /// `keycap` · `flat` · `inline`.
    #[props(default = "keycap".to_string())]
    style: String,
    /// The keys are held right now.
    #[props(default)]
    pressed: bool,
) -> Element {
    let p = if other {
        Platform::Other
    } else {
        Platform::Mac
    };
    let chords = parse_shortcut(&keys, p);
    let spoken: Vec<String> = chords
        .iter()
        .map(|c| {
            c.iter()
                .map(|k| k.name.clone())
                .collect::<Vec<_>>()
                .join(" ")
        })
        .collect();
    let aria = spoken.join(", then ");
    let ks = aria_keyshortcuts(&keys, p);
    rsx! {
        span { class: cls(&["oxyn-kbd", &style, if pressed { "pressed" } else { "" }]), "aria-label": "{aria}", "aria-keyshortcuts": "{ks}",
            for (n, chord) in chords.iter().enumerate() {
                if n > 0 { span { class: "then", "aria-hidden": "true", "then" } }
                for k in chord.iter() { kbd { "aria-hidden": "true", "{k.glyph}" } }
            }
        }
    }
}

/// Separator: horizontal or vertical, decorative or semantic. Oxyn plus: a centred label,
/// dashed, faded ends, an inset.
#[component]
pub fn Separator(
    /// Vertical.
    #[props(default)]
    vertical: bool,
    /// Purely visual.
    #[props(default = true)]
    decorative: bool,
    /// Label.
    #[props(default)]
    label: Option<String>,
    /// Dashed.
    #[props(default)]
    dashed: bool,
    /// Faded ends.
    #[props(default)]
    fade: bool,
    /// Inset px.
    #[props(default)]
    inset: f32,
) -> Element {
    let class = cls(&[
        "oxyn-sep",
        if vertical { "v" } else { "h" },
        if dashed { "dashed" } else { "" },
        if fade { "fade" } else { "" },
        if label.is_some() { "labelled" } else { "" },
    ]);
    let role = if decorative { "none" } else { "separator" };
    rsx! {
        div { class: "{class}", role: "{role}", style: "--inset:{inset}px",
            "aria-orientation": if !decorative { if vertical { "vertical" } else { "horizontal" } },
            if let Some(l) = label { span { class: "lb", "{l}" } }
        }
    }
}

/// Spinner: a 2 px arc on a 20 % ring, one turn every 750 ms, linear; announced only with a
/// label. Oxyn plus: arc length, speed, four styles (arc, dots, ray, orbit).
#[component]
pub fn Spinner(
    /// Size px (14).
    #[props(default = 14.0)]
    size: f32,
    /// Spoken label.
    #[props(default)]
    label: Option<String>,
    /// `arc` · `dots` · `ray` · `orbit`.
    #[props(default = "arc".to_string())]
    style: String,
    /// One turn, ms (750).
    #[props(default = 750)]
    speed_ms: u32,
    /// Arc length 0..1 (0.25).
    #[props(default = 0.25)]
    arc: f32,
) -> Element {
    let st = match style.as_str() {
        "dots" => SpinnerStyle::Dots,
        "ray" => SpinnerStyle::Ray,
        "orbit" => SpinnerStyle::Orbit,
        _ => SpinnerStyle::Arc,
    };
    let sp = oxyn_core::primitives::display::Spinner {
        size,
        arc,
        ..Default::default()
    };
    let (dash, gap) = sp.dash();
    let sw = sp.stroke();
    let r = (size - sw) / 2.0;
    let c = size / 2.0;
    let vars = format!("--s:{size}px;--speed:{speed_ms}ms");
    let segs = match st {
        SpinnerStyle::Dots => 8,
        SpinnerStyle::Ray => 12,
        _ => 0,
    };
    rsx! {
        span { class: "oxyn-spinner {st.id()}", style: "{vars}", role: if label.is_some() { "status" } else { "" },
            "aria-label": if let Some(l) = &label { "{l}" }, "aria-hidden": if label.is_none() { "true" },
            svg { width: "{size}", height: "{size}", view_box: "0 0 {size} {size}",
                match st {
                    SpinnerStyle::Arc => rsx! {
                        circle { class: "track", cx: "{c}", cy: "{c}", r: "{r}", fill: "none", stroke_width: "{sw}" }
                        circle { class: "head", cx: "{c}", cy: "{c}", r: "{r}", fill: "none", stroke_width: "{sw}", stroke_linecap: "round", stroke_dasharray: "{dash:.2} {gap:.2}" }
                    },
                    SpinnerStyle::Orbit => rsx! {
                        circle { class: "track", cx: "{c}", cy: "{c}", r: "{r}", fill: "none", stroke_width: "1" }
                        circle { class: "head", cx: "{c}", cy: "{sw}", r: "{sw}", }
                    },
                    _ => rsx! {
                        for i in 0..segs {
                            {
                                let a = i as f32 / segs as f32 * 360.0;
                                let delay = -(speed_ms as f32) * (1.0 - i as f32 / segs as f32);
                                if st == SpinnerStyle::Dots {
                                    rsx! { circle { class: "seg", cx: "{c}", cy: "{sw}", r: "{sw * 0.7:.2}", transform: "rotate({a} {c} {c})", style: "animation-delay:{delay:.0}ms" } }
                                } else {
                                    rsx! { line { class: "seg", x1: "{c}", y1: "{sw * 0.6:.2}", x2: "{c}", y2: "{size * 0.3:.2}", stroke_width: "{sw * 0.75:.2}", stroke_linecap: "round", transform: "rotate({a} {c} {c})", style: "animation-delay:{delay:.0}ms" } }
                                }
                            }
                        }
                    },
                }
            }
        }
    }
}

/// Empty state: icon on a soft tile, title, one line and a first-step button; framed or bare,
/// tall. Oxyn plus: inline table-row layout, secondary action / docs link, starter prompts.
#[component]
pub fn EmptyState(
    /// Model.
    state: EmptyModel,
    /// Primary action.
    #[props(default)]
    on_action: EventHandler<()>,
    /// Secondary action.
    #[props(default)]
    on_secondary: EventHandler<()>,
    /// A starter prompt was picked.
    #[props(default)]
    on_prompt: EventHandler<String>,
) -> Element {
    let s = state;
    let icon = if s.icon.is_empty() {
        "file".to_string()
    } else {
        s.icon.clone()
    };
    if s.layout == EmptyLayout::Inline {
        return rsx! {
            div { class: "oxyn-empty inline", role: "status",
                Icon { name: icon, size: 14.0 }
                span { class: "tt", "{s.title}" }
                if !s.text.is_empty() { span { class: "tx", "{s.text}" } }
                if let Some(a) = s.action.clone() { button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| on_action.call(()), span { class: "lb", "{a}" } } }
            }
        };
    }
    rsx! {
        div { class: cls(&["oxyn-empty", s.layout.id(), if s.tall { "tall" } else { "" }]), role: "status",
            span { class: "tile", Icon { name: icon, size: 18.0 } }
            div { class: "tt", "{s.title}" }
            if !s.text.is_empty() { div { class: "tx", "{s.text}" } }
            if s.action.is_some() || s.secondary.is_some() {
                div { class: "acts",
                    if let Some(a) = s.action.clone() { button { class: "oxyn-btn primary sm", r#type: "button", onclick: move |_| on_action.call(()), Icon { name: "plus", size: 14.0 } span { class: "lb", "{a}" } } }
                    if let Some(b) = s.secondary.clone() { button { class: "oxyn-btn ghost sm", r#type: "button", onclick: move |_| on_secondary.call(()), span { class: "lb", "{b}" } Icon { name: "external", size: 12.0 } } }
                }
            }
            if !s.prompts.is_empty() {
                div { class: "prompts",
                    for p in s.prompts.iter() {
                        {
                            let pc = p.clone();
                            rsx! { button { class: "oxyn-chip toggle", r#type: "button", onclick: move |_| on_prompt.call(pc.clone()), Icon { name: "spark", size: 11.0 } span { class: "lb", "{p}" } } }
                        }
                    }
                }
            }
        }
    }
}

fn rolled(text: &str, up: bool) -> Element {
    let n = text.chars().count();
    rsx! {
        span { class: if up { "roll up" } else { "roll down" }, "aria-hidden": "true",
            for (i, ch) in text.chars().enumerate() {
                span { key: "{n - i}-{ch}", class: if ch.is_ascii_digit() { "dg" } else { "sym" }, "{ch}" }
            }
        }
    }
}

/// Reaction bar: like, comment, repost and share in one pill. Icons pop and fill with their
/// colour; changed digits roll in. Oxyn plus: custom sets, compact icon-only, count rules.
#[component]
pub fn ReactionBar(
    /// Reactions and actions.
    reactions: Vec<Reaction>,
    /// Icon-only.
    #[props(default)]
    compact: bool,
    /// Count rule.
    #[props(default)]
    rule: CountRule,
    /// Clock (pop scale).
    #[props(default)]
    now: ClockMs,
    /// A reaction was pressed (`id`, now active).
    #[props(default)]
    on_react: EventHandler<(String, bool)>,
) -> Element {
    let mut list = use_signal(|| reactions.clone());
    let mut dir = use_signal(|| true);
    rsx! {
        div { class: cls(&["oxyn-reactions", if compact { "compact" } else { "" }]), role: "group", "aria-label": "Reactions",
            for (i, r) in list.read().iter().enumerate() {
                {
                    let tint = color_var(r.tint);
                    let count = rule.show(r.count);
                    let aria = r.aria(&rule);
                    let scale = r.pop(now, false);
                    let id = r.id.clone();
                    rsx! {
                        button { class: cls(&["rx", if r.active { "on" } else { "" }]), r#type: "button", style: "--tint:{tint};--pop:{scale:.3}",
                            "aria-label": "{aria}", "aria-pressed": if r.toggles { "{r.active}" },
                            onclick: move |_| {
                                let mut l = list.write();
                                let was = l[i].count;
                                let on = l[i].press(now);
                                dir.set(l[i].count >= was);
                                on_react.call((id.clone(), on));
                            },
                            span { class: "ic", key: "{r.active}", Icon { name: r.icon.clone(), size: 16.0 } }
                            if !compact && !count.is_empty() { {rolled(&count, *dir.read())} }
                        }
                    }
                }
            }
        }
    }
}

/// Number ticker: odometer digits that roll up when the value grows and down when it shrinks;
/// new columns slide in; counts in from zero on first view. Drive with the model and the clock.
#[component]
pub fn NumberTicker(
    /// Model.
    ticker: TickerModel,
    /// Clock.
    now: ClockMs,
    /// Reduced motion.
    #[props(default)]
    reduced: bool,
    /// Text size px.
    #[props(default = 28.0)]
    size: f32,
) -> Element {
    let cols = ticker.columns(now, reduced);
    let text = ticker.text();
    rsx! {
        span { class: "oxyn-ticker", style: "--fs:{size}px", role: "text", "aria-label": "{text}",
            for (i, c) in cols.iter().enumerate() {
                match c {
                    Column::Digit { pos, .. } => {
                        let base = pos.floor() as u32 % 10;
                        let f = pos.fract();
                        rsx! {
                            span { key: "{i}", class: "col", "aria-hidden": "true",
                                span { class: "strip", style: "transform:translateY(-{f:.3}em)",
                                    span { "{base}" } span { "{(base + 1) % 10}" }
                                }
                            }
                        }
                    }
                    Column::Enter { ch, k } => rsx! { span { key: "{i}", class: "col enter", style: "--k:{k:.3}", "aria-hidden": "true", span { class: "strip", "{ch}" } } },
                    Column::Exit { ch, k } => rsx! { span { key: "{i}", class: "col exit", style: "--k:{k:.3}", "aria-hidden": "true", span { class: "strip", "{ch}" } } },
                    Column::Static(ch) => rsx! { span { key: "{i}", class: "sym", "aria-hidden": "true", "{ch}" } },
                }
            }
        }
    }
}

/// Presence for docs and sheets.
pub fn presence_from(s: &str) -> Option<Presence> {
    Some(match s {
        "online" => Presence::Online,
        "away" => Presence::Away,
        "busy" => Presence::Busy,
        "offline" => Presence::Offline,
        _ => return None,
    })
}

/// Fill from an id.
pub fn fill_from(s: &str) -> Fill {
    match s {
        "outline" => Fill::Outline,
        "solid" => Fill::Solid,
        "mono" => Fill::Mono,
        _ => Fill::Soft,
    }
}
