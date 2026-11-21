//! affectUI fidelity pass, family B part 9 (plan #32): the exact **Judge Bracket**
//! ([`ModelKnockout`]). Behaviour and look only; the model is `oxyn_core::kit::judgefid`.
//! Styles: `judgefid.css` (classes `oxjb-*`), in `PRIMITIVES_CSS`. Follows the nearest
//! `data-cstyle` (Clean / Classic) unless `style` is set; time comes in as `now`.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    kit::judgefid::{
        JbContender, JbVerdict, jb_champion, jb_current, jb_demo, jb_demo_verdicts, jb_match_name,
        jb_matches, jb_on_path, jb_progress, jb_round_name, jb_rounds, jb_slots, jb_verdict,
    },
};

use crate::brand::BrandIcon;

/// Demo: one match judged per this many ms.
const STEP_MS: u64 = 1700;
/// Rest after the final before the demo loops, ms.
const REST_MS: u64 = 6000;
/// Slot pitch: two 30 px slots and a 4 px gap, plus 14 px between matches.
const MATCH_H: u32 = 78;

fn style_attr(style: &str) -> Option<String> {
    (!style.is_empty()).then(|| style.to_string())
}

fn icon(d: &'static str, size: u32) -> Element {
    rsx! { svg { width: "{size}", height: "{size}", view_box: "0 0 16 16", fill: "none", stroke: "currentColor", stroke_width: "1.5", stroke_linecap: "round", stroke_linejoin: "round", "aria-hidden": "true", path { d } } }
}

fn fmt_score(s: f32) -> String {
    format!("{s:.1}")
}

/// Judge Bracket (affectUI): answers from several models go through a knockout bracket judged
/// by another model – the match being judged shimmers, the preferred answer moves on, the other
/// fades but stays; open any match to read both answers, the scores and the judge's reason. Once
/// the final is settled the champion's route turns blue and a cup appears.
#[component]
pub fn ModelKnockout(
    /// Contenders in seed order (the demo's when `None`).
    #[props(default)]
    contenders: Option<Vec<JbContender>>,
    /// Verdicts so far (the demo's when `None`).
    #[props(default)]
    verdicts: Option<Vec<JbVerdict>>,
    /// Demo: reveal the verdicts one by one and loop.
    #[props(default)]
    live: bool,
    /// The judge model.
    #[props(default = "Opus 4.1".to_string())]
    judge: String,
    /// The task.
    #[props(default = "Fix: a retried payment charges the customer twice".to_string())]
    task: String,
    /// The match whose answers are open from the start (round, match); the one being judged
    /// (or the final) when `None`.
    #[props(default)]
    open: Option<(usize, usize)>,
    /// Heading.
    #[props(default = "Judge".to_string())]
    title: String,
    /// The clock.
    now: ClockMs,
    /// `clean`, `classic`, or empty to follow `data-cstyle`.
    #[props(default)]
    style: String,
    /// Show a Use button on the decided final; called with the champion's id.
    #[props(default)]
    on_pick: Option<EventHandler<String>>,
) -> Element {
    let t0 = use_signal(move || now);
    let mut sel = use_signal(move || open);
    let mut shut = use_signal(|| false);
    let mut nodes = use_signal(Vec::<((usize, usize), Rc<MountedData>)>::new);
    let cs = contenders.clone().unwrap_or_else(jb_demo);
    let all_v = verdicts.clone().unwrap_or_else(jb_demo_verdicts);
    let n = cs.len();
    let rounds = jb_rounds(n);
    let v: Vec<JbVerdict> = if live {
        let total = all_v.len() as u64;
        let per = (total + 1) * STEP_MS + REST_MS;
        let k = ((now.since(t0()).as_millis() as u64 % per) / STEP_MS) as usize;
        all_v.iter().take(k.min(all_v.len())).cloned().collect()
    } else {
        all_v.clone()
    };
    let cur = jb_current(n, &v);
    let champ = jb_champion(n, &v);
    let shown = sel().or(cur).unwrap_or((rounds - 1, 0));
    let h = jb_matches(n, 0) as u32 * MATCH_H;
    let status_txt = match (cur, champ) {
        (_, Some(c)) => format!("{} wins", cs[c].name),
        _ => String::new(),
    };

    let mut go = move |r: usize, m: usize| {
        sel.set(Some((r, m)));
        shut.set(false);
        if let Some(d) = nodes
            .read()
            .iter()
            .find(|(k, _)| *k == (r, m))
            .map(|(_, d)| d.clone())
        {
            spawn(async move {
                let _ = d.set_focus(true).await;
            });
        }
    };

    // The open match.
    let (sr, sm) = shown;
    let slots = jb_slots(n, &v, sr, sm);
    let sv = jb_verdict(&v, sr, sm).cloned();
    let judging_open = cur == Some(shown);
    let judge_line = if sv.is_some() {
        format!("Judged by {judge}")
    } else if judging_open {
        format!("{judge} is comparing…")
    } else {
        "Not judged yet".to_string()
    };

    rsx! {
        section { class: "oxjb", "data-style": style_attr(&style), "data-done": champ.is_some(), "aria-label": "{title}",
            onkeydown: move |e: KeyboardEvent| {
                let (r, m) = sel().or(cur).unwrap_or((0, 0));
                match e.key() {
                    Key::ArrowRight if r + 1 < rounds => { e.prevent_default(); go(r + 1, m / 2); }
                    Key::ArrowLeft if r > 0 => { e.prevent_default(); go(r - 1, m * 2); }
                    Key::ArrowDown if m + 1 < jb_matches(n, r) => { e.prevent_default(); go(r, m + 1); }
                    Key::ArrowUp if m > 0 => { e.prevent_default(); go(r, m - 1); }
                    Key::Escape => { if let Some((cr, cm)) = cur { go(cr, cm); } else { sel.set(None); } }
                    _ => {}
                }
            },
            header { class: "oxjb-bar",
                span { class: "oxjb-logo", {icon("M9.5 2.5l4 4M8 4l4 4M10 6 4.5 11.5M2.5 13.5h6", 15)} }
                h2 { class: "oxjb-ttl", "{title}" }
                span { class: "oxjb-task", "{task}" }
                span { class: "oxjb-sp" }
                if champ.is_some() {
                    span { class: "oxjb-wins", "{status_txt}" }
                } else {
                    span { class: "oxjb-pill", "Judging" }
                }
                span { class: "oxjb-prog", role: "status", "{jb_progress(n, &v)}" }
            }
            div { class: "oxjb-heads", style: "--rounds:{rounds}", "aria-hidden": "true",
                for r in 0..rounds { span { key: "{r}", "data-r": "{r}", "{jb_round_name(n, r)}" } }
            }
            div { class: "oxjb-br", style: "--rounds:{rounds};--h:{h}px",
                for r in 0..rounds {
                    {
                        let mm = jb_matches(n, r);
                        rsx! {
                            div { class: "oxjb-col", key: "{r}", "data-r": "{r}", style: "--m:{mm}", role: "group", "aria-label": "{jb_round_name(n, r)}",
                                for m in 0..mm {
                                    {
                                        let sl = jb_slots(n, &v, r, m);
                                        let vd = jb_verdict(&v, r, m).cloned();
                                        let judging = cur == Some((r, m));
                                        let path = jb_on_path(n, &v, r, m);
                                        let is_open = shown == (r, m) && !shut();
                                        let name = jb_match_name(n, r, m);
                                        let names: Vec<String> = sl.iter().map(|s| s.map_or("to be decided".to_string(), |i| cs[i].name.clone())).collect();
                                        let lbl = match &vd {
                                            Some(x) => format!("{name}: {} beat {}", names[x.winner.min(1)], names[1 - x.winner.min(1)]),
                                            None if judging => format!("{name}: {} against {}, being judged", names[0], names[1]),
                                            None => format!("{name}: {} against {}", names[0], names[1]),
                                        };
                                        let mk = format!("{r}-{m}");
                                        rsx! {
                                            button { class: "oxjb-m", r#type: "button", key: "{mk}", "data-jbm": "{mk}", "data-odd": m % 2 == 1,
                                                "data-judging": judging, "data-path": path, "data-open": is_open, "data-last": r + 1 == rounds,
                                                "aria-expanded": is_open, "aria-label": "{lbl}",
                                                onmounted: move |e| { let mut w = nodes.write(); w.retain(|(k, _)| *k != (r, m)); w.push(((r, m), e.data())); },
                                                onclick: move |_| { if sel() == Some((r, m)) || (sel().is_none() && shown == (r, m)) { shut.set(!shut()); } else { sel.set(Some((r, m))); shut.set(false); } },
                                                for (k, s) in sl.iter().enumerate() {
                                                    {
                                                        let st = match (&vd, s) {
                                                            (_, None) => "empty",
                                                            (Some(x), _) if x.winner == k => "win",
                                                            (Some(_), _) => "lose",
                                                            (None, _) => "wait",
                                                        };
                                                        let sc = vd.as_ref().map(|x| if k == 0 { x.scores.0 } else { x.scores.1 });
                                                        rsx! {
                                                            span { class: "oxjb-slot", key: "{k}", "data-k": "{k}", "data-st": st,
                                                                if let Some(i) = s {
                                                                    span { class: "oxjb-ic", BrandIcon { name: cs[*i].brand.clone(), size: 12.0, coloured: true, fallback: true } }
                                                                    span { class: "oxjb-nm", "{cs[*i].name}" }
                                                                    if let Some(sc) = sc { span { class: "oxjb-sc", "{fmt_score(sc)}" } }
                                                                    if st == "win" && r + 1 == rounds { span { class: "oxjb-cup", title: "Winner", {icon("M5 2.5h6v3a3 3 0 0 1-6 0zM5 3.5H3v1a2 2 0 0 0 2 2M11 3.5h2v1a2 2 0 0 1-2 2M8 8.5v3M5.5 13.5h5", 12)} } }
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            if !shut() {
                div { class: "oxjb-panel", key: "{sr}-{sm}", "data-k": "{sr}-{sm}",
                    div { class: "oxjb-ph",
                        b { "{jb_match_name(n, sr, sm)}" }
                        span { class: "oxjb-jl", "data-live": judging_open, "{judge_line}" }
                    }
                    div { class: "oxjb-ans",
                        for (k, s) in slots.iter().enumerate() {
                            {
                                let won = sv.as_ref().is_some_and(|x| x.winner == k);
                                let sc = sv.as_ref().map(|x| if k == 0 { x.scores.0 } else { x.scores.1 });
                                rsx! {
                                    div { class: "oxjb-a", key: "{k}", "data-k": "{k}", "data-won": won, "data-lost": sv.is_some() && !won,
                                        if let Some(i) = s {
                                            div { class: "oxjb-ah",
                                                span { class: "oxjb-ic", BrandIcon { name: cs[*i].brand.clone(), size: 12.0, coloured: true, fallback: true } }
                                                b { "{cs[*i].name}" }
                                                if won { span { class: "oxjb-win", "Winner" } }
                                                if let Some(sc) = sc { span { class: "oxjb-asc", "{fmt_score(sc)}" } }
                                            }
                                            span { class: "oxjb-meta", "{cs[*i].meta}" }
                                            p { class: "oxjb-out", "{cs[*i].output}" }
                                            code { class: "oxjb-code", "{cs[*i].code}" }
                                        } else {
                                            span { class: "oxjb-meta", "Waits for the match before it" }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(x) = sv.as_ref() {
                        p { class: "oxjb-why", span { "Judge's reason" } "{x.reason}" }
                    }
                    if let (Some(c), Some(f)) = (champ, on_pick) {
                        if sr + 1 == rounds {
                            {
                                let id = cs[c].id.clone();
                                rsx! {
                                    button { class: "oxjb-use", r#type: "button", onclick: move |_| f.call(id.clone()), "Use {cs[c].name}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
