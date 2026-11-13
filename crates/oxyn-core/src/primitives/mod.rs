//! Core primitives (plan #23): the framework-free state and geometry behind buttons, display
//! atoms, form controls and overlays. Renderers (Dioxus, GPUI) draw from these.
//!
//! - [`nav`]: roving focus, Home / End, wrap, type-ahead, pointer-to-index for drags.
//! - [`placement`]: side / align / offset with flip and collision padding for floating surfaces.
//! - [`button`]: button, icon button, action button, hold-to-confirm, split button, the three
//!   button groups and the shared gliding highlight.
//! - [`display`]: badge, chip, value pill, monogram + entity chip, avatar with agent ring, kbd,
//!   separator, spinner, empty state.
//! - [`ticker`]: number formatting and the odometer ticker; the reaction bar.
//! - [`toggle`]: switch, checkbox, radio group.
//! - [`segmented`]: the segmented control.
//! - [`select`]: the select field.
//! - [`slider`]: the slider (single, range, detents).
//! - [`text_field`]: text field.
//! - [`overlay`]: tooltip delay group, popover stack.
//! - [`menu`]: dropdown menu (and context-menu mode).
//!
//! Forms (plan #25):
//! - [`tags`]: tag input. [`combobox`]: combobox and multi-select. [`otp`]: one-time code.
//! - [`password`]: password reveal helpers and the strength meter. [`inline_edit`]: rename in
//!   place. [`phone`]: phone input with a country picker and E.164.
//! - [`date`]: dates, the calendar, date picker, range picker and time list. [`dial`]: the time
//!   dial and the date reel.
//! - [`filter_menu`]: grouped filters. [`stepper`]: stepper / multi-step form.
//!   [`questions`]: the decision card and paged approval questions. [`smart_fill`]: paste to fill.
//!
//! Behaviour follows the affectUI and Arc UI docs (rebuilt, no source used) and Beautiful UI's MIT
//! atoms (button, entity chip + monogram, value pill) and Approval Card (© 2026 Shane Levine).

pub mod button;
pub mod combobox;
pub mod date;
pub mod dial;
pub mod display;
pub mod filter_menu;
pub mod inline_edit;
pub mod menu;
pub mod nav;
pub mod otp;
pub mod overlay;
pub mod password;
pub mod phone;
pub mod placement;
pub mod questions;
pub mod segmented;
pub mod select;
pub mod slider;
pub mod smart_fill;
pub mod stepper;
pub mod tags;
pub mod text_field;
pub mod ticker;
pub mod toggle;
