//! Image reveal (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::reveal::{GRID, Phase, Reveal, SCENES, latent, scene};

use super::{ico, live};

fn scene_svg(i: usize, class: &'static str, style: String) -> Element {
    rsx! {
        svg { class: "{class}", view_box: "0 0 100 100", preserve_aspect_ratio: "xMidYMid slice", style: "{style}", "aria-hidden": "true",
            for (k, (c, pts)) in scene(i).into_iter().enumerate() {
                {
                    let d = pts.iter().enumerate().map(|(j, p)| format!("{}{:.1} {:.1}", if j == 0 { "M" } else { "L" }, p.0, p.1)).collect::<String>() + "Z";
                    rsx! { path { key: "{k}", d: "{d}", fill: "#{c:06x}" } }
                }
            }
        }
    }
}

/// Image reveal: a picture coming into being like a diffusion sampler. A coarse block grid in the
/// picture's colours sheds its noise in visible steps over a soft guess that slowly focuses; then a
/// soft-edged sweep from the top lays down the sharp result with one glint. The frosted badge shows
/// a ring and `18 / 30` (a progress bar reading Step n of total), then the time taken. Waiting is a
/// quiet shimmer; a failure dims the partial picture and offers Retry. The canvas is a named image
/// (its description once done).
#[component]
pub fn ImageReveal(
    /// Host clock.
    now: ClockMs,
    /// Sampling time, ms.
    #[props(default = 4_600)]
    duration: u64,
    /// Denoising steps.
    #[props(default = 30)]
    steps: u64,
    /// `sm`, `md` or `lg`.
    #[props(default = "md".to_string())]
    size: String,
    /// Width over height (`1`, `4 / 5`, `16 / 9`).
    #[props(default = "1".to_string())]
    aspect: String,
    /// Loop after the finished image holds.
    #[props(default = true)]
    looping: bool,
    /// Extra queue time, ms (stagger a grid).
    #[props(default)]
    delay: u64,
    /// Fails at 60 % (error scenario).
    #[props(default)]
    fail: bool,
    /// Scene seed.
    #[props(default)]
    seed: u64,
    /// Show the progress pill.
    #[props(default = true)]
    show_progress: bool,
    /// Generate again button (Oxyn extra).
    #[props(default)]
    again: bool,
) -> Element {
    let t0 = now.0;
    let mut rv = use_signal(move || Reveal::new(duration, steps, delay, looping, fail, seed, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let r = rv.read().clone();
    let mut act = move |id: String| {
        let x = rv.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let (ph, p) = r.phase(n);
    let sc = r.scene(n);
    let step = r.step(n);
    let lat = latent(sc, r.seed, step, r.noise(n));
    let blur = r.blur(n);
    let mos = r.mosaic(n);
    let dec = r.decoded(n);
    let small = size == "sm";
    let label = if ph == Phase::Done {
        SCENES[sc].to_string()
    } else {
        ph.word().to_string()
    };
    let dash = 2.0 * std::f64::consts::PI * 6.0;
    let frac = match ph {
        Phase::Generating | Phase::Error => p as f64,
        Phase::Decoding | Phase::Done => 1.0,
        Phase::Queued => 0.0,
    };
    let show_pill = show_progress && !(ph == Phase::Done && p >= 1.0);
    let cell = 100.0 / GRID as f32;
    let status_said = if said().is_empty() {
        ph.word().to_string()
    } else {
        said()
    };
    rsx! {
        div { class: "oxaik-irw",
            div { class: "oxaik-ir", "data-size": "{size}", "data-ph": "{ph:?}", style: "aspect-ratio:{aspect}", role: "img", "aria-label": "{label}",
                if ph != Phase::Queued {
                    {scene_svg(sc, "oxaik-ir-pred", format!("filter:blur({blur:.1}px) saturate({:.2})", 0.7 + 0.3 * (1.0 - r.noise(n))))}
                    svg { class: "oxaik-ir-lat", view_box: "0 0 100 100", preserve_aspect_ratio: "xMidYMid slice", style: "opacity:{mos:.2}", "shape-rendering": "crispEdges", "aria-hidden": "true",
                        for (k, c) in lat.iter().enumerate() {
                            rect { key: "{k}", x: "{(k % GRID) as f32 * cell}", y: "{(k / GRID) as f32 * cell}", width: "{cell + 0.2}", height: "{cell + 0.2}", fill: "#{c:06x}" }
                        }
                    }
                }
                if dec > 0.0 {
                    {scene_svg(sc, "oxaik-ir-crisp", format!("clip-path:inset(0 0 {:.1}% 0)", (1.0 - dec) * 100.0))}
                    if dec < 1.0 { i { class: "oxaik-ir-band", style: "top:{dec * 100.0}%" } }
                }
                if ph == Phase::Done { i { key: "{r.t0}-{sc}", class: "oxaik-ir-sheen" } }
                if ph == Phase::Queued { i { class: "oxaik-ir-q" } }
                if ph == Phase::Error {
                    div { class: "oxaik-ir-err", role: "alert",
                        {ico("alert", 16.0)} span { "Generation failed" }
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ir-retry".into()), {ico("retry", 12.0)} "Retry" }
                    }
                }
                if show_pill && ph != Phase::Error {
                    div { class: "oxaik-ir-pill", "data-ph": "{ph:?}",
                        role: if ph == Phase::Generating { "progressbar" } else { "presentation" },
                        "aria-valuemin": "0", "aria-valuemax": "{r.steps}", "aria-valuenow": "{step}", "aria-valuetext": "Step {step} of {r.steps}",
                        svg { width: "14", height: "14", view_box: "0 0 16 16", "aria-hidden": "true",
                            circle { cx: "8", cy: "8", r: "6", class: "bg" }
                            circle { cx: "8", cy: "8", r: "6", class: "fg", "stroke-dasharray": "{dash:.2}", "stroke-dashoffset": "{dash * (1.0 - frac):.2}" }
                        }
                        if !small {
                            span { class: "oxaik-ir-pl",
                                {match ph { Phase::Queued => "Queued", Phase::Decoding => "Decoding", Phase::Done => "Generated in", _ => "Generating" }}
                            }
                        }
                        span { class: "oxaik-num oxaik-ir-ct", if ph == Phase::Done { "{r.took()}" } else if ph == Phase::Queued { "" } else { "{step} / {r.steps}" } }
                    }
                }
            }
            if again {
                button { class: "oxaik-btn sm", r#type: "button", disabled: !matches!(ph, Phase::Done | Phase::Error), onclick: move |_| act("ir-again".into()), {ico("spark", 12.0)} "Generate again" }
            }
            {live(&status_said)}
        }
    }
}
