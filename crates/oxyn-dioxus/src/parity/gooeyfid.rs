//! Menu Motion family (variants 2): the gooey menu motion, offered as one shared Oxyn motion
//! (`oxgm-*` classes and an SVG goo filter) and used by [`GooeyAttachMenu`] (a drop-up of file
//! types and sources), [`GooeyModelMenu`] (models with their makers' marks), [`GooeyComposer`]
//! (a composer whose attach and model menus both pour out of their buttons), and two Oxyn
//! originals: [`LiquidTabs`] (a pill that stretches like a drop between tabs) and
//! [`LiquidDrops`] (actions that drip out of a round button). Models:
//! `oxyn_core::parity::gooeyfid`. Styles: `gooeyfid.css`.
//!
//! How the goo works: every shape that should melt together (the button, a stem, the menu sheet)
//! is drawn in one layer under an SVG filter that blurs it and then sharpens the alpha again, so
//! shapes close to each other join with a liquid neck. Text and rows sit in a normal layer on top.
//! Opening: the stem stretches out of the button and pinches off while the sheet swells from the
//! corner nearest the button and rows arrive top to bottom; closing runs the sheet back fast and
//! the menu leaves the page when that animation ends.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::parity::gooeyfid::{
    GooItem, ROW_H, goo_attach_items, goo_models, goo_move, panel_height, row_tops, stagger_ms,
    tab_box,
};

use super::attachfid::FileIcon;
use super::efflistfid::{check_glyph, chevron, focus, menu_key, use_uid};
use crate::brand::BrandIcon;

/// The goo filter (blur, then a hard alpha curve), referenced as `url(#{uid})`.
fn goo_filter(uid: &str, blur: f32) -> Element {
    let body = format!(
        "<filter id=\"{uid}\" filterUnits=\"userSpaceOnUse\" x=\"-1200\" y=\"-1200\" width=\"3000\" height=\"3000\" color-interpolation-filters=\"sRGB\"><feGaussianBlur in=\"SourceGraphic\" stdDeviation=\"{blur}\" result=\"b\"/><feColorMatrix in=\"b\" type=\"matrix\" values=\"1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  0 0 0 20 -8\" result=\"g\"/><feComposite in=\"SourceGraphic\" in2=\"g\" operator=\"atop\"/></filter>"
    );
    rsx! { svg { class: "oxgm-defs", width: "0", height: "0", "aria-hidden": "true", dangerous_inner_html: "{body}" } }
}

fn spark() -> Element {
    rsx! {
        svg { width: "16", height: "16", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M12 3l1.9 5.1L19 10l-5.1 1.9L12 17l-1.9-5.1L5 10l5.1-1.9zM19 16l.7 1.8 1.8.7-1.8.7-.7 1.8-.7-1.8-1.8-.7 1.8-.7z" }
        }
    }
}

fn plus(size: u32) -> Element {
    rsx! {
        svg { width: "{size}", height: "{size}", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
            stroke_linecap: "round", "aria-hidden": "true", path { d: "M12 5v14M5 12h14" } }
    }
}

/// The icon of a gooey row: a file-type icon, a brand mark or a glyph.
pub(crate) fn goo_icon(icon: &str) -> Element {
    if let Some(b) = icon.strip_prefix("brand:") {
        rsx! { BrandIcon { name: b.to_string(), size: 16.0, coloured: true, fallback: true } }
    } else if icon.starts_with("glyph:") {
        spark()
    } else {
        rsx! { FileIcon { name: icon.to_string(), size: 21.0 } }
    }
}

/// One gooey menu's state: `closed`, `open` or `closing`, the highlighted row, whether the user
/// opened it (only then a press outside closes it and the rows take focus), and its trigger.
#[derive(Clone, Copy)]
pub(crate) struct Goo {
    phase: Signal<&'static str>,
    hi: Signal<usize>,
    armed: Signal<bool>,
    want: Signal<bool>,
    trig: Signal<Option<Rc<MountedData>>>,
}

pub(crate) fn use_goo(start_open: bool) -> Goo {
    Goo {
        phase: use_signal(move || if start_open { "open" } else { "closed" }),
        hi: use_signal(|| 0usize),
        armed: use_signal(|| false),
        want: use_signal(|| false),
        trig: use_signal(|| None),
    }
}

impl Goo {
    fn is_open(self) -> bool {
        (self.phase)() == "open"
    }
    fn open(self, at: usize) {
        let (mut h, mut w, mut a, mut p) = (self.hi, self.want, self.armed, self.phase);
        h.set(at);
        w.set(true);
        a.set(true);
        p.set("open");
    }
    fn close(self, refocus: bool) {
        let mut p = self.phase;
        if (self.phase)() == "open" {
            p.set("closing");
        }
        if refocus {
            focus(self.trig.peek().clone());
        }
    }
    fn toggle(self, at: usize) {
        if self.is_open() {
            self.close(false)
        } else {
            self.open(at)
        }
    }
    fn ended(self) {
        let (mut p, mut w) = (self.phase, self.want);
        if (self.phase)() == "closing" {
            p.set("closed");
            w.set(false);
        }
    }
}

/// The swelling sheet and its stem, drawn inside the goo layer.
fn goo_sheet(g: Goo, pos: &str, stem: &str) -> Element {
    let ph = (g.phase)();
    rsx! {
        if ph != "closed" {
            span { class: "oxgm-sheet", "data-phase": "{ph}", style: "{pos}", onanimationend: move |_| g.ended() }
            if ph == "open" { span { class: "oxgm-stem", style: "{stem}" } }
        }
    }
}

/// The menu's rows, over the sheet. `role` is `menu` or `listbox`.
#[allow(clippy::too_many_arguments)]
fn goo_rows(
    g: Goo,
    uid: &str,
    role: &'static str,
    label: &str,
    items: &[GooItem],
    chosen: Option<usize>,
    pos: &str,
    pick: Callback<usize>,
) -> Element {
    let ph = (g.phase)();
    if ph == "closed" {
        return rsx! {};
    }
    let n = items.len();
    let tops = row_tops(items);
    let hi = (g.hi)().min(n.saturating_sub(1));
    let hl = format!(
        "transform:translateY({}px)",
        tops.get(hi).copied().unwrap_or(0.0)
    );
    let active = format!("{uid}-{hi}");
    let item_role = if role == "listbox" {
        "option"
    } else {
        "menuitem"
    };
    let mut last = "";
    let mut heads: Vec<(f32, &'static str)> = Vec::new();
    for (i, it) in items.iter().enumerate() {
        if !it.group.is_empty() && it.group != last {
            heads.push((tops[i] - 24.0, it.group));
        }
        last = it.group;
    }
    let mut hi_sig = g.hi;
    let list_h = tops.last().copied().unwrap_or(0.0) + ROW_H + 6.0;
    rsx! {
        div { class: "oxgm-pnl", "data-phase": "{ph}", style: "{pos}",
            div { class: "oxgm-scroll",
                ul { class: "oxgm-list", role: "{role}", tabindex: "-1", "aria-label": "{label}", "aria-activedescendant": "{active}",
                    style: "height:{list_h}px",
                    onmounted: move |e| { if (g.want)() { let d = e.data(); spawn(async move { let _ = d.set_focus(true).await; }); } },
                    onkeydown: move |e: KeyboardEvent| {
                        let k = e.key();
                        if let Some(s) = menu_key(&k) { e.prevent_default(); hi_sig.set(goo_move(hi, s, n)); }
                        else if k == Key::Enter || k == Key::Character(" ".into()) { e.prevent_default(); pick.call(hi); }
                        else if k == Key::Escape { e.prevent_default(); g.close(true); }
                        else if k == Key::Tab { g.close(false); }
                    },
                    li { class: "oxgm-hl", role: "presentation", "aria-hidden": "true", style: "{hl}" }
                    for (y, h) in heads.iter().copied() {
                        li { key: "h{h}", class: "oxgm-gh", role: "presentation", style: "top:{y}px", "{h}" }
                    }
                    for (i, it) in items.iter().enumerate() {
                        {
                            let st = format!("top:{}px;animation-delay:{}ms", tops[i], stagger_ms(i));
                            let sel = chosen == Some(i);
                            let aria_sel = (role == "listbox").then(|| sel.to_string());
                            rsx! {
                                li { key: "{it.key}", id: "{uid}-{i}", class: "oxgm-row", role: "{item_role}", style: "{st}",
                                    "aria-selected": aria_sel,
                                    "data-hi": "{i == hi}",
                                    onpointerenter: move |_| hi_sig.set(i),
                                    onclick: move |_| pick.call(i),
                                    span { class: "oxgm-ic", {goo_icon(it.icon)} }
                                    span { class: "oxgm-lb", "{it.label}" }
                                    if !it.hint.is_empty() { span { class: "oxgm-hint", "{it.hint}" } }
                                    if sel { span { class: "oxgm-ck", {check_glyph()} } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn trigger_key(g: Goo, e: KeyboardEvent, at: usize) {
    let k = e.key();
    if k == Key::ArrowUp || k == Key::ArrowDown {
        e.prevent_default();
        g.open(at);
    } else if k == Key::Escape && g.is_open() {
        e.prevent_default();
        g.close(true);
    }
}

/// Menu Motion – Gooey attach menu: a + button whose drop-up of file types (PDF, Image, Excel,
/// Doc, Slides, Code) and sources (Drive, GitHub) pours out of it like liquid.
#[component]
pub fn GooeyAttachMenu(
    /// Rows (file types and sources when `None`).
    #[props(default)]
    items: Option<Vec<GooItem>>,
    /// `up` (drop-up) or `down`.
    #[props(default = "up".to_string())]
    side: String,
    /// Goo strength (blur px).
    #[props(default = 7.0)]
    goo: f32,
    /// Starts open (for previews).
    #[props(default)]
    default_open: bool,
    /// A row was picked (its key).
    #[props(default)]
    on_pick: EventHandler<String>,
) -> Element {
    let items = items.unwrap_or_else(goo_attach_items);
    let g = use_goo(default_open);
    let uid = use_uid("oxgma");
    let mut said = use_signal(String::new);
    let h = panel_height(&items, 330.0);
    let up = side != "down";
    let pos = if up {
        format!("left:0;bottom:50px;width:280px;height:{h}px")
    } else {
        format!("left:0;top:50px;width:280px;height:{h}px")
    };
    let stem = if up {
        "left:6px;bottom:18px;width:26px;height:40px".to_string()
    } else {
        "left:6px;top:18px;width:26px;height:40px".to_string()
    };
    let all = items.clone();
    let pick = use_callback(move |i: usize| {
        said.set(format!("{} picked.", all[i].label));
        on_pick.call(all[i].key.to_string());
        g.close(true);
    });
    let catch = g.is_open() && (g.armed)();
    let open = g.is_open();
    let mut trig = g.trig;

    rsx! {
        div { class: "oxgm oxgm-solo", "data-side": "{side}", "data-open": "{open}", "data-armed": "{catch}",
            {goo_filter(&uid, goo)}
            if catch { div { class: "oxgm-catch", "aria-hidden": "true", onpointerdown: move |_| g.close(false) } }
            div { class: "oxgm-goo", style: "filter:url(#{uid}) var(--oxgm-shadow)", "aria-hidden": "true",
                span { class: "oxgm-blob", style: "left:0;bottom:2px;width:38px;height:38px;border-radius:13px" }
                {goo_sheet(g, &pos, &stem)}
            }
            button { class: "oxgm-plus", r#type: "button", "aria-label": "Add to the prompt", "aria-haspopup": "menu", "aria-expanded": "{open}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| g.toggle(0),
                onkeydown: move |e: KeyboardEvent| trigger_key(g, e, 0),
                span { class: "oxgm-pl", "data-open": "{open}", {plus(18)} }
            }
            span { class: "oxgm-cap", "Attach" }
            {goo_rows(g, &uid, "menu", "Add to the prompt", &items, None, &pos, pick)}
            div { class: "oxgm-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// Menu Motion – Gooey model menu: a model chip whose list of models (each with its maker's
/// mark and a one-line hint) pours out of its corner; the chosen one is checked and its name
/// blurs into the chip.
#[component]
pub fn GooeyModelMenu(
    /// Models (the default set when `None`).
    #[props(default)]
    models: Option<Vec<GooItem>>,
    /// Starting model index.
    #[props(default = 1)]
    default_model: usize,
    /// `up` or `down`.
    #[props(default = "up".to_string())]
    side: String,
    /// Goo strength (blur px).
    #[props(default = 7.0)]
    goo: f32,
    /// Starts open (for previews).
    #[props(default)]
    default_open: bool,
    /// A model was picked (its key).
    #[props(default)]
    on_change: EventHandler<String>,
) -> Element {
    let models = models.unwrap_or_else(goo_models);
    let n = models.len().max(1);
    let g = use_goo(default_open);
    let uid = use_uid("oxgmm");
    let mut cur = use_signal(move || default_model.min(n - 1));
    let mut said = use_signal(String::new);
    let h = panel_height(&models, 330.0);
    let up = side != "down";
    let pos = if up {
        format!("right:0;bottom:50px;width:268px;height:{h}px")
    } else {
        format!("right:0;top:50px;width:268px;height:{h}px")
    };
    let stem = if up {
        "right:48px;bottom:18px;width:26px;height:40px".to_string()
    } else {
        "right:48px;top:18px;width:26px;height:40px".to_string()
    };
    let all = models.clone();
    let pick = use_callback(move |i: usize| {
        cur.set(i);
        said.set(format!("Model: {}.", all[i].label));
        on_change.call(all[i].key.to_string());
        g.close(true);
    });
    let m = models[cur().min(n - 1)].clone();
    let catch = g.is_open() && (g.armed)();
    let open = g.is_open();
    let mut trig = g.trig;
    let c = cur();

    rsx! {
        div { class: "oxgm oxgm-solo oxgm-right", "data-side": "{side}", "data-open": "{open}", "data-armed": "{catch}",
            {goo_filter(&uid, goo)}
            if catch { div { class: "oxgm-catch", "aria-hidden": "true", onpointerdown: move |_| g.close(false) } }
            div { class: "oxgm-goo", style: "filter:url(#{uid}) var(--oxgm-shadow)", "aria-hidden": "true",
                span { class: "oxgm-blob", style: "right:0;bottom:2px;width:132px;height:38px;border-radius:13px" }
                {goo_sheet(g, &pos, &stem)}
            }
            button { class: "oxgm-chip", r#type: "button", "aria-label": "Model: {m.label}", "aria-haspopup": "listbox", "aria-expanded": "{open}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| g.toggle(c),
                onkeydown: move |e: KeyboardEvent| trigger_key(g, e, c),
                span { class: "oxgm-ic", {goo_icon(m.icon)} }
                for k in std::iter::once(m.label) { span { key: "{k}", class: "oxgm-name", "{k}" } }
                span { class: "oxgm-chev", {chevron(!open)} }
            }
            {goo_rows(g, &uid, "listbox", "Select a model", &models, Some(c), &pos, pick)}
            div { class: "oxgm-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// Menu Motion – Gooey composer: a prompt box whose + menu (spanning the box) and model menu
/// (lined up with its chip) both pour out of the box's top edge above their buttons. Picking a
/// file type adds a chip; picking a model blurs its name into the chip.
#[component]
pub fn GooeyComposer(
    /// Goo strength (blur px).
    #[props(default = 7.0)]
    goo: f32,
    /// Starting model index.
    #[props(default = 1)]
    default_model: usize,
    /// Start with this menu open: `attach`, `model` or empty.
    #[props(default)]
    default_open: String,
    /// The prompt was sent (its text).
    #[props(default)]
    on_send: EventHandler<String>,
) -> Element {
    let items = goo_attach_items();
    let models = goo_models();
    let a = use_goo(default_open == "attach");
    let m = use_goo(default_open == "model");
    let uid = use_uid("oxgmc");
    let mut cur = use_signal(move || default_model);
    let mut text = use_signal(String::new);
    let mut chips = use_signal(Vec::<(u32, &'static str, &'static str)>::new);
    let mut said = use_signal(String::new);
    let ha = panel_height(&items, 300.0);
    let hm = panel_height(&models, 300.0);
    let pos_a = format!("left:0;right:0;bottom:calc(100% + 10px);height:{ha}px");
    let pos_m = format!("right:0;width:268px;bottom:calc(100% + 10px);height:{hm}px");
    let stem_a = "left:14px;bottom:calc(100% - 8px);width:26px;height:26px";
    let stem_m = "right:82px;bottom:calc(100% - 8px);width:26px;height:26px";
    let ai = items.clone();
    let pick_a = use_callback(move |i: usize| {
        let id = chips.peek().iter().map(|c| c.0).max().unwrap_or(0) + 1;
        let name = match ai[i].key {
            "pdf" => "Brief.pdf",
            "image" => "Screen.png",
            "sheet" => "Budget.xlsx",
            "doc" => "Notes.docx",
            "slides" => "Deck.pptx",
            "code" => "main.rs",
            "drive" => "From Drive.pdf",
            _ => "issue-142.md",
        };
        chips.write().push((id, name, ai[i].key));
        said.set(format!("{} added.", ai[i].label));
        a.close(true);
    });
    let mi = models.clone();
    let pick_m = use_callback(move |i: usize| {
        cur.set(i);
        said.set(format!("Model: {}.", mi[i].label));
        m.close(true);
    });
    let mm = models[cur().min(models.len() - 1)].clone();
    let catch = (a.is_open() && (a.armed)()) || (m.is_open() && (m.armed)());
    let ready = !text().trim().is_empty() || !chips().is_empty();
    let (ao, mo) = (a.is_open(), m.is_open());
    let (mut ta, mut tm) = (a.trig, m.trig);
    let c = cur();

    rsx! {
        div { class: "oxgm oxgm-comp", "data-armed": "{catch}",
            {goo_filter(&uid, goo)}
            if catch { div { class: "oxgm-catch", "aria-hidden": "true", onpointerdown: move |_| { a.close(false); m.close(false); } } }
            div { class: "oxgm-goo", style: "filter:url(#{uid}) var(--oxgm-shadow)", "aria-hidden": "true",
                span { class: "oxgm-blob oxgm-box" }
                {goo_sheet(a, &pos_a, stem_a)}
                {goo_sheet(m, &pos_m, stem_m)}
            }
            div { class: "oxgm-field",
                if !chips().is_empty() {
                    div { class: "oxgm-chips",
                        for (id, name, _k) in chips() {
                            span { key: "{id}", class: "oxgm-fchip",
                                FileIcon { name: name.to_string(), size: 20.0 }
                                span { "{name}" }
                                button { class: "oxgm-x", r#type: "button", "aria-label": "Remove {name}",
                                    onclick: move |_| chips.write().retain(|c| c.0 != id), "×" }
                            }
                        }
                    }
                }
                textarea { class: "oxgm-ta", rows: "2", placeholder: "Ask anything…", "aria-label": "Message", value: "{text}",
                    oninput: move |e| text.set(e.value()) }
            }
            div { class: "oxgm-foot",
                button { class: "oxgm-plus", r#type: "button", "aria-label": "Add to the prompt", "aria-haspopup": "menu", "aria-expanded": "{ao}",
                    onmounted: move |e| ta.set(Some(e.data())),
                    onclick: move |_| { m.close(false); a.toggle(0); },
                    onkeydown: move |e: KeyboardEvent| trigger_key(a, e, 0),
                    span { class: "oxgm-pl", "data-open": "{ao}", {plus(18)} }
                }
                span { class: "oxgm-grow" }
                button { class: "oxgm-chip oxgm-inline", r#type: "button", "aria-label": "Model: {mm.label}", "aria-haspopup": "listbox", "aria-expanded": "{mo}",
                    onmounted: move |e| tm.set(Some(e.data())),
                    onclick: move |_| { a.close(false); m.toggle(c); },
                    onkeydown: move |e: KeyboardEvent| trigger_key(m, e, c),
                    span { class: "oxgm-ic", {goo_icon(mm.icon)} }
                    for k in std::iter::once(mm.label) { span { key: "{k}", class: "oxgm-name", "{k}" } }
                    span { class: "oxgm-chev", {chevron(!mo)} }
                }
                button { class: "oxgm-send", r#type: "button", disabled: !ready, "aria-label": "Send",
                    onclick: move |_| { on_send.call(text()); said.set("Sent.".into()); text.set(String::new()); chips.set(Vec::new()); },
                    svg { width: "16", height: "16", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2.2",
                        stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "M12 19V5M6 11l6-6 6 6" } }
                }
            }
            {goo_rows(a, &format!("{uid}a"), "menu", "Add to the prompt", &items, None, &pos_a, pick_a)}
            {goo_rows(m, &format!("{uid}m"), "listbox", "Select a model", &models, Some(c), &pos_m, pick_m)}
            div { class: "oxgm-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// Menu Motion – Liquid tabs (an Oxyn original): a segmented control whose ink pill moves like a
/// drop – it leaves a trailing droplet that stretches behind it and is pulled back in on arrival.
#[component]
pub fn LiquidTabs(
    /// Tab labels.
    #[props(default = vec!["Chat".to_string(), "Agent".to_string(), "Plan".to_string(), "Code".to_string()])]
    tabs: Vec<String>,
    /// Starting tab.
    #[props(default)]
    default_value: usize,
    /// Goo strength (blur px).
    #[props(default = 6.0)]
    goo: f32,
    /// The tab changed.
    #[props(default)]
    on_change: EventHandler<usize>,
) -> Element {
    let n = tabs.len().max(1);
    let mut cur = use_signal(move || default_value.min(n - 1));
    let mut prev = use_signal(move || default_value.min(n - 1));
    let uid = use_uid("oxlt");
    let i = cur();
    let (l, w) = tab_box(i, n);
    let dir = if i >= prev() { "r" } else { "l" };
    let pill = format!("left:calc({l}% + 3px);width:calc({w}% - 6px)");
    let mut go = move |j: usize| {
        prev.set(cur());
        cur.set(j);
        on_change.call(j);
    };

    rsx! {
        div { class: "oxlt", "data-dir": "{dir}",
            {goo_filter(&uid, goo)}
            div { class: "oxlt-goo", style: "filter:url(#{uid})", "aria-hidden": "true",
                span { class: "oxlt-pill", style: "{pill}" }
                span { class: "oxlt-trail", style: "{pill}" }
            }
            div { class: "oxlt-tabs", role: "tablist", "aria-label": "Mode",
                onkeydown: move |e: KeyboardEvent| {
                    let j = match e.key() { Key::ArrowRight => Some((i + 1) % n), Key::ArrowLeft => Some((i + n - 1) % n), Key::Home => Some(0), Key::End => Some(n - 1), _ => None };
                    if let Some(j) = j { e.prevent_default(); go(j); }
                },
                for (j, t) in tabs.iter().enumerate() {
                    button { key: "{t}", class: "oxlt-tab", role: "tab", r#type: "button", "aria-selected": "{j == i}",
                        tabindex: if j == i { "0" } else { "-1" },
                        onclick: move |_| go(j), "{t}" }
                }
            }
        }
    }
}

/// Menu Motion – Liquid drops (an Oxyn original): a round + button that lets its actions drip
/// out upward one after another, each pulling free of the button like a drop; they flow back in
/// when it closes.
#[component]
pub fn LiquidDrops(
    /// Actions: (key, label); icons by key (`file`, `shot`, `voice`, `link`).
    #[props(default = vec![("file".to_string(), "Upload a file".to_string()), ("shot".to_string(), "Screenshot".to_string()), ("voice".to_string(), "Voice note".to_string()), ("link".to_string(), "Paste a link".to_string())])]
    actions: Vec<(String, String)>,
    /// Gap between drops in px.
    #[props(default = 54.0)]
    gap: f32,
    /// Goo strength (blur px).
    #[props(default = 7.0)]
    goo: f32,
    /// Starts open (for previews).
    #[props(default)]
    default_open: bool,
    /// An action was picked (its key).
    #[props(default)]
    on_pick: EventHandler<String>,
) -> Element {
    let mut open = use_signal(move || default_open);
    let mut said = use_signal(String::new);
    let uid = use_uid("oxld");
    let n = actions.len();
    let o = open();
    let height = 56.0 + gap * n as f32 + 8.0;
    let icon = |k: &str| -> &'static str {
        match k {
            "file" => "M7 3h7l4 4v14H7zM14 3v4h4",
            "shot" => "M4 8V5h3M17 5h3v3M20 16v3h-3M7 19H4v-3M9 12h6",
            "voice" => {
                "M12 4a3 3 0 0 0-3 3v5a3 3 0 0 0 6 0V7a3 3 0 0 0-3-3zM6 11a6 6 0 0 0 12 0M12 17v3"
            }
            _ => {
                "M10 14a4 4 0 0 0 5.7 0l3-3a4 4 0 0 0-5.7-5.7l-1 1M14 10a4 4 0 0 0-5.7 0l-3 3a4 4 0 0 0 5.7 5.7l1-1"
            }
        }
    };

    rsx! {
        div { class: "oxld", "data-open": "{o}", style: "height:{height}px",
            {goo_filter(&uid, goo)}
            div { class: "oxld-goo", style: "filter:url(#{uid})", "aria-hidden": "true",
                span { class: "oxld-fab-b" }
                for i in 0..n {
                    {
                        let y = if o { -(gap * (i + 1) as f32) } else { 0.0 };
                        let d = if o { i * 55 } else { (n - 1 - i) * 30 };
                        rsx! { span { key: "{i}", class: "oxld-drop-b", style: "transform:translateY({y}px);transition-delay:{d}ms" } }
                    }
                }
            }
            for (i, (k, label)) in actions.iter().enumerate() {
                {
                    let y = if o { -(gap * (i + 1) as f32) } else { 0.0 };
                    let d = if o { i * 55 } else { (n - 1 - i) * 30 };
                    let key = k.clone();
                    let lab = label.clone();
                    let path = icon(k);
                    rsx! {
                        div { key: "{k}", class: "oxld-act", style: "transform:translateY({y}px);transition-delay:{d}ms",
                            span { class: "oxld-lab", style: "transition-delay:{d + 120}ms", "{label}" }
                            button { class: "oxld-drop", r#type: "button", "aria-label": "{label}", tabindex: if o { "0" } else { "-1" },
                                onclick: move |_| { said.set(format!("{lab}.")); on_pick.call(key.clone()); open.set(false); },
                                svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8",
                                    stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "{path}" } }
                            }
                        }
                    }
                }
            }
            button { class: "oxld-fab", r#type: "button", "aria-label": if o { "Close actions" } else { "Open actions" }, "aria-expanded": "{o}",
                onclick: move |_| open.set(!o),
                onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { open.set(false) },
                span { class: "oxld-pl", {plus(22)} }
            }
            div { class: "oxgm-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}
