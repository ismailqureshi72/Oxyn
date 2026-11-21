//! Parity components (plan #28): the pieces that close the gaps against affectUI, Arc UI and
//! Beautiful UI, plus the X-post references. Styles ship in [`crate::PRIMITIVES_CSS`].

mod agentscr;
mod agentsfid;
mod answers;
mod approvals;
mod arcscr;
mod arcscr2;
mod arcscr3;
mod arcscr4;
mod attachfid;
mod autonomyfid;
mod awayfid;
mod buildscr;
mod chatfid;
mod cmdboard;
mod cmdstudio;
mod collabfid;
mod compose;
mod costfid;
mod courier;
mod crewfid;
mod ctxefffid;
mod ctxmapfid;
mod ctxvisfid;
mod cursorsfid;
mod delegatefid;
mod diff;
mod difftlfid;
mod dtreefid;
mod efflistfid;
mod effpillfid;
mod fallbackfid;
mod fanfid;
mod fidelity;
mod fields;
mod fleetwin;
mod gooeyfid;
mod graphfid;
mod halochat;
mod handofffid;
mod hangar;
mod home;
mod input;
mod judgefid;
mod lanesfid;
mod ledgerfid;
mod memoryfid;
mod merge;
mod meters;
mod modeleffid;
mod monitorfid;
mod nozzlefid;
mod onbscr;
mod opsscr;
mod parallelfid;
mod pilotpanel;
mod planfid;
mod previewdesk;
mod queuefid;
mod ratefid;
mod reasonfid;
mod recapfid;
mod repodesk;
mod review;
mod rootfid;
mod rulerfid;
mod run;
mod runningfid;
mod schedfid;
mod screens2fid;
mod screensfid;
mod seqfid;
mod setscr;
mod setscr2;
mod stairsfid;
mod storagefid;
mod tabstack;
mod taskdeck;
mod text;
mod themefid;
mod threadbar;
mod toolmapfid;
mod tools;
mod tracefid;
mod upload;
mod workscr;
mod wscoder;

pub use agentscr::{
    ActivityScreen, AgentThreadScreen, InboxBoardScreen, InboxScreen, MemoryScreen, TasksScreen,
    s3_states,
};
pub use agentsfid::{FindingsFeed, TaskInbox};
pub use answers::{
    AnswerCard, CitedAnswerCard, MessageActionBar, ThinkingPanel, ThinkingPill, WritingAssistant,
};
pub use approvals::{
    AgentApprovals, ApprovalGates, ConfirmPopover, HoldToConfirm, InlineConfirm, PermissionRequest,
    gate_dot,
};
pub use arcscr::{
    GanttTimelineScreen, KanbanBoardScreen, MetricsDashboardScreen, ProjectBoardScreen,
    TeamDirectoryScreen, s12_states,
};
pub use arcscr2::{
    HelpCenterScreen, InboxTriageScreen, MonthCalendarScreen, SupportConversationScreen,
    WeekCalendarScreen, s13_states,
};
pub use arcscr3::{
    InvoiceDeskScreen, ProductDetailScreen, ProductListScreen, SignInSplitScreen,
    TradingDeskScreen, s14_states,
};
pub use arcscr4::{
    AdminOverviewScreen, AgentToolCallsScreen, ApiKeysWebhooksScreen, AuditLogScreen,
    BillingUsageScreen, MembersRolesScreen, StatusBoardScreen, s15_slugs, s15_states,
};
pub use attachfid::{AttachmentChips, DropLanes, FileIcon, PolaroidStack, UploadDropzone};
pub use autonomyfid::AutonomyLevels;
pub use awayfid::ReturnBrief;
pub use buildscr::{EvalsScreen, s5_states};
pub use chatfid::{AgentRunPanel, ModelPickerMenu};
pub use cmdboard::{CommandBoardScreen, s11_board_states};
pub use cmdstudio::{CommandCenterScreen, SessionStudioScreen, s10_states};
pub use collabfid::CrewChat;
pub use compose::{
    AiComposer, MenuModel, ModelMenu, ModelPill, PillModel, RichTextEditor, RunControls,
};
pub use costfid::SpendBreakdown;
pub use courier::{CourierScreen, s16_courier_states};
pub use crewfid::CrewTree;
pub use ctxefffid::ContextEffortMenu;
pub use ctxmapfid::ContextTreemap;
pub use ctxvisfid::{ContextBar, ContextBreakdown, ContextStack, ContextTide};
pub use cursorsfid::PresenceCursors;
pub use delegatefid::AssignBoard;
pub use diff::{ChangeSummary, CodeSuggestion, DiffTable, DiffTimeline, DiffViewer, InlineDiff};
pub use difftlfid::DiffTimelineRun;
pub use dtreefid::DecisionTree;
pub use efflistfid::EffortList;
pub use effpillfid::EffortPill;
pub use fallbackfid::FallbackChain;
pub use fanfid::TaskGrid;
pub use fidelity::{
    AgentCommentThread, AgentOrb, ChatComposer, CommentsPanel, FileTypeIcon, PhotoAvatar, Portrait,
    ThreadNote,
};
pub use fields::{NumberField, RubberSlider, ScrubInput, UnfoldSelect};
pub use fleetwin::{FleetWindowScreen, s11_fleet_states};
pub use gooeyfid::{GooeyAttachMenu, GooeyComposer, GooeyModelMenu, LiquidDrops, LiquidTabs};
pub use graphfid::ToolNetwork;
pub use halochat::{HaloChatScreen, s16_halochat_states};
pub use handofffid::RelayTrail;
pub use hangar::{HangarScreen, s16_hangar_states};
pub use home::{
    ActivityHeatmap, ActivityHome, AttachmentStrip, DocGrid, ExperimentGrid, PollingCost,
    RunModeMenu, ThreadList, TokenUsageChart,
};
pub use input::{GhostText, LabeledInput, LabeledTextarea, TextArea};
pub use judgefid::ModelKnockout;
pub use lanesfid::LiveLanes;
pub use ledgerfid::ChangeLedger;
pub use memoryfid::MemoryPanel;
pub use merge::{ConflictRadar, MergeConflict};
pub use meters::{ContextField, ContextWindowGrid, CtxModel, Gauge, SplitBar, UsageMeter};
pub use modeleffid::ModelEffortMenu;
pub use monitorfid::RunMonitor;
pub use nozzlefid::NozzleSlider;
pub use onbscr::{
    OnboardingBrandScreen, OnboardingReadyScreen, OnboardingToneScreen, OnboardingWebsiteScreen,
    s8_states,
};
pub use opsscr::{
    AgentsScreen, PermissionsScreen, RunDetailScreen, SchedulesScreen, SkillsScreen, UsageScreen,
    s4_states,
};
pub use parallelfid::AttemptRace;
pub use pilotpanel::{PilotPanelScreen, s16_pilotpanel_states};
pub use planfid::PlanVsActual;
pub use previewdesk::{PreviewDeskScreen, s16_previewdesk_states};
pub use queuefid::JobQueue;
pub use ratefid::LoadPlanner;
pub use reasonfid::ReasoningPanel;
pub use recapfid::OvernightRecap;
pub use repodesk::{RepoDeskScreen, s16_repodesk_states};
pub use review::{AiInlineEdit, CommentThread, ImageCompare, ReviewCard, ReviewCenter};
pub use rootfid::RootCauseTree;
pub use rulerfid::EffortRuler;
pub use run::{
    ActivityFeed, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus, AgentTrail,
    AgentTree, AnimatedCounter, Countdown, FlowStepper, LaunchChecklist, OnboardingChecklist,
    TaskBreakdown, TaskRows, TreeTable,
};
pub use runningfid::ActivityPill;
pub use schedfid::ScheduledRuns;
pub use screens2fid::OrchestrationScreen;
pub use screensfid::{
    AppFrame, AppShellScreen, ChatAgentsScreen, ChatOverviewScreen, ChatProgressScreen,
    ChatRailScreen, ChatRunScreen, ChatStartScreen, MenuButton, RailFrame, RailShellScreen,
    ScreenComposer, ScreenStage, SettingsScreen, StageScale, WorkflowScreen,
};
pub use seqfid::SequenceDiagram;
pub use setscr::{
    BillingSettingsScreen, GeneralSettingsScreen, MembersSettingsScreen, ModelsSettingsScreen,
    NotificationsSettingsScreen, OnboardingSettingsScreen, PreferencesSettingsScreen,
    PrivacySettingsScreen, ProfileSettingsScreen, SecuritySettingsScreen, s6_states,
};
pub use stairsfid::EffortStairs;
pub use storagefid::{StorageCleanup, StorageForecast, StorageMeter, StorageWaffle};
pub use tabstack::{TabstackScreen, s16_tabstack_states};
pub use taskdeck::{
    TaskdeckBoardScreen, TaskdeckPluginsScreen, TaskdeckScheduledScreen, TaskdeckSitesScreen,
    TaskdeckStartScreen, TaskdeckTaskScreen, s9_states,
};
pub use text::{
    LoadingState, ScrollTitle, SlotReels, Source, StatefulActionButton, StreamingText, SweepText,
    TextMorph, ThinkingLoader, WordReveal,
};
pub use themefid::{
    AccentDensity, AppIconPicker, BackgroundPicker, DisplayComfort, LightPlayground, ThemeGrid,
    ThemeSchedule, ThemeSplit, TypeScale,
};
pub use threadbar::{ThreadbarScreen, s16_threadbar_states};
pub use toolmapfid::ToolReach;
pub use tools::{
    ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard,
    ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend,
};
pub use tracefid::TraceWaterfall;
pub use upload::{AvatarUpload, FileDropzone};
pub use workscr::{
    AnalyticsScreen, CrmScreen, FoldersScreen, IntegrationsScreen, KanbanScreen, s2_states,
};
pub use wscoder::{WorkspaceCoderScreen, s11_coder_states};
