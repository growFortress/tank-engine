use bevy::prelude::*;
use crate::components::{
    Tank, TankMobility, TrackPhysics, SuspensionState, DriftState, SlopeState,
};

/// System for tank movement with realistic physics
///
/// Physics model:
/// - Track-based differential steering
/// - Engine power determines acceleration
/// - Mass affects inertia
/// - Terrain resistance affects speed
/// - Slope affects climbing ability
/// - Suspension provides visual feedback
pub fn move_tank(
    time: Res<Time>,
    mut tank_q: Query<(
        &mut Transform,
        &TankMobility,
        &TrackPhysics,
        &SuspensionState,
        &DriftState,
        &SlopeState,
    ), With<Tank>>,
) {
    let Ok((mut transform, mobility, tracks, suspension, drift, slope)) = tank_q.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    if dt < 0.0001 {
        return;
    }

    // === ROTATION FROM TRACK DIFFERENTIAL ===
    // Angular velocity = (right_speed - left_speed) / track_width
    let angular_velocity = (tracks.right_track_speed - tracks.left_track_speed) / tracks.track_width;

    if angular_velocity.abs() > 0.001 {
        let rotation_amount = angular_velocity * dt;
        transform.rotate_y(rotation_amount);
    }

    // === LINEAR MOVEMENT ===
    // Tank model uses +X as forward direction
    if mobility.current_speed.abs() > 0.001 {
        let forward = transform.right(); // +X is forward for this tank model

        // Check if we can climb the slope
        let can_move = if slope.on_slope && slope.slope_angle > slope.max_climbable_angle {
            // Too steep - slide back
            false
        } else {
            true
        };

        if can_move {
            let movement_delta = forward * mobility.current_speed * dt;
            transform.translation += movement_delta;
        } else {
            // Sliding down steep slope
            let slide_dir = Vec3::new(slope.terrain_normal.x, 0.0, slope.terrain_normal.z).normalize_or_zero();
            let slide_speed = 2.0 * (slope.slope_angle - slope.max_climbable_angle).sin();
            transform.translation += slide_dir * slide_speed * dt;
        }
    }

    // === DRIFT/LATERAL MOVEMENT ===
    if drift.lateral_velocity.length() > 0.001 {
        transform.translation += drift.lateral_velocity * dt;
    }

    // === SUSPENSION VISUAL ===
    // Apply pitch and roll from suspension to the hull visual rotation
    // Tank uses +X as forward, so:
    // - Pitch (nose up/down) = rotation around Z axis (local right)
    // - Roll (lean left/right) = rotation around X axis (local forward)

    // Get current heading
    let (yaw, _, _) = transform.rotation.to_euler(EulerRot::YXZ);

    // Reconstruct rotation with suspension effects
    // Note: For +X forward tank model:
    // - pitch around Z (positive pitch = nose up)
    // - roll around X (positive roll = lean right)
    let heading_rotation = Quat::from_rotation_y(yaw);
    let pitch_rotation = Quat::from_rotation_z(-suspension.pitch); // Z axis for pitch
    let roll_rotation = Quat::from_rotation_x(suspension.roll);    // X axis for roll

    // Apply in order: heading first, then local rotations
    transform.rotation = heading_rotation * pitch_rotation * roll_rotation;
}

/// Helper function for linear interpolation
#[allow(dead_code)]
fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}
