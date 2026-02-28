use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

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

        Self { colors, fonts }
    }
}

#[derive(Debug, Clone, Default)]
struct TextStyle {
    color: Option<[f32; 4]>,
    font_family: Option<String>,
    font_size: Option<f32>,
    font_weight: Option<u16>,
    text_align: Option<String>,
    text_case: Option<String>,
    line_height_percent: Option<f32>,
}

#[derive(Debug, Clone)]
struct LayoutChild {
    index: usize,
    size: Size,
    margin: Edges,
    position_mode: PositionMode,
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
    let dom = Html::parse_document(&html_source);
    let selector =
        Selector::parse("body").map_err(|err| format!("failed to parse body selector: {err}"))?;
    let body = dom
        .select(&selector)
        .next()
        .ok_or_else(|| "html body element not found".to_string())?;
    let root = build_html_node(body)?;

    let mut size_cache = HashMap::new();
    let mut root_path = vec![0usize];
    measure_tree(&root, &mut root_path, viewport, viewport, &mut size_cache);

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

fn build_html_node(element: ElementRef<'_>) -> Result<HtmlNode, String> {
    let tag = element.value().name().to_ascii_lowercase();
    let classes = element
        .value()
        .attr("class")
        .map(split_classes)
        .unwrap_or_default();
    let styles = element
        .value()
        .attr("style")
        .map(parse_inline_styles)
        .unwrap_or_default();
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
        children.push(build_html_node(child_element)?);
    }

    Ok(HtmlNode {
        tag,
        classes,
        styles,
        text,
        children,
    })
}

fn is_ignored_tag(tag: &str) -> bool {
    matches!(
        tag.to_ascii_lowercase().as_str(),
        "script" | "style" | "noscript" | "meta" | "link" | "title" | "head"
    )
}

fn collect_text_with_line_breaks(element: ElementRef<'_>) -> String {
    let mut output = String::new();
    collect_text_recursive(element, &mut output);
    normalize_text(&output)
}

fn collect_text_recursive(element: ElementRef<'_>, out: &mut String) {
    for child in element.children() {
        match child.value() {
            Node::Text(text) => {
                out.push_str(text.text.as_ref());
            }
            Node::Element(el) => {
                if el.name() == "br" {
                    out.push('\n');
                } else if let Some(wrapped) = ElementRef::wrap(child) {
                    collect_text_recursive(wrapped, out);
                }
            }
            _ => {}
        }
    }
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

fn measure_tree(
    node: &HtmlNode,
    path: &mut Vec<usize>,
    parent_size: Size,
    viewport: Size,
    cache: &mut HashMap<String, Size>,
) -> Size {
    let key = path_to_key(path);
    if let Some(existing) = cache.get(&key).copied() {
        return existing;
    }

    let padding = parse_padding(&node.classes, parent_size, viewport);
    let explicit_width = resolve_dimension(node, Axis::X, parent_size, viewport);
    let explicit_height = resolve_dimension(node, Axis::Y, parent_size, viewport);

    let is_text = is_textual_tag(&node.tag);
    let font_size = resolve_font_size(&node.classes).unwrap_or(16.0);
    let line_height_factor = resolve_line_height_factor(&node.classes);
    let estimated_text_size = estimate_text_size(&node.text, font_size, line_height_factor);

    let mut width = explicit_width.unwrap_or_else(|| {
        if is_text {
            estimated_text_size.width + padding.left + padding.right
        } else {
            parent_size.width
        }
    });
    if !width.is_finite() || width <= 0.0 {
        width = parent_size.width.max(1.0);
    }

    let inner_width = (width - padding.left - padding.right).max(0.0);
    let inner_parent = Size {
        width: inner_width,
        height: parent_size.height.max(0.0),
    };

    let direction = flow_direction(&node.classes);
    let gap = parse_gap(&node.classes, parent_size, viewport).unwrap_or(0.0);

    let mut flow_main = 0.0f32;
    let mut flow_cross = 0.0f32;
    let mut visible_flow_children = 0usize;

    for (index, child) in node.children.iter().enumerate() {
        path.push(index);
        let child_size = measure_tree(child, path, inner_parent, viewport, cache);
        path.pop();

        if position_mode(&child.classes) != PositionMode::Normal {
            continue;
        }

        let margin = parse_margin(&child.classes, inner_parent, viewport);
        match direction {
            FlowDirection::Column => {
                flow_main += margin.top + child_size.height + margin.bottom;
                flow_cross = flow_cross.max(margin.left + child_size.width + margin.right);
            }
            FlowDirection::Row => {
                flow_main += margin.left + child_size.width + margin.right;
                flow_cross = flow_cross.max(margin.top + child_size.height + margin.bottom);
            }
        }
        visible_flow_children += 1;
    }

    if visible_flow_children > 1 {
        flow_main += gap * (visible_flow_children as f32 - 1.0);
    }

    let mut height = explicit_height.unwrap_or(0.0);
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

    if explicit_width.is_none() && !is_text && !node.children.is_empty() {
        width = match direction {
            FlowDirection::Column => width.max(flow_cross + padding.left + padding.right),
            FlowDirection::Row => width.max(flow_main + padding.left + padding.right),
        };
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
    let node_id = format!("stitch:{}", path_to_key(path));
    let resolved_text = resolve_text_style(node, theme, inherited_text);
    let node_json = node_to_figma_json(
        node,
        &node_id,
        parent_id.as_deref(),
        rect,
        theme,
        viewport,
        &resolved_text,
    );
    out_nodes.push(node_json);

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
        });
    }

    let mut normal_total_main = 0.0f32;
    let mut normal_count = 0usize;
    for child in &children_meta {
        if child.position_mode != PositionMode::Normal {
            continue;
        }
        match direction {
            FlowDirection::Column => {
                normal_total_main += child.margin.top + child.size.height + child.margin.bottom;
            }
            FlowDirection::Row => {
                normal_total_main += child.margin.left + child.size.width + child.margin.right;
            }
        }
        normal_count += 1;
    }
    if normal_count > 1 {
        normal_total_main += gap * (normal_count as f32 - 1.0);
    }

    let main_available = match direction {
        FlowDirection::Column => content.height,
        FlowDirection::Row => content.width,
    };
    let justify_offset = match justify.as_deref() {
        Some("CENTER") => ((main_available - normal_total_main) * 0.5).max(0.0),
        Some("MAX") => (main_available - normal_total_main).max(0.0),
        _ => 0.0,
    };

    let mut cursor_main = justify_offset;
    for child_meta in children_meta {
        let child = &node.children[child_meta.index];
        let child_size = child_meta.size;
        let child_margin = child_meta.margin;

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
                        cursor_main +=
                            child_margin.top + child_size.height + child_margin.bottom + gap;
                    }
                    FlowDirection::Row => {
                        cursor_main +=
                            child_margin.left + child_size.width + child_margin.right + gap;
                    }
                }
                rect
            }
        };

        path.push(child_meta.index);
        layout_tree(
            child,
            path,
            Some(node_id.clone()),
            child_rect,
            viewport,
            size_cache,
            theme,
            &resolved_text,
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
    let mut object = JsonMap::new();
    object.insert("id".to_string(), JsonValue::String(id.to_string()));
    if let Some(parent_id) = parent_id {
        object.insert(
            "parentId".to_string(),
            JsonValue::String(parent_id.to_string()),
        );
    }
    object.insert(
        "type".to_string(),
        JsonValue::String(figma_node_type(node).to_string()),
    );
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
    if let Some(blend_mode) = resolve_blend_mode(&node.classes) {
        object.insert("blendMode".to_string(), JsonValue::String(blend_mode));
    }

    if let Some(radius) = resolve_corner_radius(&node.classes, &node.styles, rect) {
        object.insert("cornerRadius".to_string(), json_number(radius));
    }

    let mut fills = Vec::new();
    if let Some(image_ref) = resolve_background_image_ref(&node.styles) {
        fills.push(json!({
            "type": "IMAGE",
            "imageRef": image_ref,
            "scaleMode": "FILL",
            "visible": true
        }));
    }
    if let Some(color) = resolve_background_color(&node.classes, theme) {
        fills.push(solid_paint_json(color));
    }
    if is_textual_tag(&node.tag)
        && !node.text.is_empty()
        && let Some(color) = resolved_text.color
    {
        fills.push(solid_paint_json(color));
    }
    if !fills.is_empty() {
        object.insert("fills".to_string(), JsonValue::Array(fills));
    }

    if let Some((stroke_weight, stroke_color)) = resolve_stroke(&node.classes, theme) {
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

    if is_textual_tag(&node.tag) && !node.text.is_empty() {
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
        if !style.is_empty() {
            object.insert("style".to_string(), JsonValue::Object(style));
        }
    }

    JsonValue::Object(object)
}

fn resolve_text_style(node: &HtmlNode, theme: &TailwindTheme, inherited: &TextStyle) -> TextStyle {
    let mut style = inherited.clone();
    if let Some(color) = resolve_text_color(&node.classes, theme) {
        style.color = Some(color);
    }
    if let Some(font_family) = resolve_font_family(&node.classes, theme) {
        style.font_family = Some(font_family);
    }
    if let Some(font_size) = resolve_font_size(&node.classes) {
        style.font_size = Some(font_size);
    }
    if let Some(font_weight) = resolve_font_weight(&node.classes) {
        style.font_weight = Some(font_weight);
    }
    if let Some(text_align) = resolve_text_align(&node.classes) {
        style.text_align = Some(text_align);
    }
    if let Some(text_case) = resolve_text_case(&node.classes) {
        style.text_case = Some(text_case);
    }
    if let Some(line_height_factor) = resolve_line_height_factor_option(&node.classes) {
        style.line_height_percent = Some(line_height_factor * 100.0);
    }
    style
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
    if is_textual_tag(&node.tag) && !node.text.is_empty() {
        "TEXT"
    } else if !node.children.is_empty() {
        "FRAME"
    } else {
        "RECTANGLE"
    }
}

fn figma_layout_mode(classes: &[String]) -> Option<&'static str> {
    if !has_class(classes, "flex") {
        return None;
    }
    Some(match flow_direction(classes) {
        FlowDirection::Column => "VERTICAL",
        FlowDirection::Row => "HORIZONTAL",
    })
}

fn flow_direction(classes: &[String]) -> FlowDirection {
    if has_class(classes, "flex") && has_class(classes, "flex-row") {
        FlowDirection::Row
    } else {
        FlowDirection::Column
    }
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
    let prefix = format!("{property}-");
    if let Some(token) = class_value(&node.classes, &prefix) {
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
        if let Some(value) = resolve_length_token(token, parent_axis, viewport_axis) {
            return Some(value);
        }
    }

    if let Some(style_value) = node.styles.get(property) {
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

fn resolve_font_size(classes: &[String]) -> Option<f32> {
    if let Some(arbitrary) = class_value(classes, "text-[")
        && let Some(value) = arbitrary.strip_suffix(']')
        && let Some(px) = parse_css_length(value, 0.0, 0.0)
    {
        return Some(px.max(1.0));
    }

    let token = classes
        .iter()
        .rev()
        .find(|class| class.starts_with("text-"))
        .map(|class| class.trim_start_matches("text-"))?;
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

fn resolve_font_weight(classes: &[String]) -> Option<u16> {
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

fn resolve_font_family(classes: &[String], theme: &TailwindTheme) -> Option<String> {
    let family_alias = class_value(classes, "font-")?;
    if let Some(family) = theme.fonts.get(family_alias) {
        return Some(family.clone());
    }
    if family_alias.contains('-') {
        return Some(family_alias.replace('-', " "));
    }
    None
}

fn resolve_text_align(classes: &[String]) -> Option<String> {
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

fn resolve_text_case(classes: &[String]) -> Option<String> {
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

fn resolve_line_height_factor_option(classes: &[String]) -> Option<f32> {
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

fn resolve_line_height_factor(classes: &[String]) -> f32 {
    resolve_line_height_factor_option(classes).unwrap_or(1.35)
}

fn estimate_text_size(text: &str, font_size: f32, line_height_factor: f32) -> Size {
    if text.is_empty() {
        return Size {
            width: font_size * 0.5,
            height: font_size * line_height_factor,
        };
    }
    let mut max_chars = 0usize;
    let mut line_count = 0usize;
    for line in text.split('\n') {
        max_chars = max_chars.max(line.chars().count());
        line_count += 1;
    }
    let width = max_chars as f32 * font_size * 0.56;
    let height = line_count as f32 * font_size * line_height_factor;
    Size {
        width: width.max(font_size * 0.56),
        height: height.max(font_size * line_height_factor),
    }
}
fn resolve_background_image_ref(styles: &HashMap<String, String>) -> Option<String> {
    let value = styles.get("background-image")?;
    let start = value.find("url(")?;
    let tail = &value[start + 4..];
    let end = tail.find(')')?;
    let mut url = tail[..end].trim().trim_matches('\'').trim_matches('"');
    if url.starts_with("data:") {
        return None;
    }
    if url.is_empty() {
        return None;
    }
    if let Some(stripped) = url.strip_prefix("./") {
        url = stripped;
    }
    Some(url.to_string())
}

fn resolve_background_color(classes: &[String], theme: &TailwindTheme) -> Option<[f32; 4]> {
    classes
        .iter()
        .rev()
        .find_map(|class| class.strip_prefix("bg-"))
        .and_then(|token| resolve_color_token(token, theme))
}

fn resolve_text_color(classes: &[String], theme: &TailwindTheme) -> Option<[f32; 4]> {
    classes
        .iter()
        .rev()
        .find_map(|class| class.strip_prefix("text-"))
        .and_then(|token| resolve_color_token(token, theme))
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

fn resolve_blend_mode(classes: &[String]) -> Option<String> {
    let token = classes
        .iter()
        .rev()
        .find_map(|class| class.strip_prefix("mix-blend-"))?;
    let mode = match token {
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

fn resolve_stroke(classes: &[String], theme: &TailwindTheme) -> Option<(f32, [f32; 4])> {
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
        return None;
    }

    let stroke_color = classes
        .iter()
        .rev()
        .find_map(|class| class.strip_prefix("border-"))
        .and_then(|token| resolve_color_token(token, theme))
        .or_else(|| theme.colors.get("white").copied())
        .unwrap_or([1.0, 1.0, 1.0, 1.0]);

    Some((weight, stroke_color))
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
  fontFamily: { "display": ["Space Grotesk", "sans-serif"] }
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
        let root = build_html_node(body).expect("html node tree should build");
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
}
