use style_engine::{BooleanOp, SvgParseOptions, VectorPath};

fn main() {
    // 1) Parse mixed SVG commands (including smooth and arc commands).
    let parsed = VectorPath::from_svg_path_data(
        "M 0 0 Q 20 40 40 0 T 80 0 C 90 10 110 10 120 0 S 150 -10 160 0 A 30 20 0 0 1 220 0 Z",
    )
    .expect("SVG path parse should succeed");

    // 2) Parse with custom arc precision.
    let _fine = VectorPath::from_svg_path_data_with_options(
        "M 0 0 A 40 20 0 1 1 80 0",
        SvgParseOptions {
            max_arc_step_degrees: 6.0,
        },
    )
    .expect("SVG arc parse with options should succeed");

    // 3) Boolean ops smoke.
    let a = VectorPath::from_svg_path_data("M 0 0 L 80 0 L 80 80 L 0 80 Z").expect("parse a");
    let b = VectorPath::from_svg_path_data("M 40 0 L 120 0 L 120 80 L 40 80 Z").expect("parse b");

    let union = VectorPath::boolean_op(&a, &b, BooleanOp::Union).expect("union");
    let subtract = VectorPath::boolean_op(&a, &b, BooleanOp::Subtract).expect("subtract");
    let intersect = VectorPath::boolean_op(&a, &b, BooleanOp::Intersect).expect("intersect");
    let exclude = VectorPath::boolean_op(&a, &b, BooleanOp::Exclude).expect("exclude");

    // 4) Hit-test smoke (boundary-inclusive).
    assert!(union.contains_point(10.0, 40.0));
    assert!(union.contains_point(100.0, 40.0));
    assert!(!union.contains_point(140.0, 40.0));

    assert!(subtract.contains_point(10.0, 40.0));
    assert!(!subtract.contains_point(60.0, 40.0));

    assert!(intersect.contains_point(60.0, 40.0));
    assert!(!intersect.contains_point(20.0, 40.0));

    assert!(exclude.contains_point(20.0, 40.0));
    assert!(exclude.contains_point(100.0, 40.0));
    assert!(!exclude.contains_point(60.0, 40.0));

    // Boundary checks.
    assert!(a.contains_point(0.0, 0.0));
    assert!(a.contains_point(40.0, 0.0));

    println!(
        "Phase 3 path smoke passed: parsed={} commands, union={} commands",
        parsed.commands.len(),
        union.commands.len()
    );
}
