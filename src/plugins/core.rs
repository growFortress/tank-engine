use bevy::prelude::*;
use crate::resources::{GameState, DispersionState};
use crate::systems::TankGameSet;

/// Core plugin that initializes shared resources and configures system ordering
pub struct CorePlugin;

impl Plugin for CorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameState>()
            .init_resource::<DispersionState>()
            .configure_sets(
                Update,
                (
                    TankGameSet::Input,
                    TankGameSet::TankInputRead,
                    TankGameSet::TerrainDetection,
                    TankGameSet::PhysicsCalculation,
                    TankGameSet::TankMovement,
                    TankGameSet::SuspensionUpdate,
                    TankGameSet::VelocityTracking,
                    TankGameSet::AimComputation,
                    TankGameSet::TurretControl,
                    TankGameSet::BarrelControl,
                    TankGameSet::DispersionCalculation,
                    TankGameSet::CameraUpdate,
                    TankGameSet::UiUpdate,
                )
                    .chain(),
            );
    }
}
