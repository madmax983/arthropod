#![allow(missing_docs)]
//! Utility script to pull and sync Figma design files into the project.

use std::collections::{BTreeSet, HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use reqwest::blocking::Client;
use serde_json::{Map as JsonMap, Value as JsonValue, json};

const FIGMA_API_BASE: &str = "https://api.figma.com/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    file_key: String,
    token: Option<String>,
    output: PathBuf,
    assets_dir: Option<PathBuf>,
    manifest_out: Option<PathBuf>,
    timeout_secs: u64,
    page_ids: Vec<String>,
    skip_images: bool,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            file_key: String::new(),
            token: None,
            output: PathBuf::new(),
            assets_dir: None,
            manifest_out: None,
            timeout_secs: 30,
            page_ids: Vec::new(),
            skip_images: false,
        }
    }
}

fn usage() -> &'static str {
    "Usage:
  cargo run --bin figma_pull -- \\
    --file-key <FIGMA_FILE_KEY> \\
    --output <figma_scene.json> \\
    [--token <FIGMA_TOKEN>] \\
    [--page-id <CANVAS_ID> ...] \\
    [--assets-dir <asset_dir>] \\
    [--manifest-out <images_manifest.json>] \\
    [--timeout-secs <seconds>] \\
    [--skip-images]

Notes:
  - Reads FIGMA_TOKEN from environment when --token is omitted.
  - Pulls full scene graph from /v1/files/:key?geometry=paths.
  - Resolves image fills through /v1/files/:key/images.

Example:
  cargo run --bin figma_pull -- \
    --file-key AbCdEfGhIjKlMnOp \
    --output artifacts/figma/scene.json \
    --assets-dir artifacts/figma/images"
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
    let mut file_key = None;
    let mut output = None;

    let mut index = 1usize;
    while index < args.len() {
        match args[index].as_str() {
            "--file-key" => {
                file_key = Some(take_value(&args, &mut index, "--file-key")?);
            }
            "--token" => {
                options.token = Some(take_value(&args, &mut index, "--token")?);
            }
            "--output" => {
                output = Some(PathBuf::from(take_value(&args, &mut index, "--output")?));
            }
            "--assets-dir" => {
                options.assets_dir = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--assets-dir",
                )?));
            }
            "--manifest-out" => {
                options.manifest_out = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--manifest-out",
                )?));
            }
            "--page-id" => {
                options
                    .page_ids
                    .push(take_value(&args, &mut index, "--page-id")?);
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
            "--skip-images" => {
                options.skip_images = true;
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

    let file_key = file_key.ok_or_else(|| format!("missing required --file-key\n\n{}", usage()))?;
    let output = output.ok_or_else(|| format!("missing required --output\n\n{}", usage()))?;
    options.file_key = file_key;
    options.output = output;
    Ok(options)
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value for {flag}\n\n{}", usage()))
}

fn resolve_api_token(cli_token: Option<&str>, env_token: Option<&str>) -> Result<String, String> {
    if let Some(token) = cli_token.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(token.to_string());
    }
    if let Some(token) = env_token.map(str::trim).filter(|value| !value.is_empty()) {
        return Ok(token.to_string());
    }
    Err("missing Figma API token; pass --token or set FIGMA_TOKEN".to_string())
}

fn make_client(timeout_secs: u64) -> Result<Client, String> {
    Client::builder()
        .timeout(Duration::from_secs(timeout_secs))
        .build()
        .map_err(|err| format!("failed to initialize HTTP client: {err}"))
}

fn fetch_json(client: &Client, url: &str, token: &str) -> Result<JsonValue, String> {
    let response = client
        .get(url)
        .header("X-Figma-Token", token)
        .send()
        .map_err(|err| format!("request failed for {url}: {err}"))?;

    let status = response.status();
    let body = response
        .text()
        .map_err(|err| format!("failed to read response body from {url}: {err}"))?;
    if !status.is_success() {
        return Err(format!(
            "request failed for {url}: HTTP {status} body={}",
            truncate_for_error(&body)
        ));
    }

    serde_json::from_str(&body).map_err(|err| {
        format!(
            "failed to parse JSON response from {url}: {err}; body={}",
            truncate_for_error(&body)
        )
    })
}

fn truncate_for_error(text: &str) -> String {
    const LIMIT: usize = 512;
    if text.len() <= LIMIT {
        text.to_string()
    } else {
        format!("{}...", &text[..LIMIT])
    }
}

fn extract_export_nodes(
    file_response: &JsonValue,
    page_ids: &[String],
) -> Result<Vec<JsonValue>, String> {
    let document = file_response
        .get("document")
        .and_then(JsonValue::as_object)
        .ok_or_else(|| "file response missing `document` object".to_string())?;
    let children = document
        .get("children")
        .and_then(JsonValue::as_array)
        .ok_or_else(|| "file response `document` missing `children` array".to_string())?;

    if page_ids.is_empty() {
        return Ok(children.clone());
    }

    let requested: HashSet<&str> = page_ids.iter().map(String::as_str).collect();
    let mut selected = Vec::new();
    let mut found = HashSet::new();
    for child in children {
        let Some(id) = child.get("id").and_then(JsonValue::as_str) else {
            continue;
        };
        if requested.contains(id) {
            selected.push(child.clone());
            found.insert(id.to_string());
        }
    }

    if selected.is_empty() {
        return Err(format!(
            "none of the requested --page-id values were found in document children: {}",
            page_ids.join(", ")
        ));
    }

    if found.len() != requested.len() {
        let missing = page_ids
            .iter()
            .filter(|id| !found.contains((*id).as_str()))
            .cloned()
            .collect::<Vec<_>>();
        return Err(format!(
            "some requested --page-id values were not found: {}",
            missing.join(", ")
        ));
    }

    Ok(selected)
}

fn collect_image_refs(nodes: &[JsonValue]) -> Vec<String> {
    let mut refs = BTreeSet::new();
    for node in nodes {
        collect_image_refs_from_node(node, &mut refs);
    }
    refs.into_iter().collect()
}

fn collect_image_refs_from_node(node: &JsonValue, refs: &mut BTreeSet<String>) {
    let Some(object) = node.as_object() else {
        return;
    };

    for paint_key in ["fills", "strokes", "background", "backgrounds"] {
        if let Some(paints) = object.get(paint_key).and_then(JsonValue::as_array) {
            for paint in paints {
                collect_image_ref_from_paint(paint, refs);
            }
        }
    }

    if let Some(children) = object.get("children").and_then(JsonValue::as_array) {
        for child in children {
            collect_image_refs_from_node(child, refs);
        }
    }
}

fn collect_image_ref_from_paint(paint: &JsonValue, refs: &mut BTreeSet<String>) {
    let Some(object) = paint.as_object() else {
        return;
    };
    let paint_type = object
        .get("type")
        .and_then(JsonValue::as_str)
        .map(str::to_ascii_uppercase);
    if paint_type.as_deref() != Some("IMAGE") {
        return;
    }

    for key in ["imageRef", "imageHash", "imageId"] {
        if let Some(reference) = object.get(key).and_then(JsonValue::as_str) {
            let trimmed = reference.trim();
            if !trimmed.is_empty() {
                refs.insert(trimmed.to_string());
                return;
            }
        }
    }
}

fn extract_image_url_map(
    images_response: &JsonValue,
) -> Result<HashMap<String, Option<String>>, String> {
    let images_object = images_response
        .get("meta")
        .and_then(JsonValue::as_object)
        .and_then(|meta| meta.get("images"))
        .or_else(|| images_response.get("images"))
        .and_then(JsonValue::as_object)
        .ok_or_else(|| "image response missing `meta.images` object".to_string())?;

    let mut map = HashMap::with_capacity(images_object.len());
    for (image_ref, url) in images_object {
        let value = url.as_str().map(str::to_string);
        map.insert(image_ref.clone(), value);
    }
    Ok(map)
}

fn sanitize_file_stem(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "image".to_string()
    } else {
        out
    }
}

fn extension_from_url(url: &str) -> Option<String> {
    let path = url.split('?').next().unwrap_or(url);
    let file = path.rsplit('/').next()?;
    let ext = file.rsplit('.').next()?;
    if ext == file {
        return None;
    }
    let ext = ext.to_ascii_lowercase();
    if ext.chars().all(|ch| ch.is_ascii_alphanumeric()) && (2..=5).contains(&ext.len()) {
        Some(ext)
    } else {
        None
    }
}

fn extension_from_content_type(content_type: Option<&str>) -> Option<String> {
    let content_type = content_type?.to_ascii_lowercase();
    match content_type.as_str() {
        "image/png" => Some("png".to_string()),
        "image/jpeg" => Some("jpg".to_string()),
        "image/webp" => Some("webp".to_string()),
        "image/gif" => Some("gif".to_string()),
        "image/bmp" => Some("bmp".to_string()),
        "image/svg+xml" => Some("svg".to_string()),
        _ => None,
    }
}

fn figma_image_reference_to_id(reference: &str) -> u64 {
    if let Ok(parsed) = reference.parse::<u64>() {
        return parsed;
    }
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in reference.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash | (1_u64 << 63)
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    Ok(())
}

fn write_json_pretty(path: &std::path::Path, value: &JsonValue) -> Result<(), String> {
    ensure_parent_dir(path)?;
    let text = serde_json::to_string_pretty(value)
        .map_err(|err| format!("failed to serialize JSON for {}: {err}", path.display()))?;
    fs::write(path, text).map_err(|err| format!("failed to write {}: {err}", path.display()))
}

fn download_image(
    client: &Client,
    token: &str,
    image_ref: &str,
    url: &str,
    assets_dir: &Path,
) -> JsonValue {
    let mut entry = JsonMap::new();
    entry.insert(
        "image_ref".to_string(),
        JsonValue::String(image_ref.to_string()),
    );
    entry.insert(
        "image_id_u64".to_string(),
        JsonValue::Number(serde_json::Number::from(figma_image_reference_to_id(
            image_ref,
        ))),
    );
    entry.insert(
        "download_url".to_string(),
        JsonValue::String(url.to_string()),
    );

    let response = match client.get(url).header("X-Figma-Token", token).send() {
        Ok(response) => response,
        Err(err) => {
            entry.insert(
                "status".to_string(),
                JsonValue::String("download_failed".to_string()),
            );
            entry.insert("error".to_string(), JsonValue::String(err.to_string()));
            return JsonValue::Object(entry);
        }
    };

    let status = response.status();
    if !status.is_success() {
        entry.insert(
            "status".to_string(),
            JsonValue::String("download_failed".to_string()),
        );
        entry.insert(
            "error".to_string(),
            JsonValue::String(format!("HTTP {status} while downloading image")),
        );
        return JsonValue::Object(entry);
    }

    let content_type = response
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    let bytes = match response.bytes() {
        Ok(bytes) => bytes,
        Err(err) => {
            entry.insert(
                "status".to_string(),
                JsonValue::String("download_failed".to_string()),
            );
            entry.insert("error".to_string(), JsonValue::String(err.to_string()));
            return JsonValue::Object(entry);
        }
    };

    let extension = extension_from_url(url)
        .or_else(|| extension_from_content_type(content_type.as_deref()))
        .unwrap_or_else(|| "bin".to_string());
    let file_name = format!("{}.{}", sanitize_file_stem(image_ref), extension);
    let path = assets_dir.join(file_name);
    if let Err(err) = ensure_parent_dir(&path) {
        entry.insert(
            "status".to_string(),
            JsonValue::String("download_failed".to_string()),
        );
        entry.insert("error".to_string(), JsonValue::String(err));
        return JsonValue::Object(entry);
    }
    if let Err(err) = fs::write(&path, bytes.as_ref()) {
        entry.insert(
            "status".to_string(),
            JsonValue::String("download_failed".to_string()),
        );
        entry.insert("error".to_string(), JsonValue::String(err.to_string()));
        return JsonValue::Object(entry);
    }

    entry.insert(
        "status".to_string(),
        JsonValue::String("downloaded".to_string()),
    );
    entry.insert(
        "local_path".to_string(),
        JsonValue::String(path.to_string_lossy().to_string()),
    );
    entry.insert(
        "bytes".to_string(),
        JsonValue::Number(serde_json::Number::from(bytes.len() as u64)),
    );
    if let Some(content_type) = content_type {
        entry.insert("content_type".to_string(), JsonValue::String(content_type));
    }
    JsonValue::Object(entry)
}

fn run_with_options(options: &CliOptions) -> Result<(), String> {
    let env_token = std::env::var("FIGMA_TOKEN").ok();
    let token = resolve_api_token(options.token.as_deref(), env_token.as_deref())?;
    let client = make_client(options.timeout_secs)?;

    let file_url = format!("{FIGMA_API_BASE}/files/{}?geometry=paths", options.file_key);
    let file_response = fetch_json(&client, &file_url, &token)?;
    let nodes = extract_export_nodes(&file_response, &options.page_ids)?;
    let scene_output = json!({ "nodes": nodes });
    write_json_pretty(&options.output, &scene_output)?;

    let image_refs = scene_output
        .get("nodes")
        .and_then(JsonValue::as_array)
        .map_or_else(Vec::new, |nodes| collect_image_refs(nodes));

    let mut manifest_entries = Vec::new();
    let mut resolved_image_urls = 0usize;
    let mut downloaded_images = 0usize;

    if !options.skip_images && !image_refs.is_empty() {
        let images_url = format!("{FIGMA_API_BASE}/files/{}/images", options.file_key);
        let images_response = fetch_json(&client, &images_url, &token)?;
        let image_map = extract_image_url_map(&images_response)?;

        for image_ref in &image_refs {
            let mut entry = JsonMap::new();
            entry.insert(
                "image_ref".to_string(),
                JsonValue::String(image_ref.clone()),
            );
            entry.insert(
                "image_id_u64".to_string(),
                JsonValue::Number(serde_json::Number::from(figma_image_reference_to_id(
                    image_ref,
                ))),
            );

            let Some(url) = image_map.get(image_ref).and_then(|value| value.as_ref()) else {
                entry.insert(
                    "status".to_string(),
                    JsonValue::String("missing_download_url".to_string()),
                );
                manifest_entries.push(JsonValue::Object(entry));
                continue;
            };
            resolved_image_urls += 1;

            if let Some(assets_dir) = &options.assets_dir {
                let downloaded = download_image(&client, &token, image_ref, url, assets_dir);
                if downloaded
                    .get("status")
                    .and_then(JsonValue::as_str)
                    .is_some_and(|status| status == "downloaded")
                {
                    downloaded_images += 1;
                }
                manifest_entries.push(downloaded);
            } else {
                entry.insert(
                    "status".to_string(),
                    JsonValue::String("resolved".to_string()),
                );
                entry.insert("download_url".to_string(), JsonValue::String(url.clone()));
                manifest_entries.push(JsonValue::Object(entry));
            }
        }
    }

    if options.skip_images && !image_refs.is_empty() {
        for image_ref in &image_refs {
            let mut entry = JsonMap::new();
            entry.insert(
                "image_ref".to_string(),
                JsonValue::String(image_ref.clone()),
            );
            entry.insert(
                "image_id_u64".to_string(),
                JsonValue::Number(serde_json::Number::from(figma_image_reference_to_id(
                    image_ref,
                ))),
            );
            entry.insert(
                "status".to_string(),
                JsonValue::String("skipped_by_flag".to_string()),
            );
            manifest_entries.push(JsonValue::Object(entry));
        }
    }

    let manifest_out = options.manifest_out.clone().or_else(|| {
        options
            .assets_dir
            .as_ref()
            .map(|assets| assets.join("images_manifest.json"))
    });
    if let Some(path) = manifest_out {
        let manifest = json!({
            "file_key": options.file_key,
            "output_json": options.output.to_string_lossy(),
            "image_ref_count": image_refs.len(),
            "resolved_image_url_count": resolved_image_urls,
            "downloaded_image_count": downloaded_images,
            "entries": manifest_entries,
        });
        write_json_pretty(&path, &manifest)?;
        println!("Wrote image manifest: {}", path.display());
    }

    let node_count = scene_output
        .get("nodes")
        .and_then(JsonValue::as_array)
        .map(Vec::len)
        .unwrap_or(0);
    println!("Pulled Figma file: {}", options.file_key);
    println!("Wrote scene JSON: {}", options.output.display());
    println!("Top-level nodes exported: {node_count}");
    println!("Image refs discovered: {}", image_refs.len());
    if !options.skip_images {
        println!("Image URLs resolved: {resolved_image_urls}");
        if options.assets_dir.is_some() {
            println!("Images downloaded: {downloaded_images}");
        }
    }

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
    fn parse_args_reads_required_values() {
        let options = parse_args([
            "figma_pull".to_string(),
            "--file-key".to_string(),
            "AbCd".to_string(),
            "--output".to_string(),
            "out/scene.json".to_string(),
        ])
        .expect("required arguments should parse");

        assert_eq!(options.file_key, "AbCd");
        assert_eq!(options.output, PathBuf::from("out/scene.json"));
        assert_eq!(options.timeout_secs, 30);
        assert!(options.page_ids.is_empty());
        assert!(!options.skip_images);
    }

    #[test]
    fn parse_args_reads_optional_values() {
        let options = parse_args([
            "figma_pull".to_string(),
            "--file-key".to_string(),
            "AbCd".to_string(),
            "--token".to_string(),
            "token123".to_string(),
            "--output".to_string(),
            "out/scene.json".to_string(),
            "--assets-dir".to_string(),
            "out/images".to_string(),
            "--manifest-out".to_string(),
            "out/manifest.json".to_string(),
            "--page-id".to_string(),
            "12:0".to_string(),
            "--page-id".to_string(),
            "13:0".to_string(),
            "--timeout-secs".to_string(),
            "60".to_string(),
            "--skip-images".to_string(),
        ])
        .expect("optional arguments should parse");

        assert_eq!(options.token.as_deref(), Some("token123"));
        assert_eq!(options.assets_dir, Some(PathBuf::from("out/images")));
        assert_eq!(
            options.manifest_out,
            Some(PathBuf::from("out/manifest.json"))
        );
        assert_eq!(
            options.page_ids,
            vec!["12:0".to_string(), "13:0".to_string()]
        );
        assert_eq!(options.timeout_secs, 60);
        assert!(options.skip_images);
    }

    #[test]
    fn parse_args_requires_file_key_and_output() {
        let missing_key = parse_args([
            "figma_pull".to_string(),
            "--output".to_string(),
            "out.json".to_string(),
        ])
        .expect_err("missing key should fail");
        assert!(
            missing_key.contains("missing required --file-key"),
            "unexpected missing_key error: {missing_key}"
        );

        let missing_output = parse_args([
            "figma_pull".to_string(),
            "--file-key".to_string(),
            "AbCd".to_string(),
        ])
        .expect_err("missing output should fail");
        assert!(
            missing_output.contains("missing required --output"),
            "unexpected missing_output error: {missing_output}"
        );
    }

    #[test]
    fn resolve_api_token_prefers_cli_then_env() {
        assert_eq!(
            resolve_api_token(Some("cli"), Some("env")).expect("cli token should win"),
            "cli"
        );
        assert_eq!(
            resolve_api_token(None, Some("env")).expect("env token should be used"),
            "env"
        );
        assert!(
            resolve_api_token(None, None).is_err(),
            "missing both sources should fail"
        );
    }

    #[test]
    fn extract_export_nodes_selects_document_children() {
        let response = json!({
            "document": {
                "id": "0:0",
                "children": [
                    { "id": "1:0", "type": "CANVAS", "children": [] },
                    { "id": "2:0", "type": "CANVAS", "children": [] }
                ]
            }
        });
        let nodes = extract_export_nodes(&response, &[]).expect("should extract all pages");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0].get("id").and_then(JsonValue::as_str), Some("1:0"));
    }

    #[test]
    fn extract_export_nodes_filters_requested_pages() {
        let response = json!({
            "document": {
                "id": "0:0",
                "children": [
                    { "id": "1:0", "type": "CANVAS", "children": [] },
                    { "id": "2:0", "type": "CANVAS", "children": [] }
                ]
            }
        });
        let nodes = extract_export_nodes(&response, &["2:0".to_string()])
            .expect("should extract requested page");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0].get("id").and_then(JsonValue::as_str), Some("2:0"));
    }

    #[test]
    fn collect_image_refs_discovers_nested_image_paints() {
        let nodes = vec![json!({
            "id": "1:0",
            "fills": [
                { "type": "SOLID", "color": {"r":0.2,"g":0.2,"b":0.2} },
                { "type": "IMAGE", "imageRef": "abc123" }
            ],
            "children": [
                {
                    "id": "1:1",
                    "strokes": [
                        { "type": "IMAGE", "imageHash": "def456" }
                    ],
                    "children": []
                }
            ]
        })];

        let refs = collect_image_refs(&nodes);
        assert_eq!(refs, vec!["abc123".to_string(), "def456".to_string()]);
    }

    #[test]
    fn extract_image_url_map_reads_meta_images() {
        let response = json!({
            "meta": {
                "images": {
                    "abc": "https://figma-alpha-api.s3.amazonaws.com/images/abc.png",
                    "def": null
                }
            }
        });
        let map = extract_image_url_map(&response).expect("map should extract");
        assert_eq!(
            map.get("abc").and_then(|value| value.as_deref()),
            Some("https://figma-alpha-api.s3.amazonaws.com/images/abc.png")
        );
        assert_eq!(map.get("def"), Some(&None));
    }

    #[test]
    fn figma_image_reference_to_id_is_stable() {
        assert_eq!(figma_image_reference_to_id("42"), 42_u64);
        assert_eq!(
            figma_image_reference_to_id("abc"),
            figma_image_reference_to_id("abc")
        );
        assert_ne!(
            figma_image_reference_to_id("abc"),
            figma_image_reference_to_id("xyz")
        );
    }

    #[test]
    fn extension_guessing_works_for_url_and_content_type() {
        assert_eq!(
            extension_from_url("https://example.com/image.preview.webp?token=1"),
            Some("webp".to_string())
        );
        assert_eq!(
            extension_from_content_type(Some("image/jpeg")),
            Some("jpg".to_string())
        );
        assert_eq!(extension_from_content_type(Some("application/json")), None);
    }
}
