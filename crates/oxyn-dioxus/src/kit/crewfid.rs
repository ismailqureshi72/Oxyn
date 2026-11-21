//! affectUI fidelity pass, overnight batch, family B part 2 (plan #32): the exact **Agent Crew**
//! ([`CrewTree`]). Behaviour and look only; the model is `oxyn_core::kit::crewfid`. Styles:
//! `crewfid.css` (classes `oxcw-*`), in `PRIMITIVES_CSS`. Follows the nearest `data-cstyle`
//! (Clean / Classic) unless `style` is set; time comes in as `now`.

use std::{collections::HashMap, rc::Rc};

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::crewfid::{
        CrewCard, CrewHandoff, CrewProject, CrewState, clamp_context, context_tone, crew_at,
        crew_cost, crew_duration, crew_loop_ms, crew_projects, handoff_hops, handoff_said,
        running_ms, short_tokens, steer_label, steer_result, steer_said, tokens_label,
    },
};

use crate::{brand::BrandIcon, composer::icons::Icon};

/// Card width, gap between sub-agents, indent of helpers (px).
const CW: f32 = 205.0;
const GAP: f32 = 38.0;
const IND: f32 = 28.0;
/// Heights of a lead / sub-agent card and of a helper card; gaps (px).
const MAIN_H: f32 = 112.0;
const HELP_H: f32 = 79.0;
const VGAP: f32 = 40.0;
const HGAP: f32 = 14.0;
/// Where a helper's connector comes in, from its top; the corner radius.
const ELBOW: f32 = 21.0;
const R: f32 = 6.0;
/// The details card's height, about.
const DLG_H: f32 = 262.0;
/// One hop of a handoff light (ms).
const HOP: u64 = 520;
/// The full width the side-by-side layout is drawn for.
const WIDE: f32 = 760.0;

const PALETTE: [&str; 6] = ["purple", "cyan", "green", "pink", "orange", "blue"];
/// 3×3 pixel marks (bit 0 = top left, row by row).
const MARKS: [u16; 8] = [
    0b111_111_111,
    0b111_111_111,
    0b010_010_101,
    0b101_111_010,
    0b011_100_011,
    0b111_101_000,
    0b110_011_110,
    0b010_111_101,
];

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

/// (key, entry edge wide, entry edge narrow, delay ms) of a Gradient Floor.
type Floor = (String, &'static str, &'static str, u64);
/// (parent slot, circle wide, circle narrow, colour).
type Knob = (usize, (f32, f32), (f32, f32), &'static str);
/// (key, path, runs up, delay ms, colour) of a handoff light.
type Light = (String, String, bool, u64, &'static str);

#[derive(Clone, PartialEq)]
struct Slot {
    i: usize,
    depth: usize,
    parent: Option<usize>,
    card: CrewCard,
    color: &'static str,
    mark: u16,
    h: f32,
    w: (f32, f32),
    n: (f32, f32),
}

/// Lays the tree out twice: side by side (wide) and indented (narrow).
fn layout(root: &CrewCard) -> (Vec<Slot>, f32, f32) {
    let flat = root.flatten();
    let n_kids = root.children.len();
    let span = if n_kids == 0 {
        CW
    } else {
        n_kids as f32 * CW + (n_kids as f32 - 1.0) * GAP
    };
    let extra = root
        .children
        .last()
        .map(|c| {
            c.flatten()
                .iter()
                .map(|x| x.0 as f32 * IND)
                .fold(0.0, f32::max)
        })
        .unwrap_or(0.0);
    let ox = ((WIDE - span - extra) / 2.0).max(0.0);
    let mut slots: Vec<Slot> = vec![];
    let mut col_y: Vec<f32> = vec![MAIN_H + VGAP + MAIN_H + HGAP; n_kids.max(1)];
    let mut col = 0usize;
    let mut kid = 0usize;
    let mut ny = 0.0f32;
    for (i, (depth, parent, c)) in flat.iter().enumerate() {
        let h = if *depth <= 1 { MAIN_H } else { HELP_H };
        let color = if *depth == 0 {
            "blue"
        } else if *depth == 1 {
            PALETTE[kid % PALETTE.len()]
        } else {
            parent.map(|p| slots[p].color).unwrap_or("blue")
        };
        let w = match depth {
            0 => (ox + (span - CW) / 2.0, 0.0),
            1 => {
                col = kid;
                kid += 1;
                (ox + col as f32 * (CW + GAP), MAIN_H + VGAP)
            }
            d => {
                let x = ox + col as f32 * (CW + GAP) + (*d as f32 - 1.0) * IND;
                let y = col_y[col];
                col_y[col] += HELP_H + HGAP;
                (x, y)
            }
        };
        let nx = *depth as f32 * IND;
        let n = (nx, ny);
        ny += h + HGAP;
        let mark = if *depth == 0 {
            MARKS[0]
        } else {
            MARKS[1 + (i % (MARKS.len() - 1))]
        };
        slots.push(Slot {
            i,
            depth: *depth,
            parent: *parent,
            card: (*c).clone(),
            color,
            mark,
            h,
            w,
            n,
        });
    }
    let wh = slots.iter().map(|s| s.w.1 + s.h).fold(MAIN_H, f32::max);
    let nh = (ny - HGAP).max(MAIN_H);
    (slots, wh, nh)
}

/// The connector into `s` from its parent, wide or narrow.
fn edge(slots: &[Slot], s: &Slot, wide: bool) -> Option<String> {
    let p = &slots[s.parent?];
    let ((px, py), (cx, cy)) = if wide { (p.w, s.w) } else { (p.n, s.n) };
    if wide && s.depth == 1 {
        let x0 = px + CW / 2.0;
        let y0 = py + p.h + 7.5;
        let bus = py + p.h + 20.0;
        let x1 = cx + CW / 2.0;
        if (x1 - x0).abs() < 1.0 {
            return Some(format!("M{x0:.1} {y0:.1}V{cy:.1}"));
        }
        let dir = if x1 > x0 { 1.0 } else { -1.0 };
        return Some(format!(
            "M{x0:.1} {y0:.1}V{a:.1}Q{x0:.1} {bus:.1} {b:.1} {bus:.1}H{c:.1}Q{x1:.1} {bus:.1} {x1:.1} {d:.1}V{cy:.1}",
            a = bus - R,
            b = x0 + dir * R,
            c = x1 - dir * R,
            d = bus + R,
        ));
    }
    let x0 = px + 19.0;
    let y0 = py + p.h + 7.5;
    let ye = cy + ELBOW;
    Some(format!(
        "M{x0:.1} {y0:.1}V{a:.1}Q{x0:.1} {ye:.1} {b:.1} {ye:.1}H{cx:.1}",
        a = ye - R,
        b = x0 + R,
    ))
}

/// Where a connector leaves a parent: the little open circle.
fn knob(s: &Slot, wide: bool) -> (f32, f32) {
    let (x, y) = if wide { s.w } else { s.n };
    if wide && s.depth == 0 {
        (x + CW / 2.0, y + s.h + 4.0)
    } else {
        (x + 19.0, y + s.h + 4.0)
    }
}

/// Opens the details below the card, or above where the crew ends.
fn opens_up(s: &Slot, wide: bool, tree_h: f32) -> bool {
    let y = if wide { s.w.1 } else { s.n.1 };
    y + s.h + 6.0 + DLG_H > tree_h && y >= DLG_H + 6.0
}

#[component]
fn PixelMark(mask: u16, active: bool) -> Element {
    rsx! {
        svg { class: "oxcw-px", "data-on": "{active}", width: "12", height: "12", view_box: "0 0 12 12", "aria-hidden": "true",
            for k in 0..9u16 {
                if mask & (1 << k) != 0 {
                    rect { key: "{k}", x: "{(k % 3) * 4 + 1}", y: "{(k / 3) * 4 + 1}", width: "2.4", height: "2.4", rx: ".6",
                        style: "animation-delay:{((k % 3) + (k / 3)) * 140}ms" }
                }
            }
        }
    }
}

#[component]
fn StateBadge(state: CrewState, icon_only: bool) -> Element {
    let label = state.label();
    rsx! {
        span { class: "oxcw-bdg", "data-st": "{state.id()}", "data-icon": "{icon_only}", title: "{label}",
            match state {
                CrewState::Active => rsx! { svg { class: "oxcw-spin", width: "10", height: "10", view_box: "0 0 10 10", "aria-hidden": "true",
                    circle { cx: "5", cy: "5", r: "3.6", fill: "none", stroke: "currentColor", stroke_width: "1.4", opacity: ".3" }
                    path { d: "M5 1.4A3.6 3.6 0 0 1 8.6 5", fill: "none", stroke: "currentColor", stroke_width: "1.4", stroke_linecap: "round" } } },
                CrewState::Done => rsx! { Icon { name: "check", size: 10.0, stroke: 2.2 } },
                CrewState::Waiting => rsx! { Icon { name: "clock", size: 10.0, stroke: 2.0 } },
                CrewState::Failed => rsx! { Icon { name: "alert", size: 10.0, stroke: 2.0 } },
                CrewState::Paused => rsx! { Icon { name: "pause", size: 9.0, stroke: 2.0 } },
                CrewState::Idle => rsx! { i { class: "oxcw-dot" } },
            }
            if !icon_only { span { "{label}" } } else { span { class: "oxcw-sr", "{label}" } }
        }
    }
}

/// Agent Crew (affectUI): a lead and its sub-agents as one live tree of cards. Every card keeps
/// its place and says what the agent does now (a twinkling pixel mark and a shimmering line
/// while it works), how full its context is (a thin bar in its colour – orange from 80 %, red
/// from 95 %, with "136k / 200k"), its tools and the last thing it was handed. Handoffs run as a
/// light along the rounded connectors, hop by hop, and the card they reach lights up with a
/// Gradient Floor in its colour. A click opens the details under the card – task, model, usage
/// (time, cost, tokens), tools, last handoff and steering (Message, Pause / Resume, Stop,
/// Approve / Decline, Retry). The header's switcher swaps projects. Three sub-agents sit side by
/// side in 760 px; narrower, every level is indented.
#[component]
pub fn CrewTree(
    /// The lead and its tree (the demo project's crew when `None`).
    #[props(default)]
    root: Option<CrewCard>,
    /// Handoffs; new ids run a light (the demo's when `root` is `None`).
    #[props(default)]
    handoffs: Vec<CrewHandoff>,
    /// Header title when there are no projects.
    #[props(default)]
    title: String,
    /// Projects for the switcher (the demo's three when `root` is `None` and this is empty).
    #[props(default)]
    projects: Vec<CrewProject>,
    /// The project at the start.
    #[props(default = "checkout".to_string())]
    project: String,
    /// Freeze the demo at this many ms into a pass (a settled state).
    #[props(default)]
    at: Option<u64>,
    /// The card whose details are open at the start (agent id).
    #[props(default)]
    open: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A steering action: (agent id, action, message).
    #[props(default)]
    on_steer: EventHandler<(String, String, String)>,
    /// A card was opened (agent id).
    #[props(default)]
    on_select: EventHandler<String>,
    /// Another project was chosen (its id).
    #[props(default)]
    on_project: EventHandler<String>,
) -> Element {
    let demo = root.is_none();
    let projects = if projects.is_empty() && demo {
        crew_projects()
    } else {
        projects
    };
    let mut proj = use_signal(move || project.clone());
    let mut start = use_signal(move || now);
    let mut opened = use_signal(move || (!open.is_empty()).then(|| open.clone()));
    let mut cur = use_signal(|| 0usize);
    let mut menu = use_signal(|| false);
    let mut menu_i = use_signal(|| 0usize);
    let mut msg_open = use_signal(|| false);
    let mut draft = use_signal(String::new);
    let mut overrides = use_signal(HashMap::<String, (CrewState, String)>::new);
    let mut lasts = use_signal(HashMap::<String, (String, String)>::new);
    let mut said = use_signal(|| (0u64, String::new()));
    let mut handles = use_signal(Vec::<Option<Rc<MountedData>>>::new);
    let mut switch_btn = use_signal(|| None::<Rc<MountedData>>);
    let mut close_btn = use_signal(|| None::<Rc<MountedData>>);

    let raw = now.since(start()).as_millis() as u64;
    let p = proj();
    let (lap, el) = match (demo, at) {
        (true, Some(a)) => (0, a),
        (true, None) => {
            let l = crew_loop_ms(&p).max(1);
            (raw / l, raw % l)
        }
        _ => (0, raw),
    };
    let (mut tree, hand) = if let Some(r) = root.clone() {
        (r, handoffs.clone())
    } else {
        crew_at(&p, el)
    };
    // Steering applied on top of the data (the demo steers itself).
    for (k, (st, act)) in overrides().iter() {
        if let Some((pp, id)) = k.split_once('/')
            && pp == p
            && let Some(c) = tree.find_mut(id)
        {
            c.state = *st;
            c.activity = act.clone();
        }
    }
    for (k, last) in lasts().iter() {
        if let Some((pp, id)) = k.split_once('/')
            && pp == p
            && let Some(c) = tree.find_mut(id)
        {
            c.last = Some(last.clone());
        }
    }
    let (working, needs, all) = tree.counts();
    let (slots, wh, nh) = layout(&tree);
    if handles.peek().len() != slots.len() {
        handles.write().resize(slots.len(), None);
    }
    let cur_i = cur().min(slots.len().saturating_sub(1));
    let open_i = opened().and_then(|id| slots.iter().position(|s| s.card.id == id));
    // Room for the details where they hang below the crew.
    let (mut wh2, mut nh2) = (wh, nh);
    if let Some(i) = open_i {
        let s = &slots[i];
        if !opens_up(s, true, wh) {
            wh2 = wh2.max(s.w.1 + s.h + 6.0 + DLG_H);
        }
        if !opens_up(s, false, nh) {
            nh2 = nh2.max(s.n.1 + s.h + 6.0 + DLG_H);
        }
    }

    // Lights running now: (key, slot index of the hop, up, delay ms).
    let mut lights: Vec<(String, usize, bool, u64)> = vec![];
    let mut floors: HashMap<usize, Floor> = HashMap::new();
    for h in hand.iter() {
        let hops = handoff_hops(&tree, &h.from, &h.to);
        let age = el.saturating_sub(h.at_ms);
        let span = hops.len() as u64 * HOP + 1400;
        if h.at_ms == 0 || h.at_ms > el || age > span || at.is_some() {
            continue;
        }
        for (k, (si, up)) in hops.iter().enumerate() {
            lights.push((format!("{}-{lap}-{k}", h.id), *si, *up, k as u64 * HOP));
        }
        if let Some(t) = slots.iter().position(|s| s.card.id == h.to) {
            let last_up = hops.last().map(|x| x.1).unwrap_or(false);
            let s = &slots[t];
            let ew = if last_up {
                "bottom"
            } else if s.depth == 1 {
                "top"
            } else {
                "left"
            };
            let en = if last_up { "bottom" } else { "left" };
            floors.insert(
                t,
                (format!("{}-{lap}", h.id), ew, en, hops.len() as u64 * HOP),
            );
        }
    }
    let latest = hand
        .iter()
        .filter(|h| h.at_ms > 0 && h.at_ms <= el)
        .max_by_key(|h| h.at_ms)
        .map(|h| (h.at_ms, handoff_said(&tree, h)));
    let live = match latest {
        Some((t, s)) if t >= said().0 || said().1.is_empty() => s,
        _ => said().1,
    };

    let title_now = projects
        .iter()
        .find(|x| x.id == p)
        .map(|x| x.name.clone())
        .unwrap_or_else(|| {
            if title.is_empty() {
                "Agents".to_string()
            } else {
                title.clone()
            }
        });
    let proj_rows: Vec<(usize, CrewProject, usize, usize)> = projects
        .iter()
        .cloned()
        .enumerate()
        .map(|(i, x)| {
            let (w, n, _) = if x.id == p {
                (working, needs, 0)
            } else if demo {
                crew_at(&x.id, at.unwrap_or(el)).0.counts()
            } else {
                (0, 0, 0)
            };
            (i, x, w, n)
        })
        .collect();
    let cur_proj = proj_rows.iter().position(|r| r.1.id == p).unwrap_or(0);
    let paths_w: Vec<(usize, String, &'static str)> = slots
        .iter()
        .filter_map(|s| edge(&slots, s, true).map(|d| (s.i, d, slots[s.parent.unwrap_or(0)].color)))
        .collect();
    let paths_n: Vec<(usize, String, &'static str)> = slots
        .iter()
        .filter_map(|s| {
            edge(&slots, s, false).map(|d| (s.i, d, slots[s.parent.unwrap_or(0)].color))
        })
        .collect();
    let knobs: Vec<Knob> = slots
        .iter()
        .filter(|s| slots.iter().any(|k| k.parent == Some(s.i)))
        .map(|s| (s.i, knob(s, true), knob(s, false), s.color))
        .collect();
    let light_w: Vec<Light> = lights
        .iter()
        .filter_map(|(k, si, up, d)| {
            paths_w
                .iter()
                .find(|x| x.0 == *si)
                .map(|x| (k.clone(), x.1.clone(), *up, *d, x.2))
        })
        .collect();
    let light_n: Vec<Light> = lights
        .iter()
        .filter_map(|(k, si, up, d)| {
            paths_n
                .iter()
                .find(|x| x.0 == *si)
                .map(|x| (k.clone(), x.1.clone(), *up, *d, x.2))
        })
        .collect();
    let n_slots = slots.len();
    let pk = p.clone();

    rsx! {
        div { class: "oxcw", "data-style": style_attr(&style),
            div { class: "oxcw-hd",
                if projects.is_empty() {
                    span { class: "oxcw-ttl", Icon { name: "folder", size: 14.0 } b { "{title_now}" } }
                } else {
                    div { class: "oxcw-sw",
                        button { class: "oxcw-ttl oxcw-swb", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{menu()}",
                            onmounted: move |e| switch_btn.set(Some(e.data())),
                            onclick: move |_| {
                                menu_i.set(cur_proj);
                                menu.set(!menu());
                            },
                            Icon { name: "folder", size: 14.0 } b { "{title_now}" } Icon { name: "chevron", size: 12.0, stroke: 1.8 }
                        }
                        if menu() {
                            ul { class: "oxcw-menu", role: "listbox", "aria-label": "Projects", tabindex: "0",
                                onmounted: move |e| focus(Some(e.data())),
                                onkeydown: {
                                    let ids: Vec<String> = proj_rows.iter().map(|r| r.1.id.clone()).collect();
                                    move |e: KeyboardEvent| match e.key() {
                                        Key::ArrowDown => { e.prevent_default(); menu_i.set((menu_i() + 1) % ids.len().max(1)) }
                                        Key::ArrowUp => { e.prevent_default(); menu_i.set((menu_i() + ids.len().max(1) - 1) % ids.len().max(1)) }
                                        Key::Enter => {
                                            e.prevent_default();
                                            if let Some(id) = ids.get(menu_i()) {
                                                proj.set(id.clone()); start.set(now); opened.set(None); menu.set(false);
                                                on_project.call(id.clone());
                                            }
                                            focus(switch_btn());
                                        }
                                        Key::Escape => { menu.set(false); focus(switch_btn()) }
                                        _ => {}
                                    }
                                },
                                for (i, x, w, nd) in proj_rows.iter().cloned() {
                                    li { key: "{x.id}", class: "oxcw-opt", role: "option", "aria-selected": "{x.id == p}", "data-act": "{menu_i() == i}",
                                        onmouseenter: move |_| menu_i.set(i),
                                        onclick: { let id = x.id.clone(); move |_| {
                                            proj.set(id.clone()); start.set(now); opened.set(None); menu.set(false);
                                            on_project.call(id.clone());
                                        } },
                                        Icon { name: "folder", size: 14.0 }
                                        span { class: "oxcw-on", "{x.name}" }
                                        if nd > 0 { span { class: "oxcw-om", "data-k": "needs", "{nd} needs you" } }
                                        else { span { class: "oxcw-om", "{w} working" } }
                                        span { class: "oxcw-ock", if x.id == p { Icon { name: "check", size: 13.0, stroke: 2.0 } } }
                                    }
                                }
                            }
                        }
                    }
                }
                span { class: "oxcw-sp" }
                if working > 0 { span { class: "oxcw-cnt", "data-k": "working", "{working} working" } }
                if needs > 0 { span { class: "oxcw-cnt", "data-k": "needs", "{needs} needs you" } }
                span { class: "oxcw-cnt", "{all} agents" }
            }
            div { key: "{pk}", "data-project": "{pk}", class: "oxcw-tree", role: "tree", "aria-label": "{title_now}",
                style: "--wh:{wh2}px;--nh:{nh2}px",
                onkeydown: move |e: KeyboardEvent| {
                    let n = n_slots;
                    if n == 0 { return; }
                    let c = cur();
                    let next = match e.key() {
                        Key::ArrowRight | Key::ArrowDown => Some((c + 1).min(n - 1)),
                        Key::ArrowLeft | Key::ArrowUp => Some(c.saturating_sub(1)),
                        Key::Home => Some(0),
                        Key::End => Some(n - 1),
                        _ => None,
                    };
                    if let Some(x) = next {
                        e.prevent_default();
                        cur.set(x);
                        focus(handles.peek().get(x).cloned().flatten());
                    }
                },
                svg { class: "oxcw-lines", "data-l": "wide", width: "{WIDE}", height: "{wh}", "aria-hidden": "true",
                    for (i, d, col) in paths_w.iter().cloned() {
                        path { key: "{i}", "data-i": "{i}", class: "oxcw-ln", d: "{d}", "data-c": "{col}", "pathLength": "1" }
                    }
                    for (k, d, up, delay, col) in light_w.iter().cloned() {
                        g { key: "{k}", "data-k": "{k}", class: "oxcw-lt", "data-c": "{col}", "data-up": "{up}", style: "animation-delay:{delay}ms",
                            path { class: "oxcw-glow", d: "{d}", "pathLength": "1", style: "animation-delay:{delay}ms" }
                            path { class: "oxcw-core", d: "{d}", "pathLength": "1", style: "animation-delay:{delay}ms" }
                        }
                    }
                    for (i, (x, y), _, col) in knobs.iter().cloned() {
                        circle { key: "k{i}", "data-i": "{i}", class: "oxcw-knob", "data-c": "{col}", cx: "{x}", cy: "{y}", r: "3" }
                    }
                }
                svg { class: "oxcw-lines", "data-l": "narrow", width: "64", height: "{nh}", "aria-hidden": "true",
                    for (i, d, col) in paths_n.iter().cloned() {
                        path { key: "{i}", "data-i": "{i}", class: "oxcw-ln", d: "{d}", "data-c": "{col}", "pathLength": "1" }
                    }
                    for (k, d, up, delay, col) in light_n.iter().cloned() {
                        g { key: "{k}", "data-k": "{k}", class: "oxcw-lt", "data-c": "{col}", "data-up": "{up}",
                            path { class: "oxcw-glow", d: "{d}", "pathLength": "1", style: "animation-delay:{delay}ms" }
                            path { class: "oxcw-core", d: "{d}", "pathLength": "1", style: "animation-delay:{delay}ms" }
                        }
                    }
                    for (i, _, (x, y), col) in knobs.iter().cloned() {
                        circle { key: "k{i}", "data-i": "{i}", class: "oxcw-knob", "data-c": "{col}", cx: "{x}", cy: "{y}", r: "3" }
                    }
                }
                for s in slots.iter().cloned() {
                    {
                        let i = s.i;
                        let c = s.card.clone();
                        let is_open = open_i == Some(i);
                        let ctx = clamp_context(c.context);
                        let pct = (ctx * 1000.0).round() / 10.0;
                        let tone = context_tone(ctx);
                        let helper = s.depth >= 2;
                        let up_w = opens_up(&s, true, wh);
                        let up_n = opens_up(&s, false, nh);
                        let floor = floors.get(&i).cloned();
                        let more = c.tools.len() > 4;
                        let shown: Vec<(String, String)> = c.tools.iter().take(if more { 3 } else { 4 }).cloned().collect();
                        let extra = c.tools.len().saturating_sub(3);
                        let aria = format!("{}: {}, {}, context {}%", c.name, c.state.label(), c.activity, (ctx * 100.0).round());
                        let ran = running_ms(&c, el);
                        let id_open = c.id.clone();
                        let id_sel = c.id.clone();
                        rsx! {
                            div { key: "{c.id}", class: "oxcw-slot", "data-depth": "{s.depth.min(2)}", "data-open": "{is_open}",
                                "data-upw": "{up_w}", "data-upn": "{up_n}",
                                style: "--wx:{s.w.0}px;--wy:{s.w.1}px;--nx:{s.n.0}px;--ny:{s.n.1}px;--h:{s.h}px;--ac:var(--oxcw-{s.color});--rise:{i * 40}ms",
                                div { class: "oxcw-c", role: "treeitem", "aria-level": "{s.depth + 1}", "aria-label": "{aria}",
                                    "aria-haspopup": "dialog", "aria-expanded": "{is_open}", "data-st": "{c.state.id()}",
                                    tabindex: if i == cur_i { "0" } else { "-1" },
                                    onmounted: move |e| { if let Some(h) = handles.write().get_mut(i) { *h = Some(e.data()); } },
                                    onfocus: move |_| cur.set(i),
                                    onclick: move |_| {
                                        cur.set(i);
                                        msg_open.set(false);
                                        if opened() == Some(id_open.clone()) { opened.set(None) } else {
                                            opened.set(Some(id_open.clone()));
                                            on_select.call(id_sel.clone());
                                        }
                                    },
                                    onkeydown: {
                                        let id = c.id.clone();
                                        move |e: KeyboardEvent| match e.key() {
                                            Key::Enter => { e.prevent_default(); if opened() == Some(id.clone()) { opened.set(None) } else { opened.set(Some(id.clone())); on_select.call(id.clone()); } }
                                            Key::Character(ref k) if k == " " => { e.prevent_default(); if opened() == Some(id.clone()) { opened.set(None) } else { opened.set(Some(id.clone())); on_select.call(id.clone()); } }
                                            Key::Escape => opened.set(None),
                                            _ => {}
                                        }
                                    },
                                    if let Some((fk, ew, en, delay)) = floor {
                                        i { key: "{fk}", "data-k": "{fk}", class: "oxcw-floor", "data-ew": "{ew}", "data-en": "{en}", style: "animation-delay:{delay}ms" }
                                    }
                                    div { class: "oxcw-top",
                                        PixelMark { mask: s.mark, active: c.state == CrewState::Active }
                                        span { class: "oxcw-nm",
                                            b { title: "{c.name}", "{c.name}" }
                                            if !helper && !c.model.is_empty() { span { crate::brand::MarkFor { name: c.model.clone(), size: 11.0 } "{c.model}" } }
                                        }
                                        StateBadge { state: c.state, icon_only: helper }
                                    }
                                    div { class: "oxcw-act", "data-on": "{c.state == CrewState::Active}", title: "{c.activity}",
                                        span { key: "{c.activity}", "{c.activity}" }
                                    }
                                    div { class: "oxcw-ctx", "data-tone": "{tone}",
                                        span { class: "oxcw-bar", i { style: "width:{pct}%" } }
                                        span { class: "oxcw-tok", "{tokens_label(&c)}" }
                                    }
                                    if !helper {
                                        div { class: "oxcw-ft",
                                            span { class: "oxcw-tools",
                                                for (b, n) in shown.iter().cloned() {
                                                    span { key: "{n}", class: "oxcw-tool", title: "{n}", BrandIcon { name: b, size: 12.0, coloured: true, fallback: true } }
                                                }
                                                if more { span { class: "oxcw-more", "+{extra}" } }
                                            }
                                            if let Some((from, text)) = c.last.clone() {
                                                span { class: "oxcw-last", title: "{from}: {text}",
                                                    span { class: "oxcw-arr", "↳" } b { "{from}" } span { "{text}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                if is_open {
                                    div { class: "oxcw-dlg", role: "dialog", "aria-label": "{c.name}",
                                        onkeydown: move |e: KeyboardEvent| {
                                            if e.key() == Key::Escape {
                                                e.stop_propagation();
                                                if msg_open() { msg_open.set(false) } else {
                                                    opened.set(None);
                                                    focus(handles.peek().get(i).cloned().flatten());
                                                }
                                            }
                                        },
                                        div { class: "oxcw-dh",
                                            b { "{c.name}" }
                                            button { class: "oxcw-x", r#type: "button", "aria-label": "Close",
                                                onmounted: move |e| { close_btn.set(Some(e.data())); focus(Some(e.data())) },
                                                onclick: move |_| { opened.set(None); focus(handles.peek().get(i).cloned().flatten()) },
                                                Icon { name: "x", size: 12.0, stroke: 2.0 }
                                            }
                                        }
                                        if !c.task.is_empty() { p { class: "oxcw-task", title: "{c.task}", "{c.task}" } }
                                        if !c.model.is_empty() {
                                            span { class: "oxcw-model",
                                                BrandIcon { name: c.model_brand.clone(), size: 12.0, coloured: true, fallback: true }
                                                "{c.model}"
                                            }
                                        }
                                        div { class: "oxcw-use",
                                            span { i { if c.state == CrewState::Active { "Running" } else { "Ran" } } b { "{crew_duration(ran)}" } }
                                            if !crew_cost(c.cost).is_empty() { span { i { "Cost" } b { "{crew_cost(c.cost)}" } } }
                                            span { i { "Tokens" } b { "{short_tokens(c.tokens())}" } }
                                        }
                                        if !c.tools.is_empty() {
                                            div { class: "oxcw-tps",
                                                for (b, n) in c.tools.iter().cloned() {
                                                    span { key: "{n}", class: "oxcw-tp", BrandIcon { name: b, size: 12.0, coloured: true, fallback: true } "{n}" }
                                                }
                                            }
                                        }
                                        if let Some((from, text)) = c.last.clone() {
                                            div { class: "oxcw-last", "data-in": "dlg", title: "{from}: {text}",
                                                span { class: "oxcw-arr", "↳" } b { "{from}" } span { "{text}" }
                                            }
                                        }
                                        if msg_open() {
                                            input { class: "oxcw-in", r#type: "text", placeholder: "Message {c.name}…", "aria-label": "Message {c.name}",
                                                value: "{draft}",
                                                onmounted: move |e| focus(Some(e.data())),
                                                oninput: move |e| draft.set(e.value()),
                                                onkeydown: {
                                                    let id = c.id.clone();
                                                    let name = c.name.clone();
                                                    let p2 = p.clone();
                                                    move |e: KeyboardEvent| if e.key() == Key::Enter {
                                                        e.prevent_default();
                                                        let t = draft().trim().to_string();
                                                        if t.is_empty() { return; }
                                                        lasts.write().insert(format!("{p2}/{id}"), ("You".into(), t.clone()));
                                                        said.set((el, format!("Sent to {name}: {t}")));
                                                        on_steer.call((id.clone(), "message".into(), t));
                                                        draft.set(String::new());
                                                        msg_open.set(false);
                                                    }
                                                },
                                            }
                                        }
                                        div { class: "oxcw-steer",
                                            for a in c.state.steering().iter().copied() {
                                                button { key: "{a}", class: "oxcw-sb", r#type: "button", "data-a": "{a}",
                                                    "data-primary": "{a == \"approve\" || a == \"resume\" || a == \"retry\"}",
                                                    onclick: {
                                                        let id = c.id.clone();
                                                        let name = c.name.clone();
                                                        let before = c.activity.clone();
                                                        let p2 = p.clone();
                                                        move |_| {
                                                            if a == "message" { msg_open.set(!msg_open()); return; }
                                                            if let Some(r) = steer_result(a, &before) {
                                                                overrides.write().insert(format!("{p2}/{id}"), r);
                                                            }
                                                            said.set((el, steer_said(a, &name)));
                                                            on_steer.call((id.clone(), a.to_string(), String::new()));
                                                        }
                                                    },
                                                    "{steer_label(a)}"
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxcw-sr", role: "status", "aria-live": "polite", "{live}" }
        }
    }
}
