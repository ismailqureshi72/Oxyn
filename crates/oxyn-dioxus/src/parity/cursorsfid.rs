//! affectUI fidelity pass, overnight batch, family B part 3 (plan #32): the exact **Agent
//! Cursors** ([`AgentCursors`]). Behaviour and look only; the model is
//! `oxyn_core::parity::cursorsfid`. Styles: `cursorsfid.css` (classes `oxcu-*`), in
//! `PRIMITIVES_CSS`. Follows the nearest `data-cstyle` (Clean / Classic) unless `style` is set;
//! time comes in as `now`.
//!
//! Pointers are anchored inline – a zero-width span at their character – so they stay on their
//! letters while the text wraps, at any width, font size or zoom.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::cursorsfid::{
        AgentCursor, BlockKind, CursorScene, DocBlock, DocComment, DocSuggestion, ImagePoint,
        accept_suggestion, cursor_tabs, cursors_at, cursors_loop_ms, face_label, format_cost,
        presence_label, segments, suggestion_label,
    },
};

use crate::{brand::BrandIcon, composer::icons::Icon};

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

/// Everything a run of text needs to draw itself.
#[derive(Clone, PartialEq)]
struct Ctx {
    scene: CursorScene,
    following: Option<usize>,
    lap: u64,
}

fn agent_ix(scene: &CursorScene, id: &str) -> Option<usize> {
    scene.agents.iter().position(|a| a.id == id)
}

/// The soft rounded arrow, tip at (0, 0).
fn pointer(ink: &str) -> Element {
    rsx! {
        svg { class: "oxcu-ptr", width: "18", height: "20", view_box: "0 0 18 20", "aria-hidden": "true",
            path { d: "M2.2 1.6C2.2 1 2.9.7 3.4 1.1l11.4 9.3c.5.4.2 1.2-.4 1.2l-4.9.3c-.3 0-.6.2-.8.4l-2.9 4.1c-.4.5-1.2.3-1.3-.3L2.2 1.6z",
                fill: "{ink}", stroke: "var(--oxcu-ptr-edge)", stroke_width: "1.3", stroke_linejoin: "round" }
        }
    }
}

/// A pointer with its flag (and caret), anchored where it is drawn.
fn cursor_mark(ctx: &Ctx, i: usize, caret: bool) -> Element {
    let a = &ctx.scene.agents[i];
    let c = ctx.scene.cursors[i].clone();
    let doing = c.as_ref().map(|c| c.doing.clone()).unwrap_or_default();
    let typing = c.as_ref().map(|c| c.typing).unwrap_or(false);
    let fade = ctx.following.is_some_and(|f| f != i);
    let ink = if a.color.is_empty() {
        "var(--oxcu-ink)".to_string()
    } else {
        a.color.clone()
    };
    let cost = format_cost(a.cost);
    let name = a.name.clone();
    let id = a.id.clone();
    rsx! {
        span { class: "oxcu-anc", "aria-hidden": "true", "data-fade": "{fade}", "data-agent": "{id}", "data-typing": "{typing}",
            if caret { i { class: "oxcu-caret", style: "background:{ink}" } }
            span { class: "oxcu-cur",
                {pointer(&ink)}
                span { class: "oxcu-flag",
                    span { class: "oxcu-fl1", b { "{name}" } span { class: "oxcu-fc", "{cost}" } }
                    if !doing.is_empty() { span { class: "oxcu-fl2", key: "{doing}", "data-d": "{doing}", "{doing}" } }
                }
            }
        }
    }
}

/// The window of an open suggestion.
fn sugg_window(
    ctx: &Ctx,
    s: &DocSuggestion,
    mut open: Signal<Option<String>>,
    mut decided: Signal<Vec<(u64, String, bool)>>,
    on_accept: EventHandler<DocSuggestion>,
    on_reject: EventHandler<DocSuggestion>,
) -> Element {
    let a = agent_ix(&ctx.scene, &s.agent).map(|i| ctx.scene.agents[i].clone());
    let (name, brand) = a
        .map(|a| (a.name, a.brand))
        .unwrap_or_else(|| (s.agent.clone(), String::new()));
    let lap = ctx.lap;
    let (s1, s2) = (s.clone(), s.clone());
    let what = if s.remove.is_empty() {
        "suggests adding"
    } else if s.insert.is_empty() {
        "suggests removing"
    } else {
        "suggests a rewrite"
    };
    rsx! {
        span { class: "oxcu-swin", role: "dialog", "aria-label": "Suggestion from {name}", contenteditable: "false",
            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(None) },
            span { class: "oxcu-swh",
                span { class: "oxcu-swm", BrandIcon { name: brand.clone(), size: 12.0, coloured: true, fallback: true } }
                b { "{name}" }
                span { class: "oxcu-mute", "{what}" }
            }
            if !s.note.is_empty() { span { class: "oxcu-swn", "{s.note}" } }
            span { class: "oxcu-swb",
                button { class: "oxcu-btn", "data-k": "reject", r#type: "button",
                    onclick: move |_| { decided.write().push((lap, s1.id.clone(), false)); open.set(None); on_reject.call(s1.clone()) },
                    "Reject" }
                button { class: "oxcu-btn", "data-k": "accept", r#type: "button",
                    onclick: move |_| { decided.write().push((lap, s2.id.clone(), true)); open.set(None); on_accept.call(s2.clone()) },
                    "Accept" }
            }
        }
    }
}

/// Draws one block's text with its selections, suggestions and carets.
#[allow(clippy::too_many_arguments)]
fn runs(
    ctx: &Ctx,
    block: &str,
    text: &str,
    open: Signal<Option<String>>,
    decided: Signal<Vec<(u64, String, bool)>>,
    on_accept: EventHandler<DocSuggestion>,
    on_reject: EventHandler<DocSuggestion>,
) -> Element {
    let segs = segments(block, text, &ctx.scene.cursors, &ctx.scene.suggestions);
    // Carets don't count: a suggestion ends at its last run of text.
    let sug_of: Vec<Option<usize>> = segs
        .iter()
        .map(|g| g.del.or(g.ins).filter(|_| g.caret.is_none()))
        .collect();
    rsx! {
        for (k, g) in segs.into_iter().enumerate() {
            {
                let sug = g.del.or(g.ins);
                let s = sug.map(|i| ctx.scene.suggestions[i].clone());
                // The last run of a suggestion carries its window.
                let last_of = sug.filter(|_| g.caret.is_none() && !sug_of[k + 1..].contains(&sug));
                let fade = |ai: Option<usize>| ai.is_some_and(|ai| ctx.following.is_some_and(|f| f != ai));
                let sel_fade = fade(g.sel);
                let sug_fade = fade(s.as_ref().and_then(|s| agent_ix(&ctx.scene, &s.agent)));
                let k_s = format!("{block}-{k}");
                if let Some(ai) = g.caret {
                    let typing = ctx.scene.cursors[ai].as_ref().map(|c| c.typing).unwrap_or(false);
                    rsx! { span { key: "{k_s}", "data-k": "{k_s}", {cursor_mark(ctx, ai, typing)} } }
                } else if let Some(s) = s {
                    let aname = agent_ix(&ctx.scene, &s.agent).map(|i| ctx.scene.agents[i].name.clone()).unwrap_or_default();
                    let is_del = g.del.is_some();
                    // One focusable stop per suggestion: its removal, or its insert when it only adds.
                    let stop = (is_del && !s.remove.is_empty()) || (!is_del && s.remove.is_empty() && !g.text.is_empty());
                    let label = suggestion_label(&aname, &s, is_del);
                    let sid = s.id.clone();
                    let (sid1, sid2) = (sid.clone(), sid.clone());
                    let mut open = open;
                    let is_open = open().as_deref() == Some(sid.as_str()) && last_of.is_some();
                    let cls = if is_del { "oxcu-del" } else { "oxcu-ins" };
                    let ink = ctx.scene.agents.iter().find(|a| a.id == s.agent).map(|a| a.color.clone()).unwrap_or_default();
                    let st = if ink.is_empty() { String::new() } else { format!("--oxcu-sc:{ink}") };
                    rsx! {
                        span { key: "{k_s}", "data-k": "{k_s}", class: "oxcu-sg", "data-fade": "{sug_fade}",
                            if stop {
                                span { class: "{cls}", style: "{st}", tabindex: "0", role: "button", "aria-label": "{label}", "aria-haspopup": "dialog",
                                    "data-open": "{open().as_deref() == Some(sid.as_str())}",
                                    onclick: move |_| open.set(if open().as_deref() == Some(sid1.as_str()) { None } else { Some(sid1.clone()) }),
                                    onfocus: move |_| open.set(Some(sid2.clone())),
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(None) },
                                    "{g.text}"
                                }
                            } else {
                                span { class: "{cls}", style: "{st}", "{g.text}" }
                            }
                            if is_open { {sugg_window(ctx, &s, open, decided, on_accept, on_reject)} }
                        }
                    }
                } else if g.sel.is_some() {
                    rsx! { span { key: "{k_s}", "data-k": "{k_s}", class: "oxcu-sel", "data-fade": "{sel_fade}", "{g.text}" } }
                } else {
                    rsx! { span { key: "{k_s}", "data-k": "{k_s}", "{g.text}" } }
                }
            }
        }
    }
}

/// A comment pin (or its open window) beside a block.
#[component]
fn CommentPin(
    ctx: Ctx,
    c: DocComment,
    resolved: Signal<Vec<(u64, String)>>,
    on_resolve: EventHandler<DocComment>,
    on_reply: EventHandler<(DocComment, String)>,
) -> Element {
    let mut resolved = resolved;
    let a = agent_ix(&ctx.scene, &c.agent).map(|i| ctx.scene.agents[i].clone());
    let (name, brand) = a
        .map(|a| (a.name, a.brand))
        .unwrap_or_else(|| (c.agent.clone(), String::new()));
    let lap = ctx.lap;
    let (c1, c2) = (c.clone(), c.clone());
    let mut draft = use_signal(String::new);
    let left = format!("calc({:.3} * 100% - 26px)", c.x.clamp(0.0, 1.0));
    let fade =
        agent_ix(&ctx.scene, &c.agent).is_some_and(|ai| ctx.following.is_some_and(|f| f != ai));
    rsx! {
        div { class: "oxcu-pin", "data-open": "{c.open}", "data-fade": "{fade}", style: "left:{left};top:{c.y - 4.0}px",
            button { class: "oxcu-pinb", r#type: "button", "aria-label": "Comment from {name}: {c.text}",
                BrandIcon { name: brand.clone(), size: 13.0, coloured: true, fallback: true }
            }
            div { class: "oxcu-cwin",
                div { class: "oxcu-cwh",
                    span { class: "oxcu-swm", BrandIcon { name: brand.clone(), size: 12.0, coloured: true, fallback: true } }
                    b { "{name}" }
                    span { class: "oxcu-sp" }
                    span { class: "oxcu-cnow", "now" }
                }
                div { class: "oxcu-ctx", "{c.text}" }
                div { class: "oxcu-crow",
                    input { class: "oxcu-rep", placeholder: "Reply", "aria-label": "Reply to {name}", value: "{draft}",
                        oninput: move |e| draft.set(e.value()),
                        onkeydown: move |e: KeyboardEvent| if e.key() == Key::Enter && !draft().trim().is_empty() {
                            on_reply.call((c1.clone(), draft().trim().to_string())); draft.set(String::new());
                        } }
                    button { class: "oxcu-res", r#type: "button", "aria-label": "Resolve",
                        onclick: move |_| { resolved.write().push((lap, c2.id.clone())); on_resolve.call(c2.clone()) },
                        Icon { name: "check", size: 13.0, stroke: 2.2 } }
                }
            }
        }
    }
}

/// The image block: grey with a shimmer while it loads, then a soft render; held, it shows
/// handles at its corners and its size.
fn image_block(ctx: &Ctx, b: &DocBlock) -> Element {
    let holders: Vec<(usize, AgentCursor)> = ctx
        .scene
        .cursors
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.clone().filter(|c| c.block == b.id).map(|c| (i, c)))
        .collect();
    let held = holders.iter().any(|(_, c)| c.select.is_some());
    let w = (b.width.clamp(0.05, 1.0) * 100.0).round();
    let side = if b.float == "right" {
        "right"
    } else {
        &b.float
    };
    let px_w = (b.width * 612.0).round() as u32;
    let px_h = (px_w as f32 / b.ratio.max(0.1)).round() as u32;
    let label = if b.loading {
        "Image loading".to_string()
    } else {
        b.text.clone()
    };
    rsx! {
        figure { class: "oxcu-img", "data-float": "{side}", "data-loading": "{b.loading}", "data-held": "{held}",
            style: "width:{w}%;aspect-ratio:{b.ratio}", role: "img", "aria-label": "{label}",
            span { class: "oxcu-art" }
            if held {
                span { class: "oxcu-hdl", "data-c": "tl" } span { class: "oxcu-hdl", "data-c": "tr" }
                span { class: "oxcu-hdl", "data-c": "bl" } span { class: "oxcu-hdl", "data-c": "br" }
                span { class: "oxcu-size", "{px_w} × {px_h}" }
            }
            for (i, c) in holders.iter() {
                {
                    let corner = c.point == Some(ImagePoint::Corner);
                    let kk = format!("{}-{i}", b.id);
                    rsx! { span { key: "{kk}", "data-k": "{kk}", class: "oxcu-ipt", "data-corner": "{corner}", {cursor_mark(ctx, *i, false)} } }
                }
            }
        }
    }
}

/// Agent Cursors (affectUI): agents working in a document like teammates in a multiplayer
/// editor – live pointers with names, suggestions typed letter by letter, images they resize,
/// code they fix and comments beside the page. Each change stays a suggestion (struck through /
/// underlined) until you accept it from its window or with "Accept all"; the faces in the
/// header follow an agent (the others fade back).
#[component]
pub fn AgentCursors(
    /// Your own document (the demo's when `None`); switch it with `tab` / `on_tab`.
    #[props(default)]
    scene: Option<CursorScene>,
    /// Tabs above the page: (id, label) – the demo's Doc / Table when empty and `scene` is
    /// `None`.
    #[props(default)]
    tabs: Vec<(String, String)>,
    /// The open tab.
    #[props(default = "doc".to_string())]
    tab: String,
    /// Freeze the demo at this many ms into a pass (a settled state).
    #[props(default)]
    at: Option<u64>,
    /// The agent followed at the start (id).
    #[props(default)]
    following: String,
    /// A suggestion whose window is open at the start (id).
    #[props(default)]
    open: String,
    /// A fixed page height in px – the rest of the document fades out (0 = all of it).
    #[props(default = 470.0)]
    height: f32,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// A suggestion accepted from its window.
    #[props(default)]
    on_accept: EventHandler<DocSuggestion>,
    /// A suggestion rejected from its window.
    #[props(default)]
    on_reject: EventHandler<DocSuggestion>,
    /// "Accept all".
    #[props(default)]
    on_accept_all: EventHandler<()>,
    /// The agent followed (id), or empty when you stop following.
    #[props(default)]
    on_following: EventHandler<String>,
    /// The check in a comment's window.
    #[props(default)]
    on_resolve: EventHandler<DocComment>,
    /// A reply sent from a comment's window (Enter).
    #[props(default)]
    on_reply: EventHandler<(DocComment, String)>,
    /// Another tab was chosen (its id).
    #[props(default)]
    on_tab: EventHandler<String>,
) -> Element {
    let demo = scene.is_none();
    let tabs = if tabs.is_empty() && demo {
        cursor_tabs()
    } else {
        tabs
    };
    let mut cur_tab = use_signal(move || tab.clone());
    let mut start = use_signal(move || now);
    let mut follow = use_signal(move || (!following.is_empty()).then(|| following.clone()));
    let open_sig = use_signal(move || (!open.is_empty()).then(|| open.clone()));
    let mut decided = use_signal(Vec::<(u64, String, bool)>::new);
    let resolved = use_signal(Vec::<(u64, String)>::new);

    let raw = now.since(start()).as_millis() as u64;
    let t_id = cur_tab();
    let (lap, el) = match (demo, at) {
        (true, Some(a)) => (0, a),
        (true, None) => {
            let l = cursors_loop_ms(&t_id).max(1);
            (raw / l, raw % l)
        }
        _ => (0, raw),
    };
    let mut sc = scene.clone().unwrap_or_else(|| cursors_at(&t_id, el));
    // Your decisions on top of the data (the demo's for this pass).
    for (l, id, ok) in decided().iter() {
        if *l != lap {
            continue;
        }
        if *ok {
            let (b, s) = accept_suggestion(&sc.blocks, &sc.suggestions, id);
            sc.blocks = b;
            sc.suggestions = s;
        } else {
            sc.suggestions.retain(|s| &s.id != id);
        }
        // A cursor typing into a decided suggestion drops back to plain text.
        for c in sc.cursors.iter_mut().flatten() {
            if &c.in_sugg == id {
                c.in_sugg.clear();
                c.typing = false;
                c.at = None;
            }
        }
    }
    let gone: Vec<String> = resolved()
        .iter()
        .filter(|(l, _)| *l == lap)
        .map(|x| x.1.clone())
        .collect();
    sc.comments.retain(|c| !gone.contains(&c.id));
    let following_i = follow().and_then(|f| agent_ix(&sc, &f));
    let ctx = Ctx {
        scene: sc.clone(),
        following: following_i,
        lap,
    };
    let n_sugg = sc.suggestions.len();
    let here = sc.cursors.iter().flatten().count();
    let total: f64 = sc.agents.iter().map(|a| a.cost).sum();
    let presence = presence_label(n_sugg, here);
    let total_s = format_cost(total);
    let body_style = if height > 0.0 {
        format!("height:{height}px")
    } else {
        String::new()
    };
    let fades = height > 0.0;

    // Blocks grouped so consecutive items form one list.
    let mut groups: Vec<Vec<DocBlock>> = vec![];
    for b in sc.blocks.iter() {
        match groups.last_mut() {
            Some(g) if b.kind == BlockKind::Item && g[0].kind == BlockKind::Item => {
                g.push(b.clone())
            }
            _ => groups.push(vec![b.clone()]),
        }
    }
    let comments_for = |id: &str| -> Vec<DocComment> {
        sc.comments
            .iter()
            .filter(|c| c.block == id)
            .cloned()
            .collect()
    };
    let lap_key = format!("{t_id}-{lap}");

    rsx! {
        div { class: "oxcu", "data-style": style_attr(&style), "data-tab": "{t_id}", "data-following": "{following_i.is_some()}",
            if !tabs.is_empty() {
                div { class: "oxcu-tabs", role: "tablist", "aria-label": "Views",
                    for (id, label) in tabs.iter().cloned() {
                        {
                            let sel = id == t_id;
                            let id2 = id.clone();
                            rsx! {
                                button { key: "{id}", "data-k": "{id}", class: "oxcu-tab", role: "tab", r#type: "button", "aria-selected": "{sel}",
                                    onclick: move |_| if cur_tab() != id2 {
                                        cur_tab.set(id2.clone()); start.set(now); follow.set(None); on_tab.call(id2.clone())
                                    },
                                    "{label}" }
                            }
                        }
                    }
                }
            }
            div { class: "oxcu-page",
                div { class: "oxcu-hd",
                    span { class: "oxcu-ttl", Icon { name: "file", size: 15.0 } span { "{sc.title}" } }
                    span { class: "oxcu-sp" }
                    div { class: "oxcu-faces", role: "group", "aria-label": "Agents in this document",
                        for (i, a) in sc.agents.iter().cloned().enumerate() {
                            {
                                let c = sc.cursors.get(i).cloned().flatten();
                                let inside = c.is_some();
                                let pressed = following_i == Some(i);
                                let label = face_label(&a.name, c.as_ref().map(|c| c.doing.as_str()), pressed);
                                let id = a.id.clone();
                                let tint = if a.tint.is_empty() { String::new() } else { format!("--oxcu-tint:{}", a.tint) };
                                let arc = if a.color.is_empty() { "var(--oxcu-ink)".to_string() } else { a.color.clone() };
                                let dim = following_i.is_some() && !pressed;
                                rsx! {
                                    button { key: "{id}", "data-k": "{id}", class: "oxcu-face", r#type: "button", "aria-pressed": "{pressed}", "aria-label": "{label}",
                                        "data-in": "{inside}", "data-dim": "{dim}", style: "{tint}",
                                        onclick: move |_| {
                                            if follow().as_deref() == Some(id.as_str()) { follow.set(None); on_following.call(String::new()) }
                                            else { follow.set(Some(id.clone())); on_following.call(id.clone()) }
                                        },
                                        BrandIcon { name: a.brand.clone(), size: 12.0, coloured: true, fallback: true }
                                        if inside {
                                            svg { class: "oxcu-arc", width: "24", height: "24", view_box: "0 0 24 24", "aria-hidden": "true",
                                                circle { cx: "12", cy: "12", r: "11", fill: "none", stroke: "{arc}", stroke_width: "1.4", stroke_linecap: "round", path_length: "100" } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    span { class: "oxcu-cnt", "{presence}" }
                    span { class: "oxcu-tot", "{total_s}" }
                    button { class: "oxcu-all", r#type: "button", disabled: n_sugg == 0,
                        onclick: move |_| {
                            let ids: Vec<String> = sc.suggestions.iter().map(|s| s.id.clone()).collect();
                            for id in ids { decided.write().push((lap, id, true)); }
                            on_accept_all.call(())
                        },
                        "Accept all" }
                }
                div { class: "oxcu-body", "data-fade": "{fades}", style: "{body_style}", key: "{lap_key}", "data-lap": "{lap_key}",
                    div { class: "oxcu-doc",
                        for g in groups.iter() {
                            {
                                let b0 = g[0].clone();
                                let gk = b0.id.clone();
                                match b0.kind {
                                    BlockKind::Item => rsx! {
                                        ul { key: "{gk}", "data-k": "{gk}", class: "oxcu-ul",
                                            for b in g.iter() {
                                                li { key: "{b.id}", "data-k": "{b.id}", class: "oxcu-blk",
                                                    for c in comments_for(&b.id) { CommentPin { key: "{c.id}", ctx: ctx.clone(), c: c.clone(), resolved, on_resolve, on_reply } }
                                                    {runs(&ctx, &b.id, &b.text, open_sig, decided, on_accept, on_reject)}
                                                }
                                            }
                                        }
                                    },
                                    BlockKind::Image => rsx! { div { key: "{gk}", "data-k": "{gk}", class: "oxcu-iw", "data-float": "{b0.float}", {image_block(&ctx, &b0)} } },
                                    BlockKind::Table => rsx! {
                                        div { key: "{gk}", "data-k": "{gk}", class: "oxcu-tw",
                                            table { class: "oxcu-tbl",
                                                for (r, row) in b0.rows.iter().enumerate() {
                                                    tr { key: "r{r}", "data-k": "r{r}", class: "oxcu-tr", "data-head": "{r == 0}",
                                                        for cell in row.iter() {
                                                            {
                                                                let tag_head = r == 0;
                                                                let inner = rsx! {
                                                                    for c in comments_for(&cell.id) { CommentPin { key: "{c.id}", ctx: ctx.clone(), c: c.clone(), resolved, on_resolve, on_reply } }
                                                                    if cell.tone.is_empty() {
                                                                        {runs(&ctx, &cell.id, &cell.text, open_sig, decided, on_accept, on_reject)}
                                                                    } else {
                                                                        span { class: "oxcu-pill", "data-tone": "{pill_tone(&ctx, cell)}",
                                                                            {runs(&ctx, &cell.id, &cell.text, open_sig, decided, on_accept, on_reject)} }
                                                                    }
                                                                };
                                                                if tag_head {
                                                                    rsx! { th { key: "{cell.id}", "data-k": "{cell.id}", scope: "col", class: "oxcu-td", {inner} } }
                                                                } else {
                                                                    rsx! { td { key: "{cell.id}", "data-k": "{cell.id}", class: "oxcu-td", {inner} } }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                                tr { class: "oxcu-tr", "data-add": "true",
                                                    td { class: "oxcu-td", colspan: "4", span { class: "oxcu-add", Icon { name: "plus", size: 13.0 } "Add row" } }
                                                }
                                            }
                                        }
                                    },
                                    kind => {
                                        let cls = match kind {
                                            BlockKind::Title => "oxcu-h2",
                                            BlockKind::Heading => "oxcu-h3",
                                            BlockKind::Code => "oxcu-code",
                                            _ => "oxcu-p",
                                        };
                                        let inner = rsx! {
                                            for c in comments_for(&b0.id) { CommentPin { key: "{c.id}", ctx: ctx.clone(), c: c.clone(), resolved, on_resolve, on_reply } }
                                            {runs(&ctx, &b0.id, &b0.text, open_sig, decided, on_accept, on_reject)}
                                        };
                                        match kind {
                                            BlockKind::Title => rsx! { h2 { key: "{gk}", "data-k": "{gk}", class: "oxcu-blk {cls}", {inner} } },
                                            BlockKind::Heading => rsx! { h3 { key: "{gk}", "data-k": "{gk}", class: "oxcu-blk {cls}", {inner} } },
                                            BlockKind::Code => rsx! { pre { key: "{gk}", "data-k": "{gk}", class: "oxcu-blk {cls}", code { {inner} } } },
                                            _ => rsx! { p { key: "{gk}", "data-k": "{gk}", class: "oxcu-blk {cls}", {inner} } },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxcu-sr", role: "status", "aria-live": "polite", "{sc.status}" }
        }
    }
}

/// A pill's tone: an accepted or pending status change shows the new status' tone.
fn pill_tone(ctx: &Ctx, cell: &DocBlock) -> String {
    let pending = ctx
        .scene
        .suggestions
        .iter()
        .find(|s| s.block == cell.id && !s.insert.is_empty());
    let word = pending.map(|s| s.insert.as_str()).unwrap_or(&cell.text);
    match word {
        "Doing" => "blue",
        "Done" => "green",
        "Idea" => "amber",
        "Blocked" => "red",
        "To do" => "grey",
        _ => cell.tone.as_str(),
    }
    .to_string()
}
