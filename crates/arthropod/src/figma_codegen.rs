use std::fs;
use std::path::Path;

use thiserror::Error;

use crate::figma::{FigmaImportError, import_figma_document};

/// Configuration options for the Figma Rust codegen process.
///
/// This dictates the names of the generated Rust module and the
/// functions exposed within it.
///
/// ## Examples
///
/// ```
/// use arthropod::figma_codegen::FigmaCodegenOptions;
///
/// let options = FigmaCodegenOptions {
///     module_name: "my_figma_module".to_string(),
///     document_fn: "load_figma_doc".to_string(),
///     runtime_fn: "create_figma_runtime".to_string(),
/// };
///
/// assert_eq!(options.module_name, "my_figma_module");
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FigmaCodegenOptions {
    /// The name of the generated Rust module (e.g. `"generated_figma"`).
    pub module_name: String,
    /// The name of the function that returns the [`ImportedFigmaDocument`].
    pub document_fn: String,
    /// The name of the function that returns a `FigmaRuntime` instance.
    pub runtime_fn: String,
}

impl Default for FigmaCodegenOptions {
    fn default() -> Self {
        Self {
            module_name: "generated_figma".to_string(),
            document_fn: "imported_document".to_string(),
            runtime_fn: "runtime".to_string(),
        }
    }
}

/// Errors that can occur during Figma code generation.
#[derive(Debug, Error)]
pub enum FigmaCodegenError {
    /// An error occurred while parsing the provided Figma JSON.
    #[error("failed to import figma json before codegen: {0}")]
    Import(#[from] FigmaImportError),
    /// An error occurred while writing the generated module file to disk.
    #[error("failed to write generated rust module: {0}")]
    Io(#[from] std::io::Error),
}

/// Generate a deterministic Rust module that embeds Figma JSON and provides
/// runtime/document entrypoints.
///
/// ## Examples
///
/// ```
/// use arthropod::figma_codegen::{FigmaCodegenOptions, generate_rust_module_from_json};
///
/// let json = r#"{ "document": { "id": "0:0", "type": "DOCUMENT" } }"#;
/// let options = FigmaCodegenOptions::default();
///
/// // In a real scenario, this requires a valid Figma JSON export format
/// // let code = generate_rust_module_from_json(json, &options).unwrap();
/// // assert!(code.contains("pub mod generated_figma"));
/// ```
pub fn generate_rust_module_from_json(
    figma_json: &str,
    options: &FigmaCodegenOptions,
) -> Result<String, FigmaCodegenError> {
    let normalized_json = normalize_line_endings(figma_json);
    let imported = import_figma_document(&normalized_json)?;
    let module_name = sanitize_identifier(&options.module_name, "generated_figma");
    let document_fn = sanitize_identifier(&options.document_fn, "imported_document");
    let runtime_fn = sanitize_identifier(&options.runtime_fn, "runtime");

    let source_bytes = normalized_json.len();
    let source_fnv64 = fnv1a64(normalized_json.as_bytes());
    let node_count = imported.figma_to_scene.len();
    let prototype_edge_count = imported.prototype_graph.edges.len();
    let raw_json = to_raw_string_literal(&normalized_json);

    Ok(format!(
        "pub mod {module_name} {{\n\
         \n\
         use arthropod::figma::{{FigmaImportError, ImportedFigmaDocument, import_figma_document}};\n\
         use arthropod::figma_runtime::{{FigmaRuntime, FigmaRuntimeError}};\n\
         \n\
         pub const SOURCE_BYTES: usize = {source_bytes};\n\
         pub const SOURCE_FNV64: u64 = 0x{source_fnv64:016x};\n\
         pub const NODE_COUNT: usize = {node_count};\n\
         pub const PROTOTYPE_EDGE_COUNT: usize = {prototype_edge_count};\n\
         \n\
         pub const FIGMA_JSON: &str = {raw_json};\n\
         \n\
         pub fn {document_fn}() -> Result<ImportedFigmaDocument, FigmaImportError> {{\n\
             import_figma_document(FIGMA_JSON)\n\
         }}\n\
         \n\
         pub fn {runtime_fn}() -> Result<FigmaRuntime, FigmaRuntimeError> {{\n\
             FigmaRuntime::from_figma_json(FIGMA_JSON)\n\
         }}\n\
         }}\n"
    ))
}

/// Generate and write a Rust module file to disk.
pub fn write_rust_module_from_json<P: AsRef<Path>>(
    path: P,
    figma_json: &str,
    options: &FigmaCodegenOptions,
) -> Result<(), FigmaCodegenError> {
    let code = generate_rust_module_from_json(figma_json, options)?;
    fs::write(path, code)?;
    Ok(())
}

fn sanitize_identifier(input: &str, fallback: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return fallback.to_string();
    }

    let mut out = String::new();
    for (index, ch) in trimmed.chars().enumerate() {
        let is_valid = ch == '_' || ch.is_ascii_alphanumeric();
        let ch = if is_valid { ch } else { '_' };

        if index == 0 && ch.is_ascii_digit() {
            out.push('_');
        }
        out.push(ch);
    }

    if out.is_empty() {
        fallback.to_string()
    } else {
        out
    }
}

fn normalize_line_endings(input: &str) -> String {
    if !input.contains('\r') {
        return input.to_string();
    }

    let mut normalized = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\r' {
            if chars.peek() == Some(&'\n') {
                chars.next();
            }
            normalized.push('\n');
        } else {
            normalized.push(ch);
        }
    }
    normalized
}

fn to_raw_string_literal(value: &str) -> String {
    for hashes in 0..16 {
        let fence = "#".repeat(hashes);
        let terminator = format!("\"{fence}");
        if !value.contains(&terminator) {
            return format!("r{fence}\"{value}\"{fence}");
        }
    }

    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\r', "\\r")
        .replace('\n', "\\n");
    format!("\"{escaped}\"")
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_identifier_rewrites_invalid_symbols() {
        assert_eq!(
            sanitize_identifier("hero-screen", "fallback"),
            "hero_screen"
        );
        assert_eq!(sanitize_identifier("123scene", "fallback"), "_123scene");
        assert_eq!(sanitize_identifier("   ", "fallback"), "fallback");
    }

    #[test]
    fn raw_string_literal_falls_back_to_escaped_if_needed() {
        let text = "abc";
        let raw = to_raw_string_literal(text);
        assert!(raw.starts_with("r"));

        let pathological = (0..16)
            .map(|count| format!("\"{}", "#".repeat(count)))
            .collect::<Vec<_>>()
            .join("|");
        let fallback = to_raw_string_literal(&pathological);
        assert!(
            fallback.starts_with('"'),
            "pathological payload should still produce valid rust string literal"
        );
    }

    #[test]
    fn normalize_line_endings_collapses_crlf_and_cr() {
        assert_eq!(normalize_line_endings("a\r\nb\rc\n"), "a\nb\nc\n");
        assert_eq!(normalize_line_endings("a\nb\n"), "a\nb\n");
    }
}
