#[test]
fn test_window_id_sign_extension_simulation() {
    // Simulate 32-bit environment behavior
    // On 32-bit, isize is i32, usize is u32.
    // WindowId is u64.

    // Case 1: ID fits in i32 positive range
    let id_small: u64 = 100;
    let stored_small = id_small as i32; // isize on 32-bit
    let retrieved_small = stored_small as u64; // direct cast
    assert_eq!(id_small, retrieved_small);

    // Case 2: ID is large (has high bit set in 32-bit) e.g. 0xF0000000
    // This represents a pointer or a large ID
    let id_large: u64 = 0xF0000000;
    let stored_large = id_large as i32; // becomes negative: -268435456

    // BUG REPRODUCTION:
    // Direct cast from i32 (isize) to u64 performs sign extension
    let retrieved_large_buggy = stored_large as u64;

    // 0xF0000000 as i32 is -268435456.
    // -268435456 as u64 is 0xFFFFFFFFF0000000 (huge number)

    assert_ne!(
        id_large, retrieved_large_buggy,
        "Direct cast should fail/corrupt data"
    );
    assert_eq!(retrieved_large_buggy, 0xFFFFFFFFF0000000);

    // FIX VERIFICATION:
    // Cast to usize (u32) first, then u64
    let retrieved_large_fixed = stored_large as u32 as u64;
    assert_eq!(
        id_large, retrieved_large_fixed,
        "Cast via usize should preserve value"
    );
}
