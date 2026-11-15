//! Plan #28 parity demos, chunk 4b: number entry, choice, meters, uploads, composers and model
//! menus.

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    parity::{
        compose::Effort,
        fields::Choice,
        meters::{Band, Cat, GridItem, Part, PartItem, Share, Tone},
    },
};
use oxyn_dioxus::{
    AiComposer, AvatarUpload, ContextField, ContextWindowGrid, CtxModel, ElasticSlider,
    FileDropzone, Gauge, MenuModel, ModelMenu, ModelPill, MorphSelect, NumberField, PillModel,
    RichTextEditor, RunControls, ScrubInput, SplitBar, UsageMeter,
};

use super::{Code, mk, rng, sel, tog, txt, var};
use crate::{Demo, Vals};

fn status(msg: Signal<String>) -> Element {
    rsx! { span { style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

// ---------- number field ----------

fn number_field(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let (label, value, min, max, step, prefix, suffix, one) = match v.s("kind") {
        "price" => ("Budget", 1250.0, 0.0, 5000.0, 50.0, "$", "", None),
        "ratio" => ("Temperature", 0.7, 0.0, 2.0, 0.05, "", "", None),
        _ => (
            "Seats",
            9.0,
            1.0,
            10.0,
            1.0,
            "",
            " seats",
            Some(" seat".to_string()),
        ),
    };
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:10px;align-items:flex-start",
            NumberField { label, value, min, max, step, prefix, suffix, suffix_one: one,
                description: if v.b("desc") { Some("Hold + to repeat; Shift steps ten.".to_string()) } else { None },
                size: v.string("size"), scrub: v.b("scrub"), limit_hint: v.b("limit"), disabled: v.b("disabled"), now,
                on_change: move |x: f64| msg.set(format!("onValueChange({x})")),
            }
            {status(msg)}
        }
    }
}

fn number_field_code(v: &Vals) -> String {
    Code::new("NumberField")
        .raw("label", "\"Seats\"")
        .raw("value", "9.0")
        .raw("min", "1.0")
        .raw("max", "10.0")
        .raw("suffix", "\" seats\"")
        .raw("suffix_one", "Some(\" seat\".into())")
        .s("size", &v.string("size"), "md")
        .b("scrub", v.b("scrub"))
        .raw("now", "clock.now()")
        .raw("on_change", "move |n: f64| seats.set(n)")
        .done()
}

// ---------- scrub input ----------

fn scrub_input(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let kind = v.string("kind");
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:10px;align-items:flex-start",
            div { style: "display:flex;gap:8px;flex-wrap:wrap",
                {match kind.as_str() {
                    "rotation" => rsx! { ScrubInput { label: "R", value: 350.0, min: 0.0, max: 360.0, step: 1.0, wrap: true, unit: "°", reset_value: Some(0.0), size: v.string("size"), disabled: v.b("disabled"), now,
                        on_change: move |(x, u): (f64, String)| msg.set(format!("{x}{u} (wraps at 360)")) } },
                    "opacity" => rsx! { ScrubInput { label: "Opacity", value: 80.0, min: 0.0, max: 100.0, step: 1.0, unit: "%", reset_value: Some(100.0), size: v.string("size"), disabled: v.b("disabled"), now,
                        on_change: move |(x, u): (f64, String)| msg.set(format!("{x}{u}")) } },
                    _ => rsx! {
                        ScrubInput { label: "W", value: 240.0, min: 0.0, max: 1200.0, step: 1.0, units: vec!["px".into(), "%".into(), "rem".into()], reset_value: Some(240.0), size: v.string("size"), track: v.b("track"), disabled: v.b("disabled"), now,
                            on_change: move |(x, u): (f64, String)| msg.set(format!("W {x}{u}")) }
                        ScrubInput { label: "X", value: -12.0, step: 1.0, unit: "px", size: v.string("size"), disabled: v.b("disabled"), now,
                            on_change: move |(x, u): (f64, String)| msg.set(format!("X {x}{u} (no limits)")) }
                    },
                }}
            }
            span { style: "font-size:11.5px;color:var(--oxyn-color-fg-subtle)", "Drag a label · Shift ×10 · Alt ×0.1 · type 12*2 or 50% · double-click resets" }
            {status(msg)}
        }
    }
}

fn scrub_input_code(v: &Vals) -> String {
    Code::new("ScrubInput")
        .raw("label", "\"W\"")
        .raw("value", "240.0")
        .raw("min", "0.0")
        .raw("max", "1200.0")
        .raw("units", "vec![\"px\".into(), \"%\".into(), \"rem\".into()]")
        .raw("reset_value", "Some(240.0)")
        .s("size", &v.string("size"), "md")
        .raw("now", "clock.now()")
        .raw("on_commit", "move |(value, unit)| save(value, unit)")
        .done()
}

// ---------- elastic slider ----------

fn elastic(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let marks = if v.b("marks") {
        vec![
            (0.0, "Off".to_string()),
            (25.0, String::new()),
            (50.0, "Half".to_string()),
            (75.0, String::new()),
            (100.0, "Max".to_string()),
        ]
    } else {
        vec![]
    };
    let icons = v.b("icons");
    rsx! {
        div { style: "width:380px;max-width:100%;display:flex;flex-direction:column;gap:8px",
            ElasticSlider { label: v.string("label"), value: 40.0, marks, suffix: "%",
                start_icon: if icons { Some("volume".to_string()) } else { None },
                end_icon: if icons { Some("volume".to_string()) } else { None },
                show_value: v.b("show_value"), disabled: v.b("disabled"),
                on_commit: move |x: f64| msg.set(format!("onValueCommit({x})")),
            }
            {status(msg)}
        }
    }
}

fn elastic_code(v: &Vals) -> String {
    Code::new("ElasticSlider")
        .raw("label", super::q(&v.string("label")))
        .raw("value", "40.0")
        .raw(
            "marks",
            "vec![(0.0, \"Off\".into()), (50.0, \"Half\".into()), (100.0, \"Max\".into())]",
        )
        .raw("start_icon", "Some(\"volume\".into())")
        .raw("on_commit", "move |v: f64| set_volume(v)")
        .done()
}

// ---------- morph select ----------

fn tz_options(n: usize) -> Vec<Choice> {
    let all = [
        ("utc", "UTC", "+00:00"),
        ("lon", "London", "+00:00"),
        ("par", "Paris", "+01:00"),
        ("kar", "Karachi", "+05:00"),
        ("del", "Delhi", "+05:30"),
        ("sgp", "Singapore", "+08:00"),
        ("tok", "Tokyo", "+09:00"),
        ("syd", "Sydney", "+11:00"),
        ("nyc", "New York", "−05:00"),
        ("chi", "Chicago", "−06:00"),
        ("den", "Denver", "−07:00"),
        ("sfo", "San Francisco", "−08:00"),
    ];
    all.iter()
        .take(n)
        .map(|(v, l, h)| Choice::new(v, l).hint(h))
        .collect()
}

fn morph(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let mut opts = if v.s("kind") == "long" {
        tz_options(12)
    } else {
        vec![
            Choice::new("haiku", "Haiku").hint("Fastest"),
            Choice::new("sonnet", "Sonnet").hint("Balanced"),
            Choice::new("opus", "Opus").hint("Deepest"),
            Choice::new("legacy", "Legacy 3").hint("Retired").off(),
        ]
    };
    if v.s("kind") == "long" {
        opts[1].disabled = true;
    }
    let value = if v.s("kind") == "long" {
        Some("kar".to_string())
    } else {
        Some("sonnet".to_string())
    };
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:10px;align-items:flex-start;min-height:280px",
            MorphSelect { label: v.string("label"), hide_label: v.b("hide_label"), options: opts, value, searchable: v.string("search"),
                align: v.string("align"), disabled: v.b("disabled"), now, name: Some("choice".to_string()),
                on_change: move |x: String| msg.set(format!("onValueChange({x:?})")) }
            {status(msg)}
        }
    }
}

fn morph_code(v: &Vals) -> String {
    Code::new("MorphSelect")
        .raw("label", super::q(&v.string("label")))
        .raw(
            "options",
            "models.iter().map(|m| Choice::new(&m.id, &m.name)).collect()",
        )
        .s("searchable", &v.string("search"), "auto")
        .s("align", &v.string("align"), "start")
        .raw("now", "clock.now()")
        .raw("on_change", "move |id: String| model.set(id)")
        .done()
}

// ---------- gauge ----------

fn bands() -> Vec<Band> {
    vec![
        Band::new(0.0, Tone::Success, "Healthy"),
        Band::new(70.0, Tone::Warning, "Busy"),
        Band::new(90.0, Tone::Danger, "Critical"),
    ]
}

fn gauge_demo(v: &Vals, _: ClockMs) -> Element {
    let value = v.f("value");
    rsx! {
        div { style: "display:flex;gap:24px;align-items:center;flex-wrap:wrap;justify-content:center",
            Gauge { value, label: "CPU", detail: Some("8 cores".to_string()), bands: if v.b("bands") { bands() } else { vec![] }, tone: v.string("tone") }
            if v.b("pair") {
                Gauge { value: (value * 0.6).round(), max: 64.0, raw: true, unit: " GB", label: "Memory", size: 112.0, bands: vec![] }
            }
        }
    }
}

fn gauge_code(v: &Vals) -> String {
    Code::new("Gauge")
        .raw("value", format!("{:.0}.0", v.f("value")))
        .raw("label", "\"CPU\"")
        .raw("bands", "vec![Band::new(0.0, Tone::Success, \"Healthy\"), Band::new(70.0, Tone::Warning, \"Busy\"), Band::new(90.0, Tone::Danger, \"Critical\")]")
        .done()
}

// ---------- usage meter ----------

fn usage_demo(v: &Vals, _: ClockMs) -> Element {
    let scale = v.f("fill") / 100.0;
    let shares = vec![
        Share::new("docs", "Documents", 38.0 * scale),
        Share::new("media", "Photos & video", 29.0 * scale),
        Share::new("backups", "Backups", 14.0 * scale),
        Share::new("other", "Other", 6.0 * scale),
    ];
    rsx! {
        div { style: "width:440px;max-width:100%",
            UsageMeter { title: "Storage", shares, limit: 100.0, unit: v.string("unit"), decimals: 1, warn_at: v.f("warn") / 100.0 }
        }
    }
}

fn usage_code(_: &Vals) -> String {
    Code::new("UsageMeter")
        .raw("title", "\"Storage\"")
        .raw("shares", "vec![Share::new(\"docs\", \"Documents\", 38.0), Share::new(\"media\", \"Photos & video\", 29.0)]")
        .raw("limit", "100.0")
        .raw("unit", "\"GB\"")
        .done()
}

// ---------- split bar ----------

fn split_demo(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let shares = if v.s("data") == "budget" {
        vec![
            Share::new("eng", "Engineering", 420_000.0),
            Share::new("mkt", "Marketing", 180_000.0),
            Share::new("ops", "Operations", 96_000.0),
            Share::new("other", "Other", 24_000.0),
        ]
    } else {
        vec![
            Share::new("search", "Search", 4820.0),
            Share::new("direct", "Direct", 2310.0),
            Share::new("social", "Social", 1490.0),
            Share::new("email", "Email", 760.0),
            Share::new("ref", "Referral", 410.0),
            Share::new("other", "Other", 210.0),
        ]
    };
    let budget = v.s("data") == "budget";
    rsx! {
        div { style: "width:480px;max-width:100%;display:flex;flex-direction:column;gap:6px",
            SplitBar { title: if budget { "Budget by team" } else { "Sessions by source" },
                description: Some(if budget { "This quarter" } else { "Last 30 days" }.to_string()),
                shares, unit: if budget { "" } else { "sessions" }, prefix: if budget { "$" } else { "" },
                legend: v.b("legend"), default_hidden: if v.b("hide_one") { vec!["direct".to_string(), "ops".to_string()] } else { vec![] },
                on_hidden_change: move |h: Vec<String>| msg.set(format!("hidden: {h:?}")) }
            {status(msg)}
        }
    }
}

fn split_code(_: &Vals) -> String {
    Code::new("SplitBar")
        .raw("title", "\"Sessions by source\"")
        .raw(
            "shares",
            "sources.iter().map(|s| Share::new(&s.id, &s.label, s.sessions)).collect()",
        )
        .raw("unit", "\"sessions\"")
        .raw("on_hidden_change", "move |ids| hidden.set(ids)")
        .done()
}

// ---------- context field (affectUI) ----------

fn ctx_parts(fill: f64) -> Vec<Part> {
    let k = fill / 100.0;
    vec![
        Part {
            locked: true,
            ..Part::new("sys", "System prompt", 6_000)
        },
        Part::new("tools", "Tools", (14_000.0 * k) as u64),
        Part {
            items: vec![
                PartItem {
                    id: "a".into(),
                    label: "router.rs".into(),
                    tokens: (18_000.0 * k) as u64,
                },
                PartItem {
                    id: "b".into(),
                    label: "schema.sql".into(),
                    tokens: (9_000.0 * k) as u64,
                },
                PartItem {
                    id: "c".into(),
                    label: "README.md".into(),
                    tokens: (4_000.0 * k) as u64,
                },
            ],
            ..Part::new("files", "Files", 0)
        },
        Part {
            compact: Some(0.15),
            ..Part::new("msgs", "Conversation", (112_000.0 * k) as u64)
        },
    ]
}

fn ctx_field(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    rsx! {
        div { style: "width:480px;max-width:100%;display:flex;flex-direction:column;gap:6px",
            ContextField { parts: ctx_parts(v.f("fill")), limit: 200_000, model: Some(v.string("model")), cells: v.u("cells"), columns: if v.u("cells") == 216 { 36 } else { 30 },
                action: if v.b("action") { Some("New chat".to_string()) } else { None },
                on_remove: move |(p, i): (String, String)| msg.set(format!("removed {i} from {p}")),
                on_compact: move |p: String| msg.set(format!("compacted {p}")) }
            {status(msg)}
        }
    }
}

fn ctx_field_code(_: &Vals) -> String {
    Code::new("ContextField")
        .raw("parts", "parts")
        .raw("limit", "200_000")
        .raw("model", "Some(\"Sonnet\".into())")
        .raw("action", "Some(\"New chat\".into())")
        .raw("on_compact", "move |part: String| compact(part)")
        .done()
}

// ---------- context grid (Arc) ----------

fn ctx_grid(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let models = vec![
        CtxModel {
            id: "opus".into(),
            name: "Opus · 1M".into(),
            window: 1_000_000,
            reserve: 64_000,
        },
        CtxModel {
            id: "sonnet".into(),
            name: "Sonnet · 200K".into(),
            window: 200_000,
            reserve: 32_000,
        },
        CtxModel {
            id: "mini".into(),
            name: "Mini · 128K".into(),
            window: 128_000,
            reserve: 16_000,
        },
    ];
    let k = v.f("fill") / 100.0;
    let items = vec![
        GridItem {
            cat: Cat::System,
            label: "System".into(),
            tokens: 12_000,
        },
        GridItem {
            cat: Cat::Tools,
            label: "Tools".into(),
            tokens: 38_000,
        },
        GridItem {
            cat: Cat::Files,
            label: "spec.pdf".into(),
            tokens: 90_000,
        },
        GridItem {
            cat: Cat::Messages,
            label: "Turns".into(),
            tokens: (420_000.0 * k) as u64,
        },
    ];
    let turns = (1..=8)
        .map(|i| (format!("Turn {i}"), 60_000 + i * 9_000))
        .collect();
    let files = vec![
        ("contract.pdf".to_string(), 48_000),
        ("logs.txt".to_string(), 120_000),
        ("design.fig".to_string(), 70_000),
    ];
    rsx! {
        div { style: "width:480px;max-width:100%;display:flex;flex-direction:column;gap:6px",
            ContextWindowGrid { models, items, threshold: v.f("threshold") / 100.0, turns, files, now,
                on_compact: move |(f, s, b, a, auto): (usize, u64, u64, u64, bool)| msg.set(format!("onCompact {{ folded: {f}, summary: {s}, before: {b}, after: {a}, automatic: {auto} }}")) }
            {status(msg)}
        }
    }
}

fn ctx_grid_code(_: &Vals) -> String {
    Code::new("ContextWindowGrid")
        .raw("models", "models")
        .raw("items", "items")
        .raw("threshold", "0.8")
        .raw("turns", "turns")
        .raw("files", "files")
        .raw("now", "clock.now()")
        .done()
}

// ---------- file dropzone ----------

fn dropzone(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let preset = if v.b("preset") {
        vec![
            ("brand-guidelines.pdf".to_string(), 3_400_000),
            ("logo.svg".to_string(), 24_000),
        ]
    } else {
        vec![]
    };
    rsx! {
        div { style: "width:440px;max-width:100%;display:flex;flex-direction:column;gap:6px",
            FileDropzone { variant: v.string("variant"), accept: v.string("accept"), max_size: Some(25 * 1024 * 1024),
                max_files: Some(v.u("max_files")), list_placement: v.string("placement"),
                compact_at: if v.b("compact") { Some(2) } else { None }, preset, autoplay: v.b("autoplay"), multiple: v.b("multiple"),
                disabled: v.b("disabled"), now,
                on_change: move |n: Vec<String>| msg.set(format!("{} file(s)", n.len())) }
            {status(msg)}
        }
    }
}

fn dropzone_code(v: &Vals) -> String {
    Code::new("FileDropzone")
        .s("variant", &v.string("variant"), "plain")
        .raw("accept", super::q(&v.string("accept")))
        .raw("max_size", "Some(25 * 1024 * 1024)")
        .raw("max_files", format!("Some({})", v.u("max_files")))
        .s("list_placement", &v.string("placement"), "below")
        .b("autoplay", v.b("autoplay"))
        .raw("now", "clock.now()")
        .done()
}

// ---------- avatar upload ----------

const SAMPLE: &str = "data:image/svg+xml;utf8,<svg xmlns='http://www.w3.org/2000/svg' width='420' height='300' viewBox='0 0 420 300'><defs><linearGradient id='g' x1='0' y1='0' x2='1' y2='1'><stop offset='0' stop-color='%23d9d6cf'/><stop offset='1' stop-color='%23686661'/></linearGradient></defs><rect width='420' height='300' fill='url(%23g)'/><circle cx='210' cy='118' r='54' fill='%23f2f0eb'/><path d='M110 300c10-70 55-104 100-104s90 34 100 104z' fill='%23f2f0eb'/></svg>";

fn avatar(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let has = v.s("state") != "empty";
    rsx! {
        div { style: "display:flex;flex-direction:column;gap:8px;min-height:300px;justify-content:center",
            AvatarUpload { src: if has { Some(SAMPLE.to_string()) } else { None }, start_cropping: v.s("state") == "cropping",
                fail_first: v.b("fail"), size_px: v.f("size") as f32, disabled: v.b("disabled"), now,
                on_upload: move |(x, y, s): (f32, f32, f32)| msg.set(format!("crop centre ({x:.2}, {y:.2}), side {s:.2}")),
                on_remove: move |_| msg.set("removed".into()) }
            {status(msg)}
        }
    }
}

fn avatar_code(_: &Vals) -> String {
    Code::new("AvatarUpload")
        .raw("src", "user.photo.clone()")
        .raw("now", "clock.now()")
        .raw(
            "on_upload",
            "move |(cx, cy, side)| upload_cropped(cx, cy, side)",
        )
        .raw("on_remove", "move |_| clear_photo()")
        .done()
}

// ---------- rich text ----------

const RTE_MD: &str = "# Launch notes\n\nWe ship **Thursday**. Read the *checklist* and the [runbook](https://example.com/runbook).\n\n- Freeze `main` at noon\n- Run the smoke tests\n\n> Keep the rollback ready.";

fn rte(v: &Vals, _: ClockMs) -> Element {
    let mut out = use_signal(|| None::<(String, String)>);
    let md = if v.b("empty") {
        String::new()
    } else {
        RTE_MD.to_string()
    };
    let show = v.string("output");
    rsx! {
        div { style: "width:560px;max-width:100%;display:flex;flex-direction:column;gap:8px",
            RichTextEditor { markdown: md, history_bar: v.b("history"), read_only: v.b("read_only"),
                on_change: move |(h, m, _): (String, String, String)| out.set(Some((h, m))) }
            if show != "none" {
                pre { style: "margin:0;max-height:120px;overflow:auto;font-size:11px;padding:8px 10px;border-radius:8px;background:var(--oxyn-color-surface-hover);white-space:pre-wrap",
                    {match (out(), show.as_str()) { (Some((h, _)), "html") => h, (Some((_, m)), _) => m, (None, _) => "Type to see the output…".to_string() }}
                }
            }
        }
    }
}

fn rte_code(v: &Vals) -> String {
    Code::new("RichTextEditor")
        .raw("markdown", "notes.clone()")
        .b("history_bar", v.b("history"))
        .raw(
            "on_change",
            "move |(html, markdown, text)| save(html, markdown)",
        )
        .done()
}

// ---------- AI composer ----------

fn ai_composer(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let models = vec![
        (
            "opus".to_string(),
            "Opus 5.5".to_string(),
            "Deepest reasoning".to_string(),
        ),
        (
            "sonnet".to_string(),
            "Sonnet 5.5".to_string(),
            "Fast and capable".to_string(),
        ),
        (
            "gemini".to_string(),
            "Gemini Argon".to_string(),
            "Long context".to_string(),
        ),
    ];
    let max = v.u("max");
    rsx! {
        div { style: "width:560px;max-width:100%;display:flex;flex-direction:column;gap:6px;min-height:340px;justify-content:flex-end",
            AiComposer { models, max_length: max, now, text: v.string("text"),
                suggestions: if v.b("suggestions") { vec![("Summarize the thread".to_string(), "/summarize the last 20 messages".to_string()), ("Draft a reply".to_string(), "Draft a reply to Dana about the launch".to_string()), ("Fix a bug".to_string(), "/fix the 429s in the gateway".to_string())] } else { vec![] },
                mentions: vec!["Q3 report.pdf".into(), "roadmap.md".into(), "api/rate_limit.rs".into(), "Design review notes".into(), "pricing.csv".into()],
                on_send: move |(t, m): (String, String)| msg.set(format!("onSend({t:?}, {m})")) }
            {status(msg)}
        }
    }
}

fn ai_composer_code(_: &Vals) -> String {
    Code::new("AiComposer")
        .raw("models", "models")
        .raw("mentions", "workspace_files")
        .raw("max_length", "4000")
        .raw("now", "clock.now()")
        .raw("on_send", "move |(text, model)| run(text, model)")
        .done()
}

// ---------- chat composer ----------

// ---------- run controls ----------

const ANSWER: &str = "I'll add a token bucket per API key in the gateway middleware. Each key gets 60 requests a minute with a burst of 20; when the bucket is empty the request gets a 429 with a Retry-After header. Then I'll add two integration tests that hammer one key and check the header, and update the API docs with the new limits.";

fn run_controls(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    rsx! {
        div { style: "width:560px;max-width:100%;display:flex;flex-direction:column;gap:6px",
            RunControls { agent: "Builder", agent_id: "builder-7", prompt: "Add rate limiting to the public API", answer: ANSWER,
                status: v.string("status"), mode: v.string("mode"), permission: v.string("permission"), context: v.f("context") as f32 / 100.0,
                queue: if v.b("queued") { vec!["Then update the changelog".to_string()] } else { vec![] }, now,
                on_action: move |(a, t): (String, String)| msg.set(if t.is_empty() { format!("on{a}") } else { format!("on{a}({t:?})") }) }
            {status(msg)}
        }
    }
}

fn run_controls_code(v: &Vals) -> String {
    Code::new("RunControls")
        .raw("agent", "\"Builder\"")
        .raw("prompt", "run.prompt.clone()")
        .raw("answer", "run.answer.clone()")
        .s("status", &v.string("status"), "idle")
        .s("mode", &v.string("mode"), "act")
        .raw("now", "clock.now()")
        .raw("on_action", "move |(what, text)| dispatch(what, text)")
        .done()
}

// ---------- model menu (affectUI) ----------

fn menu_models() -> Vec<MenuModel> {
    let m = |id: &str,
             name: &str,
             d: &str,
             s: Option<u8>,
             q: Option<u8>,
             c: Option<u8>,
             l: Option<u32>,
             p: Option<f64>| MenuModel {
        id: id.into(),
        name: name.into(),
        description: d.into(),
        speed: s,
        quality: q,
        cost: c,
        latency_ms: l,
        price_per_m: p,
    };
    vec![
        m(
            "auto",
            "Auto",
            "Routes each message to the best model",
            None,
            None,
            None,
            None,
            None,
        ),
        m(
            "opus",
            "Opus 5.5",
            "Hardest problems and long agent runs",
            Some(2),
            Some(5),
            Some(5),
            Some(1400),
            Some(15.0),
        ),
        m(
            "sonnet",
            "Sonnet 5.5",
            "Everyday coding and writing",
            Some(4),
            Some(4),
            Some(3),
            Some(620),
            Some(3.0),
        ),
        m(
            "haiku",
            "Haiku 5",
            "Quick answers and classification",
            Some(5),
            Some(3),
            Some(1),
            Some(240),
            Some(0.8),
        ),
    ]
}

fn model_menu(v: &Vals, _: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let top = v.s("side") == "top";
    rsx! {
        div { style: if top { "min-height:420px;display:flex;flex-direction:column;justify-content:flex-end;gap:6px" } else { "min-height:420px;display:flex;flex-direction:column;gap:6px" },
            ModelMenu { models: menu_models(), value: "sonnet", thinking: v.b("thinking"), side: v.string("side"), align: v.string("align"),
                on_change: move |id: String| msg.set(format!("onChange({id:?})")),
                on_option: move |(k, x): (String, String)| msg.set(format!("{k} → {x}")) }
            {status(msg)}
        }
    }
}

fn model_menu_code(v: &Vals) -> String {
    Code::new("ModelMenu")
        .raw("models", "models")
        .raw("value", "\"sonnet\"")
        .b("thinking", v.b("thinking"))
        .s("side", &v.string("side"), "bottom")
        .raw("on_change", "move |id| model.set(id)")
        .done()
}

// ---------- model pill (Arc) ----------

fn pill_models() -> Vec<PillModel> {
    use Effort::*;
    let m = |id: &str,
             name: &str,
             p: &str,
             blurb: &str,
             desc: &str,
             ctx: &str,
             price: (f64, f64),
             e: Vec<Effort>,
             d: Effort| PillModel {
        id: id.into(),
        name: name.into(),
        provider: p.into(),
        blurb: blurb.into(),
        description: desc.into(),
        context: ctx.into(),
        price,
        efforts: e,
        default_effort: d,
    };
    vec![
        m(
            "claude-opus-5-5",
            "Claude Opus 5.5",
            "anthropic",
            "Deepest reasoning",
            "Anthropic's most capable model for long agent runs and hard problems.",
            "1M tokens",
            (15.0, 75.0),
            vec![Low, Medium, High, XHigh, Max],
            High,
        ),
        m(
            "claude-sonnet-5-5",
            "Claude Sonnet 5.5",
            "anthropic",
            "Fast, strong all-rounder",
            "The everyday model: quick, capable at code and writing.",
            "1M tokens",
            (3.0, 15.0),
            vec![Low, Medium, High],
            Medium,
        ),
        m(
            "gpt-6-1-sol",
            "GPT-6.1 Sol",
            "openai",
            "Great at tools",
            "OpenAI's flagship with strong tool use.",
            "400K tokens",
            (5.0, 20.0),
            vec![Low, Medium, High, XHigh],
            Medium,
        ),
        m(
            "gemini-4-argon",
            "Gemini 4 Argon",
            "google",
            "Huge context",
            "Google's long-context model for big codebases and documents.",
            "2M tokens",
            (2.5, 10.0),
            vec![Low, High],
            Low,
        ),
        m(
            "claude-haiku-5",
            "Claude Haiku 5",
            "anthropic",
            "Fastest Claude",
            "Small and quick, for routing and summaries.",
            "200K tokens",
            (0.8, 4.0),
            vec![Low],
            Low,
        ),
        m(
            "gpt-6-mini",
            "GPT-6 mini",
            "openai",
            "Cheap and quick",
            "A small OpenAI model for high volume.",
            "128K tokens",
            (0.4, 1.6),
            vec![Low, Medium],
            Low,
        ),
        m(
            "gemini-4-flash",
            "Gemini 4 Flash",
            "google",
            "Low latency",
            "Google's fast model.",
            "1M tokens",
            (0.3, 1.2),
            vec![Low, Medium],
            Low,
        ),
    ]
}

fn model_pill(v: &Vals, now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let providers = vec![
        ("anthropic".to_string(), "Anthropic".to_string()),
        ("openai".to_string(), "OpenAI".to_string()),
        ("google".to_string(), "Google".to_string()),
    ];
    rsx! {
        div { style: "min-height:420px;width:360px;max-width:100%;display:flex;flex-direction:column;gap:6px",
            ModelPill { models: pill_models(), providers, show_summary: v.b("summary"), now,
                featured: if v.b("three") { vec!["claude-sonnet-5-5".into(), "gpt-6-1-sol".into(), "gemini-4-argon".into()] } else { vec![] },
                on_change: move |id: String| msg.set(format!("onValueChange({id:?})")),
                on_effort: move |e: String| msg.set(format!("onEffortChange({e:?})")) }
            {status(msg)}
        }
    }
}

fn model_pill_code(v: &Vals) -> String {
    Code::new("ModelPill")
        .raw("models", "catalog")
        .raw("providers", "providers")
        .b("show_summary", !v.b("summary"))
        .raw("now", "clock.now()")
        .raw("on_change", "move |id| model.set(id)")
        .done()
}

pub fn demos() -> Vec<Demo> {
    vec![
        Demo {
            controls: vec![
                sel("kind", "Kind", &["seats", "price", "ratio"], "seats"),
                sel("size", "Size", &["sm", "md", "lg"], "md"),
                tog("scrub", "Scrub label", false),
                tog("limit", "Limit note", true),
                tog("desc", "Description", true),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "seats",
                    "Plural suffix; press + at 10 to see the strain and the note.",
                    &[],
                ),
                var(
                    "price",
                    "Prefix, grouping and a step of 50.",
                    &[("kind", "price")],
                ),
                var(
                    "ratio",
                    "Decimals from the step; drag the label to scrub.",
                    &[("kind", "ratio"), ("scrub", "true")],
                ),
                var(
                    "small",
                    "Small, no limit note.",
                    &[("size", "sm"), ("limit", "false"), ("desc", "false")],
                ),
            ],
            render: number_field,
            code: number_field_code,
            keyboard: vec![
                ("↑ / ↓", "Step; held keys repeat"),
                ("Shift + ↑ / ↓, Page Up / Down", "Large step"),
                ("Home / End", "Min / max"),
                ("Enter", "Commit a typed draft (clamped)"),
                ("Escape", "Revert the draft"),
            ],
            a11y: vec![
                "The input is role=spinbutton with min, max, now and a valuetext that includes the affixes.",
                "The limit note is role=status.",
                "Reduced motion changes the value at once and answers a refused press with a warning tint instead of a shake.",
            ],
            responsive: vec!["Widths 164, 196 and 228 px for sm, md and lg."],
            height: 200,
            live: true,
            ..mk(
                "number-field",
                "Number field",
                "inputs",
                "A bounded number with steppers that repeat and speed up when held, a strain and a limit note at the edges, typed drafts that spring back to the limit, and digits that roll the way the value moved.",
                "NumberField",
                "NumberFieldEl",
                "number-field",
                "parity/fields.rs",
            )
        },
        Demo {
            controls: vec![
                sel("kind", "Fields", &["size", "rotation", "opacity"], "size"),
                sel("size", "Size", &["md", "sm"], "md"),
                tog("track", "Track fill", true),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "size",
                    "W with px / % / rem cycling and X without limits.",
                    &[],
                ),
                var(
                    "rotation",
                    "Rotation that wraps from 360 back to 0.",
                    &[("kind", "rotation")],
                ),
                var(
                    "opacity",
                    "Percent with a quiet track; double-click resets to 100.",
                    &[("kind", "opacity")],
                ),
            ],
            render: scrub_input,
            code: scrub_input_code,
            keyboard: vec![
                ("↑ / ↓", "Step (Shift ×10, Alt ×0.1)"),
                ("Enter", "Commit typed text, units or arithmetic"),
                ("Escape", "Restore while typing, or cancel a scrub"),
            ],
            a11y: vec![
                "The input is role=spinbutton; the label is the drag handle and stays a real <label>.",
                "Reduced motion removes the strain, shake and glide.",
            ],
            responsive: vec!["150 px wide; grows with its container."],
            height: 200,
            live: true,
            ..mk(
                "scrub-input",
                "Scrub input",
                "inputs",
                "A Figma-style number field: drag the label to scrub with fine and coarse gears, type values with units or arithmetic, wrap rotations, and feel a rubber-band strain at the limits.",
                "ScrubInput",
                "ScrubInputEl",
                "scrub-input",
                "parity/fields.rs",
            )
        },
        Demo {
            controls: vec![
                txt("label", "Label", "Volume"),
                tog("marks", "Detents", true),
                tog("icons", "Start and end icons", true),
                tog("show_value", "Show value", true),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "detents",
                    "Detents the thumb clicks into; labels can be clicked.",
                    &[],
                ),
                var(
                    "plain",
                    "No marks: just the rubber band and the swaying tag.",
                    &[("marks", "false")],
                ),
                var(
                    "bare",
                    "No icons or readout.",
                    &[
                        ("icons", "false"),
                        ("show_value", "false"),
                        ("label", "Brightness"),
                    ],
                ),
            ],
            render: elastic,
            code: elastic_code,
            keyboard: vec![
                ("← / → / ↑ / ↓", "One step"),
                ("Shift + arrows, Page Up / Down", "A tenth of the range"),
                (
                    "Home / End",
                    "Min / max; at a limit the track stretches toward the press",
                ),
            ],
            a11y: vec![
                "The thumb is role=slider with a formatted aria-valuetext.",
                "Reduced motion removes the stretch, sway and tick pop.",
            ],
            responsive: vec!["The track fills its container; icons stay at the ends."],
            height: 180,
            ..mk(
                "elastic-slider",
                "Elastic slider",
                "inputs",
                "A slider that stretches like rubber past its ends and springs back, pushing its icons outward, with a value tag that sways with the thumb's speed and detents it clicks into.",
                "ElasticSlider",
                "ElasticSliderEl",
                "elastic-slider",
                "parity/fields.rs",
            )
        },
        Demo {
            controls: vec![
                txt("label", "Label", "Model"),
                sel("kind", "Options", &["short", "long"], "short"),
                sel("search", "Search", &["auto", "on", "off"], "auto"),
                sel("align", "Align", &["start", "end"], "start"),
                tog("hide_label", "Hide label", false),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "short",
                    "Four options, one disabled: type-ahead and a gliding highlight.",
                    &[],
                ),
                var(
                    "long",
                    "Twelve time zones: a search field takes the trigger's place.",
                    &[("kind", "long"), ("label", "Time zone")],
                ),
                var("end", "Grows from the end edge.", &[("align", "end")]),
            ],
            render: morph,
            code: morph_code,
            keyboard: vec![
                (
                    "Enter / Space / ↓ / ↑",
                    "Open on the selected option; pick when open",
                ),
                ("Page Up / Down", "Eight options"),
                ("Home / End", "Ends (open on them when closed)"),
                ("Alt + ↑", "Pick and close"),
                ("Letters", "Type-ahead (picks while closed)"),
                ("Escape", "Close back to the trigger"),
                ("Tab", "Close without picking"),
            ],
            a11y: vec![
                "The trigger has aria-haspopup=listbox; options are role=option with aria-selected; the list follows aria-activedescendant.",
                "Reduced motion jumps the size and uses short fades.",
            ],
            responsive: vec!["The surface is never narrower than the trigger."],
            height: 320,
            live: true,
            ..mk(
                "morph-select",
                "Morph select",
                "inputs",
                "A select whose trigger grows into the list, with one gliding highlight, type-ahead, paging, an automatic search field for long lists and a value that rolls the way you moved.",
                "MorphSelect",
                "MorphSelectEl",
                "morph-select",
                "parity/fields.rs",
            )
        },
        Demo {
            controls: vec![
                rng("value", "Value", 0.0, 100.0, 1.0, "72"),
                tog("bands", "Threshold bands", true),
                sel(
                    "tone",
                    "Tone (no bands)",
                    &["accent", "success", "warning", "danger"],
                    "accent",
                ),
                tog("pair", "Raw value gauge", true),
            ],
            variants: vec![
                var(
                    "bands",
                    "Bands set the tone and caption as the value crosses them.",
                    &[],
                ),
                var("critical", "Past the danger band.", &[("value", "94")]),
                var(
                    "plain",
                    "No bands: one tone.",
                    &[("bands", "false"), ("pair", "false"), ("value", "38")],
                ),
            ],
            render: gauge_demo,
            code: gauge_code,
            keyboard: vec![],
            a11y: vec![
                "role=meter with min, max, now and a valuetext that includes the band.",
                "Reduced motion lands on the value at once.",
            ],
            responsive: vec!["The size prop sets the diameter."],
            height: 220,
            ..mk(
                "gauge",
                "Gauge",
                "data-display",
                "A value against a known range as a ring that fills once, springs to new values while the percent rolls, and changes tone and caption exactly at its thresholds.",
                "Gauge",
                "GaugeEl",
                "gauge",
                "parity/meters.rs",
            )
        },
        Demo {
            controls: vec![
                rng("fill", "Fill", 10.0, 140.0, 1.0, "75"),
                rng("warn", "Warn at (%)", 50.0, 100.0, 1.0, "90"),
                txt("unit", "Unit", "GB"),
            ],
            variants: vec![
                var("ok", "Under the warning line.", &[]),
                var("almost", "Past 90 %: Almost full.", &[("fill", "106")]),
                var(
                    "over",
                    "Past the limit: hatched overage and a limit marker.",
                    &[("fill", "130")],
                ),
            ],
            render: usage_demo,
            code: usage_code,
            keyboard: vec![
                ("← / → / ↑ / ↓", "Move between legend items"),
                ("Home / End", "First / last item"),
                ("Enter / Space", "Pin or unpin a category"),
                ("Escape", "Clear the pin"),
            ],
            a11y: vec![
                "The bar is role=img with the totals and the badge in its label.",
                "The legend is a roving tab stop with aria-pressed pins.",
            ],
            responsive: vec!["The legend wraps."],
            height: 200,
            ..mk(
                "usage-meter",
                "Usage meter",
                "data-display",
                "What fills an allowance: segments that grow in, a hatched overage past the limit, a badge that warns when almost full, and a legend you can pin.",
                "UsageMeter",
                "UsageMeterEl",
                "usage-meter",
                "parity/meters.rs",
            )
        },
        Demo {
            controls: vec![
                sel("data", "Data", &["traffic", "budget"], "traffic"),
                tog("legend", "Legend", true),
                tog("hide_one", "Start with one hidden", false),
            ],
            variants: vec![
                var(
                    "traffic",
                    "Six sources: small ones label below on leader lines.",
                    &[],
                ),
                var("budget", "Currency values.", &[("data", "budget")]),
                var(
                    "hidden",
                    "One segment switched off; the rest reflow.",
                    &[("hide_one", "true")],
                ),
            ],
            render: split_demo,
            code: split_code,
            keyboard: vec![
                ("Tab", "The bar, then each legend item"),
                ("→ / ↓ and ← / ↑", "Highlight the next / previous segment"),
                ("Home / End", "First / last"),
                ("Enter / Space", "Switch the highlighted segment off"),
                ("Escape", "Clear the highlight"),
            ],
            a11y: vec![
                "The readout is aria-live=polite; legend items are toggle buttons with aria-pressed.",
                "Reduced motion swaps widths without the clip-path glide.",
            ],
            responsive: vec!["Labels move outside segments narrower than about 13 % of the bar."],
            height: 240,
            ..mk(
                "split-bar",
                "Split bar",
                "data-display",
                "One 100 % bar split into shares: rest on a segment to widen it and read its figures, switch segments from the legend and the rest reflow, with labels inside or on leader lines.",
                "SplitBar",
                "SplitBarEl",
                "split-bar",
                "parity/meters.rs",
            )
        },
        Demo {
            controls: vec![
                rng("fill", "Fill", 20.0, 140.0, 1.0, "100"),
                txt("model", "Model", "Sonnet 5.5"),
                sel("cells", "Cells", &["216", "180"], "216"),
                tog("action", "Tray action", true),
            ],
            variants: vec![
                var(
                    "default",
                    "Hover a part to light its cells; open Files to remove one; Compact the conversation.",
                    &[],
                ),
                var(
                    "warn",
                    "Past 80 %: amber and the tray says what's left.",
                    &[("fill", "125")],
                ),
                var(
                    "light",
                    "Early in a chat.",
                    &[("fill", "30"), ("action", "false")],
                ),
            ],
            render: ctx_field,
            code: ctx_field_code,
            keyboard: vec![("Tab", "Parts with items, Compact and the tray action")],
            a11y: vec![
                "The field is role=img with tokens in use; the tray line is role=status.",
                "Reduced motion clears compacted cells with a short fade.",
            ],
            responsive: vec!["Cells keep their aspect; the field fills the card."],
            height: 420,
            ..mk(
                "context-field",
                "Context window (cells)",
                "composer",
                "What fills the model's context as a field of cells: hover a part to light it, remove files, and compact the conversation while its cells shimmer and clear.",
                "ContextField",
                "ContextFieldEl",
                "context-field",
                "parity/meters.rs",
            )
        },
        Demo {
            controls: vec![
                rng("fill", "Messages", 10.0, 160.0, 1.0, "100"),
                rng("threshold", "Compact at (%)", 50.0, 95.0, 1.0, "80"),
            ],
            variants: vec![
                var(
                    "default",
                    "Send messages until it compacts on its own.",
                    &[],
                ),
                var("near", "Close to the threshold.", &[("fill", "150")]),
                var(
                    "early",
                    "A low threshold.",
                    &[("threshold", "60"), ("fill", "40")],
                ),
            ],
            render: ctx_grid,
            code: ctx_grid_code,
            keyboard: vec![
                ("Tab", "Model menu, Send message, Attach file, Compact now"),
                ("Enter / Space", "Open the menu or run an action"),
                ("↑ / ↓, Enter", "Choose a model"),
                ("Escape", "Close the menu"),
            ],
            a11y: vec![
                "The grid is role=img with totals and the threshold; status lines are role=status.",
                "Reduced motion changes cells and numbers at once.",
            ],
            responsive: vec!["Twenty columns that scale with the card."],
            height: 520,
            live: true,
            ..mk(
                "context-grid",
                "Context window (grid)",
                "composer",
                "The context window as 200 cells grouped by category with a threshold row: send, attach and compact show exactly how much still fits, crossing the threshold compacts on its own, and switching models re-scales every cell.",
                "ContextWindowGrid",
                "ContextGridEl",
                "context-window-grid",
                "parity/meters.rs",
            )
        },
        Demo {
            controls: vec![
                sel("variant", "Variant", &["plain", "fan"], "plain"),
                txt("accept", "Accept", ".pdf,.csv,.docx,image/*"),
                rng("max_files", "Max files", 1.0, 8.0, 1.0, "5"),
                sel("placement", "List", &["below", "inside"], "below"),
                tog("compact", "Fold at 2 files", false),
                tog("preset", "Earlier uploads", false),
                tog("autoplay", "Sample drop", true),
                tog("multiple", "Multiple", true),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "autoplay",
                    "A sample drop of four files: progress, time left, one failure with Retry.",
                    &[],
                ),
                var(
                    "fan",
                    "affectUI style: documents fan out, type badges.",
                    &[("variant", "fan")],
                ),
                var(
                    "folding",
                    "Rows inside the edge; the prompt folds at two files.",
                    &[
                        ("placement", "inside"),
                        ("compact", "true"),
                        ("autoplay", "false"),
                        ("preset", "true"),
                    ],
                ),
                var(
                    "single",
                    "One file: a new one replaces the old.",
                    &[
                        ("multiple", "false"),
                        ("autoplay", "false"),
                        ("max_files", "1"),
                    ],
                ),
            ],
            render: dropzone,
            code: dropzone_code,
            keyboard: vec![
                ("Tab", "The file picker, then each row's Retry and Remove"),
                ("Enter / Space", "Open the picker"),
                ("↑ / ↓", "Same action on the next row"),
                ("Delete / Backspace", "Remove the focused row"),
            ],
            a11y: vec![
                "The picker input carries the label, description and limits.",
                "Failed rows announce their reason (role=alert); progress bars are role=progressbar.",
            ],
            responsive: vec!["Fills its container; names truncate."],
            height: 420,
            live: true,
            ..mk(
                "file-dropzone",
                "File dropzone",
                "forms",
                "A generous target for files: rows upload with progress and time left, fail with a reason and Retry, collapse when removed; rejected types and sizes keep their reason; the prompt can fold to a slim bar.",
                "FileDropzone",
                "FileDropzoneEl",
                "file-dropzone",
                "parity/upload.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "state",
                    "Start",
                    &["photo", "cropping", "empty"],
                    "cropping",
                ),
                tog("fail", "First upload fails", false),
                rng("size", "Avatar size", 56.0, 120.0, 4.0, "88"),
                tog("disabled", "Disabled", false),
            ],
            variants: vec![
                var(
                    "cropping",
                    "The crop circle: drag, wheel, slider, arrows, + / −, 0, Enter, Escape.",
                    &[],
                ),
                var(
                    "photo",
                    "At rest with a photo: Replace or Remove.",
                    &[("state", "photo")],
                ),
                var(
                    "failed",
                    "Save to see the progress ring stop and Retry.",
                    &[("fail", "true")],
                ),
                var("empty", "No photo yet: initials.", &[("state", "empty")]),
            ],
            render: avatar,
            code: avatar_code,
            keyboard: vec![
                (
                    "Tab",
                    "Avatar, Upload / Replace, Remove; while cropping the crop area, zoom, Save, Cancel",
                ),
                ("Arrows", "Move 2 % (Shift 10 %)"),
                ("+ / −", "Zoom"),
                ("0", "Reset"),
                ("Enter", "Save and upload"),
                ("Escape", "Cancel"),
            ],
            a11y: vec![
                "The crop area is a focusable application region with its keys in its label.",
                "Errors are role=alert.",
            ],
            responsive: vec!["The crop circle is 184 px; the row wraps on narrow screens."],
            height: 360,
            live: true,
            ..mk(
                "avatar-upload",
                "Avatar upload",
                "forms",
                "Click or drop a photo onto the avatar to crop it in place with drag, wheel zoom and keys, then upload through a progress ring with clear errors and Retry.",
                "AvatarUpload",
                "AvatarUploadEl",
                "avatar-upload",
                "parity/upload.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "output",
                    "Output",
                    &["markdown", "html", "none"],
                    "markdown",
                ),
                tog("history", "Undo / redo bar", true),
                tog("empty", "Start empty", false),
                tog("read_only", "Read only", false),
            ],
            variants: vec![
                var(
                    "default",
                    "Select text for the toolbar; type / for blocks.",
                    &[],
                ),
                var(
                    "html",
                    "Clean HTML out, with custom undo buttons.",
                    &[("output", "html")],
                ),
                var(
                    "empty",
                    "Empty: placeholder and the block hint.",
                    &[("empty", "true"), ("history", "false")],
                ),
            ],
            render: rte,
            code: rte_code,
            keyboard: vec![
                ("⌘B / ⌘I / ⌘⇧X / ⌘E", "Bold, italic, strike, code"),
                ("⌘K", "Link the selection"),
                ("⌘Z / ⌘⇧Z / ⌘Y", "Undo / redo"),
                ("/", "Block menu (↑ ↓ Enter Tab Escape)"),
                ("Tab / Shift+Tab", "Indent a list item; two spaces in code"),
                ("Enter / Shift+Enter", "New block / line break"),
            ],
            a11y: vec![
                "role=textbox with aria-multiline; toolbar buttons expose aria-pressed.",
                "Reduced motion jumps the toolbar and fades the menus.",
            ],
            responsive: vec!["Fills its container up to 560 px."],
            height: 360,
            ..mk(
                "rich-text-editor",
                "Rich text editor",
                "composer",
                "A lightweight editor with markdown shortcuts, a floating toolbar with a link face, a slash menu, its own undo and redo, and clean HTML and Markdown out.",
                "RichTextEditor",
                "RichTextEl",
                "rich-text-editor",
                "parity/compose.rs",
            )
        },
        Demo {
            controls: vec![
                rng("max", "Character limit", 40.0, 4000.0, 10.0, "4000"),
                txt("text", "Starting text", ""),
                tog("suggestions", "Suggestions", true),
            ],
            variants: vec![
                var("default", "Type / or @; send to see the stop ring.", &[]),
                var(
                    "limit",
                    "A low limit shows the ring counter.",
                    &[
                        ("max", "60"),
                        ("text", "Summarise the incident and list the follow-ups"),
                    ],
                ),
                var("bare", "No suggestions.", &[("suggestions", "false")]),
            ],
            render: ai_composer,
            code: ai_composer_code,
            keyboard: vec![
                ("Enter", "Send, or insert the highlighted option"),
                ("Shift + Enter", "New line"),
                ("/", "Commands at a line start"),
                ("@", "Files and docs"),
                ("↑ / ↓", "Move through the open list"),
                ("Tab", "Insert the option"),
                ("Escape", "Close the list"),
                ("Backspace", "Remove a token whole"),
            ],
            a11y: vec![
                "The field controls the list through aria-activedescendant.",
                "Send flips its label to Stop while streaming.",
            ],
            responsive: vec!["Fills its container up to 560 px."],
            height: 380,
            live: true,
            ..mk(
                "ai-composer",
                "AI composer",
                "composer",
                "The prompt box of an AI app: / commands and @ mentions as atomic tokens, a model and effort menu, a ring counter near the limit, and a send button that turns into a stop control ringed by the reply's progress.",
                "AiComposer",
                "AiComposerEl",
                "ai-composer",
                "parity/compose.rs",
            )
        },
        Demo {
            controls: vec![
                sel(
                    "status",
                    "Start",
                    &["running", "stopped", "error", "done", "idle"],
                    "running",
                ),
                sel("mode", "Mode", &["act", "chat"], "act"),
                sel("permission", "Permission", &["ask", "edits", "full"], "ask"),
                rng("context", "Context (%)", 0.0, 100.0, 1.0, "42"),
                tog("queued", "A queued message", true),
            ],
            variants: vec![
                var(
                    "running",
                    "Streaming: Enter steers into the answer, ⌥ Enter queues, the chevron opens the send menu.",
                    &[],
                ),
                var(
                    "stopped",
                    "Stopped: Resume, Retry, Dismiss.",
                    &[("status", "stopped")],
                ),
                var(
                    "full",
                    "Full access turns red; context past 80 %.",
                    &[("permission", "full"), ("context", "86")],
                ),
                var(
                    "chat",
                    "Chat mode hides the permission menu.",
                    &[("mode", "chat"), ("status", "idle"), ("queued", "false")],
                ),
            ],
            render: run_controls,
            code: run_controls_code,
            keyboard: vec![
                ("Enter", "Send when idle; steer while running"),
                ("⌥ Enter", "Queue"),
                ("Shift + Enter", "New line"),
            ],
            a11y: vec![
                "The run area is aria-live=polite; Mode is a radiogroup; menus use menuitemradio.",
                "Reduced motion stops the caret blink and shimmer.",
            ],
            responsive: vec!["Fills its container up to 560 px."],
            height: 420,
            live: true,
            ..mk(
                "run-controls",
                "Run controls",
                "composer",
                "Every control a running agent needs in one composer: stop, resume from the word it stopped at, retry or dismiss, steer mid-answer or queue what's next through Enter, ⌥ Enter or the send menu, with Chat or Act, permissions and context.",
                "RunControls",
                "RunControlsEl",
                "run-controls",
                "parity/compose.rs",
            )
        },
        Demo {
            controls: vec![
                tog("thinking", "Thinking on", false),
                sel("side", "Side", &["bottom", "top"], "bottom"),
                sel("align", "Align", &["start", "end"], "start"),
            ],
            variants: vec![
                var(
                    "default",
                    "Point at a model: its bars fill and the tray rolls.",
                    &[],
                ),
                var(
                    "thinking",
                    "Thinking on: the effort control unfolds.",
                    &[("thinking", "true")],
                ),
                var(
                    "top",
                    "Opens upward for a composer at the bottom.",
                    &[("side", "top")],
                ),
            ],
            render: model_menu,
            code: model_menu_code,
            keyboard: vec![
                ("↓", "Open, then move"),
                ("↑", "Move"),
                ("Enter", "Pick"),
                ("Escape", "Close"),
            ],
            a11y: vec![
                "The list is role=listbox with aria-activedescendant; switches are role=switch.",
                "Reduced motion drops the bar fill and roll.",
            ],
            responsive: vec!["The menu is 300 px wide and lines up with either edge."],
            height: 440,
            ..mk(
                "model-menu",
                "Model menu",
                "composer",
                "The model menu of an AI chat: point at a model and its speed, quality and cost bars fill, switch on extended thinking and its effort unfolds, while the tray rolls the first-token time and price.",
                "ModelMenu",
                "ModelMenuEl",
                "model-menu",
                "parity/compose.rs",
            )
        },
        Demo {
            controls: vec![
                tog("summary", "Summary", true),
                tog("three", "Three featured", false),
            ],
            variants: vec![
                var(
                    "default",
                    "Featured models, an Effort submenu and More models by provider.",
                    &[],
                ),
                var(
                    "pill",
                    "The pill alone, for a composer toolbar.",
                    &[("summary", "false")],
                ),
                var(
                    "three",
                    "Three featured: more under More models.",
                    &[("three", "true")],
                ),
            ],
            render: model_pill,
            code: model_pill_code,
            keyboard: vec![
                ("Enter / Space / ↓", "Open"),
                ("↑ / ↓", "Move"),
                ("Letters", "Jump to a model"),
                ("→ / ←", "Open / close a submenu"),
                ("Escape", "Close (or back out of a submenu)"),
            ],
            a11y: vec![
                "Menu items are menuitemradio with aria-checked; the confirmation is role=status.",
                "Reduced motion swaps everything in place.",
            ],
            responsive: vec!["The summary fills its container up to 360 px."],
            height: 440,
            live: true,
            ..mk(
                "model-pill",
                "Model pill",
                "composer",
                "A model picker in one quiet pill: featured models with a note each, an effort submenu that snaps to levels the model accepts, every other model by provider, and a summary that confirms each change.",
                "ModelPill",
                "ModelPillEl",
                "model-pill",
                "parity/compose.rs",
            )
        },
    ]
}
