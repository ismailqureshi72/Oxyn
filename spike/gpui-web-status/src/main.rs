//! Spike: the same GPUI StatusIndicator, compiled for the browser (gpui_web, single-threaded).
use std::{borrow::Cow, cell::RefCell, time::Duration};

use gpui::{
    App, ApplicationHandle, Bounds, Context, Hsla, Window, WindowBounds, WindowOptions, div,
    prelude::*, px, rgb, size,
};
use oxyn_core_spike::{AgentState, Run, Tone, demo_script};
use web_time::Instant;

thread_local! {
    static APPLICATION: RefCell<Option<ApplicationHandle>> = const { RefCell::new(None) };
}

fn tone(t: Tone) -> u32 {
    match t {
        Tone::Neutral => 0x8a8a8f,
        Tone::Active => 0xff5a1f,
        Tone::Attention => 0xf2a33a,
        Tone::Success => 0x3fb37f,
        Tone::Warning => 0xe0b341,
        Tone::Danger => 0xef5a4c,
    }
}

struct Demo { started: Instant }

impl Demo {
    fn state(&self) -> (AgentState, u64) {
        let now = self.started.elapsed().as_millis() as u64 % 13_000;
        let mut run = Run::default();
        for (t, e) in demo_script() { if t <= now { run.apply(e, t); } }
        (run.state_at(now), now)
    }
}

impl Render for Demo {
    fn render(&mut self, _w: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let (state, now) = self.state();
        let phase = (now % 1000) as f32 / 1000.0;
        let mut dot: Hsla = rgb(tone(state.tone())).into();
        if state.is_busy() { dot.a = 0.725 + 0.275 * (phase * std::f32::consts::TAU).cos(); }
        div()
            .size_full().flex().items_center().justify_center().bg(rgb(0x0b0b0c))
            .font_family("Inter")
            .child(
                div()
                    .flex().items_center().gap_2().px_3().py_1p5().rounded_full()
                    .bg(rgb(0x151517)).border_1().border_color(rgb(0x26262a))
                    .text_size(px(13.)).text_color(rgb(0xededef))
                    .child(div().size(px(8.)).rounded_full().bg(dot))
                    .child(state.label()),
            )
    }
}

fn main() {
    gpui_platform::web_init();
    let handle = gpui_platform::single_threaded_web().run_embedded(|cx: &mut App| {
        if let Err(e) = cx.text_system().add_fonts(vec![Cow::Borrowed(include_bytes!("../Inter-Medium.ttf"))]) {
            web_sys::console::error_1(&format!("font: {e:#}").into());
            return;
        }
        let bounds = Bounds::centered(None, size(px(480.), px(160.)), cx);
        cx.open_window(
            WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
            |_, cx| cx.new(|cx| {
                cx.spawn(async move |this, cx| loop {
                    cx.background_executor().timer(Duration::from_millis(16)).await;
                    if this.update(cx, |_, cx| cx.notify()).is_err() { break; }
                }).detach();
                Demo { started: Instant::now() }
            }),
        ).expect("open window");
        cx.activate(true);
    });
    APPLICATION.with(|c| *c.borrow_mut() = Some(handle));
}
