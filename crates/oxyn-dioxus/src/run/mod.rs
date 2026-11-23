//! Family F views: tool calls and stacks, plans and todo lists, and the sub-agent tree.

#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod plan;
mod tool_call;
mod tree;

#[cfg(oxyn_pro)]
pub use plan::{PlanCard, PlanCardProps, StepTimeline, StepTimelineProps, TodoList, TodoListProps};
pub use tool_call::{ToolCallRow, ToolCallRowProps};
#[cfg(oxyn_pro)]
pub use tool_call::{ToolStack, ToolStackProps};
pub use tree::{SubagentTree, SubagentTreeProps};

/// Stylesheet for family F. Needs the Oxyn token sheet (`tokens/dist/oxyn.css`).
pub const RUN_CSS: &str = include_str!("run.css");
