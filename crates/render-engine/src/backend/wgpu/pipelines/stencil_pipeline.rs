//! Stencil clip-state helper for Phase 4 clipping/masking.

use crate::backend::wgpu::effects::EffectPassKind;

/// CPU-side clip stack helper mirrored by stencil operations during rendering.
#[derive(Debug, Default, Clone)]
pub struct ClipStack {
    depth: u8,
}

impl ClipStack {
    /// Creates a new, empty clip stack.
    #[must_use]
    pub fn new() -> Self {
        Self { depth: 0 }
    }

    /// Push one clip layer.
    ///
    /// Returns the new depth value or `None` when the stack is full.
    pub fn push(&mut self) -> Option<u8> {
        if self.depth == u8::MAX {
            return None;
        }
        self.depth = self.depth.saturating_add(1);
        Some(self.depth)
    }

    /// Pop one clip layer.
    ///
    /// Returns the new depth value or `None` when already empty.
    pub fn pop(&mut self) -> Option<u8> {
        if self.depth == 0 {
            return None;
        }
        self.depth -= 1;
        Some(self.depth)
    }

    /// Reports how many levels deep the renderer is currently clipping.
    ///
    /// The maximum supported depth in hardware is 255 (the limit of an 8-bit stencil buffer).
    #[must_use]
    pub fn depth(&self) -> u8 {
        self.depth
    }
}

/// Planned stencil sequence for two levels of nested clipping.
#[must_use]
pub fn plan_clip_sequence_for_nested_clips() -> Vec<EffectPassKind> {
    vec![
        EffectPassKind::StencilPush,
        EffectPassKind::StencilPush,
        EffectPassKind::StencilPop,
        EffectPassKind::StencilPop,
    ]
}

/// Returns a depth/stencil state suitable for stencil clipping passes.
#[must_use]
pub fn stencil_state() -> wgpu::DepthStencilState {
    wgpu::DepthStencilState {
        format: wgpu::TextureFormat::Stencil8,
        depth_write_enabled: false,
        depth_compare: wgpu::CompareFunction::Always,
        stencil: wgpu::StencilState {
            front: wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Replace,
            },
            back: wgpu::StencilFaceState {
                compare: wgpu::CompareFunction::Equal,
                fail_op: wgpu::StencilOperation::Keep,
                depth_fail_op: wgpu::StencilOperation::Keep,
                pass_op: wgpu::StencilOperation::Replace,
            },
            read_mask: 0xFF,
            write_mask: 0xFF,
        },
        bias: wgpu::DepthBiasState::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clip_stack_push_pop() {
        let mut stack = ClipStack::new();
        assert_eq!(stack.depth(), 0);
        assert_eq!(stack.push(), Some(1));
        assert_eq!(stack.push(), Some(2));
        assert_eq!(stack.depth(), 2);
        assert_eq!(stack.pop(), Some(1));
        assert_eq!(stack.pop(), Some(0));
        assert_eq!(stack.pop(), None);
    }
}
