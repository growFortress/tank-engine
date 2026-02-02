use bevy::prelude::*;
use crate::components::{Tank, TerrainResistance, SlopeState, DriftState};
use crate::resources::{
    TerrainInfo, TerrainType,
    MonteCassinoTerrain, get_snakes_head_ridge, calculate_ridge_height,
};

/// Detect terrain type and slope under the tank (Monte Cassino heightmap)
pub fn detect_terrain(
    mc_terrain: Res<MonteCassinoTerrain>,
    mut terrain_info: ResMut<TerrainInfo>,
    mut tank_q: Query<(
        &Transform,
        &mut TerrainResistance,
        &mut SlopeState,
        &mut DriftState,
    ), With<Tank>>,
) {
    let Ok((transform, mut resistance, mut slope, mut drift)) = tank_q.get_single_mut() else {
        return;
    };

    let tank_pos = transform.translation;

    // Calculate terrain from Monte Cassino heightmap
    let (terrain_height, terrain_normal, terrain_type) = calculate_terrain_at_position_mc(tank_pos, &mc_terrain);

    // Update terrain info resource
    terrain_info.current_terrain_type = terrain_type;
    terrain_info.terrain_height = terrain_height;
    terrain_info.terrain_normal = terrain_normal;

    // Update slope state
    slope.terrain_normal = terrain_normal;

    // Calculate slope angle from normal
    let up = Vec3::Y;
    let dot = terrain_normal.dot(up).clamp(-1.0, 1.0);
    slope.slope_angle = dot.acos();
    slope.on_slope = slope.slope_angle > 0.03; // ~1.7 degree threshold

    // Calculate slope direction relative to tank forward
    if slope.on_slope {
        let forward = transform.right(); // Tank uses +X as forward
        let slope_horizontal = Vec3::new(terrain_normal.x, 0.0, terrain_normal.z);

        if slope_horizontal.length() > 0.001 {
            let slope_dir = slope_horizontal.normalize();
            // Dot product gives us if we're facing uphill (positive) or downhill (negative)
            let facing_dot = forward.dot(slope_dir);
            slope.slope_direction = facing_dot.acos();

            // Adjust direction based on which way we're facing
            if forward.cross(slope_dir).y < 0.0 {
                slope.slope_direction = -slope.slope_direction;
            }
        }
    } else {
        slope.slope_direction = 0.0;
    }

    // Update terrain resistance
    resistance.current_resistance = terrain_type.resistance();

    // Update grip for drift calculations (slopes reduce grip)
    let slope_grip_penalty = (slope.slope_angle * 0.5).sin(); // Steeper = less grip
    drift.grip = (terrain_type.grip() - slope_grip_penalty * 0.3).max(0.2);
}

/// Calculate terrain at position using Monte Cassino heightmap
fn calculate_terrain_at_position_mc(
    position: Vec3,
    terrain: &MonteCassinoTerrain,
) -> (f32, Vec3, TerrainType) {
    // Base height from heightmap
    let mut height = terrain.sample_height(position.x, position.z);
    let mut normal = terrain.sample_normal(position.x, position.z);

    // Add Snake's Head Ridge overlay (analytical on top of heightmap)
    let ridge = get_snakes_head_ridge();
    if let Some(ridge_height) = calculate_ridge_height(
        bevy::math::Vec2::new(position.x, position.z),
        &ridge
    ) {
        height += ridge_height;
        // Slightly steepen normal on ridge
        normal = (normal + Vec3::new(0.0, 0.5, 0.0)).normalize();
    }

    // Get terrain type from MC terrain zones
    let terrain_type = terrain.get_terrain_type(position.x, position.z);

    (height, normal, terrain_type)
}

// update_tank_height removed - height is now controlled by RaycastSuspension
