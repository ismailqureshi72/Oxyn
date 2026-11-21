//! MCP servers (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::blocks::mcp::{BUDGET, DIRECTORY, Mcp, Sheet, SrvSt, UNDO_MS};
use oxyn_core::blocks::thousands;

use super::{status, switch};
use crate::aikit::{cls, cmd, dots, ico, key_of, live, mark, seg};

const HUES: [&str; 6] = ["blue", "violet", "teal", "amber", "green", "neutral"];

/// MCP servers: look after the Model Context Protocol connections behind an agent. The
/// header says how many context tokens the switched-on tools take over a stacked budget bar split per
/// server and a budget line (the overflow turns amber). Each server row shows its mark, a status
/// word with its icon, Remote or Local, the tools on and their tokens; open, it lists every tool
/// with a switch, read or write and its tokens, the default for tool calls, the endpoint and
/// Remove (Cmd/Ctrl+Z undoes). Add server slides a catalogue over the card: search the directory,
/// authorize (simulated), watch the tools arrive, add. A server whose sign-in lapsed goes
/// back through that sheet, and a failed one gets a Retry button on its row.
#[component]
pub fn McpServers(
    /// Host clock.
    now: ClockMs,
    /// Open this server's tools at the start.
    #[props(default)]
    open: Option<usize>,
    /// Open the Add server sheet at the start.
    #[props(default)]
    adding: bool,
    /// Show the context bar.
    #[props(default = true)]
    meter: bool,
) -> Element {
    let mut mc = use_signal(move || {
        let mut m = Mcp::new();
        m.open = open;
        if adding {
            m.sheet = Some(Sheet::Browse(String::new()));
        }
        m
    });
    let mut said = use_signal(String::new);
    let n = now.0;
    let m = mc.read().clone();
    let mut act = move |id: String| {
        let x = mc.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let (_, total) = m.totals(n);
    let scale = total.max(BUDGET) as f64;
    let over = total > BUDGET;
    rsx! {
        section { class: "oxaik oxbk oxbk-mcp", "aria-label": "MCP servers", "data-sheet": "{m.sheet.is_some()}",
            onkeydown: move |e: KeyboardEvent| {
                let k = key_of(&e);
                let c = cmd(&e);
                if matches!(k.as_str(), "Escape" | "z" | "Z") {
                    let x = mc.write().key(&k, c, n);
                    if let Some(x) = x { said.set(x) }
                }
            },
            div { class: "oxbk-mcp-main", "aria-hidden": "{m.sheet.is_some()}",
                header { class: "oxaik-hd",
                    div { class: "oxaik-hl", h3 { class: "oxaik-h", "MCP servers" } p { class: "oxaik-sub oxaik-num", "{m.header(n)}" } }
                    button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("mcp-add".into()), {ico("plus", 12.0)} "Add server" }
                }
                if meter {
                    div { class: "oxbk-mcp-meter",
                        div { class: "oxbk-stack", role: "meter", "aria-label": "Context used by tools", "aria-valuemin": "0", "aria-valuemax": "{BUDGET}", "aria-valuenow": "{total}", "aria-valuetext": "{m.meter_text(n)}", "data-over": "{over}",
                            for (i, s) in m.servers.iter().enumerate() {
                                if m.cost(i, n) > 0 {
                                    span { key: "{s.name}", "data-hue": "{HUES[i % HUES.len()]}", style: "width:{m.cost(i, n) as f64 / scale * 100.0}%", title: "{s.name}: {thousands(m.cost(i, n))} tokens" }
                                }
                            }
                            i { class: "oxbk-mcp-budget", style: "left:{BUDGET as f64 / scale * 100.0}%" }
                        }
                        p { class: "oxaik-row oxaik-mute oxaik-num", "data-hue": if over { "amber" } else { "neutral" },
                            if over { {ico("alert", 12.0)} }
                            "{m.meter_text(n)}"
                        }
                    }
                }
                ul { class: "oxbk-mcp-list",
                    onkeydown: move |e: KeyboardEvent| {
                        let k = key_of(&e);
                        if matches!(k.as_str(), "ArrowUp" | "ArrowDown" | "Home" | "End") {
                            e.prevent_default();
                            mc.write().key(&k, false, n);
                        }
                    },
                    for (i, s) in m.servers.iter().enumerate() {
                        {
                            let st = m.st(i, n);
                            let (w, h) = st.label();
                            let open = m.open == Some(i);
                            let on = m.on(i);
                            let total_t = s.tools.len();
                            let tr = if s.local { "Local" } else { "Remote" };
                            rsx! {
                                li { key: "{s.name}", class: cls("oxbk-mcp-row", open), "data-st": "{w}",
                                    div { class: "oxbk-mcp-head",
                                        button { class: "oxbk-mcp-hb", r#type: "button", "aria-expanded": "{open}", "aria-controls": "oxbk-mcp-t-{i}",
                                            tabindex: if m.focus == i { "0" } else { "-1" },
                                            "aria-label": "{s.name}, {w}, {tr}, {on} of {total_t} tools on, {thousands(m.cost(i, n))} tokens",
                                            onclick: move |_| act(format!("mcp-open-{i}")),
                                            span { class: "oxbk-mcp-mk", if s.brand.is_empty() { {ico("folder", 14.0)} } else { {mark(s.brand, 16.0)} } }
                                            span { class: "oxbk-mcp-nm", b { "{s.name}" }
                                                span { class: "oxaik-row oxaik-mute",
                                                    if st == SrvSt::Connecting { {dots()} } else { {status(w, h)} }
                                                    span { class: "oxaik-pill", if s.local { "Local" } else { "Remote" } }
                                                }
                                            }
                                            span { class: "oxbk-mcp-cnt oxaik-mute oxaik-num", span { class: "oxbk-mcp-of", "{on} of {total_t} tools" } span { "{thousands(m.cost(i, n))} tk" } }
                                            {ico(if open { "chevron" } else { "chevron-right" }, 12.0)}
                                        }
                                        if st == SrvSt::Error { button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act(format!("mcp-retry-{i}")), {ico("retry", 12.0)} "Retry" } }
                                        if st == SrvSt::NeedsAuth { button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act(format!("mcp-signin-{i}")), "Sign in" } }
                                    }
                                    if st == SrvSt::Error { p { class: "oxbk-mcp-err", {ico("alert", 12.0)} "{s.error}" } }
                                    if open {
                                        div { class: "oxbk-mcp-tools oxaik-fade", id: "oxbk-mcp-t-{i}",
                                            ul {
                                                for (t, tool) in s.tools.iter().enumerate() {
                                                    li { key: "{tool.name}", class: "oxbk-mcp-tool", "data-on": "{tool.on}",
                                                        {switch(&format!("{}, {}, {} tokens", tool.name, if tool.write { "write" } else { "read" }, tool.tokens), tool.on, EventHandler::new(move |_| act(format!("mcp-tool-{i}-{t}"))))}
                                                        span { class: "oxaik-mono oxbk-mcp-tn", "{tool.name}" }
                                                        span { class: "oxbk-mcp-rw oxaik-mute", {ico(if tool.write { "pencil" } else { "eye" }, 11.0)} span { class: "oxbk-mcp-rww", if tool.write { "Write" } else { "Read" } } }
                                                        span { class: "oxaik-mute oxaik-num", "{tool.tokens}" }
                                                    }
                                                }
                                            }
                                            div { class: "oxaik-row oxaik-between oxbk-mcp-perm",
                                                span { class: "oxaik-mute", "Tool calls" }
                                                {seg("Tool calls", vec!["Ask every time".into(), "Always allow reads".into()], usize::from(s.allow_reads),
                                                    EventHandler::new(move |v| act(format!("mcp-perm-{i}-{v}"))),
                                                    EventHandler::new(move |k: String| { let v = if k == "ArrowLeft" || k == "Home" { 0 } else { 1 }; act(format!("mcp-perm-{i}-{v}")) }))}
                                            }
                                            div { class: "oxaik-row oxaik-between",
                                                code { class: "oxaik-mono oxaik-mute oxbk-mcp-ep", "{s.endpoint}" }
                                                button { class: "oxaik-btn sm danger", r#type: "button", onclick: move |_| act(format!("mcp-remove-{i}")), {ico("trash", 12.0)} "Remove" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                if let Some((_, s, t)) = m.removed.as_ref().filter(|r| n < r.2 + UNDO_MS) {
                    div { class: "oxaik-row oxaik-between oxbk-undo", key: "{t}",
                        span { "{s.name} removed" }
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("mcp-undo".into()), {ico("undo", 12.0)} "Undo" }
                    }
                }
            }
            if let Some(sheet) = m.sheet.clone() {
                div { class: "oxbk-sheet oxaik-fade", role: "dialog", "aria-modal": "true", "aria-label": "Add server",
                    div { class: "oxaik-row oxaik-between",
                        h4 { class: "oxaik-h", {match &sheet { Sheet::Browse(_) => "Add a server", Sheet::Authorize(_) => "Authorize", Sheet::Discover(..) => "Discovering tools" }} }
                        button { class: "oxaik-ib", r#type: "button", "aria-label": "Close", onclick: move |_| act("mcp-close".into()), {ico("x", 13.0)} }
                    }
                    {match sheet {
                        Sheet::Browse(q) => rsx! {
                            input { class: "oxaik-in", r#type: "search", value: "{q}", placeholder: "Search servers", "aria-label": "Search servers", autofocus: true,
                                oninput: move |e| mc.write().search(&e.value()) }
                            ul { class: "oxbk-sheet-list",
                                if m.directory().is_empty() { li { class: "oxaik-mute oxbk-empty", "No server matches “{q}”." } }
                                for d in m.directory() {
                                    li { key: "{d}",
                                        button { class: "oxbk-sheet-item", r#type: "button", onclick: move |_| act(format!("mcp-pick-{d}")),
                                            span { class: "oxbk-mcp-mk", if DIRECTORY[d].1.is_empty() { {ico("folder", 14.0)} } else { {mark(DIRECTORY[d].1, 16.0)} } }
                                            span { class: "oxbk-mcp-nm", b { "{DIRECTORY[d].0}" } span { class: "oxaik-mute", "{DIRECTORY[d].2}" } }
                                            span { class: "oxaik-pill", if DIRECTORY[d].3 { "OAuth" } else { "Local" } }
                                        }
                                    }
                                }
                            }
                        },
                        Sheet::Authorize(d) => {
                            let (name, brand) = if let Some(i) = m.signing { (m.servers[i].name, m.servers[i].brand) } else { (DIRECTORY[d].0, DIRECTORY[d].1) };
                            rsx! {
                                div { class: "oxbk-auth",
                                    div { class: "oxaik-row oxbk-auth-mk", {mark(brand, 28.0)} {ico("link", 14.0)} span { class: "oxbk-av oxbk-av-you", "A" } }
                                    p { b { "Support agent" } " wants to use your {name} account" }
                                    ul { class: "oxbk-ro-rep", li { "Read the data its tools need" } li { "Write only with tools you leave on" } }
                                    p { class: "oxaik-mute", "Simulated: no account is contacted in this preview." }
                                    div { class: "oxaik-row",
                                        button { class: "oxaik-btn sm pri", r#type: "button", autofocus: true, onclick: move |_| act("mcp-auth".into()), "Authorize" }
                                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("mcp-close".into()), "Cancel" }
                                    }
                                }
                            }
                        }
                        Sheet::Discover(d, _) => {
                            let (_, got, done) = m.discovered(n).unwrap_or((d, 0, false));
                            rsx! {
                                div { class: "oxbk-auth",
                                    div { class: "oxaik-row", {mark(DIRECTORY[d].1, 20.0)} b { "{DIRECTORY[d].0}" } if !done { {dots()} } }
                                    ul { class: "oxbk-sheet-tools", "aria-live": "polite",
                                        for (k, t) in DIRECTORY[d].4.iter().enumerate() {
                                            if k < got {
                                                li { key: "{k}", class: "oxaik-fade", {ico("check", 12.0)} span { class: "oxaik-mono", "{t.0}" } span { class: "oxaik-mute oxaik-num", "{t.2} tk" } }
                                            }
                                        }
                                    }
                                    button { class: "oxaik-btn sm pri", r#type: "button", disabled: !done, onclick: move |_| act("mcp-finish".into()), "Add to agent" }
                                }
                            }
                        }
                    }}
                }
            }
            {live(&said.read())}
        }
    }
}
