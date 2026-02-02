//! Barrel mesh generation for T-54/55 tank
//!
//! Basic low-poly barrel geometry.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::TAU;

/// Configuration for barrel mesh generation
#[derive(Clone, Debug)]
pub struct BarrelConfig {
    pub length: f32,
    pub base_radius: f32,
    pub muzzle_radius: f32,
    pub rifling_grooves: u32,
    pub rifling_depth: f32,
    pub rifling_twist: f32,
    pub muzzle_brake: bool,
    pub muzzle_brake_length: f32,
    pub muzzle_brake_radius: f32,
    pub muzzle_brake_slots: u32,
    pub bore_evacuator: bool,
    pub bore_evacuator_position: f32,
    pub bore_evacuator_radius: f32,
    pub bore_evacuator_length: f32,
    pub radial_segments: u32,
    pub length_segments: u32,
}

impl Default for BarrelConfig {
    fn default() -> Self {
        Self::d10t()
    }
}

impl BarrelConfig {
    pub fn d10t() -> Self {
        Self {
            length: 2.8,
            base_radius: 0.065,
            muzzle_radius: 0.052,
            rifling_grooves: 32,
            rifling_depth: 0.002,
            rifling_twist: 25.0,
            muzzle_brake: true,
            muzzle_brake_length: 0.22,
            muzzle_brake_radius: 0.075,
            muzzle_brake_slots: 6,
            bore_evacuator: false,
            bore_evacuator_position: 1.2,
            bore_evacuator_radius: 0.10,
            bore_evacuator_length: 0.25,
            radial_segments: 12,  // Low-poly
            length_segments: 4,   // Low-poly
        }
    }

    pub fn d10t2s() -> Self {
        let mut config = Self::d10t();
        config.bore_evacuator = true;
        config
    }
}

/// Generate basic low-poly barrel mesh
pub fn generate_barrel_mesh(config: &BarrelConfig) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    let segments = config.radial_segments;
    let length = config.length;
    let radius = config.base_radius;

    // Simple cylinder for barrel
    for l in 0..=config.length_segments {
        let t = l as f32 / config.length_segments as f32;
        let x = t * length;

        for r in 0..=segments {
            let u = r as f32 / segments as f32;
            let angle = u * TAU;

            let y = angle.cos() * radius;
            let z = angle.sin() * radius;

            positions.push([x, y, z]);

            let normal = Vec3::new(0.0, angle.cos(), angle.sin()).normalize();
            normals.push([normal.x, normal.y, normal.z]);

            uvs.push([t, u]);
        }
    }

    // Generate indices
    let row_width = segments + 1;
    for l in 0..config.length_segments {
        for r in 0..segments {
            let i00 = l * row_width + r;
            let i10 = l * row_width + r + 1;
            let i01 = (l + 1) * row_width + r;
            let i11 = (l + 1) * row_width + r + 1;

            indices.extend_from_slice(&[i00, i01, i10]);
            indices.extend_from_slice(&[i10, i01, i11]);
        }
    }

    // Breech cap (back)
    let breech_center_idx = positions.len() as u32;
    positions.push([0.0, 0.0, 0.0]);
    normals.push([-1.0, 0.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for r in 0..=segments {
        let angle = (r as f32 / segments as f32) * TAU;
        let y = angle.cos() * radius;
        let z = angle.sin() * radius;

        positions.push([0.0, y, z]);
        normals.push([-1.0, 0.0, 0.0]);
        uvs.push([0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5]);
    }

    for r in 0..segments {
        indices.extend_from_slice(&[breech_center_idx, breech_center_idx + r + 2, breech_center_idx + r + 1]);
    }

    // Muzzle cap (front)
    let muzzle_center_idx = positions.len() as u32;
    positions.push([length, 0.0, 0.0]);
    normals.push([1.0, 0.0, 0.0]);
    uvs.push([0.5, 0.5]);

    for r in 0..=segments {
        let angle = (r as f32 / segments as f32) * TAU;
        let y = angle.cos() * radius;
        let z = angle.sin() * radius;

        positions.push([length, y, z]);
        normals.push([1.0, 0.0, 0.0]);
        uvs.push([0.5 + angle.cos() * 0.5, 0.5 + angle.sin() * 0.5]);
    }

    for r in 0..segments {
        indices.extend_from_slice(&[muzzle_center_idx, muzzle_center_idx + r + 1, muzzle_center_idx + r + 2]);
    }

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}
