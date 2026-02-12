use geo::{BooleanOps, Coord, LineString, MultiPolygon, Polygon};
use glam::Vec2;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

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

/// Boolean operation type for vector paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum BooleanOp {
    Union,
    Subtract,
    Intersect,
    Exclude,
}

/// Errors produced by vector path parsing/operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VectorPathError {
    InvalidSvgPathData(String),
    UnsupportedSvgCommand(char),
    InvalidBooleanOperands,
}

/// SVG parser configuration options for path flattening precision.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SvgParseOptions {
    /// Maximum angular step for arc flattening in degrees.
    /// Smaller values increase fidelity and segment count.
    pub max_arc_step_degrees: f32,
}

impl Default for SvgParseOptions {
    fn default() -> Self {
        Self {
            max_arc_step_degrees: 22.5,
        }
    }
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

    /// Test if a point is inside this path using winding/even-odd fill rules.
    pub fn contains_point(&self, x: f32, y: f32) -> bool {
        let point = Vec2::new(x, y);
        match self.winding_rule {
            WindingRule::NonZero => self.winding_number(point) != 0,
            WindingRule::EvenOdd => self.crossing_count(point) % 2 == 1,
        }
    }

    /// Parse SVG path `d` data into a `VectorPath`.
    ///
    /// Supported commands: `M/m`, `L/l`, `H/h`, `V/v`, `Q/q`, `T/t`, `C/c`, `S/s`, `A/a`, `Z/z`.
    pub fn from_svg_path_data(data: &str) -> Result<Self, VectorPathError> {
        Self::from_svg_path_data_with_options(data, SvgParseOptions::default())
    }

    /// Parse SVG path `d` data into a `VectorPath` with custom flattening options.
    pub fn from_svg_path_data_with_options(
        data: &str,
        options: SvgParseOptions,
    ) -> Result<Self, VectorPathError> {
        let mut parser = SvgPathParser::new(data, options);
        parser.parse()
    }

    /// Perform a boolean operation between two paths.
    ///
    /// Curves are flattened to polylines for polygon conversion.
    pub fn boolean_op(
        a: &VectorPath,
        b: &VectorPath,
        op: BooleanOp,
    ) -> Result<VectorPath, VectorPathError> {
        let a_poly = a.to_multi_polygon();
        let b_poly = b.to_multi_polygon();

        if a_poly.0.is_empty() || b_poly.0.is_empty() {
            return Err(VectorPathError::InvalidBooleanOperands);
        }

        let out = match op {
            BooleanOp::Union => a_poly.union(&b_poly),
            BooleanOp::Subtract => a_poly.difference(&b_poly),
            BooleanOp::Intersect => a_poly.intersection(&b_poly),
            BooleanOp::Exclude => a_poly.xor(&b_poly),
        };

        Ok(Self::from_multi_polygon(&out))
    }

    fn winding_number(&self, point: Vec2) -> i32 {
        let mut winding = 0;
        for contour in self.flattened_contours(12) {
            if contour.len() < 3 {
                continue;
            }
            for edge in contour.windows(2) {
                let p1 = edge[0];
                let p2 = edge[1];
                if point_on_segment(p1, p2, point, 1e-5) {
                    return 1;
                }
                if p1.y <= point.y {
                    if p2.y > point.y && is_left(p1, p2, point) > 0.0 {
                        winding += 1;
                    }
                } else if p2.y <= point.y && is_left(p1, p2, point) < 0.0 {
                    winding -= 1;
                }
            }
        }
        winding
    }

    fn crossing_count(&self, point: Vec2) -> usize {
        let mut crossings = 0usize;
        for contour in self.flattened_contours(12) {
            if contour.len() < 3 {
                continue;
            }
            for edge in contour.windows(2) {
                let p1 = edge[0];
                let p2 = edge[1];
                if point_on_segment(p1, p2, point, 1e-5) {
                    return 1;
                }
                if (p1.y > point.y) != (p2.y > point.y) {
                    let t = (point.y - p1.y) / (p2.y - p1.y);
                    let x = p1.x + t * (p2.x - p1.x);
                    if x > point.x {
                        crossings += 1;
                    }
                }
            }
        }
        crossings
    }

    fn to_multi_polygon(&self) -> MultiPolygon<f64> {
        #[derive(Clone)]
        struct RingData {
            contour: Vec<Vec2>,
            area_signed: f32,
            area_abs: f32,
            depth: usize,
            parent: Option<usize>,
        }

        let contours: Vec<Vec<Vec2>> = self
            .flattened_contours(12)
            .into_iter()
            .filter(|c| c.len() >= 4)
            .collect();
        if contours.is_empty() {
            return MultiPolygon(vec![]);
        }

        let mut rings: Vec<RingData> = contours
            .into_iter()
            .map(|contour| RingData {
                area_signed: signed_area(&contour),
                area_abs: signed_area(&contour).abs(),
                contour,
                depth: 0,
                parent: None,
            })
            .collect();

        for i in 0..rings.len() {
            let sample = first_distinct_point(&rings[i].contour).unwrap_or(Vec2::ZERO);
            let mut containing: Vec<usize> = Vec::new();
            for (j, ring_j) in rings.iter().enumerate() {
                if i == j {
                    continue;
                }
                if point_in_ring(sample, &ring_j.contour) {
                    containing.push(j);
                }
            }

            rings[i].depth = containing.len();
            rings[i].parent = containing.into_iter().min_by(|a, b| {
                rings[*a]
                    .area_abs
                    .partial_cmp(&rings[*b].area_abs)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
        }

        let is_hole = |idx: usize| -> bool {
            let ring = &rings[idx];
            let Some(parent_idx) = ring.parent else {
                return false;
            };
            match self.winding_rule {
                WindingRule::EvenOdd => ring.depth % 2 == 1,
                WindingRule::NonZero => {
                    let parent = &rings[parent_idx];
                    ring.area_signed.signum() != parent.area_signed.signum()
                }
            }
        };

        let mut outer_indices = Vec::new();
        for i in 0..rings.len() {
            if !is_hole(i) {
                outer_indices.push(i);
            }
        }

        let mut outer_to_poly = std::collections::HashMap::new();
        let mut exteriors = Vec::new();
        let mut holes_by_poly: Vec<Vec<LineString<f64>>> = Vec::new();
        for (poly_idx, ring_idx) in outer_indices.iter().copied().enumerate() {
            outer_to_poly.insert(ring_idx, poly_idx);
            exteriors.push(ring_to_linestring(&rings[ring_idx].contour));
            holes_by_poly.push(Vec::new());
        }

        for ring_idx in 0..rings.len() {
            if !is_hole(ring_idx) {
                continue;
            }

            let mut current = rings[ring_idx].parent;
            while let Some(parent_idx) = current {
                if let Some(&poly_idx) = outer_to_poly.get(&parent_idx) {
                    holes_by_poly[poly_idx].push(ring_to_linestring(&rings[ring_idx].contour));
                    break;
                }
                current = rings[parent_idx].parent;
            }
        }

        let mut polygons = Vec::with_capacity(exteriors.len());
        for (exterior, holes) in exteriors.into_iter().zip(holes_by_poly.into_iter()) {
            polygons.push(Polygon::new(exterior, holes));
        }
        MultiPolygon(polygons)
    }

    fn from_multi_polygon(multi: &MultiPolygon<f64>) -> Self {
        let mut out = VectorPath::with_winding_rule(WindingRule::EvenOdd);

        fn push_ring(path: &mut VectorPath, ring: &LineString<f64>) {
            let mut iter = ring.points();
            if let Some(first) = iter.next() {
                path.move_to(Vec2::new(first.x() as f32, first.y() as f32));
                for p in iter {
                    path.line_to(Vec2::new(p.x() as f32, p.y() as f32));
                }
                path.close();
            }
        }

        for poly in &multi.0 {
            push_ring(&mut out, poly.exterior());
            for hole in poly.interiors() {
                push_ring(&mut out, hole);
            }
        }
        out
    }

    fn flattened_contours(&self, curve_steps: usize) -> Vec<Vec<Vec2>> {
        let mut contours: Vec<Vec<Vec2>> = Vec::new();
        let mut current: Vec<Vec2> = Vec::new();
        let mut cursor = Vec2::ZERO;
        let mut start = Vec2::ZERO;
        let mut has_cursor = false;

        for cmd in &self.commands {
            match *cmd {
                PathCommand::MoveTo(p) => {
                    if current.len() >= 2 {
                        contours.push(current);
                    }
                    current = vec![p];
                    cursor = p;
                    start = p;
                    has_cursor = true;
                }
                PathCommand::LineTo(p) if has_cursor => {
                    current.push(p);
                    cursor = p;
                }
                PathCommand::QuadraticTo { control, to } if has_cursor => {
                    for i in 1..=curve_steps {
                        let t = i as f32 / curve_steps as f32;
                        current.push(quadratic_point(cursor, control, to, t));
                    }
                    cursor = to;
                }
                PathCommand::CubicTo {
                    control1,
                    control2,
                    to,
                } if has_cursor => {
                    for i in 1..=curve_steps {
                        let t = i as f32 / curve_steps as f32;
                        current.push(cubic_point(cursor, control1, control2, to, t));
                    }
                    cursor = to;
                }
                PathCommand::Close if has_cursor => {
                    if current.last().copied() != Some(start) {
                        current.push(start);
                    }
                    if current.len() >= 2 {
                        contours.push(current);
                    }
                    current = Vec::new();
                    has_cursor = false;
                }
                _ => {}
            }
        }

        if current.len() >= 2 {
            if let (Some(first), Some(last)) = (current.first().copied(), current.last().copied())
                && first != last
            {
                current.push(first);
            }
            contours.push(current);
        }

        contours
    }
}

impl Default for VectorPath {
    fn default() -> Self {
        Self::new()
    }
}

fn is_left(a: Vec2, b: Vec2, p: Vec2) -> f32 {
    (b.x - a.x) * (p.y - a.y) - (p.x - a.x) * (b.y - a.y)
}

fn quadratic_point(p0: Vec2, p1: Vec2, p2: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    mt * mt * p0 + 2.0 * mt * t * p1 + t * t * p2
}

fn cubic_point(p0: Vec2, p1: Vec2, p2: Vec2, p3: Vec2, t: f32) -> Vec2 {
    let mt = 1.0 - t;
    mt * mt * mt * p0 + 3.0 * mt * mt * t * p1 + 3.0 * mt * t * t * p2 + t * t * t * p3
}

fn point_on_segment(a: Vec2, b: Vec2, p: Vec2, eps: f32) -> bool {
    let ab = b - a;
    let ap = p - a;
    let cross = ab.perp_dot(ap).abs();
    if cross > eps {
        return false;
    }
    let dot = ap.dot(ab);
    if dot < -eps {
        return false;
    }
    dot <= ab.length_squared() + eps
}

fn ring_to_linestring(contour: &[Vec2]) -> LineString<f64> {
    let coords: Vec<Coord<f64>> = contour
        .iter()
        .map(|p| Coord {
            x: p.x as f64,
            y: p.y as f64,
        })
        .collect();
    LineString::from(coords)
}

fn signed_area(contour: &[Vec2]) -> f32 {
    if contour.len() < 3 {
        return 0.0;
    }
    let mut area = 0.0f32;
    for edge in contour.windows(2) {
        let a = edge[0];
        let b = edge[1];
        area += a.x * b.y - b.x * a.y;
    }
    0.5 * area
}

fn first_distinct_point(contour: &[Vec2]) -> Option<Vec2> {
    if contour.is_empty() {
        return None;
    }
    let first = contour[0];
    for &p in contour.iter().skip(1) {
        if p != first {
            return Some(p);
        }
    }
    Some(first)
}

fn point_in_ring(point: Vec2, ring: &[Vec2]) -> bool {
    if ring.len() < 3 {
        return false;
    }
    let mut crossings = 0usize;
    for edge in ring.windows(2) {
        let p1 = edge[0];
        let p2 = edge[1];
        if point_on_segment(p1, p2, point, 1e-5) {
            return true;
        }
        if (p1.y > point.y) != (p2.y > point.y) {
            let t = (point.y - p1.y) / (p2.y - p1.y);
            let x = p1.x + t * (p2.x - p1.x);
            if x > point.x {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}

fn angle_between(u: Vec2, v: Vec2) -> f32 {
    let cross = u.x * v.y - u.y * v.x;
    let dot = u.dot(v);
    cross.atan2(dot)
}

#[allow(clippy::too_many_arguments)]
fn arc_to_polyline(
    from: Vec2,
    to: Vec2,
    rx: f32,
    ry: f32,
    x_axis_rotation_deg: f32,
    large_arc: bool,
    sweep: bool,
    max_arc_step_degrees: f32,
) -> Vec<Vec2> {
    if rx <= 0.0 || ry <= 0.0 || (from - to).length_squared() < 1e-8 {
        return vec![to];
    }

    let phi = x_axis_rotation_deg.to_radians();
    let cos_phi = phi.cos();
    let sin_phi = phi.sin();

    let dx2 = (from.x - to.x) * 0.5;
    let dy2 = (from.y - to.y) * 0.5;
    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    let mut rx_adj = rx.abs();
    let mut ry_adj = ry.abs();
    let lambda = (x1p * x1p) / (rx_adj * rx_adj) + (y1p * y1p) / (ry_adj * ry_adj);
    if lambda > 1.0 {
        let s = lambda.sqrt();
        rx_adj *= s;
        ry_adj *= s;
    }

    let rx2 = rx_adj * rx_adj;
    let ry2 = ry_adj * ry_adj;
    let x1p2 = x1p * x1p;
    let y1p2 = y1p * y1p;

    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let num = (rx2 * ry2 - rx2 * y1p2 - ry2 * x1p2).max(0.0);
    let den = (rx2 * y1p2 + ry2 * x1p2).max(1e-12);
    let coef = sign * (num / den).sqrt();

    let cxp = coef * (rx_adj * y1p / ry_adj);
    let cyp = coef * (-ry_adj * x1p / rx_adj);
    let cx = cos_phi * cxp - sin_phi * cyp + (from.x + to.x) * 0.5;
    let cy = sin_phi * cxp + cos_phi * cyp + (from.y + to.y) * 0.5;

    let u = Vec2::new((x1p - cxp) / rx_adj, (y1p - cyp) / ry_adj);
    let v = Vec2::new((-x1p - cxp) / rx_adj, (-y1p - cyp) / ry_adj);

    let theta1 = angle_between(Vec2::new(1.0, 0.0), u);
    let mut delta = angle_between(u, v);
    if sweep && delta < 0.0 {
        delta += std::f32::consts::TAU;
    } else if !sweep && delta > 0.0 {
        delta -= std::f32::consts::TAU;
    }

    let max_step_rad = max_arc_step_degrees.to_radians().max(1e-3);
    let segments = ((delta.abs() / max_step_rad).ceil() as usize).max(1);
    let mut points = Vec::with_capacity(segments);
    for i in 1..=segments {
        let t = i as f32 / segments as f32;
        let theta = theta1 + delta * t;
        let ct = theta.cos();
        let st = theta.sin();
        let x = cx + cos_phi * rx_adj * ct - sin_phi * ry_adj * st;
        let y = cy + sin_phi * rx_adj * ct + cos_phi * ry_adj * st;
        points.push(Vec2::new(x, y));
    }
    points
}

struct SvgPathParser<'a> {
    data: &'a str,
    options: SvgParseOptions,
    tokens: Vec<SvgToken>,
    idx: usize,
}

#[derive(Clone, Copy)]
enum SvgToken {
    Command(char),
    Number(f32),
}

impl<'a> SvgPathParser<'a> {
    fn new(data: &'a str, options: SvgParseOptions) -> Self {
        Self {
            data,
            options,
            tokens: Vec::new(),
            idx: 0,
        }
    }

    fn parse(&mut self) -> Result<VectorPath, VectorPathError> {
        self.tokens = tokenize_svg_path(self.data)?;
        self.idx = 0;

        let mut path = VectorPath::new();
        let mut current = Vec2::ZERO;
        let mut subpath_start = Vec2::ZERO;
        let mut last_cmd = None::<char>;
        let mut last_quad_control = None::<Vec2>;
        let mut last_cubic_control2 = None::<Vec2>;

        while self.idx < self.tokens.len() {
            let cmd = match self.peek_token() {
                Some(SvgToken::Command(c)) => {
                    self.idx += 1;
                    c
                }
                Some(SvgToken::Number(_)) => last_cmd.ok_or_else(|| {
                    VectorPathError::InvalidSvgPathData("number without command".to_string())
                })?,
                None => break,
            };

            match cmd {
                'M' | 'm' => {
                    let abs = cmd == 'M';
                    let first = self.read_point(current, abs)?;
                    path.move_to(first);
                    current = first;
                    subpath_start = first;
                    last_quad_control = None;
                    last_cubic_control2 = None;

                    while self.peek_is_number() {
                        let p = self.read_point(current, abs)?;
                        path.line_to(p);
                        current = p;
                        last_quad_control = None;
                        last_cubic_control2 = None;
                    }
                }
                'L' | 'l' => {
                    let abs = cmd == 'L';
                    while self.peek_is_number() {
                        let p = self.read_point(current, abs)?;
                        path.line_to(p);
                        current = p;
                        last_quad_control = None;
                        last_cubic_control2 = None;
                    }
                }
                'H' | 'h' => {
                    let abs = cmd == 'H';
                    while self.peek_is_number() {
                        let x = self.read_number()?;
                        current = if abs {
                            Vec2::new(x, current.y)
                        } else {
                            Vec2::new(current.x + x, current.y)
                        };
                        path.line_to(current);
                        last_quad_control = None;
                        last_cubic_control2 = None;
                    }
                }
                'V' | 'v' => {
                    let abs = cmd == 'V';
                    while self.peek_is_number() {
                        let y = self.read_number()?;
                        current = if abs {
                            Vec2::new(current.x, y)
                        } else {
                            Vec2::new(current.x, current.y + y)
                        };
                        path.line_to(current);
                        last_quad_control = None;
                        last_cubic_control2 = None;
                    }
                }
                'Q' | 'q' => {
                    let abs = cmd == 'Q';
                    while self.peek_is_number() {
                        let control = self.read_point(current, abs)?;
                        let to = self.read_point(current, abs)?;
                        path.quadratic_to(control, to);
                        current = to;
                        last_quad_control = Some(control);
                        last_cubic_control2 = None;
                    }
                }
                'T' | 't' => {
                    let abs = cmd == 'T';
                    while self.peek_is_number() {
                        let control = match last_quad_control {
                            Some(prev) => current * 2.0 - prev,
                            None => current,
                        };
                        let to = self.read_point(current, abs)?;
                        path.quadratic_to(control, to);
                        current = to;
                        last_quad_control = Some(control);
                        last_cubic_control2 = None;
                    }
                }
                'C' | 'c' => {
                    let abs = cmd == 'C';
                    while self.peek_is_number() {
                        let c1 = self.read_point(current, abs)?;
                        let c2 = self.read_point(current, abs)?;
                        let to = self.read_point(current, abs)?;
                        path.cubic_to(c1, c2, to);
                        current = to;
                        last_cubic_control2 = Some(c2);
                        last_quad_control = None;
                    }
                }
                'S' | 's' => {
                    let abs = cmd == 'S';
                    while self.peek_is_number() {
                        let c1 = match last_cubic_control2 {
                            Some(prev) => current * 2.0 - prev,
                            None => current,
                        };
                        let c2 = self.read_point(current, abs)?;
                        let to = self.read_point(current, abs)?;
                        path.cubic_to(c1, c2, to);
                        current = to;
                        last_cubic_control2 = Some(c2);
                        last_quad_control = None;
                    }
                }
                'A' | 'a' => {
                    let abs = cmd == 'A';
                    while self.peek_is_number() {
                        let rx = self.read_number()?;
                        let ry = self.read_number()?;
                        let rot = self.read_number()?;
                        let large_arc = self.read_number()? != 0.0;
                        let sweep = self.read_number()? != 0.0;
                        let to = self.read_point(current, abs)?;
                        for p in arc_to_polyline(
                            current,
                            to,
                            rx.abs(),
                            ry.abs(),
                            rot,
                            large_arc,
                            sweep,
                            self.options.max_arc_step_degrees.max(1.0),
                        ) {
                            path.line_to(p);
                        }
                        current = to;
                        last_quad_control = None;
                        last_cubic_control2 = None;
                    }
                }
                'Z' | 'z' => {
                    path.close();
                    current = subpath_start;
                    last_quad_control = None;
                    last_cubic_control2 = None;
                }
                other => return Err(VectorPathError::UnsupportedSvgCommand(other)),
            }

            last_cmd = Some(cmd);
        }

        Ok(path)
    }

    fn read_number(&mut self) -> Result<f32, VectorPathError> {
        match self.tokens.get(self.idx).copied() {
            Some(SvgToken::Number(v)) => {
                self.idx += 1;
                Ok(v)
            }
            _ => Err(VectorPathError::InvalidSvgPathData(
                "expected number token".to_string(),
            )),
        }
    }

    fn read_point(&mut self, current: Vec2, absolute: bool) -> Result<Vec2, VectorPathError> {
        let x = self.read_number()?;
        let y = self.read_number()?;
        Ok(if absolute {
            Vec2::new(x, y)
        } else {
            Vec2::new(current.x + x, current.y + y)
        })
    }

    fn peek_token(&self) -> Option<SvgToken> {
        self.tokens.get(self.idx).copied()
    }

    fn peek_is_number(&self) -> bool {
        matches!(self.peek_token(), Some(SvgToken::Number(_)))
    }
}

fn tokenize_svg_path(data: &str) -> Result<Vec<SvgToken>, VectorPathError> {
    let mut tokens = Vec::new();
    let bytes = data.as_bytes();
    let mut i = 0usize;

    while i < bytes.len() {
        let c = bytes[i] as char;
        if c.is_ascii_whitespace() || c == ',' {
            i += 1;
            continue;
        }
        if c.is_ascii_alphabetic() {
            tokens.push(SvgToken::Command(c));
            i += 1;
            continue;
        }

        let start = i;
        if c == '+' || c == '-' {
            i += 1;
        }
        while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
            i += 1;
        }
        if i < bytes.len() && bytes[i] as char == '.' {
            i += 1;
            while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                i += 1;
            }
        }
        if i < bytes.len() {
            let ec = bytes[i] as char;
            if ec == 'e' || ec == 'E' {
                i += 1;
                if i < bytes.len() {
                    let sign = bytes[i] as char;
                    if sign == '+' || sign == '-' {
                        i += 1;
                    }
                }
                while i < bytes.len() && (bytes[i] as char).is_ascii_digit() {
                    i += 1;
                }
            }
        }

        if start == i {
            return Err(VectorPathError::InvalidSvgPathData(format!(
                "invalid numeric token at byte index {start}"
            )));
        }

        let raw = &data[start..i];
        let value = f32::from_str(raw).map_err(|_| {
            VectorPathError::InvalidSvgPathData(format!("invalid numeric value: {raw}"))
        })?;
        tokens.push(SvgToken::Number(value));
    }

    Ok(tokens)
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

    #[test]
    fn test_contains_point_triangle() {
        let mut path = VectorPath::new();
        path.move_to(Vec2::new(0.0, 0.0));
        path.line_to(Vec2::new(100.0, 0.0));
        path.line_to(Vec2::new(50.0, 100.0));
        path.close();

        assert!(path.contains_point(50.0, 25.0));
        assert!(!path.contains_point(90.0, 90.0));
        assert!(
            path.contains_point(50.0, 0.0),
            "edge point should be inside"
        );
        assert!(
            path.contains_point(0.0, 0.0),
            "vertex point should be inside"
        );
    }

    #[test]
    fn test_from_svg_path_data_basic() {
        let path = VectorPath::from_svg_path_data("M 0 0 L 10 0 L 10 10 Z")
            .expect("svg parse should succeed");
        assert_eq!(path.commands.len(), 4);
        assert!(matches!(path.commands[0], PathCommand::MoveTo(_)));
        assert!(matches!(path.commands[1], PathCommand::LineTo(_)));
        assert!(matches!(path.commands[2], PathCommand::LineTo(_)));
        assert!(matches!(path.commands[3], PathCommand::Close));
    }

    #[test]
    fn test_from_svg_path_data_relative() {
        let path =
            VectorPath::from_svg_path_data("m 10 10 l 20 0 l 0 20 z").expect("parse should work");
        assert_eq!(path.commands.len(), 4);
        match path.commands[1] {
            PathCommand::LineTo(p) => assert_eq!(p, Vec2::new(30.0, 10.0)),
            _ => panic!("expected line"),
        }
    }

    #[test]
    fn test_boolean_union_rectangles() {
        let a = VectorPath::from_svg_path_data("M 0 0 L 10 0 L 10 10 L 0 10 Z").expect("parse a");
        let b = VectorPath::from_svg_path_data("M 5 0 L 15 0 L 15 10 L 5 10 Z").expect("parse b");
        let out = VectorPath::boolean_op(&a, &b, BooleanOp::Union).expect("union should work");

        assert!(!out.commands.is_empty());
        assert!(out.contains_point(2.0, 5.0));
        assert!(out.contains_point(12.0, 5.0));
        assert!(!out.contains_point(20.0, 5.0));
    }

    #[test]
    fn test_boolean_subtract_rectangles() {
        let a = VectorPath::from_svg_path_data("M 0 0 L 10 0 L 10 10 L 0 10 Z").expect("parse a");
        let b = VectorPath::from_svg_path_data("M 5 0 L 15 0 L 15 10 L 5 10 Z").expect("parse b");
        let out = VectorPath::boolean_op(&a, &b, BooleanOp::Subtract).expect("subtract works");
        assert!(out.contains_point(2.0, 5.0));
        assert!(!out.contains_point(7.0, 5.0));
    }

    #[test]
    fn test_boolean_intersect_rectangles() {
        let a = VectorPath::from_svg_path_data("M 0 0 L 10 0 L 10 10 L 0 10 Z").expect("parse a");
        let b = VectorPath::from_svg_path_data("M 5 0 L 15 0 L 15 10 L 5 10 Z").expect("parse b");
        let out = VectorPath::boolean_op(&a, &b, BooleanOp::Intersect).expect("intersect works");
        assert!(out.contains_point(7.0, 5.0));
        assert!(!out.contains_point(2.0, 5.0));
        assert!(!out.contains_point(12.0, 5.0));
    }

    #[test]
    fn test_boolean_exclude_rectangles() {
        let a = VectorPath::from_svg_path_data("M 0 0 L 10 0 L 10 10 L 0 10 Z").expect("parse a");
        let b = VectorPath::from_svg_path_data("M 5 0 L 15 0 L 15 10 L 5 10 Z").expect("parse b");
        let out = VectorPath::boolean_op(&a, &b, BooleanOp::Exclude).expect("xor works");
        assert!(out.contains_point(2.0, 5.0));
        assert!(out.contains_point(12.0, 5.0));
        assert!(!out.contains_point(7.0, 5.0));
    }

    #[test]
    fn test_boolean_with_hole_operand() {
        let outer = VectorPath::from_svg_path_data("M 0 0 L 20 0 L 20 20 L 0 20 Z").expect("outer");
        let inner = VectorPath::from_svg_path_data("M 5 5 L 15 5 L 15 15 L 5 15 Z").expect("inner");
        let donut = VectorPath::boolean_op(&outer, &inner, BooleanOp::Subtract).expect("donut");
        let clip = VectorPath::from_svg_path_data("M 10 0 L 20 0 L 20 20 L 10 20 Z").expect("clip");

        let out = VectorPath::boolean_op(&donut, &clip, BooleanOp::Intersect).expect("intersect");
        assert!(out.contains_point(17.0, 10.0));
        assert!(!out.contains_point(12.0, 10.0)); // falls in donut hole
    }

    #[test]
    fn test_boolean_intersect_respects_hole_from_single_path() {
        let mut donut = VectorPath::from_svg_path_data(
            "M 0 0 L 20 0 L 20 20 L 0 20 Z M 5 5 L 15 5 L 15 15 L 5 15 Z",
        )
        .expect("donut path should parse");
        donut.winding_rule = WindingRule::EvenOdd;
        let clip = VectorPath::from_svg_path_data("M 10 0 L 20 0 L 20 20 L 10 20 Z").expect("clip");

        let out = VectorPath::boolean_op(&donut, &clip, BooleanOp::Intersect).expect("intersect");
        assert!(out.contains_point(17.0, 10.0));
        assert!(!out.contains_point(12.0, 10.0));
    }

    #[test]
    fn test_from_svg_path_data_smooth_commands() {
        let path = VectorPath::from_svg_path_data(
            "M 0 0 Q 10 20 20 0 T 40 0 C 45 10 55 10 60 0 S 75 -10 80 0",
        )
        .expect("smooth command parse should succeed");
        assert_eq!(path.commands.len(), 5);
        assert!(matches!(path.commands[1], PathCommand::QuadraticTo { .. }));
        assert!(matches!(path.commands[2], PathCommand::QuadraticTo { .. }));
        assert!(matches!(path.commands[3], PathCommand::CubicTo { .. }));
        assert!(matches!(path.commands[4], PathCommand::CubicTo { .. }));
    }

    #[test]
    fn test_from_svg_path_data_arc_command() {
        let path = VectorPath::from_svg_path_data("M 0 0 A 20 10 0 0 1 40 0 Z")
            .expect("arc parse should succeed");
        assert!(
            path.commands.len() > 4,
            "arc should expand to multiple line segments"
        );
        assert!(matches!(path.commands[0], PathCommand::MoveTo(_)));
        assert!(matches!(
            path.commands[path.commands.len() - 1],
            PathCommand::Close
        ));
    }

    #[test]
    fn test_arc_tolerance_options_change_segment_count() {
        let coarse = VectorPath::from_svg_path_data_with_options(
            "M 0 0 A 20 10 0 0 1 40 0",
            SvgParseOptions {
                max_arc_step_degrees: 45.0,
            },
        )
        .expect("coarse");
        let fine = VectorPath::from_svg_path_data_with_options(
            "M 0 0 A 20 10 0 0 1 40 0",
            SvgParseOptions {
                max_arc_step_degrees: 5.0,
            },
        )
        .expect("fine");

        assert!(fine.commands.len() > coarse.commands.len());
    }
}
