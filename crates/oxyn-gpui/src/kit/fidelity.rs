//! affectUI fidelity pass (plan #32) for GPUI: the exact Chat Composer, Comment Thread and
//! Comments Panel, drawn in affectUI's own greys (dark and light), over the
//! `oxyn_core::kit::fidelity` models. Elements are `RenderOnce`; the host keeps the state,
//! drives the clock and routes [`Act`] ids and [`Pick`] indices back to it. Brand marks come from
//! [`crate::brand_mark`] (serve [`crate::BrandAssets`]).
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, rgb, rgba,
};
use oxyn_core::kit::fidelity::{
    Approval, Attachment, EditRun, FileKind, PanelComment, PanelFilter, ago, is_edit, open_edits,
    panel_view, split_command,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Pick, c};
use crate::brand_mark;

/// affectUI's greys for one theme.
#[derive(Clone, Copy)]
struct Pal {
    band: Hsla,
    card: Hsla,
    line: Hsla,
    fg: Hsla,
    ph: Hsla,
    label: Hsla,
    hov: Hsla,
    menu: Hsla,
    warn: Hsla,
    warn_bg: Hsla,
    send: Hsla,
    send_fg: Hsla,
    off: Hsla,
    off_fg: Hsla,
    ok: Hsla,
    outer: Hsla,
    c1: Hsla,
    tray: Hsla,
    wave: Hsla,
}

fn pal(t: &Theme) -> Pal {
    let dark = c(t, ColorRole::BgBase).l < 0.5;
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if dark {
        Pal {
            band: h(0x161618),
            card: h(0x262628),
            line: h(0x2E2E31),
            fg: h(0xF4F4F5),
            ph: h(0x747476),
            label: h(0xA1A1A3),
            hov: rgba(0xffffff10).into(),
            menu: h(0x232326),
            warn: h(0xE08A46),
            warn_bg: rgba(0xE0803B2B).into(),
            send: h(0xECECEE),
            send_fg: h(0x111113),
            off: h(0x2F2F31),
            off_fg: h(0x6B6B6E),
            ok: h(0x22A05B),
            outer: h(0x19191B),
            c1: h(0x1C1C1E),
            tray: h(0x161618),
            wave: h(0x8AB8F0),
        }
    } else {
        Pal {
            band: h(0xF4F4F5),
            card: h(0xFFFFFF),
            line: h(0xE4E4E7),
            fg: h(0x18181B),
            ph: h(0xA1A1AA),
            label: h(0x52525B),
            hov: rgba(0x0000000d).into(),
            menu: h(0xFFFFFF),
            warn: h(0xC2560F),
            warn_bg: h(0xFFF1E6),
            send: h(0x18181B),
            send_fg: h(0xFFFFFF),
            off: h(0xE9E9EB),
            off_fg: h(0xA1A1AA),
            ok: h(0x16A34A),
            outer: h(0xF7F7F8),
            c1: h(0xFFFFFF),
            tray: h(0xF4F4F5),
            wave: h(0x3D74D0),
        }
    }
}

fn hex(s: &str) -> Hsla {
    rgb(u32::from_str_radix(s, 16).unwrap_or(0)).into()
}

/// A file-type badge: a rounded tile in the type's colour with its short label.
pub fn file_badge(kind: FileKind, size: f32) -> AnyElement {
    let label = match kind {
        FileKind::Image => "IMG",
        FileKind::Folder => "",
        k => k.badge(),
    };
    div()
        .flex_none()
        .size(px(size))
        .rounded(px(size * 0.24))
        .bg(hex(kind.hex()))
        .flex()
        .items_center()
        .justify_center()
        .text_color(gpui::white())
        .text_size(px(size * 0.3))
        .font_weight(FontWeight::BOLD)
        .child(label)
        .into_any_element()
}

fn ibtn(id: &'static str, glyph: &'static str, p: Pal, act: Option<Act>) -> impl IntoElement {
    div()
        .id(id)
        .flex_none()
        .size(px(32.))
        .rounded(px(9.))
        .flex()
        .items_center()
        .justify_center()
        .text_color(p.label)
        .text_size(px(17.))
        .cursor_pointer()
        .hover(move |s| s.bg(p.hov).text_color(p.fg))
        .child(glyph)
        .when_some(act, move |d, f| d.on_click(move |_, w, cx| f(id, w, cx)))
}

fn menu_box(p: Pal) -> gpui::Div {
    div()
        .absolute()
        .flex()
        .flex_col()
        .p(px(5.))
        .rounded(px(14.))
        .border_1()
        .border_color(p.line)
        .bg(p.menu)
        .shadow_lg()
}

fn menu_row(
    id: SharedString,
    lead: AnyElement,
    title: SharedString,
    desc: Option<SharedString>,
    checked: bool,
    p: Pal,
    on: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex()
        .items_center()
        .gap(px(10.))
        .min_h(px(34.))
        .px(px(9.))
        .py(px(6.))
        .rounded(px(9.))
        .cursor_pointer()
        .hover(move |s| s.bg(p.hov))
        .child(lead)
        .child(
            div()
                .flex()
                .flex_col()
                .flex_1()
                .min_w_0()
                .child(div().text_size(px(14.)).text_color(p.fg).child(title))
                .when_some(desc, |d, x| {
                    d.child(div().text_size(px(12.5)).text_color(p.ph).child(x))
                }),
        )
        .when(checked, |d| d.child(div().text_color(p.fg).child("✓")))
        .on_click(move |_, w, cx| on(w, cx))
}

fn plus_row(
    id: &'static str,
    lead: AnyElement,
    title: &'static str,
    hint: &'static str,
    checked: bool,
    p: Pal,
    on: impl Fn(&mut Window, &mut App) + 'static,
) -> impl IntoElement {
    div()
        .id(id)
        .flex_none()
        .flex()
        .items_center()
        .gap(px(12.))
        .h(px(39.))
        .px(px(10.))
        .rounded(px(9.))
        .cursor_pointer()
        .hover(move |s| s.bg(p.hov))
        .child(lead)
        .child(div().text_size(px(16.)).text_color(p.fg).child(title))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .text_size(px(16.))
                .text_color(p.ph)
                .child(hint),
        )
        .when(checked, |d| d.child(div().text_color(p.fg).child("●")))
        .on_click(move |_, w, cx| on(w, cx))
}

/// `chat-composer` (affectUI Chat Composer): field, + drop-up with file types and tools, the
/// approval chip (orange **Unrestricted access** for Full), the model menu with coloured marks,
/// mic → recording (trace, ✕ discard, ✓ keep), white send, and the folder / Connect apps tray.
/// Acts: `plus`, `approval`, `model`, `mic`, `send`, `discard`, `keep`, `folder`, `apps`,
/// `approval:ask|auto|full`, `attach:pdf|image|sheet|doc|code|folder`, `tool:goal|plan|skill|draw`;
/// model, folder and app picks arrive through [`Pick`].
#[derive(IntoElement)]
pub struct ChatComposerEl {
    text: SharedString,
    placeholder: SharedString,
    models: Vec<String>,
    model: usize,
    approval: Approval,
    tray: bool,
    folder: Option<String>,
    folders: Vec<String>,
    apps: Vec<(String, String)>,
    connected: Vec<String>,
    attachments: Vec<Attachment>,
    recording: Option<(String, Vec<f32>)>,
    open: Option<&'static str>,
    plan: bool,
    classic: bool,
    act: Option<Act>,
    pick: Option<Pick>,
    theme: &'static Theme,
}

impl ChatComposerEl {
    /// Classic style: the writing card inset on a darker band (Clean, the default, is one
    /// continuous container with the tray under a hairline).
    pub fn classic(mut self, on: bool) -> Self {
        self.classic = on;
        self
    }
    /// A composer with its text and model list (the first is picked).
    pub fn new(text: impl Into<SharedString>, models: Vec<String>) -> Self {
        Self {
            text: text.into(),
            placeholder: "Start by typing…".into(),
            models,
            model: 0,
            approval: Approval::Ask,
            tray: true,
            folder: None,
            folders: ["Q3 Campaign", "Brand assets", "Hiring", "Research"]
                .map(String::from)
                .to_vec(),
            apps: [
                ("gmail", "Gmail"),
                ("slack", "Slack"),
                ("clickup", "ClickUp"),
            ]
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .to_vec(),
            connected: vec!["gmail".into()],
            attachments: vec![],
            recording: None,
            open: None,
            plan: false,
            classic: false,
            act: None,
            pick: None,
            theme: &Theme::DARK,
        }
    }
    /// Picked model, approval mode, folder, connected apps, attachments and plan mode.
    #[allow(clippy::too_many_arguments)]
    pub fn state(
        mut self,
        model: usize,
        approval: Approval,
        folder: Option<String>,
        connected: Vec<String>,
        attachments: Vec<Attachment>,
        plan: bool,
    ) -> Self {
        self.model = model;
        self.approval = approval;
        self.folder = folder;
        self.connected = connected;
        self.attachments = attachments;
        self.plan = plan;
        self
    }
    /// Recording: the words heard so far and the trace levels (0..1).
    pub fn recording(mut self, heard: impl Into<String>, trace: Vec<f32>) -> Self {
        self.recording = Some((heard.into(), trace));
        self
    }
    /// The open menu: `plus`, `approval`, `model`, `folder` or `apps`.
    pub fn open(mut self, menu: Option<&'static str>) -> Self {
        self.open = menu;
        self
    }
    /// Show or hide the tray (off once a chat has started).
    pub fn tray(mut self, on: bool) -> Self {
        self.tray = on;
        self
    }
    /// Folders and apps (id, name).
    pub fn options(mut self, folders: Vec<String>, apps: Vec<(String, String)>) -> Self {
        self.folders = folders;
        self.apps = apps;
        self
    }
    /// Placeholder.
    pub fn placeholder(mut self, p: impl Into<SharedString>) -> Self {
        self.placeholder = p.into();
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Button and menu acts.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
    /// Model (0..), folder (100 + i) and app (200 + i) picks.
    pub fn on_pick(mut self, p: Pick) -> Self {
        self.pick = Some(p);
        self
    }
}

/// The plus menu, as affectUI: (act, glyph, label, hint); the Plugins group follows.
const PLUS: [(&str, &str, &str, &str); 7] = [
    ("attach:files", "⌇", "Files and folders", ""),
    ("attach:claude", "", "Attach Claude", ""),
    (
        "attach:folder",
        "▭",
        "Work in a folder",
        "Choose a folder for this chat",
    ),
    (
        "tool:goal",
        "◎",
        "Goal",
        "Set a goal to keep working toward",
    ),
    ("tool:plan", "◌", "Plan mode", "Turn on plan mode"),
    ("tool:skill", "◉", "Record a skill", ""),
    ("tool:draw", "∿", "Draw", "Sketch a drawing"),
];
const PLUGINS: [(&str, FileKind, &str, &str); 2] = [
    (
        "attach:documents",
        FileKind::Doc,
        "Documents",
        "Create and edit documents",
    ),
    (
        "attach:pdf",
        FileKind::Pdf,
        "PDF",
        "Read, create and verify PDFs",
    ),
];

impl RenderOnce for ChatComposerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let a = self.act.clone();
        let pick = self.pick.clone();
        let fire = move |id: &'static str| {
            let a = a.clone();
            move |w: &mut Window, cx: &mut App| {
                if let Some(f) = &a {
                    f(id, w, cx)
                }
            }
        };
        let cur = self.models.get(self.model).cloned().unwrap_or_default();
        let short = cur.strip_prefix("Claude ").unwrap_or(&cur).to_string();
        let can_send = !self.text.is_empty() || !self.attachments.is_empty();
        let full = self.approval == Approval::Full;
        let rec = self.recording.clone();

        let mut card = div()
            .relative()
            .flex()
            .flex_col()
            .gap(px(6.))
            .pt(px(14.))
            .pb(px(10.))
            .px(px(13.))
            .rounded(px(18.))
            .border_1()
            .border_color(p.line)
            .bg(p.card);
        if !self.attachments.is_empty() {
            card = card.child(div().flex().flex_wrap().gap(px(6.)).children(
                self.attachments.iter().map(|at| {
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .h(px(44.))
                        .pl(px(8.))
                        .pr(px(12.))
                        .rounded(px(12.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.off)
                        .child(file_badge(at.kind, 26.))
                        .child(
                            div()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .text_size(px(13.))
                                        .text_color(p.fg)
                                        .child(at.name.clone()),
                                )
                                .child(
                                    div()
                                        .text_size(px(11.5))
                                        .text_color(p.ph)
                                        .child(at.kind.label()),
                                ),
                        )
                }),
            ));
        }
        card = card.child(match &rec {
            Some((heard, _)) => {
                div()
                    .text_size(px(17.))
                    .text_color(p.ph)
                    .child(if heard.is_empty() {
                        "Listening…".to_string()
                    } else {
                        heard.clone()
                    })
            }
            None => div()
                .text_size(px(17.))
                .text_color(if self.text.is_empty() { p.ph } else { p.fg })
                .child(if self.text.is_empty() {
                    self.placeholder.clone()
                } else {
                    self.text.clone()
                }),
        });

        let chip = div()
            .id("cc-approval")
            .flex()
            .items_center()
            .gap(px(7.))
            .h(px(32.))
            .px(px(9.))
            .rounded(px(9.))
            .text_size(px(14.5))
            .cursor_pointer()
            .when(full, |d| d.bg(p.warn_bg).text_color(p.warn))
            .when(!full, |d| {
                d.text_color(p.label)
                    .hover(move |s| s.bg(p.hov).text_color(p.fg))
            })
            .child(match self.approval {
                Approval::Ask => "✋",
                Approval::Auto => "✓",
                Approval::Full => "⛉",
            })
            .child(self.approval.chip())
            .on_click({
                let f = fire("approval");
                move |_, w, cx| f(w, cx)
            });

        let mut bar = div()
            .relative()
            .flex()
            .items_center()
            .gap(px(4.))
            .child(ibtn("plus", "+", p, self.act.clone()))
            .child(chip)
            .child(div().flex_1());
        bar = match &rec {
            Some((_, trace)) => bar
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(2.2))
                        .h(px(34.))
                        .px(px(10.))
                        .children(trace.iter().map(|l| {
                            div()
                                .w(px(3.5))
                                .h(px((l * 16.).max(3.5)))
                                .rounded(px(2.))
                                .bg(p.wave)
                        })),
                )
                .child(ibtn("discard", "✕", p, self.act.clone()))
                .child(ibtn("keep", "✓", p, self.act.clone())),
            None => bar
                .child(
                    div()
                        .id("cc-model")
                        .flex()
                        .items_center()
                        .gap(px(4.))
                        .h(px(32.))
                        .px(px(8.))
                        .rounded(px(9.))
                        .text_size(px(14.5))
                        .text_color(p.label)
                        .cursor_pointer()
                        .hover(move |s| s.bg(p.hov).text_color(p.fg))
                        .child(short)
                        .child("⌄")
                        .on_click({
                            let f = fire("model");
                            move |_, w, cx| f(w, cx)
                        }),
                )
                .child(ibtn("mic", "🎙", p, self.act.clone()))
                .child(
                    div()
                        .id("send")
                        .size(px(32.))
                        .rounded(px(9.))
                        .flex()
                        .items_center()
                        .justify_center()
                        .bg(if can_send { p.send } else { p.off })
                        .text_color(if can_send { p.send_fg } else { p.off_fg })
                        .child("↑")
                        .when(can_send, |d| {
                            d.cursor_pointer().on_click({
                                let f = fire("send");
                                move |_, w, cx| f(w, cx)
                            })
                        }),
                ),
        };

        // Menus.
        let open = self.open;
        if open == Some("plus") {
            // A capped, scrolling list spanning the composer.
            let mut list = div()
                .id("cc-plus-list")
                .flex()
                .flex_col()
                .max_h(px(144.))
                .overflow_y_scroll()
                .child(
                    div()
                        .px(px(9.))
                        .py(px(6.))
                        .text_size(px(14.))
                        .text_color(p.ph)
                        .child("Add"),
                );
            for (id, g, l, hint) in PLUS {
                let f = fire(id);
                let on = id == "tool:plan" && self.plan;
                let lead = if id == "attach:claude" {
                    brand_mark("claude", 17., p.label)
                        .unwrap_or_else(|| div().w(px(17.)).into_any_element())
                } else {
                    div()
                        .w(px(18.))
                        .text_color(if id == "tool:draw" {
                            hex("14B8A6")
                        } else {
                            p.label
                        })
                        .child(g)
                        .into_any_element()
                };
                list = list.child(plus_row(id, lead, l, hint, on, p, f));
                if id == "attach:folder" {
                    list = list.child(div().h(px(1.)).my(px(5.)).bg(p.line));
                }
            }
            list = list.child(div().h(px(1.)).my(px(5.)).bg(p.line)).child(
                div()
                    .px(px(9.))
                    .py(px(6.))
                    .text_size(px(14.))
                    .text_color(p.ph)
                    .child("Plugins"),
            );
            for (id, k, l, hint) in PLUGINS {
                let f = fire(id);
                list = list.child(plus_row(id, file_badge(k, 17.), l, hint, false, p, f));
            }
            bar = bar.child(
                menu_box(p)
                    .bottom(px(60.))
                    .left(px(-14.))
                    .right(px(-13.))
                    .p(px(6.))
                    .child(list),
            );
        }
        if open == Some("approval") {
            let mut m = menu_box(p).bottom(px(40.)).left(px(30.)).w(px(352.));
            m = m.child(
                div()
                    .px(px(9.))
                    .py(px(5.))
                    .text_size(px(12.5))
                    .text_color(p.ph)
                    .child("How should actions be approved?"),
            );
            for mode in Approval::ALL {
                let id: &'static str = match mode {
                    Approval::Ask => "approval:ask",
                    Approval::Auto => "approval:auto",
                    Approval::Full => "approval:full",
                };
                let f = fire(id);
                let lead = div()
                    .w(px(18.))
                    .text_color(if mode == Approval::Full {
                        p.warn
                    } else {
                        p.label
                    })
                    .child(match mode {
                        Approval::Ask => "✋",
                        Approval::Auto => "✓",
                        Approval::Full => "⛉",
                    })
                    .into_any_element();
                m = m.child(menu_row(
                    id.into(),
                    lead,
                    mode.label().into(),
                    Some(mode.desc().into()),
                    mode == self.approval,
                    p,
                    f,
                ));
            }
            bar = bar.child(m);
        }
        if open == Some("model") {
            let mut m = div()
                .id("cc-model-list")
                .flex()
                .flex_col()
                .max_h(px(234.))
                .overflow_y_scroll();
            for (i, name) in self.models.iter().enumerate() {
                let pk = pick.clone();
                let lead = if name == "Auto" {
                    div()
                        .w(px(18.))
                        .text_color(p.label)
                        .child("✦")
                        .into_any_element()
                } else {
                    brand_mark(name, 16., p.label)
                        .unwrap_or_else(|| div().w(px(16.)).into_any_element())
                };
                m = m.child(menu_row(
                    SharedString::from(format!("cc-m{i}")),
                    lead,
                    name.clone().into(),
                    None,
                    i == self.model,
                    p,
                    move |w, cx| {
                        if let Some(f) = &pk {
                            f(i, w, cx)
                        }
                    },
                ));
            }
            bar = bar.child(
                menu_box(p)
                    .bottom(px(40.))
                    .right(px(30.))
                    .w(px(316.))
                    .p(px(6.))
                    .child(m),
            );
        }
        card = card.child(bar);

        // Clean (affectUI default): one continuous container, the tray a darker band under a
        // hairline; Classic: the card inset on the band.
        let classic = self.classic;
        let card = if classic {
            card
        } else {
            card.border_0()
                .bg(gpui::transparent_black())
                .rounded(px(17.))
        };
        let mut root = div()
            .relative()
            .w(px(680.))
            .flex()
            .flex_col()
            .when(classic, |d| {
                d.rounded(px(22.)).bg(p.band).pt(px(12.)).px(px(12.))
            })
            .when(!classic, |d| {
                d.rounded(px(18.)).border_1().border_color(p.line).bg(p.c1)
            })
            .child(card);
        if self.tray {
            let mut tray = div()
                .relative()
                .flex()
                .items_center()
                .gap(px(4.))
                .h(px(40.))
                .px(px(7.))
                .text_size(px(15.))
                .text_color(p.label)
                .when(!classic, |d| {
                    d.border_t_1()
                        .border_color(p.line)
                        .bg(p.tray)
                        .rounded_b(px(17.))
                });
            if !self.folders.is_empty() {
                tray = tray.child(
                    div()
                        .id("folder")
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .h(px(30.))
                        .px(px(8.))
                        .rounded(px(8.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(p.hov).text_color(p.fg))
                        .child(file_badge(FileKind::Folder, 17.))
                        .child(
                            self.folder
                                .clone()
                                .unwrap_or_else(|| "Work in a folder".into()),
                        )
                        .on_click({
                            let f = fire("folder");
                            move |_, w, cx| f(w, cx)
                        }),
                );
            }
            tray = tray.child(div().flex_1()).child(
                div()
                    .id("apps")
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(30.))
                    .px(px(8.))
                    .rounded(px(8.))
                    .cursor_pointer()
                    .hover(move |s| s.bg(p.hov).text_color(p.fg))
                    .child("Connect apps")
                    .child(
                        div().flex().gap(px(5.)).children(
                            self.apps
                                .iter()
                                .take(3)
                                .filter_map(|(_, n)| brand_mark(n, 15., p.label)),
                        ),
                    )
                    .on_click({
                        let f = fire("apps");
                        move |_, w, cx| f(w, cx)
                    }),
            );
            if open == Some("folder") {
                let mut m = menu_box(p).top(px(40.)).left(px(6.)).w(px(220.));
                for (i, f) in self.folders.iter().enumerate() {
                    let pk = pick.clone();
                    m = m.child(menu_row(
                        SharedString::from(format!("cc-f{i}")),
                        file_badge(FileKind::Folder, 16.),
                        f.clone().into(),
                        None,
                        self.folder.as_deref() == Some(f.as_str()),
                        p,
                        move |w, cx| {
                            if let Some(g) = &pk {
                                g(100 + i, w, cx)
                            }
                        },
                    ));
                }
                tray = tray.child(m);
            }
            if open == Some("apps") {
                let mut m = menu_box(p).top(px(40.)).right(px(6.)).w(px(210.));
                for (i, (id, n)) in self.apps.iter().enumerate() {
                    let pk = pick.clone();
                    let lead = brand_mark(n, 16., p.label)
                        .unwrap_or_else(|| div().w(px(16.)).into_any_element());
                    m = m.child(menu_row(
                        SharedString::from(format!("cc-a{i}")),
                        lead,
                        n.clone().into(),
                        None,
                        self.connected.contains(id),
                        p,
                        move |w, cx| {
                            if let Some(g) = &pk {
                                g(200 + i, w, cx)
                            }
                        },
                    ));
                }
                tray = tray.child(m);
            }
            root = root.child(tray);
        }
        root
    }
}

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
