//! Core primitives for Dioxus (plan #23): buttons, display atoms, form controls and overlays;
//! plan #25 adds form fields, pickers and flows (`fields`, `pickers`, `flows`); plan #26 adds
//! surfaces (`surfaces`) and command / search components (`command`); plan #27 adds navigation
//! and layout (`navigation`) and app chrome (`chrome`).
//!
//! Logic lives in `oxyn_core::primitives`; these components hold that state in signals, map DOM
//! events onto it and draw with `PRIMITIVES_CSS`. Motion runs in CSS with the motion-token
//! durations; the one shared highlight (button groups, menus, select, segmented, radio) is
//! measured from the DOM with `onmounted` and glides with a transform transition. Server
//! rendering falls back to per-item highlights, so every state still reads without script.

use std::rc::Rc;

use dioxus::prelude::*;

mod buttons;
mod chrome;
mod command;
mod display;
mod fields;
mod flows;
mod forms;
mod navigation;
mod overlays;
mod pickers;
mod surfaces;

pub use buttons::*;
pub use chrome::*;
pub use command::*;
pub use display::*;
pub use fields::*;
pub use flows::*;
pub use forms::*;
pub use navigation::*;
pub use overlays::*;
pub use pickers::*;
pub use surfaces::*;

/// Stylesheet for every primitive.
pub const PRIMITIVES_CSS: &str = concat!(
    include_str!("primitives.css"),
    include_str!("forms.css"),
    include_str!("surfaces.css"),
    include_str!("navigation.css"),
    include_str!("../chat/chat.css"),
    include_str!("../composer/parts.css"),
    include_str!("../shell/shell.css"),
    include_str!("../parity/approvals.css"),
    include_str!("../parity/run.css")
);

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

    #[test]
    fn form_fields_render_their_roles() {
        use oxyn_core::primitives::combobox::ComboOption;
        let h = render(|| {
            rsx! {
                TagInput { label: "Invite", value: vec!["sam@acme.dev".to_string()], validate: "email", max: Some(5) }
                Combobox { options: vec![ComboOption::new("maya", "Maya Patel").avatar()], value: vec!["maya".to_string()], default_open: true }
                MultiSelect { label: "Tools", options: vec![ComboOption::new("a", "Search"), ComboOption::new("b", "Shell")], value: vec!["a".to_string()] }
                OtpInput { length: 6, separator: Some(0) }
                PasswordField { label: "Password", value: "pw" }
                PasswordStrength { label: "New password", value: "abcdefgH1!" }
                InlineEdit { value: "Atlas", label: "Project name" }
                PhoneInput { label: "Phone", value: "+447400123456", default_country: "US" }
            }
        });
        assert!(h.contains("Remove sam@acme.dev"), "{h}");
        assert!(h.contains("1/5"));
        assert!(h.contains(r#"role="combobox""#) && h.contains(r#"aria-multiselectable="true""#));
        assert!(h.contains("Tools") && h.contains(r#"aria-haspopup="listbox""#));
        assert!(h.contains("Verification code, 6 digits") && h.contains("one-time-code"));
        assert!(h.contains("Show password") && h.contains(r#"aria-pressed="false""#));
        assert!(h.contains(r#"role="meter""#) && h.contains(r#"aria-valuetext="Strong""#));
        assert!(h.contains("Project name: Atlas"));
        assert!(h.contains("Country, United Kingdom +44") && h.contains("7400 123456"));
    }

    #[test]
    fn pickers_render_grids_and_spinbuttons() {
        use oxyn_core::primitives::dial::City;
        let h = render(|| {
            rsx! {
                Calendar { value: Some("2026-11-18".to_string()), today: "2026-11-13", show_today: true }
                DatePicker { label: "Due", value: Some("2026-11-20".to_string()), open: true }
                DateRangePicker { default_open: true }
                TimePicker { label: "At", value: Some("09:30".to_string()), open: true }
                TimeDial { cities: vec![City::new("khi", "Karachi", "KHI", 300), City::new("lon", "London", "LON", 0)] }
                DateReel {}
            }
        });
        assert!(h.contains(r#"role="grid""#) && h.contains("Wednesday, November 18, 2026"));
        assert!(h.contains(r#"aria-current="date""#));
        assert!(h.contains(r#"aria-haspopup="dialog""#) && h.contains("Nov 20, 2026"));
        assert!(h.contains("Last 30 days") && h.contains("Apply"));
        assert!(h.contains("9:30 AM"));
        assert!(h.contains(r#"role="slider""#) && h.contains("in Karachi"));
        assert!(h.contains(r#"role="spinbutton""#) && h.contains("Today at 9:30 AM"));
    }

    #[test]
    fn flows_render() {
        use oxyn_core::primitives::{
            filter_menu::FilterGroup,
            questions::{Question, QuestionKind, QuestionOption},
            stepper::{Step, StepField},
        };
        let h = render(|| {
            rsx! {
                FilterMenu { groups: vec![FilterGroup::new("s", "Status", &[("o", "Open")])], value: vec![("s".to_string(), "o".to_string())] }
                StepperForm { steps: vec![Step::new("a", "Workspace", "Name it", vec![StepField::text("name", "Name")]), Step::review("Review")] }
                DecisionCard { decisions: vec![Question::new("q", "Which?", QuestionKind::Single, vec![QuestionOption::new("a", "A").recommended(), QuestionOption::new("b", "B")])] }
                ApprovalQuestions { questions: vec![Question::new("q", "How many?", QuestionKind::Multi, vec![QuestionOption::new("1", "One")])] }
                SmartFormFill { prefill: true }
            }
        });
        assert!(h.contains("Status: Open") && h.contains(r#"role="tree""#));
        assert!(h.contains(r#"aria-current="step""#) && h.contains("Continue"));
        assert!(h.contains("Recommended") && h.contains(r#"aria-keyshortcuts="1""#));
        assert!(h.contains("Question 1 of 1") && h.contains(r#"role="checkbox""#));
        assert!(h.contains("maya.patel@lumenlabs.io") && h.contains("Confidence"));
    }
}
