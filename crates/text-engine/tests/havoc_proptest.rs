use proptest::prelude::*;
use text_engine::shape_text_parallel;

proptest! {
    #[test]
    fn havoc_font_size_panic(font_size in proptest::num::f32::ANY) {
        // This will crash if font_size <= 0.0 or non-finite
        let _ = shape_text_parallel("Chaos!", font_size);
    }
}
