//! Parity components (plan #28): the pieces that close the gaps against affectUI, Arc UI and
//! Beautiful UI, plus the X-post references. Styles ship in [`crate::PRIMITIVES_CSS`].

mod agentsfid;
mod answers;
mod approvals;
mod autonomyfid;
mod awayfid;
mod chatfid;
mod collabfid;
mod compose;
mod costfid;
mod crewfid;
mod ctxmapfid;
mod cursorsfid;
mod delegatefid;
mod diff;
mod dtreefid;
mod fallbackfid;
mod fidelity;
mod fields;
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
mod monitorfid;
mod parallelfid;
mod planfid;
mod queuefid;
mod recapfid;
mod review;
mod rootfid;
mod run;
mod runningfid;
mod seqfid;
mod text;
mod toolmapfid;
mod tools;
mod upload;

pub use agentsfid::{AgentFindings, AgentInbox};
pub use answers::{
    AnswerCard, CitedAnswerCard, MessageActionBar, ThinkingPanel, ThinkingPill, WritingAssistant,
};
pub use approvals::{
    AgentApprovals, ApprovalGates, ConfirmMorph, ConfirmPopover, HoldToConfirm, PermissionRequest,
    gate_dot,
};
pub use autonomyfid::AutonomyDial;
pub use awayfid::AwaySummary;
pub use chatfid::{AgentRunPanel, ModelPickerMenu};
pub use collabfid::CollabStream;
pub use compose::{
    AiComposer, MenuModel, ModelMenu, ModelPill, PillModel, RichTextEditor, RunControls,
};
pub use costfid::CostWaterfall;
pub use crewfid::AgentCrew;
pub use ctxmapfid::ContextMap;
pub use cursorsfid::AgentCursors;
pub use delegatefid::DelegateBoard;
pub use diff::{ChangeSummary, CodeSuggestion, DiffTable, DiffTimeline, DiffViewer, InlineDiff};
pub use dtreefid::DecisionTree;
pub use fallbackfid::FallbackChain;
pub use fidelity::{
    AgentCommentThread, AgentOrb, ChatComposer, CommentsPanel, FileTypeIcon, PhotoAvatar, Portrait,
    ThreadNote,
};
pub use fields::{ElasticSlider, MorphSelect, NumberField, ScrubInput};
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
pub use monitorfid::AgentMonitor;
pub use parallelfid::ParallelAttempts;
pub use planfid::PlanVsActual;
pub use queuefid::AgentQueue;
pub use recapfid::AgentRecap;
pub use review::{AiInlineEdit, CommentThread, ImageCompare, ReviewCard, ReviewCenter};
pub use rootfid::RootCauseTree;
pub use run::{
    ActivityFeed, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus, AgentTrail,
    AgentTree, AnimatedCounter, Countdown, FlowStepper, OnboardingChecklist, ReleaseReadiness,
    TaskBreakdown, TaskRows, TreeTable,
};
pub use runningfid::RunningNow;
pub use seqfid::AgentSequence;
pub use text::{
    ActionSwapButton, InViewTitle, LoadingState, SlotReels, Source, StreamingText, SweepText,
    TextMorph, ThinkingLoader, WordReveal,
};
pub use toolmapfid::ToolAccessMap;
pub use tools::{
    ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard,
    ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend,
};
pub use upload::{AvatarUpload, FileDropzone};
