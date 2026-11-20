//! Enrich column (Dioxus).

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_core::aikit::enrich::{COLUMNS, CellSt, Enrich, ROWS};

use super::{cmd, ico, key_of, live};

/// Enrich column: an AI column on a data table. One prompt runs down every row, a few cells at
/// a time; each cell shimmers while it works, then settles into a value with its source, or a
/// failure with Retry. The column menu picks the prompt; Cmd/Ctrl+Enter runs and Escape stops.
/// Concurrency is a control (Oxyn extra).
#[component]
pub fn EnrichColumn(
    /// Host clock.
    now: ClockMs,
    /// Column (0 industry, 1 employees, 2 HQ city).
    #[props(default)]
    column: usize,
    /// Cells at once.
    #[props(default = 3)]
    concurrency: usize,
    /// Run on mount.
    #[props(default = true)]
    autoplay: bool,
) -> Element {
    let t0 = now.0;
    let mut ec = use_signal(move || Enrich::new(column, concurrency, autoplay, t0));
    let mut said = use_signal(String::new);
    let n = now.0;
    let e = ec.read().clone();
    let mut act = move |id: String| {
        let x = ec.write().press(&id, n);
        if let Some(x) = x {
            said.set(x);
        }
    };
    let col = COLUMNS[e.column];
    let running = e.counts(n).0 > 0;
    rsx! {
        section { class: "oxaik oxaik-ec", "aria-label": "Accounts with an AI column",
            onkeydown: move |ev: KeyboardEvent| { let k = key_of(&ev); if (k == "Enter" && cmd(&ev)) || k == "Escape" { ev.prevent_default(); let c = cmd(&ev); let x = ec.write().key(&k, c, n); if let Some(x) = x { said.set(x) } } },
            header { class: "oxaik-hd",
                div { class: "oxaik-hl", h3 { class: "oxaik-h", "Accounts" } p { class: "oxaik-sub", "{col.1}" } }
                div { class: "oxaik-row",
                    if running {
                        button { class: "oxaik-btn sm", r#type: "button", onclick: move |_| act("ec-stop".into()), {ico("stop", 11.0)} "Stop" }
                    } else {
                        button { class: "oxaik-btn sm pri", r#type: "button", onclick: move |_| act("ec-run".into()), {ico("spark", 12.0)} "Run column" }
                    }
                }
            }
            div { class: "oxaik-scroll oxaik-ec-wrap",
                table { class: "oxaik-ec-t",
                    thead { tr {
                        th { scope: "col", "Company" }
                        th { scope: "col", "Domain" }
                        th { scope: "col", class: "oxaik-ec-ai",
                            div { class: "oxaik-menu-wrap",
                                button { class: "oxaik-ec-h", r#type: "button", "aria-haspopup": "listbox", "aria-expanded": "{e.menu}", onclick: move |_| act("ec-menu".into()),
                                    {ico("spark", 12.0)} "{col.0}" {ico("chevron", 11.0)}
                                }
                                if e.menu {
                                    ul { class: "oxaik-menu oxaik-ec-menu", role: "listbox", "aria-label": "Column prompt",
                                        for (i, c) in COLUMNS.iter().enumerate() {
                                            li { key: "{i}", role: "option", "aria-selected": "{i == e.column}", class: if i == e.column { "oxaik-opt on" } else { "oxaik-opt" }, onclick: move |_| act(format!("ec-col-{i}")),
                                                b { "{c.0}" } span { class: "oxaik-mute", "{c.1}" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } }
                    tbody {
                        for (i, (name, dom)) in ROWS.iter().enumerate() {
                            {
                                let st = e.st(i, n);
                                rsx! {
                                    tr { key: "{i}",
                                        td { class: "oxaik-ec-name", "{name}" }
                                        td { class: "oxaik-mute oxaik-mono", "{dom}" }
                                        td { class: "oxaik-ec-cell", "data-st": match st { CellSt::Empty => "empty", CellSt::Queued => "queued", CellSt::Running => "running", CellSt::Done => "done", CellSt::Failed => "failed" },
                                            {match st {
                                                CellSt::Empty => rsx! { span { class: "oxaik-mute", "—" } },
                                                CellSt::Queued => rsx! { span { class: "oxaik-mute", "Queued" } },
                                                CellSt::Running => rsx! { span { class: "oxaik-ec-bar" } span { class: "oxaik-shim", "Looking up" } },
                                                CellSt::Done => rsx! { span { class: "oxaik-ec-v oxaik-fade", "{e.value(i)}" } span { class: "oxaik-ec-src oxaik-row", {ico("globe", 10.0)} "{dom}" } },
                                                CellSt::Failed => rsx! {
                                                    span { class: "oxaik-bad oxaik-row", {ico("alert", 11.0)} "Not found" }
                                                    button { class: "oxaik-ib oxaik-ec-retry", r#type: "button", "aria-label": "Retry {name}", onclick: move |_| act(format!("ec-retry-{i}")), {ico("retry", 11.0)} }
                                                },
                                            }}
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "oxaik-row oxaik-between",
                span { class: "oxaik-mute oxaik-num", "{e.footer(n)}" }
                div { class: "oxaik-row",
                    span { class: "oxaik-mute", "At a time" }
                    for k in [1usize, 3, 6] {
                        button { key: "{k}", class: if e.concurrency == k { "oxaik-chip on oxaik-num" } else { "oxaik-chip oxaik-num" }, r#type: "button", "aria-pressed": "{e.concurrency == k}", onclick: move |_| act(format!("ec-conc-{k}")), "{k}" }
                    }
                }
            }
            {live(&said())}
        }
    }
}
