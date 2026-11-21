//! Dioxus components for Oxyn (web demos, Studio, web apps).
//!
//! Components style themselves with the `--oxyn-*` custom properties from `tokens/dist/oxyn.css`;
//! include that stylesheet once (and set `data-oxyn-theme` for light or high contrast). Each
//! component also ships its own small stylesheet as a constant (`STATUS_INDICATOR_CSS`).

mod aikit;
mod approval;
mod blocks;
mod brand;
mod chat;
mod checkpoint;
mod composer;
mod glyph_loader;
mod kit;
mod primitives;
mod run;
mod shell;
mod status_indicator;
mod status_text;
mod xpost;

pub use aikit::{
    AgentDocCursor, AgentTimeline, AiMascot, AiSpark, AiTranslate, AiWorkflow, AskYourData,
    ConnectorSetup, CubeMascot, EnrichColumn, GenerationStats, GhostMascot, HumanHandoff,
    ImageGeneration, ImageReveal, LiveVisitors, LlmCostCalculator, MeetingSummary, ModelCard,
    PiiRedaction, PixelMascot, ProviderKeys, ProviderStatus, ReferralCard, ReplyDraft,
    SocialProofToast, SpeechPlayer, TicketRouter, TldrToggle, ToneTuner, TrainingRun, TrialBanner,
    UpgradePrompt,
};
pub use approval::{ApprovalCard, ApprovalCardProps, ApprovalPager, ApprovalPagerProps};
pub use blocks::{
    AgentBuilder, AgentGallery, AgentRoutines, AiFeatureLaunch, AiInsightCard, AiSetupWizard,
    AssistantMemory, BackgroundAgents, BlindCompare, CallInsights, CatchUp, EvalSuite,
    FleetOverview, FleetTimeline, KnowledgeBase, LeadScoring, LiveSession, McpServers, PromptBench,
    RunInspector,
};
pub use brand::{BrandIcon, BrandIconProps, has_brand};
pub use chat::{
    AiAnswer, AiChat, CHAT_CSS, ChatThread, ContextCards, DocumentExtraction, Markdown,
    MessageActions, MessageBubble, ReasoningBlock, RecommendationCard, ResearchRun,
    ResponseFeedback, SourceCards, SourceCitation, StarterPrompts, StreamingMarkdown,
    SupportConversation, ThinkingTraces,
};
pub use checkpoint::{
    CheckpointFilmStrip, CheckpointFilmStripProps, CheckpointLanes, CheckpointLanesProps,
    CheckpointList, CheckpointListProps, CheckpointMarker, CheckpointMarkerProps, CheckpointTree,
    CheckpointTreeProps, RailTone, RestorePreviewSheet, RestorePreviewSheetProps, RestoreToast,
    RestoreToastProps, UndoLadder, UndoLadderProps,
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
pub use xpost::{
    AgentPromptCopy, DoneShockwave, LoopPipeline, ProactiveCard, PublishPopover, SecretCard,
    SplitConfirm, TestTicks, TriageMetro, WriteSweep,
};
/// Stylesheet for approvals and checkpoints.
pub const HITL_CSS: &str = include_str!("hitl.css");
pub use kit::AccentDensity;
pub use kit::ActivityPill;
pub use kit::ActivityScreen;
pub use kit::AgentThreadScreen;
pub use kit::AnalyticsScreen;
pub use kit::AppFrame;
pub use kit::AppIconPicker;
pub use kit::AppShellScreen;
pub use kit::AttachmentChips;
pub use kit::AttemptRace;
pub use kit::BackgroundPicker;
pub use kit::ChatAgentsScreen;
pub use kit::ChatOverviewScreen;
pub use kit::ChatProgressScreen;
pub use kit::ChatRailScreen;
pub use kit::ChatRunScreen;
pub use kit::ChatStartScreen;
pub use kit::ContextBar;
pub use kit::ContextBreakdown;
pub use kit::ContextEffortMenu;
pub use kit::ContextStack;
pub use kit::ContextTide;
pub use kit::CrmScreen;
pub use kit::DiffTimelineRun;
pub use kit::DisplayComfort;
pub use kit::DropLanes;
pub use kit::EffortList;
pub use kit::EffortPill;
pub use kit::EffortRuler;
pub use kit::EffortStairs;
pub use kit::FileIcon;
pub use kit::FoldersScreen;
pub use kit::GooeyAttachMenu;
pub use kit::GooeyComposer;
pub use kit::GooeyModelMenu;
pub use kit::IntegrationsScreen;
pub use kit::KanbanScreen;
pub use kit::LightPlayground;
pub use kit::LiquidDrops;
pub use kit::LiquidTabs;
pub use kit::LoadPlanner;
pub use kit::MemoryPanel;
pub use kit::MenuButton;
pub use kit::ModelEffortMenu;
pub use kit::ModelKnockout;
pub use kit::NozzleSlider;
pub use kit::OrchestrationScreen;
pub use kit::PlanVsActual;
pub use kit::PolaroidStack;
pub use kit::RailFrame;
pub use kit::RailShellScreen;
pub use kit::ReasoningPanel;
pub use kit::RootCauseTree;
pub use kit::ScheduledRuns;
pub use kit::ScreenComposer;
pub use kit::ScreenStage;
pub use kit::SettingsScreen;
pub use kit::StageScale;
pub use kit::StorageCleanup;
pub use kit::StorageForecast;
pub use kit::StorageMeter;
pub use kit::StorageWaffle;
pub use kit::TaskGrid;
pub use kit::ThemeGrid;
pub use kit::ThemeSchedule;
pub use kit::ThemeSplit;
pub use kit::ToolReach;
pub use kit::TraceWaterfall;
pub use kit::TypeScale;
pub use kit::UploadDropzone;
pub use kit::WorkflowScreen;
pub use kit::s2_states;
pub use kit::s3_states;
pub use kit::s4_states;
pub use kit::s6_states;
pub use kit::s8_states;
pub use kit::s9_states;
pub use kit::s10_states;
pub use kit::{
    ActivityFeed, AgentApprovals, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus,
    AgentTrail, AgentTree, AnimatedCounter, ApprovalGates, ConfirmPopover, Countdown, FlowStepper,
    HoldToConfirm, InlineConfirm, LaunchChecklist, OnboardingChecklist, PermissionRequest,
    TaskBreakdown, TaskRows, TreeTable, gate_dot,
};
pub use kit::{
    ActivityHeatmap, ActivityHome, AttachmentStrip, DocGrid, ExperimentGrid, PollingCost,
    RunModeMenu, ThreadList, TokenUsageChart,
};
pub use kit::{
    AdminOverviewScreen, AgentToolCallsScreen, ApiKeysWebhooksScreen, AuditLogScreen,
    BillingUsageScreen, MembersRolesScreen, StatusBoardScreen, s15_slugs, s15_states,
};
pub use kit::{
    AgentCommentThread, AgentOrb, AgentRunPanel, CommentsPanel, FileTypeIcon, ModelPickerMenu,
    PhotoAvatar, Portrait, ThreadNote,
};
pub use kit::{
    AgentsScreen, PermissionsScreen, RunDetailScreen, SchedulesScreen, SkillsScreen, UsageScreen,
};
pub use kit::{
    AiComposer, AvatarUpload, ChatComposer, ContextField, ContextWindowGrid, CtxModel,
    FileDropzone, Gauge, MenuModel, ModelMenu, ModelPill, NumberField, PillModel, RichTextEditor,
    RubberSlider, RunControls, ScrubInput, SplitBar, UnfoldSelect, UsageMeter,
};
pub use kit::{
    AiInlineEdit, ChangeSummary, CodeSuggestion, CommentThread, DiffTable, DiffTimeline,
    DiffViewer, ImageCompare, InlineDiff, ReviewCard, ReviewCenter,
};
pub use kit::{
    AnswerCard, CitedAnswerCard, MessageActionBar, ThinkingPanel, ThinkingPill, WritingAssistant,
};
pub use kit::{
    AssignBoard, AutonomyLevels, ChangeLedger, ContextTreemap, CrewChat, CrewTree, DecisionTree,
    FallbackChain, FindingsFeed, JobQueue, LiveLanes, OvernightRecap, PresenceCursors, RelayTrail,
    ReturnBrief, RunMonitor, SequenceDiagram, SpendBreakdown, TaskInbox, ToolNetwork,
};
pub use kit::{
    BillingSettingsScreen, GeneralSettingsScreen, MembersSettingsScreen, ModelsSettingsScreen,
    NotificationsSettingsScreen, OnboardingSettingsScreen, PreferencesSettingsScreen,
    PrivacySettingsScreen, ProfileSettingsScreen, SecuritySettingsScreen,
};
pub use kit::{
    ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard,
    ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend,
};
pub use kit::{CommandBoardScreen, s11_board_states};
pub use kit::{CommandCenterScreen, SessionStudioScreen};
pub use kit::{ConflictRadar, MergeConflict};
pub use kit::{
    CourierScreen, HaloChatScreen, HangarScreen, PilotPanelScreen, s16_courier_states,
    s16_halochat_states, s16_hangar_states, s16_pilotpanel_states,
};
pub use kit::{EvalsScreen, s5_states};
pub use kit::{FleetWindowScreen, s11_fleet_states};
pub use kit::{
    GanttTimelineScreen, KanbanBoardScreen, MetricsDashboardScreen, ProjectBoardScreen,
    TeamDirectoryScreen, s12_states,
};
pub use kit::{GhostText, LabeledInput, LabeledTextarea, TextArea};
pub use kit::{
    HelpCenterScreen, InboxTriageScreen, MonthCalendarScreen, SupportConversationScreen,
    WeekCalendarScreen, s13_states,
};
pub use kit::{InboxBoardScreen, InboxScreen, MemoryScreen, TasksScreen};
pub use kit::{
    InvoiceDeskScreen, ProductDetailScreen, ProductListScreen, SignInSplitScreen,
    TradingDeskScreen, s14_states,
};
pub use kit::{
    LoadingState, ScrollTitle, SlotReels, Source, StatefulActionButton, StreamingText, SweepText,
    TextMorph, ThinkingLoader, WordReveal,
};
pub use kit::{
    OnboardingBrandScreen, OnboardingReadyScreen, OnboardingToneScreen, OnboardingWebsiteScreen,
};
pub use kit::{
    PreviewDeskScreen, RepoDeskScreen, TabstackScreen, ThreadbarScreen, s16_previewdesk_states,
    s16_repodesk_states, s16_tabstack_states, s16_threadbar_states,
};
pub use kit::{
    TaskdeckBoardScreen, TaskdeckPluginsScreen, TaskdeckScheduledScreen, TaskdeckSitesScreen,
    TaskdeckStartScreen, TaskdeckTaskScreen,
};
pub use kit::{WorkspaceCoderScreen, s11_coder_states};
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
    ShortcutRecorder, Skeleton, SkeletonReveal, StackPage, StackRow, Toaster, Toasts, use_toasts,
};
pub use primitives::{
    ApprovalQuestions, Calendar, Combobox, DatePicker, DateRangePicker, DateWheel, DecisionCard,
    FilterMenu, FormAutofill, InlineEdit, MultiSelect, OtpInput, PasswordField, PasswordStrength,
    PhoneInput, StepperForm, TagInput, TimeDial, TimePicker,
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

/// Old names of components renamed in the naming pass (docs/renames.md). They still work so
/// existing code keeps building; use the new names in new code.
#[doc(hidden)]
pub use self::{
    ActivityPill as RunningNow, AssignBoard as DelegateBoard, AttemptRace as ParallelAttempts,
    AutonomyLevels as AutonomyDial, BlindCompare as ModelArena, ChangeLedger as AgentLedger,
    ContextTreemap as ContextMap, CrewChat as CollabStream, CrewTree as AgentCrew,
    DateWheel as DateReel, FindingsFeed as AgentFindings, FleetOverview as MissionControl,
    FleetTimeline as MissionTimeline, FormAutofill as SmartFormFill, InlineConfirm as ConfirmMorph,
    InvoiceDeskScreen as InvoiceStudioScreen, JobQueue as AgentQueue,
    LaunchChecklist as ReleaseReadiness, LiveLanes as AgentLanes, LoadPlanner as RateLimitPlanner,
    ModelKnockout as JudgeBracket, OvernightRecap as AgentRecap, PresenceCursors as AgentCursors,
    PromptBench as PromptStudio, RelayTrail as AgentHandoff, ReturnBrief as AwaySummary,
    RubberSlider as ElasticSlider, RunMonitor as AgentMonitor, ScrollTitle as InViewTitle,
    SequenceDiagram as AgentSequence, SkeletonReveal as SkeletonMorph,
    SpendBreakdown as CostWaterfall, StatefulActionButton as ActionSwapButton,
    TaskGrid as FanoutGrid, TaskInbox as AgentInbox, ToolNetwork as AgentGraph,
    ToolReach as ToolAccessMap, TradingDeskScreen as MarketTerminalScreen,
    UnfoldSelect as MorphSelect,
};

/// `var(--oxyn-color-…)` for a colour role.
pub fn color_var(role: oxyn_tokens::ColorRole) -> String {
    format!("var(--oxyn-color-{})", role.path().replace('.', "-"))
}
