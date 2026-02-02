//! Normal calculation utilities for procedural meshes

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, VertexAttributeValues};
use std::collections::HashMap;

/// Compute smooth normals with angle threshold
///
/// Vertices sharing an edge sharper than `angle_threshold` (in radians)
/// will have separate normals (hard edge).
pub fn compute_smooth_normals_threshold(mesh: &mut Mesh, angle_threshold: f32) {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(pos)) => pos.clone(),
        _ => return,
    };

    let indices = match mesh.indices() {
        Some(Indices::U32(idx)) => idx.clone(),
        Some(Indices::U16(idx)) => idx.iter().map(|&i| i as u32).collect(),
        None => return,
    };

    let vertex_count = positions.len();
    let triangle_count = indices.len() / 3;

    // Calculate face normals
    let mut face_normals = Vec::with_capacity(triangle_count);
    for tri in 0..triangle_count {
        let i0 = indices[tri * 3] as usize;
        let i1 = indices[tri * 3 + 1] as usize;
        let i2 = indices[tri * 3 + 2] as usize;

        let v0 = Vec3::from_array(positions[i0]);
        let v1 = Vec3::from_array(positions[i1]);
        let v2 = Vec3::from_array(positions[i2]);

        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize_or_zero();

        face_normals.push(normal);
    }

    // Build vertex -> triangles mapping
    let mut vertex_triangles: Vec<Vec<usize>> = vec![Vec::new(); vertex_count];
    for tri in 0..triangle_count {
        for j in 0..3 {
            let vi = indices[tri * 3 + j] as usize;
            vertex_triangles[vi].push(tri);
        }
    }

    // Calculate smooth normals
    let cos_threshold = angle_threshold.cos();
    let mut new_normals = vec![[0.0_f32; 3]; vertex_count];

    for vi in 0..vertex_count {
        let tris = &vertex_triangles[vi];
        if tris.is_empty() {
            new_normals[vi] = [0.0, 1.0, 0.0];
            continue;
        }

        let mut accumulated = Vec3::ZERO;
        let base_normal = face_normals[tris[0]];

        for &tri in tris {
            let face_normal = face_normals[tri];

            // Check if this face should contribute (angle check)
            if base_normal.dot(face_normal) >= cos_threshold {
                // Weight by face area (approximated by normal length before normalization)
                accumulated += face_normal;
            }
        }

        let final_normal = accumulated.normalize_or_zero();
        new_normals[vi] = [final_normal.x, final_normal.y, final_normal.z];
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals);
}

/// Compute tangents for normal mapping using MikkTSpace algorithm (simplified)
pub fn compute_tangents(mesh: &mut Mesh) {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(pos)) => pos.clone(),
        _ => return,
    };

    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(norm)) => norm.clone(),
        _ => return,
    };

    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uv)) => uv.clone(),
        _ => return,
    };

    let indices = match mesh.indices() {
        Some(Indices::U32(idx)) => idx.clone(),
        Some(Indices::U16(idx)) => idx.iter().map(|&i| i as u32).collect(),
        None => return,
    };

    let vertex_count = positions.len();
    let mut tangents = vec![[0.0_f32; 4]; vertex_count];
    let mut tan1 = vec![Vec3::ZERO; vertex_count];
    let mut tan2 = vec![Vec3::ZERO; vertex_count];

    // Calculate tangent per triangle
    for tri in (0..indices.len()).step_by(3) {
        let i0 = indices[tri] as usize;
        let i1 = indices[tri + 1] as usize;
        let i2 = indices[tri + 2] as usize;

        let v0 = Vec3::from_array(positions[i0]);
        let v1 = Vec3::from_array(positions[i1]);
        let v2 = Vec3::from_array(positions[i2]);

        let uv0 = Vec2::from_array(uvs[i0]);
        let uv1 = Vec2::from_array(uvs[i1]);
        let uv2 = Vec2::from_array(uvs[i2]);

        let delta_pos1 = v1 - v0;
        let delta_pos2 = v2 - v0;

        let delta_uv1 = uv1 - uv0;
        let delta_uv2 = uv2 - uv0;

        let denom = delta_uv1.x * delta_uv2.y - delta_uv1.y * delta_uv2.x;
        if denom.abs() < 0.0001 {
            continue;
        }

        let r = 1.0 / denom;
        let tangent = (delta_pos1 * delta_uv2.y - delta_pos2 * delta_uv1.y) * r;
        let bitangent = (delta_pos2 * delta_uv1.x - delta_pos1 * delta_uv2.x) * r;

        tan1[i0] += tangent;
        tan1[i1] += tangent;
        tan1[i2] += tangent;

        tan2[i0] += bitangent;
        tan2[i1] += bitangent;
        tan2[i2] += bitangent;
    }

    // Orthogonalize and calculate handedness
    for i in 0..vertex_count {
        let n = Vec3::from_array(normals[i]);
        let t = tan1[i];

        // Gram-Schmidt orthogonalize
        let tangent = (t - n * n.dot(t)).normalize_or_zero();

        // Calculate handedness
        let w = if n.cross(t).dot(tan2[i]) < 0.0 { -1.0 } else { 1.0 };

        tangents[i] = [tangent.x, tangent.y, tangent.z, w];
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_TANGENT, tangents);
}

/// Flip normals (for inside-out meshes)
pub fn flip_normals(mesh: &mut Mesh) {
    if let Some(VertexAttributeValues::Float32x3(normals)) = mesh.attribute_mut(Mesh::ATTRIBUTE_NORMAL) {
        for normal in normals.iter_mut() {
            normal[0] = -normal[0];
            normal[1] = -normal[1];
            normal[2] = -normal[2];
        }
    }

    // Also flip winding order
    if let Some(indices) = mesh.indices_mut() {
        match indices {
            Indices::U32(idx) => {
                for tri in idx.chunks_exact_mut(3) {
                    tri.swap(1, 2);
                }
            }
            Indices::U16(idx) => {
                for tri in idx.chunks_exact_mut(3) {
                    tri.swap(1, 2);
                }
            }
        }
    }
}

/// Recalculate all normals as flat (per-face)
pub fn compute_flat_normals(mesh: &mut Mesh) {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(pos)) => pos.clone(),
        _ => return,
    };

    let old_indices = match mesh.indices() {
        Some(Indices::U32(idx)) => idx.clone(),
        Some(Indices::U16(idx)) => idx.iter().map(|&i| i as u32).collect(),
        None => return,
    };

    let old_uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uv)) => Some(uv.clone()),
        _ => None,
    };

    // Create new vertices with duplicated positions for flat shading
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_normals: Vec<[f32; 3]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();
    let mut new_indices: Vec<u32> = Vec::new();

    for tri in (0..old_indices.len()).step_by(3) {
        let i0 = old_indices[tri] as usize;
        let i1 = old_indices[tri + 1] as usize;
        let i2 = old_indices[tri + 2] as usize;

        let v0 = Vec3::from_array(positions[i0]);
        let v1 = Vec3::from_array(positions[i1]);
        let v2 = Vec3::from_array(positions[i2]);

        let normal = (v1 - v0).cross(v2 - v0).normalize_or_zero();
        let normal_arr = [normal.x, normal.y, normal.z];

        let base_idx = new_positions.len() as u32;

        new_positions.push(positions[i0]);
        new_positions.push(positions[i1]);
        new_positions.push(positions[i2]);

        new_normals.push(normal_arr);
        new_normals.push(normal_arr);
        new_normals.push(normal_arr);

        if let Some(ref uvs) = old_uvs {
            new_uvs.push(uvs[i0]);
            new_uvs.push(uvs[i1]);
            new_uvs.push(uvs[i2]);
        }

        new_indices.push(base_idx);
        new_indices.push(base_idx + 1);
        new_indices.push(base_idx + 2);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals);
    if !new_uvs.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, new_uvs);
    }
    mesh.insert_indices(Indices::U32(new_indices));
}

/// Weld vertices that are at the same position (within tolerance)
pub fn weld_vertices(mesh: &mut Mesh, tolerance: f32) {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(pos)) => pos.clone(),
        _ => return,
    };

    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(norm)) => Some(norm.clone()),
        _ => None,
    };

    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uv)) => Some(uv.clone()),
        _ => None,
    };

    let indices = match mesh.indices() {
        Some(Indices::U32(idx)) => idx.clone(),
        Some(Indices::U16(idx)) => idx.iter().map(|&i| i as u32).collect(),
        None => return,
    };

    // Map old vertex index -> new vertex index
    let mut vertex_map: HashMap<u32, u32> = HashMap::new();
    let mut new_positions: Vec<[f32; 3]> = Vec::new();
    let mut new_normals: Vec<[f32; 3]> = Vec::new();
    let mut new_uvs: Vec<[f32; 2]> = Vec::new();

    let tolerance_sq = tolerance * tolerance;

    for (old_idx, pos) in positions.iter().enumerate() {
        let pos_vec = Vec3::from_array(*pos);

        // Check if this vertex can be merged with an existing one
        let mut found_match = None;
        for (new_idx, new_pos) in new_positions.iter().enumerate() {
            if pos_vec.distance_squared(Vec3::from_array(*new_pos)) < tolerance_sq {
                found_match = Some(new_idx as u32);
                break;
            }
        }

        if let Some(new_idx) = found_match {
            vertex_map.insert(old_idx as u32, new_idx);
        } else {
            let new_idx = new_positions.len() as u32;
            vertex_map.insert(old_idx as u32, new_idx);
            new_positions.push(*pos);
            if let Some(ref norms) = normals {
                new_normals.push(norms[old_idx]);
            }
            if let Some(ref uv) = uvs {
                new_uvs.push(uv[old_idx]);
            }
        }
    }

    // Remap indices
    let new_indices: Vec<u32> = indices.iter().map(|&i| vertex_map[&i]).collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);
    if !new_normals.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals);
    }
    if !new_uvs.is_empty() {
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, new_uvs);
    }
    mesh.insert_indices(Indices::U32(new_indices));
}
