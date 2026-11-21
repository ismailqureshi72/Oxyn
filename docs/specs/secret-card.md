**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Secret card (`secret-card`)

Page `/components/agents/secret-card/` (family approval). Reference: an X post the founders saved (R49 (routines with secrets kept out of the chat) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `SecretCard` (Dioxus, `oxyn_dioxus::xpost`, `xpost/trust.rs`) · `XPostView` with `XPostVariant::SecretCard` (GPUI), rules in `oxyn_core::xpost::secret`.

## 1. Purpose
Let an agent use an API key without the key ever entering the conversation: the chat gets a handle, the person keeps control of showing, rotating and revoking it.

## 2. States
Active (masked) · Confirm (show, rotate or revoke) · Shown (5 s, then masked again) · Rotating (progress) · Revoked (routines paused, Undo for 5 s, then Restore).

## 3. Micro-interactions
The masked value keeps its last four characters; a shown key gets an amber outline and a countdown in the status line; rotation swaps the tail when the bar fills; revoking turns the routine dots amber with Paused.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Reveal | 5000 ms (REVEAL_MS) | — |
| Rotate | 1600 ms (ROTATE_MS) | linear |
| Undo window | 5000 ms (UNDO_MS) | — |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `active` | An active key with its scopes and routines. |
| `compact` | Key row and actions only (Oxyn extra). |
| `revoked` | Revoked, routines paused, with Restore. |

## 6. Responsive and edge cases
The value truncates inside its own box; actions wrap under the status line; the Stripe mark sits on a white tile in both themes. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
The masked value is labelled as hidden except its last four characters; confirms are alert dialogs that say the consequence; Enter confirms, Escape cancels or hides. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::SecretCard, ..)`; presses by id (sc-reveal, sc-rotate, sc-revoke, sc-confirm, sc-cancel, sc-undo, sc-restore, sc-copy), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
pii-redaction, provider-keys, permission-request.
