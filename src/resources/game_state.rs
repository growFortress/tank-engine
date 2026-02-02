use bevy::prelude::*;

/// Shared state for the aiming system
#[derive(Resource)]
pub struct GameState {
    /// Raw aim point from camera raycast (unsmoothed)
    pub target_aim_point: Vec3,
    /// Smoothed world position where the camera is aiming
    pub aim_point: Vec3,
    /// World position where the gun barrel is actually pointing
    pub gun_point: Vec3,
    /// Whether the gun has converged on the aim point
    pub is_aimed: bool,
    /// Previous is_aimed state for hysteresis
    pub was_aimed: bool,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            target_aim_point: Vec3::new(0.0, 0.0, 50.0),
            aim_point: Vec3::new(0.0, 0.0, 50.0),
            gun_point: Vec3::new(0.0, 0.0, 50.0),
            is_aimed: false,
            was_aimed: false,
        }
    }
}
