//! Merge conflicts for GPUI (plan #31), drawn from `oxyn_core::parity::merge`: the 3-way
//! merge-conflict editor (`MergeConflictEl`) and Conflict Radar (`ConflictRadarEl`).
//! Elements are `RenderOnce`; the host keeps the [`Merge`] / [`Radar`] model, routes the
//! callbacks into it (`merge.take(..)`, `merge.go(..)`, `radar.resolve(..)`) and re-renders.
//! Keys: run [`oxyn_core::parity::merge::merge_key`] in the host's key handler.

use std::rc::Rc;

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative,
};
use oxyn_core::parity::merge::{Merge, Phase, Radar, Seg, count_markers, marker_block, pick_lines};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Weight, alpha, button, c, muted, tag};

/// Merge callback: conflict (or file) index, action id (`ours`, `theirs`, `both`, `base`,
/// `agent`, `reopen`, `focus`, `next`, `prev`, `undo`, `mark`, `complete`, `abort`, `confirm`,
/// `cancel`, `file`) and the window.
pub type MergeAct = Rc<dyn Fn(usize, &'static str, &mut Window, &mut App)>;
/// Radar callback: file path, agent id and the window.
pub type RadarAct = Rc<dyn Fn(String, String, &mut Window, &mut App)>;

fn side_color(t: &'static Theme, side: &str) -> Hsla {
    match side {
        "ours" => c(t, ColorRole::MergeCurrent),
        "theirs" => c(t, ColorRole::MergeIncoming),
        "base" => c(t, ColorRole::MergeBase),
        _ => c(t, ColorRole::FgDefault),
    }
}

fn code_line(t: &'static Theme, no: Option<usize>, text: &str, marker: bool) -> gpui::Div {
    let txt: SharedString = if text.is_empty() {
        " ".into()
    } else {
        text.to_string().into()
    };
    div()
        .flex()
        .w_full()
        .min_h(px(20.))
        .font_family("monospace")
        .text_size(px(12.))
        .when(marker, |d| {
            d.bg(c(t, ColorRole::MergeMarker))
                .text_color(c(t, ColorRole::StatusDangerFg))
        })
        .child(
            div()
                .flex_none()
                .w(px(36.))
                .pr(px(8.))
                .text_color(c(t, ColorRole::FgSubtle))
                .flex()
                .justify_end()
                .child(no.map(|n| n.to_string()).unwrap_or_default()),
        )
        .child(div().flex_1().min_w(px(0.)).pr(px(10.)).child(txt))
}

/// `merge-conflict`: Ours · Base · Theirs panes and a Result pane with per-conflict actions.
#[derive(IntoElement)]
pub struct MergeConflictEl {
    merge: Merge,
    ours_ref: SharedString,
    theirs_ref: SharedString,
    base_ref: SharedString,
    layout: &'static str,
    show_base: bool,
    suggestions: bool,
    act: Option<MergeAct>,
    theme: &'static Theme,
}

impl MergeConflictEl {
    /// An editor over `merge`.
    pub fn new(merge: Merge) -> Self {
        Self {
            merge,
            ours_ref: "main".into(),
            theirs_ref: "feature".into(),
            base_ref: "merge-base".into(),
            layout: "three-pane",
            show_base: true,
            suggestions: true,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Branch names and the base label.
    pub fn refs(
        mut self,
        ours: impl Into<SharedString>,
        theirs: impl Into<SharedString>,
        base: impl Into<SharedString>,
    ) -> Self {
        self.ours_ref = ours.into();
        self.theirs_ref = theirs.into();
        self.base_ref = base.into();
        self
    }
    /// `three-pane` · `result-center` · `inline`.
    pub fn layout(mut self, l: &'static str) -> Self {
        self.layout = l;
        self
    }
    /// Show the base pane.
    pub fn show_base(mut self, b: bool) -> Self {
        self.show_base = b;
        self
    }
    /// Show agent suggestions.
    pub fn suggestions(mut self, s: bool) -> Self {
        self.suggestions = s;
        self
    }
    /// Action callback.
    pub fn on_action(mut self, f: MergeAct) -> Self {
        self.act = Some(f);
        self
    }
    /// Theme.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }

    fn btn(
        &self,
        id: impl Into<gpui::ElementId>,
        label: impl Into<SharedString>,
        w: Weight,
        n: usize,
        action: &'static str,
    ) -> impl IntoElement {
        let a = self.act.clone();
        let wrapped: Option<Act> = a.map(|f| {
            Rc::new(move |id: &'static str, win: &mut Window, cx: &mut App| f(n, id, win, cx))
                as Act
        });
        button(self.theme, id, label, w, action, wrapped)
    }

    fn block(&self, n: usize, start: usize) -> gpui::Div {
        let t = self.theme;
        let m = &self.merge;
        let cf = m.cur().conflicts()[n].clone();
        let pk = m.pick(n).cloned();
        let resolved = m.is_resolved(m.file, n);
        let focus = n == m.focus;
        let readonly = m.marked.contains(&m.file) || m.phase != Phase::Open;
        let edge = if resolved {
            c(t, ColorRole::MergeResolved)
        } else {
            c(t, ColorRole::MergeConflict)
        };
        let lines: Vec<(String, bool)> = match &pk {
            Some(p) => pick_lines(&cf, p)
                .into_iter()
                .map(|l| {
                    let mk = count_markers(&l) > 0;
                    (l, mk)
                })
                .collect(),
            None => marker_block(&cf, &self.ours_ref, &self.theirs_ref)
                .into_iter()
                .map(|l| {
                    let mk = count_markers(&l) > 0;
                    (l, mk)
                })
                .collect(),
        };
        let state: SharedString = match &pk {
            Some(p) if resolved => p.tag().into(),
            Some(_) => "⚑ markers left".into(),
            None => "unresolved".into(),
        };
        let mut d = div()
            .flex()
            .flex_col()
            .mx(px(8.))
            .my(px(4.))
            .rounded(px(9.))
            .border_1()
            .border_color(alpha(edge, if focus { 1.0 } else { 0.45 }))
            .bg(alpha(edge, 0.05))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .px(px(10.))
                    .py(px(4.))
                    .child(
                        div()
                            .text_size(px(11.5))
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(c(t, ColorRole::FgMuted))
                            .child(format!("Conflict {}", n + 1)),
                    )
                    .child(tag(
                        t,
                        state,
                        Some(if resolved {
                            ColorRole::MergeResolved
                        } else if pk.is_some() {
                            ColorRole::StatusDangerFg
                        } else {
                            ColorRole::MergeConflict
                        }),
                    ))
                    .child(div().flex_1())
                    .when(pk.is_some() && !readonly, |d| {
                        d.child(self.btn(("mrg-reopen", n), "Reopen", Weight::Quiet, n, "reopen"))
                    }),
            );
        for (i, (l, mk)) in lines.iter().enumerate() {
            d = d.child(code_line(t, Some(start + i), l, *mk));
        }
        if let Some(s) = cf
            .suggestion
            .clone()
            .filter(|_| self.suggestions && !resolved)
        {
            d = d.child(
                div()
                    .mx(px(10.))
                    .my(px(4.))
                    .p(px(8.))
                    .rounded(px(7.))
                    .bg(alpha(c(t, ColorRole::MergeSuggested), 0.1))
                    .text_size(px(12.))
                    .flex()
                    .gap(px(6.))
                    .child(div().text_color(c(t, ColorRole::MergeSuggested)).child("✦"))
                    .child(div().flex_1().child(s.rationale.clone()))
                    .when_some(s.confidence, |d, conf| {
                        d.child(muted(t, format!("{conf} confidence")))
                    }),
            );
        }
        if !resolved && !readonly {
            let mut row = div()
                .flex()
                .flex_wrap()
                .gap(px(6.))
                .px(px(10.))
                .pb(px(9.))
                .pt(px(4.));
            row = row
                .child(self.btn(("mrg-ours", n), "Accept ours", Weight::Plain, n, "ours"))
                .child(self.btn(
                    ("mrg-theirs", n),
                    "Accept theirs",
                    Weight::Plain,
                    n,
                    "theirs",
                ))
                .child(self.btn(("mrg-both", n), "Accept both", Weight::Plain, n, "both"));
            if cf.base.is_some() {
                row = row.child(self.btn(("mrg-base", n), "Keep base", Weight::Quiet, n, "base"));
            }
            if cf.suggestion.is_some() && self.suggestions {
                row = row.child(self.btn(
                    ("mrg-agent", n),
                    "✦ Accept suggestion",
                    Weight::Plain,
                    n,
                    "agent",
                ));
            }
            d = d.child(row);
        }
        d
    }
}

impl RenderOnce for MergeConflictEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let m = &self.merge;
        let f = m.file;
        let file = m.cur().clone();
        let total = m.total(f);
        let done_all = m.resolved_all();
        let total_all = m.total_all();
        let pct = if total_all == 0 {
            1.0
        } else {
            done_all as f32 / total_all as f32
        };
        let base_on = self.show_base && file.conflicts().iter().any(|c| c.base.is_some());
        let sides: Vec<&'static str> = if self.layout == "result-center" {
            vec!["ours", "result", "theirs"]
        } else if base_on {
            vec!["ours", "base", "theirs"]
        } else {
            vec!["ours", "theirs"]
        };
        let pane_name = |s: &str| -> String {
            match s {
                "ours" => format!("Ours · {}", self.ours_ref),
                "theirs" => format!("Theirs · {}", self.theirs_ref),
                "base" => format!("Base · {}", self.base_ref),
                _ => "Result".to_string(),
            }
        };

        // header
        let mut root = div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(12.))
                    .px(px(14.))
                    .py(px(9.))
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(format!(
                        "Merging {} into {}",
                        self.theirs_ref, self.ours_ref
                    )))
                    .child(muted(t, format!("{done_all} of {total_all} resolved")))
                    .child(
                        div()
                            .w(px(72.))
                            .h(px(4.))
                            .rounded_full()
                            .bg(c(t, ColorRole::SurfaceSunken))
                            .child(
                                div()
                                    .h_full()
                                    .w(relative(pct))
                                    .rounded_full()
                                    .bg(c(t, ColorRole::MergeResolved)),
                            ),
                    )
                    .child(div().flex_1())
                    .child(self.btn("mrg-prev", "‹", Weight::Quiet, 0, "prev"))
                    .child(div().text_size(px(12.)).child(if total == 0 {
                        "No conflicts".to_string()
                    } else {
                        format!("Conflict {} of {total}", m.focus + 1)
                    }))
                    .child(self.btn("mrg-next", "›", Weight::Quiet, 0, "next"))
                    .child(self.btn("mrg-undo", "Undo", Weight::Quiet, 0, "undo")),
            );

        // file strip
        if m.files.len() > 1 {
            let mut strip = div()
                .flex()
                .flex_wrap()
                .gap(px(4.))
                .px(px(10.))
                .py(px(6.))
                .bg(c(t, ColorRole::SurfaceSunken))
                .border_b_1()
                .border_color(c(t, ColorRole::BorderSubtle));
            for (i, mf) in m.files.iter().enumerate() {
                let label = if m.marked.contains(&i) {
                    format!("{}  ✓ staged", mf.path)
                } else {
                    format!("{}  {}/{}", mf.path, m.resolved(i), m.total(i))
                };
                strip = strip.child(self.btn(
                    ("mrg-file", i),
                    label,
                    if i == f { Weight::Plain } else { Weight::Quiet },
                    i,
                    "file",
                ));
            }
            root = root.child(strip);
        }

        match m.phase {
            Phase::Completed | Phase::Aborted => {
                let (title, sub) = if m.phase == Phase::Completed {
                    (
                        format!("✓ Merged {} into {}", self.theirs_ref, self.ours_ref),
                        format!("{total_all} conflicts resolved."),
                    )
                } else {
                    (
                        "Merge aborted".to_string(),
                        format!("Files restored to {}.", self.ours_ref),
                    )
                };
                return root.child(
                    div()
                        .m(px(14.))
                        .p(px(14.))
                        .rounded(px(10.))
                        .border_1()
                        .border_color(c(t, ColorRole::BorderDefault))
                        .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
                        .child(muted(t, sub)),
                );
            }
            _ => {}
        }

        // per-segment line numbers: ours, base, theirs, result
        let mut nums = Vec::new();
        let (mut a, mut b, mut cc, mut r) = (1, 1, 1, 1);
        let mut k = 0;
        for s in &file.segs {
            nums.push((a, b, cc, r));
            match s {
                Seg::Same(l) => {
                    a += l.len();
                    b += l.len();
                    cc += l.len();
                    r += l.len();
                }
                Seg::Conflict(x) => {
                    a += x.ours.len();
                    b += x.base.as_ref().map_or(0, Vec::len);
                    cc += x.theirs.len();
                    r += match m.picks[f].get(&k) {
                        Some(p) => pick_lines(x, p).len(),
                        None => marker_block(x, "", "").len(),
                    };
                    k += 1;
                }
            }
        }

        if self.layout != "inline" {
            let mut head = div().flex().w_full().bg(c(t, ColorRole::SurfaceSunken));
            for s in &sides {
                head = head.child(
                    div()
                        .flex_1()
                        .min_w(px(0.))
                        .flex()
                        .items_center()
                        .gap(px(7.))
                        .px(px(12.))
                        .py(px(6.))
                        .text_size(px(11.5))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(c(t, ColorRole::FgMuted))
                        .child(div().size(px(8.)).rounded(px(2.)).bg(side_color(t, s)))
                        .child(pane_name(s)),
                );
            }
            root = root.child(head);
            let mut ci = 0;
            for (si, seg) in file.segs.iter().enumerate() {
                let (na, nb, nc, nr) = nums[si];
                let mut row = div()
                    .flex()
                    .w_full()
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle));
                let this_c = if matches!(seg, Seg::Conflict(_)) {
                    Some(ci)
                } else {
                    None
                };
                for s in &sides {
                    let mut cell = div().flex_1().min_w(px(0.)).flex().flex_col().py(px(2.));
                    match seg {
                        Seg::Same(l) => {
                            let start = match *s {
                                "ours" => na,
                                "base" => nb,
                                "theirs" => nc,
                                _ => nr,
                            };
                            for (i, x) in l.iter().enumerate() {
                                cell = cell.child(code_line(t, Some(start + i), x, false));
                            }
                        }
                        Seg::Conflict(x) => {
                            let n = this_c.unwrap_or(0);
                            if *s == "result" {
                                cell = cell.child(self.block(n, nr));
                            } else {
                                let (lines, start) = match *s {
                                    "ours" => (x.ours.clone(), na),
                                    "base" => (x.base.clone().unwrap_or_default(), nb),
                                    _ => (x.theirs.clone(), nc),
                                };
                                let col = side_color(t, s);
                                let resolved = m.is_resolved(f, n);
                                cell = cell
                                    .bg(alpha(col, if resolved { 0.0 } else { 0.09 }))
                                    .border_l_2()
                                    .border_color(alpha(col, if n == m.focus { 1.0 } else { 0.5 }));
                                for (i, l) in lines.iter().enumerate() {
                                    cell = cell.child(code_line(t, Some(start + i), l, false));
                                }
                            }
                        }
                    }
                    row = row.child(cell);
                }
                if this_c.is_some() {
                    ci += 1;
                }
                root = root.child(row);
            }
        }

        if self.layout != "result-center" {
            let mut res = div().flex().flex_col().pb(px(4.)).child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .px(px(12.))
                    .py(px(6.))
                    .bg(c(t, ColorRole::SurfaceSunken))
                    .text_size(px(11.5))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(c(t, ColorRole::FgMuted))
                    .child(
                        div()
                            .size(px(8.))
                            .rounded(px(2.))
                            .bg(c(t, ColorRole::FgDefault)),
                    )
                    .child("Result")
                    .child(div().flex_1())
                    .child({
                        let mk = m.markers_left();
                        div()
                            .text_color(if mk == 0 {
                                c(t, ColorRole::MergeResolved)
                            } else {
                                c(t, ColorRole::StatusDangerFg)
                            })
                            .child(if mk == 0 {
                                "No markers left".to_string()
                            } else {
                                format!("⚑ {mk} markers left")
                            })
                    }),
            );
            let mut ci = 0;
            for (si, seg) in file.segs.iter().enumerate() {
                match seg {
                    Seg::Same(l) => {
                        for (i, x) in l.iter().enumerate() {
                            res = res.child(code_line(t, Some(nums[si].3 + i), x, false));
                        }
                    }
                    Seg::Conflict(_) => {
                        res = res.child(self.block(ci, nums[si].3));
                        ci += 1;
                    }
                }
            }
            root = root.child(res);
        }

        // footer
        let mut ft = div()
            .flex()
            .flex_wrap()
            .items_center()
            .gap(px(8.))
            .px(px(12.))
            .py(px(9.))
            .border_t_1()
            .border_color(c(t, ColorRole::BorderSubtle));
        if m.phase == Phase::ConfirmAbort {
            ft = ft
                .child(div().flex_1().child(format!(
                    "Abort the merge? {done_all} resolutions will be lost."
                )))
                .child(self.btn("mrg-cancel", "Keep resolving", Weight::Quiet, 0, "cancel"))
                .child(self.btn("mrg-confirm", "Abort merge", Weight::Danger, 0, "confirm"));
        } else {
            ft = ft
                .child(self.btn("mrg-abort", "Abort", Weight::Quiet, 0, "abort"))
                .child(div().flex_1());
            if let Some(why) = m.mark_blocker() {
                ft = ft.child(muted(t, why));
            }
            if !m.marked.contains(&f) {
                ft = ft.child(self.btn("mrg-mark", "✓ Mark resolved", Weight::Primary, f, "mark"));
            }
            if m.files.len() > 1 {
                if let Some(why) = m.complete_blocker() {
                    ft = ft.child(muted(t, why));
                }
                ft = ft.child(self.btn(
                    "mrg-complete",
                    "Complete merge",
                    Weight::Primary,
                    0,
                    "complete",
                ));
            }
        }
        root.child(ft)
    }
}

/// `conflict-radar`: files × agents with edit bars, clash lines and the ways to settle.
#[derive(IntoElement)]
pub struct ConflictRadarEl {
    radar: Radar,
    title: SharedString,
    heat: bool,
    order: bool,
    expanded: Option<String>,
    on_resolve: Option<RadarAct>,
    on_merge: Option<Act>,
    theme: &'static Theme,
}

const MARKS: [ColorRole; 8] = [
    ColorRole::AgentBlue,
    ColorRole::AgentViolet,
    ColorRole::AgentCyan,
    ColorRole::AgentPink,
    ColorRole::AgentGreen,
    ColorRole::AgentOrange,
    ColorRole::AgentYellow,
    ColorRole::AgentRed,
];

impl ConflictRadarEl {
    /// A radar over `radar`.
    pub fn new(radar: Radar) -> Self {
        Self {
            radar,
            title: "Parallel work".into(),
            heat: false,
            order: false,
            expanded: None,
            on_resolve: None,
            on_merge: None,
            theme: &Theme::DARK,
        }
    }
    /// Heading.
    pub fn title(mut self, s: impl Into<SharedString>) -> Self {
        self.title = s.into();
        self
    }
    /// Tint conflicting cells by overlap.
    pub fn heat(mut self, h: bool) -> Self {
        self.heat = h;
        self
    }
    /// Show the suggested merge order.
    pub fn order(mut self, o: bool) -> Self {
        self.order = o;
        self
    }
    /// The file whose ways are open.
    pub fn expanded(mut self, path: Option<String>) -> Self {
        self.expanded = path;
        self
    }
    /// A way was picked.
    pub fn on_resolve(mut self, f: RadarAct) -> Self {
        self.on_resolve = Some(f);
        self
    }
    /// Merge pressed (`"merge"`).
    pub fn on_merge(mut self, f: Act) -> Self {
        self.on_merge = Some(f);
        self
    }
    /// Theme.
    pub fn theme(mut self, t: &'static Theme) -> Self {
        self.theme = t;
        self
    }
}

impl RenderOnce for ConflictRadarEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let r = &self.radar;
        let mark = |id: &str| -> Hsla {
            let i = r.agents.iter().position(|a| a.id == id).unwrap_or(0);
            c(t, MARKS[i % MARKS.len()])
        };
        let max_line = r
            .files
            .iter()
            .flat_map(|f| f.edits.iter().map(|e| e.to))
            .max()
            .unwrap_or(1)
            .max(1) as f32;
        let blocker = r.merge_blocker();
        let conflicts = r.conflicts().len();
        let open = r.open().len();
        let mut root = div()
            .flex()
            .flex_col()
            .w_full()
            .rounded(px(12.))
            .border_1()
            .border_color(c(t, ColorRole::BorderDefault))
            .bg(c(t, ColorRole::SurfaceRaised))
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .overflow_hidden()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap(px(10.))
                    .px(px(14.))
                    .py(px(10.))
                    .border_b_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(self.title.clone()),
                    )
                    .child(muted(
                        t,
                        format!(
                            "{} files · {conflicts} conflicts · {} settled",
                            r.files.len(),
                            conflicts - open
                        ),
                    ))
                    .child(div().flex_1())
                    .when_some(blocker.clone(), |d, b| d.child(muted(t, b)))
                    .child(button(
                        t,
                        "crd-merge",
                        "Merge",
                        if blocker.is_none() {
                            Weight::Primary
                        } else {
                            Weight::Plain
                        },
                        "merge",
                        if blocker.is_none() {
                            self.on_merge.clone()
                        } else {
                            None
                        },
                    )),
            );
        let mut head = div()
            .flex()
            .w_full()
            .bg(c(t, ColorRole::SurfaceSunken))
            .child(
                div()
                    .w(relative(0.32))
                    .flex_none()
                    .px(px(14.))
                    .py(px(8.))
                    .text_size(px(11.5))
                    .text_color(c(t, ColorRole::FgMuted))
                    .child("File"),
            );
        for a in &r.agents {
            head = head.child(
                div()
                    .flex_1()
                    .min_w(px(0.))
                    .flex()
                    .items_center()
                    .gap(px(7.))
                    .px(px(8.))
                    .py(px(7.))
                    .child(
                        div()
                            .size(px(20.))
                            .rounded(px(6.))
                            .bg(mark(&a.id))
                            .text_color(c(t, ColorRole::BgBase))
                            .text_size(px(10.5))
                            .font_weight(FontWeight::BOLD)
                            .flex()
                            .items_center()
                            .justify_center()
                            .child(a.monogram()),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .min_w(px(0.))
                            .child(
                                div()
                                    .text_size(px(12.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(a.name.clone()),
                            )
                            .child(muted(
                                t,
                                if a.running {
                                    format!("{} · working", a.model)
                                } else {
                                    a.model.clone()
                                },
                            )),
                    ),
            );
        }
        root = root.child(head);
        for f in &r.files {
            let conflicting = f.conflicting();
            let clash = f.clash_agents();
            let pick = r.resolved.get(&f.path).cloned();
            let mut row = div()
                .flex()
                .w_full()
                .min_h(px(38.))
                .border_t_1()
                .border_color(c(t, ColorRole::BorderSubtle))
                .child(
                    div()
                        .w(relative(0.32))
                        .flex_none()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(6.))
                        .px(px(14.))
                        .py(px(8.))
                        .child(
                            div()
                                .font_family("monospace")
                                .text_size(px(12.))
                                .child(f.path.clone()),
                        )
                        .when(conflicting, |d| {
                            let (label, role) = match &pick {
                                Some(p) => (format!("✓ {p}"), ColorRole::MergeResolved),
                                None => ("Conflict".to_string(), ColorRole::MergeConflict),
                            };
                            d.child(tag(t, label, Some(role)))
                        }),
                );
            for a in &r.agents {
                let mut cell = div().flex_1().min_w(px(0.)).relative().h(px(38.));
                if let Some(e) = f.edit(&a.id) {
                    let kept = r.kept(&f.path, &a.id);
                    let in_clash = clash.contains(&a.id);
                    if self.heat && in_clash && pick.is_none() {
                        cell = cell.bg(alpha(
                            c(t, ColorRole::MergeConflict),
                            (f.overlap_lines() as f32 / 20.0).clamp(0.15, 1.0) * 0.22,
                        ));
                    }
                    let left = (e.from - 1) as f32 / max_line;
                    let w = (e.len() as f32 / max_line).max(0.03);
                    cell = cell.child(
                        div()
                            .absolute()
                            .top(px(15.))
                            .left(relative(left * 0.9 + 0.05))
                            .w(relative(w * 0.9))
                            .h(px(8.))
                            .rounded_full()
                            .bg(alpha(
                                mark(&a.id),
                                if kept == Some(false) { 0.28 } else { 1.0 },
                            ))
                            .when(in_clash && pick.is_none(), |d| {
                                d.border_1().border_color(c(t, ColorRole::MergeConflict))
                            }),
                    );
                }
                row = row.child(cell);
            }
            root = root.child(row);
            if conflicting && self.expanded.as_deref() == Some(f.path.as_str()) {
                let mut ways = div()
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .px(px(14.))
                    .py(px(8.))
                    .bg(c(t, ColorRole::SurfaceSunken));
                for (i, id) in clash.iter().enumerate() {
                    let Some(a) = r.agents.iter().find(|a| &a.id == id) else {
                        continue;
                    };
                    let checked = pick.as_deref() == Some(id.as_str());
                    let summary = f
                        .edit(id)
                        .and_then(|e| e.summary.clone())
                        .unwrap_or_else(|| "Keep this agent's version".to_string());
                    let cb = self.on_resolve.clone();
                    let path = f.path.clone();
                    let aid = id.clone();
                    ways = ways.child(
                        div()
                            .id(("crd-way", i))
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .px(px(9.))
                            .py(px(7.))
                            .rounded(px(8.))
                            .border_1()
                            .border_color(if checked {
                                c(t, ColorRole::FgDefault)
                            } else {
                                c(t, ColorRole::BorderDefault)
                            })
                            .bg(c(t, ColorRole::SurfaceRaised))
                            .cursor_pointer()
                            .child(
                                div()
                                    .size(px(14.))
                                    .rounded_full()
                                    .border_color(c(t, ColorRole::FgDefault))
                                    .when(checked, |d| d.border_4())
                                    .when(!checked, |d| d.border_1()),
                            )
                            .child(div().size(px(10.)).rounded(px(3.)).bg(mark(id)))
                            .child(
                                div()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .text_size(px(12.))
                                    .child(a.name.clone()),
                            )
                            .child(muted(t, summary))
                            .when_some(cb, move |d, cb| {
                                d.on_click(move |_, win, cx| cb(path.clone(), aid.clone(), win, cx))
                            }),
                    );
                }
                root = root.child(ways);
            }
        }
        if self.order {
            let names: Vec<String> = r
                .merge_order()
                .iter()
                .filter_map(|id| {
                    r.agents
                        .iter()
                        .find(|a| &a.id == id)
                        .map(|a| a.name.clone())
                })
                .collect();
            root = root.child(
                div()
                    .px(px(14.))
                    .py(px(8.))
                    .border_t_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .text_size(px(12.))
                    .child(format!("Suggested merge order: {}", names.join(" → "))),
            );
        }
        root
    }
}
