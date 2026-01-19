//! Unit tests for plat-core type definitions.

use plat_core::{Point, Position, Rect, Size};

#[test]
fn test_size_new() {
    let size = Size::new(800, 600);
    assert_eq!(size.width, 800);
    assert_eq!(size.height, 600);
}

#[test]
fn test_size_default() {
    let size: Size<u32> = Size::default();
    assert_eq!(size.width, 0);
    assert_eq!(size.height, 0);
}

#[test]
fn test_position_new() {
    let pos = Position::new(100, 200);
    assert_eq!(pos.x, 100);
    assert_eq!(pos.y, 200);
}

#[test]
fn test_position_default() {
    let pos: Position<i32> = Position::default();
    assert_eq!(pos.x, 0);
    assert_eq!(pos.y, 0);
}

#[test]
fn test_point_new() {
    let point = Point::new(10.5, 20.3);
    assert_eq!(point.x, 10.5);
    assert_eq!(point.y, 20.3);
}

#[test]
fn test_point_default() {
    let point: Point<f64> = Point::default();
    assert_eq!(point.x, 0.0);
    assert_eq!(point.y, 0.0);
}

#[test]
fn test_rect_new() {
    let rect = Rect::new(10.0, 20.0, 100.0, 50.0);
    assert_eq!(rect.x, 10.0);
    assert_eq!(rect.y, 20.0);
    assert_eq!(rect.width, 100.0);
    assert_eq!(rect.height, 50.0);
}

#[test]
fn test_rect_default() {
    let rect = Rect::default();
    assert_eq!(rect.x, 0.0);
    assert_eq!(rect.y, 0.0);
    assert_eq!(rect.width, 0.0);
    assert_eq!(rect.height, 0.0);
}

#[test]
fn test_rect_contains_inside() {
    let rect = Rect::new(10.0, 20.0, 100.0, 50.0);

    // Point inside
    assert!(rect.contains(50.0, 40.0));

    // Top-left corner (inclusive)
    assert!(rect.contains(10.0, 20.0));

    // Bottom-left (just inside)
    assert!(rect.contains(10.0, 69.9));

    // Top-right (just inside)
    assert!(rect.contains(109.9, 20.0));
}

#[test]
fn test_rect_contains_outside() {
    let rect = Rect::new(10.0, 20.0, 100.0, 50.0);

    // Point to the left
    assert!(!rect.contains(5.0, 40.0));

    // Point to the right
    assert!(!rect.contains(110.0, 40.0));

    // Point above
    assert!(!rect.contains(50.0, 15.0));

    // Point below
    assert!(!rect.contains(50.0, 70.0));

    // Bottom-right corner (exclusive)
    assert!(!rect.contains(110.0, 70.0));
}

#[test]
fn test_rect_contains_edges() {
    let rect = Rect::new(0.0, 0.0, 10.0, 10.0);

    // Left and top edges are inclusive
    assert!(rect.contains(0.0, 0.0));
    assert!(rect.contains(0.0, 5.0));
    assert!(rect.contains(5.0, 0.0));

    // Right and bottom edges are exclusive
    assert!(!rect.contains(10.0, 0.0));
    assert!(!rect.contains(0.0, 10.0));
    assert!(!rect.contains(10.0, 10.0));
}

#[test]
fn test_rect_union_overlapping() {
    let rect1 = Rect::new(0.0, 0.0, 10.0, 10.0);
    let rect2 = Rect::new(5.0, 5.0, 10.0, 10.0);

    let union = rect1.union(&rect2);

    assert_eq!(union.x, 0.0);
    assert_eq!(union.y, 0.0);
    assert_eq!(union.width, 15.0);
    assert_eq!(union.height, 15.0);
}

#[test]
fn test_rect_union_separate() {
    let rect1 = Rect::new(0.0, 0.0, 10.0, 10.0);
    let rect2 = Rect::new(20.0, 20.0, 10.0, 10.0);

    let union = rect1.union(&rect2);

    // Should create a bounding rectangle that contains both
    assert_eq!(union.x, 0.0);
    assert_eq!(union.y, 0.0);
    assert_eq!(union.width, 30.0);
    assert_eq!(union.height, 30.0);
}

#[test]
fn test_rect_union_nested() {
    let rect1 = Rect::new(0.0, 0.0, 100.0, 100.0);
    let rect2 = Rect::new(25.0, 25.0, 50.0, 50.0);

    let union = rect1.union(&rect2);

    // Union should be same as the larger rectangle
    assert_eq!(union.x, 0.0);
    assert_eq!(union.y, 0.0);
    assert_eq!(union.width, 100.0);
    assert_eq!(union.height, 100.0);
}

#[test]
fn test_rect_union_same() {
    let rect = Rect::new(10.0, 20.0, 30.0, 40.0);
    let union = rect.union(&rect);

    // Union with itself should be itself
    assert_eq!(union.x, rect.x);
    assert_eq!(union.y, rect.y);
    assert_eq!(union.width, rect.width);
    assert_eq!(union.height, rect.height);
}

#[test]
fn test_rect_union_negative_coordinates() {
    let rect1 = Rect::new(-10.0, -10.0, 20.0, 20.0);
    let rect2 = Rect::new(5.0, 5.0, 10.0, 10.0);

    let union = rect1.union(&rect2);

    assert_eq!(union.x, -10.0);
    assert_eq!(union.y, -10.0);
    assert_eq!(union.width, 25.0);
    assert_eq!(union.height, 25.0);
}
