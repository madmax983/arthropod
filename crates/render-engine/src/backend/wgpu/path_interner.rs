use crate::backend::wgpu::pipelines::path_pipeline::TessellationCache;
use std::collections::HashMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PathFingerprint {
    command_len: usize,
    winding: style_engine::WindingRule,
    first: u64,
    last: u64,
}

#[derive(Debug, Clone, Copy)]
struct InternedPathEntry {
    path_hash: u64,
    fingerprint: PathFingerprint,
}

#[derive(Debug, Default)]
pub(crate) struct PathInterner {
    by_ptr: HashMap<usize, InternedPathEntry>,
}

impl PathInterner {
    fn fingerprint(path: &style_engine::VectorPath) -> PathFingerprint {
        fn command_fp(command: style_engine::PathCommand) -> u64 {
            match command {
                style_engine::PathCommand::MoveTo(p) => {
                    0x01u64
                        ^ ((p.x.to_bits() as u64) << 1)
                        ^ ((p.y.to_bits() as u64).rotate_left(17))
                }
                style_engine::PathCommand::LineTo(p) => {
                    0x02u64
                        ^ ((p.x.to_bits() as u64) << 1)
                        ^ ((p.y.to_bits() as u64).rotate_left(17))
                }
                style_engine::PathCommand::QuadraticTo { control, to } => {
                    0x03u64
                        ^ ((control.x.to_bits() as u64) << 1)
                        ^ ((control.y.to_bits() as u64).rotate_left(9))
                        ^ ((to.x.to_bits() as u64).rotate_left(17))
                        ^ ((to.y.to_bits() as u64).rotate_left(29))
                }
                style_engine::PathCommand::CubicTo {
                    control1,
                    control2,
                    to,
                } => {
                    0x04u64
                        ^ ((control1.x.to_bits() as u64) << 1)
                        ^ ((control1.y.to_bits() as u64).rotate_left(7))
                        ^ ((control2.x.to_bits() as u64).rotate_left(13))
                        ^ ((control2.y.to_bits() as u64).rotate_left(19))
                        ^ ((to.x.to_bits() as u64).rotate_left(23))
                        ^ ((to.y.to_bits() as u64).rotate_left(31))
                }
                style_engine::PathCommand::Close => 0x05u64,
            }
        }

        let first = path.commands.first().copied().map(command_fp).unwrap_or(0);
        let last = path.commands.last().copied().map(command_fp).unwrap_or(0);
        PathFingerprint {
            command_len: path.commands.len(),
            winding: path.winding_rule,
            first,
            last,
        }
    }

    pub(crate) fn hash_for(&mut self, path: &style_engine::VectorPath) -> u64 {
        let ptr = path as *const style_engine::VectorPath as usize;
        let fingerprint = Self::fingerprint(path);
        if let Some(entry) = self.by_ptr.get(&ptr)
            && entry.fingerprint == fingerprint
        {
            return entry.path_hash;
        }

        let path_hash = TessellationCache::fill_key(path);
        self.by_ptr.insert(
            ptr,
            InternedPathEntry {
                path_hash,
                fingerprint,
            },
        );
        path_hash
    }
}
