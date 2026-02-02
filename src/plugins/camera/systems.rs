use bevy::prelude::*;
use crate::components::{Tank, Turret, GameCamera};
use crate::resources::GameState;

/// Pitch damping factor for look direction (consistent across modes)
const PITCH_LOOK_FACTOR: f32 = 0.4;
/// Stabilization speed for pivot point (lower = more stable, less responsive)
const PIVOT_STABILIZATION_SPEED: f32 = 8.0;
/// Mode transition speed (how fast to blend between arcade/sniper)
const MODE_TRANSITION_SPEED: f32 = 6.0;

pub fn compute_aim_point(
    time: Res<Time>,
    cam_q: Query<(&Transform, &GameCamera)>,
    mut state: ResMut<GameState>,
) {
    let Ok((cam_tf, cam)) = cam_q.get_single() else {
        return;
    };

    let dt = time.delta_secs();

    // Camera forward direction
    let cam_fwd = cam_tf.forward().as_vec3();
    let cam_pos = cam_tf.translation;

    // Calculate raw aim point from raycast (scaled for 500x500 map)
    let raw_aim_point = if cam_fwd.y < -0.001 {
        let t = -cam_pos.y / cam_fwd.y;
        if t > 0.0 && t < 2000.0 {
            cam_pos + cam_fwd * t
        } else {
            cam_pos + cam_fwd * 375.0
        }
    } else {
        cam_pos + cam_fwd * 375.0
    };

    // Store raw target
    state.target_aim_point = raw_aim_point;

    // Smooth aim_point with different speeds for arcade/sniper
    // Sniper mode needs slower smoothing to prevent jarring jumps
    let smooth_speed = if cam.sniper_mode { 6.0 } else { 12.0 };
    let blend = (smooth_speed * dt).min(1.0);

    // Use lerp for smooth transition
    state.aim_point = state.aim_point.lerp(raw_aim_point, blend);
}

pub fn position_camera(
    time: Res<Time>,
    tank_q: Query<&GlobalTransform, With<Tank>>,
    turret_q: Query<&GlobalTransform, (With<Turret>, Without<Tank>)>,
    mut cam_q: Query<(&mut GameCamera, &mut Transform, &mut Projection)>,
) {
    let Ok(tank_gt) = tank_q.get_single() else {
        return;
    };
    let Ok(turret_gt) = turret_q.get_single() else {
        return;
    };
    let Ok((mut cam, mut cam_tf, mut proj)) = cam_q.get_single_mut() else {
        return;
    };

    let dt = time.delta_secs();
    let tank_pos = tank_gt.translation();
    let turret_pos = turret_gt.translation();

    // Update mode transition blend
    let target_blend = if cam.sniper_mode { 1.0 } else { 0.0 };
    cam.mode_blend += (target_blend - cam.mode_blend) * (MODE_TRANSITION_SPEED * dt).min(1.0);

    // Detect mode change for offset reset
    if cam.sniper_mode && !cam.prev_sniper_mode {
        // Entering sniper mode - reset offsets to center
        cam.sniper_yaw_offset = 0.0;
        cam.sniper_pitch_offset = 0.0;
    }
    cam.prev_sniper_mode = cam.sniper_mode;

    // Get turret's forward direction (turret uses +X as forward in local space)
    let turret_forward: Vec3 = turret_gt.right().into();

    // Stabilize pivot point - smooth out tank movement vibrations
    let raw_pivot = turret_pos + Vec3::Y * 0.6;
    if cam.stabilized_pivot == Vec3::ZERO {
        cam.stabilized_pivot = raw_pivot;
    }
    let stabilization = (PIVOT_STABILIZATION_SPEED * dt).min(1.0);
    cam.stabilized_pivot = cam.stabilized_pivot.lerp(raw_pivot, stabilization);

    // Calculate ARCADE mode camera position
    let arcade_pivot = tank_pos + Vec3::Y * 1.6;
    let arcade_offset = Vec3::new(
        cam.yaw.sin() * cam.pitch.cos(),
        cam.pitch.sin(),
        cam.yaw.cos() * cam.pitch.cos(),
    ) * cam.distance;
    let arcade_pos = arcade_pivot + arcade_offset;
    let arcade_look_dir = Vec3::new(
        -cam.yaw.sin(),
        -cam.pitch.sin() * PITCH_LOOK_FACTOR,
        -cam.yaw.cos(),
    );
    let arcade_look_at = arcade_pivot + arcade_look_dir.normalize() * 50.0;

    // Calculate SNIPER mode camera position with full rotation control
    // Apply yaw offset relative to turret direction
    let sniper_yaw = cam.sniper_yaw_offset;
    let sniper_pitch = cam.sniper_pitch_offset;

    // Rotate turret forward by sniper yaw offset
    let cos_yaw = sniper_yaw.cos();
    let sin_yaw = sniper_yaw.sin();
    let look_forward = Vec3::new(
        turret_forward.x * cos_yaw - turret_forward.z * sin_yaw,
        0.0,
        turret_forward.x * sin_yaw + turret_forward.z * cos_yaw,
    ).normalize();

    // Camera position: behind the look direction, using stabilized pivot
    let back_offset = -look_forward * 2.5;
    let up_offset = Vec3::Y * 1.0;
    let pitch_height = Vec3::Y * sniper_pitch.sin() * 1.5;
    let sniper_pos = cam.stabilized_pivot + back_offset + up_offset + pitch_height;

    // Look direction with pitch
    let sniper_look_dir = Vec3::new(
        look_forward.x,
        -sniper_pitch.sin() * PITCH_LOOK_FACTOR,
        look_forward.z,
    ).normalize();
    let sniper_look_at = cam.stabilized_pivot + sniper_look_dir * 100.0;

    // Blend between arcade and sniper positions
    let blend = cam.mode_blend;
    let target_pos = arcade_pos.lerp(sniper_pos, blend);
    let target_look_at = arcade_look_at.lerp(sniper_look_at, blend);

    // Interpolate camera position and rotation
    let interp_speed = 18.0 + blend * 4.0; // Slightly faster in sniper mode
    let s = (interp_speed * dt).min(1.0);
    cam_tf.translation = cam_tf.translation.lerp(target_pos, s);

    let target_rot = Transform::from_translation(cam_tf.translation)
        .looking_at(target_look_at, Vec3::Y)
        .rotation;
    cam_tf.rotation = cam_tf.rotation.slerp(target_rot, s);

    // Blend FOV
    if let Projection::Perspective(ref mut p) = *proj {
        let arcade_fov = 60.0_f32.to_radians();
        let sniper_fov = cam.sniper_fov.to_radians();
        let target_fov = arcade_fov + (sniper_fov - arcade_fov) * blend;
        p.fov = p.fov + (target_fov - p.fov) * s;
    }
}
