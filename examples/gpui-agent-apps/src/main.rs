//! Every Oxyn screen set as interactive GPUI views in one native window. The strip at the top
//! picks a screen set (S0 app shells … S8 onboarding, S12 to S15 Arc screens) or one of the six agent
//! apps (Taskdeck, Command Center, Session Studio, Fleet Window, Workspace Coder, Command Board).
//!
//! Screen sets: press the sidebar, buttons and the account row as in any app; the bar above the
//! screen has the set's screens, the state bar and a command bar (type a screen or a state, then
//! Enter). With focus on the screen, Left / Right step the states, Up / Down the screens, Escape
//! closes the menu and then returns to Data, `/` jumps to the command bar.
//!
//! Agent apps: press rows, nav, tabs, switches and the account row, type in the composer and send
//! with Enter; Up / Down move the open row, Left / Right step through the app's views, Escape
//! closes the menu, then the rail.
//!
//! Run from the repo root: `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml`.
//! `OXYN_OPEN=<n>` opens strip item `n` first (0 is S0, 9 is S12 Arc screens I, 10 is S13 Arc
//! screens II, 11 is S14 Arc screens III, 12 is S15 Arc screens IV).
//!
//! Arc screens I: press a board card, then a column (or drag the card there), Alt + arrows move
//! the picked card; press a timeline bar, then Earlier / Later (Alt + ← / →) or drag it onto a
//! day; type in the directory's search; switch the metrics range, KPI and Compare.
//!
//! Arc screens II: press a calendar event, then a day header or the nudge buttons (Alt + arrows),
//! or drag it onto a day; J / K / E / H / X / U / A triage the inbox; type a support reply and
//! press Enter; search the help center.
//!
//! Arc screens III: pick a symbol and range, step the ticket and place it; filter the products,
//! open one, pick a size and add it; add, step and remove invoice lines, then Send; fill the
//! demo account and sign in.
//!
//! Arc screens IV: range and KPI tabs, audit filters, status days, create, roll and revoke keys,
//! invite and change roles, pick a plan; the agent's tools: modes, Approve / Reject, Run again.

use gpui::{
    App, Bounds, Context, Entity, Focusable, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_platform::application;
use oxyn_gpui::{AgentApp, AgentAppView, ScreenView, bind_ime_keys};

/// Strip labels for the screen sets, in `oxyn_core::parity::screenui::SETS` order.
const SETS: [&str; 13] = [
    "S0 Shells",
    "S1 Chat",
    "S2 Workspace",
    "S3 Ops I",
    "S4 Ops II",
    "S5 Build",
    "S6 Settings I",
    "S7 Settings II",
    "S8 Onboarding",
    "S12 Arc I",
    "S13 Arc II",
    "S14 Arc III",
    "S15 Arc IV",
];

struct Gallery {
    screens: Entity<ScreenView>,
    apps: Vec<Entity<AgentAppView>>,
    /// The open strip item: a set below `SETS.len()`, then the apps.
    open: usize,
    /// Redraws the strip when the screen view moves to another set (a sidebar press or the
    /// command bar can open a screen of another set).
    _watch: gpui::Subscription,
}

impl Gallery {
    fn show(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.open = i;
        let f = if i < SETS.len() {
            self.screens.update(cx, |v, cx| v.open_set(i, cx));
            self.screens.read(cx).focus_handle(cx)
        } else {
            self.apps[i - SETS.len()].read(cx).focus_handle(cx)
        };
        window.focus(&f, cx);
        cx.notify();
    }
}

impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // The highlighted item follows the set the screen view shows, not only the last press.
        let cur = if self.open < SETS.len() {
            self.screens.read(cx).ui().set.min(SETS.len() - 1)
        } else {
            self.open
        };
        let mut bar = div()
            .flex()
            .flex_wrap()
            .gap(px(3.))
            .p(px(5.))
            .bg(rgb(0x141316))
            .text_color(rgb(0xF4F4F5))
            .text_size(px(12.));
        let names = SETS
            .iter()
            .copied()
            .chain(AgentApp::ALL.iter().map(|a| a.name()));
        for (i, name) in names.enumerate() {
            bar = bar.child(
                div()
                    .id(i)
                    .px(px(8.))
                    .py(px(4.))
                    .rounded(px(7.))
                    .cursor_pointer()
                    .when(i == SETS.len(), |d| d.ml(px(10.)))
                    .when(i == cur, |d| d.bg(rgb(0x2A2A2E)))
                    .child(name)
                    .on_click(cx.listener(move |this, _, w, cx| this.show(i, w, cx))),
            );
        }
        let body = if self.open < SETS.len() {
            self.screens.clone().into_any_element()
        } else {
            self.apps[self.open - SETS.len()].clone().into_any_element()
        };
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(bar)
            .child(div().flex_1().min_h_0().child(body))
    }
}

fn main() {
    let first: usize = std::env::var("OXYN_OPEN")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(0)
        .min(SETS.len() + AgentApp::ALL.len() - 1);
    application().run(move |cx: &mut App| {
        bind_ime_keys(cx);
        let bounds = Bounds::centered(None, size(px(1440.), px(940.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let screens = cx.new(|cx| ScreenView::new(first.min(SETS.len() - 1), window, cx));
                let apps: Vec<Entity<AgentAppView>> = AgentApp::ALL
                    .iter()
                    .map(|a| cx.new(|cx| AgentAppView::new(*a, window, cx)))
                    .collect();
                let f = if first < SETS.len() {
                    screens.read(cx).focus_handle(cx)
                } else {
                    apps[first - SETS.len()].read(cx).focus_handle(cx)
                };
                window.focus(&f, cx);
                cx.new(|cx| {
                    let _watch = cx.observe(&screens, |_, _, cx| cx.notify());
                    Gallery {
                        screens,
                        apps,
                        open: first,
                        _watch,
                    }
                })
            },
        )
        .expect("open the Oxyn screens window");
        cx.activate(true);
    });
}
