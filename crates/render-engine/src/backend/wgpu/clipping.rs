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

        if let NodeContent::Styled { style } = &parent_node.content
            && style.clips_content
        {
            clip = Some(match clip {
                Some(existing) => rect_intersection(existing, parent_node.bounds)?,
                None => parent_node.bounds,
            });
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
            if let NodeContent::Styled { style } = &sibling.content
                && style.is_mask
            {
                level_mask = Some(sibling.bounds);
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

pub(crate) fn rect_path_for_size(width: f32, height: f32) -> style_engine::VectorPath {
    let mut path = style_engine::VectorPath::new();
    path.move_to(glam::Vec2::new(0.0, 0.0));
    path.line_to(glam::Vec2::new(width, 0.0));
    path.line_to(glam::Vec2::new(width, height));
    path.line_to(glam::Vec2::new(0.0, height));
    path.close();
    path
}
