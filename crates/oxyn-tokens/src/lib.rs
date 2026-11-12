//! Oxyn design tokens for Rust.
//!
//! The source of truth is `tokens/oxyn.tokens.json` and `tokens/oxyn.motion.json` (DTCG 2025.10).
//! Everything in [`generated`] is written by `cargo run -p oxyn-tokens-gen` and checked for drift in
//! CI, so never edit it by hand. The same run writes `tokens/dist/oxyn.css` for the web.
//!
//! Components read roles, never palettes:
//!
//! ```
//! use oxyn_tokens::{ColorRole, Theme};
//!
//! let theme = Theme::DARK;
//! let warn = theme.color(ColorRole::StatusWarningFg);
//! assert_eq!(warn.a, 1.0);
//! assert_eq!(Theme::by_name("light").map(|t| t.name), Some("light"));
//! ```
//!
//! Renderer crates convert these plain values: `oxyn-gpui` maps [`Color`] to `gpui::Hsla`, and
//! `oxyn-dioxus` reads the CSS variables.

mod types;

#[rustfmt::skip]
pub mod generated;

pub use generated::{ColorRole, ElevationRole, motion, primitive, semantic};
pub use types::*;

impl Theme {
    /// Every theme, default first.
    pub const ALL: [&'static Theme; 3] = [&Theme::DARK, &Theme::LIGHT, &Theme::HC_DARK];

    /// Looks a theme up by its token name (`dark`, `light`, `hc-dark`).
    pub fn by_name(name: &str) -> Option<&'static Theme> {
        Self::ALL.into_iter().find(|t| t.name == name)
    }

    /// The colour for a role in this theme.
    pub const fn color(&self, role: ColorRole) -> Color {
        self.colors[role as usize]
    }

    /// The shadow for an elevation role in this theme.
    pub const fn elevation(&self, role: ElevationRole) -> Shadow {
        self.elevation[role as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brand_values_hold() {
        let d = Theme::DARK;
        assert_eq!(d.color(ColorRole::BgBase).hex(), *b"#0B0B0C");
        assert_eq!(d.color(ColorRole::FgDefault).hex(), *b"#F2EFEA");
        assert_eq!(primitive::color::rust::N500.hex(), *b"#FF5A1F");
    }

    #[test]
    fn roles_round_trip() {
        for role in ColorRole::ALL {
            assert_eq!(ColorRole::from_path(role.path()), Some(role));
        }
        assert_eq!(ColorRole::ALL.len(), ColorRole::COUNT);
    }

    #[test]
    fn themes_are_distinct() {
        assert_ne!(
            Theme::DARK.color(ColorRole::BgBase),
            Theme::LIGHT.color(ColorRole::BgBase)
        );
        assert_eq!(Theme::by_name("hc-dark").map(|t| t.name), Some("hc-dark"));
        assert!(Theme::by_name("sepia").is_none());
    }

    #[test]
    fn contrast_matches_wcag() {
        let black = Color::rgb(0, 0, 0);
        let white = Color::rgb(255, 255, 255);
        assert!((black.contrast(white) - 21.0).abs() < 1e-9);
        let d = Theme::DARK;
        assert!(
            d.color(ColorRole::FgDefault)
                .contrast(d.color(ColorRole::BgBase))
                >= 4.5
        );
    }

    #[test]
    fn motion_values_hold() {
        assert_eq!(motion::dur::FAST.as_millis(), 120);
        assert_eq!(motion::ease::OUT.0, [0.22, 1.0, 0.36, 1.0]);
    }
}
