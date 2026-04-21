use std::any::Any;
use std::sync::{Arc, RwLock};

/// Macro to generate downcast checks for common types
macro_rules! downcast_check {
    ($any:expr, $($t:ty),+) => {
        $(
            if let Some(guard) = $any.downcast_ref::<RwLock<$t>>() {
                return match guard.try_read() {
                    Ok(val) => {
                        let formatted = format!("{:?}", *val);
                        if formatted.len() > 1000 {
                            let mut max_len = 1000;
                            while !formatted.is_char_boundary(max_len) {
                                max_len -= 1;
                            }
                            Some(format!("{}...", &formatted[..max_len]))
                        } else {
                            Some(formatted)
                        }
                    },
                    Err(_) => Some("<locked>".to_string()),
                };
            }
        )+
    };
}

/// Attempts to inspect the value of a dynamically typed signal or computed value.
/// Returns a string representation if the type is supported, or None if it's an opaque type.
pub fn try_inspect_value(any: &Arc<dyn Any + Send + Sync>) -> Option<String> {
    downcast_check!(
        any,
        i8,
        i16,
        i32,
        i64,
        isize,
        u8,
        u16,
        u32,
        u64,
        usize,
        f32,
        f64,
        bool,
        String,
        &'static str
    );

    // Fallback for types that we didn't explicitly register
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_truncate_multibyte_safely() {
        let s = format!("a{}", "あ".repeat(334));
        let any: Arc<dyn Any + Send + Sync> = Arc::new(RwLock::new(s));
        let result = try_inspect_value(&any);
        assert!(result.is_some());
        let result_str = result.unwrap();
        assert!(result_str.ends_with("..."));
        assert!(result_str.len() <= 1003); // 1000 max_len + 3 for "..."
    }

    #[test]
    fn should_inspect_supported_types() {
        // Table-driven test for supported types
        let test_cases: Vec<(Arc<dyn Any + Send + Sync>, String)> = vec![
            (Arc::new(RwLock::new(42i8)), "42".to_string()),
            (Arc::new(RwLock::new(42i16)), "42".to_string()),
            (Arc::new(RwLock::new(42i32)), "42".to_string()),
            (Arc::new(RwLock::new(42i64)), "42".to_string()),
            (Arc::new(RwLock::new(42isize)), "42".to_string()),
            (Arc::new(RwLock::new(42u8)), "42".to_string()),
            (Arc::new(RwLock::new(42u16)), "42".to_string()),
            (Arc::new(RwLock::new(42u32)), "42".to_string()),
            (Arc::new(RwLock::new(42u64)), "42".to_string()),
            (Arc::new(RwLock::new(42usize)), "42".to_string()),
            (Arc::new(RwLock::new(3.14f32)), "3.14".to_string()),
            (Arc::new(RwLock::new(3.14f64)), "3.14".to_string()),
            (Arc::new(RwLock::new(true)), "true".to_string()),
            (
                Arc::new(RwLock::new("hello".to_string())),
                "\"hello\"".to_string(),
            ),
            (Arc::new(RwLock::new("world")), "\"world\"".to_string()),
        ];

        for (any, expected) in test_cases {
            let result = try_inspect_value(&any);
            assert_eq!(result, Some(expected));
        }
    }

    #[test]
    fn should_return_none_for_unsupported_types() {
        let unsupported: Arc<dyn Any + Send + Sync> = Arc::new(RwLock::new(vec![1, 2, 3]));
        let result = try_inspect_value(&unsupported);
        assert_eq!(result, None);
    }

    #[test]
    fn should_return_locked_when_lock_fails() {
        let val: Arc<RwLock<i32>> = Arc::new(RwLock::new(42));
        let any: Arc<dyn Any + Send + Sync> = val.clone();

        // Acquire the write lock to simulate contention
        let _guard = val.write().unwrap();

        let result = try_inspect_value(&any);
        assert_eq!(result, Some("<locked>".to_string()));
    }
}
