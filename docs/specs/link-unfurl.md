**Status:** built v0.1 (2026-11-14, plan #28).

# Spec: Link unfurl (`link-unfurl`)

Family: Composer. Closes the parity row(s): Arc UI · Link unfurl.

## 1. Purpose
A composer where a pasted URL (found with `find_urls`) shimmers on its host while it resolves, then unfurls into a preview card (favicon, site, title, description, image) — or, with auto expand off, gains only its favicon and title and opens on click. A card folds back to the inline link or is removed (the link stays as text). Enter sends (Shift+Enter is a new line). The demo offers sample links as Paste buttons, autoplays the first one, and has a speed multiplier for the simulated fetch.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| loading | Host shimmers | → card / inline / plain |
| card | Preview card | collapse → inline; remove → plain |
| inline | Favicon + title | click → card |
| plain | Address only |  |

## 3. Timing and motion
- The card rises in over 200 ms; the morph from the inline link is a fade + rise, not the source's clip unroll.
- Reduced motion: no entrance rise or row stagger, no shimmer (static muted text), chevrons and the card morph jump to their end state.

## 4. Variants
| Variant | What differs |
|---|---|
| unfurl | Auto |
| inline | Folded |
| no-preview | Plain |
| manual | Auto expand off |

## 5. Keyboard
| Key | Does |
|---|---|
| Enter | Send |
| Shift + Enter | New line |
| Tab | Links, Collapse, Remove, Send |

## 6. Accessibility
- Labelled textarea; card buttons are named; Send is named Send message.

## 7. Responsive
- Cards cap at 420 px.

## 8. Props
| Prop | Type | Default | Notes |
|---|---|---|---|
| text | String | — |  |
| links | Vec<Unfurl> | — | url, state, at |
| now | ClockMs | — |  |
| label | String | Message #lisbon-offsite |  |
| on_input | EventHandler<String> | — |  |
| on_toggle / on_remove | EventHandler<usize> | — |  |
| on_send | EventHandler<()> | — |  |

## 9. Edge cases
- Resolution is the host's (`Unfurl::resolve(preview, auto_expand)`); None leaves the link plain.
- Trailing , . ) are trimmed from pasted URLs.

## 10. Implementation
- Core: `Unfurl`, `UnfurlState`, `Preview`, `find_urls` (`oxyn_core::parity::tools`), with tests.
- Dioxus: `LinkUnfurl` (parity/tools.rs, tools.css).
- GPUI: `LinkUnfurlEl`; typing waits on the GPUI IME input (plan #28 chunk 4).

## 11. References
- Arc UI Link unfurl (https://uiarc.dev/components/link-unfurl) — behaviour-only rebuild, no source copied.
