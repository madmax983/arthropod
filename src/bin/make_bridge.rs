use std::fs;
use std::path::PathBuf;

use arthropod::figma_codegen::{FigmaCodegenOptions, write_rust_module_from_json};
use arthropod::make_bridge::{
    MakeExtractionOptions, decode_make_archive_to_import_json, extract_make_archive,
    inspect_make_archive,
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    input: PathBuf,
    out_dir: PathBuf,
    overwrite: bool,
    write_manifest: bool,
    write_canonical_copy: bool,
    decoded_json_out: Option<PathBuf>,
    generated_rust_out: Option<PathBuf>,
    module_name: String,
    document_fn: String,
    runtime_fn: String,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            input: PathBuf::new(),
            out_dir: PathBuf::new(),
            overwrite: true,
            write_manifest: true,
            write_canonical_copy: true,
            decoded_json_out: None,
            generated_rust_out: None,
            module_name: "generated_figma".to_string(),
            document_fn: "document".to_string(),
            runtime_fn: "runtime".to_string(),
        }
    }
}

fn usage() -> &'static str {
    "Usage:
  cargo run --bin make_bridge -- \
    --input <design.make> \
    --out <extract-dir> \
    [--no-overwrite] \
    [--no-manifest] \
    [--no-canonical-copy] \
    [--decode-json-out <import.json>] \
    [--generate-rust-out <generated.rs>] \
    [--module-name <name>] \
    [--document-fn <name>] \
    [--runtime-fn <name>]

Examples:
  cargo run --bin make_bridge -- \
    --input \"C:\\Users\\markm\\Downloads\\Indie Artist Spotify App.make\" \
    --out artifacts/indie_make

  cargo run --bin make_bridge -- \
    --input \"C:\\Users\\markm\\Downloads\\Indie Artist Spotify App.make\" \
    --out artifacts/indie_make \
    --decode-json-out artifacts/indie_make/import_from_make.json \
    --generate-rust-out examples/generated/indie_make_generated.rs \
    --module-name indie_make_generated \
    --document-fn document \
    --runtime-fn runtime"
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
    let mut input = None;
    let mut out_dir = None;
    let mut index = 1usize;

    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                input = Some(take_value(&args, &mut index, "--input")?);
            }
            "--out" => {
                out_dir = Some(take_value(&args, &mut index, "--out")?);
            }
            "--no-overwrite" => {
                options.overwrite = false;
            }
            "--no-manifest" => {
                options.write_manifest = false;
            }
            "--no-canonical-copy" => {
                options.write_canonical_copy = false;
            }
            "--decode-json-out" => {
                options.decoded_json_out = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--decode-json-out",
                )?));
            }
            "--generate-rust-out" => {
                options.generated_rust_out = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--generate-rust-out",
                )?));
            }
            "--module-name" => {
                options.module_name = take_value(&args, &mut index, "--module-name")?;
            }
            "--document-fn" => {
                options.document_fn = take_value(&args, &mut index, "--document-fn")?;
            }
            "--runtime-fn" => {
                options.runtime_fn = take_value(&args, &mut index, "--runtime-fn")?;
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

    options.input =
        PathBuf::from(input.ok_or_else(|| format!("missing required --input\n\n{}", usage()))?);
    options.out_dir =
        PathBuf::from(out_dir.ok_or_else(|| format!("missing required --out\n\n{}", usage()))?);
    Ok(options)
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value for {flag}\n\n{}", usage()))
}

fn run(options: &CliOptions) -> Result<(), String> {
    let info = inspect_make_archive(&options.input)
        .map_err(|err| format!("failed to inspect {}: {err}", options.input.display()))?;
    println!("Archive: {}", info.path.display());
    println!("Entries: {}", info.entries.len());
    println!("Contains canvas.fig: {}", info.has_canvas_fig);
    println!("Contains meta.json: {}", info.has_meta_json);
    println!("Contains ai_chat.json: {}", info.has_ai_chat_json);
    println!(
        "Canonical JSON entry: {}",
        info.canonical_figma_json_entry
            .as_deref()
            .unwrap_or("<none>")
    );

    let extract_options = MakeExtractionOptions {
        overwrite: options.overwrite,
        write_manifest: options.write_manifest,
        write_canonical_copy: options.write_canonical_copy,
    };
    let report =
        extract_make_archive(&options.input, &options.out_dir, extract_options).map_err(|err| {
            format!(
                "failed to extract {} into {}: {err}",
                options.input.display(),
                options.out_dir.display()
            )
        })?;
    println!("Extracted files: {}", report.extracted_files.len());
    if !report.skipped_existing_files.is_empty() {
        println!(
            "Skipped existing files: {}",
            report.skipped_existing_files.len()
        );
    }
    if let Some(path) = &report.canvas_fig_path {
        println!("canvas.fig path: {}", path.display());
    }
    if let Some(path) = &report.canonical_figma_json_copy_path {
        println!("canonical import JSON path: {}", path.display());
    }
    if let Some(path) = &report.manifest_path {
        println!("manifest path: {}", path.display());
    }

    let mut decoded_json_cache = None;
    if options.decoded_json_out.is_some() || options.generated_rust_out.is_some() {
        let decoded = decode_make_archive_to_import_json(&options.input).map_err(|err| {
            format!(
                "failed to decode import JSON from {}: {err}",
                options.input.display()
            )
        })?;
        decoded_json_cache = Some(decoded);
    }

    if let Some(path) = &options.decoded_json_out {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)
                    .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
            }
        }
        fs::write(path, decoded_json_cache.as_deref().unwrap_or_default())
            .map_err(|err| format!("failed to write {}: {err}", path.display()))?;
        println!("decoded import JSON written: {}", path.display());
    }

    if let Some(path) = &options.generated_rust_out {
        let codegen_options = FigmaCodegenOptions {
            module_name: options.module_name.clone(),
            document_fn: options.document_fn.clone(),
            runtime_fn: options.runtime_fn.clone(),
        };
        write_rust_module_from_json(
            path,
            decoded_json_cache.as_deref().unwrap_or_default(),
            &codegen_options,
        )
        .map_err(|err| {
            format!(
                "failed to write generated Rust module {}: {err}",
                path.display()
            )
        })?;
        println!("generated Rust module written: {}", path.display());
    }

    Ok(())
}

fn main() {
    let result = parse_args(std::env::args()).and_then(|options| run(&options));
    if let Err(message) = result {
        eprintln!("{message}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_args_reads_required_flags() {
        let options = parse_args([
            "make_bridge".to_string(),
            "--input".to_string(),
            "design.make".to_string(),
            "--out".to_string(),
            "artifacts/design".to_string(),
        ])
        .expect("required args should parse");

        assert_eq!(options.input, PathBuf::from("design.make"));
        assert_eq!(options.out_dir, PathBuf::from("artifacts/design"));
        assert!(options.overwrite);
        assert!(options.write_manifest);
        assert!(options.write_canonical_copy);
    }

    #[test]
    fn parse_args_reads_optional_flags() {
        let options = parse_args([
            "make_bridge".to_string(),
            "--input".to_string(),
            "design.make".to_string(),
            "--out".to_string(),
            "artifacts/design".to_string(),
            "--no-overwrite".to_string(),
            "--no-manifest".to_string(),
            "--no-canonical-copy".to_string(),
            "--decode-json-out".to_string(),
            "artifacts/design/import.json".to_string(),
            "--generate-rust-out".to_string(),
            "examples/generated/design.rs".to_string(),
            "--module-name".to_string(),
            "design_generated".to_string(),
            "--document-fn".to_string(),
            "doc".to_string(),
            "--runtime-fn".to_string(),
            "run".to_string(),
        ])
        .expect("optional args should parse");

        assert!(!options.overwrite);
        assert!(!options.write_manifest);
        assert!(!options.write_canonical_copy);
        assert_eq!(
            options.decoded_json_out,
            Some(PathBuf::from("artifacts/design/import.json"))
        );
        assert_eq!(
            options.generated_rust_out,
            Some(PathBuf::from("examples/generated/design.rs"))
        );
        assert_eq!(options.module_name, "design_generated");
        assert_eq!(options.document_fn, "doc");
        assert_eq!(options.runtime_fn, "run");
    }

    #[test]
    fn parse_args_requires_input_and_out() {
        let err_missing_input = parse_args([
            "make_bridge".to_string(),
            "--out".to_string(),
            "artifacts/design".to_string(),
        ])
        .expect_err("missing --input should fail");
        assert!(err_missing_input.contains("missing required --input"));

        let err_missing_out = parse_args([
            "make_bridge".to_string(),
            "--input".to_string(),
            "design.make".to_string(),
        ])
        .expect_err("missing --out should fail");
        assert!(err_missing_out.contains("missing required --out"));
    }
}
