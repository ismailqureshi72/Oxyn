//! Interactive GPUI views for the six agent apps: Taskdeck, Command Center, Session Studio,
//! Fleet Window, Workspace Coder and Command Board. Each [`AgentAppView`] is a real GPUI view
//! (`Entity` + `Render`) that keeps an [`AppUi`] from `oxyn_core::parity::appui` and redraws the
//! same screens as the snapshot elements from it, so the Dioxus apps and these views share one
//! set of rules:
//!
//! - sidebar rows, nav items and pane tabs are pressable (`row-N`, `view:<name>`, the app's own
//!   ids such as `tasks`, `place`, `open-pr`);
//! - the logo toggles the icon rail, the account row opens a menu (Theme, Sign out) that Escape
//!   or a second press closes;
//! - the composer is a real [`ImeInput`]: Enter or the send button sends, and the sent prompts
//!   show above it;
//! - a press anywhere in the composer's box puts the caret in its field;
//! - with focus on the app (not the composer): Up / Down move the open row, Left / Right step
//!   through the app's views, Escape closes the menu and then the rail;
//! - each app's live parts keep an [`AppData`] (`Own::data`): Taskdeck's plugin and skill
//!   switches, Plugins / Skills tabs, schedule Edit / Pause / Run now and the site preview;
//!   Command Center's Review / Terminal / Browser tabs, a terminal that runs typed commands and a
//!   browser with an address field, links, Back and Reload; Session Studio's side panes, reordered
//!   by drag and drop or the keyboard (Up / Down pick, Alt + Up / Down move) and its side chat,
//!   which answers from the composer; Fleet Window's model toggles; Workspace Coder's four
//!   workspace tabs; Command Board's timeline scrubber (stops, ◀ / ▶, Alt + Left / Right, Live)
//!   and Sleep / Wake; and running clocks that tick once a second.
//!
//! The press handling per app lives in [`AgentApp::press`] so it reads next to the views it
//! drives. Create a view with `cx.new(|cx| AgentAppView::new(AgentApp::FleetWindow, window, cx))`
//! after calling [`super::bind_ime_keys`] once.

use std::rc::Rc;
use std::time::Duration;

use gpui::{
    AnyView, App, Context, Div, Entity, FocusHandle, Focusable, FontWeight, KeyDownEvent,
    MouseButton, Render, SharedString, Stateful, Task, Window, div, prelude::*, px,
};
use oxyn_core::parity::appui::{AppData, AppKey, AppUi, views};
use oxyn_core::parity::screensfid::{ScreenState, ShellLayout};
use oxyn_core::parity::{cmdboard, cmdstudio, fleetwin, taskdeck, wscoder};
use oxyn_tokens::Theme;

use super::Act;
use super::ime::{ImeEvent, ImeInput};
use super::rulerfid::{ThPal, fire, th_pal};

/// What an interactive view lays over a screen: the open row, the rail, the composer input and
/// the sent prompts. The snapshot elements pass `Live::default()` and draw exactly as before.
#[derive(Clone, Default)]
pub struct Live {
    /// The open sidebar row; `None` keeps the snapshot's own highlight.
    pub sel: Option<usize>,
    /// Sidebar collapsed to the icon rail.
    pub rail: bool,
    /// The composer's text field, drawn in place of the placeholder.
    pub input: Option<AnyView>,
    /// Sent prompts, oldest first.
    pub sent: Vec<SharedString>,
    /// The live data the screens draw instead of their samples (switches, terminal, panes,
    /// models, timeline, timers); `None` keeps the snapshot.
    pub data: Option<AppData>,
    /// A second text field: the terminal's command line or the browser's address.
    pub aux: Option<AnyView>,
    /// Drops a dragged side pane: (from, to) positions.
    pub mover: Option<Mover>,
}

/// Moves a dragged pane from one position to another.
pub type Mover = Rc<dyn Fn(usize, usize, &mut Window, &mut App)>;

/// A side pane being dragged (its position), drawn as a small chip under the pointer.
#[derive(Clone, Copy)]
pub struct PaneDrag {
    /// The pane's position when the drag started.
    pub from: usize,
    /// Its title.
    pub title: &'static str,
}

impl Render for PaneDrag {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .px(px(10.))
            .py(px(4.))
            .rounded(px(8.))
            .bg(gpui::rgb(0x2A2A2E))
            .text_color(gpui::rgb(0xF4F4F5))
            .text_size(px(12.))
            .child(self.title)
    }
}

impl Live {
    /// Whether row `i` is the open one; `snap` is the snapshot's answer when no row is set.
    pub fn on(&self, i: usize, snap: bool) -> bool {
        self.sel.map_or(snap, |s| s == i)
    }
}

/// Makes `d` pressable: an element id and a click that fires `id` through `act`.
pub(super) fn hit(d: Div, act: &Option<Act>, id: &'static str) -> Stateful<Div> {
    let f = fire(act, id);
    d.id(id).cursor_pointer().on_click(move |_, w, cx| f(w, cx))
}

/// One of the six agent apps.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AgentApp {
    /// S9 Taskdeck: tasks, plugins and skills, schedules, sites.
    Taskdeck,
    /// S10 Command Center: threads by project with a review pane.
    CommandCenter,
    /// S10 Session Studio: sessions with read / edit / bash blocks and a diff pane.
    SessionStudio,
    /// S11 Fleet Window: parallel agents with Compare.
    FleetWindow,
    /// S11 Workspace Coder: chat with the agent's own computer.
    WorkspaceCoder,
    /// S11 Command Board: autonomous sessions with a workspace timeline.
    CommandBoard,
}

impl AgentApp {
    /// The six apps, in gallery order.
    pub const ALL: [AgentApp; 6] = [
        AgentApp::Taskdeck,
        AgentApp::CommandCenter,
        AgentApp::SessionStudio,
        AgentApp::FleetWindow,
        AgentApp::WorkspaceCoder,
        AgentApp::CommandBoard,
    ];

    /// The app's display name.
    pub fn name(self) -> &'static str {
        match self {
            Self::Taskdeck => "Taskdeck",
            Self::CommandCenter => "Command Center",
            Self::SessionStudio => "Session Studio",
            Self::FleetWindow => "Fleet Window",
            Self::WorkspaceCoder => "Workspace Coder",
            Self::CommandBoard => "Command Board",
        }
    }

    /// The views Left / Right step through.
    pub fn views(self) -> &'static [&'static str] {
        match self {
            Self::Taskdeck => views::TASKDECK,
            Self::CommandCenter => views::COMMAND_CENTER,
            Self::SessionStudio => views::SESSION_STUDIO,
            Self::FleetWindow => views::FLEET_WINDOW,
            Self::WorkspaceCoder => views::WORKSPACE_CODER,
            Self::CommandBoard => views::COMMAND_BOARD,
        }
    }

    /// Sidebar rows in data state `st`, from the same core lists the screens draw.
    pub fn rows(self, st: ScreenState, cloud: bool) -> usize {
        match self {
            Self::Taskdeck => {
                let env = if cloud {
                    taskdeck::EnvMode::Cloud
                } else {
                    taskdeck::EnvMode::Local
                };
                taskdeck::tasks(st).iter().filter(|t| t.env == env).count()
            }
            Self::CommandCenter => cmdstudio::threads(st).len(),
            Self::SessionStudio => cmdstudio::sessions(st).len(),
            Self::FleetWindow => fleetwin::fw_agents(st).len(),
            Self::WorkspaceCoder => wscoder::wc_chats(st).len(),
            Self::CommandBoard => cmdboard::cb_sessions(st).len(),
        }
    }

    /// The second field's placeholder: the terminal's command line, or the browser's address.
    pub fn aux_hint(self) -> &'static str {
        match self {
            Self::CommandCenter => "Type a command, or an address in the browser",
            _ => "Type a command",
        }
    }

    /// The composer placeholder.
    pub fn hint(self) -> &'static str {
        match self {
            Self::Taskdeck => "Describe a task · @ for files, plugins and skills",
            Self::CommandCenter => "Queue a follow-up for when this finishes…",
            Self::SessionStudio => "Ask for a change · Shift + Tab switches mode",
            Self::FleetWindow => "Plan, build, fix anything",
            Self::WorkspaceCoder => "Ask anything, or describe what to build",
            Self::CommandBoard => "Reply, or add something for the agent to do",
        }
    }

    /// Applies press `id` to `ui` and the app's own flags; true when anything changed. The app
    /// ids come first, then the shared ones in [`AppUi::press`].
    pub fn press(self, ui: &mut AppUi, own: &mut Own, id: &'static str) -> bool {
        if own.data.press(id) {
            match (self, id) {
                (Self::WorkspaceCoder, _) if id.starts_with("wstab-") => {
                    // Files and Preview are also the finished chat's views.
                    match own.data.ws_tab {
                        0 => ui.open_view("files"),
                        3 => ui.open_view("preview"),
                        _ => false,
                    };
                }
                (Self::CommandBoard, "sleep") => {
                    ui.last = Some(id);
                }
                _ => {}
            }
            return true;
        }
        match (self, id) {
            (Self::SessionStudio, "send") if ui.view_name() == "chat" => {
                let q = std::mem::take(&mut ui.draft);
                own.data.ask_side(&q)
            }
            (Self::CommandCenter, "tab-review") => ui.open_view(""),
            (Self::WorkspaceCoder, "view:files" | "view:preview") => {
                own.data.ws_tab = if id == "view:files" { 0 } else { 3 };
                ui.press(id)
            }
            (Self::Taskdeck, "new-task") => {
                own.approval = false;
                ui.view = 0;
                true
            }
            (Self::Taskdeck, "tasks") => ui.open_view("board"),
            (Self::Taskdeck, "plugins" | "scheduled" | "sites") => ui.open_view(id),
            (Self::Taskdeck, "env-local" | "env-cloud") => {
                let cloud = id == "env-cloud";
                let changed = own.cloud != cloud;
                own.cloud = cloud;
                let rows = self.rows(ui.st, cloud);
                ui.set_state(ui.st, rows);
                changed
            }
            (Self::Taskdeck, "allow" | "deny") => {
                ui.last = Some(id);
                std::mem::replace(&mut own.approval, false)
            }
            (Self::Taskdeck, "send") => {
                if ui.submit().is_some() {
                    own.approval = true;
                    ui.open_view("task");
                    true
                } else {
                    false
                }
            }
            (Self::Taskdeck, _) if id.starts_with("row-") || id.starts_with("open-") => {
                if let Some(i) = oxyn_core::parity::appui::parse_row(id) {
                    ui.open_row(i);
                }
                // The first task (row 0, board card 1) is the one waiting on an approval.
                own.approval = id == "row-0" || id == "open-1";
                ui.open_view("task");
                true
            }
            (Self::CommandCenter, "place") => {
                let to = if ui.view_name() == "worktree" {
                    ""
                } else {
                    "worktree"
                };
                ui.open_view(to)
            }
            (Self::CommandCenter, "handoff") => ui.open_view(""),
            (Self::SessionStudio, "allow-once" | "allow-always" | "deny") => {
                ui.last = Some(id);
                ui.open_view("")
            }
            (Self::FleetWindow, "apply") => {
                ui.last = Some(id);
                ui.open_view("")
            }
            (Self::FleetWindow, "send") if ui.view_name() == "plan" && ui.draft.is_empty() => {
                ui.open_view("")
            }
            (Self::CommandBoard, "open-pr") => ui.open_view("pr"),
            (Self::CommandBoard, "answer") => {
                ui.last = Some(id);
                ui.sent.push("Answered the agent's question".into());
                true
            }
            _ => ui.press(id),
        }
    }

    /// The snapshot `extra` for the current state.
    pub fn extra(self, ui: &AppUi, own: &Own) -> &'static str {
        match self {
            Self::Taskdeck => match ui.view_name() {
                "task" if own.approval => "approval",
                "task" => "",
                "plugins" if own.data.skills_tab => "skills",
                _ if own.cloud => "cloud",
                _ => "",
            },
            _ => ui.view_name(),
        }
    }
}

/// App state that is not a view: Taskdeck's Local / Cloud and its pending approval, and the live
/// data every app edits in place ([`AppData`]).
#[derive(Clone, Debug, PartialEq)]
pub struct Own {
    /// Taskdeck on Cloud.
    pub cloud: bool,
    /// Taskdeck's open task waits on an approval.
    pub approval: bool,
    /// Switches, schedules, terminal, browser, panes, models, tabs, timeline and the clock.
    pub data: AppData,
}

impl Default for Own {
    fn default() -> Self {
        Self {
            cloud: false,
            approval: false,
            data: AppData::for_state(ScreenState::Sample),
        }
    }
}

/// An interactive agent app (see the module docs).
pub struct AgentAppView {
    app: AgentApp,
    ui: AppUi,
    own: Own,
    dark: bool,
    layout: ShellLayout,
    focus: FocusHandle,
    input: Entity<ImeInput>,
    aux: Entity<ImeInput>,
    _clock: Task<()>,
}

impl Focusable for AgentAppView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus.clone()
    }
}

impl AgentAppView {
    /// A view of `app` on its sample data, focused so the keys work at once.
    pub fn new(app: AgentApp, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let input = cx.new(|cx| {
            let mut i = ImeInput::new(cx, app.hint());
            i.set_bare(true);
            i
        });
        cx.subscribe(&input, |this, inp, ev: &ImeEvent, cx| {
            match ev {
                ImeEvent::Changed => this.ui.draft = inp.read(cx).text().to_string(),
                ImeEvent::Enter { .. } => {
                    this.ui.draft = inp.read(cx).text().to_string();
                    this.apply("send", cx);
                }
                ImeEvent::Accepted { .. } | ImeEvent::Dismissed(_) => {}
            }
            cx.notify();
        })
        .detach();
        let aux = cx.new(|cx| {
            let mut i = ImeInput::new(cx, app.aux_hint());
            i.set_bare(true);
            i
        });
        cx.subscribe(&aux, |this, inp, ev: &ImeEvent, cx| {
            if let ImeEvent::Enter { .. } = ev {
                let t = inp.read(cx).text().to_string();
                let done = match this.ui.view_name() {
                    "browser" => this.own.data.go_addr(&t),
                    _ => this.own.data.run_term(&t),
                };
                if this.ui.view_name() == "browser" {
                    if done {
                        let a = oxyn_core::parity::cmdstudio::PAGES[this.own.data.page].0;
                        inp.update(cx, |i, cx| i.set_text(a, cx));
                    }
                } else {
                    inp.update(cx, |i, cx| i.set_text("", cx));
                }
                cx.notify();
            }
        })
        .detach();
        // Running timers: one tick a second while the view lives.
        let clock = cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(Duration::from_secs(1)).await;
                if this
                    .update(cx, |v, cx| {
                        v.own.data.tick();
                        cx.notify();
                    })
                    .is_err()
                {
                    break;
                }
            }
        });
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        let mut ui = AppUi::new(app.rows(ScreenState::Sample, false), app.views());
        if app == AgentApp::Taskdeck {
            ui.open_view("task");
        }
        Self {
            app,
            ui,
            own: Own {
                approval: app == AgentApp::Taskdeck,
                ..Own::default()
            },
            dark: true,
            layout: ShellLayout::Inset,
            focus,
            input,
            aux,
            _clock: clock,
        }
    }

    /// The app this view shows.
    pub fn app(&self) -> AgentApp {
        self.app
    }

    /// The interaction state.
    pub fn ui(&self) -> &AppUi {
        &self.ui
    }

    /// Sets Sample / Loading / Empty.
    pub fn set_state(&mut self, st: ScreenState, cx: &mut Context<Self>) {
        let rows = self.app.rows(st, self.own.cloud);
        self.ui.set_state(st, rows);
        let secs = self.own.data.secs;
        self.own.data = AppData::for_state(st);
        self.own.data.secs = secs;
        cx.notify();
    }

    /// The live data (switches, terminal, panes, models, timeline, clock).
    pub fn data(&self) -> &AppData {
        &self.own.data
    }

    /// Light or dark chrome.
    pub fn set_dark(&mut self, dark: bool, cx: &mut Context<Self>) {
        self.dark = dark;
        let t = self.theme();
        self.input.update(cx, |i, cx| i.set_theme(t, cx));
        self.aux.update(cx, |i, cx| i.set_theme(t, cx));
        cx.notify();
    }

    /// Inset or Flat main area.
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

    /// Applies a press, keeping the composer field in step with the draft.
    fn apply(&mut self, id: &'static str, cx: &mut Context<Self>) {
        match id {
            "menu-theme" => {
                self.ui.menu = false;
                self.set_dark(!self.dark, cx);
                return;
            }
            "menu-out" => {
                self.ui.menu = false;
                self.ui.last = Some(id);
                cx.notify();
                return;
            }
            _ => {}
        }
        let had = self.ui.draft.clone();
        let st = self.ui.st;
        let view = self.ui.view;
        self.app.press(&mut self.ui, &mut self.own, id);
        if self.ui.view != view || id.starts_with("page-") {
            let t = if self.ui.view_name() == "browser" {
                oxyn_core::parity::cmdstudio::PAGES[self.own.data.page].0
            } else {
                ""
            };
            self.aux.update(cx, |i, cx| i.set_text(t, cx));
        }
        if self.ui.st != st {
            let rows = self.app.rows(self.ui.st, self.own.cloud);
            self.ui.rows = rows;
        }
        if self.ui.draft != had || self.ui.draft.is_empty() {
            let d = self.ui.draft.clone();
            self.input.update(cx, |i, cx| {
                if i.text() != d {
                    i.set_text(&d, cx)
                }
            });
        }
        cx.notify();
    }

    fn key(&mut self, ev: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let Some(k) = AppKey::from_name(ev.keystroke.key.as_str()) else {
            return;
        };
        let typing = self.input.read(cx).focus().is_focused(window)
            || self.aux.read(cx).focus().is_focused(window);
        if typing {
            if k == AppKey::Escape && (self.ui.menu || self.ui.rail) {
                self.ui.key(k);
                cx.notify();
            } else if k == AppKey::Escape {
                window.focus(&self.focus, cx);
            }
            return;
        }
        let view = self.ui.view;
        if k == AppKey::Enter {
            return;
        }
        // Session Studio, Rearrange: Up / Down pick a pane, Alt + Up / Down move it.
        if self.app == AgentApp::SessionStudio
            && self.ui.view_name() == "arrange"
            && matches!(k, AppKey::Up | AppKey::Down)
        {
            let up = k == AppKey::Up;
            let d = &mut self.own.data;
            let changed = if ev.keystroke.modifiers.alt {
                d.move_pick(up)
            } else if up && d.pane > 0 {
                d.pane -= 1;
                true
            } else if !up && d.pane + 1 < d.panes.len() {
                d.pane += 1;
                true
            } else {
                false
            };
            if changed {
                cx.notify();
            }
            return;
        }
        // Command Board: Alt + Left / Right scrub the timeline.
        if self.app == AgentApp::CommandBoard && ev.keystroke.modifiers.alt {
            let back = k == AppKey::Left;
            if matches!(k, AppKey::Left | AppKey::Right) && self.own.data.scrub_step(back) {
                cx.notify();
            }
            return;
        }
        if self.ui.key(k) {
            if self.app == AgentApp::Taskdeck && self.ui.view != view {
                self.own.approval = false;
            }
            if self.app == AgentApp::Taskdeck && matches!(k, AppKey::Up | AppKey::Down) {
                self.ui.open_view("task");
            }
            if self.app == AgentApp::WorkspaceCoder && self.ui.view != view {
                self.own.data.ws_tab = if self.ui.view_name() == "preview" {
                    3
                } else {
                    0
                };
            }
            cx.notify();
        }
    }

    fn mover(&self, cx: &mut Context<Self>) -> Mover {
        let me = cx.entity().downgrade();
        Rc::new(move |from, to, _, cx| {
            let _ = me.update(cx, |v, cx| {
                if oxyn_core::parity::cmdstudio::move_pane(&mut v.own.data.panes, from, to) {
                    v.own.data.pane = to.min(v.own.data.panes.len() - 1);
                    cx.notify();
                }
            });
        })
    }

    fn act(&self, cx: &mut Context<Self>) -> Act {
        let me = cx.entity().downgrade();
        Rc::new(move |id, w, cx| {
            let _ = me.update(cx, |v, cx| {
                if id == "focus-composer" {
                    let f = v.input.read(cx).focus().clone();
                    w.focus(&f, cx);
                } else {
                    v.apply(id, cx)
                }
            });
        })
    }

    fn menu(&self, p: &ThPal, act: &Option<Act>) -> Div {
        let item = |l: &'static str, id: &'static str| {
            hit(div(), act, id)
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
                    .child(oxyn_core::parity::cmdstudio::PERSON),
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

impl Render for AgentAppView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = self.theme();
        let p = th_pal(theme);
        let act = Some(self.act(cx));
        let lv = Live {
            sel: Some(self.ui.sel),
            rail: self.ui.rail,
            input: Some(self.input.clone().into()),
            sent: self.ui.sent.iter().map(|s| s.clone().into()).collect(),
            data: Some(self.own.data.clone()),
            aux: Some(self.aux.clone().into()),
            mover: Some(self.mover(cx)),
        };
        let st = self.ui.st;
        let extra = self.app.extra(&self.ui, &self.own);
        let lay = self.layout;
        let screen = match self.app {
            AgentApp::Taskdeck => {
                super::taskdeck::screen(theme, lay, st, extra, &act, self.ui.view_name(), &lv)
            }
            AgentApp::CommandCenter => {
                super::cmdstudio::screen(theme, lay, st, extra, &act, "cc", &lv)
            }
            AgentApp::SessionStudio => {
                super::cmdstudio::screen(theme, lay, st, extra, &act, "ss", &lv)
            }
            AgentApp::FleetWindow => super::fleetwin::fw_screen(&p, lay, st, extra, &act, &lv),
            AgentApp::WorkspaceCoder => super::wscoder::wc_screen(&p, lay, st, extra, &act, &lv),
            AgentApp::CommandBoard => super::cmdboard::cb_screen(&p, lay, st, extra, &act, &lv),
        };
        let menu = self.ui.menu.then(|| self.menu(&p, &act));
        div()
            .id("agent-app")
            .key_context("OxynAgentApp")
            .track_focus(&self.focus)
            .on_key_down(cx.listener(|this, ev: &KeyDownEvent, w, cx| this.key(ev, w, cx)))
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(|this, _, w, cx| {
                    if !this.input.read(cx).focus().is_focused(w)
                        && !this.aux.read(cx).focus().is_focused(w)
                    {
                        w.focus(&this.focus, cx);
                    }
                }),
            )
            .relative()
            .size_full()
            .child(screen)
            .children(menu)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn taskdeck_presses() {
        let a = AgentApp::Taskdeck;
        let mut ui = AppUi::new(a.rows(ScreenState::Sample, false), a.views());
        let mut own = Own::default();
        assert!(a.press(&mut ui, &mut own, "tasks"));
        assert_eq!(ui.view_name(), "board");
        assert!(a.press(&mut ui, &mut own, "open-3"));
        assert_eq!((ui.view_name(), own.approval), ("task", false));
        assert!(a.press(&mut ui, &mut own, "row-0"));
        assert_eq!(a.extra(&ui, &own), "approval");
        assert!(a.press(&mut ui, &mut own, "allow"));
        assert_eq!(a.extra(&ui, &own), "");
        assert!(a.press(&mut ui, &mut own, "env-cloud"));
        a.press(&mut ui, &mut own, "sites");
        assert_eq!(a.extra(&ui, &own), "cloud");
        assert_eq!(ui.rows, a.rows(ScreenState::Sample, true));
    }

    #[test]
    fn s10_s11_presses() {
        let a = AgentApp::CommandCenter;
        let mut ui = AppUi::new(3, a.views());
        let mut own = Own::default();
        a.press(&mut ui, &mut own, "place");
        assert_eq!(a.extra(&ui, &own), "worktree");
        a.press(&mut ui, &mut own, "handoff");
        assert_eq!(a.extra(&ui, &own), "");
        let b = AgentApp::CommandBoard;
        let mut ui = AppUi::new(3, b.views());
        b.press(&mut ui, &mut own, "open-pr");
        assert_eq!(b.extra(&ui, &own), "pr");
        b.press(&mut ui, &mut own, "view:planner");
        assert_eq!(b.extra(&ui, &own), "planner");
        b.press(&mut ui, &mut own, "view:");
        assert_eq!(b.extra(&ui, &own), "");
        let f = AgentApp::FleetWindow;
        let mut ui = AppUi::new(3, f.views());
        ui.key(AppKey::Right);
        assert_eq!(f.extra(&ui, &own), "compare");
        f.press(&mut ui, &mut own, "apply");
        assert_eq!((f.extra(&ui, &own), ui.last), ("", Some("apply")));
        for app in AgentApp::ALL {
            assert!(app.rows(ScreenState::Sample, false) > 0, "{}", app.name());
        }
    }
}
