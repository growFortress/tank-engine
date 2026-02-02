use bevy::prelude::*;
use std::f32::consts::PI;
use crate::components::{Tank, Turret};
use crate::resources::GameState;

pub fn rotate_turret_to_aim(
    time: Res<Time>,
    state: Res<GameState>,
    tank_q: Query<&GlobalTransform, With<Tank>>,
    mut turret_q: Query<(&Turret, &mut Transform, &GlobalTransform), Without<Tank>>,
) {
    let Ok(tank_gt) = tank_q.get_single() else {
        return;
    };
    let Ok((turret, mut turret_tf, turret_gt)) = turret_q.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Turret world position
    let turret_pos = turret_gt.translation();

    // Direction to aim point in world space
    let to_aim = (state.aim_point - turret_pos).normalize();

    // Hull rotation (parent of turret)
    let (_, hull_rot, _) = tank_gt.to_scale_rotation_translation();

    // Transform direction to hull local space
    let local_dir = hull_rot.inverse() * to_aim;

    // Target yaw angle
    // Tank model uses forward = +X (not Bevy's default -Z)
    // Positive Y rotation turns +X toward -Z (left turn when +Z is right)
    // atan2(-z, x) gives angle from +X axis, positive = left turn
    let target_yaw = (-local_dir.z).atan2(local_dir.x);

    // Current turret yaw
    let (current_yaw, _, _) = turret_tf.rotation.to_euler(EulerRot::YXZ);

    // Difference (shortest path)
    let mut diff = target_yaw - current_yaw;
    while diff > PI {
        diff -= 2.0 * PI;
    }
    while diff < -PI {
        diff += 2.0 * PI;
    }

    // Speed limit
    let max_delta = turret.traverse_speed.to_radians() * dt;
    let delta = diff.clamp(-max_delta, max_delta);

    // Apply new yaw
    let new_yaw = current_yaw + delta;
    turret_tf.rotation = Quat::from_rotation_y(new_yaw);
}
