use glam::Vec2;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    pub position: Vec2,
    pub zoom: f32,
    pub viewport_size: Vec2,
}

impl Camera {
    pub fn new(viewport_size: Vec2) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            viewport_size,
        }
    }

    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let center = self.viewport_size / 2.0;
        let offset = (world_pos - self.position) * self.zoom;
        center + offset
    }

    pub fn screen_to_world(&self, screen_pos: Vec2) -> Vec2 {
        let center = self.viewport_size / 2.0;
        let offset = screen_pos - center;
        self.position + (offset / self.zoom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_world_to_screen() {
        let viewport = Vec2::new(800.0, 600.0);
        let camera = Camera::new(viewport);

        // At origin, should be center of screen
        assert_eq!(camera.world_to_screen(Vec2::ZERO), Vec2::new(400.0, 300.0));

        // Move right 100 units
        assert_eq!(
            camera.world_to_screen(Vec2::new(100.0, 0.0)),
            Vec2::new(500.0, 300.0)
        );
    }

    #[test]
    fn test_screen_to_world() {
        let viewport = Vec2::new(800.0, 600.0);
        let camera = Camera::new(viewport);

        // Center of screen should be origin
        assert_eq!(camera.screen_to_world(Vec2::new(400.0, 300.0)), Vec2::ZERO);

        // 100 units right of center should be (100, 0)
        assert_eq!(
            camera.screen_to_world(Vec2::new(500.0, 300.0)),
            Vec2::new(100.0, 0.0)
        );
    }

    #[test]
    fn test_zoom() {
        let viewport = Vec2::new(800.0, 600.0);
        let mut camera = Camera::new(viewport);
        camera.zoom = 2.0;

        // At origin, still center
        assert_eq!(camera.world_to_screen(Vec2::ZERO), Vec2::new(400.0, 300.0));

        // Move right 100 units, should be 200 screen units
        assert_eq!(
            camera.world_to_screen(Vec2::new(100.0, 0.0)),
            Vec2::new(600.0, 300.0)
        );

        // Reverse
        assert_eq!(
            camera.screen_to_world(Vec2::new(600.0, 300.0)),
            Vec2::new(100.0, 0.0)
        );
    }

    #[test]
    fn test_pan() {
        let viewport = Vec2::new(800.0, 600.0);
        let mut camera = Camera::new(viewport);
        camera.position = Vec2::new(100.0, 0.0);

        // World (100, 0) is now at center
        assert_eq!(
            camera.world_to_screen(Vec2::new(100.0, 0.0)),
            Vec2::new(400.0, 300.0)
        );

        // World (0, 0) is to the left
        assert_eq!(camera.world_to_screen(Vec2::ZERO), Vec2::new(300.0, 300.0));
    }
}
