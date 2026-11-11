# Vision

## One line
**Oxyn is the agent-UI layer for Rust apps:** the composer, thinking, tool, approval, diff and shell
pieces every agent product needs, in more variants than anyone else, tunable in a Studio.

## Why now (as of 2026-11)
- Basic chat UI is commoditised in React (assistant-ui, shadcn absorbing Vercel AI Elements, which is
  deprecated at the end of November 2026). Rust has almost nothing with depth: gpui-ai, a gpui-kit PR
  with AI Elements ports that the maintainer wants outside core, egui-agent-chat, rune-kit (pre-release).
- Agent apps are moving to native desktop (Zed, Codex desktop, Warp, OpenCode desktop) where Rust/GPUI
  is a natural fit.
- Paid React kits (affectUI, Arc UI, Skiper, Aceternity) ship **one** design per component. Variant depth
  and agent-state correctness are open.

## Principles
1. **State first.** Components render an `AgentState`; they never invent their own status logic.
2. **Variants are first-class.** Every component has a variant matrix (layout × surface × motion), named
   and documented the same way ([variants.md](variants.md)).
3. **Micro matters.** We specify timing, easing, frame counts and stall behaviour, not just looks.
4. **Calm by default.** Matte, low-glow, monochrome-plus-one-accent. Expressive motion is opt-in.
5. **Accessible always.** Reduced-motion, keyboard, screen-reader labels and contrast on every variant.
6. **Agent-native docs.** llms.txt, copy-prompt, MCP server and a CLI that coding agents can drive.
7. **Honest sourcing.** Port only MIT/Apache with credit; study everything else and build our own.

## Audience
- Rust developers building agent apps on GPUI (desktop) or Dioxus (web/desktop).
- Teams prototyping vertical agents (CRM, finance, video, app builders) who need a full shell fast.
- Designers who want to tune motion and tokens in the Studio and hand over exact code.

## Free vs Pro (direction, not final)
- **Free (MIT):** core crate, tokens, base variants of every component, CLI, Studio, MCP server, llms.txt.
- **Pro:** extra variants, category kits (shells/dashboards per vertical), premium motion packs,
  Studio extras (saved presets, team libraries, video export). Pricing to be decided after V2 research.

## Not doing (for now)
- React or JS packages (Rust only; JS appears only in the website shell).
- A hosted agent backend. Oxyn is UI; adapters connect to AI SDK / AG-UI / ACP / MCP event streams.
