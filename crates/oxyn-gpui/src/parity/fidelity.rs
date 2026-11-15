//! affectUI fidelity pass (plan #32) for GPUI: the exact Chat Composer, Comment Thread and
//! Comments Panel, drawn in affectUI's own greys (dark and light), over the
//! `oxyn_core::parity::fidelity` models. Elements are `RenderOnce`; the host keeps the state,
//! drives the clock and routes [`Act`] ids and [`Pick`] indices back to it. Brand marks come from
//! [`crate::brand_mark`] (serve [`crate::BrandAssets`]).

use gpui::{
    AnyElement, App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window, div, prelude::*, px, rgb, rgba,
};
use oxyn_core::parity::fidelity::{
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
    act: Option<Act>,
    pick: Option<Pick>,
    theme: &'static Theme,
}

impl ChatComposerEl {
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

const ATTACH: [&str; 6] = [
    "attach:pdf",
    "attach:image",
    "attach:sheet",
    "attach:doc",
    "attach:code",
    "attach:folder",
];
const TOOL: [(&str, &str, &str); 4] = [
    ("tool:goal", "◎", "Goal"),
    ("tool:plan", "◌", "Plan mode"),
    ("tool:skill", "◉", "Record a skill"),
    ("tool:draw", "∿", "Draw"),
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
                    .text_size(px(16.5))
                    .text_color(p.ph)
                    .child(if heard.is_empty() {
                        "Listening…".to_string()
                    } else {
                        heard.clone()
                    })
            }
            None => div()
                .text_size(px(16.5))
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
                        .gap(px(3.))
                        .h(px(32.))
                        .px(px(8.))
                        .children(trace.iter().map(|l| {
                            div()
                                .w(px(2.))
                                .h(px((l * 16.).max(2.)))
                                .rounded(px(1.))
                                .bg(p.fg)
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
            let mut m = menu_box(p).bottom(px(40.)).left(px(-4.)).right(px(-4.));
            m = m.child(
                div()
                    .px(px(9.))
                    .py(px(5.))
                    .text_size(px(12.5))
                    .text_color(p.ph)
                    .child("Add files"),
            );
            m = m.child(div().flex().gap(px(4.)).children(
                FileKind::ALL.into_iter().zip(ATTACH).map(|(k, id)| {
                    let f = fire(id);
                    div()
                        .id(id)
                        .flex_1()
                        .flex()
                        .flex_col()
                        .items_center()
                        .gap(px(6.))
                        .py(px(8.))
                        .rounded(px(10.))
                        .cursor_pointer()
                        .hover(move |s| s.bg(p.hov))
                        .child(file_badge(k, 26.))
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.label)
                                .child(k.label()),
                        )
                        .on_click(move |_, w, cx| f(w, cx))
                }),
            ));
            m = m.child(div().h(px(1.)).my(px(5.)).bg(p.line));
            for (id, g, l) in TOOL {
                let f = fire(id);
                let on = id == "tool:plan" && self.plan;
                m = m.child(menu_row(
                    id.into(),
                    div()
                        .w(px(18.))
                        .text_color(p.label)
                        .child(g)
                        .into_any_element(),
                    l.into(),
                    None,
                    on,
                    p,
                    f,
                ));
            }
            bar = bar.child(m);
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
            let mut m = menu_box(p).bottom(px(40.)).right(px(30.)).w(px(236.));
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
            bar = bar.child(m);
        }
        card = card.child(bar);

        let mut root = div()
            .relative()
            .w(px(540.))
            .flex()
            .flex_col()
            .rounded(px(21.))
            .when(self.tray, |d| d.bg(p.band).pt(px(3.)).px(px(3.)))
            .child(card);
        if self.tray {
            let mut tray = div()
                .relative()
                .flex()
                .items_center()
                .gap(px(4.))
                .h(px(42.))
                .px(px(9.))
                .text_size(px(14.5))
                .text_color(p.label);
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

fn avatar(seed: Option<u8>, size: f32) -> AnyElement {
    const BG: [u32; 6] = [0xE9B8A0, 0x9DBEE3, 0xBCA6E3, 0xA5CFB2, 0xE2C485, 0xCFA2AE];
    let d = div().flex_none().size(px(size)).rounded_full();
    match seed {
        Some(s) => d.bg(rgb(BG[s as usize % 6])).into_any_element(),
        None => d.bg(rgb(0x4C7DFF)).into_any_element(),
    }
}

/// One message of [`AgentCommentThreadEl`]: author, portrait seed (`None` = the agent's orb),
/// minutes ago, text, and the agent's status line with its details.
pub type ThreadMsg = (
    String,
    Option<u8>,
    u32,
    String,
    Option<(String, Vec<String>)>,
);

/// `comment-thread` (affectUI Comment Thread): portraits, greyed `/Edit` commands, the agent's
/// orb and its foldable "✓ Edit completed" line, and the reply field. Acts: `close`, `reply`;
/// the status line folds through [`Pick`] (message index).
#[derive(IntoElement)]
pub struct AgentCommentThreadEl {
    msgs: Vec<ThreadMsg>,
    unfolded: Vec<usize>,
    draft: SharedString,
    agent: SharedString,
    thinking: bool,
    act: Option<Act>,
    pick: Option<Pick>,
    theme: &'static Theme,
}

impl AgentCommentThreadEl {
    /// A thread.
    pub fn new(msgs: Vec<ThreadMsg>, agent: impl Into<SharedString>) -> Self {
        Self {
            msgs,
            unfolded: vec![],
            draft: SharedString::default(),
            agent: agent.into(),
            thinking: false,
            act: None,
            pick: None,
            theme: &Theme::DARK,
        }
    }
    /// Unfolded status lines, the reply draft and whether the agent is thinking.
    pub fn state(
        mut self,
        unfolded: Vec<usize>,
        draft: impl Into<SharedString>,
        thinking: bool,
    ) -> Self {
        self.unfolded = unfolded;
        self.draft = draft.into();
        self.thinking = thinking;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// `close` and `reply`.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
    /// Status line folds (message index).
    pub fn on_pick(mut self, p: Pick) -> Self {
        self.pick = Some(p);
        self
    }
}

impl RenderOnce for AgentCommentThreadEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let n = self.msgs.len();
        let mut body = div()
            .flex()
            .flex_col()
            .gap(px(16.))
            .p(px(14.))
            .rounded(px(15.))
            .bg(p.card);
        for (i, (who, seed, mins, text, status)) in self.msgs.into_iter().enumerate() {
            let mut m = div().flex().flex_col().gap(px(7.)).child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .child(avatar(seed, 24.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(who))
                    .child(div().flex_1())
                    .child(div().text_size(px(13.)).text_color(p.ph).child(ago(mins))),
            );
            if !text.is_empty() {
                let (cmd, rest) = split_command(&text);
                m = m.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(4.))
                        .when_some(cmd.map(str::to_string), |d, c| {
                            d.child(div().text_color(p.ph).child(c))
                        })
                        .child(rest.to_string()),
                );
            }
            if let Some((st, det)) = status {
                let open = self.unfolded.contains(&i);
                let pk = self.pick.clone();
                m = m.child(
                    div()
                        .id(SharedString::from(format!("ct-st{i}")))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .cursor_pointer()
                        .child(
                            div()
                                .size(px(16.))
                                .rounded_full()
                                .bg(p.ok)
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(gpui::white())
                                .text_size(px(10.))
                                .child("✓"),
                        )
                        .child(st)
                        .child(
                            div()
                                .text_color(p.label)
                                .child(if open { "⌃" } else { "⌄" }),
                        )
                        .on_click(move |_, w, cx| {
                            if let Some(f) = &pk {
                                f(i, w, cx)
                            }
                        }),
                );
                if open {
                    m = m.child(
                        div().flex().flex_col().pl(px(20.)).children(
                            det.into_iter().map(|d| {
                                div().text_color(p.label).py(px(3.)).child(format!("└ {d}"))
                            }),
                        ),
                    );
                }
            }
            body = body.child(m);
        }
        if self.thinking {
            body = body.child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .child(avatar(None, 24.))
                    .child(div().text_color(p.ph).child("Thinking…")),
            );
        }
        let a = self.act.clone();
        let has = !self.draft.is_empty();
        div()
            .w(px(420.))
            .flex()
            .flex_col()
            .p(px(4.))
            .rounded(px(18.))
            .border_1()
            .border_color(p.line)
            .bg(p.outer)
            .text_color(p.fg)
            .text_size(px(14.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(40.))
                    .px(px(12.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Comment"))
                    .child(div().text_color(p.ph).text_size(px(13.)).child(format!(
                        "{n} {}",
                        if n == 1 { "message" } else { "messages" }
                    )))
                    .child(div().flex_1())
                    .child(ibtn("close", "✕", p, a.clone())),
            )
            .child(body)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .h(px(50.))
                    .px(px(12.))
                    .child(
                        div()
                            .flex_1()
                            .text_color(if has { p.fg } else { p.ph })
                            .child(if has {
                                self.draft.clone()
                            } else {
                                format!("Reply, or ask @{}…", self.agent).into()
                            }),
                    )
                    .child(
                        div()
                            .id("reply")
                            .size(px(30.))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(if has { p.send } else { p.off })
                            .text_color(if has { p.send_fg } else { p.off_fg })
                            .child("↑")
                            .when_some(a.filter(|_| has), |d, f| {
                                d.cursor_pointer()
                                    .on_click(move |_, w, cx| f("reply", w, cx))
                            }),
                    ),
            )
    }
}

/// `comments-panel` (affectUI Comments Panel): All / Questions / Edits tabs with the open-edit
/// count, sort, Resolve toggles, runnable `/edit` comments (Run edit → Running… → Edit applied +
/// Undo) and the composer. Acts: `tab:all|questions|edits`, `sort`, `add`, `close`; per-comment
/// picks arrive through [`Pick`] as `id * 10 + 1` (resolve), `+ 2` (run edit), `+ 3` (undo).
#[derive(IntoElement)]
pub struct CommentsPanelEl {
    comments: Vec<PanelComment>,
    filter: PanelFilter,
    oldest_first: bool,
    draft: SharedString,
    act: Option<Act>,
    pick: Option<Pick>,
    theme: &'static Theme,
}

impl CommentsPanelEl {
    /// A panel over the comments (their `run` field carries Running / Applied).
    pub fn new(comments: Vec<PanelComment>) -> Self {
        Self {
            comments,
            filter: PanelFilter::All,
            oldest_first: false,
            draft: SharedString::default(),
            act: None,
            pick: None,
            theme: &Theme::DARK,
        }
    }
    /// Tab, order and the composer draft.
    pub fn state(
        mut self,
        filter: PanelFilter,
        oldest_first: bool,
        draft: impl Into<SharedString>,
    ) -> Self {
        self.filter = filter;
        self.oldest_first = oldest_first;
        self.draft = draft.into();
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Tab, sort, add and close acts.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
    /// Per-comment picks.
    pub fn on_pick(mut self, p: Pick) -> Self {
        self.pick = Some(p);
        self
    }
}

impl RenderOnce for CommentsPanelEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let view = panel_view(&self.comments, self.filter, self.oldest_first);
        let edits = open_edits(&self.comments);
        let a = self.act.clone();
        let tab = |f: PanelFilter, id: &'static str, l: &'static str| {
            let on = self.filter == f;
            let a = a.clone();
            div()
                .id(id)
                .flex()
                .items_center()
                .gap(px(6.))
                .h(px(28.))
                .px(px(10.))
                .rounded(px(8.))
                .text_size(px(13.))
                .text_color(if on { p.fg } else { p.ph })
                .when(on, |d| d.bg(p.off))
                .cursor_pointer()
                .child(l)
                .when(f == PanelFilter::Edits && edits > 0, |d| {
                    d.child(
                        div()
                            .px(px(5.))
                            .rounded_full()
                            .bg(p.fg)
                            .text_color(p.card)
                            .text_size(px(10.5))
                            .child(edits.to_string()),
                    )
                })
                .when_some(a, move |d, f| d.on_click(move |_, w, cx| f(id, w, cx)))
        };
        let mut list = div().flex().flex_col();
        for i in view {
            let cm = &self.comments[i];
            let (cmd, rest) = split_command(&cm.text);
            let id = cm.id as usize;
            let pk = self.pick.clone();
            let send = move |k: usize| {
                let pk = pk.clone();
                move |_: &gpui::ClickEvent, w: &mut Window, cx: &mut App| {
                    if let Some(f) = &pk {
                        f(id * 10 + k, w, cx)
                    }
                }
            };
            let mut row = div()
                .flex()
                .flex_col()
                .gap(px(6.))
                .py(px(12.))
                .pl(px(16.))
                .pr(px(12.))
                .border_b_1()
                .border_color(p.line)
                .when(cm.resolved, |d| d.opacity(0.5))
                .child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(avatar(Some(cm.avatar), 22.))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(cm.author.clone()),
                        )
                        .child(
                            div()
                                .text_size(px(12.))
                                .text_color(p.ph)
                                .child(ago(cm.minutes)),
                        )
                        .child(div().flex_1())
                        .child(
                            div()
                                .id(SharedString::from(format!("cp-r{id}")))
                                .size(px(26.))
                                .rounded(px(7.))
                                .flex()
                                .items_center()
                                .justify_center()
                                .text_color(if cm.resolved { p.ok } else { p.label })
                                .cursor_pointer()
                                .child("◯")
                                .on_click(send(1)),
                        ),
                )
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap(px(4.))
                        .when(cm.resolved, |d| d.line_through())
                        .when_some(cmd.map(str::to_string), |d, c| {
                            d.child(div().text_color(p.ph).child(c))
                        })
                        .child(rest.to_string()),
                );
            if is_edit(&cm.text) && !cm.resolved {
                row = row.child(match cm.run {
                    EditRun::Running => div()
                        .h(px(28.))
                        .px(px(10.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(p.line)
                        .text_color(p.label)
                        .child("Running…")
                        .into_any_element(),
                    EditRun::Applied => div()
                        .flex()
                        .gap(px(8.))
                        .child(
                            div()
                                .px(px(9.))
                                .rounded_full()
                                .bg(gpui::Hsla { a: 0.15, ..p.ok })
                                .text_color(p.ok)
                                .child("✓ Edit applied"),
                        )
                        .child(
                            div()
                                .id(SharedString::from(format!("cp-u{id}")))
                                .text_color(p.label)
                                .cursor_pointer()
                                .child("Undo")
                                .on_click(send(3)),
                        )
                        .into_any_element(),
                    EditRun::Idle => div()
                        .id(SharedString::from(format!("cp-e{id}")))
                        .flex()
                        .items_center()
                        .h(px(28.))
                        .px(px(10.))
                        .rounded(px(8.))
                        .border_1()
                        .border_color(p.line)
                        .cursor_pointer()
                        .child("▶ Run edit")
                        .on_click(send(2))
                        .into_any_element(),
                });
            }
            list = list.child(row);
        }
        let has = !self.draft.is_empty();
        div()
            .w(px(400.))
            .flex()
            .flex_col()
            .rounded(px(18.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .text_color(p.fg)
            .text_size(px(13.5))
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(44.))
                    .pl(px(16.))
                    .pr(px(8.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child("Comments"))
                    .child(div().flex_1())
                    .child(ibtn("close", "✕", p, a.clone())),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(2.))
                    .pb(px(8.))
                    .pl(px(12.))
                    .pr(px(10.))
                    .border_b_1()
                    .border_color(p.line)
                    .child(tab(PanelFilter::All, "tab:all", "All"))
                    .child(tab(PanelFilter::Questions, "tab:questions", "Questions"))
                    .child(tab(PanelFilter::Edits, "tab:edits", "Edits"))
                    .child(div().flex_1())
                    .child(ibtn("sort", "☰", p, a.clone())),
            )
            .child(list)
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .p(px(10.))
                    .pl(px(14.))
                    .child(
                        div()
                            .flex_1()
                            .text_color(if has { p.fg } else { p.ph })
                            .child(if has {
                                self.draft.clone()
                            } else {
                                "Add a comment or /edit…".into()
                            }),
                    )
                    .child(
                        div()
                            .id("add")
                            .size(px(28.))
                            .rounded_full()
                            .flex()
                            .items_center()
                            .justify_center()
                            .bg(if has { p.send } else { p.off })
                            .text_color(if has { p.send_fg } else { p.off_fg })
                            .child("↑")
                            .when_some(a.filter(|_| has), |d, f| {
                                d.cursor_pointer().on_click(move |_, w, cx| f("add", w, cx))
                            }),
                    ),
            )
    }
}
