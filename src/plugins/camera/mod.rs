mod spawn;
mod systems;

use bevy::prelude::*;
use crate::systems::TankGameSet;
use spawn::spawn_camera;
use systems::{compute_aim_point, position_camera};

/// Plugin for camera spawning and control
pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(
                Update,
                compute_aim_point.in_set(TankGameSet::AimComputation),
            )
            .add_systems(
                Update,
                position_camera.in_set(TankGameSet::CameraUpdate),
            );
    }
}
