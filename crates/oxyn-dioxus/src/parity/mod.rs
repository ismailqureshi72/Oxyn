//! Parity components (plan #28): the pieces that close the gaps against affectUI, Arc UI and
//! Beautiful UI, plus the X-post references. Styles ship in [`crate::PRIMITIVES_CSS`].

mod approvals;

pub use approvals::{
    AgentApprovals, ApprovalGates, ConfirmMorph, ConfirmPopover, HoldToConfirm, PermissionRequest,
    gate_dot,
};
