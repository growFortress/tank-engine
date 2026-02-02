use bevy::prelude::*;
use crate::components::WorldAABB;

// ============================================================================
// PENETRATION INFO
// ============================================================================

/// Informacja o penetracji (przenikaniu) dwóch obiektów
#[derive(Clone, Debug)]
pub struct PenetrationInfo {
    /// Normalna separacji (kierunek wypychania A od B)
    pub normal: Vec3,
    /// Głębokość penetracji [m]
    pub depth: f32,
    /// Punkt kontaktu (przybliżony)
    pub contact_point: Vec3,
}

// ============================================================================
// AABB vs AABB
// ============================================================================

/// Sprawdza czy dwa AABB się przecinają
pub fn aabb_intersects(a: &WorldAABB, b: &WorldAABB) -> bool {
    a.min.x <= b.max.x && a.max.x >= b.min.x &&
    a.min.y <= b.max.y && a.max.y >= b.min.y &&
    a.min.z <= b.max.z && a.max.z >= b.min.z
}

/// AABB vs AABB z informacją o penetracji
pub fn aabb_vs_aabb(a: &WorldAABB, b: &WorldAABB) -> Option<PenetrationInfo> {
    // Oblicz overlap na każdej osi
    let overlap_x = (a.max.x.min(b.max.x) - a.min.x.max(b.min.x)).max(0.0);
    let overlap_y = (a.max.y.min(b.max.y) - a.min.y.max(b.min.y)).max(0.0);
    let overlap_z = (a.max.z.min(b.max.z) - a.min.z.max(b.min.z)).max(0.0);

    // Jeśli którykolwiek overlap to 0, brak kolizji
    if overlap_x <= 0.0 || overlap_y <= 0.0 || overlap_z <= 0.0 {
        return None;
    }

    // Znajdź oś minimalnej penetracji
    let center_a = (a.min + a.max) * 0.5;
    let center_b = (b.min + b.max) * 0.5;
    let direction = center_a - center_b;

    let (normal, depth) = if overlap_x <= overlap_y && overlap_x <= overlap_z {
        (Vec3::new(direction.x.signum(), 0.0, 0.0), overlap_x)
    } else if overlap_y <= overlap_z {
        (Vec3::new(0.0, direction.y.signum(), 0.0), overlap_y)
    } else {
        (Vec3::new(0.0, 0.0, direction.z.signum()), overlap_z)
    };

    // Punkt kontaktu - środek obszaru przecięcia
    let contact_point = Vec3::new(
        (a.min.x.max(b.min.x) + a.max.x.min(b.max.x)) * 0.5,
        (a.min.y.max(b.min.y) + a.max.y.min(b.max.y)) * 0.5,
        (a.min.z.max(b.min.z) + a.max.z.min(b.max.z)) * 0.5,
    );

    Some(PenetrationInfo {
        normal,
        depth,
        contact_point,
    })
}

// ============================================================================
// OBB vs OBB (Separating Axis Theorem)
// ============================================================================

/// OBB vs OBB collision detection używając SAT
///
/// Zwraca informację o penetracji jeśli obiekty kolidują.
pub fn obb_vs_obb(
    transform_a: &Transform,
    half_extents_a: &Vec3,
    transform_b: &Transform,
    half_extents_b: &Vec3,
) -> Option<PenetrationInfo> {
    // Osie lokalne OBB A
    let axes_a = [
        transform_a.rotation * Vec3::X,
        transform_a.rotation * Vec3::Y,
        transform_a.rotation * Vec3::Z,
    ];

    // Osie lokalne OBB B
    let axes_b = [
        transform_b.rotation * Vec3::X,
        transform_b.rotation * Vec3::Y,
        transform_b.rotation * Vec3::Z,
    ];

    // Wektor między środkami
    let center_diff = transform_b.translation - transform_a.translation;

    // Testuj wszystkie 15 osi separacji:
    // 3 osie A + 3 osie B + 9 cross products
    let mut min_overlap = f32::MAX;
    let mut min_axis = Vec3::ZERO;

    // Testuj osie A
    for (_i, axis) in axes_a.iter().enumerate() {
        if let Some(overlap) = test_axis(*axis, &axes_a, half_extents_a, &axes_b, half_extents_b, center_diff) {
            if overlap < min_overlap {
                min_overlap = overlap;
                min_axis = *axis;
            }
        } else {
            return None; // Separating axis found
        }
    }

    // Testuj osie B
    for axis in axes_b.iter() {
        if let Some(overlap) = test_axis(*axis, &axes_a, half_extents_a, &axes_b, half_extents_b, center_diff) {
            if overlap < min_overlap {
                min_overlap = overlap;
                min_axis = *axis;
            }
        } else {
            return None;
        }
    }

    // Testuj cross products (9 osi)
    for axis_a in axes_a.iter() {
        for axis_b in axes_b.iter() {
            let cross = axis_a.cross(*axis_b);

            // Pomijamy zdegenerowane osie (równoległe)
            if cross.length_squared() < 1e-6 {
                continue;
            }

            let axis = cross.normalize();

            if let Some(overlap) = test_axis(axis, &axes_a, half_extents_a, &axes_b, half_extents_b, center_diff) {
                if overlap < min_overlap {
                    min_overlap = overlap;
                    min_axis = axis;
                }
            } else {
                return None;
            }
        }
    }

    // Kolizja! Upewnij się że normalna wskazuje od B do A
    if min_axis.dot(center_diff) > 0.0 {
        min_axis = -min_axis;
    }

    // Oblicz punkt kontaktu (przybliżenie - najbliższy punkt na OBB A)
    let contact_point = closest_point_on_obb(
        transform_b.translation,
        transform_a,
        half_extents_a,
    );

    Some(PenetrationInfo {
        normal: min_axis,
        depth: min_overlap,
        contact_point,
    })
}

/// Testuje pojedynczą oś separacji
fn test_axis(
    axis: Vec3,
    axes_a: &[Vec3; 3],
    half_extents_a: &Vec3,
    axes_b: &[Vec3; 3],
    half_extents_b: &Vec3,
    center_diff: Vec3,
) -> Option<f32> {
    // Projekcja half extents na oś
    let proj_a =
        (axes_a[0].dot(axis)).abs() * half_extents_a.x +
        (axes_a[1].dot(axis)).abs() * half_extents_a.y +
        (axes_a[2].dot(axis)).abs() * half_extents_a.z;

    let proj_b =
        (axes_b[0].dot(axis)).abs() * half_extents_b.x +
        (axes_b[1].dot(axis)).abs() * half_extents_b.y +
        (axes_b[2].dot(axis)).abs() * half_extents_b.z;

    // Projekcja odległości między środkami
    let dist = center_diff.dot(axis).abs();

    // Overlap
    let overlap = proj_a + proj_b - dist;

    if overlap > 0.0 {
        Some(overlap)
    } else {
        None // Separating axis
    }
}

/// Znajduje najbliższy punkt na OBB do podanego punktu
pub fn closest_point_on_obb(
    point: Vec3,
    obb_transform: &Transform,
    half_extents: &Vec3,
) -> Vec3 {
    // Transformuj punkt do local space OBB
    let local_point = obb_transform.rotation.inverse() * (point - obb_transform.translation);

    // Clamp do half extents
    let clamped = Vec3::new(
        local_point.x.clamp(-half_extents.x, half_extents.x),
        local_point.y.clamp(-half_extents.y, half_extents.y),
        local_point.z.clamp(-half_extents.z, half_extents.z),
    );

    // Transformuj z powrotem do world space
    obb_transform.rotation * clamped + obb_transform.translation
}

// ============================================================================
// SPHERE vs OBB
// ============================================================================

/// Sphere vs OBB collision
pub fn sphere_vs_obb(
    sphere_center: Vec3,
    sphere_radius: f32,
    obb_transform: &Transform,
    half_extents: &Vec3,
) -> Option<PenetrationInfo> {
    // Znajdź najbliższy punkt na OBB
    let closest = closest_point_on_obb(sphere_center, obb_transform, half_extents);

    // Sprawdź odległość
    let diff = sphere_center - closest;
    let dist_sq = diff.length_squared();

    if dist_sq > sphere_radius * sphere_radius {
        return None; // Brak kolizji
    }

    let dist = dist_sq.sqrt();

    // Normalna od OBB do sfery
    let normal = if dist > 1e-6 {
        diff / dist
    } else {
        // Sfera jest wewnątrz OBB - użyj najbliższej ściany
        Vec3::Y // Fallback
    };

    Some(PenetrationInfo {
        normal,
        depth: sphere_radius - dist,
        contact_point: closest,
    })
}

// ============================================================================
// POINT IN OBB
// ============================================================================

/// Sprawdza czy punkt jest wewnątrz OBB
pub fn point_in_obb(
    point: Vec3,
    obb_transform: &Transform,
    half_extents: &Vec3,
) -> bool {
    // Transformuj punkt do local space
    let local = obb_transform.rotation.inverse() * (point - obb_transform.translation);

    // Sprawdź czy wewnątrz
    local.x.abs() <= half_extents.x &&
    local.y.abs() <= half_extents.y &&
    local.z.abs() <= half_extents.z
}

// ============================================================================
// COMPOUND COLLIDER HELPERS
// ============================================================================

/// Sprawdza kolizję compound collider vs single box
pub fn compound_vs_box(
    compound_transform: &Transform,
    compound_shapes: &[(Vec3, Quat, Vec3)], // (offset, rotation, half_extents)
    box_transform: &Transform,
    box_half_extents: &Vec3,
) -> Option<PenetrationInfo> {
    let mut deepest: Option<PenetrationInfo> = None;

    for (offset, rotation, half_extents) in compound_shapes {
        // Oblicz world transform dla tego shape
        let shape_transform = Transform {
            translation: compound_transform.translation + compound_transform.rotation * *offset,
            rotation: compound_transform.rotation * *rotation,
            scale: Vec3::ONE,
        };

        if let Some(pen) = obb_vs_obb(&shape_transform, half_extents, box_transform, box_half_extents) {
            match &deepest {
                None => deepest = Some(pen),
                Some(current) if pen.depth > current.depth => deepest = Some(pen),
                _ => {}
            }
        }
    }

    deepest
}

// ============================================================================
// COLLISION RESPONSE HELPERS
// ============================================================================

/// Oblicza impuls kolizji dla dwóch ciał
pub fn compute_collision_impulse(
    velocity_a: Vec3,
    mass_a: f32,
    velocity_b: Vec3,
    mass_b: f32,
    normal: Vec3,
    restitution: f32,
) -> f32 {
    // Prędkość względna wzdłuż normalnej
    let rel_vel = velocity_a - velocity_b;
    let vel_along_normal = rel_vel.dot(normal);

    // Jeśli obiekty się oddalają, brak impulsu
    if vel_along_normal > 0.0 {
        return 0.0;
    }

    // Impuls: j = -(1 + e) * v_rel / (1/m_a + 1/m_b)
    let inv_mass_sum = if mass_a > 0.0 { 1.0 / mass_a } else { 0.0 }
                     + if mass_b > 0.0 { 1.0 / mass_b } else { 0.0 };

    if inv_mass_sum < 1e-8 {
        return 0.0; // Oba obiekty są statyczne
    }

    -(1.0 + restitution) * vel_along_normal / inv_mass_sum
}

/// Oblicza separację pozycyjną (position correction)
pub fn compute_position_correction(
    penetration_depth: f32,
    normal: Vec3,
    mass_a: f32,
    mass_b: f32,
    correction_percent: f32, // Typowo 0.2-0.8
    slop: f32, // Tolerancja penetracji (typowo 0.01)
) -> (Vec3, Vec3) {
    let correction = (penetration_depth - slop).max(0.0) * correction_percent;

    let inv_mass_a = if mass_a > 0.0 { 1.0 / mass_a } else { 0.0 };
    let inv_mass_b = if mass_b > 0.0 { 1.0 / mass_b } else { 0.0 };
    let inv_mass_sum = inv_mass_a + inv_mass_b;

    if inv_mass_sum < 1e-8 {
        return (Vec3::ZERO, Vec3::ZERO);
    }

    let correction_vec = normal * correction / inv_mass_sum;

    (
        correction_vec * inv_mass_a,  // Korekta dla A (w kierunku normalnej)
        -correction_vec * inv_mass_b, // Korekta dla B (przeciwny kierunek)
    )
}
