//! Family F views: tool calls and stacks, plans and todo lists, and the sub-agent tree.

mod plan;
mod tool_call;
mod tree;

pub use plan::{PlanCard, PlanCardProps, StepTimeline, StepTimelineProps, TodoList, TodoListProps};
pub use tool_call::{ToolCallRow, ToolCallRowProps, ToolStack, ToolStackProps};
pub use tree::{SubagentTree, SubagentTreeProps};

/// Stylesheet for family F. Needs the Oxyn token sheet (`tokens/dist/oxyn.css`).
pub const RUN_CSS: &str = include_str!("run.css");
