//! Thinking Selector family, variant "Nozzle line" (an Oxyn original after the user's reference):
//! a row of perfectly even, thin vertical lines carrying a colour gradient from steel blue through
//! coral to periwinkle; a solid nozzle – a knob filled with the gradient colour where it stands –
//! slides along it and snaps to the levels. Behaviour and colour only; level logic is in
//! [`super::thinkfid`].

/// How the line is coloured.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum NozzleTone {
    /// The reference gradient: steel blue → grey → pale yellow → peach → coral (centre) → rose →
    /// plum → violet → indigo → periwinkle.
    #[default]
    Gradient,
    /// One red for every line and the nozzle (the earlier Oxyn look).
    Red,
}

impl NozzleTone {
    /// `"red"` → [`NozzleTone::Red`]; anything else is the gradient.
    pub fn parse(s: &str) -> Self {
        if s.eq_ignore_ascii_case("red") {
            Self::Red
        } else {
            Self::Gradient
        }
    }
    /// `"gradient"` or `"red"`.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Gradient => "gradient",
            Self::Red => "red",
        }
    }
}

/// Gradient stops for a dark surface: (position 0–1, `0xRRGGBB`). The reference's colours.
pub const NOZZLE_DARK: [(f32, u32); 13] = [
    (0.00, 0x8FACC0),
    (0.12, 0xACB5B4),
    (0.24, 0xBDBFA3),
    (0.33, 0xDCC496),
    (0.41, 0xE5B27F),
    (0.46, 0xE46854),
    (0.50, 0xF6644C),
    (0.60, 0xB5596D),
    (0.70, 0x8D4D79),
    (0.80, 0x563C77),
    (0.87, 0x504984),
    (0.94, 0x67629C),
    (1.00, 0x717CB1),
];

/// The same stops darkened for a light surface: each keeps its hue and has at least 4:1 contrast
/// on white.
pub const NOZZLE_LIGHT: [(f32, u32); 13] = [
    (0.00, 0x4E7690),
    (0.12, 0x6B7574),
    (0.24, 0x77784E),
    (0.33, 0x977535),
    (0.41, 0xAD6526),
    (0.46, 0xD2483A),
    (0.50, 0xDB3F27),
    (0.60, 0xB5596D),
    (0.70, 0x8D4D79),
    (0.80, 0x563C77),
    (0.87, 0x504984),
    (0.94, 0x67629C),
    (1.00, 0x5A66A3),
];

/// The red tone, dark and light.
pub const NOZZLE_RED: (u32, u32) = (0xF0533F, 0xE0442F);

fn ch(c: u32, s: u32) -> f32 {
    ((c >> s) & 0xFF) as f32
}

/// The line's colour at `f` (0 = left end, 1 = right end) as `0xRRGGBB`, interpolated between
/// the stops.
pub fn nozzle_rgb(f: f32, tone: NozzleTone, dark: bool) -> u32 {
    if tone == NozzleTone::Red {
        return if dark { NOZZLE_RED.0 } else { NOZZLE_RED.1 };
    }
    let stops = if dark { &NOZZLE_DARK } else { &NOZZLE_LIGHT };
    let f = if f.is_finite() {
        f.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut i = 0;
    while i + 2 < stops.len() && f > stops[i + 1].0 {
        i += 1;
    }
    let (a, ca) = stops[i];
    let (b, cb) = stops[i + 1];
    let t = if b > a {
        ((f - a) / (b - a)).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mix = |s: u32| -> u32 { (ch(ca, s) + (ch(cb, s) - ch(ca, s)) * t).round() as u32 };
    (mix(16) << 16) | (mix(8) << 8) | mix(0)
}

/// `#RRGGBB`.
pub fn hex(c: u32) -> String {
    format!("#{:06X}", c & 0xFF_FFFF)
}

/// Relative luminance (WCAG) of `0xRRGGBB`.
pub fn luminance(c: u32) -> f32 {
    let lin = |s: u32| {
        let v = ch(c, s) / 255.0;
        if v <= 0.039_28 {
            v / 12.92
        } else {
            ((v + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * lin(16) + 0.7152 * lin(8) + 0.0722 * lin(0)
}

/// WCAG contrast ratio of two colours.
pub fn contrast(a: u32, b: u32) -> f32 {
    let (x, y) = (luminance(a), luminance(b));
    (x.max(y) + 0.05) / (x.min(y) + 0.05)
}

/// The nozzle's light rim and glyph colour.
pub const NOZZLE_INK_LIGHT: u32 = 0xFFF6EE;
/// The glyph colour on a pale fill.
pub const NOZZLE_INK_DARK: u32 = 0x1A1A1C;

/// Glyph colour for a nozzle filled with `fill`: the light ink (as in the reference) while it
/// keeps 2.2:1, the dark one on pale fills.
pub fn nozzle_ink(fill: u32) -> u32 {
    if contrast(fill, NOZZLE_INK_LIGHT) >= 2.2 {
        NOZZLE_INK_LIGHT
    } else {
        NOZZLE_INK_DARK
    }
}

/// How lit line `x` is with the nozzle at `pos` (both 0–1): 1 under the nozzle, falling to 0 at
/// 0.4 of the line away. Lines near the nozzle are drawn brightest.
pub fn line_glow(x: f32, pos: f32) -> f32 {
    let d = (x - pos).abs();
    (1.0 - d / 0.4).clamp(0.0, 1.0)
}

/// The line a level sits on: level 0 on the first, the last level on the last, the rest spread
/// evenly between.
pub fn level_tick(level: usize, levels: usize, ticks: usize) -> usize {
    if levels <= 1 || ticks <= 1 {
        return 0;
    }
    let l = level.min(levels - 1);
    ((l as f32 / (levels - 1) as f32) * (ticks - 1) as f32).round() as usize
}

/// Whether line `t` is a level stop (marked `data-stop`; drawn the same as the others).
pub fn is_stop(t: usize, levels: usize, ticks: usize) -> bool {
    (0..levels).any(|l| level_tick(l, levels, ticks) == t)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gradient_ends_and_centre() {
        assert_eq!(nozzle_rgb(0.0, NozzleTone::Gradient, true), 0x8FACC0);
        assert_eq!(nozzle_rgb(0.5, NozzleTone::Gradient, true), 0xF6644C);
        assert_eq!(nozzle_rgb(1.0, NozzleTone::Gradient, true), 0x717CB1);
        assert_eq!(nozzle_rgb(f32::NAN, NozzleTone::Gradient, true), 0x8FACC0);
        assert_eq!(nozzle_rgb(0.3, NozzleTone::Red, false), 0xE0442F);
        assert_eq!(hex(0xF6644C), "#F6644C");
        assert_eq!(NozzleTone::parse("RED"), NozzleTone::Red);
        assert_eq!(NozzleTone::parse("x"), NozzleTone::Gradient);
    }

    #[test]
    fn light_stops_read_on_white() {
        for i in 0..=40 {
            let c = nozzle_rgb(i as f32 / 40.0, NozzleTone::Gradient, false);
            assert!(contrast(c, 0xFFFFFF) >= 3.0, "{}", hex(c));
        }
        assert_eq!(nozzle_ink(0xF6644C), NOZZLE_INK_LIGHT);
        assert_eq!(nozzle_ink(0xDCC496), NOZZLE_INK_DARK);
    }

    #[test]
    fn stops_and_glow() {
        assert_eq!(level_tick(0, 5, 37), 0);
        assert_eq!(level_tick(2, 5, 37), 18);
        assert_eq!(level_tick(4, 5, 37), 36);
        assert!(is_stop(9, 5, 37));
        assert!(!is_stop(10, 5, 37));
        assert_eq!(level_tick(3, 1, 10), 0);
        assert_eq!(line_glow(0.5, 0.5), 1.0);
        assert_eq!(line_glow(0.0, 0.5), 0.0);
    }
}
