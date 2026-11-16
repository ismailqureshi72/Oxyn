//! affectUI fidelity pass, family B part 9 (plan #32) for GPUI: the exact **Root Cause Tree**
//! ([`RootCauseTreeEl`]) over `oxyn_core::parity::rootfid`. The element is `RenderOnce`: the host
//! passes the failure tree and the run time and gets [`Act`] ids back (`node-<n>`, depth-first
//! with the failure as `node-0`). A static snapshot: no spinner, fades or evidence swap motion.

use gpui::{
    App, Div, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled,
    Window, div, prelude::*, px, rgb,
};
use oxyn_core::parity::rootfid::{
    RcEvidence, RcGate, RcNode, RcStatus, rc_counts, rc_demo, rc_flat, rc_fmt, rc_found, rc_len,
    rc_path, rc_roots, rc_status,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, c};

const NODE: [&str; 16] = [
    "node-0", "node-1", "node-2", "node-3", "node-4", "node-5", "node-6", "node-7", "node-8",
    "node-9", "node-10", "node-11", "node-12", "node-13", "node-14", "node-15",
];

/// `root-cause-tree` (affectUI Root Cause Tree): the host passes the tree and the time.
#[derive(IntoElement)]
pub struct RootCauseTreeEl {
    failure: RcNode,
    at: u32,
    selected: Option<String>,
    classic: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl Default for RootCauseTreeEl {
    fn default() -> Self {
        let f = rc_demo();
        let at = rc_len(&f);
        Self {
            failure: f,
            at,
            selected: Some("round".into()),
            classic: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
}

impl RootCauseTreeEl {
    /// The failure tree and the run seconds.
    pub fn tree(mut self, failure: RcNode, at: u32) -> Self {
        self.failure = failure;
        self.at = at;
        self
    }
    /// The node whose evidence shows.
    pub fn selected(mut self, id: Option<String>) -> Self {
        self.selected = id;
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
    /// Node clicks.
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

#[derive(Clone, Copy)]
struct Pal {
    card: Hsla,
    bd: Hsla,
    fg: Hsla,
    mute: Hsla,
    line: Hsla,
    chip: Hsla,
    path: Hsla,
    red: Hsla,
    green: Hsla,
}

struct Cx<'a> {
    t: u32,
    sel: &'a str,
    path: Vec<String>,
    roots: Vec<String>,
    found: bool,
    order: Vec<String>,
    act: &'a Option<Act>,
    p: Pal,
    classic: bool,
}

fn line_of(n: &RcNode, st: RcStatus, root: bool) -> String {
    let at = n.check.map(|c| rc_fmt(c.1)).unwrap_or_default();
    match st {
        RcStatus::Open => "Not checked".into(),
        RcStatus::Checking => "Checking".into(),
        RcStatus::RuledOut => format!("Ruled out · {at}"),
        RcStatus::Confirmed if root => format!("Root cause · {at}"),
        RcStatus::Confirmed => format!("Confirmed · {at}"),
    }
}

fn node(n: &RcNode, depth: usize, cx: &Cx) -> Div {
    let p = cx.p;
    let st = rc_status(n, cx.t);
    let root = cx.roots.contains(&n.id);
    let on_path = cx.path.contains(&n.id) || (depth == 0 && cx.found);
    let dim = cx.found && !on_path && depth > 0;
    let idx = cx
        .order
        .iter()
        .position(|x| x == &n.id)
        .unwrap_or(0)
        .min(15);
    let f = fire(cx.act, NODE[idx]);
    let (line, tone) = if depth == 0 {
        if cx.found {
            ("Cause found".to_string(), p.path)
        } else {
            ("Failed".to_string(), p.red)
        }
    } else {
        let tone = match st {
            RcStatus::Confirmed => p.red,
            RcStatus::Checking => p.path,
            _ => p.mute,
        };
        (line_of(n, st, root), tone)
    };
    let selected = cx.sel == n.id;
    let mut card = div()
        .id(SharedString::from(format!("oxrt-{}", n.id)))
        .w(px(if depth == 0 { 150. } else { 124. }))
        .p(px(8.))
        .flex()
        .flex_col()
        .gap(px(4.))
        .border_1()
        .border_color(if selected || on_path { p.path } else { p.bd })
        .rounded(px(if cx.classic { 6. } else { 10. }))
        .bg(p.card)
        .when(dim, |d| d.opacity(0.4))
        .on_click(move |_, w, cx| f(w, cx))
        .child(
            div()
                .text_size(px(12.))
                .font_weight(FontWeight::SEMIBOLD)
                .text_color(
                    if matches!(st, RcStatus::Open | RcStatus::RuledOut) && depth > 0 {
                        p.mute
                    } else {
                        p.fg
                    },
                )
                .child(n.title.clone()),
        );
    if let Some(first) = n.files.first() {
        let more = n.files.len().saturating_sub(1);
        card = card.child(
            div()
                .flex()
                .gap(px(4.))
                .text_size(px(10.))
                .text_color(p.mute)
                .child(
                    div()
                        .px(px(5.))
                        .rounded(px(5.))
                        .bg(p.chip)
                        .child(first.clone()),
                )
                .when(more > 0, |d| d.child(format!("+{more}"))),
        );
    }
    card = card.child(div().text_size(px(11.)).text_color(tone).child(line));
    let mut col = div().flex().flex_col().items_center().child(card);
    if !n.children.is_empty() {
        col = col
            .child(
                div()
                    .w(px(1.))
                    .h(px(8.))
                    .bg(if on_path { p.path } else { p.line }),
            )
            .child(
                div()
                    .px(px(6.))
                    .border_1()
                    .border_color(if on_path { p.path } else { p.bd })
                    .rounded(px(999.))
                    .text_size(px(9.))
                    .text_color(if on_path { p.path } else { p.mute })
                    .child(gate_label(n)),
            )
            .child(div().w(px(1.)).h(px(8.)).bg(p.line));
        let mut kids = div()
            .flex()
            .gap(px(10.))
            .border_t_1()
            .border_color(p.line)
            .pt(px(8.));
        for k in &n.children {
            kids = kids.child(node(k, depth + 1, cx));
        }
        col = col.child(kids);
    }
    col
}

fn gate_label(n: &RcNode) -> &'static str {
    match n.gate {
        RcGate::Or => "OR",
        RcGate::And => "AND",
    }
}

impl RenderOnce for RootCauseTreeEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let dark = c(self.theme, ColorRole::BgBase).l < 0.5;
        let h = |x: u32| -> Hsla { rgb(x).into() };
        let p = if dark {
            Pal {
                card: h(0x1B1B1B),
                bd: h(0x2E2E2E),
                fg: h(0xF2F2F2),
                mute: h(0x85858C),
                line: h(0x3A3A3E),
                chip: h(0x262628),
                path: h(0x9D8CF5),
                red: h(0xF87171),
                green: h(0x4ADE80),
            }
        } else {
            Pal {
                card: h(0xFFFFFF),
                bd: h(0xECECEC),
                fg: h(0x111111),
                mute: h(0x8A8A8E),
                line: h(0xD9D9DC),
                chip: h(0xF3F3F4),
                path: h(0x6E56CF),
                red: h(0xC0362C),
                green: h(0x1A7F4B),
            }
        };
        let f = &self.failure;
        let t = self.at;
        let found = rc_found(f, t);
        let (n_h, n_out, _) = rc_counts(f, t);
        let order: Vec<String> = std::iter::once(f.id.clone())
            .chain(rc_flat(f).iter().map(|(n, _)| n.id.clone()))
            .collect();
        let sel = self.selected.clone().unwrap_or_else(|| f.id.clone());
        let cx = Cx {
            t,
            sel: &sel,
            path: rc_path(f, t),
            roots: rc_roots(f, t),
            found,
            order,
            act: &self.act,
            p,
            classic: self.classic,
        };
        let tree = node(f, 0, &cx);
        let flat = rc_flat(f);
        let shown = flat
            .iter()
            .find(|(n, _)| n.id == sel)
            .map(|(n, _)| (*n).clone())
            .unwrap_or_else(|| f.clone());
        let shown_st = rc_status(&shown, t);
        let mut ev = div()
            .p(px(12.))
            .flex()
            .flex_col()
            .gap(px(6.))
            .border_1()
            .border_color(p.bd)
            .rounded(px(12.))
            .bg(p.card)
            .child(
                div()
                    .text_size(px(13.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(shown.title.clone()),
            )
            .child(
                div()
                    .text_size(px(12.))
                    .text_color(if shown.verdict.is_empty() {
                        p.mute
                    } else {
                        p.fg
                    })
                    .child(if shown.verdict.is_empty() {
                        "Not checked yet.".to_string()
                    } else {
                        shown.verdict.clone()
                    }),
            );
        if matches!(shown_st, RcStatus::RuledOut | RcStatus::Confirmed) {
            let (label, ls) = match &shown.evidence {
                Some(RcEvidence::Tool(n, ls)) => (format!("TOOL CALL  {n}"), ls.clone()),
                Some(RcEvidence::Diff(fl, ls)) => (format!("DIFF  {fl}"), ls.clone()),
                None => (String::new(), vec![]),
            };
            if !ls.is_empty() {
                let mut code = div().p(px(8.)).rounded(px(8.)).bg(p.chip).flex().flex_col();
                for l in ls {
                    let tone = if l.starts_with('-') {
                        p.red
                    } else if l.starts_with('+') {
                        p.green
                    } else {
                        p.fg
                    };
                    code = code.child(div().text_size(px(11.)).text_color(tone).child(l));
                }
                ev = ev
                    .child(div().text_size(px(10.)).text_color(p.mute).child(label))
                    .child(code);
            }
        }
        div()
            .w(px(720.))
            .flex()
            .flex_col()
            .gap(px(14.))
            .text_color(p.fg)
            .when(self.classic, |d| {
                d.p(px(12.)).border_1().border_color(p.bd).rounded(px(12.))
            })
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Root cause"),
                    )
                    .when(!found, |d| {
                        d.child(
                            div()
                                .px(px(8.))
                                .rounded(px(999.))
                                .text_size(px(11.))
                                .text_color(p.path)
                                .child("Investigating"),
                        )
                    })
                    .child(div().flex_1())
                    .child(
                        div()
                            .text_size(px(12.))
                            .text_color(p.mute)
                            .child(format!("{n_h} hypotheses · {n_out} ruled out")),
                    ),
            )
            .child(div().flex().justify_center().child(tree))
            .child(ev)
    }
}
