**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Release readiness (`release-readiness`)

Family: F Tool calls & plans ([tools.md](tools.md)). Closes the parity row(s): Arc UI · Release readiness (block).

## 1. Purpose
A launch checklist that keeps prerequisites, progress and the final action together: items grouped (Quality, Docs, Launch) with owners, blockers tagged, a progress bar and a count that swaps with a small rise, and a Release button that is disabled while blockers remain, shows a spinner while it runs, and an inline alert when it fails.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| blocked | Blocker tags, Release disabled |  |
| ready | Release enabled | press → releasing |
| releasing | Spinner, aria-busy | → released / failed |
| failed | Inline alert |  |
| released | Check |  |

## 3. Timing and motion
- Reduced motion: no entrance rise, no shimmer (static muted text), rings and bars jump to their value.

## 4. Variants
| Variant | What differs |
|---|---|
| blocked | One blocker |
| ready | Ready |
| failed | Failure |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Checkboxes, Release |

## 6. Accessibility
- progressbar with aria-valuenow; failure role=alert.

## 7. Responsive
- Owners stay right.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| model | Readiness | — | version, items, release, error |
| on_toggle | EventHandler<usize> | — |  |
| on_release | EventHandler<()> | — |  |

## 9. Edge cases
- Non-blocking items never disable Release.

## 10. Implementation
- Core: `Readiness`, `Prereq` (`oxyn_core::parity`), with tests.
- Dioxus: `ReleaseReadiness` (parity/run.rs, run.css).
- GPUI: `ReadinessEl` (`release`).

## 11. References
- Arc UI Release readiness (https://uiarc.dev/components/blocks/release-readiness) — behaviour-only rebuild, no source copied.
