**Status:** built v0.1 (2026-11-16, variants 2: Attachments family).

# Spec: Upload dropzone (`upload-dropzone`)

Family: Attachments. Reference: the behaviour of a public upload-dropzone block (drag a file chip in, progress with percent, Uploaded, refused when too big; fill, hairline and corner options). Behaviour only; look, icons and wording are our own.
Components: `UploadDropzone` (Dioxus) · `UploadDropzoneEl` (GPUI), models in `oxyn_core::parity::attachfid`.

## 1. Purpose
The simplest honest upload: one box, what it takes, and what happened to your file.

## 2. Parts
| Part | Look |
|---|---|
| Box | full width, `corner` radius (20), grey fill, optional hairline; icon tile 44 px, "Drop a file here" 15/600, the limit 13 muted |
| File row | while uploading: file icon 40 px, name 15.5/600, "2.4 MB · 64%" (percent counts with a registered CSS property) and a 5 px blue bar |
| Refusal | the heading turns red: "Too big – the limit is 10 MB" or "Not a type this box takes" (`refusal_line`) |
| Files | sample file icons under the box: a page with a folded corner and a coloured type badge (`FileIcon`, `file_badge`), name under it |

## 3. Behaviour
Drag a file onto the box (it lifts 2 % and rings blue), or press a file, or press the box for the next one. `check_file` decides: the bar runs for `upload_ms` (size over `speed`, 0.7–4.5 s), then Uploaded for about a second, then idle; a refusal shows for about 2 s. Every timed step ends on its CSS animation's end. `on_upload(name)`.
prefers-reduced-motion: no transitions or animation.

## 4. Variants
default · hairline (60 MB) · typed (PDF and images only).

## 5. Keyboard and accessibility
The box is a button (Enter or Space adds the next file); each file is a button whose label names type, size and how to upload. Results are said in a polite status line.

## 6. Responsive
340 px; files wrap under the box.

## 7. Known gaps
Desktop files dropped on the box are not read (sample files only). GPUI: static snapshot (`browse`, `file-<i>`).
