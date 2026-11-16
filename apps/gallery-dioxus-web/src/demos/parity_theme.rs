//! Theme Selector family (variants 3) demos: how the app looks – background pictures and texture,
//! a theme grid with font size, two themes split over one window, and accent with density.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{AccentDensity, BackgroundPicker, ThemeGrid, ThemeSplit};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

const CAT: &str = "theme-selector";

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

fn low(s: &str) -> String {
    s.to_lowercase()
}

// ---------- Background picker ----------

fn background_picker(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("pick"),
        v.s("texture"),
        v.s("preview"),
        v.s("cols")
    );
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:560px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            BackgroundPicker { default_pick: low(v.s("pick")), default_texture: low(v.s("texture")), preview: v.b("preview"), cols: v.u("cols"),
                on_change: move |(b, t): (String, String)| msg.set(format!("onChange({b}, {t})")) }
            {status(msg)}
        }
    }
}

fn background_picker_code(v: &Vals) -> String {
    Code::new("BackgroundPicker")
        .s("default_pick", &low(v.s("pick")), "default")
        .s("default_texture", &low(v.s("texture")), "none")
        .b("preview", v.b("preview"))
        .raw("on_change", "move |(bg, texture)| save(bg, texture)")
        .done()
}

fn background_picker_demo() -> Demo {
    Demo {
        controls: vec![
            sel(
                "pick",
                "Background",
                &["Default", "Canopy", "Orbit", "Neon", "Tide"],
                "Default",
            ),
            sel(
                "texture",
                "Texture",
                &["None", "Film", "Static", "Dither"],
                "None",
            ),
            tog("preview", "Home preview", true),
            rng("cols", "Columns", 2.0, 4.0, 1.0, "3"),
        ],
        variants: vec![
            var(
                "default",
                "Plain default picked: a home screen preview over six tiles, the last one for your own picture.",
                &[],
            ),
            var(
                "orbit with film",
                "A drawn planet's edge behind the home screen with film grain laid over it.",
                &[("pick", "Orbit"), ("texture", "Film")],
            ),
            var(
                "neon dithered",
                "Out-of-focus lights under an ordered dot texture.",
                &[("pick", "Neon"), ("texture", "Dither")],
            ),
            var(
                "tiles only",
                "No preview, two columns: the picker as it sits in a narrow settings pane.",
                &[
                    ("preview", "false"),
                    ("cols", "2"),
                    ("pick", "Tide"),
                    ("texture", "Static"),
                ],
            ),
        ],
        render: background_picker,
        code: background_picker_code,
        keyboard: vec![
            ("Tab", "Each tile, Your own, then each texture"),
            ("Enter / Space on a tile", "Makes it the background"),
            ("Enter / Space on Your own", "Opens the file picker"),
            ("Enter / Space on the lit texture", "Takes the texture off"),
        ],
        a11y: vec![
            "Tiles are a radio group; each picture is named and described in a few words.",
            "Your own is a real file input (images up to 8 MB); a wrong file says why in the live note.",
            "Each texture reports its state through aria-pressed; the preview is an image labelled with the picture and texture.",
        ],
        responsive: vec![
            "560 px card; tiles drop to two columns and the texture row stacks under 480 px.",
        ],
        height: 560,
        live: false,
        align: Align::Center,
        ..mk(
            "background-picker",
            "Background picker",
            CAT,
            "Picture tiles we draw ourselves plus Your own, with Film, Static or Dither laid over the picture and a home screen preview.",
            "BackgroundPicker",
            "BackgroundPickerEl",
            "background-picker",
            "parity/themefid.rs",
        )
    }
}

// ---------- Theme grid ----------

fn theme_grid(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}{}",
        v.s("theme"),
        v.s("font"),
        v.s("device"),
        v.s("preview"),
        v.s("cols")
    );
    let cols = if v.s("cols") == "2" { 2 } else { 4 };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:600px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            ThemeGrid { default_theme: low(v.s("theme")), default_font: low(v.s("font")), device_dark: v.s("device") == "Dark",
                preview: v.b("preview"), cols: cols,
                on_change: move |(t, f): (String, String)| msg.set(format!("onChange({t}, {f})")) }
            {status(msg)}
        }
    }
}

fn theme_grid_code(v: &Vals) -> String {
    Code::new("ThemeGrid")
        .s("default_theme", &low(v.s("theme")), "system")
        .s("default_font", &low(v.s("font")), "default")
        .raw("device_dark", "prefers_dark()")
        .raw("on_change", "move |(theme, font)| save(theme, font)")
        .done()
}

fn theme_grid_demo() -> Demo {
    Demo {
        controls: vec![
            sel(
                "theme",
                "Theme",
                &[
                    "System", "Light", "Dark", "Midnight", "Plum", "Moss", "Harbor", "Contrast",
                ],
                "System",
            ),
            sel(
                "font",
                "Font size",
                &["Small", "Default", "Large", "Larger"],
                "Default",
            ),
            sel("device", "Device", &["Dark", "Light"], "Dark"),
            tog("preview", "Live sample", true),
            sel("cols", "Columns", &["4", "2"], "4"),
        ],
        variants: vec![
            var(
                "default",
                "System picked – the swatch split corner to corner – with the sample following a dark device.",
                &[],
            ),
            var(
                "contrast",
                "Contrast: black panes outlined in white, and larger text in the sample.",
                &[("theme", "Contrast"), ("font", "Large")],
            ),
            var(
                "light device",
                "System on a light device, small text: the sample turns light.",
                &[("device", "Light"), ("font", "Small")],
            ),
            var(
                "two columns",
                "Harbor picked in a two-column grid without the sample.",
                &[("theme", "Harbor"), ("cols", "2"), ("preview", "false")],
            ),
        ],
        render: theme_grid,
        code: theme_grid_code,
        keyboard: vec![
            ("Tab", "Each swatch, then the font size button"),
            ("Enter / Space on a swatch", "Picks the theme"),
            ("Up / Down on the button", "Opens the size list"),
            ("Up / Down, Home / End", "Move through sizes"),
            ("Enter / Escape", "Pick / close back to the button"),
        ],
        a11y: vec![
            "Swatches are a radio group named by theme; the check is decoration on top of aria-checked.",
            "Font size opens a listbox from its trigger; aria-activedescendant follows the highlighted size, and each size is drawn at its own scale.",
            "The sample is an image labelled with the theme and size it shows.",
        ],
        responsive: vec![
            "600 px card; swatches drop to two columns and the sample's sidebar folds into a row under 480 px.",
        ],
        height: 600,
        live: false,
        align: Align::Center,
        ..mk(
            "theme-grid",
            "Theme grid",
            CAT,
            "Eight mini-window swatches – System split corner to corner, Light, Dark, four tinted darks and Contrast – with a Font size menu and a live sample.",
            "ThemeGrid",
            "ThemeGridEl",
            "theme-grid",
            "parity/themefid.rs",
        )
    }
}

// ---------- Theme split ----------

fn theme_split(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("current"), v.s("candidate"), v.s("split"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:560px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            ThemeSplit { default_current: low(v.s("current")), default_candidate: low(v.s("candidate")), default_split: v.f("split"),
                on_apply: move |k: String| msg.set(format!("onApply({k})")) }
            {status(msg)}
        }
    }
}

fn theme_split_code(v: &Vals) -> String {
    Code::new("ThemeSplit")
        .s("default_current", &low(v.s("current")), "light")
        .s("default_candidate", &low(v.s("candidate")), "midnight")
        .raw("on_apply", "move |theme| save(theme)")
        .done()
}

fn theme_split_demo() -> Demo {
    let themes = &[
        "Light", "Dark", "Midnight", "Plum", "Moss", "Harbor", "Contrast",
    ];
    Demo {
        controls: vec![
            sel("current", "In use", themes, "Light"),
            sel("candidate", "Candidate", themes, "Midnight"),
            rng("split", "Divider %", 8.0, 92.0, 1.0, "50"),
        ],
        variants: vec![
            var(
                "default",
                "Light against Midnight, divider in the middle.",
                &[],
            ),
            var(
                "dark vs contrast",
                "Dark against Contrast with the divider pulled left.",
                &[
                    ("current", "Dark"),
                    ("candidate", "Contrast"),
                    ("split", "30"),
                ],
            ),
            var(
                "moss vs plum",
                "Two tinted darks side by side.",
                &[("current", "Moss"), ("candidate", "Plum"), ("split", "62")],
            ),
        ],
        render: theme_split,
        code: theme_split_code,
        keyboard: vec![
            ("Tab", "Candidate dots, the divider, Centre, Use"),
            ("Left / Right on the divider", "Move it one step"),
            ("Home / End", "Divider to either edge"),
            ("Enter on Use", "Swaps: the candidate is in use"),
        ],
        a11y: vec![
            "The divider is a native range input labelled with both themes, so drag, keys and screen readers share one control.",
            "Candidates are a radio group; the theme in use is disabled there.",
            "Use and Centre say what happened in a polite live note.",
        ],
        responsive: vec!["560 px card; the sample's sidebar narrows under 480 px."],
        height: 440,
        live: false,
        align: Align::Center,
        ..mk(
            "theme-split",
            "Theme split",
            CAT,
            "Oxyn original: the theme in use and a candidate over one sample window, a divider you drag between them, then Use to swap.",
            "ThemeSplit",
            "ThemeSplitEl",
            "theme-split",
            "parity/themefid.rs",
        )
    }
}

// ---------- Accent and density ----------

fn accent_density(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("accent"), v.s("density"), v.s("corner"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:600px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            AccentDensity { default_accent: low(v.s("accent")), default_density: low(v.s("density")), default_corner: low(v.s("corner")),
                on_change: move |(a, d, c): (String, String, String)| msg.set(format!("onChange({a}, {d}, {c})")) }
            {status(msg)}
        }
    }
}

fn accent_density_code(v: &Vals) -> String {
    Code::new("AccentDensity")
        .s("default_accent", &low(v.s("accent")), "blue")
        .s("default_density", &low(v.s("density")), "cozy")
        .s("default_corner", &low(v.s("corner")), "soft")
        .raw(
            "on_change",
            "move |(accent, density, corners)| save(accent, density, corners)",
        )
        .done()
}

fn accent_density_demo() -> Demo {
    Demo {
        controls: vec![
            sel(
                "accent",
                "Accent",
                &["Graphite", "Blue", "Violet", "Green", "Rose", "Orange"],
                "Blue",
            ),
            sel("density", "Density", &["Compact", "Cozy", "Roomy"], "Cozy"),
            sel("corner", "Corners", &["Square", "Soft", "Round"], "Soft"),
        ],
        variants: vec![
            var("default", "Blue, cozy rows, soft corners.", &[]),
            var(
                "compact square",
                "Graphite, compact rows without their second line, square corners: a dense tool panel.",
                &[
                    ("accent", "Graphite"),
                    ("density", "Compact"),
                    ("corner", "Square"),
                ],
            ),
            var(
                "roomy round",
                "Rose, roomy rows and round corners.",
                &[
                    ("accent", "Rose"),
                    ("density", "Roomy"),
                    ("corner", "Round"),
                ],
            ),
            var(
                "green",
                "Green accent, cozy rows, round corners.",
                &[("accent", "Green"), ("corner", "Round")],
            ),
        ],
        render: accent_density,
        code: accent_density_code,
        keyboard: vec![
            (
                "Tab",
                "Accents, density, corners, then the sample's switches and buttons",
            ),
            ("Enter / Space", "Picks an option or flips a switch"),
        ],
        a11y: vec![
            "Accent, Density and Corners are radio groups; accents are named, not only coloured.",
            "Sample switches are role=switch with aria-checked.",
            "Every change is said in a polite live note.",
        ],
        responsive: vec!["600 px card; controls stack above the sample under 560 px."],
        height: 420,
        live: false,
        align: Align::Center,
        ..mk(
            "accent-density",
            "Accent and density",
            CAT,
            "Oxyn original: accent, row spacing and corner shape beside a settings sample that takes each at once.",
            "AccentDensity",
            "AccentDensityEl",
            "accent-density",
            "parity/themefid.rs",
        )
    }
}

/// Every Theme Selector (variants 3) demo, in sidebar order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        background_picker_demo(),
        theme_grid_demo(),
        theme_split_demo(),
        accent_density_demo(),
    ]
}
