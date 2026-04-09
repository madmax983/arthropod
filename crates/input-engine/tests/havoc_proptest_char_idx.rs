use proptest::prelude::*;

fn char_idx_to_byte_idx(s: &str, char_idx: usize) -> Option<usize> {
    let mut count = 0;
    for (idx, _) in s.char_indices() {
        if count == char_idx {
            return Some(idx);
        }
        count += 1;
    }
    // If we reached here, char_idx >= count
    if count == char_idx {
        Some(s.len())
    } else {
        None
    }
}

proptest! {
    #[test]
    fn test_char_idx_to_byte_idx(
        s in ".*",
        idx in 0..1000usize
    ) {
        let res = char_idx_to_byte_idx(&s, idx);
        if idx <= s.chars().count() {
            assert!(res.is_some());
        } else {
            assert!(res.is_none());
        }
    }
}
