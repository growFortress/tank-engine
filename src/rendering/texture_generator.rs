//! Procedural texture generation utilities
//!
//! Generates PBR texture maps (base color, normal, metallic-roughness) for tank materials.

use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use super::noise::*;

/// Configuration for texture generation
#[derive(Clone, Debug)]
pub struct TextureConfig {
    /// Texture resolution (width = height)
    pub resolution: u32,
    /// Random seed for noise
    pub seed: u32,
    /// UV scale multiplier
    pub uv_scale: f32,
}

impl Default for TextureConfig {
    fn default() -> Self {
        Self {
            resolution: 1024,
            seed: 42,
            uv_scale: 1.0,
        }
    }
}

/// RGBA color
#[derive(Clone, Copy, Debug)]
pub struct Color4 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color4 {
    pub fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub fn from_float(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self {
            r: (r.clamp(0.0, 1.0) * 255.0) as u8,
            g: (g.clamp(0.0, 1.0) * 255.0) as u8,
            b: (b.clamp(0.0, 1.0) * 255.0) as u8,
            a: (a.clamp(0.0, 1.0) * 255.0) as u8,
        }
    }

    pub fn lerp(a: Self, b: Self, t: f32) -> Self {
        let t = t.clamp(0.0, 1.0);
        Self {
            r: (a.r as f32 + (b.r as f32 - a.r as f32) * t) as u8,
            g: (a.g as f32 + (b.g as f32 - a.g as f32) * t) as u8,
            b: (a.b as f32 + (b.b as f32 - a.b as f32) * t) as u8,
            a: (a.a as f32 + (b.a as f32 - a.a as f32) * t) as u8,
        }
    }
}

/// Texture buffer for procedural generation
pub struct TextureBuffer {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

impl TextureBuffer {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            data: vec![0u8; (width * height * 4) as usize],
        }
    }

    pub fn set_pixel(&mut self, x: u32, y: u32, color: Color4) {
        if x >= self.width || y >= self.height {
            return;
        }
        let idx = ((y * self.width + x) * 4) as usize;
        self.data[idx] = color.r;
        self.data[idx + 1] = color.g;
        self.data[idx + 2] = color.b;
        self.data[idx + 3] = color.a;
    }

    pub fn get_pixel(&self, x: u32, y: u32) -> Color4 {
        let idx = ((y * self.width + x) * 4) as usize;
        Color4 {
            r: self.data[idx],
            g: self.data[idx + 1],
            b: self.data[idx + 2],
            a: self.data[idx + 3],
        }
    }

    /// Convert to Bevy Image
    pub fn to_image(&self) -> Image {
        Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            self.data.clone(),
            TextureFormat::Rgba8UnormSrgb,
            default(),
        )
    }

    /// Convert to Bevy Image (linear color space, for normal maps)
    pub fn to_image_linear(&self) -> Image {
        Image::new(
            Extent3d {
                width: self.width,
                height: self.height,
                depth_or_array_layers: 1,
            },
            TextureDimension::D2,
            self.data.clone(),
            TextureFormat::Rgba8Unorm,
            default(),
        )
    }
}

/// Generate a normal map from a height map
pub fn height_to_normal_map(height_map: &[f32], width: u32, height: u32, strength: f32) -> TextureBuffer {
    let mut buffer = TextureBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let idx = (y * width + x) as usize;

            // Sample neighboring heights
            let left = if x > 0 { height_map[idx - 1] } else { height_map[idx] };
            let right = if x < width - 1 { height_map[idx + 1] } else { height_map[idx] };
            let up = if y > 0 { height_map[idx - width as usize] } else { height_map[idx] };
            let down = if y < height - 1 { height_map[idx + width as usize] } else { height_map[idx] };

            // Calculate normal from height differences
            let dx = (left - right) * strength;
            let dy = (up - down) * strength;
            let dz = 1.0;

            // Normalize
            let len = (dx * dx + dy * dy + dz * dz).sqrt();
            let nx = dx / len;
            let ny = dy / len;
            let nz = dz / len;

            // Convert from [-1,1] to [0,255]
            buffer.set_pixel(x, y, Color4 {
                r: ((nx * 0.5 + 0.5) * 255.0) as u8,
                g: ((ny * 0.5 + 0.5) * 255.0) as u8,
                b: ((nz * 0.5 + 0.5) * 255.0) as u8,
                a: 255,
            });
        }
    }

    buffer
}

/// Generate a height map using FBM
pub fn generate_height_map(
    width: u32,
    height: u32,
    scale: f32,
    octaves: u32,
    seed: u32,
) -> Vec<f32> {
    let mut heights = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32 * scale;
            let v = y as f32 / height as f32 * scale;

            let h = fbm_2d_01(u, v, octaves, 2.0, 0.5, seed);
            heights.push(h);
        }
    }

    heights
}

/// Generate scratches pattern
pub fn generate_scratch_map(
    width: u32,
    height: u32,
    scale: f32,
    angle: f32,
    stretch: f32,
    seed: u32,
) -> Vec<f32> {
    let mut scratches = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32 * scale;
            let v = y as f32 / height as f32 * scale;

            let s = scratch_noise(u, v, angle, stretch, seed);
            scratches.push(s);
        }
    }

    scratches
}

/// Generate rust pattern map
pub fn generate_rust_map(
    width: u32,
    height: u32,
    scale: f32,
    edge_map: Option<&[f32]>,
    seed: u32,
) -> Vec<f32> {
    let mut rust = Vec::with_capacity((width * height) as usize);

    for y in 0..height {
        for x in 0..width {
            let u = x as f32 / width as f32 * scale;
            let v = y as f32 / height as f32 * scale;

            let edge_factor = edge_map.map(|e| e[(y * width + x) as usize]).unwrap_or(0.0);
            let r = rust_pattern(u, v, edge_factor, seed);
            rust.push(r);
        }
    }

    rust
}

/// Apply color grading/variation to a base color texture
pub fn apply_color_variation(
    buffer: &mut TextureBuffer,
    variation_scale: f32,
    seed: u32,
) {
    for y in 0..buffer.height {
        for x in 0..buffer.width {
            let u = x as f32 / buffer.width as f32 * 8.0;
            let v = y as f32 / buffer.height as f32 * 8.0;

            let variation = fbm_2d(u, v, 3, 2.0, 0.5, seed) * variation_scale;

            let pixel = buffer.get_pixel(x, y);
            let r = (pixel.r as f32 / 255.0 + variation).clamp(0.0, 1.0);
            let g = (pixel.g as f32 / 255.0 + variation * 0.8).clamp(0.0, 1.0);
            let b = (pixel.b as f32 / 255.0 + variation * 0.6).clamp(0.0, 1.0);

            buffer.set_pixel(x, y, Color4::from_float(r, g, b, 1.0));
        }
    }
}

/// PBR texture set
pub struct PbrTextureSet {
    pub base_color: Image,
    pub normal: Image,
    pub metallic_roughness: Image,
}

/// Configuration for armor texture generation
#[derive(Clone, Debug)]
pub struct ArmorTextureConfig {
    pub resolution: u32,
    pub seed: u32,
    /// Base color (olive green)
    pub base_color: [f32; 3],
    /// Rust color
    pub rust_color: [f32; 3],
    /// Amount of rust (0.0-1.0)
    pub rust_amount: f32,
    /// Amount of scratches (0.0-1.0)
    pub scratch_amount: f32,
    /// Base metallic value
    pub metallic: f32,
    /// Base roughness value
    pub roughness: f32,
    /// Normal map strength
    pub normal_strength: f32,
}

impl Default for ArmorTextureConfig {
    fn default() -> Self {
        Self {
            resolution: 1024,
            seed: 42,
            base_color: [0.25, 0.28, 0.18],  // Olive green
            rust_color: [0.45, 0.25, 0.12],   // Rust brown
            rust_amount: 0.15,
            scratch_amount: 0.3,
            metallic: 0.85,
            roughness: 0.45,
            normal_strength: 1.0,
        }
    }
}

/// Generate a complete PBR texture set for tank armor
pub fn generate_armor_textures(config: &ArmorTextureConfig) -> PbrTextureSet {
    let res = config.resolution;

    // Generate noise maps
    let height_map = generate_height_map(res, res, 16.0, 5, config.seed);
    let rust_map = generate_rust_map(res, res, 8.0, None, config.seed.wrapping_add(100));
    let scratch_map = generate_scratch_map(res, res, 20.0, 0.3, 8.0, config.seed.wrapping_add(200));

    // Base color texture
    let mut base_color_buf = TextureBuffer::new(res, res);
    for y in 0..res {
        for x in 0..res {
            let idx = (y * res + x) as usize;

            let rust = rust_map[idx] * config.rust_amount;
            let scratch = scratch_map[idx].powf(2.0) * config.scratch_amount;

            // Mix base color with rust
            let mut r = config.base_color[0];
            let mut g = config.base_color[1];
            let mut b = config.base_color[2];

            // Apply rust
            r = r * (1.0 - rust) + config.rust_color[0] * rust;
            g = g * (1.0 - rust) + config.rust_color[1] * rust;
            b = b * (1.0 - rust) + config.rust_color[2] * rust;

            // Scratches reveal lighter metal underneath
            let scratch_color = 0.5;
            r = r * (1.0 - scratch) + scratch_color * scratch;
            g = g * (1.0 - scratch) + scratch_color * scratch;
            b = b * (1.0 - scratch) + scratch_color * scratch;

            base_color_buf.set_pixel(x, y, Color4::from_float(r, g, b, 1.0));
        }
    }

    // Apply subtle color variation
    apply_color_variation(&mut base_color_buf, 0.03, config.seed.wrapping_add(300));

    // Normal map
    let mut combined_height = height_map.clone();
    for i in 0..combined_height.len() {
        combined_height[i] += scratch_map[i] * 0.3;
    }
    let normal_buf = height_to_normal_map(&combined_height, res, res, config.normal_strength);

    // Metallic-roughness map (R=unused, G=roughness, B=metallic, A=unused)
    let mut mr_buf = TextureBuffer::new(res, res);
    for y in 0..res {
        for x in 0..res {
            let idx = (y * res + x) as usize;

            let rust = rust_map[idx] * config.rust_amount;
            let scratch = scratch_map[idx].powf(2.0) * config.scratch_amount;

            // Rust increases roughness, decreases metallic
            let roughness = (config.roughness + rust * 0.4 - scratch * 0.1).clamp(0.0, 1.0);
            let metallic = (config.metallic - rust * 0.3 + scratch * 0.1).clamp(0.0, 1.0);

            mr_buf.set_pixel(x, y, Color4::from_float(0.0, roughness, metallic, 1.0));
        }
    }

    PbrTextureSet {
        base_color: base_color_buf.to_image(),
        normal: normal_buf.to_image_linear(),
        metallic_roughness: mr_buf.to_image_linear(),
    }
}

/// Configuration for track texture generation
#[derive(Clone, Debug)]
pub struct TrackTextureConfig {
    pub resolution: u32,
    pub seed: u32,
    /// Rubber color
    pub rubber_color: [f32; 3],
    /// Metal color
    pub metal_color: [f32; 3],
    /// Dirt/mud amount
    pub dirt_amount: f32,
}

impl Default for TrackTextureConfig {
    fn default() -> Self {
        Self {
            resolution: 512,
            seed: 42,
            rubber_color: [0.08, 0.08, 0.08],
            metal_color: [0.35, 0.33, 0.30],
            dirt_amount: 0.25,
        }
    }
}

/// Generate track textures
pub fn generate_track_textures(config: &TrackTextureConfig) -> PbrTextureSet {
    let res = config.resolution;

    let height_map = generate_height_map(res, res, 32.0, 4, config.seed);

    let mut base_color_buf = TextureBuffer::new(res, res);
    let mut mr_buf = TextureBuffer::new(res, res);

    for y in 0..res {
        for x in 0..res {
            let idx = (y * res + x) as usize;
            let u = x as f32 / res as f32;
            let v = y as f32 / res as f32;

            // Alternating rubber and metal pattern
            let metal_mask = ((u * 4.0).floor() as i32 % 2 == 0) as i32 as f32;

            // Add dirt variation
            let dirt = fbm_2d_01(u * 10.0, v * 10.0, 3, 2.0, 0.5, config.seed) * config.dirt_amount;
            let dirt_color = [0.25, 0.2, 0.15];

            let base = if metal_mask > 0.5 {
                config.metal_color
            } else {
                config.rubber_color
            };

            let r = base[0] * (1.0 - dirt) + dirt_color[0] * dirt;
            let g = base[1] * (1.0 - dirt) + dirt_color[1] * dirt;
            let b = base[2] * (1.0 - dirt) + dirt_color[2] * dirt;

            base_color_buf.set_pixel(x, y, Color4::from_float(r, g, b, 1.0));

            // Metallic-roughness
            let metallic = if metal_mask > 0.5 { 0.9 } else { 0.0 };
            let roughness = if metal_mask > 0.5 { 0.4 + dirt * 0.3 } else { 0.9 };

            mr_buf.set_pixel(x, y, Color4::from_float(0.0, roughness, metallic, 1.0));
        }
    }

    let normal_buf = height_to_normal_map(&height_map, res, res, 0.8);

    PbrTextureSet {
        base_color: base_color_buf.to_image(),
        normal: normal_buf.to_image_linear(),
        metallic_roughness: mr_buf.to_image_linear(),
    }
}

/// Configuration for steel barrel texture
#[derive(Clone, Debug)]
pub struct SteelTextureConfig {
    pub resolution: u32,
    pub seed: u32,
    /// Base steel color
    pub steel_color: [f32; 3],
    /// Heat discoloration amount
    pub heat_discoloration: f32,
}

impl Default for SteelTextureConfig {
    fn default() -> Self {
        Self {
            resolution: 512,
            seed: 42,
            steel_color: [0.4, 0.38, 0.36],
            heat_discoloration: 0.2,
        }
    }
}

/// Generate steel/barrel textures
pub fn generate_steel_textures(config: &SteelTextureConfig) -> PbrTextureSet {
    let res = config.resolution;

    let height_map = generate_height_map(res, res, 64.0, 4, config.seed);

    let mut base_color_buf = TextureBuffer::new(res, res);
    let mut mr_buf = TextureBuffer::new(res, res);

    for y in 0..res {
        for x in 0..res {
            let u = x as f32 / res as f32;
            let v = y as f32 / res as f32;

            // Heat gradient along barrel (more heat at muzzle end)
            let heat = v * config.heat_discoloration;

            // Heat colors: blue -> purple -> brown
            let heat_r = config.steel_color[0] + heat * 0.15;
            let heat_g = config.steel_color[1] - heat * 0.1;
            let heat_b = config.steel_color[2] + heat * 0.2;

            // Add micro variation
            let var = fbm_2d(u * 50.0, v * 50.0, 3, 2.0, 0.5, config.seed) * 0.02;

            let r = (heat_r + var).clamp(0.0, 1.0);
            let g = (heat_g + var).clamp(0.0, 1.0);
            let b = (heat_b + var).clamp(0.0, 1.0);

            base_color_buf.set_pixel(x, y, Color4::from_float(r, g, b, 1.0));

            // High metallic, moderate roughness
            mr_buf.set_pixel(x, y, Color4::from_float(0.0, 0.35 + heat * 0.15, 0.95, 1.0));
        }
    }

    let normal_buf = height_to_normal_map(&height_map, res, res, 0.5);

    PbrTextureSet {
        base_color: base_color_buf.to_image(),
        normal: normal_buf.to_image_linear(),
        metallic_roughness: mr_buf.to_image_linear(),
    }
}
