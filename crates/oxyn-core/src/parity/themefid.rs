//! Theme Selector family (variants 3): how an app looks – the Background picker (picture tiles,
//! a tile for your own picture and a texture laid over it), the Theme grid (eight mini-window
//! swatches and a font size), and two Oxyn originals: Theme split (two themes over one sample
//! window, a divider between them) and Accent and density (accent, spacing and corners).
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
}
