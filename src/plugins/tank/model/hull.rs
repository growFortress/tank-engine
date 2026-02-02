//! Hull mesh generation for T-54/55 tank
//!
//! Basic low-poly hull geometry.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;

/// Configuration for hull mesh generation
#[derive(Clone, Debug)]
pub struct HullConfig {
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub glacis_angle: f32,
    pub track_width: f32,
}

impl Default for HullConfig {
    fn default() -> Self {
        Self::t54()
    }
}

impl HullConfig {
    pub fn t54() -> Self {
        Self {
            length: 4.0,
            width: 2.3,
            height: 0.65,
            glacis_angle: 60.0,
            track_width: 0.32,
        }
    }
}

/// Generate complete hull mesh - basic low-poly version
pub fn generate_hull_mesh(config: &HullConfig) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let l = config.length;
    let w = config.width;
    let h = config.height;
    let half_w = w * 0.5;
    let track_w = config.track_width;

    // Main hull box
    add_box(
        &mut positions, &mut normals, &mut uvs, &mut indices,
        Vec3::new(0.0, h * 0.5, 0.0),
        Vec3::new(l * 0.85, h, w * 0.65),
    );

    // Left track housing
    add_box(
        &mut positions, &mut normals, &mut uvs, &mut indices,
        Vec3::new(0.0, h * 0.3, -(half_w - track_w * 0.5)),
        Vec3::new(l * 0.95, h * 0.6, track_w),
    );

    // Right track housing
    add_box(
        &mut positions, &mut normals, &mut uvs, &mut indices,
        Vec3::new(0.0, h * 0.3, half_w - track_w * 0.5),
        Vec3::new(l * 0.95, h * 0.6, track_w),
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Add a box
fn add_box(
    positions: &mut Vec<[f32; 3]>,
    normals: &mut Vec<[f32; 3]>,
    uvs: &mut Vec<[f32; 2]>,
    indices: &mut Vec<u32>,
    center: Vec3,
    size: Vec3,
) {
    let half = size * 0.5;

    let faces = [
        ([1.0, 0.0, 0.0], [[half.x, -half.y, -half.z], [half.x, -half.y, half.z], [half.x, half.y, half.z], [half.x, half.y, -half.z]]),
        ([-1.0, 0.0, 0.0], [[-half.x, -half.y, half.z], [-half.x, -half.y, -half.z], [-half.x, half.y, -half.z], [-half.x, half.y, half.z]]),
        ([0.0, 1.0, 0.0], [[-half.x, half.y, -half.z], [half.x, half.y, -half.z], [half.x, half.y, half.z], [-half.x, half.y, half.z]]),
        ([0.0, -1.0, 0.0], [[-half.x, -half.y, half.z], [half.x, -half.y, half.z], [half.x, -half.y, -half.z], [-half.x, -half.y, -half.z]]),
        ([0.0, 0.0, 1.0], [[-half.x, -half.y, half.z], [-half.x, half.y, half.z], [half.x, half.y, half.z], [half.x, -half.y, half.z]]),
        ([0.0, 0.0, -1.0], [[half.x, -half.y, -half.z], [half.x, half.y, -half.z], [-half.x, half.y, -half.z], [-half.x, -half.y, -half.z]]),
    ];

    for (normal, verts) in faces {
        let base = positions.len() as u32;
        for v in verts {
            positions.push([center.x + v[0], center.y + v[1], center.z + v[2]]);
            normals.push(normal);
        }
        uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        indices.extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}
