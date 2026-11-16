//! Theme Selector family (variants 3) for GPUI over `oxyn_core::parity::themefid`: **Background
//! picker** ([`BackgroundPickerEl`]), **Theme grid** ([`ThemeGridEl`]), **Theme split**
//! ([`ThemeSplitEl`]) and **Accent and density** ([`AccentDensityEl`]). `RenderOnce` snapshots:
//! the host keeps the state and routes [`Act`] ids back (`bg-<i>`, `own`, `tex-<i>`, `theme-<i>`,
//! `font`, `cand-<i>`, `apply`, `acc-<i>`, `den-<i>`, `cor-<i>`). The drawn pictures are shown as
//! their main colours in GPUI.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative,
};
use oxyn_core::parity::themefid::{
    Texture, ThemeSwatch, accents, backdrops, corners, densities, font_sizes, split_clamp,
    swatch_for, swatches,
};
use oxyn_tokens::Theme;

use super::Act;
use super::ctxvisfid::hex;
use super::rulerfid::{fire, th_pal};

const BG_ID: [&str; 5] = ["bg-0", "bg-1", "bg-2", "bg-3", "bg-4"];
const TEX_ID: [&str; 3] = ["tex-0", "tex-1", "tex-2"];
const TH_ID: [&str; 8] = [
    "theme-0", "theme-1", "theme-2", "theme-3", "theme-4", "theme-5", "theme-6", "theme-7",
];
const CAND_ID: [&str; 8] = [
    "cand-0", "cand-1", "cand-2", "cand-3", "cand-4", "cand-5", "cand-6", "cand-7",
];
const ACC_ID: [&str; 6] = ["acc-0", "acc-1", "acc-2", "acc-3", "acc-4", "acc-5"];
const DEN_ID: [&str; 3] = ["den-0", "den-1", "den-2"];
const COR_ID: [&str; 3] = ["cor-0", "cor-1", "cor-2"];

fn card(theme: &Theme, w: f32) -> gpui::Div {
    let p = th_pal(theme);
    div()
        .w(px(w))
        .rounded(px(20.))
        .bg(p.card)
        .border_1()
        .border_color(p.line)
        .text_color(p.fg)
        .text_size(px(14.))
        .overflow_hidden()
}

/// The two main colours of one of our pictures (sky and detail).
fn art_cols(key: &str) -> (Hsla, Hsla) {
    match key {
        "canopy" => (hex("7DB2CC"), hex("D9472F")),
        "orbit" => (hex("030305"), hex("3E8EE6")),
        "neon" => (hex("D23CB4"), hex("FFD9F6")),
        "tide" => (hex("12407A"), hex("7FC8F8")),
        _ => (hex("141316"), hex("2A2A2E")),
    }
}

/// `background-picker` (Theme Selector – Background picker): tiles, Your own, texture segments.
#[derive(IntoElement)]
pub struct BackgroundPickerEl {
    pick: usize,
    texture: Texture,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for BackgroundPickerEl {
    fn default() -> Self {
        Self {
            pick: 0,
            texture: Texture::None,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl BackgroundPickerEl {
    /// The picked tile (index into `backdrops()`).
    pub fn pick(mut self, i: usize) -> Self {
        self.pick = i;
        self
    }
    /// The texture.
    pub fn texture(mut self, t: Texture) -> Self {
        self.texture = t;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `bg-<i>`, `own`, `tex-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for BackgroundPickerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut grid = div().flex().flex_wrap().gap(px(10.)).mt(px(12.));
        for (i, b) in backdrops().iter().enumerate() {
            let (a, d) = art_cols(b.key);
            let f = fire(&self.act, BG_ID[i.min(4)]);
            let mut t = div()
                .id(SharedString::from(format!("oxbgk-{i}")))
                .relative()
                .w(px(166.))
                .h(px(94.))
                .rounded(px(12.))
                .overflow_hidden()
                .flex()
                .items_center()
                .justify_center()
                .bg(a)
                .border_1()
                .border_color(p.line)
                .on_click(move |_, w, cx| f(w, cx));
            if i == 0 {
                t = t.bg(p.inset).child(b.label);
            } else {
                t = t.child(div().size(px(28.)).rounded(px(14.)).bg(d));
            }
            if i == self.pick {
                t = t.border_2().border_color(p.fg).child(
                    div()
                        .absolute()
                        .top(px(7.))
                        .right(px(7.))
                        .size(px(20.))
                        .rounded(px(10.))
                        .bg(hex("FFFFFF"))
                        .text_color(hex("111113"))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(12.))
                        .child("✓"),
                );
            }
            grid = grid.child(t);
        }
        let fo = fire(&self.act, "own");
        grid = grid.child(
            div()
                .id("oxbgk-own")
                .w(px(166.))
                .h(px(94.))
                .rounded(px(12.))
                .bg(p.inset)
                .border_1()
                .border_color(p.line)
                .flex()
                .flex_col()
                .items_center()
                .justify_center()
                .gap(px(2.))
                .child("+")
                .child("Your own")
                .on_click(move |_, w, cx| fo(w, cx)),
        );
        let mut seg = div()
            .flex()
            .gap(px(2.))
            .p(px(3.))
            .rounded(px(12.))
            .bg(p.inset);
        for (i, x) in Texture::ALL.iter().enumerate() {
            let f = fire(&self.act, TEX_ID[i]);
            let on = *x == self.texture;
            seg = seg.child(
                div()
                    .id(SharedString::from(format!("oxbgk-tex-{i}")))
                    .h(px(32.))
                    .px(px(14.))
                    .flex()
                    .items_center()
                    .rounded(px(9.))
                    .text_color(if on { p.fg } else { p.mute })
                    .when(on, |d| d.bg(p.sel))
                    .child(x.label())
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        card(self.theme, 560.)
            .child(
                div()
                    .p(px(18.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Background"))
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(p.mute)
                            .child("Sits behind the home screen. Moving pictures play too."),
                    )
                    .child(grid),
            )
            .child(
                div()
                    .px(px(18.))
                    .py(px(14.))
                    .border_t_1()
                    .border_color(p.line)
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(
                        div()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("Texture"))
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(p.mute)
                                    .child("Grain drawn over the picture."),
                            ),
                    )
                    .child(seg),
            )
    }
}

/// A mini window in one theme.
fn mini(s: &ThemeSwatch, w: f32) -> gpui::Div {
    let h = w * 9. / 16.;
    let mut side = div().w(relative(0.34)).flex().flex_col().gap(px(4.));
    for k in 0..4 {
        side = side.child(
            div()
                .h(px(4.))
                .w(relative(if k == 0 { 0.95 } else { 0.7 }))
                .rounded(px(2.))
                .bg(hex(s.line)),
        );
    }
    let mut pane = div()
        .flex_1()
        .rounded(px(6.))
        .bg(hex(s.pane))
        .flex()
        .items_center()
        .justify_center()
        .child(
            div()
                .w(relative(0.62))
                .h(px(h * 0.16))
                .rounded(px(4.))
                .bg(hex(s.block)),
        );
    if s.key == "contrast" {
        pane = pane.border_1().border_color(hex(s.line));
    }
    div()
        .w(px(w))
        .h(px(h))
        .p(px(7.))
        .flex()
        .gap(px(8.))
        .rounded(px(10.))
        .bg(hex(s.side))
        .child(side)
        .child(pane)
}

/// `theme-grid` (Theme Selector – Theme grid): eight swatches, the font size row and a sample.
#[derive(IntoElement)]
pub struct ThemeGridEl {
    pick: usize,
    font: usize,
    device_dark: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ThemeGridEl {
    fn default() -> Self {
        Self {
            pick: 0,
            font: 1,
            device_dark: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ThemeGridEl {
    /// The picked swatch (index into `swatches()`).
    pub fn pick(mut self, i: usize) -> Self {
        self.pick = i;
        self
    }
    /// The font size (index into `font_sizes()`).
    pub fn font(mut self, i: usize) -> Self {
        self.font = i;
        self
    }
    /// What System follows.
    pub fn device_dark(mut self, d: bool) -> Self {
        self.device_dark = d;
        self
    }
    /// Light or dark chrome.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `theme-<i>`, `font` (the host opens its menu).
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ThemeGridEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let sws = swatches();
        let fonts = font_sizes();
        let fi = self.font.min(fonts.len() - 1);
        let mut grid = div().flex().flex_wrap().gap(px(10.)).mt(px(12.));
        for (i, s) in sws.iter().enumerate() {
            let f = fire(&self.act, TH_ID[i.min(7)]);
            let shown = if s.key == "system" {
                swatch_for("system", self.device_dark)
            } else {
                s.clone()
            };
            let mut fr = div()
                .rounded(px(10.))
                .border_1()
                .border_color(p.line)
                .child(mini(&shown, 128.));
            if i == self.pick {
                fr = fr.border_2().border_color(p.fg);
            }
            grid = grid.child(
                div()
                    .id(SharedString::from(format!("oxtsg-{i}")))
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .text_size(px(12.))
                    .text_color(if i == self.pick { p.fg } else { p.mute })
                    .child(fr)
                    .child(s.label)
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        let s = swatch_for(sws[self.pick.min(7)].key, self.device_dark);
        let ff = fire(&self.act, "font");
        card(self.theme, 600.)
            .child(
                div()
                    .p(px(18.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Theme"))
                    .child(grid),
            )
            .child(
                div()
                    .px(px(18.))
                    .py(px(12.))
                    .border_t_1()
                    .border_color(p.line)
                    .flex()
                    .items_center()
                    .justify_between()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Font size"))
                    .child(
                        div()
                            .id("oxtsg-font")
                            .w(px(220.))
                            .h(px(38.))
                            .px(px(12.))
                            .rounded(px(10.))
                            .bg(p.card2)
                            .border_1()
                            .border_color(p.line)
                            .flex()
                            .items_center()
                            .justify_between()
                            .child(fonts[fi].label)
                            .child("⌄")
                            .on_click(move |_, w, cx| ff(w, cx)),
                    ),
            )
            .child(
                div()
                    .mx(px(18.))
                    .mb(px(16.))
                    .p(px(8.))
                    .rounded(px(12.))
                    .bg(hex(s.side))
                    .text_color(hex(s.fg))
                    .text_size(px(fonts[fi].px as f32))
                    .flex()
                    .gap(px(8.))
                    .child(div().w(px(110.)).text_color(hex(s.mute)).child("Today"))
                    .child(
                        div()
                            .flex_1()
                            .p(px(12.))
                            .rounded(px(9.))
                            .bg(hex(s.pane))
                            .flex()
                            .flex_col()
                            .gap(px(6.))
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(s.label))
                            .child(
                                div()
                                    .text_color(hex(s.mute))
                                    .child("Three runs finished while you were away."),
                            ),
                    ),
            )
    }
}

/// `theme-split` (Theme Selector – Theme split): two themes over one window, a divider at `split`.
#[derive(IntoElement)]
pub struct ThemeSplitEl {
    current: &'static str,
    candidate: &'static str,
    split: f64,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for ThemeSplitEl {
    fn default() -> Self {
        Self {
            current: "light",
            candidate: "midnight",
            split: 50.,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl ThemeSplitEl {
    /// The two themes (keys).
    pub fn themes(mut self, current: &'static str, candidate: &'static str) -> Self {
        self.current = current;
        self.candidate = candidate;
        self
    }
    /// Divider position in %.
    pub fn split(mut self, v: f64) -> Self {
        self.split = split_clamp(v);
        self
    }
    /// Light or dark chrome.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `cand-<i>`, `apply`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for ThemeSplitEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let a = swatch_for(self.current, false);
        let b = swatch_for(self.candidate, true);
        let w: f32 = 526.;
        let left = w * (self.split as f32) / 100.;
        let half = |s: &ThemeSwatch, wd: f32| {
            div()
                .w(px(wd))
                .h_full()
                .bg(hex(s.pane))
                .text_color(hex(s.fg))
                .p(px(14.))
                .overflow_hidden()
                .child(div().font_weight(FontWeight::SEMIBOLD).child(s.label))
                .child(div().text_color(hex(s.mute)).child("Release notes"))
        };
        let mut dots = div().flex().gap(px(6.));
        for (i, s) in swatches().iter().skip(1).enumerate() {
            let f = fire(&self.act, CAND_ID[i.min(7)]);
            let mut d = div()
                .id(SharedString::from(format!("oxtpv-{i}")))
                .size(px(22.))
                .rounded(px(11.))
                .bg(hex(s.pane))
                .border_1()
                .border_color(p.line)
                .on_click(move |_, w, cx| f(w, cx));
            if s.key == b.key {
                d = d.border_2().border_color(p.fg);
            }
            dots = dots.child(d);
        }
        let fa = fire(&self.act, "apply");
        card(self.theme, 560.)
            .p(px(16.))
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Compare themes"),
                    )
                    .child(dots),
            )
            .child(
                div()
                    .mt(px(10.))
                    .h(px(220.))
                    .rounded(px(12.))
                    .overflow_hidden()
                    .flex()
                    .child(half(&a, left))
                    .child(div().w(px(2.)).h_full().bg(hex("FFFFFF")))
                    .child(half(&b, w - left)),
            )
            .child(
                div().flex().justify_end().mt(px(12.)).child(
                    div()
                        .id("oxtpv-apply")
                        .h(px(32.))
                        .px(px(14.))
                        .rounded(px(16.))
                        .bg(p.fg)
                        .text_color(p.card)
                        .flex()
                        .items_center()
                        .child(format!("Use {}", b.label))
                        .on_click(move |_, w, cx| fa(w, cx)),
                ),
            )
    }
}

/// `accent-density` (Theme Selector – Accent and density): accent, density and corners beside a
/// settings sample.
#[derive(IntoElement)]
pub struct AccentDensityEl {
    accent: usize,
    density: usize,
    corner: usize,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AccentDensityEl {
    fn default() -> Self {
        Self {
            accent: 1,
            density: 1,
            corner: 1,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AccentDensityEl {
    /// The choices (indices into `accents()`, `densities()`, `corners()`).
    pub fn picks(mut self, accent: usize, density: usize, corner: usize) -> Self {
        self.accent = accent;
        self.density = density;
        self.corner = corner;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `acc-<i>`, `den-<i>`, `cor-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for AccentDensityEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let acs = accents();
        let ds = densities();
        let cs = corners();
        let ac = hex(acs[self.accent.min(5)].hex);
        let d = ds[self.density.min(2)].clone();
        let r = cs[self.corner.min(2)].2 as f32;
        let mut dots = div().flex().gap(px(8.));
        for (i, a) in acs.iter().enumerate() {
            let f = fire(&self.act, ACC_ID[i]);
            let mut x = div()
                .id(SharedString::from(format!("oxadn-a{i}")))
                .size(px(24.))
                .rounded(px(12.))
                .bg(hex(a.hex))
                .on_click(move |_, w, cx| f(w, cx));
            if i == self.accent {
                x = x.border_2().border_color(p.fg);
            }
            dots = dots.child(x);
        }
        let seg = |labels: Vec<&'static str>, on: usize, ids: &[&'static str; 3], tag: &str| {
            let mut s = div()
                .flex()
                .gap(px(2.))
                .p(px(3.))
                .rounded(px(10.))
                .bg(p.inset);
            for (i, l) in labels.into_iter().enumerate() {
                let f = fire(&self.act, ids[i]);
                s = s.child(
                    div()
                        .id(SharedString::from(format!("oxadn-{tag}{i}")))
                        .h(px(28.))
                        .px(px(8.))
                        .flex()
                        .items_center()
                        .rounded(px(7.))
                        .text_size(px(12.))
                        .text_color(if i == on { p.fg } else { p.mute })
                        .when(i == on, |x| x.bg(p.sel))
                        .child(l)
                        .on_click(move |_, w, cx| f(w, cx)),
                );
            }
            s
        };
        let mut pv = div()
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(d.gap as f32))
            .p(px(12.))
            .rounded(px(r + 4.))
            .bg(p.inset);
        for (i, t) in ["Notifications", "Sounds", "Auto-save"].iter().enumerate() {
            let on = i != 1;
            pv = pv.child(
                div()
                    .h(px(d.row as f32))
                    .px(px(d.pad as f32))
                    .rounded(px(r))
                    .bg(p.card)
                    .flex()
                    .items_center()
                    .justify_between()
                    .when(i == 0, |x| x.border_1().border_color(ac))
                    .child(*t)
                    .child(div().w(px(34.)).h(px(20.)).rounded(px(10.)).bg(if on {
                        ac
                    } else {
                        p.off
                    })),
            );
        }
        pv = pv.child(
            div().flex().justify_end().child(
                div()
                    .h(px(30.))
                    .px(px(14.))
                    .rounded(px(r))
                    .bg(ac)
                    .text_color(hex("FFFFFF"))
                    .flex()
                    .items_center()
                    .child("Save"),
            ),
        );
        card(self.theme, 600.)
            .p(px(18.))
            .flex()
            .gap(px(18.))
            .child(
                div()
                    .w(px(200.))
                    .flex()
                    .flex_col()
                    .gap(px(14.))
                    .child("Accent")
                    .child(dots)
                    .child("Density")
                    .child(seg(
                        ds.iter().map(|x| x.label).collect(),
                        self.density,
                        &DEN_ID,
                        "d",
                    ))
                    .child("Corners")
                    .child(seg(
                        cs.iter().map(|x| x.1).collect(),
                        self.corner,
                        &COR_ID,
                        "c",
                    )),
            )
            .child(pv)
    }
}
