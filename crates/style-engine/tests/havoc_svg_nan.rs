use style_engine::{BooleanOp, VectorPath};

#[test]
fn havoc_svg_nan_panic() {
    let mut a = VectorPath::new();
    a.move_to(glam::Vec2::new(0.0, 0.0));
    a.line_to(glam::Vec2::new(10.0, 0.0));
    a.line_to(glam::Vec2::new(10.0, 10.0));
    a.line_to(glam::Vec2::new(0.0, 10.0));
    a.close();

    let mut b = VectorPath::new();
    b.move_to(glam::Vec2::new(f32::NAN, f32::NAN));
    b.line_to(glam::Vec2::new(15.0, 0.0));
    b.line_to(glam::Vec2::new(15.0, 10.0));
    b.line_to(glam::Vec2::new(5.0, 10.0));
    b.close();

    // This should fail gracefully, not panic
    let result = VectorPath::boolean_op(&a, &b, BooleanOp::Union);
    assert!(
        result.is_err(),
        "Expected boolean_op to return an error when paths contain NaN"
    );
}
