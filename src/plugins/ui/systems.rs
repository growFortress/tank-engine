use bevy::prelude::*;
use crate::components::{AimCircleUI, AimMarker3D, GameCamera, SpeedUI, TerrainUI, Tank, TankMobility};
use crate::resources::{GameState, DispersionState, TerrainInfo, TerrainType};

/// Calculate reticle size based on dispersion and distance
fn calculate_reticle_size(
    dispersion: f32,
    distance_to_target: f32,
    fov_degrees: f32,
    screen_height: f32,
) -> f32 {
    // Dispersion is in meters at 100m distance
    // The spread at actual distance scales linearly with distance
    let actual_spread = dispersion * (distance_to_target / 100.0);

    // Convert world-space spread to screen-space pixels using angular projection
    // angular_size = atan(spread / distance) ≈ spread / distance for small angles
    let fov_rad = fov_degrees.to_radians();
    let projection_factor = screen_height / (2.0 * (fov_rad / 2.0).tan());

    // For proper screen-space size: angular_size * projection_factor
    let angular_size = actual_spread / distance_to_target.max(1.0);
    let size_pixels = angular_size * projection_factor;

    // Clamp to reasonable range (tighter than before for cleaner visuals)
    size_pixels.clamp(20.0, 150.0)
}

/// Interpolate between aim circle colors based on dispersion ratio
/// Returns smooth gradient instead of discrete color bands
fn get_aim_color(dispersion_ratio: f32, is_aimed: bool) -> [f32; 4] {
    // Color definitions [R, G, B, A]
    let green = [0.3, 1.0, 0.3, 0.92];        // Fully aimed
    let yellow_green = [0.6, 1.0, 0.3, 0.90]; // Almost aimed
    let orange = [1.0, 0.65, 0.2, 0.88];      // Partial
    let red = [1.0, 0.35, 0.15, 0.92];        // High dispersion

    // If fully aimed, use bright green regardless of ratio
    if is_aimed && dispersion_ratio < 0.15 {
        return green;
    }

    // Smooth interpolation between color bands
    let ratio = dispersion_ratio.clamp(0.0, 1.0);

    if ratio < 0.33 {
        let t = ratio / 0.33;
        lerp_color(green, yellow_green, t)
    } else if ratio < 0.66 {
        let t = (ratio - 0.33) / 0.33;
        lerp_color(yellow_green, orange, t)
    } else {
        let t = (ratio - 0.66) / 0.34;
        lerp_color(orange, red, t)
    }
}

/// Linear interpolation between two RGBA colors
fn lerp_color(a: [f32; 4], b: [f32; 4], t: f32) -> [f32; 4] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
        a[3] + (b[3] - a[3]) * t,
    ]
}

pub fn update_ui(
    time: Res<Time>,
    state: Res<GameState>,
    dispersion: Res<DispersionState>,
    windows: Query<&Window>,
    cam_q: Query<(&GameCamera, &Transform, &Projection)>,
    mut aim_circle_q: Query<(&mut AimCircleUI, &mut Node, &mut BorderColor)>,
    mut aim_marker_q: Query<&mut Transform, (With<AimMarker3D>, Without<GameCamera>)>,
) {
    let Ok(window) = windows.get_single() else { return };
    let Ok((_cam, cam_tf, proj)) = cam_q.get_single() else { return };

    let dt = time.delta_secs();

    // Get current FOV
    let fov_degrees = match proj {
        Projection::Perspective(p) => p.fov.to_degrees(),
        _ => 60.0,
    };

    // Distance from camera to aim point
    let distance = (state.aim_point - cam_tf.translation).length();

    // Calculate target reticle size based on dispersion
    let target_size = calculate_reticle_size(
        dispersion.current_dispersion,
        distance,
        fov_degrees,
        window.height(),
    );

    // Update aim circle with smooth interpolation
    if let Ok((mut circle, mut node, mut border)) = aim_circle_q.get_single_mut() {
        // Calculate dispersion ratio for color
        let dispersion_ratio = if dispersion.max_dispersion > dispersion.base_dispersion {
            (dispersion.current_dispersion - dispersion.base_dispersion)
                / (dispersion.max_dispersion - dispersion.base_dispersion)
        } else {
            0.0
        };

        // Get target color with smooth gradient
        let target_color = get_aim_color(dispersion_ratio, state.is_aimed);

        // Smooth interpolation for size and color
        // Different speeds: size changes faster, color changes smoother
        let size_blend = (12.0 * dt).min(1.0);
        let color_blend = (8.0 * dt).min(1.0);

        // Interpolate size
        circle.current_size = circle.current_size + (target_size - circle.current_size) * size_blend;

        // Interpolate color
        for i in 0..4 {
            circle.current_color[i] = circle.current_color[i]
                + (target_color[i] - circle.current_color[i]) * color_blend;
        }

        // Apply interpolated values
        let c = circle.current_color;
        border.0 = Color::srgba(c[0], c[1], c[2], c[3]);

        let half_size = circle.current_size / 2.0;
        node.width = Val::Px(circle.current_size);
        node.height = Val::Px(circle.current_size);
        node.margin = UiRect::all(Val::Px(-half_size));
    }

    // Update aim marker position with smooth interpolation
    if let Ok(mut marker_tf) = aim_marker_q.get_single_mut() {
        let target_pos = state.aim_point + Vec3::Y * 0.1;
        marker_tf.translation = marker_tf.translation.lerp(target_pos, 15.0 * dt);
    }
}

/// Update speed display
pub fn update_speed_ui(
    mobility_q: Query<&TankMobility, With<Tank>>,
    mut ui_q: Query<(&mut Text, &mut TextColor), With<SpeedUI>>,
) {
    let Ok(mobility) = mobility_q.get_single() else { return };

    for (mut text, mut color) in ui_q.iter_mut() {
        // Convert m/s to km/h
        let speed_kmh = mobility.current_speed.abs() * 3.6;

        // Format with sign for reverse
        let display_text = if mobility.current_speed < -0.1 {
            format!("-{:.0} km/h", speed_kmh)
        } else {
            format!("{:.0} km/h", speed_kmh)
        };

        *text = Text::new(display_text);

        // Color based on speed relative to max
        let speed_ratio = speed_kmh / (mobility.max_forward_speed * 3.6);
        if mobility.current_speed < -0.1 {
            // Reversing - orange
            color.0 = Color::srgba(1.0, 0.6, 0.2, 0.9);
        } else if speed_ratio > 0.9 {
            // Near max speed - green
            color.0 = Color::srgba(0.3, 1.0, 0.3, 0.9);
        } else if speed_ratio > 0.5 {
            // Medium speed - yellow
            color.0 = Color::srgba(1.0, 1.0, 0.3, 0.9);
        } else {
            // Low speed or stationary - white
            color.0 = Color::srgba(1.0, 1.0, 1.0, 0.9);
        }
    }
}

/// Update terrain type display
pub fn update_terrain_ui(
    terrain_info: Res<TerrainInfo>,
    mut ui_q: Query<(&mut Text, &mut TextColor), With<TerrainUI>>,
) {
    for (mut text, mut color) in ui_q.iter_mut() {
        let (terrain_name, terrain_color) = match terrain_info.current_terrain_type {
            TerrainType::Hard => ("Hard (Road)", Color::srgba(0.6, 0.6, 0.8, 0.9)),
            TerrainType::Medium => ("Medium (Grass)", Color::srgba(0.5, 0.8, 0.4, 0.9)),
            TerrainType::Soft => ("Soft (Mud)", Color::srgba(0.8, 0.6, 0.3, 0.9)),
        };

        *text = Text::new(format!("TERRAIN: {}", terrain_name));
        color.0 = terrain_color;
    }
}
