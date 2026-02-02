use bevy::prelude::*;

/// Dispersion state for the aiming system
#[derive(Resource)]
pub struct DispersionState {
    /// Current dispersion value in meters (at 100m distance)
    pub current_dispersion: f32,
    /// Base dispersion when fully aimed (minimum possible)
    pub base_dispersion: f32,
    /// Maximum dispersion cap
    pub max_dispersion: f32,
    /// Time to fully aim from max dispersion (seconds)
    pub aim_time: f32,
}

impl Default for DispersionState {
    fn default() -> Self {
        Self {
            current_dispersion: 0.5,
            base_dispersion: 0.36,
            max_dispersion: 4.0,
            aim_time: 2.3,
        }
    }
}
