//! Home and run surfaces for GPUI (plan #28, chunk 6), drawn from `oxyn_core::parity::home`:
//! thread list, activity heat map and home, token usage bars, attachment strip, run modes,
//! experiment grid, document grid and polling cost. Elements are `RenderOnce`; hover, menus and
//! the upload clock are driven by the host view, which re-renders with new state.

use gpui::{
    App, FontWeight, Hsla, IntoElement, ParentElement, RenderOnce, SharedString, Styled, Window,
    div, prelude::*, px, relative,
};
use oxyn_core::{
    ClockMs,
    parity::home::{
        Doc, DocSort, Experiment, Outcome, RunMode, Thread, ThreadState, UsageDay, budget_line,
        compact, delta, doc_view, experiment_counts, experiment_grid, group_threads, heat_recent,
        heat_year, hm, initials, poll_cost, tile_name, tile_upload, usage_totals,
    },
};
use oxyn_tokens::{ColorRole, Theme};

use super::{Act, alpha, c, card, muted};

fn level(t: &'static Theme, l: u8) -> Hsla {
    alpha(
        c(t, ColorRole::FgDefault),
        [0.07, 0.22, 0.4, 0.64, 0.9][l.min(4) as usize],
    )
}

fn chip_role(s: ThreadState) -> ColorRole {
    match s {
        ThreadState::Approval => ColorRole::StatusWarningFg,
        ThreadState::Input => ColorRole::StatusInfoFg,
        ThreadState::Done => ColorRole::StatusSuccessFg,
        _ => ColorRole::FgMuted,
    }
}

fn shape(t: &'static Theme, s: &str, size: f32, col: Hsla) -> impl IntoElement {
    let d = div().flex_none().size(px(size));
    match s {
        "circle" => d.rounded_full().bg(col),
        "square-o" => d.rounded(px(2.)).border_1().border_color(col),
        "triangle" | "diamond" => d.rounded(px(1.)).bg(col).opacity(0.85),
        _ => d.rounded(px(2.)).bg(col),
    }
    .when(s == "triangle", |d| d.bg(c(t, ColorRole::StatusDangerFg)))
}

/// `thread-list`: threads grouped by state (needs-you first, Working, Settled).
#[derive(IntoElement)]
pub struct ThreadListEl {
    threads: Vec<Thread>,
    selected: Option<String>,
    settled_open: bool,
    act: Option<Act>,
    theme: &'static Theme,
}

impl ThreadListEl {
    /// The list.
    pub fn new(threads: Vec<Thread>) -> Self {
        Self {
            threads,
            selected: None,
            settled_open: false,
            act: None,
            theme: &Theme::DARK,
        }
    }
    /// Open thread.
    pub fn selected(mut self, id: Option<String>) -> Self {
        self.selected = id;
        self
    }
    /// Settled group open.
    pub fn settled_open(mut self, on: bool) -> Self {
        self.settled_open = on;
        self
    }
    /// Reports `open` (the host reads its own cursor) and `toggle-settled`.
    pub fn on_act(mut self, f: Act) -> Self {
        self.act = Some(f);
        self
    }
}

impl RenderOnce for ThreadListEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let groups = group_threads(&self.threads, "");
        let mut col = div().flex().flex_col().gap(px(2.)).w(px(280.));
        for g in groups {
            if !g.label.is_empty() {
                let act = self.act.clone();
                col = col.child(
                    div()
                        .id(SharedString::from(format!("g-{}", g.id)))
                        .flex()
                        .items_center()
                        .h(px(28.))
                        .px(px(8.))
                        .mt(px(8.))
                        .text_size(px(12.5))
                        .text_color(c(t, ColorRole::FgMuted))
                        .cursor_pointer()
                        .child(g.label)
                        .child(div().ml_auto().child(format!("{}", g.rows.len())))
                        .when_some(act, |d, f| {
                            d.on_click(move |_, w, cx| f("toggle-settled", w, cx))
                        }),
                );
                if g.id == "settled" && !self.settled_open {
                    continue;
                }
            }
            for r in g.rows {
                let th = &self.threads[r];
                let on = self.selected.as_deref() == Some(th.id.as_str());
                let right: SharedString = th.meta().into();
                col = col.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap(px(3.))
                        .px(px(8.))
                        .py(px(7.))
                        .rounded(px(9.))
                        .when(on, |d| d.bg(c(t, ColorRole::SurfaceSunken)))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap(px(6.))
                                .child(
                                    div()
                                        .size(px(15.))
                                        .rounded(px(4.))
                                        .border_1()
                                        .border_color(c(t, ColorRole::BorderDefault))
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .text_size(px(7.5))
                                        .text_color(c(t, ColorRole::FgMuted))
                                        .child(initials(&th.project)),
                                )
                                .child(muted(t, th.project.clone()))
                                .child(
                                    div()
                                        .ml_auto()
                                        .text_size(px(12.))
                                        .text_color(c(t, chip_role(th.state)))
                                        .child(right),
                                ),
                        )
                        .child(
                            div()
                                .text_size(px(13.5))
                                .text_color(c(t, ColorRole::FgDefault))
                                .when(
                                    matches!(
                                        th.state,
                                        ThreadState::Approval
                                            | ThreadState::Input
                                            | ThreadState::Done
                                    ),
                                    |d| d.font_weight(FontWeight::MEDIUM),
                                )
                                .child(th.title.clone()),
                        ),
                );
            }
        }
        col
    }
}

/// `activity-heatmap`: a year (or the last `weeks`) of activity in five levels.
#[derive(IntoElement)]
pub struct ActivityHeatmapEl {
    year: i32,
    counts: Vec<u32>,
    weeks: Option<u32>,
    cell: f32,
    title: SharedString,
    theme: &'static Theme,
}

impl ActivityHeatmapEl {
    /// A year.
    pub fn new(year: i32, counts: Vec<u32>) -> Self {
        Self {
            year,
            counts,
            weeks: None,
            cell: 10.,
            title: "".into(),
            theme: &Theme::DARK,
        }
    }
    /// The compact strip of the last `w` weeks.
    pub fn weeks(mut self, w: u32) -> Self {
        self.weeks = Some(w);
        self
    }
    /// Title.
    pub fn title(mut self, s: impl Into<SharedString>) -> Self {
        self.title = s.into();
        self
    }
}

impl RenderOnce for ActivityHeatmapEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let cell = self.cell;
        let cells: Vec<(u32, u32, u8)> = match self.weeks {
            Some(w) => heat_recent(&self.counts, w),
            None => heat_year(self.year, &self.counts)
                .into_iter()
                .map(|h| (h.col, h.row, h.level))
                .collect(),
        };
        let ncols = cells.iter().map(|x| x.0).max().map_or(0, |m| m + 1);
        let mut grid = div().flex().gap(px(3.));
        for colx in 0..ncols {
            let mut colv = div().flex().flex_col().gap(px(3.));
            for r in 0..7 {
                let l = cells.iter().find(|x| x.0 == colx && x.1 == r).map(|x| x.2);
                colv = colv.child(
                    div()
                        .size(px(cell))
                        .rounded(px(2.5))
                        .when_some(l, |d, l| d.bg(level(t, l))),
                );
            }
            grid = grid.child(colv);
        }
        let total: u32 = self.counts.iter().sum();
        div()
            .flex()
            .flex_col()
            .gap(px(8.))
            .when(!self.title.is_empty(), |d| {
                d.child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(c(t, ColorRole::FgDefault))
                        .child(self.title.clone()),
                )
                .child(muted(t, format!("{total} in {}", self.year)))
            })
            .child(grid)
    }
}

/// `activity-home`: the overview cards (activity, usage, this month) under a composer slot.
#[derive(IntoElement)]
pub struct ActivityHomeEl {
    counts: Vec<u32>,
    credits: (u32, u32),
    month: [(SharedString, i64, i64); 2],
    theme: &'static Theme,
}

impl ActivityHomeEl {
    /// The overview.
    pub fn new(counts: Vec<u32>, credits: (u32, u32)) -> Self {
        Self {
            counts,
            credits,
            month: [("Chats".into(), 36, 12), ("Files made".into(), 14, -3)],
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for ActivityHomeEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let (u, of) = self.credits;
        let f = (u as f32 / of.max(1) as f32).min(1.0);
        div()
            .flex()
            .flex_wrap()
            .gap(px(10.))
            .child(
                card(t)
                    .child("Activity")
                    .child(ActivityHeatmapEl::new(2026, self.counts.clone()).weeks(24)),
            )
            .child(
                card(t)
                    .w(px(200.))
                    .child("Usage")
                    .child(
                        div()
                            .text_size(px(22.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("{u}")),
                    )
                    .child(muted(t, format!("of {of} credits")))
                    .child(
                        div()
                            .h(px(5.))
                            .rounded(px(999.))
                            .bg(c(t, ColorRole::SurfaceSunken))
                            .child(div().h_full().w(relative(f)).rounded(px(999.)).bg(c(
                                t,
                                if f >= 0.9 {
                                    ColorRole::StatusWarningFg
                                } else {
                                    ColorRole::FgDefault
                                },
                            ))),
                    ),
            )
            .child(
                card(t)
                    .w(px(200.))
                    .child("This month")
                    .children(self.month.into_iter().map(move |(l, v, d)| {
                        div()
                            .flex()
                            .items_end()
                            .gap(px(6.))
                            .child(muted(t, l))
                            .child(div().text_size(px(18.)).child(format!("{v}")))
                            .child(
                                div()
                                    .text_size(px(11.5))
                                    .text_color(c(
                                        t,
                                        if d > 0 {
                                            ColorRole::StatusSuccessFg
                                        } else {
                                            ColorRole::StatusDangerFg
                                        },
                                    ))
                                    .child(delta(d)),
                            )
                    })),
            )
    }
}

/// `token-usage-chart`: daily tokens as bars with totals; `hover` marks one day.
#[derive(IntoElement)]
pub struct TokenUsageChartEl {
    days: Vec<UsageDay>,
    per_m_cents: f64,
    hover: Option<usize>,
    theme: &'static Theme,
}

impl TokenUsageChartEl {
    /// The chart.
    pub fn new(days: Vec<UsageDay>, per_m_cents: f64) -> Self {
        Self {
            days,
            per_m_cents,
            hover: None,
            theme: &Theme::DARK,
        }
    }
    /// The day under the pointer.
    pub fn hover(mut self, i: Option<usize>) -> Self {
        self.hover = i;
        self
    }
}

impl RenderOnce for TokenUsageChartEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let tot = usage_totals(&self.days, self.per_m_cents);
        let max = self.days.iter().map(|d| d.tokens).fold(1.0, f64::max);
        let hv = self.hover;
        let bars = div().flex().items_end().gap(px(1.)).h(px(96.)).children(
            self.days.iter().enumerate().map(move |(i, d)| {
                div()
                    .flex_1()
                    .h(relative((d.tokens / max) as f32))
                    .bg(alpha(
                        c(t, ColorRole::FgDefault),
                        if hv == Some(i) { 0.9 } else { 0.3 },
                    ))
            }),
        );
        let mut tip = None;
        if let Some(i) = hv.filter(|&i| i < self.days.len()) {
            let d = self.days[i];
            tip = Some(muted(
                t,
                format!(
                    "Tokens {} · Cached {}",
                    compact(d.tokens),
                    compact(d.cached)
                ),
            ));
        }
        card(t)
            .w(px(380.))
            .child("Token usage")
            .child(bars)
            .children(tip)
            .child(
                div()
                    .flex()
                    .gap(px(16.))
                    .child(muted(t, format!("Total {}", tot.total)))
                    .child(muted(t, format!("Avg {}", tot.avg)))
                    .child(muted(t, format!("Est. {}", tot.cost))),
            )
    }
}

/// `attachment-strip`: 56 px tiles; the uploading tile shows its percent until 100.
#[derive(IntoElement)]
pub struct AttachmentStripEl {
    files: Vec<SharedString>,
    weights: Vec<f32>,
    added: ClockMs,
    now: ClockMs,
    theme: &'static Theme,
}

impl AttachmentStripEl {
    /// The strip at `now`, for a batch added at `added`.
    pub fn new(files: Vec<SharedString>, added: ClockMs, now: ClockMs) -> Self {
        Self {
            weights: vec![1.0; files.len()],
            files,
            added,
            now,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for AttachmentStripEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let ws = self.weights.clone();
        let (added, now) = (self.added, self.now);
        div()
            .flex()
            .gap(px(8.))
            .children(self.files.iter().enumerate().filter_map(move |(i, f)| {
                let s = tile_upload(&ws, i, added, now, 1400, 320);
                (!s.queued).then(|| {
                    div()
                        .relative()
                        .size(px(56.))
                        .rounded(px(12.))
                        .border_2()
                        .border_color(alpha(
                            c(t, ColorRole::FgDefault),
                            if s.progress < 1.0 {
                                0.3 + 0.7 * s.progress
                            } else {
                                0.12
                            },
                        ))
                        .bg(c(t, ColorRole::SurfaceSunken))
                        .flex()
                        .items_center()
                        .justify_center()
                        .text_size(px(9.5))
                        .text_color(c(t, ColorRole::FgMuted))
                        .child(tile_name(f, 7))
                        .when(!s.done, |d| {
                            d.child(
                                div()
                                    .absolute()
                                    .top(px(3.))
                                    .right(px(5.))
                                    .text_size(px(9.))
                                    .opacity(1.0 - s.fade)
                                    .text_color(c(t, ColorRole::FgDefault))
                                    .child(format!("{}%", s.pct)),
                            )
                        })
                })
            }))
    }
}

/// `run-mode-menu`: the mode list with Autoresearch's budget.
#[derive(IntoElement)]
pub struct RunModeMenuEl {
    mode: RunMode,
    budget: u32,
    theme: &'static Theme,
}

impl RunModeMenuEl {
    /// The open menu with `mode` checked.
    pub fn new(mode: RunMode, budget: u32) -> Self {
        Self {
            mode,
            budget,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for RunModeMenuEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let cur = self.mode;
        let budget = self.budget;
        card(t)
            .w(px(248.))
            .children(RunMode::ALL.into_iter().map(move |m| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(9.))
                    .px(px(8.))
                    .py(px(6.))
                    .rounded(px(8.))
                    .when(m == cur, |d| d.bg(c(t, ColorRole::SurfaceHover)))
                    .child(shape(t, m.shape(), 9., c(t, ColorRole::FgDefault)))
                    .child(
                        div()
                            .flex()
                            .flex_col()
                            .child(div().font_weight(FontWeight::MEDIUM).child(m.label()))
                            .child(muted(t, m.blurb())),
                    )
                    .when(m.budgeted(), |d| d.child(muted(t, hm(budget))))
            }))
    }
}

/// `experiment-grid`: experiments as shapes in bands of hours.
#[derive(IntoElement)]
pub struct ExperimentGridEl {
    experiments: Vec<Experiment>,
    budget: u32,
    elapsed: u64,
    band: u32,
    theme: &'static Theme,
}

impl ExperimentGridEl {
    /// The grid at `elapsed` seconds.
    pub fn new(experiments: Vec<Experiment>, budget: u32, elapsed: u64) -> Self {
        Self {
            experiments,
            budget,
            elapsed,
            band: 4,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for ExperimentGridEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let shown: Vec<Experiment> = self
            .experiments
            .into_iter()
            .filter(|e| e.at <= self.elapsed)
            .collect();
        let grid = experiment_grid(&shown, self.band);
        let rows = self.budget.div_ceil(60).div_ceil(self.band.max(1));
        let (line, _) = budget_line(self.elapsed, self.budget);
        let band = self.band;
        card(t)
            .w(px(420.))
            .child(muted(t, format!("Autoresearch · {line}")))
            .child(muted(t, experiment_counts(&shown)))
            .children((0..rows).map(move |r| {
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().w(px(26.)).child(muted(t, format!("{}h", r * band))))
                    .children(
                        grid.iter()
                            .filter(|g| g.1 == r)
                            .map(|g| {
                                let o = shown[g.0].outcome;
                                let col = match o {
                                    Outcome::NoGain | Outcome::Summary => c(t, ColorRole::FgMuted),
                                    _ => c(t, ColorRole::FgDefault),
                                };
                                shape(t, o.shape(), 9., col)
                            })
                            .collect::<Vec<_>>(),
                    )
            }))
    }
}

/// `doc-grid`: document cards, favourites first when asked.
#[derive(IntoElement)]
pub struct DocGridEl {
    docs: Vec<Doc>,
    favorites: bool,
    sort: DocSort,
    theme: &'static Theme,
}

impl DocGridEl {
    /// The grid.
    pub fn new(docs: Vec<Doc>) -> Self {
        Self {
            docs,
            favorites: false,
            sort: DocSort::LastVisited,
            theme: &Theme::DARK,
        }
    }
    /// Favorites tab and sort.
    pub fn view(mut self, favorites: bool, sort: DocSort) -> Self {
        self.favorites = favorites;
        self.sort = sort;
        self
    }
}

impl RenderOnce for DocGridEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let idx = doc_view(&self.docs, self.favorites, "", self.sort);
        let docs = self.docs;
        div()
            .flex()
            .flex_wrap()
            .gap(px(10.))
            .children(idx.into_iter().map(move |i| {
                let d = &docs[i];
                card(t)
                    .w(px(200.))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(d.title.clone()),
                    )
                    .child(muted(t, d.excerpt.clone()))
            }))
    }
}

/// `polling-cost`: full re-reads against status pings.
#[derive(IntoElement)]
pub struct PollingCostEl {
    agents: u32,
    every: u32,
    minutes: u32,
    transcript: f64,
    theme: &'static Theme,
}

impl PollingCostEl {
    /// `agents` polled every `every` seconds for `minutes`.
    pub fn new(agents: u32, every: u32, minutes: u32, transcript: f64) -> Self {
        Self {
            agents,
            every,
            minutes,
            transcript,
            theme: &Theme::DARK,
        }
    }
}

impl RenderOnce for PollingCostEl {
    fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
        let t = self.theme;
        let p = poll_cost(
            self.agents,
            self.every,
            self.minutes,
            self.transcript,
            600.0,
            300.0,
        );
        let bar = |f: f32, a: f32| {
            div()
                .flex_1()
                .h(px(8.))
                .rounded(px(999.))
                .bg(c(t, ColorRole::SurfaceSunken))
                .child(
                    div()
                        .h_full()
                        .w(relative(f.clamp(0.006, 1.0)))
                        .rounded(px(999.))
                        .bg(alpha(c(t, ColorRole::FgDefault), a)),
                )
        };
        card(t)
            .w(px(420.))
            .child(
                div()
                    .flex()
                    .gap(px(10.))
                    .items_center()
                    .child(muted(t, "Re-read transcript"))
                    .child(bar(1.0, 0.35))
                    .child(compact(p.full)),
            )
            .child(
                div()
                    .flex()
                    .gap(px(10.))
                    .items_center()
                    .child(muted(t, "Status ping"))
                    .child(bar((p.ping / p.full.max(1.0)) as f32, 1.0))
                    .child(compact(p.ping)),
            )
            .child(muted(
                t,
                format!("{:.0}% less over {} polls", p.saved * 100.0, p.polls),
            ))
    }
}
