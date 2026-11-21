//! Theme Selector family (variants 3): how an app looks – the Background picker (picture tiles,
//! a tile for your own picture and a texture laid over it), the Theme grid (eight mini-window
//! swatches and a font size), and Oxyn originals: Theme split (two themes over one sample
//! window, a divider between them), Accent and density (accent, spacing and corners), Theme
//! schedule (light and dark by the clock), Type scale (family, size and spacing on a reading
//! sample), Light playground (a lamp you drag; shadows follow), App icon (alternate icons we draw
//! in Colour, Dark or Tinted) and Display comfort (contrast, motion, transparency, bold text).
//! Behaviour only: the pictures are drawn by us, and every name, colour and line of copy is our own.

/// One background tile.
#[derive(Clone, Debug, PartialEq)]
pub struct Backdrop {
    /// Stable key (`default`, `canopy`, `orbit`, `neon`, `tide`, `own`).
    pub key: &'static str,
    /// Label (the tile's accessible name).
    pub label: &'static str,
    /// What the picture shows, in a few words.
    pub alt: &'static str,
}

/// The built-in backgrounds, in tile order: the plain default, then four pictures we draw.
pub fn backdrops() -> Vec<Backdrop> {
    let b = |key, label, alt| Backdrop { key, label, alt };
    vec![
        b("default", "Default", "Plain, no picture"),
        b("canopy", "Canopy", "Leaves against a pale sky"),
        b("orbit", "Orbit", "A planet's edge at sunrise"),
        b("neon", "Neon", "Pink lights, out of focus"),
        b("tide", "Tide", "Waves in blue light"),
    ]
}

/// A texture laid over the background.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Texture {
    /// No texture.
    None,
    /// Soft film grain.
    Film,
    /// Fine scan lines, like a screen between channels.
    Static,
    /// An ordered dot pattern.
    Dither,
}

impl Texture {
    /// The three textures, in segment order.
    pub const ALL: [Texture; 3] = [Texture::Film, Texture::Static, Texture::Dither];

    /// Stable key.
    pub fn key(self) -> &'static str {
        match self {
            Texture::None => "none",
            Texture::Film => "film",
            Texture::Static => "static",
            Texture::Dither => "dither",
        }
    }

    /// Label.
    pub fn label(self) -> &'static str {
        match self {
            Texture::None => "None",
            Texture::Film => "Film",
            Texture::Static => "Static",
            Texture::Dither => "Dither",
        }
    }

    /// From a key (`none` for anything unknown).
    pub fn from_key(k: &str) -> Texture {
        match k {
            "film" => Texture::Film,
            "static" => Texture::Static,
            "dither" => Texture::Dither,
            _ => Texture::None,
        }
    }
}

/// Pressing a texture: the pressed one, or none when it was already on.
pub fn texture_press(cur: Texture, pressed: Texture) -> Texture {
    if cur == pressed {
        Texture::None
    } else {
        pressed
    }
}

/// Largest picture accepted for "Your own", in bytes (8 MB).
pub const OWN_MAX: u64 = 8 * 1024 * 1024;

/// Whether a picked file can be a background: an image, at most [`OWN_MAX`].
pub fn own_check(mime: &str, size: u64) -> Result<(), &'static str> {
    if !mime.starts_with("image/") {
        return Err("That file is not a picture.");
    }
    if size > OWN_MAX {
        return Err("Pick a picture under 8 MB.");
    }
    Ok(())
}

/// A `data:` URL for a picture's bytes (standard base64 with padding).
pub fn data_url(mime: &str, bytes: &[u8]) -> String {
    const A: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut s = String::with_capacity(bytes.len() * 4 / 3 + mime.len() + 16);
    s.push_str("data:");
    s.push_str(mime);
    s.push_str(";base64,");
    for ch in bytes.chunks(3) {
        let n = (ch[0] as u32) << 16
            | (*ch.get(1).unwrap_or(&0) as u32) << 8
            | *ch.get(2).unwrap_or(&0) as u32;
        s.push(A[(n >> 18) as usize & 63] as char);
        s.push(A[(n >> 12) as usize & 63] as char);
        s.push(if ch.len() > 1 {
            A[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        s.push(if ch.len() > 2 {
            A[n as usize & 63] as char
        } else {
            '='
        });
    }
    s
}

/// The colours of one theme, as `RRGGBB`.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeSwatch {
    /// Stable key (`system`, `light`, `dark`, `midnight`, `plum`, `moss`, `harbor`, `contrast`).
    pub key: &'static str,
    /// Label.
    pub label: &'static str,
    /// Sidebar.
    pub side: &'static str,
    /// Main pane.
    pub pane: &'static str,
    /// Hairlines and the placeholder bars.
    pub line: &'static str,
    /// The button block in the pane.
    pub block: &'static str,
    /// Text.
    pub fg: &'static str,
    /// Quiet text.
    pub mute: &'static str,
    /// Light (`false`) or dark (`true`).
    pub dark: bool,
}

fn sw(key: &'static str, label: &'static str, c: [&'static str; 6], dark: bool) -> ThemeSwatch {
    ThemeSwatch {
        key,
        label,
        side: c[0],
        pane: c[1],
        line: c[2],
        block: c[3],
        fg: c[4],
        mute: c[5],
        dark,
    }
}

/// The eight themes, in grid order. `system` borrows Light or Dark from the device.
pub fn swatches() -> Vec<ThemeSwatch> {
    let light = ["EDEDEF", "F7F7F8", "D9D9DE", "E4E4E8", "18181B", "6B6B73"];
    let dark = ["141316", "1C1C1E", "2E2E33", "26262A", "F2F2F3", "8A8A93"];
    vec![
        sw("system", "System", dark, true),
        sw("light", "Light", light, false),
        sw("dark", "Dark", dark, true),
        sw(
            "midnight",
            "Midnight",
            ["15152A", "1D1D38", "30305A", "141429", "EDEDF7", "8E8EB3"],
            true,
        ),
        sw(
            "plum",
            "Plum",
            ["241A22", "31232E", "4A3644", "1F161D", "F4ECF2", "A48F9F"],
            true,
        ),
        sw(
            "moss",
            "Moss",
            ["161A17", "1E2420", "303833", "131714", "ECF2EE", "8A968E"],
            true,
        ),
        sw(
            "harbor",
            "Harbor",
            ["141B2A", "1B2638", "2C3A52", "111827", "EAF0F8", "8796AE"],
            true,
        ),
        sw(
            "contrast",
            "Contrast",
            ["0B0B0C", "000000", "FFFFFF", "000000", "FFFFFF", "C8C8CC"],
            true,
        ),
    ]
}

/// The theme a key shows: `system` follows the device (`device_dark`); unknown keys are Dark.
pub fn swatch_for(key: &str, device_dark: bool) -> ThemeSwatch {
    let all = swatches();
    let key = if key == "system" {
        if device_dark { "dark" } else { "light" }
    } else {
        key
    };
    all.iter()
        .find(|s| s.key == key)
        .cloned()
        .unwrap_or_else(|| all[2].clone())
}

/// A font size choice.
#[derive(Clone, Debug, PartialEq)]
pub struct FontSize {
    /// Stable key.
    pub key: &'static str,
    /// Label.
    pub label: &'static str,
    /// Body size in px.
    pub px: u32,
}

/// The font sizes, smallest first.
pub fn font_sizes() -> Vec<FontSize> {
    let f = |key, label, px| FontSize { key, label, px };
    vec![
        f("small", "Small", 13),
        f("default", "Default", 14),
        f("large", "Large", 16),
        f("larger", "Larger", 18),
    ]
}

/// Body px for a font size key (14 when unknown).
pub fn font_px(key: &str) -> u32 {
    font_sizes()
        .iter()
        .find(|f| f.key == key)
        .map(|f| f.px)
        .unwrap_or(14)
}

/// The next option for a listbox key press: `delta` −1 or +1, clamped to the ends.
pub fn step(i: usize, delta: i32, len: usize) -> usize {
    if len == 0 {
        return 0;
    }
    (i as i32 + delta).clamp(0, len as i32 - 1) as usize
}

/// Theme split: the divider position, kept clear of both edges (8–92 %).
pub fn split_clamp(v: f64) -> f64 {
    v.clamp(8.0, 92.0)
}

/// An accent colour (`RRGGBB`) – the one hue the Accent and density preview uses.
#[derive(Clone, Debug, PartialEq)]
pub struct Accent {
    /// Stable key.
    pub key: &'static str,
    /// Label.
    pub label: &'static str,
    /// Colour.
    pub hex: &'static str,
}

/// The accents: Graphite first (no hue), then five colours.
pub fn accents() -> Vec<Accent> {
    let a = |key, label, hex| Accent { key, label, hex };
    vec![
        a("graphite", "Graphite", "8A8A93"),
        a("blue", "Blue", "3B82F6"),
        a("violet", "Violet", "8B5CF6"),
        a("green", "Green", "22C55E"),
        a("rose", "Rose", "F43F5E"),
        a("orange", "Orange", "F97316"),
    ]
}

/// Spacing of rows and controls.
#[derive(Clone, Debug, PartialEq)]
pub struct Density {
    /// Stable key.
    pub key: &'static str,
    /// Label.
    pub label: &'static str,
    /// Row height in px.
    pub row: u32,
    /// Inner padding in px.
    pub pad: u32,
    /// Gap between rows in px.
    pub gap: u32,
}

/// Compact, Cozy (the default) and Roomy.
pub fn densities() -> Vec<Density> {
    let d = |key, label, row, pad, gap| Density {
        key,
        label,
        row,
        pad,
        gap,
    };
    vec![
        d("compact", "Compact", 30, 8, 2),
        d("cozy", "Cozy", 38, 11, 5),
        d("roomy", "Roomy", 46, 14, 8),
    ]
}

/// Corner radius choices: (key, label, px).
pub fn corners() -> Vec<(&'static str, &'static str, u32)> {
    vec![
        ("square", "Square", 3),
        ("soft", "Soft", 9),
        ("round", "Round", 16),
    ]
}

/// How many sample rows fit a preview `h` px tall at a density.
pub fn rows_fit(h: u32, d: &Density) -> u32 {
    let per = d.row + d.gap;
    if per == 0 {
        return 0;
    }
    (h + d.gap) / per
}

/// Theme schedule: minutes since midnight as `HH:MM`.
pub fn fmt_hm(min: u32) -> String {
    let m = min % (24 * 60);
    format!("{:02}:{:02}", m / 60, m % 60)
}

/// Sunrise and sunset of the demo place, in minutes (06:42 and 17:18).
pub const SUN: (u32, u32) = (6 * 60 + 42, 17 * 60 + 18);

/// Whether the light theme shows at `at` for a light window `from`–`to` (minutes; a window that
/// wraps past midnight works too).
pub fn light_at(at: u32, from: u32, to: u32) -> bool {
    let at = at % 1440;
    if from == to {
        return true;
    }
    if from < to {
        at >= from && at < to
    } else {
        at >= from || at < to
    }
}

/// The schedule modes: (key, label).
pub fn sched_modes() -> Vec<(&'static str, &'static str)> {
    vec![
        ("light", "Light"),
        ("dark", "Dark"),
        ("sun", "Sun"),
        ("custom", "Custom"),
    ]
}

/// What a mode shows at `at`: `true` for light.
pub fn sched_light(mode: &str, at: u32, from: u32, to: u32) -> bool {
    match mode {
        "light" => true,
        "dark" => false,
        "sun" => light_at(at, SUN.0, SUN.1),
        _ => light_at(at, from, to),
    }
}

/// Minutes until the next switch for a window (`None` when it never switches).
pub fn next_switch(at: u32, from: u32, to: u32) -> Option<u32> {
    if from == to {
        return None;
    }
    let at = at % 1440;
    let until = |t: u32| (t + 1440 - at) % 1440;
    let a = until(from);
    let b = until(to);
    let a = if a == 0 { 1440 } else { a };
    let b = if b == 0 { 1440 } else { b };
    Some(a.min(b))
}

/// "in 3 h 20 min", "in 45 min".
pub fn fmt_in(min: u32) -> String {
    match (min / 60, min % 60) {
        (0, m) => format!("in {m} min"),
        (h, 0) => format!("in {h} h"),
        (h, m) => format!("in {h} h {m} min"),
    }
}

/// Type scale: the text sizes on the A− / A+ steps, in px.
pub const TYPE_STEPS: [u32; 6] = [13, 14, 15, 16, 18, 20];

/// Type families: (key, label, average glyph width as a share of the size).
pub fn type_families() -> Vec<(&'static str, &'static str, f64)> {
    vec![
        ("sans", "Sans", 0.52),
        ("serif", "Serif", 0.48),
        ("mono", "Mono", 0.6),
    ]
}

/// Line spacing: (key, label, line height).
pub fn type_spacing() -> Vec<(&'static str, &'static str, f64)> {
    vec![
        ("tight", "Tight", 1.3),
        ("normal", "Normal", 1.5),
        ("relaxed", "Relaxed", 1.75),
    ]
}

/// About how many characters fit one line `width` px wide.
pub fn chars_per_line(width: f64, size: u32, family: &str) -> u32 {
    let w = type_families()
        .iter()
        .find(|f| f.0 == family)
        .map(|f| f.2)
        .unwrap_or(0.52);
    (width / (size.max(1) as f64 * w)).round() as u32
}

/// How a line length reads: under 45 short, over 85 long, else comfortable.
pub fn measure_word(chars: u32) -> &'static str {
    if chars < 45 {
        "short lines"
    } else if chars > 85 {
        "long lines"
    } else {
        "comfortable"
    }
}

/// Light playground: keeps the lamp inside the stage (4–96 %).
pub fn lamp_clamp(v: f64) -> f64 {
    v.clamp(4.0, 96.0)
}

/// The shadow a card at (`cx`, `cy`) % casts from a lamp at (`lx`, `ly`) %: (dx, dy, blur) in px.
/// The shadow falls away from the lamp and softens as it gets further; `strength` is 0–100.
pub fn lamp_shadow(lx: f64, ly: f64, cx: f64, cy: f64, strength: f64) -> (f64, f64, f64) {
    let k = strength.clamp(0.0, 100.0) / 100.0;
    let (dx, dy) = (cx - lx, cy - ly);
    let dist = (dx * dx + dy * dy).sqrt();
    let reach = 0.28 * k;
    (
        (dx * reach).clamp(-24.0, 24.0),
        (dy * reach).clamp(-24.0, 24.0),
        6.0 + dist * 0.35 * k,
    )
}

/// The angle (degrees, CSS `linear-gradient` convention) from a card at (`cx`, `cy`) toward the lamp.
pub fn lamp_angle(lx: f64, ly: f64, cx: f64, cy: f64) -> f64 {
    let a = (lx - cx).atan2(-(ly - cy)).to_degrees();
    (a + 360.0) % 360.0
}

/// An alternate app icon we draw: a ring-and-dot mark on a two-stop ground.
#[derive(Clone, Debug, PartialEq)]
pub struct AppIcon {
    /// Stable key.
    pub key: &'static str,
    /// Label.
    pub label: &'static str,
    /// Ground, top-left stop.
    pub from: &'static str,
    /// Ground, bottom-right stop.
    pub to: &'static str,
    /// The mark.
    pub mark: &'static str,
}

/// The six icons, in grid order.
pub fn app_icons() -> Vec<AppIcon> {
    let i = |key, label, from, to, mark| AppIcon {
        key,
        label,
        from,
        to,
        mark,
    };
    vec![
        i("classic", "Classic", "2A2A2E", "111113", "F2F2F3"),
        i("paper", "Paper", "FFFFFF", "E4E4E8", "111113"),
        i("daybreak", "Daybreak", "FF8A4C", "E8457C", "FFFFFF"),
        i("ocean", "Ocean", "38BDF8", "2453D6", "FFFFFF"),
        i("mint", "Mint", "5EEAD4", "16A36A", "06281C"),
        i("nebula", "Nebula", "A855F7", "EC4899", "FFFFFF"),
    ]
}

/// Icon styles: (key, label).
pub fn icon_styles() -> Vec<(&'static str, &'static str)> {
    vec![("color", "Colour"), ("dark", "Dark"), ("tinted", "Tinted")]
}

/// The colours one icon shows in a style: (ground from, ground to, mark), as CSS colours.
/// Dark lays the icon's own colours on a near-black ground; Tinted draws it in one hue (`tint`).
pub fn icon_colors(icon: &AppIcon, style: &str, tint: u32) -> (String, String, String) {
    match style {
        "dark" => {
            let mark = if icon.key == "classic" || icon.key == "paper" {
                "#F2F2F3".to_string()
            } else {
                format!("#{}", icon.from)
            };
            ("#1C1C1E".into(), "#0B0B0C".into(), mark)
        }
        "tinted" => {
            let h = tint % 360;
            (
                format!("hsl({h} 45% 22%)"),
                format!("hsl({h} 50% 10%)"),
                format!("hsl({h} 90% 78%)"),
            )
        }
        _ => (
            format!("#{}", icon.from),
            format!("#{}", icon.to),
            format!("#{}", icon.mark),
        ),
    }
}

/// Display comfort switches: (key, label, what it does).
pub fn comfort_opts() -> Vec<(&'static str, &'static str, &'static str)> {
    vec![
        (
            "contrast",
            "Increase contrast",
            "Darker text and firmer edges",
        ),
        ("motion", "Reduce motion", "Stops loops and slides"),
        ("solid", "Reduce transparency", "Glass turns solid"),
        ("bold", "Bold text", "Heavier type everywhere"),
    ]
}

/// "Off", "1 on" … "All on" for the comfort switches.
pub fn comfort_said(on: &[bool]) -> String {
    let n = on.iter().filter(|x| **x).count();
    match n {
        0 => "All off".to_string(),
        n if n == on.len() => "All on".to_string(),
        n => format!("{n} on"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn textures_and_uploads() {
        assert_eq!(texture_press(Texture::Film, Texture::Film), Texture::None);
        assert_eq!(
            texture_press(Texture::None, Texture::Dither),
            Texture::Dither
        );
        assert_eq!(Texture::from_key("static"), Texture::Static);
        assert_eq!(Texture::from_key("x"), Texture::None);
        assert!(own_check("image/png", 1000).is_ok());
        assert!(own_check("text/plain", 10).is_err());
        assert!(own_check("image/gif", OWN_MAX + 1).is_err());
        assert_eq!(data_url("image/png", b"Man"), "data:image/png;base64,TWFu");
        assert_eq!(data_url("image/png", b"Ma"), "data:image/png;base64,TWE=");
        assert_eq!(data_url("image/png", b"M"), "data:image/png;base64,TQ==");
        assert_eq!(backdrops().len(), 5);
    }

    #[test]
    fn themes() {
        let s = swatches();
        assert_eq!(s.len(), 8);
        assert_eq!(swatch_for("system", false).key, "light");
        assert_eq!(swatch_for("system", true).key, "dark");
        assert_eq!(swatch_for("nope", false).key, "dark");
        assert_eq!(swatch_for("moss", false).label, "Moss");
        assert_eq!(font_px("large"), 16);
        assert_eq!(font_px("?"), 14);
        assert_eq!(step(0, -1, 4), 0);
        assert_eq!(step(3, 1, 4), 3);
        assert_eq!(step(1, 1, 4), 2);
    }

    #[test]
    fn split_and_density() {
        assert_eq!(split_clamp(0.0), 8.0);
        assert_eq!(split_clamp(50.0), 50.0);
        assert_eq!(split_clamp(120.0), 92.0);
        let d = densities();
        assert!(rows_fit(160, &d[0]) > rows_fit(160, &d[2]));
        assert_eq!(rows_fit(46, &d[2]), 1);
        assert_eq!(accents()[0].key, "graphite");
        assert_eq!(corners().len(), 3);
    }

    #[test]
    fn schedule_and_type() {
        assert_eq!(fmt_hm(7 * 60 + 5), "07:05");
        assert!(light_at(12 * 60, 7 * 60, 19 * 60));
        assert!(!light_at(22 * 60, 7 * 60, 19 * 60));
        assert!(light_at(23 * 60, 22 * 60, 6 * 60));
        assert!(!sched_light("dark", 600, 0, 1));
        assert!(sched_light("sun", 12 * 60, 0, 0));
        assert_eq!(next_switch(18 * 60, 7 * 60, 19 * 60), Some(60));
        assert_eq!(next_switch(0, 5, 5), None);
        assert_eq!(fmt_in(200), "in 3 h 20 min");
        assert_eq!(fmt_in(45), "in 45 min");
        assert_eq!(chars_per_line(520.0, 16, "sans"), 63);
        assert_eq!(measure_word(63), "comfortable");
        assert_eq!(measure_word(30), "short lines");
    }

    #[test]
    fn light_icons_comfort() {
        assert_eq!(lamp_clamp(0.0), 4.0);
        let (dx, dy, b) = lamp_shadow(20.0, 20.0, 50.0, 50.0, 100.0);
        assert!(dx > 0.0 && dy > 0.0 && b > 6.0);
        let (dx0, _, b0) = lamp_shadow(50.0, 50.0, 50.0, 50.0, 100.0);
        assert_eq!(dx0, 0.0);
        assert_eq!(b0, 6.0);
        assert_eq!(lamp_angle(50.0, 0.0, 50.0, 50.0).round(), 0.0);
        assert_eq!(lamp_angle(100.0, 50.0, 50.0, 50.0).round(), 90.0);
        let ic = app_icons();
        assert_eq!(ic.len(), 6);
        assert_eq!(icon_colors(&ic[3], "color", 0).0, "#38BDF8");
        assert_eq!(icon_colors(&ic[0], "dark", 0).2, "#F2F2F3");
        assert!(icon_colors(&ic[2], "tinted", 200).2.starts_with("hsl(200"));
        assert_eq!(comfort_said(&[false, false]), "All off");
        assert_eq!(comfort_said(&[true, false, true]), "2 on");
        assert_eq!(comfort_said(&[true, true]), "All on");
        assert_eq!(comfort_opts().len(), 4);
    }
}
