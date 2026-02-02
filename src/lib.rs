pub mod components;
pub mod resources;
pub mod systems;
pub mod plugins;
pub mod physics;
pub mod prelude;
pub mod rendering;

use bevy::prelude::*;
use bevy::app::PluginGroupBuilder;
use plugins::*;

/// Game-specific plugins (add after DefaultPlugins)
pub struct TankGamePlugins;

impl PluginGroup for TankGamePlugins {
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::start::<Self>()
            .add(CorePlugin)
            .add(InputPlugin)
            .add(TankPlugin)
            .add(CameraPlugin)
            .add(UiPlugin)
            .add(EnvironmentPlugin)
    }
}
