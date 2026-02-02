use bevy::prelude::*;
use crate::components::{Tank, TankVelocities, Turret, TurretVelocity, GunDispersion};
use crate::resources::DispersionState;

/// Main dispersion calculation system (WoT-style)
pub fn calculate_dispersion(
    time: Res<Time>,
    mut dispersion: ResMut<DispersionState>,
    tank_q: Query<(&TankVelocities, &GunDispersion), With<Tank>>,
    turret_q: Query<&TurretVelocity, With<Turret>>,
) {
    let dt = time.delta_secs();
    if dt < 0.0001 {
        return;
    }

    let Ok((tank_vel, gun)) = tank_q.get_single() else {
        return;
    };
    let Ok(turret_vel) = turret_q.get_single() else {
        return;
    };

    // Stabilization reduces movement penalties (0.0 = no reduction, 0.8 = 80% reduction)
    let stab_factor = 1.0 - gun.stabilization.clamp(0.0, 0.8);

    // === Calculate penalties from current actions ===

    // Hull movement penalty
    let move_penalty = if tank_vel.is_moving {
        tank_vel.linear_speed * gun.dispersion_move * stab_factor
    } else {
        0.0
    };

    // Hull rotation penalty (convert rad/s to deg/s)
    let hull_rot_penalty = if tank_vel.is_hull_rotating {
        tank_vel.hull_rotation_speed.to_degrees() * gun.dispersion_hull_rotation * stab_factor
    } else {
        0.0
    };

    // Turret traverse penalty (convert rad/s to deg/s)
    let turret_penalty = if turret_vel.is_traversing {
        turret_vel.rotation_speed.to_degrees() * gun.dispersion_turret_rotation * stab_factor
    } else {
        0.0
    };

    // Target dispersion = base + sum of penalties
    let target_dispersion = (gun.base_accuracy + move_penalty + hull_rot_penalty + turret_penalty)
        .min(dispersion.max_dispersion);

    // === Apply dispersion changes ===

    if target_dispersion > dispersion.current_dispersion {
        // Bloom is fast (instant feel)
        let bloom_speed = 12.0;
        let blend = (bloom_speed * dt).min(1.0);
        dispersion.current_dispersion = dispersion.current_dispersion
            + (target_dispersion - dispersion.current_dispersion) * blend;
    } else {
        // Decay toward target (aim time)
        // aim_time is time to reduce from ~2.5x base to 1x base
        let decay_factor = (1.0 / gun.aim_time) * 2.5;
        let decay = (decay_factor * dt).min(1.0);
        dispersion.current_dispersion = dispersion.current_dispersion
            + (target_dispersion - dispersion.current_dispersion) * decay;
    }

    // Clamp to valid range
    dispersion.current_dispersion = dispersion.current_dispersion.clamp(
        gun.base_accuracy,
        dispersion.max_dispersion,
    );

    // Update resource values from gun params
    dispersion.base_dispersion = gun.base_accuracy;
    dispersion.aim_time = gun.aim_time;
}
