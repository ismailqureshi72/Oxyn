**Status:** built v0.1 (2026-11-15, plan #28).

# Spec: File dropzone (`file-dropzone`)

Family: Composer & inputs ([composer-attachments.md](composer-attachments.md)). Closes the parity row(s): affectUI · Dropzone; Arc UI · File dropzone; Arc UI · File upload. Behaviour-only rebuild; nothing is copied.

## 1. Purpose
A generous target for one or more files and the list that follows. Files are checked against `accept` (".pdf, image/*") and `max_size`: a rejected file keeps a row with its reason and never uploads; past `max_files` extra files are refused with a note. Each accepted file uploads with progress and time left ("1.2 MB of 4.0 MB · 3 s left"), fails with a reason and Retry, or settles with a check; Remove collapses the row (and aborts). `variant: fan` (affectUI) fans three documents over the target while files hover and gives rows type badges; `plain` (Arc) lifts the icon. `list_placement: inside` puts rows inside the target's edge; `compact_at` folds the prompt to a slim bar once the list holds that many. `preset` rows render without an entrance; `autoplay` plays a sample drop of four files (one fails). `multiple: false` replaces the file.

## 2. States
| State | Looks | Leaves on |
|---|---|---|
| idle | Prompt, Browse | drop / pick |
| hover | Solid edge, lift or fan, drop label | drop / leave |
| uploading | Row progress, time left | done / fail |
| failed | Red row, reason, Retry | retry / remove |
| compact | Slim bar | — |

## 3. Timing and motion
- Rows enter 320 ms; documents fan 380 ms spring; progress eases 250 ms.
- Reduced motion replaces height and lift animations with fades and keeps progress instant.

## 4. Variants
| Variant | What differs |
|---|---|
| autoplay | Sample drop |
| fan | affectUI look |
| folding | Inside list, folds at two |
| single | One file |

## 5. Keyboard
| Key | Does |
|---|---|
| Tab | Picker, then each row's Retry and Remove |
| Enter / Space | Open the picker |
| ↑ / ↓ | Same action on the next row |
| Delete / Backspace | Remove the focused row |

## 6. Accessibility
- The picker input carries the label, description and limits; failed rows are role=alert; bars are role=progressbar.

## 7. Responsive
- Fills its container; names truncate.

## 8. Props (Dioxus)
| Prop | Type | Default | Notes |
|---|---|---|---|
| label / description / note / drop_label | String | Add files / … / limits / Drop to upload |  |
| accept | String | — |  |
| max_size / max_files | Option | — |  |
| multiple | bool | true |  |
| variant | plain · fan | plain |  |
| list_placement | below · inside | below |  |
| compact_at | Option<usize> | — |  |
| preset | Vec<(String, u64)> | — |  |
| autoplay / disabled | bool | false |  |
| now | ClockMs | — | progress |
| on_change | EventHandler<Vec<String>> | — |  |

## 9. Gaps against the source
- Uploads are simulated on the clock; a real host maps its upload progress onto rows. ⌘V paste of files and image thumbnails (object URLs) need JS and are not offered.

## 10. GPUI
FileDropzoneEl renders the target and rows from `kit::upload::Uploads`.
