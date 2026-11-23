//! Surfaces (plan #26): `Dialog`, `Sheet`, `SheetStack`, `BottomSheet`, `Toaster` (with
//! [`use_toasts`]), `NotificationStack`, `Alert`, `Card`, `ScrollArea`, `Pagination`,
//! `Skeleton`, `ShimmerLabel`, `SkeletonReveal` + `MorphBlock`.
//!
//! Logic lives in `oxyn_core::primitives::{modal, toast, scroll, feedback, skeleton}`. Overlays
//! render into a layer that is fixed to the viewport, or absolute inside the nearest positioned
//! ancestor with `contained` (previews, inspectors, embedded apps). Timers (exits, a running
//! confirm, toast timers and the hairline) are CSS animations whose `animationend` advances the
//! model, so they pause with `animation-play-state` and need no host clock.
#![cfg_attr(not(oxyn_pro), allow(dead_code, unused_imports))]

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

// Oxyn Pro part of this module (private repo ismailqureshi72/oxyn-pro; overlaid by CI).
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pro;
#[cfg(oxyn_pro)]
#[allow(unused_imports)]
pub use pro::*;
