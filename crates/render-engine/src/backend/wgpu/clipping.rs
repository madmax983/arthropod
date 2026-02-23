use crate::Scene;

pub(crate) fn rect_intersection(a: plat_core::Rect, b: plat_core::Rect) -> Option<plat_core::Rect> {
    let x0 = a.x.max(b.x);
    let y0 = a.y.max(b.y);
    let x1 = (a.x + a.width).min(b.x + b.width);
    let y1 = (a.y + a.height).min(b.y + b.height);

    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    Some(plat_core::Rect::new(x0, y0, x1 - x0, y1 - y0))
}

pub(crate) fn ancestor_clip_bounds(
    scene: &Scene,
    node_id: crate::NodeId,
) -> Option<plat_core::Rect> {
    use crate::NodeContent;

    let mut current = scene.parent(node_id);
    let mut clip: Option<plat_core::Rect> = None;

    while let Some(parent_id) = current {
        let Some(parent_node) = scene.get_node(parent_id) else {
            break;
        };

        match &parent_node.content {
            NodeContent::Styled { style } if style.clips_content => {
                clip = Some(match clip {
                    Some(existing) => rect_intersection(existing, parent_node.bounds)?,
                    None => parent_node.bounds,
                });
            }
            _ => {}
        }

        current = parent_node.parent;
    }

    clip
}

/// Compute active sibling-mask bounds for `node_id` across ancestor levels.
///
/// Figma-style behavior is approximated by selecting the last preceding mask sibling
/// in each ancestor level and intersecting those bounds.
pub(crate) fn ancestor_mask_bounds(
    scene: &Scene,
    node_id: crate::NodeId,
) -> Option<plat_core::Rect> {
    use crate::NodeContent;

    let mut current = node_id;
    let mut mask_clip: Option<plat_core::Rect> = None;

    while let Some(parent_id) = scene.parent(current) {
        let Some(parent_node) = scene.get_node(parent_id) else {
            break;
        };

        let mut level_mask: Option<plat_core::Rect> = None;
        for &sibling_id in &parent_node.children {
            if sibling_id == current {
                break;
            }
            let Some(sibling) = scene.get_node(sibling_id) else {
                continue;
            };
            if !sibling.visible || sibling.opacity <= 0.0 {
                continue;
            }
            match &sibling.content {
                NodeContent::Styled { style } if style.is_mask => {
                    level_mask = Some(sibling.bounds);
                }
                _ => {}
            }
        }

        if let Some(level_mask) = level_mask {
            mask_clip = Some(match mask_clip {
                Some(existing) => rect_intersection(existing, level_mask)?,
                None => level_mask,
            });
        }

        current = parent_id;
    }

    mask_clip
}

pub(crate) fn clipped_bounds_for_node(
    scene: &Scene,
    node_id: crate::NodeId,
    node_bounds: plat_core::Rect,
) -> Option<plat_core::Rect> {
    let mut clipped = node_bounds;
    if let Some(clip) = ancestor_clip_bounds(scene, node_id) {
        clipped = rect_intersection(clipped, clip)?;
    }
    if let Some(mask) = ancestor_mask_bounds(scene, node_id) {
        clipped = rect_intersection(clipped, mask)?;
    }
    Some(clipped)
}

pub(crate) fn rect_to_scissor_bounds(
    bounds: plat_core::Rect,
    frame_width: u32,
    frame_height: u32,
) -> Option<[u32; 4]> {
    let x0 = bounds.x.max(0.0).min(frame_width as f32).floor() as u32;
    let y0 = bounds.y.max(0.0).min(frame_height as f32).floor() as u32;
    let x1 = (bounds.x + bounds.width)
        .max(0.0)
        .min(frame_width as f32)
        .ceil() as u32;
    let y1 = (bounds.y + bounds.height)
        .max(0.0)
        .min(frame_height as f32)
        .ceil() as u32;

    if x1 <= x0 || y1 <= y0 {
        return None;
    }

    Some([x0, y0, x1 - x0, y1 - y0])
}

pub(crate) fn rounded_rect_path_for_size(
    width: f32,
    height: f32,
    radii: style_engine::CornerRadii,
) -> style_engine::VectorPath {
    let width = width.max(0.0);
    let height = height.max(0.0);
    if width <= 0.0 || height <= 0.0 {
        return style_engine::VectorPath::new();
    }

    let mut tl = radii.top_left.max(0.0);
    let mut tr = radii.top_right.max(0.0);
    let mut br = radii.bottom_right.max(0.0);
    let mut bl = radii.bottom_left.max(0.0);

    let max_radius = (width.min(height)) * 0.5;
    tl = tl.min(max_radius);
    tr = tr.min(max_radius);
    br = br.min(max_radius);
    bl = bl.min(max_radius);

    let top = tl + tr;
    let bottom = bl + br;
    let left = tl + bl;
    let right = tr + br;
    let scale = 1.0_f32
        .min(if top > 0.0 { width / top } else { 1.0 })
        .min(if bottom > 0.0 { width / bottom } else { 1.0 })
        .min(if left > 0.0 { height / left } else { 1.0 })
        .min(if right > 0.0 { height / right } else { 1.0 });
    if scale < 1.0 {
        tl *= scale;
        tr *= scale;
        br *= scale;
        bl *= scale;
    }

    let mut path = style_engine::VectorPath::new();
    path.move_to(glam::Vec2::new(tl, 0.0));
    path.line_to(glam::Vec2::new(width - tr, 0.0));
    if tr > 0.0 {
        path.quadratic_to(glam::Vec2::new(width, 0.0), glam::Vec2::new(width, tr));
    }
    path.line_to(glam::Vec2::new(width, height - br));
    if br > 0.0 {
        path.quadratic_to(
            glam::Vec2::new(width, height),
            glam::Vec2::new(width - br, height),
        );
    }
    path.line_to(glam::Vec2::new(bl, height));
    if bl > 0.0 {
        path.quadratic_to(
            glam::Vec2::new(0.0, height),
            glam::Vec2::new(0.0, height - bl),
        );
    }
    path.line_to(glam::Vec2::new(0.0, tl));
    if tl > 0.0 {
        path.quadratic_to(glam::Vec2::new(0.0, 0.0), glam::Vec2::new(tl, 0.0));
    }
    path.close();
    path
}
