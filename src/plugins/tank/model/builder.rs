//! Mesh builder primitives for procedural geometry generation
//!
//! Provides builder pattern for creating detailed meshes with
//! beveled edges, rivets, and other details.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, PrimitiveTopology};
use bevy::render::render_asset::RenderAssetUsages;
use std::f32::consts::{PI, TAU};

// ============================================================================
// MESH BUILDER TRAIT
// ============================================================================

/// Trait for mesh generation with builder pattern
pub trait MeshBuilder {
    /// Build the final mesh
    fn build(&self) -> Mesh;

    /// Estimate triangle count (for LOD/budgeting)
    fn triangle_count(&self) -> u32;
}

// ============================================================================
// BEVELED BOX
// ============================================================================

/// A box with beveled (rounded) edges
#[derive(Clone, Debug)]
pub struct BeveledBox {
    /// Full size (not half-extents)
    pub size: Vec3,
    /// Bevel radius
    pub bevel_radius: f32,
    /// Number of segments for bevel curve
    pub bevel_segments: u32,
}

impl BeveledBox {
    pub fn new(size: Vec3) -> Self {
        Self {
            size,
            bevel_radius: 0.02,
            bevel_segments: 3,
        }
    }

    pub fn with_bevel(mut self, radius: f32, segments: u32) -> Self {
        self.bevel_radius = radius;
        self.bevel_segments = segments.max(1);
        self
    }

    /// Generate vertices for one beveled edge
    fn generate_edge_vertices(
        &self,
        start: Vec3,
        end: Vec3,
        normal1: Vec3,
        normal2: Vec3,
    ) -> Vec<(Vec3, Vec3)> {
        let mut vertices = Vec::new();
        let edge_dir = (end - start).normalize();

        for i in 0..=self.bevel_segments {
            let t = i as f32 / self.bevel_segments as f32;
            let angle = t * PI * 0.5;

            // Interpolate normal around the bevel
            let blend_normal = (normal1 * angle.cos() + normal2 * angle.sin()).normalize();
            let offset = blend_normal * self.bevel_radius;

            // Two points along the edge
            vertices.push((start + offset, blend_normal));
            vertices.push((end + offset, blend_normal));
        }

        vertices
    }
}

impl MeshBuilder for BeveledBox {
    fn build(&self) -> Mesh {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let half = self.size * 0.5;
        let r = self.bevel_radius.min(half.x.min(half.y.min(half.z)) * 0.5);
        let inner = half - Vec3::splat(r);

        // Generate 6 faces with inset for bevel
        let faces = [
            // (normal, u_axis, v_axis, center_offset)
            (Vec3::X, Vec3::Z, Vec3::Y, Vec3::new(half.x, 0.0, 0.0)),   // +X
            (Vec3::NEG_X, Vec3::NEG_Z, Vec3::Y, Vec3::new(-half.x, 0.0, 0.0)), // -X
            (Vec3::Y, Vec3::X, Vec3::NEG_Z, Vec3::new(0.0, half.y, 0.0)),  // +Y
            (Vec3::NEG_Y, Vec3::X, Vec3::Z, Vec3::new(0.0, -half.y, 0.0)), // -Y
            (Vec3::Z, Vec3::NEG_X, Vec3::Y, Vec3::new(0.0, 0.0, half.z)),  // +Z
            (Vec3::NEG_Z, Vec3::X, Vec3::Y, Vec3::new(0.0, 0.0, -half.z)), // -Z
        ];

        for (normal, u_axis, v_axis, center) in faces {
            let base_idx = positions.len() as u32;

            // Get the dimensions for this face
            let u_extent = if u_axis.x.abs() > 0.5 {
                inner.x
            } else if u_axis.y.abs() > 0.5 {
                inner.y
            } else {
                inner.z
            };
            let v_extent = if v_axis.x.abs() > 0.5 {
                inner.x
            } else if v_axis.y.abs() > 0.5 {
                inner.y
            } else {
                inner.z
            };

            // 4 corners of inner face
            let corners = [
                center + u_axis * u_extent + v_axis * v_extent,
                center - u_axis * u_extent + v_axis * v_extent,
                center - u_axis * u_extent - v_axis * v_extent,
                center + u_axis * u_extent - v_axis * v_extent,
            ];

            // Add center face vertices
            for (i, corner) in corners.iter().enumerate() {
                positions.push([corner.x, corner.y, corner.z]);
                normals.push([normal.x, normal.y, normal.z]);
                let u = if i == 0 || i == 3 { 1.0 } else { 0.0 };
                let v = if i == 0 || i == 1 { 1.0 } else { 0.0 };
                uvs.push([u, v]);
            }

            // Center face triangles
            indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
            indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);
        }

        // Generate beveled edges (12 edges total)
        let edge_segments = self.bevel_segments;
        if r > 0.001 && edge_segments > 0 {
            // Edges along X axis
            for &(y_sign, z_sign) in &[(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                let base_idx = positions.len() as u32;
                let y = inner.y * y_sign;
                let z = inner.z * z_sign;

                for i in 0..=edge_segments {
                    let t = i as f32 / edge_segments as f32;
                    let angle = t * PI * 0.5;

                    let ny = y_sign * angle.cos();
                    let nz = z_sign * angle.sin();
                    let normal = Vec3::new(0.0, ny, nz).normalize();

                    let py = y + r * ny;
                    let pz = z + r * nz;

                    // Two ends of edge
                    positions.push([-inner.x, py, pz]);
                    positions.push([inner.x, py, pz]);
                    normals.push([normal.x, normal.y, normal.z]);
                    normals.push([normal.x, normal.y, normal.z]);
                    uvs.push([0.0, t]);
                    uvs.push([1.0, t]);
                }

                // Generate triangles for this edge bevel
                for i in 0..edge_segments {
                    let idx = base_idx + i * 2;
                    indices.extend_from_slice(&[idx, idx + 2, idx + 1]);
                    indices.extend_from_slice(&[idx + 1, idx + 2, idx + 3]);
                }
            }

            // Edges along Y axis
            for &(x_sign, z_sign) in &[(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                let base_idx = positions.len() as u32;
                let x = inner.x * x_sign;
                let z = inner.z * z_sign;

                for i in 0..=edge_segments {
                    let t = i as f32 / edge_segments as f32;
                    let angle = t * PI * 0.5;

                    let nx = x_sign * angle.cos();
                    let nz = z_sign * angle.sin();
                    let normal = Vec3::new(nx, 0.0, nz).normalize();

                    let px = x + r * nx;
                    let pz = z + r * nz;

                    positions.push([px, -inner.y, pz]);
                    positions.push([px, inner.y, pz]);
                    normals.push([normal.x, normal.y, normal.z]);
                    normals.push([normal.x, normal.y, normal.z]);
                    uvs.push([0.0, t]);
                    uvs.push([1.0, t]);
                }

                for i in 0..edge_segments {
                    let idx = base_idx + i * 2;
                    indices.extend_from_slice(&[idx, idx + 1, idx + 2]);
                    indices.extend_from_slice(&[idx + 1, idx + 3, idx + 2]);
                }
            }

            // Edges along Z axis
            for &(x_sign, y_sign) in &[(1.0, 1.0), (1.0, -1.0), (-1.0, 1.0), (-1.0, -1.0)] {
                let base_idx = positions.len() as u32;
                let x = inner.x * x_sign;
                let y = inner.y * y_sign;

                for i in 0..=edge_segments {
                    let t = i as f32 / edge_segments as f32;
                    let angle = t * PI * 0.5;

                    let nx = x_sign * angle.cos();
                    let ny = y_sign * angle.sin();
                    let normal = Vec3::new(nx, ny, 0.0).normalize();

                    let px = x + r * nx;
                    let py = y + r * ny;

                    positions.push([px, py, -inner.z]);
                    positions.push([px, py, inner.z]);
                    normals.push([normal.x, normal.y, normal.z]);
                    normals.push([normal.x, normal.y, normal.z]);
                    uvs.push([0.0, t]);
                    uvs.push([1.0, t]);
                }

                for i in 0..edge_segments {
                    let idx = base_idx + i * 2;
                    indices.extend_from_slice(&[idx, idx + 2, idx + 1]);
                    indices.extend_from_slice(&[idx + 1, idx + 2, idx + 3]);
                }
            }

            // Generate corner bevels (8 corners)
            for &x_sign in &[1.0_f32, -1.0] {
                for &y_sign in &[1.0_f32, -1.0] {
                    for &z_sign in &[1.0_f32, -1.0] {
                        let corner = Vec3::new(
                            inner.x * x_sign,
                            inner.y * y_sign,
                            inner.z * z_sign,
                        );

                        // Simplified corner: single triangle fan
                        let base_idx = positions.len() as u32;
                        let corner_normal = Vec3::new(x_sign, y_sign, z_sign).normalize();

                        // Center of corner sphere
                        positions.push([
                            corner.x + r * corner_normal.x,
                            corner.y + r * corner_normal.y,
                            corner.z + r * corner_normal.z,
                        ]);
                        normals.push([corner_normal.x, corner_normal.y, corner_normal.z]);
                        uvs.push([0.5, 0.5]);

                        // 3 edge points
                        let edge_normals = [
                            Vec3::new(x_sign, 0.0, 0.0),
                            Vec3::new(0.0, y_sign, 0.0),
                            Vec3::new(0.0, 0.0, z_sign),
                        ];

                        for en in &edge_normals {
                            positions.push([corner.x + r * en.x, corner.y + r * en.y, corner.z + r * en.z]);
                            normals.push([en.x, en.y, en.z]);
                            uvs.push([0.5 + en.x * 0.5, 0.5 + en.y * 0.5]);
                        }

                        // Triangle fan
                        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
                        indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);
                        indices.extend_from_slice(&[base_idx, base_idx + 3, base_idx + 1]);
                    }
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }

    fn triangle_count(&self) -> u32 {
        // 6 faces * 2 triangles = 12
        // 12 edges * bevel_segments * 2 triangles
        // 8 corners * 3 triangles
        12 + 12 * self.bevel_segments * 2 + 8 * 3
    }
}

// ============================================================================
// DETAILED CYLINDER
// ============================================================================

/// A cylinder with configurable detail level
#[derive(Clone, Debug)]
pub struct DetailedCylinder {
    pub radius_bottom: f32,
    pub radius_top: f32,
    pub height: f32,
    pub radial_segments: u32,
    pub height_segments: u32,
    pub cap_segments: u32,
    pub open_ended: bool,
}

impl DetailedCylinder {
    pub fn new(radius: f32, height: f32) -> Self {
        Self {
            radius_bottom: radius,
            radius_top: radius,
            height,
            radial_segments: 32,
            height_segments: 1,
            cap_segments: 1,
            open_ended: false,
        }
    }

    pub fn cone(radius_bottom: f32, radius_top: f32, height: f32) -> Self {
        Self {
            radius_bottom,
            radius_top,
            height,
            radial_segments: 32,
            height_segments: 1,
            cap_segments: 1,
            open_ended: false,
        }
    }

    pub fn with_segments(mut self, radial: u32, height: u32) -> Self {
        self.radial_segments = radial.max(3);
        self.height_segments = height.max(1);
        self
    }

    pub fn with_caps(mut self, segments: u32) -> Self {
        self.cap_segments = segments.max(1);
        self.open_ended = false;
        self
    }

    pub fn open(mut self) -> Self {
        self.open_ended = true;
        self
    }
}

impl MeshBuilder for DetailedCylinder {
    fn build(&self) -> Mesh {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let half_height = self.height * 0.5;

        // Generate side vertices
        for y in 0..=self.height_segments {
            let v = y as f32 / self.height_segments as f32;
            let py = -half_height + v * self.height;
            let radius = self.radius_bottom + (self.radius_top - self.radius_bottom) * v;

            // Calculate normal slope for cone
            let slope = (self.radius_bottom - self.radius_top) / self.height;

            for x in 0..=self.radial_segments {
                let u = x as f32 / self.radial_segments as f32;
                let angle = u * TAU;

                let cos_a = angle.cos();
                let sin_a = angle.sin();

                positions.push([cos_a * radius, py, sin_a * radius]);

                // Normal includes slope for cones
                let normal = Vec3::new(cos_a, slope, sin_a).normalize();
                normals.push([normal.x, normal.y, normal.z]);

                uvs.push([u, v]);
            }
        }

        // Generate side indices
        for y in 0..self.height_segments {
            for x in 0..self.radial_segments {
                let row_width = self.radial_segments + 1;
                let i00 = y * row_width + x;
                let i10 = y * row_width + x + 1;
                let i01 = (y + 1) * row_width + x;
                let i11 = (y + 1) * row_width + x + 1;

                indices.extend_from_slice(&[i00, i01, i10]);
                indices.extend_from_slice(&[i10, i01, i11]);
            }
        }

        // Generate caps
        if !self.open_ended {
            // Bottom cap
            if self.radius_bottom > 0.001 {
                let base_idx = positions.len() as u32;

                // Center vertex
                positions.push([0.0, -half_height, 0.0]);
                normals.push([0.0, -1.0, 0.0]);
                uvs.push([0.5, 0.5]);

                // Ring vertices
                for i in 0..=self.radial_segments {
                    let u = i as f32 / self.radial_segments as f32;
                    let angle = u * TAU;
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();

                    positions.push([cos_a * self.radius_bottom, -half_height, sin_a * self.radius_bottom]);
                    normals.push([0.0, -1.0, 0.0]);
                    uvs.push([0.5 + cos_a * 0.5, 0.5 + sin_a * 0.5]);
                }

                // Bottom cap triangles (reversed winding)
                for i in 0..self.radial_segments {
                    indices.extend_from_slice(&[base_idx, base_idx + i + 2, base_idx + i + 1]);
                }
            }

            // Top cap
            if self.radius_top > 0.001 {
                let base_idx = positions.len() as u32;

                // Center vertex
                positions.push([0.0, half_height, 0.0]);
                normals.push([0.0, 1.0, 0.0]);
                uvs.push([0.5, 0.5]);

                // Ring vertices
                for i in 0..=self.radial_segments {
                    let u = i as f32 / self.radial_segments as f32;
                    let angle = u * TAU;
                    let cos_a = angle.cos();
                    let sin_a = angle.sin();

                    positions.push([cos_a * self.radius_top, half_height, sin_a * self.radius_top]);
                    normals.push([0.0, 1.0, 0.0]);
                    uvs.push([0.5 + cos_a * 0.5, 0.5 + sin_a * 0.5]);
                }

                // Top cap triangles
                for i in 0..self.radial_segments {
                    indices.extend_from_slice(&[base_idx, base_idx + i + 1, base_idx + i + 2]);
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }

    fn triangle_count(&self) -> u32 {
        let side_tris = self.radial_segments * self.height_segments * 2;
        let cap_tris = if self.open_ended { 0 } else { self.radial_segments * 2 };
        side_tris + cap_tris
    }
}

// ============================================================================
// RIVETED PANEL
// ============================================================================

/// Pattern for rivet placement
#[derive(Clone, Debug)]
pub enum RivetPattern {
    /// Grid of rivets
    Grid { rows: u32, cols: u32, spacing: Vec2 },
    /// Rivets along edges only
    Edge { count_per_edge: u32, inset: f32 },
    /// Custom positions (normalized 0-1)
    Custom(Vec<Vec2>),
}

/// A flat panel with rivet details
#[derive(Clone, Debug)]
pub struct RivetedPanel {
    pub size: Vec2,
    pub thickness: f32,
    pub rivet_pattern: RivetPattern,
    pub rivet_radius: f32,
    pub rivet_height: f32,
}

impl RivetedPanel {
    pub fn new(size: Vec2, thickness: f32) -> Self {
        Self {
            size,
            thickness,
            rivet_pattern: RivetPattern::Edge {
                count_per_edge: 4,
                inset: 0.05,
            },
            rivet_radius: 0.008,
            rivet_height: 0.004,
        }
    }

    pub fn with_grid_rivets(mut self, rows: u32, cols: u32, spacing: Vec2) -> Self {
        self.rivet_pattern = RivetPattern::Grid { rows, cols, spacing };
        self
    }

    pub fn with_edge_rivets(mut self, count: u32, inset: f32) -> Self {
        self.rivet_pattern = RivetPattern::Edge {
            count_per_edge: count,
            inset,
        };
        self
    }

    fn get_rivet_positions(&self) -> Vec<Vec2> {
        match &self.rivet_pattern {
            RivetPattern::Grid { rows, cols, spacing } => {
                let mut positions = Vec::new();
                let start = Vec2::new(
                    -((*cols - 1) as f32 * spacing.x) * 0.5,
                    -((*rows - 1) as f32 * spacing.y) * 0.5,
                );
                for row in 0..*rows {
                    for col in 0..*cols {
                        positions.push(start + Vec2::new(col as f32 * spacing.x, row as f32 * spacing.y));
                    }
                }
                positions
            }
            RivetPattern::Edge { count_per_edge, inset } => {
                let mut positions = Vec::new();
                let half = self.size * 0.5 - Vec2::splat(*inset);

                // Top and bottom edges
                for i in 0..*count_per_edge {
                    let t = (i as f32 + 0.5) / *count_per_edge as f32;
                    let x = -half.x + t * half.x * 2.0;
                    positions.push(Vec2::new(x, half.y));
                    positions.push(Vec2::new(x, -half.y));
                }

                // Left and right edges (excluding corners)
                for i in 1..count_per_edge.saturating_sub(1) {
                    let t = i as f32 / (*count_per_edge - 1) as f32;
                    let y = -half.y + t * half.y * 2.0;
                    positions.push(Vec2::new(-half.x, y));
                    positions.push(Vec2::new(half.x, y));
                }

                positions
            }
            RivetPattern::Custom(positions) => {
                positions
                    .iter()
                    .map(|p| (*p - Vec2::splat(0.5)) * self.size)
                    .collect()
            }
        }
    }
}

impl MeshBuilder for RivetedPanel {
    fn build(&self) -> Mesh {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        let half_size = self.size * 0.5;
        let half_thick = self.thickness * 0.5;

        // Main panel (front face)
        let base_idx = positions.len() as u32;
        positions.extend_from_slice(&[
            [-half_size.x, -half_size.y, half_thick],
            [half_size.x, -half_size.y, half_thick],
            [half_size.x, half_size.y, half_thick],
            [-half_size.x, half_size.y, half_thick],
        ]);
        for _ in 0..4 {
            normals.push([0.0, 0.0, 1.0]);
        }
        uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);

        // Back face
        let base_idx = positions.len() as u32;
        positions.extend_from_slice(&[
            [half_size.x, -half_size.y, -half_thick],
            [-half_size.x, -half_size.y, -half_thick],
            [-half_size.x, half_size.y, -half_thick],
            [half_size.x, half_size.y, -half_thick],
        ]);
        for _ in 0..4 {
            normals.push([0.0, 0.0, -1.0]);
        }
        uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
        indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);

        // Side faces (4 edges)
        let edge_faces = [
            (Vec3::NEG_Y, [-half_size.x, -half_size.y], [half_size.x, -half_size.y]),
            (Vec3::Y, [half_size.x, half_size.y], [-half_size.x, half_size.y]),
            (Vec3::NEG_X, [-half_size.x, half_size.y], [-half_size.x, -half_size.y]),
            (Vec3::X, [half_size.x, -half_size.y], [half_size.x, half_size.y]),
        ];

        for (normal, start, end) in edge_faces {
            let base_idx = positions.len() as u32;
            positions.extend_from_slice(&[
                [start[0], start[1], half_thick],
                [end[0], end[1], half_thick],
                [end[0], end[1], -half_thick],
                [start[0], start[1], -half_thick],
            ]);
            for _ in 0..4 {
                normals.push([normal.x, normal.y, normal.z]);
            }
            uvs.extend_from_slice(&[[0.0, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]]);
            indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
            indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);
        }

        // Add rivets
        let rivet_segments = 6;
        for rivet_pos in self.get_rivet_positions() {
            let base_idx = positions.len() as u32;

            // Center of rivet
            positions.push([rivet_pos.x, rivet_pos.y, half_thick + self.rivet_height]);
            normals.push([0.0, 0.0, 1.0]);
            uvs.push([0.5, 0.5]);

            // Ring around rivet
            for i in 0..=rivet_segments {
                let angle = (i as f32 / rivet_segments as f32) * TAU;
                let cos_a = angle.cos();
                let sin_a = angle.sin();

                let px = rivet_pos.x + cos_a * self.rivet_radius;
                let py = rivet_pos.y + sin_a * self.rivet_radius;

                positions.push([px, py, half_thick]);

                // Outward-facing normal for rivet dome
                let normal = Vec3::new(cos_a * 0.5, sin_a * 0.5, 0.866).normalize();
                normals.push([normal.x, normal.y, normal.z]);
                uvs.push([0.5 + cos_a * 0.5, 0.5 + sin_a * 0.5]);
            }

            // Triangles for rivet dome
            for i in 0..rivet_segments {
                indices.extend_from_slice(&[base_idx, base_idx + i + 1, base_idx + i + 2]);
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }

    fn triangle_count(&self) -> u32 {
        let panel_tris = 12; // 6 faces * 2 triangles
        let rivet_count = self.get_rivet_positions().len() as u32;
        let rivet_tris = rivet_count * 6; // 6 segments per rivet
        panel_tris + rivet_tris
    }
}

// ============================================================================
// COMPOUND MESH BUILDER
// ============================================================================

/// Combines multiple mesh builders into one mesh
pub struct CompoundMeshBuilder {
    parts: Vec<(Mesh, Transform)>,
}

impl CompoundMeshBuilder {
    pub fn new() -> Self {
        Self { parts: Vec::new() }
    }

    /// Add a mesh builder with transform
    pub fn add<T: MeshBuilder>(mut self, builder: T, transform: Transform) -> Self {
        self.parts.push((builder.build(), transform));
        self
    }

    /// Add a pre-built mesh with transform
    pub fn add_mesh(mut self, mesh: Mesh, transform: Transform) -> Self {
        self.parts.push((mesh, transform));
        self
    }
}

impl Default for CompoundMeshBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshBuilder for CompoundMeshBuilder {
    fn build(&self) -> Mesh {
        let mut positions: Vec<[f32; 3]> = Vec::new();
        let mut normals: Vec<[f32; 3]> = Vec::new();
        let mut uvs: Vec<[f32; 2]> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();

        for (part_mesh, transform) in &self.parts {
            let base_idx = positions.len() as u32;

            // Get attributes from part mesh
            if let Some(pos_attr) = part_mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
                if let bevy::render::mesh::VertexAttributeValues::Float32x3(pos) = pos_attr {
                    for p in pos {
                        let transformed = transform.transform_point(Vec3::from_array(*p));
                        positions.push([transformed.x, transformed.y, transformed.z]);
                    }
                }
            }

            if let Some(norm_attr) = part_mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
                if let bevy::render::mesh::VertexAttributeValues::Float32x3(norms) = norm_attr {
                    for n in norms {
                        let transformed = transform.rotation * Vec3::from_array(*n);
                        normals.push([transformed.x, transformed.y, transformed.z]);
                    }
                }
            }

            if let Some(uv_attr) = part_mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
                if let bevy::render::mesh::VertexAttributeValues::Float32x2(uv) = uv_attr {
                    uvs.extend_from_slice(uv);
                }
            }

            if let Some(Indices::U32(idx)) = part_mesh.indices() {
                for i in idx {
                    indices.push(base_idx + i);
                }
            }
        }

        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::all());
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.insert_indices(Indices::U32(indices));

        mesh
    }

    fn triangle_count(&self) -> u32 {
        self.parts
            .iter()
            .filter_map(|(mesh, _)| mesh.indices())
            .map(|idx| match idx {
                Indices::U32(v) => v.len() as u32 / 3,
                Indices::U16(v) => v.len() as u32 / 3,
            })
            .sum()
    }
}
