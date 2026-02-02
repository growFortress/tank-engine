use bevy::prelude::*;
use crate::components::{Tank, RigidBody6DOF, RaycastSuspension};
use crate::resources::MonteCassinoTerrain;
use crate::physics::raycast_heightfield;

/// System zawieszenia opartego na raycastach
///
/// Dla każdego punktu zawieszenia:
/// 1. Rzuca ray w dół (local -Y)
/// 2. Oblicza kompresję sprężyny
/// 3. Aplikuje siłę sprężyny + tłumienia do RigidBody6DOF
pub fn update_raycast_suspension(
    time: Res<Time>,
    terrain: Res<MonteCassinoTerrain>,
    mut query: Query<(
        &Transform,
        &mut RigidBody6DOF,
        &mut RaycastSuspension,
    ), With<Tank>>,
) {
    let dt = time.delta_secs();
    if dt < 0.0001 {
        return;
    }

    for (transform, mut rb, mut suspension) in query.iter_mut() {
        let center_of_mass = transform.translation;

        // Extract suspension parameters before iterating (borrow checker)
        let max_length = suspension.max_length;
        let rest_length = suspension.rest_length;
        let spring_strength = suspension.spring_strength;
        let damper_strength = suspension.damper_strength;
        let compression_mult = suspension.compression_damping_mult;
        let rebound_mult = suspension.rebound_damping_mult;

        for point in suspension.suspension_points.iter_mut() {
            // Transformuj lokalną pozycję punktu do world space
            let world_pos = transform.transform_point(point.local_position);

            // Kierunek raycasta - w dół względem kadłuba (local -Y)
            let ray_dir = -transform.up();

            // Raycast do terenu
            if let Some(hit) = raycast_heightfield(
                world_pos,
                *ray_dir, // Dereference Dir3 to Vec3
                max_length,
                &terrain,
            ) {
                point.grounded = true;
                point.contact_normal = hit.normal;
                point.contact_point = hit.point;

                let current_length = hit.distance;

                // Kompresja sprężyny (dodatnia gdy ściśnięta)
                let compression = rest_length - current_length;

                // Prędkość sprężyny (dla tłumienia)
                // Dodatnia gdy sprężyna się ściska
                let spring_velocity = (point.last_length - current_length) / dt;
                point.last_length = current_length;

                // Siła sprężyny: F = k * x
                let mut spring_force = compression * spring_strength;

                // === HARD STOP - zapobiega wchodzeniu w teren ===
                // Minimalna dopuszczalna odległość (poniżej tej wartości = w terenie)
                let min_clearance = 0.15; // Track bottom jest ~0.16m poniżej suspension point
                if current_length < min_clearance {
                    // Czołg jest w terenie! Mocna siła wypychająca
                    let penetration = min_clearance - current_length;
                    // Nieliniowa siła: rośnie wykładniczo gdy penetracja rośnie
                    let hard_stop_force = penetration * spring_strength * 5.0;
                    spring_force += hard_stop_force;

                    // Dodatkowo: jeśli prędkość jest w dół, zatrzymaj ją
                    let vertical_vel = rb.velocity.y;
                    if vertical_vel < 0.0 {
                        // Impuls przeciwny do ruchu w dół
                        rb.velocity.y *= 0.5; // Tłumienie przy penetracji
                    }
                }

                // Siła tłumienia: F = c * v
                // Asymetryczne tłumienie: różne współczynniki dla kompresji i odbicia
                let damping_mult = if spring_velocity > 0.0 {
                    compression_mult // Kompresja (sprężyna się ściska)
                } else {
                    rebound_mult     // Odbicie (sprężyna się rozciąga)
                };
                let damper_force = spring_velocity * damper_strength * damping_mult;

                // Całkowita siła (tylko push, nie pull)
                let total_force = (spring_force + damper_force).max(0.0);
                point.current_force = total_force;

                // Aplikuj siłę wzdłuż normalnej kontaktu
                // Siła działa w punkcie kontaktu koła z terenem
                let force_vec = hit.normal * total_force;
                rb.apply_force_at_point(force_vec, world_pos, center_of_mass);
            } else {
                // Brak kontaktu z terenem - sprawdź czy może jesteśmy PONIŻEJ terenu
                // Użyj bezpośredniego sprawdzenia wysokości
                let terrain_height = terrain.sample_height(world_pos.x, world_pos.z);
                if world_pos.y < terrain_height + 0.3 {
                    // Jesteśmy blisko/poniżej terenu ale raycast nie trafił
                    // To może się zdarzyć na stromych zboczach
                    point.grounded = true;
                    let penetration = terrain_height + 0.3 - world_pos.y;
                    let emergency_force = penetration.max(0.0) * spring_strength * 3.0;
                    let normal = terrain.sample_normal(world_pos.x, world_pos.z);
                    point.contact_normal = normal;
                    point.contact_point = Vec3::new(world_pos.x, terrain_height, world_pos.z);
                    point.current_force = emergency_force;

                    let force_vec = normal * emergency_force;
                    rb.apply_force_at_point(force_vec, world_pos, center_of_mass);
                } else {
                    // Faktycznie w powietrzu
                    point.grounded = false;
                    point.last_length = max_length;
                    point.current_force = 0.0;
                }
            }
        }
    }
}

/// Debug system - rysuje raycasty zawieszenia (opcjonalny)
#[allow(dead_code)]
pub fn debug_draw_suspension(
    query: Query<(&Transform, &RaycastSuspension), With<Tank>>,
    mut gizmos: Gizmos,
) {
    for (transform, suspension) in query.iter() {
        for point in suspension.suspension_points.iter() {
            let world_pos = transform.transform_point(point.local_position);
            let ray_dir = -transform.up();

            // Kolor zależy od kontaktu
            let color = if point.grounded {
                // Gradient od zielonego (mała siła) do czerwonego (duża siła)
                let force_ratio = (point.current_force / 100000.0).clamp(0.0, 1.0);
                Color::srgb(force_ratio, 1.0 - force_ratio, 0.0)
            } else {
                Color::srgb(0.5, 0.5, 0.5) // Szary gdy w powietrzu
            };

            // Rysuj ray
            let ray_end = if point.grounded {
                point.contact_point
            } else {
                world_pos + *ray_dir * suspension.max_length
            };

            gizmos.line(world_pos, ray_end, color);

            // Rysuj punkt kontaktu
            if point.grounded {
                gizmos.sphere(
                    Isometry3d::from_translation(point.contact_point),
                    0.05,
                    color,
                );
            }
        }
    }
}
