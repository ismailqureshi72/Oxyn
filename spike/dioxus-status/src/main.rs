//! Spike: Dioxus web StatusIndicator driven by oxyn-core-spike's demo script.
use dioxus::prelude::*;
use gloo_timers::future::TimeoutFuture;
use oxyn_core_spike::{demo_script, AgentState, Run, Tone};

fn main() { dioxus::launch(App); }

fn tone_var(t: Tone) -> &'static str {
    match t {
        Tone::Neutral => "var(--oxyn-color-fg-muted, #8a8a8f)",
        Tone::Active => "var(--oxyn-color-accent, #ff5a1f)",
        Tone::Attention => "var(--oxyn-color-risk-high-fg, #f2a33a)",
        Tone::Success => "var(--oxyn-color-success, #3fb37f)",
        Tone::Warning => "var(--oxyn-color-warning, #e0b341)",
        Tone::Danger => "var(--oxyn-color-danger, #ef5a4c)",
    }
}

#[component]
fn App() -> Element {
    let mut state = use_signal(|| AgentState::Idle);
    use_future(move || async move {
        loop {
            let t0 = js_sys::Date::now() as u64;
            let mut run = Run::default();
            let mut script = demo_script().into_iter().peekable();
            loop {
                let now = js_sys::Date::now() as u64 - t0;
                while let Some((t, _)) = script.peek() {
                    if *t > now { break; }
                    let (t, e) = script.next().unwrap();
                    run.apply(e, t);
                }
                state.set(run.state_at(now));
                if script.peek().is_none() && now > 13_000 { break; }
                TimeoutFuture::new(100).await;
            }
        }
    });
    rsx! { StatusIndicator { state: state() } }
}

#[component]
fn StatusIndicator(state: AgentState) -> Element {
    let colour = tone_var(state.tone());
    let pulse = if state.is_busy() { "oxyn-pulse 1000ms ease-in-out infinite" } else { "none" };
    rsx! {
        style { "@keyframes oxyn-pulse {{ 0%,100% {{ opacity:1 }} 50% {{ opacity:.45 }} }}
                 @media (prefers-reduced-motion: reduce) {{ .oxyn-dot {{ animation:none !important }} }}" }
        div {
            role: "status",
            "aria-live": "polite",
            style: "display:inline-flex;align-items:center;gap:8px;padding:6px 12px;border-radius:999px;
                    font:500 13px/1 Inter,system-ui,sans-serif;color:#ededef;background:#151517;
                    border:1px solid #26262a;",
            span {
                class: "oxyn-dot",
                width: "8px", height: "8px", border_radius: "50%", background_color: "{colour}", animation: "{pulse}",
            }
            span { "{state.label()}" }
        }
    }
}
