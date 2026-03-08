//! Riot Waves generated Figma example.
//!
//! Regenerate the embedded module with:
//! `cargo run --bin figma_codegen -- --input riot_waves.json --output examples/generated/riot_waves_generated_module.rs --module-name riot_waves_generated --document-fn document --runtime-fn runtime`

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::time::Instant;

use arthropod::figma_runtime::FigmaRuntime;
use arthropod::prototype_runtime::PrototypeRuntimeEvent;
use arthropod_test::visual_test::load_image;
use plat_core::{
    Application, ControlFlow, ElementState, Event, EventLoop, MouseButton, Size, Window,
    WindowConfig, WindowEvent, WindowId,
};
use render_engine::{NodeId, backend::WgpuBackend};
use serde_json::Value as JsonValue;
use style_engine::ImageId;

#[allow(dead_code)]
#[path = "generated/riot_waves_generated_module.rs"]
mod riot_waves_generated_module;

const DEFAULT_WIDTH: u32 = 1366;
const DEFAULT_HEIGHT: u32 = 884;
const APPLY_RUNTIME_LAYOUT_ON_SIZE_MISMATCH: bool = false;

#[derive(Debug, Clone, PartialEq)]
struct GeneratedImageAsset {
    image_ref: String,
    source_ref: String,
    filter: Option<ImageFilterSpec>,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct ImageFilterSpec {
    grayscale: Option<f32>,
    contrast: Option<f32>,
    invert: Option<f32>,
}

struct RiotWavesApp {
    backend: WgpuBackend,
    window: Window,
    runtime: FigmaRuntime,
    hovered_node: Option<NodeId>,
    last_frame: Instant,
}

impl Application for RiotWavesApp {
    fn new(event_loop: &EventLoop) -> Self {
        let config = WindowConfig {
            title: "Riot Waves (Generated from Figma JSON)".to_string(),
            size: Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT),
            resizable: true,
            visible: true,
            ..Default::default()
        };

        let window = event_loop
            .create_window(config)
            .expect("failed to create window");
        let size = window.inner_size();

        // SAFETY: backend is dropped before window based on struct field order.
        let backend = unsafe { WgpuBackend::new(&window, size.width, size.height, false) }
            .expect("failed to create backend");
        let mut backend = backend;
        backend.set_design_space(DEFAULT_WIDTH as f32, DEFAULT_HEIGHT as f32);
        register_generated_fonts(&mut backend);
        register_generated_images(&mut backend);

        let mut runtime = riot_waves_generated_module::riot_waves_generated::runtime()
            .expect("failed to initialize generated riot_waves runtime");
        if should_apply_runtime_layout(size) {
            runtime.apply_layout(size.width as f32, size.height as f32);
        }

        window.request_redraw();

        Self {
            backend,
            window,
            runtime,
            hovered_node: None,
            last_frame: Instant::now(),
        }
    }

    fn on_event(&mut self, event: Event, control_flow: &mut ControlFlow) {
        match event {
            Event::Window {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            Event::Window {
                event: WindowEvent::Resized(size),
                ..
            } => {
                self.backend.resize(size.width, size.height);
                if should_apply_runtime_layout(size) {
                    self.runtime
                        .apply_layout(size.width as f32, size.height as f32);
                }
                self.window.request_redraw();
            }
            Event::Window {
                event: WindowEvent::CursorMoved { position },
                ..
            } => {
                let window_size = self.window.inner_size();
                let hovered =
                    map_window_to_design_point(window_size, position.x as f32, position.y as f32)
                        .and_then(|(x, y)| self.runtime.scene().hit_test(x, y));
                if hovered != self.hovered_node {
                    self.hovered_node = hovered;
                    if let Some(node) = hovered {
                        let _ = self.runtime.dispatch(PrototypeRuntimeEvent::Hover { node });
                        self.window.request_redraw();
                    }
                }
            }
            Event::Window {
                event: WindowEvent::MouseInput(input),
                ..
            } => {
                if input.button == MouseButton::Left && input.state == ElementState::Pressed {
                    let window_size = self.window.inner_size();
                    let target = map_window_to_design_point(
                        window_size,
                        input.position.x as f32,
                        input.position.y as f32,
                    )
                    .and_then(|(x, y)| self.runtime.scene().hit_test(x, y));
                    if let Some(node) = target {
                        self.hovered_node = Some(node);
                        let _ = self.runtime.dispatch(PrototypeRuntimeEvent::Press { node });
                        let _ = self.runtime.dispatch(PrototypeRuntimeEvent::Click { node });
                        self.window.request_redraw();
                    }
                }
            }
            Event::Window {
                event: WindowEvent::KeyboardInput(input),
                ..
            } => {
                if input.state == ElementState::Pressed {
                    let target = self.hovered_node.or_else(|| self.runtime.current_screen());
                    if let Some(node) = target {
                        let _ = self
                            .runtime
                            .dispatch(PrototypeRuntimeEvent::KeyDown { node });
                        self.window.request_redraw();
                    }
                }
            }
            _ => {}
        }
    }

    fn on_redraw(&mut self, _window_id: WindowId) {
        let now = Instant::now();
        let elapsed_ms_u128 = now.duration_since(self.last_frame).as_millis();
        self.last_frame = now;
        let elapsed_ms = u32::try_from(elapsed_ms_u128).unwrap_or(u32::MAX);

        if elapsed_ms > 0 {
            let _ = self
                .runtime
                .dispatch(PrototypeRuntimeEvent::Tick { elapsed_ms });
        }

        if let Err(err) = self.backend.render(self.runtime.scene()) {
            eprintln!("render error: {err}");
        }

        self.window.request_redraw();
    }
}

fn register_generated_fonts(backend: &mut WgpuBackend) {
    let roots = image_search_roots();
    let font_paths = collect_font_assets(&roots);
    if font_paths.is_empty() {
        return;
    }

    let mut loaded_files = 0usize;
    let mut loaded_faces = 0usize;
    let mut failed = 0usize;
    for path in font_paths {
        match std::fs::read(&path) {
            Ok(bytes) => {
                let faces = backend.register_font_bytes(bytes);
                if faces > 0 {
                    loaded_files += 1;
                    loaded_faces += faces;
                } else {
                    eprintln!(
                        "warning: font asset `{}` contained no registerable faces",
                        path.display()
                    );
                    failed += 1;
                }
            }
            Err(err) => {
                eprintln!(
                    "warning: failed to read font asset `{}`: {err}",
                    path.display()
                );
                failed += 1;
            }
        }
    }

    eprintln!(
        "riot_waves_generated: loaded {loaded_files} font file(s), {loaded_faces} face(s), {failed} failed"
    );
}

fn collect_font_assets(roots: &[PathBuf]) -> Vec<PathBuf> {
    let mut search_dirs = Vec::new();
    let mut seen_dirs = BTreeSet::new();
    for root in roots {
        let root_assets = root.join("assets");
        if root_assets.is_dir() && seen_dirs.insert(root_assets.to_string_lossy().to_string()) {
            search_dirs.push(root_assets);
        }

        if root.is_dir()
            && root
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| name.eq_ignore_ascii_case("assets"))
            && seen_dirs.insert(root.to_string_lossy().to_string())
        {
            search_dirs.push(root.clone());
        }
    }

    let mut fonts = Vec::new();
    let mut seen_files = BTreeSet::new();
    let mut visited_dirs = BTreeSet::new();
    let mut pending_dirs = search_dirs;
    while let Some(dir) = pending_dirs.pop() {
        let dir_key = dir.to_string_lossy().to_string();
        if !visited_dirs.insert(dir_key) {
            continue;
        }

        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                pending_dirs.push(path);
                continue;
            }
            if !path.is_file() || !is_font_asset_path(&path) {
                continue;
            }
            let key = path.to_string_lossy().to_string();
            if seen_files.insert(key) {
                fonts.push(path);
            }
        }
    }
    fonts
}

fn is_font_asset_path(path: &Path) -> bool {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .is_some_and(|ext| matches!(ext.as_str(), "ttf" | "otf" | "ttc" | "woff" | "woff2"))
}

fn should_apply_runtime_layout(size: Size<u32>) -> bool {
    APPLY_RUNTIME_LAYOUT_ON_SIZE_MISMATCH
        && (size.width != DEFAULT_WIDTH || size.height != DEFAULT_HEIGHT)
}

fn map_window_to_design_point(size: Size<u32>, x: f32, y: f32) -> Option<(f32, f32)> {
    if size.width == 0 || size.height == 0 {
        return None;
    }

    let window_w = size.width as f32;
    let window_h = size.height as f32;
    let design_w = DEFAULT_WIDTH as f32;
    let design_h = DEFAULT_HEIGHT as f32;
    let scale = (window_w / design_w).min(window_h / design_h);
    if !scale.is_finite() || scale <= 0.0 {
        return None;
    }

    let fitted_w = design_w * scale;
    let fitted_h = design_h * scale;
    let offset_x = (window_w - fitted_w) * 0.5;
    let offset_y = (window_h - fitted_h) * 0.5;
    let design_x = (x - offset_x) / scale;
    let design_y = (y - offset_y) / scale;
    if !(0.0..=design_w).contains(&design_x) || !(0.0..=design_h).contains(&design_y) {
        return None;
    }

    Some((design_x, design_y))
}

fn register_generated_images(backend: &mut WgpuBackend) {
    let assets =
        collect_image_assets(riot_waves_generated_module::riot_waves_generated::FIGMA_JSON);
    if assets.is_empty() {
        return;
    }

    let roots = image_search_roots();
    let mut loaded = 0usize;
    let mut missing = Vec::new();
    let mut failed = 0usize;
    for asset in assets {
        let GeneratedImageAsset {
            image_ref,
            source_ref,
            filter: _renderer_color_filter,
        } = asset;
        let image_id = ImageId(figma_image_reference_to_id(&image_ref));
        if let Some((rgba, width, height)) = load_procedural_image(&source_ref, &roots) {
            if let Err(err) = backend.register_image_rgba8(image_id, width, height, rgba) {
                eprintln!(
                    "warning: failed to register procedural image `{}` for `{}`: {err}",
                    source_ref, image_ref
                );
                failed += 1;
            } else {
                loaded += 1;
            }
            continue;
        }

        let Some(path) = resolve_image_path_with_roots(&source_ref, &roots) else {
            missing.push(source_ref.clone());
            continue;
        };

        match load_image(&path) {
            Ok((rgba, width, height)) => {
                if let Err(err) = backend.register_image_rgba8(image_id, width, height, rgba) {
                    eprintln!(
                        "warning: failed to register image `{}` for `{}`: {err}",
                        path.display(),
                        image_ref
                    );
                    failed += 1;
                } else {
                    loaded += 1;
                }
            }
            Err(err) => {
                eprintln!(
                    "warning: failed to decode image `{}` for `{}`: {err}",
                    path.display(),
                    source_ref
                );
                failed += 1;
            }
        }
    }

    if !missing.is_empty() {
        for source_ref in &missing {
            eprintln!("warning: image source `{source_ref}` not found on disk");
        }
    }

    eprintln!(
        "riot_waves_generated: loaded {loaded} image(s), {} missing, {failed} failed",
        missing.len()
    );
}

fn collect_image_assets(figma_json: &str) -> Vec<GeneratedImageAsset> {
    let Ok(root) = serde_json::from_str::<JsonValue>(figma_json) else {
        return Vec::new();
    };
    let mut dedupe = BTreeSet::new();
    let mut assets = Vec::new();
    collect_image_assets_recursive(&root, &mut dedupe, &mut assets);
    assets
}

fn collect_image_assets_recursive(
    value: &JsonValue,
    dedupe: &mut BTreeSet<String>,
    assets: &mut Vec<GeneratedImageAsset>,
) {
    match value {
        JsonValue::Object(object) => {
            if object
                .get("type")
                .and_then(JsonValue::as_str)
                .is_some_and(|kind| kind.eq_ignore_ascii_case("IMAGE"))
                && let Some(image_ref) = object.get("imageRef").and_then(JsonValue::as_str)
            {
                let source_ref = object
                    .get("imageSourceRef")
                    .and_then(JsonValue::as_str)
                    .unwrap_or(image_ref);
                if likely_path_reference(source_ref) && dedupe.insert(image_ref.to_string()) {
                    assets.push(GeneratedImageAsset {
                        image_ref: image_ref.to_string(),
                        source_ref: source_ref.to_string(),
                        filter: object.get("imageFilter").and_then(parse_image_filter_spec),
                    });
                }
            }
            for child in object.values() {
                collect_image_assets_recursive(child, dedupe, assets);
            }
        }
        JsonValue::Array(items) => {
            for child in items {
                collect_image_assets_recursive(child, dedupe, assets);
            }
        }
        _ => {}
    }
}

fn parse_image_filter_spec(value: &JsonValue) -> Option<ImageFilterSpec> {
    let JsonValue::Object(object) = value else {
        return None;
    };
    let spec = ImageFilterSpec {
        grayscale: object
            .get("grayscale")
            .and_then(JsonValue::as_f64)
            .map(|v| v as f32),
        contrast: object
            .get("contrast")
            .and_then(JsonValue::as_f64)
            .map(|v| v as f32),
        invert: object
            .get("invert")
            .and_then(JsonValue::as_f64)
            .map(|v| v as f32),
    };
    if spec == ImageFilterSpec::default() {
        None
    } else {
        Some(spec)
    }
}

fn likely_path_reference(value: &str) -> bool {
    value.contains('/') || value.contains('\\')
}

#[cfg(test)]
fn apply_image_filter(mut rgba: Vec<u8>, filter: ImageFilterSpec) -> Vec<u8> {
    for pixel in rgba.chunks_exact_mut(4) {
        let mut r = pixel[0] as f32 / 255.0;
        let mut g = pixel[1] as f32 / 255.0;
        let mut b = pixel[2] as f32 / 255.0;

        if let Some(amount) = filter.grayscale {
            let a = amount.clamp(0.0, 1.0);
            let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            r = r * (1.0 - a) + luma * a;
            g = g * (1.0 - a) + luma * a;
            b = b * (1.0 - a) + luma * a;
        }

        if let Some(contrast) = filter.contrast {
            let c = contrast.max(0.0);
            r = ((r - 0.5) * c + 0.5).clamp(0.0, 1.0);
            g = ((g - 0.5) * c + 0.5).clamp(0.0, 1.0);
            b = ((b - 0.5) * c + 0.5).clamp(0.0, 1.0);
        }

        if let Some(invert) = filter.invert {
            let a = invert.clamp(0.0, 1.0);
            r = r * (1.0 - a) + (1.0 - r) * a;
            g = g * (1.0 - a) + (1.0 - g) * a;
            b = b * (1.0 - a) + (1.0 - b) * a;
        }

        pixel[0] = (r * 255.0).round().clamp(0.0, 255.0) as u8;
        pixel[1] = (g * 255.0).round().clamp(0.0, 255.0) as u8;
        pixel[2] = (b * 255.0).round().clamp(0.0, 255.0) as u8;
    }
    rgba
}

fn load_procedural_image(source_ref: &str, roots: &[PathBuf]) -> Option<(Vec<u8>, u32, u32)> {
    if let Some(raw_source_ref) = source_ref.strip_prefix("composite://halftone/") {
        let path = resolve_image_path_with_roots(raw_source_ref, roots)?;
        let (mut rgba, width, height) = load_image(&path).ok()?;
        let halftone = generate_halftone_texture(width, height);
        apply_overlay_halftone(&mut rgba, &halftone, 0.15);
        return Some((rgba, width, height));
    }

    if let Some(noise_key) = source_ref.strip_prefix("procedural://noise") {
        let seed = figma_image_reference_to_id(noise_key);
        let width = 128_u32;
        let height = 128_u32;
        return Some((generate_noise_texture(width, height, seed), width, height));
    }

    if source_ref.strip_prefix("procedural://halftone").is_some() {
        let width = 96_u32;
        let height = 96_u32;
        return Some((generate_halftone_texture(width, height), width, height));
    }

    None
}

fn overlay_blend_channel(src: f32, dst: f32) -> f32 {
    if dst <= 0.5 {
        2.0 * src * dst
    } else {
        1.0 - (2.0 * (1.0 - src) * (1.0 - dst))
    }
}

fn apply_overlay_halftone(base_rgba: &mut [u8], halftone_rgba: &[u8], opacity: f32) {
    let opacity = opacity.clamp(0.0, 1.0);
    if base_rgba.len() != halftone_rgba.len() {
        return;
    }

    for (dst_px, src_px) in base_rgba
        .chunks_exact_mut(4)
        .zip(halftone_rgba.chunks_exact(4))
    {
        let dst_a = dst_px[3] as f32 / 255.0;
        let src_a = (src_px[3] as f32 / 255.0) * opacity;
        if src_a <= f32::EPSILON {
            continue;
        }

        let dst_rgb = [
            dst_px[0] as f32 / 255.0,
            dst_px[1] as f32 / 255.0,
            dst_px[2] as f32 / 255.0,
        ];
        let src_rgb = [
            src_px[0] as f32 / 255.0,
            src_px[1] as f32 / 255.0,
            src_px[2] as f32 / 255.0,
        ];
        let blended_rgb = [
            overlay_blend_channel(src_rgb[0], dst_rgb[0]),
            overlay_blend_channel(src_rgb[1], dst_rgb[1]),
            overlay_blend_channel(src_rgb[2], dst_rgb[2]),
        ];

        // Porter-Duff source-over with blend-mode color function.
        let out_rgb_premul = [
            (src_a * (1.0 - dst_a) * src_rgb[0])
                + (src_a * dst_a * blended_rgb[0])
                + ((1.0 - src_a) * dst_a * dst_rgb[0]),
            (src_a * (1.0 - dst_a) * src_rgb[1])
                + (src_a * dst_a * blended_rgb[1])
                + ((1.0 - src_a) * dst_a * dst_rgb[1]),
            (src_a * (1.0 - dst_a) * src_rgb[2])
                + (src_a * dst_a * blended_rgb[2])
                + ((1.0 - src_a) * dst_a * dst_rgb[2]),
        ];
        let out_a = src_a + dst_a - src_a * dst_a;
        if out_a <= f32::EPSILON {
            dst_px[0] = 0;
            dst_px[1] = 0;
            dst_px[2] = 0;
            dst_px[3] = 0;
            continue;
        }

        dst_px[0] = ((out_rgb_premul[0] / out_a) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        dst_px[1] = ((out_rgb_premul[1] / out_a) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        dst_px[2] = ((out_rgb_premul[2] / out_a) * 255.0)
            .round()
            .clamp(0.0, 255.0) as u8;
        dst_px[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
    }
}

fn generate_noise_texture(width: u32, height: u32, seed: u64) -> Vec<u8> {
    let mut out = vec![0_u8; width as usize * height as usize * 4];
    let mut state = seed.wrapping_add(0x9E3779B97F4A7C15_u64);
    // The exported SVG turbulence texture is centered around mid-gray.
    // Keep procedural fallback low-contrast so overlay/screen blends don't
    // wash out the entire composition.
    const NOISE_ALPHA: u8 = 96;
    const NOISE_BASE: u8 = 128;
    const NOISE_AMPLITUDE: u8 = 32;
    for pixel in out.chunks_exact_mut(4) {
        // xorshift* variant for deterministic lightweight procedural noise.
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        let noise = state.wrapping_mul(0x2545F4914F6CDD1D_u64);
        let centered = ((noise >> 56) & 0xFF) as i16;
        let value = (NOISE_BASE as i16 - NOISE_AMPLITUDE as i16)
            + ((centered * (NOISE_AMPLITUDE as i16 * 2 + 1)) / 256);
        let value = value.clamp(0, 255) as u8;
        pixel[0] = value;
        pixel[1] = value;
        pixel[2] = value;
        pixel[3] = NOISE_ALPHA;
    }
    out
}

fn generate_halftone_texture(width: u32, height: u32) -> Vec<u8> {
    let mut out = vec![0_u8; width as usize * height as usize * 4];
    let spacing = 6_u32;
    let hard_radius = 2.0_f32;
    let feather_radius = 2.5_f32;
    for y in 0..height {
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            let fx = ((x % spacing) as f32 + 0.5) - spacing as f32 * 0.5;
            let fy = ((y % spacing) as f32 + 0.5) - spacing as f32 * 0.5;
            let dist = (fx * fx + fy * fy).sqrt();
            let alpha = if dist <= hard_radius {
                255_u8
            } else if dist <= feather_radius {
                let t = ((feather_radius - dist) / (feather_radius - hard_radius)).clamp(0.0, 1.0);
                (t * 255.0).round() as u8
            } else {
                0
            };
            out[idx] = 255;
            out[idx + 1] = 255;
            out[idx + 2] = 255;
            out[idx + 3] = alpha;
        }
    }
    out
}

fn resolve_image_path_with_roots(reference: &str, roots: &[PathBuf]) -> Option<PathBuf> {
    let as_path = PathBuf::from(reference);
    if as_path.is_absolute() && as_path.exists() {
        return Some(as_path);
    }

    for root in roots {
        let candidate = root.join(reference);
        if candidate.exists() {
            return Some(candidate);
        }
    }

    None
}

fn image_search_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();
    if let Ok(path) = std::env::var("ARTHROPOD_FIGMA_ASSET_ROOT") {
        roots.push(PathBuf::from(path));
    }

    roots.push(PathBuf::from("."));
    roots.push(PathBuf::from("artifacts"));
    roots.extend(artifact_subdirectories(Path::new("artifacts")));

    let mut deduped = Vec::new();
    roots
        .into_iter()
        .filter(|path| path.as_path() == Path::new(".") || path.exists())
        .for_each(|path| {
            if !deduped.iter().any(|existing| existing == &path) {
                deduped.push(path);
            }
        });
    deduped
}

fn artifact_subdirectories(root: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(root) else {
        return Vec::new();
    };

    entries
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_dir())
        .collect()
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

fn main() {
    env_logger::init();
    plat_core::run::<RiotWavesApp>().expect("failed to run riot waves example");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn should_apply_runtime_layout_is_disabled_for_design_viewport() {
        assert!(!should_apply_runtime_layout(Size::new(
            DEFAULT_WIDTH,
            DEFAULT_HEIGHT
        )));
    }

    #[test]
    fn should_apply_runtime_layout_is_disabled_for_clamped_viewport_by_default() {
        assert!(!should_apply_runtime_layout(Size::new(1366, 768)));
        assert!(!should_apply_runtime_layout(Size::new(
            1280,
            DEFAULT_HEIGHT
        )));
    }

    #[test]
    fn map_window_to_design_point_is_identity_for_design_size() {
        let size = Size::new(DEFAULT_WIDTH, DEFAULT_HEIGHT);
        let mapped = map_window_to_design_point(size, 683.0, 442.0).expect("point should map");
        assert!((mapped.0 - 683.0).abs() < 1e-3);
        assert!((mapped.1 - 442.0).abs() < 1e-3);
    }

    #[test]
    fn map_window_to_design_point_handles_letterbox_side_margins() {
        let size = Size::new(1366, 768);

        // Left side letterbox should not map to design space.
        assert!(map_window_to_design_point(size, 20.0, 200.0).is_none());

        // Center point should map to center of design.
        let mapped = map_window_to_design_point(size, 683.0, 384.0).expect("center should map");
        assert!((mapped.0 - 683.0).abs() < 1.0);
        assert!((mapped.1 - 442.0).abs() < 1.0);
    }

    #[test]
    fn figma_image_reference_to_id_is_stable() {
        assert_eq!(
            figma_image_reference_to_id("assets/foo.png"),
            figma_image_reference_to_id("assets/foo.png")
        );
        assert_ne!(
            figma_image_reference_to_id("assets/foo.png"),
            figma_image_reference_to_id("assets/bar.png")
        );
    }

    #[test]
    fn collect_image_assets_extracts_filtered_variants() {
        let json = r#"{
          "nodes": [
            {
              "fills": [
                { "type": "IMAGE", "imageRef": "assets/a.png" },
                {
                  "type": "IMAGE",
                  "imageRef": "filtered://assets/a.png#abc",
                  "imageSourceRef": "assets/a.png",
                  "imageFilter": { "grayscale": 1.0, "contrast": 1.5, "invert": 1.0 }
                }
              ]
            },
            { "fills": [ { "type": "IMAGE", "imageRef": "assets/b.png" } ] },
            { "fills": [ { "type": "IMAGE", "imageRef": "procedural://noise/1234" } ] },
            { "fills": [ { "type": "IMAGE", "imageRef": "assets/a.png" } ] },
            { "fills": [ { "type": "IMAGE", "imageRef": "12345" } ] }
          ]
        }"#;
        let refs = collect_image_assets(json);
        assert_eq!(
            refs,
            vec![
                GeneratedImageAsset {
                    image_ref: "assets/a.png".to_string(),
                    source_ref: "assets/a.png".to_string(),
                    filter: None
                },
                GeneratedImageAsset {
                    image_ref: "filtered://assets/a.png#abc".to_string(),
                    source_ref: "assets/a.png".to_string(),
                    filter: Some(ImageFilterSpec {
                        grayscale: Some(1.0),
                        contrast: Some(1.5),
                        invert: Some(1.0)
                    })
                },
                GeneratedImageAsset {
                    image_ref: "assets/b.png".to_string(),
                    source_ref: "assets/b.png".to_string(),
                    filter: None
                },
                GeneratedImageAsset {
                    image_ref: "procedural://noise/1234".to_string(),
                    source_ref: "procedural://noise/1234".to_string(),
                    filter: None
                },
            ]
        );
    }

    #[test]
    fn load_procedural_image_generates_deterministic_noise() {
        let (a, w1, h1) = load_procedural_image("procedural://noise/abcd", &[])
            .expect("procedural image should load");
        let (b, w2, h2) = load_procedural_image("procedural://noise/abcd", &[])
            .expect("procedural image should load");
        assert_eq!((w1, h1), (128, 128));
        assert_eq!((w2, h2), (128, 128));
        assert_eq!(a, b);
        assert_eq!(a.len(), (128 * 128 * 4) as usize);
        assert!(a.chunks_exact(4).all(|px| px[3] == 96));
    }

    #[test]
    fn load_procedural_image_noise_stays_in_low_contrast_band() {
        let (rgba, _, _) =
            load_procedural_image("procedural://noise/abcd", &[]).expect("noise should load");

        let mut min_value = u8::MAX;
        let mut max_value = u8::MIN;
        for px in rgba.chunks_exact(4) {
            min_value = min_value.min(px[0]);
            max_value = max_value.max(px[0]);
            assert_eq!(px[0], px[1]);
            assert_eq!(px[1], px[2]);
            assert_eq!(px[3], 96);
        }

        assert!(
            min_value >= 96,
            "noise floor should stay near middle gray, got {min_value}"
        );
        assert!(
            max_value <= 160,
            "noise ceiling should stay near middle gray, got {max_value}"
        );
    }

    #[test]
    fn load_procedural_image_supports_halftone_patterns() {
        let (rgba, width, height) = load_procedural_image("procedural://halftone/abcd", &[])
            .expect("halftone procedural image should load");
        assert_eq!((width, height), (96, 96));
        assert_eq!(rgba.len(), (96 * 96 * 4) as usize);
        assert!(
            rgba.chunks_exact(4).any(|px| px[3] > 0),
            "halftone texture should contain visible dots"
        );
        assert!(
            rgba.chunks_exact(4)
                .filter(|px| px[3] > 0)
                .all(|px| px[0] == 255 && px[1] == 255 && px[2] == 255),
            "halftone dots should be white before blend/filter composition"
        );
    }

    #[test]
    fn apply_image_filter_supports_invert_contrast_grayscale() {
        let input = vec![10_u8, 30_u8, 200_u8, 255_u8];
        let filtered = apply_image_filter(
            input,
            ImageFilterSpec {
                grayscale: Some(1.0),
                contrast: Some(1.5),
                invert: Some(1.0),
            },
        );
        assert_eq!(filtered.len(), 4);
        assert!(filtered[0] > 100, "expected high-contrast inversion");
        assert_eq!(filtered[0], filtered[1]);
        assert_eq!(filtered[1], filtered[2]);
    }

    #[test]
    fn apply_overlay_halftone_keeps_midtones_visible() {
        let mut base = vec![128_u8, 128_u8, 128_u8, 255_u8];
        let halftone = vec![255_u8, 255_u8, 255_u8, 255_u8];
        apply_overlay_halftone(&mut base, &halftone, 0.15);
        assert!(
            base[0] > 128 && base[0] < 200,
            "overlay blend should brighten midtone without clipping, got {}",
            base[0]
        );
        assert_eq!(base[0], base[1]);
        assert_eq!(base[1], base[2]);
        assert_eq!(base[3], 255);
    }

    #[test]
    fn riot_waves_runtime_preserves_hero_stack_node_opacity() {
        let runtime = riot_waves_generated_module::riot_waves_generated::runtime()
            .expect("runtime should initialize");
        let scene = runtime.scene();

        let hero_image = runtime
            .node_for_figma_id("stitch:0.1.0.0.0")
            .expect("hero image node should exist");
        let overlay_noise = runtime
            .node_for_figma_id("stitch:0.1.0.0.1")
            .expect("overlay noise node should exist");

        let hero_node = scene
            .get_node(hero_image)
            .expect("hero node should resolve");
        let overlay_node = scene
            .get_node(overlay_noise)
            .expect("overlay node should resolve");

        assert!(
            (hero_node.opacity - 0.8).abs() < 1e-3,
            "hero image opacity should stay authored at 0.8, got {}",
            hero_node.opacity
        );
        assert!(
            (overlay_node.opacity - 0.4).abs() < 1e-3,
            "overlay noise opacity should stay authored at 0.4, got {}",
            overlay_node.opacity
        );
    }

    #[test]
    fn image_search_roots_includes_artifact_subdirectories() {
        let stitch_run = PathBuf::from("artifacts/stitch_import_run");
        if stitch_run.exists() {
            let roots = image_search_roots();
            assert!(roots.iter().any(|root| root == &stitch_run));
        }
    }

    #[test]
    fn resolve_image_path_uses_any_configured_root() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("arthropod_riot_waves_paths_{unique}"));
        let root_a = base.join("a");
        let root_b = base.join("b");
        let target = root_b.join("assets").join("cover.png");
        std::fs::create_dir_all(target.parent().expect("target has parent"))
            .expect("create test directories");
        std::fs::write(&target, b"placeholder").expect("write placeholder file");

        let roots = vec![root_a, root_b];
        let resolved =
            resolve_image_path_with_roots("assets/cover.png", &roots).expect("path should resolve");
        assert_eq!(resolved, target);

        std::fs::remove_dir_all(base).expect("cleanup test directory");
    }

    #[test]
    fn collect_font_assets_reads_assets_directories() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock before unix epoch")
            .as_nanos();
        let base = std::env::temp_dir().join(format!("arthropod_riot_waves_fonts_{unique}"));
        let asset_root = base.join("stitch").join("assets");
        std::fs::create_dir_all(&asset_root).expect("create font asset directory");
        let nested = asset_root.join("fonts");
        std::fs::create_dir_all(&nested).expect("create nested font directory");
        let font_file = nested.join("material_symbols.ttf");
        let non_font = asset_root.join("readme.txt");
        std::fs::write(&font_file, b"font-bytes").expect("write font file");
        std::fs::write(&non_font, b"not-font").expect("write non-font file");

        let roots = vec![base.join("stitch"), asset_root.clone()];
        let fonts = collect_font_assets(&roots);
        assert_eq!(fonts.len(), 1);
        assert_eq!(fonts[0], font_file);

        std::fs::remove_dir_all(base).expect("cleanup test directory");
    }

    #[test]
    fn is_font_asset_path_matches_supported_extensions() {
        assert!(is_font_asset_path(Path::new("x.ttf")));
        assert!(is_font_asset_path(Path::new("x.otf")));
        assert!(is_font_asset_path(Path::new("x.ttc")));
        assert!(is_font_asset_path(Path::new("x.woff")));
        assert!(is_font_asset_path(Path::new("x.woff2")));
        assert!(!is_font_asset_path(Path::new("x.png")));
        assert!(!is_font_asset_path(Path::new("x.css")));
    }
}
