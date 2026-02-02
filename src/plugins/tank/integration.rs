use bevy::prelude::*;
use crate::components::RigidBody6DOF;

/// Integracja fizyki - Semi-implicit Euler
///
/// Aktualizuje pozycję i rotację na podstawie prędkości i sił.
/// To jest JEDYNY system który modyfikuje Transform dla obiektów z RigidBody6DOF.
pub fn integrate_rigid_body(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut RigidBody6DOF)>,
) {
    let dt = time.delta_secs();
    if dt < 0.0001 || dt > 0.1 {
        // Zbyt mały lub zbyt duży dt - pomiń
        return;
    }

    for (mut transform, mut rb) in query.iter_mut() {
        if !rb.is_dynamic {
            rb.clear_forces();
            continue;
        }

        // === INTEGRACJA PRĘDKOŚCI (Semi-implicit Euler) ===
        // Najpierw aktualizuj prędkość, potem pozycję
        // v(t+dt) = v(t) + a * dt
        // x(t+dt) = x(t) + v(t+dt) * dt

        // Prędkość liniowa
        let acceleration = rb.force * rb.inv_mass;
        rb.velocity += acceleration * dt;

        // Prędkość kątowa
        let angular_acceleration = rb.torque * rb.inv_inertia;
        rb.angular_velocity += angular_acceleration * dt;

        // === DAMPING (tłumienie) ===
        // Exponential damping: v *= (1 - d)^dt ≈ v *= 1 - d*dt dla małych dt
        let linear_damp = 1.0 - rb.linear_damping * dt;
        let angular_damp = 1.0 - rb.angular_damping * dt;
        rb.velocity *= linear_damp;
        rb.angular_velocity *= angular_damp;

        // === APPLY LOCKS ===
        rb.apply_locks();

        // === CLAMP VELOCITIES ===
        // Ograniczenie maksymalnych prędkości dla stabilności
        let max_linear_speed = 30.0; // m/s (~108 km/h)
        let max_angular_speed = 5.0; // rad/s (~286 deg/s)

        if rb.velocity.length_squared() > max_linear_speed * max_linear_speed {
            rb.velocity = rb.velocity.normalize() * max_linear_speed;
        }

        if rb.angular_velocity.length_squared() > max_angular_speed * max_angular_speed {
            rb.angular_velocity = rb.angular_velocity.normalize() * max_angular_speed;
        }

        // === INTEGRACJA POZYCJI ===
        transform.translation += rb.velocity * dt;

        // === INTEGRACJA ROTACJI ===
        // Quaternion integration:
        // q(t+dt) = q(t) + 0.5 * ω * q(t) * dt
        // gdzie ω to quaternion z angular_velocity: (ω.x, ω.y, ω.z, 0)
        let omega = rb.angular_velocity;

        // Metoda 1: Prosty quaternion integration
        let omega_quat = Quat::from_xyzw(
            omega.x * dt * 0.5,
            omega.y * dt * 0.5,
            omega.z * dt * 0.5,
            0.0,
        );

        // q' = q + dq, gdzie dq = 0.5 * omega_quat * q
        let dq = omega_quat * transform.rotation;
        transform.rotation = Quat::from_xyzw(
            transform.rotation.x + dq.x,
            transform.rotation.y + dq.y,
            transform.rotation.z + dq.z,
            transform.rotation.w + dq.w,
        ).normalize();

        // === ZERO OUT SMALL VALUES ===
        // Zapobiega driftowi przy bardzo małych prędkościach
        if rb.velocity.length_squared() < 0.0001 {
            rb.velocity = Vec3::ZERO;
        }
        if rb.angular_velocity.length_squared() < 0.00001 {
            rb.angular_velocity = Vec3::ZERO;
        }

        // === CLEAR FORCE ACCUMULATORS ===
        rb.clear_forces();
    }
}

/// Substep integration dla większej stabilności (opcjonalne)
#[allow(dead_code)]
pub fn integrate_rigid_body_substeps(
    time: Res<Time>,
    mut query: Query<(&mut Transform, &mut RigidBody6DOF)>,
) {
    let total_dt = time.delta_secs();
    if total_dt < 0.0001 || total_dt > 0.1 {
        return;
    }

    // Liczba substepów zależna od dt
    let substeps = ((total_dt / 0.004).ceil() as u32).clamp(1, 8);
    let dt = total_dt / substeps as f32;

    for (mut transform, mut rb) in query.iter_mut() {
        if !rb.is_dynamic {
            rb.clear_forces();
            continue;
        }

        // Podziel siły na substepy
        let force_per_step = rb.force;
        let torque_per_step = rb.torque;

        for _ in 0..substeps {
            // Integracja prędkości
            let acceleration = force_per_step * rb.inv_mass;
            rb.velocity += acceleration * dt;

            let angular_acceleration = torque_per_step * rb.inv_inertia;
            rb.angular_velocity += angular_acceleration * dt;

            // Damping
            let linear_damp = 1.0 - rb.linear_damping * dt;
            let angular_damp = 1.0 - rb.angular_damping * dt;
            rb.velocity *= linear_damp;
            rb.angular_velocity *= angular_damp;

            // Integracja pozycji
            transform.translation += rb.velocity * dt;

            // Integracja rotacji
            let omega = rb.angular_velocity;
            let omega_quat = Quat::from_xyzw(
                omega.x * dt * 0.5,
                omega.y * dt * 0.5,
                omega.z * dt * 0.5,
                0.0,
            );
            let dq = omega_quat * transform.rotation;
            transform.rotation = Quat::from_xyzw(
                transform.rotation.x + dq.x,
                transform.rotation.y + dq.y,
                transform.rotation.z + dq.z,
                transform.rotation.w + dq.w,
            ).normalize();
        }

        rb.clear_forces();
    }
}
