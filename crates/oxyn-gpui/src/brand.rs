//! Brand and model marks (`oxyn_core::brand`) for GPUI. GPUI draws SVGs from the app's
//! [`AssetSource`], so hosts serve [`BrandAssets`] (directly, or by trying it first in their own
//! source) and draw a mark with [`brand_icon`]. Marks are monochrome and take the text colour,
//! per the colour rule; pass `coloured` for the brand colour.

use std::borrow::Cow;

use gpui::{
    AnyElement, AssetSource, Hsla, IntoElement, Result, SharedString, Styled, Svg, img, px, rgb,
    svg,
};
use oxyn_core::brand::{BRANDS, brand};

/// Serves `brand/<id>.svg` for every mark in [`oxyn_core::brand::BRANDS`].
#[derive(Clone, Copy, Debug, Default)]
pub struct BrandAssets;

impl AssetSource for BrandAssets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        // Full-colour marks (Gmail, Slack, Google Drive, ClickUp, Claude…) draw as images.
        if let Some(id) = path
            .strip_prefix("brand-color/")
            .and_then(|p| p.strip_suffix(".svg"))
        {
            return Ok(BRANDS
                .iter()
                .find(|b| b.id == id)
                .map(|b| Cow::Owned(b.svg_full().into_bytes())));
        }
        let id = path
            .strip_prefix("brand/")
            .and_then(|p| p.strip_suffix(".svg"));
        Ok(id
            .and_then(|id| BRANDS.iter().find(|b| b.id == id))
            .map(|b| Cow::Owned(b.svg(false).into_bytes())))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(
            if path.is_empty() || "brand/".starts_with(path) || path.starts_with("brand") {
                BRANDS
                    .iter()
                    .map(|b| SharedString::from(format!("brand/{}.svg", b.id)))
                    .collect()
            } else {
                Vec::new()
            },
        )
    }
}

/// A mark in its own colours: the full-colour image for multi-colour logos (Gmail, Slack, Google
/// Drive, ClickUp, Claude, Gemini, Figma), else the brand colour, else `color` for near-black
/// marks. `None` when the name is unknown.
pub fn brand_mark(name: &str, size: f32, color: Hsla) -> Option<AnyElement> {
    let b = brand(name)?;
    if b.coloured_body().is_some() {
        return Some(
            img(SharedString::from(format!("brand-color/{}.svg", b.id)))
                .size(px(size))
                .flex_none()
                .into_any_element(),
        );
    }
    brand_icon(name, size, color, true).map(IntoElement::into_any_element)
}

/// The asset path of a mark, when `name` (an id, title, alias, model name or domain) resolves.
pub fn brand_path(name: &str) -> Option<SharedString> {
    brand(name).map(|b| SharedString::from(format!("brand/{}.svg", b.id)))
}

/// A mark `size` px square in `color` (the brand colour instead when `coloured` and the mark has
/// one that reads on both themes). `None` when the name is unknown: keep the monogram.
pub fn brand_icon(name: &str, size: f32, color: Hsla, coloured: bool) -> Option<Svg> {
    let b = brand(name)?;
    let s = svg()
        .path(SharedString::from(format!("brand/{}.svg", b.id)))
        .size(px(size))
        .flex_none();
    Some(match (coloured, b.display_hex()) {
        (true, Some(h)) => s.text_color(rgb(u32::from_str_radix(h, 16).unwrap_or(0))),
        _ => s.text_color(color),
    })
}
