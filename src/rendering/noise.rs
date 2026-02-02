//! Noise functions for procedural texture generation
//!
//! Includes Perlin, FBM, Worley, and Ridged noise implementations.

use std::f32::consts::PI;

/// Simple hash function for noise generation
#[inline]
fn hash(x: i32, y: i32, seed: u32) -> u32 {
    let mut h = seed;
    h ^= x as u32;
    h = h.wrapping_mul(0x85ebca6b);
    h ^= y as u32;
    h = h.wrapping_mul(0xc2b2ae35);
    h ^= h >> 16;
    h
}

/// Convert hash to float in range [0, 1]
#[inline]
fn hash_to_float(h: u32) -> f32 {
    (h & 0xFFFFFF) as f32 / 0xFFFFFF as f32
}

/// Gradient vectors for Perlin noise
const GRADIENTS: [[f32; 2]; 8] = [
    [1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0],
    [0.707, 0.707], [-0.707, 0.707], [0.707, -0.707], [-0.707, -0.707],
];

/// Get gradient vector from hash
#[inline]
fn gradient(h: u32) -> [f32; 2] {
    GRADIENTS[(h & 7) as usize]
}

/// Smoothstep interpolation
#[inline]
fn smoothstep(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Quintic interpolation (Ken Perlin's improved version)
#[inline]
fn quintic(t: f32) -> f32 {
    t * t * t * (t * (t * 6.0 - 15.0) + 10.0)
}

/// Linear interpolation
#[inline]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + t * (b - a)
}

/// 2D Perlin noise
///
/// Returns value in range [-1, 1]
pub fn perlin_2d(x: f32, y: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let dx = x - x0 as f32;
    let dy = y - y0 as f32;

    let u = quintic(dx);
    let v = quintic(dy);

    // Get gradients at corners
    let g00 = gradient(hash(x0, y0, seed));
    let g10 = gradient(hash(x1, y0, seed));
    let g01 = gradient(hash(x0, y1, seed));
    let g11 = gradient(hash(x1, y1, seed));

    // Compute dot products
    let n00 = g00[0] * dx + g00[1] * dy;
    let n10 = g10[0] * (dx - 1.0) + g10[1] * dy;
    let n01 = g01[0] * dx + g01[1] * (dy - 1.0);
    let n11 = g11[0] * (dx - 1.0) + g11[1] * (dy - 1.0);

    // Interpolate
    let nx0 = lerp(n00, n10, u);
    let nx1 = lerp(n01, n11, u);
    lerp(nx0, nx1, v)
}

/// 2D Perlin noise normalized to [0, 1]
pub fn perlin_2d_01(x: f32, y: f32, seed: u32) -> f32 {
    (perlin_2d(x, y, seed) + 1.0) * 0.5
}

/// Fractal Brownian Motion (FBM) - layered Perlin noise
///
/// Combines multiple octaves of noise for more natural patterns.
///
/// * `octaves` - Number of noise layers (typically 4-8)
/// * `lacunarity` - Frequency multiplier per octave (typically 2.0)
/// * `gain` - Amplitude multiplier per octave (typically 0.5)
pub fn fbm_2d(x: f32, y: f32, octaves: u32, lacunarity: f32, gain: f32, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for i in 0..octaves {
        value += amplitude * perlin_2d(x * frequency, y * frequency, seed.wrapping_add(i));
        max_value += amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    value / max_value
}

/// FBM normalized to [0, 1]
pub fn fbm_2d_01(x: f32, y: f32, octaves: u32, lacunarity: f32, gain: f32, seed: u32) -> f32 {
    (fbm_2d(x, y, octaves, lacunarity, gain, seed) + 1.0) * 0.5
}

/// Worley (cellular) noise - distance to nearest random point
///
/// Creates cell-like patterns, good for rust and weathering effects.
/// Returns value in range [0, 1]
pub fn worley_2d(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;

    let mut min_dist = f32::MAX;

    // Check 3x3 neighborhood
    for dx in -1..=1 {
        for dy in -1..=1 {
            let cx = xi + dx;
            let cy = yi + dy;

            // Random point in this cell
            let h1 = hash(cx, cy, seed);
            let h2 = hash(cx, cy, seed.wrapping_add(1));
            let px = cx as f32 + hash_to_float(h1);
            let py = cy as f32 + hash_to_float(h2);

            let dist = ((x - px).powi(2) + (y - py).powi(2)).sqrt();
            min_dist = min_dist.min(dist);
        }
    }

    // Clamp to [0, 1]
    min_dist.min(1.0)
}

/// Worley noise returning distance to second nearest point
///
/// Creates different patterns useful for cracks and veins.
pub fn worley_2d_f2(x: f32, y: f32, seed: u32) -> f32 {
    let xi = x.floor() as i32;
    let yi = y.floor() as i32;

    let mut min_dist1 = f32::MAX;
    let mut min_dist2 = f32::MAX;

    for dx in -1..=1 {
        for dy in -1..=1 {
            let cx = xi + dx;
            let cy = yi + dy;

            let h1 = hash(cx, cy, seed);
            let h2 = hash(cx, cy, seed.wrapping_add(1));
            let px = cx as f32 + hash_to_float(h1);
            let py = cy as f32 + hash_to_float(h2);

            let dist = ((x - px).powi(2) + (y - py).powi(2)).sqrt();

            if dist < min_dist1 {
                min_dist2 = min_dist1;
                min_dist1 = dist;
            } else if dist < min_dist2 {
                min_dist2 = dist;
            }
        }
    }

    min_dist2.min(1.5) / 1.5
}

/// Ridged noise - creates sharp ridges
///
/// Good for scratches and directional damage patterns.
pub fn ridged_2d(x: f32, y: f32, octaves: u32, lacunarity: f32, gain: f32, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut weight = 1.0;

    for i in 0..octaves {
        let signal = 1.0 - perlin_2d(x * frequency, y * frequency, seed.wrapping_add(i)).abs();
        let signal = signal * signal * weight;
        weight = (signal * 2.0).clamp(0.0, 1.0);
        value += signal * amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    value
}

/// Turbulence noise - absolute value of FBM
///
/// Creates billowy, cloud-like patterns.
pub fn turbulence_2d(x: f32, y: f32, octaves: u32, lacunarity: f32, gain: f32, seed: u32) -> f32 {
    let mut value = 0.0;
    let mut amplitude = 1.0;
    let mut frequency = 1.0;
    let mut max_value = 0.0;

    for i in 0..octaves {
        value += amplitude * perlin_2d(x * frequency, y * frequency, seed.wrapping_add(i)).abs();
        max_value += amplitude;
        amplitude *= gain;
        frequency *= lacunarity;
    }

    value / max_value
}

/// Directional scratch pattern
///
/// Creates elongated noise patterns along a direction, useful for scratches.
/// * `angle` - Direction of scratches in radians
/// * `stretch` - How elongated the scratches are (higher = more stretched)
pub fn scratch_noise(x: f32, y: f32, angle: f32, stretch: f32, seed: u32) -> f32 {
    let cos_a = angle.cos();
    let sin_a = angle.sin();

    // Rotate and stretch coordinates
    let rx = (x * cos_a + y * sin_a) * stretch;
    let ry = -x * sin_a + y * cos_a;

    ridged_2d(rx, ry, 4, 2.0, 0.5, seed)
}

/// Edge-aware noise for rust patterns
///
/// Combines multiple noise types to create realistic rust accumulation
/// that tends to appear at edges and in recesses.
pub fn rust_pattern(x: f32, y: f32, edge_factor: f32, seed: u32) -> f32 {
    // Base Worley for cellular rust spots
    let worley = worley_2d(x * 3.0, y * 3.0, seed);

    // FBM for variation
    let fbm = fbm_2d_01(x * 5.0, y * 5.0, 4, 2.0, 0.5, seed.wrapping_add(100));

    // Combine with edge factor
    let rust = (1.0 - worley) * fbm * (1.0 + edge_factor * 2.0);

    rust.clamp(0.0, 1.0)
}

/// Dent/impact pattern
///
/// Creates circular impact marks useful for battle damage.
pub fn dent_pattern(x: f32, y: f32, cx: f32, cy: f32, radius: f32, depth: f32) -> f32 {
    let dist = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt();

    if dist < radius {
        let t = dist / radius;
        // Smooth bowl shape
        depth * (1.0 - smoothstep(t))
    } else {
        0.0
    }
}

/// Value noise (simpler, blockier than Perlin)
///
/// Useful for low-frequency variation.
pub fn value_noise_2d(x: f32, y: f32, seed: u32) -> f32 {
    let x0 = x.floor() as i32;
    let y0 = y.floor() as i32;
    let x1 = x0 + 1;
    let y1 = y0 + 1;

    let dx = smoothstep(x - x0 as f32);
    let dy = smoothstep(y - y0 as f32);

    let v00 = hash_to_float(hash(x0, y0, seed)) * 2.0 - 1.0;
    let v10 = hash_to_float(hash(x1, y0, seed)) * 2.0 - 1.0;
    let v01 = hash_to_float(hash(x0, y1, seed)) * 2.0 - 1.0;
    let v11 = hash_to_float(hash(x1, y1, seed)) * 2.0 - 1.0;

    let nx0 = lerp(v00, v10, dx);
    let nx1 = lerp(v01, v11, dx);
    lerp(nx0, nx1, dy)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_range() {
        for i in 0..100 {
            let x = i as f32 * 0.1;
            let y = i as f32 * 0.13;
            let v = perlin_2d(x, y, 42);
            assert!(v >= -1.0 && v <= 1.0, "Perlin out of range: {}", v);
        }
    }

    #[test]
    fn test_worley_range() {
        for i in 0..100 {
            let x = i as f32 * 0.1;
            let y = i as f32 * 0.13;
            let v = worley_2d(x, y, 42);
            assert!(v >= 0.0 && v <= 1.0, "Worley out of range: {}", v);
        }
    }
}
