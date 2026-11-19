//! [`ScreenView`]: every screen set from S0 to S8 and S12 to S14 (app shells, AI Chat, Workspace, Agent ops
//! I and II, Orchestration / Workflow / Evals, Settings I and II, Onboarding, Arc screens I and II) as one
//! interactive GPUI view
//! (`Entity` + `Render`). It keeps an [`ScreenUi`] from `oxyn_core::parity::screenui` and draws the
//! open screen's snapshot element in the open state, so one set of rules drives every screen:
//!
//! - the screen's own presses route through the core: sidebar nav (`nav-<key>`, `set-<page>`,
//!   `open-<page>`) opens that screen, even in another set; actions open their state (`export` →
//!   Exporting, `invite` → Inviting, `continue` → the next onboarding step); anything else is said
//!   back in the bar;
//! - the bar above the screen has the set's screens as tabs, the state bar (Data, Loading, Empty
//!   and the screen's own states), a command bar (a real [`ImeInput`]: type a screen or a state,
//!   pick a match or press Enter) and the theme switch;
//! - the account row opens a menu (Theme, Sign out) that Escape or a second press closes;
//! - with focus on the view: Left / Right step the states, Up / Down the screens, Escape closes
//!   the menu and then returns to Data, `/` focuses the command bar;
//! - the Arc screens (S12) draw the core's live `ArcUi`: the metrics range, KPI, Compare and zoom;
//!   board cards picked by a press and dropped on a column (or dragged there), Alt + arrows move
//!   the picked card; tasks completed or reopened; timeline bars picked and nudged (Alt + ← / →,
//!   Alt + Shift for the end) or dragged onto a day; the directory's search is a second
//!   [`ImeInput`] and its filters, views and favourites are presses (the same field filters the
//!   timeline's tasks);
//! - the Arc screens II (S13) draw the core's live `Arc2Ui`: calendar views, days and calendars,
//!   events picked and nudged (Alt + arrows, Alt + Shift + ↑ ↓ resize) or dragged onto a day;
//!   inbox triage with J / K / E / H / X / U / A; the support composer (a third [`ImeInput`],
//!   Enter sends) with saved replies, notes, Snooze and Close; the help center's search (the same
//!   field), articles, feedback and Publish;
//! - the Arc screens III (S14) draw the core's live `Arc3Ui`: symbols, ranges and the order
//!   ticket; collections, colour and price filters, sizes and the cart; invoice lines, terms,
//!   tax and Send; the sign-in's demo account, Show password, Sign in and SSO.
//!
//! Create one with `cx.new(|cx| ScreenView::new(6, window, cx))` (set index into
//! [`oxyn_core::parity::screenui::SETS`]) after calling [`super::bind_ime_keys`] once.

use std::rc::Rc;

use gpui::{
    AnyElement, App, Context, Div, Entity, FocusHandle, Focusable, FontWeight, KeyDownEvent,
    MouseButton, Render, SharedString, Window, div, px,
};
use oxyn_core::parity::appui::AppKey;
use oxyn_core::parity::arcscr::ArcUi;
use oxyn_core::parity::arcscr2::Arc2Ui;
use oxyn_core::parity::arcscr3::Arc3Ui;
use oxyn_core::parity::screensfid::{ScreenState, ShellLayout};
use oxyn_core::parity::screenui::{Moved, SETS, ScreenUi};
use oxyn_tokens::Theme;

use super::Act;
use super::ime::{ImeEvent, ImeInput};
use super::rulerfid::{ThPal, th_pal};
use super::*;

/// The snapshot element for screen `slug` in state (`st`, `extra`).
pub fn screen_element(
    slug: &str,
    st: ScreenState,
    extra: &'static str,
    lay: ShellLayout,
    theme: &'static Theme,
    act: Act,
) -> AnyElement {
    macro_rules! el {
        ($t:ty) => {
            <$t>::default()
                .state(st)
                .extra(extra)
                .layout(lay)
                .theme(theme)
                .on_act(act)
                .into_any_element()
        };
    }
    macro_rules! el0 {
        ($t:ty) => {
            <$t>::default()
                .state(st)
                .layout(lay)
                .theme(theme)
                .on_act(act)
                .into_any_element()
        };
    }
    match slug {
        "shell-three-pane" => el0!(ScreenRailEl),
        "settings" => el0!(SettingsEl),
        "chat-start" => el!(ChatStartEl),
        "chat-overview" => el!(ChatOverviewEl),
        "chat-agents" => el!(ChatAgentsEl),
        "chat-double-sidebar" => el!(ChatRailEl),
        "chat-progress" => el!(ChatProgressEl),
        "chat-run" => el!(ChatRunEl),
        "folders-overview" => el!(FoldersEl),
        "folders-detail" => el!(FolderDetailEl),
        "integrations" => el!(IntegrationsEl),
        "kanban" => el!(KanbanEl),
        "crm" => el!(CrmEl),
        "analytics" => el!(AnalyticsEl),
        "agent-thread" => el!(AgentThreadEl),
        "inbox" => el!(InboxScreenEl),
        "agent-inbox-board" => el!(InboxBoardEl),
        "agent-tasks" => el!(TasksEl),
        "agent-activity" => el!(ActivityEl),
        "agent-memory" => el!(MemoryEl),
        "agent-agents" => el!(AgentsEl),
        "agent-skills" => el!(SkillsEl),
        "agent-schedules" => el!(SchedulesEl),
        "agent-usage" => el!(UsageEl),
        "agent-permissions" => el!(PermissionsEl),
        "agent-run-detail" => el!(RunDetailEl),
        "agent-orchestration" => el0!(OrchestrationEl),
        "workflow" => el0!(WorkflowEl),
        "evals" => el!(EvalsEl),
        "settings-profile" => el!(ProfileSettingsEl),
        "settings-preferences" => el!(PreferencesSettingsEl),
        "settings-notifications" => el!(NotificationsSettingsEl),
        "settings-privacy" => el!(PrivacySettingsEl),
        "settings-general" => el!(GeneralSettingsEl),
        "settings-members" => el!(MembersSettingsEl),
        "settings-models" => el!(ModelsSettingsEl),
        "settings-security" => el!(SecuritySettingsEl),
        "settings-billing" => el!(BillingSettingsEl),
        "settings-onboarding" => el!(OnboardingSettingsEl),
        "onboarding-website" => el!(OnboardingWebsiteEl),
        "onboarding-brand" => el!(OnboardingBrandEl),
        "onboarding-tone" => el!(OnboardingToneEl),
        "onboarding-ready" => el!(OnboardingReadyEl),
        "metrics-dashboard" | "kanban-board" | "project-board" | "gantt-timeline"
        | "team-directory" => arc_element(slug, st, extra, lay, theme, act, None, None),
        "week-calendar"
        | "month-calendar"
        | "inbox-triage"
        | "support-conversation"
        | "help-center" => arc2_element(slug, st, extra, lay, theme, act, None, None),
        "market-terminal" | "product-list" | "product-detail" | "invoice-studio"
        | "sign-in-split" => arc3_element(slug, st, extra, lay, theme, act, None),
        _ => el0!(ScreenShellEl),
    }
}

/// An Arc screen II (S13) for slug `slug`, drawing the live state `arc` when given (else the
/// sample for the state) with `field` as the support composer or the help center search.
#[allow(clippy::too_many_arguments)]
pub fn arc2_element(
    slug: &str,
    st: ScreenState,
    extra: &'static str,
    lay: ShellLayout,
    theme: &'static Theme,
    act: Act,
    arc: Option<Arc2Ui>,
    field: Option<gpui::AnyView>,
) -> AnyElement {
    macro_rules! el {
        ($t:ty) => {{
            let mut e = <$t>::default()
                .state(st)
                .extra(extra)
                .layout(lay)
                .theme(theme)
                .on_act(act);
            if let Some(a) = arc {
                e = e.arc2(a);
            }
            if let Some(v) = field {
                e = e.field(v);
            }
            e.into_any_element()
        }};
    }
    match slug {
        "month-calendar" => el!(MonthCalendarEl),
        "inbox-triage" => el!(InboxTriageEl),
        "support-conversation" => el!(SupportConversationEl),
        "help-center" => el!(HelpCenterEl),
        "week-calendar" => el!(WeekCalendarEl),
        // Not an S13 slug: the first S13 screen.
        _ => el!(WeekCalendarEl),
    }
}

/// An Arc screen III (S14) for slug `slug`, drawing the live state `arc` when given (else the
/// sample for the state).
pub fn arc3_element(
    slug: &str,
    st: ScreenState,
    extra: &'static str,
    lay: ShellLayout,
    theme: &'static Theme,
    act: Act,
    arc: Option<Arc3Ui>,
) -> AnyElement {
    macro_rules! el {
        ($t:ty) => {{
            let mut e = <$t>::default()
                .state(st)
                .extra(extra)
                .layout(lay)
                .theme(theme)
                .on_act(act);
            if let Some(a) = arc {
                e = e.arc3(a);
            }
            e.into_any_element()
        }};
    }
    match slug {
        "product-list" => el!(ProductListEl),
        "product-detail" => el!(ProductDetailEl),
        "invoice-studio" => el!(InvoiceStudioEl),
        "sign-in-split" => el!(SignInSplitEl),
        "market-terminal" => el!(MarketTerminalEl),
        // Not an S14 slug: the first S14 screen.
        _ => el!(MarketTerminalEl),
    }
}

/// An Arc screen (S12) for slug `slug`, drawing the live state `arc` when given (else the
/// sample for the state) with `search` as the Team directory's search field.
#[allow(clippy::too_many_arguments)]
pub fn arc_element(
    slug: &str,
    st: ScreenState,
    extra: &'static str,
    lay: ShellLayout,
    theme: &'static Theme,
    act: Act,
    arc: Option<ArcUi>,
    search: Option<gpui::AnyView>,
) -> AnyElement {
    macro_rules! el {
        ($t:ty) => {{
            let mut e = <$t>::default()
                .state(st)
                .extra(extra)
                .layout(lay)
                .theme(theme)
                .on_act(act);
            if let Some(a) = arc {
                e = e.arc(a);
            }
            if let Some(v) = search {
                e = e.search(v);
            }
            e.into_any_element()
        }};
    }
    match slug {
        "kanban-board" => el!(KanbanBoardEl),
        "project-board" => el!(ProjectBoardEl),
        "gantt-timeline" => el!(GanttTimelineEl),
        "team-directory" => el!(TeamDirectoryEl),
        "metrics-dashboard" => el!(MetricsDashboardEl),
        // Not an Arc slug: the first Arc screen.
        _ => el!(MetricsDashboardEl),
    }
}

/// Every screen set as one interactive view (see the module docs).
pub struct ScreenView {
    ui: ScreenUi,
    dark: bool,
    layout: ShellLayout,
    focus: FocusHandle,
    input: Entity<ImeInput>,
    search: Entity<ImeInput>,
    /// The S13 field: the support composer or the help center search.
    field: Entity<ImeInput>,
    /// The search field's current placeholder (it follows the open screen).
    ph: &'static str,
}

impl Focusable for ScreenView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl ScreenView {
    /// A view on the first screen of set `set`, focused so the keys work at once.
    pub fn new(set: usize, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            let mut i = ImeInput::new(cx, "Go to a screen or state…");
            i.set_bare(true);
            i
        });
        cx.subscribe(&input, |this, inp, ev: &ImeEvent, cx| {
            match ev {
                ImeEvent::Changed => this.ui.query = inp.read(cx).text().to_string(),
                ImeEvent::Enter { .. } => {
                    this.ui.query = inp.read(cx).text().to_string();
                    this.ui.key(AppKey::Enter);
                    inp.update(cx, |i, cx| i.set_text("", cx));
                }
                ImeEvent::Accepted { .. } | ImeEvent::Dismissed(_) => {}
            }
            cx.notify();
        })
        .detach();
        let search = cx.new(|cx| {
            let mut i = ImeInput::new(cx, "Search people, roles, skills");
            i.set_bare(true);
            i
        });
        cx.subscribe(&search, |this, inp, ev: &ImeEvent, cx| {
            match ev {
                ImeEvent::Changed | ImeEvent::Enter { .. } => {
                    this.ui.arc.query = inp.read(cx).text().to_string();
                }
                ImeEvent::Accepted { .. } | ImeEvent::Dismissed(_) => {}
            }
            cx.notify();
        })
        .detach();
        let field = cx.new(|cx| {
            let mut i = ImeInput::new(cx, "Search articles");
            i.set_bare(true);
            i
        });
        cx.subscribe(&field, |this, inp, ev: &ImeEvent, cx| {
            let text = inp.read(cx).text().to_string();
            match (this.ui.screen_def().slug, ev) {
                ("support-conversation", ImeEvent::Enter { .. }) => {
                    this.ui.arc2.desk.draft = text;
                    this.press("sp-send", cx);
                }
                ("support-conversation", ImeEvent::Changed) => this.ui.arc2.desk.draft = text,
                ("help-center", ImeEvent::Changed | ImeEvent::Enter { .. }) => {
                    this.ui.arc2.help.query = text
                }
                _ => {}
            }
            cx.notify();
        })
        .detach();
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            ui: ScreenUi::new(set),
            dark: true,
            layout: ShellLayout::Inset,
            focus,
            input,
            search,
            field,
            ph: "Search people, roles, skills",
        }
    }

    /// The interaction state.
    pub fn ui(&self) -> &ScreenUi {
        &self.ui
    }

    /// Opens set `set` on its first screen.
    pub fn open_set(&mut self, set: usize, cx: &mut Context<Self>) {
        if set != self.ui.set {
            self.ui.open(set, 0);
            cx.notify();
        }
    }

    /// Light or dark chrome.
    pub fn set_dark(&mut self, dark: bool, cx: &mut Context<Self>) {
        self.dark = dark;
        let t = self.theme();
        self.input.update(cx, |i, cx| i.set_theme(t, cx));
        self.search.update(cx, |i, cx| i.set_theme(t, cx));
        self.field.update(cx, |i, cx| i.set_theme(t, cx));
        cx.notify();
    }

    /// Inset or Flat frame.
    pub fn set_layout(&mut self, l: ShellLayout, cx: &mut Context<Self>) {
        self.layout = l;
        cx.notify();
    }

    fn theme(&self) -> &'static Theme {
        if self.dark {
            &Theme::DARK
        } else {
            &Theme::LIGHT
        }
    }

    /// Applies a press from the screen or the menu.
    pub fn press(&mut self, id: &'static str, cx: &mut Context<Self>) {
        match id {
            "menu-theme" => {
                self.ui.menu = false;
                self.set_dark(!self.dark, cx);
            }
            "menu-out" => {
                self.ui.menu = false;
                self.ui.said = Some("signed out".into());
                cx.notify();
            }
            "td-clear" => {
                self.ui.press(id);
                self.search.update(cx, |i, cx| i.set_text("", cx));
                cx.notify();
            }
            _ => {
                let screen = self.ui.screen_def().slug;
                if self.ui.press(id) != Moved::None {
                    self.sync_fields(screen != self.ui.screen_def().slug, cx);
                    cx.notify();
                }
            }
        }
    }

    /// Keeps the text fields in step with the open screen: the S12 search's placeholder (people
    /// or tasks) and, on S13, the field's text (the support draft or the help search) after a
    /// press changed it. A new screen starts with an empty search.
    fn sync_fields(&mut self, moved: bool, cx: &mut Context<Self>) {
        let slug = self.ui.screen_def().slug;
        if moved {
            self.ui.arc.query.clear();
            self.search.update(cx, |i, cx| i.set_text("", cx));
        }
        let ph = if slug == "gantt-timeline" {
            "Filter tasks"
        } else {
            "Search people, roles, skills"
        };
        if ph != self.ph {
            self.ph = ph;
            self.search.update(cx, |i, _| i.set_placeholder(ph));
        }
        let want = match slug {
            "support-conversation" => Some((self.ui.arc2.desk.draft.clone(), "Write a reply…")),
            "help-center" => Some((self.ui.arc2.help.query.clone(), "Search articles")),
            _ => None,
        };
        if let Some((t, ph)) = want {
            self.field.update(cx, |i, cx| {
                i.set_placeholder(ph);
                if i.text() != t {
                    i.set_text(&t, cx);
                }
            });
        }
    }

    /// Alt + arrows on the Arc screens: move the picked board card, or nudge the picked bar
    /// (Shift changes its end). True when handled.
    fn arc_key(&mut self, name: &str, shift: bool) -> bool {
        let id = match (self.ui.screen_def().slug, name, shift) {
            ("kanban-board", "left", _) => "kb-left",
            ("kanban-board", "right", _) => "kb-right",
            ("kanban-board", "up", _) => "kb-up",
            ("kanban-board", "down", _) => "kb-down",
            ("gantt-timeline", "left", false) => "gt-earlier",
            ("gantt-timeline", "right", false) => "gt-later",
            ("gantt-timeline", "left", true) => "gt-shorter",
            ("gantt-timeline", "right", true) => "gt-longer",
            ("week-calendar" | "month-calendar", "left", _) => "cal-earlier",
            ("week-calendar" | "month-calendar", "right", _) => "cal-later",
            ("week-calendar" | "month-calendar", "up", false) => "cal-up",
            ("week-calendar" | "month-calendar", "down", false) => "cal-down",
            ("week-calendar" | "month-calendar", "up", true) => "cal-shorter",
            ("week-calendar" | "month-calendar", "down", true) => "cal-longer",
            _ => return false,
        };
        self.ui.press(id);
        true
    }

    fn key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let typing = self.input.read(cx).focus().is_focused(window)
            || self.search.read(cx).focus().is_focused(window)
            || self.field.read(cx).focus().is_focused(window);
        let name = ev.keystroke.key.as_str();
        if typing {
            if name == "escape" {
                window.focus(&self.focus, cx);
                cx.notify();
            }
            return;
        }
        if name == "/" {
            let f = if matches!(
                self.ui.screen_def().slug,
                "team-directory" | "gantt-timeline"
            ) {
                self.search.read(cx).focus().clone()
            } else if matches!(
                self.ui.screen_def().slug,
                "help-center" | "support-conversation"
            ) {
                self.field.read(cx).focus().clone()
            } else {
                self.input.read(cx).focus().clone()
            };
            window.focus(&f, cx);
            return;
        }
        if ev.keystroke.modifiers.alt && self.arc_key(name, ev.keystroke.modifiers.shift) {
            cx.notify();
            return;
        }
        let m = &ev.keystroke.modifiers;
        if !(m.alt || m.control || m.platform) && self.ui.arc2.key(self.ui.screen_def().slug, name)
        {
            self.ui.said = self.ui.arc2.said.take();
            cx.notify();
            return;
        }
        if let Some(k) = AppKey::from_name(name)
            && k != AppKey::Enter
            && self.ui.key(k) != Moved::None
        {
            cx.notify();
        }
    }

    fn act(&self, cx: &mut Context<Self>) -> Act {
        let me = cx.entity().downgrade();
        Rc::new(move |id, _, cx| {
            let _ = me.update(cx, |v, cx| v.press(id, cx));
        })
    }

    fn chip(
        p: &ThPal,
        id: SharedString,
        l: impl Into<SharedString>,
        on: bool,
    ) -> gpui::Stateful<Div> {
        div()
            .id(id)
            .h(px(26.))
            .px(px(10.))
            .flex()
            .items_center()
            .rounded(px(7.))
            .cursor_pointer()
            .text_color(if on { p.fg } else { p.fg2 })
            .when(on, |d| d.bg(p.sel))
            .hover(|s| s.bg(p.sel))
            .child(l.into())
    }

    fn bar(&self, p: &ThPal, cx: &mut Context<Self>) -> Div {
        let sd = self.ui.set_def();
        let mut tabs = div().flex().items_center().gap(px(2.));
        for (i, s) in sd.screens.iter().enumerate() {
            tabs = tabs.child(
                Self::chip(p, format!("scr-{i}").into(), s.name, i == self.ui.screen).on_click(
                    cx.listener(move |this, _, _, cx| {
                        let set = this.ui.set;
                        if this.ui.open(set, i) {
                            cx.notify();
                        }
                    }),
                ),
            );
        }
        let mut states = div().flex().items_center().gap(px(2.));
        for (i, (_, l)) in self.ui.screen_def().states.iter().enumerate() {
            states = states.child(
                Self::chip(p, format!("st-{i}").into(), *l, i == self.ui.state).on_click(
                    cx.listener(move |this, _, _, cx| {
                        this.ui.state = i;
                        this.ui.menu = false;
                        cx.notify();
                    }),
                ),
            );
        }
        let theme = Self::chip(
            p,
            "theme".into(),
            if self.dark { "Light" } else { "Dark" },
            false,
        )
        .on_click(cx.listener(|this, _, _, cx| {
            let d = !this.dark;
            this.set_dark(d, cx);
        }));
        let cmd = div()
            .id("cmd")
            .w(px(260.))
            .h(px(28.))
            .px(px(10.))
            .flex()
            .items_center()
            .rounded(px(8.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .child(self.input.clone());
        let said = self
            .ui
            .said
            .clone()
            .map(|s| div().text_color(p.mute).child(SharedString::from(s)));
        div()
            .flex_none()
            .flex()
            .flex_col()
            .gap(px(4.))
            .px(px(10.))
            .py(px(6.))
            .bg(p.inset)
            .border_b_1()
            .border_color(p.line)
            .text_size(px(12.5))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(sd.name))
                    .child(tabs)
                    .child(div().flex_1())
                    .child(cmd)
                    .child(theme),
            )
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .child(div().text_color(p.mute).child("State"))
                    .child(states)
                    .child(div().flex_1())
                    .children(said),
            )
    }

    fn found(&self, p: &ThPal, cx: &mut Context<Self>) -> Option<Div> {
        let m = ScreenUi::matches(&self.ui.query, 6);
        if m.is_empty() {
            return None;
        }
        let mut l = div()
            .absolute()
            .top(px(42.))
            .right(px(70.))
            .w(px(300.))
            .p(px(6.))
            .flex()
            .flex_col()
            .rounded(px(10.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .shadow_lg()
            .text_size(px(12.5));
        for (n, (s, i, st)) in m.into_iter().enumerate() {
            let sc = &SETS[s].screens[i];
            let label = match st {
                Some(k) => format!("{} · {}", sc.name, sc.states[k].1),
                None => sc.name.to_string(),
            };
            l = l.child(
                Self::chip(p, format!("hit-{n}").into(), label, n == 0)
                    .child(div().flex_1())
                    .child(div().text_color(p.mute).child(SETS[s].name))
                    .gap(px(8.))
                    .on_click(cx.listener(move |this, _, w, cx| {
                        this.ui.open(s, i);
                        if let Some(k) = st {
                            this.ui.state = k;
                        }
                        this.ui.query.clear();
                        this.input.update(cx, |x, cx| x.set_text("", cx));
                        w.focus(&this.focus, cx);
                        cx.notify();
                    })),
            );
        }
        Some(l)
    }

    fn menu(&self, p: &ThPal, act: &Act) -> Div {
        let item = |l: &'static str, id: &'static str| {
            super::apps::hit(div(), &Some(act.clone()), id)
                .h(px(32.))
                .px(px(10.))
                .flex()
                .items_center()
                .rounded(px(8.))
                .text_color(p.fg2)
                .hover(|s| s.bg(p.sel))
                .child(l)
        };
        div()
            .absolute()
            .left(px(12.))
            .bottom(px(64.))
            .w(px(232.))
            .p(px(6.))
            .flex()
            .flex_col()
            .rounded(px(12.))
            .border_1()
            .border_color(p.line)
            .bg(p.card)
            .shadow_lg()
            .child(
                div()
                    .px(px(10.))
                    .py(px(6.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .text_color(p.fg)
                    .child(oxyn_core::parity::screensfid::PERSON),
            )
            .child(item(
                if self.dark {
                    "Switch to light"
                } else {
                    "Switch to dark"
                },
                "menu-theme",
            ))
            .child(item("Sign out", "menu-out"))
    }
}

impl Render for ScreenView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let p = th_pal(theme);
        let act = self.act(cx);
        let (st, extra) = self.ui.split();
        let slug = self.ui.screen_def().slug;
        let screen = if self.ui.set_def().id == "s14" {
            arc3_element(
                slug,
                st,
                extra,
                self.layout,
                theme,
                act.clone(),
                Some(self.ui.arc3.clone()),
            )
        } else if self.ui.set_def().id == "s13" {
            arc2_element(
                slug,
                st,
                extra,
                self.layout,
                theme,
                act.clone(),
                Some(self.ui.arc2.clone()),
                Some(self.field.clone().into()),
            )
        } else if self.ui.set_def().id == "s12" {
            arc_element(
                slug,
                st,
                extra,
                self.layout,
                theme,
                act.clone(),
                Some(self.ui.arc.clone()),
                Some(self.search.clone().into()),
            )
        } else {
            screen_element(slug, st, extra, self.layout, theme, act.clone())
        };
        let bar = self.bar(&p, cx);
        let found = self.found(&p, cx);
        let menu = self.ui.menu.then(|| self.menu(&p, &act));
        div()
            .id("screen-view")
            .key_context("OxynScreenView")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, w, cx| this.key(ev, w, cx)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, w, cx| {
                    if !this.input.read(cx).focus().is_focused(w) {
                        w.focus(&this.focus, cx);
                    }
                }),
            )
            .relative()
            .size_full()
            .flex()
            .flex_col()
            .bg(p.inset)
            .text_color(p.fg)
            .child(bar)
            .child(
                div()
                    .relative()
                    .flex_1()
                    .min_h_0()
                    .overflow_hidden()
                    .child(screen)
                    .children(menu),
            )
            .children(found)
    }
}

#[cfg(test)]
mod tests {
    use oxyn_core::parity::screenui::SETS;

    #[test]
    fn every_slug_has_an_element() {
        // The fallback arm draws the plain shell; every other slug must have its own arm.
        let src = include_str!("screenview.rs");
        for s in SETS {
            for x in s.screens {
                if x.slug != "shell-inset" {
                    assert!(
                        src.contains(&format!("\"{}\" =>", x.slug)),
                        "{} has no element",
                        x.slug
                    );
                }
            }
        }
    }
}
