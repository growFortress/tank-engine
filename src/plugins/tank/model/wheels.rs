//! Wheel mesh generation for T-54/55 tank
//!
//! Basic low-poly wheel geometry.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::TAU;

/// Configuration for road wheel
#[derive(Clone, Debug)]
pub struct RoadWheelConfig {
    pub outer_radius: f32,
    pub hub_radius: f32,
    pub width: f32,
    pub spoke_count: u32,
    pub spoke_width: f32,
    pub rubber_thickness: f32,
    pub rim_segments: u32,
}

impl Default for RoadWheelConfig {
    fn default() -> Self {
        Self::t54()
    }
}

impl RoadWheelConfig {
    pub fn t54() -> Self {
        Self {
            outer_radius: 0.28,
            hub_radius: 0.08,
            width: 0.20,
            spoke_count: 5,
            spoke_width: 0.06,
            rubber_thickness: 0.025,
            rim_segments: 12,  // Low-poly
        }
    }
}

/// Configuration for drive sprocket
#[derive(Clone, Debug)]
pub struct SprocketConfig {
    pub radius: f32,
    pub hub_radius: f32,
    pub width: f32,
    pub tooth_count: u32,
    pub tooth_height: f32,
    pub tooth_width: f32,
}

impl Default for SprocketConfig {
    fn default() -> Self {
        Self::t54()
    }
}

impl SprocketConfig {
    pub fn t54() -> Self {
        Self {
            radius: 0.32,
            hub_radius: 0.12,
            width: 0.24,
            tooth_count: 13,
            tooth_height: 0.04,
            tooth_width: 0.05,
        }
    }
}

/// Configuration for idler wheel
#[derive(Clone, Debug)]
pub struct IdlerConfig {
    pub radius: f32,
    pub hub_radius: f32,
    pub width: f32,
    pub rim_segments: u32,
}

impl Default for IdlerConfig {
    fn default() -> Self {
        Self::t54()
    }
}

impl IdlerConfig {
    pub fn t54() -> Self {
        Self {
            radius: 0.30,
            hub_radius: 0.10,
            width: 0.22,
            rim_segments: 12,  // Low-poly
        }
    }
}

/// Generate simple cylinder wheel
fn generate_simple_cylinder(radius: f32, width: f32, segments: u32) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let half_width = width * 0.5;

    // Outer surface
    for i in 0..=segments {
        let u = i as f32 / segments as f32;
        let angle = u * TAU;
        let x = angle.cos() * radius;
        let y = angle.sin() * radius;

        positions.push([x, y, -half_width]);
        positions.push([x, y, half_width]);

        let normal = Vec3::new(angle.cos(), angle.sin(), 0.0);
        normals.push([normal.x, normal.y, normal.z]);
        normals.push([normal.x, normal.y, normal.z]);

        uvs.push([u, 0.0]);
        uvs.push([u, 1.0]);
    }

    for i in 0..segments {
        let idx = i * 2;
        indices.extend_from_slice(&[idx, idx + 1, idx + 3]);
        indices.extend_from_slice(&[idx, idx + 3, idx + 2]);
    }

    // Front cap
    let front_center = positions.len() as u32;
    positions.push([0.0, 0.0, half_width]);
    normals.push([0.0, 0.0, 1.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * TAU;
        positions.push([angle.cos() * radius, angle.sin() * radius, half_width]);
        normals.push([0.0, 0.0, 1.0]);
        uvs.push([0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5]);
    }

    for i in 0..segments {
        indices.extend_from_slice(&[front_center, front_center + i + 1, front_center + i + 2]);
    }

    // Back cap
    let back_center = positions.len() as u32;
    positions.push([0.0, 0.0, -half_width]);
    normals.push([0.0, 0.0, -1.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * TAU;
        positions.push([angle.cos() * radius, angle.sin() * radius, -half_width]);
        normals.push([0.0, 0.0, -1.0]);
        uvs.push([0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5]);
    }

    for i in 0..segments {
        indices.extend_from_slice(&[back_center, back_center + i + 2, back_center + i + 1]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Generate road wheel mesh - simple cylinder
pub fn generate_road_wheel(config: &RoadWheelConfig) -> Mesh {
    generate_simple_cylinder(config.outer_radius, config.width, config.rim_segments)
}

/// Generate drive sprocket - simple cylinder
pub fn generate_drive_sprocket(config: &SprocketConfig) -> Mesh {
    generate_simple_cylinder(config.radius, config.width, 12)
}

/// Generate idler wheel - simple cylinder
pub fn generate_idler_wheel(config: &IdlerConfig) -> Mesh {
    generate_simple_cylinder(config.radius, config.width, config.rim_segments)
}

/// Generate return roller - simple cylinder
pub fn generate_return_roller(radius: f32, width: f32) -> Mesh {
    generate_simple_cylinder(radius, width, 8)
}
