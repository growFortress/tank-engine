use bevy::prelude::*;
use crate::resources::MonteCassinoTerrain;
use crate::components::WorldAABB;

// ============================================================================
// RAYCAST HIT RESULT
// ============================================================================

/// Wynik raycasta
#[derive(Clone, Debug)]
pub struct RaycastHit {
    /// Punkt trafienia w world space
    pub point: Vec3,
    /// Normalna powierzchni w punkcie trafienia
    pub normal: Vec3,
    /// Odległość od origin do punktu trafienia
    pub distance: f32,
}

// ============================================================================
// HEIGHTFIELD RAYCAST
// ============================================================================

/// Raycast przeciwko heightfield (terrain)
///
/// Używa ray marching z binary search refinement dla dokładności.
/// Działa tylko dla rayów skierowanych w dół (direction.y < 0).
pub fn raycast_heightfield(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    terrain: &MonteCassinoTerrain,
) -> Option<RaycastHit> {
    // Normalizuj kierunek
    let dir = direction.normalize();

    // Optymalizacja: jeśli ray idzie w górę i zaczyna powyżej terenu, brak trafienia
    if dir.y > 0.0 {
        let ground_height = terrain.sample_height(origin.x, origin.z);
        if origin.y > ground_height {
            return None;
        }
    }

    // Ray marching z adaptive step (optimized for 500x500 map)
    let mut t = 0.0;
    let base_step = 0.5; // Bazowy krok [m] - increased for larger map

    // Sprawdź czy zaczynamy pod terenem
    let start_height = terrain.sample_height(origin.x, origin.z);
    if origin.y < start_height {
        // Już pod terenem - zwróć natychmiastowe trafienie
        return Some(RaycastHit {
            point: origin,
            normal: terrain.sample_normal(origin.x, origin.z),
            distance: 0.0,
        });
    }

    while t < max_distance {
        let point = origin + dir * t;
        let terrain_height = terrain.sample_height(point.x, point.z);

        if point.y <= terrain_height {
            // Trafienie! Refine z binary search dla dokładności
            let mut lo = (t - base_step).max(0.0);
            let mut hi = t;

            // 8 iteracji binary search = ~0.002m precyzja przy 0.3m step
            for _ in 0..8 {
                let mid = (lo + hi) * 0.5;
                let p = origin + dir * mid;
                let h = terrain.sample_height(p.x, p.z);

                if p.y <= h {
                    hi = mid;
                } else {
                    lo = mid;
                }
            }

            let hit_distance = hi;
            let hit_point = origin + dir * hit_distance;
            let normal = terrain.sample_normal(hit_point.x, hit_point.z);

            return Some(RaycastHit {
                point: hit_point,
                normal,
                distance: hit_distance,
            });
        }

        // Adaptive step - większy krok gdy daleko od terenu (optimized for larger map)
        let height_diff = point.y - terrain_height;
        let step = if height_diff > 5.0 {
            base_step * 3.0
        } else if height_diff > 2.0 {
            base_step * 2.0
        } else {
            base_step
        };

        t += step;
    }

    None
}

// ============================================================================
// AABB RAYCAST (Slab Method)
// ============================================================================

/// Raycast przeciwko AABB (axis-aligned bounding box)
///
/// Używa slab method - szybki i dokładny.
pub fn raycast_aabb(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    aabb: &WorldAABB,
) -> Option<RaycastHit> {
    raycast_aabb_minmax(origin, direction, max_distance, aabb.min, aabb.max)
}

/// Raycast przeciwko AABB zdefiniowanego przez min/max
pub fn raycast_aabb_minmax(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    aabb_min: Vec3,
    aabb_max: Vec3,
) -> Option<RaycastHit> {
    let dir = direction.normalize();

    // Inverse direction (z zabezpieczeniem przed division by zero)
    let inv_dir = Vec3::new(
        if dir.x.abs() > 1e-8 { 1.0 / dir.x } else { f32::MAX * dir.x.signum() },
        if dir.y.abs() > 1e-8 { 1.0 / dir.y } else { f32::MAX * dir.y.signum() },
        if dir.z.abs() > 1e-8 { 1.0 / dir.z } else { f32::MAX * dir.z.signum() },
    );

    // Oblicz t dla każdej płaszczyzny
    let t1 = (aabb_min - origin) * inv_dir;
    let t2 = (aabb_max - origin) * inv_dir;

    // Min/max dla każdej osi
    let tmin = t1.min(t2);
    let tmax = t1.max(t2);

    // Znajdź największy tmin i najmniejszy tmax
    let t_enter = tmin.x.max(tmin.y).max(tmin.z);
    let t_exit = tmax.x.min(tmax.y).min(tmax.z);

    // Sprawdź czy jest trafienie
    if t_enter > t_exit || t_exit < 0.0 || t_enter > max_distance {
        return None;
    }

    // Wybierz punkt wejścia lub wyjścia
    let t = if t_enter > 0.0 { t_enter } else { t_exit };

    if t > max_distance {
        return None;
    }

    let hit_point = origin + dir * t;
    let normal = compute_aabb_normal(hit_point, aabb_min, aabb_max);

    Some(RaycastHit {
        point: hit_point,
        normal,
        distance: t,
    })
}

/// Oblicza normalną na powierzchni AABB
fn compute_aabb_normal(point: Vec3, aabb_min: Vec3, aabb_max: Vec3) -> Vec3 {
    let epsilon = 0.001;

    // Znajdź która ściana jest najbliżej punktu
    let distances = [
        (point.x - aabb_min.x).abs(), // -X
        (point.x - aabb_max.x).abs(), // +X
        (point.y - aabb_min.y).abs(), // -Y
        (point.y - aabb_max.y).abs(), // +Y
        (point.z - aabb_min.z).abs(), // -Z
        (point.z - aabb_max.z).abs(), // +Z
    ];

    let normals = [
        Vec3::NEG_X,
        Vec3::X,
        Vec3::NEG_Y,
        Vec3::Y,
        Vec3::NEG_Z,
        Vec3::Z,
    ];

    // Znajdź minimalną odległość
    let mut min_idx = 0;
    let mut min_dist = distances[0];

    for (i, &dist) in distances.iter().enumerate().skip(1) {
        if dist < min_dist {
            min_dist = dist;
            min_idx = i;
        }
    }

    if min_dist < epsilon {
        normals[min_idx]
    } else {
        // Fallback - normalna w kierunku od środka
        let center = (aabb_min + aabb_max) * 0.5;
        (point - center).normalize_or_zero()
    }
}

// ============================================================================
// OBB RAYCAST (Oriented Bounding Box)
// ============================================================================

/// Raycast przeciwko OBB (oriented bounding box)
///
/// Transformuje ray do local space OBB i używa AABB raycast.
pub fn raycast_obb(
    origin: Vec3,
    direction: Vec3,
    max_distance: f32,
    obb_transform: &Transform,
    half_extents: Vec3,
) -> Option<RaycastHit> {
    // Transformuj ray do local space OBB
    let inv_rotation = obb_transform.rotation.inverse();
    let local_origin = inv_rotation * (origin - obb_transform.translation);
    let local_dir = inv_rotation * direction;

    // AABB w local space
    let local_min = -half_extents;
    let local_max = half_extents;

    // Raycast w local space
    if let Some(local_hit) = raycast_aabb_minmax(local_origin, local_dir, max_distance, local_min, local_max) {
        // Transformuj wynik z powrotem do world space
        let world_point = obb_transform.rotation * local_hit.point + obb_transform.translation;
        let world_normal = obb_transform.rotation * local_hit.normal;

        Some(RaycastHit {
            point: world_point,
            normal: world_normal.normalize(),
            distance: local_hit.distance,
        })
    } else {
        None
    }
}

// ============================================================================
// MULTI-RAYCAST (batch dla zawieszenia)
// ============================================================================

/// Wynik multi-raycasta
pub struct MultiRaycastResult {
    pub hits: Vec<Option<RaycastHit>>,
    pub hit_count: usize,
}

/// Wykonuje wiele raycastów naraz (optymalizacja dla zawieszenia)
pub fn multi_raycast_heightfield(
    rays: &[(Vec3, Vec3)], // [(origin, direction), ...]
    max_distance: f32,
    terrain: &MonteCassinoTerrain,
) -> MultiRaycastResult {
    let mut hits = Vec::with_capacity(rays.len());
    let mut hit_count = 0;

    for (origin, direction) in rays {
        let hit = raycast_heightfield(*origin, *direction, max_distance, terrain);
        if hit.is_some() {
            hit_count += 1;
        }
        hits.push(hit);
    }

    MultiRaycastResult { hits, hit_count }
}

// ============================================================================
// SPHERE CAST (dla collision avoidance)
// ============================================================================

/// Prosty sphere cast (ray z promieniem)
///
/// Używa ray + sphere intersection.
pub fn spherecast_heightfield(
    origin: Vec3,
    direction: Vec3,
    radius: f32,
    max_distance: f32,
    terrain: &MonteCassinoTerrain,
) -> Option<RaycastHit> {
    // Dla uproszczenia: raycast + offset o radius
    let dir = direction.normalize();

    // Przesuń origin w dół o radius
    let adjusted_origin = origin - Vec3::Y * radius;

    if let Some(mut hit) = raycast_heightfield(adjusted_origin, dir, max_distance, terrain) {
        // Przesuń hit point w górę o radius wzdłuż normalnej
        hit.point += hit.normal * radius;
        hit.distance = (hit.point - origin).length();
        Some(hit)
    } else {
        None
    }
}
