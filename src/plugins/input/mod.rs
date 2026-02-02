mod systems;

use bevy::prelude::*;
use crate::systems::TankGameSet;
use systems::handle_input;

/// Plugin for user input handling
pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, handle_input.in_set(TankGameSet::Input));
    }
}
