**Status:** built v0.1 (2026-11-16, variants 2: Attachments family).

# Spec: Attachment chips (`attachment-chips`)

Family: Attachments. Reference: common composer attachment trays (chips with type icons, progress, errors). Our own layout and wording.
Components: `AttachmentChips` (Dioxus) · `AttachmentChipsEl` (GPUI), models in `oxyn_core::kit::attachfid`.

## 1. Purpose
Show each file's progress and problems where the user is about to send, without a separate upload panel.

## 2. Parts
| Part | Look |
|---|---|
| Tray | chips wrap above the field; past `max_visible` the rest fold into a dashed +N chip (Fewer folds back) |
| Chip | 46 px, inset grey; file icon with a 2.5 px blue progress ring while uploading; name 13/500, meta 11.5 ("Uploading · 42%", "PDF · 2.4 MB") |
| Failed | red-tinted chip, "Upload failed" and Retry (the ring stops at 63 %) |
| Too big | red-tinted chip, "Too big · 48.2 MB", remove only |
| Field | "Ask about these files…", Add file, a count ("6 files · 2 uploading") and a send dot that turns ink when nothing is uploading |

## 3. Behaviour
Chips start uploading at once (`upload_ms`). A flaky file fails once; Retry restarts its ring. × removes a chip. Add file adds the next sample. `on_remove(name)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default · compact (3 before +N, slow) · roomy (60 MB, 8 visible).

## 5. Keyboard and accessibility
The tray is a list of chips labelled with name, type and size; Retry and × name their file. Results are said in a status line; errors are written, not only red.

## 6. Responsive
520 px composer; names ellipsize at 150 px.

## 7. Known gaps
No real network or pause. GPUI: static snapshot (`file-<i>`).
