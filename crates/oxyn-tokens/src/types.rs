//! Plain value types the generated tokens are made of. No renderer dependencies.

/// An sRGB colour with straight (not premultiplied) alpha.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Color {
    /// Red, 0–255.
    pub r: u8,
    /// Green, 0–255.
    pub g: u8,
    /// Blue, 0–255.
    pub b: u8,
    /// Opacity, 0.0–1.0.
    pub a: f32,
}

impl Color {
    /// An opaque colour.
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 1.0 }
    }

    /// A colour with alpha.
    pub const fn rgba(r: u8, g: u8, b: u8, a: f32) -> Self {
        Self { r, g, b, a }
    }

    /// `#RRGGBB` as ASCII bytes (alpha is not included).
    pub const fn hex(self) -> [u8; 7] {
        const H: &[u8; 16] = b"0123456789ABCDEF";
        let (r, g, b) = (self.r as usize, self.g as usize, self.b as usize);
        [
            b'#',
            H[r >> 4],
            H[r & 15],
            H[g >> 4],
            H[g & 15],
            H[b >> 4],
            H[b & 15],
        ]
    }

    /// This colour composited over an opaque background, in sRGB, rounded to 8 bits.
    pub fn over(self, bg: Color) -> Color {
        let a = self.a as f64;
        let mix = |f: u8, b: u8| {
            let v = a * (f as f64 / 255.0) + (1.0 - a) * (b as f64 / 255.0);
            (v * 255.0).round().clamp(0.0, 255.0) as u8
        };
        Color::rgb(mix(self.r, bg.r), mix(self.g, bg.g), mix(self.b, bg.b))
    }

    /// WCAG 2.x relative luminance (alpha ignored).
    pub fn luminance(self) -> f64 {
        let lin = |c: u8| {
            let c = c as f64 / 255.0;
            if c <= 0.04045 {
                c / 12.92
            } else {
                ((c + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * lin(self.r) + 0.7152 * lin(self.g) + 0.0722 * lin(self.b)
    }

    /// WCAG 2.x contrast ratio between two opaque colours (1.0–21.0).
    pub fn contrast(self, other: Color) -> f64 {
        let (a, b) = (self.luminance(), other.luminance());
        let (hi, lo) = if a > b { (a, b) } else { (b, a) };
        (hi + 0.05) / (lo + 0.05)
    }
}

/// Unit of a [`Dimension`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Unit {
    /// CSS pixels (logical pixels in GPUI).
    Px,
    /// Multiples of the root font size.
    Rem,
}

/// A length token.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Dimension {
    /// The number.
    pub value: f32,
    /// Its unit.
    pub unit: Unit,
}

impl Dimension {
    /// Pixels, given the root font size in pixels (16 on the web by default).
    pub const fn to_px(self, root_px: f32) -> f32 {
        match self.unit {
            Unit::Px => self.value,
            Unit::Rem => self.value * root_px,
        }
    }
}

/// A composite type style.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Typography {
    /// Font stack, first choice first.
    pub font_family: &'static [&'static str],
    /// Size.
    pub font_size: Dimension,
    /// Weight, 100–900.
    pub font_weight: u16,
    /// Tracking.
    pub letter_spacing: Dimension,
    /// Line height as a multiple of the font size.
    pub line_height: f32,
}

/// A drop shadow.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shadow {
    /// Colour, usually translucent.
    pub color: Color,
    /// Horizontal offset.
    pub offset_x: Dimension,
    /// Vertical offset.
    pub offset_y: Dimension,
    /// Blur radius.
    pub blur: Dimension,
    /// Spread.
    pub spread: Dimension,
}

/// A border, used for the focus ring.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Border {
    /// Colour.
    pub color: Color,
    /// Width.
    pub width: Dimension,
}

/// A CSS-style cubic Bézier easing: `(x1, y1, x2, y2)`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier(pub [f32; 4]);

/// One colour theme. Every theme has the same roles.
#[derive(Debug)]
pub struct Theme {
    /// Token name: `dark`, `light` or `hc-dark`.
    pub name: &'static str,
    /// One colour per [`crate::ColorRole`], indexed by the role.
    pub colors: [Color; crate::ColorRole::COUNT],
    /// One shadow per [`crate::ElevationRole`].
    pub elevation: [Shadow; crate::ElevationRole::COUNT],
    /// Focus ring.
    pub focus_ring: Border,
}
