#![allow(missing_docs)]
use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::json;
use zip::ZipArchive;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    input_zip: PathBuf,
    output_dir: PathBuf,
    html_out: Option<PathBuf>,
    manifest_out: Option<PathBuf>,
    timeout_secs: u64,
    skip_download: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            input_zip: PathBuf::new(),
            output_dir: PathBuf::new(),
            html_out: None,
            manifest_out: None,
            timeout_secs: 30,
            skip_download: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DiscoveredUrl {
    original: String,
    resolved: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DownloadResult {
    status: String,
    http_status: Option<u16>,
    bytes: Option<u64>,
    content_type: Option<String>,
    local_file_name: Option<String>,
    error: Option<String>,
}

fn usage() -> &'static str {
    "Usage:
  cargo run --bin stitch_import -- \\
    --input <stitch.zip> \\
    --output-dir <artifacts/stitch_import> \\
    [--html-out <normalized.html>] \\
    [--manifest-out <manifest.json>] \\
    [--timeout-secs <seconds>] \\
    [--skip-download]

What it does:
  - Unpacks Stitch export zip into <output-dir>/unpacked
  - Finds primary HTML export file
  - Discovers remote http(s) URLs in html/src/href/CSS url(...)
  - Downloads external assets into <output-dir>/assets (unless --skip-download)
  - Rewrites HTML URLs to local asset paths and writes normalized HTML
  - Emits JSON manifest with asset status/details"
}

fn parse_args<I>(args: I) -> Result<CliOptions, String>
where
    I: IntoIterator<Item = String>,
{
    let args: Vec<String> = args.into_iter().collect();
    if args.len() == 1 {
        return Err(format!("missing arguments\n\n{}", usage()));
    }

    let mut options = CliOptions::default();
    let mut input_zip = None;
    let mut output_dir = None;

    let mut index = 1usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                input_zip = Some(PathBuf::from(take_value(&args, &mut index, "--input")?));
            }
            "--output-dir" => {
                output_dir = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--output-dir",
                )?));
            }
            "--html-out" => {
                options.html_out =
                    Some(PathBuf::from(take_value(&args, &mut index, "--html-out")?));
            }
            "--manifest-out" => {
                options.manifest_out = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--manifest-out",
                )?));
            }
            "--timeout-secs" => {
                let raw = take_value(&args, &mut index, "--timeout-secs")?;
                options.timeout_secs = raw.parse::<u64>().map_err(|err| {
                    format!(
                        "invalid value for --timeout-secs `{raw}`: {err}\n\n{}",
                        usage()
                    )
                })?;
            }
            "--skip-download" => {
                options.skip_download = true;
            }
            "--help" | "-h" => {
                return Err(usage().to_string());
            }
            other => {
                return Err(format!("unknown argument: {other}\n\n{}", usage()));
            }
        }
        index += 1;
    }

    let input_zip = input_zip.ok_or_else(|| format!("missing required --input\n\n{}", usage()))?;
    let output_dir =
        output_dir.ok_or_else(|| format!("missing required --output-dir\n\n{}", usage()))?;
    options.input_zip = input_zip;
    options.output_dir = output_dir;
    Ok(options)
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value for {flag}\n\n{}", usage()))
}

fn decode_basic_html_entities(value: &str) -> String {
    value
        .replace("&amp;", "&")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn discover_http_urls(html: &str) -> Vec<DiscoveredUrl> {
    let mut urls = Vec::new();
    let mut seen = HashSet::new();
    let mut scan_from = 0usize;

    while scan_from < html.len() {
        let rel_http = html[scan_from..].find("http://");
        let rel_https = html[scan_from..].find("https://");
        let next_rel = match (rel_http, rel_https) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };
        let Some(next_rel) = next_rel else {
            break;
        };
        let start = scan_from + next_rel;

        let mut end = start;
        let bytes = html.as_bytes();
        while end < html.len() {
            let ch = bytes[end] as char;
            if ch.is_ascii_whitespace() || matches!(ch, '"' | '\'' | ')' | '<' | '>' | '\\' | '`') {
                break;
            }
            end += 1;
        }

        if end > start {
            let original = html[start..end].to_string();
            let resolved = decode_basic_html_entities(&original);
            if seen.insert(original.clone()) {
                urls.push(DiscoveredUrl { original, resolved });
            }
        }
        scan_from = end.max(start + 1);
    }

    urls
}

fn sanitize_component(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "asset".to_string()
    } else {
        out
    }
}

fn fnv1a_64(input: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in input.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn extension_from_url(url: &str) -> Option<String> {
    let stripped_fragment = url.split('#').next().unwrap_or(url);
    let stripped_query = stripped_fragment
        .split('?')
        .next()
        .unwrap_or(stripped_fragment);
    let after_scheme = stripped_query.split_once("://")?.1;
    let path_start = after_scheme.find('/')?;
    let path = &after_scheme[path_start + 1..];
    let tail = path.rsplit('/').next()?;
    if tail.is_empty() {
        return None;
    }
    let ext = tail.rsplit('.').next()?;
    if ext == tail || ext.is_empty() {
        return None;
    }
    let ext = ext.to_ascii_lowercase();
    if ext.len() <= 8 && ext.chars().all(|ch| ch.is_ascii_alphanumeric()) {
        Some(ext)
    } else {
        None
    }
}

fn extension_from_content_type(content_type: Option<&str>) -> Option<String> {
    let content_type = content_type?.to_ascii_lowercase();
    let media_type = content_type
        .split(';')
        .next()
        .map(str::trim)
        .unwrap_or_default();
    match media_type {
        "image/png" => Some("png".to_string()),
        "image/jpeg" => Some("jpg".to_string()),
        "image/webp" => Some("webp".to_string()),
        "image/gif" => Some("gif".to_string()),
        "image/svg+xml" => Some("svg".to_string()),
        "text/css" => Some("css".to_string()),
        "application/javascript" | "text/javascript" => Some("js".to_string()),
        "text/html" => Some("html".to_string()),
        _ => None,
    }
}

fn host_from_url(url: &str) -> String {
    let rest = url.split_once("://").map(|(_, tail)| tail).unwrap_or(url);
    rest.split(['/', '?', '#'])
        .next()
        .unwrap_or("unknown-host")
        .to_string()
}

fn determine_asset_file_name(url: &str, content_type: Option<&str>) -> String {
    let host = sanitize_component(&host_from_url(url));
    let hash = format!("{:016x}", fnv1a_64(url));
    let ext = extension_from_url(url)
        .or_else(|| extension_from_content_type(content_type))
        .unwrap_or_else(|| "bin".to_string());
    format!("{host}_{hash}.{ext}")
}

fn ensure_dir(path: &Path) -> Result<(), String> {
    fs::create_dir_all(path).map_err(|err| format!("failed to create {}: {err}", path.display()))
}

fn extract_zip(input_zip: &Path, output_dir: &Path) -> Result<Vec<PathBuf>, String> {
    ensure_dir(output_dir)?;
    let file = File::open(input_zip)
        .map_err(|err| format!("failed to open {}: {err}", input_zip.display()))?;
    let mut archive = ZipArchive::new(file)
        .map_err(|err| format!("failed to parse zip archive {}: {err}", input_zip.display()))?;

    let mut extracted = Vec::new();
    for index in 0..archive.len() {
        let mut entry = archive
            .by_index(index)
            .map_err(|err| format!("failed to read zip entry #{index}: {err}"))?;
        let Some(safe_name) = entry.enclosed_name().map(Path::to_path_buf) else {
            continue;
        };
        let out_path = output_dir.join(&safe_name);
        if entry.name().ends_with('/') {
            ensure_dir(&out_path)?;
            continue;
        }
        if let Some(parent) = out_path.parent() {
            ensure_dir(parent)?;
        }
        let mut bytes = Vec::new();
        entry
            .read_to_end(&mut bytes)
            .map_err(|err| format!("failed to read zip entry {}: {err}", entry.name()))?;
        fs::write(&out_path, bytes)
            .map_err(|err| format!("failed to write {}: {err}", out_path.display()))?;
        extracted.push(safe_name);
    }
    Ok(extracted)
}

fn select_primary_html(extracted_relative_paths: &[PathBuf]) -> Option<PathBuf> {
    for path in extracted_relative_paths {
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.eq_ignore_ascii_case("code.html"))
        {
            return Some(path.clone());
        }
    }
    extracted_relative_paths.iter().find_map(|path| {
        path.extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("html") || ext.eq_ignore_ascii_case("htm"))
            .then(|| path.clone())
    })
}

fn rewrite_html_urls(html: &str, replacements: &HashMap<String, String>) -> String {
    let mut out = html.to_string();
    for (from, to) in replacements {
        out = out.replace(from, to);
    }
    out
}

fn queue_discovered_urls(
    source: &str,
    pending: &mut Vec<DiscoveredUrl>,
    seen_resolved: &mut HashSet<String>,
) {
    for discovered in discover_http_urls(source) {
        if seen_resolved.insert(discovered.resolved.clone()) {
            pending.push(discovered);
        }
    }
}

fn write_text_file(path: &Path, contents: &str) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        ensure_dir(parent)?;
    }
    fs::write(path, contents).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn run_with_options(options: &CliOptions) -> Result<(), String> {
    if !options.input_zip.exists() {
        return Err(format!(
            "input zip does not exist: {}",
            options.input_zip.display()
        ));
    }
    ensure_dir(&options.output_dir)?;

    let unpacked_dir = options.output_dir.join("unpacked");
    let assets_dir = options.output_dir.join("assets");
    ensure_dir(&assets_dir)?;
    let extracted = extract_zip(&options.input_zip, &unpacked_dir)?;

    let html_relative = select_primary_html(&extracted)
        .ok_or_else(|| "no html file found in stitch zip".to_string())?;
    let html_source_path = unpacked_dir.join(&html_relative);
    let html_source = fs::read_to_string(&html_source_path).map_err(|err| {
        format!(
            "failed to read extracted html file {}: {err}",
            html_source_path.display()
        )
    })?;
    let mut pending_urls = Vec::new();
    let mut seen_resolved = HashSet::new();
    queue_discovered_urls(&html_source, &mut pending_urls, &mut seen_resolved);

    let html_out_path = options
        .html_out
        .clone()
        .unwrap_or_else(|| options.output_dir.join("code.normalized.html"));
    let manifest_out_path = options
        .manifest_out
        .clone()
        .unwrap_or_else(|| options.output_dir.join("manifest.json"));

    let client = Client::builder()
        .timeout(Duration::from_secs(options.timeout_secs))
        .build()
        .map_err(|err| format!("failed to initialize HTTP client: {err}"))?;

    let mut replacements = HashMap::new();
    let mut manifest_entries = Vec::new();
    let mut downloaded_css_files = Vec::<PathBuf>::new();
    let mut downloaded = 0usize;
    let mut skipped = 0usize;
    let mut failed = 0usize;

    let mut queue_index = 0usize;
    while queue_index < pending_urls.len() {
        let url = pending_urls[queue_index].clone();
        queue_index += 1;
        let mut local_ref = None;
        let result = if options.skip_download {
            skipped += 1;
            DownloadResult {
                status: "skipped_by_flag".to_string(),
                http_status: None,
                bytes: None,
                content_type: None,
                local_file_name: None,
                error: None,
            }
        } else {
            match client.get(&url.resolved).send() {
                Ok(response) => {
                    let http_status = response.status().as_u16();
                    let status = response.status();
                    if !status.is_success() {
                        failed += 1;
                        DownloadResult {
                            status: "http_error".to_string(),
                            http_status: Some(http_status),
                            bytes: None,
                            content_type: None,
                            local_file_name: None,
                            error: Some(format!("HTTP {}", status)),
                        }
                    } else {
                        let content_type = response
                            .headers()
                            .get(reqwest::header::CONTENT_TYPE)
                            .and_then(|value| value.to_str().ok())
                            .map(str::to_string);
                        let file_name =
                            determine_asset_file_name(&url.resolved, content_type.as_deref());
                        let out_path = assets_dir.join(&file_name);
                        match response.bytes() {
                            Ok(bytes) => {
                                if let Err(err) = fs::write(&out_path, bytes.as_ref()) {
                                    failed += 1;
                                    DownloadResult {
                                        status: "write_error".to_string(),
                                        http_status: Some(http_status),
                                        bytes: None,
                                        content_type,
                                        local_file_name: None,
                                        error: Some(err.to_string()),
                                    }
                                } else {
                                    downloaded += 1;
                                    local_ref = Some(format!("assets/{file_name}"));
                                    DownloadResult {
                                        status: "downloaded".to_string(),
                                        http_status: Some(http_status),
                                        bytes: Some(bytes.len() as u64),
                                        content_type,
                                        local_file_name: Some(file_name),
                                        error: None,
                                    }
                                }
                            }
                            Err(err) => {
                                failed += 1;
                                DownloadResult {
                                    status: "read_error".to_string(),
                                    http_status: Some(http_status),
                                    bytes: None,
                                    content_type,
                                    local_file_name: None,
                                    error: Some(err.to_string()),
                                }
                            }
                        }
                    }
                }
                Err(err) => {
                    failed += 1;
                    DownloadResult {
                        status: "request_error".to_string(),
                        http_status: None,
                        bytes: None,
                        content_type: None,
                        local_file_name: None,
                        error: Some(err.to_string()),
                    }
                }
            }
        };

        if let Some(local) = &local_ref {
            replacements.insert(url.original.clone(), local.clone());
            replacements.insert(url.resolved.clone(), local.clone());
        }

        let local_path = result
            .local_file_name
            .as_ref()
            .map(|name| assets_dir.join(name).to_string_lossy().to_string());
        if result.status == "downloaded"
            && result
                .local_file_name
                .as_ref()
                .is_some_and(|name| name.to_ascii_lowercase().ends_with(".css"))
            && let Some(local_path) = local_path.as_ref().map(PathBuf::from)
        {
            if let Ok(css_source) = fs::read_to_string(&local_path) {
                queue_discovered_urls(&css_source, &mut pending_urls, &mut seen_resolved);
            }
            downloaded_css_files.push(local_path);
        }
        manifest_entries.push(json!({
            "original_url": url.original,
            "resolved_url": url.resolved,
            "status": result.status,
            "http_status": result.http_status,
            "content_type": result.content_type,
            "bytes": result.bytes,
            "local_ref": local_ref,
            "local_path": local_path,
            "error": result.error,
        }));
    }

    for css_path in &downloaded_css_files {
        if let Ok(css_source) = fs::read_to_string(css_path) {
            let rewritten_css = rewrite_html_urls(&css_source, &replacements);
            if rewritten_css != css_source {
                fs::write(css_path, rewritten_css)
                    .map_err(|err| format!("failed to rewrite {}: {err}", css_path.display()))?;
            }
        }
    }

    let rewritten_html = rewrite_html_urls(&html_source, &replacements);
    write_text_file(&html_out_path, &rewritten_html)?;

    let mut extracted_listing = BTreeMap::new();
    for relative in &extracted {
        extracted_listing.insert(
            relative.to_string_lossy().to_string(),
            unpacked_dir.join(relative).to_string_lossy().to_string(),
        );
    }

    let manifest = json!({
        "input_zip": options.input_zip.to_string_lossy(),
        "output_dir": options.output_dir.to_string_lossy(),
        "unpacked_dir": unpacked_dir.to_string_lossy(),
        "source_html_relative": html_relative.to_string_lossy(),
        "source_html_path": html_source_path.to_string_lossy(),
        "normalized_html_path": html_out_path.to_string_lossy(),
        "asset_dir": assets_dir.to_string_lossy(),
        "timeout_secs": options.timeout_secs,
        "skip_download": options.skip_download,
        "extracted_files": extracted_listing,
        "url_count": pending_urls.len(),
        "downloaded_count": downloaded,
        "skipped_count": skipped,
        "failed_count": failed,
        "entries": manifest_entries,
    });
    let manifest_text = serde_json::to_string_pretty(&manifest)
        .map_err(|err| format!("failed to serialize manifest json: {err}"))?;
    write_text_file(&manifest_out_path, &manifest_text)?;

    println!("Input zip: {}", options.input_zip.display());
    println!("Unpacked to: {}", unpacked_dir.display());
    println!("Primary html: {}", html_source_path.display());
    println!("Normalized html: {}", html_out_path.display());
    println!("Manifest: {}", manifest_out_path.display());
    println!("Discovered URLs: {}", pending_urls.len());
    println!("Downloaded: {downloaded}, skipped: {skipped}, failed: {failed}");

    Ok(())
}

fn main() {
    let result = parse_args(std::env::args()).and_then(|options| run_with_options(&options));
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_requires_input_and_output_dir() {
        let err = parse_args([
            "stitch_import".to_string(),
            "--output-dir".to_string(),
            "out".to_string(),
        ])
        .expect_err("missing input should fail");
        assert!(err.contains("missing required --input"));

        let err = parse_args([
            "stitch_import".to_string(),
            "--input".to_string(),
            "stitch.zip".to_string(),
        ])
        .expect_err("missing output-dir should fail");
        assert!(err.contains("missing required --output-dir"));
    }

    #[test]
    fn parse_args_reads_overrides() {
        let options = parse_args([
            "stitch_import".to_string(),
            "--input".to_string(),
            "stitch.zip".to_string(),
            "--output-dir".to_string(),
            "artifacts/stitch".to_string(),
            "--html-out".to_string(),
            "artifacts/stitch/code.local.html".to_string(),
            "--manifest-out".to_string(),
            "artifacts/stitch/manifest.json".to_string(),
            "--timeout-secs".to_string(),
            "55".to_string(),
            "--skip-download".to_string(),
        ])
        .expect("arguments should parse");
        assert_eq!(options.input_zip, PathBuf::from("stitch.zip"));
        assert_eq!(options.output_dir, PathBuf::from("artifacts/stitch"));
        assert_eq!(
            options.html_out,
            Some(PathBuf::from("artifacts/stitch/code.local.html"))
        );
        assert_eq!(
            options.manifest_out,
            Some(PathBuf::from("artifacts/stitch/manifest.json"))
        );
        assert_eq!(options.timeout_secs, 55);
        assert!(options.skip_download);
    }

    #[test]
    fn decode_basic_html_entities_decodes_amp_and_quotes() {
        let decoded = decode_basic_html_entities(
            "https://example.com/a?x=1&amp;y=2&amp;z=&quot;ok&quot;&#39;yes&#39;",
        );
        assert_eq!(decoded, "https://example.com/a?x=1&y=2&z=\"ok\"'yes'");
    }

    #[test]
    fn discover_http_urls_finds_html_and_css_sources() {
        let html = r#"
<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk&amp;display=swap" rel="stylesheet" />
<script src="https://cdn.tailwindcss.com?plugins=forms"></script>
<div style="background-image: url('https://lh3.googleusercontent.com/abc123')"></div>
<div style="background-image: url('data:image/png;base64,AAA=')"></div>
"#;
        let urls = discover_http_urls(html);
        let original = urls
            .iter()
            .map(|entry| entry.original.as_str())
            .collect::<Vec<_>>();
        assert_eq!(original.len(), 3);
        assert!(
            original.contains(
                &"https://fonts.googleapis.com/css2?family=Space+Grotesk&amp;display=swap"
            )
        );
        assert!(original.contains(&"https://cdn.tailwindcss.com?plugins=forms"));
        assert!(original.contains(&"https://lh3.googleusercontent.com/abc123"));
    }

    #[test]
    fn queue_discovered_urls_finds_nested_css_font_urls() {
        let mut pending = Vec::new();
        let mut seen = HashSet::new();
        let html = r#"<link href="https://fonts.googleapis.com/css2?family=Space+Grotesk&display=swap" rel="stylesheet"/>"#;
        let css = r#"
@font-face {
  src: url(https://fonts.gstatic.com/s/spacegrotesk/v22/font.ttf) format('truetype');
}
"#;
        queue_discovered_urls(html, &mut pending, &mut seen);
        queue_discovered_urls(css, &mut pending, &mut seen);

        let urls = pending
            .iter()
            .map(|entry| entry.resolved.clone())
            .collect::<HashSet<_>>();
        assert!(
            urls.contains("https://fonts.googleapis.com/css2?family=Space+Grotesk&display=swap")
        );
        assert!(urls.contains("https://fonts.gstatic.com/s/spacegrotesk/v22/font.ttf"));
    }

    #[test]
    fn rewrite_html_urls_rewrites_nested_css_references() {
        let css = r#"
@font-face {
  src: url(https://fonts.gstatic.com/s/spacegrotesk/v22/font.ttf) format('truetype');
}
"#;
        let mut replacements = HashMap::new();
        replacements.insert(
            "https://fonts.gstatic.com/s/spacegrotesk/v22/font.ttf".to_string(),
            "assets/fonts_gstatic_spacegrotesk.ttf".to_string(),
        );
        let rewritten = rewrite_html_urls(css, &replacements);
        assert!(rewritten.contains("assets/fonts_gstatic_spacegrotesk.ttf"));
        assert!(!rewritten.contains("https://fonts.gstatic.com/s/spacegrotesk/v22/font.ttf"));
    }

    #[test]
    fn determine_asset_file_name_is_stable() {
        let url = "https://lh3.googleusercontent.com/aida-public/XYZ123?foo=bar";
        let a = determine_asset_file_name(url, Some("image/jpeg"));
        let b = determine_asset_file_name(url, Some("image/jpeg"));
        assert_eq!(a, b);
        assert!(a.ends_with(".jpg") || a.ends_with(".bin"));
    }

    #[test]
    fn rewrite_html_urls_replaces_all_known_urls() {
        let html = "<img src=\"https://a.com/x.png\"/><img src='https://a.com/x.png'/>";
        let mut map = HashMap::new();
        map.insert(
            "https://a.com/x.png".to_string(),
            "assets/a_123.png".to_string(),
        );
        let rewritten = rewrite_html_urls(html, &map);
        assert!(!rewritten.contains("https://a.com/x.png"));
        assert_eq!(
            rewritten.matches("assets/a_123.png").count(),
            2,
            "expected both occurrences rewritten"
        );
    }

    #[test]
    fn select_primary_html_prefers_code_html() {
        let files = vec![
            PathBuf::from("index.html"),
            PathBuf::from("nested/code.html"),
            PathBuf::from("about.htm"),
        ];
        let selected = select_primary_html(&files).expect("should select html file");
        assert_eq!(selected, PathBuf::from("nested/code.html"));
    }

    #[test]
    fn extension_guessing_from_url_and_content_type() {
        assert_eq!(
            extension_from_url("https://example.com/a/b/c.css?v=1"),
            Some("css".to_string())
        );
        assert_eq!(
            extension_from_url("https://cdn.tailwindcss.com?plugins=forms"),
            None
        );
        assert_eq!(
            extension_from_content_type(Some("text/javascript; charset=utf-8")),
            Some("js".to_string())
        );
        assert_eq!(
            extension_from_content_type(Some("application/octet-stream")),
            None
        );
    }

    #[test]
    fn determine_asset_file_name_prefers_content_type_when_url_has_no_file_path() {
        let file_name = determine_asset_file_name(
            "https://cdn.tailwindcss.com?plugins=forms,container-queries",
            Some("text/javascript"),
        );
        assert!(file_name.ends_with(".js"), "expected .js, got {file_name}");
    }
}
