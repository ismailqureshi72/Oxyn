//! Theme Selector family (variants 3): how the app looks – [`BackgroundPicker`] (picture tiles we
//! draw ourselves, a tile for your own picture, and a texture laid over it), [`ThemeGrid`] (eight
//! mini-window swatches and a font size menu over a live sample), and four Oxyn originals:
//! [`ThemeSplit`] (two themes over one sample window with a divider you drag) and
//! [`AccentDensity`] (accent, spacing and corners on a settings sample), [`ThemeSchedule`] (light
//! and dark by the clock on a 24-hour strip) and [`TypeScale`] (family, size and spacing on a
//! reading sample), [`LightPlayground`] (drag a lamp; shadows and highlights follow), [`AppIconPicker`]
//! (alternate icons we draw, in Colour, Dark or Tinted) and [`DisplayComfort`] (contrast, motion,
//! transparency and bold text on a sample). Models: `oxyn_core::kit::themefid`. Styles:
//! `themefid.css` (`oxbgk-`, `oxtsg-`, `oxtpv-`, `oxadn-`, `oxtsc-`, `oxtys-`, `oxtlp-`, `oxaip-`,
//! `oxdcf-`).

use dioxus::prelude::*;
use oxyn_core::kit::effmenufid::menu_move;
use oxyn_core::kit::themefid::{
    SUN, TYPE_STEPS, Texture, ThemeSwatch, accents, backdrops, chars_per_line, corners, data_url,
    densities, fmt_hm, fmt_in, font_sizes, measure_word, next_switch, own_check, sched_light,
    sched_modes, split_clamp, swatch_for, swatches, texture_press, type_families, type_spacing,
};
use oxyn_core::kit::themefid::{
    app_icons, comfort_opts, comfort_said, icon_colors, icon_styles, lamp_angle, lamp_clamp,
    lamp_shadow,
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
                    path { d: "M0 8 C30 14 52 30 70 52", stroke_width: "2.4" }
                    path { d: "M28 15 C40 6 54 4 70 2", stroke_width: "1.4" }
                    path { d: "M52 30 C66 26 86 30 104 22", stroke_width: "1.3" }
                    path { d: "M160 4 C130 12 118 26 112 46", stroke_width: "2" }
                    path { d: "M128 14 C140 24 150 24 160 29", stroke_width: "1.2" }
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
                defs {
                    radialGradient { id: "{g}", gradient_units: "userSpaceOnUse", cx: "-6", cy: "136", r: "124",
                        stop { offset: "0.86", stop_color: "#0A2A5E" } stop { offset: "0.95", stop_color: "#3E8EE6" }
                        stop { offset: "0.985", stop_color: "#E9F4FF" } stop { offset: "1", stop_color: "#F2702C" } }
                    filter { id: "{g2}", x: "-10%", y: "-10%", width: "120%", height: "120%", feGaussianBlur { std_deviation: "1.6" } }
                }
                rect { width: "160", height: "90", fill: "#030305" }
                g { fill: "#FFFFFF", opacity: ".75",
                    circle { cx: "120", cy: "14", r: ".6" } circle { cx: "146", cy: "40", r: ".5" } circle { cx: "98", cy: "8", r: ".5" }
                    circle { cx: "150", cy: "70", r: ".6" } circle { cx: "132", cy: "28", r: ".4" }
                }
                path { d: "M0 9.1 A127 127 0 0 1 112.4 90", fill: "none", stroke: "#F2702C", stroke_width: "5", opacity: ".55", filter: "{gu2}" }
                path { d: "M0 12.1 A124 124 0 0 1 109.2 90 L0 90 Z", fill: "{gu}" }
                path { d: "M96 40 l5 -1 l2 -3 l1 3 l5 1 l-5 1 l-1 3 l-2 -3 z", fill: "#E9F4FF" }
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

fn sun_glyph(moon: bool) -> Element {
    let d = if moon {
        "M20 14.5A8 8 0 1 1 9.5 4a6.5 6.5 0 0 0 10.5 10.5z"
    } else {
        "M12 8a4 4 0 1 0 0 8 4 4 0 0 0 0-8zM12 2v2M12 20v2M2 12h2M20 12h2M4.9 4.9l1.4 1.4M17.7 17.7l1.4 1.4M4.9 19.1l1.4-1.4M17.7 6.3l1.4-1.4"
    };
    rsx! {
        svg { width: "14", height: "14", view_box: "0 0 24 24", fill: "none", stroke: "currentColor", stroke_width: "1.9",
            stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d: "{d}" } }
    }
}

/// Theme Selector – Theme schedule (Oxyn original): Light, Dark, Sun (follows sunrise and sunset)
/// or Custom hours. A 24-hour strip shows the light window; drag along it to preview any time of
/// day, and the sample window turns light or dark with it. In Custom, two sliders set the hours.
#[component]
pub fn ThemeSchedule(
    /// Starting mode (`light`, `dark`, `sun`, `custom`).
    #[props(default = "custom".to_string())]
    default_mode: String,
    /// Light from (minutes after midnight).
    #[props(default = 420)]
    default_from: u32,
    /// Dark from (minutes after midnight).
    #[props(default = 1140)]
    default_to: u32,
    /// The previewed time (minutes after midnight).
    #[props(default = 1080)]
    default_at: u32,
    /// Mode or hours changed: (mode, from, to).
    #[props(default)]
    on_change: EventHandler<(String, u32, u32)>,
) -> Element {
    let mut mode = use_signal(move || default_mode);
    let mut from = use_signal(move || default_from.min(1425));
    let mut to = use_signal(move || default_to.min(1425));
    let mut at = use_signal(move || default_at.min(1439));
    let m = mode();
    let (f, t) = if m == "sun" { SUN } else { (from(), to()) };
    let now = at();
    let light = sched_light(&m, now, f, t);
    let sw = if light {
        swatch_for("light", false)
    } else {
        swatch_for("dark", true)
    };
    let vars = swatch_vars(&sw);
    let window = m == "sun" || m == "custom";
    let line = match m.as_str() {
        "light" => "Always light.".to_string(),
        "dark" => "Always dark.".to_string(),
        _ => {
            let nx = next_switch(now, f, t).map(fmt_in).unwrap_or_default();
            let what = if light { "dark" } else { "light" };
            let head = if m == "sun" {
                "Follows the sun"
            } else {
                "Your hours"
            };
            format!(
                "{head}: light {}–{}. Turns {what} {nx}.",
                fmt_hm(f),
                fmt_hm(t)
            )
        }
    };
    let pct = |x: u32| x as f64 / 1440.0 * 100.0;
    // The light band, split in two when it wraps past midnight.
    let bands: Vec<String> = if !window {
        if m == "light" {
            vec!["left:0%;width:100%".to_string()]
        } else {
            vec![]
        }
    } else if f <= t {
        vec![format!("left:{:.2}%;width:{:.2}%", pct(f), pct(t - f))]
    } else {
        vec![
            format!("left:0%;width:{:.2}%", pct(t)),
            format!("left:{:.2}%;width:{:.2}%", pct(f), 100.0 - pct(f)),
        ]
    };
    let now_st = format!("left:{:.2}%", pct(now));
    let now_s = fmt_hm(now);
    let fs = fmt_hm(from());
    let ts = fmt_hm(to());
    let fv = format!("{}", from());
    let tv = format!("{}", to());
    let av = format!("{now}");
    let custom = m == "custom";
    let off = !custom;
    let late = now > 1260;
    let theme_word = if light { "Light" } else { "Dark" };
    let fire = move || on_change.call((mode.peek().clone(), *from.peek(), *to.peek()));

    rsx! {
        div { class: "oxtsc", "data-mode": "{m}", "data-light": "{light}", "data-late": "{late}",
            div { class: "oxtsc-head",
                p { class: "oxtsc-h", "Light and dark" }
                div { class: "oxtsc-seg", role: "radiogroup", "aria-label": "Schedule",
                    for (k, l) in sched_modes() {
                        {
                            let on = m == k;
                            rsx! {
                                button { key: "{k}", class: "oxtsc-sb", r#type: "button", role: "radio", "aria-checked": "{on}",
                                    onclick: move |_| { mode.set(k.to_string()); fire(); },
                                    "{l}"
                                }
                            }
                        }
                    }
                }
            }
            p { class: "oxtsc-line", role: "status", "aria-live": "polite", "{line}" }
            div { class: "oxtsc-strip",
                for (i, b) in bands.iter().enumerate() {
                    span { key: "{i}", class: "oxtsc-band", style: "{b}" }
                }
                for h in 1..24u32 {
                    {
                        let st = format!("left:{:.3}%", h as f64 / 24.0 * 100.0);
                        let major = h % 6 == 0;
                        rsx! { i { key: "{h}", class: "oxtsc-tick", "data-major": "{major}", style: "{st}" } }
                    }
                }
                span { class: "oxtsc-now", style: "{now_st}", "aria-hidden": "true", span { class: "oxtsc-nl", "{now_s}" } }
                input { class: "oxtsc-range", r#type: "range", min: "0", max: "1439", step: "15", value: "{av}",
                    "aria-label": "Preview time", "aria-valuetext": "{now_s}, {theme_word}",
                    oninput: move |e: FormEvent| { if let Ok(v) = e.value().parse::<u32>() { at.set(v.min(1439)); } }
                }
            }
            div { class: "oxtsc-hours", "aria-hidden": "true",
                span { "00" } span { "06" } span { "12" } span { "18" } span { "24" }
            }
            div { class: "oxtsc-body",
                div { class: "oxtsc-pv", style: "{vars}", role: "img", "aria-label": "{theme_word} at {now_s}",
                    div { class: "oxtsc-pv-bar", span { class: "oxtsc-pv-ic", {sun_glyph(!light)} } span { "{now_s}" } }
                    div { class: "oxtsc-pv-main",
                        span { class: "oxtsc-pv-t", "{theme_word}" }
                        span { class: "oxtsc-pv-l" } span { class: "oxtsc-pv-l", "data-s": "true" }
                    }
                }
                div { class: "oxtsc-ctl", "data-off": "{off}",
                    label { class: "oxtsc-row",
                        span { class: "oxtsc-rl", {sun_glyph(false)} "Light from" }
                        input { class: "oxtsc-sl", r#type: "range", min: "0", max: "1425", step: "15", value: "{fv}", disabled: !custom,
                            "aria-valuetext": "{fs}",
                            oninput: move |e: FormEvent| { if let Ok(v) = e.value().parse::<u32>() { from.set(v); fire(); } }
                        }
                        span { class: "oxtsc-rv", "{fs}" }
                    }
                    label { class: "oxtsc-row",
                        span { class: "oxtsc-rl", {sun_glyph(true)} "Dark from" }
                        input { class: "oxtsc-sl", r#type: "range", min: "0", max: "1425", step: "15", value: "{tv}", disabled: !custom,
                            "aria-valuetext": "{ts}",
                            oninput: move |e: FormEvent| { if let Ok(v) = e.value().parse::<u32>() { to.set(v); fire(); } }
                        }
                        span { class: "oxtsc-rv", "{ts}" }
                    }
                }
            }
        }
    }
}

const READ_TEXT: &str = "Evening reading should feel easy. Pick a family, nudge the size until a line sits \
    comfortably under your eye, and give the lines a little room to breathe.";

/// Theme Selector – Type scale (Oxyn original): Sans, Serif or Mono; A− and A+ around a stepped
/// size slider; Tight, Normal or Relaxed lines. The reading sample takes each at once and says
/// about how many characters fit a line.
#[component]
pub fn TypeScale(
    /// Starting family (`sans`, `serif`, `mono`).
    #[props(default = "sans".to_string())]
    default_family: String,
    /// Starting size step (0–5: 13, 14, 15, 16, 18, 20 px).
    #[props(default = 3)]
    default_step: usize,
    /// Starting spacing (`tight`, `normal`, `relaxed`).
    #[props(default = "normal".to_string())]
    default_spacing: String,
    /// Width the line count is worked out for, in px.
    #[props(default = 520.0)]
    measure_width: f64,
    /// Any choice changed: (family, px, line height).
    #[props(default)]
    on_change: EventHandler<(String, u32, f64)>,
) -> Element {
    let mut fam = use_signal(move || default_family);
    let mut step = use_signal(move || default_step.min(TYPE_STEPS.len() - 1));
    let mut sp = use_signal(move || default_spacing);
    let fams = type_families();
    let sps = type_spacing();
    let fk = fams
        .iter()
        .find(|x| x.0 == fam())
        .map(|x| x.0)
        .unwrap_or("sans");
    let lh = sps.iter().find(|x| x.0 == sp()).map(|x| x.2).unwrap_or(1.5);
    let last = TYPE_STEPS.len() - 1;
    let i = step().min(last);
    let px = TYPE_STEPS[i];
    let cpl = chars_per_line(measure_width, px, fk);
    let word = measure_word(cpl);
    let vars = format!("--tsz:{px}px;--tlh:{lh}");
    let read = format!("{px} px · {lh} lines · about {cpl} characters a line, {word}");
    let iv = format!("{i}");
    let lastv = format!("{last}");
    let pxv = format!("{px} px");
    let fire = move || {
        let f = fam.peek().clone();
        let px = TYPE_STEPS[(*step.peek()).min(TYPE_STEPS.len() - 1)];
        let lh = type_spacing()
            .iter()
            .find(|x| x.0 == *sp.peek())
            .map(|x| x.2)
            .unwrap_or(1.5);
        on_change.call((f, px, lh));
    };

    rsx! {
        div { class: "oxtys", "data-family": "{fk}", "data-px": "{px}",
            div { class: "oxtys-ctl",
                div { class: "oxtys-seg", role: "radiogroup", "aria-label": "Family",
                    for (k, l, _) in fams.iter().copied() {
                        {
                            let on = fk == k;
                            rsx! {
                                button { key: "{k}", class: "oxtys-sb", "data-f": "{k}", r#type: "button", role: "radio", "aria-checked": "{on}",
                                    onclick: move |_| { fam.set(k.to_string()); fire(); },
                                    "{l}"
                                }
                            }
                        }
                    }
                }
                div { class: "oxtys-size",
                    button { class: "oxtys-ab", r#type: "button", "aria-label": "Smaller text", disabled: i == 0,
                        onclick: move |_| { let v = step().saturating_sub(1); step.set(v); fire(); },
                        span { class: "oxtys-a", "data-s": "true", "A" }
                    }
                    input { class: "oxtys-range", r#type: "range", min: "0", max: "{lastv}", step: "1", value: "{iv}",
                        "aria-label": "Text size", "aria-valuetext": "{pxv}",
                        oninput: move |e: FormEvent| { if let Ok(v) = e.value().parse::<usize>() { step.set(v.min(TYPE_STEPS.len() - 1)); fire(); } }
                    }
                    button { class: "oxtys-ab", r#type: "button", "aria-label": "Larger text", disabled: i == last,
                        onclick: move |_| { let v = (step() + 1).min(TYPE_STEPS.len() - 1); step.set(v); fire(); },
                        span { class: "oxtys-a", "A" }
                    }
                    span { class: "oxtys-px", "{pxv}" }
                }
                div { class: "oxtys-seg", role: "radiogroup", "aria-label": "Line spacing",
                    for (k, l, _) in sps.iter().copied() {
                        {
                            let on = sp() == k;
                            rsx! {
                                button { key: "{k}", class: "oxtys-sb", r#type: "button", role: "radio", "aria-checked": "{on}",
                                    onclick: move |_| { sp.set(k.to_string()); fire(); },
                                    "{l}"
                                }
                            }
                        }
                    }
                }
            }
            article { class: "oxtys-pv", "data-f": "{fk}", style: "{vars}",
                h4 { class: "oxtys-t", "After hours" }
                p { class: "oxtys-p", {READ_TEXT} }
            }
            p { class: "oxtys-read", role: "status", "aria-live": "polite", "{read}" }
        }
    }
}

/// Theme Selector – Light playground (Oxyn original): drag the lamp over a small interface and its
/// cards cast shadows away from it, softer the further they are, with a highlight on the side that
/// faces the light. Strength sets how far shadows reach; Glow swaps shadows for a lit rim.
#[component]
pub fn LightPlayground(
    /// Lamp position, % across.
    #[props(default = 26.0)]
    default_x: f64,
    /// Lamp position, % down.
    #[props(default = 18.0)]
    default_y: f64,
    /// Shadow strength, 0–100.
    #[props(default = 60.0)]
    default_strength: f64,
    /// `shadow` or `glow`.
    #[props(default = "shadow".to_string())]
    default_mode: String,
    /// The lamp moved: (x %, y %).
    #[props(default)]
    on_move: EventHandler<(f64, f64)>,
) -> Element {
    let mut lx = use_signal(move || lamp_clamp(default_x));
    let mut ly = use_signal(move || lamp_clamp(default_y));
    let mut k = use_signal(move || default_strength.clamp(0.0, 100.0));
    let mut mode = use_signal(move || default_mode);
    let mut held = use_signal(|| false);
    let mut size = use_signal(|| (520.0f64, 240.0f64));
    let mut stage = use_signal(|| None::<std::rc::Rc<MountedData>>);
    let mut put = move |x: f64, y: f64| {
        let (w, h) = *size.peek();
        let nx = lamp_clamp(x / w.max(1.0) * 100.0);
        let ny = lamp_clamp(y / h.max(1.0) * 100.0);
        lx.set(nx);
        ly.set(ny);
        on_move.call((nx, ny));
    };
    let (x, y) = (lx(), ly());
    let st = k();
    let m = mode();
    // Card centres in % of the stage: the sample card, the button and the chip.
    let cards = [
        ("card", 34.0, 58.0),
        ("btn", 76.0, 40.0),
        ("chip", 74.0, 78.0),
    ];
    let styles: Vec<String> = cards
        .iter()
        .map(|(_, cx, cy)| {
            let (dx, dy, b) = lamp_shadow(x, y, *cx, *cy, st);
            let a = lamp_angle(x, y, *cx, *cy);
            format!(
                "--dx:{dx:.1}px;--dy:{dy:.1}px;--bl:{b:.1}px;--ang:{a:.0}deg;--al:{:.2}",
                0.12 + st / 100.0 * 0.3
            )
        })
        .collect();
    let (s0, s1, s2) = (styles[0].clone(), styles[1].clone(), styles[2].clone());
    let lamp = format!("left:{x:.1}%;top:{y:.1}%");
    let pool = format!("--lx:{x:.1}%;--ly:{y:.1}%");
    let said = format!("Lamp at {x:.0}% across, {y:.0}% down");
    let kv = format!("{st:.0}");
    let glow = m == "glow";

    rsx! {
        div { class: "oxtlp", "data-mode": "{m}",
            div { class: "oxtlp-head",
                p { class: "oxtlp-h", "Light playground" }
                div { class: "oxtsc-seg", role: "radiogroup", "aria-label": "Light",
                    for (key, lab) in [("shadow", "Shadow"), ("glow", "Glow")] {
                        {
                            let on = m == key;
                            rsx! {
                                button { key: "{key}", class: "oxtsc-sb", r#type: "button", role: "radio", "aria-checked": "{on}",
                                    onclick: move |_| mode.set(key.to_string()), "{lab}" }
                            }
                        }
                    }
                }
            }
            div { class: "oxtlp-stage", style: "{pool}", "data-held": "{held}",
                onmounted: move |e: MountedEvent| async move {
                    let d = e.data();
                    if let Ok(rc) = d.get_client_rect().await { size.set((rc.size.width, rc.size.height)); }
                    stage.set(Some(d));
                },
                div { class: "oxtlp-card", "data-glow": "{glow}", style: "{s0}",
                    span { class: "oxtlp-ct", "Morning run" }
                    span { class: "oxtlp-cm", "Four checks passed, one waits for you." }
                    span { class: "oxtlp-cb" } span { class: "oxtlp-cb", "data-s": "true" }
                }
                span { class: "oxtlp-btn", "data-glow": "{glow}", style: "{s1}", "Review" }
                span { class: "oxtlp-chip", "data-glow": "{glow}", style: "{s2}", "3 new" }
                div { class: "oxtlp-hit",
                    onpointerdown: move |e: PointerEvent| {
                        held.set(true);
                        let p = e.element_coordinates();
                        put(p.x, p.y);
                        if let Some(d) = stage.peek().clone() {
                            spawn(async move { if let Ok(rc) = d.get_client_rect().await { size.set((rc.size.width, rc.size.height)); } });
                        }
                    },
                    onpointermove: move |e: PointerEvent| { if *held.peek() { let p = e.element_coordinates(); put(p.x, p.y); } },
                    onpointerup: move |_| held.set(false),
                    onpointerleave: move |_| held.set(false),
                }
                button { class: "oxtlp-lamp", r#type: "button", style: "{lamp}", "aria-label": "Lamp. Arrow keys move it.",
                    onkeydown: move |e: KeyboardEvent| {
                        let (dx, dy) = match e.key() {
                            Key::ArrowLeft => (-4.0, 0.0),
                            Key::ArrowRight => (4.0, 0.0),
                            Key::ArrowUp => (0.0, -4.0),
                            Key::ArrowDown => (0.0, 4.0),
                            _ => return,
                        };
                        e.prevent_default();
                        let (nx, ny) = (lamp_clamp(lx() + dx), lamp_clamp(ly() + dy));
                        lx.set(nx);
                        ly.set(ny);
                        on_move.call((nx, ny));
                    },
                    {sun_glyph(false)}
                }
            }
            label { class: "oxtlp-row",
                span { "Strength" }
                input { class: "oxtsc-sl", r#type: "range", min: "0", max: "100", step: "5", value: "{kv}",
                    oninput: move |e: FormEvent| { if let Ok(v) = e.value().parse::<f64>() { k.set(v.clamp(0.0, 100.0)); } }
                }
                span { class: "oxtsc-rv", "{kv}" }
            }
            p { class: "oxtlp-said", role: "status", "aria-live": "polite", "{said}" }
        }
    }
}

/// One of our icons: a rounded square ground with a ring-and-dot mark.
fn icon_art(from: &str, to: &str, mark: &str, gid: &str) -> Element {
    let gu = format!("url(#{gid})");
    rsx! {
        svg { class: "oxaip-svg", view_box: "0 0 64 64", "aria-hidden": "true",
            defs { linearGradient { id: "{gid}", x1: "0", y1: "0", x2: "1", y2: "1",
                stop { offset: "0", stop_color: "{from}" } stop { offset: "1", stop_color: "{to}" } } }
            rect { width: "64", height: "64", rx: "15", fill: "{gu}" }
            circle { cx: "32", cy: "32", r: "13", fill: "none", stroke: "{mark}", stroke_width: "4.5" }
            circle { cx: "43", cy: "21", r: "4.2", fill: "{mark}" }
        }
    }
}

fn other_app(i: usize) -> Element {
    let d = match i {
        0 => "M7 7h10v10H7z",
        1 => "M5 12h14M12 5v14",
        _ => "M6 16l4-8 4 6 2-3 2 5",
    };
    rsx! {
        svg { class: "oxaip-svg", view_box: "0 0 24 24", "aria-hidden": "true",
            rect { width: "24", height: "24", rx: "5.6", class: "oxaip-og" }
            path { d: "{d}", fill: "none", class: "oxaip-om", stroke_width: "1.8", stroke_linecap: "round", stroke_linejoin: "round" }
        }
    }
}

/// Theme Selector – App icon (Oxyn original): six alternate icons we draw (a ring-and-dot mark on
/// different grounds) and three styles – Colour, Dark (the icon's colours on near-black) and Tinted
/// (one hue from a slider). The home row below shows the pick among other apps.
#[component]
pub fn AppIconPicker(
    /// Starting icon key.
    #[props(default = "classic".to_string())]
    default_icon: String,
    /// Starting style (`color`, `dark`, `tinted`).
    #[props(default = "color".to_string())]
    default_style: String,
    /// Starting tint hue (0–359) for Tinted.
    #[props(default = 210)]
    default_tint: u32,
    /// The icon or style changed: (icon, style).
    #[props(default)]
    on_change: EventHandler<(String, String)>,
) -> Element {
    let mut pick = use_signal(move || default_icon);
    let mut style = use_signal(move || default_style);
    let mut tint = use_signal(move || default_tint % 360);
    let uid = use_uid("oxaip");
    let icons = app_icons();
    let cur = pick();
    let stl = style();
    let tv = tint();
    let chosen = icons
        .iter()
        .find(|i| i.key == cur)
        .cloned()
        .unwrap_or_else(|| icons[0].clone());
    let (cf, ct, cm) = icon_colors(&chosen, &stl, tv);
    let tinted = stl == "tinted";
    let tint_off = !tinted;
    let tvs = format!("{tv}");
    let hue = format!("--hue:{tv}");
    let label = chosen.label;
    let gid_home = format!("{uid}-home");

    rsx! {
        div { class: "oxaip", "data-icon": "{cur}", "data-style": "{stl}",
            div { class: "oxaip-head",
                p { class: "oxaip-h", "App icon" }
                div { class: "oxtsc-seg", role: "radiogroup", "aria-label": "Icon style",
                    for (k, l) in icon_styles() {
                        {
                            let on = stl == k;
                            rsx! {
                                button { key: "{k}", class: "oxtsc-sb", r#type: "button", role: "radio", "aria-checked": "{on}",
                                    onclick: move |_| { style.set(k.to_string()); on_change.call((pick.peek().clone(), k.to_string())); }, "{l}" }
                            }
                        }
                    }
                }
            }
            div { class: "oxaip-grid", role: "radiogroup", "aria-label": "App icon",
                for ic in icons.iter() {
                    {
                        let (f, t, mk) = icon_colors(ic, &stl, tv);
                        let k = ic.key;
                        let l = ic.label;
                        let on = cur == k;
                        let gid = format!("{uid}-{k}");
                        rsx! {
                            button { key: "{k}", class: "oxaip-tile", r#type: "button", role: "radio", "aria-checked": "{on}",
                                onclick: move |_| { pick.set(k.to_string()); on_change.call((k.to_string(), style.peek().clone())); },
                                span { class: "oxaip-ic", {icon_art(&f, &t, &mk, &gid)} }
                                span { class: "oxaip-l", "{l}" }
                            }
                        }
                    }
                }
            }
            label { class: "oxaip-tint", "data-off": "{tint_off}", style: "{hue}",
                span { "Tint" }
                input { class: "oxaip-hue", r#type: "range", min: "0", max: "359", step: "1", value: "{tvs}", disabled: !tinted,
                    "aria-label": "Tint hue",
                    oninput: move |e: FormEvent| { if let Ok(v) = e.value().parse::<u32>() { tint.set(v % 360); } }
                }
            }
            div { class: "oxaip-home", "data-style": "{stl}", role: "img", "aria-label": "Home row with the {label} icon",
                for i in 0..3usize {
                    span { key: "{i}", class: "oxaip-app", span { class: "oxaip-ic", {other_app(i)} } span { class: "oxaip-al", "App" } }
                }
                span { class: "oxaip-app", "data-me": "true",
                    span { class: "oxaip-ic", {icon_art(&cf, &ct, &cm, &gid_home)} }
                    span { class: "oxaip-al", "Oxyn" }
                }
            }
        }
    }
}

/// Theme Selector – Display comfort (Oxyn original): Increase contrast, Reduce motion, Reduce
/// transparency and Bold text as switches beside a sample that takes each at once – firmer edges,
/// a loop that stops, glass that turns solid, heavier type.
#[component]
pub fn DisplayComfort(
    /// Starting switches, in `comfort_opts()` order.
    #[props(default = vec![false, false, false, false])]
    default_on: Vec<bool>,
    /// A switch changed: (key, on).
    #[props(default)]
    on_change: EventHandler<(String, bool)>,
) -> Element {
    let opts = comfort_opts();
    let n = opts.len();
    let mut on = use_signal(move || {
        let mut v = default_on;
        v.resize(n, false);
        v
    });
    let cur = on();
    let said = comfort_said(&cur);
    let (f0, f1, f2, f3) = (cur[0], cur[1], cur[2], cur[3]);

    rsx! {
        div { class: "oxdcf",
            div { class: "oxdcf-list",
                div { class: "oxdcf-top", p { class: "oxdcf-h", "Comfort" } span { class: "oxdcf-n", "{said}" } }
                for (i, (k, l, d)) in opts.iter().copied().enumerate() {
                    {
                        let v = cur[i];
                        rsx! {
                            div { key: "{k}", class: "oxdcf-row",
                                span { class: "oxdcf-rt", span { class: "oxdcf-rn", "{l}" } span { class: "oxdcf-rd", "{d}" } }
                                button { class: "oxadn-tg oxdcf-tg", r#type: "button", role: "switch", "aria-checked": "{v}", "aria-label": "{l}",
                                    onclick: move |_| { let mut x = on(); x[i] = !x[i]; let nv = x[i]; on.set(x); on_change.call((k.to_string(), nv)); },
                                    span { class: "oxadn-kn" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxdcf-pv", "data-contrast": "{f0}", "data-still": "{f1}", "data-solid": "{f2}", "data-bold": "{f3}",
                role: "img", "aria-label": "Sample: {said}",
                div { class: "oxdcf-bg", span {} span {} }
                div { class: "oxdcf-glass",
                    span { class: "oxdcf-gt", "Deploy preview" }
                    span { class: "oxdcf-gm", "Building the site, step 3 of 5." }
                    span { class: "oxdcf-bar", span { class: "oxdcf-fill" } }
                    span { class: "oxdcf-row2", span { class: "oxdcf-spin" } span { class: "oxdcf-gm", "Uploading assets" } }
                }
            }
        }
    }
}
