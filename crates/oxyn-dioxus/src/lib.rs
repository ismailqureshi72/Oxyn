//! Dioxus components for Oxyn (web demos, Studio, web apps).
//!
//! Components style themselves with the `--oxyn-*` custom properties from `tokens/dist/oxyn.css`;
//! include that stylesheet once (and set `data-oxyn-theme` for light or high contrast). Each
//! component also ships its own small stylesheet as a constant (`STATUS_INDICATOR_CSS`).

mod approval;
mod checkpoint;
mod composer;
mod glyph_loader;
mod primitives;
mod run;
mod status_indicator;
mod status_text;

pub use approval::{ApprovalCard, ApprovalCardProps, ApprovalPager, ApprovalPagerProps};
pub use checkpoint::{
    CheckpointLanes, CheckpointLanesProps, CheckpointList, CheckpointListProps, CheckpointMarker,
    CheckpointMarkerProps, RestoreToast, RestoreToastProps,
};
pub use composer::{
    AttachmentRow, AttachmentRowProps, COMPOSER_CSS, Composer, ComposerProps, ContextMeterChip,
    ContextMeterChipProps, DropZone, DropZoneProps, EffortChip, EffortChipProps, Icon, IconProps,
    ModelPicker, ModelPickerProps, SendButton, SendButtonProps, Submission,
};
pub use glyph_loader::{GLYPH_LOADER_CSS, GlyphLoader, GlyphLoaderProps};
/// Stylesheet for approvals and checkpoints.
pub const HITL_CSS: &str = include_str!("hitl.css");
pub use primitives::{
    ActionButton, Avatar, AvatarGroup, Badge, Button, ButtonGroup, Bx, Checkbox, Chip,
    DropdownMenu, EmptyState, EntityChip, ExpandingButtonGroup, FloatingButtonGroup, Glide,
    IconButton, Kbd, Monogram, NumberTicker, PRIMITIVES_CSS, Popover, RadioGroup, ReactionBar,
    SegmentedControl, Select, Separator, Slider, Spinner, SplitButton, Switch, TextField, Tooltip,
    TooltipProvider, ValuePill, fill_from, presence_from, spoken_shortcut, use_glide,
};
pub use run::{
    PlanCard, PlanCardProps, RUN_CSS, StepTimeline, StepTimelineProps, SubagentTree,
    SubagentTreeProps, TodoList, TodoListProps, ToolCallRow, ToolCallRowProps, ToolStack,
    ToolStackProps,
};
pub use status_indicator::{
    IndicatorSize, IndicatorVariant, STATUS_INDICATOR_CSS, StatusIndicator, StatusIndicatorProps,
};
pub use status_text::{
    ElapsedTimer, ElapsedTimerProps, STATUS_TEXT_CSS, TextReveal, TextRevealProps, TextShimmer,
    TextShimmerProps, VerbCycler, VerbCyclerProps,
};

/// `var(--oxyn-color-…)` for a colour role.
pub fn color_var(role: oxyn_tokens::ColorRole) -> String {
    format!("var(--oxyn-color-{})", role.path().replace('.', "-"))
}
