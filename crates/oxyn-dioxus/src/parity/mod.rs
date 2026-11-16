//! Parity components (plan #28): the pieces that close the gaps against affectUI, Arc UI and
//! Beautiful UI, plus the X-post references. Styles ship in [`crate::PRIMITIVES_CSS`].

mod agentsfid;
mod answers;
mod approvals;
mod attachfid;
mod autonomyfid;
mod awayfid;
mod chatfid;
mod collabfid;
mod compose;
mod costfid;
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
mod gooeyfid;
mod graphfid;
mod handofffid;
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
mod parallelfid;
mod planfid;
mod queuefid;
mod ratefid;
mod reasonfid;
mod recapfid;
mod review;
mod rootfid;
mod rulerfid;
mod run;
mod runningfid;
mod schedfid;
mod screensfid;
mod seqfid;
mod stairsfid;
mod storagefid;
mod text;
mod themefid;
mod toolmapfid;
mod tools;
mod tracefid;
mod upload;

pub use agentsfid::{AgentFindings, AgentInbox};
pub use answers::{
    AnswerCard, CitedAnswerCard, MessageActionBar, ThinkingPanel, ThinkingPill, WritingAssistant,
};
pub use approvals::{
    AgentApprovals, ApprovalGates, ConfirmMorph, ConfirmPopover, HoldToConfirm, PermissionRequest,
    gate_dot,
};
pub use attachfid::{AttachmentChips, DropLanes, FileIcon, PolaroidStack, UploadDropzone};
pub use autonomyfid::AutonomyDial;
pub use awayfid::AwaySummary;
pub use chatfid::{AgentRunPanel, ModelPickerMenu};
pub use collabfid::CollabStream;
pub use compose::{
    AiComposer, MenuModel, ModelMenu, ModelPill, PillModel, RichTextEditor, RunControls,
};
pub use costfid::CostWaterfall;
pub use crewfid::AgentCrew;
pub use ctxefffid::ContextEffortMenu;
pub use ctxmapfid::ContextMap;
pub use ctxvisfid::{ContextBar, ContextBreakdown, ContextStack, ContextTide};
pub use cursorsfid::AgentCursors;
pub use delegatefid::DelegateBoard;
pub use diff::{ChangeSummary, CodeSuggestion, DiffTable, DiffTimeline, DiffViewer, InlineDiff};
pub use difftlfid::DiffTimelineRun;
pub use dtreefid::DecisionTree;
pub use efflistfid::EffortList;
pub use effpillfid::EffortPill;
pub use fallbackfid::FallbackChain;
pub use fanfid::FanoutGrid;
pub use fidelity::{
    AgentCommentThread, AgentOrb, ChatComposer, CommentsPanel, FileTypeIcon, PhotoAvatar, Portrait,
    ThreadNote,
};
pub use fields::{ElasticSlider, MorphSelect, NumberField, ScrubInput};
pub use gooeyfid::{GooeyAttachMenu, GooeyComposer, GooeyModelMenu, LiquidDrops, LiquidTabs};
pub use graphfid::AgentGraph;
pub use handofffid::AgentHandoff;
pub use home::{
    ActivityHeatmap, ActivityHome, AttachmentStrip, DocGrid, ExperimentGrid, PollingCost,
    RunModeMenu, ThreadList, TokenUsageChart,
};
pub use input::{GhostText, LabeledInput, LabeledTextarea, TextArea};
pub use judgefid::JudgeBracket;
pub use lanesfid::AgentLanes;
pub use ledgerfid::AgentLedger;
pub use memoryfid::MemoryPanel;
pub use merge::{ConflictRadar, MergeConflict};
pub use meters::{ContextField, ContextWindowGrid, CtxModel, Gauge, SplitBar, UsageMeter};
pub use modeleffid::ModelEffortMenu;
pub use monitorfid::AgentMonitor;
pub use nozzlefid::NozzleSlider;
pub use parallelfid::ParallelAttempts;
pub use planfid::PlanVsActual;
pub use queuefid::AgentQueue;
pub use ratefid::RateLimitPlanner;
pub use reasonfid::ReasoningPanel;
pub use recapfid::AgentRecap;
pub use review::{AiInlineEdit, CommentThread, ImageCompare, ReviewCard, ReviewCenter};
pub use rootfid::RootCauseTree;
pub use rulerfid::EffortRuler;
pub use run::{
    ActivityFeed, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus, AgentTrail,
    AgentTree, AnimatedCounter, Countdown, FlowStepper, OnboardingChecklist, ReleaseReadiness,
    TaskBreakdown, TaskRows, TreeTable,
};
pub use runningfid::RunningNow;
pub use schedfid::ScheduledRuns;
pub use screensfid::{
    AppFrame, AppShellScreen, ChatAgentsScreen, ChatOverviewScreen, ChatProgressScreen,
    ChatRailScreen, ChatRunScreen, ChatStartScreen, FoldersScreen, MenuButton, RailFrame,
    RailShellScreen, ScreenComposer,
};
pub use seqfid::AgentSequence;
pub use stairsfid::EffortStairs;
pub use storagefid::{StorageCleanup, StorageForecast, StorageMeter, StorageWaffle};
pub use text::{
    ActionSwapButton, InViewTitle, LoadingState, SlotReels, Source, StreamingText, SweepText,
    TextMorph, ThinkingLoader, WordReveal,
};
pub use themefid::{
    AccentDensity, AppIconPicker, BackgroundPicker, DisplayComfort, LightPlayground, ThemeGrid,
    ThemeSchedule, ThemeSplit, TypeScale,
};
pub use toolmapfid::ToolAccessMap;
pub use tools::{
    ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard,
    ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend,
};
pub use tracefid::TraceWaterfall;
pub use upload::{AvatarUpload, FileDropzone};
