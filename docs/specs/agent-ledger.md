**Status:** built v0.1 (2026-11-16, plan #32 affectUI fidelity pass, overnight batch, family B part 4).

# Spec: Change ledger (`agent-ledger`)

Family: Agents. Closes the parity row: affectUI · Agent Ledger (Pro). Behaviour-only rebuild; nothing is copied (the
demo's changes, systems' details and agent name are our own).
Components: `AgentLedger` (Dioxus) · `AgentLedgerEl` (GPUI), model in `oxyn_core::kit::ledgerfid`.

## 1. Purpose
Everything an agent changed in your systems, newest first, each change with an undo. Pointing at Undo is honest about
consequences: what would be undone with it, and what can't be taken back.

## 2. Parts
| Part | Look (Clean) |
|---|---|
| Root | 640 px (max 100 %), container-sized |
| Header | 28 px, 12 px above the list: blue ledger glyph, title 13.5/600, the agent muted; right "8 changes" (number 600 ink, word muted) and "3 undone" once something is undone |
| List | ordered list in a white card: hairline border, 10 px radius, 4 × 6 px padding |
| Row | 56 px, 8 px radius: 32 px logo tile (hairline, 8 px radius, coloured maker mark), line 1 = action 13/500 ink + target in a darker grey, line 2 = before (struck, faint) → after, or one muted detail line; a right slot "3m ago · Stripe" 12 muted; hairline dividers inset to the text |
| Undo / Redo | 24 px pills, 7 px radius, glyph + word, in the time's slot on hover or focus |
| Flags | "Also undone" blue pill (blue-washed row), "🔒 Can't undo" amber pill (amber-washed row) |
| Undone | text struck through and dimmed, logo grey, "Undone" in the slot |

## 3. Behaviour
- Demo: a 22 s pass that starts over: eight changes land at the top one per second (the clock runs 20× so times read in
  minutes), then the price change's Undo is pointed at – the coupon and the deals light up as Also undone, the sent email
  as Can't undo – and taken; the chain is struck through and the header counts 3 undone.
- Hover or focus Undo lights up the change and everything built on it (transitively); a click undoes the change and its
  reversible dependents (`on_undo(ids)`); Redo on an undone row (`on_redo(ids)`).
- Rows never change height; time, Undo, Redo and the flags share one slot.
- prefers-reduced-motion: rows and pills appear at once.

## 4. Variants
| Variant | What differs |
|---|---|
| default | Live pass |
| pointed | Settled, the price change's Undo pointed at |
| undone | After the undo |
| narrow | 380 px: system names drop, Undo / Redo round icons |
| classic | The header in its own grey band, the list a raised card, round tiles |

## 5. Keyboard and accessibility
Tab moves through Undo and Redo; focusing Undo shows what goes with it; Enter / Space undoes or redoes. The ledger is an
ordered list labelled with its title, newest first; Undo is named "Undo: Changed price Team plan – and 2 that build on
it"; the arrow is read as "to"; how many are undone is announced.

## 6. Responsive
Under 480 px the system name drops, the slot narrows and Undo / Redo become round icon buttons; on touch screens Undo
and Redo show in place of the time.

## 7. Known gaps
- Undo does not draw its strike line row by row down the chain (the rows dim together).
- GPUI: a static snapshot; Undo shows only on the pointed row (no hover), the lock is an emoji glyph.
