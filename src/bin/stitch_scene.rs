use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use arthropod::figma::import_figma_document;
use arthropod::figma_codegen::{FigmaCodegenOptions, write_rust_module_from_json};
use scraper::{ElementRef, Html, Node, Selector};
use serde_json::{Map as JsonMap, Number as JsonNumber, Value as JsonValue, json};

#[derive(Debug, Clone, PartialEq)]
struct CliOptions {
    input_html: PathBuf,
    output_json: PathBuf,
    viewport_width: f32,
    viewport_height: Option<f32>,
    module_out: Option<PathBuf>,
    module_name: String,
    document_fn: String,
    runtime_fn: String,
}

impl Default for CliOptions {
    fn default() -> Self {
        Self {
            input_html: PathBuf::new(),
            output_json: PathBuf::new(),
            viewport_width: 1280.0,
            viewport_height: None,
            module_out: None,
            module_name: "generated_stitch".to_string(),
            document_fn: "document".to_string(),
            runtime_fn: "runtime".to_string(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Size {
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct Edges {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

impl Default for Edges {
    fn default() -> Self {
        Self {
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            left: 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowDirection {
    Row,
    Column,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PositionMode {
    Normal,
    Absolute,
    Fixed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    X,
    Y,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParentLayoutContext {
    is_flex: bool,
    direction: FlowDirection,
    cross_axis_stretches: bool,
}

#[derive(Debug, Clone)]
struct HtmlNode {
    tag: String,
    classes: Vec<String>,
    styles: HashMap<String, String>,
    text: String,
    children: Vec<HtmlNode>,
}

#[derive(Debug, Clone)]
struct TailwindTheme {
    colors: HashMap<String, [f32; 4]>,
    fonts: HashMap<String, String>,
    background_images: HashMap<String, String>,
}

impl TailwindTheme {
    fn from_html_source(html: &str) -> Self {
        let mut theme = Self::defaults();
        let Some(script) = extract_tailwind_config_script(html) else {
            return theme;
        };

        if let Some(colors_block) = extract_js_object_block(&script, "colors") {
            for (key, value) in parse_js_object_pairs(colors_block) {
                if let Some(color) = parse_color_literal(&value) {
                    theme.colors.insert(normalize_color_key(&key), color);
                }
            }
        }

        if let Some(fonts_block) = extract_js_object_block(&script, "fontFamily") {
            for (key, value) in parse_js_object_pairs(fonts_block) {
                if let Some(font) = parse_first_quoted_token(&value) {
                    theme.fonts.insert(key, font);
                }
            }
        }

        if let Some(images_block) = extract_js_object_block(&script, "backgroundImage") {
            for (key, value) in parse_js_object_pairs(images_block) {
                let key = key.trim().to_ascii_lowercase();
                let value = value.trim().to_string();
                if !key.is_empty() && !value.is_empty() {
                    theme.background_images.insert(key, value);
                }
            }
        }

        theme
    }

    fn defaults() -> Self {
        let mut colors = HashMap::new();
        colors.insert("black".to_string(), [0.0, 0.0, 0.0, 1.0]);
        colors.insert("white".to_string(), [1.0, 1.0, 1.0, 1.0]);
        colors.insert("transparent".to_string(), [0.0, 0.0, 0.0, 0.0]);
        colors.insert(
            "gray-900".to_string(),
            hex_to_rgba("#111827").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "gray-700".to_string(),
            hex_to_rgba("#374151").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "gray-600".to_string(),
            hex_to_rgba("#4B5563").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "gray-500".to_string(),
            hex_to_rgba("#6B7280").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "gray-300".to_string(),
            hex_to_rgba("#D1D5DB").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "gray-200".to_string(),
            hex_to_rgba("#E5E7EB").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "background-dark".to_string(),
            hex_to_rgba("#050505").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "background-light".to_string(),
            hex_to_rgba("#1a1a1a").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "ink".to_string(),
            hex_to_rgba("#f2f2f2").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "paper".to_string(),
            hex_to_rgba("#121212").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "primary".to_string(),
            hex_to_rgba("#ccff00").unwrap_or([0.0; 4]),
        );
        colors.insert(
            "neon-pink".to_string(),
            hex_to_rgba("#ff00ff").unwrap_or([0.0; 4]),
        );

        let mut fonts = HashMap::new();
        fonts.insert("display".to_string(), "Space Grotesk".to_string());
        fonts.insert("ransom".to_string(), "Sedgwick Ave Display".to_string());
        fonts.insert("typewriter".to_string(), "Special Elite".to_string());
        fonts.insert("hand".to_string(), "Permanent Marker".to_string());
        fonts.insert("impact".to_string(), "Fugaz One".to_string());

        let background_images = HashMap::new();

        Self {
            colors,
            fonts,
            background_images,
        }
    }
}

#[derive(Debug, Clone, Default)]
struct TextStyle {
    color: Option<[f32; 4]>,
    font_family: Option<String>,
    font_size: Option<f32>,
    font_weight: Option<u16>,
    font_style: Option<String>,
    text_align: Option<String>,
    text_case: Option<String>,
    line_height_percent: Option<f32>,
    letter_spacing: Option<f32>,
}

type ClassStyleMap = HashMap<String, HashMap<String, String>>;

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct ImageFilterSpec {
    grayscale: Option<f32>,
    contrast: Option<f32>,
    invert: Option<f32>,
}

#[derive(Debug, Clone)]
struct LayoutChild {
    index: usize,
    size: Size,
    margin: Edges,
    position_mode: PositionMode,
    z_index: i32,
    flex_grow: f32,
    flex_basis_zero: bool,
}

fn usage() -> &'static str {
    "Usage:
  cargo run --bin stitch_scene -- \\
    --input-html <code.normalized.html> \\
    --output-json <scene.json> \\
    [--viewport-width <px>] \\
    [--viewport-height <px>] \\
    [--module-out <generated.rs>] \\
    [--module-name <name>] \\
    [--document-fn <name>] \\
    [--runtime-fn <name>]

What it does:
  - Parses Stitch-export HTML into a deterministic Figma-compatible scene JSON
  - Maps layout/text/background/image classes into Arthropod import schema
  - Emits { \"nodes\": [...] } consumable by figma_codegen / figma runtime
  - Optionally emits a generated Rust module in one command via --module-out"
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
    let mut input_html = None;
    let mut output_json = None;
    let mut index = 1usize;
    while index < args.len() {
        match args[index].as_str() {
            "--input-html" => {
                input_html = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--input-html",
                )?));
            }
            "--output-json" => {
                output_json = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--output-json",
                )?));
            }
            "--viewport-width" => {
                let raw = take_value(&args, &mut index, "--viewport-width")?;
                options.viewport_width = raw.parse::<f32>().map_err(|err| {
                    format!(
                        "invalid value for --viewport-width `{raw}`: {err}\n\n{}",
                        usage()
                    )
                })?;
            }
            "--viewport-height" => {
                let raw = take_value(&args, &mut index, "--viewport-height")?;
                options.viewport_height = Some(raw.parse::<f32>().map_err(|err| {
                    format!(
                        "invalid value for --viewport-height `{raw}`: {err}\n\n{}",
                        usage()
                    )
                })?);
            }
            "--module-out" => {
                options.module_out = Some(PathBuf::from(take_value(
                    &args,
                    &mut index,
                    "--module-out",
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
            "--help" | "-h" => return Err(usage().to_string()),
            other => return Err(format!("unknown argument: {other}\n\n{}", usage())),
        }
        index += 1;
    }

    options.input_html =
        input_html.ok_or_else(|| format!("missing required --input-html\n\n{}", usage()))?;
    options.output_json =
        output_json.ok_or_else(|| format!("missing required --output-json\n\n{}", usage()))?;
    if !options.viewport_width.is_finite() || options.viewport_width <= 0.0 {
        return Err("viewport width must be a positive finite number".to_string());
    }
    if let Some(height) = options.viewport_height
        && (!height.is_finite() || height <= 0.0)
    {
        return Err("viewport height must be a positive finite number".to_string());
    }
    Ok(options)
}

fn take_value(args: &[String], index: &mut usize, flag: &str) -> Result<String, String> {
    *index += 1;
    args.get(*index)
        .cloned()
        .ok_or_else(|| format!("missing value for {flag}\n\n{}", usage()))
}

fn run_with_options(options: &CliOptions) -> Result<(), String> {
    let html_source = fs::read_to_string(&options.input_html)
        .map_err(|err| format!("failed to read {}: {err}", options.input_html.display()))?;
    let viewport_height = options
        .viewport_height
        .or_else(|| infer_body_min_height(&html_source))
        .unwrap_or(884.0);
    let viewport = Size {
        width: options.viewport_width,
        height: viewport_height,
    };

    let theme = TailwindTheme::from_html_source(&html_source);
    let class_styles = resolve_class_style_rules(&html_source, &options.input_html);
    let dom = Html::parse_document(&html_source);
    let selector =
        Selector::parse("body").map_err(|err| format!("failed to parse body selector: {err}"))?;
    let body = dom
        .select(&selector)
        .next()
        .ok_or_else(|| "html body element not found".to_string())?;
    let root = build_html_node(body, &class_styles)?;

    let mut size_cache = HashMap::new();
    let mut root_path = vec![0usize];
    measure_tree(
        &root,
        &mut root_path,
        viewport,
        viewport,
        &mut size_cache,
        None,
        &theme,
        &TextStyle::default(),
    );

    let root_rect = Rect {
        x: 0.0,
        y: 0.0,
        width: viewport.width,
        height: viewport.height,
    };
    let mut nodes = Vec::new();
    let inherited_text = TextStyle::default();
    let mut root_layout_path = vec![0usize];
    layout_tree(
        &root,
        &mut root_layout_path,
        None,
        root_rect,
        viewport,
        &size_cache,
        &theme,
        &inherited_text,
        &mut nodes,
    );

    let document = json!({ "nodes": nodes });
    let serialized = serde_json::to_string_pretty(&document)
        .map_err(|err| format!("failed to serialize scene json: {err}"))?;
    if let Some(parent) = options.output_json.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
    }
    fs::write(&options.output_json, serialized)
        .map_err(|err| format!("failed to write {}: {err}", options.output_json.display()))?;

    let verify_source = fs::read_to_string(&options.output_json)
        .map_err(|err| format!("failed to re-read {}: {err}", options.output_json.display()))?;
    let imported = import_figma_document(&verify_source)
        .map_err(|err| format!("generated scene json failed importer verification: {err}"))?;

    if let Some(module_out) = options.module_out.as_ref() {
        if let Some(parent) = module_out.parent()
            && !parent.as_os_str().is_empty()
        {
            fs::create_dir_all(parent)
                .map_err(|err| format!("failed to create {}: {err}", parent.display()))?;
        }

        let codegen_options = FigmaCodegenOptions {
            module_name: options.module_name.clone(),
            document_fn: options.document_fn.clone(),
            runtime_fn: options.runtime_fn.clone(),
        };
        write_rust_module_from_json(module_out, &verify_source, &codegen_options).map_err(
            |err| {
                format!(
                    "failed to generate rust module {} from {}: {err}",
                    module_out.display(),
                    options.output_json.display()
                )
            },
        )?;
    }

    println!("Input html: {}", options.input_html.display());
    println!("Output json: {}", options.output_json.display());
    println!(
        "Viewport: {:.0}x{:.0}",
        viewport.width.round(),
        viewport.height.round()
    );
    println!("Generated nodes: {}", imported.figma_to_scene.len());
    println!("Prototype edges: {}", imported.prototype_graph.edges.len());
    if let Some(module_out) = options.module_out.as_ref() {
        println!("Generated module: {}", module_out.display());
    }

    Ok(())
}
fn infer_body_min_height(source: &str) -> Option<f32> {
    let marker = "min-height: max(";
    let start = source.find(marker)?;
    let tail = &source[start + marker.len()..];
    let px_end = tail.find("px")?;
    tail[..px_end].trim().parse::<f32>().ok()
}

fn build_html_node(
    element: ElementRef<'_>,
    class_styles: &ClassStyleMap,
) -> Result<HtmlNode, String> {
    let tag = element.value().name().to_ascii_lowercase();
    let classes = element
        .value()
        .attr("class")
        .map(split_classes)
        .unwrap_or_default();
    let mut styles = merged_class_styles(&classes, class_styles);
    let inline_styles = element
        .value()
        .attr("style")
        .map(parse_inline_styles)
        .unwrap_or_default();
    styles.extend(inline_styles);
    let text = collect_text_with_line_breaks(element);

    let mut children = Vec::new();
    for child in element.children() {
        let Some(child_element) = ElementRef::wrap(child) else {
            continue;
        };
        let name = child_element.value().name();
        if is_ignored_tag(name) {
            continue;
        }
        if element_is_hidden_by_default(child_element, class_styles) {
            continue;
        }
        children.push(build_html_node(child_element, class_styles)?);
    }
    if let Some(pseudo_after) = synthetic_pseudo_after_node(&classes, &styles) {
        children.push(pseudo_after);
    }

    Ok(HtmlNode {
        tag,
        classes,
        styles,
        text,
        children,
    })
}

fn element_is_hidden_by_default(element: ElementRef<'_>, class_styles: &ClassStyleMap) -> bool {
    let classes = element
        .value()
        .attr("class")
        .map(split_classes)
        .unwrap_or_default();
    let mut styles = merged_class_styles(&classes, class_styles);
    let inline_styles = element
        .value()
        .attr("style")
        .map(parse_inline_styles)
        .unwrap_or_default();
    styles.extend(inline_styles);
    is_hidden_by_default(&classes, &styles)
}

fn is_hidden_by_default(classes: &[String], styles: &HashMap<String, String>) -> bool {
    if has_class(classes, "hidden") {
        return true;
    }
    styles.get("display").is_some_and(|value| {
        value.trim().eq_ignore_ascii_case("none")
            || value.trim().eq_ignore_ascii_case("contents none")
    })
}

fn synthetic_pseudo_after_node(
    classes: &[String],
    parent_styles: &HashMap<String, String>,
) -> Option<HtmlNode> {
    if !has_class(classes, "halftone") {
        return None;
    }
    // When the halftone node has an image background we fold the pseudo-layer into
    // a composed source image reference (base + dots) so filter chains are applied once.
    if parent_styles.contains_key("background-image") {
        return None;
    }

    let pattern_hash = stable_hash64("halftone::after");
    let mut styles = HashMap::new();
    styles.insert(
        "background-image".to_string(),
        format!("url('procedural://halftone/{pattern_hash:016x}')"),
    );
    styles.insert("mix-blend-mode".to_string(), "overlay".to_string());
    styles.insert("opacity".to_string(), "0.15".to_string());

    Some(HtmlNode {
        tag: "div".to_string(),
        classes: vec![
            "absolute".to_string(),
            "inset-0".to_string(),
            "pointer-events-none".to_string(),
        ],
        styles,
        text: String::new(),
        children: Vec::new(),
    })
}

fn is_ignored_tag(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "script" | "style" | "noscript" | "meta" | "link" | "title" | "head" | "br"
    )
}

fn collect_text_with_line_breaks(element: ElementRef<'_>) -> String {
    let mut output = String::new();
    for child in element.children() {
        match child.value() {
            Node::Text(text) => output.push_str(text.text.as_ref()),
            Node::Element(el) if el.name() == "br" => output.push('\n'),
            _ => {}
        }
    }
    normalize_text(&output)
}

fn normalize_text(input: &str) -> String {
    let mut lines = Vec::new();
    for raw in input.replace('\r', "").split('\n') {
        let mut collapsed = String::new();
        let mut previous_space = false;
        for ch in raw.chars() {
            if ch.is_whitespace() {
                if !previous_space {
                    collapsed.push(' ');
                }
                previous_space = true;
            } else {
                collapsed.push(ch);
                previous_space = false;
            }
        }
        let trimmed = collapsed.trim();
        if !trimmed.is_empty() {
            lines.push(trimmed.to_string());
        }
    }
    lines.join("\n")
}

fn split_classes(raw: &str) -> Vec<String> {
    raw.split_whitespace().map(ToString::to_string).collect()
}

fn parse_inline_styles(raw: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for segment in raw.split(';') {
        let Some((key, value)) = segment.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim().to_string();
        if !key.is_empty() && !value.is_empty() {
            map.insert(key, value);
        }
    }
    map
}

fn merged_class_styles(
    classes: &[String],
    class_styles: &ClassStyleMap,
) -> HashMap<String, String> {
    let mut merged = HashMap::new();
    for class_name in classes {
        if let Some(styles) = class_styles.get(class_name) {
            for (key, value) in styles {
                merged.insert(key.clone(), value.clone());
            }
        }
    }
    merged
}

fn resolve_class_style_rules(html_source: &str, input_html_path: &Path) -> ClassStyleMap {
    let mut rules = parse_class_style_rules(html_source);
    let Some(html_parent) = input_html_path.parent() else {
        return rules;
    };
    for href in discover_stylesheet_hrefs(html_source) {
        let Some(local_path) = local_stylesheet_path(html_parent, &href) else {
            continue;
        };
        let Ok(css_source) = fs::read_to_string(&local_path) else {
            continue;
        };
        let css_rules = parse_css_class_rules(&css_source);
        merge_class_style_map(&mut rules, css_rules);
    }
    rules
}

fn parse_class_style_rules(source: &str) -> ClassStyleMap {
    let mut map: ClassStyleMap = HashMap::new();
    for block in extract_style_blocks(source) {
        merge_class_style_map(&mut map, parse_css_class_rules(block));
    }
    map
}

fn parse_css_class_rules(source: &str) -> ClassStyleMap {
    let mut map: ClassStyleMap = HashMap::new();
    for rule in source.split('}') {
        let Some((selectors, declarations)) = rule.split_once('{') else {
            continue;
        };
        let declarations = parse_inline_styles(declarations);
        if declarations.is_empty() {
            continue;
        }

        for selector in selectors.split(',') {
            let selector = selector.trim();
            let Some(class_name) = class_selector_name(selector) else {
                continue;
            };
            let entry = map.entry(class_name.to_string()).or_default();
            for (key, value) in &declarations {
                entry.insert(key.clone(), value.clone());
            }
        }
    }
    map
}

fn merge_class_style_map(base: &mut ClassStyleMap, incoming: ClassStyleMap) {
    for (class_name, styles) in incoming {
        let entry = base.entry(class_name).or_default();
        for (key, value) in styles {
            entry.insert(key, value);
        }
    }
}

fn discover_stylesheet_hrefs(source: &str) -> Vec<String> {
    let mut hrefs = Vec::new();
    let mut seen = std::collections::HashSet::new();
    let dom = Html::parse_document(source);
    let Ok(selector) = Selector::parse("link[href]") else {
        return hrefs;
    };
    for element in dom.select(&selector) {
        let is_stylesheet = element
            .value()
            .attr("rel")
            .map(|rel| {
                rel.split_whitespace()
                    .any(|token| token.eq_ignore_ascii_case("stylesheet"))
            })
            .unwrap_or(false);
        if !is_stylesheet {
            continue;
        }
        let Some(href) = element.value().attr("href") else {
            continue;
        };
        let href = href.trim();
        if href.is_empty() || !seen.insert(href.to_string()) {
            continue;
        }
        hrefs.push(href.to_string());
    }
    hrefs
}

fn local_stylesheet_path(base_dir: &Path, href: &str) -> Option<PathBuf> {
    if href.starts_with("http://")
        || href.starts_with("https://")
        || href.starts_with("data:")
        || href.starts_with("//")
    {
        return None;
    }
    let without_fragment = href.split('#').next().unwrap_or(href);
    let without_query = without_fragment
        .split('?')
        .next()
        .unwrap_or(without_fragment);
    if without_query.trim().is_empty() {
        return None;
    }
    Some(base_dir.join(without_query))
}

fn extract_style_blocks(source: &str) -> Vec<&str> {
    let mut blocks = Vec::new();
    let mut cursor = source;
    loop {
        let Some(style_start) = cursor.find("<style") else {
            break;
        };
        let style_tail = &cursor[style_start..];
        let Some(open_end) = style_tail.find('>') else {
            break;
        };
        let content_tail = &style_tail[open_end + 1..];
        let Some(close_start) = content_tail.find("</style>") else {
            break;
        };
        blocks.push(&content_tail[..close_start]);
        cursor = &content_tail[close_start + "</style>".len()..];
    }
    blocks
}

fn class_selector_name(selector: &str) -> Option<&str> {
    let selector = selector.trim();
    if !selector.starts_with('.') {
        return None;
    }
    let base = &selector[1..];
    if base.is_empty() {
        return None;
    }
    if base
        .chars()
        .any(|ch| matches!(ch, ' ' | '>' | '+' | '~' | '[' | ']' | '#' | ':' | '.'))
    {
        return None;
    }
    Some(base)
}

fn measure_tree(
    node: &HtmlNode,
    path: &mut Vec<usize>,
    parent_size: Size,
    viewport: Size,
    cache: &mut HashMap<String, Size>,
    parent_layout: Option<ParentLayoutContext>,
    theme: &TailwindTheme,
    inherited_text: &TextStyle,
) -> Size {
    let key = path_to_key(path);
    if let Some(existing) = cache.get(&key).copied() {
        return existing;
    }

    let padding = parse_padding(&node.classes, parent_size, viewport);
    let explicit_width = resolve_dimension(node, Axis::X, parent_size, viewport);
    let explicit_height = resolve_dimension(node, Axis::Y, parent_size, viewport);
    let self_position = position_mode(&node.classes);
    let positioned_parent_size = if self_position == PositionMode::Fixed {
        viewport
    } else {
        parent_size
    };
    let stretched_width = if self_position != PositionMode::Normal {
        resolve_positioned_stretch_size(node, Axis::X, positioned_parent_size, viewport)
    } else {
        None
    };
    let stretched_height = if self_position != PositionMode::Normal {
        resolve_positioned_stretch_size(node, Axis::Y, positioned_parent_size, viewport)
    } else {
        None
    };

    let resolved_text = resolve_text_style(node, theme, inherited_text);
    let is_text = node_is_text_node(node);
    let font_size = resolved_text.font_size.unwrap_or(16.0);
    let line_height_factor = resolved_text
        .line_height_percent
        .map(|percent| (percent / 100.0).max(0.1))
        .unwrap_or_else(|| resolve_line_height_factor(&node.classes, &node.styles));
    let is_ligature_icon_font = resolved_text
        .font_family
        .as_deref()
        .is_some_and(is_ligature_icon_font_family);
    let estimated_text_size = estimate_text_size(
        &node.text,
        font_size,
        line_height_factor,
        is_ligature_icon_font,
        resolved_text.letter_spacing.unwrap_or(0.0),
    );
    let fills_available_width = node_fills_available_width(node, parent_layout);

    let mut width = explicit_width.or(stretched_width).unwrap_or_else(|| {
        if is_text {
            estimated_text_size.width + padding.left + padding.right
        } else if self_position != PositionMode::Normal {
            0.0
        } else if fills_available_width {
            parent_size.width
        } else {
            0.0
        }
    });
    if !width.is_finite() || width <= 0.0 {
        width = if self_position == PositionMode::Normal && fills_available_width {
            parent_size.width.max(1.0)
        } else {
            1.0
        };
    }

    let provisional_height = explicit_height
        .or(stretched_height)
        .unwrap_or(parent_size.height)
        .max(0.0);
    let inner_width = (width - padding.left - padding.right).max(0.0);
    let inner_height = (provisional_height - padding.top - padding.bottom).max(0.0);
    let inner_parent = Size {
        width: inner_width,
        height: inner_height,
    };

    let direction = flow_direction(&node.classes);
    let gap = parse_gap(&node.classes, parent_size, viewport).unwrap_or(0.0);

    let mut flow_main = 0.0f32;
    let mut flow_cross = 0.0f32;
    let mut visible_flow_children = 0usize;

    let current_layout = parent_layout_context_for(node);
    for (index, child) in node.children.iter().enumerate() {
        path.push(index);
        let child_size = measure_tree(
            child,
            path,
            inner_parent,
            viewport,
            cache,
            Some(current_layout),
            theme,
            &resolved_text,
        );
        path.pop();

        if position_mode(&child.classes) != PositionMode::Normal {
            continue;
        }

        let margin = parse_margin(&child.classes, inner_parent, viewport);
        let child_main_size = match direction {
            FlowDirection::Column => {
                if resolve_flex_basis_zero(&child.classes, &child.styles) {
                    0.0
                } else {
                    child_size.height
                }
            }
            FlowDirection::Row => {
                if resolve_flex_basis_zero(&child.classes, &child.styles) {
                    0.0
                } else {
                    child_size.width
                }
            }
        };
        match direction {
            FlowDirection::Column => {
                flow_main += margin.top + child_main_size + margin.bottom;
                flow_cross = flow_cross.max(margin.left + child_size.width + margin.right);
            }
            FlowDirection::Row => {
                flow_main += margin.left + child_main_size + margin.right;
                flow_cross = flow_cross.max(margin.top + child_size.height + margin.bottom);
            }
        }
        visible_flow_children += 1;
    }

    if visible_flow_children > 1 {
        flow_main += gap * (visible_flow_children as f32 - 1.0);
    }

    let mut height = explicit_height.or(stretched_height).unwrap_or(0.0);
    if height <= 0.0 {
        if is_text {
            height = estimated_text_size.height + padding.top + padding.bottom;
        } else if !node.children.is_empty() {
            height = match direction {
                FlowDirection::Column => flow_main + padding.top + padding.bottom,
                FlowDirection::Row => flow_cross + padding.top + padding.bottom,
            };
        } else {
            height = 40.0 + padding.top + padding.bottom;
        }
    }

    if explicit_width.is_none() && !is_text {
        if !node.children.is_empty() {
            width = match direction {
                FlowDirection::Column => {
                    let content_width = flow_cross + padding.left + padding.right;
                    if self_position == PositionMode::Normal {
                        if fills_available_width {
                            width.max(content_width)
                        } else {
                            content_width.max(1.0)
                        }
                    } else {
                        content_width.max(1.0)
                    }
                }
                FlowDirection::Row => {
                    let content_width = flow_main + padding.left + padding.right;
                    if self_position == PositionMode::Normal {
                        if fills_available_width {
                            width.max(content_width)
                        } else {
                            content_width.max(1.0)
                        }
                    } else {
                        content_width.max(1.0)
                    }
                }
            };
        } else if self_position != PositionMode::Normal {
            width = (40.0 + padding.left + padding.right).max(1.0);
        } else if !fills_available_width {
            width = 1.0;
        }
    }

    if !height.is_finite() || height <= 0.0 {
        height = 1.0;
    }
    if !width.is_finite() || width <= 0.0 {
        width = 1.0;
    }

    let measured = Size { width, height };
    cache.insert(key, measured);
    measured
}

#[allow(clippy::too_many_arguments)]
fn layout_tree(
    node: &HtmlNode,
    path: &mut Vec<usize>,
    parent_id: Option<String>,
    rect: Rect,
    viewport: Size,
    size_cache: &HashMap<String, Size>,
    theme: &TailwindTheme,
    inherited_text: &TextStyle,
    out_nodes: &mut Vec<JsonValue>,
) {
    layout_tree_with_effect_inheritance(
        node,
        path,
        parent_id,
        rect,
        viewport,
        size_cache,
        theme,
        inherited_text,
        &[],
        None,
        out_nodes,
    );
}

#[allow(clippy::too_many_arguments)]
fn layout_tree_with_effect_inheritance(
    node: &HtmlNode,
    path: &mut Vec<usize>,
    parent_id: Option<String>,
    rect: Rect,
    viewport: Size,
    size_cache: &HashMap<String, Size>,
    theme: &TailwindTheme,
    inherited_text: &TextStyle,
    inherited_effects: &[JsonValue],
    inherited_blend_mode: Option<String>,
    out_nodes: &mut Vec<JsonValue>,
) {
    let rect = apply_translation_transform(rect, node, viewport);
    let node_id = format!("stitch:{}", path_to_key(path));
    let resolved_text = resolve_text_style(node, theme, inherited_text);
    let node_output = node_to_figma_json_with_inherited_effects(
        node,
        &node_id,
        parent_id.as_deref(),
        rect,
        theme,
        viewport,
        &resolved_text,
        inherited_effects,
        inherited_blend_mode.as_deref(),
    );
    out_nodes.push(node_output.json);

    let child_inherited_effects =
        next_inherited_effects(node, theme, &resolved_text, inherited_effects);
    let child_inherited_blend_mode = node_output.child_inherited_blend_mode;

    if node.children.is_empty() {
        return;
    }

    let padding = parse_padding(&node.classes, size_from_rect(rect), viewport);
    let content = Rect {
        x: rect.x + padding.left,
        y: rect.y + padding.top,
        width: (rect.width - padding.left - padding.right).max(0.0),
        height: (rect.height - padding.top - padding.bottom).max(0.0),
    };

    let direction = flow_direction(&node.classes);
    let gap = parse_gap(&node.classes, size_from_rect(rect), viewport).unwrap_or(0.0);
    let justify = parse_justify_content(&node.classes);
    let align = parse_align_items(&node.classes);

    let mut children_meta = Vec::with_capacity(node.children.len());
    for (index, child) in node.children.iter().enumerate() {
        let mut child_path = path.clone();
        child_path.push(index);
        let child_key = path_to_key(&child_path);
        let size = size_cache.get(&child_key).copied().unwrap_or(Size {
            width: 1.0,
            height: 1.0,
        });
        children_meta.push(LayoutChild {
            index,
            size,
            margin: parse_margin(&child.classes, size_from_rect(content), viewport),
            position_mode: position_mode(&child.classes),
            z_index: resolve_z_index(&child.classes, &child.styles),
            flex_grow: resolve_flex_grow(&child.classes, &child.styles),
            flex_basis_zero: resolve_flex_basis_zero(&child.classes, &child.styles),
        });
    }

    let mut normal_total_main_without_gap = 0.0f32;
    let mut total_flex_grow = 0.0f32;
    let mut normal_count = 0usize;
    for child in &children_meta {
        if child.position_mode != PositionMode::Normal {
            continue;
        }
        let base_main_size = match direction {
            FlowDirection::Column => {
                if child.flex_basis_zero {
                    0.0
                } else {
                    child.size.height
                }
            }
            FlowDirection::Row => {
                if child.flex_basis_zero {
                    0.0
                } else {
                    child.size.width
                }
            }
        };
        match direction {
            FlowDirection::Column => {
                normal_total_main_without_gap +=
                    child.margin.top + base_main_size + child.margin.bottom;
            }
            FlowDirection::Row => {
                normal_total_main_without_gap +=
                    child.margin.left + base_main_size + child.margin.right;
            }
        }
        total_flex_grow += child.flex_grow.max(0.0);
        normal_count += 1;
    }
    let main_available = match direction {
        FlowDirection::Column => content.height,
        FlowDirection::Row => content.width,
    };
    let default_total_main = if normal_count > 1 {
        normal_total_main_without_gap + gap * (normal_count as f32 - 1.0)
    } else {
        normal_total_main_without_gap
    };
    let flex_available_main = if total_flex_grow > 0.0 {
        (main_available - default_total_main).max(0.0)
    } else {
        0.0
    };
    let total_main_after_flex = default_total_main + flex_available_main;
    let leftover_main = (main_available - total_main_after_flex).max(0.0);

    let mut justify_offset = 0.0f32;
    let mut between_gap = gap;
    match justify.as_deref() {
        Some("CENTER") => {
            justify_offset = leftover_main * 0.5;
        }
        Some("MAX") => {
            justify_offset = leftover_main;
        }
        Some("SPACE_BETWEEN") if normal_count > 1 => {
            between_gap += leftover_main / (normal_count as f32 - 1.0);
        }
        Some("SPACE_AROUND") if normal_count > 0 => {
            let extra = leftover_main / normal_count as f32;
            between_gap += extra;
            justify_offset = extra * 0.5;
        }
        Some("SPACE_EVENLY") if normal_count > 0 => {
            let extra = leftover_main / (normal_count as f32 + 1.0);
            justify_offset = extra;
            between_gap += extra;
        }
        _ => {}
    };

    let mut cursor_main = justify_offset;
    let mut laid_out_children = Vec::with_capacity(children_meta.len());
    for child_meta in &children_meta {
        let child = &node.children[child_meta.index];
        let mut child_size = child_meta.size;
        let child_margin = child_meta.margin;
        let flex_share = if child_meta.position_mode == PositionMode::Normal
            && child_meta.flex_grow > 0.0
            && total_flex_grow > 0.0
        {
            flex_available_main * (child_meta.flex_grow / total_flex_grow)
        } else {
            0.0
        };
        match direction {
            FlowDirection::Column => {
                let base = if child_meta.flex_basis_zero {
                    0.0
                } else {
                    child_size.height
                };
                if child_meta.position_mode == PositionMode::Normal
                    && (child_meta.flex_basis_zero || child_meta.flex_grow > 0.0)
                {
                    child_size.height = (base + flex_share).max(1.0);
                }
            }
            FlowDirection::Row => {
                let base = if child_meta.flex_basis_zero {
                    0.0
                } else {
                    child_size.width
                };
                if child_meta.position_mode == PositionMode::Normal
                    && (child_meta.flex_basis_zero || child_meta.flex_grow > 0.0)
                {
                    child_size.width = (base + flex_share).max(1.0);
                }
            }
        }

        let child_rect = match child_meta.position_mode {
            PositionMode::Fixed => absolute_rect_for_child(
                child,
                child_size,
                Rect {
                    x: 0.0,
                    y: 0.0,
                    width: viewport.width,
                    height: viewport.height,
                },
                viewport,
            ),
            PositionMode::Absolute => absolute_rect_for_child(child, child_size, content, viewport),
            PositionMode::Normal => {
                let x;
                let y;

                match direction {
                    FlowDirection::Column => {
                        y = content.y + cursor_main + child_margin.top;
                        x = match align.as_deref() {
                            Some("CENTER") => {
                                content.x
                                    + (content.width - child_size.width) * 0.5
                                    + child_margin.left
                                    - child_margin.right
                            }
                            Some("MAX") => {
                                content.x + content.width - child_size.width - child_margin.right
                            }
                            _ => content.x + child_margin.left,
                        };
                    }
                    FlowDirection::Row => {
                        x = content.x + cursor_main + child_margin.left;
                        y = match align.as_deref() {
                            Some("CENTER") => {
                                content.y
                                    + (content.height - child_size.height) * 0.5
                                    + child_margin.top
                                    - child_margin.bottom
                            }
                            Some("MAX") => {
                                content.y + content.height - child_size.height - child_margin.bottom
                            }
                            _ => content.y + child_margin.top,
                        };
                    }
                }

                let rect = Rect {
                    x,
                    y,
                    width: child_size.width.max(1.0),
                    height: child_size.height.max(1.0),
                };

                match direction {
                    FlowDirection::Column => {
                        cursor_main += child_margin.top
                            + child_size.height
                            + child_margin.bottom
                            + between_gap;
                    }
                    FlowDirection::Row => {
                        cursor_main +=
                            child_margin.left + child_size.width + child_margin.right + between_gap;
                    }
                }
                rect
            }
        };

        laid_out_children.push((child_meta.index, child_rect, child_meta.z_index));
    }

    laid_out_children
        .sort_by(|left, right| left.2.cmp(&right.2).then_with(|| left.0.cmp(&right.0)));

    for (child_index, child_rect, _) in laid_out_children {
        let child = &node.children[child_index];
        path.push(child_index);
        layout_tree_with_effect_inheritance(
            child,
            path,
            Some(node_id.clone()),
            child_rect,
            viewport,
            size_cache,
            theme,
            &resolved_text,
            &child_inherited_effects,
            child_inherited_blend_mode.clone(),
            out_nodes,
        );
        path.pop();
    }
}
fn absolute_rect_for_child(
    node: &HtmlNode,
    measured: Size,
    container: Rect,
    viewport: Size,
) -> Rect {
    let size = size_from_rect(container);
    let left = resolve_offset(node, "left", size, viewport);
    let right = resolve_offset(node, "right", size, viewport);
    let top = resolve_offset(node, "top", size, viewport);
    let bottom = resolve_offset(node, "bottom", size, viewport);
    let inset = resolve_offset(node, "inset", size, viewport);

    let width = if let (Some(left), Some(right)) = (left.or(inset), right.or(inset)) {
        (container.width - left - right).max(1.0)
    } else {
        measured.width.max(1.0)
    };
    let height = if let (Some(top), Some(bottom)) = (top.or(inset), bottom.or(inset)) {
        (container.height - top - bottom).max(1.0)
    } else {
        measured.height.max(1.0)
    };

    let x = if let Some(left) = left.or(inset) {
        container.x + left
    } else if let Some(right) = right.or(inset) {
        container.x + container.width - right - width
    } else {
        container.x
    };
    let y = if let Some(top) = top.or(inset) {
        container.y + top
    } else if let Some(bottom) = bottom.or(inset) {
        container.y + container.height - bottom - height
    } else {
        container.y
    };

    Rect {
        x,
        y,
        width,
        height,
    }
}

#[allow(clippy::too_many_arguments)]
fn node_to_figma_json(
    node: &HtmlNode,
    id: &str,
    parent_id: Option<&str>,
    rect: Rect,
    theme: &TailwindTheme,
    viewport: Size,
    resolved_text: &TextStyle,
) -> JsonValue {
    node_to_figma_json_with_inherited_effects(
        node,
        id,
        parent_id,
        rect,
        theme,
        viewport,
        resolved_text,
        &[],
        None,
    )
    .json
}

#[derive(Debug)]
struct NodeJsonOutput {
    json: JsonValue,
    child_inherited_blend_mode: Option<String>,
}

#[allow(clippy::too_many_arguments)]
fn node_to_figma_json_with_inherited_effects(
    node: &HtmlNode,
    id: &str,
    parent_id: Option<&str>,
    rect: Rect,
    theme: &TailwindTheme,
    viewport: Size,
    resolved_text: &TextStyle,
    inherited_effects: &[JsonValue],
    inherited_blend_mode: Option<&str>,
) -> NodeJsonOutput {
    let node_type = figma_node_type(node);
    let is_frame = node_type == "FRAME";
    let mut object = JsonMap::new();
    object.insert("id".to_string(), JsonValue::String(id.to_string()));
    if let Some(parent_id) = parent_id {
        object.insert(
            "parentId".to_string(),
            JsonValue::String(parent_id.to_string()),
        );
    }
    object.insert("type".to_string(), JsonValue::String(node_type.to_string()));
    object.insert(
        "bounds".to_string(),
        JsonValue::Array(vec![
            json_number(rect.x),
            json_number(rect.y),
            json_number(rect.width.max(0.0)),
            json_number(rect.height.max(0.0)),
        ]),
    );

    if let Some(layout_mode) = figma_layout_mode(&node.classes) {
        object.insert(
            "layoutMode".to_string(),
            JsonValue::String(layout_mode.to_string()),
        );
        if let Some(primary) = parse_justify_content(&node.classes) {
            object.insert(
                "primaryAxisAlignItems".to_string(),
                JsonValue::String(primary),
            );
        }
        if let Some(counter) = parse_align_items(&node.classes) {
            object.insert(
                "counterAxisAlignItems".to_string(),
                JsonValue::String(counter),
            );
        }
        if let Some(gap) = parse_gap(&node.classes, size_from_rect(rect), viewport) {
            object.insert("itemSpacing".to_string(), json_number(gap.max(0.0)));
        }
    }

    let padding = parse_padding(&node.classes, size_from_rect(rect), viewport);
    if padding.top > 0.0 {
        object.insert("paddingTop".to_string(), json_number(padding.top));
    }
    if padding.right > 0.0 {
        object.insert("paddingRight".to_string(), json_number(padding.right));
    }
    if padding.bottom > 0.0 {
        object.insert("paddingBottom".to_string(), json_number(padding.bottom));
    }
    if padding.left > 0.0 {
        object.insert("paddingLeft".to_string(), json_number(padding.left));
    }

    if position_mode(&node.classes) != PositionMode::Normal {
        object.insert(
            "layoutPositioning".to_string(),
            JsonValue::String("ABSOLUTE".to_string()),
        );
    }
    if has_class(&node.classes, "flex-1") {
        object.insert("layoutGrow".to_string(), json_number(1.0));
    }

    if let Some(layout_align) = parse_layout_align(&node.classes) {
        object.insert("layoutAlign".to_string(), JsonValue::String(layout_align));
    }

    if let Some(opacity) = resolve_opacity(node) {
        object.insert("opacity".to_string(), json_number(opacity));
    }
    if let Some(rotation) = resolve_rotation_degrees(&node.classes, &node.styles)
        && rotation.abs() > f32::EPSILON
    {
        object.insert("rotation".to_string(), json_number(rotation));
    }
    if resolve_clips_content(&node.classes, &node.styles) {
        object.insert("clipsContent".to_string(), JsonValue::Bool(true));
    }
    let requested_blend_mode = resolve_blend_mode(&node.classes, &node.styles);

    if let Some(radius) = resolve_corner_radius(&node.classes, &node.styles, rect) {
        object.insert("cornerRadius".to_string(), json_number(radius));
    }

    let is_text_node = node_is_text_node(node);
    let text_background = resolve_background_color(&node.classes, &node.styles, theme);
    let mut fills = Vec::new();
    if !is_text_node {
        if let Some(image_ref) = resolve_background_image_ref(&node.classes, &node.styles, theme) {
            let source_ref = composed_source_reference_for_halftone(node, &image_ref);
            let image_scale_mode =
                resolve_image_scale_mode(&node.classes, &node.styles, &image_ref);
            let filter = resolve_image_filter(&node.styles);
            let filtered_ref = filter
                .map(|spec| filtered_image_reference(&source_ref, spec))
                .unwrap_or_else(|| source_ref.clone());
            let mut fill = json!({
                "type": "IMAGE",
                "imageRef": filtered_ref,
                "scaleMode": image_scale_mode,
                "visible": true
            });
            if let Some(spec) = filter {
                if let JsonValue::Object(fill_object) = &mut fill {
                    fill_object.insert("imageSourceRef".to_string(), JsonValue::String(source_ref));
                    fill_object.insert("imageFilter".to_string(), image_filter_to_json(spec));
                }
            }
            fills.push(fill);
        }
        if let Some(gradient_paint) = resolve_background_gradient_paint(&node.classes, theme) {
            fills.push(gradient_paint);
        }
        if let Some(color) = text_background {
            fills.push(solid_paint_json(color));
        }
    }
    if is_text_node {
        if let Some(color) = resolved_text.color {
            // Text renderer samples first fill as glyph paint while primitive fallback
            // still draws node bounds. Keep a second fill as the intended text background
            // (or transparent if absent) so badge-style spans remain legible.
            fills.push(solid_paint_json(color));
            fills.push(solid_paint_json(
                text_background.unwrap_or([0.0, 0.0, 0.0, 0.0]),
            ));
        } else if let Some(background) = text_background {
            fills.push(solid_paint_json(background));
        }
    }
    let has_fill_surface = !fills.is_empty();
    if has_fill_surface {
        object.insert("fills".to_string(), JsonValue::Array(fills));
    }

    if let Some(fill_geometry) = resolve_fill_geometry(node, rect) {
        object.insert("fillGeometry".to_string(), JsonValue::Array(fill_geometry));
    }

    let stroke = resolve_stroke(&node.classes, &node.styles, theme);
    let has_stroke_surface = stroke.is_some();
    if let Some((stroke_weight, stroke_color)) = stroke {
        object.insert(
            "strokes".to_string(),
            JsonValue::Array(vec![solid_paint_json(stroke_color)]),
        );
        object.insert(
            "strokeWeight".to_string(),
            json_number(stroke_weight.max(0.0)),
        );
        object.insert(
            "strokeAlign".to_string(),
            JsonValue::String("CENTER".to_string()),
        );
    }

    let requested_effects = resolve_effects(&node.classes, &node.styles, theme);
    let mut merged_effects = Vec::new();
    merged_effects.extend_from_slice(inherited_effects);
    merged_effects.extend(requested_effects);
    let effect_surface_allowed =
        !is_frame || has_fill_surface || has_stroke_surface || is_text_node;
    let effects = if effect_surface_allowed {
        merged_effects
    } else {
        Vec::new()
    };
    let has_effect_surface = !effects.is_empty();
    if has_effect_surface {
        object.insert("effects".to_string(), JsonValue::Array(effects));
    }
    let has_local_surface =
        has_fill_surface || has_stroke_surface || has_effect_surface || is_text_node;
    let blend_resolution = resolve_effective_blend_mode(
        requested_blend_mode.as_deref(),
        inherited_blend_mode,
        is_frame,
        has_local_surface,
    );
    if let Some(blend_mode) = blend_resolution.applied.as_ref() {
        object.insert(
            "blendMode".to_string(),
            JsonValue::String(blend_mode.clone()),
        );
    }

    if is_text_node {
        object.insert(
            "characters".to_string(),
            JsonValue::String(node.text.clone()),
        );
        let mut style = JsonMap::new();
        if let Some(size) = resolved_text.font_size {
            style.insert("fontSize".to_string(), json_number(size.max(1.0)));
        }
        if let Some(weight) = resolved_text.font_weight {
            style.insert(
                "fontWeight".to_string(),
                JsonValue::Number(JsonNumber::from(u64::from(weight))),
            );
        }
        if let Some(font_style) = resolved_text.font_style.as_ref() {
            style.insert(
                "fontStyle".to_string(),
                JsonValue::String(font_style.clone()),
            );
        }
        if let Some(font) = resolved_text.font_family.as_ref() {
            style.insert("fontFamily".to_string(), JsonValue::String(font.clone()));
        }
        if let Some(text_align) = resolved_text.text_align.as_ref() {
            style.insert(
                "textAlignHorizontal".to_string(),
                JsonValue::String(text_align.clone()),
            );
        }
        if let Some(text_case) = resolved_text.text_case.as_ref() {
            style.insert("textCase".to_string(), JsonValue::String(text_case.clone()));
        }
        if let Some(line_height) = resolved_text.line_height_percent {
            style.insert(
                "lineHeightPercentFontSize".to_string(),
                json_number(line_height.max(1.0)),
            );
        }
        if let Some(letter_spacing) = resolved_text.letter_spacing {
            style.insert("letterSpacing".to_string(), json_number(letter_spacing));
        }
        if !style.is_empty() {
            object.insert("style".to_string(), JsonValue::Object(style));
        }
    }

    NodeJsonOutput {
        json: JsonValue::Object(object),
        child_inherited_blend_mode: blend_resolution.propagated,
    }
}

#[derive(Debug)]
struct BlendResolution {
    applied: Option<String>,
    propagated: Option<String>,
}

fn resolve_effective_blend_mode(
    requested_blend_mode: Option<&str>,
    inherited_blend_mode: Option<&str>,
    is_frame: bool,
    has_local_surface: bool,
) -> BlendResolution {
    let can_apply = !is_frame || has_local_surface;
    if let Some(requested) = requested_blend_mode {
        if can_apply {
            return BlendResolution {
                applied: Some(requested.to_string()),
                propagated: Some(requested.to_string()),
            };
        }
        return BlendResolution {
            applied: None,
            propagated: Some(requested.to_string()),
        };
    }

    if let Some(inherited) = inherited_blend_mode {
        if can_apply {
            return BlendResolution {
                applied: Some(inherited.to_string()),
                propagated: Some(inherited.to_string()),
            };
        }
        return BlendResolution {
            applied: None,
            propagated: Some(inherited.to_string()),
        };
    }

    BlendResolution {
        applied: None,
        propagated: None,
    }
}

fn next_inherited_effects(
    node: &HtmlNode,
    theme: &TailwindTheme,
    resolved_text: &TextStyle,
    inherited_effects: &[JsonValue],
) -> Vec<JsonValue> {
    if effect_surface_allowed_for_node(node, theme, resolved_text) {
        return Vec::new();
    }

    let mut out = inherited_effects.to_vec();
    out.extend(resolve_effects(&node.classes, &node.styles, theme));
    out
}

fn effect_surface_allowed_for_node(
    node: &HtmlNode,
    theme: &TailwindTheme,
    resolved_text: &TextStyle,
) -> bool {
    let node_type = figma_node_type(node);
    let is_frame = node_type == "FRAME";
    let is_text_node = node_is_text_node(node);
    let text_background = resolve_background_color(&node.classes, &node.styles, theme);

    let has_fill_surface = if !is_text_node {
        resolve_background_image_ref(&node.classes, &node.styles, theme).is_some()
            || resolve_background_gradient_paint(&node.classes, theme).is_some()
            || text_background.is_some()
    } else if resolved_text.color.is_some() {
        true
    } else {
        text_background.is_some()
    };
    let has_stroke_surface = resolve_stroke(&node.classes, &node.styles, theme).is_some();
    !is_frame || has_fill_surface || has_stroke_surface || is_text_node
}

fn composed_source_reference_for_halftone(node: &HtmlNode, image_ref: &str) -> String {
    if has_class(&node.classes, "halftone")
        && !image_ref.starts_with("procedural://")
        && !image_ref.starts_with("composite://")
    {
        return format!("composite://halftone/{image_ref}");
    }
    image_ref.to_string()
}

fn resolve_text_style(node: &HtmlNode, theme: &TailwindTheme, inherited: &TextStyle) -> TextStyle {
    let mut style = inherited.clone();
    let font_size = resolve_font_size(&node.classes, &node.styles).or(style.font_size);
    if let Some(color) = resolve_text_color(&node.classes, &node.styles, theme) {
        style.color = Some(color);
    }
    if let Some(font_family) = resolve_font_family(&node.classes, &node.styles, theme) {
        style.font_family = Some(font_family);
    }
    if let Some(font_size) = font_size {
        style.font_size = Some(font_size);
    }
    if let Some(font_weight) = resolve_font_weight(&node.classes, &node.styles) {
        style.font_weight = Some(font_weight);
    }
    if let Some(font_style) = resolve_font_style(&node.classes, &node.styles) {
        style.font_style = Some(font_style);
    }
    if let Some(letter_spacing) =
        resolve_letter_spacing(&node.classes, &node.styles, style.font_size.unwrap_or(16.0))
    {
        style.letter_spacing = Some(letter_spacing);
    }
    if let Some(text_align) = resolve_text_align(&node.classes, &node.styles) {
        style.text_align = Some(text_align);
    }
    if let Some(text_case) = resolve_text_case(&node.classes, &node.styles) {
        style.text_case = Some(text_case);
    }
    if style
        .font_family
        .as_deref()
        .is_some_and(is_ligature_icon_font_family)
    {
        // Preserve ligature tokens (e.g. "search", "graphic_eq") for icon fonts.
        // Applying UPPER/LOWER/TITLE transforms breaks glyph lookup.
        style.text_case = None;
    }
    if let Some(line_height_factor) = resolve_line_height_factor_option(&node.classes, &node.styles)
    {
        style.line_height_percent = Some(line_height_factor * 100.0);
    }
    style
}

fn is_ligature_icon_font_family(font_family: &str) -> bool {
    let normalized = font_family.trim().to_ascii_lowercase();
    normalized.contains("material symbols") || normalized.contains("material icons")
}

fn path_to_key(path: &[usize]) -> String {
    path.iter()
        .map(usize::to_string)
        .collect::<Vec<_>>()
        .join(".")
}

fn size_from_rect(rect: Rect) -> Size {
    Size {
        width: rect.width,
        height: rect.height,
    }
}

fn figma_node_type(node: &HtmlNode) -> &'static str {
    if node_is_text_node(node) {
        "TEXT"
    } else if !node.children.is_empty() {
        "FRAME"
    } else {
        "RECTANGLE"
    }
}

fn figma_layout_mode(classes: &[String]) -> Option<&'static str> {
    if !is_flex_container(classes) {
        return None;
    }
    Some(match flow_direction(classes) {
        FlowDirection::Column => "VERTICAL",
        FlowDirection::Row => "HORIZONTAL",
    })
}

fn flow_direction(classes: &[String]) -> FlowDirection {
    if !is_flex_container(classes) {
        FlowDirection::Column
    } else if has_class(classes, "flex-col") {
        FlowDirection::Column
    } else {
        FlowDirection::Row
    }
}

fn is_flex_container(classes: &[String]) -> bool {
    has_class(classes, "flex") || has_class(classes, "inline-flex")
}

fn node_is_text_node(node: &HtmlNode) -> bool {
    !node.text.is_empty() && (is_textual_tag(&node.tag) || node.children.is_empty())
}

fn parent_layout_context_for(node: &HtmlNode) -> ParentLayoutContext {
    ParentLayoutContext {
        is_flex: is_flex_container(&node.classes),
        direction: flow_direction(&node.classes),
        cross_axis_stretches: parent_cross_axis_stretches(&node.classes),
    }
}

fn parent_cross_axis_stretches(classes: &[String]) -> bool {
    if !is_flex_container(classes) {
        return false;
    }
    !has_class(classes, "items-start")
        && !has_class(classes, "items-center")
        && !has_class(classes, "items-end")
        && !has_class(classes, "items-baseline")
}

fn resolve_positioned_stretch_size(
    node: &HtmlNode,
    axis: Axis,
    container_size: Size,
    viewport: Size,
) -> Option<f32> {
    let inset = resolve_offset(node, "inset", container_size, viewport);
    match axis {
        Axis::X => {
            let left = resolve_offset(node, "left", container_size, viewport).or(inset);
            let right = resolve_offset(node, "right", container_size, viewport).or(inset);
            match (left, right) {
                (Some(left), Some(right)) => Some((container_size.width - left - right).max(1.0)),
                _ => None,
            }
        }
        Axis::Y => {
            let top = resolve_offset(node, "top", container_size, viewport).or(inset);
            let bottom = resolve_offset(node, "bottom", container_size, viewport).or(inset);
            match (top, bottom) {
                (Some(top), Some(bottom)) => Some((container_size.height - top - bottom).max(1.0)),
                _ => None,
            }
        }
    }
}

fn node_fills_available_width(node: &HtmlNode, parent_layout: Option<ParentLayoutContext>) -> bool {
    if has_class(&node.classes, "inline") {
        return false;
    }
    if has_class(&node.classes, "inline-block") || has_class(&node.classes, "inline-flex") {
        return false;
    }
    if has_class(&node.classes, "self-start")
        || has_class(&node.classes, "self-center")
        || has_class(&node.classes, "self-end")
    {
        return false;
    }
    if has_class(&node.classes, "self-stretch") {
        return true;
    }
    if has_class(&node.classes, "flex-1") {
        return true;
    }
    if let Some(parent_layout) = parent_layout
        && parent_layout.is_flex
    {
        return match parent_layout.direction {
            FlowDirection::Row => false,
            FlowDirection::Column => parent_layout.cross_axis_stretches,
        };
    }
    true
}

fn resolve_flex_grow(classes: &[String], styles: &HashMap<String, String>) -> f32 {
    if has_class(classes, "grow-0") {
        return 0.0;
    }
    if has_class(classes, "flex-1") || has_class(classes, "grow") {
        return 1.0;
    }
    if let Some(value) = class_value(classes, "grow-[")
        .and_then(|token| token.strip_suffix(']'))
        .and_then(|token| token.parse::<f32>().ok())
    {
        return value.max(0.0);
    }
    if let Some(value) = styles
        .get("flex-grow")
        .and_then(|value| value.trim().parse::<f32>().ok())
    {
        return value.max(0.0);
    }
    if let Some(value) = styles.get("flex")
        && let Some(first) = value.split_whitespace().next()
        && let Ok(parsed) = first.trim().parse::<f32>()
    {
        return parsed.max(0.0);
    }
    0.0
}

fn resolve_flex_basis_zero(classes: &[String], styles: &HashMap<String, String>) -> bool {
    if has_class(classes, "flex-1") || has_class(classes, "basis-0") {
        return true;
    }
    if let Some(value) = styles.get("flex-basis") {
        let trimmed = value.trim().trim_end_matches(';');
        if trimmed == "0" || trimmed == "0px" || trimmed == "0%" {
            return true;
        }
    }
    if let Some(value) = styles.get("flex") {
        let mut tokens = value.split_whitespace();
        let _grow = tokens.next();
        let _shrink = tokens.next();
        if let Some(basis) = tokens.next() {
            let trimmed = basis.trim().trim_end_matches(';');
            if trimmed == "0" || trimmed == "0px" || trimmed == "0%" {
                return true;
            }
        }
    }
    false
}

fn position_mode(classes: &[String]) -> PositionMode {
    if has_class(classes, "fixed") {
        PositionMode::Fixed
    } else if has_class(classes, "absolute") {
        PositionMode::Absolute
    } else {
        PositionMode::Normal
    }
}

fn resolve_z_index(classes: &[String], styles: &HashMap<String, String>) -> i32 {
    if let Some(style_value) = styles.get("z-index")
        && let Some(parsed) = parse_css_integer(style_value)
    {
        return parsed;
    }

    let mut resolved = None;
    for class in classes {
        if let Some(value) = class.strip_prefix("z-[")
            && let Some(inner) = value.strip_suffix(']')
            && let Ok(parsed) = inner.parse::<i32>()
        {
            resolved = Some(parsed);
            continue;
        }

        if let Some(value) = class.strip_prefix("-z-")
            && let Ok(parsed) = value.parse::<i32>()
        {
            resolved = Some(-parsed);
            continue;
        }

        if let Some(value) = class.strip_prefix("z-")
            && let Ok(parsed) = value.parse::<i32>()
        {
            resolved = Some(parsed);
        }
    }

    resolved.unwrap_or(0)
}

fn parse_css_integer(value: &str) -> Option<i32> {
    let trimmed = value.trim().trim_end_matches(';');
    if let Ok(parsed) = trimmed.parse::<i32>() {
        return Some(parsed);
    }
    let parsed = trimmed.parse::<f32>().ok()?;
    if !parsed.is_finite() {
        return None;
    }
    Some(parsed.round() as i32)
}

fn has_class(classes: &[String], needle: &str) -> bool {
    classes.iter().any(|class| class == needle)
}
fn resolve_dimension(
    node: &HtmlNode,
    axis: Axis,
    parent_size: Size,
    viewport: Size,
) -> Option<f32> {
    let token = match axis {
        Axis::X => class_value(&node.classes, "w-"),
        Axis::Y => class_value(&node.classes, "h-"),
    };
    let parent_axis = match axis {
        Axis::X => parent_size.width,
        Axis::Y => parent_size.height,
    };
    let viewport_axis = match axis {
        Axis::X => viewport.width,
        Axis::Y => viewport.height,
    };

    if let Some(token) = token
        && let Some(value) = resolve_length_token(token, parent_axis, viewport_axis)
    {
        return Some(value.max(0.0));
    }

    let key = match axis {
        Axis::X => "width",
        Axis::Y => "height",
    };
    if let Some(style_value) = node.styles.get(key)
        && let Some(value) = parse_css_length(style_value, parent_axis, viewport_axis)
    {
        return Some(value.max(0.0));
    }

    None
}

fn resolve_offset(
    node: &HtmlNode,
    property: &str,
    parent_size: Size,
    viewport: Size,
) -> Option<f32> {
    let axis = if property == "left" || property == "right" || property == "inset" {
        Axis::X
    } else {
        Axis::Y
    };
    let parent_axis = if matches!(axis, Axis::X) {
        parent_size.width
    } else {
        parent_size.height
    };
    let viewport_axis = if matches!(axis, Axis::X) {
        viewport.width
    } else {
        viewport.height
    };

    let prefix = format!("{property}-");
    if let Some(token) = class_value(&node.classes, &prefix)
        && let Some(value) = resolve_length_token(token, parent_axis, viewport_axis)
    {
        return Some(value);
    }

    let negative_prefix = format!("-{property}-");
    if let Some(token) = class_value(&node.classes, &negative_prefix)
        && let Some(value) = resolve_length_token(token, parent_axis, viewport_axis)
    {
        return Some(-value);
    }

    if let Some(style_value) = node.styles.get(property) {
        if let Some(value) = parse_css_length(style_value, parent_axis, viewport_axis) {
            return Some(value);
        }
    }

    None
}

fn class_value<'a>(classes: &'a [String], prefix: &str) -> Option<&'a str> {
    classes
        .iter()
        .rev()
        .find_map(|class| class.strip_prefix(prefix))
}

fn apply_translation_transform(rect: Rect, node: &HtmlNode, viewport: Size) -> Rect {
    let self_size = size_from_rect(rect);
    let tx = resolve_translation_from_classes(&node.classes, Axis::X, self_size, viewport);
    let ty = resolve_translation_from_classes(&node.classes, Axis::Y, self_size, viewport);
    Rect {
        x: rect.x + tx,
        y: rect.y + ty,
        ..rect
    }
}

fn resolve_rotation_degrees(classes: &[String], styles: &HashMap<String, String>) -> Option<f32> {
    if let Some(value) = styles.get("transform")
        && let Some(rotation) = parse_css_transform_rotation_degrees(value)
    {
        return Some(rotation);
    }

    for class_name in classes.iter().rev() {
        let utility = static_responsive_utility(class_name);
        if let Some(token) = utility.strip_prefix("-rotate-")
            && let Some(value) = parse_tailwind_angle_degrees(token)
        {
            return Some(-value);
        }
        if let Some(token) = utility.strip_prefix("rotate-")
            && let Some(value) = parse_tailwind_angle_degrees(token)
        {
            return Some(value);
        }
    }
    None
}

fn parse_css_transform_rotation_degrees(value: &str) -> Option<f32> {
    let mut total = 0.0f32;
    let mut found = false;
    let mut cursor = value;
    while let Some(start) = cursor.find("rotate(") {
        let tail = &cursor[start + "rotate(".len()..];
        let Some(end) = tail.find(')') else {
            break;
        };
        if let Some(rotation) = parse_css_angle_degrees(&tail[..end]) {
            total += rotation;
            found = true;
        }
        cursor = &tail[end + 1..];
    }
    found.then_some(total)
}

fn parse_tailwind_angle_degrees(token: &str) -> Option<f32> {
    if let Some(arbitrary) = token
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    {
        return parse_css_angle_degrees(arbitrary);
    }
    token.trim().parse::<f32>().ok()
}

fn parse_css_angle_degrees(raw: &str) -> Option<f32> {
    let value = raw.trim().to_ascii_lowercase();
    if let Some(deg) = value.strip_suffix("deg") {
        return deg.trim().parse::<f32>().ok();
    }
    if let Some(rad) = value.strip_suffix("rad")
        && let Ok(parsed) = rad.trim().parse::<f32>()
    {
        return Some(parsed.to_degrees());
    }
    if let Some(turn) = value.strip_suffix("turn")
        && let Ok(parsed) = turn.trim().parse::<f32>()
    {
        return Some(parsed * 360.0);
    }
    value.parse::<f32>().ok()
}

fn resolve_translation_from_classes(
    classes: &[String],
    axis: Axis,
    self_size: Size,
    viewport: Size,
) -> f32 {
    let (negative_prefix, positive_prefix, parent_axis, viewport_axis) = match axis {
        Axis::X => (
            "-translate-x-",
            "translate-x-",
            self_size.width,
            viewport.width,
        ),
        Axis::Y => (
            "-translate-y-",
            "translate-y-",
            self_size.height,
            viewport.height,
        ),
    };

    for class_name in classes.iter().rev() {
        if let Some(token) = class_name.strip_prefix(negative_prefix) {
            return resolve_length_token(token, parent_axis, viewport_axis)
                .map(|value| -value)
                .unwrap_or(0.0);
        }
        if let Some(token) = class_name.strip_prefix(positive_prefix) {
            return resolve_length_token(token, parent_axis, viewport_axis).unwrap_or(0.0);
        }
    }
    0.0
}

fn resolve_length_token(token: &str, parent_axis: f32, viewport_axis: f32) -> Option<f32> {
    let (negative, token) = if let Some(stripped) = token.strip_prefix('-') {
        (true, stripped)
    } else {
        (false, token)
    };

    let mut value = if token == "full" {
        parent_axis
    } else if token == "screen" {
        viewport_axis
    } else if token == "px" {
        1.0
    } else if token.contains('/') {
        let (num, den) = token.split_once('/')?;
        let numerator = num.parse::<f32>().ok()?;
        let denominator = den.parse::<f32>().ok()?;
        if denominator == 0.0 {
            return None;
        }
        parent_axis * (numerator / denominator)
    } else if token.starts_with('[') && token.ends_with(']') {
        let arbitrary = &token[1..token.len() - 1];
        parse_css_length(arbitrary, parent_axis, viewport_axis)?
    } else {
        tailwind_spacing_value(token)?
    };

    if negative {
        value = -value;
    }
    Some(value)
}

fn parse_css_length(raw: &str, parent_axis: f32, viewport_axis: f32) -> Option<f32> {
    let value = raw.trim().to_ascii_lowercase();
    if value.ends_with("px") {
        return value[..value.len() - 2].trim().parse::<f32>().ok();
    }
    if value.ends_with("rem") {
        let rem = value[..value.len() - 3].trim().parse::<f32>().ok()?;
        return Some(rem * 16.0);
    }
    if value.ends_with("em") {
        let em = value[..value.len() - 2].trim().parse::<f32>().ok()?;
        return Some(em * 16.0);
    }
    if value.ends_with("vh") || value.ends_with("vw") {
        let number = value[..value.len() - 2].trim().parse::<f32>().ok()?;
        return Some(viewport_axis * (number / 100.0));
    }
    if value.ends_with('%') {
        let number = value[..value.len() - 1].trim().parse::<f32>().ok()?;
        return Some(parent_axis * (number / 100.0));
    }
    value.parse::<f32>().ok()
}

fn tailwind_spacing_value(token: &str) -> Option<f32> {
    let scale = match token {
        "0" => 0.0,
        "0.5" => 2.0,
        "1" => 4.0,
        "1.5" => 6.0,
        "2" => 8.0,
        "2.5" => 10.0,
        "3" => 12.0,
        "3.5" => 14.0,
        "4" => 16.0,
        "5" => 20.0,
        "6" => 24.0,
        "7" => 28.0,
        "8" => 32.0,
        "9" => 36.0,
        "10" => 40.0,
        "11" => 44.0,
        "12" => 48.0,
        "14" => 56.0,
        "16" => 64.0,
        "20" => 80.0,
        "24" => 96.0,
        "28" => 112.0,
        "32" => 128.0,
        "36" => 144.0,
        "40" => 160.0,
        "44" => 176.0,
        "48" => 192.0,
        "52" => 208.0,
        "56" => 224.0,
        "60" => 240.0,
        "64" => 256.0,
        "72" => 288.0,
        "80" => 320.0,
        "96" => 384.0,
        _ => return None,
    };
    Some(scale)
}

fn parse_padding(classes: &[String], parent_size: Size, viewport: Size) -> Edges {
    parse_box_spacing(classes, "p", parent_size, viewport)
}

fn parse_margin(classes: &[String], parent_size: Size, viewport: Size) -> Edges {
    parse_box_spacing(classes, "m", parent_size, viewport)
}

fn parse_box_spacing(classes: &[String], prefix: &str, parent_size: Size, viewport: Size) -> Edges {
    let mut edges = Edges::default();

    if let Some(value) = class_value(classes, &format!("{prefix}-"))
        .and_then(|token| resolve_length_token(token, parent_size.width, viewport.width))
    {
        edges.top = value;
        edges.right = value;
        edges.bottom = value;
        edges.left = value;
    }

    if let Some(value) = class_value(classes, &format!("{prefix}x-"))
        .and_then(|token| resolve_length_token(token, parent_size.width, viewport.width))
    {
        edges.left = value;
        edges.right = value;
    }
    if let Some(value) = class_value(classes, &format!("{prefix}y-"))
        .and_then(|token| resolve_length_token(token, parent_size.height, viewport.height))
    {
        edges.top = value;
        edges.bottom = value;
    }
    if let Some(value) = class_value(classes, &format!("{prefix}t-"))
        .and_then(|token| resolve_length_token(token, parent_size.height, viewport.height))
    {
        edges.top = value;
    }
    if let Some(value) = class_value(classes, &format!("{prefix}r-"))
        .and_then(|token| resolve_length_token(token, parent_size.width, viewport.width))
    {
        edges.right = value;
    }
    if let Some(value) = class_value(classes, &format!("{prefix}b-"))
        .and_then(|token| resolve_length_token(token, parent_size.height, viewport.height))
    {
        edges.bottom = value;
    }
    if let Some(value) = class_value(classes, &format!("{prefix}l-"))
        .and_then(|token| resolve_length_token(token, parent_size.width, viewport.width))
    {
        edges.left = value;
    }

    edges
}

fn parse_gap(classes: &[String], parent_size: Size, viewport: Size) -> Option<f32> {
    class_value(classes, "gap-")
        .and_then(|token| resolve_length_token(token, parent_size.width, viewport.width))
}

fn parse_justify_content(classes: &[String]) -> Option<String> {
    if has_class(classes, "justify-center") {
        Some("CENTER".to_string())
    } else if has_class(classes, "justify-end") || has_class(classes, "justify-between") {
        if has_class(classes, "justify-between") {
            Some("SPACE_BETWEEN".to_string())
        } else {
            Some("MAX".to_string())
        }
    } else if has_class(classes, "justify-around") {
        Some("SPACE_AROUND".to_string())
    } else if has_class(classes, "justify-evenly") {
        Some("SPACE_EVENLY".to_string())
    } else if has_class(classes, "justify-start") || has_class(classes, "justify-normal") {
        Some("MIN".to_string())
    } else {
        None
    }
}

fn parse_align_items(classes: &[String]) -> Option<String> {
    if has_class(classes, "items-center") {
        Some("CENTER".to_string())
    } else if has_class(classes, "items-end") {
        Some("MAX".to_string())
    } else if has_class(classes, "items-start") {
        Some("MIN".to_string())
    } else if has_class(classes, "items-baseline") {
        Some("BASELINE".to_string())
    } else if has_class(classes, "items-stretch") {
        Some("MIN".to_string())
    } else {
        None
    }
}

fn parse_layout_align(classes: &[String]) -> Option<String> {
    if has_class(classes, "self-start") {
        Some("MIN".to_string())
    } else if has_class(classes, "self-center") {
        Some("CENTER".to_string())
    } else if has_class(classes, "self-end") {
        Some("MAX".to_string())
    } else if has_class(classes, "self-stretch") {
        Some("STRETCH".to_string())
    } else {
        None
    }
}

fn is_textual_tag(tag: &str) -> bool {
    matches!(
        tag,
        "span" | "p" | "h1" | "h2" | "h3" | "h4" | "h5" | "h6" | "label" | "small"
    )
}

fn resolve_font_size(classes: &[String], styles: &HashMap<String, String>) -> Option<f32> {
    for class_name in classes.iter().rev() {
        let utility = static_responsive_utility(class_name);
        let Some(token) = utility.strip_prefix("text-") else {
            continue;
        };
        if let Some(size) = text_size_token_to_px(token) {
            return Some(size.max(1.0));
        }
    }
    if let Some(value) = styles.get("font-size")
        && let Some(px) = parse_css_length(value, 0.0, 0.0)
    {
        return Some(px.max(1.0));
    }
    None
}

fn static_responsive_utility<'a>(class_name: &'a str) -> &'a str {
    const PREFIXES: [&str; 7] = ["sm:", "md:", "lg:", "xl:", "2xl:", "3xl:", "4xl:"];
    for prefix in PREFIXES {
        if let Some(rest) = class_name.strip_prefix(prefix) {
            return rest;
        }
    }
    class_name
}

fn text_size_token_to_px(token: &str) -> Option<f32> {
    if let Some(arbitrary) = token
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
    {
        return parse_css_length(arbitrary, 0.0, 0.0);
    }
    let size = match token {
        "xs" => 12.0,
        "sm" => 14.0,
        "base" => 16.0,
        "lg" => 18.0,
        "xl" => 20.0,
        "2xl" => 24.0,
        "3xl" => 30.0,
        "4xl" => 36.0,
        "5xl" => 48.0,
        "6xl" => 60.0,
        "7xl" => 72.0,
        "8xl" => 96.0,
        "9xl" => 128.0,
        _ => return None,
    };
    Some(size)
}

fn resolve_font_weight(classes: &[String], styles: &HashMap<String, String>) -> Option<u16> {
    if let Some(value) = styles.get("font-weight") {
        let normalized = value.trim().to_ascii_lowercase();
        if let Ok(parsed) = normalized.parse::<u16>() {
            return Some(parsed.clamp(100, 900));
        }
        return match normalized.as_str() {
            "normal" => Some(400),
            "bold" => Some(700),
            "bolder" => Some(800),
            "lighter" => Some(300),
            _ => None,
        };
    }
    if has_class(classes, "font-thin") {
        Some(100)
    } else if has_class(classes, "font-extralight") {
        Some(200)
    } else if has_class(classes, "font-light") {
        Some(300)
    } else if has_class(classes, "font-normal") {
        Some(400)
    } else if has_class(classes, "font-medium") {
        Some(500)
    } else if has_class(classes, "font-semibold") {
        Some(600)
    } else if has_class(classes, "font-bold") {
        Some(700)
    } else if has_class(classes, "font-extrabold") {
        Some(800)
    } else if has_class(classes, "font-black") {
        Some(900)
    } else {
        None
    }
}

fn resolve_font_style(classes: &[String], styles: &HashMap<String, String>) -> Option<String> {
    if let Some(value) = styles.get("font-style") {
        return match value.trim().to_ascii_lowercase().as_str() {
            "italic" | "oblique" => Some("ITALIC".to_string()),
            "normal" => Some("NORMAL".to_string()),
            _ => None,
        };
    }
    if has_class(classes, "italic") {
        Some("ITALIC".to_string())
    } else if has_class(classes, "not-italic") {
        Some("NORMAL".to_string())
    } else {
        None
    }
}

fn resolve_font_family(
    classes: &[String],
    styles: &HashMap<String, String>,
    theme: &TailwindTheme,
) -> Option<String> {
    if let Some(value) = styles.get("font-family") {
        if let Some(font) = parse_first_quoted_token(value) {
            return Some(font);
        }
        if let Some((head, _)) = value.split_once(',') {
            let token = head.trim().trim_matches('\'').trim_matches('"');
            if !token.is_empty() {
                return Some(token.to_string());
            }
        }
    }
    for class_name in classes.iter().rev() {
        let Some(family_alias) = class_name.strip_prefix("font-") else {
            continue;
        };
        if is_font_weight_utility(family_alias) {
            continue;
        }
        if let Some(family) = theme.fonts.get(family_alias) {
            return Some(family.clone());
        }
        if family_alias.contains('-') {
            return Some(family_alias.replace('-', " "));
        }
    }
    None
}

fn is_font_weight_utility(token: &str) -> bool {
    matches!(
        token,
        "thin"
            | "extralight"
            | "light"
            | "normal"
            | "medium"
            | "semibold"
            | "bold"
            | "extrabold"
            | "black"
    )
}

fn resolve_text_align(classes: &[String], styles: &HashMap<String, String>) -> Option<String> {
    if let Some(value) = styles.get("text-align") {
        return match value.trim().to_ascii_lowercase().as_str() {
            "left" | "start" => Some("LEFT".to_string()),
            "center" => Some("CENTER".to_string()),
            "right" | "end" => Some("RIGHT".to_string()),
            "justify" => Some("JUSTIFIED".to_string()),
            _ => None,
        };
    }
    if has_class(classes, "text-center") {
        Some("CENTER".to_string())
    } else if has_class(classes, "text-right") {
        Some("RIGHT".to_string())
    } else if has_class(classes, "text-left") {
        Some("LEFT".to_string())
    } else if has_class(classes, "text-justify") {
        Some("JUSTIFIED".to_string())
    } else {
        None
    }
}

fn resolve_text_case(classes: &[String], styles: &HashMap<String, String>) -> Option<String> {
    if let Some(value) = styles.get("text-transform") {
        return match value.trim().to_ascii_lowercase().as_str() {
            "uppercase" => Some("UPPER".to_string()),
            "lowercase" => Some("LOWER".to_string()),
            "capitalize" => Some("TITLE".to_string()),
            _ => None,
        };
    }
    if has_class(classes, "uppercase") {
        Some("UPPER".to_string())
    } else if has_class(classes, "lowercase") {
        Some("LOWER".to_string())
    } else if has_class(classes, "capitalize") {
        Some("TITLE".to_string())
    } else {
        None
    }
}

fn resolve_letter_spacing(
    classes: &[String],
    styles: &HashMap<String, String>,
    font_size_px: f32,
) -> Option<f32> {
    for class_name in classes.iter().rev() {
        let utility = static_responsive_utility(class_name);
        let Some(token) = utility.strip_prefix("tracking-") else {
            continue;
        };
        let em = match token {
            "tighter" => Some(-0.05),
            "tight" => Some(-0.025),
            "normal" => Some(0.0),
            "wide" => Some(0.025),
            "wider" => Some(0.05),
            "widest" => Some(0.1),
            _ => None,
        };
        if let Some(em) = em {
            return Some(em * font_size_px);
        }
        if let Some(raw) = token
            .strip_prefix('[')
            .and_then(|text| text.strip_suffix(']'))
            && let Some(parsed) = parse_css_letter_spacing_value(raw, font_size_px)
        {
            return Some(parsed);
        }
    }

    styles
        .get("letter-spacing")
        .and_then(|value| parse_css_letter_spacing_value(value, font_size_px))
}

fn resolve_line_height_factor_option(
    classes: &[String],
    styles: &HashMap<String, String>,
) -> Option<f32> {
    if let Some(value) = styles.get("line-height") {
        let trimmed = value.trim();
        if let Some(percent) = trimmed.strip_suffix('%')
            && let Ok(parsed) = percent.trim().parse::<f32>()
        {
            return Some((parsed / 100.0).max(0.1));
        }
        if let Ok(parsed) = trimmed.parse::<f32>() {
            return Some(parsed.max(0.1));
        }
    }
    if has_class(classes, "leading-none") {
        Some(1.0)
    } else if has_class(classes, "leading-tight") {
        Some(1.25)
    } else if has_class(classes, "leading-snug") {
        Some(1.375)
    } else if has_class(classes, "leading-normal") {
        Some(1.5)
    } else if has_class(classes, "leading-relaxed") {
        Some(1.625)
    } else if has_class(classes, "leading-loose") {
        Some(2.0)
    } else {
        None
    }
}

fn resolve_line_height_factor(classes: &[String], styles: &HashMap<String, String>) -> f32 {
    resolve_line_height_factor_option(classes, styles).unwrap_or(1.35)
}

fn estimate_text_size(
    text: &str,
    font_size: f32,
    line_height_factor: f32,
    is_ligature_icon_font: bool,
    letter_spacing_px: f32,
) -> Size {
    if text.is_empty() {
        return Size {
            width: font_size * 0.5,
            height: font_size * line_height_factor,
        };
    }
    let advance_factor = 0.56_f32;
    let mut max_line_width = 0.0_f32;
    let mut line_count = 0usize;
    for line in text.split('\n') {
        let char_count = line.chars().count();
        let line_width = if is_ligature_icon_font {
            font_size
        } else {
            let glyph_width = char_count as f32 * font_size * advance_factor;
            let tracking_width =
                (char_count.saturating_sub(1) as f32) * letter_spacing_px;
            glyph_width + tracking_width
        };
        max_line_width = max_line_width.max(line_width);
        line_count += 1;
    }
    let width = if is_ligature_icon_font {
        font_size
    } else {
        max_line_width
    };
    let height = line_count as f32 * font_size * line_height_factor;
    Size {
        width: width.max(if is_ligature_icon_font {
            font_size
        } else {
            font_size * advance_factor
        }),
        height: height.max(font_size * line_height_factor),
    }
}
fn resolve_background_image_ref(
    classes: &[String],
    styles: &HashMap<String, String>,
    theme: &TailwindTheme,
) -> Option<String> {
    if let Some(value) = styles.get("background-image")
        && let Some(reference) = parse_background_image_reference(value)
    {
        return Some(reference);
    }

    let token = classes
        .iter()
        .rev()
        .find_map(|class_name| class_name.strip_prefix("bg-"))?;
    let value = theme.background_images.get(&token.to_ascii_lowercase())?;
    parse_background_image_reference(value)
}

fn resolve_image_scale_mode(
    classes: &[String],
    styles: &HashMap<String, String>,
    image_ref: &str,
) -> &'static str {
    if let Some(value) = styles.get("background-size") {
        let normalized = value.trim().to_ascii_lowercase();
        if normalized.contains("contain") {
            return "FIT";
        }
        if normalized.contains("cover") {
            return "FILL";
        }
    }

    if let Some(value) = styles.get("background-repeat") {
        let normalized = value.trim().to_ascii_lowercase();
        if normalized.contains("no-repeat") {
            return "FILL";
        }
        if normalized.contains("repeat") {
            return "TILE";
        }
    }

    if has_class(classes, "bg-cover") {
        return "FILL";
    }
    if has_class(classes, "bg-contain") {
        return "FIT";
    }
    if classes
        .iter()
        .any(|class_name| class_name.starts_with("bg-repeat"))
    {
        return "TILE";
    }
    if has_class(classes, "bg-no-repeat") {
        return "FILL";
    }

    if image_ref.starts_with("procedural://noise/")
        || image_ref.starts_with("procedural://halftone/")
    {
        return "TILE";
    }

    "FILL"
}

fn resolve_fill_geometry(node: &HtmlNode, rect: Rect) -> Option<Vec<JsonValue>> {
    let clip_path = node.styles.get("clip-path")?;
    let path = parse_clip_path_polygon(clip_path, rect.width, rect.height)?;
    Some(vec![JsonValue::String(path)])
}

fn parse_clip_path_polygon(value: &str, width: f32, height: f32) -> Option<String> {
    let trimmed = value.trim();
    let payload = trimmed
        .strip_prefix("polygon(")
        .and_then(|text| text.strip_suffix(')'))?;
    let mut points = Vec::new();
    for raw_point in payload.split(',') {
        let tokens = tokenize_css_function_aware(raw_point);
        if tokens.len() < 2 {
            continue;
        }
        let x = parse_clip_path_coordinate(&tokens[0], width)?;
        let y = parse_clip_path_coordinate(&tokens[1], height)?;
        points.push((x, y));
    }
    if points.len() < 3 {
        return None;
    }
    let mut path = String::new();
    for (index, (x, y)) in points.iter().enumerate() {
        if index == 0 {
            path.push_str("M ");
        } else {
            path.push_str(" L ");
        }
        path.push_str(&format!(
            "{} {}",
            format_svg_number(*x),
            format_svg_number(*y)
        ));
    }
    path.push_str(" Z");
    Some(path)
}

fn parse_clip_path_coordinate(token: &str, axis_extent: f32) -> Option<f32> {
    let trimmed = token.trim();
    if let Some(expr) = trimmed
        .strip_prefix("calc(")
        .and_then(|text| text.strip_suffix(')'))
    {
        return parse_calc_coordinate(expr, axis_extent);
    }
    parse_css_length(trimmed, axis_extent, axis_extent)
}

fn parse_calc_coordinate(expr: &str, axis_extent: f32) -> Option<f32> {
    let compact = expr
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>();
    if compact == "100%" {
        return Some(axis_extent);
    }
    if let Some(rest) = compact.strip_prefix("100%-")
        && let Some(value) = parse_css_length(rest, axis_extent, axis_extent)
    {
        return Some(axis_extent - value);
    }
    if let Some(rest) = compact.strip_prefix("100%+")
        && let Some(value) = parse_css_length(rest, axis_extent, axis_extent)
    {
        return Some(axis_extent + value);
    }
    None
}

fn format_svg_number(value: f32) -> String {
    let mut text = format!("{value:.3}");
    while text.contains('.') && text.ends_with('0') {
        text.pop();
    }
    if text.ends_with('.') {
        text.pop();
    }
    if text.is_empty() {
        "0".to_string()
    } else {
        text
    }
}

fn parse_background_image_reference(value: &str) -> Option<String> {
    let url = extract_css_url(value)?;
    normalize_background_image_reference(&url)
}

fn extract_css_url(value: &str) -> Option<String> {
    let start = value.find("url(")?;
    let tail = &value[start + 4..];
    let end = tail.find(')')?;
    let url = tail[..end].trim().trim_matches('\'').trim_matches('"');
    if url.is_empty() {
        None
    } else {
        Some(url.to_string())
    }
}

fn normalize_background_image_reference(url: &str) -> Option<String> {
    if url.starts_with("data:image/svg+xml") {
        let hash = stable_hash64(url);
        return Some(format!("procedural://noise/{hash:016x}"));
    }
    if url.starts_with("data:") {
        return None;
    }
    let normalized = url.strip_prefix("./").unwrap_or(url);
    if normalized.is_empty() {
        None
    } else {
        Some(normalized.to_string())
    }
}

fn stable_hash64(text: &str) -> u64 {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in text.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn resolve_image_filter(styles: &HashMap<String, String>) -> Option<ImageFilterSpec> {
    let filter = styles.get("filter")?;
    let mut spec = ImageFilterSpec::default();
    for token in filter.split_whitespace() {
        let token = token.trim();
        if let Some(value) = token
            .strip_prefix("grayscale(")
            .and_then(|value| value.strip_suffix(')'))
        {
            spec.grayscale = parse_css_percent_or_ratio(value);
            continue;
        }
        if let Some(value) = token
            .strip_prefix("contrast(")
            .and_then(|value| value.strip_suffix(')'))
        {
            spec.contrast = parse_css_percent_or_ratio(value);
            continue;
        }
        if let Some(value) = token
            .strip_prefix("invert(")
            .and_then(|value| value.strip_suffix(')'))
        {
            spec.invert = parse_css_percent_or_ratio(value);
        }
    }
    if spec == ImageFilterSpec::default() {
        None
    } else {
        Some(spec)
    }
}

fn parse_css_percent_or_ratio(value: &str) -> Option<f32> {
    let trimmed = value.trim();
    if let Some(percent) = trimmed.strip_suffix('%')
        && let Ok(parsed) = percent.trim().parse::<f32>()
    {
        return Some((parsed / 100.0).max(0.0));
    }
    trimmed.parse::<f32>().ok().map(|parsed| parsed.max(0.0))
}

fn filtered_image_reference(source_ref: &str, filter: ImageFilterSpec) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in source_ref.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    for value in [filter.grayscale, filter.contrast, filter.invert] {
        let bits = value.unwrap_or(0.0).to_bits();
        hash ^= u64::from(bits);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("filtered://{source_ref}#{hash:016x}")
}

fn image_filter_to_json(filter: ImageFilterSpec) -> JsonValue {
    let mut object = JsonMap::new();
    if let Some(value) = filter.grayscale {
        object.insert("grayscale".to_string(), json_number(value));
    }
    if let Some(value) = filter.contrast {
        object.insert("contrast".to_string(), json_number(value));
    }
    if let Some(value) = filter.invert {
        object.insert("invert".to_string(), json_number(value));
    }
    JsonValue::Object(object)
}

fn resolve_background_color(
    classes: &[String],
    styles: &HashMap<String, String>,
    theme: &TailwindTheme,
) -> Option<[f32; 4]> {
    if let Some(value) = styles.get("background-color")
        && let Some(color) = parse_css_color_value(value, theme)
    {
        return Some(color);
    }
    classes
        .iter()
        .rev()
        .find_map(|class| class.strip_prefix("bg-"))
        .and_then(|token| resolve_color_token(token, theme))
}

fn resolve_background_gradient_paint(
    classes: &[String],
    theme: &TailwindTheme,
) -> Option<JsonValue> {
    let direction = classes
        .iter()
        .rev()
        .find_map(|class_name| class_name.strip_prefix("bg-gradient-to-"))?;
    let (start, end) = match direction {
        "t" => ([0.5_f32, 1.0_f32], [0.5_f32, 0.0_f32]),
        "b" => ([0.5_f32, 0.0_f32], [0.5_f32, 1.0_f32]),
        "l" => ([1.0_f32, 0.5_f32], [0.0_f32, 0.5_f32]),
        "r" => ([0.0_f32, 0.5_f32], [1.0_f32, 0.5_f32]),
        "tr" => ([0.0_f32, 1.0_f32], [1.0_f32, 0.0_f32]),
        "tl" => ([1.0_f32, 1.0_f32], [0.0_f32, 0.0_f32]),
        "br" => ([0.0_f32, 0.0_f32], [1.0_f32, 1.0_f32]),
        "bl" => ([1.0_f32, 0.0_f32], [0.0_f32, 1.0_f32]),
        _ => return None,
    };

    let from = class_value(classes, "from-").and_then(|token| resolve_color_token(token, theme));
    let via = class_value(classes, "via-").and_then(|token| resolve_color_token(token, theme));
    let to = class_value(classes, "to-")
        .and_then(|token| resolve_color_token(token, theme))
        .or_else(|| Some([0.0, 0.0, 0.0, 0.0]));

    let mut stops = Vec::new();
    if let Some(color) = from {
        stops.push(json!({
            "position": 0.0,
            "color": [color[0], color[1], color[2], color[3]]
        }));
    }
    if let Some(color) = via {
        stops.push(json!({
            "position": 0.5,
            "color": [color[0], color[1], color[2], color[3]]
        }));
    }
    if let Some(color) = to {
        stops.push(json!({
            "position": 1.0,
            "color": [color[0], color[1], color[2], color[3]]
        }));
    }

    if stops.len() < 2 {
        return None;
    }

    Some(json!({
        "type": "GRADIENT_LINEAR",
        "start": [start[0], start[1]],
        "end": [end[0], end[1]],
        "stops": stops,
        "visible": true
    }))
}

fn resolve_text_color(
    classes: &[String],
    styles: &HashMap<String, String>,
    theme: &TailwindTheme,
) -> Option<[f32; 4]> {
    if let Some(value) = styles.get("color")
        && let Some(color) = parse_css_color_value(value, theme)
    {
        return Some(color);
    }
    for class_name in classes.iter().rev() {
        let utility = static_responsive_utility(class_name);
        let Some(token) = utility.strip_prefix("text-") else {
            continue;
        };
        if let Some(color) = resolve_color_token(token, theme) {
            return Some(color);
        }
    }
    None
}

fn parse_css_color_value(value: &str, theme: &TailwindTheme) -> Option<[f32; 4]> {
    let trimmed = value.trim();
    if let Some(color) = parse_color_literal(trimmed) {
        return Some(color);
    }
    theme
        .colors
        .get(&normalize_color_key(trimmed))
        .copied()
        .or_else(|| match trimmed.to_ascii_lowercase().as_str() {
            "black" => Some([0.0, 0.0, 0.0, 1.0]),
            "white" => Some([1.0, 1.0, 1.0, 1.0]),
            "transparent" => Some([0.0, 0.0, 0.0, 0.0]),
            _ => None,
        })
}

fn parse_css_letter_spacing_value(value: &str, font_size_px: f32) -> Option<f32> {
    let trimmed = value.trim().trim_end_matches(';').to_ascii_lowercase();
    if trimmed == "normal" {
        return Some(0.0);
    }
    if let Some(number) = trimmed.strip_suffix("em")
        && let Ok(parsed) = number.trim().parse::<f32>()
    {
        return Some(parsed * font_size_px);
    }
    if let Some(number) = trimmed.strip_suffix("rem")
        && let Ok(parsed) = number.trim().parse::<f32>()
    {
        return Some(parsed * 16.0);
    }
    if let Some(number) = trimmed.strip_suffix('%')
        && let Ok(parsed) = number.trim().parse::<f32>()
    {
        return Some(font_size_px * (parsed / 100.0));
    }
    parse_css_length(&trimmed, 0.0, 0.0)
}

fn resolve_color_token(token: &str, theme: &TailwindTheme) -> Option<[f32; 4]> {
    if token.starts_with('[') && token.ends_with(']') {
        return parse_color_literal(&token[1..token.len() - 1]);
    }

    let (base, alpha) = if let Some((base, alpha)) = token.split_once('/') {
        (base, alpha.parse::<f32>().ok().map(|value| value / 100.0))
    } else {
        (token, None)
    };

    let key = normalize_color_key(base);
    let mut color = *theme.colors.get(&key)?;
    if let Some(alpha) = alpha {
        color[3] *= alpha.clamp(0.0, 1.0);
    }
    Some(color)
}

fn normalize_color_key(key: &str) -> String {
    key.trim().to_ascii_lowercase()
}

fn resolve_opacity(node: &HtmlNode) -> Option<f32> {
    if let Some(value) = class_value(&node.classes, "opacity-")
        && let Ok(percent) = value.parse::<f32>()
    {
        return Some((percent / 100.0).clamp(0.0, 1.0));
    }
    node.styles
        .get("opacity")
        .and_then(|value| value.parse::<f32>().ok())
        .map(|value| value.clamp(0.0, 1.0))
}

fn resolve_clips_content(classes: &[String], styles: &HashMap<String, String>) -> bool {
    if classes
        .iter()
        .any(|class| matches!(class.as_str(), "overflow-hidden" | "overflow-clip"))
    {
        return true;
    }

    for key in ["overflow", "overflow-x", "overflow-y"] {
        if let Some(value) = styles.get(key) {
            let value = value.trim().to_ascii_lowercase();
            if value == "hidden" || value == "clip" {
                return true;
            }
        }
    }
    false
}

fn resolve_effects(
    classes: &[String],
    styles: &HashMap<String, String>,
    theme: &TailwindTheme,
) -> Vec<JsonValue> {
    let mut effects = Vec::new();

    if let Some(box_shadow) = styles.get("box-shadow") {
        append_box_shadow_effects(box_shadow, theme, &mut effects);
    }

    if !styles.contains_key("box-shadow") {
        if let Some(shadow_value) = resolve_tailwind_shadow(classes) {
            append_box_shadow_effects(&shadow_value, theme, &mut effects);
        }
    }

    if let Some(radius) = resolve_layer_blur_radius(classes, styles) {
        effects.push(json!({
            "type": "LAYER_BLUR",
            "radius": radius,
            "visible": true
        }));
    }

    if let Some(radius) = resolve_background_blur_radius(classes, styles) {
        effects.push(json!({
            "type": "BACKGROUND_BLUR",
            "radius": radius,
            "visible": true
        }));
    }

    effects
}

fn resolve_layer_blur_radius(classes: &[String], styles: &HashMap<String, String>) -> Option<f32> {
    if let Some(value) = styles.get("filter")
        && let Some(radius) = parse_blur_function_radius(value)
    {
        return Some(radius);
    }

    for class_name in classes.iter().rev() {
        if class_name == "blur-none" {
            return None;
        }
        if let Some(token) = class_name.strip_prefix("blur-")
            && let Some(radius) = parse_tailwind_blur_token(token)
        {
            return Some(radius);
        }
    }
    None
}

fn resolve_background_blur_radius(
    classes: &[String],
    styles: &HashMap<String, String>,
) -> Option<f32> {
    if let Some(value) = styles.get("backdrop-filter")
        && let Some(radius) = parse_blur_function_radius(value)
    {
        return Some(radius);
    }

    for class_name in classes.iter().rev() {
        if class_name == "backdrop-blur-none" {
            return None;
        }
        if let Some(token) = class_name.strip_prefix("backdrop-blur-")
            && let Some(radius) = parse_tailwind_blur_token(token)
        {
            return Some(radius);
        }
    }
    None
}

fn parse_blur_function_radius(value: &str) -> Option<f32> {
    let start = value.find("blur(")?;
    let tail = &value[start + "blur(".len()..];
    let end = tail.find(')')?;
    parse_css_length(&tail[..end], 0.0, 0.0).map(|radius| radius.max(0.0))
}

fn parse_tailwind_blur_token(token: &str) -> Option<f32> {
    match token {
        "sm" => Some(8.0),
        "md" => Some(12.0),
        "lg" => Some(16.0),
        "xl" => Some(24.0),
        "2xl" => Some(40.0),
        "3xl" => Some(64.0),
        _ if token.starts_with('[') && token.ends_with(']') => {
            parse_css_length(&token[1..token.len() - 1], 0.0, 0.0).map(|v| v.max(0.0))
        }
        _ => parse_css_length(token, 0.0, 0.0).map(|v| v.max(0.0)),
    }
}

fn resolve_tailwind_shadow(classes: &[String]) -> Option<String> {
    for class_name in classes.iter().rev() {
        if class_name == "shadow-none" {
            return None;
        }
        if let Some(raw) = class_name
            .strip_prefix("shadow-[")
            .and_then(|value| value.strip_suffix(']'))
        {
            return Some(raw.replace('_', " "));
        }
        if let Some(raw) = class_name
            .strip_prefix("drop-shadow-[")
            .and_then(|value| value.strip_suffix(']'))
        {
            return Some(raw.replace('_', " "));
        }
        let mapped = match class_name.as_str() {
            "shadow-sm" => Some("0 1px 2px rgba(0,0,0,0.25)"),
            "shadow" | "shadow-md" => Some("0 4px 6px rgba(0,0,0,0.3)"),
            "shadow-lg" => Some("0 10px 15px rgba(0,0,0,0.35)"),
            "shadow-xl" => Some("0 20px 25px rgba(0,0,0,0.35)"),
            "shadow-2xl" => Some("0 25px 50px rgba(0,0,0,0.45)"),
            _ => None,
        };
        if let Some(value) = mapped {
            return Some(value.to_string());
        }
    }
    None
}

fn append_box_shadow_effects(value: &str, theme: &TailwindTheme, out_effects: &mut Vec<JsonValue>) {
    for segment in split_css_function_aware_list(value) {
        if let Some(effect) = parse_box_shadow_effect(&segment, theme) {
            out_effects.push(effect);
        }
    }
}

fn split_css_function_aware_list(value: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0i32;
    for ch in value.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                paren_depth = (paren_depth - 1).max(0);
                current.push(ch);
            }
            ',' if paren_depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    items.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        items.push(trimmed.to_string());
    }
    items
}

fn tokenize_css_function_aware(value: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut paren_depth = 0i32;
    for ch in value.chars() {
        match ch {
            '(' => {
                paren_depth += 1;
                current.push(ch);
            }
            ')' => {
                paren_depth = (paren_depth - 1).max(0);
                current.push(ch);
            }
            c if c.is_whitespace() && paren_depth == 0 => {
                let trimmed = current.trim();
                if !trimmed.is_empty() {
                    tokens.push(trimmed.to_string());
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }
    let trimmed = current.trim();
    if !trimmed.is_empty() {
        tokens.push(trimmed.to_string());
    }
    tokens
}

fn parse_box_shadow_effect(value: &str, theme: &TailwindTheme) -> Option<JsonValue> {
    let mut lengths = Vec::new();
    let mut color = None;
    let mut has_inset = false;

    for token in tokenize_css_function_aware(value) {
        if token.eq_ignore_ascii_case("inset") {
            has_inset = true;
            continue;
        }
        if color.is_none()
            && let Some(parsed) = parse_css_color_value(&token, theme)
        {
            color = Some(parsed);
            continue;
        }
        if let Some(parsed) = parse_css_length(&token, 0.0, 0.0) {
            lengths.push(parsed);
        }
    }

    if has_inset {
        return None;
    }

    if lengths.len() < 2 {
        return None;
    }

    let offset_x = lengths[0];
    let offset_y = lengths[1];
    let blur = lengths.get(2).copied().unwrap_or(0.0).max(0.0);
    let color = color.unwrap_or([0.0, 0.0, 0.0, 0.35]);

    Some(json!({
        "type": "DROP_SHADOW",
        "offset": [offset_x, offset_y],
        "radius": blur,
        "color": [color[0], color[1], color[2], color[3]],
        "visible": true
    }))
}

fn resolve_blend_mode(classes: &[String], styles: &HashMap<String, String>) -> Option<String> {
    let token = classes
        .iter()
        .rev()
        .find_map(|class| class.strip_prefix("mix-blend-"))
        .map(ToString::to_string)
        .or_else(|| styles.get("mix-blend-mode").cloned())?;
    let mode = match token.as_str() {
        "normal" => "NORMAL",
        "screen" => "SCREEN",
        "overlay" => "OVERLAY",
        "difference" => "DIFFERENCE",
        "color-dodge" => "COLOR_DODGE",
        "plus-lighter" => "LINEAR_DODGE",
        _ => return None,
    };
    Some(mode.to_string())
}

fn resolve_corner_radius(
    classes: &[String],
    styles: &HashMap<String, String>,
    rect: Rect,
) -> Option<f32> {
    if let Some(value) = styles.get("border-radius")
        && let Some(radius) = parse_css_length(value, rect.width, rect.height)
    {
        return Some(radius.max(0.0));
    }
    if has_class(classes, "rounded-full") {
        return Some((rect.width.min(rect.height) * 0.5).max(0.0));
    }

    let token = classes
        .iter()
        .rev()
        .find_map(|class| class.strip_prefix("rounded-"));
    let radius = match token {
        Some("none") => 0.0,
        Some("sm") => 2.0,
        Some("md") => 6.0,
        Some("lg") => 8.0,
        Some("xl") => 12.0,
        Some("2xl") => 16.0,
        Some("3xl") => 24.0,
        Some("full") => (rect.width.min(rect.height) * 0.5).max(0.0),
        Some(raw) if raw.starts_with('[') && raw.ends_with(']') => {
            parse_css_length(&raw[1..raw.len() - 1], rect.width, rect.height).unwrap_or(0.0)
        }
        Some(_) => 4.0,
        None => {
            if has_class(classes, "rounded") {
                4.0
            } else {
                return None;
            }
        }
    };
    Some(radius)
}

fn resolve_stroke(
    classes: &[String],
    styles: &HashMap<String, String>,
    theme: &TailwindTheme,
) -> Option<(f32, [f32; 4])> {
    let mut weight = 0.0;
    if has_class(classes, "border") {
        weight = 1.0;
    }
    for class in classes {
        if let Some(token) = class.strip_prefix("border-") {
            if token == "0" {
                weight = 0.0;
                continue;
            }
            if token == "2" || token == "4" || token == "8" {
                if let Ok(parsed) = token.parse::<f32>() {
                    weight = parsed;
                }
                continue;
            }
            if token.starts_with('[') && token.ends_with(']') {
                if let Some(parsed) = parse_css_length(&token[1..token.len() - 1], 0.0, 0.0) {
                    weight = parsed.max(0.0);
                }
                continue;
            }
        }
    }
    if weight <= 0.0 {
        if let Some(width) = styles
            .get("border-width")
            .and_then(|value| parse_css_length(value, 0.0, 0.0))
        {
            weight = width.max(0.0);
        }
        for key in [
            "border",
            "border-top",
            "border-right",
            "border-bottom",
            "border-left",
        ] {
            if let Some(value) = styles.get(key)
                && let Some((parsed_weight, _)) = parse_border_declaration(value, theme)
            {
                weight = weight.max(parsed_weight.max(0.0));
            }
        }
    }
    if weight <= 0.0 {
        return None;
    }

    let stroke_color = styles
        .get("border-color")
        .and_then(|value| parse_css_color_value(value, theme))
        .or_else(|| {
            for key in [
                "border",
                "border-top",
                "border-right",
                "border-bottom",
                "border-left",
            ] {
                if let Some(value) = styles.get(key)
                    && let Some((_, color)) = parse_border_declaration(value, theme)
                {
                    return Some(color);
                }
            }
            None
        })
        .or_else(|| {
            classes
                .iter()
                .rev()
                .find_map(|class| class.strip_prefix("border-"))
                .and_then(|token| resolve_color_token(token, theme))
        })
        .or_else(|| theme.colors.get("white").copied())
        .unwrap_or([1.0, 1.0, 1.0, 1.0]);

    Some((weight, stroke_color))
}

fn parse_border_declaration(value: &str, theme: &TailwindTheme) -> Option<(f32, [f32; 4])> {
    let mut width = None;
    let mut color = None;
    for token in value.split_whitespace() {
        if width.is_none()
            && let Some(parsed) = parse_css_length(token, 0.0, 0.0)
        {
            width = Some(parsed.max(0.0));
            continue;
        }
        if color.is_none()
            && let Some(parsed) = parse_css_color_value(token, theme)
        {
            color = Some(parsed);
        }
    }
    let width = width?;
    let color = color.or_else(|| theme.colors.get("white").copied())?;
    Some((width, color))
}

fn solid_paint_json(color: [f32; 4]) -> JsonValue {
    json!({
        "type": "SOLID",
        "color": [color[0], color[1], color[2], color[3]],
        "visible": true
    })
}

fn json_number(value: f32) -> JsonValue {
    JsonValue::Number(JsonNumber::from_f64(value as f64).unwrap_or_else(|| JsonNumber::from(0)))
}

fn extract_tailwind_config_script(html: &str) -> Option<String> {
    let marker = "id=\"tailwind-config\"";
    let id_index = html.find(marker)?;
    let head = &html[..id_index];
    let script_start = head.rfind("<script")?;
    let after_open_tag = html[script_start..].find('>')? + script_start + 1;
    let close = html[after_open_tag..].find("</script>")? + after_open_tag;
    Some(html[after_open_tag..close].to_string())
}

fn extract_js_object_block<'a>(source: &'a str, key: &str) -> Option<&'a str> {
    let key_index = source.find(key)?;
    let tail = &source[key_index..];
    let brace_relative = tail.find('{')?;
    let start = key_index + brace_relative;
    let mut depth = 0i32;
    let mut end = None;
    for (offset, ch) in source[start..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = Some(start + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let end = end?;
    Some(&source[start + 1..end])
}

fn parse_js_object_pairs(block: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    let chars: Vec<char> = block.chars().collect();
    let mut index = 0usize;
    while index < chars.len() {
        while index < chars.len() && chars[index] != '"' && chars[index] != '\'' {
            index += 1;
        }
        if index >= chars.len() {
            break;
        }
        let quote = chars[index];
        index += 1;
        let key_start = index;
        while index < chars.len() && chars[index] != quote {
            index += 1;
        }
        if index >= chars.len() {
            break;
        }
        let key: String = chars[key_start..index].iter().collect();
        index += 1;

        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        if index >= chars.len() || chars[index] != ':' {
            continue;
        }
        index += 1;
        while index < chars.len() && chars[index].is_whitespace() {
            index += 1;
        }
        if index >= chars.len() {
            break;
        }

        let value = if chars[index] == '"' || chars[index] == '\'' {
            let quote = chars[index];
            index += 1;
            let start = index;
            while index < chars.len() && chars[index] != quote {
                index += 1;
            }
            let value: String = chars[start..index].iter().collect();
            if index < chars.len() {
                index += 1;
            }
            value
        } else if chars[index] == '[' {
            let start = index;
            let mut depth = 0i32;
            while index < chars.len() {
                match chars[index] {
                    '[' => depth += 1,
                    ']' => {
                        depth -= 1;
                        if depth == 0 {
                            index += 1;
                            break;
                        }
                    }
                    _ => {}
                }
                index += 1;
            }
            chars[start..index].iter().collect::<String>()
        } else {
            let start = index;
            while index < chars.len()
                && chars[index] != ','
                && chars[index] != '\n'
                && chars[index] != '}'
            {
                index += 1;
            }
            chars[start..index]
                .iter()
                .collect::<String>()
                .trim()
                .to_string()
        };
        pairs.push((key, value));
    }
    pairs
}

fn parse_first_quoted_token(value: &str) -> Option<String> {
    let mut chars = value.chars();
    let mut quote = None;
    for ch in chars.by_ref() {
        if ch == '"' || ch == '\'' {
            quote = Some(ch);
            break;
        }
    }
    let quote = quote?;
    let mut out = String::new();
    for ch in chars {
        if ch == quote {
            break;
        }
        out.push(ch);
    }
    if out.is_empty() { None } else { Some(out) }
}

fn parse_color_literal(value: &str) -> Option<[f32; 4]> {
    let raw = value.trim();
    if raw.starts_with('#') {
        return hex_to_rgba(raw);
    }
    let lower = raw.to_ascii_lowercase();
    if let Some(payload) = lower
        .strip_prefix("rgba(")
        .and_then(|text| text.strip_suffix(')'))
    {
        let parts: Vec<&str> = payload.split(',').map(str::trim).collect();
        if parts.len() != 4 {
            return None;
        }
        let r = parts[0].parse::<f32>().ok()?;
        let g = parts[1].parse::<f32>().ok()?;
        let b = parts[2].parse::<f32>().ok()?;
        let a = parts[3].parse::<f32>().ok()?;
        return Some([
            normalize_channel(r),
            normalize_channel(g),
            normalize_channel(b),
            a.clamp(0.0, 1.0),
        ]);
    }
    if let Some(payload) = lower
        .strip_prefix("rgb(")
        .and_then(|text| text.strip_suffix(')'))
    {
        let parts: Vec<&str> = payload.split(',').map(str::trim).collect();
        if parts.len() != 3 {
            return None;
        }
        let r = parts[0].parse::<f32>().ok()?;
        let g = parts[1].parse::<f32>().ok()?;
        let b = parts[2].parse::<f32>().ok()?;
        return Some([
            normalize_channel(r),
            normalize_channel(g),
            normalize_channel(b),
            1.0,
        ]);
    }
    None
}

fn normalize_channel(value: f32) -> f32 {
    if value > 1.0 {
        (value / 255.0).clamp(0.0, 1.0)
    } else {
        value.clamp(0.0, 1.0)
    }
}

fn hex_to_rgba(hex: &str) -> Option<[f32; 4]> {
    let text = hex.trim().trim_start_matches('#');
    let bytes = match text.len() {
        3 => {
            let r = parse_hex_nibble(text.as_bytes()[0])? * 17;
            let g = parse_hex_nibble(text.as_bytes()[1])? * 17;
            let b = parse_hex_nibble(text.as_bytes()[2])? * 17;
            [r, g, b, 255]
        }
        4 => {
            let r = parse_hex_nibble(text.as_bytes()[0])? * 17;
            let g = parse_hex_nibble(text.as_bytes()[1])? * 17;
            let b = parse_hex_nibble(text.as_bytes()[2])? * 17;
            let a = parse_hex_nibble(text.as_bytes()[3])? * 17;
            [r, g, b, a]
        }
        6 => {
            let bytes = text.as_bytes();
            [
                parse_hex_byte(bytes[0], bytes[1])?,
                parse_hex_byte(bytes[2], bytes[3])?,
                parse_hex_byte(bytes[4], bytes[5])?,
                255,
            ]
        }
        8 => {
            let bytes = text.as_bytes();
            [
                parse_hex_byte(bytes[0], bytes[1])?,
                parse_hex_byte(bytes[2], bytes[3])?,
                parse_hex_byte(bytes[4], bytes[5])?,
                parse_hex_byte(bytes[6], bytes[7])?,
            ]
        }
        _ => return None,
    };
    Some([
        bytes[0] as f32 / 255.0,
        bytes[1] as f32 / 255.0,
        bytes[2] as f32 / 255.0,
        bytes[3] as f32 / 255.0,
    ])
}

fn parse_hex_nibble(value: u8) -> Option<u8> {
    match value {
        b'0'..=b'9' => Some(value - b'0'),
        b'a'..=b'f' => Some(value - b'a' + 10),
        b'A'..=b'F' => Some(value - b'A' + 10),
        _ => None,
    }
}

fn parse_hex_byte(high: u8, low: u8) -> Option<u8> {
    Some((parse_hex_nibble(high)? << 4) | parse_hex_nibble(low)?)
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
            "stitch_scene".to_string(),
            "--input-html".to_string(),
            "artifacts/stitch/code.normalized.html".to_string(),
            "--output-json".to_string(),
            "artifacts/stitch/scene.json".to_string(),
            "--viewport-width".to_string(),
            "1440".to_string(),
            "--viewport-height".to_string(),
            "900".to_string(),
            "--module-out".to_string(),
            "examples/generated/stitch_scene_module.rs".to_string(),
            "--module-name".to_string(),
            "stitch_scene_generated".to_string(),
            "--document-fn".to_string(),
            "document".to_string(),
            "--runtime-fn".to_string(),
            "runtime".to_string(),
        ])
        .expect("arguments should parse");
        assert_eq!(
            options.input_html,
            PathBuf::from("artifacts/stitch/code.normalized.html")
        );
        assert_eq!(
            options.output_json,
            PathBuf::from("artifacts/stitch/scene.json")
        );
        assert_eq!(options.viewport_width, 1440.0);
        assert_eq!(options.viewport_height, Some(900.0));
        assert_eq!(
            options.module_out,
            Some(PathBuf::from("examples/generated/stitch_scene_module.rs"))
        );
        assert_eq!(options.module_name, "stitch_scene_generated");
        assert_eq!(options.document_fn, "document");
        assert_eq!(options.runtime_fn, "runtime");
    }

    #[test]
    fn parse_args_requires_input_and_output() {
        let missing_input = parse_args([
            "stitch_scene".to_string(),
            "--output-json".to_string(),
            "scene.json".to_string(),
        ])
        .expect_err("missing input should fail");
        assert!(missing_input.contains("missing required --input-html"));

        let missing_output = parse_args([
            "stitch_scene".to_string(),
            "--input-html".to_string(),
            "code.normalized.html".to_string(),
        ])
        .expect_err("missing output should fail");
        assert!(missing_output.contains("missing required --output-json"));
    }

    #[test]
    fn resolve_length_token_handles_tailwind_forms() {
        assert_eq!(resolve_length_token("full", 320.0, 1000.0), Some(320.0));
        assert_eq!(resolve_length_token("screen", 320.0, 1000.0), Some(1000.0));
        assert_eq!(resolve_length_token("12", 320.0, 1000.0), Some(48.0));
        assert_eq!(resolve_length_token("1/2", 320.0, 1000.0), Some(160.0));
        assert_eq!(resolve_length_token("[85%]", 300.0, 900.0), Some(255.0));
        assert_eq!(resolve_length_token("[85vh]", 300.0, 900.0), Some(765.0));
        assert_eq!(resolve_length_token("-3", 300.0, 900.0), Some(-12.0));
    }

    #[test]
    fn parse_color_literal_supports_hex_and_rgba() {
        assert_eq!(parse_color_literal("#ccff00"), Some([0.8, 1.0, 0.0, 1.0]));
        let rgba = parse_color_literal("rgba(40, 40, 40, 0.9)").expect("rgba should parse");
        assert!((rgba[0] - (40.0 / 255.0)).abs() < 1e-6);
        assert!((rgba[3] - 0.9).abs() < 1e-6);
    }

    #[test]
    fn extract_theme_parses_colors_and_fonts() {
        let html = r##"
<script id="tailwind-config">
tailwind.config = { theme: { extend: {
  colors: { "primary": "#ccff00", "paper": "rgba(10, 20, 30, 0.5)" },
  fontFamily: { "display": ["Space Grotesk", "sans-serif"] },
  backgroundImage: { "noise": "url('data:image/svg+xml;base64,AAAA')" }
}}}
</script>
"##;
        let theme = TailwindTheme::from_html_source(html);
        assert!(theme.colors.contains_key("primary"));
        assert!(theme.colors.contains_key("paper"));
        assert_eq!(
            theme.fonts.get("display"),
            Some(&"Space Grotesk".to_string())
        );
        assert!(theme.background_images.contains_key("noise"));
    }

    #[test]
    fn resolve_background_image_ref_maps_svg_data_uri_to_procedural_noise() {
        let mut theme = TailwindTheme::defaults();
        theme.background_images.insert(
            "noise".to_string(),
            "url('data:image/svg+xml;base64,AAAA')".to_string(),
        );
        let classes = vec!["bg-noise".to_string()];
        let styles = HashMap::new();
        let image_ref = resolve_background_image_ref(&classes, &styles, &theme)
            .expect("bg-noise should map to procedural reference");
        assert!(image_ref.starts_with("procedural://noise/"));
    }

    #[test]
    fn resolve_image_scale_mode_prefers_tile_for_noise_patterns() {
        let classes = vec!["bg-noise".to_string()];
        let styles = HashMap::new();
        assert_eq!(
            resolve_image_scale_mode(&classes, &styles, "procedural://noise/abcd"),
            "TILE"
        );
    }

    #[test]
    fn resolve_image_scale_mode_honors_cover_and_repeat() {
        let cover_classes = vec!["bg-cover".to_string()];
        let no_styles = HashMap::new();
        assert_eq!(
            resolve_image_scale_mode(&cover_classes, &no_styles, "assets/photo.png"),
            "FILL"
        );

        let repeat_classes = vec!["bg-repeat".to_string()];
        assert_eq!(
            resolve_image_scale_mode(&repeat_classes, &no_styles, "assets/pattern.png"),
            "TILE"
        );

        let mut inline_repeat = HashMap::new();
        inline_repeat.insert("background-repeat".to_string(), "repeat".to_string());
        assert_eq!(
            resolve_image_scale_mode(&Vec::new(), &inline_repeat, "assets/pattern.png"),
            "TILE"
        );
    }

    #[test]
    fn resolve_z_index_supports_tailwind_and_inline_forms() {
        let classes = vec![
            "z-40".to_string(),
            "z-[60]".to_string(),
            "-z-10".to_string(),
        ];
        let styles = HashMap::new();
        assert_eq!(resolve_z_index(&classes, &styles), -10);

        let classes = vec!["z-40".to_string(), "z-[60]".to_string()];
        assert_eq!(resolve_z_index(&classes, &styles), 60);

        let classes = vec!["z-40".to_string()];
        let styles = HashMap::from([("z-index".to_string(), "999".to_string())]);
        assert_eq!(resolve_z_index(&classes, &styles), 999);
    }

    #[test]
    fn node_to_figma_json_skips_group_blend_on_surface_less_frames() {
        let node = HtmlNode {
            tag: "div".to_string(),
            classes: vec!["mix-blend-color-dodge".to_string()],
            styles: HashMap::new(),
            text: String::new(),
            children: vec![HtmlNode {
                tag: "span".to_string(),
                classes: vec!["text-white".to_string()],
                styles: HashMap::new(),
                text: "child".to_string(),
                children: Vec::new(),
            }],
        };
        let theme = TailwindTheme::defaults();
        let json = node_to_figma_json(
            &node,
            "stitch:group-blend",
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 200.0,
                height: 80.0,
            },
            &theme,
            Size {
                width: 800.0,
                height: 600.0,
            },
            &TextStyle::default(),
        );
        let object = json.as_object().expect("node json should be object");
        assert!(
            object.get("blendMode").is_none(),
            "surface-less FRAME should not emit blendMode until group compositing is supported"
        );
    }

    #[test]
    fn layout_tree_propagates_surface_less_group_blend_to_surface_descendants() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="mix-blend-plus-lighter">
      <span class="text-white">THE PIT</span>
    </div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 640.0,
            height: 360.0,
        };
        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );
        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let parent = nodes
            .iter()
            .find(|node| node.get("id") == Some(&JsonValue::String("stitch:0.0".to_string())))
            .and_then(JsonValue::as_object)
            .expect("parent frame should exist");
        assert!(
            parent.get("blendMode").is_none(),
            "surface-less parent frame should still omit blend mode"
        );

        let text = nodes
            .iter()
            .find(|node| node.get("characters") == Some(&JsonValue::String("THE PIT".to_string())))
            .and_then(JsonValue::as_object)
            .expect("text node should exist");
        assert_eq!(
            text.get("blendMode"),
            Some(&JsonValue::String("LINEAR_DODGE".to_string())),
            "descendant surface should inherit parent blend mode when group blend is skipped"
        );
    }

    #[test]
    fn resolve_effective_blend_mode_prefers_local_request_and_keeps_propagation() {
        let resolved = resolve_effective_blend_mode(Some("OVERLAY"), Some("SCREEN"), true, true);
        assert_eq!(resolved.applied, Some("OVERLAY".to_string()));
        assert_eq!(resolved.propagated, Some("OVERLAY".to_string()));
    }

    #[test]
    fn resolve_effective_blend_mode_carries_inherited_until_surface_exists() {
        let unresolved = resolve_effective_blend_mode(None, Some("SCREEN"), true, false);
        assert!(unresolved.applied.is_none());
        assert_eq!(unresolved.propagated, Some("SCREEN".to_string()));

        let resolved = resolve_effective_blend_mode(None, Some("SCREEN"), false, false);
        assert_eq!(resolved.applied, Some("SCREEN".to_string()));
        assert_eq!(resolved.propagated, Some("SCREEN".to_string()));
    }

    #[test]
    fn node_to_figma_json_skips_effects_on_surface_less_frames() {
        let node = HtmlNode {
            tag: "h2".to_string(),
            classes: vec!["shadow-[4px_4px_0_#ccff00]".to_string()],
            styles: HashMap::new(),
            text: String::new(),
            children: vec![HtmlNode {
                tag: "span".to_string(),
                classes: vec!["text-white".to_string()],
                styles: HashMap::new(),
                text: "SCUM".to_string(),
                children: Vec::new(),
            }],
        };
        let theme = TailwindTheme::defaults();
        let json = node_to_figma_json(
            &node,
            "stitch:group-effect",
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1200.0,
                height: 180.0,
            },
            &theme,
            Size {
                width: 1366.0,
                height: 884.0,
            },
            &TextStyle::default(),
        );
        let object = json.as_object().expect("node json should be object");
        assert!(
            object.get("effects").is_none(),
            "surface-less FRAME should not emit rectangle effects until group compositing is supported"
        );
    }

    #[test]
    fn node_to_figma_json_applies_inherited_effects_to_text_nodes() {
        let node = HtmlNode {
            tag: "span".to_string(),
            classes: vec!["bg-white".to_string(), "text-black".to_string()],
            styles: HashMap::new(),
            text: "SCUM".to_string(),
            children: Vec::new(),
        };
        let inherited_effects = vec![json!({
            "type": "DROP_SHADOW",
            "offset": [4.0, 4.0],
            "radius": 0.0,
            "color": [0.8, 1.0, 0.0, 1.0],
            "visible": true
        })];
        let json = node_to_figma_json_with_inherited_effects(
            &node,
            "stitch:inherited-effects",
            None,
            Rect {
                x: 24.0,
                y: 430.0,
                width: 177.28,
                height: 72.0,
            },
            &TailwindTheme::defaults(),
            Size {
                width: 1366.0,
                height: 884.0,
            },
            &TextStyle {
                color: Some([0.0, 0.0, 0.0, 1.0]),
                ..TextStyle::default()
            },
            &inherited_effects,
            None,
        )
        .json;
        let object = json.as_object().expect("node json should be an object");
        let effects = object
            .get("effects")
            .and_then(JsonValue::as_array)
            .expect("text node should carry inherited effects");
        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0]["offset"],
            JsonValue::Array(vec![json_number(4.0), json_number(4.0)])
        );
    }

    #[test]
    fn layout_tree_propagates_surface_less_effects_to_text_descendants() {
        let root = HtmlNode {
            tag: "h2".to_string(),
            classes: vec![
                "text-5xl".to_string(),
                "font-ransom".to_string(),
                "drop-shadow-[4px_4px_0_#ccff00]".to_string(),
            ],
            styles: HashMap::new(),
            text: String::new(),
            children: vec![HtmlNode {
                tag: "span".to_string(),
                classes: vec!["bg-white".to_string(), "text-black".to_string()],
                styles: HashMap::new(),
                text: "SCUM".to_string(),
                children: Vec::new(),
            }],
        };
        let viewport = Size {
            width: 1366.0,
            height: 884.0,
        };
        let mut cache = HashMap::new();
        let theme = TailwindTheme::defaults();

        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut nodes = Vec::new();
        let mut layout_path = vec![0usize];
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 24.0,
                y: 420.0,
                width: 600.0,
                height: 200.0,
            },
            viewport,
            &cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let scum = nodes
            .iter()
            .find(|node| node.get("characters").and_then(JsonValue::as_str) == Some("SCUM"))
            .and_then(JsonValue::as_object)
            .expect("SCUM node should be emitted");
        let effects = scum
            .get("effects")
            .and_then(JsonValue::as_array)
            .expect("SCUM text should inherit wrapper drop shadow");
        assert!(
            effects.iter().any(|effect| {
                effect.get("type").and_then(JsonValue::as_str) == Some("DROP_SHADOW")
                    && effect
                        .get("offset")
                        .and_then(JsonValue::as_array)
                        .is_some_and(|offset| {
                            offset.len() == 2
                                && offset[0].as_f64().is_some_and(|v| (v - 4.0).abs() < 1e-3)
                                && offset[1].as_f64().is_some_and(|v| (v - 4.0).abs() < 1e-3)
                        })
            }),
            "expected inherited green drop shadow to land on descendant text"
        );
    }

    #[test]
    fn parse_clip_path_polygon_supports_calc_offsets() {
        let path = parse_clip_path_polygon(
            "polygon(0% 0%, 100% 0%, 100% calc(100% - 10px), 0% calc(100% - 10px))",
            1366.0,
            16.0,
        )
        .expect("clip-path polygon should parse");
        assert!(path.starts_with("M 0 0"));
        assert!(path.contains("1366 6"));
        assert!(path.ends_with(" Z"));
    }

    #[test]
    fn parse_class_style_rules_extracts_simple_class_blocks() {
        let html = r#"
<style>
  .halftone { filter: grayscale(100%) contrast(150%) invert(100%); mix-blend-mode: difference; }
  .halftone::after { mix-blend-mode: overlay; opacity: 0.15; }
  .halftone:hover { filter: none; }
  .halftone.active { opacity: 0.9; }
  .tape-strip { background-color: rgba(30,30,30,0.95); color: #f2f2f2; }
</style>
"#;
        let rules = parse_class_style_rules(html);
        let halftone = rules.get("halftone").expect("halftone class should exist");
        assert_eq!(
            halftone.get("filter").map(String::as_str),
            Some("grayscale(100%) contrast(150%) invert(100%)")
        );
        assert_eq!(
            halftone.get("mix-blend-mode").map(String::as_str),
            Some("difference")
        );
        assert_eq!(halftone.get("opacity"), None);
        assert_eq!(
            rules
                .get("tape-strip")
                .and_then(|styles| styles.get("background-color"))
                .map(String::as_str),
            Some("rgba(30,30,30,0.95)")
        );
    }

    #[test]
    fn discover_stylesheet_hrefs_selects_stylesheets_only() {
        let html = r#"
<head>
  <link rel="stylesheet" href="assets/fonts.css"/>
  <link rel="preload stylesheet" href="assets/theme.css"/>
  <link rel="icon" href="assets/favicon.png"/>
  <link href="assets/missing-rel.css"/>
</head>
"#;
        let hrefs = discover_stylesheet_hrefs(html);
        assert_eq!(
            hrefs,
            vec![
                "assets/fonts.css".to_string(),
                "assets/theme.css".to_string()
            ]
        );
    }

    #[test]
    fn resolve_class_style_rules_merges_linked_local_stylesheets() {
        let unique = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("arthropod_stitch_scene_css_{unique}"));
        let assets = base.join("assets");
        std::fs::create_dir_all(&assets).expect("create css assets directory");

        let css_path = assets.join("fonts.css");
        std::fs::write(
            &css_path,
            ".material-symbols-outlined { font-family: 'Material Symbols Outlined'; font-size: 24px; }",
        )
        .expect("write css file");

        let html_path = base.join("code.normalized.html");
        let html = r#"
<!DOCTYPE html>
<html>
  <head>
    <link rel="stylesheet" href="assets/fonts.css"/>
  </head>
  <body>
    <span class="material-symbols-outlined">skull</span>
  </body>
</html>
"#;
        std::fs::write(&html_path, html).expect("write html file");

        let rules = resolve_class_style_rules(html, &html_path);
        let icon_class = rules
            .get("material-symbols-outlined")
            .expect("expected linked stylesheet class rules");
        assert_eq!(
            icon_class.get("font-family").map(String::as_str),
            Some("'Material Symbols Outlined'")
        );
        assert_eq!(
            icon_class.get("font-size").map(String::as_str),
            Some("24px")
        );

        std::fs::remove_dir_all(base).expect("cleanup temp directory");
    }

    #[test]
    fn resolve_text_style_keeps_ligature_icon_text_case_original() {
        let node = HtmlNode {
            tag: "span".to_string(),
            classes: vec!["uppercase".to_string()],
            styles: HashMap::from([(
                "font-family".to_string(),
                "'Material Symbols Outlined'".to_string(),
            )]),
            text: "search".to_string(),
            children: Vec::new(),
        };

        let style = resolve_text_style(&node, &TailwindTheme::defaults(), &TextStyle::default());
        assert_eq!(
            style.font_family.as_deref(),
            Some("Material Symbols Outlined")
        );
        assert_eq!(
            style.text_case, None,
            "icon ligatures must keep source text casing"
        );
    }

    #[test]
    fn resolve_text_style_preserves_text_case_for_non_icon_fonts() {
        let node = HtmlNode {
            tag: "span".to_string(),
            classes: vec!["uppercase".to_string()],
            styles: HashMap::from([("font-family".to_string(), "'Inter'".to_string())]),
            text: "search".to_string(),
            children: Vec::new(),
        };

        let style = resolve_text_style(&node, &TailwindTheme::defaults(), &TextStyle::default());
        assert_eq!(style.font_family.as_deref(), Some("Inter"));
        assert_eq!(style.text_case.as_deref(), Some("UPPER"));
    }

    #[test]
    fn collect_text_with_line_breaks_uses_direct_text_only() {
        let html = r#"<h2>A <span>SCUM</span><br/><span>FUCKS</span></h2>"#;
        let document = Html::parse_fragment(html);
        let selector = Selector::parse("h2").expect("selector should parse");
        let heading = document
            .select(&selector)
            .next()
            .expect("h2 should exist in fragment");
        assert_eq!(collect_text_with_line_breaks(heading), "A");
    }

    #[test]
    fn build_html_node_treats_br_as_line_break_not_child_node() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <h2><span>SCUM</span><br/><span>FUCKS</span></h2>
  </body>
</html>
"#;
        let document = Html::parse_document(html);
        let selector = Selector::parse("h2").expect("h2 selector should parse");
        let heading = document.select(&selector).next().expect("h2 should exist");
        let node = build_html_node(heading, &ClassStyleMap::new()).expect("node should build");
        assert_eq!(
            node.children.len(),
            2,
            "br should not materialize as child node"
        );
        assert_eq!(node.children[0].tag, "span");
        assert_eq!(node.children[1].tag, "span");
    }

    #[test]
    fn build_html_node_skips_hidden_nodes_by_default() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="root">
      <span class="hidden">Hidden Label</span>
      <span class="block">Visible Label</span>
    </div>
  </body>
</html>
"#;
        let document = Html::parse_document(html);
        let selector = Selector::parse(".root").expect("selector should parse");
        let element = document
            .select(&selector)
            .next()
            .expect("root should exist");
        let node = build_html_node(element, &ClassStyleMap::new()).expect("node should build");
        assert_eq!(node.children.len(), 1, "hidden child should be omitted");
        assert_eq!(node.children[0].text, "Visible Label");
    }

    #[test]
    fn build_html_node_materializes_halftone_pseudo_overlay() {
        let html = r#"
<!DOCTYPE html>
<html>
  <head>
    <style>
      .halftone { filter: grayscale(100%) contrast(150%) invert(100%); }
    </style>
  </head>
  <body>
    <div class="halftone w-full h-full" style="background-image:url('assets/a.png')"></div>
  </body>
</html>
"#;
        let document = Html::parse_document(html);
        let selector = Selector::parse(".halftone").expect("selector should parse");
        let class_styles = parse_class_style_rules(html);
        let element = document
            .select(&selector)
            .next()
            .expect("halftone element should exist");
        let node = build_html_node(element, &class_styles).expect("node should build");
        assert_eq!(
            node.children.len(),
            0,
            "halftone with background image should compose pseudo overlay into source ref"
        );
    }

    #[test]
    fn node_to_figma_json_rewrites_halftone_image_source_to_composite_ref() {
        let node = HtmlNode {
            tag: "div".to_string(),
            classes: vec![
                "halftone".to_string(),
                "bg-cover".to_string(),
                "bg-center".to_string(),
            ],
            styles: HashMap::from([
                (
                    "background-image".to_string(),
                    "url('assets/sample.png')".to_string(),
                ),
                (
                    "filter".to_string(),
                    "grayscale(100%) contrast(150%) invert(100%)".to_string(),
                ),
            ]),
            text: String::new(),
            children: Vec::new(),
        };
        let json = node_to_figma_json(
            &node,
            "stitch:test",
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 100.0,
            },
            &TailwindTheme::defaults(),
            Size {
                width: 100.0,
                height: 100.0,
            },
            &TextStyle::default(),
        );
        let fills = json
            .get("fills")
            .and_then(JsonValue::as_array)
            .expect("fills should exist");
        let image_fill = fills
            .iter()
            .find(|fill| fill.get("type").and_then(JsonValue::as_str) == Some("IMAGE"))
            .expect("expected image fill");
        let source_ref = image_fill
            .get("imageSourceRef")
            .and_then(JsonValue::as_str)
            .expect("expected imageSourceRef");
        assert_eq!(source_ref, "composite://halftone/assets/sample.png");
    }

    #[test]
    fn resolve_image_filter_parses_css_filter_chain() {
        let mut styles = HashMap::new();
        styles.insert(
            "filter".to_string(),
            "grayscale(100%) contrast(150%) invert(100%)".to_string(),
        );
        let filter = resolve_image_filter(&styles).expect("filter should parse");
        assert_eq!(filter.grayscale, Some(1.0));
        assert_eq!(filter.contrast, Some(1.5));
        assert_eq!(filter.invert, Some(1.0));
    }

    #[test]
    fn resolve_offset_supports_negative_tailwind_position_utilities() {
        let node = HtmlNode {
            tag: "div".to_string(),
            classes: vec!["absolute".to_string(), "-top-3".to_string()],
            styles: HashMap::new(),
            text: String::new(),
            children: Vec::new(),
        };
        let parent = Size {
            width: 400.0,
            height: 200.0,
        };
        let viewport = Size {
            width: 1366.0,
            height: 884.0,
        };

        let top = resolve_offset(&node, "top", parent, viewport).expect("top offset should parse");
        assert!((top + 12.0).abs() < 1e-3);
    }

    #[test]
    fn resolve_rotation_degrees_supports_tailwind_and_inline_transform() {
        let class_rotation = resolve_rotation_degrees(
            &vec!["transform".to_string(), "-rotate-3".to_string()],
            &HashMap::new(),
        )
        .expect("class rotation should parse");
        assert!((class_rotation + 3.0).abs() < 1e-3);

        let mut styles = HashMap::new();
        styles.insert(
            "transform".to_string(),
            "translateX(10px) rotate(12deg) scale(1.1)".to_string(),
        );
        let style_rotation = resolve_rotation_degrees(&Vec::new(), &styles)
            .expect("inline transform rotation should parse");
        assert!((style_rotation - 12.0).abs() < 1e-3);
    }

    #[test]
    fn node_to_figma_json_emits_rotation_for_rotated_nodes() {
        let node = HtmlNode {
            tag: "div".to_string(),
            classes: vec!["transform".to_string(), "rotate-12".to_string()],
            styles: HashMap::new(),
            text: String::new(),
            children: Vec::new(),
        };
        let json = node_to_figma_json(
            &node,
            "stitch:rotated",
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 100.0,
                height: 40.0,
            },
            &TailwindTheme::defaults(),
            Size {
                width: 1366.0,
                height: 884.0,
            },
            &TextStyle::default(),
        );
        let object = json.as_object().expect("node json should be an object");
        assert_eq!(object.get("rotation"), Some(&json_number(12.0)));
    }

    #[test]
    fn resolve_font_family_ignores_weight_utilities_when_alias_present() {
        let classes = vec!["font-typewriter".to_string(), "font-bold".to_string()];
        let theme = TailwindTheme::defaults();
        let resolved = resolve_font_family(&classes, &HashMap::new(), &theme);
        assert_eq!(resolved.as_deref(), Some("Special Elite"));
    }

    #[test]
    fn resolve_clips_content_detects_hidden_overflow() {
        let classes = vec!["overflow-hidden".to_string()];
        let styles = HashMap::new();
        assert!(resolve_clips_content(&classes, &styles));

        let classes = Vec::<String>::new();
        let mut styles = HashMap::new();
        styles.insert("overflow".to_string(), "hidden".to_string());
        assert!(resolve_clips_content(&classes, &styles));

        let mut styles = HashMap::new();
        styles.insert("overflow-y".to_string(), "clip".to_string());
        assert!(resolve_clips_content(&classes, &styles));
    }

    #[test]
    fn resolve_effects_parses_box_shadow_variants() {
        let theme = TailwindTheme::defaults();
        let classes = vec!["shadow-[4px_4px_0_#ccff00]".to_string()];
        let styles = HashMap::new();
        let effects = resolve_effects(&classes, &styles, &theme);
        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0]["type"],
            JsonValue::String("DROP_SHADOW".to_string())
        );
        assert_eq!(
            effects[0]["offset"],
            JsonValue::Array(vec![json_number(4.0), json_number(4.0)])
        );
        assert_eq!(effects[0]["radius"], json_number(0.0));

        let classes = Vec::<String>::new();
        let mut styles = HashMap::new();
        styles.insert(
            "box-shadow".to_string(),
            "0 1px 3px rgba(0,0,0,0.8)".to_string(),
        );
        let effects = resolve_effects(&classes, &styles, &theme);
        assert_eq!(effects.len(), 1);
        assert_eq!(
            effects[0]["offset"],
            JsonValue::Array(vec![json_number(0.0), json_number(1.0)])
        );
        assert_eq!(effects[0]["radius"], json_number(3.0));
    }

    #[test]
    fn resolve_effects_parses_layer_and_background_blur() {
        let theme = TailwindTheme::defaults();
        let classes = vec!["blur-sm".to_string(), "backdrop-blur-sm".to_string()];
        let styles = HashMap::new();
        let effects = resolve_effects(&classes, &styles, &theme);
        assert!(effects.iter().any(|effect| {
            effect
                .as_object()
                .and_then(|obj| obj.get("type"))
                .is_some_and(|value| value == "LAYER_BLUR")
        }));
        assert!(effects.iter().any(|effect| {
            effect
                .as_object()
                .and_then(|obj| obj.get("type"))
                .is_some_and(|value| value == "BACKGROUND_BLUR")
        }));
    }

    #[test]
    fn resolve_font_size_ignores_text_color_tokens() {
        let classes = vec!["text-5xl".to_string(), "text-white".to_string()];
        let styles = HashMap::new();
        assert_eq!(resolve_font_size(&classes, &styles), Some(48.0));
    }

    #[test]
    fn resolve_text_color_skips_text_size_tokens_and_finds_color_token() {
        let theme = TailwindTheme::defaults();
        let classes = vec![
            "material-symbols-outlined".to_string(),
            "text-primary".to_string(),
            "text-[20px]".to_string(),
        ];
        let styles = HashMap::new();
        assert_eq!(
            resolve_text_color(&classes, &styles, &theme),
            Some([0.8, 1.0, 0.0, 1.0])
        );
    }

    #[test]
    fn resolve_font_size_prefers_text_utility_over_class_style_font_size() {
        let classes = vec![
            "material-symbols-outlined".to_string(),
            "text-[20px]".to_string(),
        ];
        let styles = HashMap::from([("font-size".to_string(), "24px".to_string())]);
        assert_eq!(resolve_font_size(&classes, &styles), Some(20.0));
    }

    #[test]
    fn resolve_letter_spacing_maps_tracking_widest_from_font_size() {
        let classes = vec!["tracking-widest".to_string()];
        let styles = HashMap::new();
        let spacing = resolve_letter_spacing(&classes, &styles, 18.0)
            .expect("tracking-widest should map to spacing");
        assert!((spacing - 1.8).abs() < 1e-6);
    }

    #[test]
    fn estimate_text_size_treats_ligature_icon_tokens_as_single_glyph_width() {
        let icon = estimate_text_size("graphic_eq", 24.0, 1.0, true, 0.0);
        let plain = estimate_text_size("graphic_eq", 24.0, 1.0, false, 0.0);
        assert!(icon.width <= 24.0 + f32::EPSILON);
        assert!(plain.width > icon.width * 2.0);
    }

    #[test]
    fn estimate_text_size_includes_tracking_in_width() {
        let no_tracking = estimate_text_size("\"RAT POISON\"", 18.0, 1.0, false, 0.0);
        let with_tracking = estimate_text_size("\"RAT POISON\"", 18.0, 1.0, false, 1.8);
        assert!(
            with_tracking.width > no_tracking.width + 15.0,
            "tracking should noticeably increase measured text width"
        );
    }

    #[test]
    fn resolve_font_size_supports_responsive_and_arbitrary_tokens() {
        let classes = vec!["md:text-7xl".to_string(), "text-white".to_string()];
        let styles = HashMap::new();
        assert_eq!(resolve_font_size(&classes, &styles), Some(72.0));

        let classes = vec!["text-[1.25rem]".to_string()];
        assert_eq!(resolve_font_size(&classes, &styles), Some(20.0));
    }

    #[test]
    fn node_to_figma_json_emits_fill_geometry_for_clip_path() {
        let node = HtmlNode {
            tag: "div".to_string(),
            classes: vec!["bg-black".to_string()],
            styles: HashMap::from([(
                "clip-path".to_string(),
                "polygon(0% 0%, 100% 0%, 100% calc(100% - 10px), 0% calc(100% - 10px))".to_string(),
            )]),
            text: String::new(),
            children: Vec::new(),
        };
        let theme = TailwindTheme::defaults();
        let json = node_to_figma_json(
            &node,
            "stitch:test",
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: 1366.0,
                height: 16.0,
            },
            &theme,
            Size {
                width: 1366.0,
                height: 884.0,
            },
            &TextStyle::default(),
        );
        let object = json.as_object().expect("node json should be an object");
        let geometry = object
            .get("fillGeometry")
            .and_then(JsonValue::as_array)
            .expect("fillGeometry should be emitted");
        assert_eq!(geometry.len(), 1);
        let path = geometry[0]
            .as_str()
            .expect("fill geometry item should be an svg path string");
        assert!(path.contains("1366 6"));
    }

    #[test]
    fn node_to_figma_json_text_nodes_emit_text_fill_then_background_fill() {
        let node = HtmlNode {
            tag: "span".to_string(),
            classes: vec!["bg-white".to_string(), "text-black".to_string()],
            styles: HashMap::new(),
            text: "SCUM".to_string(),
            children: Vec::new(),
        };
        let theme = TailwindTheme::defaults();
        let json = node_to_figma_json(
            &node,
            "stitch:text-fill",
            None,
            Rect {
                x: 24.0,
                y: 400.0,
                width: 200.0,
                height: 72.0,
            },
            &theme,
            Size {
                width: 1366.0,
                height: 884.0,
            },
            &TextStyle {
                color: Some([0.0, 0.0, 0.0, 1.0]),
                ..TextStyle::default()
            },
        );
        let object = json.as_object().expect("node json should be an object");
        let fills = object
            .get("fills")
            .and_then(JsonValue::as_array)
            .expect("text node should contain a fill");
        assert_eq!(
            fills.len(),
            2,
            "text nodes should emit glyph color first and background color second"
        );
        assert_eq!(
            fills[0]["color"],
            JsonValue::Array(vec![
                json_number(0.0),
                json_number(0.0),
                json_number(0.0),
                json_number(1.0)
            ])
        );
        assert_eq!(
            fills[1]["color"],
            JsonValue::Array(vec![
                json_number(1.0),
                json_number(1.0),
                json_number(1.0),
                json_number(1.0)
            ])
        );
    }

    #[test]
    fn node_to_figma_json_text_nodes_without_background_emit_transparent_backfill() {
        let node = HtmlNode {
            tag: "span".to_string(),
            classes: vec!["text-white".to_string()],
            styles: HashMap::new(),
            text: "LOUD".to_string(),
            children: Vec::new(),
        };
        let theme = TailwindTheme::defaults();
        let json = node_to_figma_json(
            &node,
            "stitch:text-transparent-backfill",
            None,
            Rect {
                x: 24.0,
                y: 400.0,
                width: 200.0,
                height: 72.0,
            },
            &theme,
            Size {
                width: 1366.0,
                height: 884.0,
            },
            &TextStyle {
                color: Some([1.0, 1.0, 1.0, 1.0]),
                ..TextStyle::default()
            },
        );
        let object = json.as_object().expect("node json should be an object");
        let fills = object
            .get("fills")
            .and_then(JsonValue::as_array)
            .expect("text node should contain fills");
        assert_eq!(fills.len(), 2);
        assert_eq!(
            fills[1]["color"],
            JsonValue::Array(vec![
                json_number(0.0),
                json_number(0.0),
                json_number(0.0),
                json_number(0.0)
            ]),
            "text without explicit background should not paint a backing rectangle"
        );
    }

    #[test]
    fn resolve_background_gradient_paint_parses_tailwind_gradient_classes() {
        let theme = TailwindTheme::defaults();
        let classes = vec![
            "bg-gradient-to-t".to_string(),
            "from-black".to_string(),
            "via-black/50".to_string(),
            "to-transparent".to_string(),
        ];

        let paint = resolve_background_gradient_paint(&classes, &theme)
            .expect("gradient classes should produce a paint");
        assert_eq!(
            paint["type"],
            JsonValue::String("GRADIENT_LINEAR".to_string())
        );
        let stops = paint["stops"]
            .as_array()
            .expect("gradient paint should have stops");
        assert_eq!(stops.len(), 3);
        assert_eq!(stops[0]["position"], json_number(0.0));
        assert_eq!(stops[1]["position"], json_number(0.5));
        assert_eq!(stops[2]["position"], json_number(1.0));
    }

    #[test]
    fn generated_scene_imports_with_figma_importer() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body class="bg-black text-white">
    <div class="w-24 h-12 bg-primary border-2 border-white rounded-lg p-2">
      <h1 class="text-xl font-bold">LOUD</h1>
      <p class="text-sm text-gray-300">NOISE FLOOR</p>
    </div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };
        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );
        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );
        let json =
            serde_json::to_string(&json!({ "nodes": nodes })).expect("scene json should serialize");
        let imported = import_figma_document(&json).expect("generated scene must import");
        assert!(
            imported.figma_to_scene.len() >= 3,
            "expected body + block + text nodes, got {}",
            imported.figma_to_scene.len()
        );
    }

    #[test]
    fn h_full_children_use_parent_height_not_grandparent() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="w-full h-[85vh]">
      <div class="w-full h-full"></div>
    </div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 800.0,
            height: 1000.0,
        };

        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let parent = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0" {
                    return None;
                }
                object
                    .get("bounds")?
                    .as_array()?
                    .get(3)?
                    .as_f64()
                    .map(|v| v as f32)
            })
            .expect("parent bounds should exist");
        let child = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0.0" {
                    return None;
                }
                object
                    .get("bounds")?
                    .as_array()?
                    .get(3)?
                    .as_f64()
                    .map(|v| v as f32)
            })
            .expect("child bounds should exist");

        assert!((parent - 850.0).abs() < 0.01);
        assert!((child - 850.0).abs() < 0.01);
    }

    #[test]
    fn absolute_wrapper_without_width_shrinks_to_child_content() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="relative w-[400px] h-[200px]">
      <div class="absolute bottom-0 right-0">
        <div class="w-20 h-10"></div>
      </div>
    </div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };

        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let wrapper_bounds = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0.0" {
                    return None;
                }
                let bounds = object.get("bounds")?.as_array()?;
                Some((
                    bounds.first()?.as_f64()? as f32,
                    bounds.get(2)?.as_f64()? as f32,
                ))
            })
            .expect("absolute wrapper bounds should exist");

        assert!(
            (wrapper_bounds.1 - 80.0).abs() < 0.01,
            "absolute wrapper width should match child width, got {}",
            wrapper_bounds.1
        );
        assert!(
            (wrapper_bounds.0 - 320.0).abs() < 0.01,
            "right anchored wrapper should sit at x=320, got {}",
            wrapper_bounds.0
        );
    }

    #[test]
    fn translate_full_moves_node_by_its_own_size() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="w-10 h-10 -translate-y-full"></div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };

        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let translated_y = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0" {
                    return None;
                }
                object
                    .get("bounds")?
                    .as_array()?
                    .get(1)?
                    .as_f64()
                    .map(|v| v as f32)
            })
            .expect("translated node bounds should exist");

        assert!((translated_y - -40.0).abs() < 0.01);
    }

    #[test]
    fn row_flex_children_without_explicit_width_hug_content() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="flex flex-row w-[300px]">
      <div class="bg-white px-2 py-1">
        <span>THE PIT</span>
      </div>
    </div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };

        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let child_width = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0.0" {
                    return None;
                }
                object
                    .get("bounds")?
                    .as_array()?
                    .get(2)?
                    .as_f64()
                    .map(|v| v as f32)
            })
            .expect("row flex child bounds should exist");

        assert!(
            child_width < 300.0,
            "row flex child should not default to full parent width, got {}",
            child_width
        );
        assert!(
            child_width > 40.0,
            "row flex child should retain intrinsic content width, got {}",
            child_width
        );
    }

    #[test]
    fn non_text_tag_with_direct_text_emits_text_node() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="bg-primary text-black px-4 py-2">PLAY LOUD</div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };

        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let play_loud = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0" {
                    return None;
                }
                Some(object)
            })
            .expect("text div node should exist");

        assert_eq!(
            play_loud
                .get("type")
                .and_then(JsonValue::as_str)
                .unwrap_or_default(),
            "TEXT",
            "direct text on non-text tags should emit a text node"
        );
        assert_eq!(
            play_loud
                .get("characters")
                .and_then(JsonValue::as_str)
                .unwrap_or_default(),
            "PLAY LOUD"
        );
    }

    #[test]
    fn text_children_inherit_parent_font_size_for_measurement() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div>
      <h2 class="text-7xl leading-none">
        <span class="bg-white text-black px-2 inline-block">SCUM</span>
      </h2>
    </div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };

        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let scum_width = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("characters")?.as_str()? != "SCUM" {
                    return None;
                }
                object
                    .get("bounds")?
                    .as_array()?
                    .get(2)?
                    .as_f64()
                    .map(|v| v as f32)
            })
            .expect("SCUM text bounds should exist");

        assert!(
            scum_width > 150.0,
            "expected inherited large font size to produce wide bounds, got {}",
            scum_width
        );
    }

    #[test]
    fn flex_row_with_flex1_children_share_available_width() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="w-[300px] flex flex-row">
      <div class="flex-1 h-4 bg-white"></div>
      <div class="flex-1 h-4 bg-white"></div>
    </div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };

        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let first = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0.0" {
                    return None;
                }
                object.get("bounds")?.as_array().cloned()
            })
            .expect("first flex child bounds should exist");
        let second = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0.1" {
                    return None;
                }
                object.get("bounds")?.as_array().cloned()
            })
            .expect("second flex child bounds should exist");

        let first_width = first[2]
            .as_f64()
            .map(|v| v as f32)
            .expect("first width should be numeric");
        let second_width = second[2]
            .as_f64()
            .map(|v| v as f32)
            .expect("second width should be numeric");
        let second_x = second[0]
            .as_f64()
            .map(|v| v as f32)
            .expect("second x should be numeric");

        assert!(
            (first_width - 150.0).abs() <= 1.0,
            "first flex-1 child should consume half row width, got {first_width}"
        );
        assert!(
            (second_width - 150.0).abs() <= 1.0,
            "second flex-1 child should consume half row width, got {second_width}"
        );
        assert!(
            (second_x - 150.0).abs() <= 1.0,
            "second flex-1 child should start after first half, got x={second_x}"
        );
    }

    #[test]
    fn justify_between_spreads_row_children_to_edges() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="w-[300px] flex flex-row justify-between">
      <div class="w-10 h-4 bg-white"></div>
      <div class="w-10 h-4 bg-white"></div>
    </div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 800.0,
            height: 600.0,
        };

        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let first_x = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0.0" {
                    return None;
                }
                object
                    .get("bounds")?
                    .as_array()?
                    .first()?
                    .as_f64()
                    .map(|v| v as f32)
            })
            .expect("first child x should exist");
        let second_x = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0.1" {
                    return None;
                }
                object
                    .get("bounds")?
                    .as_array()?
                    .first()?
                    .as_f64()
                    .map(|v| v as f32)
            })
            .expect("second child x should exist");

        assert!(
            first_x.abs() <= 0.5,
            "first justify-between child should stay at row start, got x={first_x}"
        );
        assert!(
            (second_x - 260.0).abs() <= 1.0,
            "second justify-between child should align near row end, got x={second_x}"
        );
    }

    #[test]
    fn fixed_inset_parent_preserves_full_width_for_w_full_child() {
        let html = r#"
<!DOCTYPE html>
<html>
  <body>
    <div class="fixed inset-0">
      <div class="w-full h-8 bg-white"></div>
    </div>
  </body>
</html>
"#;
        let theme = TailwindTheme::from_html_source(html);
        let document = Html::parse_document(html);
        let selector = Selector::parse("body").expect("body selector should parse");
        let body = document
            .select(&selector)
            .next()
            .expect("body should exist in test html");
        let root =
            build_html_node(body, &ClassStyleMap::new()).expect("html node tree should build");
        let viewport = Size {
            width: 500.0,
            height: 300.0,
        };

        let mut size_cache = HashMap::new();
        let mut measure_path = vec![0usize];
        measure_tree(
            &root,
            &mut measure_path,
            viewport,
            viewport,
            &mut size_cache,
            None,
            &theme,
            &TextStyle::default(),
        );

        let mut layout_path = vec![0usize];
        let mut nodes = Vec::new();
        layout_tree(
            &root,
            &mut layout_path,
            None,
            Rect {
                x: 0.0,
                y: 0.0,
                width: viewport.width,
                height: viewport.height,
            },
            viewport,
            &size_cache,
            &theme,
            &TextStyle::default(),
            &mut nodes,
        );

        let width = nodes
            .iter()
            .find_map(|node| {
                let object = node.as_object()?;
                if object.get("id")?.as_str()? != "stitch:0.0.0" {
                    return None;
                }
                object
                    .get("bounds")?
                    .as_array()?
                    .get(2)?
                    .as_f64()
                    .map(|v| v as f32)
            })
            .expect("w-full child width should exist");

        assert!(
            (width - viewport.width).abs() <= 1.0,
            "w-full child under fixed inset parent should match viewport width, got {width}"
        );
    }
}
