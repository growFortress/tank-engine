use bevy::prelude::*;
use crate::components::{CrosshairUI, AimCircleUI, AimMarker3D, GunMarker3D, SpeedUI, TerrainUI};

pub fn spawn_ui(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // === 2D UI ===

    // Crosshair - ALWAYS at screen center (where we WANT to aim)
    commands.spawn((
        CrosshairUI,
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(4.0),
            height: Val::Px(4.0),
            margin: UiRect::all(Val::Px(-2.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.95)),
    ));

    // Crosshair lines (static)
    for (w, h, mx, my) in [
        (12.0, 1.5, -20.0, -0.75), // left
        (12.0, 1.5, 8.0, -0.75),   // right
        (1.5, 12.0, -0.75, -20.0), // top
        (1.5, 12.0, -0.75, 8.0),   // bottom
    ] {
        commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                width: Val::Px(w),
                height: Val::Px(h),
                margin: UiRect {
                    left: Val::Px(mx),
                    top: Val::Px(my),
                    ..default()
                },
                ..default()
            },
            BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.7)),
        ));
    }

    // Aim Circle - shows where the gun ACTUALLY is (dynamic)
    commands.spawn((
        AimCircleUI::default(),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),
            top: Val::Percent(50.0),
            width: Val::Px(40.0),
            height: Val::Px(40.0),
            margin: UiRect::all(Val::Px(-20.0)),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        BorderColor(Color::srgba(0.3, 1.0, 0.3, 0.8)),
        BackgroundColor(Color::NONE),
    ));

    // Info text
    commands.spawn((
        Text::new("Scroll=Zoom | Shift=Sniper (free look) | ESC=Cursor"),
        TextFont {
            font_size: 13.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.45)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(10.0),
            top: Val::Px(10.0),
            ..default()
        },
    ));

    // === 3D MARKERS ===

    // AimMarker - where we're aiming (red sphere)
    let aim_marker_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(1.0, 0.2, 0.2, 0.8),
        emissive: LinearRgba::new(1.5, 0.1, 0.1, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        AimMarker3D,
        Mesh3d(meshes.add(Sphere::new(0.12))),
        MeshMaterial3d(aim_marker_mat),
        Transform::from_xyz(0.0, 0.1, 30.0),
    ));

    // GunMarker - where the gun is pointing (green sphere)
    let gun_marker_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.2, 1.0, 0.2, 0.8),
        emissive: LinearRgba::new(0.1, 1.5, 0.1, 1.0),
        alpha_mode: AlphaMode::Blend,
        ..default()
    });

    commands.spawn((
        GunMarker3D,
        Mesh3d(meshes.add(Sphere::new(0.10))),
        MeshMaterial3d(gun_marker_mat),
        Transform::from_xyz(0.0, 0.1, 30.0),
    ));

    // === SPEED AND MOBILITY UI ===

    // Speed display (bottom left)
    commands.spawn((
        SpeedUI,
        Text::new("0 km/h"),
        TextFont {
            font_size: 28.0,
            ..default()
        },
        TextColor(Color::srgba(1.0, 1.0, 1.0, 0.9)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(50.0),
            ..default()
        },
    ));

    // Speed label
    commands.spawn((
        Text::new("SPEED"),
        TextFont {
            font_size: 12.0,
            ..default()
        },
        TextColor(Color::srgba(0.7, 0.7, 0.7, 0.7)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(80.0),
            ..default()
        },
    ));

    // Terrain type display (bottom left, below speed)
    commands.spawn((
        TerrainUI,
        Text::new("TERRAIN: Medium"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::srgba(0.8, 0.8, 0.6, 0.8)),
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(20.0),
            bottom: Val::Px(20.0),
            ..default()
        },
    ));
}
