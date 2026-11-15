**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Attachment strip (`attachment-strip`)

Family: Composer ([composer-attachments.md](composer-attachments.md)). Closes the parity row(s): X #36 @componest ai composer upload by @sitenley. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Composer attachments as 56 px tiles above the prompt: images show a thumbnail (or a placeholder), everything else a document mark (`file_icon`) with a short name (`tile_name`). Uploads run one after another (`tile_upload`): each tile lands when the one before finishes, a ring draws clockwise round the uploading tile and a 9 px percent counts up in its corner with an ease-out curve; at 100 the percent blurs out on the same spot over 320 ms and the ring goes. Queued files wait as a dashed `+n` tile (or all show at once with `land: false`). Dismiss sits in the corner, shown on hover or focus once a tile is up.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| queued | Counted in the +n tile | previous upload finishes |
| uploading | Ring and percent | 100 |
| fading | Percent blurs out | 320 ms |
| done | Tile only; dismiss on hover | dismiss |

## 3. Timing and motion
- Landing 340 ms overshoot; ring follows the clock; percent blur 0 → 4 px with the fade.
- Reduced motion: no landing or ring transition; percent still counts.

## 4. Variants
| Variant | What differs |
|---|---|
| batch | Five files landing in turn |
| one | One image |
| all-at-once | All tiles shown, queued ones waiting |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Each tile's dismiss (visible on focus) |

## 6. Accessibility
- Tiles are list items labelled with the name and percent; a polite status says when all files are up.

## 7. Responsive
- Tiles wrap.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| files | Vec<String> | — |  |
| weights | Vec<f32> | 1 each | relative size |
| thumbs | Vec<Option<String>> | — | image URLs |
| added / now | ClockMs | — |  |
| dur_ms | u64 | 1400 | weight-1 upload |
| land | bool | true |  |
| on_remove | EventHandler<usize> | — |  |

## 9. Gaps against the source
- Real upload progress is the host's: wire `added` and the weights (or swap in measured progress).

## 10. GPUI
AttachmentStripEl draws the tiles with an outline that brightens with progress and the corner percent.
