//! Core primitives for Dioxus (plan #23): buttons, display atoms, form controls and overlays.
//!
//! Logic lives in `oxyn_core::primitives`; these components hold that state in signals, map DOM
//! events onto it and draw with `PRIMITIVES_CSS`. Motion runs in CSS with the motion-token
//! durations; the one shared highlight (button groups, menus, select, segmented, radio) is
//! measured from the DOM with `onmounted` and glides with a transform transition. Server
//! rendering falls back to per-item highlights, so every state still reads without script.

use std::rc::Rc;

use dioxus::prelude::*;

mod buttons;
mod display;
mod forms;
mod overlays;

pub use buttons::*;
pub use display::*;
pub use forms::*;
pub use overlays::*;

/// Stylesheet for every primitive.
pub const PRIMITIVES_CSS: &str = include_str!("primitives.css");

/// A measured box relative to its container.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Bx {
    /// Left.
    pub x: f64,
    /// Top.
    pub y: f64,
    /// Width.
    pub w: f64,
    /// Height.
    pub h: f64,
}

/// One shared highlight over a set of items: register the container and each item with
/// `onmounted`, then call [`Glide::go`] on hover / focus. The box glides between items (pointer)
/// or jumps (keyboard), and grows in from 94 % when it appears from nothing.
#[derive(Clone, Copy, PartialEq)]
pub struct Glide {
    root: Signal<Option<Rc<MountedData>>>,
    nodes: Signal<Vec<Option<Rc<MountedData>>>>,
    /// Current box.
    pub bx: Signal<Option<Bx>>,
    /// Highlighted index.
    pub at: Signal<Option<usize>>,
    /// The last move came from the keyboard.
    pub jump: Signal<bool>,
    /// It appeared from nothing on the last move.
    pub appear: Signal<bool>,
}

/// Creates a [`Glide`].
pub fn use_glide() -> Glide {
    Glide {
        root: use_signal(|| None),
        nodes: use_signal(Vec::new),
        bx: use_signal(|| None),
        at: use_signal(|| None),
        jump: use_signal(|| false),
        appear: use_signal(|| false),
    }
}

impl Glide {
    /// Registers the container.
    pub fn set_root(mut self, d: Rc<MountedData>) {
        self.root.set(Some(d));
    }
    /// Registers item `i`.
    pub fn set_item(mut self, i: usize, d: Rc<MountedData>) {
        let mut n = self.nodes.write();
        if n.len() <= i {
            n.resize(i + 1, None);
        }
        n[i] = Some(d);
    }
    /// Moves to `i` (`None` hides it).
    pub fn go(mut self, i: Option<usize>, keyboard: bool) {
        let was = *self.at.peek();
        if was == i && self.bx.peek().is_some() {
            return;
        }
        self.at.set(i);
        self.jump.set(keyboard);
        self.appear.set(was.is_none());
        let Some(i) = i else {
            self.bx.set(None);
            return;
        };
        let root = self.root.peek().clone();
        let node = self.nodes.peek().get(i).cloned().flatten();
        let mut bx = self.bx;
        if let (Some(root), Some(node)) = (root, node) {
            spawn(async move {
                if let (Ok(r), Ok(n)) = (root.get_client_rect().await, node.get_client_rect().await)
                {
                    bx.set(Some(Bx {
                        x: n.origin.x - r.origin.x,
                        y: n.origin.y - r.origin.y,
                        w: n.size.width,
                        h: n.size.height,
                    }));
                }
            });
        }
    }
    /// Whether a measured box is drawn (otherwise items light themselves).
    pub fn measured(&self) -> bool {
        self.bx.read().is_some()
    }
    /// Inline style for the highlight element.
    pub fn style(&self) -> String {
        match *self.bx.read() {
            Some(b) => format!(
                "transform:translate({:.1}px,{:.1}px);width:{:.1}px;height:{:.1}px",
                b.x, b.y, b.w, b.h
            ),
            None => "opacity:0".into(),
        }
    }
    /// Classes for the highlight element.
    pub fn class(&self, base: &str) -> String {
        let mut c = base.to_string();
        if *self.jump.read() {
            c.push_str(" jump");
        }
        if *self.appear.read() {
            c.push_str(" appear");
        }
        c
    }
}

/// Joins classes, skipping empty ones.
pub(crate) fn cls(parts: &[&str]) -> String {
    parts
        .iter()
        .filter(|p| !p.is_empty())
        .copied()
        .collect::<Vec<_>>()
        .join(" ")
}

/// Maps a DOM key to a [`oxyn_core::primitives::nav::NavKey`] for `vertical` / horizontal lists.
pub(crate) fn nav_key(k: &Key, vertical: bool) -> Option<oxyn_core::primitives::nav::NavKey> {
    use oxyn_core::primitives::nav::NavKey as N;
    Some(match k {
        Key::ArrowDown if vertical => N::Next,
        Key::ArrowUp if vertical => N::Prev,
        Key::ArrowRight if !vertical => N::Next,
        Key::ArrowLeft if !vertical => N::Prev,
        Key::Home => N::First,
        Key::End => N::Last,
        Key::PageDown => N::PageForward,
        Key::PageUp => N::PageBack,
        _ => return None,
    })
}

#[cfg(test)]
mod tests {

    use oxyn_core::{
        AgentState, ClockMs,
        primitives::{
            button::GroupItem,
            display::{Avatar as AvatarModel, Badge as BadgeModel, Tone},
            menu::MenuItem,
            segmented::Segment,
            select::SelectOption,
            slider::Slider as SliderModel,
            ticker::{NumberFormat, NumberTicker as TickerModel},
            toggle::RadioOption,
        },
    };

    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn buttons_carry_state_and_names() {
        let h = render(|| {
            rsx! {
                Button { label: "Generating", loading: true }
                Button { label: "Send", disabled: true, reason: "Waiting for your approval" }
                IconButton { icon: "audio", label: "Mute", pressed: true }
                ButtonGroup { items: vec![GroupItem::new("a", "Bold", None), GroupItem::new("b", "Code", None)], selected: "b" }
            }
        });
        assert!(h.contains("aria-busy=\"true\"") && h.contains("loading shimmer"));
        assert!(
            h.contains("title=\"Waiting for your approval\"")
                && h.contains("aria-disabled=\"true\"")
        );
        assert!(h.contains("aria-label=\"Mute\"") && h.contains("aria-pressed=\"true\""));
        assert!(h.contains("seg sel") && h.contains("role=\"group\""));
        assert!(
            !h.contains("aria-pressed=\"\""),
            "absent attributes are omitted, not empty"
        );
    }

    #[test]
    fn display_atoms() {
        let h = render(|| {
            let mut b = BadgeModel::new("unread", Tone::Info);
            b.count = Some(250);
            let a = AvatarModel::agent("Coder", &AgentState::Streaming);
            let mut t = TickerModel::new(1.0, NumberFormat::Integer);
            t.set(10_240.0, ClockMs(0));
            rsx! {
                Badge { badge: b }
                Avatar { avatar: a }
                Kbd { keys: "Mod+Shift+K" }
                Spinner { label: "Loading runs" }
                Spinner {}
                NumberTicker { ticker: t, now: ClockMs(5_000) }
                Separator { label: "Today", decorative: false }
            }
        });
        assert!(h.contains("99+") && h.contains("More than 99 unread"));
        assert!(h.contains("Coder, working") && h.contains("ring-breathe"));
        assert!(h.contains("aria-keyshortcuts=\"Meta+Shift+K\"") && h.contains("Command Shift K"));
        assert!(h.contains("role=\"status\"") && h.contains("aria-label=\"Loading runs\""));
        assert!(h.contains("aria-label=\"10,240\""));
        assert!(h.contains("role=\"separator\"") && h.contains("aria-orientation=\"horizontal\""));
    }

    #[test]
    fn form_controls() {
        let h = render(|| {
            rsx! {
                Switch { label: "Stream", checked: true }
                Checkbox { label: "All", value: "indeterminate" }
                RadioGroup { label: "Mode", options: vec![RadioOption::new("a", "A"), RadioOption::new("b", "B")], value: "b" }
                SegmentedControl { options: vec![Segment::new("x", "X"), Segment::new("y", "Y")], kind: "tabs" }
                Select { options: vec![SelectOption::new("s", "Sonnet")], label: "Model", open: true }
                Slider { slider: SliderModel::new(3.0, 0.0, 10.0, 1.0), label: "Effort" }
                TextField { label: "Email", value: "x@", kind: "email", error: "Enter an email" }
            }
        });
        assert!(h.contains("role=\"switch\"") && h.contains("aria-checked=\"true\""));
        assert!(h.contains("aria-checked=\"mixed\""));
        assert!(h.contains("role=\"radiogroup\"") && h.contains("tabindex=\"0\""));
        assert!(h.contains("role=\"tablist\"") && h.contains("aria-selected=\"true\""));
        assert!(
            h.contains("role=\"listbox\"")
                && h.contains("aria-expanded=\"true\"")
                && h.contains("Choose…")
        );
        assert!(h.contains("role=\"slider\"") && h.contains("aria-valuenow=\"3\""));
        assert!(h.contains("aria-invalid=\"true\"") && h.contains("Enter an email"));
    }

    #[test]
    fn overlays() {
        let h = render(|| {
            rsx! {
                Tooltip { content: "Search", shortcut: "Mod+K", open: true, button { "S" } }
                DropdownMenu { items: vec![MenuItem::action("a", "Rename"), MenuItem::checkbox("w", "Wrap", true), MenuItem::action("d", "Delete").danger()], open: true, highlight: 2 }
            }
        });
        assert!(h.contains("role=\"tooltip\"") && h.contains("aria-describedby"));
        assert!(h.contains("role=\"menu\"") && h.contains("menuitemcheckbox"));
        assert!(h.contains("row danger") && h.contains("hl-self"));
    }
}
