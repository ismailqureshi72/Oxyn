# Roadmap

Dates are targets, not promises. "Done" means merged to `main` through a co-authored PR.

## V1 — Baseline (2026-11-11) ✅ this PR
Repo, licence, contributing rules, README, vision, architecture, component taxonomy, variant rules,
composer send-control spec, website stack, Studio blueprint, reference index, brand assets.

## V2 — Research & design (before any component code)

| Phase | Goal | Output (in `docs/`) | Exit check |
|---|---|---|---|
| **V2.1 Reference capture** | Turn every saved post, site and app into a catalogued reference | `research/sources.md` (one row per reference: link, family, category, interaction, licence), screenshots/videos kept locally in `/captures` (git-ignored) | Every family A–M has ≥ 10 references |
| **V2.2 Interaction specs** | Write a spec at the composer-arrow depth for **every** component, family by family | `specs/` — one spec per component following `specs/README.md` (starting with thinking, status text, loaders, tool call, approval, checkpoint, diff & merge, context meter, model/effort picker, voice orb, then the rest) | Every component in the taxonomy has states × micro-interactions × timing × variants × edge cases |
| **V2.3 Design language** | Tokens and motion language | [`tokens.md`](tokens.md) + `tokens/*.json`, [`motion.md`](motion.md) (durations, springs, stagger rules, reduced-motion), vector logo + mark states | Tokens reviewed in light + dark; logo legible at 16–20 px |
| **V2.4 Category kits** | Define each agent-app category's shell and screens | [`categories/<name>.md`](categories/README.md) for coding, app builder, CRM, finance, video (first five) | Each lists screens + components used |
| **V2.5 Site & Studio IA** | Information architecture and wireframes | [`site/ia.md`](site/ia.md), [Studio wireframes](site/studio-wireframes.md), [gallery filter model](site/gallery.md) | Both maintainers sign off |
| **V2.6 Tech spike** | Prove the risky parts with throwaway code on a `spike/` branch | Spike report: core state + one GPUI and one Dioxus component, WASM size, headless poster render | Decisions in `website-stack.md` confirmed or revised |

## V3 — Build (after V2)

| Milestone | Scope |
|---|---|
| M0 | Cargo workspace, CI (fmt, clippy, test — no artifacts), tokens pipeline |
| M1 | `oxyn-core` state machine + `StatusIndicator` |
| M2 | Loaders + status-text effects (data-driven, fastest to multiply) |
| M3 | Composer (incl. all send-control variants) |
| M4 | Tool calls, approvals, checkpoints |
| M5 | Diff viewer + 3-way merge conflict editor |
| M6 | Site + gallery + CLI + free registry |
| M7 | Studio v0 (inspector, states, export) |
| M8 | Category shells: coding, app builder, CRM, finance, video |
| M9 | Pro registry, Polar licences, launch |
