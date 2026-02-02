use bevy::prelude::*;
use crate::components::{Turret, GunMount, Barrel, GunMarker3D};
use crate::resources::GameState;

/// Approximate barrel length for tip position calculation
const BARREL_LENGTH: f32 = 3.5;

pub fn elevate_barrel_to_aim(
    time: Res<Time>,
    mut state: ResMut<GameState>,
    turret_q: Query<&GlobalTransform, With<Turret>>,
    mount_q: Query<&GlobalTransform, With<GunMount>>,
    mut barrel_q: Query<
        (&Barrel, &mut Transform, &GlobalTransform),
        (Without<Turret>, Without<GunMount>),
    >,
    mut gun_marker_q: Query<
        &mut Transform,
        (With<GunMarker3D>, Without<Barrel>, Without<Turret>, Without<GunMount>),
    >,
) {
    let Ok(turret_gt) = turret_q.get_single() else {
        return;
    };
    let Ok(mount_gt) = mount_q.get_single() else {
        return;
    };
    let Ok((barrel, mut barrel_tf, barrel_gt)) = barrel_q.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();

    // Barrel pivot point
    let mount_pos = mount_gt.translation();

    // Direction to aim point in world space
    // Handle edge case where aim_point is very close to mount
    let to_aim_raw = state.aim_point - mount_pos;
    if to_aim_raw.length_squared() < 0.01 {
        return; // Avoid division by zero
    }
    let to_aim = to_aim_raw.normalize();

    // Turret rotation
    let (_, turret_rot, _) = turret_gt.to_scale_rotation_translation();

    // Transform to turret local space
    let local_dir = turret_rot.inverse() * to_aim;

    // Target pitch
    let horizontal_dist = (local_dir.x.powi(2) + local_dir.z.powi(2)).sqrt();
    let target_pitch = local_dir.y.atan2(horizontal_dist);

    // Current pitch
    let (_, _, current_pitch) = barrel_tf.rotation.to_euler(EulerRot::YXZ);

    // FIXED: Apply speed limit BEFORE clamping to limits
    // This prevents the barrel from getting "stuck" at limits
    let diff = target_pitch - current_pitch;
    let max_delta = barrel.elevation_speed.to_radians() * dt;
    let delta = diff.clamp(-max_delta, max_delta);

    // Apply new pitch with clamping to physical limits
    let new_pitch = (current_pitch + delta).clamp(
        barrel.max_depression.to_radians(),
        barrel.max_elevation.to_radians(),
    );
    barrel_tf.rotation = Quat::from_rotation_z(new_pitch);

    // Calculate gun point (where the barrel is actually pointing)
    // Gun barrel points along +X axis (not -Z), so use right() instead of forward()
    let barrel_fwd = barrel_gt.right().as_vec3();
    let barrel_pos = barrel_gt.translation();

    // Use barrel tip position for more accurate gun_point calculation
    let barrel_tip = barrel_pos + barrel_fwd * BARREL_LENGTH;

    // Raycast from barrel tip to ground
    if barrel_fwd.y < -0.001 {
        let t = -barrel_tip.y / barrel_fwd.y;
        if t > 0.0 && t < 800.0 {
            state.gun_point = barrel_tip + barrel_fwd * t;
        } else {
            state.gun_point = barrel_tip + barrel_fwd * 150.0;
        }
    } else {
        state.gun_point = barrel_tip + barrel_fwd * 150.0;
    }

    // Distance-scaled is_aimed threshold with hysteresis
    // Prevents flickering when gun_point oscillates around aim_point
    let distance = (state.aim_point - barrel_pos).length();
    let aim_dist = (state.aim_point - state.gun_point).length();

    // Threshold scales with distance: 1.5% of distance, clamped to reasonable range
    let threshold_enter = (distance * 0.012).clamp(0.5, 2.5); // Easier to become aimed
    let threshold_exit = (distance * 0.020).clamp(0.8, 4.0);   // Harder to lose aimed status

    // Hysteresis: use different thresholds for entering vs exiting aimed state
    state.is_aimed = if state.was_aimed {
        aim_dist < threshold_exit
    } else {
        aim_dist < threshold_enter
    };
    state.was_aimed = state.is_aimed;

    // Update gun marker with smooth interpolation
    if let Ok(mut marker_tf) = gun_marker_q.get_single_mut() {
        let target_pos = state.gun_point + Vec3::Y * 0.1;
        marker_tf.translation = marker_tf.translation.lerp(target_pos, 15.0 * dt);
    }
}
