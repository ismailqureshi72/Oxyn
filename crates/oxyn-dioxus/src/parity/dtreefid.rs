//! affectUI fidelity pass, family B part 8 (plan #32): the exact **Decision Tree**
//! ([`DecisionTree`]). Behaviour and look only; the model is `oxyn_core::parity::dtreefid`.
//! Styles: `dtreefid.css` (classes `oxdt-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::dtreefid::{
        DtDecision, DtKey, DtOption, DtSeg, DtState, dt_announce, dt_demo, dt_flags, dt_key,
        dt_next_flag, dt_pick, dt_reply, dt_waiting,
    },
};

use crate::brand::BrandIcon;

/// How long a settled decision stays before the next one grows, ms.
const LEAVE_MS: u64 = 1600;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn path(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

/// Monochrome glyphs for the non-brand icons; brand ids draw their coloured mark.
fn glyph(id: &str, size: u32) -> Element {
    let d = match id {
        "tree" => "M3 3.5h3.5M6.5 3.5v9M6.5 8H10M6.5 12.5H10M10 3.5h3M10 8h3M10 12.5h3",
        "burst" => {
            "M8 1.8v2.4M8 11.8v2.4M1.8 8h2.4M11.8 8h2.4M3.6 3.6l1.7 1.7M10.7 10.7l1.7 1.7M3.6 12.4l1.7-1.7M10.7 5.3l1.7-1.7"
        }
        "coin" => {
            "M8 2.2a5.8 5.8 0 1 0 0 11.6A5.8 5.8 0 0 0 8 2.2zM9.8 6.1C9.5 5.5 8.8 5.2 8 5.2c-1 0-1.8.5-1.8 1.3 0 1.8 3.7.9 3.7 2.8 0 .8-.8 1.4-1.9 1.4-.9 0-1.6-.4-1.9-1M8 4.2v1M8 10.7v1"
        }
        "cloud" => "M4.6 12.5h7a2.6 2.6 0 0 0 .3-5.2 4 4 0 0 0-7.7-.9A3 3 0 0 0 4.6 12.5z",
        "revert" => "M3.5 6h6.2a3 3 0 0 1 0 6H6M3.5 6 6 3.5M3.5 6 6 8.5",
        "flag" => "M3.5 14V2.5M3.5 3h8l-1.6 2.8L11.5 8.6h-8",
        "pause" => "M5.5 3.5v9M10.5 3.5v9",
        "limit" => "M8 2.5a5.5 5.5 0 1 0 0 11 5.5 5.5 0 0 0 0-11zM4.1 4.1l7.8 7.8",
        "page" => {
            "M8 2.5a4 4 0 0 0-4 4v3l-1.2 2h10.4L12 9.5v-3a4 4 0 0 0-4-4zM6.5 13.5a1.6 1.6 0 0 0 3 0"
        }
        "mail" => "M2.5 4h11v8h-11zM2.5 4.5 8 9l5.5-4.5",
        "status" => {
            "M2.5 6.2a8 8 0 0 1 11 0M4.5 8.4a5 5 0 0 1 7 0M6.6 10.6a2 2 0 0 1 2.8 0M8 13h.01"
        }
        "check" => "M3.5 8.5l3 3 6-7",
        "warn" => "M8 2.5 14 13H2zM8 6.5v3M8 11.2h.01",
        "arrow" => "M3 8h10M9.5 4.5 13 8l-3.5 3.5",
        _ => "",
    };
    if d.is_empty() {
        rsx! { BrandIcon { name: id.to_string(), size: size as f32, coloured: true } }
    } else {
        path(d, size)
    }
}

fn icon_cls(id: &str) -> &'static str {
    match id {
        "burst" => "oxdt-ag oxdt-ag-burst",
        "coin" => "oxdt-ag oxdt-ag-coin",
        "cloud" => "oxdt-ag oxdt-ag-cloud",
        "status" => "oxdt-ic oxdt-ic-status",
        _ => "oxdt-ic",
    }
}

/// One consequence line; flagged runs are focusable and carry their note.
fn effect_line(
    line: &[DtSeg],
    key: String,
    flags: &[(usize, Option<usize>, usize, usize)],
    at: (usize, Option<usize>, usize),
    open_flag: Option<usize>,
) -> Element {
    rsx! {
        span { class: "oxdt-eff",
            for (si, s) in line.iter().enumerate() {
                if s.flagged() {
                    {
                        let fi = flags.iter().position(|f| *f == (at.0, at.1, at.2, si));
                        let open = fi.is_some() && fi == open_flag;
                        let tid = format!("{key}-n{si}");
                        rsx! {
                            span { class: "oxdt-flag", tabindex: "0", "aria-describedby": "{tid}", "data-open": open,
                                "{s.text}"
                                span { class: "oxdt-note", id: "{tid}", role: "tooltip",
                                    span { class: "oxdt-note-h", "Agent's note" }
                                    span { class: "oxdt-note-t", "{s.note}" }
                                    if !s.source.is_empty() {
                                        span { class: "oxdt-note-s", {glyph("warn", 11)} "{s.source}" }
                                    }
                                }
                            }
                        }
                    }
                } else {
                    "{s.text}"
                }
            }
        }
    }
}

/// Decision Tree (affectUI): an agent stuck on a decision lays its options out as a tree – the
/// question, two to four branches with what each leads to, risky consequences flagged, its own pick
/// marked, a one-level follow-up per branch – and you pick a path (click or 1–4) or write your own
/// instruction in "Other…". Settled decisions leave; the next one grows in.
#[component]
pub fn DecisionTree(
    /// Everything waiting for a call (the demo queue when `None`).
    #[props(default)]
    decisions: Option<Vec<DtDecision>>,
    /// Offer the "Other…" branch.
    #[props(default = true)]
    allow_other: bool,
    /// Heading of the bar.
    #[props(default = "Decisions".to_string())]
    title: String,
    /// Demo: the tree plays itself (hover, pick, follow-up, next) until touched.
    #[props(default)]
    auto: bool,
    /// Start on this decision.
    #[props(default)]
    start: usize,
    /// Start with this branch of the first decision picked.
    #[props(default)]
    branch: Option<usize>,
    /// Start with this follow-up option picked too (decision stays, no fade).
    #[props(default)]
    then: Option<usize>,
    /// Start with this branch pointed at.
    #[props(default)]
    hover: Option<usize>,
    /// Start with "Other…" open.
    #[props(default)]
    other_open: bool,
    /// Start with this instruction already sent.
    #[props(default)]
    sent: String,
    /// Start with the n-th flag's note open.
    #[props(default)]
    flag: Option<usize>,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A path was picked: (decision id, branch, follow-up pick).
    #[props(default)]
    on_choose: EventHandler<(String, usize, Option<usize>)>,
    /// An instruction was sent: (decision id, text).
    #[props(default)]
    on_other: EventHandler<(String, String)>,
) -> Element {
    let t0 = use_signal(move || now);
    let mut touched = use_signal(|| false);
    let mut cur = use_signal(move || start);
    let mut hov = use_signal(move || hover);
    let init = decisions.clone();
    let mut states = use_signal(move || {
        let mut m: Vec<(String, DtState, Option<ClockMs>)> = vec![];
        if branch.is_some() || !sent.is_empty() {
            let id = init
                .as_ref()
                .map_or_else(dt_demo, Clone::clone)
                .get(start)
                .map(|d| d.id.clone())
                .unwrap_or_default();
            m.push((
                id,
                DtState {
                    branch,
                    then,
                    other: (!sent.is_empty()).then(|| sent.clone()),
                },
                None,
            ));
        }
        m
    });
    let mut other = use_signal(move || other_open);
    let mut draft = use_signal(String::new);
    let mut note = use_signal(move || flag);
    let mut live_msg = use_signal(String::new);

    let all = decisions.clone().unwrap_or_else(dt_demo);
    let el = now.since(t0()).as_millis() as u64;
    let scripted = auto && !touched();

    // Decisions settled more than LEAVE_MS ago have left the queue.
    let st = states();
    let gone = |id: &str| {
        st.iter().any(|(i, _, at)| {
            i == id && at.is_some_and(|t| now.since(t).as_millis() as u64 > LEAVE_MS)
        })
    };
    let queue: Vec<DtDecision> = all.iter().filter(|d| !gone(&d.id)).cloned().collect();
    let n = queue.len();

    // The scripted demo: 7.2 s per decision – settle, point at the pick, pick it, answer its
    // follow-up with the agent's pick, fade, next.
    let (ci, s, h, fading) = if scripted && n > 0 {
        let per = 7200u64;
        let ci = ((el / per) as usize + start) % n;
        let t = el % per;
        let d = &queue[ci];
        let p = dt_pick(&d.options).unwrap_or(0);
        let tp = d.options[p]
            .next
            .as_ref()
            .and_then(|f| dt_pick(&f.options))
            .or(Some(0));
        let s = DtState {
            branch: (t >= 3300).then_some(p),
            then: if t >= 4800 && d.options[p].next.is_some() {
                tp
            } else {
                None
            },
            other: None,
        };
        let h = (2200..3300).contains(&t).then_some(p);
        (ci, s, h, t >= 6400)
    } else {
        let ci = cur().min(n.saturating_sub(1));
        let (s, at) = queue
            .get(ci)
            .and_then(|d| st.iter().find(|(i, _, _)| *i == d.id))
            .map(|(_, s, at)| (s.clone(), *at))
            .unwrap_or_default();
        let fading = at.is_some_and(|t| now.since(t).as_millis() as u64 > LEAVE_MS / 2);
        (ci, s, hov(), fading)
    };

    let Some(d) = queue.get(ci).cloned() else {
        return rsx! {
            section { class: "oxdt", "data-style": style_attr(&style), "data-empty": "true", "aria-label": "{title}",
                header { class: "oxdt-bar",
                    span { class: "oxdt-logo", {glyph("tree", 18)} }
                    h2 { class: "oxdt-ttl", "{title}" }
                    span { class: "oxdt-wait", "{all.len()} of {all.len()} decided" }
                }
                div { class: "oxdt-empty", role: "status",
                    span { class: "oxdt-empty-ic", {glyph("check", 20)} }
                    b { "All decided" }
                    span { "Anything else your agents need from you will land here." }
                }
                div { class: "oxdt-sr", role: "status", "aria-live": "polite", "{live_msg}" }
            }
        };
    };
    let flags = dt_flags(&d);
    let done = s.done(&d);
    let active_n = s.active_len(&d);
    let first_level = s.branch.is_none();
    let did = d.id.clone();

    // Picks and instructions.
    let choose = {
        let d = d.clone();
        let all_len = n;
        move |i: usize| {
            touched.set(true);
            let mut ss = states();
            let pos = ss.iter().position(|(x, _, _)| *x == d.id);
            let mut cur_s = pos.map(|p| ss[p].1.clone()).unwrap_or_default();
            if !cur_s.pick(&d, i) {
                return;
            }
            other.set(false);
            note.set(None);
            let fin = cur_s.done(&d);
            live_msg.set(dt_announce(&d, &cur_s));
            if fin {
                on_choose.call((d.id.clone(), cur_s.branch.unwrap_or(0), cur_s.then));
            }
            let entry = (d.id.clone(), cur_s, fin.then_some(now));
            match pos {
                Some(p) => ss[p] = entry,
                None => ss.push(entry),
            }
            states.set(ss);
            let _ = all_len;
        }
    };
    let send = {
        let d = d.clone();
        move |_: ()| {
            let text = draft().trim().to_string();
            if text.is_empty() {
                return;
            }
            touched.set(true);
            let mut ss = states();
            ss.retain(|(x, _, _)| *x != d.id);
            let s2 = DtState {
                other: Some(text.clone()),
                ..DtState::default()
            };
            live_msg.set(format!("{} {}", dt_announce(&d, &s2), dt_reply(&d)));
            ss.push((d.id.clone(), s2, Some(now)));
            states.set(ss);
            other.set(false);
            draft.set(String::new());
            on_other.call((d.id.clone(), text));
        }
    };
    let mut go = move |delta: i64| {
        touched.set(true);
        if n == 0 {
            return;
        }
        let c = cur().min(n - 1) as i64;
        cur.set(((c + delta).rem_euclid(n as i64)) as usize);
        hov.set(None);
        other.set(false);
        note.set(None);
    };

    let lit = s.branch.or(h);
    let pick_i = dt_pick(&d.options);
    let state_of = |i: usize| -> &'static str {
        match (s.branch, lit) {
            (Some(b), _) if b == i => "picked",
            (Some(_), _) => "gone",
            (None, Some(l)) if l == i => "lit",
            (None, Some(_)) => "dim",
            _ if s.other.is_some() || other() => "dim",
            _ => "idle",
        }
    };
    let outcome = if let Some(t) = &s.other {
        Some((false, t.clone()))
    } else if done {
        Some((true, String::new()))
    } else {
        None
    };
    let key = did.clone();
    let top_key = if allow_other { active_n + 1 } else { active_n };

    rsx! {
        section { class: "oxdt", "data-style": style_attr(&style), "data-fading": fading, "data-level": if first_level { "1" } else { "2" },
            "aria-label": "{title}",
            onkeydown: {
                let mut choose = choose.clone();
                let d = d.clone();
                move |e: KeyboardEvent| {
                    let k = e.key().to_string();
                    if k == "Enter" || k == " " { return; }
                    let Some(a) = dt_key(&k, active_n, allow_other && first_level && !done) else { return };
                    match a {
                        DtKey::Pick(i) => { e.prevent_default(); choose(i); }
                        DtKey::Other => { e.prevent_default(); touched.set(true); other.set(true); }
                        DtKey::Next => go(1),
                        DtKey::Prev => go(-1),
                        DtKey::Flag => { touched.set(true); note.set(dt_next_flag(&d, note())); }
                        DtKey::Close => { other.set(false); note.set(None); }
                    }
                }
            },
            header { class: "oxdt-bar",
                span { class: "oxdt-logo", {glyph("tree", 18)} }
                h2 { class: "oxdt-ttl", "{title}" }
                span { class: "oxdt-wait", "{dt_waiting(n)}" }
                span { class: "oxdt-sp" }
                if n > 1 {
                    span { class: "oxdt-dots",
                        for i in 0..n {
                            button { class: "oxdt-dot", r#type: "button", "aria-current": (i == ci).then_some("true"),
                                "aria-label": "Decision {i + 1} of {n}", onclick: move |_| { touched.set(true); cur.set(i); hov.set(None); other.set(false); note.set(None); } }
                        }
                    }
                }
                span { class: "oxdt-count", "{ci + 1} of {n}" }
            }
            div { class: "oxdt-tree", key: "{key}", "data-k": "{key}",
                div { class: "oxdt-q", "data-lit": lit.is_some() || done,
                    if let Some(tag) = d.risk.tag() {
                        span { class: "oxdt-risk", "data-risk": d.risk.id(), "{tag}" }
                    }
                    div { class: "oxdt-who",
                        span { class: "oxdt-agic {icon_cls(&d.icon)}", {glyph(&d.icon, 14)} }
                        span { class: "oxdt-whot",
                            b { "{d.agent}" }
                            if !d.model.is_empty() { small { crate::brand::MarkFor { name: d.model.clone(), size: 11.0 } "{d.model}" } }
                        }
                    }
                    h3 { class: "oxdt-qh", "{d.question}" }
                    if !d.context.is_empty() { p { class: "oxdt-ctx", "{d.context}" } }
                }
                ol { class: "oxdt-br", "aria-label": "Branches",
                    for (i, o) in d.options.iter().enumerate() {
                        {
                            let o: DtOption = o.clone();
                            let stt = state_of(i);
                            let mut choose = choose.clone();
                            let rowkey = format!("{did}-{i}");
                            let awake = s.branch == Some(i) && !done;
                            let picked_then = if s.branch == Some(i) { s.then } else { None };
                            let show_fu = lit == Some(i) || (lit.is_none() && o.pick) || s.branch == Some(i);
                            let lbl = if o.pick { format!("{}, the agent's pick", o.label) } else { o.label.clone() };
                            rsx! {
                                li { class: "oxdt-row", "data-st": stt, "data-pick": o.pick, key: "{rowkey}", "data-k": "{rowkey}",
                                    style: "--i:{i}",
                                    onmouseenter: move |_| if !scripted { hov.set(Some(i)) },
                                    onmouseleave: move |_| if !scripted { hov.set(None) },
                                    button { class: "oxdt-opt", r#type: "button", "aria-label": "{lbl}", "aria-keyshortcuts": "{i + 1}",
                                        "aria-pressed": s.branch == Some(i),
                                        disabled: s.branch.is_some() && s.branch != Some(i) || s.other.is_some(),
                                        onclick: move |_| choose(i),
                                        span { class: "oxdt-num", "{i + 1}" }
                                        if !o.icon.is_empty() { span { class: "{icon_cls(&o.icon)}", {glyph(&o.icon, 14)} } }
                                        span { class: "oxdt-lab",
                                            span { class: "oxdt-lt", "{o.label}" }
                                            if o.pick { span { class: "oxdt-pk", "Agent's pick" } }
                                        }
                                    }
                                    if !o.effects.is_empty() || o.next.is_some() || !o.apps.is_empty() {
                                        div { class: "oxdt-cons",
                                            if !o.effects.is_empty() || !o.apps.is_empty() {
                                                div { class: "oxdt-effs",
                                                    for (li, line) in o.effects.iter().enumerate() {
                                                        {effect_line(line, format!("{rowkey}-{li}"), &flags, (i, None, li), note())}
                                                    }
                                                    if !o.apps.is_empty() && o.next.is_none() {
                                                        span { class: "oxdt-apps", for a in o.apps.iter() { span { class: "{icon_cls(a)}", {glyph(a, 13)} } } }
                                                    }
                                                }
                                            }
                                            if let Some(f) = o.next.clone().filter(|_| show_fu) {
                                                div { class: "oxdt-fu", "data-awake": awake || picked_then.is_some(),
                                                    span { class: "oxdt-fq", "{f.question}" }
                                                    for (j, fo) in f.options.iter().enumerate() {
                                                        {
                                                            let fo = fo.clone();
                                                            let mut choose = choose.clone();
                                                            let fst = match picked_then { Some(t) if t == j => "picked", Some(_) => "gone", None => "idle" };
                                                            let fl = if fo.pick { format!("{}, the agent's pick", fo.label) } else { fo.label.clone() };
                                                            rsx! {
                                                                button { class: "oxdt-fo", r#type: "button", "data-st": fst, "data-pick": fo.pick,
                                                                    disabled: !awake, "aria-label": "{fl}", "aria-keyshortcuts": "{j + 1}",
                                                                    onclick: move |_| choose(j),
                                                                    span { class: "oxdt-fot",
                                                                        span { class: "oxdt-lt", "{fo.label}" }
                                                                        if fo.pick { span { class: "oxdt-pk", "Agent's pick" } }
                                                                    }
                                                                    span { class: "oxdt-fob",
                                                                        for (li, line) in fo.effects.iter().enumerate() {
                                                                            {effect_line(line, format!("{rowkey}-f{j}-{li}"), &flags, (i, Some(j), li), note())}
                                                                        }
                                                                        if !fo.apps.is_empty() {
                                                                            span { class: "oxdt-apps", for a in fo.apps.iter() { span { class: "{icon_cls(a)}", {glyph(a, 13)} } } }
                                                                        }
                                                                    }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                            if s.branch == Some(i) && done && s.other.is_none() {
                                                span { class: "oxdt-out", {glyph("check", 13)} b { "Approved" } span { "· running" } }
                                            }
                                        }
                                    }
                                    if s.branch == Some(i) && done && o.effects.is_empty() && o.next.is_none() && o.apps.is_empty() {
                                        span { class: "oxdt-out", {glyph("check", 13)} b { "Approved" } span { "· running" } }
                                    }
                                }
                            }
                        }
                    }
                    if allow_other {
                        li { class: "oxdt-row oxdt-orow", "data-st": if s.other.is_some() { "picked" } else if s.branch.is_some() { "gone" } else if lit.is_some() { "dim" } else { "idle" },
                            style: "--i:{d.options.len()}",
                            if let Some(t) = s.other.clone() {
                                div { class: "oxdt-sent", "{t}" }
                                span { class: "oxdt-reply", "Sent to the {d.agent.to_lowercase()}" }
                            } else if other() {
                                textarea { class: "oxdt-ta", rows: "2", value: "{draft}", placeholder: "Write the agent a different instruction…",
                                    "aria-label": "Your instruction for the {d.agent.to_lowercase()}",
                                    onmounted: move |e: MountedEvent| { spawn(async move { let _ = e.data().set_focus(true).await; }); },
                                    oninput: move |e| draft.set(e.value()),
                                    onkeydown: {
                                        let mut send = send.clone();
                                        move |e: KeyboardEvent| {
                                            e.stop_propagation();
                                            match e.key() {
                                                Key::Enter if !e.modifiers().shift() => { e.prevent_default(); send(()); }
                                                Key::Escape => other.set(false),
                                                _ => {}
                                            }
                                        }
                                    }
                                }
                            } else {
                                button { class: "oxdt-opt oxdt-other", r#type: "button", "aria-keyshortcuts": "{d.options.len() + 1}",
                                    disabled: s.branch.is_some(),
                                    onclick: move |_| { touched.set(true); other.set(true); },
                                    span { class: "oxdt-num", "{d.options.len() + 1}" }
                                    span { class: "oxdt-lt", "Other…" }
                                }
                            }
                        }
                    }
                }
            }
            footer { class: "oxdt-foot", "aria-hidden": "true",
                if let Some((ok, _)) = outcome.clone() {
                    if ok { span { "Decided · next one in a moment" } } else { span { "{dt_reply(&d)}" } }
                } else if first_level {
                    span { "Choose a path" }
                    kbd { "1" } span { "–" } kbd { "{top_key}" }
                    if n > 1 { span { "· next" } kbd { {glyph("arrow", 10)} } }
                } else {
                    span { "Then" } kbd { "1" } span { "–" } kbd { "{active_n}" }
                }
                if !flags.is_empty() && outcome.is_none() { span { "· note" } kbd { "F" } }
            }
            div { class: "oxdt-sr", role: "status", "aria-live": "polite", "{live_msg}" }
            if let Some(p) = pick_i { span { class: "oxdt-sr", "data-pick": "{p}" } }
        }
    }
}
