//! Parity components (plan #28): the pieces that close the gaps against affectUI, Arc UI and
//! Beautiful UI, plus the X-post references. Styles ship in [`crate::PRIMITIVES_CSS`].

#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod agentscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod agentsfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod answers;
mod approvals;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod arcscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod arcscr2;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod arcscr3;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod arcscr4;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod attachfid;
mod autonomyfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod awayfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod buildscr;
mod chatfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod cmdboard;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod cmdstudio;
mod collabfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod compose;
mod costfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod courier;
mod crewfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod ctxefffid;
mod ctxmapfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod ctxvisfid;
mod cursorsfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod delegatefid;
mod diff;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod difftlfid;
mod dtreefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod efflistfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod effpillfid;
mod fallbackfid;
mod fanfid;
mod fidelity;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod fields;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod fleetwin;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod gooeyfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod graphfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod halochat;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod handofffid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod hangar;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod home;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod input;
mod judgefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod lanesfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod ledgerfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod memoryfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod merge;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod meters;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod modeleffid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod monitorfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod nozzlefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod onbscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod opsscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod parallelfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod pilotpanel;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod planfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod previewdesk;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod queuefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod ratefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod reasonfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod recapfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod repodesk;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod review;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod rootfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod rulerfid;
mod run;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod runningfid;
mod schedfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod screens2fid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod screensfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod seqfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod setscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod setscr2;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod stairsfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod storagefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod tabstack;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod taskdeck;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod text;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod themefid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod threadbar;
mod toolmapfid;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod tools;
mod tracefid;
mod upload;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod workscr;
#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod wscoder;

#[cfg(oxyn_pro)]
pub use agentscr::{ActivityScreen, AgentThreadScreen, InboxBoardScreen, InboxScreen, MemoryScreen, TasksScreen, s3_states};
#[cfg(oxyn_pro)]
pub use agentsfid::{FindingsFeed, TaskInbox};
#[cfg(oxyn_pro)]
pub use answers::{AnswerCard, CitedAnswerCard, MessageActionBar, ThinkingPanel, ThinkingPill, WritingAssistant};
pub use approvals::{ConfirmPopover, gate_dot};
#[cfg(oxyn_pro)]
pub use approvals::{AgentApprovals, ApprovalGates, HoldToConfirm, InlineConfirm, PermissionRequest};
#[cfg(oxyn_pro)]
pub use arcscr::{GanttTimelineScreen, KanbanBoardScreen, MetricsDashboardScreen, ProjectBoardScreen, TeamDirectoryScreen, s12_states};
#[cfg(oxyn_pro)]
pub use arcscr2::{HelpCenterScreen, InboxTriageScreen, MonthCalendarScreen, SupportConversationScreen, WeekCalendarScreen, s13_states};
#[cfg(oxyn_pro)]
pub use arcscr3::{InvoiceDeskScreen, ProductDetailScreen, ProductListScreen, SignInSplitScreen, TradingDeskScreen, s14_states};
#[cfg(oxyn_pro)]
pub use arcscr4::{AdminOverviewScreen, AgentToolCallsScreen, ApiKeysWebhooksScreen, AuditLogScreen, BillingUsageScreen, MembersRolesScreen, StatusBoardScreen, s15_slugs, s15_states};
#[cfg(oxyn_pro)]
pub use attachfid::{AttachmentChips, DropLanes, FileIcon, PolaroidStack, UploadDropzone};
pub use autonomyfid::AutonomyLevels;
#[cfg(oxyn_pro)]
pub use awayfid::ReturnBrief;
#[cfg(oxyn_pro)]
pub use buildscr::{EvalsScreen, s5_states};
pub use chatfid::{AgentRunPanel, ModelPickerMenu};
#[cfg(oxyn_pro)]
pub use cmdboard::{CommandBoardScreen, s11_board_states};
#[cfg(oxyn_pro)]
pub use cmdstudio::{CommandCenterScreen, SessionStudioScreen, s10_states};
pub use collabfid::CrewChat;
#[cfg(oxyn_pro)]
pub use compose::{AiComposer, MenuModel, ModelMenu, ModelPill, PillModel, RichTextEditor, RunControls};
pub use costfid::SpendBreakdown;
#[cfg(oxyn_pro)]
pub use courier::{CourierScreen, s16_courier_states};
pub use crewfid::CrewTree;
#[cfg(oxyn_pro)]
pub use ctxefffid::ContextEffortMenu;
pub use ctxmapfid::ContextTreemap;
#[cfg(oxyn_pro)]
pub use ctxvisfid::{ContextBar, ContextBreakdown, ContextStack, ContextTide};
pub use cursorsfid::PresenceCursors;
#[cfg(oxyn_pro)]
pub use delegatefid::AssignBoard;
pub use diff::{CodeSuggestion, DiffTimeline, DiffViewer, InlineDiff};
#[cfg(oxyn_pro)]
pub use diff::{ChangeSummary, DiffTable};
#[cfg(oxyn_pro)]
pub use difftlfid::DiffTimelineRun;
pub use dtreefid::DecisionTree;
#[cfg(oxyn_pro)]
pub use efflistfid::EffortList;
#[cfg(oxyn_pro)]
pub use effpillfid::EffortPill;
pub use fallbackfid::FallbackChain;
pub use fanfid::TaskGrid;
pub use fidelity::{ChatComposer, FileTypeIcon};
#[cfg(oxyn_pro)]
pub use fidelity::{AgentCommentThread, AgentOrb, CommentsPanel, PhotoAvatar, Portrait, ThreadNote};
#[cfg(oxyn_pro)]
pub use fields::{NumberField, RubberSlider, ScrubInput, UnfoldSelect};
#[cfg(oxyn_pro)]
pub use fleetwin::{FleetWindowScreen, s11_fleet_states};
#[cfg(oxyn_pro)]
pub use gooeyfid::{GooeyAttachMenu, GooeyComposer, GooeyModelMenu, LiquidDrops, LiquidTabs};
#[cfg(oxyn_pro)]
pub use graphfid::ToolNetwork;
#[cfg(oxyn_pro)]
pub use halochat::{HaloChatScreen, s16_halochat_states};
#[cfg(oxyn_pro)]
pub use handofffid::RelayTrail;
#[cfg(oxyn_pro)]
pub use hangar::{HangarScreen, s16_hangar_states};
#[cfg(oxyn_pro)]
pub use home::{ActivityHeatmap, ActivityHome, AttachmentStrip, DocGrid, ExperimentGrid, PollingCost, RunModeMenu, ThreadList, TokenUsageChart};
#[cfg(oxyn_pro)]
pub use input::{GhostText, LabeledInput, LabeledTextarea, TextArea};
pub use judgefid::ModelKnockout;
#[cfg(oxyn_pro)]
pub use lanesfid::LiveLanes;
#[cfg(oxyn_pro)]
pub use ledgerfid::ChangeLedger;
#[cfg(oxyn_pro)]
pub use memoryfid::MemoryPanel;
#[cfg(oxyn_pro)]
pub use merge::{ConflictRadar, MergeConflict};
#[cfg(oxyn_pro)]
pub use meters::{ContextField, ContextWindowGrid, CtxModel, Gauge, SplitBar, UsageMeter};
#[cfg(oxyn_pro)]
pub use modeleffid::ModelEffortMenu;
#[cfg(oxyn_pro)]
pub use monitorfid::RunMonitor;
#[cfg(oxyn_pro)]
pub use nozzlefid::NozzleSlider;
#[cfg(oxyn_pro)]
pub use onbscr::{OnboardingBrandScreen, OnboardingReadyScreen, OnboardingToneScreen, OnboardingWebsiteScreen, s8_states};
#[cfg(oxyn_pro)]
pub use opsscr::{AgentsScreen, PermissionsScreen, RunDetailScreen, SchedulesScreen, SkillsScreen, UsageScreen, s4_states};
#[cfg(oxyn_pro)]
pub use parallelfid::AttemptRace;
#[cfg(oxyn_pro)]
pub use pilotpanel::{PilotPanelScreen, s16_pilotpanel_states};
#[cfg(oxyn_pro)]
pub use planfid::PlanVsActual;
#[cfg(oxyn_pro)]
pub use previewdesk::{PreviewDeskScreen, s16_previewdesk_states};
#[cfg(oxyn_pro)]
pub use queuefid::JobQueue;
#[cfg(oxyn_pro)]
pub use ratefid::LoadPlanner;
#[cfg(oxyn_pro)]
pub use reasonfid::ReasoningPanel;
#[cfg(oxyn_pro)]
pub use recapfid::OvernightRecap;
#[cfg(oxyn_pro)]
pub use repodesk::{RepoDeskScreen, s16_repodesk_states};
#[cfg(oxyn_pro)]
pub use review::{AiInlineEdit, CommentThread, ImageCompare, ReviewCard, ReviewCenter};
#[cfg(oxyn_pro)]
pub use rootfid::RootCauseTree;
#[cfg(oxyn_pro)]
pub use rulerfid::EffortRuler;
pub use run::{AgentTrail, FlowStepper};
#[cfg(oxyn_pro)]
pub use run::{ActivityFeed, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus, AgentTree, AnimatedCounter, Countdown, LaunchChecklist, OnboardingChecklist, TaskBreakdown, TaskRows, TreeTable};
#[cfg(oxyn_pro)]
pub use runningfid::ActivityPill;
pub use schedfid::ScheduledRuns;
#[cfg(oxyn_pro)]
pub use screens2fid::OrchestrationScreen;
#[cfg(oxyn_pro)]
pub use screensfid::{AppFrame, AppShellScreen, ChatAgentsScreen, ChatOverviewScreen, ChatProgressScreen, ChatRailScreen, ChatRunScreen, ChatStartScreen, MenuButton, RailFrame, RailShellScreen, ScreenComposer, ScreenStage, SettingsScreen, StageScale, WorkflowScreen};
#[cfg(oxyn_pro)]
pub use seqfid::SequenceDiagram;
#[cfg(oxyn_pro)]
pub use setscr::{BillingSettingsScreen, GeneralSettingsScreen, MembersSettingsScreen, ModelsSettingsScreen, NotificationsSettingsScreen, OnboardingSettingsScreen, PreferencesSettingsScreen, PrivacySettingsScreen, ProfileSettingsScreen, SecuritySettingsScreen, s6_states};
#[cfg(oxyn_pro)]
pub use stairsfid::EffortStairs;
#[cfg(oxyn_pro)]
pub use storagefid::{StorageCleanup, StorageForecast, StorageMeter, StorageWaffle};
#[cfg(oxyn_pro)]
pub use tabstack::{TabstackScreen, s16_tabstack_states};
#[cfg(oxyn_pro)]
pub use taskdeck::{TaskdeckBoardScreen, TaskdeckPluginsScreen, TaskdeckScheduledScreen, TaskdeckSitesScreen, TaskdeckStartScreen, TaskdeckTaskScreen, s9_states};
#[cfg(oxyn_pro)]
pub use text::{LoadingState, ScrollTitle, SlotReels, Source, StatefulActionButton, StreamingText, SweepText, TextMorph, ThinkingLoader, WordReveal};
#[cfg(oxyn_pro)]
pub use themefid::{AccentDensity, AppIconPicker, BackgroundPicker, DisplayComfort, LightPlayground, ThemeGrid, ThemeSchedule, ThemeSplit, TypeScale};
#[cfg(oxyn_pro)]
pub use threadbar::{ThreadbarScreen, s16_threadbar_states};
pub use toolmapfid::ToolReach;
#[cfg(oxyn_pro)]
pub use tools::{ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard, ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend};
pub use tracefid::TraceWaterfall;
pub use upload::FileDropzone;
#[cfg(oxyn_pro)]
pub use upload::AvatarUpload;
#[cfg(oxyn_pro)]
pub use workscr::{AnalyticsScreen, CrmScreen, FoldersScreen, IntegrationsScreen, KanbanScreen, s2_states};
#[cfg(oxyn_pro)]
pub use wscoder::{WorkspaceCoderScreen, s11_coder_states};
