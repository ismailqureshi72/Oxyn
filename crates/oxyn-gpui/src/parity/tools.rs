//! Tool calls for GPUI (plan #28, chunk 3).

use gpui::{
    App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window, div,
    prelude::*, px,
};
use oxyn_core::{
    ClockMs,
    parity::tools::{
        CallCard, CallStatus, JsonTree, ToolKindMap, Unfurl, UnfurlState, group_label, ms_label,
    },
    verbs::VerbPack,
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, Weight, button, c, card, muted, tag};

fn status(t: &'static Theme, s: CallStatus) -> impl IntoElement {
    let (txt, r) = match s {
        CallStatus::Running => ("◌", ColorRole::FgMuted),
        CallStatus::Success => ("✓", ColorRole::StatusSuccessFg),
        CallStatus::Error => ("Failed", ColorRole::StatusDangerFg),
        CallStatus::Approval => ("Approval", ColorRole::StatusWarningFg),
        CallStatus::Denied => ("Denied", ColorRole::StatusDangerFg),
    };
    div().text_size(px(11.5)).text_color(c(t, r)).child(txt)
}

fn code(t: &'static Theme, s: String) -> impl IntoElement {
    div()
        .p(px(8.))
        .rounded(px(8.))
        .bg(c(t, ColorRole::SurfaceSunken))
        .text_size(px(11.5))
        .max_h(px(220.))
        .overflow_hidden()
        .children(
            s.lines()
                .map(|l| div().child(SharedString::from(l.to_string())))
                .collect::<Vec<_>>(),
        )
}

/// `tool-call-card` and its group: header line, open body with arguments and result or error.
/// Reports `toggle` and `retry`.
#[derive(IntoElement)]
pub struct ToolCallCardEl {
    cards: Vec<CallCard>,
    now: ClockMs,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ToolCallCardEl {
    /// One card (a single-item vec) or a group (several, with the "Used N tools" header).
    pub fn new(cards: Vec<CallCard>, now: ClockMs) -> Self {
        Self {
            cards,
            now,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Button reports.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for ToolCallCardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let map = ToolKindMap::standard();
        let act = self.act;
        let now = self.now;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .w_full()
            .text_color(c(t, ColorRole::FgDefault))
            .text_size(px(13.))
            .when(self.cards.len() > 1, |d| {
                d.child(muted(t, group_label(&self.cards)))
            })
            .children(self.cards.iter().enumerate().map(|(i, k)| {
                card(t)
                    .p(px(8.))
                    .gap(px(6.))
                    .child(
                        div()
                            .flex()
                            .items_center()
                            .gap(px(8.))
                            .child(tag(t, map.kind(&k.name).icon(), None))
                            .child(div().text_size(px(12.)).child(k.name.clone()))
                            .child(muted(t, k.summary.clone()))
                            .child(div().flex_1())
                            .child(status(t, k.status))
                            .when_some(k.time(now), |d, x| d.child(muted(t, x))),
                    )
                    .when(k.open, |d| {
                        d.when_some(k.args.clone(), |d, a| {
                            d.child(muted(t, "ARGUMENTS")).child(code(t, a))
                        })
                        .when_some(k.error.clone(), |d, e| {
                            d.child(
                                div()
                                    .flex()
                                    .gap(px(8.))
                                    .items_center()
                                    .child(
                                        div()
                                            .flex_1()
                                            .text_color(c(t, ColorRole::StatusDangerFg))
                                            .child(e),
                                    )
                                    .child(button(
                                        t,
                                        ("tc-retry", i),
                                        "Retry",
                                        Weight::Plain,
                                        "retry",
                                        act.clone(),
                                    )),
                            )
                        })
                        .when_some(k.result.clone().filter(|_| k.error.is_none()), |d, r| {
                            d.child(muted(t, "RESULT")).child(code(t, r))
                        })
                    })
            }))
    }
}

/// `tool-call` row: kind mark, phrase, status, Allow / Deny in approval. Reports `allow`,
/// `deny`, `retry`.
#[derive(IntoElement)]
pub struct ToolCallEl {
    tool: SharedString,
    title: SharedString,
    status: CallStatus,
    duration_ms: Option<u64>,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ToolCallEl {
    /// A row.
    pub fn new(
        tool: impl Into<SharedString>,
        title: impl Into<SharedString>,
        status: CallStatus,
    ) -> Self {
        Self {
            tool: tool.into(),
            title: title.into(),
            status,
            duration_ms: None,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Duration.
    pub fn duration(mut self, ms: u64) -> Self {
        self.duration_ms = Some(ms);
        self
    }
    /// Button reports.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for ToolCallEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let info = ToolKindMap::standard().info(&self.tool, VerbPack::neutral());
        let title: SharedString = if self.status == CallStatus::Running {
            format!("{}…", info.present).into()
        } else {
            self.title
        };
        let act = self.act;
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .text_size(px(13.))
            .text_color(c(t, ColorRole::FgDefault))
            .child(
                div()
                    .flex()
                    .gap(px(8.))
                    .items_center()
                    .child(tag(t, info.icon, None))
                    .when_some(info.server, |d, s| d.child(muted(t, s)))
                    .child(title)
                    .child(div().flex_1())
                    .when_some(self.duration_ms, |d, m| d.child(muted(t, ms_label(m))))
                    .child(status(t, self.status)),
            )
            .when(self.status == CallStatus::Approval, |d| {
                d.child(
                    div()
                        .flex()
                        .gap(px(8.))
                        .justify_end()
                        .child(button(
                            t,
                            "tr-deny",
                            "Deny",
                            Weight::Plain,
                            "deny",
                            act.clone(),
                        ))
                        .child(button(
                            t,
                            "tr-allow",
                            "Allow",
                            Weight::Primary,
                            "allow",
                            act.clone(),
                        )),
                )
            })
            .when(self.status == CallStatus::Error, |d| {
                d.child(div().flex().justify_end().child(button(
                    t,
                    "tr-retry",
                    "Retry",
                    Weight::Plain,
                    "retry",
                    act.clone(),
                )))
            })
    }
}

/// `tool-chips`: header count, one row per step with its chip, then diff chips with +/−.
#[derive(IntoElement)]
pub struct ToolChipsEl {
    steps: Vec<(SharedString, SharedString, CallStatus)>,
    diffs: Vec<(SharedString, usize, usize)>,
    theme: &'static Theme,
}

impl ToolChipsEl {
    /// Steps `(label, chip, status)` and diffs `(file, add, del)`.
    pub fn new(
        steps: Vec<(SharedString, SharedString, CallStatus)>,
        diffs: Vec<(SharedString, usize, usize)>,
    ) -> Self {
        Self {
            steps,
            diffs,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for ToolChipsEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .flex_col()
            .gap(px(4.))
            .text_size(px(12.5))
            .text_color(c(t, ColorRole::FgMuted))
            .child(format!("{} tool calls", self.steps.len()))
            .children(self.steps.into_iter().map(|(l, ch, s)| {
                div()
                    .flex()
                    .gap(px(6.))
                    .items_center()
                    .child(l)
                    .child(tag(t, ch, Some(ColorRole::FgDefault)))
                    .child(status(t, s))
            }))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap(px(6.))
                    .children(self.diffs.into_iter().map(|(f, a, d)| {
                        div()
                            .flex()
                            .gap(px(6.))
                            .px(px(9.))
                            .h(px(26.))
                            .items_center()
                            .rounded(px(7.))
                            .border_1()
                            .border_color(c(t, ColorRole::BorderDefault))
                            .child(div().text_color(c(t, ColorRole::FgDefault)).child(f))
                            .child(
                                div()
                                    .text_color(c(t, ColorRole::StatusSuccessFg))
                                    .child(format!("+{a}")),
                            )
                            .child(
                                div()
                                    .text_color(c(t, ColorRole::StatusDangerFg))
                                    .child(format!("−{d}")),
                            )
                    })),
            )
    }
}

/// `json-viewer`: visible rows with depth, key, preview; current row highlighted; path bar.
#[derive(IntoElement)]
pub struct JsonViewerEl {
    tree: JsonTree,
    theme: &'static Theme,
}

impl JsonViewerEl {
    /// The tree.
    pub fn new(tree: JsonTree) -> Self {
        Self {
            tree,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for JsonViewerEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let rows = self.tree.rows();
        let cur = self.tree.current.min(rows.len().saturating_sub(1));
        let path = rows.get(cur).map(|r| r.path.clone()).unwrap_or_default();
        card(t)
            .p(px(0.))
            .gap(px(0.))
            .text_size(px(12.))
            .children(rows.into_iter().enumerate().map(|(i, r)| {
                div()
                    .flex()
                    .gap(px(6.))
                    .pl(px(10. + r.depth as f32 * 16.))
                    .py(px(2.))
                    .when(i == cur, |d| d.bg(c(t, ColorRole::SurfaceSelected)))
                    .when(r.hit, |d| d.bg(c(t, ColorRole::Highlight)))
                    .child(if r.branch {
                        if r.open { "▾" } else { "▸" }
                    } else {
                        " "
                    })
                    .child(
                        div()
                            .text_color(c(t, ColorRole::FgDefault))
                            .child(format!("{}:", r.key)),
                    )
                    .child(div().text_color(c(t, ColorRole::FgMuted)).child(r.preview))
            }))
            .child(
                div()
                    .px(px(10.))
                    .py(px(6.))
                    .border_t_1()
                    .border_color(c(t, ColorRole::BorderSubtle))
                    .child(muted(t, path)),
            )
    }
}

/// `expandable-card`: header with title and description; details below when open. Reports
/// `toggle`.
#[derive(IntoElement)]
pub struct ExpandableCardEl {
    title: SharedString,
    description: SharedString,
    details: Vec<SharedString>,
    open: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ExpandableCardEl {
    /// A card.
    pub fn new(
        title: impl Into<SharedString>,
        description: impl Into<SharedString>,
        open: bool,
    ) -> Self {
        Self {
            title: title.into(),
            description: description.into(),
            details: vec![],
            open,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Detail lines.
    pub fn details(mut self, d: Vec<SharedString>) -> Self {
        self.details = d;
        self
    }
    /// Button reports.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for ExpandableCardEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let act = self.act;
        card(t)
            .child(
                div()
                    .id("xc-head")
                    .flex()
                    .gap(px(8.))
                    .cursor_pointer()
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .flex_1()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(self.title))
                            .child(muted(t, self.description)),
                    )
                    .child(if self.open { "▴" } else { "▾" })
                    .when_some(act, |d, f| d.on_click(move |_, w, cx| f("toggle", w, cx))),
            )
            .when(self.open, |d| {
                d.children(self.details.into_iter().map(|x| muted(t, x)))
            })
    }
}

/// `link-unfurl`: the links of a message as shimmer, inline title or a preview card.
#[derive(IntoElement)]
pub struct LinkUnfurlEl {
    links: Vec<Unfurl>,
    theme: &'static Theme,
}

impl LinkUnfurlEl {
    /// The links.
    pub fn new(links: Vec<Unfurl>) -> Self {
        Self {
            links,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for LinkUnfurlEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        div()
            .flex()
            .flex_col()
            .gap(px(6.))
            .text_size(px(12.5))
            .children(self.links.into_iter().map(|l| {
                let host = l.host().to_string();
                match l.state {
                    UnfurlState::Loading => div()
                        .text_color(c(t, ColorRole::FgMuted))
                        .child(format!("{host} …")),
                    UnfurlState::Plain => div().text_color(c(t, ColorRole::Link)).child(l.url),
                    UnfurlState::Inline(p) => {
                        div().text_color(c(t, ColorRole::Link)).child(p.title)
                    }
                    UnfurlState::Card(p) => card(t)
                        .max_w(px(420.))
                        .child(muted(t, p.site))
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(c(t, ColorRole::FgDefault))
                                .child(p.title),
                        )
                        .child(muted(t, p.description)),
                }
            }))
    }
}
