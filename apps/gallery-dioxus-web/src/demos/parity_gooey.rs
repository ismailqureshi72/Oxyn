//! Menu Motion family (variants 2) demos: the gooey menu motion on an attach drop-up, a model
//! menu and a whole composer, plus two Oxyn originals – liquid tabs and liquid drops.

use dioxus::prelude::*;
use oxyn_core::ClockMs;
use oxyn_dioxus::{GooeyAttachMenu, GooeyComposer, GooeyModelMenu, LiquidDrops, LiquidTabs};

use super::{Code, mk, rng, sel, tog, var};
use crate::{Align, Demo, Vals};

const CAT: &str = "menu-motion";

fn status(msg: Signal<String>) -> Element {
    rsx! { span { class: "oxd-status", style: "font-size:12px;color:var(--oxyn-color-fg-muted);min-height:16px", role: "status", "{msg}" } }
}

/// Room above (or below) for a menu that opens out of its button.
fn room(side: &str) -> &'static str {
    if side == "down" {
        "padding:0 0 380px"
    } else {
        "padding:380px 0 0"
    }
}

// ---------- Gooey attach menu ----------

fn gooey_attach(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("side"), v.s("goo"), v.s("open"));
    let side = v.string("side");
    let pad = room(&side);
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:300px;max-width:100%;display:flex;flex-direction:column;gap:10px;{pad}",
            GooeyAttachMenu { side, goo: v.f("goo") as f32, default_open: v.b("open"),
                on_pick: move |k: String| msg.set(format!("onPick({k})")) }
            {status(msg)}
        }
    }
}

fn gooey_attach_code(v: &Vals) -> String {
    Code::new("GooeyAttachMenu")
        .s("side", v.s("side"), "up")
        .raw("on_pick", "move |kind| attach(kind)")
        .done()
}

fn gooey_attach_demo() -> Demo {
    Demo {
        controls: vec![
            sel("side", "Opens", &["up", "down"], "up"),
            rng("goo", "Goo", 0.0, 12.0, 1.0, "7"),
            tog("open", "Start open", true),
        ],
        variants: vec![
            var(
                "open",
                "The drop-up settled open: file types with their icons, then sources with their marks. Close it and open it again to watch it pour out of the button.",
                &[],
            ),
            var(
                "closed",
                "Press + : a stem stretches out of the button and pinches off as the menu swells from the corner and the rows arrive top to bottom.",
                &[("open", "false")],
            ),
            var(
                "drop-down",
                "The same motion opening downward, with a softer goo.",
                &[("side", "down"), ("goo", "10"), ("open", "false")],
            ),
        ],
        render: gooey_attach,
        code: gooey_attach_code,
        keyboard: vec![
            (
                "Enter / Space / ↑ / ↓ on +",
                "Opens the menu (focus moves into it)",
            ),
            ("↑ / ↓ · Home / End", "Move the grey fill"),
            ("Enter", "Picks the row"),
            ("Escape", "Closes and returns to +"),
        ],
        a11y: vec![
            "The + button has aria-haspopup=menu and aria-expanded; rows are menuitems followed with aria-activedescendant.",
            "A closed menu leaves the page; a press outside closes a menu you opened.",
            "With reduced motion the menu appears and leaves at once.",
        ],
        responsive: vec!["280 px menu; it scrolls inside 330 px."],
        height: 470,
        live: false,
        align: Align::Center,
        ..mk(
            "gooey-attach-menu",
            "Gooey attach menu",
            CAT,
            "A + button whose drop-up – PDF, Image, Excel, Doc, Slides, Code, Drive, GitHub – pours out of it like liquid and runs back in when it closes.",
            "GooeyAttachMenu",
            "GooeyAttachMenuEl",
            "gooey-attach-menu",
            "parity/gooeyfid.rs",
        )
    }
}

// ---------- Gooey model menu ----------

fn gooey_model(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!(
        "{}{}{}{}",
        v.s("side"),
        v.s("goo"),
        v.s("open"),
        v.s("model")
    );
    let side = v.string("side");
    let pad = room(&side);
    let m = match v.s("model") {
        "auto" => 0,
        "gpt" => 2,
        "gemini" => 3,
        _ => 1,
    };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:300px;max-width:100%;display:flex;flex-direction:column;align-items:flex-end;gap:10px;{pad}",
            GooeyModelMenu { side, goo: v.f("goo") as f32, default_open: v.b("open"), default_model: m,
                on_change: move |k: String| msg.set(format!("onChange({k})")) }
            {status(msg)}
        }
    }
}

fn gooey_model_code(v: &Vals) -> String {
    Code::new("GooeyModelMenu")
        .s("side", v.s("side"), "up")
        .raw("on_change", "move |model| set_model(model)")
        .done()
}

fn gooey_model_demo() -> Demo {
    Demo {
        controls: vec![
            sel(
                "model",
                "Model",
                &["claude", "auto", "gpt", "gemini"],
                "claude",
            ),
            sel("side", "Opens", &["up", "down"], "up"),
            rng("goo", "Goo", 0.0, 12.0, 1.0, "7"),
            tog("open", "Start open", true),
        ],
        variants: vec![
            var(
                "open",
                "Models with their makers' marks and a one-line hint; the chosen one is checked.",
                &[],
            ),
            var(
                "closed",
                "Press the chip: the list pours out of its right corner; a pick blurs the new name into the chip.",
                &[("open", "false")],
            ),
            var(
                "no goo",
                "Goo at 0: the same swell and cascade without the liquid neck.",
                &[("goo", "0"), ("open", "false"), ("model", "auto")],
            ),
        ],
        render: gooey_model,
        code: gooey_model_code,
        keyboard: vec![
            (
                "Enter / Space / ↑ / ↓ on the chip",
                "Opens the list on the chosen model",
            ),
            ("↑ / ↓ · Home / End · Enter", "Move and pick"),
            ("Escape", "Closes and returns to the chip"),
        ],
        a11y: vec![
            "The chip has aria-haspopup=listbox; rows are options with aria-selected.",
            "Each maker's mark is decoration; the name is the label.",
        ],
        responsive: vec!["268 px list, lined up with the chip's right edge."],
        height: 470,
        live: false,
        align: Align::Center,
        ..mk(
            "gooey-model-menu",
            "Gooey model menu",
            CAT,
            "A model chip whose list – each model with its maker's mark – pours out of its corner like liquid; the chosen name blurs into the chip.",
            "GooeyModelMenu",
            "GooeyModelMenuEl",
            "gooey-model-menu",
            "parity/gooeyfid.rs",
        )
    }
}

// ---------- Gooey composer ----------

fn gooey_composer(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}", v.s("open"), v.s("goo"));
    let open = match v.s("open") {
        "attach" => "attach".to_string(),
        "model" => "model".to_string(),
        _ => String::new(),
    };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:560px;max-width:100%;display:flex;flex-direction:column;gap:10px;padding-top:330px",
            GooeyComposer { goo: v.f("goo") as f32, default_open: open,
                on_send: move |t: String| msg.set(format!("onSend({t:?})")) }
            {status(msg)}
        }
    }
}

fn gooey_composer_code(_: &Vals) -> String {
    Code::new("GooeyComposer")
        .raw("on_send", "move |text| send(text)")
        .done()
}

fn gooey_composer_demo() -> Demo {
    Demo {
        controls: vec![
            sel("open", "Open", &["none", "attach", "model"], "attach"),
            rng("goo", "Goo", 0.0, 12.0, 1.0, "7"),
        ],
        variants: vec![
            var(
                "attach open",
                "The + menu spans the composer and joins its top edge with a liquid neck above the button.",
                &[],
            ),
            var(
                "model open",
                "The model menu, lined up with its chip.",
                &[("open", "model")],
            ),
            var(
                "closed",
                "Type, add files from + (each becomes a chip), change the model and send.",
                &[("open", "none")],
            ),
        ],
        render: gooey_composer,
        code: gooey_composer_code,
        keyboard: vec![
            ("Tab", "Message, +, model, send"),
            ("↑ / ↓ on + or the model chip", "Opens that menu"),
            ("Escape", "Closes the open menu"),
        ],
        a11y: vec![
            "Both triggers report aria-expanded; the menus follow the menu and listbox patterns with aria-activedescendant.",
            "Opening one menu closes the other; a press outside closes either.",
        ],
        responsive: vec![
            "Up to 560 px; the + menu takes the composer's width, the model menu 268 px.",
        ],
        height: 520,
        live: false,
        align: Align::Center,
        ..mk(
            "gooey-composer",
            "Gooey composer",
            CAT,
            "A composer whose attach drop-up and model menu both pour out of its top edge above their buttons – the shared gooey motion on a real prompt box.",
            "GooeyComposer",
            "GooeyComposerEl",
            "gooey-composer",
            "parity/gooeyfid.rs",
        )
    }
}

// ---------- Liquid tabs ----------

fn liquid_tabs(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}", v.s("set"), v.s("goo"));
    let tabs: Vec<String> = match v.s("set") {
        "two" => vec!["Ask".into(), "Act".into()],
        "five" => vec![
            "All".into(),
            "Runs".into(),
            "Files".into(),
            "Notes".into(),
            "Logs".into(),
        ],
        _ => vec!["Chat".into(), "Agent".into(), "Plan".into(), "Code".into()],
    };
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:360px;max-width:100%;display:flex;flex-direction:column;gap:10px",
            LiquidTabs { tabs, goo: v.f("goo") as f32, on_change: move |i: usize| msg.set(format!("onChange({i})")) }
            {status(msg)}
        }
    }
}

fn liquid_tabs_code(_: &Vals) -> String {
    Code::new("LiquidTabs")
        .raw(
            "tabs",
            "vec![\"Chat\".into(), \"Agent\".into(), \"Plan\".into(), \"Code\".into()]",
        )
        .raw("on_change", "move |i| mode.set(i)")
        .done()
}

fn liquid_tabs_demo() -> Demo {
    Demo {
        controls: vec![
            sel("set", "Tabs", &["four", "two", "five"], "four"),
            rng("goo", "Goo", 0.0, 12.0, 1.0, "6"),
        ],
        variants: vec![
            var(
                "four",
                "Press a far tab: the pill springs there and a droplet trails behind, then is pulled back in.",
                &[],
            ),
            var("two", "A two-way switch.", &[("set", "two")]),
            var(
                "five",
                "Five tabs, extra goo.",
                &[("set", "five"), ("goo", "9")],
            ),
        ],
        render: liquid_tabs,
        code: liquid_tabs_code,
        keyboard: vec![("← / → · Home / End", "Move between tabs (roving focus)")],
        a11y: vec!["A tablist; the chosen tab has aria-selected and the only tab stop."],
        responsive: vec!["Tabs share the width; up to 360 px."],
        height: 160,
        live: false,
        align: Align::Center,
        ..mk(
            "liquid-tabs",
            "Liquid tabs",
            CAT,
            "An Oxyn original: a segmented control whose ink pill moves like a drop, trailing a droplet that is pulled back in when it lands.",
            "LiquidTabs",
            "LiquidTabsEl",
            "liquid-tabs",
            "parity/gooeyfid.rs",
        )
    }
}

// ---------- Liquid drops ----------

fn liquid_drops(v: &Vals, _now: ClockMs) -> Element {
    let mut msg = use_signal(String::new);
    let key = format!("{}{}{}", v.s("gap"), v.s("goo"), v.s("open"));
    rsx! {
        div { key: "{key}", "data-k": "{key}", style: "width:220px;max-width:100%;display:flex;flex-direction:column;align-items:flex-end;gap:10px",
            LiquidDrops { gap: v.f("gap") as f32, goo: v.f("goo") as f32, default_open: v.b("open"),
                on_pick: move |k: String| msg.set(format!("onPick({k})")) }
            {status(msg)}
        }
    }
}

fn liquid_drops_code(_: &Vals) -> String {
    Code::new("LiquidDrops")
        .raw("on_pick", "move |action| run(action)")
        .done()
}

fn liquid_drops_demo() -> Demo {
    Demo {
        controls: vec![
            rng("gap", "Gap", 46.0, 70.0, 2.0, "54"),
            rng("goo", "Goo", 0.0, 12.0, 1.0, "7"),
            tog("open", "Start open", true),
        ],
        variants: vec![
            var(
                "open",
                "Four actions resting above the button, labels beside them.",
                &[],
            ),
            var(
                "closed",
                "Press the button: each action drips out after the one before and pulls free.",
                &[("open", "false")],
            ),
            var(
                "tight",
                "Drops closer together with heavy goo – they stay joined like a column of liquid.",
                &[("gap", "46"), ("goo", "11"), ("open", "false")],
            ),
        ],
        render: liquid_drops,
        code: liquid_drops_code,
        keyboard: vec![
            ("Enter on the button", "Opens or closes"),
            ("Tab", "Each action while open"),
            ("Escape", "Closes"),
        ],
        a11y: vec![
            "The button reports aria-expanded; closed actions leave the tab order.",
            "Each action is a button named by its label.",
        ],
        responsive: vec!["220 px column anchored to its right edge."],
        height: 340,
        live: false,
        align: Align::Center,
        ..mk(
            "liquid-drops",
            "Liquid drops",
            CAT,
            "An Oxyn original: a round + button whose actions drip out one after another and flow back in when it closes.",
            "LiquidDrops",
            "LiquidDropsEl",
            "liquid-drops",
            "parity/gooeyfid.rs",
        )
    }
}

/// Every Menu Motion (variants 2) demo, in sidebar order.
pub(crate) fn demos() -> Vec<Demo> {
    vec![
        gooey_composer_demo(),
        gooey_attach_demo(),
        gooey_model_demo(),
        liquid_tabs_demo(),
        liquid_drops_demo(),
    ]
}
