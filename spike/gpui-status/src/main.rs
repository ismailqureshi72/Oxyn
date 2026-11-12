//! Spike: GPUI StatusIndicator driven by oxyn-core-spike's demo script.
//! `OXYN_POSTER_MS=<t>` freezes the script at t ms so a poster can be captured.
use std::time::{Duration, Instant};

use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds, Context, Hsla, Window,
    WindowBounds, WindowOptions,
};
use oxyn_core_spike::{demo_script, AgentState, Run, Tone};

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

struct Demo { started: Instant, frozen_at: Option<u64> }

impl Demo {
    fn state(&self) -> (AgentState, u64) {
        let now = self.frozen_at.unwrap_or_else(|| self.started.elapsed().as_millis() as u64 % 13_000);
        let mut run = Run::default();
        for (t, e) in demo_script() { if t <= now { run.apply(e, t); } }
        (run.state_at(now), now)
    }
}

impl Render for Demo {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        let (state, now) = self.state();
        // 1 Hz pulse (loop.base = 1000 ms) on busy states only.
        let phase = (now % 1000) as f32 / 1000.0;
        let alpha = if state.is_busy() && self.frozen_at.is_none() {
            0.725 + 0.275 * (phase * std::f32::consts::TAU).cos()
        } else { 1.0 };
        let mut dot: Hsla = rgb(tone(state.tone())).into();
        dot.a = alpha;
        div()
            .size_full().flex().items_center().justify_center().bg(rgb(0x0b0b0c))
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
    env_logger::init();
    let frozen_at = std::env::var("OXYN_POSTER_MS").ok().and_then(|v| v.parse().ok());
    Application::new().run(move |cx: &mut App| {
        let bounds = Bounds::centered(None, size(px(480.), px(160.)), cx);
        cx.open_window(
            WindowOptions { window_bounds: Some(WindowBounds::Windowed(bounds)), ..Default::default() },
            move |_, cx| {
                cx.new(|cx| {
                    if frozen_at.is_none() {
                        cx.spawn(async move |this, cx| loop {
                            cx.background_executor().timer(Duration::from_millis(16)).await;
                            if this.update(cx, |_, cx| cx.notify()).is_err() { break; }
                        })
                        .detach();
                    }
                    Demo { started: Instant::now(), frozen_at }
                })
            },
        )
        .unwrap();
        cx.activate(true);
    });
}
