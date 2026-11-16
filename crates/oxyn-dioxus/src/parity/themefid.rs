//! Theme Selector family (variants 3): how the app looks – [`BackgroundPicker`] (picture tiles we
//! draw ourselves, a tile for your own picture, and a texture laid over it), [`ThemeGrid`] (eight
//! mini-window swatches and a font size menu over a live sample), and two Oxyn originals:
//! [`ThemeSplit`] (two themes over one sample window with a divider you drag) and
//! [`AccentDensity`] (accent, spacing and corners on a settings sample). Models:
//! `oxyn_core::parity::themefid`. Styles: `themefid.css` (`oxbgk-`, `oxtsg-`, `oxtpv-`, `oxadn-`).

use dioxus::prelude::*;
use oxyn_core::parity::effmenufid::menu_move;
use oxyn_core::parity::themefid::{
    Texture, ThemeSwatch, accents, backdrops, corners, data_url, densities, font_sizes, own_check,
    split_clamp, swatch_for, swatches, texture_press,
};

use super::efflistfid::{check_glyph, chevron, focus, menu_key, use_uid};

/// The CSS variables a sample window reads for one theme.
fn swatch_vars(s: &ThemeSwatch) -> String {
    format!(
        "--ws:#{};--wp:#{};--wl:#{};--wb:#{};--wf:#{};--wm:#{}",
        s.side, s.pane, s.line, s.block, s.fg, s.mute
    )
}

/// One of our own background pictures, drawn as SVG (`slot` keeps gradient ids apart).
pub(crate) fn backdrop_art(key: &str, slot: &str) -> Element {
    let g = format!("oxbgk-{key}-{slot}");
    let gu = format!("url(#{g})");
    let g2 = format!("{g}b");
    let gu2 = format!("url(#{g2})");
    match key {
        "canopy" => rsx! {
            svg { class: "oxbgk-art", view_box: "0 0 160 90", preserve_aspect_ratio: "xMidYMid slice", "aria-hidden": "true",
                defs { linearGradient { id: "{g}", x1: "0", y1: "0", x2: "0", y2: "1",
                    stop { offset: "0", stop_color: "#8FC3D8" } stop { offset: "1", stop_color: "#5E98B8" } } }
                rect { width: "160", height: "90", fill: "{gu}" }
                g { stroke: "#1E2A24", stroke_linecap: "round", fill: "none",
                    path { d: "M-4 8 C30 14 52 30 70 52", stroke_width: "2.4" }
                    path { d: "M28 15 C40 6 54 4 70 2", stroke_width: "1.4" }
                    path { d: "M52 30 C66 26 86 30 104 22", stroke_width: "1.3" }
                    path { d: "M164 4 C130 12 118 26 112 46", stroke_width: "2" }
                    path { d: "M128 14 C140 24 150 24 164 30", stroke_width: "1.2" }
                }
                g { fill: "#D9472F",
                    for (x, y) in [(12, 9), (22, 14), (34, 19), (44, 24), (56, 33), (62, 42), (40, 8), (58, 5), (74, 27), (88, 29), (98, 24),
                        (120, 30), (116, 40), (132, 17), (142, 22), (152, 26), (150, 9), (138, 8)] {
                        circle { key: "{x}-{y}", cx: "{x}", cy: "{y}", r: "2.3" }
                    }
                }
                g { fill: "#F08A4B",
                    for (x, y) in [(17, 12), (30, 22), (50, 28), (68, 36), (82, 25), (126, 22), (146, 14), (110, 34)] {
                        circle { key: "{x}-{y}", cx: "{x}", cy: "{y}", r: "1.8" }
                    }
                }
                circle { cx: "96", cy: "58", r: "1.6", fill: "#F5D76E" }
            }
        },
        "orbit" => rsx! {
            svg { class: "oxbgk-art", view_box: "0 0 160 90", preserve_aspect_ratio: "xMidYMid slice", "aria-hidden": "true",
                defs { linearGradient { id: "{g}", x1: "0", y1: "0", x2: "1", y2: "1",
                    stop { offset: "0.30", stop_color: "#F2702C" } stop { offset: "0.40", stop_color: "#FFE8C2" }
                    stop { offset: "0.50", stop_color: "#E9F4FF" } stop { offset: "0.66", stop_color: "#3E8EE6" }
                    stop { offset: "1", stop_color: "#071226" } } }
                rect { width: "160", height: "90", fill: "#030305" }
                path { d: "M30 -10 C70 30 110 70 190 96 L190 120 L-20 120 Z", fill: "{gu}" }
                path { d: "M98 48 l6 -1 l2 -3 l1 3 l5 1 l-5 1 l-1 3 l-2 -3 z", fill: "#050507" }
                g { fill: "#FFFFFF", opacity: ".7",
                    circle { cx: "12", cy: "70", r: ".6" } circle { cx: "24", cy: "52", r: ".5" } circle { cx: "8", cy: "30", r: ".5" }
                }
            }
        },
        "neon" => rsx! {
            svg { class: "oxbgk-art", view_box: "0 0 160 90", preserve_aspect_ratio: "xMidYMid slice", "aria-hidden": "true",
                defs {
                    filter { id: "{g}", x: "-20%", y: "-20%", width: "140%", height: "140%", feGaussianBlur { std_deviation: "5" } }
                    radialGradient { id: "{g2}", cx: ".55", cy: ".45", r: ".7",
                        stop { offset: "0", stop_color: "#FF9BEA" } stop { offset: ".45", stop_color: "#D23CB4" } stop { offset: "1", stop_color: "#3A0D36" } }
                }
                rect { width: "160", height: "90", fill: "{gu2}" }
                g { filter: "{gu}",
                    rect { x: "70", y: "8", width: "26", height: "70", fill: "#FFD9F6", opacity: ".85" }
                    rect { x: "104", y: "20", width: "12", height: "56", fill: "#FF6FD8", opacity: ".8" }
                    rect { x: "20", y: "40", width: "38", height: "10", fill: "#FF7CDF", opacity: ".6" }
                    circle { cx: "128", cy: "26", r: "10", fill: "#FFFFFF", opacity: ".6" }
                    rect { x: "30", y: "60", width: "50", height: "26", fill: "#5A1450", opacity: ".8" }
                }
                g { stroke: "#FFE6FA", stroke_width: ".7", opacity: ".55",
                    path { d: "M0 70 L160 34" } path { d: "M0 78 L160 44" }
                }
            }
        },
        "tide" => rsx! {
            svg { class: "oxbgk-art", view_box: "0 0 160 90", preserve_aspect_ratio: "xMidYMid slice", "aria-hidden": "true",
                defs { linearGradient { id: "{g}", x1: "0", y1: "0", x2: "0", y2: "1",
                    stop { offset: "0", stop_color: "#0B1A33" } stop { offset: ".55", stop_color: "#12407A" } stop { offset: "1", stop_color: "#0A2347" } } }
                rect { width: "160", height: "90", fill: "{gu}" }
                circle { cx: "118", cy: "20", r: "8", fill: "#DCEBFF", opacity: ".9" }
                path { d: "M0 52 C20 44 34 58 54 50 S90 42 110 50 S146 58 160 48 V90 H0 Z", fill: "#1D5FA8" }
                path { d: "M0 62 C18 56 36 68 58 60 S96 54 118 62 S148 68 160 60 V90 H0 Z", fill: "#2F7DD1" }
                path { d: "M0 74 C22 68 40 80 62 72 S100 66 122 74 S150 80 160 72 V90 H0 Z", fill: "#7FC8F8" }
                path { d: "M0 62 C18 56 36 68 58 60 S96 54 118 62 S148 68 160 60", fill: "none", stroke: "#E9F6FF", stroke_width: "1", opacity: ".7" }
            }
        },
        _ => rsx! {},
    }
}

/// The texture layer over a picture (`none` draws nothing).
fn texture_layer(t: Texture) -> Element {
    if t == Texture::None {
        return rsx! {};
    }
    let k = t.key();
    rsx! { span { class: "oxbgk-tex", "data-tex": "{k}", "aria-hidden": "true" } }
}

fn plus_glyph() -> Element {
    rsx! {
        svg { width: "18", height: "18", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.8",
            stroke_linecap: "round", "aria-hidden": "true", path { d: "M12 5v14M5 12h14" } }
    }
}

/// Theme Selector – Background picker: tiles for the plain default, four pictures we draw and
/// "Your own +" (opens the file picker; the picture joins the tiles), then a Film / Static / Dither
/// texture laid over the picture – press the lit one again to take it off. The preview above shows
/// a home screen over the chosen picture.
#[component]
pub fn BackgroundPicker(
    /// Starting tile key (`default`, `canopy`, `orbit`, `neon`, `tide`).
    #[props(default = "default".to_string())]
    default_pick: String,
    /// Starting texture key (`none`, `film`, `static`, `dither`).
    #[props(default = "none".to_string())]
    default_texture: String,
    /// The home screen preview above the tiles.
    #[props(default = true)]
    preview: bool,
    /// Tile columns (2–4).
    #[props(default = 3)]
    cols: usize,
    /// The background or texture changed: (tile key, texture key).
    #[props(default)]
    on_change: EventHandler<(String, String)>,
) -> Element {
    let mut pick = use_signal(move || default_pick);
    let mut tex = use_signal(move || Texture::from_key(&default_texture));
    let mut own = use_signal(|| None::<String>);
    let mut note = use_signal(String::new);
    let all = backdrops();
    let cur = pick();
    let t = tex();
    let cur_label = if cur == "own" {
        "Your picture".to_string()
    } else {
        all.iter()
            .find(|b| b.key == cur)
            .map(|b| b.label)
            .unwrap_or("Default")
            .to_string()
    };
    let grid = format!(
        "grid-template-columns:repeat({},minmax(0,1fr))",
        cols.clamp(2, 4)
    );
    let own_url = own();
    let own_bg = own_url
        .as_ref()
        .map(|u| format!("background-image:url(\"{u}\")"))
        .unwrap_or_default();
    let pv_bg = own_bg.clone();
    let cur_pv = cur.clone();
    let pv_plain = cur == "default";

    rsx! {
        div { class: "oxbgk", "data-pick": "{cur}", "data-tex": "{t.key()}",
            if preview {
                div { class: "oxbgk-pv", "data-plain": "{pv_plain}", role: "img",
                    "aria-label": "Preview: {cur_label}, texture {t.label()}",
                    if cur_pv == "own" { span { class: "oxbgk-own-img", style: "{pv_bg}" } } else { {backdrop_art(&cur_pv, "p")} }
                    {texture_layer(t)}
                    div { class: "oxbgk-pv-ui",
                        span { class: "oxbgk-pv-hi", "Afternoon. What shall we start?" }
                        span { class: "oxbgk-pv-box", span { "Ask anything" } span { class: "oxbgk-pv-send" } }
                    }
                }
            }
            div { class: "oxbgk-sec",
                p { class: "oxbgk-h", "Background" }
                p { class: "oxbgk-sub", "Sits behind the home screen. Moving pictures play too." }
                div { class: "oxbgk-grid", role: "radiogroup", "aria-label": "Background", style: "{grid}",
                    for b in all.iter() {
                        {
                            let on = cur == b.key;
                            let key = b.key;
                            let plain = key == "default";
                            let label = b.label;
                            let alt = b.alt;
                            rsx! {
                                button { key: "{key}", class: "oxbgk-tile", r#type: "button", role: "radio", "aria-checked": "{on}",
                                    "aria-label": "{label}: {alt}", "data-plain": "{plain}",
                                    onclick: move |_| { pick.set(key.to_string()); note.set(format!("{label} is your background.")); on_change.call((key.to_string(), tex.peek().key().to_string())); },
                                    if plain { span { class: "oxbgk-name", "{label}" } } else { {backdrop_art(key, "t")} {texture_layer(t)} span { class: "oxbgk-cap", "{label}" } }
                                    if on { span { class: "oxbgk-check", {check_glyph()} } }
                                }
                            }
                        }
                    }
                    if own_url.is_some() {
                        {
                            let on = cur == "own";
                            rsx! {
                                button { class: "oxbgk-tile", r#type: "button", role: "radio", "aria-checked": "{on}", "aria-label": "Your picture",
                                    onclick: move |_| { pick.set("own".into()); note.set("Your picture is your background.".into()); on_change.call(("own".into(), tex.peek().key().to_string())); },
                                    span { class: "oxbgk-own-img", style: "{own_bg}" }
                                    {texture_layer(t)}
                                    span { class: "oxbgk-cap", "Yours" }
                                    if on { span { class: "oxbgk-check", {check_glyph()} } }
                                }
                            }
                        }
                    }
                    label { class: "oxbgk-tile oxbgk-add",
                        input { class: "oxbgk-file", r#type: "file", accept: "image/*", "aria-label": "Your own picture",
                            onchange: move |e: FormEvent| {
                                if let Some(f) = e.files().into_iter().next() {
                                    let mime = f.content_type().unwrap_or_default();
                                    match own_check(&mime, f.size()) {
                                        Err(m) => note.set(m.to_string()),
                                        Ok(()) => {
                                            note.set("Reading your picture…".into());
                                            spawn(async move {
                                                match f.read_bytes().await {
                                                    Ok(b) => {
                                                        own.set(Some(data_url(&mime, &b)));
                                                        pick.set("own".into());
                                                        note.set("Your picture is your background.".into());
                                                        on_change.call(("own".into(), tex.peek().key().to_string()));
                                                    }
                                                    Err(_) => note.set("That picture could not be read.".into()),
                                                }
                                            });
                                        }
                                    }
                                }
                            }
                        }
                        span { class: "oxbgk-plus", {plus_glyph()} }
                        span { class: "oxbgk-name", "Your own" }
                    }
                }
            }
            div { class: "oxbgk-row",
                div { class: "oxbgk-rt",
                    p { class: "oxbgk-h", "Texture" }
                    p { class: "oxbgk-sub", "Grain drawn over the picture. Press again for none." }
                }
                div { class: "oxbgk-seg", role: "group", "aria-label": "Texture",
                    for x in Texture::ALL {
                        {
                            let on = t == x;
                            let lab = x.label();
                            rsx! {
                                button { key: "{lab}", class: "oxbgk-segb", r#type: "button", "aria-pressed": "{on}",
                                    onclick: move |_| {
                                        let n = texture_press(*tex.peek(), x);
                                        tex.set(n);
                                        note.set(if n == Texture::None { "No texture.".to_string() } else { format!("{} texture on.", n.label()) });
                                        on_change.call((pick.peek().clone(), n.key().to_string()));
                                    },
                                    "{lab}"
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxbgk-note", role: "status", "aria-live": "polite", "{note}" }
        }
    }
}

/// A mini window drawn in one theme: a sidebar of bars and a pane holding a button block.
fn mini_window(s: &ThemeSwatch, class: &str) -> Element {
    let vars = swatch_vars(s);
    let k = s.key;
    rsx! {
        span { class: "{class}", style: "{vars}", "data-sw": "{k}",
            span { class: "oxtsg-side", i {} i {} i {} i {} }
            span { class: "oxtsg-pane", b {} }
        }
    }
}

fn monitor_glyph() -> Element {
    rsx! {
        svg { width: "14", height: "14", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "2",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true",
            path { d: "M4 5h16v11H4zM9 20h6M12 16v4" } }
    }
}

/// Theme Selector – Theme grid: eight mini-window swatches (System split corner to corner,
/// Light, Dark, four tinted darks and Contrast), then a Font size menu. The sample under them
/// takes the chosen theme and size at once.
#[component]
pub fn ThemeGrid(
    /// Starting theme key.
    #[props(default = "system".to_string())]
    default_theme: String,
    /// Starting font size key (`small`, `default`, `large`, `larger`).
    #[props(default = "default".to_string())]
    default_font: String,
    /// Whether the device is dark (what System follows).
    #[props(default = true)]
    device_dark: bool,
    /// The live sample under the grid.
    #[props(default = true)]
    preview: bool,
    /// Swatch columns (2 or 4).
    #[props(default = 4)]
    cols: usize,
    /// Theme or font size changed: (theme key, font key).
    #[props(default)]
    on_change: EventHandler<(String, String)>,
) -> Element {
    let mut theme = use_signal(move || default_theme);
    let mut font = use_signal(move || default_font);
    let mut open = use_signal(|| false);
    let mut hi = use_signal(|| 0usize);
    let mut chip = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut want = use_signal(|| false);
    let mut said = use_signal(String::new);
    let uid = use_uid("oxtsg");
    let sws = swatches();
    let fonts = font_sizes();
    let nf = fonts.len();
    let cur = theme();
    let fcur = fonts.iter().position(|f| f.key == font()).unwrap_or(1);
    let shown = swatch_for(&cur, device_dark);
    let pv_vars = format!("{};--fs:{}px", swatch_vars(&shown), fonts[fcur].px);
    let light = swatch_for("light", false);
    let dark = swatch_for("dark", true);
    let grid = format!(
        "grid-template-columns:repeat({},minmax(0,1fr))",
        if cols <= 2 { 2 } else { 4 }
    );
    let fl = fonts.clone();
    let pick_font = use_callback(move |i: usize| {
        let k = fl[i].key.to_string();
        said.set(format!("Font size {}.", fl[i].label));
        font.set(k.clone());
        open.set(false);
        on_change.call((theme.peek().clone(), k));
        focus(chip.peek().clone());
    });
    let is_open = open();
    let active = format!("{uid}-{}", hi());
    let font_label = fonts[fcur].label;
    let pv_hc = shown.key == "contrast";
    let shown_label = if cur == "system" {
        format!("System ({})", shown.label)
    } else {
        shown.label.to_string()
    };

    rsx! {
        div { class: "oxtsg", "data-theme": "{cur}", "data-font": "{font_label}",
            div { class: "oxtsg-sec",
                p { class: "oxtsg-h", "Theme" }
                div { class: "oxtsg-grid", role: "radiogroup", "aria-label": "Theme", style: "{grid}",
                    for s in sws.iter() {
                        {
                            let on = cur == s.key;
                            let key = s.key;
                            let label = s.label;
                            let sys = key == "system";
                            let hc = key == "contrast";
                            rsx! {
                                button { key: "{key}", class: "oxtsg-sw", r#type: "button", role: "radio", "aria-checked": "{on}",
                                    "aria-label": "{label}", title: "{label}", "data-hc": "{hc}",
                                    onclick: move |_| { theme.set(key.to_string()); said.set(format!("{label} theme.")); on_change.call((key.to_string(), font.peek().clone())); },
                                    span { class: "oxtsg-frame",
                                        if sys {
                                            {mini_window(&light, "oxtsg-win")}
                                            span { class: "oxtsg-dk", {mini_window(&dark, "oxtsg-win")} }
                                            span { class: "oxtsg-mon", {monitor_glyph()} }
                                        } else {
                                            {mini_window(s, "oxtsg-win")}
                                        }
                                        if on { span { class: "oxtsg-check", {check_glyph()} } }
                                    }
                                    span { class: "oxtsg-lab", "{label}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxtsg-row",
                p { class: "oxtsg-h", id: "{uid}-l", "Font size" }
                div { class: "oxtsg-dd",
                    if is_open { div { class: "oxtsg-catch", "aria-hidden": "true", onpointerdown: move |_| open.set(false) } }
                    button { class: "oxtsg-btn", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{is_open}",
                        "aria-labelledby": "{uid}-l {uid}-v",
                        onmounted: move |e| chip.set(Some(e.data())),
                        onclick: move |_| { if open() { open.set(false) } else { hi.set(fcur); want.set(true); open.set(true) } },
                        onkeydown: move |e: KeyboardEvent| match e.key() {
                            Key::ArrowUp | Key::ArrowDown if !open() => { e.prevent_default(); hi.set(fcur); want.set(true); open.set(true); }
                            _ => {}
                        },
                        span { id: "{uid}-v", "{font_label}" }
                        {chevron(is_open)}
                    }
                    if is_open {
                        ul { class: "oxtsg-list", role: "listbox", tabindex: "-1", "aria-labelledby": "{uid}-l", "aria-activedescendant": "{active}",
                            onmounted: move |e| { let d = e.data(); if *want.peek() { want.set(false); focus(Some(d)); } },
                            onkeydown: move |e: KeyboardEvent| {
                                let k = e.key();
                                if let Some(d) = menu_key(&k) { e.prevent_default(); hi.set(menu_move(hi(), d, nf)); }
                                else if k == Key::Enter || k == Key::Character(" ".into()) { e.prevent_default(); pick_font.call(hi()); }
                                else if k == Key::Escape || k == Key::Tab { if k == Key::Escape { e.prevent_default(); } open.set(false); focus(chip.peek().clone()); }
                            },
                            for (i, f) in fonts.iter().enumerate() {
                                {
                                    let fs = format!("font-size:{}px", f.px);
                                    let id = format!("{uid}-{i}");
                                    let sel = i == fcur;
                                    let h = i == hi();
                                    let lab = f.label;
                                    rsx! {
                                        li { key: "{i}", id: "{id}", class: "oxtsg-opt", role: "option", "aria-selected": "{sel}", "data-hi": "{h}",
                                            onpointerenter: move |_| hi.set(i),
                                            onclick: move |_| pick_font.call(i),
                                            span { style: "{fs}", "{lab}" }
                                            if sel { span { class: "oxtsg-ok", {check_glyph()} } }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if preview {
                div { class: "oxtsg-pv", style: "{pv_vars}", "data-hc": "{pv_hc}", role: "img",
                    "aria-label": "Sample in {shown_label}, {font_label} text",
                    div { class: "oxtsg-pv-side",
                        span { class: "oxtsg-pv-it", "data-on": "true", "Today" }
                        span { class: "oxtsg-pv-it", "Projects" }
                        span { class: "oxtsg-pv-it", "Archive" }
                    }
                    div { class: "oxtsg-pv-main",
                        span { class: "oxtsg-pv-t", "{shown_label}" }
                        span { class: "oxtsg-pv-m", "Three runs finished while you were away." }
                        span { class: "oxtsg-pv-b", "Review" }
                    }
                }
            }
            p { class: "oxtsg-sr", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// A sample app window in one theme (for Theme split).
fn split_window(s: &ThemeSwatch) -> Element {
    let vars = swatch_vars(s);
    let k = s.key;
    rsx! {
        div { class: "oxtpv-win", style: "{vars}", "data-sw": "{k}",
            div { class: "oxtpv-side",
                span { class: "oxtpv-dot" }
                span { class: "oxtpv-it", "data-on": "true", "Inbox" }
                span { class: "oxtpv-it", "Runs" }
                span { class: "oxtpv-it", "Files" }
            }
            div { class: "oxtpv-main",
                span { class: "oxtpv-t", "Release notes" }
                span { class: "oxtpv-m", "Draft ready for review. Two comments left." }
                span { class: "oxtpv-card", span { class: "oxtpv-bar" } span { class: "oxtpv-bar", "data-s": "true" } }
                span { class: "oxtpv-b", "Open draft" }
            }
        }
    }
}

/// Theme Selector – Theme split (Oxyn original): your theme and a candidate over one sample
/// window, a divider between them. Drag the divider (or use the arrow keys) to compare, pick the
/// candidate from the dots, then Use it – the two swap so you can compare back.
#[component]
pub fn ThemeSplit(
    /// The theme in use (left).
    #[props(default = "light".to_string())]
    default_current: String,
    /// The candidate (right).
    #[props(default = "midnight".to_string())]
    default_candidate: String,
    /// Divider position in % (8–92).
    #[props(default = 50.0)]
    default_split: f64,
    /// The candidate was put to use (its key).
    #[props(default)]
    on_apply: EventHandler<String>,
) -> Element {
    let mut a = use_signal(move || default_current);
    let mut b = use_signal(move || default_candidate);
    let mut x = use_signal(move || split_clamp(default_split));
    let mut note = use_signal(String::new);
    let sa = swatch_for(&a(), false);
    let sb = swatch_for(&b(), true);
    let pos = x();
    let clip = format!("clip-path:inset(0 0 0 {pos:.1}%)");
    let knob = format!("left:{pos:.1}%");
    let posr = format!("{pos:.0}");
    let la = sa.label;
    let lb = sb.label;
    let cands: Vec<ThemeSwatch> = swatches()
        .into_iter()
        .filter(|s| s.key != "system")
        .collect();
    let cur_a = sa.key;

    rsx! {
        div { class: "oxtpv", "data-a": "{cur_a}", "data-b": "{sb.key}",
            div { class: "oxtpv-head",
                p { class: "oxtpv-h", "Compare themes" }
                div { class: "oxtpv-dots", role: "radiogroup", "aria-label": "Candidate theme",
                    for s in cands.iter() {
                        {
                            let k = s.key;
                            let l = s.label;
                            let on = sb.key == k;
                            let mine = cur_a == k;
                            let st = format!("--dp:#{};--dl:#{}", s.pane, s.line);
                            rsx! {
                                button { key: "{k}", class: "oxtpv-pick", r#type: "button", role: "radio", "aria-checked": "{on}", disabled: mine,
                                    "aria-label": "{l}", title: "{l}", style: "{st}",
                                    onclick: move |_| { b.set(k.to_string()); note.set(format!("Comparing with {l}.")); }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxtpv-stage",
                div { class: "oxtpv-layer", {split_window(&sa)} }
                div { class: "oxtpv-layer", style: "{clip}", {split_window(&sb)} }
                span { class: "oxtpv-tag", "data-side": "a", "{la}" }
                span { class: "oxtpv-tag", "data-side": "b", "{lb}" }
                span { class: "oxtpv-rule", style: "{knob}", "aria-hidden": "true", span { class: "oxtpv-grip" } }
                input { class: "oxtpv-range", r#type: "range", min: "8", max: "92", step: "1", value: "{posr}",
                    "aria-label": "Divider between {la} and {lb}", "aria-valuetext": "{posr}% {la}",
                    oninput: move |e: FormEvent| { if let Ok(v) = e.value().parse::<f64>() { x.set(split_clamp(v)); } }
                }
            }
            div { class: "oxtpv-foot",
                button { class: "oxtpv-btn", r#type: "button",
                    onclick: move |_| { x.set(50.0); note.set("Divider centred.".into()); },
                    "Centre"
                }
                button { class: "oxtpv-btn", "data-main": "true", r#type: "button",
                    onclick: move |_| {
                        let (oa, ob) = (a.peek().clone(), b.peek().clone());
                        a.set(ob.clone());
                        b.set(oa);
                        let l = swatch_for(&ob, true).label;
                        note.set(format!("{l} is on. Drag to compare back."));
                        on_apply.call(ob);
                    },
                    "Use {lb}"
                }
            }
            p { class: "oxtpv-note", role: "status", "aria-live": "polite", "{note}" }
        }
    }
}

/// Theme Selector – Accent and density (Oxyn original): an accent, row spacing and corner shape
/// on the left; a settings sample on the right takes each at once – its toggles, selected row
/// and button wear the accent, rows close up or open out, corners square off or round.
#[component]
pub fn AccentDensity(
    /// Starting accent key.
    #[props(default = "blue".to_string())]
    default_accent: String,
    /// Starting density key (`compact`, `cozy`, `roomy`).
    #[props(default = "cozy".to_string())]
    default_density: String,
    /// Starting corner key (`square`, `soft`, `round`).
    #[props(default = "soft".to_string())]
    default_corner: String,
    /// Any choice changed: (accent, density, corner).
    #[props(default)]
    on_change: EventHandler<(String, String, String)>,
) -> Element {
    let mut acc = use_signal(move || default_accent);
    let mut den = use_signal(move || default_density);
    let mut cor = use_signal(move || default_corner);
    let mut toggles = use_signal(|| [true, false, true]);
    let mut row = use_signal(|| 0usize);
    let mut note = use_signal(String::new);
    let acs = accents();
    let ds = densities();
    let cs = corners();
    let a = acs
        .iter()
        .find(|x| x.key == acc())
        .cloned()
        .unwrap_or_else(|| acs[1].clone());
    let d = ds
        .iter()
        .find(|x| x.key == den())
        .cloned()
        .unwrap_or_else(|| ds[1].clone());
    let c = cs.iter().find(|x| x.0 == cor()).copied().unwrap_or(cs[1]);
    let vars = format!(
        "--ac:#{};--rh:{}px;--rp:{}px;--rg:{}px;--rr:{}px",
        a.hex, d.row, d.pad, d.gap, c.2
    );
    let tg = toggles();
    let fire = move || on_change.call((acc.peek().clone(), den.peek().clone(), cor.peek().clone()));
    let rows = [
        ("Notifications", "Ping when a run ends"),
        ("Sounds", "A soft click on send"),
        ("Auto-save", "Every thirty seconds"),
    ];
    let al = a.label;
    let dl = d.label;
    let cl = c.1;

    rsx! {
        div { class: "oxadn", "data-accent": "{al}", "data-density": "{dl}", "data-corner": "{cl}",
            div { class: "oxadn-ctl",
                div { class: "oxadn-g",
                    p { class: "oxadn-h", "Accent" }
                    div { class: "oxadn-acc", role: "radiogroup", "aria-label": "Accent",
                        for x in acs.iter() {
                            {
                                let k = x.key;
                                let l = x.label;
                                let on = a.key == k;
                                let st = format!("--sw:#{}", x.hex);
                                rsx! {
                                    button { key: "{k}", class: "oxadn-dot", r#type: "button", role: "radio", "aria-checked": "{on}", "aria-label": "{l}", title: "{l}", style: "{st}",
                                        onclick: move |_| { acc.set(k.to_string()); note.set(format!("{l} accent.")); fire(); }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "oxadn-g",
                    p { class: "oxadn-h", "Density" }
                    div { class: "oxadn-seg", role: "radiogroup", "aria-label": "Density",
                        for x in ds.iter() {
                            {
                                let k = x.key;
                                let l = x.label;
                                let on = d.key == k;
                                rsx! {
                                    button { key: "{k}", class: "oxadn-sb", r#type: "button", role: "radio", "aria-checked": "{on}",
                                        onclick: move |_| { den.set(k.to_string()); note.set(format!("{l} spacing.")); fire(); },
                                        "{l}"
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "oxadn-g",
                    p { class: "oxadn-h", "Corners" }
                    div { class: "oxadn-seg", role: "radiogroup", "aria-label": "Corners",
                        for x in cs.iter() {
                            {
                                let (k, l, px) = *x;
                                let on = c.0 == k;
                                let st = format!("--cr:{}px", px.min(6));
                                rsx! {
                                    button { key: "{k}", class: "oxadn-sb", r#type: "button", role: "radio", "aria-checked": "{on}",
                                        onclick: move |_| { cor.set(k.to_string()); note.set(format!("{l} corners.")); fire(); },
                                        span { class: "oxadn-cn", style: "{st}", "aria-hidden": "true" }
                                        "{l}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxadn-pv", style: "{vars}",
                p { class: "oxadn-pt", "General" }
                for (i, (t, s)) in rows.iter().enumerate() {
                    {
                        let on = tg[i];
                        let sel = row() == i;
                        let t = *t;
                        let s = *s;
                        rsx! {
                            div { key: "{i}", class: "oxadn-row", "data-sel": "{sel}", onclick: move |_| row.set(i),
                                span { class: "oxadn-rt", span { class: "oxadn-rn", "{t}" } span { class: "oxadn-rs", "{s}" } }
                                button { class: "oxadn-tg", r#type: "button", role: "switch", "aria-checked": "{on}", "aria-label": "{t}",
                                    onclick: move |e| { e.stop_propagation(); row.set(i); let mut v = toggles(); v[i] = !v[i]; toggles.set(v); note.set(format!("{t} {}.", if v[i] { "on" } else { "off" })); },
                                    span { class: "oxadn-kn" }
                                }
                            }
                        }
                    }
                }
                div { class: "oxadn-acts",
                    button { class: "oxadn-b", r#type: "button", onclick: move |_| note.set("Kept as it was.".into()), "Cancel" }
                    button { class: "oxadn-b", "data-main": "true", r#type: "button", onclick: move |_| note.set(format!("Saved: {al}, {dl}, {cl}.")), "Save" }
                }
            }
            p { class: "oxadn-note", role: "status", "aria-live": "polite", "{note}" }
        }
    }
}
