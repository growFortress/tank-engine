use bevy::prelude::*;
use crate::components::{Tank, TankInput};

/// Read raw keyboard input and apply smoothing/ramping
pub fn read_tank_input(
    time: Res<Time>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut tank_q: Query<&mut TankInput, With<Tank>>,
) {
    let dt = time.delta_secs();
    if dt < 0.0001 {
        return;
    }

    let Ok(mut input) = tank_q.get_single_mut() else {
        return;
    };

    // Read raw input with proper key combination handling
    // W+S cancel each other out, A+D cancel each other out
    let forward_pressed = keyboard.pressed(KeyCode::KeyW);
    let backward_pressed = keyboard.pressed(KeyCode::KeyS);
    let left_pressed = keyboard.pressed(KeyCode::KeyA) || keyboard.pressed(KeyCode::ArrowLeft);
    let right_pressed = keyboard.pressed(KeyCode::KeyD) || keyboard.pressed(KeyCode::ArrowRight);

    // Forward/backward: if both pressed, they cancel (0.0)
    input.raw_forward = match (forward_pressed, backward_pressed) {
        (true, false) => 1.0,   // Only W
        (false, true) => -1.0,  // Only S
        (true, true) => 0.0,    // W+S cancel
        (false, false) => 0.0,  // Neither
    };

    // Rotation: if both pressed, they cancel (0.0)
    input.raw_rotation = match (left_pressed, right_pressed) {
        (true, false) => -1.0,  // Only A/← - turn left
        (false, true) => 1.0,   // Only D/→ - turn right
        (true, true) => 0.0,    // A+D cancel
        (false, false) => 0.0,  // Neither
    };

    // Brake input (Space)
    input.braking = keyboard.pressed(KeyCode::Space);

    // === HAMULCE GĄSIENICOWE ===
    // Q = hamulec lewej gąsienicy, E = hamulec prawej gąsienicy
    input.brake_left = if keyboard.pressed(KeyCode::KeyQ) { 1.0 } else { 0.0 };
    input.brake_right = if keyboard.pressed(KeyCode::KeyE) { 1.0 } else { 0.0 };

    // Space aktywuje oba hamulce
    if input.braking {
        input.brake_left = 1.0;
        input.brake_right = 1.0;
    }

    // Apply ramping to forward input
    input.forward = apply_input_ramping(
        input.forward,
        input.raw_forward,
        input.acceleration_rate,
        input.deceleration_rate,
        dt,
    );

    // Apply ramping to rotation input
    input.rotation = apply_input_ramping(
        input.rotation,
        input.raw_rotation,
        input.acceleration_rate * 1.5,  // Rotation ramps faster
        input.deceleration_rate * 1.5,
        dt,
    );
}

/// Apply smooth ramping to input value
fn apply_input_ramping(
    current: f32,
    target: f32,
    accel_rate: f32,
    decel_rate: f32,
    dt: f32,
) -> f32 {
    let diff = target - current;

    if diff.abs() < 0.001 {
        return target;
    }

    // Determine rate based on whether we're accelerating toward target or decelerating
    let rate = if target.abs() > current.abs() || target.signum() != current.signum() {
        // Moving toward higher magnitude or changing direction
        if target == 0.0 {
            decel_rate  // Releasing input
        } else {
            accel_rate  // Pressing input
        }
    } else {
        decel_rate  // Reducing input
    };

    let change = diff.signum() * rate * dt;

    // Don't overshoot
    if change.abs() > diff.abs() {
        target
    } else {
        current + change
    }
}
