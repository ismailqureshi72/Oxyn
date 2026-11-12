//! `GlyphLoader`: the T1 glyph loaders (`docs/specs/loader-glyph.md`).
//!
//! Hold an `oxyn_core::loader::GlyphLoader` in your view, call `update` with the agent state each
//! frame, and render `GlyphLoaderView::new(loader.frame(now, still))`. Repaint after
//! `loader.next_due(now, still)`: an 80 ms table repaints at 12.5 Hz, not 60.

use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};
use oxyn_core::loader::GlyphFrame;
use oxyn_tokens::Theme;

use crate::{hsla, mix};

/// Draws one [`GlyphFrame`] in fixed-width cells, so the line never moves.
#[derive(IntoElement)]
pub struct GlyphLoaderView {
    frame: Option<GlyphFrame>,
    cells: usize,
    theme: &'static Theme,
    font_px: f32,
}

impl GlyphLoaderView {
    /// A view of `frame`, reserving `cells` cells (the table's width) even when hidden.
    pub fn new(frame: Option<GlyphFrame>, cells: usize) -> Self {
        Self {
            frame,
            cells,
            theme: &Theme::DARK,
            font_px: 13.0,
        }
    }

    /// Theme (dark by default).
    pub fn theme(mut self, theme: &'static Theme) -> Self {
        self.theme = theme;
        self
    }

    /// Font size in px (12–32; the cell box scales with it).
    pub fn font_size(mut self, px: f32) -> Self {
        self.font_px = px;
        self
    }
}

impl RenderOnce for GlyphLoaderView {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let cell_w = (self.font_px * 0.62).ceil();
        let row = div()
            .flex()
            .flex_none()
            .h(px(self.font_px * 1.35))
            .items_center()
            .font_family("JetBrains Mono")
            .text_size(px(self.font_px));
        let Some(f) = self.frame else {
            return row.w(px(cell_w * self.cells as f32));
        };
        let colour = mix(
            self.theme.color(f.color_from),
            self.theme.color(f.color),
            f.color_t,
        );
        row.children(
            f.text
                .chars()
                .zip(f.cell_alpha.iter().copied())
                .map(|(ch, a)| {
                    let mut c = hsla(colour);
                    c.a *= a * f.opacity;
                    div()
                        .w(px(cell_w))
                        .flex()
                        .justify_center()
                        .text_color(c)
                        .child(ch.to_string())
                }),
        )
    }
}
