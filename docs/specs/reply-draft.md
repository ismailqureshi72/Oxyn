**Status:** built v0.1 (2026-11-21, C1d: AI special components I). Pro.

# Spec: Reply draft (`reply-draft`)

Family: AI Components (page `/components/ai/reply-draft/`, Pro). Reference: uiarc "Special components · AI", behaviour only, under our own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `ReplyDraft` (Dioxus, `oxyn_dioxus::aikit`) · `AiKitView` with `AiVariant::Reply` (GPUI), rules in `oxyn_core::aikit::reply`.

## 1. Purpose
Answer an email fast: quick replies steer a fresh draft, tone and length rewrite it, the draft can be edited, and Insert hands it over.

## 2. States
Empty (Draft a reply) · Thinking · Streaming · Ready · Editing · Inserted · Earlier drafts (restore).

## 3. Micro-interactions
Quick replies start a new draft; tone (Friendly, Concise, Formal) and length (Short, Medium, Long) rewrite with one crossfade. Cmd/Ctrl+Enter inserts.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Think | 500 ms (THINK_MS) | — |
| Stream | 34 ms per word tick | linear |
| Rewrite crossfade | 180 ms | ease-out |

Reduced motion: streamed text and moves appear at their end state; progress still advances, without travel animation.

## 5. Variants
| Variant | What it shows |
|---|---|
| `drafting` | The first draft streams in. |
| `formal-long` | A long, formal draft with earlier drafts to restore (Oxyn extra). |
| `empty` | No draft yet: Draft a reply. |

## 6. Responsive and edge cases
The received-message header keeps the sender name visible: the name truncates last, the time shrinks first. Long drafts scroll inside the card. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the providers' coloured brand marks.

## 7. Accessibility
Tone and length are radio groups; the draft is a labelled textbox while editing; Insert announces itself. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`AiKitView::new(AiVariant::Reply, ..)`; presses by id (rd-draft, rd-insert, rd-regen, rd-hist-<n>, rd-sug-<n>, rd-tone-<n>, rd-len-<n>), keys through `AiKitState::key`. The view advances with `advance(ms)`.

## 9. Related
composer-input-shell, streaming-markdown.
