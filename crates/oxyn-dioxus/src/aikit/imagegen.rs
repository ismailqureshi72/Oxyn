//! Image generation (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::imagegen::{ASPECTS, ImageGen, MODELS, SCENES, SUGGESTIONS, VarSt};

use super::{cls, cmd, dots, ico, key_of, live, mark};

/// The layers of one forming image: neutral dot wave, dot-masked colour, the sharp image.
fn art(scene: usize, st: VarSt, style: &str, dot: f64) -> Element {
    let c = SCENES[scene].1;
    let photo = format!(
        "background:radial-gradient(circle at 72% 30%,{s} 0 7%,transparent 7.5%),radial-gradient(circle at 72% 30%,{s}88 0,transparent 26%),radial-gradient(ellipse 70% 30% at 22% 66%,{g} 0 98%,transparent 100%),linear-gradient(180deg,transparent 0 60%,rgb(255 255 255 / .12) 60.5%,transparent 70%),linear-gradient(180deg,{a} 0%,{b} 58%,{g} 58.5%,{g} 100%)",
        s = c[3],
        a = c[0],
        b = c[1],
        g = c[2]
    );
    let (wave, color_on, radius, sharp, blur) = match st {
        VarSt::Empty => (false, 0.0, 0.0, 0.0, 0.0),
        VarSt::Forming(p) if p < 0.2 => (true, 0.0, 0.0, 0.0, 0.0),
        VarSt::Forming(p) if p < 0.8 => (
            true,
            ((p - 0.2) / 0.15).min(1.0),
            18.0 + (p - 0.2) / 0.6 * 30.0,
            0.0,
            0.0,
        ),
        VarSt::Forming(p) => {
            let k = (p - 0.8) / 0.2;
            (false, 1.0 - k, 48.0, k, 12.0 * (1.0 - k))
        }
        VarSt::Done => (false, 0.0, 0.0, 1.0, 0.0),
        VarSt::Failed | VarSt::Stopped => (false, 0.35, 30.0, 0.0, 0.0),
    };
    let dot_px = dot;
    rsx! {
        div { class: "oxaik-ig-art", "data-style": "{style}", "aria-hidden": "true",
            if wave || st == VarSt::Empty { i { class: "oxaik-ig-wave", "data-on": "{wave}", style: "background-size:{dot_px}px {dot_px}px" } }
            i { class: "oxaik-ig-col", style: "{photo};opacity:{color_on};-webkit-mask-image:radial-gradient(circle,#000 0 {radius}%,transparent {radius + 4.0}%);mask-image:radial-gradient(circle,#000 0 {radius}%,transparent {radius + 4.0}%);-webkit-mask-size:{dot_px}px {dot_px}px;mask-size:{dot_px}px {dot_px}px" }
            i { class: "oxaik-ig-photo", style: "{photo};opacity:{sharp};filter:blur({blur}px);transform:scale({1.0 + blur / 300.0})" }
        }
    }
}

/// Image generation: an image studio card. Four variations form on a dot matrix (a soft wave,
/// then the colours of the coming image as a halftone, then a blur into the sharp image); a
/// filmstrip picks one; Upscale doubles its size; Vary makes four more like it; Download saves
/// it. The prompt bar has model and aspect chips and Generate (Cmd/Ctrl+Enter); Escape stops a
/// running batch and keeps what finished. The sample runs a local simulation.
#[component]
pub fn ImageGeneration(
    /// Host clock.
    now: ClockMs,
    /// Generate once on mount.
    #[props(default = true)]
    autoplay: bool,
    /// The third variation fails (error demo).
    #[props(default)]
    fail: bool,
    /// Starting aspect (0 1:1, 1 4:5, 2 16:9).
    #[props(default)]
    aspect: usize,
    /// Placeholder style: `dots` (halftone), `blur` (blur-up) or `scan` (Oxyn extras).
    #[props(default = "dots".to_string())]
    placeholder: String,
    /// Show the four variations as a grid instead of canvas + filmstrip (Oxyn extra).
    #[props(default)]
    grid: bool,
) -> Element {
    let t0 = now.0;
    let mut ig = use_signal(move || {
        let mut g = ImageGen::new(false, fail, t0);
        g.aspect = aspect.min(2);
        if autoplay {
            g.press("ig-gen", t0);
        }
        g
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let g = ig.read().clone();
    let mut act = move |id: String| {
        let x = ig.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let (_, w, h) = ASPECTS[g.aspect];
    let chosen = g.chosen;
    let st = g.var_st(chosen, n);
    let busy = g.busy(n);
    let done = st == VarSt::Done;
    let scene = g.vars.get(chosen).map(|v| v.scene).unwrap_or(0);
    let mut key = move |part: &'static str, e: KeyboardEvent| {
        let k = key_of(&e);
        let ok = matches!(
            k.as_str(),
            "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" | "Home" | "End" | "Escape"
        ) || (part == "gen" && k == "Enter" && cmd(&e));
        if ok {
            e.prevent_default();
            let x = ig.write().key(part, &k, n);
            if let Some(x) = x {
                said.set(x);
            }
        }
    };
    rsx! {
        section { class: "oxaik oxaik-ig", "aria-label": "Image generation", "data-grid": "{grid}",
            onkeydown: move |e: KeyboardEvent| { if key_of(&e) == "Escape" { e.prevent_default(); act("ig-stop".into()) } },
            if grid {
                div { class: "oxaik-ig-grid",
                    for i in 0..4usize {
                        {
                            let vs = g.var_st(i, n);
                            let sc = g.vars.get(i).map(|v| v.scene).unwrap_or(0);
                            rsx! {
                                button { key: "{i}", class: cls("oxaik-ig-cell", i == chosen), r#type: "button", style: "aspect-ratio:{w}/{h}", "aria-label": "Variation {i + 1}", "aria-pressed": "{i == chosen}",
                                    onclick: move |_| act(format!("ig-pick-{i}")),
                                    {art(sc, vs, &placeholder, 8.0)}
                                    if vs == VarSt::Failed { span { class: "oxaik-ig-bad", {ico("alert", 14.0)} "Failed" } }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxaik-ig-canvas", style: "aspect-ratio:{w}/{h}", role: "img", "aria-label": "{g.canvas_label(n)}",
                    {art(scene, st, &placeholder, 10.0)}
                    if g.vars.is_empty() {
                        div { class: "oxaik-ig-sugs",
                            p { class: "oxaik-mute", "Try a prompt" }
                            for (i, s) in SUGGESTIONS.iter().enumerate() {
                                button { key: "{i}", class: "oxaik-chip", r#type: "button", onclick: move |_| act(format!("ig-suggest-{i}")), "{s}" }
                            }
                        }
                    }
                    if st == VarSt::Failed { div { class: "oxaik-ig-bad", role: "alert", {ico("alert", 16.0)} "Failed: the model timed out. Vary or generate again." } }
                    span { class: "oxaik-ig-size oxaik-mono", "{g.size_label(n)}" }
                    span { class: "oxaik-ig-status", if busy || g.upscale_p(n).is_some_and(|p| p < 1.0) { {dots()} } else if done { {ico("check", 12.0)} } "{g.status(n)}" }
                }
                div { class: "oxaik-ig-strip",
                    div { class: "oxaik-ig-thumbs", role: "radiogroup", "aria-label": "Variations",
                        onkeydown: move |e: KeyboardEvent| key("strip", e),
                        for i in 0..4usize {
                            {
                                let vs = g.var_st(i, n);
                                let sc = g.vars.get(i).map(|v| v.scene).unwrap_or(0);
                                rsx! {
                                    button { key: "{i}", class: cls("oxaik-ig-th", i == chosen), r#type: "button", role: "radio", "aria-checked": "{i == chosen}", tabindex: if i == chosen { "0" } else { "-1" },
                                        "aria-label": "Variation {i + 1}", disabled: g.vars.is_empty(),
                                        onclick: move |_| act(format!("ig-pick-{i}")),
                                        {art(sc, vs, &placeholder, 4.0)}
                                        if vs == VarSt::Failed { span { class: "oxaik-ig-x", {ico("x", 12.0)} } }
                                    }
                                }
                            }
                        }
                    }
                    div { class: "oxaik-ig-acts",
                        button { class: "oxaik-btn sm", r#type: "button", disabled: !done || g.upscale.is_some(), "aria-label": "Upscale", onclick: move |_| act("ig-upscale".into()), {ico("maximize", 13.0)} span { class: "oxaik-ig-al", "Upscale" } }
                        button { class: "oxaik-btn sm", r#type: "button", disabled: !done, "aria-label": "Vary", onclick: move |_| act("ig-vary".into()), {ico("spark", 13.0)} span { class: "oxaik-ig-al", "Vary" } }
                        button { class: "oxaik-btn sm", r#type: "button", disabled: !done, "aria-label": "Download", onclick: move |_| act("ig-download".into()), {ico("download", 13.0)} span { class: "oxaik-ig-al", "Download" } }
                    }
                }
            }
            div { class: "oxaik-ig-bar",
                textarea { class: "oxaik-ig-prompt", rows: "2", "aria-label": "Prompt", placeholder: "Describe the image you want", value: "{g.prompt}",
                    oninput: move |e| act(format!("ig-prompt:{}", e.value())),
                    onkeydown: move |e: KeyboardEvent| key("gen", e),
                }
                div { class: "oxaik-ig-chips",
                    div { class: "oxaik-ig-models", role: "radiogroup", "aria-label": "Model", onkeydown: move |e: KeyboardEvent| key("model", e),
                        for (i, (name, mk)) in MODELS.iter().enumerate() {
                            button { key: "{i}", class: cls("oxaik-chip", i == g.model), r#type: "button", role: "radio", "aria-checked": "{i == g.model}", "aria-label": "{name}", tabindex: if i == g.model { "0" } else { "-1" },
                                onclick: move |_| act(format!("ig-model-{i}")),
                                {mark(mk, 14.0)} if i == g.model { "{name}" }
                            }
                        }
                    }
                    div { class: "oxaik-ig-models", role: "radiogroup", "aria-label": "Aspect ratio", onkeydown: move |e: KeyboardEvent| key("aspect", e),
                        for (i, (lab, _, _)) in ASPECTS.iter().enumerate() {
                            button { key: "{i}", class: cls("oxaik-chip", i == g.aspect), r#type: "button", role: "radio", "aria-checked": "{i == g.aspect}", tabindex: if i == g.aspect { "0" } else { "-1" },
                                onclick: move |_| act(format!("ig-aspect-{i}")), "{lab}"
                            }
                        }
                    }
                    if busy {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ig-stop".into()), {ico("stop", 12.0)} "Stop" }
                    } else {
                        button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("ig-gen".into()), {ico("spark", 12.0)} "Generate" }
                    }
                }
            }
            {live(&said())}
        }
    }
}
