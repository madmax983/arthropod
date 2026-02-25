//! Mouse gesture recognition.

use crate::gestures::InputPattern;
use plat_core::{ElementState, MouseButton, Point, WindowEvent};
use std::f64::consts::PI;

/// Recognized mouse gestures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MouseGesture {
    SwipeUp,
    SwipeDown,
    SwipeLeft,
    SwipeRight,
    CircleClockwise,
    CircleCounterClockwise,
}

/// Matches mouse strokes (gestures) drawn while holding a specific button.
pub struct StrokeMatcher {
    trigger_button: MouseButton,
    points: Vec<Point<f64>>,
    is_tracking: bool,
    min_distance: f64,
}

impl StrokeMatcher {
    pub fn new(trigger_button: MouseButton) -> Self {
        Self {
            trigger_button,
            points: Vec::new(),
            is_tracking: false,
            min_distance: 50.0, // Minimum pixel distance for a valid swipe
        }
    }

    fn analyze_stroke(&self) -> Option<MouseGesture> {
        if self.points.len() < 5 {
            // Not enough points for a gesture
            return None;
        }

        let first = self.points.first()?;
        let last = self.points.last()?;

        let dx = last.x - first.x;
        let dy = last.y - first.y;
        let dist_sq = dx * dx + dy * dy;
        let dist = dist_sq.sqrt();

        // Calculate total winding number (sum of angle changes)
        let mut total_angle = 0.0;
        let mut path_length = 0.0;

        for i in 0..self.points.len() - 1 {
            let p1 = self.points[i];
            let p2 = self.points[i + 1];
            let seg_dx = p2.x - p1.x;
            let seg_dy = p2.y - p1.y;
            path_length += (seg_dx * seg_dx + seg_dy * seg_dy).sqrt();

            if i < self.points.len() - 2 {
                let p3 = self.points[i + 2];
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

        // Circle Detection
        // Criteria:
        // 1. Total angle is close to +/- 2*PI (360 degrees)
        // 2. Start and end points are relatively close (closed loop)
        // 3. Path length is significant
        if path_length > 100.0 && dist < path_length * 0.3 {
            if (total_angle - 2.0 * PI).abs() < 1.0 {
                // Approx 60 degrees tolerance
                return Some(MouseGesture::CircleClockwise);
            }
            if (total_angle + 2.0 * PI).abs() < 1.0 {
                return Some(MouseGesture::CircleCounterClockwise);
            }
        }

        // Swipe Detection
        // Criteria:
        // 1. Distance > Threshold
        // 2. Not a circle (implied by previous check failing or return)
        if dist > self.min_distance {
            if dx.abs() > dy.abs() {
                // Horizontal
                if dx > 0.0 {
                    return Some(MouseGesture::SwipeRight);
                } else {
                    return Some(MouseGesture::SwipeLeft);
                }
            } else {
                // Vertical
                if dy > 0.0 {
                    return Some(MouseGesture::SwipeDown);
                } else {
                    return Some(MouseGesture::SwipeUp);
                }
            }
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
                                self.points.push(input.position);
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
                if self.is_tracking {
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
        let start = points.first().unwrap();

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
        let end = points.last().unwrap();
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
    fn test_short_stroke_ignored() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let points = vec![Point::new(0.0, 0.0), Point::new(10.0, 0.0)];

        let gesture = simulate_stroke(&mut matcher, points);
        assert_eq!(gesture, None);
    }
}
