//! The six agent apps as interactive GPUI views in one window: the strip at the top picks the
//! app (Taskdeck, Command Center, Session Studio, Fleet Window, Workspace Coder, Command Board).
//! Inside an app, press rows, nav, tabs and the account row, type in the composer and send with
//! Enter; with focus on the app, Up / Down move the open row, Left / Right step through its views
//! and Escape closes the menu, then the rail.
//!
//! Run from the repo root: `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml`

use gpui::{
    App, Bounds, Context, Entity, Focusable, Render, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use gpui_platform::application;
use oxyn_gpui::{AgentApp, AgentAppView, bind_ime_keys};

struct Gallery {
    apps: Vec<Entity<AgentAppView>>,
    open: usize,
}

impl Gallery {
    fn show(&mut self, i: usize, window: &mut Window, cx: &mut Context<Self>) {
        self.open = i;
        let f = self.apps[i].read(cx).focus_handle(cx);
        window.focus(&f, cx);
        cx.notify();
    }
}

impl Render for Gallery {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let mut bar = div()
            .flex()
            .gap(px(4.))
            .p(px(6.))
            .bg(rgb(0x141316))
            .text_color(rgb(0xF4F4F5))
            .text_size(px(12.5));
        for (i, app) in AgentApp::ALL.iter().enumerate() {
            bar = bar.child(
                div()
                    .id(i)
                    .px(px(10.))
                    .py(px(5.))
                    .rounded(px(7.))
                    .cursor_pointer()
                    .when(i == self.open, |d| d.bg(rgb(0x2A2A2E)))
                    .child(app.name())
                    .on_click(cx.listener(move |this, _, w, cx| this.show(i, w, cx))),
            );
        }
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(bar)
            .child(div().flex_1().min_h_0().child(self.apps[self.open].clone()))
    }
}

fn main() {
    application().run(|cx: &mut App| {
        bind_ime_keys(cx);
        let bounds = Bounds::centered(None, size(px(1440.), px(940.)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |window, cx| {
                let apps: Vec<Entity<AgentAppView>> = AgentApp::ALL
                    .iter()
                    .map(|a| cx.new(|cx| AgentAppView::new(*a, window, cx)))
                    .collect();
                let first = apps[0].read(cx).focus_handle(cx);
                window.focus(&first, cx);
                cx.new(|_| Gallery { apps, open: 0 })
            },
        )
        .expect("open the agent apps window");
        cx.activate(true);
    });
}
