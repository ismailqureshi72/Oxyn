//! Dioxus components for Oxyn (web demos, Studio, web apps).
//!
//! Components style themselves with the `--oxyn-*` custom properties from `tokens/dist/oxyn.css`;
//! include that stylesheet once (and set `data-oxyn-theme` for light or high contrast). Each
//! component also ships its own small stylesheet as a constant (`STATUS_INDICATOR_CSS`).

mod composer;
mod glyph_loader;
mod status_indicator;
mod status_text;

pub use composer::{
    AttachmentRow, AttachmentRowProps, COMPOSER_CSS, Composer, ComposerProps, ContextMeterChip,
    ContextMeterChipProps, DropZone, DropZoneProps, EffortChip, EffortChipProps, Icon, IconProps,
    ModelPicker, ModelPickerProps, Submission,
};
pub use glyph_loader::{GLYPH_LOADER_CSS, GlyphLoader, GlyphLoaderProps};
pub use status_indicator::{
    IndicatorSize, IndicatorVariant, STATUS_INDICATOR_CSS, StatusIndicator, StatusIndicatorProps,
};
pub use status_text::{
    ElapsedTimer, ElapsedTimerProps, STATUS_TEXT_CSS, TextReveal, TextRevealProps, TextShimmer,
    TextShimmerProps, VerbCycler, VerbCyclerProps,
};

/// `var(--oxyn-color-…)` for a colour role.
pub fn color_var(role: oxyn_tokens::ColorRole) -> String {
    format!("var(--oxyn-color-{})", role.path().replace('.', "-"))
}
