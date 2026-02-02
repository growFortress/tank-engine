use bevy::prelude::*;

/// Main game camera with orbit and sniper modes
#[derive(Component)]
pub struct GameCamera {
    /// Horizontal rotation angle (radians)
    pub yaw: f32,
    /// Vertical rotation angle (radians)
    pub pitch: f32,
    /// Distance from pivot point in arcade mode
    pub distance: f32,
    /// Minimum pitch angle
    pub min_pitch: f32,
    /// Maximum pitch angle
    pub max_pitch: f32,
    /// Whether sniper mode is active
    pub sniper_mode: bool,
    /// Field of view in sniper mode (degrees)
    pub sniper_fov: f32,
    /// Sniper mode yaw offset from turret direction (radians)
    pub sniper_yaw_offset: f32,
    /// Sniper mode pitch offset (radians)
    pub sniper_pitch_offset: f32,
    /// Stabilized pivot position (smoothed turret position)
    pub stabilized_pivot: Vec3,
    /// Previous sniper mode state for transition detection
    pub prev_sniper_mode: bool,
    /// Transition blend factor (0.0 = arcade, 1.0 = sniper)
    pub mode_blend: f32,
}

impl Default for GameCamera {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.15,
            distance: 12.0, // Increased for larger 500x500 map
            min_pitch: -0.1,
            max_pitch: 0.7,
            sniper_mode: false,
            sniper_fov: 12.0,
            sniper_yaw_offset: 0.0,
            sniper_pitch_offset: 0.0,
            stabilized_pivot: Vec3::ZERO,
            prev_sniper_mode: false,
            mode_blend: 0.0,
        }
    }
}
