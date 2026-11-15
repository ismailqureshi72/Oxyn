//! Parity components (plan #28): the pieces that close the gaps against affectUI, Arc UI and
//! Beautiful UI, plus the X-post references. Styles ship in [`crate::PRIMITIVES_CSS`].

mod answers;
mod approvals;
mod chatfid;
mod compose;
mod diff;
mod fidelity;
mod fields;
mod home;
mod input;
mod merge;
mod meters;
mod review;
mod run;
mod text;
mod tools;
mod upload;

pub use answers::{
    AnswerCard, CitedAnswerCard, MessageActionBar, ThinkingPanel, ThinkingPill, WritingAssistant,
};
pub use approvals::{
    AgentApprovals, ApprovalGates, ConfirmMorph, ConfirmPopover, HoldToConfirm, PermissionRequest,
    gate_dot,
};
pub use chatfid::{AgentRunPanel, ModelPickerMenu};
pub use compose::{
    AiComposer, MenuModel, ModelMenu, ModelPill, PillModel, RichTextEditor, RunControls,
};
pub use diff::{ChangeSummary, CodeSuggestion, DiffTable, DiffTimeline, DiffViewer, InlineDiff};
pub use fidelity::{
    AgentCommentThread, AgentOrb, ChatComposer, CommentsPanel, FileTypeIcon, PhotoAvatar, Portrait,
    ThreadNote,
};
pub use fields::{ElasticSlider, MorphSelect, NumberField, ScrubInput};
pub use home::{
    ActivityHeatmap, ActivityHome, AttachmentStrip, DocGrid, ExperimentGrid, PollingCost,
    RunModeMenu, ThreadList, TokenUsageChart,
};
pub use input::{GhostText, LabeledInput, LabeledTextarea, TextArea};
pub use merge::{ConflictRadar, MergeConflict};
pub use meters::{ContextField, ContextWindowGrid, CtxModel, Gauge, SplitBar, UsageMeter};
pub use review::{AiInlineEdit, CommentThread, ImageCompare, ReviewCard, ReviewCenter};
pub use run::{
    ActivityFeed, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus, AgentTrail,
    AgentTree, AnimatedCounter, Countdown, FlowStepper, OnboardingChecklist, ReleaseReadiness,
    TaskBreakdown, TaskRows, TreeTable,
};
pub use text::{
    ActionSwapButton, InViewTitle, LoadingState, SlotReels, Source, StreamingText, SweepText,
    TextMorph, ThinkingLoader, WordReveal,
};
pub use tools::{
    ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard,
    ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend,
};
pub use upload::{AvatarUpload, FileDropzone};
