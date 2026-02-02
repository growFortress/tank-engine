//! Track link mesh generation for T-54/55 tank
//!
//! Basic low-poly track geometry.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::PI;

/// Configuration for track link mesh
#[derive(Clone, Debug)]
pub struct TrackLinkConfig {
    pub width: f32,
    pub length: f32,
    pub height: f32,
    pub guide_tooth_height: f32,
    pub guide_tooth_width: f32,
    pub pin_radius: f32,
    pub pad_thickness: f32,
    pub grouser_count: u32,
    pub grouser_height: f32,
}

impl Default for TrackLinkConfig {
    fn default() -> Self {
        Self::t54()
    }
}

impl TrackLinkConfig {
    pub fn t54() -> Self {
        Self {
            width: 0.58,
            length: 0.12,
            height: 0.065,
            guide_tooth_height: 0.035,
            guide_tooth_width: 0.08,
            pin_radius: 0.012,
            pad_thickness: 0.008,
            grouser_count: 2,
            grouser_height: 0.015,
        }
    }
}

/// Generate simple box track link mesh
pub fn generate_track_link(config: &TrackLinkConfig) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Just a simple box for the track link
    add_box(
        &mut positions, &mut normals, &mut uvs, &mut indices,
        Vec3::ZERO,
        Vec3::new(config.length, config.height, config.width),
    );

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Helper function to add a box to the mesh
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
        let face_base = positions.len() as u32;

        for v in verts {
            positions.push([center.x + v[0], center.y + v[1], center.z + v[2]]);
            normals.push(normal);
        }

        uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);

        indices.extend_from_slice(&[
            face_base, face_base + 1, face_base + 2,
            face_base, face_base + 2, face_base + 3,
        ]);
    }
}

/// Track path definition for positioning links
#[derive(Clone, Debug)]
pub struct TrackPath {
    pub sprocket: (Vec3, f32),
    pub idler: (Vec3, f32),
    pub road_wheels: Vec<(Vec3, f32)>,
    pub return_rollers: Vec<(Vec3, f32)>,
    pub tension: f32,
}

impl Default for TrackPath {
    fn default() -> Self {
        Self::t54()
    }
}

impl TrackPath {
    pub fn t54() -> Self {
        Self {
            sprocket: (Vec3::new(1.65, 0.45, 0.0), 0.35),
            idler: (Vec3::new(-1.65, 0.45, 0.0), 0.32),
            road_wheels: vec![
                (Vec3::new(-1.1, 0.28, 0.0), 0.28),
                (Vec3::new(-0.55, 0.28, 0.0), 0.28),
                (Vec3::new(0.0, 0.28, 0.0), 0.28),
                (Vec3::new(0.55, 0.28, 0.0), 0.28),
                (Vec3::new(1.1, 0.28, 0.0), 0.28),
            ],
            return_rollers: vec![
                (Vec3::new(-0.6, 0.68, 0.0), 0.09),
                (Vec3::new(0.0, 0.68, 0.0), 0.09),
                (Vec3::new(0.6, 0.68, 0.0), 0.09),
            ],
            tension: 0.95,
        }
    }
}

/// Calculate transforms for all track links along the path
pub fn calculate_track_link_transforms(
    path: &TrackPath,
    link_config: &TrackLinkConfig,
    z_offset: f32,
) -> Vec<Transform> {
    let mut transforms = Vec::new();

    let link_length = link_config.length;

    // Bottom section
    let ground_y = 0.12;
    let front_x = path.road_wheels.last().map(|(p, _)| p.x).unwrap_or(1.0) + 0.3;
    let rear_x = path.road_wheels.first().map(|(p, _)| p.x).unwrap_or(-1.0) - 0.3;

    let bottom_length = front_x - rear_x;
    let bottom_links = (bottom_length / link_length).ceil() as u32;

    for i in 0..bottom_links {
        let t = i as f32 / bottom_links as f32;
        let x = rear_x + t * bottom_length;
        transforms.push(Transform::from_xyz(x, ground_y, z_offset));
    }

    // Front curve
    let sprocket_circumference = 2.0 * PI * path.sprocket.1 * 0.5;
    let sprocket_links = (sprocket_circumference / link_length).ceil() as u32;

    for i in 0..sprocket_links {
        let t = i as f32 / sprocket_links as f32;
        let angle = -PI * 0.5 + t * PI;

        let x = path.sprocket.0.x + angle.cos() * path.sprocket.1;
        let y = path.sprocket.0.y + angle.sin() * path.sprocket.1;

        transforms.push(
            Transform::from_xyz(x, y, z_offset)
                .with_rotation(Quat::from_rotation_z(angle + PI * 0.5))
        );
    }

    // Top section
    let top_y = 0.78;
    let top_links = bottom_links;

    for i in 0..top_links {
        let t = i as f32 / top_links as f32;
        let x = front_x - t * bottom_length;
        let sag = 0.02 * (t * PI * 3.0).sin().abs();

        transforms.push(
            Transform::from_xyz(x, top_y - sag, z_offset)
                .with_rotation(Quat::from_rotation_z(PI))
        );
    }

    // Rear curve
    let idler_circumference = 2.0 * PI * path.idler.1 * 0.5;
    let idler_links = (idler_circumference / link_length).ceil() as u32;

    for i in 0..idler_links {
        let t = i as f32 / idler_links as f32;
        let angle = PI * 0.5 + t * PI;

        let x = path.idler.0.x + angle.cos() * path.idler.1;
        let y = path.idler.0.y + angle.sin() * path.idler.1;

        transforms.push(
            Transform::from_xyz(x, y, z_offset)
                .with_rotation(Quat::from_rotation_z(angle + PI * 0.5))
        );
    }

    transforms
}

/// Marker component for track link entities
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct TrackLinkMarker {
    pub index: u32,
    pub side: f32,
}
