use bevy::prelude::*;

/// System sets for controlling execution order across plugins
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum TankGameSet {
    /// Process raw input (keyboard, mouse) for camera
    Input,
    /// Process tank input with smoothing/ramping
    TankInputRead,
    /// Detect terrain type and slope under tank
    TerrainDetection,
    /// Calculate physics (engine, tracks, forces)
    PhysicsCalculation,
    /// Move tank based on physics
    TankMovement,
    /// Update suspension visuals
    SuspensionUpdate,
    /// Track velocities for dispersion calculation
    VelocityTracking,
    /// Compute aim point from camera direction
    AimComputation,
    /// Rotate turret toward aim point
    TurretControl,
    /// Elevate barrel toward aim point
    BarrelControl,
    /// Calculate dispersion based on velocities
    DispersionCalculation,
    /// Update camera position
    CameraUpdate,
    /// Update UI elements
    UiUpdate,
}
