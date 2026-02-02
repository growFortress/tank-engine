use bevy::prelude::*;
use bevy::window::CursorGrabMode;
use tank_3d::TankGamePlugins;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Tank 3D - Monte Cassino".into(),
                resolution: (1280., 720.).into(),
                cursor_options: bevy::window::CursorOptions {
                    grab_mode: CursorGrabMode::Confined,
                    visible: false,
                    ..default()
                },
                ..default()
            }),
            ..default()
        }))
        .add_plugins(TankGamePlugins)
        .run();
}
