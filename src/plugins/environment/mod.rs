mod spawn;
pub mod terrain_mesh;
pub mod monte_cassino_spawn;

use bevy::prelude::*;
use monte_cassino_spawn::spawn_monte_cassino_environment;

// Export terrain marker component
pub use spawn::TerrainMesh;

/// Plugin for world environment (Monte Cassino terrain)
pub struct EnvironmentPlugin;

impl Plugin for EnvironmentPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_monte_cassino_environment);
    }
}
