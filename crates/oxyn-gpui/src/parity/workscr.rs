//! Screens S2 in GPUI: Folders Overview and Detail, Integrations, Kanban, CRM and Analytics on the
//! shared frame with each persona's sidebar (`oxyn_core::parity::workscr::persona`). The elements
//! draw one state at a time: `.state()` for Data / Loading / Empty and `.extra()` for the screen's
//! own state (`creating`, `uploading`, `failed`, `dragging`, `editing`, `compare`) or an Oxyn
//! option (`list`, `starred`, `totals`, `30d`). Presses fire `nav-<key>`, `open-<id>`,
//! `link-<id>`, `card-<id>` and similar ids through `on_act`.

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px,
};
use oxyn_core::icon_tone::IconTone;
use oxyn_core::parity::screensfid::{SIDE_W, ShellLayout, TOP_H, WORKSPACE};
use oxyn_core::parity::workscr::{
    CRM_COLS, CRM_TABS, CardStatus, CrmStage, FOLDER_TABS, FileKind, INTEG_TABS, KB_LANES, KB_TABS,
    POPULAR_SHOWN, Persona, RANGES, ScreenState, TEMPLATE_TABS, breakdown, cards, cat_tone,
    channel_total, companies, dollars, dollars_k, folder_docs, folder_tiles, integ_view,
    integrations, kpis, lane_view, money, more_words, persona, tab_counts, templates, tile_line,
    top_products, weekly, when_words,
};
use oxyn_tokens::Theme;

use super::Act;
use super::chatscr::{clip, mark};
use super::rulerfid::{ThPal, fire, th_pal};
use super::screensfid::{frame, gh, side_foot, skel};
use crate::brand::{brand_mark, tone_color};

macro_rules! work_el {
    ($(#[$m:meta])* $name:ident) => {
        $(#[$m])*
        #[derive(IntoElement)]
        pub struct $name {
            state: ScreenState,
            extra: &'static str,
            layout: ShellLayout,
            act: Option<Act>,
            theme: &'static Theme,
        }
        impl Default for $name {
            fn default() -> Self {
                Self { state: ScreenState::Sample, extra: "", layout: ShellLayout::Inset, act: None, theme: &Theme::DARK }
            }
        }
        impl $name {
            /// Sample, Loading or Empty.
            pub fn state(mut self, s: ScreenState) -> Self {
                self.state = s;
                self
            }
            /// The screen's own state or an Oxyn option (see the module docs).
            pub fn extra(mut self, k: &'static str) -> Self {
                self.extra = k;
                self
            }
            /// Inset or Flat.
            pub fn layout(mut self, l: ShellLayout) -> Self {
                self.layout = l;
                self
            }
            /// Light or dark chrome.
            pub fn theme(mut self, t: &'static Theme) -> Self {
                self.theme = t;
                self
            }
            /// Press ids (`nav-<key>`, `open-<id>`, `link-<id>`…).
            pub fn on_act(mut self, a: Act) -> Self {
                self.act = Some(a);
                self
            }
        }
    };
}

fn tone(p: &ThPal, t: IconTone) -> Hsla {
    tone_color(t, p.fg.l > 0.5)
}

fn quiet(p: &ThPal, t: impl Into<SharedString>) -> Div {
    div()
        .flex_none()
        .whitespace_nowrap()
        .text_size(px(12.))
        .text_color(p.mute)
        .child(t.into())
}

fn nav_id(k: &'static str) -> &'static str {
    match k {
        "new" => "nav-new",
        "files" => "nav-files",
        "apps" => "nav-apps",
        "inbox" => "nav-inbox",
        "activity" => "nav-activity",
        "mine" => "nav-mine",
        "board" => "nav-board",
        "projects" => "nav-projects",
        "home" => "nav-home",
        "companies" => "nav-companies",
        "contacts" => "nav-contacts",
        "deals" => "nav-deals",
        "reports" => "nav-reports",
        "revenue" => "nav-revenue",
        "invoices" => "nav-invoices",
        "payouts" => "nav-payouts",
        "customers" => "nav-customers",
        _ => "nav-group",
    }
}

/// The persona sidebar: workspace row, search, nav with counts, the group, then the foot.
fn side(p: &ThPal, pe: &Persona, on: &str, st: ScreenState, act: &Option<Act>) -> Div {
    let mut s = div()
        .w(px(SIDE_W.0 as f32))
        .flex_none()
        .flex()
        .flex_col()
        .p(px(8.))
        .bg(p.inset)
        .child(
            div()
                .flex()
                .items_center()
                .gap(px(8.))
                .h(px(34.))
                .px(px(6.))
                .child(div().w(px(22.)).h(px(22.)).rounded(px(7.)).bg(p.fg))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(WORKSPACE)),
        )
        .child(
            div()
                .h(px(32.))
                .my(px(8.))
                .px(px(8.))
                .flex()
                .items_center()
                .rounded(px(9.))
                .bg(p.card)
                .border_1()
                .border_color(p.line)
                .text_color(p.mute)
                .child(clip("Search…"))
                .child(quiet(p, "⌘K")),
        );
    for &(k, l, _, n) in pe.nav {
        let f = fire(act, nav_id(k));
        s = s.child(
            div()
                .id(SharedString::from(format!("nav-{k}")))
                .h(px(32.))
                .px(px(8.))
                .flex()
                .items_center()
                .gap(px(8.))
                .rounded(px(8.))
                .text_color(if k == on { p.fg } else { p.fg2 })
                .when(k == on, |d| d.bg(p.sel))
                .child(clip(l))
                .when(n > 0, |d| d.child(quiet(p, n.to_string())))
                .on_click(move |_, w, cx| f(w, cx)),
        );
    }
    s = s.child(gh(p, pe.group));
    if st == ScreenState::Loading {
        s = s.child(skel(p, 4));
    } else {
        for &(k, l, _) in pe.items {
            s = s.child(
                div()
                    .h(px(30.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .rounded(px(8.))
                    .text_color(p.fg2)
                    .when(k == on, |d| d.bg(p.sel))
                    .child(clip(l)),
            );
        }
    }
    s.child(div().flex_1()).child(side_foot(p, act))
}

/// The page with an optional 48 px top bar (crumb / title, actions).
fn page(p: &ThPal, crumb: Option<(&'static str, String)>, actions: Vec<Div>, body: Div) -> Div {
    let mut col = div().flex_1().min_w_0().flex().flex_col();
    if let Some((root, title)) = crumb {
        let mut bar = div()
            .h(px(TOP_H as f32))
            .flex_none()
            .px(px(14.))
            .flex()
            .items_center()
            .gap(px(8.))
            .border_b_1()
            .border_color(p.line)
            .child(div().text_color(p.mute).child(root))
            .child(div().text_color(p.line).child("/"))
            .child(
                div()
                    .font_weight(FontWeight::MEDIUM)
                    .min_w_0()
                    .overflow_hidden()
                    .whitespace_nowrap()
                    .text_ellipsis()
                    .child(title),
            )
            .child(div().flex_1());
        for a in actions {
            bar = bar.child(a);
        }
        col = col.child(bar);
    }
    col.child(body)
}

fn btn(p: &ThPal, l: &'static str, primary: bool) -> Div {
    let d = div()
        .flex_none()
        .h(px(28.))
        .px(px(10.))
        .flex()
        .items_center()
        .rounded(px(7.))
        .whitespace_nowrap()
        .text_size(px(13.));
    if primary {
        d.bg(p.fg).text_color(p.card).child(l)
    } else {
        d.border_1()
            .border_color(p.line)
            .bg(p.card)
            .text_color(p.fg2)
            .child(l)
    }
}

fn tabs(p: &ThPal, items: &[(&'static str, String)], on: &str) -> Div {
    let mut r = div().flex().items_center().gap(px(6.)).min_w_0();
    for (k, l) in items {
        let sel = *k == on;
        r = r.child(
            div()
                .flex_none()
                .h(px(28.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .whitespace_nowrap()
                .border_1()
                .border_color(if sel { p.sel } else { p.line })
                .bg(if sel { p.sel } else { p.card })
                .text_color(if sel { p.fg } else { p.fg2 })
                .child(l.clone()),
        );
    }
    r
}

fn find(p: &ThPal, ph: &'static str, w: f32) -> Div {
    div()
        .w(px(w))
        .flex_none()
        .h(px(30.))
        .px(px(9.))
        .flex()
        .items_center()
        .rounded(px(8.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .text_color(p.mute)
        .child(clip(ph))
}

fn cap(p: &ThPal, t: &'static str, n: usize) -> Div {
    div()
        .mt(px(36.))
        .mb(px(14.))
        .text_size(px(11.5))
        .text_color(p.mute)
        .child(format!("{} {n}", t.to_uppercase()))
}

fn none(p: &ThPal, t: &'static str, l: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(4.))
        .py(px(40.))
        .child(div().w(px(44.)).h(px(44.)).rounded(px(12.)).bg(p.inset))
        .child(div().font_weight(FontWeight::SEMIBOLD).child(t))
        .child(div().text_color(p.mute).child(l))
}

fn folder_glyph(p: &ThPal, s: f32) -> Div {
    let c = tone(p, IconTone::Folder);
    div()
        .flex_none()
        .flex()
        .flex_col()
        .child(
            div()
                .w(px(s * 0.4))
                .h(px(s * 0.12))
                .rounded_t(px(2.))
                .bg(c.opacity(0.72)),
        )
        .child(div().w(px(s)).h(px(s * 0.62)).rounded(px(3.)).bg(c))
}

fn kind_tone(k: FileKind) -> IconTone {
    match k {
        FileKind::Doc => IconTone::Doc,
        FileKind::Sheet => IconTone::Sheet,
        FileKind::Pdf => IconTone::Pdf,
        FileKind::Slides => IconTone::Slides,
        FileKind::Image => IconTone::Image,
    }
}

fn preview(p: &ThPal, k: FileKind, w: f32, h: f32) -> Div {
    let mut sheet = div()
        .relative()
        .w(px(54.))
        .h(px(70.))
        .p(px(10.))
        .flex()
        .flex_col()
        .gap(px(5.))
        .rounded(px(4.))
        .border_1()
        .border_color(p.line)
        .bg(p.card);
    for i in 0..5 {
        sheet = sheet.child(
            div()
                .h(px(2.))
                .w(px(if i % 2 == 0 { 34. } else { 24. }))
                .bg(p.line),
        );
    }
    sheet = sheet.child(
        div()
            .absolute()
            .left(px(-8.))
            .bottom(px(12.))
            .px(px(4.))
            .rounded(px(3.))
            .bg(tone(p, kind_tone(k)))
            .text_color(gpui::white())
            .text_size(px(9.))
            .font_weight(FontWeight::BOLD)
            .child(k.badge()),
    );
    div()
        .w(px(w))
        .h(px(h))
        .flex()
        .items_center()
        .justify_center()
        .rounded(px(10.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
        .child(sheet)
}

fn file_card(p: &ThPal, k: FileKind, name: String, meta: String, up: Option<u32>) -> Div {
    let mut c = div()
        .w(px(236.))
        .flex()
        .flex_col()
        .gap(px(2.))
        .child(preview(p, k, 236., 148.).mb(px(12.)))
        .child(div().font_weight(FontWeight::MEDIUM).child(clip(name)))
        .child(
            div()
                .text_size(px(12.))
                .text_color(p.mute)
                .child(clip(meta)),
        );
    if let Some(v) = up {
        c = c.child(
            div().h(px(4.)).mt(px(6.)).rounded(px(2.)).bg(p.sel).child(
                div()
                    .h_full()
                    .w(px(236. * v as f32 / 100.))
                    .rounded(px(2.))
                    .bg(tone(p, IconTone::Running)),
            ),
        );
    }
    c
}

// ───────────────────────────────────────────────────────────── Folders ──

work_el!(
    /// Folders Overview: tabs, search, sort, grid or list, Create folder, folder tiles and file
    /// templates. `.extra("creating")` draws the new-folder field, `"list"` the list view.
    FoldersEl
);

impl RenderOnce for FoldersEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let tiles = folder_tiles(st == ScreenState::Sample);
        let tpl = templates(st == ScreenState::Sample);
        let tab_items: Vec<(&'static str, String)> = FOLDER_TABS
            .iter()
            .map(|(k, l)| (*k, l.to_string()))
            .collect();
        let tt: Vec<(&'static str, String)> = TEMPLATE_TABS
            .iter()
            .map(|(k, l)| (*k, l.to_string()))
            .collect();
        let mut body = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .px(px(32.))
            .pt(px(44.))
            .child(
                div()
                    .text_size(px(24.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Files"),
            )
            .child(
                div()
                    .mt(px(8.))
                    .text_color(p.mute)
                    .child("Everything your chats made or used, and the folders you keep it in."),
            )
            .child(cap(&p, "Folders", tiles.len()))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(6.))
                    .mb(px(14.))
                    .child(tabs(&p, &tab_items, "all"))
                    .child(div().flex_1())
                    .child(find(&p, "Search folders…", 174.))
                    .child(
                        btn(&p, "Create folder", true)
                            .id("create-folder")
                            .on_click({
                                let f = fire(&self.act, "create-folder");
                                move |_, w, cx| f(w, cx)
                            }),
                    ),
            );
        if st == ScreenState::Loading {
            body = body.child(skel(&p, 4));
        } else if tiles.is_empty() && self.extra != "creating" {
            body = body.child(none(
                &p,
                "No folders yet",
                "Group the files for one project, then share the folder.",
            ));
        } else {
            let list = self.extra == "list";
            let mut g = div().flex().gap(px(10.)).when(list, |d| d.flex_col());
            if self.extra == "creating" {
                g = g.child(
                    div()
                        .w(px(262.))
                        .h(px(56.))
                        .px(px(13.))
                        .flex()
                        .items_center()
                        .gap(px(10.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(p.fg2)
                        .child(folder_glyph(&p, 26.))
                        .child(clip("Folder name").text_color(p.mute))
                        .child(btn(&p, "Create", true)),
                );
            }
            for f in tiles {
                let id: &'static str = match f.id {
                    1 => "open-1",
                    2 => "open-2",
                    3 => "open-3",
                    _ => "open-4",
                };
                let fi = fire(&self.act, id);
                g = g.child(
                    div()
                        .id(id)
                        .when(!list, |d| d.w(px(262.)))
                        .h(px(if list { 46. } else { 56. }))
                        .px(px(13.))
                        .flex()
                        .items_center()
                        .gap(px(12.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(p.line)
                        .bg(p.card)
                        .child(folder_glyph(&p, 26.))
                        .child(
                            div()
                                .flex_1()
                                .min_w_0()
                                .flex()
                                .flex_col()
                                .child(
                                    div()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(clip(f.name.clone())),
                                )
                                .child(
                                    div()
                                        .text_size(px(12.))
                                        .text_color(p.mute)
                                        .child(clip(tile_line(&f))),
                                ),
                        )
                        .on_click(move |_, w, cx| fi(w, cx)),
                );
            }
            body = body.child(g);
        }
        body = body.child(cap(&p, "File templates", tpl.len())).child(
            div()
                .flex()
                .items_center()
                .gap(px(6.))
                .mb(px(14.))
                .child(tabs(&p, &tt, "all"))
                .child(div().flex_1())
                .child(btn(&p, "Type", false))
                .child(find(&p, "Search templates…", 174.))
                .child(btn(&p, "Add template", false)),
        );
        if st == ScreenState::Loading {
            body = body.child(skel(&p, 3));
        } else if tpl.is_empty() {
            body = body.child(
                div()
                    .text_color(p.mute)
                    .child("Templates you save show up here."),
            );
        } else {
            let mut row = div().flex().gap(px(15.));
            for t in tpl {
                let meta = format!(
                    "{} · {} · {}",
                    t.kind.label(),
                    t.owner,
                    if t.shared { "Shared" } else { "Private" }
                );
                row = row.child(file_card(&p, t.kind, t.name.to_string(), meta, None));
            }
            body = body.child(row);
        }
        frame(
            self.theme,
            self.layout,
            side(&p, persona("files"), "files", st, &self.act),
            page(&p, None, vec![], body),
            None,
        )
    }
}

work_el!(
    /// Folders Detail: breadcrumbs, back, the title row with Share and Upload files, file cards
    /// with previews, or the empty drop area. `.extra("uploading")` adds two files in flight.
    FolderDetailEl
);

impl RenderOnce for FolderDetailEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let docs = folder_docs(st == ScreenState::Sample);
        let up = fire(&self.act, "upload");
        let mut body = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .px(px(32.))
            .pt(px(28.))
            .child(div().mb(px(22.)).text_color(p.mute).child("← Files"))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .mb(px(22.))
                    .child(folder_glyph(&p, 30.))
                    .child(
                        div()
                            .text_size(px(24.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Autumn launch"),
                    )
                    .child(div().flex_1())
                    .child(btn(&p, "Share", false))
                    .child(
                        btn(&p, "Upload files", true)
                            .id("upload")
                            .on_click(move |_, w, cx| up(w, cx)),
                    ),
            );
        if st == ScreenState::Loading {
            body = body.child(skel(&p, 5));
        } else if docs.is_empty() {
            body = body.child(
                div()
                    .rounded(px(14.))
                    .border_1()
                    .border_color(p.line)
                    .child(none(
                        &p,
                        "This folder is empty",
                        "Drop files here or upload them.",
                    )),
            );
        } else {
            let mut g = div().flex().flex_wrap().gap(px(15.));
            if self.extra == "uploading" {
                g = g
                    .child(file_card(
                        &p,
                        FileKind::Sheet,
                        "Survey-Answers.csv".into(),
                        "Uploading…".into(),
                        Some(64),
                    ))
                    .child(file_card(
                        &p,
                        FileKind::Image,
                        "Field-Photos.png".into(),
                        "Uploading…".into(),
                        Some(22),
                    ));
            }
            for d in docs {
                let meta = format!("{} · {}", d.kind.label(), when_words(d.mins));
                g = g.child(file_card(&p, d.kind, d.name, meta, None));
            }
            body = body.child(g);
        }
        frame(
            self.theme,
            self.layout,
            side(&p, persona("files"), "autumn", st, &self.act),
            page(
                &p,
                Some(("Files", "Autumn launch".to_string())),
                vec![],
                body,
            ),
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── Integrations ──

work_el!(
    /// Integrations: tabs, search and Request over apps grouped by category with their official
    /// marks and link states. `.extra("failed")` draws one link that didn't go through.
    IntegrationsEl
);

impl RenderOnce for IntegrationsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let all = integrations(st == ScreenState::Sample);
        let ti: Vec<(&'static str, String)> = INTEG_TABS
            .iter()
            .map(|(k, l)| (*k, l.to_string()))
            .collect();
        let mut col =
            div()
                .w(px(958.))
                .flex()
                .flex_col()
                .pt(px(40.))
                .child(
                    div()
                        .text_size(px(24.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Integrations"),
                )
                .child(div().mt(px(8.)).text_color(p.mute).child(
                    "Link the tools your team already works in, then use them from any chat.",
                ))
                .child(
                    div()
                        .mt(px(34.))
                        .py(px(14.))
                        .flex()
                        .items_center()
                        .gap(px(6.))
                        .child(tabs(&p, &ti, "all"))
                        .child(div().flex_1())
                        .child(find(&p, "Search integrations", 255.))
                        .child(btn(&p, "Request", true)),
                );
        if st == ScreenState::Loading {
            col = col.child(skel(&p, 8));
        } else if all.is_empty() {
            col = col.child(none(
                &p,
                "No integrations to show",
                "Ask for the app you need and we'll pass it on.",
            ));
        } else {
            for (_, t, items) in integ_view(&all, "all", "", "") {
                let fold = items.len() > POPULAR_SHOWN && t == "Popular";
                let shown = if fold {
                    &items[..POPULAR_SHOWN]
                } else {
                    &items[..]
                };
                let mut grid = div().flex().flex_wrap().gap(px(12.));
                for i in shown {
                    let fail = self.extra == "failed" && i.id == "figma";
                    let id: &'static str = i.id;
                    let f = fire(&self.act, id);
                    let action = div()
                        .id(SharedString::from(format!("link-{id}")))
                        .w(px(28.))
                        .h(px(28.))
                        .flex_none()
                        .flex()
                        .items_center()
                        .justify_center()
                        .rounded(px(7.))
                        .when(i.on, |d| d.bg(p.fg).text_color(p.card).child("✓"))
                        .when(!i.on, |d| {
                            d.border_1()
                                .border_color(p.line)
                                .text_color(p.fg2)
                                .child("+")
                        })
                        .on_click(move |_, w, cx| f(w, cx));
                    grid = grid.child(
                        div()
                            .w(px(462.))
                            .h(px(46.))
                            .flex()
                            .items_center()
                            .gap(px(12.))
                            .child(
                                div()
                                    .w(px(30.))
                                    .h(px(30.))
                                    .flex_none()
                                    .flex()
                                    .items_center()
                                    .justify_center()
                                    .rounded(px(8.))
                                    .border_1()
                                    .border_color(p.line)
                                    .bg(gpui::white())
                                    .children(brand_mark(i.id, 18., gpui::black())),
                            )
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .child(
                                        div()
                                            .text_size(px(14.5))
                                            .font_weight(FontWeight::MEDIUM)
                                            .child(clip(i.name)),
                                    )
                                    .child(if fail {
                                        div()
                                            .text_size(px(12.5))
                                            .text_color(tone(&p, IconTone::Failed))
                                            .child(clip(format!("Couldn't link {}.", i.name)))
                                    } else {
                                        div()
                                            .text_size(px(13.))
                                            .text_color(p.mute)
                                            .child(clip(i.blurb))
                                    }),
                            )
                            .child(if fail {
                                btn(&p, "Try again", false)
                                    .id(SharedString::from(format!("retry-{id}")))
                            } else {
                                action
                            }),
                    );
                }
                col = col
                    .child(
                        div()
                            .mt(px(34.))
                            .mb(px(16.))
                            .text_size(px(15.))
                            .font_weight(FontWeight::MEDIUM)
                            .child(t),
                    )
                    .child(grid);
                if fold {
                    col = col.child(
                        div()
                            .mt(px(14.))
                            .text_color(p.fg2)
                            .child(format!("{} ›", more_words(&items[POPULAR_SHOWN..]))),
                    );
                }
            }
        }
        let body = div().flex_1().min_w_0().flex().justify_center().child(col);
        frame(
            self.theme,
            self.layout,
            side(&p, persona("files"), "apps", st, &self.act),
            page(&p, None, vec![], body),
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── Kanban ──

fn status_color(p: &ThPal, s: CardStatus) -> Hsla {
    tone(
        p,
        match s {
            CardStatus::Review => IconTone::Running,
            CardStatus::Progress => IconTone::Skills,
            CardStatus::Attention => IconTone::Approval,
            CardStatus::Fixed => IconTone::Done,
        },
    )
}

work_el!(
    /// Kanban: Inbox / Overview with Filter, Sort by and Board | List, task tabs, New task and four
    /// 276 px lanes. `.extra("dragging")` draws a card held over High priority; `"list"` the list.
    KanbanEl
);

impl RenderOnce for KanbanEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let all = cards(st == ScreenState::Sample);
        let (a, o, d) = tab_counts(&all);
        let ti: Vec<(&'static str, String)> = KB_TABS
            .iter()
            .map(|(k, l)| {
                (
                    *k,
                    format!(
                        "{l} {}",
                        match *k {
                            "open" => o,
                            "done" => d,
                            _ => a,
                        }
                    ),
                )
            })
            .collect();
        let nt = fire(&self.act, "new-task");
        let mut body = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(14.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .child(tabs(&p, &ti, "all"))
                    .child(div().flex_1())
                    .child(
                        btn(&p, "New task", true)
                            .id("new-task")
                            .on_click(move |_, w, cx| nt(w, cx)),
                    ),
            );
        if st == ScreenState::Loading {
            body = body.child(skel(&p, 6));
        } else if all.is_empty() {
            body = body.child(none(
                &p,
                "No tasks on this board",
                "Add the first one; its priority picks the lane.",
            ));
        } else {
            let mut lanes = div().flex().gap(px(10.));
            for (lk, ll) in KB_LANES {
                let n = all.iter().filter(|c| c.lane == lk).count();
                let hot = self.extra == "dragging" && lk == "high";
                let mut lane = div()
                    .w(px(276.))
                    .flex_none()
                    .p(px(6.))
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .rounded(px(14.))
                    .bg(if hot { p.sel } else { p.inset })
                    .child(
                        div()
                            .h(px(30.))
                            .px(px(8.))
                            .flex()
                            .items_center()
                            .child(
                                div()
                                    .flex_1()
                                    .text_size(px(11.5))
                                    .text_color(p.mute)
                                    .child(ll.to_uppercase()),
                            )
                            .child(quiet(&p, n.to_string())),
                    );
                for c in lane_view(&all, lk, "all", "", "manual") {
                    let lifted = self.extra == "dragging" && c.id == 5;
                    lane = lane.child(
                        div()
                            .p(px(12.))
                            .flex()
                            .flex_col()
                            .gap(px(8.))
                            .rounded(px(12.))
                            .border_1()
                            .border_color(if lifted { p.fg2 } else { p.line })
                            .bg(p.card)
                            .when(lifted, |d| d.opacity(0.55))
                            .child(
                                div()
                                    .font_weight(FontWeight::MEDIUM)
                                    .child(clip(c.title.clone())),
                            )
                            .child(
                                div()
                                    .text_size(px(13.))
                                    .text_color(p.mute)
                                    .h(px(38.))
                                    .overflow_hidden()
                                    .child(c.body.clone()),
                            )
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap(px(6.))
                                    .text_size(px(12.5))
                                    .child(
                                        div()
                                            .w(px(10.))
                                            .h(px(10.))
                                            .rounded(px(5.))
                                            .bg(status_color(&p, c.status)),
                                    )
                                    .child(clip(c.status.label()))
                                    .child(quiet(&p, format!("{} · {}", c.talk, c.links))),
                            ),
                    );
                }
                lanes = lanes.child(lane);
            }
            body = body.child(lanes);
        }
        let acts = vec![
            btn(&p, "Filter", false),
            btn(&p, "Sort by", false),
            btn(&p, "Board | List", false),
        ];
        frame(
            self.theme,
            self.layout,
            side(&p, persona("board"), "board", st, &self.act),
            page(&p, Some(("Inbox", "Overview".to_string())), acts, body),
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── CRM ──

fn tag_color(p: &ThPal, t: &str) -> Hsla {
    match t {
        "blue" | "qualified" => tone(p, IconTone::Running),
        "purple" | "proposal" => tone(p, IconTone::Skills),
        "pink" => tone(p, IconTone::Image),
        "amber" | "negotiation" => tone(p, IconTone::Approval),
        "won" => tone(p, IconTone::Done),
        "lost" => tone(p, IconTone::Failed),
        _ => p.fg2,
    }
}

fn tag(p: &ThPal, l: String, t: &str) -> Div {
    let c = tag_color(p, t);
    div()
        .flex_none()
        .px(px(6.))
        .rounded(px(5.))
        .text_size(px(12.5))
        .text_color(c)
        .bg(c.opacity(0.12))
        .child(l)
}

work_el!(
    /// CRM: Sales / Head of Sales, open views as tabs with marks and the Company CRM table.
    /// `.extra("editing")` draws a Value cell open; `"totals"` the pipeline strip.
    CrmEl
);

impl RenderOnce for CrmEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let rows = companies(st == ScreenState::Sample);
        let mut tabs_r = div().flex().items_center().gap(px(4.));
        for (i, (_, l, b)) in CRM_TABS.iter().enumerate() {
            tabs_r = tabs_r.child(
                div()
                    .h(px(32.))
                    .px(px(11.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .rounded(px(9.))
                    .when(i == 0, |d| d.bg(p.sel))
                    .text_color(if i == 0 { p.fg } else { p.mute })
                    .children(mark(&p, b, 14.))
                    .child(*l),
            );
        }
        let widths = [210., 150., 150., 140., 110., 120., 120.];
        let mut head = div()
            .h(px(36.))
            .px(px(14.))
            .flex()
            .items_center()
            .border_b_1()
            .border_color(p.line)
            .child(div().w(px(30.)));
        for (i, (_, l, _)) in CRM_COLS.iter().enumerate() {
            head = head.child(div().w(px(widths[i])).text_color(p.fg2).child(*l));
        }
        let mut table = div()
            .flex_1()
            .flex()
            .flex_col()
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .child(head);
        if st == ScreenState::Loading {
            table = table.child(skel(&p, 8));
        } else if rows.is_empty() {
            table = table.child(none(
                &p,
                "No companies yet",
                "Add a row, or link a CRM in Integrations.",
            ));
        } else {
            for r in rows {
                let editing = self.extra == "editing" && r.id == 4;
                table = table.child(
                    div()
                        .h(px(40.))
                        .px(px(14.))
                        .flex()
                        .items_center()
                        .border_b_1()
                        .border_color(p.line)
                        .child(
                            div().w(px(30.)).child(
                                div()
                                    .w(px(14.))
                                    .h(px(14.))
                                    .rounded(px(3.))
                                    .border_1()
                                    .border_color(p.line),
                            ),
                        )
                        .child(
                            div()
                                .w(px(widths[0]))
                                .flex()
                                .items_center()
                                .gap(px(9.))
                                .children(mark(&p, &r.name, 15.))
                                .child(clip(r.name.clone())),
                        )
                        .child(div().w(px(widths[1])).child(tag(
                            &p,
                            r.cat.clone(),
                            cat_tone(&r.cat),
                        )))
                        .child(
                            div()
                                .w(px(widths[2]))
                                .text_color(p.fg2)
                                .child(clip(r.domain.clone())),
                        )
                        .child(
                            div()
                                .w(px(widths[3]))
                                .text_color(p.fg2)
                                .child(clip(r.owner.clone())),
                        )
                        .child(
                            div()
                                .w(px(widths[4]))
                                .when(editing, |d| {
                                    d.border_2()
                                        .border_color(tone(&p, IconTone::Running))
                                        .rounded(px(4.))
                                        .px(px(4.))
                                })
                                .child(money(r.value)),
                        )
                        .child(div().w(px(widths[5])).child(tag(
                            &p,
                            r.stage.label().to_string(),
                            r.stage.key(),
                        )))
                        .child(
                            div()
                                .w(px(widths[6]))
                                .text_color(p.mute)
                                .child(r.last.clone()),
                        ),
                );
            }
        }
        let mut body = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(12.))
            .child(
                div()
                    .flex()
                    .items_center()
                    .child(tabs_r)
                    .child(div().flex_1())
                    .child(btn(&p, "Stage", false))
                    .child(btn(&p, "Owner", false))
                    .child(find(&p, "Search", 160.)),
            );
        if self.extra == "totals" && st == ScreenState::Sample {
            let mut strip = div().flex().gap(px(6.));
            for s in CrmStage::ALL {
                strip = strip.child(
                    div()
                        .px(px(10.))
                        .py(px(6.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(p.line)
                        .child(s.label()),
                );
            }
            body = body.child(strip);
        }
        body = body.child(table);
        frame(
            self.theme,
            self.layout,
            side(&p, persona("sales"), "companies", st, &self.act),
            page(
                &p,
                Some(("Sales", "Head of Sales".to_string())),
                vec![],
                body,
            ),
            None,
        )
    }
}

// ───────────────────────────────────────────────────────────── Analytics ──

fn card(p: &ThPal) -> Div {
    div()
        .min_w_0()
        .p(px(16.))
        .rounded(px(14.))
        .border_1()
        .border_color(p.line)
        .bg(p.card)
}

work_el!(
    /// Analytics: Analytics / Revenue with the range, Export and Save as report; KPI tiles, channel
    /// revenue, a breakdown, best sellers and two weekly charts. `.extra("30d")` opens on 30 days.
    AnalyticsEl
);

impl RenderOnce for AnalyticsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = th_pal(self.theme);
        let st = self.state;
        let r = if self.extra == "30d" { "30d" } else { "7d" };
        let blank = st == ScreenState::Empty;
        let loading = st == ScreenState::Loading;
        let good = tone(&p, IconTone::Done);
        let mut kp = div().flex().gap(px(10.));
        for k in kpis(r) {
            let mut c = card(&p)
                .flex_1()
                .h(px(88.))
                .child(div().text_color(p.mute).child(k.name));
            c = if loading {
                c.child(skel(&p, 1))
            } else {
                c.child(
                    div()
                        .mt(px(10.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().text_size(px(21.)).child(if blank {
                            "—".to_string()
                        } else {
                            k.value
                        }))
                        .when(!blank, |d| {
                            d.child(div().text_size(px(12.5)).text_color(good).child(k.delta))
                        }),
                )
            };
            kp = kp.child(c);
        }
        let mut chan = card(&p)
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(8.))
            .child(div().text_color(p.mute).child("Revenue by channel"));
        if loading {
            chan = chan.child(skel(&p, 5));
        } else if blank {
            chan = chan.child(none(
                &p,
                "No sales in this range",
                "Revenue shows up after the first paid order.",
            ));
        } else {
            chan = chan.child(div().text_size(px(21.)).child(dollars_k(channel_total(r))));
            for (n, _, v) in oxyn_core::parity::workscr::channels(r) {
                let last = v[6];
                chan = chan.child(
                    div()
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .child(div().w(px(120.)).text_color(p.fg2).child(n))
                        .child(
                            div()
                                .h(px(6.))
                                .w(px((last / 60.) as f32))
                                .rounded(px(3.))
                                .bg(tone(&p, IconTone::Browser)),
                        )
                        .child(quiet(&p, dollars(last))),
                );
            }
        }
        let mut brk = card(&p)
            .w(px(342.))
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(2.))
            .child(
                div()
                    .text_color(p.mute)
                    .mb(px(10.))
                    .child("Revenue breakdown"),
            );
        for (i, (n, v, _)) in breakdown(r).into_iter().enumerate() {
            brk = brk.child(
                div()
                    .h(px(36.))
                    .px(px(8.))
                    .flex()
                    .items_center()
                    .rounded(px(8.))
                    .when(i % 2 == 1, |d| d.bg(p.inset))
                    .child(div().flex_1().text_color(p.mute).child(n))
                    .child(div().child(if blank { "$0".to_string() } else { dollars(v) })),
            );
        }
        let mut top = card(&p)
            .flex_1()
            .flex()
            .flex_col()
            .gap(px(14.))
            .child(div().text_color(p.mute).child("Top products"));
        let tp = top_products(r);
        let mx = tp.iter().map(|x| x.1).fold(1.0, f64::max);
        for (n, v) in tp {
            top = top.child(
                div()
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex()
                            .child(div().flex_1().text_color(p.mute).child(n))
                            .child(dollars(v)),
                    )
                    .child(
                        div()
                            .h(px(6.))
                            .w(px((v / mx * 300.) as f32))
                            .rounded(px(3.))
                            .bg(tone(&p, IconTone::Browser)),
                    ),
            );
        }
        let mut row3 = div().flex().gap(px(10.)).child(top);
        for (t, head, delta, _, _, _) in weekly(r) {
            row3 = row3.child(
                card(&p)
                    .flex_1()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(div().text_color(p.mute).child(t))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .child(div().text_size(px(21.)).child(head))
                            .child(div().text_color(good).child(delta)),
                    )
                    .child(quiet(&p, "This week vs. last week")),
            );
        }
        let body = div()
            .flex_1()
            .min_w_0()
            .flex()
            .flex_col()
            .gap(px(10.))
            .p(px(14.))
            .child(kp)
            .child(div().flex().gap(px(10.)).child(chan).child(brk))
            .child(row3);
        let rl = RANGES
            .iter()
            .find(|x| x.0 == r)
            .map(|x| x.1)
            .unwrap_or("Last 7 days");
        let acts = vec![
            btn(&p, rl, false),
            btn(&p, "Export", false),
            btn(&p, "Save as report", true),
        ];
        frame(
            self.theme,
            self.layout,
            side(&p, persona("revenue"), "revenue", st, &self.act),
            page(&p, Some(("Analytics", "Revenue".to_string())), acts, body),
            None,
        )
    }
}
