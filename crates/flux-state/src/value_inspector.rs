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
}
