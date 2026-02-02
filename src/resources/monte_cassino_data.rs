use bevy::prelude::*;
use super::monte_cassino::{MC_GRID_SIZE, MC_MAP_HALF, MC_HEIGHT_SCALE, MC_BASE_ELEVATION};

// ============================================================================
// ELEVATION CONTROL POINTS
// ============================================================================

/// Key elevation control points based on historical data (scaled 2.5x for 500x500 map)
/// (position in game units, real-world elevation in meters)
pub fn get_elevation_control_points() -> Vec<(Vec2, f32)> {
    vec![
        // Valley floor (Cassino town area)
        (Vec2::new(-175.0, -200.0), 40.0),   // Cassino town center
        (Vec2::new(-200.0, -150.0), 45.0),   // West of town
        (Vec2::new(-150.0, -175.0), 50.0),   // East approach
        (Vec2::new(-187.5, -100.0), 55.0),   // River valley
        (Vec2::new(-200.0, 0.0), 50.0),      // River area
        (Vec2::new(-212.5, 100.0), 45.0),    // Northern valley

        // Lower slopes and Castle Hill area
        (Vec2::new(-125.0, -150.0), 80.0),   // Approach to Castle Hill
        (Vec2::new(-75.0, -100.0), 165.0),   // Castle Hill (Point 165)
        (Vec2::new(-100.0, -75.0), 140.0),   // Near Castle Hill
        (Vec2::new(-50.0, -125.0), 120.0),   // East of Castle Hill

        // Point 236 area
        (Vec2::new(-50.0, -50.0), 236.0),    // Point 236
        (Vec2::new(-25.0, -75.0), 200.0),    // Below Point 236
        (Vec2::new(-75.0, -25.0), 210.0),    // West of Point 236

        // Hangman's Hill area (Point 435)
        (Vec2::new(0.0, 50.0), 435.0),       // Hangman's Hill (Point 435)
        (Vec2::new(-25.0, 25.0), 380.0),     // Southwest of Hangman's
        (Vec2::new(25.0, 25.0), 400.0),      // Southeast of Hangman's

        // Point 445 area
        (Vec2::new(25.0, 75.0), 445.0),      // Point 445
        (Vec2::new(50.0, 62.5), 420.0),      // East of Point 445
        (Vec2::new(12.5, 100.0), 460.0),     // North of Point 445

        // Point 476 area
        (Vec2::new(50.0, 100.0), 476.0),     // Point 476
        (Vec2::new(75.0, 112.5), 490.0),     // Toward abbey

        // Abbey (Monte Cassino) - 519m
        (Vec2::new(100.0, 150.0), 519.0),    // Abbey main
        (Vec2::new(87.5, 137.5), 505.0),     // Abbey approach SW
        (Vec2::new(112.5, 137.5), 510.0),    // Abbey approach SE
        (Vec2::new(87.5, 162.5), 515.0),     // Abbey north side

        // Point 569 area
        (Vec2::new(125.0, 175.0), 569.0),    // Point 569
        (Vec2::new(137.5, 162.5), 550.0),    // South of 569
        (Vec2::new(112.5, 187.5), 555.0),    // West of 569

        // Point 593 (Monte Calvario) - highest point on Snake's Head Ridge
        (Vec2::new(150.0, 200.0), 593.0),    // Point 593
        (Vec2::new(162.5, 187.5), 575.0),    // Southeast of 593
        (Vec2::new(137.5, 212.5), 580.0),    // Northwest of 593

        // Colle Sant'Angelo area (edge of map)
        (Vec2::new(200.0, 225.0), 650.0),    // Toward Colle Sant'Angelo
        (Vec2::new(225.0, 212.5), 620.0),    // Eastern edge

        // Snake's Head Ridge spine
        (Vec2::new(175.0, 175.0), 560.0),    // Ridge center
        (Vec2::new(187.5, 187.5), 585.0),    // Ridge bend

        // Northern slopes
        (Vec2::new(0.0, 200.0), 350.0),      // Northern area
        (Vec2::new(-75.0, 175.0), 280.0),    // Northwest
        (Vec2::new(75.0, 225.0), 450.0),     // North toward peaks

        // Eastern edges
        (Vec2::new(225.0, 0.0), 300.0),      // East edge center
        (Vec2::new(225.0, -125.0), 200.0),   // Southeast
        (Vec2::new(225.0, 125.0), 400.0),    // Northeast

        // Southern edges
        (Vec2::new(0.0, -225.0), 100.0),     // South center
        (Vec2::new(125.0, -200.0), 150.0),   // Southeast approach
        (Vec2::new(-125.0, -225.0), 60.0),   // Southwest (valley)

        // Western edges (river valley)
        (Vec2::new(-225.0, -125.0), 50.0),   // River valley south
        (Vec2::new(-225.0, 0.0), 55.0),      // River valley center
        (Vec2::new(-225.0, 125.0), 60.0),    // River valley north
        (Vec2::new(-225.0, 225.0), 80.0),    // Northwest corner

        // Corners
        (Vec2::new(-237.5, -237.5), 45.0),   // SW corner (valley)
        (Vec2::new(237.5, -237.5), 120.0),   // SE corner
        (Vec2::new(-237.5, 237.5), 90.0),    // NW corner
        (Vec2::new(237.5, 237.5), 550.0),    // NE corner (high ground)
    ]
}

// ============================================================================
// HEIGHTMAP GENERATION
// ============================================================================

/// Generate Monte Cassino heightmap from control points
/// Uses inverse distance weighting (IDW) interpolation
pub fn generate_monte_cassino_heights() -> Vec<f32> {
    let control_points = get_elevation_control_points();
    let mut heights = vec![0.0; MC_GRID_SIZE * MC_GRID_SIZE];

    let cell_size = (MC_MAP_HALF * 2.0) / (MC_GRID_SIZE - 1) as f32;

    for gz in 0..MC_GRID_SIZE {
        for gx in 0..MC_GRID_SIZE {
            // Convert grid coords to world coords
            let world_x = -MC_MAP_HALF + gx as f32 * cell_size;
            let world_z = -MC_MAP_HALF + gz as f32 * cell_size;
            let pos = Vec2::new(world_x, world_z);

            // Inverse distance weighting interpolation
            let height = idw_interpolate(&control_points, pos);

            // Convert from real meters to game units
            let game_height = (height - MC_BASE_ELEVATION) * MC_HEIGHT_SCALE;

            // Clamp to valid range
            heights[gz * MC_GRID_SIZE + gx] = game_height.max(0.0);
        }
    }

    // Apply smoothing pass to reduce artifacts
    smooth_heightmap(&mut heights, 2);

    // Add noise for natural variation
    add_terrain_noise(&mut heights, 0.3);

    heights
}

/// Inverse Distance Weighting interpolation
fn idw_interpolate(control_points: &[(Vec2, f32)], pos: Vec2) -> f32 {
    let power = 2.5; // IDW power parameter (higher = more local influence)
    let mut weight_sum = 0.0;
    let mut value_sum = 0.0;

    for (ctrl_pos, ctrl_height) in control_points {
        let dist = (*ctrl_pos - pos).length();

        if dist < 0.001 {
            // Very close to control point - return its value
            return *ctrl_height;
        }

        let weight = 1.0 / dist.powf(power);
        weight_sum += weight;
        value_sum += weight * ctrl_height;
    }

    if weight_sum > 0.0 {
        value_sum / weight_sum
    } else {
        MC_BASE_ELEVATION
    }
}

/// Apply smoothing to heightmap
fn smooth_heightmap(heights: &mut [f32], iterations: usize) {
    let size = MC_GRID_SIZE;

    for _ in 0..iterations {
        let original = heights.to_vec();

        for z in 1..size - 1 {
            for x in 1..size - 1 {
                let idx = z * size + x;

                // Average with neighbors (3x3 kernel)
                let sum = original[idx]
                    + original[(z - 1) * size + x]
                    + original[(z + 1) * size + x]
                    + original[z * size + (x - 1)]
                    + original[z * size + (x + 1)]
                    + original[(z - 1) * size + (x - 1)]
                    + original[(z - 1) * size + (x + 1)]
                    + original[(z + 1) * size + (x - 1)]
                    + original[(z + 1) * size + (x + 1)];

                heights[idx] = sum / 9.0;
            }
        }
    }
}

/// Add subtle noise for natural terrain variation
fn add_terrain_noise(heights: &mut [f32], amplitude: f32) {
    use std::f32::consts::PI;

    let size = MC_GRID_SIZE;

    for z in 0..size {
        for x in 0..size {
            let idx = z * size + x;

            // Simple deterministic noise using sin waves
            let fx = x as f32 / size as f32;
            let fz = z as f32 / size as f32;

            let noise = (fx * 17.3 * PI).sin() * 0.3
                + (fz * 23.7 * PI).sin() * 0.3
                + ((fx + fz) * 31.1 * PI).sin() * 0.2
                + ((fx * 2.0 + fz * 3.0) * 11.0 * PI).sin() * 0.2;

            heights[idx] += noise * amplitude;
            heights[idx] = heights[idx].max(0.0);
        }
    }
}

// ============================================================================
// TERRAIN FEATURES - ADDITIONAL DETAIL POINTS
// ============================================================================

/// Phantom Ridge data (connects to Massa Albaneta) - scaled 2.5x
pub fn get_phantom_ridge_points() -> Vec<(Vec2, f32)> {
    vec![
        (Vec2::new(75.0, 125.0), 480.0),
        (Vec2::new(62.5, 137.5), 470.0),
        (Vec2::new(50.0, 150.0), 450.0),
        (Vec2::new(37.5, 162.5), 430.0),
        (Vec2::new(25.0, 175.0), 400.0),
    ]
}

/// Massa Albaneta area - scaled 2.5x
pub fn get_massa_albaneta_points() -> Vec<(Vec2, f32)> {
    vec![
        (Vec2::new(75.0, 175.0), 500.0),   // Massa Albaneta main
        (Vec2::new(87.5, 187.5), 520.0),   // North side
        (Vec2::new(62.5, 187.5), 490.0),   // West side
    ]
}

// ============================================================================
// BUILDING DATA
// ============================================================================

/// Building structure for Cassino town
pub struct BuildingData {
    pub position: Vec2,
    pub size: Vec3,
    pub rotation: f32,
    pub is_destroyed: bool,
}

/// Get Cassino town buildings (mostly destroyed ruins) - scaled 2.5x
pub fn get_cassino_town_buildings() -> Vec<BuildingData> {
    vec![
        // Town center ruins
        BuildingData {
            position: Vec2::new(-170.0, -187.5),
            size: Vec3::new(20.0, 10.0, 15.0),
            rotation: 0.1,
            is_destroyed: true,
        },
        BuildingData {
            position: Vec2::new(-180.0, -195.0),
            size: Vec3::new(15.0, 7.5, 12.5),
            rotation: -0.05,
            is_destroyed: true,
        },
        BuildingData {
            position: Vec2::new(-162.5, -180.0),
            size: Vec3::new(25.0, 12.5, 20.0),
            rotation: 0.0,
            is_destroyed: true,
        },
        // Hotel Continental (famous strongpoint)
        BuildingData {
            position: Vec2::new(-150.0, -170.0),
            size: Vec3::new(30.0, 15.0, 25.0),
            rotation: 0.15,
            is_destroyed: true,
        },
        // Railway station area
        BuildingData {
            position: Vec2::new(-137.5, -205.0),
            size: Vec3::new(37.5, 10.0, 15.0),
            rotation: 0.3,
            is_destroyed: true,
        },
        // More town ruins
        BuildingData {
            position: Vec2::new(-187.5, -175.0),
            size: Vec3::new(12.5, 6.25, 10.0),
            rotation: 0.2,
            is_destroyed: true,
        },
        BuildingData {
            position: Vec2::new(-155.0, -200.0),
            size: Vec3::new(17.5, 8.75, 12.5),
            rotation: -0.1,
            is_destroyed: true,
        },
    ]
}

// ============================================================================
// DEBUG / VERIFICATION
// ============================================================================

/// Print heightmap statistics for verification
#[allow(dead_code)]
pub fn print_heightmap_stats(heights: &[f32]) {
    let min = heights.iter().cloned().fold(f32::INFINITY, f32::min);
    let max = heights.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let sum: f32 = heights.iter().sum();
    let avg = sum / heights.len() as f32;

    println!("Monte Cassino Heightmap Statistics:");
    println!("  Grid size: {}x{}", MC_GRID_SIZE, MC_GRID_SIZE);
    println!("  Min height: {:.2} units ({:.0}m real)", min, min / MC_HEIGHT_SCALE + MC_BASE_ELEVATION);
    println!("  Max height: {:.2} units ({:.0}m real)", max, max / MC_HEIGHT_SCALE + MC_BASE_ELEVATION);
    println!("  Avg height: {:.2} units ({:.0}m real)", avg, avg / MC_HEIGHT_SCALE + MC_BASE_ELEVATION);
}
