//! Dioxus components for Oxyn (web demos, Studio, web apps).
//!
//! Components style themselves with the `--oxyn-*` custom properties from `tokens/dist/oxyn.css`;
//! include that stylesheet once (and set `data-oxyn-theme` for light or high contrast). Each
//! component also ships its own small stylesheet as a constant (`STATUS_INDICATOR_CSS`).

mod approval;
mod chat;
mod checkpoint;
mod composer;
mod glyph_loader;
mod primitives;
mod run;
mod shell;
mod status_indicator;
mod status_text;

pub use approval::{ApprovalCard, ApprovalCardProps, ApprovalPager, ApprovalPagerProps};
pub use chat::{
    AiAnswer, AiChat, CHAT_CSS, ChatThread, ContextCards, DocumentExtraction, Markdown,
    MessageActions, MessageBubble, ReasoningBlock, RecommendationCard, ResearchRun,
    ResponseFeedback, SourceCards, SourceCitation, StarterPrompts, StreamingMarkdown,
    SupportConversation, ThinkingTraces,
};
pub use checkpoint::{
    CheckpointLanes, CheckpointLanesProps, CheckpointList, CheckpointListProps, CheckpointMarker,
    CheckpointMarkerProps, RestoreToast, RestoreToastProps,
};
pub use composer::{
    AgentSteering, AgentSteeringProps, DictationButton, DictationButtonProps, MentionInput,
    MentionInputProps, ModeSwitch, ModeSwitchProps, PARTS_CSS, PermissionSelector,
    PermissionSelectorProps, QueueStack, QueueStackProps, ToolsMenu, ToolsMenuProps, VoiceRecorder,
    VoiceRecorderProps,
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
    Accordion, AccordionItem, Breadcrumb, ChipGroup, CodeBlock, CopyButton, Dock, Drawer,
    EmptyPreset, ExpandableText, HoverCard, NotificationCenter, PageHeader, PaneView, PresenceDot,
    Profile, RadioCards, Rating, Reactions, ReorderItem, ReorderableList, ResizablePanels, Tabs,
    TeamSwitcher, TocProgress, TocSection, ToggleGroup, TreeView, UserMenu, WorkspaceTile,
};
pub use primitives::{
    ActionButton, Avatar, AvatarGroup, Badge, Button, ButtonGroup, Bx, Checkbox, Chip,
    DropdownMenu, EmptyState, EntityChip, ExpandingButtonGroup, FloatingButtonGroup, Glide,
    IconButton, Kbd, Monogram, NumberTicker, PRIMITIVES_CSS, Popover, RadioGroup, ReactionBar,
    SegmentedControl, Select, Separator, Slider, Spinner, SplitButton, Switch, TextField, Tooltip,
    TooltipProvider, ValuePill, fill_from, presence_from, spoken_shortcut, use_glide,
};
pub use primitives::{
    AiCommand, AiSearchBar, Alert, BottomSheet, Card, CommandPalette, ControlCenter, Dialog,
    ExpandingSearch, Line, MorphBlock, NotificationStack, Pagination, ScriptedAnswer, ScrollArea,
    SearchField, SeedToast, SettingsCommand, Sheet, SheetStack, ShimmerLabel, ShortcutList,
    ShortcutRecorder, Skeleton, SkeletonMorph, StackPage, StackRow, Toaster, Toasts, use_toasts,
};
pub use primitives::{
    ApprovalQuestions, Calendar, Combobox, DatePicker, DateRangePicker, DateReel, DecisionCard,
    FilterMenu, InlineEdit, MultiSelect, OtpInput, PasswordField, PasswordStrength, PhoneInput,
    SmartFormFill, StepperForm, TagInput, TimeDial, TimePicker,
};
pub use run::{
    PlanCard, PlanCardProps, RUN_CSS, StepTimeline, StepTimelineProps, SubagentTree,
    SubagentTreeProps, TodoList, TodoListProps, ToolCallRow, ToolCallRowProps, ToolStack,
    ToolStackProps,
};
pub use shell::{
    AiSidePanel, AiSidePanelProps, AppShell, AppShellProps, ChatShell, ChatShellProps, Harness,
    HarnessProps, InboxSidebar, InboxSidebarProps, SHELL_CSS, Sidebar, SidebarNav, SidebarNavProps,
    SidebarProps, SidebarRail, SidebarRailProps, TerminalTabs, TerminalTabsProps, WorkspaceSidebar,
    WorkspaceSidebarProps, sample_inbox, sample_page, sample_sections,
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
