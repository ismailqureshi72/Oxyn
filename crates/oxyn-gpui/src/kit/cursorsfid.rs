//! affectUI fidelity pass, overnight batch, family B part 3 (plan #32) for GPUI: the exact
//! **Presence cursors** ([`PresenceCursorsEl`]) over `oxyn_core::kit::cursorsfid`. The element is
//! `RenderOnce`: the host keeps the clock, the tab, who is followed and the decided
//! suggestions, and routes [`Act`] ids back (`tab-<n>`, `follow-<n>`, `accept-all`,
//! `accept-<n>` / `reject-<n>` for suggestion n, `resolve-<n>` for comment n).
//!
//! Text is laid out by GPUI, so pointers are drawn as a caret in the text and the agent's flag
//! at the end of its block's first line, not measured onto the letter.

use gpui::{
    AnyElement, App, FontWeight, HighlightStyle, Hsla, IntoElement, ParentElement, RenderOnce,
    SharedString, StrikethroughStyle, Styled, StyledText, UnderlineStyle, Window, div, prelude::*,
    px, rgb,
};
use oxyn_core::kit::cursorsfid::{
    BlockKind, CursorScene, DocBlock, accept_suggestion, cursors_at, format_cost, presence_label,
    segments,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c};
use crate::brand_mark;

const TAB_ACTS: [&str; 4] = ["tab-0", "tab-1", "tab-2", "tab-3"];
const FOLLOW_ACTS: [&str; 6] = [
    "follow-0", "follow-1", "follow-2", "follow-3", "follow-4", "follow-5",
];
const ACCEPT_ACTS: [&str; 6] = [
    "accept-0", "accept-1", "accept-2", "accept-3", "accept-4", "accept-5",
];
const REJECT_ACTS: [&str; 6] = [
    "reject-0", "reject-1", "reject-2", "reject-3", "reject-4", "reject-5",
];

#[derive(Clone, Copy)]
struct Pal {
    bg: Hsla,
    line: Hsla,
    fg: Hsla,
    txt: Hsla,
    mute: Hsla,
    faint: Hsla,
    ink: Hsla,
    ink_fg: Hsla,
    ink_mute: Hsla,
    sel: Hsla,
    code: Hsla,
    img: Hsla,
    tab: Hsla,
    band: Hsla,
    tones: [(Hsla, Hsla); 5],
}

fn pal(t: &Theme) -> Pal {
    let h = |x: u32| -> Hsla { rgb(x).into() };
    if c(t, ColorRole::BgBase).l < 0.5 {
        Pal {
            bg: h(0x1C1C1F),
            line: h(0x2C2C31),
            fg: h(0xF4F4F5),
            txt: h(0xD4D4D8),
            mute: h(0x8E8E97),
            faint: h(0x6B6B73),
            ink: h(0xF4F4F5),
            ink_fg: h(0x111114),
            ink_mute: h(0x6B6B73),
            sel: alpha(h(0xC8C8E1), 0.16),
            code: h(0x232327),
            img: h(0x2A2A2E),
            tab: h(0x26262A),
            band: h(0x232327),
            tones: [
                (h(0x0C2E45), h(0x7DD3FC)),
                (h(0x2A2A2E), h(0xC4C4CC)),
                (h(0x3A2A0C), h(0xFBBF24)),
                (h(0x123222), h(0x4ADE80)),
                (h(0x3B1515), h(0xF87171)),
            ],
        }
    } else {
        Pal {
            bg: h(0xFFFFFF),
            line: h(0xE8E8EB),
            fg: h(0x18181B),
            txt: h(0x3F3F46),
            mute: h(0x8A8A93),
            faint: h(0xA1A1AA),
            ink: h(0x111114),
            ink_fg: h(0xFFFFFF),
            ink_mute: h(0xA1A1AA),
            sel: alpha(h(0x6E6E87), 0.17),
            code: h(0xFAFAFA),
            img: h(0xECECEE),
            tab: h(0xEDEDEF),
            band: h(0xF4F4F5),
            tones: [
                (h(0xE0F2FE), h(0x0369A1)),
                (h(0xF0F0F2), h(0x52525B)),
                (h(0xFEF3C7), h(0xB45309)),
                (h(0xDCFCE7), h(0x15803D)),
                (h(0xFEE2E2), h(0xB91C1C)),
            ],
        }
    }
}

fn tone(p: Pal, t: &str) -> Option<(Hsla, Hsla)> {
    let i = match t {
        "blue" => 0,
        "grey" => 1,
        "amber" => 2,
        "green" => 3,
        "red" => 4,
        _ => return None,
    };
    Some(p.tones[i])
}

/// `agent-cursors` (affectUI Agent Cursors): the host passes the scene (the demo's `doc` or
/// `table` tab at some ms, or its own), who is followed and the open suggestion.
#[derive(IntoElement)]
pub struct PresenceCursorsEl {
    scene: CursorScene,
    tabs: Vec<String>,
    tab: usize,
    following: Option<usize>,
    open: Option<usize>,
    height: f32,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for PresenceCursorsEl {
    fn default() -> Self {
        Self {
            scene: cursors_at("doc", 16_000),
            tabs: vec!["Doc".into(), "Table".into()],
            tab: 0,
            following: None,
            open: None,
            height: 470.0,
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl PresenceCursorsEl {
    /// A demo tab (`doc`, `table`) `ms` into its pass.
    pub fn demo(mut self, tab: &str, ms: u64) -> Self {
        self.scene = cursors_at(tab, ms);
        self.tab = usize::from(tab == "table");
        self
    }
    /// Your own document, tab labels and open tab.
    pub fn scene(mut self, scene: CursorScene, tabs: Vec<String>, tab: usize) -> Self {
        self.scene = scene;
        self.tabs = tabs;
        self.tab = tab;
        self
    }
    /// Writes suggestions (by id) into the document, as Accept does.
    pub fn accepted(mut self, ids: &[&str]) -> Self {
        for id in ids {
            let (b, s) = accept_suggestion(&self.scene.blocks, &self.scene.suggestions, id);
            self.scene.blocks = b;
            self.scene.suggestions = s;
        }
        self
    }
    /// The agent followed (index): the other pointers fade back.
    pub fn following(mut self, i: Option<usize>) -> Self {
        self.following = i;
        self
    }
    /// The suggestion (index) whose window is open.
    pub fn open(mut self, i: Option<usize>) -> Self {
        self.open = i;
        self
    }
    /// A fixed page height (px; 0 = all of it).
    pub fn height(mut self, h: f32) -> Self {
        self.height = h;
        self
    }
    /// Classic style.
    pub fn classic(mut self, on: bool) -> Self {
        self.classic = on;
        self
    }
    /// Light or dark.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
    /// Acts.
    pub fn on_act(mut self, a: Act) -> Self {
        self.act = Some(a);
        self
    }
}

fn fire(a: &Option<Act>, id: &'static str) -> impl Fn(&mut Window, &mut App) + 'static {
    let a = a.clone();
    move |w, cx| {
        if let Some(f) = &a {
            f(id, w, cx)
        }
    }
}

impl PresenceCursorsEl {
    /// One block's text: struck / underlined suggestions, selections, carets as a thin bar.
    fn text(&self, p: Pal, block: &str, text: &str, base: Hsla) -> (StyledText, Vec<usize>) {
        let segs = segments(block, text, &self.scene.cursors, &self.scene.suggestions);
        let mut s = String::new();
        let mut hl: Vec<(std::ops::Range<usize>, HighlightStyle)> = vec![];
        let mut here = vec![];
        for g in segs {
            let faded =
                |ai: Option<usize>| ai.is_some_and(|a| self.following.is_some_and(|f| f != a));
            if let Some(ai) = g.caret {
                here.push(ai);
                let a = s.len();
                s.push('▏');
                hl.push((
                    a..s.len(),
                    HighlightStyle {
                        color: Some(alpha(p.ink, if faded(Some(ai)) { 0.3 } else { 1.0 })),
                        ..Default::default()
                    },
                ));
                continue;
            }
            let a = s.len();
            s.push_str(&g.text);
            let r = a..s.len();
            if r.is_empty() {
                continue;
            }
            if g.del.is_some() {
                hl.push((
                    r,
                    HighlightStyle {
                        color: Some(p.faint),
                        strikethrough: Some(StrikethroughStyle {
                            thickness: px(1.),
                            color: Some(p.faint),
                        }),
                        ..Default::default()
                    },
                ));
            } else if g.ins.is_some() {
                hl.push((
                    r,
                    HighlightStyle {
                        color: Some(p.fg),
                        background_color: Some(alpha(p.fg, 0.06)),
                        underline: Some(UnderlineStyle {
                            thickness: px(1.5),
                            color: Some(p.fg),
                            wavy: false,
                        }),
                        ..Default::default()
                    },
                ));
            } else if g.sel.is_some() {
                hl.push((
                    r,
                    HighlightStyle {
                        background_color: Some(alpha(p.sel, if faded(g.sel) { 0.4 } else { 1.0 })),
                        ..Default::default()
                    },
                ));
            } else {
                hl.push((
                    r,
                    HighlightStyle {
                        color: Some(base),
                        ..Default::default()
                    },
                ));
            }
        }
        (StyledText::new(s).with_highlights(hl), here)
    }

    fn flag(&self, p: Pal, i: usize) -> AnyElement {
        let a = &self.scene.agents[i];
        let doing = self.scene.cursors[i]
            .as_ref()
            .map(|c| c.doing.clone())
            .unwrap_or_default();
        let fade = self.following.is_some_and(|f| f != i);
        div()
            .absolute()
            .right(px(-8.))
            .top(px(-2.))
            .flex()
            .gap(px(4.))
            .opacity(if fade { 0.28 } else { 1.0 })
            .child(div().text_color(p.ink).text_size(px(14.)).child("➤"))
            .child(
                div()
                    .px(px(8.))
                    .py(px(5.))
                    .rounded(px(9.))
                    .bg(p.ink)
                    .text_color(p.ink_fg)
                    .text_size(px(12.))
                    .flex()
                    .flex_col()
                    .gap(px(1.))
                    .child(
                        div()
                            .flex()
                            .gap(px(8.))
                            .items_center()
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(a.name.clone()),
                            )
                            .child(
                                div()
                                    .px(px(5.))
                                    .rounded(px(4.))
                                    .bg(alpha(p.ink_fg, 0.14))
                                    .text_size(px(11.))
                                    .child(format_cost(a.cost)),
                            ),
                    )
                    .when(!doing.is_empty(), |d| {
                        d.child(
                            div()
                                .text_color(p.ink_mute)
                                .text_size(px(11.5))
                                .child(doing),
                        )
                    }),
            )
            .into_any_element()
    }

    fn block(&self, p: Pal, b: &DocBlock) -> AnyElement {
        let (size, weight, color, mb) = match b.kind {
            BlockKind::Title => (20.0, FontWeight::SEMIBOLD, p.fg, 12.0),
            BlockKind::Heading => (14.5, FontWeight::SEMIBOLD, p.fg, 8.0),
            BlockKind::Code => (12.5, FontWeight::NORMAL, p.txt, 18.0),
            _ => (14.5, FontWeight::NORMAL, p.txt, 14.0),
        };
        if b.kind == BlockKind::Image {
            let holder = self
                .scene
                .cursors
                .iter()
                .position(|c| c.as_ref().is_some_and(|c| c.block == b.id));
            let held = self
                .scene
                .cursors
                .iter()
                .flatten()
                .any(|c| c.block == b.id && c.select.is_some());
            let w = 612.0 * b.width.clamp(0.05, 1.0);
            let mut img = div()
                .relative()
                .w(px(w))
                .h(px(w / b.ratio.max(0.1)))
                .mb(px(14.))
                .rounded(px(8.))
                .bg(if b.loading {
                    p.img
                } else {
                    alpha(rgb(0xE3B8A8).into(), 0.55)
                })
                .when(held, |d| d.border_1().border_color(p.ink));
            if let Some(i) = holder {
                img = img.child(self.flag(p, i));
            }
            return img.into_any_element();
        }
        let (st, here) = self.text(p, &b.id, &b.text, color);
        let mut d = div()
            .relative()
            .mb(px(mb))
            .text_size(px(size))
            .font_weight(weight)
            .line_height(px(size * 1.52))
            .child(st);
        if b.kind == BlockKind::Code {
            d = d
                .px(px(14.))
                .py(px(12.))
                .rounded(px(8.))
                .bg(p.code)
                .font_family("monospace");
        }
        if b.kind == BlockKind::Item {
            d = d.pl(px(20.));
        }
        for i in here {
            d = d.child(self.flag(p, i));
        }
        d.into_any_element()
    }

    fn table(&self, p: Pal, b: &DocBlock) -> AnyElement {
        let mut t = div()
            .mt(px(6.))
            .border_1()
            .border_color(p.line)
            .rounded(px(10.))
            .flex()
            .flex_col();
        for (r, row) in b.rows.iter().enumerate() {
            let mut tr = div()
                .h(px(40.))
                .flex()
                .items_center()
                .when(r > 0, |d| d.border_t_1().border_color(p.line));
            for (k, cell) in row.iter().enumerate() {
                let col = if r == 0 { p.mute } else { p.fg };
                let (st, here) = self.text(p, &cell.id, &cell.text, col);
                let pending = self
                    .scene
                    .suggestions
                    .iter()
                    .find(|s| s.block == cell.id)
                    .map(|s| s.insert.clone());
                let word = pending.unwrap_or_else(|| cell.text.clone());
                let tn = match word.as_str() {
                    "Doing" => "blue",
                    "To do" => "grey",
                    "Idea" => "amber",
                    "Done" => "green",
                    _ => cell.tone.as_str(),
                };
                let inner = match tone(p, tn).filter(|_| !cell.tone.is_empty()) {
                    Some((bg, fg)) => div()
                        .px(px(7.))
                        .rounded(px(5.))
                        .bg(bg)
                        .text_color(fg)
                        .text_size(px(12.))
                        .child(st)
                        .into_any_element(),
                    None => div()
                        .text_size(px(if r == 0 { 12.5 } else { 13.5 }))
                        .child(st)
                        .into_any_element(),
                };
                let mut td = div()
                    .relative()
                    .px(px(12.))
                    .child(inner)
                    .when(k == 0, |d| d.w(px(220.)))
                    .when(k > 0, |d| d.flex_1());
                for i in here {
                    td = td.child(self.flag(p, i));
                }
                tr = tr.child(td);
            }
            t = t.child(tr);
        }
        t = t.child(
            div()
                .h(px(40.))
                .px(px(12.))
                .border_t_1()
                .border_color(p.line)
                .flex()
                .items_center()
                .text_size(px(13.))
                .text_color(p.mute)
                .child("+  Add row"),
        );
        t.into_any_element()
    }
}

impl RenderOnce for PresenceCursorsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let p = pal(self.theme);
        let sc = &self.scene;
        let n_sugg = sc.suggestions.len();
        let here = sc.cursors.iter().flatten().count();
        let total: f64 = sc.agents.iter().map(|a| a.cost).sum();

        let mut tabs = div().flex().gap(px(4.)).mb(px(7.));
        for (i, l) in self.tabs.iter().enumerate().take(TAB_ACTS.len()) {
            let on = i == self.tab;
            let f = fire(&self.act, TAB_ACTS[i]);
            tabs = tabs.child(
                div()
                    .id(SharedString::from(format!("acu-tab-{i}")))
                    .h(px(28.))
                    .px(px(10.))
                    .rounded(px(7.))
                    .flex()
                    .items_center()
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .cursor_pointer()
                    .when(on, |d| {
                        d.bg(p.bg).border_1().border_color(p.line).text_color(p.fg)
                    })
                    .when(!on, |d| d.bg(p.tab).text_color(p.mute))
                    .on_click(move |_, w, cx| f(w, cx))
                    .child(l.clone()),
            );
        }

        let mut faces = div().flex().gap(px(3.)).items_center().mr(px(4.));
        for (i, a) in sc.agents.iter().enumerate().take(FOLLOW_ACTS.len()) {
            let inside = sc.cursors.get(i).is_some_and(|c| c.is_some());
            let pressed = self.following == Some(i);
            let dim = self.following.is_some() && !pressed;
            let f = fire(&self.act, FOLLOW_ACTS[i]);
            let mut face = div()
                .id(SharedString::from(format!("acu-face-{i}")))
                .size(px(24.))
                .rounded_full()
                .flex()
                .items_center()
                .justify_center()
                .cursor_pointer()
                .opacity(if dim {
                    0.35
                } else if inside || pressed {
                    1.0
                } else {
                    0.62
                })
                .when(inside, |d| d.border_1().border_color(p.ink))
                .when(pressed, |d| d.bg(alpha(p.mute, 0.18)))
                .on_click(move |_, w, cx| f(w, cx));
            if let Some(m) = brand_mark(&a.brand, 12.0, p.fg) {
                face = face.child(m);
            }
            faces = faces.child(face);
        }
        let all = fire(&self.act, "accept-all");
        let head = div()
            .h(px(44.))
            .pl(px(14.))
            .pr(px(6.))
            .flex()
            .items_center()
            .gap(px(8.))
            .when(!self.classic, |d| d.border_b_1().border_color(p.line))
            .child(div().text_color(p.mute).text_size(px(13.)).child("▤"))
            .child(
                div()
                    .text_color(p.mute)
                    .text_size(px(13.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(sc.title.clone()),
            )
            .child(div().flex_1())
            .child(faces)
            .child(
                div()
                    .text_size(px(12.5))
                    .text_color(p.mute)
                    .child(presence_label(n_sugg, here)),
            )
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.fg)
                    .child(format_cost(total)),
            )
            .child(
                div()
                    .id("acu-all")
                    .h(px(28.))
                    .px(px(11.))
                    .rounded(px(7.))
                    .flex()
                    .items_center()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .when(n_sugg > 0, |d| {
                        d.bg(p.ink)
                            .text_color(p.ink_fg)
                            .cursor_pointer()
                            .on_click(move |_, w, cx| all(w, cx))
                    })
                    .when(n_sugg == 0, |d| d.bg(alpha(p.mute, 0.35)).text_color(p.bg))
                    .child("Accept all"),
            );

        let mut doc = div().px(px(44.)).pt(px(30.)).pb(px(40.)).flex().flex_col();
        for b in sc.blocks.iter() {
            doc = doc.child(if b.kind == BlockKind::Table {
                self.table(p, b)
            } else {
                self.block(p, b)
            });
        }
        // The open suggestion's window, under the document's header.
        if let Some(i) = self.open.filter(|i| *i < sc.suggestions.len().min(6)) {
            let s = &sc.suggestions[i];
            let name = sc
                .agents
                .iter()
                .find(|a| a.id == s.agent)
                .map(|a| a.name.clone())
                .unwrap_or_default();
            let (acc, rej) = (
                fire(&self.act, ACCEPT_ACTS[i]),
                fire(&self.act, REJECT_ACTS[i]),
            );
            doc = doc.child(
                div()
                    .absolute()
                    .left(px(44.))
                    .top(px(66.))
                    .w(px(250.))
                    .p(px(10.))
                    .rounded(px(12.))
                    .border_1()
                    .border_color(alpha(p.mute, 0.3))
                    .bg(rgb(0xFFFFFF))
                    .shadow_lg()
                    .text_color(rgb(0x18181B))
                    .text_size(px(13.))
                    .flex()
                    .flex_col()
                    .gap(px(6.))
                    .child(
                        div()
                            .flex()
                            .gap(px(6.))
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(name))
                            .child(div().text_color(rgb(0x71717A)).child("suggests a rewrite")),
                    )
                    .child(div().text_color(rgb(0x3F3F46)).child(s.note.clone()))
                    .child(
                        div()
                            .flex()
                            .justify_end()
                            .gap(px(6.))
                            .child(
                                div()
                                    .id("acu-reject")
                                    .h(px(28.))
                                    .px(px(11.))
                                    .rounded(px(7.))
                                    .border_1()
                                    .border_color(rgb(0xE4E4E7))
                                    .flex()
                                    .items_center()
                                    .cursor_pointer()
                                    .on_click(move |_, w, cx| rej(w, cx))
                                    .child("Reject"),
                            )
                            .child(
                                div()
                                    .id("acu-accept")
                                    .h(px(28.))
                                    .px(px(11.))
                                    .rounded(px(7.))
                                    .bg(rgb(0x111114))
                                    .text_color(rgb(0xFFFFFF))
                                    .flex()
                                    .items_center()
                                    .cursor_pointer()
                                    .on_click(move |_, w, cx| acc(w, cx))
                                    .child("Accept"),
                            ),
                    ),
            );
        }
        let body = div()
            .relative()
            .overflow_hidden()
            .when(self.height > 0.0, |d| d.h(px(self.height)))
            .when(self.classic, |d| {
                d.mx(px(5.)).mb(px(5.)).rounded(px(12.)).bg(p.bg)
            })
            .child(doc);
        let page = div()
            .border_1()
            .border_color(p.line)
            .rounded(px(if self.classic { 16. } else { 12. }))
            .bg(if self.classic { p.band } else { p.bg })
            .when(self.classic, |d| d.shadow_md())
            .overflow_hidden()
            .child(head)
            .child(body);
        div()
            .w(px(700.))
            .text_color(p.fg)
            .flex()
            .flex_col()
            .when(!self.tabs.is_empty(), |d| d.child(tabs))
            .child(page)
    }
}
