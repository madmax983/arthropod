//! Transformation math for mapping world coordinates to screen coordinates.
//!
//! Handles the linear algebra required to zoom and pan the infinite world space
//! onto a bounded screen viewport.

use glam::Vec2;

/// A mathematical camera used to project an infinite 2D plane onto a finite screen viewport.
///
/// This struct holds the linear algebra state (position and zoom) and provides the projection methods
/// to map world-space coordinates (where the entities actually exist) to screen-space coordinates
/// (where pixels are drawn) and vice-versa.
///
/// ## Examples
///
/// ```rust
/// use spatial_graph::transform::Camera;
/// use glam::Vec2;
///
/// let mut camera = Camera::new(Vec2::new(800.0, 600.0));
/// camera.position = Vec2::new(100.0, 50.0);
/// camera.zoom = 2.0;
/// ```
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// The center point of the camera in world coordinates.
    pub position: Vec2,
    /// The zoom scale factor. Values > 1.0 zoom in, < 1.0 zoom out.
    pub zoom: f32,
    /// The dimensions of the screen rendering area (width, height).
    pub viewport_size: Vec2,
}

impl Camera {
    /// Creates a new camera centered at the origin `(0.0, 0.0)` with a zoom level of `1.0`.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spatial_graph::transform::Camera;
    /// use glam::Vec2;
    ///
    /// let camera = Camera::new(Vec2::new(800.0, 600.0));
    /// assert_eq!(camera.zoom, 1.0);
    /// assert_eq!(camera.position, Vec2::ZERO);
    /// ```
    pub fn new(viewport_size: Vec2) -> Self {
        Self {
            position: Vec2::ZERO,
            zoom: 1.0,
            viewport_size,
        }
    }

    /// Converts an absolute position in world space to a relative pixel position on the screen.
    ///
    /// The formula translates the point relative to the camera's position, scales it by the zoom,
    /// and then centers it within the viewport.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spatial_graph::transform::Camera;
    /// use glam::Vec2;
    ///
    /// let camera = Camera::new(Vec2::new(800.0, 600.0));
    /// // The origin in world space is in the dead center of the screen
    /// let screen_pos = camera.world_to_screen(Vec2::ZERO);
    /// assert_eq!(screen_pos, Vec2::new(400.0, 300.0));
    /// ```
    pub fn world_to_screen(&self, world_pos: Vec2) -> Vec2 {
        let center = self.viewport_size / 2.0;
        let offset = (world_pos - self.position) * self.zoom;
        center + offset
    }

    /// Converts a pixel position on the screen to an absolute position in the 2D world space.
    ///
    /// This is typically used for projecting mouse click coordinates into the virtual world
    /// to determine what entity the user clicked on.
    ///
    /// ## Examples
    ///
    /// ```rust
    /// use spatial_graph::transform::Camera;
    /// use glam::Vec2;
    ///
    /// let camera = Camera::new(Vec2::new(800.0, 600.0));
    /// // Clicking the center of the screen targets the camera's current position
    /// let world_pos = camera.screen_to_world(Vec2::new(400.0, 300.0));
    /// assert_eq!(world_pos, Vec2::ZERO);
    /// ```
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
