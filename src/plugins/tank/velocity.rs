use bevy::prelude::*;
use std::f32::consts::PI;
use crate::components::{Tank, TankVelocities, Turret, TurretVelocity};

/// Movement threshold - below this, considered stationary
const MOVEMENT_THRESHOLD: f32 = 0.05;
/// Rotation threshold - below this, considered not rotating
const ROTATION_THRESHOLD: f32 = 0.01;
/// Turret rotation threshold
const TURRET_ROTATION_THRESHOLD: f32 = 0.005;

/// Track tank hull velocity (movement and rotation)
pub fn track_tank_velocity(
    time: Res<Time>,
    mut tank_q: Query<(&Transform, &mut TankVelocities), With<Tank>>,
) {
    let dt = time.delta_secs();
    if dt < 0.0001 {
        return;
    }

    for (transform, mut velocities) in tank_q.iter_mut() {
        // Calculate linear speed
        let position = transform.translation;
        let movement = position - velocities.prev_position;
        let horizontal_movement = Vec3::new(movement.x, 0.0, movement.z);
        velocities.linear_speed = horizontal_movement.length() / dt;
        velocities.is_moving = velocities.linear_speed > MOVEMENT_THRESHOLD;

        // Calculate hull rotation speed
        let (current_yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let mut yaw_diff = current_yaw - velocities.prev_hull_yaw;

        // Normalize to [-PI, PI]
        while yaw_diff > PI {
            yaw_diff -= 2.0 * PI;
        }
        while yaw_diff < -PI {
            yaw_diff += 2.0 * PI;
        }

        velocities.hull_rotation_speed = yaw_diff.abs() / dt;
        velocities.is_hull_rotating = velocities.hull_rotation_speed > ROTATION_THRESHOLD;

        // Update previous frame values
        velocities.prev_position = position;
        velocities.prev_hull_yaw = current_yaw;
    }
}

/// Track turret rotation velocity
pub fn track_turret_velocity(
    time: Res<Time>,
    mut turret_q: Query<(&Transform, &mut TurretVelocity), With<Turret>>,
) {
    let dt = time.delta_secs();
    if dt < 0.0001 {
        return;
    }

    for (transform, mut velocity) in turret_q.iter_mut() {
        let (current_yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);
        let mut yaw_diff = current_yaw - velocity.prev_yaw;

        // Normalize
        while yaw_diff > PI {
            yaw_diff -= 2.0 * PI;
        }
        while yaw_diff < -PI {
            yaw_diff += 2.0 * PI;
        }

        velocity.rotation_speed = yaw_diff.abs() / dt;
        velocity.is_traversing = velocity.rotation_speed > TURRET_ROTATION_THRESHOLD;

        velocity.prev_yaw = current_yaw;
    }
}
