mod spawn;
mod systems;

use bevy::prelude::*;
use crate::systems::TankGameSet;
use spawn::spawn_ui;
use systems::{update_ui, update_speed_ui, update_terrain_ui};

/// Plugin for UI elements (crosshair, aim indicators, speed display)
pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_ui)
            .add_systems(
                Update,
                (update_ui, update_speed_ui, update_terrain_ui)
                    .in_set(TankGameSet::UiUpdate),
            );
    }
}
