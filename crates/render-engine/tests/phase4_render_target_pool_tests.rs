use render_engine::backend::wgpu::effects::{RenderTargetKey, RenderTargetPool};

#[test]
fn test_pool_enforces_soft_budget_and_evicts_oldest_free_targets() {
    let mut pool = RenderTargetPool::new(1024);
    let key = RenderTargetKey::new(32, 32, false);

    let h1 = pool.acquire(key, |_| (1, 700));
    let h2 = pool.acquire(key, |_| (2, 700));
    pool.release(h1).expect("release first handle");
    pool.release(h2).expect("release second handle");

    let mut evicted = Vec::new();
    pool.end_frame_with(|handle| evicted.push(handle));

    assert!(
        pool.free_bytes() <= pool.soft_budget_bytes(),
        "free bytes {} exceeded budget {}",
        pool.free_bytes(),
        pool.soft_budget_bytes()
    );
    assert_eq!(
        evicted,
        vec![1],
        "oldest free target should be evicted first"
    );
}
