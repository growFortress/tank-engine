use bevy::prelude::*;
use bevy::input::mouse::{MouseMotion, MouseWheel};
use bevy::window::CursorGrabMode;
use crate::components::GameCamera;

/// Dead zone for mouse input (pixels) - prevents micro-movements
const MOUSE_DEAD_ZONE: f32 = 0.3;
/// Sniper mode yaw limit (radians) - how far left/right from turret direction
const SNIPER_YAW_LIMIT: f32 = 0.8;
/// Sniper mode pitch limits
const SNIPER_PITCH_MIN: f32 = -0.3;
const SNIPER_PITCH_MAX: f32 = 0.5;

/// Apply acceleration curve to mouse input for better feel
/// Small movements are precise, large movements are amplified
fn apply_acceleration_curve(delta: f32) -> f32 {
    let abs_delta = delta.abs();
    // Power curve: small moves stay small, big moves get bigger
    // Using 1.15 for subtle but noticeable acceleration
    let curved = abs_delta.powf(1.15);
    curved.copysign(delta)
}

pub fn handle_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut scroll: EventReader<MouseWheel>,
    mut camera_q: Query<&mut GameCamera>,
    mut windows: Query<&mut Window>,
) {
    let Ok(mut cam) = camera_q.get_single_mut() else {
        return;
    };
    let Ok(mut window) = windows.get_single_mut() else {
        return;
    };

    // ESC - toggle cursor
    if keyboard.just_pressed(KeyCode::Escape) {
        let confined = window.cursor_options.grab_mode == CursorGrabMode::Confined;
        window.cursor_options.grab_mode = if confined {
            CursorGrabMode::None
        } else {
            CursorGrabMode::Confined
        };
        window.cursor_options.visible = confined;
    }

    // Sniper mode
    cam.sniper_mode = keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight);

    // Mouse sensitivity - different for each mode
    let sens = if cam.sniper_mode {
        // Sniper: scales with sqrt of FOV ratio for consistent feel at different zoom levels
        let fov_factor = (cam.sniper_fov / 12.0).sqrt();
        0.001 * fov_factor.clamp(0.4, 1.6)
    } else {
        0.0025
    };

    // Mouse -> camera rotation with dead zone and acceleration
    for ev in mouse_motion.read() {
        // Apply dead zone - ignore very small movements
        let dx = if ev.delta.x.abs() < MOUSE_DEAD_ZONE { 0.0 } else { ev.delta.x };
        let dy = if ev.delta.y.abs() < MOUSE_DEAD_ZONE { 0.0 } else { ev.delta.y };

        // Skip if both are in dead zone
        if dx == 0.0 && dy == 0.0 {
            continue;
        }

        // Apply acceleration curve for better feel
        let curved_dx = apply_acceleration_curve(dx);
        let curved_dy = apply_acceleration_curve(dy);

        if cam.sniper_mode {
            // Sniper mode: control offset relative to turret direction
            cam.sniper_yaw_offset = (cam.sniper_yaw_offset - curved_dx * sens)
                .clamp(-SNIPER_YAW_LIMIT, SNIPER_YAW_LIMIT);
            cam.sniper_pitch_offset = (cam.sniper_pitch_offset - curved_dy * sens)
                .clamp(SNIPER_PITCH_MIN, SNIPER_PITCH_MAX);
        } else {
            // Arcade mode: free orbit camera
            cam.yaw -= curved_dx * sens;
            cam.pitch = (cam.pitch - curved_dy * sens).clamp(cam.min_pitch, cam.max_pitch);
        }
    }

    // Scroll -> zoom (updated limits)
    for ev in scroll.read() {
        if cam.sniper_mode {
            // Sniper FOV: 2.0 to 30.0 degrees (more zoom capability)
            cam.sniper_fov = (cam.sniper_fov - ev.y * 2.5).clamp(2.0, 30.0);
        } else {
            // Arcade distance: 3.0 to 35.0 (closer to tank)
            cam.distance = (cam.distance - ev.y * 2.0).clamp(3.0, 35.0);
        }
    }
    // Note: Hull rotation/movement handled by TankMovement system (WASD/arrows)
}
