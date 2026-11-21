**Status:** built v0.1 (2026-11-21, C2a: X-post components I). Free.

# Spec: Publish popover (`publish-popover`)

Page `/components/navigation/publish-popover/` (family overlays). Reference: an X post the founders saved (R02 (publish-to-the-web popover in a notes app) in `docs/research/sources.md`), idea only, rebuilt in Oxyn's own design; no source, artwork or copy taken. Every company, person, key and amount is fictional.
Components: `PublishPopover` (Dioxus, `oxyn_dioxus::xpost`, `xpost/moments.rs`) · `XPostView` with `XPostVariant::PublishPopover` (GPUI), rules in `oxyn_core::xpost::publish`.

## 1. Purpose
Publish an agent-written document to the web from where it is edited, with the address, access and host decided in one small popover.

## 2. States
Closed · Form · Publishing (upload, build, live) · Address taken (offers a free one) · Live (link, Copy, views, Unpublish).

## 3. Micro-interactions
The popover slides out of its button (scale from the top right); publishing steps shimmer under a progress bar; the toolbar button turns into a Live dot.

## 4. Timing
Everything on screen is derived from the host clock `now` (no timers in the component), so playback, tests and the GPUI view agree.

| Transition | Duration | Easing |
|---|---|---|
| Slide in | 220 ms | overshoot |
| Upload | 900 ms | — |
| Build | 1100 ms | — |
| Go live | 600 ms | — |

Reduced motion: CSS motion is cut to a frame; decorative bands and rings are hidden or still; state and text changes stay.

## 5. Variants
| Variant | What it shows |
|---|---|
| `form` | The popover open on the form. |
| `taken` | The address is taken; offers a free one. |
| `closed` | Just the toolbar button (Oxyn extra). |

## 6. Responsive and edge cases
The popover is 340 px and narrows to 300 px under 520 px, staying inside the card; the address truncates its base. Host marks (Cloudflare, Vercel, Netlify) are official marks. Checked at 1440, 768 and 390 px in light and dark with no text overlap. Chrome is monochrome grey; colour is semantic only (success, warning, danger, info) plus the official coloured brand marks.

## 7. Accessibility
The button is aria-haspopup dialog with aria-expanded; the address input is labelled and aria-invalid when taken; access and host are roving radio groups; switches report on/off; Escape closes. Every action returns the line the polite live region announces. The focus outline shows only for keyboard focus.

## 8. GPUI
`XPostView::new(XPostVariant::PublishPopover, ..)`; presses by id (pp-toggle, pp-close, pp-go, pp-copy, pp-unpublish, pp-index, pp-history, pp-access-<n>, pp-host-<n>), keys through `XPostState::key`. The view advances with `advance(ms)`.

## 9. Related
share-dialog, tooltip-popover, git-chips.
