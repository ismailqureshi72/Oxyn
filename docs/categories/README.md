# Category kits

A **category kit** is a full agent app for one vertical, assembled only from Oxyn parts (families A–L
in [../component-taxonomy.md](../component-taxonomy.md)). Each kit ships a **shell**, a **dashboard**
and a **screen set**, around one made-up company so demos read like a real product. Kits become
family M in the registry (`shell/<category>-<screen>`), and are built in #27 and #28.

**Rule:** a kit adds as few new components as it can. A screen that needs something A–L does not
cover lists it under *New components*, with a one-paragraph brief; the full 10-point spec
([../specs/README.md](../specs/README.md)) is written before that component is built.

## Every kit covers
1. **Who it's for**: the job the user hires the agent for, and the made-up company used in demos.
2. **Shell**: the persistent frame (rails, panels, top bar, composer placement) for desktop (GPUI),
   web (Dioxus) and a narrow / mobile layout, with ASCII wireframes and breakpoints.
3. **Screens**: every screen, each with a wireframe, purpose, the Oxyn components it uses (linked to
   their specs), and its empty / loading / error states.
4. **Agent flows**: 3–5 end-to-end runs walked through `AgentState` (Idle → … → Done / Error), naming
   which component shows each step, where approvals stop the run, and what the user can steer.
5. **Approvals and risk**: which actions in this vertical are Low / Medium / High / Critical
   (per [../specs/approvals.md](../specs/approvals.md)); Critical always stops, even in Bypass.
6. **New components**: anything A–L lacks, with a brief, the family it joins, and its tier.
7. **Data model and fixtures**: the records the demo needs (Rust struct sketch) and a fixture plan.
   Fixtures are invented; no real people, companies or logos.
8. **Variants**: shell and screen variants (layout × surface × motion), free vs Pro.
9. **Accessibility and edge cases**: dense data, long names, RTL, keyboard-only, screen readers,
   slow and very fast agents.
10. **References**: products studied, with licence notes (study only unless MIT / Apache / ISC).

Tokens come from [../tokens.md](../tokens.md) and motion from [../motion.md](../motion.md).

## Kits
| Kit | Shell | Status |
|---|---|---|
| [coding.md](coding.md) | Desktop IDE agent window, terminal look, review queue | draft v0.1 |
| [app-builder.md](app-builder.md) | Prompt → preview split, versions, deploy | draft v0.1 |
| [crm.md](crm.md) | Inbox + record view, pipeline board | draft v0.1 |
| [finance.md](finance.md) | Ledger dashboard, reconciliation | draft v0.1 |
| [video.md](video.md) | Timeline editor with agent panel, render queue | draft v0.1 |

Support, research, data / BI, browser / computer-use, voice / live and multi-agent ops follow later
(section B of the taxonomy).
