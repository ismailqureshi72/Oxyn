**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: Avatar upload (`avatar-upload`)

Family: Composer & inputs ([composer-attachments.md](composer-attachments.md)). Closes the parity row(s): Arc UI · Avatar upload. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
Click or drop a photo onto the avatar to crop it in place. The file is checked (type, then size) before it is read; its size comes from the PNG / GIF / JPEG / WebP header (`image_size`) and it shows as a data URL. The avatar grows from `size_px` (88) into a 184 px crop circle on a tinted stage. Drag moves the photo one to one; past an edge it follows at a falling rate, never more than a quarter of the circle (`Crop::shown`), and springs back inside on release. The dimmed overflow brightens while dragging. The wheel zooms toward the pointer (the point under it stays still), the slider and + / − zoom about the centre, arrows move 2 % (Shift 10 %), 0 resets. Save uploads through a progress ring and settles with a check; the host gets the crop square (centre and side as fractions). Errors (type, size, decode, upload) show with Retry for uploads.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| rest | Avatar or initials, Upload / Replace, Remove | pick / drop |
| cropping | 184 px circle, stage, zoom tools | Save / Cancel |
| uploading | Progress ring | done / fail |
| failed | Red ring, Retry | retry |

## 3. Timing and motion
- Grow 450 ms (cubic-bezier .25,1,.4,1); spring back 350 ms; done check pops 300 ms.
- Reduced motion swaps sizes and positions without travel.

## 4. Variants
| Variant | What differs |
|---|---|
| cropping | Start in the cropper |
| photo | At rest |
| failed | First upload fails |
| empty | Initials |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Avatar, Upload, Remove; while cropping the crop area, zoom, Save, Cancel |
| Arrows | Move (Shift ×5) |
| + / − | Zoom |
| 0 | Reset |
| Enter | Save |
| Esc | Cancel |

## 6. Accessibility
- The crop area is a focusable region with its keys in its label; errors are role=alert.

## 7. Responsive
- The crop circle is fixed at 184 px.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| label / hint | String / Option<String> | Profile photo / types and size |  |
| src / initials | Option<String> / String | — / AK |  |
| accept | Vec<String> | png, jpeg, webp |  |
| max_size | u64 | 5 MB |  |
| size_px | f32 | 88 |  |
| fail_first / start_cropping / removable / disabled | bool |  |  |
| now | ClockMs | — | ring |
| on_upload | EventHandler<(f32, f32, f32)> | — | crop square |
| on_remove | EventHandler<()> | — |  |

## 9. Gaps against the source
- Pinch zoom on touch and re-encoding to a 512 px PNG/JPEG need canvas access (JS); the host cuts the square from the reported crop.

## 10. GPUI
AvatarUploadEl renders the avatar, ring and crop circle from `parity::upload::Crop`.
