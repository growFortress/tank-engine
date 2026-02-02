use bevy::prelude::*;

/// Static crosshair at screen center (where we WANT to aim)
#[derive(Component)]
pub struct CrosshairUI;

/// Dynamic aim circle (shows gun convergence state)
#[derive(Component)]
pub struct AimCircleUI {
    /// Current interpolated size for smooth transitions
    pub current_size: f32,
    /// Current interpolated color for smooth transitions
    pub current_color: [f32; 4],
}

impl Default for AimCircleUI {
    fn default() -> Self {
        Self {
            current_size: 40.0,
            current_color: [0.3, 1.0, 0.3, 0.9],
        }
    }
}

/// 3D marker showing where the camera is aiming (red sphere)
#[derive(Component)]
pub struct AimMarker3D;

/// 3D marker showing where the gun is actually pointing (green sphere)
#[derive(Component)]
pub struct GunMarker3D;

/// Speed display UI element
#[derive(Component)]
pub struct SpeedUI;

/// Terrain type display UI element
#[derive(Component)]
pub struct TerrainUI;
