//! Turret mesh generation for T-54/55 tank
//!
//! Basic low-poly turret geometry.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::TAU;

/// Configuration for turret mesh generation
#[derive(Clone, Debug)]
pub struct TurretConfig {
    pub base_radius: f32,
    pub front_radius: f32,
    pub rear_radius: f32,
    pub height: f32,
    pub front_thickness: f32,
    pub side_thickness: f32,
    pub radial_segments: u32,
    pub height_segments: u32,
    pub cupola_radius: f32,
    pub cupola_height: f32,
    pub hatch_radius: f32,
    pub periscope_count: u32,
}

impl Default for TurretConfig {
    fn default() -> Self {
        Self::t54()
    }
}

impl TurretConfig {
    pub fn t54() -> Self {
        Self {
            base_radius: 1.1,
            front_radius: 0.9,
            rear_radius: 0.85,
            height: 0.55,
            front_thickness: 0.22,
            side_thickness: 0.16,
            radial_segments: 16,  // Low-poly
            height_segments: 4,   // Low-poly
            cupola_radius: 0.22,
            cupola_height: 0.18,
            hatch_radius: 0.20,
            periscope_count: 4,
        }
    }
}

/// Generate basic low-poly turret mesh
pub fn generate_turret_mesh(config: &TurretConfig) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let segments = config.radial_segments;
    let radius = config.base_radius;
    let height = config.height;

    // Simple cylinder for turret body
    for h in 0..=config.height_segments {
        let v = h as f32 / config.height_segments as f32;
        let y = v * height;

        for i in 0..=segments {
            let u = i as f32 / segments as f32;
            let angle = u * TAU;

            let x = angle.sin() * radius;
            let z = angle.cos() * radius;

            positions.push([x, y, z]);

            let normal = Vec3::new(angle.sin(), 0.0, angle.cos()).normalize();
            normals.push([normal.x, normal.y, normal.z]);

            uvs.push([u, v]);
        }
    }

    // Generate indices for cylinder sides
    let row_width = segments + 1;
    for h in 0..config.height_segments {
        for i in 0..segments {
            let i00 = h * row_width + i;
            let i10 = h * row_width + i + 1;
            let i01 = (h + 1) * row_width + i;
            let i11 = (h + 1) * row_width + i + 1;

            indices.extend_from_slice(&[i00, i01, i10]);
            indices.extend_from_slice(&[i10, i01, i11]);
        }
    }

    // Top cap
    let center_idx = positions.len() as u32;
    positions.push([0.0, height, 0.0]);
    normals.push([0.0, 1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    let top_ring_start = config.height_segments * row_width;
    for i in 0..segments {
        let i0 = top_ring_start + i;
        let i1 = top_ring_start + i + 1;
        indices.extend_from_slice(&[center_idx, i0, i1]);
    }

    // Bottom cap
    let bottom_center_idx = positions.len() as u32;
    positions.push([0.0, 0.0, 0.0]);
    normals.push([0.0, -1.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for i in 0..segments {
        let i0 = i;
        let i1 = i + 1;
        indices.extend_from_slice(&[bottom_center_idx, i1, i0]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
