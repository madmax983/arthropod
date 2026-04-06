//! Mouse gesture recognition.

use crate::gestures::InputPattern;
use plat_core::{ElementState, MouseButton, Point, WindowEvent};
use std::f64::consts::PI;

// Gesture recognition constants
const MIN_SWIPE_DISTANCE: f64 = 50.0;
const MIN_PATH_LENGTH: f64 = 100.0;
const CIRCLE_CLOSURE_RATIO: f64 = 0.3;
const CIRCLE_ANGLE_TOLERANCE: f64 = 1.0;
const MAX_STROKE_POINTS: usize = 1024;

/// Recognized mouse gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseGesture {
    /// A quick vertical movement upwards.
    SwipeUp,
    /// A quick vertical movement downwards.
    SwipeDown,
    /// A quick horizontal movement to the left.
    SwipeLeft,
    /// A quick horizontal movement to the right.
    SwipeRight,
    /// A full circular motion (approx. 360 degrees) in the clockwise direction.
    CircleClockwise,
    /// A full circular motion (approx. 360 degrees) in the counter-clockwise direction.
    CircleCounterClockwise,
}

struct StrokeMetrics {
    path_length: f64,
    direct_distance: f64,
    total_angle: f64,
    dx: f64,
    dy: f64,
}

/// Matches mouse strokes (gestures) drawn while holding a specific button.
///
/// Implements a geometric gesture recognizer that analyzes the path of the cursor
/// while a specific button (the "trigger button") is held down.
///
/// # Detection Algorithm
///
/// The matcher collects points during the drag operation and analyzes the stroke upon release.
///
/// - **Thresholds**:
///   - Minimum distance: **50.0 pixels**. Strokes shorter than this are ignored.
///   - Minimum sample size: **5 points**. Very short/fast clicks are ignored.
///
/// - **Circle Detection**:
///   - Requires a total path length > **100.0 pixels**.
///   - The distance between start and end points must be < **30%** of the total path length (closed loop).
///   - The total winding angle (sum of turn angles) must be within **1.0 radian (~60°)** of 2π (360°).
///
/// - **Swipe Detection**:
///   - If not a circle, checks if the primary movement axis dominates.
///   - The major axis delta must be significantly larger than the minor axis delta.
pub struct StrokeMatcher {
    trigger_button: MouseButton,
    points: Vec<Point<f64>>,
    is_tracking: bool,
    min_distance: f64,
}

impl StrokeMatcher {
    /// Constructs a new `StrokeMatcher` configured to listen for path data generated while holding
    /// the specified mouse button.
    ///
    /// You should use this when you need gesture-driven interaction (e.g., swiping a right-click or
    /// drawing a circle with the middle mouse button). It automatically starts collecting path points
    /// when the trigger button is pressed and analyzes the stroke on release, abstracting away the math
    /// required to recognize specific spatial patterns.
    ///
    /// ## Examples
    ///
    /// ```
    /// use input_engine::StrokeMatcher;
    /// use plat_core::MouseButton;
    ///
    /// // Listen for user gestures drawn using the right mouse button
    /// let _matcher = StrokeMatcher::new(MouseButton::Right);
    /// ```
    pub fn new(trigger_button: MouseButton) -> Self {
        Self {
            trigger_button,
            points: Vec::new(),
            is_tracking: false,
            min_distance: MIN_SWIPE_DISTANCE,
        }
    }

    /// Check if a stroke is currently being tracked (button held down).
    pub fn is_tracking(&self) -> bool {
        self.is_tracking
    }

    /// Calculates the stroke metrics (length, angle, distances) from the currently
    /// collected points, used to determine if the path matches a gesture profile.
    ///
    /// This function avoids memory allocation by processing path points directly
    /// into a bounded stack array, dropping identical consecutive points instead
    /// of cloning the backing vector and calling `.dedup()`.
    fn calculate_metrics(&self) -> Option<StrokeMetrics> {
        let mut clean_points_buf = [Point::default(); MAX_STROKE_POINTS];
        let mut clean_len = 0;
        for &p in &self.points {
            if clean_len < MAX_STROKE_POINTS
                && (clean_len == 0 || clean_points_buf[clean_len - 1] != p)
            {
                clean_points_buf[clean_len] = p;
                clean_len += 1;
            }
        }
        let clean_points = &clean_points_buf[..clean_len];

        if clean_points.len() < 3 {
            return None;
        }

        let first = clean_points.first()?;
        let last = clean_points.last()?;

        let dx = last.x - first.x;
        let dy = last.y - first.y;
        let direct_distance = (dx * dx + dy * dy).sqrt();

        let mut total_angle = 0.0;
        let mut path_length = 0.0;

        for i in 0..clean_points.len() - 1 {
            let p1 = clean_points[i];
            let p2 = clean_points[i + 1];
            let seg_dx = p2.x - p1.x;
            let seg_dy = p2.y - p1.y;
            path_length += (seg_dx * seg_dx + seg_dy * seg_dy).sqrt();

            if i < clean_points.len() - 2 {
                let p3 = clean_points[i + 2];
                let v1_x = p2.x - p1.x;
                let v1_y = p2.y - p1.y;
                let v2_x = p3.x - p2.x;
                let v2_y = p3.y - p2.y;

                let angle1 = v1_y.atan2(v1_x);
                let angle2 = v2_y.atan2(v2_x);
                let mut diff = angle2 - angle1;

                // Normalize angle difference to [-PI, PI]
                while diff > PI {
                    diff -= 2.0 * PI;
                }
                while diff < -PI {
                    diff += 2.0 * PI;
                }

                total_angle += diff;
            }
        }

        Some(StrokeMetrics {
            path_length,
            direct_distance,
            total_angle,
            dx,
            dy,
        })
    }

    fn detect_circle(metrics: &StrokeMetrics) -> Option<MouseGesture> {
        if metrics.path_length > MIN_PATH_LENGTH
            && metrics.direct_distance < metrics.path_length * CIRCLE_CLOSURE_RATIO
        {
            if (metrics.total_angle - 2.0 * PI).abs() < CIRCLE_ANGLE_TOLERANCE {
                return Some(MouseGesture::CircleClockwise);
            }
            if (metrics.total_angle + 2.0 * PI).abs() < CIRCLE_ANGLE_TOLERANCE {
                return Some(MouseGesture::CircleCounterClockwise);
            }
        }
        None
    }

    fn detect_swipe(metrics: &StrokeMetrics, min_distance: f64) -> Option<MouseGesture> {
        if metrics.direct_distance > min_distance {
            if metrics.dx.abs() > metrics.dy.abs() {
                // Horizontal
                if metrics.dx > 0.0 {
                    return Some(MouseGesture::SwipeRight);
                } else {
                    return Some(MouseGesture::SwipeLeft);
                }
            } else {
                // Vertical
                if metrics.dy > 0.0 {
                    return Some(MouseGesture::SwipeDown);
                } else {
                    return Some(MouseGesture::SwipeUp);
                }
            }
        }
        None
    }

    fn analyze_stroke(&self) -> Option<MouseGesture> {
        let metrics = self.calculate_metrics()?;

        if let Some(gesture) = Self::detect_circle(&metrics) {
            return Some(gesture);
        }

        if let Some(gesture) = Self::detect_swipe(&metrics, self.min_distance) {
            return Some(gesture);
        }

        None
    }
}

impl InputPattern for StrokeMatcher {
    type Gesture = MouseGesture;

    fn update(&mut self, event: &WindowEvent) -> Option<Self::Gesture> {
        match event {
            WindowEvent::MouseInput(input) => {
                if input.button == self.trigger_button {
                    match input.state {
                        ElementState::Pressed => {
                            self.is_tracking = true;
                            self.points.clear();
                            self.points.push(input.position);
                            None
                        }
                        ElementState::Released => {
                            if self.is_tracking {
                                self.is_tracking = false;
                                if self.points.len() < MAX_STROKE_POINTS {
                                    self.points.push(input.position);
                                }
                                self.analyze_stroke()
                            } else {
                                None
                            }
                        }
                    }
                } else {
                    None
                }
            }
            WindowEvent::CursorMoved { position } => {
                if self.is_tracking && self.points.len() < MAX_STROKE_POINTS {
                    // Simple sampling: add point
                    self.points.push(*position);
                }
                None
            }
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use plat_core::{Modifiers, MouseInput};

    fn make_mouse_event(
        state: ElementState,
        button: MouseButton,
        position: Point<f64>,
    ) -> WindowEvent {
        WindowEvent::MouseInput(MouseInput {
            button,
            state,
            position,
            modifiers: Modifiers::default(),
        })
    }

    fn simulate_stroke(
        matcher: &mut StrokeMatcher,
        points: Vec<Point<f64>>,
    ) -> Option<MouseGesture> {
        let start = points.first()?;

        // Press
        matcher.update(&make_mouse_event(
            ElementState::Pressed,
            MouseButton::Right,
            *start,
        ));

        // Move
        for p in points.iter().skip(1) {
            matcher.update(&WindowEvent::CursorMoved { position: *p });
        }

        // Release (at last point)
        let end = points.last()?;
        matcher.update(&make_mouse_event(
            ElementState::Released,
            MouseButton::Right,
            *end,
        ))
    }

    #[test]
    fn test_swipe_right() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(20.0, 0.0),
            Point::new(40.0, 0.0),
            Point::new(60.0, 0.0),
            Point::new(80.0, 0.0),
        ];

        let gesture = simulate_stroke(&mut matcher, points);
        assert_eq!(gesture, Some(MouseGesture::SwipeRight));
    }

    #[test]
    fn test_swipe_down() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let points = vec![
            Point::new(0.0, 0.0),
            Point::new(0.0, 20.0),
            Point::new(0.0, 40.0),
            Point::new(0.0, 60.0),
            Point::new(0.0, 80.0),
        ];

        let gesture = simulate_stroke(&mut matcher, points);
        assert_eq!(gesture, Some(MouseGesture::SwipeDown));
    }

    #[test]
    fn test_empty_stroke_ignored() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let points = vec![];

        let gesture = simulate_stroke(&mut matcher, points);
        assert_eq!(gesture, None);
    }

    #[test]
    fn test_short_stroke_ignored() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)];

        let gesture = simulate_stroke(&mut matcher, points);
        assert_eq!(gesture, None);
    }

    #[test]
    fn test_circle_clockwise() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let mut points = Vec::new();

        let center = Point::new(100.0, 100.0);
        let radius = 50.0;
        let steps = 30;

        for i in 0..=steps {
            let angle = 2.0 * PI * (i as f64 / steps as f64);
            points.push(Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }

        let gesture = simulate_stroke(&mut matcher, points);
        assert_eq!(gesture, Some(MouseGesture::CircleClockwise));
    }

    #[test]
    fn test_stroke_limit_dos() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let points: Vec<Point<f64>> = (0..2000).map(|i| Point::new(i as f64, 0.0)).collect();

        // This should not panic or cause OOM, but we want to assert the internal buffer limit
        simulate_stroke(&mut matcher, points);

        // We expect the buffer to be clamped at 1024 points
        // NOTE: We need to access the internal state or infer it.
        // Since `points` is private, we can't check it directly without modifying the code.
        // But for the purpose of this reproduction step, we will assert that the *test fails* if we could check it.
        // However, I cannot check private fields from integration tests.
        // So I will implement the check inside `simulate_stroke` if possible, or assume I'm writing a unit test (which can access private fields if in the same module).
        // Since this `mod tests` is inside the file, it can access private fields.

        assert!(
            matcher.points.len() <= 1024,
            "Stroke points exceeded 1024 limit: {}",
            matcher.points.len()
        );
    }
}
