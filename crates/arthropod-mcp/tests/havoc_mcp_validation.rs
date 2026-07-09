use arthropod_mcp::validation::validate_range;
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_validate_range_bypass(
        value in proptest::num::f32::ANY,
        min in proptest::num::f32::ANY,
        max in proptest::num::f32::ANY
    ) {
        let result = validate_range(value, min, max, "fuzz");

        if result.is_ok() {
            assert!(value >= min, "Bypass: {} is not >= {}", value, min);
            assert!(value <= max, "Bypass: {} is not <= {}", value, max);
            assert!(value.is_finite(), "Bypass: {} is not finite", value);
        }
    }
}
