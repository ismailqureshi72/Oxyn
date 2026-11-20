//! Dioxus components for Oxyn (web demos, Studio, web apps).
//!
//! Components style themselves with the `--oxyn-*` custom properties from `tokens/dist/oxyn.css`;
//! include that stylesheet once (and set `data-oxyn-theme` for light or high contrast). Each
//! component also ships its own small stylesheet as a constant (`STATUS_INDICATOR_CSS`).

mod approval;
mod brand;
mod chat;
mod checkpoint;
mod composer;
mod glyph_loader;
mod parity;
mod primitives;
mod run;
mod shell;
mod status_indicator;
mod status_text;

pub use approval::{ApprovalCard, ApprovalCardProps, ApprovalPager, ApprovalPagerProps};
pub use brand::{BrandIcon, BrandIconProps, has_brand};
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
pub use parity::AccentDensity;
pub use parity::ActivityScreen;
pub use parity::AgentThreadScreen;
pub use parity::AnalyticsScreen;
pub use parity::AppFrame;
pub use parity::AppIconPicker;
pub use parity::AppShellScreen;
pub use parity::AttachmentChips;
pub use parity::BackgroundPicker;
pub use parity::ChatAgentsScreen;
pub use parity::ChatOverviewScreen;
pub use parity::ChatProgressScreen;
pub use parity::ChatRailScreen;
pub use parity::ChatRunScreen;
pub use parity::ChatStartScreen;
pub use parity::ContextBar;
pub use parity::ContextBreakdown;
pub use parity::ContextEffortMenu;
pub use parity::ContextStack;
pub use parity::ContextTide;
pub use parity::CrmScreen;
pub use parity::DiffTimelineRun;
pub use parity::DisplayComfort;
pub use parity::DropLanes;
pub use parity::EffortList;
pub use parity::EffortPill;
pub use parity::EffortRuler;
pub use parity::EffortStairs;
pub use parity::FanoutGrid;
pub use parity::FileIcon;
pub use parity::FoldersScreen;
pub use parity::GooeyAttachMenu;
pub use parity::GooeyComposer;
pub use parity::GooeyModelMenu;
pub use parity::IntegrationsScreen;
pub use parity::JudgeBracket;
pub use parity::KanbanScreen;
pub use parity::LightPlayground;
pub use parity::LiquidDrops;
pub use parity::LiquidTabs;
pub use parity::MemoryPanel;
pub use parity::MenuButton;
pub use parity::ModelEffortMenu;
pub use parity::NozzleSlider;
pub use parity::OrchestrationScreen;
pub use parity::ParallelAttempts;
pub use parity::PlanVsActual;
pub use parity::PolaroidStack;
pub use parity::RailFrame;
pub use parity::RailShellScreen;
pub use parity::RateLimitPlanner;
pub use parity::ReasoningPanel;
pub use parity::RootCauseTree;
pub use parity::RunningNow;
pub use parity::ScheduledRuns;
pub use parity::ScreenComposer;
pub use parity::ScreenStage;
pub use parity::SettingsScreen;
pub use parity::StageScale;
pub use parity::StorageCleanup;
pub use parity::StorageForecast;
pub use parity::StorageMeter;
pub use parity::StorageWaffle;
pub use parity::ThemeGrid;
pub use parity::ThemeSchedule;
pub use parity::ThemeSplit;
pub use parity::ToolAccessMap;
pub use parity::TraceWaterfall;
pub use parity::TypeScale;
pub use parity::UploadDropzone;
pub use parity::WorkflowScreen;
pub use parity::s2_states;
pub use parity::s3_states;
pub use parity::s4_states;
pub use parity::s6_states;
pub use parity::s8_states;
pub use parity::s9_states;
pub use parity::s10_states;
pub use parity::{
    ActionSwapButton, InViewTitle, LoadingState, SlotReels, Source, StreamingText, SweepText,
    TextMorph, ThinkingLoader, WordReveal,
};
pub use parity::{
    ActivityFeed, AgentApprovals, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus,
    AgentTrail, AgentTree, AnimatedCounter, ApprovalGates, ConfirmMorph, ConfirmPopover, Countdown,
    FlowStepper, HoldToConfirm, OnboardingChecklist, PermissionRequest, ReleaseReadiness,
    TaskBreakdown, TaskRows, TreeTable, gate_dot,
};
pub use parity::{
    ActivityHeatmap, ActivityHome, AttachmentStrip, DocGrid, ExperimentGrid, PollingCost,
    RunModeMenu, ThreadList, TokenUsageChart,
};
pub use parity::{
    AdminOverviewScreen, AgentToolCallsScreen, ApiKeysWebhooksScreen, AuditLogScreen,
    BillingUsageScreen, MembersRolesScreen, StatusBoardScreen, s15_slugs, s15_states,
};
pub use parity::{
    AgentCommentThread, AgentOrb, AgentRunPanel, CommentsPanel, FileTypeIcon, ModelPickerMenu,
    PhotoAvatar, Portrait, ThreadNote,
};
pub use parity::{
    AgentCrew, AgentCursors, AgentFindings, AgentGraph, AgentHandoff, AgentInbox, AgentLanes,
    AgentLedger, AgentMonitor, AgentQueue, AgentRecap, AgentSequence, AutonomyDial, AwaySummary,
    CollabStream, ContextMap, CostWaterfall, DecisionTree, DelegateBoard, FallbackChain,
};
pub use parity::{
    AgentsScreen, PermissionsScreen, RunDetailScreen, SchedulesScreen, SkillsScreen, UsageScreen,
};
pub use parity::{
    AiComposer, AvatarUpload, ChatComposer, ContextField, ContextWindowGrid, CtxModel,
    ElasticSlider, FileDropzone, Gauge, MenuModel, ModelMenu, ModelPill, MorphSelect, NumberField,
    PillModel, RichTextEditor, RunControls, ScrubInput, SplitBar, UsageMeter,
};
pub use parity::{
    AiInlineEdit, ChangeSummary, CodeSuggestion, CommentThread, DiffTable, DiffTimeline,
    DiffViewer, ImageCompare, InlineDiff, ReviewCard, ReviewCenter,
};
pub use parity::{
    AnswerCard, CitedAnswerCard, MessageActionBar, ThinkingPanel, ThinkingPill, WritingAssistant,
};
pub use parity::{
    BillingSettingsScreen, GeneralSettingsScreen, MembersSettingsScreen, ModelsSettingsScreen,
    NotificationsSettingsScreen, OnboardingSettingsScreen, PreferencesSettingsScreen,
    PrivacySettingsScreen, ProfileSettingsScreen, SecuritySettingsScreen,
};
pub use parity::{
    ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard,
    ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend,
};
pub use parity::{CommandBoardScreen, s11_board_states};
pub use parity::{CommandCenterScreen, SessionStudioScreen};
pub use parity::{ConflictRadar, MergeConflict};
pub use parity::{EvalsScreen, s5_states};
pub use parity::{FleetWindowScreen, s11_fleet_states};
pub use parity::{
    GanttTimelineScreen, KanbanBoardScreen, MetricsDashboardScreen, ProjectBoardScreen,
    TeamDirectoryScreen, s12_states,
};
pub use parity::{GhostText, LabeledInput, LabeledTextarea, TextArea};
pub use parity::{
    HelpCenterScreen, InboxTriageScreen, MonthCalendarScreen, SupportConversationScreen,
    WeekCalendarScreen, s13_states,
};
pub use parity::{InboxBoardScreen, InboxScreen, MemoryScreen, TasksScreen};
pub use parity::{
    InvoiceStudioScreen, MarketTerminalScreen, ProductDetailScreen, ProductListScreen,
    SignInSplitScreen, s14_states,
};
pub use parity::{
    OnboardingBrandScreen, OnboardingReadyScreen, OnboardingToneScreen, OnboardingWebsiteScreen,
};
pub use parity::{
    PreviewDeskScreen, RepoDeskScreen, TabstackScreen, ThreadbarScreen, s16_previewdesk_states,
    s16_repodesk_states, s16_tabstack_states, s16_threadbar_states,
};
pub use parity::{
    TaskdeckBoardScreen, TaskdeckPluginsScreen, TaskdeckScheduledScreen, TaskdeckSitesScreen,
    TaskdeckStartScreen, TaskdeckTaskScreen,
};
pub use parity::{WorkspaceCoderScreen, s11_coder_states};
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
