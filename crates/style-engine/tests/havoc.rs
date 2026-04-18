use proptest::prelude::*;
use style_engine::path::{VectorPath, VectorPathError};

#[test]
fn test_dos_long_number_token() {
    let mut s = String::from("M ");
    s.push_str(&"1".repeat(256));
    s.push_str(" 0 L 0 0 Z");
    let result = VectorPath::from_svg_path_data(&s);
    assert!(
        result.is_err(),
        "Expected parsing to fail for excessively long numeric token"
    );
    if let Err(e) = result {
        match e {
            VectorPathError::InvalidSvgPathData(_) => {}
            _ => panic!("Expected InvalidSvgPathData error"),
        }
    }
}

proptest! {
    #[test]
    fn does_not_crash_on_random_svg(s in "\\PC*") {
        let _ = VectorPath::from_svg_path_data(&s);
    }
}
