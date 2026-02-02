use bevy::prelude::*;
use bevy::render::mesh::{Indices, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use crate::resources::MonteCassinoTerrain;

/// Generate a terrain mesh from the Monte Cassino heightmap with vertex colors
pub fn generate_terrain_mesh(
    terrain: &MonteCassinoTerrain,
    subdivisions: u32,
) -> Mesh {
    let vertices_per_row = subdivisions + 1;
    let total_vertices = (vertices_per_row * vertices_per_row) as usize;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(total_vertices);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(total_vertices);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(total_vertices);
    let mut colors: Vec<[f32; 4]> = Vec::with_capacity(total_vertices);

    let world_size_x = terrain.world_max.x - terrain.world_min.x;
    let world_size_z = terrain.world_max.y - terrain.world_min.y;

    // Generate vertices
    for z in 0..=subdivisions {
        for x in 0..=subdivisions {
            let u = x as f32 / subdivisions as f32;
            let v = z as f32 / subdivisions as f32;

            let world_x = terrain.world_min.x + u * world_size_x;
            let world_z = terrain.world_min.y + v * world_size_z;

            let height = terrain.sample_height(world_x, world_z);
            let normal = terrain.sample_normal(world_x, world_z);

            positions.push([world_x, height, world_z]);
            normals.push([normal.x, normal.y, normal.z]);
            // Scale UVs for texture tiling (50x repeat across 500 unit terrain)
            uvs.push([u * 50.0, v * 50.0]);
            // Add vertex color based on height and slope
            colors.push(get_elevation_color_smooth(height, normal.y));
        }
    }

    // Generate indices for triangles
    let mut indices: Vec<u32> = Vec::new();
    for z in 0..subdivisions {
        for x in 0..subdivisions {
            let i00 = z * vertices_per_row + x;
            let i10 = z * vertices_per_row + x + 1;
            let i01 = (z + 1) * vertices_per_row + x;
            let i11 = (z + 1) * vertices_per_row + x + 1;

            // Two triangles per quad
            indices.extend_from_slice(&[i00, i01, i10]);
            indices.extend_from_slice(&[i10, i01, i11]);
        }
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::all(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Smooth elevation color with slope influence (public for LOD chunks)
pub fn get_elevation_color_smooth(height: f32, normal_y: f32) -> [f32; 4] {
    // Base colors for different elevations
    let valley = [0.32, 0.28, 0.22, 1.0];      // Dark brown - valley floor
    let grass_low = [0.28, 0.38, 0.22, 1.0];   // Green - lower slopes
    let grass_high = [0.35, 0.42, 0.28, 1.0];  // Lighter green - mid slopes
    let rock = [0.48, 0.45, 0.40, 1.0];        // Gray-brown - rocky areas
    let peak = [0.58, 0.55, 0.52, 1.0];        // Light gray - mountain peaks

    // Slope factor: steep slopes show more rock
    let slope_factor = (1.0 - normal_y).clamp(0.0, 1.0);

    // Height-based interpolation
    let base_color = if height < 2.0 {
        valley
    } else if height < 8.0 {
        let t = (height - 2.0) / 6.0;
        lerp_color(valley, grass_low, t)
    } else if height < 18.0 {
        let t = (height - 8.0) / 10.0;
        lerp_color(grass_low, grass_high, t)
    } else if height < 28.0 {
        let t = (height - 18.0) / 10.0;
        lerp_color(grass_high, rock, t)
    } else {
        let t = ((height - 28.0) / 10.0).clamp(0.0, 1.0);
        lerp_color(rock, peak, t)
    };

    // Blend with rock color on steep slopes
    if slope_factor > 0.3 {
        let rock_blend = ((slope_factor - 0.3) / 0.4).clamp(0.0, 1.0);
        lerp_color(base_color, rock, rock_blend)
    } else {
        base_color
    }
}

/// Linear interpolation between two colors
fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        1.0,
    ]
}

/// Generate a river strip mesh following the river path
pub fn generate_river_mesh(
    terrain: &MonteCassinoTerrain,
    center_points: &[Vec2],
    widths: &[f32],
) -> Mesh {
    let segments = center_points.len() - 1;
    let vertices_per_segment = 2;
    let total_vertices = (segments + 1) * vertices_per_segment;

    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(total_vertices);
    let mut normals: Vec<[f32; 3]> = Vec::with_capacity(total_vertices);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(total_vertices);

    // Generate vertices along the river path
    for i in 0..=segments {
        let center = center_points[i.min(center_points.len() - 1)];
        let width = widths[i.min(widths.len() - 1)];

        // Calculate perpendicular direction
        let dir = if i < segments {
            (center_points[i + 1] - center).normalize()
        } else if i > 0 {
            (center - center_points[i - 1]).normalize()
        } else {
            Vec2::new(0.0, 1.0)
        };
        let perp = Vec2::new(-dir.y, dir.x);

        // Left and right points
        let left = center - perp * width * 0.5;
        let right = center + perp * width * 0.5;

        // Sample terrain height at these positions (water sits slightly above terrain)
        let height_left = terrain.sample_height(left.x, left.y) + 0.1;
        let height_right = terrain.sample_height(right.x, right.y) + 0.1;

        // Use minimum height for water level
        let water_height = height_left.min(height_right);

        let v = i as f32 / segments as f32;

        positions.push([left.x, water_height, left.y]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([0.0, v]);

        positions.push([right.x, water_height, right.y]);
        normals.push([0.0, 1.0, 0.0]);
        uvs.push([1.0, v]);
    }

    // Generate indices
    let mut indices: Vec<u32> = Vec::new();
    for i in 0..segments as u32 {
        let base = i * 2;
        // Two triangles per segment
        indices.extend_from_slice(&[base, base + 2, base + 1]);
        indices.extend_from_slice(&[base + 1, base + 2, base + 3]);
    }

    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::all(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_indices(Indices::U32(indices));

    mesh
}

/// Color lookup based on elevation for terrain visualization
pub fn get_elevation_color(height: f32) -> Color {
    // Heights in game units (after scaling)
    match height {
        h if h < 2.0 => Color::srgb(0.35, 0.32, 0.25),   // Valley floor - brown
        h if h < 8.0 => Color::srgb(0.32, 0.42, 0.28),   // Lower slopes - green
        h if h < 20.0 => Color::srgb(0.38, 0.45, 0.32),  // Mid slopes - lighter green
        h if h < 30.0 => Color::srgb(0.48, 0.46, 0.42),  // Upper slopes - gray-brown
        _ => Color::srgb(0.55, 0.52, 0.48),              // Peaks - light gray
    }
}
