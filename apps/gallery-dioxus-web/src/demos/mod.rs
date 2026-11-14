//! The demo catalogue: one entry per component page.

use dioxus::prelude::*;
use oxyn_core::ClockMs;

use crate::{Align, Control, Demo, Kind, Vals, Variant};

mod approvals;
mod buttons;
mod chat;
mod checkpoints;
mod composer;
mod composer_parts;
mod display;
mod feedback;
mod forms;
mod inputs;
mod loaders;
mod navigation;
mod overlays;
mod parity_approvals;
mod pickers;
mod search;
mod shells;
mod status_text;
mod surfaces;
mod tools;

/// A category of the Components section.
pub struct Category {
    /// URL slug.
    pub slug: &'static str,
    /// Name.
    pub name: &'static str,
    /// One line.
    pub blurb: &'static str,
}

/// Categories in nav order.
pub const CATEGORIES: [Category; 20] = [
    Category {
        slug: "composer",
        name: "Composer",
        blurb: "The prompt box and its parts: send and stop, model, effort, context and attachments.",
    },
    Category {
        slug: "loaders",
        name: "Loaders",
        blurb: "One busy signal per region, driven by the agent state.",
    },
    Category {
        slug: "status-text",
        name: "Status text",
        blurb: "Shimmering labels, cycling verbs, run timers and streamed text.",
    },
    Category {
        slug: "tools",
        name: "Tool calls & plans",
        blurb: "What the agent is doing: tool calls, stacks, plans, steps and sub-agents.",
    },
    Category {
        slug: "approvals",
        name: "Approvals",
        blurb: "The agent asks before it acts: risk, preview, decide with a key.",
    },
    Category {
        slug: "checkpoints-git",
        name: "Checkpoints",
        blurb: "Rewind a run to any point; new attempts branch out of it.",
    },
    Category {
        slug: "buttons",
        name: "Buttons",
        blurb: "Buttons, icon buttons, split buttons and the groups they sit in.",
    },
    Category {
        slug: "inputs",
        name: "Inputs",
        blurb: "Switches, checkboxes, radios, segmented controls, selects, sliders and text fields.",
    },
    Category {
        slug: "forms",
        name: "Forms",
        blurb: "Tags, comboboxes, codes, passwords, phones, filters, steppers and paste-to-fill.",
    },
    Category {
        slug: "pickers",
        name: "Pickers",
        blurb: "Calendars, dates, ranges, times, a time-zone dial and date reels.",
    },
    Category {
        slug: "data-display",
        name: "Data display",
        blurb: "Badges, chips, avatars, keys, numbers and empty states.",
    },
    Category {
        slug: "overlays",
        name: "Overlays",
        blurb: "Tooltips, popovers and menus.",
    },
    Category {
        slug: "surfaces",
        name: "Surfaces",
        blurb: "Dialogs, sheets, stacked sheets, cards, scroll areas and quick settings.",
    },
    Category {
        slug: "feedback",
        name: "Feedback",
        blurb: "Toasts, notice stacks, alerts and skeletons that turn into content.",
    },
    Category {
        slug: "search",
        name: "Search & commands",
        blurb: "Command palettes, AI search, settings search, shortcut recording and pagination.",
    },
    Category {
        slug: "chat",
        name: "Chat",
        blurb: "Streaming answers, reasoning, sources and citations, feedback, threads, starter prompts and research runs.",
    },
    Category {
        slug: "navigation",
        name: "Navigation & layout",
        blurb: "Tabs, breadcrumbs, accordions, resizable panes, drawers, trees, docks and reading progress.",
    },
    Category {
        slug: "choice",
        name: "Choice & feedback",
        blurb: "Toggle groups, filter chips, radio cards, ratings, reactions and reorderable lists.",
    },
    Category {
        slug: "chrome",
        name: "App chrome",
        blurb: "Account menus, workspace switchers, notification centers, hover cards, code blocks and empty states.",
    },
    Category {
        slug: "shells",
        name: "App shells & screens",
        blurb: "Sidebars, icon rails, inbox and workspace sidebars, app shells, an AI side panel, terminal tabs, a demo harness and seven AI Chat screens.",
    },
];

/// Every demo, in category order.
pub fn catalog() -> Vec<Demo> {
    let mut v = Vec::new();
    v.extend(composer::demos());
    v.extend(composer_parts::demos());
    v.extend(shells::demos());
    v.extend(loaders::demos());
    v.extend(status_text::demos());
    v.extend(tools::demos());
    v.extend(approvals::demos());
    v.extend(parity_approvals::demos());
    v.extend(checkpoints::demos());
    v.extend(buttons::demos());
    v.extend(inputs::demos());
    v.extend(forms::demos());
    v.extend(pickers::demos());
    v.extend(display::demos());
    v.extend(overlays::demos());
    v.extend(surfaces::demos());
    v.extend(feedback::demos());
    v.extend(search::demos());
    v.extend(chat::demos());
    v.extend(navigation::demos());
    v
}

pub(crate) fn sel(
    key: &'static str,
    label: &'static str,
    opts: &[&'static str],
    default: &'static str,
) -> Control {
    Control {
        key,
        label,
        kind: Kind::Select(opts.to_vec()),
        default,
    }
}

pub(crate) fn tog(key: &'static str, label: &'static str, default: bool) -> Control {
    Control {
        key,
        label,
        kind: Kind::Toggle,
        default: if default { "true" } else { "false" },
    }
}

pub(crate) fn rng(
    key: &'static str,
    label: &'static str,
    min: f64,
    max: f64,
    step: f64,
    default: &'static str,
) -> Control {
    Control {
        key,
        label,
        kind: Kind::Range(min, max, step),
        default,
    }
}

pub(crate) fn txt(key: &'static str, label: &'static str, default: &'static str) -> Control {
    Control {
        key,
        label,
        kind: Kind::Text,
        default,
    }
}

pub(crate) fn var(
    name: &'static str,
    note: &'static str,
    set: &[(&'static str, &'static str)],
) -> Variant {
    Variant {
        name,
        note,
        set: set.to_vec(),
    }
}

/// Rust string literal.
pub(crate) fn q(s: &str) -> String {
    format!("{s:?}")
}

/// `prop: "value",` lines, skipping values equal to the default.
pub(crate) struct Code {
    name: String,
    lines: Vec<String>,
    pre: Vec<String>,
}

impl Code {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            name: name.into(),
            lines: Vec::new(),
            pre: Vec::new(),
        }
    }
    /// A line before the `rsx!` (model setup).
    pub(crate) fn pre(mut self, line: impl Into<String>) -> Self {
        self.pre.push(line.into());
        self
    }
    /// A raw prop line.
    pub(crate) fn raw(mut self, prop: &str, value: impl Into<String>) -> Self {
        self.lines.push(format!("{prop}: {}", value.into()));
        self
    }
    /// A string prop, left out when it equals `default`.
    pub(crate) fn s(self, prop: &str, value: &str, default: &str) -> Self {
        if value == default {
            self
        } else {
            self.raw(prop, q(value))
        }
    }
    /// An optional string prop, left out when empty or `none`.
    pub(crate) fn opt(self, prop: &str, value: &str) -> Self {
        if value.is_empty() || value == "none" {
            self
        } else {
            self.raw(prop, q(value))
        }
    }
    /// A bool prop, left out when false.
    pub(crate) fn b(self, prop: &str, value: bool) -> Self {
        if value { self.raw(prop, "true") } else { self }
    }
    /// The finished snippet.
    pub(crate) fn done(self) -> String {
        let mut out = String::new();
        for p in &self.pre {
            out.push_str(p);
            out.push('\n');
        }
        if !self.pre.is_empty() {
            out.push('\n');
        }
        out.push_str("rsx! {\n");
        if self.lines.is_empty() {
            out.push_str(&format!("    {} {{}}\n", self.name));
        } else {
            out.push_str(&format!("    {} {{\n", self.name));
            for l in &self.lines {
                out.push_str(&format!("        {l},\n"));
            }
            out.push_str("    }\n");
        }
        out.push('}');
        out
    }
}

fn no_render(_: &Vals, _: ClockMs) -> Element {
    rsx! {}
}

fn no_code(_: &Vals) -> String {
    String::new()
}

/// A demo with its identity filled in; the rest comes from struct-update syntax.
#[allow(clippy::too_many_arguments)]
pub(crate) fn mk(
    slug: &'static str,
    name: &'static str,
    category: &'static str,
    summary: &'static str,
    dioxus: &'static str,
    gpui: &'static str,
    spec: &'static str,
    source: &'static str,
) -> Demo {
    Demo {
        slug,
        name,
        category,
        summary,
        dioxus,
        gpui,
        spec,
        source,
        controls: vec![],
        variants: vec![],
        render: no_render,
        code: no_code,
        keyboard: vec![],
        a11y: vec![],
        responsive: vec![],
        height: 320,
        align: Align::Center,
        live: false,
    }
}
