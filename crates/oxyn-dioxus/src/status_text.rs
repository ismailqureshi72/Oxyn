//! Status-text components (family C): `TextShimmer`, `VerbCycler`, `ElapsedTimer`, `TextReveal`.
//! Each holds the shared `oxyn-core` engine and draws its output; the host re-renders with the
//! shared clock's `now`.

use std::{cell::RefCell, rc::Rc, time::Duration};

use dioxus::prelude::*;
use oxyn_core::{
    AgentState, ClockMs, VerbPack,
    text::{
        AsciiTier, CaretVariant, CyclerVariant, Reveal, RevealVariant, RunMeter, Shimmer,
        ShimmerPaint, ShimmerVariant, TimerVariant, VerbCycler as CyclerEngine, graphemes,
        timer_view,
    },
};
use oxyn_tokens::ColorRole;

use crate::color_var;

/// Stylesheet for the status-text components.
pub const STATUS_TEXT_CSS: &str = r#"
.oxyn-sh{font-family:var(--oxyn-typography-ui-font-family,system-ui);white-space:pre}
.oxyn-sh.band{-webkit-background-clip:text;background-clip:text;color:transparent}
.oxyn-vc{display:inline-grid;vertical-align:bottom;overflow:hidden;white-space:pre}
.oxyn-vc>span{grid-area:1/1;will-change:opacity,transform,filter}
.oxyn-vc .dots>span{display:inline-block}
.oxyn-et{font-variant-numeric:tabular-nums;white-space:pre;display:inline-block;text-align:left}
.oxyn-et.paused{color:var(--oxyn-color-fg-muted)}
.oxyn-tr{white-space:pre-wrap}
.oxyn-tr>span{position:relative}
.oxyn-caret{display:inline-block;vertical-align:text-bottom;background:currentColor}
.oxyn-caret.bar{width:2px;height:1.1em;border-radius:1px}
.oxyn-caret.block{width:.55em;height:1.1em;opacity:.7}
.oxyn-caret.underscore{width:.55em;height:2px;vertical-align:baseline}
"#;

fn mix(from: &str, to: &str, t: f32) -> String {
    if t <= 0.0 {
        from.to_string()
    } else if t >= 1.0 {
        to.to_string()
    } else {
        format!("color-mix(in oklab, {from}, {to} {:.0}%)", t * 100.0)
    }
}

/// Shimmering status text.
#[component]
pub fn TextShimmer(
    /// The text.
    text: String,
    /// The agent state (drives start, stop, stall).
    state: AgentState,
    /// Shared clock time.
    now: ClockMs,
    /// `sweep` · `calm` · `pulse` · `ascii`.
    #[props(default = "sweep".to_string())]
    variant: String,
    /// Measured line width in px; estimated from the text when absent.
    #[props(default)]
    line_px: Option<f32>,
    /// Font size in px.
    #[props(default = 13.0)]
    size: f32,
    /// Reduced motion: plain text at full fg.
    #[props(default)]
    reduced: bool,
) -> Element {
    let v = ShimmerVariant::from_id(&variant).unwrap_or_default();
    let engine = use_hook(|| Rc::new(RefCell::new(Shimmer::new(v))));
    if engine.borrow().variant() != v {
        *engine.borrow_mut() = Shimmer::new(v);
    }
    let n = graphemes(&text).count();
    let line = line_px.unwrap_or(n as f32 * size * 0.55);
    engine.borrow_mut().set_line_px(line);
    engine.borrow_mut().update(&text, &state, now);
    let e = engine.borrow();
    let fg = color_var(ColorRole::TextStatusFg);
    let base = mix(
        &color_var(ColorRole::TextStatusBase),
        &fg,
        e.base_level(now, reduced),
    );
    let base = mix(
        &base,
        &color_var(ColorRole::StatusWarningFg),
        e.warn_level(now),
    );
    let crest = color_var(ColorRole::TextStatusCrest);
    let busy = e.busy().to_string();
    let paint = e.paint(now, size, reduced);
    if v == ShimmerVariant::Ascii {
        let cells: Vec<(String, AsciiTier)> = graphemes(&text)
            .enumerate()
            .map(|(i, g)| {
                (
                    g.to_string(),
                    AsciiTier::of(paint.at((i as f32 + 0.5) * size * 0.55)),
                )
            })
            .collect();
        return rsx! {
            span { class: "oxyn-sh", "aria-busy": "{busy}", style: "font-size:{size}px;color:{fg}",
                for (g, tier) in cells {
                    span { style: match tier {
                        AsciiTier::Dim => "opacity:.55",
                        AsciiTier::Normal => "opacity:.8",
                        AsciiTier::Bold => "font-weight:700",
                    }, "{g}" }
                }
            }
        };
    }
    let (class, style) = match paint {
        ShimmerPaint::None => ("oxyn-sh", format!("color:{base}")),
        ShimmerPaint::Uniform(i) => ("oxyn-sh", format!("color:{}", mix(&base, &crest, i))),
        ShimmerPaint::Band {
            centre,
            half_width: hw,
            amplitude: a,
        } => {
            let half = mix(&base, &crest, 0.5 * a);
            let top = mix(&base, &crest, a);
            (
                "oxyn-sh band",
                format!(
                    "background-image:linear-gradient(90deg,{base} {:.1}px,{half} {:.1}px,{top} {centre:.1}px,{half} {:.1}px,{base} {:.1}px)",
                    centre - hw,
                    centre - hw / 2.0,
                    centre + hw / 2.0,
                    centre + hw
                ),
            )
        }
    };
    rsx! {
        span { class: "{class}", "aria-busy": "{busy}", style: "font-size:{size}px;{style}", "{text}" }
    }
}

/// Swaps status verbs from a pack, pins tool verbs, settles to past tense.
#[component]
pub fn VerbCycler(
    /// The agent state.
    state: AgentState,
    /// Shared clock time.
    now: ClockMs,
    /// `fade` · `slide` · `blur-swap` · `ellipsis` · `static`.
    #[props(default = "fade".to_string())]
    variant: String,
    /// Built-in pack id.
    #[props(default = "neutral".to_string())]
    pack: String,
    /// Shuffle seed (hash of the run id).
    #[props(default)]
    seed: u64,
    /// Font size in px.
    #[props(default = 13.0)]
    size: f32,
    /// Reduced motion: 80 ms crossfades, dwell ×2.
    #[props(default)]
    reduced: bool,
) -> Element {
    let v = CyclerVariant::from_id(&variant).unwrap_or_default();
    let p = VerbPack::builtin(&pack).unwrap_or_else(VerbPack::neutral);
    let engine = use_hook(|| Rc::new(RefCell::new(CyclerEngine::new(v, p.clone(), seed))));
    if engine.borrow().variant() != v || engine.borrow().pack().id != p.id {
        *engine.borrow_mut() = CyclerEngine::new(v, p.clone(), seed);
    }
    engine.borrow_mut().update(&state, now, reduced);
    let Some(view) = engine.borrow().view(now, reduced) else {
        return rsx! {};
    };
    let fx = |t: f32, out: bool| -> String {
        let k = if out { t } else { 1.0 - t };
        let op = if out { 1.0 - t } else { t };
        match (v, reduced) {
            (_, true) | (CyclerVariant::Static, _) => format!("opacity:{op:.3}"),
            (CyclerVariant::Slide, _) => {
                let y = if out { -0.6 * k } else { 0.6 * k };
                format!("opacity:{op:.3};transform:translateY({y:.3}em)")
            }
            (CyclerVariant::BlurSwap, _) => {
                format!("opacity:{op:.3};filter:blur({:.2}px)", 6.0 * k)
            }
            _ => {
                let y = if out { -4.0 * k } else { 4.0 * k };
                format!("opacity:{op:.3};transform:translateY({y:.2}px)")
            }
        }
    };
    let in_style = fx(view.in_t, false);
    let out = view.outgoing.clone().map(|o| (o, fx(view.out_t, true)));
    let dots = view.dots;
    let show_dots = v == CyclerVariant::Ellipsis && !view.settled && !view.paused;
    rsx! {
        span { class: "oxyn-vc", style: "font-size:{size}px", "data-settled": "{view.settled}",
            if let Some((o, s)) = out {
                span { "aria-hidden": "true", style: "{s}", "{o}" }
            }
            span { style: "{in_style}",
                "{view.text}"
                if show_dots {
                    span { class: "dots", "aria-hidden": "true",
                        for d in dots { span { style: "opacity:{d:.2}", "." } }
                    }
                }
            }
        }
    }
}

/// Run timer: live digits, paused while waiting, summary when done.
#[component]
pub fn ElapsedTimer(
    /// The agent state.
    state: AgentState,
    /// Working time from the run clock (`AgentMachine::elapsed`).
    elapsed: Duration,
    /// `units` · `clock` · `summary` · `meter`.
    #[props(default = "units".to_string())]
    variant: String,
    /// Built-in pack id for the summary phrase.
    #[props(default = "neutral".to_string())]
    pack: String,
    /// Counters for the `meter` variant.
    #[props(default)]
    meter: RunMeter,
    /// Font size in px.
    #[props(default = 13.0)]
    size: f32,
) -> Element {
    let v = TimerVariant::from_id(&variant).unwrap_or_default();
    let p = VerbPack::builtin(&pack).unwrap_or_else(VerbPack::neutral);
    let view = timer_view(v, &state, elapsed, &meter, p);
    if view.text.is_empty() {
        return rsx! {};
    }
    let class = if view.paused {
        "oxyn-et paused"
    } else {
        "oxyn-et"
    };
    let reserve = view.reserve.chars().count();
    rsx! {
        span { class: "{class}", style: "font-size:{size}px;min-width:{reserve}ch",
            "{view.text}"
            if view.paused { " ⏸" }
        }
    }
}

/// Streaming answer text with a pacer, an effect and a caret.
#[component]
pub fn TextReveal(
    /// Everything received so far (append-only).
    text: String,
    /// Shared clock time.
    now: ClockMs,
    /// The stream ended normally.
    #[props(default)]
    ended: bool,
    /// The user stopped the stream.
    #[props(default)]
    interrupted: bool,
    /// The stream is stalled.
    #[props(default)]
    stalled: bool,
    /// `fade-token` · `word-blur` · `typewriter` · `line-commit` · `instant`.
    #[props(default = "fade-token".to_string())]
    variant: String,
    /// `bar` · `block` · `underscore`, or empty for no caret.
    #[props(default = "bar".to_string())]
    caret: String,
    /// Reduced motion.
    #[props(default)]
    reduced: bool,
) -> Element {
    let v = RevealVariant::from_id(&variant).unwrap_or_default();
    let engine = use_hook(|| Rc::new(RefCell::new(Reveal::new(v))));
    {
        let mut e = engine.borrow_mut();
        let have = e.received();
        if e.variant() != v || !text.starts_with(&have) {
            *e = Reveal::new(v);
        }
        let have = e.received();
        if text.len() > have.len() {
            e.push(&text[have.len()..], now);
        }
        if interrupted {
            e.interrupt(now);
        } else if ended {
            e.finish(now);
        }
        e.set_stalled(stalled);
        e.tick(now);
    }
    let e = engine.borrow();
    let spans = e.spans(now, reduced);
    let c = e.caret(now, reduced);
    let caret_class = CaretVariant::from_id(&caret).map(|cv| match cv {
        CaretVariant::Bar => "oxyn-caret bar",
        CaretVariant::Block => "oxyn-caret block",
        CaretVariant::Underscore => "oxyn-caret underscore",
    });
    let caret_op = if c.dim { 0.45 } else { c.opacity };
    rsx! {
        span { class: "oxyn-tr",
            for s in spans {
                if s.opacity >= 1.0 && s.blur_px == 0.0 && s.dy_px == 0.0 {
                    "{s.text}"
                } else {
                    span {
                        style: "opacity:{s.opacity:.3};filter:blur({s.blur_px:.2}px);top:{s.dy_px:.2}px",
                        "{s.text}"
                    }
                }
            }
            if let Some(cls) = caret_class.filter(|_| c.opacity > 0.0) {
                span { class: "{cls}", "aria-hidden": "true", style: "opacity:{caret_op:.2}" }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(e: Element) -> String {
        dioxus_ssr::render_element(e)
    }

    #[test]
    fn shimmer_band_and_reduced() {
        let th = AgentState::Thinking { header: None };
        let h = render(
            rsx! { TextShimmer { text: "Thinking".to_string(), state: th.clone(), now: ClockMs(0) } },
        );
        assert!(
            h.contains("aria-busy=\"true\"") && h.contains("Thinking"),
            "{h}"
        );
        let h = render(
            rsx! { TextShimmer { text: "Thinking".to_string(), state: th, now: ClockMs(0), reduced: true } },
        );
        assert!(
            h.contains("text-status-fg") && !h.contains("linear-gradient"),
            "{h}"
        );
    }

    #[test]
    fn cycler_and_timer() {
        let th = AgentState::Thinking { header: None };
        let h = render(
            rsx! { VerbCycler { state: th.clone(), now: ClockMs(0), variant: "ellipsis".to_string() } },
        );
        assert!(h.contains("Thinking") && h.contains("dots"), "{h}");
        let h = render(
            rsx! { ElapsedTimer { state: th, elapsed: Duration::from_secs(65), variant: "clock".to_string() } },
        );
        assert!(h.contains("1:05") && h.contains("min-width:4ch"), "{h}");
    }

    #[test]
    fn reveal_with_caret() {
        let h = render(
            rsx! { TextReveal { text: "Hello world".to_string(), now: ClockMs(0), interrupted: true } },
        );
        assert!(
            h.contains("Hello world") && !h.contains("oxyn-caret"),
            "{h}"
        );
        let h = render(rsx! { TextReveal { text: String::new(), now: ClockMs(0) } });
        assert!(h.contains("oxyn-caret bar"), "{h}");
    }
}
