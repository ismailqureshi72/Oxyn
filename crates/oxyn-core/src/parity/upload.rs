//! Uploads (plan #28, chunk 4b): the behaviour behind the dropzone, the file dropzone, the
//! file-upload block and the avatar upload.
//!
//! - [`accepts`] / [`check`]: accept lists (".pdf", "image/*") and size limits, with the reason a
//!   rejected file keeps on its row.
//! - [`Upload`] / [`Uploads`]: rows that queue, upload with progress and time left, fail with a
//!   reason and retry, collapse when removed, and fold the prompt to a slim bar past `compact_at`.
//! - [`Crop`]: the avatar cropper: drag with rubber-band past the edges, zoom toward a point, arrow
//!   keys, reset, and the square the upload gets.
//!
//! Behaviour-only rebuilds of the affectUI Dropzone and the Arc UI File dropzone, File upload
//! and Avatar upload; nothing is copied. Time is passed in as [`ClockMs`].

use crate::ClockMs;

/// Upper-cased extension without the dot ("PDF"), or "FILE".
pub fn ext(name: &str) -> String {
    name.rsplit_once('.')
        .map(|(_, e)| e.to_uppercase())
        .filter(|e| !e.is_empty() && e.len() <= 5)
        .unwrap_or_else(|| "FILE".into())
}

/// Broad kind from the extension, for the type badge.
pub fn kind(name: &str) -> &'static str {
    match ext(name).as_str() {
        "PNG" | "JPG" | "JPEG" | "GIF" | "WEBP" | "SVG" | "HEIC" => "image",
        "PDF" | "DOC" | "DOCX" | "TXT" | "MD" | "RTF" => "doc",
        "CSV" | "XLS" | "XLSX" | "TSV" => "sheet",
        "ZIP" | "TAR" | "GZ" | "RAR" | "7Z" => "archive",
        "MP3" | "WAV" | "M4A" | "FLAC" => "audio",
        "MP4" | "MOV" | "WEBM" => "video",
        "RS" | "TS" | "JS" | "PY" | "JSON" | "TOML" | "GO" => "code",
        _ => "file",
    }
}

/// Rough MIME type from the extension.
pub fn mime(name: &str) -> String {
    let k = kind(name);
    let e = ext(name).to_lowercase();
    match k {
        "image" => format!("image/{}", if e == "jpg" { "jpeg" } else { &e }),
        "audio" => format!("audio/{e}"),
        "video" => format!("video/{e}"),
        _ if e == "pdf" => "application/pdf".into(),
        _ => format!("application/{e}"),
    }
}

/// `2.4 MB`, `820 KB`, `12 B`.
pub fn size(bytes: u64) -> String {
    const K: f64 = 1024.0;
    let b = bytes as f64;
    if b >= K * K * K {
        format!("{:.1} GB", b / K / K / K)
    } else if b >= K * K {
        let v = b / K / K;
        if v >= 10.0 {
            format!("{v:.0} MB")
        } else {
            format!("{v:.1} MB")
        }
    } else if b >= K {
        format!("{:.0} KB", b / K)
    } else {
        format!("{bytes} B")
    }
}

/// Whether `name` matches a native-style accept list (".pdf, image/*, application/zip"). An empty
/// list accepts anything.
pub fn accepts(name: &str, accept: &str) -> bool {
    let list: Vec<String> = accept
        .split(',')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();
    if list.is_empty() {
        return true;
    }
    let e = format!(".{}", ext(name).to_lowercase());
    let m = mime(name);
    list.iter().any(|a| {
        if a.starts_with('.') {
            *a == e
        } else if let Some(p) = a.strip_suffix("/*") {
            m.starts_with(&format!("{p}/"))
        } else {
            *a == m
        }
    })
}

/// The accept list as words for the hint: "PDF, PNG or images".
pub fn accept_words(accept: &str) -> String {
    let w: Vec<String> = accept
        .split(',')
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|a| {
            if let Some(e) = a.strip_prefix('.') {
                e.to_uppercase()
            } else if let Some(p) = a.strip_suffix("/*") {
                format!("{p}s")
            } else {
                a.to_string()
            }
        })
        .collect();
    match w.len() {
        0 => "Any file".into(),
        1 => w[0].clone(),
        n => format!("{} or {}", w[..n - 1].join(", "), w[n - 1]),
    }
}

/// Checks a file: `Err(reason)` the row keeps.
pub fn check(name: &str, bytes: u64, accept: &str, max: Option<u64>) -> Result<(), String> {
    if !accepts(name, accept) {
        return Err(format!("{} files aren't accepted", ext(name)));
    }
    if let Some(m) = max
        && bytes > m
    {
        return Err(format!("Too large · {} of {}", size(bytes), size(m)));
    }
    Ok(())
}

/// Row status.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// Waiting for a slot.
    Queued,
    /// Uploading.
    Uploading,
    /// Uploaded.
    Done,
    /// Failed or rejected.
    Failed,
}

/// One file row.
#[derive(Clone, Debug, PartialEq)]
pub struct Upload {
    /// Id.
    pub id: u64,
    /// Name.
    pub name: String,
    /// Bytes.
    pub bytes: u64,
    /// Status.
    pub status: Status,
    /// When the current attempt started.
    pub started: ClockMs,
    /// Simulated duration of a full upload (ms).
    pub dur_ms: u64,
    /// Fails at this progress on the first attempt (simulation), if set.
    pub fail_at: Option<f32>,
    /// Reason, when failed.
    pub error: Option<String>,
    /// Rejected by a check (no retry).
    pub rejected: bool,
    /// Attempts made.
    pub attempts: u32,
    /// Already there at mount (no entrance).
    pub preset: bool,
}

/// Rows plus the rules they were checked against.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct Uploads {
    /// Rows, newest last.
    pub rows: Vec<Upload>,
    /// Accept list.
    pub accept: String,
    /// Size limit.
    pub max_size: Option<u64>,
    /// Most files kept.
    pub max_files: Option<usize>,
    /// One file only: a new file replaces the old one.
    pub single: bool,
    next: u64,
}

impl Uploads {
    /// An empty list with rules.
    pub fn new(accept: &str, max_size: Option<u64>, max_files: Option<usize>) -> Self {
        Self {
            accept: accept.into(),
            max_size,
            max_files,
            ..Self::default()
        }
    }

    /// Adds files at `now`. Each passes the checks or keeps a failed row with its reason. Past
    /// `max_files` the extra files are refused (the count comes back).
    pub fn add(&mut self, files: &[(&str, u64)], now: ClockMs) -> usize {
        let mut refused = 0;
        for (name, bytes) in files {
            if self.single {
                self.rows.clear();
            }
            if self
                .max_files
                .is_some_and(|m| self.rows.iter().filter(|r| !r.rejected).count() >= m)
            {
                refused += 1;
                continue;
            }
            self.next += 1;
            let check = check(name, *bytes, &self.accept, self.max_size);
            let dur = 1200 + (*bytes / 4096).min(4800);
            self.rows.push(Upload {
                id: self.next,
                name: (*name).into(),
                bytes: *bytes,
                status: if check.is_ok() {
                    Status::Uploading
                } else {
                    Status::Failed
                },
                started: now,
                dur_ms: dur,
                fail_at: None,
                error: check.err(),
                rejected: false,
                attempts: 1,
                preset: false,
            });
            if let Some(r) = self.rows.last_mut() {
                r.rejected = r.error.is_some();
            }
        }
        refused
    }

    /// Marks the newest row to fail at `at` progress on its first attempt (simulation).
    pub fn fail_last(&mut self, at: f32, reason: &str) {
        if let Some(r) = self.rows.last_mut() {
            r.fail_at = Some(at);
            r.error = Some(reason.into());
        }
    }

    /// Settles rows on the clock: progress runs out, simulated failures land.
    pub fn tick(&mut self, now: ClockMs) {
        for r in &mut self.rows {
            if r.status != Status::Uploading {
                continue;
            }
            let p = progress(r, now);
            if let Some(f) = r.fail_at
                && r.attempts == 1
                && p >= f
            {
                r.status = Status::Failed;
            } else if p >= 1.0 {
                r.status = Status::Done;
                r.error = None;
            }
        }
    }

    /// Retry a failed (not rejected) row.
    pub fn retry(&mut self, id: u64, now: ClockMs) -> bool {
        let Some(r) = self.rows.iter_mut().find(|r| r.id == id) else {
            return false;
        };
        if r.status != Status::Failed || r.rejected {
            return false;
        }
        r.status = Status::Uploading;
        r.started = now;
        r.attempts += 1;
        r.error = None;
        true
    }

    /// Remove a row (aborts its upload).
    pub fn remove(&mut self, id: u64) {
        self.rows.retain(|r| r.id != id);
    }

    /// Prompt folds to a slim bar once the list holds `at` files.
    pub fn compact(&self, at: Option<usize>) -> bool {
        at.is_some_and(|a| self.rows.len() >= a)
    }

    /// Default hint: "PDF or PNG · up to 25 MB · 5 files".
    pub fn hint(&self) -> String {
        let mut parts = vec![accept_words(&self.accept)];
        if let Some(m) = self.max_size {
            parts.push(format!("up to {}", size(m)));
        }
        if let Some(n) = self.max_files {
            parts.push(format!("{n} files max"));
        }
        parts.join(" · ")
    }
}

/// Eased progress of a row at `now` (0..1). Done rows are 1; rejected rows 0.
pub fn progress(r: &Upload, now: ClockMs) -> f32 {
    match r.status {
        Status::Done => 1.0,
        Status::Queued => 0.0,
        Status::Failed if r.rejected => 0.0,
        _ => {
            let t = now.since(r.started).as_millis() as f32 / r.dur_ms.max(1) as f32;
            let p = 1.0 - (1.0 - t.clamp(0.0, 1.0)).powf(1.6);
            match (r.status, r.fail_at) {
                (Status::Failed, Some(f)) => f,
                _ => p,
            }
        }
    }
}

/// "12 s left" from progress and elapsed time; `None` early on (too noisy).
pub fn time_left(p: f32, elapsed_ms: u64) -> Option<String> {
    if !(0.06..1.0).contains(&p) || elapsed_ms < 300 {
        return None;
    }
    let total = elapsed_ms as f32 / p;
    let left = ((total - elapsed_ms as f32) / 1000.0).ceil().max(1.0) as u64;
    Some(if left >= 60 {
        format!("{} min left", left.div_ceil(60))
    } else {
        format!("{left} s left")
    })
}

/// The avatar cropper (Arc Avatar upload behaviour). Offsets are fractions of the crop circle;
/// zoom 1 fits the image's short side to the circle.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Crop {
    /// Horizontal offset of the image centre.
    pub x: f32,
    /// Vertical offset of the image centre.
    pub y: f32,
    /// Zoom, 1..=4.
    pub zoom: f32,
    /// Image width / height.
    pub aspect: f32,
}

impl Crop {
    /// Centred, unzoomed.
    pub fn new(aspect: f32) -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
            aspect: aspect.max(0.05),
        }
    }

    /// How far the image may move each way before an edge shows inside the circle.
    pub fn bounds(&self) -> (f32, f32) {
        let (w, h) = if self.aspect >= 1.0 {
            (self.aspect, 1.0)
        } else {
            (1.0, 1.0 / self.aspect)
        };
        (
            ((w * self.zoom - 1.0) / 2.0).max(0.0),
            ((h * self.zoom - 1.0) / 2.0).max(0.0),
        )
    }

    /// Where a drag to `(x, y)` shows: inside the bounds one to one, past them at a falling rate
    /// and never more than a quarter of the circle.
    pub fn shown(&self, x: f32, y: f32) -> (f32, f32) {
        let (bx, by) = self.bounds();
        let r = |v: f32, b: f32| {
            if v.abs() <= b {
                v
            } else {
                let over = v.abs() - b;
                v.signum() * (b + 0.25 * (1.0 - 1.0 / (over * 4.0 + 1.0)))
            }
        };
        (r(x, bx), r(y, by))
    }

    /// Release: springs back inside the bounds.
    pub fn clamp(&mut self) {
        let (bx, by) = self.bounds();
        self.x = self.x.clamp(-bx, bx);
        self.y = self.y.clamp(-by, by);
    }

    /// Arrow keys: 2 % of the circle, 10 % with Shift.
    pub fn nudge(&mut self, dx: i32, dy: i32, shift: bool) {
        let s = if shift { 0.10 } else { 0.02 };
        self.x += dx as f32 * s;
        self.y += dy as f32 * s;
        self.clamp();
    }

    /// Zoom by `factor` toward a point `(px, py)` of the circle (centre 0,0, edge ±0.5): that
    /// point stays still.
    pub fn zoom_at(&mut self, factor: f32, px: f32, py: f32) {
        let z0 = self.zoom;
        let z1 = (z0 * factor).clamp(1.0, 4.0);
        let k = z1 / z0;
        self.x = px - (px - self.x) * k;
        self.y = py - (py - self.y) * k;
        self.zoom = z1;
        self.clamp();
    }

    /// 0: reset zoom and position.
    pub fn reset(&mut self) {
        self.x = 0.0;
        self.y = 0.0;
        self.zoom = 1.0;
    }

    /// The square to export: its centre as fractions of the image width and height, and its
    /// side as a fraction of the image's short side.
    pub fn source(&self) -> (f32, f32, f32) {
        let (w, h) = if self.aspect >= 1.0 {
            (self.aspect, 1.0)
        } else {
            (1.0, 1.0 / self.aspect)
        };
        (
            0.5 - self.x / (w * self.zoom),
            0.5 - self.y / (h * self.zoom),
            1.0 / self.zoom,
        )
    }
}

/// A `data:` URL for bytes (base64), so a picked photo can show without an object URL.
pub fn data_url(mime: &str, bytes: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(bytes.len() * 4 / 3 + 32);
    out.push_str("data:");
    out.push_str(mime);
    out.push_str(";base64,");
    for ch in bytes.chunks(3) {
        let b = [ch[0], *ch.get(1).unwrap_or(&0), *ch.get(2).unwrap_or(&0)];
        let n = (u32::from(b[0]) << 16) | (u32::from(b[1]) << 8) | u32::from(b[2]);
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if ch.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if ch.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// Width and height from a PNG, GIF, JPEG or WebP header, without decoding the image.
pub fn image_size(b: &[u8]) -> Option<(u32, u32)> {
    let be16 =
        |i: usize| -> Option<u32> { Some(u32::from(*b.get(i)?) << 8 | u32::from(*b.get(i + 1)?)) };
    let le16 =
        |i: usize| -> Option<u32> { Some(u32::from(*b.get(i)?) | u32::from(*b.get(i + 1)?) << 8) };
    if b.starts_with(b"\x89PNG") {
        let w = be16(16)? << 16 | be16(18)?;
        let h = be16(20)? << 16 | be16(22)?;
        return Some((w, h));
    }
    if b.starts_with(b"GIF8") {
        return Some((le16(6)?, le16(8)?));
    }
    if b.len() > 30 && &b[0..4] == b"RIFF" && &b[8..12] == b"WEBP" {
        return match &b[12..16] {
            b"VP8X" => Some((
                (le16(24)? | u32::from(b[26]) << 16) + 1,
                (le16(27)? | u32::from(b[29]) << 16) + 1,
            )),
            b"VP8 " => Some((le16(26)? & 0x3fff, le16(28)? & 0x3fff)),
            b"VP8L" => {
                let v = u32::from(b[21])
                    | u32::from(b[22]) << 8
                    | u32::from(b[23]) << 16
                    | u32::from(b[24]) << 24;
                Some(((v & 0x3fff) + 1, ((v >> 14) & 0x3fff) + 1))
            }
            _ => None,
        };
    }
    if b.starts_with(&[0xff, 0xd8]) {
        let mut i = 2;
        while i + 9 < b.len() {
            if b[i] != 0xff {
                i += 1;
                continue;
            }
            let m = b[i + 1];
            let len = be16(i + 2)? as usize;
            if (0xc0..=0xcf).contains(&m) && m != 0xc4 && m != 0xc8 && m != 0xcc {
                return Some((be16(i + 7)?, be16(i + 5)?));
            }
            i += 2 + len;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accept_and_size_checks() {
        assert!(accepts("a.PDF", ".pdf,image/*"));
        assert!(accepts("shot.png", ".pdf, image/*"));
        assert!(!accepts("run.exe", ".pdf,image/*"));
        assert!(accepts("x.bin", ""));
        assert_eq!(
            check("big.pdf", 31 * 1024 * 1024, ".pdf", Some(25 * 1024 * 1024)),
            Err("Too large · 31 MB of 25 MB".into())
        );
        assert_eq!(accept_words(".pdf,.png,image/*"), "PDF, PNG or images");
        assert_eq!(size(2_516_582), "2.4 MB");
    }

    #[test]
    fn uploads_progress_fail_retry() {
        let mut u = Uploads::new(".pdf,.png", Some(10_000_000), Some(3));
        let refused = u.add(
            &[("a.pdf", 400_000), ("b.exe", 10), ("c.png", 900_000)],
            ClockMs(0),
        );
        assert_eq!(refused, 0);
        assert!(u.rows[1].rejected);
        u.fail_last(0.5, "Network lost");
        u.tick(ClockMs(100_000));
        assert_eq!(u.rows[0].status, Status::Done);
        assert_eq!(u.rows[2].status, Status::Failed);
        assert!(!u.retry(u.rows[1].id, ClockMs(1)));
        assert!(u.retry(u.rows[2].id, ClockMs(100_000)));
        u.tick(ClockMs(200_000));
        assert_eq!(u.rows[2].status, Status::Done);
        assert_eq!(u.add(&[("d.pdf", 1), ("e.pdf", 1)], ClockMs(0)), 1);
        assert!(u.compact(Some(3)));
        assert_eq!(time_left(0.5, 4000).as_deref(), Some("4 s left"));
        assert_eq!(time_left(0.01, 4000), None);
    }

    #[test]
    fn data_urls_and_headers() {
        assert_eq!(
            data_url("text/plain", b"hi!"),
            "data:text/plain;base64,aGkh"
        );
        assert_eq!(data_url("text/plain", b"h"), "data:text/plain;base64,aA==");
        let mut png = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
        png.extend([0, 0, 1, 0, 0, 0, 0, 200]);
        assert_eq!(image_size(&png), Some((256, 200)));
        assert_eq!(image_size(b"GIF89a\x10\x00\x20\x00"), Some((16, 32)));
    }

    #[test]
    fn crop_bounds_rubber_zoom() {
        let mut c = Crop::new(1.5);
        assert_eq!(c.bounds(), (0.25, 0.0));
        let (sx, sy) = c.shown(1.0, 0.0);
        assert!(sx > 0.25 && sx < 0.5 && sy == 0.0);
        c.x = 1.0;
        c.clamp();
        assert_eq!(c.x, 0.25);
        c.zoom_at(2.0, 0.0, 0.0);
        assert_eq!(c.zoom, 2.0);
        c.nudge(1, 0, true);
        c.reset();
        assert_eq!((c.x, c.y, c.zoom), (0.0, 0.0, 1.0));
    }
}
