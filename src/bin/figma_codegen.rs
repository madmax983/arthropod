#![allow(missing_docs)]
//! Utility script to generate Rust code from pulled Figma design files.

use std::fs;
use std::path::PathBuf;

use arthropod::figma_codegen::{FigmaCodegenOptions, write_rust_module_from_json};

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliOptions {
    input: PathBuf,
    output: PathBuf,
    module_name: String,
    document_fn: String,
    runtime_fn: String,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            input: PathBuf::new(),
            output: PathBuf::new(),
            module_name: "generated_figma".to_string(),
            document_fn: "document".to_string(),
            runtime_fn: "runtime".to_string(),
        }
    }
}

fn usage() -> &'static str {
    "Usage:
  cargo run --bin figma_codegen -- \\
    --input <figma.json> \\
    --output <generated.rs> \\
    [--module-name <name>] \\
    [--document-fn <name>] \\
    [--runtime-fn <name>]

Example:
  cargo run --bin figma_codegen -- \
    --input riot_waves.json \
    --output examples/generated/riot_waves_generated_module.rs \
    --module-name riot_waves_generated \
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
    let mut output = None;

    let mut index = 1usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input" => {
                input = Some(take_value(&args, &mut index, "--input")?);
            }
            "--output" => {
                output = Some(take_value(&args, &mut index, "--output")?);
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

    let input = input.ok_or_else(|| format!("missing required --input\n\n{}", usage()))?;
    let output = output.ok_or_else(|| format!("missing required --output\n\n{}", usage()))?;
    options.input = PathBuf::from(input);
    options.output = PathBuf::from(output);
    Ok(options)
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value for {flag}\n\n{}", usage()))
}

fn run_with_options(options: &CliOptions) -> Result<(), String> {
    let figma_json = fs::read_to_string(&options.input)
        .map_err(|err| format!("failed to read {}: {err}", options.input.display()))?;

    if let Some(parent) = options
        .output
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }

    let codegen_options = FigmaCodegenOptions {
        module_name: options.module_name.clone(),
        document_fn: options.document_fn.clone(),
        runtime_fn: options.runtime_fn.clone(),
    };

    write_rust_module_from_json(&options.output, &figma_json, &codegen_options).map_err(|err| {
        format!(
            "failed to generate module {} from {}: {err}",
            options.output.display(),
            options.input.display()
        )
    })?;

    println!(
        "Generated {} from {}",
        options.output.display(),
        options.input.display()
    );
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
    fn parse_args_reads_required_values_and_defaults() {
        let options = parse_args([
            "figma_codegen".to_string(),
            "--input".to_string(),
            "riot_waves.json".to_string(),
            "--output".to_string(),
            "examples/generated/riot.rs".to_string(),
        ])
        .expect("required arguments should parse");

        assert_eq!(options.input, PathBuf::from("riot_waves.json"));
        assert_eq!(options.output, PathBuf::from("examples/generated/riot.rs"));
        assert_eq!(options.module_name, "generated_figma");
        assert_eq!(options.document_fn, "document");
        assert_eq!(options.runtime_fn, "runtime");
    }

    #[test]
    fn parse_args_applies_overrides() {
        let options = parse_args([
            "figma_codegen".to_string(),
            "--input".to_string(),
            "riot_waves.json".to_string(),
            "--output".to_string(),
            "examples/generated/riot.rs".to_string(),
            "--module-name".to_string(),
            "riot_waves_generated".to_string(),
            "--document-fn".to_string(),
            "doc".to_string(),
            "--runtime-fn".to_string(),
            "run".to_string(),
        ])
        .expect("override arguments should parse");

        assert_eq!(options.module_name, "riot_waves_generated");
        assert_eq!(options.document_fn, "doc");
        assert_eq!(options.runtime_fn, "run");
    }

    #[test]
    fn parse_args_requires_input_and_output() {
        let missing_output = parse_args([
            "figma_codegen".to_string(),
            "--input".to_string(),
            "riot_waves.json".to_string(),
        ])
        .expect_err("missing output should fail");
        assert!(
            missing_output.contains("missing required --output"),
            "unexpected error message: {missing_output}"
        );

        let missing_input = parse_args([
            "figma_codegen".to_string(),
            "--output".to_string(),
            "examples/generated/riot.rs".to_string(),
        ])
        .expect_err("missing input should fail");
        assert!(
            missing_input.contains("missing required --input"),
            "unexpected error message: {missing_input}"
        );
    }
}
