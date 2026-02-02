//! Bevel and chamfer operations for mesh edges
//!
//! These operations add rounded or angled edges to meshes
//! for more realistic appearance.

use bevy::prelude::*;
use bevy::render::mesh::{Indices, Mesh, VertexAttributeValues};
use std::collections::{HashMap, HashSet};

/// Error type for bevel operations
#[derive(Debug)]
pub enum BevelError {
    MissingAttribute(&'static str),
    InvalidMesh,
}

/// Edge representation (vertex indices, smaller first)
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Edge(pub u32, pub u32);

impl Edge {
    pub fn new(a: u32, b: u32) -> Self {
        if a < b {
            Edge(a, b)
        } else {
            Edge(b, a)
        }
    }
}

/// Information about an edge and its adjacent faces
#[derive(Clone, Debug)]
pub struct EdgeInfo {
    pub edge: Edge,
    pub face_indices: Vec<usize>,
    pub face_normals: Vec<Vec3>,
    pub edge_angle: f32, // Angle between faces in radians
}

/// Find all edges in a mesh and their adjacent faces
pub fn find_edges(mesh: &Mesh) -> Result<Vec<EdgeInfo>, BevelError> {
    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(pos)) => pos,
        _ => return Err(BevelError::MissingAttribute("POSITION")),
    };

    let indices = match mesh.indices() {
        Some(Indices::U32(idx)) => idx.clone(),
        Some(Indices::U16(idx)) => idx.iter().map(|&i| i as u32).collect(),
        None => return Err(BevelError::InvalidMesh),
    };

    // Calculate face normals
    let face_count = indices.len() / 3;
    let mut face_normals = Vec::with_capacity(face_count);

    for tri in 0..face_count {
        let i0 = indices[tri * 3] as usize;
        let i1 = indices[tri * 3 + 1] as usize;
        let i2 = indices[tri * 3 + 2] as usize;

        let v0 = Vec3::from_array(positions[i0]);
        let v1 = Vec3::from_array(positions[i1]);
        let v2 = Vec3::from_array(positions[i2]);

        let normal = (v1 - v0).cross(v2 - v0).normalize_or_zero();
        face_normals.push(normal);
    }

    // Build edge -> faces mapping
    let mut edge_faces: HashMap<Edge, Vec<usize>> = HashMap::new();

    for tri in 0..face_count {
        let i0 = indices[tri * 3];
        let i1 = indices[tri * 3 + 1];
        let i2 = indices[tri * 3 + 2];

        for edge in [Edge::new(i0, i1), Edge::new(i1, i2), Edge::new(i2, i0)] {
            edge_faces.entry(edge).or_default().push(tri);
        }
    }

    // Create EdgeInfo for each edge
    let mut edges = Vec::new();

    for (edge, face_indices) in edge_faces {
        let normals: Vec<Vec3> = face_indices.iter().map(|&fi| face_normals[fi]).collect();

        // Calculate angle between faces (for 2-face edges)
        let edge_angle = if normals.len() == 2 {
            normals[0].dot(normals[1]).clamp(-1.0, 1.0).acos()
        } else {
            0.0 // Boundary edge or non-manifold
        };

        edges.push(EdgeInfo {
            edge,
            face_indices,
            face_normals: normals,
            edge_angle,
        });
    }

    Ok(edges)
}

/// Apply chamfer (single-segment bevel) to edges sharper than threshold
///
/// This is a simplified bevel that adds a flat surface at each sharp edge.
pub fn chamfer_edges(
    mesh: &mut Mesh,
    distance: f32,
    angle_threshold: f32,
) -> Result<(), BevelError> {
    let edges = find_edges(mesh)?;

    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(pos)) => pos.clone(),
        _ => return Err(BevelError::MissingAttribute("POSITION")),
    };

    let normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(norm)) => norm.clone(),
        _ => return Err(BevelError::MissingAttribute("NORMAL")),
    };

    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uv)) => uv.clone(),
        _ => vec![[0.0, 0.0]; positions.len()],
    };

    let old_indices = match mesh.indices() {
        Some(Indices::U32(idx)) => idx.clone(),
        Some(Indices::U16(idx)) => idx.iter().map(|&i| i as u32).collect(),
        None => return Err(BevelError::InvalidMesh),
    };

    // Find edges that need chamfering
    let sharp_edges: Vec<&EdgeInfo> = edges
        .iter()
        .filter(|e| e.edge_angle > angle_threshold && e.face_normals.len() == 2)
        .collect();

    if sharp_edges.is_empty() {
        return Ok(()); // No edges to chamfer
    }

    // For each sharp edge, we need to:
    // 1. Move the edge vertices inward along both face normals
    // 2. Create new vertices for the chamfer surface
    // 3. Update face indices

    let mut new_positions = positions.clone();
    let mut new_normals = normals.clone();
    let mut new_uvs = uvs.clone();
    let mut new_indices = old_indices.clone();

    // Track vertices that have been split
    let mut _split_vertices: HashMap<u32, Vec<u32>> = HashMap::new();

    for edge_info in &sharp_edges {
        let i0 = edge_info.edge.0 as usize;
        let i1 = edge_info.edge.1 as usize;

        let v0 = Vec3::from_array(positions[i0]);
        let v1 = Vec3::from_array(positions[i1]);

        let n0 = edge_info.face_normals[0];
        let n1 = edge_info.face_normals[1];

        // Edge direction
        let edge_dir = (v1 - v0).normalize_or_zero();

        // Chamfer normal (average of face normals)
        let chamfer_normal = (n0 + n1).normalize_or_zero();

        // Create 4 new vertices for the chamfer quad
        let offset0 = n0 * distance;
        let offset1 = n1 * distance;

        let base_idx = new_positions.len() as u32;

        // Chamfer vertices
        new_positions.push((v0 - offset0).to_array());
        new_positions.push((v1 - offset0).to_array());
        new_positions.push((v1 - offset1).to_array());
        new_positions.push((v0 - offset1).to_array());

        for _ in 0..4 {
            new_normals.push([chamfer_normal.x, chamfer_normal.y, chamfer_normal.z]);
        }

        new_uvs.push([0.0, 0.0]);
        new_uvs.push([1.0, 0.0]);
        new_uvs.push([1.0, 1.0]);
        new_uvs.push([0.0, 1.0]);

        // Chamfer quad triangles
        new_indices.extend_from_slice(&[base_idx, base_idx + 1, base_idx + 2]);
        new_indices.extend_from_slice(&[base_idx, base_idx + 2, base_idx + 3]);
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, new_uvs);
    mesh.insert_indices(Indices::U32(new_indices));

    Ok(())
}

/// Apply smooth bevel to edges sharper than threshold
///
/// Creates multiple segments for a rounded appearance.
pub fn bevel_edges(
    mesh: &mut Mesh,
    radius: f32,
    segments: u32,
    angle_threshold: f32,
) -> Result<(), BevelError> {
    if segments == 1 {
        return chamfer_edges(mesh, radius, angle_threshold);
    }

    let edges = find_edges(mesh)?;

    let positions = match mesh.attribute(Mesh::ATTRIBUTE_POSITION) {
        Some(VertexAttributeValues::Float32x3(pos)) => pos.clone(),
        _ => return Err(BevelError::MissingAttribute("POSITION")),
    };

    let _normals = match mesh.attribute(Mesh::ATTRIBUTE_NORMAL) {
        Some(VertexAttributeValues::Float32x3(norm)) => norm.clone(),
        _ => return Err(BevelError::MissingAttribute("NORMAL")),
    };

    let uvs = match mesh.attribute(Mesh::ATTRIBUTE_UV_0) {
        Some(VertexAttributeValues::Float32x2(uv)) => uv.clone(),
        _ => vec![[0.0, 0.0]; positions.len()],
    };

    let old_indices = match mesh.indices() {
        Some(Indices::U32(idx)) => idx.clone(),
        Some(Indices::U16(idx)) => idx.iter().map(|&i| i as u32).collect(),
        None => return Err(BevelError::InvalidMesh),
    };

    // Find edges that need beveling
    let sharp_edges: Vec<&EdgeInfo> = edges
        .iter()
        .filter(|e| e.edge_angle > angle_threshold && e.face_normals.len() == 2)
        .collect();

    if sharp_edges.is_empty() {
        return Ok(());
    }

    let mut new_positions = positions.clone();
    let mut new_normals: Vec<[f32; 3]> = vec![[0.0, 1.0, 0.0]; positions.len()];
    let mut new_uvs = uvs.clone();
    let mut new_indices = old_indices.clone();

    for edge_info in &sharp_edges {
        let i0 = edge_info.edge.0 as usize;
        let i1 = edge_info.edge.1 as usize;

        let v0 = Vec3::from_array(positions[i0]);
        let v1 = Vec3::from_array(positions[i1]);

        let n0 = edge_info.face_normals[0];
        let n1 = edge_info.face_normals[1];

        // Generate bevel strip
        let base_idx = new_positions.len() as u32;

        for seg in 0..=segments {
            let t = seg as f32 / segments as f32;
            let angle = t * edge_info.edge_angle;

            // Interpolate normal
            let normal = (n0 * (1.0 - t) + n1 * t).normalize_or_zero();

            // Offset position
            let blend = (n0 * angle.cos() + n1 * angle.sin()).normalize_or_zero();
            let offset = blend * radius * (1.0 - angle.cos().abs() * 0.5);

            // Two vertices per segment (one at each end of edge)
            new_positions.push((v0 - offset).to_array());
            new_positions.push((v1 - offset).to_array());

            new_normals.push([normal.x, normal.y, normal.z]);
            new_normals.push([normal.x, normal.y, normal.z]);

            new_uvs.push([0.0, t]);
            new_uvs.push([1.0, t]);
        }

        // Generate triangles for bevel strip
        for seg in 0..segments {
            let idx = base_idx + seg * 2;
            new_indices.extend_from_slice(&[idx, idx + 2, idx + 1]);
            new_indices.extend_from_slice(&[idx + 1, idx + 2, idx + 3]);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, new_positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, new_normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, new_uvs);
    mesh.insert_indices(Indices::U32(new_indices));

    Ok(())
}

/// Find boundary edges (edges with only one adjacent face)
pub fn find_boundary_edges(mesh: &Mesh) -> Result<Vec<Edge>, BevelError> {
    let edges = find_edges(mesh)?;

    Ok(edges
        .into_iter()
        .filter(|e| e.face_indices.len() == 1)
        .map(|e| e.edge)
        .collect())
}

/// Check if mesh is watertight (no boundary edges)
pub fn is_watertight(mesh: &Mesh) -> Result<bool, BevelError> {
    let boundary = find_boundary_edges(mesh)?;
    Ok(boundary.is_empty())
}
