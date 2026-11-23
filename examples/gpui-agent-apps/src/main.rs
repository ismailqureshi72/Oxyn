//! Every Oxyn screen set as interactive GPUI views in one native window. The strip at the top
//! picks a screen set (S0 app shells … S8 onboarding, S12 to S15 Arc screens) or one of the agent
//! apps (Taskdeck, Command Center, Session Studio, Fleet Window, Workspace Coder, Command Board,
//! Threadbar, Tabstack, Repo Desk, Preview Desk, Hangar, Halo Chat, Pilot Panel, Courier).
//!
//! Screen sets: press the sidebar, buttons and the account row as in any app; the bar above the
//! screen has the set's screens, the state bar and a command bar (type a screen or a state, then
//! Enter). With focus on the screen, Left / Right step the states, Up / Down the screens, Escape
//! closes the menu and then returns to Data, `/` jumps to the command bar.
//!
//! Agent apps: press rows, nav, tabs, switches and the account row, type in the composer and send
//! with Enter; Up / Down move the open row, Left / Right step through the app's views, Escape
//! closes the menu, then the rail.
//!
//! Run from the repo root: `cargo run --manifest-path examples/gpui-agent-apps/Cargo.toml`.
//! `OXYN_OPEN=<n>` opens strip item `n` first (0 is S0, 9 is S12 Arc screens I, 10 is S13 Arc
//! screens II, 11 is S14 Arc screens III, 12 is S15 Arc screens IV).
//!
//! Arc screens I: press a board card, then a column (or drag the card there), Alt + arrows move
//! the picked card; press a timeline bar, then Earlier / Later (Alt + ← / →) or drag it onto a
//! day; type in the directory's search; switch the metrics range, KPI and Compare.
//!
//! Arc screens II: press a calendar event, then a day header or the nudge buttons (Alt + arrows),
//! or drag it onto a day; J / K / E / H / X / U / A triage the inbox; type a support reply and
//! press Enter; search the help center.
//!
//! Arc screens III: pick a symbol and range, step the ticket and place it; filter the products,
//! open one, pick a size and add it; add, step and remove invoice lines, then Send; fill the
//! demo account and sign in.
//!
//! Arc screens IV: range and KPI tabs, audit filters, status days, create, roll and revoke keys,
//! invite and change roles, pick a plan; the agent's tools: modes, Approve / Reject, Run again.
//!
//! The viewer itself is part of Oxyn Pro: it lives in the private `oxyn-pro` repository and is
//! overlaid by CI (see CONTRIBUTING.md). Without it this example builds and prints a note.

#[cfg(oxyn_pro)]
#[rustfmt::skip]
mod viewer;

fn main() {
    #[cfg(oxyn_pro)]
    viewer::main();
    #[cfg(not(oxyn_pro))]
    eprintln!("The GPUI screen viewer is part of Oxyn Pro: build it with the Pro sources overlaid.");
}
