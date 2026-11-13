//! Live demos for the Oxyn site.
//!
//! One crate, two outputs:
//! - **WASM** (`wasm32-unknown-unknown`): one bundle exporting [`mount`]. The site's `<oxyn-demo>`
//!   custom element calls it when the demo scrolls into view; every demo on the site shares the
//!   same cached module.
//! - **Host**: the `site-data` binary server-renders every demo (the poster the page shows before
//!   the WASM loads, with JavaScript off, and to crawlers) and writes the catalogue the site reads:
//!   controls, variants, keyboard, accessibility and responsive notes.
//!
//! The playground (preview, controls panel and generated code) is itself a Dioxus component, so
//! the server-rendered page and the live demo are the same markup.

use std::collections::BTreeMap;

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::TooltipProvider;

mod data;
mod demos;

pub use demos::{Category, catalog};

/// Categories of the Components section, in nav order.
pub fn categories() -> &'static [Category] {
    &demos::CATEGORIES
}

/// The clock every demo starts at; sample data is written relative to it.
pub const BASE: ClockMs = data::NOW;

/// Control values by key (all strings, parsed by the demo).
#[derive(Clone, PartialEq, Debug, Default)]
pub struct Vals(pub BTreeMap<String, String>);

impl Vals {
    /// Raw value ("" when unset).
    pub fn s(&self, k: &str) -> &str {
        self.0.get(k).map(String::as_str).unwrap_or("")
    }
    /// Owned value.
    pub fn string(&self, k: &str) -> String {
        self.s(k).to_string()
    }
    /// `Some(value)` unless empty or `none`.
    pub fn opt(&self, k: &str) -> Option<String> {
        match self.s(k) {
            "" | "none" => None,
            v => Some(v.to_string()),
        }
    }
    /// Toggle value.
    pub fn b(&self, k: &str) -> bool {
        self.s(k) == "true"
    }
    /// Number value.
    pub fn f(&self, k: &str) -> f64 {
        self.s(k).parse().unwrap_or(0.0)
    }
    /// Count value.
    pub fn u(&self, k: &str) -> usize {
        self.f(k).max(0.0) as usize
    }
}

/// What a control edits.
#[derive(Clone, PartialEq, Debug)]
pub enum Kind {
    /// One of a fixed set (segmented when short, a menu when long).
    Select(Vec<&'static str>),
    /// On or off.
    Toggle,
    /// A number: min, max, step.
    Range(f64, f64, f64),
    /// Free text.
    Text,
}

/// One row of the controls panel.
#[derive(Clone, PartialEq, Debug)]
pub struct Control {
    /// Key in [`Vals`] (usually the prop name).
    pub key: &'static str,
    /// Label in the panel.
    pub label: &'static str,
    /// Kind.
    pub kind: Kind,
    /// Default value.
    pub default: &'static str,
}

/// A named preset of control values: one variant of the component.
#[derive(Clone, PartialEq, Debug)]
pub struct Variant {
    /// Variant name (`<component>/<style>` tail).
    pub name: &'static str,
    /// One line on what differs.
    pub note: &'static str,
    /// Control overrides.
    pub set: Vec<(&'static str, &'static str)>,
}

/// Where the demo sits in its stage.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Align {
    /// Centred.
    Center,
    /// Top-left, full width.
    Start,
    /// Bottom, full width (composers).
    End,
}

impl Align {
    fn id(self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

/// One component page.
#[derive(Clone)]
pub struct Demo {
    /// URL slug.
    pub slug: &'static str,
    /// Display name.
    pub name: &'static str,
    /// Category slug.
    pub category: &'static str,
    /// One line.
    pub summary: &'static str,
    /// Dioxus components on the page (exports of `oxyn-dioxus`).
    pub dioxus: &'static str,
    /// GPUI elements (exports of `oxyn-gpui`), empty when not built yet.
    pub gpui: &'static str,
    /// Spec file stem in `docs/specs/`.
    pub spec: &'static str,
    /// Source file in `crates/oxyn-dioxus/src/`.
    pub source: &'static str,
    /// Controls.
    pub controls: Vec<Control>,
    /// Variants (presets of the controls).
    pub variants: Vec<Variant>,
    /// Renders the component for the values at `now`.
    pub render: fn(&Vals, ClockMs) -> Element,
    /// Dioxus usage for the values.
    pub code: fn(&Vals) -> String,
    /// Keyboard: key, what it does.
    pub keyboard: Vec<(&'static str, &'static str)>,
    /// Accessibility notes.
    pub a11y: Vec<&'static str>,
    /// Responsive notes.
    pub responsive: Vec<&'static str>,
    /// Stage minimum height in px.
    pub height: u32,
    /// Stage alignment.
    pub align: Align,
    /// Needs a running clock (timers, streams, countdowns).
    pub live: bool,
}

impl Demo {
    /// Default values with a variant's overrides.
    pub fn values(&self, variant: Option<usize>) -> Vals {
        let mut v = Vals::default();
        for c in &self.controls {
            v.0.insert(c.key.to_string(), c.default.to_string());
        }
        if let Some(var) = variant.and_then(|i| self.variants.get(i)) {
            for (k, x) in &var.set {
                v.0.insert((*k).to_string(), (*x).to_string());
            }
        }
        v
    }
}

/// Finds a demo by slug.
pub fn find(slug: &str) -> Option<Demo> {
    catalog().into_iter().find(|d| d.slug == slug)
}

/// Root of one mount: a tooltip provider around the playground.
#[component]
pub fn DemoRoot(slug: String, variant: Option<usize>, controls: bool) -> Element {
    rsx! {
        TooltipProvider {
            Playground { slug, variant, controls }
        }
    }
}

fn stage_key(v: &Vals) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for (k, x) in &v.0 {
        for b in k.bytes().chain([0]).chain(x.bytes()).chain([1]) {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0100_0000_01b3);
        }
    }
    format!("k{h:x}")
}

/// Preview stage, controls panel and generated code for one component.
#[component]
pub fn Playground(slug: String, variant: Option<usize>, controls: bool) -> Element {
    let demo = use_hook(|| find(&slug).map(std::rc::Rc::new));
    let Some(demo) = demo else {
        return rsx! { div { class: "oxd-missing", "Unknown demo: {slug}" } };
    };
    let initial = demo.values(variant);
    let init = initial.clone();
    let mut vals = use_signal(move || init);
    let mut now = use_signal(|| BASE);
    let mut playing = use_signal(|| demo.live);
    let mut nonce = use_signal(|| 0u32);
    #[cfg(target_arch = "wasm32")]
    {
        let live = demo.live;
        use_future(move || async move {
            if !live {
                return;
            }
            loop {
                gloo_timers::future::TimeoutFuture::new(100).await;
                if *playing.peek() {
                    let t = now.peek().0;
                    now.set(ClockMs(t + 100));
                }
            }
        });
    }
    let v = vals.read().clone();
    let key = format!("{}-{}", stage_key(&v), nonce());
    let render = demo.render;
    let t = now();
    let code = (demo.code)(&v);
    let align = demo.align.id();
    let h = demo.height;
    let ctl = demo.controls.clone();
    let live = demo.live;
    let reset_to = initial.clone();
    rsx! {
        div { class: if controls { "oxd oxd-full" } else { "oxd oxd-mini" }, "data-demo": "{slug}",
            div { class: "oxd-main",
                div { class: "oxd-stage {align}", style: "min-height:{h}px",
                    for k in [key] {
                        div { key: "{k}", class: "oxd-mount", {render(&v, t)} }
                    }
                }
                if controls {
                    aside { class: "oxd-panel", "aria-label": "Controls",
                        div { class: "oxd-ph",
                            span { "Controls" }
                            button { class: "oxd-reset", r#type: "button",
                                onclick: move |_| { vals.set(reset_to.clone()); now.set(BASE); nonce += 1; },
                                "Reset"
                            }
                        }
                        if live {
                            div { class: "oxd-row oxd-clock",
                                span { class: "oxd-l", "Clock" }
                                div { class: "oxd-seg",
                                    button { r#type: "button", class: if playing() { "on" } else { "" }, "aria-pressed": "{playing()}", onclick: move |_| playing.set(true), "Play" }
                                    button { r#type: "button", class: if !playing() { "on" } else { "" }, "aria-pressed": "{!playing()}", onclick: move |_| playing.set(false), "Pause" }
                                    button { r#type: "button", onclick: move |_| now.set(BASE), "Restart" }
                                }
                            }
                        }
                        for c in ctl {
                            ControlRow { control: c.clone(), value: v.string(c.key), on_change: move |(k, x): (String, String)| { vals.write().0.insert(k, x); } }
                        }
                    }
                }
            }
            if controls {
                div { class: "oxd-code",
                    div { class: "oxd-ch",
                        span { "Dioxus" }
                        button { class: "oxd-copy", r#type: "button", "data-copy-from": "next", "Copy" }
                    }
                    pre { code { class: "language-rust", "{code}" } }
                }
            }
        }
    }
}

#[component]
fn ControlRow(
    control: Control,
    value: String,
    on_change: EventHandler<(String, String)>,
) -> Element {
    let key = control.key.to_string();
    let id = format!("oxd-{}", control.key);
    let label = control.label;
    match control.kind {
        Kind::Select(opts)
            if opts.len() <= 4 && opts.iter().map(|o| o.len()).sum::<usize>() <= 22 =>
        {
            rsx! {
                div { class: "oxd-row",
                    span { class: "oxd-l", id: "{id}", "{label}" }
                    div { class: "oxd-seg", role: "radiogroup", "aria-labelledby": "{id}",
                        for o in opts {
                            button { r#type: "button", role: "radio", "aria-checked": "{value == o}", class: if value == o { "on" } else { "" },
                                onclick: { let key = key.clone(); move |_| on_change.call((key.clone(), o.to_string())) },
                                "{o}"
                            }
                        }
                    }
                }
            }
        }
        Kind::Select(opts) => rsx! {
            div { class: "oxd-row",
                label { class: "oxd-l", r#for: "{id}", "{label}" }
                select { id: "{id}", class: "oxd-select", value: "{value}",
                    onchange: move |e: FormEvent| on_change.call((key.clone(), e.value())),
                    for o in opts { option { value: "{o}", selected: value == o, "{o}" } }
                }
            }
        },
        Kind::Toggle => {
            let on = value == "true";
            rsx! {
                div { class: "oxd-row",
                    span { class: "oxd-l", id: "{id}", "{label}" }
                    button { r#type: "button", role: "switch", class: if on { "oxd-sw on" } else { "oxd-sw" }, "aria-checked": "{on}", "aria-labelledby": "{id}",
                        onclick: move |_| on_change.call((key.clone(), (!on).to_string())),
                        span { class: "oxd-knob" }
                    }
                }
            }
        }
        Kind::Range(min, max, step) => rsx! {
            div { class: "oxd-row",
                label { class: "oxd-l", r#for: "{id}", "{label}" }
                div { class: "oxd-range",
                    input { id: "{id}", r#type: "range", min: "{min}", max: "{max}", step: "{step}", value: "{value}",
                        oninput: move |e: FormEvent| on_change.call((key.clone(), e.value())),
                    }
                    output { "{value}" }
                }
            }
        },
        Kind::Text => rsx! {
            div { class: "oxd-row",
                label { class: "oxd-l", r#for: "{id}", "{label}" }
                input { id: "{id}", class: "oxd-text", r#type: "text", value: "{value}",
                    oninput: move |e: FormEvent| on_change.call((key.clone(), e.value())),
                }
            }
        },
    }
}

/// Server-renders one mount (the poster markup the site shows before the WASM loads).
#[cfg(not(target_arch = "wasm32"))]
pub fn render_html(slug: &str, variant: Option<usize>, controls: bool) -> String {
    let mut dom = VirtualDom::new_with_props(
        DemoRoot,
        DemoRootProps {
            slug: slug.to_string(),
            variant,
            controls,
        },
    );
    dom.rebuild_in_place();
    dioxus_ssr::render(&dom)
}

/// Mounts a live demo into `el`, replacing its server-rendered content. `variant` < 0 means the
/// default values; `controls` adds the controls panel and code.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn mount(el: web_sys::Element, slug: String, variant: i32, controls: bool) {
    el.set_inner_html("");
    let vdom = VirtualDom::new_with_props(
        DemoRoot,
        DemoRootProps {
            slug,
            variant: usize::try_from(variant).ok(),
            controls,
        },
    );
    dioxus_web::launch::launch_virtual_dom(vdom, dioxus_web::Config::new().rootelement(el));
}
