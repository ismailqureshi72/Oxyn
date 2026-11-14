//! Parity components (plan #28): the pieces that close the gaps against affectUI, Arc UI and
//! Beautiful UI, plus the X-post references. Styles ship in [`crate::PRIMITIVES_CSS`].

mod approvals;
mod run;
mod tools;

pub use approvals::{
    AgentApprovals, ApprovalGates, ConfirmMorph, ConfirmPopover, HoldToConfirm, PermissionRequest,
    gate_dot,
};
pub use run::{
    ActivityFeed, AgentFleet, AgentOrchestrator, AgentPlan, AgentRun, AgentStatus, AgentTrail,
    AgentTree, AnimatedCounter, Countdown, FlowStepper, OnboardingChecklist, ReleaseReadiness,
    TaskBreakdown, TaskRows, TreeTable,
};
pub use tools::{
    ChipStep, DiffChip, ExpandableCard, JsonViewer, LinkUnfurl, RowOutput, ToolCall, ToolCallCard,
    ToolCallGroup, ToolCallList, ToolChips, ToolKindLegend,
};
