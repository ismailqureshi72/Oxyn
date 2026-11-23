//! Dioxus components for Oxyn (web demos, Studio, web apps).
//!
//! Components style themselves with the `--oxyn-*` custom properties from `tokens/dist/oxyn.css`;
//! include that stylesheet once (and set `data-oxyn-theme` for light or high contrast). Each
//! component also ships its own small stylesheet as a constant (`STATUS_INDICATOR_CSS`).

#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod aikit;
mod approval;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod blocks;
mod brand;
mod chat;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod checkpoint;
mod composer;
mod glyph_loader;
mod kit;
mod primitives;
mod run;
mod shell;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod status_indicator;
mod status_text;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod xpost;

#[cfg(oxyn_pro)]
pub use aikit::{
    AgentDocCursor, AgentTimeline, AiMascot, AiSpark, AiTranslate, AiWorkflow, AskYourData,
    ConnectorSetup, CubeMascot, EnrichColumn, GenerationStats, GhostMascot, HumanHandoff,
    ImageGeneration, ImageReveal, LiveVisitors, LlmCostCalculator, MeetingSummary, ModelCard,
    PiiRedaction, PixelMascot, ProviderKeys, ProviderStatus, ReferralCard, ReplyDraft,
    SocialProofToast, SpeechPlayer, TicketRouter, TldrToggle, ToneTuner, TrainingRun, TrialBanner,
    UpgradePrompt,
};
pub use approval::{ApprovalCard, ApprovalCardProps};
#[cfg(oxyn_pro)]
pub use approval::{ApprovalPager, ApprovalPagerProps};
#[cfg(oxyn_pro)]
pub use blocks::{
    AgentBuilder, AgentGallery, AgentRoutines, AiFeatureLaunch, AiInsightCard, AiSetupWizard,
    AssistantMemory, BackgroundAgents, BlindCompare, CallInsights, CatchUp, EvalSuite,
    FleetOverview, FleetTimeline, KnowledgeBase, LeadScoring, LiveSession, McpServers, PromptBench,
    RunInspector,
};
pub use brand::{BrandIcon, BrandIconProps, has_brand};
#[cfg(oxyn_pro)]
pub use chat::AiAnswer;
#[cfg(oxyn_pro)]
pub use chat::{
    AiChat, ContextCards, DocumentExtraction, RecommendationCard, ResearchRun, SourceCards,
    SourceCitation, StarterPrompts, SupportConversation, ThinkingTraces,
};
pub use chat::{
    CHAT_CSS, ChatThread, Markdown, MessageActions, MessageBubble, ReasoningBlock,
    ResponseFeedback, StreamingMarkdown,
};
#[cfg(oxyn_pro)]
pub use checkpoint::{
    CheckpointFilmStrip, CheckpointFilmStripProps, CheckpointLanes, CheckpointLanesProps,
    CheckpointList, CheckpointListProps, CheckpointMarker, CheckpointMarkerProps, CheckpointTree,
    CheckpointTreeProps, RailTone, RestorePreviewSheet, RestorePreviewSheetProps, RestoreToast,
    RestoreToastProps, UndoLadder, UndoLadderProps,
};
#[cfg(oxyn_pro)]
pub use composer::{
    AgentSteering, AgentSteeringProps, DictationButton, DictationButtonProps, MentionInput,
    MentionInputProps, ModeSwitch, ModeSwitchProps, PermissionSelector, PermissionSelectorProps,
    ToolsMenu, ToolsMenuProps,
};
pub use composer::{
    AttachmentRow, AttachmentRowProps, COMPOSER_CSS, Composer, ComposerProps, ContextMeterChip,
    ContextMeterChipProps, DropZone, DropZoneProps, EffortChip, EffortChipProps, Icon, IconProps,
    ModelPicker, ModelPickerProps, SendButton, SendButtonProps, Submission,
};
pub use composer::{PARTS_CSS, QueueStack, QueueStackProps, VoiceRecorder, VoiceRecorderProps};
pub use glyph_loader::{GLYPH_LOADER_CSS, GlyphLoader, GlyphLoaderProps};
#[cfg(oxyn_pro)]
pub use xpost::{
    AgentPromptCopy, DoneShockwave, LoopPipeline, ProactiveCard, PublishPopover, SecretCard,
    SplitConfirm, TestTicks, TriageMetro, WriteSweep,
};
/// Stylesheet for approvals and checkpoints.
pub const HITL_CSS: &str = include_str!("hitl.css");
#[cfg(oxyn_pro)]
pub use kit::AccentDensity;
#[cfg(oxyn_pro)]
pub use kit::ActivityPill;
#[cfg(oxyn_pro)]
pub use kit::ActivityScreen;
#[cfg(oxyn_pro)]
pub use kit::AgentThreadScreen;
#[cfg(oxyn_pro)]
pub use kit::AnalyticsScreen;
#[cfg(oxyn_pro)]
pub use kit::AppFrame;
#[cfg(oxyn_pro)]
pub use kit::AppIconPicker;
#[cfg(oxyn_pro)]
pub use kit::AppShellScreen;
#[cfg(oxyn_pro)]
pub use kit::AttachmentChips;
#[cfg(oxyn_pro)]
pub use kit::AttemptRace;
#[cfg(oxyn_pro)]
pub use kit::BackgroundPicker;
#[cfg(oxyn_pro)]
pub use kit::ChatAgentsScreen;
#[cfg(oxyn_pro)]
pub use kit::ChatOverviewScreen;
#[cfg(oxyn_pro)]
pub use kit::ChatProgressScreen;
#[cfg(oxyn_pro)]
pub use kit::ChatRailScreen;
#[cfg(oxyn_pro)]
pub use kit::ChatRunScreen;
#[cfg(oxyn_pro)]
pub use kit::ChatStartScreen;
#[cfg(oxyn_pro)]
pub use kit::ContextBar;
#[cfg(oxyn_pro)]
pub use kit::ContextBreakdown;
#[cfg(oxyn_pro)]
pub use kit::ContextEffortMenu;
#[cfg(oxyn_pro)]
pub use kit::ContextStack;
#[cfg(oxyn_pro)]
pub use kit::ContextTide;
#[cfg(oxyn_pro)]
pub use kit::CrmScreen;
#[cfg(oxyn_pro)]
pub use kit::DiffTimelineRun;
#[cfg(oxyn_pro)]
pub use kit::DisplayComfort;
#[cfg(oxyn_pro)]
pub use kit::DropLanes;
#[cfg(oxyn_pro)]
pub use kit::EffortList;
#[cfg(oxyn_pro)]
pub use kit::EffortPill;
#[cfg(oxyn_pro)]
pub use kit::EffortRuler;
#[cfg(oxyn_pro)]
pub use kit::EffortStairs;
#[cfg(oxyn_pro)]
pub use kit::FileIcon;
#[cfg(oxyn_pro)]
pub use kit::FoldersScreen;
#[cfg(oxyn_pro)]
pub use kit::GooeyAttachMenu;
#[cfg(oxyn_pro)]
pub use kit::GooeyComposer;
#[cfg(oxyn_pro)]
pub use kit::GooeyModelMenu;
#[cfg(oxyn_pro)]
pub use kit::IntegrationsScreen;
#[cfg(oxyn_pro)]
pub use kit::KanbanScreen;
#[cfg(oxyn_pro)]
pub use kit::LightPlayground;
#[cfg(oxyn_pro)]
pub use kit::LiquidDrops;
#[cfg(oxyn_pro)]
pub use kit::LiquidTabs;
#[cfg(oxyn_pro)]
pub use kit::LoadPlanner;
#[cfg(oxyn_pro)]
pub use kit::MemoryPanel;
#[cfg(oxyn_pro)]
pub use kit::MenuButton;
#[cfg(oxyn_pro)]
pub use kit::ModelEffortMenu;
pub use kit::ModelKnockout;
#[cfg(oxyn_pro)]
pub use kit::NozzleSlider;
#[cfg(oxyn_pro)]
pub use kit::OrchestrationScreen;
#[cfg(oxyn_pro)]
pub use kit::PlanVsActual;
#[cfg(oxyn_pro)]
pub use kit::PolaroidStack;
#[cfg(oxyn_pro)]
pub use kit::RailFrame;
#[cfg(oxyn_pro)]
pub use kit::RailShellScreen;
#[cfg(oxyn_pro)]
pub use kit::ReasoningPanel;
#[cfg(oxyn_pro)]
pub use kit::RootCauseTree;
pub use kit::ScheduledRuns;
#[cfg(oxyn_pro)]
pub use kit::ScreenComposer;
#[cfg(oxyn_pro)]
pub use kit::ScreenStage;
#[cfg(oxyn_pro)]
pub use kit::SettingsScreen;
#[cfg(oxyn_pro)]
pub use kit::StageScale;
#[cfg(oxyn_pro)]
pub use kit::StorageCleanup;
#[cfg(oxyn_pro)]
pub use kit::StorageForecast;
#[cfg(oxyn_pro)]
pub use kit::StorageMeter;
#[cfg(oxyn_pro)]
pub use kit::StorageWaffle;
pub use kit::TaskGrid;
#[cfg(oxyn_pro)]
pub use kit::ThemeGrid;
#[cfg(oxyn_pro)]
pub use kit::ThemeSchedule;
#[cfg(oxyn_pro)]
pub use kit::ThemeSplit;
pub use kit::ToolReach;
pub use kit::TraceWaterfall;
#[cfg(oxyn_pro)]
pub use kit::TypeScale;
#[cfg(oxyn_pro)]
pub use kit::UploadDropzone;
#[cfg(oxyn_pro)]
pub use kit::WorkflowScreen;
#[cfg(oxyn_pro)]
pub use kit::s2_states;
#[cfg(oxyn_pro)]
pub use kit::s3_states;
#[cfg(oxyn_pro)]
pub use kit::s4_states;
#[cfg(oxyn_pro)]
pub use kit::s6_states;
#[cfg(oxyn_pro)]
pub use kit::s8_states;
#[cfg(oxyn_pro)]
pub use kit::s9_states;
#[cfg(oxyn_pro)]
pub use kit::s10_states;
#[cfg(oxyn_pro)]
pub use kit::{
    ActivityFeed, AgentApprovals, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus,
    AgentTree, AnimatedCounter, ApprovalGates, Countdown, HoldToConfirm, InlineConfirm,
    LaunchChecklist, OnboardingChecklist, PermissionRequest, TaskBreakdown, TaskRows, TreeTable,
};
#[cfg(oxyn_pro)]
pub use kit::{
    ActivityHeatmap, ActivityHome, AttachmentStrip, DocGrid, ExperimentGrid, PollingCost,
    RunModeMenu, ThreadList, TokenUsageChart,
};
#[cfg(oxyn_pro)]
pub use kit::{
    AdminOverviewScreen, AgentToolCallsScreen, ApiKeysWebhooksScreen, AuditLogScreen,
    BillingUsageScreen, MembersRolesScreen, StatusBoardScreen, s15_slugs, s15_states,
};
#[cfg(oxyn_pro)]
pub use kit::{AgentCommentThread, AgentOrb, CommentsPanel, PhotoAvatar, Portrait, ThreadNote};
pub use kit::{AgentRunPanel, FileTypeIcon, ModelPickerMenu};
pub use kit::{AgentTrail, ConfirmPopover, FlowStepper, gate_dot};
#[cfg(oxyn_pro)]
pub use kit::{
    AgentsScreen, PermissionsScreen, RunDetailScreen, SchedulesScreen, SkillsScreen, UsageScreen,
};
#[cfg(oxyn_pro)]
pub use kit::{
    AiComposer, AvatarUpload, ContextField, ContextWindowGrid, CtxModel, Gauge, MenuModel,
    ModelMenu, ModelPill, NumberField, PillModel, RichTextEditor, RubberSlider, RunControls,
    ScrubInput, SplitBar, UnfoldSelect, UsageMeter,
};
#[cfg(oxyn_pro)]
pub use kit::{
    AiInlineEdit, ChangeSummary, CommentThread, DiffTable, ImageCompare, ReviewCard, ReviewCenter,
};
#[cfg(oxyn_pro)]
pub use kit::{
    AnswerCard, CitedAnswerCard, MessageActionBar, ThinkingPanel, ThinkingPill, WritingAssistant,
};
#[cfg(oxyn_pro)]
pub use kit::{
    AssignBoard, ChangeLedger, FindingsFeed, JobQueue, LiveLanes, OvernightRecap, RelayTrail,
    ReturnBrief, RunMonitor, SequenceDiagram, TaskInbox, ToolNetwork,
};
pub use kit::{
    AutonomyLevels, ContextTreemap, CrewChat, CrewTree, DecisionTree, FallbackChain,
    PresenceCursors, SpendBreakdown,
};
#[cfg(oxyn_pro)]
pub use kit::{
    BillingSettingsScreen, GeneralSettingsScreen, MembersSettingsScreen, ModelsSettingsScreen,
    NotificationsSettingsScreen, OnboardingSettingsScreen, PreferencesSettingsScreen,
    PrivacySettingsScreen, ProfileSettingsScreen, SecuritySettingsScreen,
};
pub use kit::{ChatComposer, FileDropzone};
#[cfg(oxyn_pro)]
pub use kit::{
    ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard,
    ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend,
};
pub use kit::{CodeSuggestion, DiffTimeline, DiffViewer, InlineDiff};
#[cfg(oxyn_pro)]
pub use kit::{CommandBoardScreen, s11_board_states};
#[cfg(oxyn_pro)]
pub use kit::{CommandCenterScreen, SessionStudioScreen};
#[cfg(oxyn_pro)]
pub use kit::{ConflictRadar, MergeConflict};
#[cfg(oxyn_pro)]
pub use kit::{
    CourierScreen, HaloChatScreen, HangarScreen, PilotPanelScreen, s16_courier_states,
    s16_halochat_states, s16_hangar_states, s16_pilotpanel_states,
};
#[cfg(oxyn_pro)]
pub use kit::{EvalsScreen, s5_states};
#[cfg(oxyn_pro)]
pub use kit::{FleetWindowScreen, s11_fleet_states};
#[cfg(oxyn_pro)]
pub use kit::{
    GanttTimelineScreen, KanbanBoardScreen, MetricsDashboardScreen, ProjectBoardScreen,
    TeamDirectoryScreen, s12_states,
};
#[cfg(oxyn_pro)]
pub use kit::{GhostText, LabeledInput, LabeledTextarea, TextArea};
#[cfg(oxyn_pro)]
pub use kit::{
    HelpCenterScreen, InboxTriageScreen, MonthCalendarScreen, SupportConversationScreen,
    WeekCalendarScreen, s13_states,
};
#[cfg(oxyn_pro)]
pub use kit::{InboxBoardScreen, InboxScreen, MemoryScreen, TasksScreen};
#[cfg(oxyn_pro)]
pub use kit::{
    InvoiceDeskScreen, ProductDetailScreen, ProductListScreen, SignInSplitScreen,
    TradingDeskScreen, s14_states,
};
#[cfg(oxyn_pro)]
pub use kit::{
    LoadingState, ScrollTitle, SlotReels, Source, StatefulActionButton, StreamingText, SweepText,
    TextMorph, ThinkingLoader, WordReveal,
};
#[cfg(oxyn_pro)]
pub use kit::{
    OnboardingBrandScreen, OnboardingReadyScreen, OnboardingToneScreen, OnboardingWebsiteScreen,
};
#[cfg(oxyn_pro)]
pub use kit::{
    PreviewDeskScreen, RepoDeskScreen, TabstackScreen, ThreadbarScreen, s16_previewdesk_states,
    s16_repodesk_states, s16_tabstack_states, s16_threadbar_states,
};
#[cfg(oxyn_pro)]
pub use kit::{
    TaskdeckBoardScreen, TaskdeckPluginsScreen, TaskdeckScheduledScreen, TaskdeckSitesScreen,
    TaskdeckStartScreen, TaskdeckTaskScreen,
};
#[cfg(oxyn_pro)]
pub use kit::{WorkspaceCoderScreen, s11_coder_states};
#[cfg(oxyn_pro)]
pub use primitives::{
    Accordion, AccordionItem, Breadcrumb, ChipGroup, Dock, Drawer, EmptyPreset, ExpandableText,
    HoverCard, NotificationCenter, PageHeader, PaneView, PresenceDot, Profile, RadioCards, Rating,
    Reactions, ReorderItem, ReorderableList, ResizablePanels, TeamSwitcher, TocProgress,
    TocSection, ToggleGroup, TreeView, UserMenu, WorkspaceTile,
};
#[cfg(oxyn_pro)]
pub use primitives::{
    ActionButton, EmptyState, EntityChip, ExpandingButtonGroup, FloatingButtonGroup, IconButton,
    Kbd, Monogram, NumberTicker, Popover, RadioGroup, ReactionBar, Separator, SplitButton, Tooltip,
    TooltipProvider, ValuePill,
};
#[cfg(oxyn_pro)]
pub use primitives::{
    AiCommand, AiSearchBar, Alert, BottomSheet, ControlCenter, ExpandingSearch, Line, MorphBlock,
    NotificationStack, Pagination, ScriptedAnswer, ScrollArea, SearchField, SeedToast,
    SettingsCommand, Sheet, SheetStack, ShortcutList, ShortcutRecorder, SkeletonReveal, StackPage,
    StackRow, Toaster, Toasts, use_toasts,
};
#[cfg(oxyn_pro)]
pub use primitives::{
    ApprovalQuestions, DatePicker, DateRangePicker, DecisionCard, FilterMenu, FormAutofill,
    MultiSelect, PasswordField, StepperForm, TimeDial, TimePicker,
};
pub use primitives::{
    Avatar, AvatarGroup, Badge, Button, ButtonGroup, Bx, Checkbox, Chip, DropdownMenu, Glide,
    PRIMITIVES_CSS, SegmentedControl, Select, Slider, Spinner, Switch, TextField, fill_from,
    presence_from, spoken_shortcut, use_glide,
};
pub use primitives::{
    Calendar, Combobox, DateWheel, InlineEdit, OtpInput, PasswordStrength, PhoneInput, TagInput,
};
pub use primitives::{Card, CommandPalette, Dialog, ShimmerLabel, Skeleton};
pub use primitives::{CodeBlock, CopyButton, Tabs};
#[cfg(oxyn_pro)]
pub use run::{
    PlanCard, PlanCardProps, StepTimeline, StepTimelineProps, TodoList, TodoListProps, ToolStack,
    ToolStackProps,
};
pub use run::{RUN_CSS, SubagentTree, SubagentTreeProps, ToolCallRow, ToolCallRowProps};
#[cfg(oxyn_pro)]
pub use shell::{
    AiSidePanel, AiSidePanelProps, AppShell, AppShellProps, ChatShell, ChatShellProps, Harness,
    HarnessProps, InboxSidebar, InboxSidebarProps, Sidebar, SidebarNav, SidebarNavProps,
    SidebarProps, SidebarRail, SidebarRailProps, TerminalTabs, TerminalTabsProps, sample_inbox,
    sample_page, sample_sections,
};
pub use shell::{SHELL_CSS, WorkspaceSidebar, WorkspaceSidebarProps};
#[cfg(oxyn_pro)]
pub use status_indicator::{
    IndicatorSize, IndicatorVariant, STATUS_INDICATOR_CSS, StatusIndicator, StatusIndicatorProps,
};
pub use status_text::{
    ElapsedTimer, ElapsedTimerProps, STATUS_TEXT_CSS, TextReveal, TextRevealProps, TextShimmer,
    TextShimmerProps, VerbCycler, VerbCyclerProps,
};

/// Old names of components renamed in the naming pass (docs/renames.md). They still work so
/// existing code keeps building; use self::{AutonomyLevels as AutonomyDial, ContextTreemap as ContextMap, CrewChat as CollabStream, CrewTree as AgentCrew, DateWheel as DateReel, ModelKnockout as JudgeBracket, PresenceCursors as AgentCursors, SpendBreakdown as CostWaterfall, TaskGrid as FanoutGrid, ToolReach as ToolAccessMap};
#[cfg(oxyn_pro)]
use self::{
    ActivityPill as RunningNow, AssignBoard as DelegateBoard, AttemptRace as ParallelAttempts,
    BlindCompare as ModelArena, ChangeLedger as AgentLedger, FindingsFeed as AgentFindings,
    FleetOverview as MissionControl, FleetTimeline as MissionTimeline,
    FormAutofill as SmartFormFill, InlineConfirm as ConfirmMorph,
    InvoiceDeskScreen as InvoiceStudioScreen, JobQueue as AgentQueue,
    LaunchChecklist as ReleaseReadiness, LiveLanes as AgentLanes, LoadPlanner as RateLimitPlanner,
    OvernightRecap as AgentRecap, PromptBench as PromptStudio, RelayTrail as AgentHandoff,
    ReturnBrief as AwaySummary, RubberSlider as ElasticSlider, RunMonitor as AgentMonitor,
    ScrollTitle as InViewTitle, SequenceDiagram as AgentSequence, SkeletonReveal as SkeletonMorph,
    StatefulActionButton as ActionSwapButton, TaskInbox as AgentInbox, ToolNetwork as AgentGraph,
    TradingDeskScreen as MarketTerminalScreen, UnfoldSelect as MorphSelect,
};

/// `var(--oxyn-color-…)` for a colour role.
pub fn color_var(role: oxyn_tokens::ColorRole) -> String {
    format!("var(--oxyn-color-{})", role.path().replace('.', "-"))
}
