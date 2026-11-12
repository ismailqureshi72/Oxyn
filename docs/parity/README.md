# Parity

Oxyn is a **Rust** UI library for agent interfaces. Every component is a real, interactive component in
GPUI and Dioxus, not a placeholder. The bar is set by three sites, covered top to bottom: every
component, screen, block, illustration, motion item and site feature. Oxyn matches each one exactly and
then adds more variants and more user controls. The components from the X posts we collected are in
scope too. More sources will be added here as they come in.

Agreed with Ismail and Khubaib on 2026-11-12.

| Source | Items | What it is | How we use it |
|---|---|---|---|
| [affectUI](https://www.affectui.com/) | 192 (103 components, 41 screens, 14 blocks, 26 illustrations, 8 motion; 56 free) | React kit for agent products | **Design reference for the Oxyn site.** Behaviour only: we rebuild from its public docs, never copy source |
| [Arc UI](https://uiarc.dev/) | 389 rows (135 components, 136 blocks, 106 special, 12 variant pages) | React kit, components + blocks | Behaviour only, rebuilt from public docs |
| [Beautiful UI](https://www.beautifului.dev/) | 21 components, 7 atoms, 11 effects, 17 site features | MIT agent-UI kit | May be ported with attribution (MIT, © 2026 Shane Levine). Never port the paid Central Icons used by its Sidebar Nav |
| X posts (`docs/research/sources.md`) | 51 posts | Agent-UI micro-interactions and screens | Ideas; rebuild, no code |

**653 items** in total. 69 already have an Oxyn component (plans #15 to #21) and still need their gaps
closed; 584 are planned.

## Files
- `status.csv`: one row per item: source, item, url, section, tier, Oxyn spec, batch, plan PR, status.
  **Every PR that builds an item updates its row** (`planned` → `built` → `parity ✓`).
- `affectui.md`, `arc.md`, `beautifului.md`: the full inventories: purpose, states, every prop and
  control, keyboard, micro-interactions, timings, the Oxyn spec it maps to, and an "Oxyn plus" column
  (extra variants and controls). Batch ids (C1, N1, S1…) map to plan PRs in `status.csv`.

## Definition of done (every component PR from plan #23)
1. **Real component.** Interactive, all states reachable, real sample data. No text pills standing in.
2. **Both renderers.** Dioxus (web, SSR) and GPUI (desktop, and GPUI-web where it is used). A gap is
   listed in the PR body.
3. **Exact parity.** Every prop, state, keyboard binding, accessibility and responsive rule on the
   source page is implemented, and checked off in `status.csv`.
4. **Oxyn plus.** At least two extra variants or user controls beyond the source.
5. **Spec.** States, micro-interactions, timing, variants, edge cases and accessibility per
   `docs/specs/README.md`. The depth shown for one component applies to all of them.
6. **Colour.** Chrome is monochrome (dark: black and grey; light: white and grey). Semantic colour stays:
   status, risk, diff, merge, git, review, impact, agent identity, monograms. Other hues only in Studio.
7. **Site page** (from plan #24): live preview, controls panel, variants, code, props, keyboard,
   accessibility, responsive notes, install line. Review sheets are for the maintainers only and are
   not the site.
8. **Licence.** affectUI and Arc: behaviour only. Beautiful UI: port with attribution. Check every other
   source before porting.
9. When stuck on a design, open the matching affectUI page and study it critically.
