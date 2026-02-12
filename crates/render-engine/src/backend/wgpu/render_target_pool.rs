//! Render-target pooling for Phase 4 multipass effects.

use hashbrown::HashMap;
use std::collections::VecDeque;

/// Pool key for offscreen render targets.
///
/// Format is represented as a compact, backend-independent code so this type
/// remains unit-testable without constructing wgpu objects.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RenderTargetKey {
    pub width: u32,
    pub height: u32,
    pub format_code: u16,
    pub has_stencil: bool,
}

impl RenderTargetKey {
    /// Common SRGBA8 format code used by tests and default backend paths.
    pub const FORMAT_SRGBA8: u16 = 1;

    /// Create a key using the default SRGBA8 format code.
    #[must_use]
    pub fn new(width: u32, height: u32, has_stencil: bool) -> Self {
        Self {
            width,
            height,
            format_code: Self::FORMAT_SRGBA8,
            has_stencil,
        }
    }

    /// Estimated texture memory footprint in bytes.
    #[must_use]
    pub fn estimated_bytes(self) -> u64 {
        let color_bytes = self.width as u64 * self.height as u64 * 4;
        let stencil_bytes = if self.has_stencil {
            self.width as u64 * self.height as u64
        } else {
            0
        };
        color_bytes + stencil_bytes
    }
}

/// Lightweight handle for pooled render targets.
pub type RenderTargetHandle = u64;

/// CPU-side render-target pool bookkeeping.
///
/// The actual GPU texture allocation is delegated to the caller through the
/// `acquire` closure, which keeps this type straightforward to unit test.
#[derive(Debug)]
pub struct RenderTargetPool {
    free: HashMap<RenderTargetKey, Vec<RenderTargetHandle>>,
    free_order: VecDeque<RenderTargetHandle>,
    in_use: HashMap<RenderTargetHandle, RenderTargetKey>,
    key_by_handle: HashMap<RenderTargetHandle, RenderTargetKey>,
    bytes_by_handle: HashMap<RenderTargetHandle, u64>,
    bytes_in_use: u64,
    bytes_free: u64,
    soft_budget_bytes: u64,
}

impl RenderTargetPool {
    /// Create a new pool with a soft memory budget.
    #[must_use]
    pub fn new(soft_budget_bytes: u64) -> Self {
        Self {
            free: HashMap::new(),
            free_order: VecDeque::new(),
            in_use: HashMap::new(),
            key_by_handle: HashMap::new(),
            bytes_by_handle: HashMap::new(),
            bytes_in_use: 0,
            bytes_free: 0,
            soft_budget_bytes,
        }
    }

    /// Acquire an offscreen target matching `key`.
    ///
    /// Reuses a free target when available, otherwise calls `create` to allocate
    /// a new one and registers it with the pool.
    pub fn acquire<F>(&mut self, key: RenderTargetKey, create: F) -> RenderTargetHandle
    where
        F: FnOnce(RenderTargetKey) -> (RenderTargetHandle, u64),
    {
        if let Some(handles) = self.free.get_mut(&key)
            && let Some(handle) = handles.pop()
        {
            let bytes = self.bytes_by_handle.get(&handle).copied().unwrap_or(0);
            self.in_use.insert(handle, key);
            self.bytes_in_use = self.bytes_in_use.saturating_add(bytes);
            self.bytes_free = self.bytes_free.saturating_sub(bytes);
            if let Some(pos) = self.free_order.iter().position(|h| *h == handle) {
                self.free_order.remove(pos);
            }
            return handle;
        }

        let (handle, bytes) = create(key);
        self.in_use.insert(handle, key);
        self.key_by_handle.insert(handle, key);
        self.bytes_by_handle.insert(handle, bytes);
        self.bytes_in_use = self.bytes_in_use.saturating_add(bytes);
        handle
    }

    /// Release an in-use target back into the free list.
    pub fn release(&mut self, handle: RenderTargetHandle) -> Result<(), &'static str> {
        let Some(key) = self.in_use.remove(&handle) else {
            return Err("render target handle is not currently in use");
        };
        let bytes = self.bytes_by_handle.get(&handle).copied().unwrap_or(0);
        self.bytes_in_use = self.bytes_in_use.saturating_sub(bytes);
        self.bytes_free = self.bytes_free.saturating_add(bytes);
        self.free.entry(key).or_default().push(handle);
        self.free_order.push_back(handle);
        Ok(())
    }

    /// Return all currently in-use targets to the free list.
    pub fn end_frame(&mut self) {
        self.end_frame_with(|_| {});
    }

    /// Return all in-use targets to free and evict oldest free targets to fit budget.
    pub fn end_frame_with<F>(&mut self, mut evict: F)
    where
        F: FnMut(RenderTargetHandle),
    {
        let drained: Vec<_> = self.in_use.drain().collect();
        for (handle, key) in drained {
            let bytes = self.bytes_by_handle.get(&handle).copied().unwrap_or(0);
            self.bytes_in_use = self.bytes_in_use.saturating_sub(bytes);
            self.bytes_free = self.bytes_free.saturating_add(bytes);
            self.free.entry(key).or_default().push(handle);
            self.free_order.push_back(handle);
        }

        while self.bytes_free > self.soft_budget_bytes {
            let Some(handle) = self.free_order.pop_front() else {
                break;
            };
            let Some(key) = self.key_by_handle.remove(&handle) else {
                continue;
            };

            if let Some(handles) = self.free.get_mut(&key) {
                handles.retain(|h| *h != handle);
                if handles.is_empty() {
                    self.free.remove(&key);
                }
            }

            let bytes = self.bytes_by_handle.remove(&handle).unwrap_or(0);
            self.bytes_free = self.bytes_free.saturating_sub(bytes);
            evict(handle);
        }
    }

    /// Current bytes tracked as in-use.
    #[must_use]
    pub fn bytes_in_use(&self) -> u64 {
        self.bytes_in_use
    }

    /// Current bytes tracked as free and reusable.
    #[must_use]
    pub fn free_bytes(&self) -> u64 {
        self.bytes_free
    }

    /// Soft budget configured for the pool.
    #[must_use]
    pub fn soft_budget_bytes(&self) -> u64 {
        self.soft_budget_bytes
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pool_reuses_released_target_for_same_key() {
        let mut pool = RenderTargetPool::new(8 * 1024 * 1024);
        let key = RenderTargetKey::new(256, 256, false);
        let h1 = pool.acquire(key, |_| (1, 1024));
        pool.release(h1).expect("release should succeed");
        let h2 = pool.acquire(key, |_| (2, 1024));
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_pool_evicts_oldest_targets_when_over_budget() {
        let mut pool = RenderTargetPool::new(1024);
        let key = RenderTargetKey::new(32, 32, false);

        let h1 = pool.acquire(key, |_| (1, 700));
        let h2 = pool.acquire(key, |_| (2, 700));
        pool.release(h1).expect("release h1");
        pool.release(h2).expect("release h2");

        let mut evicted = Vec::new();
        pool.end_frame_with(|h| evicted.push(h));

        assert!(pool.free_bytes() <= pool.soft_budget_bytes());
        assert_eq!(evicted, vec![1]);
    }
}
