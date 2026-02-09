use glam::Vec2;
use serde::{Deserialize, Serialize};

/// Winding rule for path filling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
pub enum WindingRule {
    /// Non-zero winding rule
    #[default]
    NonZero,
    /// Even-odd winding rule
    EvenOdd,
}

/// Path command
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum PathCommand {
    /// Move to a point without drawing
    MoveTo(Vec2),
    /// Draw a line to a point
    LineTo(Vec2),
    /// Draw a quadratic Bézier curve
    QuadraticTo {
        /// Control point
        control: Vec2,
        /// End point
        to: Vec2,
    },
    /// Draw a cubic Bézier curve
    CubicTo {
        /// First control point
        control1: Vec2,
        /// Second control point
        control2: Vec2,
        /// End point
        to: Vec2,
    },
    /// Close the current path
    Close,
}

/// Vector path (sequence of path commands)
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VectorPath {
    /// Path commands
    pub commands: Vec<PathCommand>,
    /// Winding rule for filling
    pub winding_rule: WindingRule,
}

impl VectorPath {
    /// Create a new empty path
    pub fn new() -> Self {
        Self {
            commands: Vec::new(),
            winding_rule: WindingRule::default(),
        }
    }

    /// Create a path with specified winding rule
    pub fn with_winding_rule(winding_rule: WindingRule) -> Self {
        Self {
            commands: Vec::new(),
            winding_rule,
        }
    }

    /// Create a path from commands
    pub fn from_commands(commands: Vec<PathCommand>) -> Self {
        Self {
            commands,
            winding_rule: WindingRule::default(),
        }
    }

    /// Add a command to the path
    pub fn push(&mut self, command: PathCommand) {
        self.commands.push(command);
    }

    /// Move to a point
    pub fn move_to(&mut self, point: Vec2) {
        self.push(PathCommand::MoveTo(point));
    }

    /// Draw a line to a point
    pub fn line_to(&mut self, point: Vec2) {
        self.push(PathCommand::LineTo(point));
    }

    /// Draw a quadratic Bézier curve
    pub fn quadratic_to(&mut self, control: Vec2, to: Vec2) {
        self.push(PathCommand::QuadraticTo { control, to });
    }

    /// Draw a cubic Bézier curve
    pub fn cubic_to(&mut self, control1: Vec2, control2: Vec2, to: Vec2) {
        self.push(PathCommand::CubicTo {
            control1,
            control2,
            to,
        });
    }

    /// Close the current path
    pub fn close(&mut self) {
        self.push(PathCommand::Close);
    }
}

impl Default for VectorPath {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_triangle_path() {
        let mut path = VectorPath::new();
        path.move_to(Vec2::new(0.0, 0.0));
        path.line_to(Vec2::new(100.0, 0.0));
        path.line_to(Vec2::new(50.0, 100.0));
        path.close();

        assert_eq!(path.commands.len(), 4);
        assert!(matches!(path.commands[0], PathCommand::MoveTo(_)));
        assert!(matches!(path.commands[1], PathCommand::LineTo(_)));
        assert!(matches!(path.commands[2], PathCommand::LineTo(_)));
        assert!(matches!(path.commands[3], PathCommand::Close));
    }

    #[test]
    fn test_cubic_bezier_construction() {
        let mut path = VectorPath::new();
        path.cubic_to(
            Vec2::new(10.0, 20.0),
            Vec2::new(30.0, 40.0),
            Vec2::new(50.0, 60.0),
        );

        assert_eq!(path.commands.len(), 1);
        match path.commands[0] {
            PathCommand::CubicTo {
                control1,
                control2,
                to,
            } => {
                assert_eq!(control1, Vec2::new(10.0, 20.0));
                assert_eq!(control2, Vec2::new(30.0, 40.0));
                assert_eq!(to, Vec2::new(50.0, 60.0));
            }
            _ => panic!("Expected cubic bezier command"),
        }
    }

    #[test]
    fn test_winding_rule_default() {
        let path = VectorPath::new();
        assert_eq!(path.winding_rule, WindingRule::NonZero);
    }

    #[test]
    fn test_from_commands() {
        let commands = vec![
            PathCommand::MoveTo(Vec2::ZERO),
            PathCommand::LineTo(Vec2::ONE),
            PathCommand::Close,
        ];
        let path = VectorPath::from_commands(commands);

        assert_eq!(path.commands.len(), 3);
        assert_eq!(path.winding_rule, WindingRule::NonZero);
    }

    #[test]
    fn test_with_winding_rule() {
        let path = VectorPath::with_winding_rule(WindingRule::EvenOdd);
        assert_eq!(path.winding_rule, WindingRule::EvenOdd);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut path = VectorPath::new();
        path.move_to(Vec2::ZERO);
        path.line_to(Vec2::ONE);
        path.close();

        let json = serde_json::to_string(&path).expect("serialize failed");
        let deserialized: VectorPath = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(path, deserialized);
    }
}
