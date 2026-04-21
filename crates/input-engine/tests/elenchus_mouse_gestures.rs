#![cfg(feature = "nova")]

#[cfg(test)]
mod tests {
    use input_engine::InputPattern;
    use input_engine::{MouseGesture, StrokeMatcher};
    use plat_core::{ElementState, Modifiers, MouseButton, MouseInput, Point, WindowEvent};
    use std::f64::consts::PI;

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
    fn test_swipe_up() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let points = vec![
            Point::new(0.0, 100.0),
            Point::new(0.0, 80.0),
            Point::new(0.0, 60.0),
            Point::new(0.0, 40.0),
            Point::new(0.0, 20.0),
            Point::new(0.0, 0.0),
        ];

        let gesture = simulate_stroke(&mut matcher, points);
        assert_eq!(gesture, Some(MouseGesture::SwipeUp));
    }

    #[test]
    fn test_swipe_left() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let points = vec![
            Point::new(100.0, 0.0),
            Point::new(80.0, 0.0),
            Point::new(60.0, 0.0),
            Point::new(40.0, 0.0),
            Point::new(20.0, 0.0),
            Point::new(0.0, 0.0),
        ];

        let gesture = simulate_stroke(&mut matcher, points);
        assert_eq!(gesture, Some(MouseGesture::SwipeLeft));
    }

    #[test]
    fn test_circle_counter_clockwise() {
        let mut matcher = StrokeMatcher::new(MouseButton::Right);
        let mut points = Vec::new();

        let center = Point::new(100.0, 100.0);
        let radius = 50.0;
        let steps = 30;

        for i in 0..=steps {
            // Negative angle for counter-clockwise
            let angle = -2.0 * PI * (i as f64 / steps as f64);
            points.push(Point::new(
                center.x + radius * angle.cos(),
                center.y + radius * angle.sin(),
            ));
        }

        let gesture = simulate_stroke(&mut matcher, points);
        assert_eq!(gesture, Some(MouseGesture::CircleCounterClockwise));
    }
}
