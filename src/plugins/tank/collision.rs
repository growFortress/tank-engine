use bevy::prelude::*;
use crate::components::{
    Tank, RigidBody6DOF, CompoundCollider, BoxCollider,
    WorldAABB, StaticBody, Destructible, DestructionEvent,
};
use crate::physics::collision::{obb_vs_obb, PenetrationInfo};

// ============================================================================
// COLLISION CONSTANTS
// ============================================================================

/// Współczynnik odbicia (restitution) - niski dla ciężkiego czołgu
const RESTITUTION: f32 = 0.08;
/// Współczynnik tarcia przy kontakcie
const FRICTION_COEFFICIENT: f32 = 0.5;
/// Siła korekcji pozycji (0.2-1.0) - WYSOKA żeby nie wchodzić w obiekty
const POSITION_CORRECTION_PERCENT: f32 = 0.95;
/// Tolerancja penetracji [m] - BARDZO MAŁA
const PENETRATION_SLOP: f32 = 0.005;
/// Minimalna prędkość do przetwarzania kolizji (używana w CCD)
#[allow(dead_code)]
const MIN_COLLISION_VELOCITY: f32 = 0.02;
/// Maksymalna korekcja pozycji na klatkę [m]
const MAX_POSITION_CORRECTION: f32 = 1.0;
/// Iteracje korekcji pozycji dla lepszego rozwiązania
const POSITION_CORRECTION_ITERATIONS: u32 = 3;

// ============================================================================
// AABB UPDATE SYSTEM
// ============================================================================

/// Aktualizuje WorldAABB dla obiektów z BoxCollider
pub fn update_world_aabbs(
    mut box_query: Query<(&Transform, &BoxCollider, &mut WorldAABB), Without<CompoundCollider>>,
) {
    for (transform, collider, mut aabb) in box_query.iter_mut() {
        // Środek collidera w world space
        let center = transform.translation + transform.rotation * collider.offset;

        // Konserwatywne AABB dla obróconego boxa
        // Bierzemy maksymalny zasięg po rotacji
        let rotated_x = transform.rotation * Vec3::new(collider.half_extents.x, 0.0, 0.0);
        let rotated_y = transform.rotation * Vec3::new(0.0, collider.half_extents.y, 0.0);
        let rotated_z = transform.rotation * Vec3::new(0.0, 0.0, collider.half_extents.z);

        let extent = Vec3::new(
            rotated_x.x.abs() + rotated_y.x.abs() + rotated_z.x.abs(),
            rotated_x.y.abs() + rotated_y.y.abs() + rotated_z.y.abs(),
            rotated_x.z.abs() + rotated_y.z.abs() + rotated_z.z.abs(),
        );

        aabb.min = center - extent;
        aabb.max = center + extent;
    }
}

/// Aktualizuje WorldAABB dla obiektów z CompoundCollider (czołg)
pub fn update_compound_aabbs(
    mut query: Query<(&Transform, &CompoundCollider, &mut WorldAABB)>,
) {
    for (transform, compound, mut aabb) in query.iter_mut() {
        let mut min = Vec3::splat(f32::MAX);
        let mut max = Vec3::splat(f32::MIN);

        for shape in &compound.shapes {
            // World position tego shape
            let shape_center = transform.translation + transform.rotation * shape.offset;
            let shape_rotation = transform.rotation * shape.rotation;

            // Extent po rotacji
            let he = shape.collider.half_extents;
            let rotated_x = shape_rotation * Vec3::new(he.x, 0.0, 0.0);
            let rotated_y = shape_rotation * Vec3::new(0.0, he.y, 0.0);
            let rotated_z = shape_rotation * Vec3::new(0.0, 0.0, he.z);

            let extent = Vec3::new(
                rotated_x.x.abs() + rotated_y.x.abs() + rotated_z.x.abs(),
                rotated_x.y.abs() + rotated_y.y.abs() + rotated_z.y.abs(),
                rotated_x.z.abs() + rotated_y.z.abs() + rotated_z.z.abs(),
            );

            min = min.min(shape_center - extent);
            max = max.max(shape_center + extent);
        }

        aabb.min = min;
        aabb.max = max;
    }
}

// ============================================================================
// COLLISION DETECTION & RESPONSE
// ============================================================================

/// System wykrywania i rozwiązywania kolizji czołg vs statyczne obiekty
///
/// Ulepszony system z:
/// - Iteracyjną korekcją pozycji (wielokrotne przejścia dla lepszego rozwiązania)
/// - Natychmiastową korekcją po każdej kolizji (nie czeka na koniec)
/// - Blokowaniem prędkości w kierunku penetracji
/// - Generowaniem momentu obrotowego przy kolizji
pub fn detect_and_resolve_collisions(
    mut destruction_events: EventWriter<DestructionEvent>,
    mut tank_query: Query<(
        Entity,
        &mut Transform,
        &CompoundCollider,
        &mut RigidBody6DOF,
    ), With<Tank>>,
    static_query: Query<(
        Entity,
        &Transform,
        &BoxCollider,
        &WorldAABB,
        Option<&Destructible>,
    ), (With<StaticBody>, Without<Tank>)>,
) {
    for (_tank_entity, mut tank_transform, tank_collider, mut rb) in tank_query.iter_mut() {
        // === ITERACYJNA KOREKCJA POZYCJI ===
        // Wielokrotne przejścia zapewniają że czołg nie wchodzi w obiekty
        for _iteration in 0..POSITION_CORRECTION_ITERATIONS {
            let mut had_collision = false;

            for (static_entity, static_transform, static_collider, static_aabb, destructible) in static_query.iter() {
                // === BROAD PHASE: szybki test AABB ===
                // Oblicz AABB czołgu na bieżąco (po ewentualnej korekcji)
                let tank_center = tank_transform.translation;
                let tank_extent = Vec3::splat(3.5); // Konserwatywny rozmiar

                if tank_center.x + tank_extent.x < static_aabb.min.x ||
                   tank_center.x - tank_extent.x > static_aabb.max.x ||
                   tank_center.y + tank_extent.y < static_aabb.min.y ||
                   tank_center.y - tank_extent.y > static_aabb.max.y ||
                   tank_center.z + tank_extent.z < static_aabb.min.z ||
                   tank_center.z - tank_extent.z > static_aabb.max.z {
                    continue;
                }

                // === NARROW PHASE: OBB vs OBB dla każdego shape ===
                for shape in &tank_collider.shapes {
                    let shape_center = tank_transform.translation + tank_transform.rotation * shape.offset;
                    let shape_transform = Transform {
                        translation: shape_center,
                        rotation: tank_transform.rotation * shape.rotation,
                        scale: Vec3::ONE,
                    };

                    if let Some(penetration) = obb_vs_obb(
                        &shape_transform,
                        &shape.collider.half_extents,
                        static_transform,
                        &static_collider.half_extents,
                    ) {
                        had_collision = true;

                        // === SPRAWDŹ NISZCZALNOŚĆ ===
                        if let Some(dest) = destructible {
                            let impact_force = rb.velocity.length() * rb.mass;
                            if impact_force > dest.destruction_threshold {
                                destruction_events.send(DestructionEvent {
                                    entity: static_entity,
                                    position: static_transform.translation,
                                    impact_velocity: rb.velocity,
                                    impact_force,
                                });
                                rb.velocity *= 0.85;
                                continue;
                            }
                        }

                        // === NATYCHMIASTOWA KOREKCJA POZYCJI ===
                        // Wypychamy czołg z obiektu ZANIM przetwarzamy prędkość
                        if penetration.depth > PENETRATION_SLOP {
                            let correction = penetration.normal
                                * (penetration.depth - PENETRATION_SLOP + 0.02) // Dodatkowy margines
                                * POSITION_CORRECTION_PERCENT;
                            tank_transform.translation += correction.clamp_length_max(MAX_POSITION_CORRECTION);
                        }

                        // === BLOKOWANIE PRĘDKOŚCI W KIERUNKU PENETRACJI ===
                        let vel_along_normal = rb.velocity.dot(penetration.normal);
                        if vel_along_normal < 0.0 {
                            // Usuń składową prędkości wchodzącą w obiekt
                            rb.velocity -= penetration.normal * vel_along_normal;

                            // Dodaj małe odbicie
                            rb.velocity += penetration.normal * (-vel_along_normal * RESTITUTION);

                            // === TARCIE POWIERZCHNIOWE ===
                            let tangent_vel = rb.velocity - penetration.normal * rb.velocity.dot(penetration.normal);
                            let tangent_speed = tangent_vel.length();

                            if tangent_speed > 0.05 {
                                let friction_reduction = (FRICTION_COEFFICIENT * (-vel_along_normal).abs()).min(tangent_speed * 0.5);
                                rb.velocity -= tangent_vel.normalize_or_zero() * friction_reduction;
                            }

                            // === MOMENT OBROTOWY ===
                            let contact_offset = penetration.contact_point - tank_transform.translation;
                            let impact_force_vec = penetration.normal * (-vel_along_normal) * rb.mass * 0.1;
                            let collision_torque = contact_offset.cross(impact_force_vec) * rb.inv_inertia * 0.2;
                            rb.angular_velocity += collision_torque;

                            // Tłumienie oscylacji
                            rb.angular_velocity *= 0.90;
                        }
                    }
                }
            }

            // Jeśli nie było kolizji w tej iteracji, zakończ wcześniej
            if !had_collision {
                break;
            }
        }
    }
}

// ============================================================================
// CONTINUOUS COLLISION DETECTION (CCD)
// ============================================================================

/// Prędkość powyżej której włączamy CCD [m/s]
const CCD_VELOCITY_THRESHOLD: f32 = 8.0;

/// System predykcji kolizji dla szybko poruszających się obiektów
/// Zapobiega "tunelowaniu" przez cienkie ściany przy dużych prędkościach
pub fn predict_and_prevent_tunneling(
    time: Res<Time>,
    mut tank_query: Query<(
        &mut Transform,
        &CompoundCollider,
        &mut RigidBody6DOF,
    ), With<Tank>>,
    static_query: Query<(
        &Transform,
        &BoxCollider,
        &WorldAABB,
    ), (With<StaticBody>, Without<Tank>)>,
) {
    let dt = time.delta_secs();
    if dt < 0.0001 {
        return;
    }

    for (mut tank_transform, tank_collider, mut rb) in tank_query.iter_mut() {
        let speed = rb.velocity.length();

        // Tylko dla szybkich obiektów
        if speed < CCD_VELOCITY_THRESHOLD {
            continue;
        }

        // Przewidywana pozycja w następnej klatce
        let predicted_translation = tank_transform.translation + rb.velocity * dt;
        let _movement_dir = rb.velocity.normalize_or_zero();
        let movement_dist = speed * dt;

        // Sprawdź kolizję na ścieżce ruchu
        for (static_transform, static_collider, static_aabb) in static_query.iter() {
            // Szybkie odrzucenie - czy AABB ścieżki przecina obiekt?
            let path_min = tank_transform.translation.min(predicted_translation) - Vec3::splat(3.0);
            let path_max = tank_transform.translation.max(predicted_translation) + Vec3::splat(3.0);

            if path_max.x < static_aabb.min.x || path_min.x > static_aabb.max.x ||
               path_max.y < static_aabb.min.y || path_min.y > static_aabb.max.y ||
               path_max.z < static_aabb.min.z || path_min.z > static_aabb.max.z {
                continue;
            }

            // Sweep test - sprawdź kilka punktów wzdłuż ścieżki
            let steps = ((movement_dist / 0.5).ceil() as usize).max(2).min(10);

            for step in 1..=steps {
                let t = step as f32 / steps as f32;
                let test_pos = tank_transform.translation + rb.velocity * dt * t;

                // Testuj główny shape czołgu
                for shape in &tank_collider.shapes {
                    let shape_transform = Transform {
                        translation: test_pos + tank_transform.rotation * shape.offset,
                        rotation: tank_transform.rotation * shape.rotation,
                        scale: Vec3::ONE,
                    };

                    if let Some(penetration) = obb_vs_obb(
                        &shape_transform,
                        &shape.collider.half_extents,
                        static_transform,
                        &static_collider.half_extents,
                    ) {
                        // Kolizja wykryta na ścieżce!
                        // Zatrzymaj przed obiektem

                        // Oblicz bezpieczną pozycję (cofnij do punktu przed kolizją)
                        let safe_t = ((step - 1) as f32 / steps as f32).max(0.0);
                        let safe_pos = tank_transform.translation + rb.velocity * dt * safe_t;

                        // Ustaw bezpieczną pozycję
                        tank_transform.translation = safe_pos;

                        // Odbij prędkość
                        let vel_along_normal = rb.velocity.dot(penetration.normal);
                        if vel_along_normal < 0.0 {
                            rb.velocity += penetration.normal * (-vel_along_normal * (1.0 + RESTITUTION));

                            // Tłumienie przy zderzeniu z dużą prędkością
                            rb.velocity *= 0.8;
                            rb.angular_velocity *= 0.85;
                        }

                        // Przerwij sweep test dla tego obiektu
                        break;
                    }
                }
            }
        }
    }
}

// ============================================================================
// CORNER COLLISION HANDLING
// ============================================================================

/// Oblicza lepszą normalną kolizji dla narożników
/// Standardowa SAT może dawać niestabilne normalne przy narożnikach
#[allow(dead_code)]
fn compute_corner_adjusted_normal(
    penetration: &PenetrationInfo,
    tank_velocity: Vec3,
    contact_point: Vec3,
    object_center: Vec3,
) -> Vec3 {
    // Wektor od środka obiektu do punktu kontaktu
    let to_contact = (contact_point - object_center).normalize_or_zero();

    // Wektor ruchu czołgu
    let move_dir = tank_velocity.normalize_or_zero();

    // Jeśli normalna SAT jest prawie prostopadła do ruchu, może być niestabilna
    let normal_dot_move = penetration.normal.dot(move_dir).abs();

    if normal_dot_move < 0.3 {
        // Prawdopodobnie kolizja z narożnikiem
        // Użyj wektora od środka obiektu jako normalnej
        let adjusted = (to_contact * 0.6 + penetration.normal * 0.4).normalize_or_zero();

        // Upewnij się że normalna odpycha czołg (przeciwna do ruchu)
        if adjusted.dot(move_dir) > 0.0 {
            -adjusted
        } else {
            adjusted
        }
    } else {
        penetration.normal
    }
}

// ============================================================================
// DESTRUCTION SYSTEM
// ============================================================================

/// Obsługuje zniszczenie obiektów
pub fn handle_destruction(
    mut commands: Commands,
    mut destruction_events: EventReader<DestructionEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for event in destruction_events.read() {
        // Usuń zniszczony obiekt
        if let Some(entity_commands) = commands.get_entity(event.entity) {
            entity_commands.despawn_recursive();
        }

        // Spawn gruzu
        spawn_debris(
            &mut commands,
            &mut meshes,
            &mut materials,
            event.position,
            event.impact_velocity,
        );

        // Debug info
        #[cfg(debug_assertions)]
        println!(
            "Object destroyed at {:?}, impact force: {:.0} N",
            event.position, event.impact_force
        );
    }
}

/// Tworzy gruz po zniszczeniu obiektu
fn spawn_debris(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    _impact_velocity: Vec3,
) {
    let debris_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.42, 0.38),
        perceptual_roughness: 0.95,
        ..default()
    });

    let debris_count = 4;
    let spread = 2.5;

    for i in 0..debris_count {
        // Pseudo-random offset
        let angle = (i as f32 * 1.618 * std::f32::consts::TAU) % std::f32::consts::TAU;
        let dist = 0.5 + (i as f32 * 0.7) % spread;

        let offset = Vec3::new(
            angle.cos() * dist,
            0.3 + (i as f32 * 0.4) % 1.0,
            angle.sin() * dist,
        );

        // Rozmiar kawałka gruzu
        let size = Vec3::new(
            0.4 + (i as f32 * 0.3) % 0.6,
            0.3 + (i as f32 * 0.2) % 0.4,
            0.35 + (i as f32 * 0.25) % 0.5,
        );

        // Losowa rotacja
        let rotation = Quat::from_euler(
            EulerRot::XYZ,
            (i as f32 * 0.7) % 1.0,
            (i as f32 * 1.3) % 1.0,
            (i as f32 * 0.5) % 1.0,
        );

        commands.spawn((
            Name::new("Debris"),
            Mesh3d(meshes.add(Cuboid::new(size.x, size.y, size.z))),
            MeshMaterial3d(debris_mat.clone()),
            Transform::from_translation(position + offset).with_rotation(rotation),
        ));
    }
}

// ============================================================================
// DEBUG VISUALIZATION
// ============================================================================

/// Debug: rysuje AABB colliderów
#[allow(dead_code)]
pub fn debug_draw_aabbs(
    tank_query: Query<&WorldAABB, With<Tank>>,
    static_query: Query<&WorldAABB, With<StaticBody>>,
    mut gizmos: Gizmos,
) {
    // AABB czołgu - zielony
    for aabb in tank_query.iter() {
        draw_aabb_gizmo(&mut gizmos, aabb, Color::srgb(0.0, 1.0, 0.0));
    }

    // AABB obiektów statycznych - niebieski
    for aabb in static_query.iter() {
        draw_aabb_gizmo(&mut gizmos, aabb, Color::srgb(0.0, 0.5, 1.0));
    }
}

#[allow(dead_code)]
fn draw_aabb_gizmo(gizmos: &mut Gizmos, aabb: &WorldAABB, color: Color) {
    let center = (aabb.min + aabb.max) * 0.5;
    let size = aabb.max - aabb.min;

    gizmos.cuboid(
        Transform::from_translation(center).with_scale(size),
        color,
    );
}
