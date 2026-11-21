//! Attachments family (variants 2) for GPUI over `oxyn_core::kit::attachfid`: **Upload
//! dropzone** ([`UploadDropzoneEl`]), **Attachment chips** ([`AttachmentChipsEl`]), **Polaroid
//! stack** ([`PolaroidStackEl`]) and **Drop lanes** ([`DropLanesEl`]). `RenderOnce` static
//! snapshots: the host keeps the files and routes [`Act`] ids back (`browse`, `file-<i>`,
//! `remove-<i>`, `retry-<i>`, `lane-<i>`).

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px,
};
use oxyn_core::kit::attachfid::{DemoFile, LANES, demo_files, fan, file_badge, fmt_bytes};
use oxyn_tokens::Theme;

use super::Act;
use super::ctxvisfid::hex;
use super::rulerfid::{ThPal, fire, th_pal};

const FILE_ID: [&str; 8] = [
    "file-0", "file-1", "file-2", "file-3", "file-4", "file-5", "file-6", "file-7",
];
const LANE_ID: [&str; 3] = ["lane-0", "lane-1", "lane-2"];

/// A page icon with its coloured badge (GPUI has no inline SVG text, so the badge is a pill).
fn file_icon(p: &ThPal, name: &str, h: f32) -> gpui::Div {
    let (badge, c) = file_badge(name);
    div()
        .relative()
        .w(px(h * 0.8))
        .h(px(h))
        .rounded(px(3.))
        .border_1()
        .border_color(p.mute)
        .bg(p.card)
        .child(
            div()
                .absolute()
                .left(px(-2.))
                .bottom(px(h * 0.08))
                .px(px(3.))
                .rounded(px(3.))
                .bg(hex(c))
                .text_size(px((h * 0.2).max(7.)))
                .font_weight(FontWeight::BOLD)
                .text_color(gpui::white())
                .child(badge),
        )
}

fn blue(t: &Theme) -> Hsla {
    if th_pal(t).card.l < 0.5 {
        hex("4C7DFF")
    } else {
        hex("2F5BEA")
    }
}

/// `upload-dropzone` (Attachments – Upload dropzone): box and sample files.
#[derive(IntoElement)]
pub struct UploadDropzoneEl {
    files: Vec<DemoFile>,
    max_mb: f64,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for UploadDropzoneEl {
    fn default() -> Self {
        let d = demo_files();
        Self {
            files: vec![d[0].clone(), d[1].clone(), d[4].clone()],
            max_mb: 10.0,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl UploadDropzoneEl {
    /// Size limit in MB.
    pub fn max_mb(mut self, m: f64) -> Self {
        self.max_mb = m;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `browse`, `file-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for UploadDropzoneEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let fb = fire(&self.act, "browse");
        let mut chips = div().flex().gap(px(26.)).justify_center();
        for (i, f) in self.files.iter().enumerate().take(8) {
            let fc = fire(&self.act, FILE_ID[i]);
            chips = chips.child(
                div()
                    .id(SharedString::from(format!("oxud-{i}")))
                    .flex()
                    .flex_col()
                    .items_center()
                    .gap(px(6.))
                    .child(file_icon(&p, f.name, 44.))
                    .child(div().text_size(px(12.)).text_color(p.mute).child(f.name))
                    .on_click(move |_, w, cx| fc(w, cx)),
            );
        }
        div()
            .w(px(340.))
            .flex()
            .flex_col()
            .items_center()
            .gap(px(18.))
            .text_color(p.fg)
            .child(
                div()
                    .id("oxud-box")
                    .w_full()
                    .h(px(118.))
                    .rounded(px(20.))
                    .bg(p.sel)
                    .flex()
                    .flex_col()
                    .items_center()
                    .justify_center()
                    .gap(px(4.))
                    .child(div().size(px(44.)).rounded(px(14.)).bg(p.line))
                    .child(
                        div()
                            .text_size(px(15.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Drop a file here"),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(p.mute)
                            .child(format!("Up to {}", fmt_bytes((self.max_mb * 1e6) as u64))),
                    )
                    .on_click(move |_, w, cx| fb(w, cx)),
            )
            .child(chips)
    }
}

/// `attachment-chips` (Attachments – Attachment chips): chips with type and size.
#[derive(IntoElement)]
pub struct AttachmentChipsEl {
    files: Vec<DemoFile>,
    uploading: Option<usize>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for AttachmentChipsEl {
    fn default() -> Self {
        let d = demo_files();
        Self {
            files: vec![d[0].clone(), d[1].clone(), d[2].clone(), d[3].clone()],
            uploading: Some(1),
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl AttachmentChipsEl {
    /// Files.
    pub fn files(mut self, f: Vec<DemoFile>) -> Self {
        self.files = f;
        self
    }
    /// Which chip shows a progress ring.
    pub fn uploading(mut self, i: Option<usize>) -> Self {
        self.uploading = i;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `file-<i>` (remove).
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for AttachmentChipsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut tray = div().flex().flex_wrap().gap(px(8.)).pb(px(8.));
        for (i, f) in self.files.iter().enumerate().take(8) {
            let fc = fire(&self.act, FILE_ID[i]);
            let (badge, _) = file_badge(f.name);
            let up = self.uploading == Some(i);
            tray = tray.child(
                div()
                    .id(SharedString::from(format!("oxac-{i}")))
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .h(px(46.))
                    .px(px(8.))
                    .rounded(px(12.))
                    .bg(p.inset)
                    .border_1()
                    .border_color(if up { blue(self.theme) } else { p.line })
                    .child(file_icon(&p, f.name, 30.))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().text_size(px(13.)).child(f.name))
                            .child(div().text_size(px(11.5)).text_color(p.mute).child(if up {
                                "Uploading".to_string()
                            } else {
                                format!("{badge} · {}", fmt_bytes(f.bytes))
                            })),
                    )
                    .on_click(move |_, w, cx| fc(w, cx)),
            );
        }
        div()
            .w(px(520.))
            .p(px(10.))
            .rounded(px(20.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.fg)
            .child(tray)
            .child(
                div()
                    .border_t_1()
                    .border_color(p.line)
                    .p(px(8.))
                    .text_color(p.mute)
                    .child("Ask about these files…"),
            )
    }
}

/// `polaroid-stack` (Attachments – Polaroid stack, Oxyn original): prints, closed or fanned.
#[derive(IntoElement)]
pub struct PolaroidStackEl {
    count: usize,
    open: bool,
    theme: &'static Theme,
}

impl Default for PolaroidStackEl {
    fn default() -> Self {
        Self {
            count: 4,
            open: true,
            theme: &Theme::DARK,
        }
    }
}

impl PolaroidStackEl {
    /// Fanned out.
    pub fn open(mut self, o: bool) -> Self {
        self.open = o;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
}

impl RenderOnce for PolaroidStackEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let tints = ["9CC9F5", "F6B49A", "7D86C7", "BFE3D6", "E3B8E8", "F3D27A"];
        let n = self.count.clamp(1, 6);
        let mut stage = div().relative().w(px(340.)).h(px(210.));
        for (i, tint) in tints.iter().enumerate().take(n) {
            // GPUI draws no rotation here: the fan becomes a horizontal spread.
            let (_, x) = fan(i, n, self.open, 36.);
            stage = stage.child(
                div()
                    .absolute()
                    .top(px(30.))
                    .left(px(104. + x))
                    .w(px(132.))
                    .p(px(8.))
                    .pb(px(26.))
                    .rounded(px(6.))
                    .bg(p.card)
                    .border_1()
                    .border_color(p.line)
                    .child(div().w_full().h(px(116.)).rounded(px(3.)).bg(hex(tint))),
            );
        }
        div().text_color(p.fg).child(stage)
    }
}

/// `drop-lanes` (Attachments – Drop lanes, Oxyn original): three lanes and their counts.
#[derive(IntoElement)]
pub struct DropLanesEl {
    counts: [usize; 3],
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for DropLanesEl {
    fn default() -> Self {
        Self {
            counts: [1, 0, 1],
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl DropLanesEl {
    /// Files in each lane.
    pub fn counts(mut self, c: [usize; 3]) -> Self {
        self.counts = c;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `lane-<i>`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

impl RenderOnce for DropLanesEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let mut lanes = div().flex().gap(px(8.));
        for (l, (_, label, hint)) in LANES.iter().enumerate() {
            let f = fire(&self.act, LANE_ID[l]);
            lanes = lanes.child(
                div()
                    .id(SharedString::from(format!("oxdl-{l}")))
                    .flex_1()
                    .h(px(120.))
                    .p(px(8.))
                    .rounded(px(14.))
                    .border_1()
                    .border_color(p.mute)
                    .child(
                        div()
                            .flex()
                            .justify_between()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(*label)
                            .child(self.counts[l].to_string()),
                    )
                    .child(div().text_size(px(11.5)).text_color(p.mute).child(*hint))
                    .on_click(move |_, w, cx| f(w, cx)),
            );
        }
        div()
            .w(px(560.))
            .p(px(14.))
            .rounded(px(20.))
            .bg(p.card)
            .border_1()
            .border_color(p.line)
            .text_color(p.fg)
            .text_size(px(13.))
            .child(
                div()
                    .mb(px(12.))
                    .text_color(p.mute)
                    .child("Drag a file to a lane, or press it"),
            )
            .child(lanes)
    }
}
