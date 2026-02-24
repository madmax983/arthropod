use flux_state::{Computed, ReadSignal, Runtime};
use render_engine::Vec2;
use std::sync::Arc;

/// A reactive 1D Perlin noise signal.
///
/// Wraps an input signal (typically time or position) and outputs a smooth, continuous noise value
/// in the range [-1.0, 1.0].
#[derive(Clone)]
pub struct NoiseSignal {
    computed: Computed<f32>,
}

impl NoiseSignal {
    /// Create a new 1D noise signal driven by the given input signal.
    ///
    /// The input signal is typically a time value or a spatial coordinate.
    /// The output will be continuous and smooth.
    pub fn new(runtime: Arc<Runtime>, input: ReadSignal<f32>) -> Self {
        let computed = Computed::new(runtime, move || {
            let t = input.get();
            perlin_1d(t)
        });
        Self { computed }
    }

    /// Get the current noise value.
    pub fn get(&self) -> f32 {
        self.computed.get()
    }

    /// Get the underlying computed signal.
    pub fn signal(&self) -> &Computed<f32> {
        &self.computed
    }
}

/// A reactive 2D Perlin noise signal.
///
/// Wraps an input signal (typically a 2D position) and outputs a smooth, continuous noise value
/// in the range [-1.0, 1.0].
#[derive(Clone)]
pub struct NoiseSignal2D {
    computed: Computed<f32>,
}

impl NoiseSignal2D {
    /// Create a new 2D noise signal driven by the given input signal.
    pub fn new(runtime: Arc<Runtime>, input: ReadSignal<Vec2>) -> Self {
        let computed = Computed::new(runtime, move || {
            let pos = input.get();
            perlin_2d(pos.x, pos.y)
        });
        Self { computed }
    }

    /// Get the current noise value.
    pub fn get(&self) -> f32 {
        self.computed.get()
    }

    /// Get the underlying computed signal.
    pub fn signal(&self) -> &Computed<f32> {
        &self.computed
    }
}

// --- Perlin Noise Implementation (Zero Dependencies) ---

// Permutation table. This is a standard shuffle of 0..255.
// Duplicated to avoid wrapping indices.
const P: [u8; 512] = [
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180, // Duplicate 0..255
    151, 160, 137, 91, 90, 15, 131, 13, 201, 95, 96, 53, 194, 233, 7, 225, 140, 36, 103, 30, 69,
    142, 8, 99, 37, 240, 21, 10, 23, 190, 6, 148, 247, 120, 234, 75, 0, 26, 197, 62, 94, 252, 219,
    203, 117, 35, 11, 32, 57, 177, 33, 88, 237, 149, 56, 87, 174, 20, 125, 136, 171, 168, 68, 175,
    74, 165, 71, 134, 139, 48, 27, 166, 77, 146, 158, 231, 83, 111, 229, 122, 60, 211, 133, 230,
    220, 105, 92, 41, 55, 46, 245, 40, 244, 102, 143, 54, 65, 25, 63, 161, 1, 216, 80, 73, 209, 76,
    132, 187, 208, 89, 18, 169, 200, 196, 135, 130, 116, 188, 159, 86, 164, 100, 109, 198, 173,
    186, 3, 64, 52, 217, 226, 250, 124, 123, 5, 202, 38, 147, 118, 126, 255, 82, 85, 212, 207, 206,
    59, 227, 47, 16, 58, 17, 182, 189, 28, 42, 223, 183, 170, 213, 119, 248, 152, 2, 44, 154, 163,
    70, 221, 153, 101, 155, 167, 43, 172, 9, 129, 22, 39, 253, 19, 98, 108, 110, 79, 113, 224, 232,
    178, 185, 112, 104, 218, 246, 97, 228, 251, 34, 242, 193, 238, 210, 144, 12, 191, 179, 162,
    241, 81, 51, 145, 235, 249, 14, 239, 107, 49, 192, 214, 31, 181, 199, 106, 157, 184, 84, 204,
    176, 115, 121, 50, 45, 127, 4, 150, 254, 138, 236, 205, 93, 222, 114, 67, 29, 24, 72, 243, 141,
    128, 195, 78, 66, 215, 61, 156, 180,
];

/// Smooth step function (6t^5 - 15t^4 + 10t^3)
fn fade(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Linear interpolation
fn lerp(t: f32, a: f32, b: f32) -> f32 {
    a + t * (b - a)
}

/// Gradient function for 1D/2D Perlin noise
fn grad(hash: u8, x: f32, y: f32) -> f32 {
    let h = hash & 15;
    let u = if h < 8 { x } else { y };
    let v = if h < 4 {
        y
    } else if h == 12 || h == 14 {
        x
    } else {
        0.0 // For 1D/2D we can simplify Z to 0
    };
    (if (h & 1) == 0 { u } else { -u }) + (if (h & 2) == 0 { v } else { -v })
}

/// Compute 1D Perlin noise for coordinate x.
pub fn perlin_1d(x: f32) -> f32 {
    // 1D Perlin is just 2D with y=0
    perlin_2d(x, 0.0)
}

/// Compute 2D Perlin noise for coordinates (x, y).
pub fn perlin_2d(x: f32, y: f32) -> f32 {
    let xi = x.floor() as i32 & 255;
    let yi = y.floor() as i32 & 255;

    let x = x - x.floor();
    let y = y - y.floor();

    let u = fade(x);
    let v = fade(y);

    let aa = P[xi as usize] as usize + yi as usize;
    let bb = P[(xi + 1) as usize] as usize + yi as usize;

    lerp(
        v,
        lerp(u, grad(P[aa], x, y), grad(P[bb], x - 1.0, y)),
        lerp(
            u,
            grad(P[aa + 1], x, y - 1.0),
            grad(P[bb + 1], x - 1.0, y - 1.0),
        ),
    )
}
