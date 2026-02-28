use std::fs::{self, File};
use std::io::{Read, Seek, Write};
use std::path::{Path, PathBuf};

use fig2json::FigError;
use serde::Serialize;
use thiserror::Error;
use zip::ZipArchive;

use crate::figma::import_figma_document;

const META_JSON: &str = "meta.json";
const AI_CHAT_JSON: &str = "ai_chat.json";
const CANVAS_FIG: &str = "canvas.fig";
const CANONICAL_COPY_NAME: &str = "canonical_figma.json";
const MANIFEST_NAME: &str = "make_bridge_manifest.json";
const FIGMA_MAGIC: &[u8; 8] = b"fig-kiwi";
const FIG_MAKE_MAGIC: &[u8; 8] = b"fig-make";

#[derive(Debug, Error)]
pub enum MakeBridgeError {
    #[error("failed to read path {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse zip archive {path}: {source}")]
    ZipArchiveOpen {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("zip archive read error: {0}")]
    Zip(#[from] zip::result::ZipError),
    #[error("archive entry path escapes output directory: {0}")]
    UnsafeArchivePath(String),
    #[error("archive entry is not valid UTF-8 text: {0}")]
    NonUtf8Entry(String),
    #[error("canonical figma json not found in archive")]
    CanonicalJsonNotFound,
    #[error("canvas.fig entry not found in archive")]
    CanvasFigNotFound,
    #[error("fig2json failed to decode canvas.fig: {0}")]
    FigDecode(#[from] FigError),
    #[error("failed to serialize json: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MakeArchiveEntryInfo {
    pub name: String,
    pub compressed_size: u64,
    pub uncompressed_size: u64,
    pub is_dir: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MakeArchiveInfo {
    pub path: PathBuf,
    pub entries: Vec<MakeArchiveEntryInfo>,
    pub has_canvas_fig: bool,
    pub has_meta_json: bool,
    pub has_ai_chat_json: bool,
    pub canonical_figma_json_entry: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MakeExtractionOptions {
    pub overwrite: bool,
    pub write_manifest: bool,
    pub write_canonical_copy: bool,
}

impl Default for MakeExtractionOptions {
    fn default() -> Self {
        Self {
            overwrite: true,
            write_manifest: true,
            write_canonical_copy: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MakeExtractionReport {
    pub output_dir: PathBuf,
    pub extracted_files: Vec<PathBuf>,
    pub skipped_existing_files: Vec<PathBuf>,
    pub canvas_fig_path: Option<PathBuf>,
    pub canonical_figma_json_entry: Option<String>,
    pub canonical_figma_json_entry_path: Option<PathBuf>,
    pub canonical_figma_json_copy_path: Option<PathBuf>,
    pub manifest_path: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MakeBridgeManifest {
    pub archive: MakeArchiveInfo,
    pub extraction: MakeExtractionReport,
}

pub fn inspect_make_archive<P: AsRef<Path>>(path: P) -> Result<MakeArchiveInfo, MakeBridgeError> {
    let archive_path = path.as_ref().to_path_buf();
    let mut archive = open_archive(&archive_path)?;
    let mut entries = Vec::with_capacity(archive.len());

    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        entries.push(MakeArchiveEntryInfo {
            name: normalize_entry_name(entry.name()),
            compressed_size: entry.compressed_size(),
            uncompressed_size: entry.size(),
            is_dir: entry.is_dir(),
        });
    }

    let has_canvas_fig = entries.iter().any(|entry| {
        entry
            .name
            .rsplit('/')
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case(CANVAS_FIG))
    });
    let has_meta_json = entries.iter().any(|entry| {
        entry
            .name
            .rsplit('/')
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case(META_JSON))
    });
    let has_ai_chat_json = entries.iter().any(|entry| {
        entry
            .name
            .rsplit('/')
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case(AI_CHAT_JSON))
    });
    let canonical_figma_json_entry = discover_canonical_json_entry(&mut archive)?;

    Ok(MakeArchiveInfo {
        path: archive_path,
        entries,
        has_canvas_fig,
        has_meta_json,
        has_ai_chat_json,
        canonical_figma_json_entry,
    })
}

pub fn read_canonical_figma_json<P: AsRef<Path>>(
    path: P,
) -> Result<Option<String>, MakeBridgeError> {
    let archive_path = path.as_ref().to_path_buf();
    let mut archive = open_archive(&archive_path)?;
    let canonical_entry = discover_canonical_json_entry(&mut archive)?;
    canonical_entry
        .map(|entry_name| read_archive_entry_text(&mut archive, &entry_name))
        .transpose()
}

pub fn decode_canvas_fig_to_import_json(
    canvas_fig_bytes: &[u8],
) -> Result<String, MakeBridgeError> {
    let normalized_canvas_fig = normalize_canvas_fig_magic(canvas_fig_bytes);
    let raw = fig2json::convert_raw(&normalized_canvas_fig)?;
    let document = raw
        .get("document")
        .cloned()
        .ok_or(MakeBridgeError::CanonicalJsonNotFound)?;

    let nodes = document
        .get("children")
        .cloned()
        .filter(|children| children.is_array())
        .unwrap_or_else(|| serde_json::Value::Array(vec![document]));

    let wrapped = serde_json::json!({ "nodes": nodes });
    serde_json::to_string_pretty(&wrapped).map_err(MakeBridgeError::from)
}

pub fn decode_make_archive_to_import_json<P: AsRef<Path>>(
    path: P,
) -> Result<String, MakeBridgeError> {
    let archive_path = path.as_ref().to_path_buf();
    if let Some(canonical_json) = read_canonical_figma_json(&archive_path)? {
        return Ok(canonical_json);
    }

    let mut archive = open_archive(&archive_path)?;
    let canvas_entry_name = find_entry_name_by_basename(&mut archive, CANVAS_FIG)?
        .ok_or(MakeBridgeError::CanvasFigNotFound)?;
    let canvas_bytes = read_archive_entry_bytes(&mut archive, &canvas_entry_name)?;
    decode_canvas_fig_to_import_json(&canvas_bytes)
}

pub fn extract_make_archive<P: AsRef<Path>, Q: AsRef<Path>>(
    archive_path: P,
    output_dir: Q,
    options: MakeExtractionOptions,
) -> Result<MakeExtractionReport, MakeBridgeError> {
    let archive_path = archive_path.as_ref().to_path_buf();
    let output_dir = output_dir.as_ref().to_path_buf();
    let archive_info = inspect_make_archive(&archive_path)?;
    let mut archive = open_archive(&archive_path)?;
    create_dir_all_path(&output_dir)?;

    let mut extracted_files = Vec::new();
    let mut skipped_existing_files = Vec::new();
    let mut canvas_fig_path = None;
    let mut canonical_entry_path = None;

    for index in 0..archive.len() {
        let mut entry = archive.by_index(index)?;
        let raw_name = normalize_entry_name(entry.name());
        let enclosed = entry
            .enclosed_name()
            .map(Path::to_path_buf)
            .ok_or_else(|| MakeBridgeError::UnsafeArchivePath(raw_name.clone()))?;
        let destination = output_dir.join(enclosed);

        if entry.is_dir() {
            create_dir_all_path(&destination)?;
            continue;
        }

        if destination.exists() && !options.overwrite {
            skipped_existing_files.push(destination.clone());
            continue;
        }

        if let Some(parent) = destination.parent() {
            create_dir_all_path(parent)?;
        }
        let mut output = create_file_path(&destination)?;
        std::io::copy(&mut entry, &mut output).map_err(|source| MakeBridgeError::Io {
            path: destination.clone(),
            source,
        })?;
        output.flush().map_err(|source| MakeBridgeError::Io {
            path: destination.clone(),
            source,
        })?;

        if raw_name
            .rsplit('/')
            .next()
            .is_some_and(|name| name.eq_ignore_ascii_case(CANVAS_FIG))
        {
            canvas_fig_path = Some(destination.clone());
        }
        if archive_info
            .canonical_figma_json_entry
            .as_deref()
            .is_some_and(|name| name == raw_name)
        {
            canonical_entry_path = Some(destination.clone());
        }

        extracted_files.push(destination);
    }

    let mut canonical_copy_path = None;
    if options.write_canonical_copy {
        if let Some(canonical_json) = read_canonical_figma_json(&archive_path)? {
            let copy_path = output_dir.join(CANONICAL_COPY_NAME);
            if copy_path.exists() && !options.overwrite {
                skipped_existing_files.push(copy_path.clone());
            } else {
                fs::write(&copy_path, canonical_json).map_err(|source| MakeBridgeError::Io {
                    path: copy_path.clone(),
                    source,
                })?;
                extracted_files.push(copy_path.clone());
            }
            canonical_copy_path = Some(copy_path);
        }
    }

    let mut report = MakeExtractionReport {
        output_dir: output_dir.clone(),
        extracted_files,
        skipped_existing_files,
        canvas_fig_path,
        canonical_figma_json_entry: archive_info.canonical_figma_json_entry.clone(),
        canonical_figma_json_entry_path: canonical_entry_path,
        canonical_figma_json_copy_path: canonical_copy_path,
        manifest_path: None,
    };

    if options.write_manifest {
        let manifest = MakeBridgeManifest {
            archive: archive_info,
            extraction: report.clone(),
        };
        let manifest_path = output_dir.join(MANIFEST_NAME);
        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        fs::write(&manifest_path, manifest_json).map_err(|source| MakeBridgeError::Io {
            path: manifest_path.clone(),
            source,
        })?;
        report.extracted_files.push(manifest_path.clone());
        report.manifest_path = Some(manifest_path);
    }

    Ok(report)
}

fn open_archive(path: &Path) -> Result<ZipArchive<File>, MakeBridgeError> {
    let file = File::open(path).map_err(|source| MakeBridgeError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    ZipArchive::new(file).map_err(|source| MakeBridgeError::ZipArchiveOpen {
        path: path.to_path_buf(),
        source,
    })
}

fn normalize_entry_name(name: &str) -> String {
    name.replace('\\', "/")
}

fn create_dir_all_path(path: &Path) -> Result<(), MakeBridgeError> {
    fs::create_dir_all(path).map_err(|source| MakeBridgeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn create_file_path(path: &Path) -> Result<File, MakeBridgeError> {
    File::create(path).map_err(|source| MakeBridgeError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn find_entry_name_by_basename<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    basename: &str,
) -> Result<Option<String>, MakeBridgeError> {
    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        let name = normalize_entry_name(entry.name());
        if name
            .rsplit('/')
            .next()
            .is_some_and(|part| part.eq_ignore_ascii_case(basename))
        {
            return Ok(Some(name));
        }
    }
    Ok(None)
}

fn discover_canonical_json_entry<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
) -> Result<Option<String>, MakeBridgeError> {
    let mut candidates: Vec<(u8, String, String)> = Vec::new();

    for index in 0..archive.len() {
        let entry = archive.by_index(index)?;
        if entry.is_dir() {
            continue;
        }

        let name = normalize_entry_name(entry.name());
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".json") {
            continue;
        }
        if is_ignored_json_entry(&lower) {
            continue;
        }

        let basename = lower.rsplit('/').next().unwrap_or(lower.as_str());
        let priority = canonical_json_priority(basename);
        candidates.push((priority, lower, name));
    }

    candidates.sort_by(|left, right| left.0.cmp(&right.0).then(left.1.cmp(&right.1)));
    for (_, _, entry_name) in candidates {
        let text = match read_archive_entry_text(archive, &entry_name) {
            Ok(text) => text,
            Err(MakeBridgeError::NonUtf8Entry(_)) => continue,
            Err(err) => return Err(err),
        };
        if import_figma_document(&text).is_ok() {
            return Ok(Some(entry_name));
        }
    }

    Ok(None)
}

fn is_ignored_json_entry(path_lower: &str) -> bool {
    path_lower.ends_with("/meta.json")
        || path_lower == META_JSON
        || path_lower.ends_with("/ai_chat.json")
        || path_lower == AI_CHAT_JSON
}

fn canonical_json_priority(basename: &str) -> u8 {
    match basename {
        "figma.json" => 0,
        "document.json" => 1,
        "nodes.json" => 2,
        "export.json" => 3,
        "scene.json" => 4,
        "canvas.json" => 5,
        _ => 100,
    }
}

fn read_archive_entry_bytes<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    entry_name: &str,
) -> Result<Vec<u8>, MakeBridgeError> {
    let mut entry = archive.by_name(entry_name)?;
    let mut bytes = Vec::with_capacity(entry.size() as usize);
    entry
        .read_to_end(&mut bytes)
        .map_err(|source| MakeBridgeError::Io {
            path: PathBuf::from(entry_name),
            source,
        })?;
    Ok(bytes)
}

fn read_archive_entry_text<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    entry_name: &str,
) -> Result<String, MakeBridgeError> {
    let bytes = read_archive_entry_bytes(archive, entry_name)?;
    String::from_utf8(bytes).map_err(|_| MakeBridgeError::NonUtf8Entry(entry_name.to_string()))
}

fn normalize_canvas_fig_magic(canvas_fig_bytes: &[u8]) -> Vec<u8> {
    let mut normalized = canvas_fig_bytes.to_vec();
    if normalized.starts_with(FIG_MAKE_MAGIC) {
        normalized[0..FIGMA_MAGIC.len()].copy_from_slice(FIGMA_MAGIC);
    }
    normalized
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;
    use std::time::{SystemTime, UNIX_EPOCH};
    use zip::CompressionMethod;
    use zip::write::FileOptions;

    fn canonical_fixture_json() -> &'static str {
        r#"{
            "nodes": [
                {
                    "id": "1",
                    "type": "FRAME",
                    "x": 0,
                    "y": 0,
                    "width": 100,
                    "height": 100
                }
            ]
        }"#
    }

    fn build_zip(entries: &[(&str, &[u8])]) -> Vec<u8> {
        let cursor = Cursor::new(Vec::new());
        let mut writer = zip::ZipWriter::new(cursor);
        let options = FileOptions::default().compression_method(CompressionMethod::Stored);

        for (name, contents) in entries {
            if name.ends_with('/') {
                writer
                    .add_directory(*name, options)
                    .expect("directory entry should be created");
                continue;
            }
            writer
                .start_file(*name, options)
                .expect("file entry should be created");
            writer
                .write_all(contents)
                .expect("entry bytes should be written");
        }

        writer.finish().expect("zip should finish").into_inner()
    }

    fn unique_path(suffix: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!(
            "arthropod_make_bridge_{}_{}_{}",
            std::process::id(),
            now,
            suffix
        ))
    }

    fn write_archive(bytes: &[u8]) -> PathBuf {
        let path = unique_path("archive.make");
        fs::write(&path, bytes).expect("archive should be written");
        path
    }

    #[test]
    fn inspect_make_archive_detects_flags_and_canonical_json() {
        let archive_bytes = build_zip(&[
            ("canvas.fig", b"fig-makej-test"),
            ("meta.json", br#"{"file_name":"Example"}"#),
            ("ai_chat.json", br#"{"threads":[]}"#),
            ("figma_export.json", canonical_fixture_json().as_bytes()),
        ]);
        let archive_path = write_archive(&archive_bytes);

        let info = inspect_make_archive(&archive_path).expect("inspection should succeed");
        assert!(info.has_canvas_fig);
        assert!(info.has_meta_json);
        assert!(info.has_ai_chat_json);
        assert_eq!(
            info.canonical_figma_json_entry.as_deref(),
            Some("figma_export.json")
        );

        fs::remove_file(&archive_path).expect("archive should be removed");
    }

    #[test]
    fn read_canonical_figma_json_returns_payload() {
        let archive_bytes = build_zip(&[
            ("canvas.fig", b"fig-makej-test"),
            ("meta.json", br#"{"file_name":"Example"}"#),
            ("document.json", canonical_fixture_json().as_bytes()),
        ]);
        let archive_path = write_archive(&archive_bytes);

        let json = read_canonical_figma_json(&archive_path)
            .expect("canonical read should succeed")
            .expect("canonical json should exist");
        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"FRAME\""));

        fs::remove_file(&archive_path).expect("archive should be removed");
    }

    #[test]
    fn extract_make_archive_writes_manifest_and_canonical_copy() {
        let archive_bytes = build_zip(&[
            ("canvas.fig", b"fig-makej-test"),
            ("meta.json", br#"{"file_name":"Example"}"#),
            ("thumbnail.png", b"png"),
            ("images/", b""),
            ("images/asset_a", b"binary"),
            ("scene.json", canonical_fixture_json().as_bytes()),
        ]);
        let archive_path = write_archive(&archive_bytes);
        let output_dir = unique_path("output");

        let report =
            extract_make_archive(&archive_path, &output_dir, MakeExtractionOptions::default())
                .expect("extraction should succeed");
        assert!(report.canvas_fig_path.is_some());
        assert!(report.canonical_figma_json_copy_path.is_some());
        assert!(report.manifest_path.is_some());
        assert!(output_dir.join("canvas.fig").exists());
        assert!(output_dir.join("meta.json").exists());
        assert!(output_dir.join("thumbnail.png").exists());
        assert!(output_dir.join("images/asset_a").exists());
        assert!(output_dir.join(CANONICAL_COPY_NAME).exists());
        assert!(output_dir.join(MANIFEST_NAME).exists());

        fs::remove_file(&archive_path).expect("archive should be removed");
        fs::remove_dir_all(&output_dir).expect("output directory should be removed");
    }

    #[test]
    fn decode_make_archive_prefers_canonical_json_when_present() {
        let archive_bytes = build_zip(&[
            ("canvas.fig", b"fig-makej-test"),
            ("figma.json", canonical_fixture_json().as_bytes()),
        ]);
        let archive_path = write_archive(&archive_bytes);

        let json = decode_make_archive_to_import_json(&archive_path)
            .expect("decode should succeed with canonical json");
        assert!(json.contains("\"nodes\""));
        assert!(json.contains("\"FRAME\""));

        fs::remove_file(&archive_path).expect("archive should be removed");
    }

    #[test]
    fn extract_make_archive_rejects_unsafe_paths() {
        let archive_bytes = build_zip(&[
            ("../outside.txt", b"unsafe"),
            ("canvas.fig", b"fig-makej-test"),
        ]);
        let archive_path = write_archive(&archive_bytes);
        let output_dir = unique_path("unsafe_output");

        let err =
            extract_make_archive(&archive_path, &output_dir, MakeExtractionOptions::default())
                .expect_err("unsafe path should fail extraction");
        assert!(
            matches!(err, MakeBridgeError::UnsafeArchivePath(_)),
            "unexpected error: {err:?}"
        );

        fs::remove_file(&archive_path).expect("archive should be removed");
        if output_dir.exists() {
            fs::remove_dir_all(&output_dir).expect("output directory should be removed");
        }
    }

    #[test]
    fn normalize_canvas_fig_magic_rewrites_fig_make_header() {
        let bytes = b"fig-makepayload".to_vec();
        let normalized = normalize_canvas_fig_magic(&bytes);
        assert_eq!(&normalized[..8], b"fig-kiwi");
        assert_eq!(&normalized[8..], b"payload");
    }

    #[test]
    fn normalize_canvas_fig_magic_preserves_non_make_headers() {
        let bytes = b"fig-kiwipayload".to_vec();
        let normalized = normalize_canvas_fig_magic(&bytes);
        assert_eq!(normalized, bytes);
    }
}
