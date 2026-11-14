//! Surfaces (plan #26): `Dialog`, `Sheet`, `SheetStack`, `BottomSheet`, `Toaster` (with
//! [`use_toasts`]), `NotificationStack`, `Alert`, `Card`, `ScrollArea`, `Pagination`,
//! `Skeleton`, `ShimmerLabel`, `SkeletonMorph` + `MorphBlock`.
//!
//! Logic lives in `oxyn_core::primitives::{modal, toast, scroll, feedback, skeleton}`. Overlays
//! render into a layer that is fixed to the viewport, or absolute inside the nearest positioned
//! ancestor with `contained` (previews, inspectors, embedded apps). Timers (exits, a running
//! confirm, toast timers and the hairline) are CSS animations whose `animationend` advances the
//! model, so they pause with `animation-play-state` and need no host clock.

use std::rc::Rc;

use dioxus::prelude::*;
use oxyn_core::{
    ClockMs,
    primitives::{
        feedback::{Alert as AlertModel, AlertTone, Card as CardModel, CardEdge},
        modal::{
            BottomSheet as SheetModel, CloseReason, DetentKey, Dialog as DialogModel, DialogSize,
            EdgeDrag, SheetRelease, SheetSide, SheetStack as StackModel, StackMode, StackSheet,
        },
        scroll::{
            Metrics, PageMode, PageSlot, Pagination as PageModel, Stick, StickToBottom, edges,
            fades, page_count, page_for_size, range_label, thumb,
        },
        skeleton::{Shape, Sweep, line_widths},
        toast::{
            Notice, NotificationStack as NoticeModel, ToastKind, ToastOptions, ToastPosition,
            Toaster as ToasterModel,
        },
    },
};

use super::{Switch, buttons::Spin, cls};
use crate::composer::icons::Icon;

fn slug(s: &str) -> String {
    s.chars()
        .filter(|c| c.is_alphanumeric())
        .take(18)
        .collect::<String>()
        .to_lowercase()
}

fn focus(d: Option<Rc<MountedData>>) {
    if let Some(d) = d {
        spawn(async move {
            let _ = d.set_focus(true).await;
        });
    }
}

/// Dialog: a modal window on a dimmed backdrop. Title and quiet line, a body that scrolls under a
/// fixed header and footer, Escape and backdrop close (each can be turned off), focus kept inside
/// and handed back to the trigger. Sizes `sm` · `md` · `lg` · `full`. Oxyn plus: a `danger`
/// confirm whose action runs for `busy_ms` with a spinner while the dialog can't be dismissed,
/// and a non-modal floating variant (`modal: false`).
#[component]
pub fn Dialog(
    /// Title (names the dialog).
    title: String,
    /// Quiet line (describes it).
    #[props(default)]
    description: Option<String>,
    /// Start open.
    #[props(default)]
    open: bool,
    /// A secondary button that opens it.
    #[props(default)]
    trigger_label: Option<String>,
    /// `sm` · `md` · `lg` · `full`.
    #[props(default = "md".to_string())]
    size: String,
    /// Width px (overrides size).
    #[props(default)]
    width: Option<f32>,
    /// A press on the backdrop closes it.
    #[props(default = true)]
    close_on_backdrop: bool,
    /// Escape closes it.
    #[props(default = true)]
    close_on_escape: bool,
    /// The × button.
    #[props(default = true)]
    close_button: bool,
    /// Its label.
    #[props(default = "Close".to_string())]
    close_label: String,
    /// Inside the nearest positioned ancestor instead of the viewport.
    #[props(default)]
    contained: bool,
    /// Modal (traps focus, dims the page). Off = floating card.
    #[props(default = true)]
    modal: bool,
    /// `default` · `danger` (an alert dialog with a red confirm).
    #[props(default = "default".to_string())]
    tone: String,
    /// Footer confirm label (adds Cancel + Confirm).
    #[props(default)]
    confirm_label: Option<String>,
    /// Cancel label.
    #[props(default = "Cancel".to_string())]
    cancel_label: String,
    /// Oxyn plus: the confirm runs this long (spinner, can't dismiss).
    #[props(default)]
    busy_ms: u32,
    /// Body.
    #[props(default)]
    children: Element,
    /// Open state changed.
    #[props(default)]
    on_open_change: EventHandler<bool>,
    /// Confirm finished.
    #[props(default)]
    on_confirm: EventHandler<()>,
) -> Element {
    let mut m = use_signal(|| {
        let mut d = DialogModel::new();
        d.size = DialogSize::from_id(&size);
        d.close_on_backdrop = close_on_backdrop;
        d.close_on_escape = close_on_escape;
        d.modal = modal;
        if open {
            d.show(ClockMs(0));
        }
        d
    });
    let mut closing = use_signal(|| false);
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let mut first = use_signal(|| None::<Rc<MountedData>>);
    let mut last = use_signal(|| None::<Rc<MountedData>>);
    let mut panel = use_signal(|| None::<Rc<MountedData>>);
    let id = format!("oxyn-dlg-{}", slug(&title));
    let mut close = move |r: CloseReason| {
        if m.write().request_close(r, ClockMs(0)) {
            closing.set(true);
            on_open_change.call(false);
            focus(trig.peek().clone());
        }
    };
    let st = m.read().clone();
    let shown = st.is_open() || closing();
    let busy = st.busy;
    let danger = tone == "danger";
    let w = width
        .or(DialogSize::from_id(&size).width())
        .map(|w| format!("width:{w}px"))
        .unwrap_or_default();
    rsx! {
        if let Some(t) = trigger_label.clone() {
            button { class: "oxyn-btn secondary sm", r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{st.is_open()}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| { m.write().show(ClockMs(0)); closing.set(false); on_open_change.call(true) },
                "{t}"
            }
        }
        if shown {
            div { class: cls(&["oxyn-layer", if contained { "contained" } else { "" }, if closing() { "closing" } else { "" }, if modal { "" } else { "floating" }]),
                if modal { div { class: "oxyn-scrim", onpointerdown: move |_| close(CloseReason::Backdrop) } }
                div { class: cls(&["oxyn-dlg", &size, if danger { "danger" } else { "" }]), style: "{w}",
                    role: if danger { "alertdialog" } else { "dialog" }, "aria-modal": "{modal}", "aria-labelledby": "{id}-t",
                    "aria-describedby": if description.is_some() { "{id}-d" }, "aria-busy": if busy { "true" }, tabindex: "-1",
                    onmounted: move |e| { panel.set(Some(e.data())); focus(Some(e.data())) },
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); close(CloseReason::Escape) },
                    span { class: "oxyn-guard", tabindex: "0", "aria-hidden": "true", onfocus: move |_| focus(last.peek().clone().or(panel.peek().clone())) }
                    header { class: "dh",
                        div { class: "dtt",
                            h2 { id: "{id}-t", class: "dt", "{title}" }
                            if let Some(d) = description.clone() { p { id: "{id}-d", class: "dd", "{d}" } }
                        }
                        if close_button {
                            button { class: "oxyn-x", r#type: "button", "aria-label": "{close_label}", disabled: busy,
                                onmounted: move |e| first.set(Some(e.data())),
                                onclick: move |_| close(CloseReason::CloseButton),
                                Icon { name: "x", size: 14.0, stroke: 2.0 }
                            }
                        }
                    }
                    div { class: "db", {children} }
                    if let Some(c) = confirm_label.clone() {
                        footer { class: "df",
                            button { class: "oxyn-btn ghost sm", r#type: "button", disabled: busy, onclick: move |_| close(CloseReason::Action), "{cancel_label}" }
                            button { class: cls(&["oxyn-btn sm", if danger { "destructive" } else { "primary" }]), r#type: "button", "aria-busy": if busy { "true" },
                                onmounted: move |e| last.set(Some(e.data())),
                                onclick: move |_| {
                                    if busy_ms == 0 { on_confirm.call(()); close(CloseReason::Action) } else { m.write().busy = true; }
                                },
                                if busy { Spin { size: 12.0 } }
                                "{c}"
                            }
                        }
                    }
                    span { class: "oxyn-guard", tabindex: "0", "aria-hidden": "true", onfocus: move |_| focus(first.peek().clone().or(panel.peek().clone())) }
                    if busy { span { class: "oxyn-timer", style: "animation-duration:{busy_ms}ms", onanimationend: move |_| { m.write().busy = false; on_confirm.call(()); close(CloseReason::Action) } } }
                }
                if closing() { span { class: "oxyn-timer", style: "animation-duration:170ms", onanimationend: move |_| closing.set(false) } }
            }
        }
    }
}

/// Sheet (Oxyn plus): the dialog docked to an edge, for inspectors, run details and settings.
/// Slides in from its side; Escape, the scrim and the × close it; drag the handle toward the edge
/// (or fling) to dismiss.
#[component]
pub fn Sheet(
    /// Title.
    title: String,
    /// Quiet line.
    #[props(default)]
    description: Option<String>,
    /// Start open.
    #[props(default)]
    open: bool,
    /// A button that opens it.
    #[props(default)]
    trigger_label: Option<String>,
    /// `right` · `left` · `top` · `bottom`.
    #[props(default = "right".to_string())]
    side: String,
    /// Width (left / right) or height (top / bottom) px.
    #[props(default = 360.0)]
    size: f32,
    /// Floating with a gutter and rounded corners.
    #[props(default)]
    inset: bool,
    /// Inside the nearest positioned ancestor.
    #[props(default)]
    contained: bool,
    /// A press on the scrim closes it.
    #[props(default = true)]
    close_on_backdrop: bool,
    /// Drag handle on the inner edge.
    #[props(default = true)]
    draggable: bool,
    /// Footer action label.
    #[props(default)]
    footer: Option<String>,
    /// Body.
    #[props(default)]
    children: Element,
    /// Open state changed.
    #[props(default)]
    on_open_change: EventHandler<bool>,
) -> Element {
    let s = SheetSide::from_id(&side);
    let mut m = use_signal(|| {
        let mut d = DialogModel::new();
        d.close_on_backdrop = close_on_backdrop;
        if open {
            d.show(ClockMs(0));
        }
        d
    });
    let mut closing = use_signal(|| false);
    let mut drag = use_signal(EdgeDrag::default);
    let mut last_pos = use_signal(|| (0.0f32, 0.0f32));
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let id = format!("oxyn-sheet-{}", slug(&title));
    let mut close = move |r: CloseReason| {
        if m.write().request_close(r, ClockMs(0)) {
            closing.set(true);
            on_open_change.call(false);
            focus(trig.peek().clone());
        }
    };
    let horiz = s.horizontal();
    let pos = move |e: &PointerEvent| {
        let c = e.client_coordinates();
        if horiz { c.x as f32 } else { c.y as f32 }
    };
    let st = m.read().clone();
    let shown = st.is_open() || closing();
    let d = drag();
    let off = d.offset * s.close_sign();
    let tf = if d.offset != 0.0 {
        if horiz {
            format!("transform:translateX({off:.1}px)")
        } else {
            format!("transform:translateY({off:.1}px)")
        }
    } else {
        String::new()
    };
    let dim = if d.active {
        (1.0 - (d.offset / size).clamp(0.0, 1.0)).to_string()
    } else {
        "1".into()
    };
    let dimension = if horiz {
        format!("width:{size}px")
    } else {
        format!("height:{size}px")
    };
    rsx! {
        if let Some(t) = trigger_label.clone() {
            button { class: "oxyn-btn secondary sm", r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{st.is_open()}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| { m.write().show(ClockMs(0)); closing.set(false); on_open_change.call(true) },
                "{t}"
            }
        }
        if shown {
            div { class: cls(&["oxyn-layer sheet", if contained { "contained" } else { "" }, if closing() { "closing" } else { "" }]),
                onpointermove: move |e| if drag.peek().active { let p = pos(&e); let (a, _) = *last_pos.peek(); last_pos.set((p, a)); drag.write().moved(p, s.close_sign(), size) },
                onpointerup: move |e| if drag.peek().active {
                    let p = pos(&e);
                    let (a, b) = *last_pos.peek();
                    let v = (a - b) * s.close_sign() * 60.0;
                    let _ = p;
                    if drag.write().release(v, size) { close(CloseReason::Drag) }
                },
                div { class: "oxyn-scrim", style: "opacity:{dim}", onpointerdown: move |_| close(CloseReason::Backdrop) }
                div { class: cls(&["oxyn-sheet", s.id(), if inset { "inset" } else { "" }, if d.active { "dragging" } else { "" }]), style: "{dimension};{tf}",
                    role: "dialog", "aria-modal": "true", "aria-labelledby": "{id}-t", tabindex: "-1",
                    onmounted: move |e| focus(Some(e.data())),
                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); close(CloseReason::Escape) },
                    if draggable {
                        span { class: "grip", "aria-hidden": "true",
                            onpointerdown: move |e| { let p = pos(&e); last_pos.set((p, p)); drag.write().begin(p) },
                        }
                    }
                    header { class: "dh",
                        div { class: "dtt",
                            h2 { id: "{id}-t", class: "dt", "{title}" }
                            if let Some(d) = description.clone() { p { class: "dd", "{d}" } }
                        }
                        button { class: "oxyn-x", r#type: "button", "aria-label": "Close", onclick: move |_| close(CloseReason::CloseButton), Icon { name: "x", size: 14.0, stroke: 2.0 } }
                    }
                    div { class: "db", {children} }
                    if let Some(f) = footer.clone() {
                        footer { class: "df",
                            button { class: "oxyn-btn ghost sm", r#type: "button", onclick: move |_| close(CloseReason::Action), "Cancel" }
                            button { class: "oxyn-btn primary sm", r#type: "button", onclick: move |_| close(CloseReason::Action), "{f}" }
                        }
                    }
                }
                if closing() { span { class: "oxyn-timer", style: "animation-duration:230ms", onanimationend: move |_| { closing.set(false); drag.set(EdgeDrag::default()) } } }
            }
        }
    }
}

/// One row inside a [`SheetStack`] page.
#[derive(Clone, Debug, PartialEq)]
pub struct StackRow {
    /// Label.
    pub label: String,
    /// Quiet detail on the right.
    pub detail: Option<String>,
    /// Pushes this sheet id.
    pub push: Option<String>,
    /// A switch with this value.
    pub switch: Option<bool>,
}

impl StackRow {
    /// A row that opens another sheet.
    pub fn push(label: &str, detail: &str, to: &str) -> Self {
        Self {
            label: label.into(),
            detail: Some(detail.into()).filter(|d: &String| !d.is_empty()),
            push: Some(to.into()),
            switch: None,
        }
    }
    /// A switch row.
    pub fn switch(label: &str, on: bool) -> Self {
        Self {
            label: label.into(),
            detail: None,
            push: None,
            switch: Some(on),
        }
    }
    /// A plain row.
    pub fn text(label: &str, detail: &str) -> Self {
        Self {
            label: label.into(),
            detail: Some(detail.into()),
            push: None,
            switch: None,
        }
    }
}

/// One sheet of a [`SheetStack`].
#[derive(Clone, Debug, PartialEq)]
pub struct StackPage {
    /// Id, title, back label, dismissible.
    pub sheet: StackSheet,
    /// Line under the title.
    pub description: Option<String>,
    /// Rows.
    pub rows: Vec<StackRow>,
    /// Footer action (pinned under the body).
    pub footer: Option<String>,
}

impl StackPage {
    /// A page.
    pub fn new(id: &str, title: &str, description: &str, rows: Vec<StackRow>) -> Self {
        Self {
            sheet: StackSheet::new(id, title),
            description: Some(description.into()).filter(|d: &String| !d.is_empty()),
            rows,
            footer: None,
        }
    }
    /// With a footer action.
    pub fn footer(mut self, f: &str) -> Self {
        self.footer = Some(f.into());
        self
    }
}

/// Sheet stack: nested sheets with depth. A new sheet rises while the one below scales back 5 %,
/// dims and peeks above it; the back button names the sheet below, the close button reads "Close
/// all" when more than one is open; Escape closes the top one only. Drag the top sheet down (or
/// fling past 35 % / 450 px/s) to pop it. `auto` becomes stacked dialogs at or above `breakpoint`.
/// Oxyn plus: `side` mode (inspectors that stack sideways) and a breadcrumb.
#[component]
pub fn SheetStack(
    /// Sheets (rows that `push` open the next one).
    pages: Vec<StackPage>,
    /// Sheets open at first, bottom first.
    #[props(default)]
    default_stack: Vec<String>,
    /// `auto` · `sheet` · `dialog` · `side`.
    #[props(default = "auto".to_string())]
    mode: String,
    /// Auto switches to dialogs at this width px.
    #[props(default = 640.0)]
    breakpoint: f32,
    /// Inside the nearest positioned ancestor.
    #[props(default)]
    contained: bool,
    /// A button that opens the first page.
    #[props(default)]
    trigger_label: Option<String>,
    /// Oxyn plus: a breadcrumb of the open sheets.
    #[props(default)]
    breadcrumb: bool,
    /// Stack changed.
    #[props(default)]
    on_stack_change: EventHandler<Vec<String>>,
) -> Element {
    let mut m = use_signal(|| {
        let mut s = StackModel::new(pages.iter().map(|p| p.sheet.clone()).collect());
        s.mode = StackMode::from_id(&mode);
        s.breakpoint = breakpoint;
        for id in &default_stack {
            s.push(id);
        }
        s
    });
    let mut width = use_signal(|| 0.0f32);
    let mut last_y = use_signal(|| (0.0f32, 0.0f32));
    let changed = move || on_stack_change.call(m.peek().stack.ids().to_vec());
    // Mounted sheets by id: after a pop, focus returns to the sheet that is now on top (the
    // popped one held focus, so Escape would otherwise stop reaching the stack).
    let mut nodes = use_signal(std::collections::HashMap::<String, Rc<MountedData>>::new);
    // The sheet below is still `inert` until the pop re-renders, so focus moves in an effect
    // that runs after the DOM update rather than inside the handler.
    let mut want_focus = use_signal(|| false);
    use_effect(move || {
        if want_focus() {
            want_focus.set(false);
            let id = m.peek().open().last().map(|p| p.id.clone());
            focus(id.and_then(|id| nodes.peek().get(&id).cloned()));
        }
    });
    let mut refocus = move || want_focus.set(true);
    let st = m.read().clone();
    let w = width();
    let resolved = st
        .mode
        .resolve(if w > 0.0 { w } else { 400.0 }, st.breakpoint);
    let open = st.open();
    let n = open.len();
    let sheet_h = 300.0f32;
    let follow = if st.drag.active {
        (st.drag.offset / sheet_h).clamp(0.0, 1.0)
    } else {
        0.0
    };
    let first_id = pages
        .first()
        .map(|p| p.sheet.id.clone())
        .unwrap_or_default();
    let mode_cls = match resolved {
        StackMode::Dialog => "dialog",
        StackMode::Side => "side",
        _ => "sheet",
    };
    rsx! {
        if let Some(t) = trigger_label.clone() {
            button { class: "oxyn-btn secondary sm", r#type: "button", "aria-haspopup": "dialog",
                onclick: move |_| { m.write().push(&first_id); changed() },
                "{t}"
            }
        }
        div { class: cls(&["oxyn-layer stack", mode_cls, if contained { "contained" } else { "" }, if n == 0 { "empty" } else { "" }]),
            onmounted: move |e| { let d = e.data(); spawn(async move { if let Ok(r) = d.get_client_rect().await { width.set(r.size.width as f32) } }); },
            onpointermove: move |e| if m.peek().drag.active { let y = e.client_coordinates().y as f32; let (a, _) = *last_y.peek(); last_y.set((y, a)); m.write().drag.moved(y, 1.0, sheet_h) },
            onpointerup: move |_| if m.peek().drag.active {
                let (a, b) = *last_y.peek();
                let pop = m.write().drag.release((a - b) * 60.0, sheet_h);
                if pop { m.write().pop(); changed(); refocus() }
            },
            if n > 0 {
                div { class: "oxyn-scrim", onpointerdown: move |_| { m.write().close(); changed() } }
            }
            for (i, page) in open.iter().enumerate() {
                {
                    let depth = n - 1 - i;
                    let top = depth == 0;
                    let lv = st.level(depth, resolved, follow);
                    let p = pages.iter().find(|x| x.sheet.id == page.id).cloned();
                    let drag_off = if top && st.drag.active { st.drag.offset } else { 0.0 };
                    let (tf, origin) = match resolved {
                        StackMode::Side => (format!("transform:translateX({:.1}px) scale({:.3})", -lv.lift * 3.0, lv.scale), "left center"),
                        StackMode::Dialog => (format!("transform:translate(-50%, calc(-50% - {:.1}px + {:.1}px)) scale({:.3})", lv.lift, drag_off, lv.scale), "top center"),
                        _ => (format!("transform:translateY({:.1}px) scale({:.3})", drag_off - lv.lift, lv.scale), "top center"),
                    };
                    let back = if top { st.back_label() } else { None };
                    let close_label = st.close_label();
                    let tid = format!("oxyn-ss-{}", page.id);
                    let dismissible = page.dismissible;
                    rsx! {
                        div { key: "{page.id}", class: cls(&["oxyn-stack-sheet", if top { "top" } else { "under" }, if st.drag.active && top { "dragging" } else { "" }]),
                            style: "{tf};transform-origin:{origin};--dim:{lv.dim:.2};z-index:{10 + i}",
                            role: "dialog", "aria-modal": if top { "true" } else { "false" }, "aria-labelledby": "{tid}", "aria-hidden": if !top { "true" },
                            "inert": if !top { "true" }, tabindex: "-1",
                            onmounted: { let pid = page.id.clone(); move |e: MountedEvent| { nodes.write().insert(pid.clone(), e.data()); if top { focus(Some(e.data())) } } },
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); m.write().pop(); changed(); refocus() },
                            if resolved == StackMode::Sheet && dismissible {
                                span { class: "grab", "aria-hidden": "true",
                                    onpointerdown: move |e| { let y = e.client_coordinates().y as f32; last_y.set((y, y)); m.write().drag.begin(y) } }
                            }
                            header { class: "sh",
                                if let Some(b) = back.clone() {
                                    button { class: "oxyn-btn ghost xs back", r#type: "button", "aria-label": "Back to {b}", onclick: move |_| { m.write().pop(); changed(); refocus() },
                                        Icon { name: "chevron-left", size: 13.0, stroke: 2.0 } span { "{b}" }
                                    }
                                }
                                button { class: "oxyn-x", r#type: "button", "aria-label": "{close_label}", onclick: move |_| { m.write().close(); changed() }, Icon { name: "x", size: 14.0, stroke: 2.0 } }
                            }
                            if breadcrumb && top && n > 1 {
                                nav { class: "crumbs", "aria-label": "Open sheets",
                                    for (k, t) in st.trail().into_iter().enumerate() {
                                        if k > 0 { span { class: "sep", "aria-hidden": "true", "/" } }
                                        span { class: if k + 1 == n { "c cur" } else { "c" }, "{t}" }
                                    }
                                }
                            }
                            h2 { id: "{tid}", class: "st", "{page.title}" }
                            if let Some(p) = p.clone() {
                                if let Some(d) = p.description.clone() { p { class: "sd", "{d}" } }
                                div { class: "sb",
                                    for (ri, r) in p.rows.iter().cloned().enumerate() {
                                        {
                                            let to = r.push.clone();
                                            match (to, r.switch) {
                                                (Some(to), _) => rsx! {
                                                    button { key: "{ri}", class: "row push", r#type: "button", onclick: move |_| { m.write().push(&to); changed() },
                                                        span { class: "rl", "{r.label}" }
                                                        if let Some(d) = r.detail.clone() { span { class: "rd", "{d}" } }
                                                        Icon { name: "chevron-right", size: 13.0, stroke: 2.0 }
                                                    }
                                                },
                                                (None, Some(on)) => rsx! { div { key: "{ri}", class: "row", Switch { label: r.label.clone(), checked: on, size: "sm", placement: "end" } } },
                                                _ => rsx! {
                                                    div { key: "{ri}", class: "row",
                                                        span { class: "rl", "{r.label}" }
                                                        if let Some(d) = r.detail.clone() { span { class: "rd", "{d}" } }
                                                    }
                                                },
                                            }
                                        }
                                    }
                                }
                                if let Some(f) = p.footer.clone() {
                                    footer { class: "sf", button { class: "oxyn-btn primary sm full", r#type: "button", onclick: move |_| { m.write().pop(); changed() }, "{f}" } }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Bottom sheet: rests at detents (fractions of the viewport height), follows the finger 1:1,
/// rubber-bands past the tallest detent, and projects the release velocity to pick a detent or
/// dismiss. The backdrop dim follows the sheet's height. The grabber is a button: ↑ ↓ step
/// detents, Home / End jump, Enter / Space toggle; detent changes are announced.
#[component]
pub fn BottomSheet(
    /// Title.
    title: String,
    /// Quiet line.
    #[props(default)]
    description: Option<String>,
    /// Start open.
    #[props(default)]
    open: bool,
    /// A button that opens it.
    #[props(default)]
    trigger_label: Option<String>,
    /// Resting heights as fractions of the viewport (`[0.45, 0.92]`).
    #[props(default = vec![0.45, 0.92])]
    detents: Vec<f32>,
    /// Index into the sorted detents it opens at.
    #[props(default)]
    initial_detent: usize,
    /// Close button label.
    #[props(default = "Close".to_string())]
    close_label: String,
    /// Inside the nearest positioned ancestor (detents use its height).
    #[props(default)]
    contained: bool,
    /// Body.
    #[props(default)]
    children: Element,
    /// Open state changed.
    #[props(default)]
    on_open_change: EventHandler<bool>,
    /// Settled on a different detent.
    #[props(default)]
    on_detent_change: EventHandler<usize>,
) -> Element {
    let mut is_open = use_signal(|| open);
    let mut closing = use_signal(|| false);
    let mut m = use_signal(|| SheetModel::new(&detents, initial_detent, 640.0));
    let mut last_y = use_signal(|| (0.0f32, 0.0f32));
    let mut said = use_signal(String::new);
    let mut trig = use_signal(|| None::<Rc<MountedData>>);
    let id = format!("oxyn-bs-{}", slug(&title));
    let mut close = move || {
        if *is_open.peek() {
            is_open.set(false);
            closing.set(true);
            on_open_change.call(false);
            focus(trig.peek().clone());
        }
    };
    let mut settle = move |i: usize| {
        on_detent_change.call(i);
        said.set(m.peek().announce());
    };
    let st = m.read().clone();
    let shown = is_open() || closing();
    let dragging = st.drag_height.is_some();
    let h = st.height();
    rsx! {
        if let Some(t) = trigger_label.clone() {
            button { class: "oxyn-btn secondary sm", r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{is_open()}",
                onmounted: move |e| trig.set(Some(e.data())),
                onclick: move |_| { let i = initial_detent.min(m.peek().detents().len() - 1); m.write().index = i; is_open.set(true); closing.set(false); on_open_change.call(true) },
                "{t}"
            }
        }
        if shown {
            div { class: cls(&["oxyn-layer bsheet", if contained { "contained" } else { "" }, if closing() { "closing" } else { "" }]),
                onmounted: move |e| { let d = e.data(); spawn(async move { if let Ok(r) = d.get_client_rect().await { let v = r.size.height as f32; if v > 0.0 { m.write().viewport = v; } } }); },
                onpointermove: move |e| if m.peek().drag_height.is_some() { let y = e.client_coordinates().y as f32; let (a, _) = *last_y.peek(); last_y.set((y, a)); m.write().moved(y) },
                onpointerup: move |_| if m.peek().drag_height.is_some() {
                    let (a, b) = *last_y.peek();
                    match m.write().release((a - b) * 60.0) {
                        SheetRelease::Dismiss => close(),
                        SheetRelease::Detent(i) => settle(i),
                    }
                },
                div { class: "oxyn-scrim", style: "opacity:{st.dim() / 0.45:.3}", onpointerdown: move |_| close() }
                div { class: cls(&["oxyn-bsheet", if dragging { "dragging" } else { "" }]), style: "height:{h:.0}px",
                    role: "dialog", "aria-modal": "true", "aria-labelledby": "{id}-t", tabindex: "-1",
                    onmounted: move |e| focus(Some(e.data())),
                    onkeydown: move |e: KeyboardEvent| match e.key() {
                        Key::Escape => { e.stop_propagation(); close() }
                        Key::PageDown | Key::ArrowDown if !m.peek().expanded() => { e.prevent_default(); let i = m.write().key(DetentKey::Home); settle(i) }
                        _ => {}
                    },
                    button { class: "grabber", r#type: "button", "aria-label": "{st.grabber_label()}", "aria-expanded": "{st.expanded()}",
                        onpointerdown: move |e| { let y = e.client_coordinates().y as f32; last_y.set((y, y)); m.write().begin(y) },
                        onkeydown: move |e: KeyboardEvent| {
                            let k = match e.key() {
                                Key::ArrowUp => Some(DetentKey::Up),
                                Key::ArrowDown => Some(DetentKey::Down),
                                Key::Home => Some(DetentKey::Home),
                                Key::End => Some(DetentKey::End),
                                Key::Enter => Some(DetentKey::Toggle),
                                Key::Character(c) if c == " " => Some(DetentKey::Toggle),
                                _ => None,
                            };
                            if let Some(k) = k { e.prevent_default(); e.stop_propagation(); let i = m.write().key(k); settle(i) }
                        },
                        span { class: "bar" }
                    }
                    header { class: "dh",
                        onpointerdown: move |e| { let y = e.client_coordinates().y as f32; last_y.set((y, y)); m.write().begin(y) },
                        div { class: "dtt",
                            h2 { id: "{id}-t", class: "dt", "{title}" }
                            if let Some(d) = description.clone() { p { class: "dd", "{d}" } }
                        }
                        button { class: "oxyn-x", r#type: "button", "aria-label": "{close_label}", onpointerdown: move |e| e.stop_propagation(), onclick: move |_| close(), Icon { name: "x", size: 14.0, stroke: 2.0 } }
                    }
                    div { class: "db", {children} }
                    span { class: "sr", role: "status", "aria-live": "polite", "{said}" }
                }
                if closing() { span { class: "oxyn-timer", style: "animation-duration:230ms", onanimationend: move |_| closing.set(false) } }
            }
        }
    }
}

/// Handle for sending toasts from anywhere inside a [`Toaster`].
#[derive(Clone, Copy, PartialEq)]
pub struct Toasts {
    m: Signal<ToasterModel>,
    said: Signal<(String, bool)>,
}

impl Toasts {
    fn announce(mut self, title: &str, kind: ToastKind) {
        self.said.set((title.to_string(), kind == ToastKind::Error));
    }
    /// A toast; returns its id.
    pub fn show(
        mut self,
        title: &str,
        description: Option<&str>,
        kind: ToastKind,
        action: Option<&str>,
    ) -> u64 {
        self.announce(title, kind);
        self.m.write().add(
            title,
            ToastOptions {
                description: description.map(Into::into),
                kind,
                action: action.map(Into::into),
                duration: None,
            },
            ClockMs(0),
        )
    }
    /// A success toast.
    pub fn success(self, title: &str, description: Option<&str>) -> u64 {
        self.show(title, description, ToastKind::Success, None)
    }
    /// An error toast.
    pub fn error(self, title: &str, description: Option<&str>) -> u64 {
        self.show(title, description, ToastKind::Error, None)
    }
    /// A loading toast for work in flight; settle it with [`Toasts::resolve`].
    pub fn promise(mut self, title: &str, description: Option<&str>) -> u64 {
        self.announce(title, ToastKind::Loading);
        self.m.write().promise(title, description, ClockMs(0))
    }
    /// Settles a promise toast (it morphs; its timer starts).
    pub fn resolve(mut self, id: u64, ok: bool, title: &str, description: Option<&str>) {
        self.announce(
            title,
            if ok {
                ToastKind::Success
            } else {
                ToastKind::Error
            },
        );
        self.m
            .write()
            .resolve(id, ok, title, description, ClockMs(0));
    }
    /// Closes one toast.
    pub fn dismiss(mut self, id: u64) {
        self.m.write().dismiss(id, ClockMs(0));
    }
    /// Closes all.
    pub fn dismiss_all(mut self) {
        self.m.write().dismiss_all(ClockMs(0));
    }
}

/// The toast handle from the nearest [`Toaster`].
pub fn use_toasts() -> Toasts {
    use_context::<Toasts>()
}

/// A toast to show on mount (demos, restored sessions).
#[derive(Clone, Debug, PartialEq)]
pub struct SeedToast {
    /// Title.
    pub title: String,
    /// Quiet line.
    pub description: Option<String>,
    /// `default` · `success` · `error` · `info` · `warning` · `loading`.
    pub kind: String,
    /// Action label.
    pub action: Option<String>,
}

impl SeedToast {
    /// A seed.
    pub fn new(title: &str, description: &str, kind: &str) -> Self {
        Self {
            title: title.into(),
            description: Some(description.into()).filter(|d: &String| !d.is_empty()),
            kind: kind.into(),
            action: None,
        }
    }
    /// With an action.
    pub fn action(mut self, a: &str) -> Self {
        self.action = Some(a.into());
        self
    }
}

fn toast_icon(kind: ToastKind) -> Element {
    match kind {
        ToastKind::Loading => {
            rsx! { svg { class: "ti spin", view_box: "0 0 16 16", width: "16", height: "16", circle { cx: "8", cy: "8", r: "6", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_dasharray: "26 38", stroke_linecap: "round" } } }
        }
        ToastKind::Success => {
            rsx! { svg { class: "ti ok", view_box: "0 0 16 16", width: "16", height: "16", circle { cx: "8", cy: "8", r: "6", fill: "none", stroke: "currentColor", stroke_width: "1.6" } path { class: "mk", d: "M5.3 8.2 7.2 10l3.5-3.8", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", path_length: "1" } } }
        }
        ToastKind::Error => {
            rsx! { svg { class: "ti err", view_box: "0 0 16 16", width: "16", height: "16", circle { cx: "8", cy: "8", r: "6", fill: "none", stroke: "currentColor", stroke_width: "1.6" } path { class: "mk", d: "M6 6l4 4M10 6l-4 4", fill: "none", stroke: "currentColor", stroke_width: "1.6", stroke_linecap: "round", path_length: "1" } } }
        }
        ToastKind::Warning => {
            rsx! { span { class: "ti warn", Icon { name: "alert", size: 16.0 } } }
        }
        ToastKind::Info => rsx! { span { class: "ti", Icon { name: "info", size: 16.0 } } },
        ToastKind::Default => rsx! {},
    }
}

/// Toaster: calm toasts from anywhere (via [`use_toasts`]) into one stack. At most `visible` show;
/// older ones wait with a paused timer. Collapsed, the ones behind move back, shrink and peek;
/// hover, focus or `expand` fans the stack out. Timers (`duration` ms, 0 = until dismissed) pause
/// while the pointer or focus is in the stack, and a hairline runs down with them. Promise toasts
/// morph from a spinner to a check or cross. Swipe sideways to dismiss; Escape closes the focused
/// toast and moves to the next; F8 (inside the wrapped app) focuses the newest.
#[component]
pub fn Toaster(
    /// `top-left` · `top-center` · `top-right` · `bottom-left` · `bottom-center` · `bottom-right`.
    #[props(default = "bottom-right".to_string())]
    position: String,
    /// Positioned inside the nearest positioned ancestor (20 px offset).
    #[props(default)]
    inline: bool,
    /// Time on screen ms (0 = until dismissed).
    #[props(default = 4000)]
    duration: u32,
    /// Toasts visible at once.
    #[props(default = 3)]
    visible: usize,
    /// Always fanned out.
    #[props(default)]
    expand: bool,
    /// Distance from the edges px (24, inline 20).
    #[props(default)]
    offset: Option<f32>,
    /// Gap when fanned out px.
    #[props(default = 10.0)]
    gap: f32,
    /// Key that focuses the newest toast.
    #[props(default = "F8".to_string())]
    hotkey: String,
    /// Timer hairline.
    #[props(default = true)]
    hairline: bool,
    /// Close button on hover and focus.
    #[props(default = true)]
    close_button: bool,
    /// Region name.
    #[props(default = "Notifications".to_string())]
    label: String,
    /// Toasts shown on mount.
    #[props(default)]
    seed: Vec<SeedToast>,
    /// The app (calls `use_toasts()`).
    #[props(default)]
    children: Element,
    /// An action was pressed (toast id, label).
    #[props(default)]
    on_action: EventHandler<(u64, String)>,
) -> Element {
    let pos = ToastPosition::from_id(&position);
    let mut m = use_signal(|| {
        let mut t = ToasterModel::new();
        t.position = pos;
        t.visible = visible.max(1);
        t.expand = expand;
        t.gap = gap;
        t.offset = offset.unwrap_or(if inline { 20.0 } else { 24.0 });
        t.hotkey = Some(hotkey.clone()).filter(|h| !h.is_empty());
        t.label = label.clone();
        t.duration = (duration > 0).then(|| std::time::Duration::from_millis(u64::from(duration)));
        for s in &seed {
            t.add(
                &s.title,
                ToastOptions {
                    description: s.description.clone(),
                    kind: ToastKind::from_id(&s.kind),
                    action: s.action.clone(),
                    duration: None,
                },
                ClockMs(0),
            );
        }
        t
    });
    let said = use_signal(|| (String::new(), false));
    let api = use_context_provider(|| Toasts { m, said });
    let _ = api;
    let mut heights = use_signal(std::collections::BTreeMap::<u64, f32>::new);
    let mut nodes = use_signal(std::collections::BTreeMap::<u64, Rc<MountedData>>::new);
    let mut swipe = use_signal(|| None::<(u64, f32, f32)>);
    let st = m.read().clone();
    if m.peek().visible != visible.max(1) || m.peek().expand != expand {
        let mut mm = m.write();
        mm.visible = visible.max(1);
        mm.expand = expand;
    }
    let all = st.toasts().to_vec();
    let live: Vec<u64> = st.live().iter().map(|t| t.id).collect();
    let hs: Vec<f32> = live
        .iter()
        .map(|id| heights.read().get(id).copied().unwrap_or(64.0))
        .collect();
    let slots = st.layout(&hs);
    let extent = st.extent(&hs);
    let expanded = st.expanded();
    let top = pos.top();
    let sign = if top { 1.0 } else { -1.0 };
    let off = st.offset;
    let edge = if top {
        format!("top:{off}px")
    } else {
        format!("bottom:{off}px")
    };
    let hot = hotkey.clone();
    let (msg, err) = said();
    rsx! {
        div { class: "oxyn-toaster-app",
            onkeydown: move |e: KeyboardEvent| {
                let k = e.key().to_string();
                if !hot.is_empty() && (k == hot || e.code().to_string() == hot) {
                    e.prevent_default();
                    let newest = m.peek().newest();
                    if let Some(id) = newest { m.write().set_hovered(true, ClockMs(0)); focus(nodes.peek().get(&id).cloned()); }
                }
            },
            {children}
            section { class: cls(&["oxyn-toaster", pos.id(), if inline { "inline" } else { "" }, if expanded { "expanded" } else { "" }]),
                "aria-label": "{st.region_label()}", style: "{edge};--extent:{extent:.0}px",
                onpointerenter: move |_| m.write().set_hovered(true, ClockMs(0)),
                onpointerleave: move |_| { if swipe.peek().is_none() { m.write().set_hovered(false, ClockMs(0)) } },
                onfocusin: move |_| m.write().set_hovered(true, ClockMs(0)),
                ol { class: "list",
                    for t in all.iter().cloned() {
                        {
                            let id = t.id;
                            let li = live.iter().position(|x| *x == id);
                            let slot = li.map(|i| slots[i]);
                            let closing = t.closing_at.is_some();
                            let (y, sc, op, shown) = match slot { Some(s) => (s.y, s.scale, s.opacity, s.shown), None => (0.0, 0.96, 0.0, false) };
                            let running = shown && !expanded && !closing && t.duration.is_some();
                            let dur = t.duration.map(|d| d.as_millis()).unwrap_or(0);
                            let style = format!("transform:translate({:.1}px,{:.1}px) scale({sc:.3});opacity:{op:.2};z-index:{}", t.swipe, sign * y, 100 - li.map(|i| live.len() - i).unwrap_or(0));
                            let kind = t.kind;
                            let action = t.action.clone();
                            let ver = t.version;
                            let play = if running { "running" } else { "paused" };
                            rsx! {
                                li { key: "{id}", class: cls(&["oxyn-toast", kind.id(), if closing { "closing" } else { "" }, if shown { "" } else { "waiting" }]),
                                    style: "{style}", tabindex: if shown && !closing { "0" } else { "-1" }, "aria-hidden": if !shown || closing { "true" },
                                    "aria-label": "{t.title}",
                                    onmounted: move |e| { let d = e.data(); nodes.write().insert(id, d.clone()); spawn(async move { if let Ok(r) = d.get_client_rect().await { heights.write().insert(id, r.size.height as f32); } }); },
                                    onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape {
                                        e.stop_propagation();
                                        let next = m.write().escape(Some(id), ClockMs(0));
                                        match next { Some(n) => focus(nodes.peek().get(&n).cloned()), None => m.write().set_hovered(false, ClockMs(0)) }
                                    },
                                    onpointerdown: move |e| swipe.set(Some((id, e.client_coordinates().x as f32, 0.0))),
                                    onpointermove: move |e| { let cur = *swipe.peek(); if let Some((sid, x0, _)) = cur && sid == id { let dx = e.client_coordinates().x as f32 - x0; m.write().swipe(id, dx); swipe.set(Some((sid, x0, dx))); } },
                                    onpointerup: move |_| if let Some((sid, _, _)) = swipe.take() && sid == id { m.write().swipe_end(id, 0.0, 356.0, ClockMs(0)); },
                                    div { class: "tb",
                                        for _k in std::iter::once(format!("i{ver}")) { div { key: "{_k}", class: "ic", {toast_icon(kind)} } }
                                        for _k in std::iter::once(format!("t{ver}")) { div { key: "{_k}", class: "tx",
                                            div { class: "tt", "{t.title}" }
                                            if let Some(d) = t.description.clone() { div { class: "td", "{d}" } }
                                        } }
                                        if let Some(a) = action.clone() {
                                            button { class: "oxyn-btn secondary xs act", r#type: "button",
                                                onclick: move |_| { on_action.call((id, a.clone())); m.write().dismiss(id, ClockMs(0)); },
                                                "{t.action.clone().unwrap_or_default()}"
                                            }
                                        }
                                        if close_button {
                                            button { class: "oxyn-x close", r#type: "button", "aria-label": "Close notification", onclick: move |_| { m.write().dismiss(id, ClockMs(0)); }, Icon { name: "x", size: 12.0, stroke: 2.0 } }
                                        }
                                    }
                                    if dur > 0 && !closing {
                                        span { key: "h{ver}", class: if hairline { "hair" } else { "hair off" }, "aria-hidden": "true",
                                            style: "animation-duration:{dur}ms;animation-play-state:{play}",
                                            onanimationend: move |_| { m.write().dismiss(id, ClockMs(0)); } }
                                    }
                                    if closing { span { class: "oxyn-timer", style: "animation-duration:210ms", onanimationend: move |_| { m.write().remove(id); heights.write().remove(&id); nodes.write().remove(&id); } } }
                                }
                            }
                        }
                    }
                }
                span { class: "sr", role: if err { "alert" } else { "status" }, "aria-live": if err { "assertive" } else { "polite" }, "{msg}" }
            }
        }
    }
}

/// Notification stack: notice cards stacked behind each other; dismiss the front one to reveal
/// the next. Only the front card is focusable and read out. Oxyn plus: `depth` (cards drawn),
/// priority ordering, an Undo for the last dismissed notice, background patterns.
#[component]
pub fn NotificationStack(
    /// Notices (first = front).
    notices: Vec<Notice>,
    /// Cards drawn.
    #[props(default = 3)]
    depth: usize,
    /// Order by priority.
    #[props(default)]
    by_priority: bool,
    /// Offer Undo after a dismiss.
    #[props(default = true)]
    undo: bool,
    /// Region name.
    #[props(default = "Notices".to_string())]
    label: String,
    /// Dismissed (id).
    #[props(default)]
    on_dismiss: EventHandler<String>,
    /// Action pressed (id).
    #[props(default)]
    on_action: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| NoticeModel::new(notices.clone()));
    let mut leaving = use_signal(|| None::<String>);
    if m.peek().depth != depth.max(1) || m.peek().by_priority != by_priority {
        let mut w = m.write();
        w.depth = depth.max(1);
        w.by_priority = by_priority;
    }
    let st = m.read().clone();
    let shown: Vec<Notice> = st.shown().into_iter().cloned().collect();
    let n = shown.len();
    rsx! {
        section { class: "oxyn-notices", "aria-label": "{label}", style: "--n:{n}",
            if n == 0 {
                div { class: "done", role: "status", Icon { name: "check", size: 14.0, stroke: 2.0 } span { "You're all caught up" }
                    if undo && st.can_undo() { button { class: "oxyn-btn ghost xs", r#type: "button", onclick: move |_| { m.write().undo(); }, "Undo" } }
                }
            }
            for (k, nt) in shown.iter().cloned().enumerate().rev() {
                {
                    let front = k == 0;
                    let id = nt.id.clone();
                    let id2 = id.clone();
                    let out = leaving.read().as_deref() == Some(id.as_str());
                    let tid = format!("oxyn-nt-{}", slug(&id));
                    let pat = nt.pattern.clone().unwrap_or_default();
                    rsx! {
                        article { key: "{id}", class: cls(&["oxyn-notice", if front { "front" } else { "back" }, if out { "out" } else { "" }, if pat.is_empty() { "" } else { "pat" }, &format!("pat-{pat}")]),
                            style: "--k:{k}", "aria-labelledby": "{tid}", "aria-hidden": if !front { "true" }, "inert": if !front { "true" },
                            div { class: "nh",
                                if let Some(b) = nt.badge.clone() { span { class: "badge", "{b}" } }
                                if front && st.len() > 1 { span { class: "count", "{st.position_label()}" } }
                                button { class: "oxyn-x", r#type: "button", "aria-label": "Dismiss {nt.title}", onclick: move |_| leaving.set(Some(id.clone())), Icon { name: "x", size: 12.0, stroke: 2.0 } }
                            }
                            h3 { id: "{tid}", class: "nt", "{nt.title}" }
                            p { class: "nx", "{nt.text}" }
                            if let Some(a) = nt.action.clone() {
                                button { class: "oxyn-btn secondary xs na", r#type: "button", onclick: move |_| on_action.call(id2.clone()),
                                    "{a}" if nt.external { Icon { name: "external", size: 12.0 } }
                                }
                            }
                            if out { span { class: "oxyn-timer", style: "animation-duration:220ms", onanimationend: move |_| { if let Some(d) = m.write().dismiss_front() { on_dismiss.call(d.id) } leaving.set(None) } } }
                        }
                    }
                }
            }
            if n > 0 && undo && st.can_undo() {
                button { class: "oxyn-btn ghost xs undo", r#type: "button", onclick: move |_| { m.write().undo(); }, Icon { name: "undo", size: 12.0 } "Undo" }
            }
        }
    }
}

/// Alert: a persistent in-flow message with a tone icon, a title and details. Danger is
/// `role=alert`; the rest `role=status`. Dismissing collapses its height so content below closes
/// the gap; tone changes morph the icon, copy changes rise in. Oxyn plus: `agent` tone, actions
/// (Retry, Show logs), collapsible details, `soft` · `outline` · `solid` styles.
#[component]
pub fn Alert(
    /// Headline.
    title: String,
    /// `info` · `success` · `warning` · `danger` · `agent`.
    #[props(default = "info".to_string())]
    tone: String,
    /// Details under the title.
    #[props(default)]
    body: Option<String>,
    /// Shown (presence).
    #[props(default = true)]
    open: bool,
    /// Dismiss button.
    #[props(default)]
    dismissible: bool,
    /// Action buttons.
    #[props(default)]
    actions: Vec<String>,
    /// Collapsible technical details.
    #[props(default)]
    details: Option<String>,
    /// `soft` · `outline` · `solid`.
    #[props(default = "soft".to_string())]
    variant: String,
    /// Dismissed.
    #[props(default)]
    on_dismiss: EventHandler<()>,
    /// Action pressed.
    #[props(default)]
    on_action: EventHandler<String>,
) -> Element {
    let mut m = use_signal(|| {
        let mut a = AlertModel::new(AlertTone::from_id(&tone), &title);
        a.body = body.clone();
        a.dismissible = dismissible;
        a.details = details.clone();
        a
    });
    {
        let cur = m.peek().clone();
        let t = AlertTone::from_id(&tone);
        if cur.tone != t || cur.title != title || cur.body != body {
            m.write().set(t, &title, body.as_deref());
        }
        if !open && cur.open {
            m.write().open = false;
        } else if open && !cur.open {
            m.write().open = true;
        }
    }
    let a = m.read().clone();
    let tid = format!("oxyn-al-{}", slug(&a.title));
    rsx! {
        div { class: cls(&["oxyn-alert-wrap", if a.open { "open" } else { "" }]),
            div { class: cls(&["oxyn-alert", a.tone.id(), &variant]), role: "{a.tone.role()}", "aria-labelledby": "{tid}",
                for _k in std::iter::once(a.tone.id().to_string()) { span { key: "{_k}", class: "ai", "aria-hidden": "true", Icon { name: a.tone.icon(), size: 16.0 } } }
                for _k in std::iter::once(a.version.to_string()) { div { key: "{_k}", class: "ab",
                    div { id: "{tid}", class: "at", "{a.title}" }
                    if let Some(b) = a.body.clone() { div { class: "ad", "{b}" } }
                    if !actions.is_empty() || a.details.is_some() {
                        div { class: "aa",
                            for x in actions.iter().cloned() {
                                {
                                    let lbl = x.clone();
                                    let logs = x == "Show logs" && a.details.is_some();
                                    rsx! {
                                        button { key: "{x}", class: "oxyn-btn secondary xs", r#type: "button", "aria-expanded": if logs { "{a.expanded}" },
                                            onclick: move |_| { if logs { m.write().toggle_details(); } else { on_action.call(lbl.clone()) } },
                                            if x == "Retry" { Icon { name: "retry", size: 12.0 } }
                                            if logs { if a.expanded { "Hide logs" } else { "Show logs" } } else { "{x}" }
                                        }
                                    }
                                }
                            }
                            if a.details.is_some() && !actions.iter().any(|x| x == "Show logs") {
                                button { class: "oxyn-btn ghost xs", r#type: "button", "aria-expanded": "{a.expanded}", onclick: move |_| { m.write().toggle_details(); },
                                    if a.expanded { "Hide details" } else { "Details" }
                                }
                            }
                        }
                    }
                    if let Some(d) = a.details.clone() {
                        div { class: cls(&["adet", if a.expanded { "open" } else { "" }]), pre { "{d}" } }
                    }
                } }
                if a.dismissible {
                    button { class: "oxyn-x", r#type: "button", "aria-label": "{a.dismiss_label()}", onclick: move |_| { m.write().dismiss(); on_dismiss.call(()) }, Icon { name: "x", size: 12.0, stroke: 2.0 } }
                }
            }
        }
    }
}

/// Card: a content card with media, byline and status that can open into a quick look. Hover lifts
/// it 2 px and zooms the media; status changes roll in word by word. Oxyn plus: a status edge (a
/// beam while an agent works on it), a selectable card and a compact `row` layout.
#[component]
pub fn Card(
    /// Title (opens the quick look when `details` is set).
    title: String,
    /// Line under the title.
    #[props(default)]
    description: Option<String>,
    /// Media: an image URL, or a pattern (`grid`, `dots`, `waves`).
    #[props(default)]
    media: Option<String>,
    /// Owner name (avatar + byline).
    #[props(default)]
    meta: Option<String>,
    /// Status under the byline (rolls in when it changes).
    #[props(default)]
    status: Option<String>,
    /// Footer action label.
    #[props(default)]
    action: Option<String>,
    /// Quick look text.
    #[props(default)]
    details: Option<String>,
    /// Quick look open.
    #[props(default)]
    open: bool,
    /// Oxyn plus: `none` · `working` · `attention` · `done` · `error`.
    #[props(default = "none".to_string())]
    edge: String,
    /// Oxyn plus: selectable.
    #[props(default)]
    selectable: bool,
    /// Oxyn plus: `card` · `row`.
    #[props(default = "card".to_string())]
    layout: String,
    /// Width px (a definite width keeps the grid cell steady).
    #[props(default = 300.0)]
    width: f32,
    /// Extra body.
    #[props(default)]
    children: Element,
    /// Quick look changed.
    #[props(default)]
    on_open_change: EventHandler<bool>,
    /// Action pressed.
    #[props(default)]
    on_action: EventHandler<()>,
    /// Selection changed.
    #[props(default)]
    on_select: EventHandler<bool>,
) -> Element {
    let mut m = use_signal(|| {
        let mut c = CardModel::new(status.as_deref());
        c.open = open && details.is_some();
        c
    });
    if m.peek().status != status
        && let Some(s) = status.as_deref()
    {
        m.write().set_status(s);
    }
    let mut title_btn = use_signal(|| None::<Rc<MountedData>>);
    let c = m.read().clone();
    let e = CardEdge::from_id(&edge);
    let tid = format!("oxyn-card-{}", slug(&title));
    let has = details.is_some();
    let words = c.status_words();
    let initials: String = meta
        .as_deref()
        .unwrap_or("")
        .split_whitespace()
        .filter_map(|w| w.chars().next())
        .take(2)
        .collect();
    let row = layout == "row";
    let media_el = media.clone().map(|md| {
        if md.starts_with("http") || md.starts_with('/') {
            rsx! { div { class: "cm", img { src: "{md}", alt: "", loading: "lazy" } } }
        } else {
            rsx! { div { class: "cm pat-{md}", "aria-hidden": "true" } }
        }
    });
    rsx! {
        article { class: cls(&["oxyn-card", if row { "row" } else { "" }, &format!("edge-{}", e.id()), if c.selected { "selected" } else { "" }]),
            style: if row { "width:min(100%, {width.max(420.0)}px)".to_string() } else { format!("width:{width}px;max-width:100%") }, "aria-labelledby": "{tid}",
            if e == CardEdge::Working { span { class: "beam", "aria-hidden": "true" } }
            if let Some(md) = media_el { {md} }
            div { class: "cb",
                if selectable {
                    button { class: cls(&["sel", if c.selected { "on" } else { "" }]), r#type: "button", role: "checkbox", "aria-checked": "{c.selected}", "aria-label": "Select {title}",
                        onclick: move |_| { let s = m.write().toggle_selected(); on_select.call(s) },
                        if c.selected { Icon { name: "check", size: 11.0, stroke: 2.4 } }
                    }
                }
                h3 { id: "{tid}", class: "ct",
                    if has {
                        button { class: "tbtn", r#type: "button", "aria-haspopup": "dialog", "aria-expanded": "{c.open}",
                            onmounted: move |e| title_btn.set(Some(e.data())),
                            onclick: move |_| { m.write().open_quick_look(true); on_open_change.call(true) },
                            "{title}"
                        }
                    } else { "{title}" }
                }
                if let Some(d) = description.clone() { p { class: "cd", "{d}" } }
                {children}
            }
            if meta.is_some() || action.is_some() {
                footer { class: "cf",
                    if let Some(who) = meta.clone() {
                        span { class: "av", "aria-hidden": "true", "{initials}" }
                        div { class: "by",
                            span { class: "who", "{who}" }
                            if status.is_some() {
                                span { class: "cs", role: "status", "aria-live": "polite",
                                    for (wi, w) in words.iter().enumerate() {
                                        span { key: "{wi}-{w.text}", class: if w.changed { "w roll" } else { "w" }, style: "--o:{w.order}", "{w.text} " }
                                    }
                                }
                            }
                        }
                    }
                    if let Some(a) = action.clone() { button { class: "oxyn-btn secondary xs ca", r#type: "button", onclick: move |_| on_action.call(()), "{a}" } }
                }
            }
            if c.open {
                if let Some(d) = details.clone() {
                    div { class: "oxyn-layer contained quick",
                        div { class: "oxyn-scrim", onpointerdown: move |_| { m.write().close(); on_open_change.call(false); focus(title_btn.peek().clone()) } }
                        div { class: "oxyn-dlg sm look", role: "dialog", "aria-modal": "true", "aria-labelledby": "{tid}-q", tabindex: "-1",
                            onmounted: move |e| focus(Some(e.data())),
                            onkeydown: move |e: KeyboardEvent| if e.key() == Key::Escape { e.stop_propagation(); m.write().close(); on_open_change.call(false); focus(title_btn.peek().clone()) },
                            if let Some(md) = media.clone() { div { class: "cm pat-{md}", "aria-hidden": "true" } }
                            header { class: "dh",
                                div { class: "dtt", h2 { id: "{tid}-q", class: "dt", "{title}" } if let Some(who) = meta.clone() { p { class: "dd", "{who}" } } }
                                button { class: "oxyn-x", r#type: "button", "aria-label": "Close", onclick: move |_| { m.write().close(); on_open_change.call(false); focus(title_btn.peek().clone()) }, Icon { name: "x", size: 14.0, stroke: 2.0 } }
                            }
                            div { class: "db look-body", "{d}" }
                        }
                    }
                }
            }
        }
    }
}

/// A demo message for [`ScrollArea`]'s stick-to-bottom transcript.
#[derive(Clone, Debug, PartialEq)]
pub struct Line {
    /// Who.
    pub who: String,
    /// Text.
    pub text: String,
}

/// Scroll area: native scrolling with thin overlay scrollbars that show while scrolling or on
/// hover and fade after `hide_delay`, and edge fades that grow with the content left beyond each
/// edge; pressing the track pages 90 % of the viewport. Oxyn plus: `stick_to_bottom` follows new
/// content while the reader is at the bottom, lets go when they scroll up, counts what arrives
/// and offers a "Jump to latest" pill; `stream` appends `lines` on a timer to show it.
#[component]
pub fn ScrollArea(
    /// `vertical` · `horizontal` · `both`.
    #[props(default = "vertical".to_string())]
    orientation: String,
    /// Edge fade px (0 = off).
    #[props(default = 28.0)]
    fade: f32,
    /// `auto` · `always` · `hidden`.
    #[props(default = "auto".to_string())]
    scrollbars: String,
    /// Scrollbars linger ms.
    #[props(default = 900)]
    hide_delay: u32,
    /// Viewport height px.
    #[props(default = 240.0)]
    height: f32,
    /// Viewport width px.
    #[props(default = 360.0)]
    width: f32,
    /// Accessible name (the viewport becomes a region).
    #[props(default)]
    label: Option<String>,
    /// `scroll-snap-type` for the viewport.
    #[props(default)]
    snap: Option<String>,
    /// Oxyn plus: follow new content.
    #[props(default)]
    stick_to_bottom: bool,
    /// Oxyn plus: transcript lines (with `stream`, appended one by one).
    #[props(default)]
    lines: Vec<Line>,
    /// Append a line every `stream_ms`.
    #[props(default)]
    stream: bool,
    /// Stream interval ms.
    #[props(default = 1400)]
    stream_ms: u32,
    /// Static content.
    #[props(default)]
    children: Element,
    /// Edges changed (top, bottom, left, right).
    #[props(default)]
    on_edge_change: EventHandler<(bool, bool, bool, bool)>,
) -> Element {
    let mut met = use_signal(Metrics::default);
    let mut stick = use_signal(StickToBottom::default);
    let mut shown_n = use_signal(|| {
        if stream {
            lines.len().min(4)
        } else {
            lines.len()
        }
    });
    let mut tick = use_signal(|| 0u32);
    let mut vp = use_signal(|| None::<Rc<MountedData>>);
    let mut hot = use_signal(|| 0u32);
    let mut last_edges = use_signal(|| (false, false, false, false));
    let measure = move || {
        if let Some(d) = vp.peek().clone() {
            spawn(async move {
                let (Ok(off), Ok(sz), Ok(r)) = (
                    d.get_scroll_offset().await,
                    d.get_scroll_size().await,
                    d.get_client_rect().await,
                ) else {
                    return;
                };
                let mm = Metrics {
                    top: off.y as f32,
                    left: off.x as f32,
                    height: sz.height as f32,
                    width: sz.width as f32,
                    view_h: r.size.height as f32,
                    view_w: r.size.width as f32,
                };
                met.set(mm);
            });
        }
    };
    let to_bottom = move || {
        if let Some(d) = vp.peek().clone() {
            spawn(async move {
                if let (Ok(sz), Ok(r)) = (d.get_scroll_size().await, d.get_client_rect().await) {
                    let y = (sz.height - r.size.height).max(0.0);
                    let _ = d
                        .scroll(
                            dioxus::html::geometry::PixelsVector2D::new(0.0, y),
                            ScrollBehavior::Smooth,
                        )
                        .await;
                }
            });
        }
    };
    let mm = met();
    let e = edges(&mm);
    let (ft, fb, fl, fr) = fades(&mm, fade);
    let vert = orientation != "horizontal";
    let horiz = orientation != "vertical";
    let (tl, to) = thumb(mm.top, mm.height, mm.view_h, (height - 8.0).max(8.0));
    let (hl, ho) = thumb(mm.left, mm.width, mm.view_w, (width - 8.0).max(8.0));
    let sb = scrollbars.clone();
    let st = stick();
    let n = shown_n();
    let total = lines.len();
    let ox = if horiz { "auto" } else { "hidden" };
    let oy = if vert { "auto" } else { "hidden" };
    let snap_s = snap
        .clone()
        .map(|s| format!(";scroll-snap-type:{s}"))
        .unwrap_or_default();
    rsx! {
        div { class: cls(&["oxyn-scroll", &orientation, &format!("sb-{sb}"), if hot() > 0 { "active" } else { "" }]),
            style: "width:{width}px;max-width:100%;height:{height}px;--ft:{ft:.1}px;--fb:{fb:.1}px;--fl:{fl:.1}px;--fr:{fr:.1}px;--hide:{hide_delay}ms",
            div { class: "vp", tabindex: "0", role: if label.is_some() { "region" }, "aria-label": if let Some(l) = &label { "{l}" },
                style: "overflow-x:{ox};overflow-y:{oy}{snap_s}",
                onmounted: move |e| { vp.set(Some(e.data())); measure(); if stick_to_bottom { to_bottom() } },
                onscroll: move |ev| {
                    let mm = Metrics { top: ev.scroll_top() as f32, left: ev.scroll_left() as f32, height: ev.scroll_height() as f32, width: ev.scroll_width() as f32, view_h: ev.client_height() as f32, view_w: ev.client_width() as f32 };
                    met.set(mm);
                    if stick_to_bottom { stick.write().scrolled(&mm); }
                    let e = edges(&mm);
                    let t = (e.top, e.bottom, e.left, e.right);
                    if t != *last_edges.peek() { last_edges.set(t); on_edge_change.call(t); }
                    hot += 1;
                },
                if !lines.is_empty() {
                    div { class: "transcript",
                        for (i, l) in lines.iter().take(n).enumerate() {
                            div { key: "{i}", class: if l.who == "You" { "msg me" } else { "msg" },
                                span { class: "who", "{l.who}" }
                                p { "{l.text}" }
                            }
                        }
                    }
                }
                {children}
            }
            if fade > 0.0 { span { class: "fd", "aria-hidden": "true" } }
            if vert && e.top || vert && e.bottom {
                div { class: "trk y", "aria-hidden": "true",
                    onpointerdown: move |ev| {
                        let at = ev.element_coordinates().y as f32;
                        let m0 = *met.peek();
                        let target = oxyn_core::primitives::scroll::page_toward(m0.top, m0.height, m0.view_h, (height - 8.0).max(8.0), at);
                        if let Some(d) = vp.peek().clone() { spawn(async move { let _ = d.scroll(dioxus::html::geometry::PixelsVector2D::new(m0.left as f64, target as f64), ScrollBehavior::Smooth).await; }); }
                    },
                    span { class: "th", style: "height:{tl:.1}px;transform:translateY({to:.1}px)" }
                }
            }
            if horiz && (e.left || e.right) {
                div { class: "trk x", "aria-hidden": "true",
                    span { class: "th", style: "width:{hl:.1}px;transform:translateX({ho:.1}px)" }
                }
            }
            if hot() > 0 { for _k in std::iter::once(format!("{hot}")) { span { key: "{_k}", class: "oxyn-timer", style: "animation-duration:{hide_delay}ms", onanimationend: move |_| hot.set(0) } } }
            if stick_to_bottom && st.pill() {
                button { class: "oxyn-btn secondary xs pill", r#type: "button", onclick: move |_| { let m0 = *met.peek(); stick.write().jump(&m0); to_bottom() },
                    Icon { name: "chevron", size: 12.0, stroke: 2.0 } "{st.pill_label()}"
                }
            }
            if stream && n < total {
                for _k in std::iter::once(format!("s{tick}")) { span { key: "{_k}", class: "oxyn-timer", style: "animation-duration:{stream_ms}ms",
                    onanimationend: move |_| {
                        shown_n += 1;
                        tick += 1;
                        let m0 = *met.peek();
                        if stick_to_bottom {
                            match stick.write().content_changed(&m0, 1) { Stick::ScrollTo(_) => to_bottom(), Stick::Stay => {} }
                        }
                        measure();
                    }
                } }
            }
        }
    }
}

/// Pagination: previous and next around a sliding five-page window with a travelling current
/// mark; when the window shifts the numbers slide like a belt. Oxyn plus: `ellipsis` layout with
/// first and last, `cursor` mode (no total), `load-more`, a page-size picker with a range label.
#[component]
pub fn Pagination(
    /// Current page (1-based).
    #[props(default = 1)]
    page: usize,
    /// Total pages (ignored with `total`).
    #[props(default = 12)]
    count: usize,
    /// `window` · `ellipsis` · `cursor` · `load-more`.
    #[props(default = "window".to_string())]
    mode: String,
    /// Landmark name.
    #[props(default = "Pagination".to_string())]
    label: String,
    /// Items in total (adds the range label; pages follow the page size).
    #[props(default)]
    total: Option<usize>,
    /// Page sizes offered (with `total`).
    #[props(default)]
    sizes: Vec<usize>,
    /// Page changed.
    #[props(default)]
    on_page_change: EventHandler<usize>,
) -> Element {
    let mut size = use_signal(|| sizes.first().copied().unwrap_or(20));
    let mut m = use_signal(|| {
        let mut p = PageModel::new(page, count.max(1));
        p.mode = PageMode::from_id(&mode);
        p
    });
    let mut belt = use_signal(|| (0isize, 0u32));
    let mut loaded = use_signal(|| 1usize);
    if let Some(t) = total {
        let c = page_count(t, size());
        if m.peek().count != c {
            let mut w = m.write();
            w.count = c;
            w.page = w.page.clamp(1, c.max(1));
        }
    }
    let p = *m.read();
    let slots = p.slots();
    let mark = p.mark_index();
    let (shift, nonce) = belt();
    let mut go = move |to: usize| {
        let s = m.write().go(to);
        if s != 0 {
            let n = belt.peek().1 + 1;
            belt.set((s, n));
        }
        on_page_change.call(m.peek().page);
    };
    if p.mode == PageMode::LoadMore {
        let shown = (loaded() * size()).min(total.unwrap_or(240));
        let all = total.unwrap_or(240);
        return rsx! {
            nav { class: "oxyn-pager more", "aria-label": "{label}",
                span { class: "range", role: "status", "Showing {shown} of {all}" }
                button { class: "oxyn-btn secondary sm", r#type: "button", disabled: shown >= all, onclick: move |_| loaded += 1, if shown >= all { "All loaded" } else { "Load more" } }
            }
        };
    }
    rsx! {
        nav { class: cls(&["oxyn-pager", &mode]), "aria-label": "{label}",
            button { class: "pb prev", r#type: "button", "aria-label": "Previous page", disabled: p.prev_disabled(), onclick: move |_| { let c = m.peek().page; go(c.saturating_sub(1).max(1)) },
                Icon { name: "chevron-left", size: 14.0, stroke: 2.0 }
            }
            if p.mode == PageMode::Cursor {
                span { class: "cur", "aria-current": "page", "Page {p.page}" }
            } else {
                div { class: "pages", style: "--slots:{slots.len()}",
                    if let Some(i) = mark { span { class: "mark", style: "--i:{i}", "aria-hidden": "true" } }
                    for nk in std::iter::once(nonce) {
                    div { key: "{nk}", class: if shift != 0 { "belt moving" } else { "belt" }, style: "--shift:{shift}",
                        for (i, s) in slots.iter().copied().enumerate() {
                            match s {
                                PageSlot::Gap => rsx! { span { key: "g{i}", class: "gap", "aria-hidden": "true", "…" } },
                                PageSlot::Page(n) => {
                                    let cur = n == p.page;
                                    rsx! {
                                        button { key: "p{n}", class: if cur { "pg on" } else { "pg" }, r#type: "button", "aria-current": if cur { "page" }, "aria-label": "{p.page_label(n)}",
                                            onclick: move |_| go(n), "{n}" }
                                    }
                                }
                            }
                        }
                    }
                    }
                }
            }
            button { class: "pb next", r#type: "button", "aria-label": "Next page", disabled: p.next_disabled(), onclick: move |_| { let c = m.peek().page; go(c + 1) },
                Icon { name: "chevron-right", size: 14.0, stroke: 2.0 }
            }
            if let Some(t) = total {
                span { class: "range", "{range_label(p.page, size(), t)}" }
                if !sizes.is_empty() {
                    label { class: "size",
                        span { class: "sr", "Rows per page" }
                        select { onchange: move |e| { let new: usize = e.value().parse().unwrap_or(20); let np = page_for_size(m.peek().page, *size.peek(), new); size.set(new); m.write().page = np; },
                            for s in sizes.iter().copied() { option { value: "{s}", selected: s == size(), "{s} / page" } }
                        }
                    }
                }
            }
        }
    }
}

/// Skeleton: grey shapes the size of what they stand in for, a shared sheen sweeping across all
/// of them in step (or Arc's staggered pulse), shown only after `delay` so a fast load never
/// flashes; with `loading` off the content crossfades in and rises 4 px. Oxyn plus: shape presets
/// (`message`, `tool-row`, `diff`, `card`, `table`, `form`), sweep style, speed and angle.
#[component]
pub fn Skeleton(
    /// Text lines (1–6).
    #[props(default = 3)]
    lines: usize,
    /// Round avatar placeholder.
    #[props(default)]
    avatar: bool,
    /// Shape preset.
    #[props(default = "text".to_string())]
    shape: String,
    /// `shimmer` · `pulse` · `wave` · `none`.
    #[props(default = "shimmer".to_string())]
    sweep: String,
    /// Sweep period s.
    #[props(default = 1.6)]
    speed: f32,
    /// Sweep angle deg.
    #[props(default = 100.0)]
    angle: f32,
    /// Still loading.
    #[props(default = true)]
    loading: bool,
    /// Reveal delay ms.
    #[props(default = 150)]
    delay: u32,
    /// Status label.
    #[props(default = "Loading content".to_string())]
    label: String,
    /// Width px.
    #[props(default = 320.0)]
    width: f32,
    /// Content shown once loaded.
    #[props(default)]
    children: Element,
) -> Element {
    let rows = Shape::from_id(&shape).rows(lines, avatar);
    let sw = Sweep::from_id(&sweep);
    rsx! {
        div { class: "oxyn-skel-wrap", style: "width:{width}px;max-width:100%", "aria-busy": "{loading}",
            if loading {
                div { class: cls(&["oxyn-skel", sw.id()]), role: "status", "aria-label": "{label}", style: "--spd:{speed}s;--ang:{angle}deg;--delay:{delay}ms",
                    for (ri, row) in rows.iter().enumerate() {
                        div { key: "{ri}", class: "r", style: "--ri:{ri}", "aria-hidden": "true",
                            for (bi, b) in row.iter().enumerate() {
                                {
                                    let w = if b.px { format!("{}px", b.w) } else { format!("{}%", b.w * 100.0) };
                                    let r = if b.r.is_infinite() { "50%".to_string() } else { format!("{}px", b.r) };
                                    let grow = if !b.px { "flex:0 1 auto" } else { "flex:none" };
                                    rsx! { span { key: "{bi}", class: "bone", style: "width:{w};height:{b.h}px;border-radius:{r};{grow}" } }
                                }
                            }
                        }
                    }
                }
            } else {
                div { class: "oxyn-skel-content", {children} }
            }
        }
    }
}

/// Shimmering text for "Thinking…" lines (affectUI Shimmer Loader), with its period in seconds.
#[component]
pub fn ShimmerLabel(
    /// Text.
    text: String,
    /// Period s.
    #[props(default = 2.0)]
    duration: f32,
) -> Element {
    rsx! { span { class: "oxyn-shtext", style: "--spd:{duration}s", "{text}" } }
}

/// Skeleton morph: a loading region whose skeleton blocks become the real content, block by
/// block (`stagger` ms apart, in reading order). Each placeholder stretches to the content's box
/// while it fades; the content sharpens out of a 4 px blur. Going back to loading is immediate.
#[component]
pub fn SkeletonMorph(
    /// Loading.
    loading: bool,
    /// Ms between blocks (40).
    #[props(default = 40)]
    stagger: u32,
    /// Status read while loading.
    #[props(default = "Loading".to_string())]
    loading_label: String,
    /// Width px.
    #[props(default = 340.0)]
    width: f32,
    /// `MorphBlock`s and layout.
    children: Element,
) -> Element {
    let mut ctx = use_context_provider(|| Signal::new((loading, stagger)));
    if *ctx.peek() != (loading, stagger) {
        ctx.set((loading, stagger));
    }
    rsx! {
        div { class: cls(&["oxyn-morph", if loading { "loading" } else { "ready" }]), style: "width:{width}px;max-width:100%", "aria-busy": "{loading}",
            if loading { span { class: "sr", role: "status", "{loading_label}" } }
            {children}
        }
    }
}

/// One block of a [`SkeletonMorph`]: a size-matched skeleton while loading, the content after.
#[component]
pub fn MorphBlock(
    /// Order in the reveal.
    index: usize,
    /// Skeleton width (`100%`, `64px`).
    #[props(default = "100%".to_string())]
    width: String,
    /// Skeleton height px (or the lines' height).
    #[props(default = 12.0)]
    height: f32,
    /// Corner radius px; `circle` for an avatar.
    #[props(default = "6".to_string())]
    radius: String,
    /// Text lines (each a bar in its line box; the last of several is shorter).
    #[props(default)]
    lines: Option<usize>,
    /// Line height px.
    #[props(default = 20.0)]
    line_height: f32,
    /// Extra class.
    #[props(default)]
    class: String,
    /// The loaded content.
    #[props(default)]
    children: Element,
) -> Element {
    let ctx = use_context::<Signal<(bool, u32)>>();
    let (loading, stagger) = ctx();
    let r = if radius == "circle" {
        "50%".to_string()
    } else {
        format!("{radius}px")
    };
    let h = lines
        .map(|n| n.clamp(1, 12) as f32 * line_height)
        .unwrap_or(height);
    let d = stagger as usize * index;
    rsx! {
        div { class: cls(&["oxyn-mb", if loading { "sk" } else { "in" }, &class]), style: "--d:{d}ms;--sw:{width};--sh:{h}px;--r:{r}",
            if loading {
                if let Some(n) = lines {
                    div { class: "lines", "aria-hidden": "true",
                        for (i, w) in line_widths(n).into_iter().enumerate() {
                            span { key: "{i}", class: "ln", style: "height:{line_height}px", span { class: "bone", style: "width:{w * 100.0}%" } }
                        }
                    }
                } else {
                    span { class: "bone solo", "aria-hidden": "true", style: "width:{width};height:{h}px;border-radius:{r}" }
                }
            } else {
                span { class: "ph", "aria-hidden": "true" }
                div { class: "ct", {children} }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn overlays_render_roles() {
        let h = render(|| {
            rsx! {
                Dialog { title: "Delete project", description: Some("This can't be undone.".to_string()), open: true, contained: true, tone: "danger", confirm_label: Some("Delete".to_string()), "Body" }
                Sheet { title: "Run details", open: true, contained: true, side: "left", "Body" }
                BottomSheet { title: "Filters", open: true, contained: true }
                SheetStack { pages: vec![StackPage::new("a", "Run settings", "", vec![StackRow::push("Tools", "3 on", "b")]), StackPage::new("b", "Tools", "", vec![StackRow::switch("Shell", true)])], default_stack: vec!["a".to_string(), "b".to_string()], contained: true, mode: "sheet" }
            }
        });
        assert!(h.contains(r#"role="alertdialog""#) && h.contains(r#"aria-modal="true""#));
        assert!(h.contains("be undone."));
        assert!(h.contains("oxyn-sheet left"));
        assert!(h.contains("Expand sheet"));
        assert!(h.contains("Back to Run settings") && h.contains("Close all"));
    }

    #[test]
    fn feedback_renders() {
        let h = render(|| {
            rsx! {
                Toaster { inline: true, seed: vec![SeedToast::new("Draft saved", "Q3 report", "success"), SeedToast::new("Uploading…", "", "loading")] }
                NotificationStack { notices: vec![Notice::new("m", "Maintenance", "Sunday").badge("Notice"), Notice::new("r", "Release", "v2")] }
                Alert { title: "Deploy failed", tone: "danger", dismissible: true, actions: vec!["Retry".to_string()] }
                Card { title: "Q3 plan", meta: Some("Maya Chen".to_string()), status: Some("Updated just now".to_string()), details: Some("More".to_string()), edge: "working" }
                Pagination { page: 6, count: 20 }
                Skeleton { lines: 3, avatar: true }
            }
        });
        assert!(h.contains("Notifications (F8)") && h.contains("Draft saved"));
        assert!(h.contains("1 of 2") && h.contains("Dismiss Maintenance"));
        assert!(h.contains(r#"role="alert""#) && h.contains("Dismiss: Deploy failed"));
        assert!(h.contains(r#"aria-haspopup="dialog""#) && h.contains("beam"));
        assert!(h.contains(r#"aria-current="page""#) && h.contains("Go to page 4"));
        assert!(h.contains(r#"aria-label="Loading content""#));
    }
}
