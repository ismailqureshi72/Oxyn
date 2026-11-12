//! Renders the loaders and status-text components as a frame sequence of static HTML pages, one
//! per clock tick, driven by a scripted agent run. Screenshot the frames for a poster or a clip.
//!
//! ```text
//! cargo run -p oxyn-dioxus --example motion_sheet -- tokens/dist/oxyn.css out/ 6000 50
//! ```

use std::{cell::Cell, time::Duration};

use dioxus::prelude::*;
use oxyn_core::{
    ActivityKind, AgentEvent, AgentMachine, AgentState, ClockMs, ErrorKind, StallLevel, ToolRef,
    loader::FREE_GLYPH_TABLES, text::RunMeter,
};
use oxyn_dioxus::{
    ElapsedTimer, GLYPH_LOADER_CSS, GlyphLoader, STATUS_TEXT_CSS, TextReveal, TextShimmer,
    VerbCycler,
};

thread_local! {
    static T: Cell<u64> = const { Cell::new(0) };
}

/// Clock offset so the run timer reads past a minute.
const BASE: u64 = 1_000_000;
const RUN_START: u64 = BASE - 61_000;

const ANSWER: &str = "The token refresh races the request queue. I moved the refresh behind a \
single-flight lock in auth.rs, so concurrent calls now wait for one refresh instead of firing \
three. Tests pass locally.\n";

const CODE: &str =
    "fn refresh(&self) -> Token {\n    let _g = self.lock.lock();\n    self.fetch()\n}\n";

fn read_tool() -> ToolRef {
    ToolRef {
        id: "t1".into(),
        name: "read".into(),
        activity: ActivityKind::Read,
        target: Some("auth.rs".into()),
    }
}

/// The scripted run, replayed up to sheet time `t` (ms from 0).
fn agent_at(t: u64) -> (AgentState, Duration) {
    let mut m = AgentMachine::default();
    m.apply(AgentEvent::Submit, ClockMs(RUN_START));
    m.apply(
        AgentEvent::Thinking { header: None },
        ClockMs(RUN_START + 400),
    );
    let mut events: Vec<(u64, AgentEvent)> = vec![
        (0, AgentEvent::Heartbeat),
        (1800, AgentEvent::ToolStarted(read_tool())),
        (3200, AgentEvent::ToolFinished { id: "t1".into() }),
        (3300, AgentEvent::Thinking { header: None }),
    ];
    for k in 0..8 {
        events.push((3700 + k * 220, AgentEvent::TextDelta));
    }
    events.push((5500, AgentEvent::Finished));
    for (at, e) in events {
        if at <= t {
            m.apply(e, ClockMs(BASE + at));
        }
    }
    let now = ClockMs(BASE + t);
    let s = m.tick(now).clone();
    (s, m.elapsed(now))
}

fn streamed(text: &str, t: u64, from: u64, cps: f64) -> String {
    let n = ((t.saturating_sub(from)) as f64 * cps / 1000.0) as usize;
    text.chars().take(n).collect()
}

const LOADER_STATES: [(&str, u8); 6] = [
    ("thinking", 0),
    ("tool", 1),
    ("waiting", 2),
    ("stalled", 3),
    ("done", 4),
    ("failed", 5),
];

fn fixed_state(k: u8) -> AgentState {
    match k {
        0 => AgentState::Thinking { header: None },
        1 => AgentState::ToolRunning {
            tool: read_tool(),
            concurrent: 1,
        },
        2 => AgentState::AwaitingApproval {
            request: oxyn_core::RequestRef {
                id: "r".into(),
                tool: None,
            },
        },
        3 => AgentState::Stalled {
            since: ClockMs(0),
            level: StallLevel::Hard,
            was: Box::new(AgentState::Thinking { header: None }),
        },
        4 => AgentState::Done {
            elapsed: Duration::from_secs(42),
            waited: Duration::ZERO,
        },
        _ => AgentState::Error {
            kind: ErrorKind::Tool,
        },
    }
}

#[component]
fn Sheet() -> Element {
    let t = T.with(Cell::get);
    let now = ClockMs(BASE + t);
    let (state, elapsed) = agent_at(t);
    // Done needs a visible run first: think for 1 s, then ✓ for its 600 ms hold, on a loop.
    let loader_state = |k: u8| {
        if k == 4 && t % 1600 < 1000 {
            AgentState::Thinking { header: None }
        } else {
            fixed_state(k)
        }
    };
    let answer = streamed(ANSWER, t, 3700, 70.0);
    let answer_done = t >= 3700 + ANSWER.len() as u64 * 1000 / 70;
    let code = streamed(CODE, t, 900, 22.0);
    let meter = RunMeter {
        tokens: Some((t / 5 + 14_000, false)),
        tok_per_s: Some(41.6),
        cost_usd: Some((0.07, false)),
    };
    let reveal_variants = ["fade-token", "word-blur", "typewriter", "instant"];
    let carets = ["bar", "bar", "block", "underscore"];
    let user_sent = "Why does login fail when two tabs refresh at once?";
    rsx! {
        div { class: "sheet",
            h1 { "Oxyn · loaders and status text" span { "GitHub #17–#18 · Dioxus renderer · dark theme · same engine as GPUI" } }

            // A realistic composition first.
            section { class: "card agent",
                h2 { "In context · agent panel" }
                div { class: "msg user", "{user_sent}" }
                div { class: "msg bot",
                    TextReveal { text: answer.clone(), now, ended: answer_done, variant: "fade-token".to_string() }
                }
                div { class: "status",
                    GlyphLoader { state: state.clone(), now, variant: "glyph-braille".to_string(), size: 14.0 }
                    VerbCycler { state: state.clone(), now, variant: "fade".to_string(), pack: "coder".to_string(), seed: 7, size: 13.5 }
                    span { class: "sep", "·" }
                    ElapsedTimer { state: state.clone(), elapsed, variant: "units".to_string(), size: 13.5 }
                }
                div { class: "status ide",
                    GlyphLoader { state: state.clone(), now, variant: "glyph-scanner".to_string(), size: 12.0 }
                    TextShimmer { text: "Indexing workspace".to_string(), state: state.clone(), now, variant: "calm".to_string(), size: 12.0 }
                    span { class: "grow" }
                    ElapsedTimer { state: state.clone(), elapsed, variant: "meter".to_string(), meter, size: 12.0 }
                }
            }

            section { class: "card loaders",
                h2 { "Glyph loaders · 14 free · one table each, phase-locked" }
                div { class: "lgrid",
                    span {}
                    for (name, _) in LOADER_STATES { span { class: "hd", "{name}" } }
                    for tb in FREE_GLYPH_TABLES.iter() {
                        span { class: "nm", "{tb.id}" }
                        for (_, k) in LOADER_STATES {
                            span { class: "cell",
                                GlyphLoader { state: loader_state(k), now, variant: tb.id.to_string(), size: 19.0 }
                            }
                        }
                    }
                }
            }

            section { class: "card",
                h2 { "Text shimmer · 4" }
                for v in ["sweep", "calm", "pulse", "ascii"] {
                    div { class: "row",
                        span { class: "nm", "{v}" }
                        TextShimmer { text: "Thinking through the auth flow".to_string(), state: AgentState::Thinking { header: None }, now, variant: v.to_string(), size: 15.0 }
                    }
                }
                h2 { class: "gap", "Verb cycler · 5 · same scripted run" }
                for (v, pack) in [("fade", "neutral"), ("slide", "coder"), ("blur-swap", "neutral"), ("ellipsis", "terse"), ("static", "neutral")] {
                    div { class: "row",
                        span { class: "nm", "{v}" }
                        VerbCycler { state: state.clone(), now, variant: v.to_string(), pack: pack.to_string(), seed: 3, size: 15.0 }
                    }
                }
                h2 { class: "gap", "Elapsed timer · 4" }
                for v in ["units", "clock", "summary", "meter"] {
                    div { class: "row",
                        span { class: "nm", "{v}" }
                        ElapsedTimer { state: state.clone(), elapsed, variant: v.to_string(), meter, size: 15.0 }
                    }
                }
            }

            section { class: "card reveal",
                h2 { "Text reveal · 5 · carets · 3" }
                for (v, c) in reveal_variants.into_iter().zip(carets) {
                    div { class: "rv",
                        span { class: "nm", "{v} · caret/{c}" }
                        div { class: "body",
                            TextReveal { text: streamed(ANSWER, t, 300, 55.0), now, ended: t > 300 + ANSWER.len() as u64 * 1000 / 55, variant: v.to_string(), caret: c.to_string() }
                        }
                    }
                }
                div { class: "rv",
                    span { class: "nm", "line-commit · caret/block" }
                    div { class: "body code",
                        TextReveal { text: code, now, variant: "line-commit".to_string(), caret: "block".to_string() }
                    }
                }
            }
        }
    }
}

const PAGE_CSS: &str = r#"
html,body{margin:0;background:#0B0B0C}
.sheet{padding:28px;font-family:Inter,system-ui,sans-serif;display:grid;grid-template-columns:560px 620px;gap:18px;width:max-content;color:var(--oxyn-color-fg-default)}
.sheet h1{grid-column:1/-1;margin:0 0 2px;color:#F2EFEA;font-size:22px;font-weight:600}
.sheet h1 span{margin-left:12px;color:#8A867F;font-size:13px;font-weight:400}
.card{background:var(--oxyn-color-bg-base);border:1px solid var(--oxyn-color-border-subtle);border-radius:14px;padding:16px 18px}
.card h2{margin:0 0 12px;font-size:11.5px;letter-spacing:.06em;text-transform:uppercase;color:var(--oxyn-color-fg-muted);font-weight:600}
.card h2.gap{margin-top:18px}
.nm,.hd{font-size:12px;color:var(--oxyn-color-fg-muted);font-family:"DejaVu Sans Mono",monospace}
.hd{font-size:11px}
.row{display:grid;grid-template-columns:96px 1fr;align-items:center;min-height:28px}
.lgrid{display:grid;grid-template-columns:128px repeat(6,62px);gap:4px 6px;align-items:center}
.cell{display:flex;justify-content:center;height:24px;align-items:center}
.oxyn-gl{font-family:"DejaVu Sans Mono",monospace}
.agent{grid-row:span 1}
.msg{border-radius:12px;padding:10px 13px;font-size:14.5px;line-height:1.5;margin-bottom:10px}
.msg.user{background:var(--oxyn-color-surface-raised);margin-left:80px}
.msg.bot{min-height:92px;color:var(--oxyn-color-fg-default)}
.status{display:flex;align-items:center;gap:8px;padding:9px 12px;border-radius:10px;background:var(--oxyn-color-surface-raised);margin-top:6px}
.status .sep{color:var(--oxyn-color-fg-subtle)}
.status.ide{font-family:"DejaVu Sans Mono",monospace;font-size:12px;border-radius:6px}
.grow{flex:1}
.rv{margin-bottom:12px}
.rv .body{font-size:14px;line-height:1.5;min-height:42px;margin-top:3px}
.rv .body.code{font-family:"DejaVu Sans Mono",monospace;font-size:12.5px;white-space:pre;min-height:76px;background:var(--oxyn-color-surface-raised);padding:8px 10px;border-radius:8px}
"#;

fn main() {
    let mut args = std::env::args().skip(1);
    let tokens = args
        .next()
        .and_then(|p| std::fs::read_to_string(p).ok())
        .unwrap_or_default();
    let out = args.next().unwrap_or_else(|| "frames".into());
    let total: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(6000);
    let step: u64 = args.next().and_then(|s| s.parse().ok()).unwrap_or(50);
    std::fs::create_dir_all(&out).expect("output dir");
    let mut dom = VirtualDom::new(Sheet);
    dom.rebuild_in_place();
    let mut i = 0;
    let mut t = 0;
    while t <= total {
        T.with(|c| c.set(t));
        dom.mark_dirty(ScopeId::APP);
        dom.render_immediate_to_vec();
        let body = dioxus_ssr::render(&dom);
        let html = format!(
            "<!doctype html><html><head><meta charset=utf-8><style>{tokens}{GLYPH_LOADER_CSS}{STATUS_TEXT_CSS}{PAGE_CSS}</style></head><body>{body}</body></html>"
        );
        std::fs::write(format!("{out}/f{i:04}.html"), html).expect("write frame");
        i += 1;
        t += step;
    }
    eprintln!("{i} frames");
}
