use bevy::prelude::*;
use super::terrain::TerrainType;

// ============================================================================
// MONTE CASSINO TERRAIN CONSTANTS
// ============================================================================

/// Map size in game units (500x500 for 5km x 5km at 1:10 scale)
pub const MC_MAP_SIZE: f32 = 500.0;
pub const MC_MAP_HALF: f32 = 250.0;

/// Heightmap grid resolution (power of 2 + 1 for better interpolation)
pub const MC_GRID_SIZE: usize = 129;

/// Vertical scale factor (1:15 - real meters to game units)
/// 553m elevation range -> ~37 game units
pub const MC_HEIGHT_SCALE: f32 = 1.0 / 15.0;

/// Base elevation in real meters (valley floor)
pub const MC_BASE_ELEVATION: f32 = 40.0;

// ============================================================================
// MONTE CASSINO TERRAIN RESOURCE
// ============================================================================

/// Monte Cassino heightmap terrain resource
#[derive(Resource)]
pub struct MonteCassinoTerrain {
    /// Height values in a 2D grid (row-major order)
    /// Index: heights[z * MC_GRID_SIZE + x]
    pub heights: Vec<f32>,
    /// World space bounds
    pub world_min: Vec2,
    pub world_max: Vec2,
}

impl MonteCassinoTerrain {
    /// Create terrain from height data
    pub fn new(heights: Vec<f32>) -> Self {
        assert_eq!(heights.len(), MC_GRID_SIZE * MC_GRID_SIZE,
            "Height data must be {}x{} = {} values",
            MC_GRID_SIZE, MC_GRID_SIZE, MC_GRID_SIZE * MC_GRID_SIZE);

        Self {
            heights,
            world_min: Vec2::new(-MC_MAP_HALF, -MC_MAP_HALF),
            world_max: Vec2::new(MC_MAP_HALF, MC_MAP_HALF),
        }
    }

    /// Sample height at world position using bilinear interpolation
    pub fn sample_height(&self, world_x: f32, world_z: f32) -> f32 {
        // Convert world coords to grid coords (0 to MC_GRID_SIZE-1)
        let grid_x = ((world_x - self.world_min.x) /
                     (self.world_max.x - self.world_min.x)) * (MC_GRID_SIZE - 1) as f32;
        let grid_z = ((world_z - self.world_min.y) /
                     (self.world_max.y - self.world_min.y)) * (MC_GRID_SIZE - 1) as f32;

        // Clamp to valid range
        let gx = grid_x.clamp(0.0, (MC_GRID_SIZE - 1) as f32);
        let gz = grid_z.clamp(0.0, (MC_GRID_SIZE - 1) as f32);

        // Get integer grid coordinates
        let x0 = gx.floor() as usize;
        let z0 = gz.floor() as usize;
        let x1 = (x0 + 1).min(MC_GRID_SIZE - 1);
        let z1 = (z0 + 1).min(MC_GRID_SIZE - 1);

        // Fractional parts for interpolation
        let fx = gx.fract();
        let fz = gz.fract();

        // Sample four corners
        let h00 = self.heights[z0 * MC_GRID_SIZE + x0];
        let h10 = self.heights[z0 * MC_GRID_SIZE + x1];
        let h01 = self.heights[z1 * MC_GRID_SIZE + x0];
        let h11 = self.heights[z1 * MC_GRID_SIZE + x1];

        // Bilinear interpolation
        let h0 = h00 + (h10 - h00) * fx;
        let h1 = h01 + (h11 - h01) * fx;

        h0 + (h1 - h0) * fz
    }

    /// Calculate surface normal using finite differences
    pub fn sample_normal(&self, world_x: f32, world_z: f32) -> Vec3 {
        let epsilon = 0.5; // Sample spacing for gradient

        let h_px = self.sample_height(world_x + epsilon, world_z);
        let h_nx = self.sample_height(world_x - epsilon, world_z);
        let h_pz = self.sample_height(world_x, world_z + epsilon);
        let h_nz = self.sample_height(world_x, world_z - epsilon);

        // Central difference gradient
        let dx = (h_px - h_nx) / (2.0 * epsilon);
        let dz = (h_pz - h_nz) / (2.0 * epsilon);

        // Normal from gradient: (-dh/dx, 1, -dh/dz) normalized
        Vec3::new(-dx, 1.0, -dz).normalize()
    }

    /// Get terrain type based on position and height
    pub fn get_terrain_type(&self, x: f32, z: f32) -> TerrainType {
        let height = self.sample_height(x, z);

        // River zone check (Rapido/Gari river in western portion)
        if self.is_in_river_zone(x, z) {
            return TerrainType::Soft;
        }

        // Road network check (main roads are hard terrain)
        if self.is_on_road(x, z) {
            return TerrainType::Hard;
        }

        // Height-based zones
        // Heights are in game units (scaled from real meters)
        match height {
            h if h < 5.0 => TerrainType::Hard,    // Valley floor (roads, town)
            h if h < 15.0 => TerrainType::Medium, // Lower slopes
            h if h < 30.0 => TerrainType::Soft,   // Mountain slopes (rocky/muddy)
            _ => TerrainType::Medium,              // High peaks (exposed rock)
        }
    }

    /// Check if position is in river zone
    fn is_in_river_zone(&self, x: f32, z: f32) -> bool {
        // Rapido/Gari river runs roughly north-south in western portion
        // River center at approximately x = -187.5 (scaled 2.5x from -75)
        let river_center_x = -187.5 + z * 0.012;
        let river_width = 25.0;

        (x - river_center_x).abs() < river_width / 2.0 && z > -225.0 && z < 225.0
    }

    /// Check if position is on a road
    fn is_on_road(&self, x: f32, z: f32) -> bool {
        let road_width = 10.0; // Scaled 2.5x from 4.0

        // Main road: Via Casilina - from Cassino town toward abbey
        // Scaled 2.5x: (-70, -80) → (-175, -200), (40, 60) → (100, 150)
        let road_start = Vec2::new(-175.0, -200.0);
        let road_end = Vec2::new(100.0, 150.0);
        let road_dir = (road_end - road_start).normalize();
        let road_length = (road_end - road_start).length();

        let to_point = Vec2::new(x, z) - road_start;
        let along = to_point.dot(road_dir);

        if along >= 0.0 && along <= road_length {
            let perp = (to_point - road_dir * along).length();
            if perp < road_width / 2.0 {
                return true;
            }
        }

        // Secondary road: from Castle Hill area
        // Scaled 2.5x: (-30, -40) → (-75, -100), (-60, -70) → (-150, -175)
        let road2_start = Vec2::new(-75.0, -100.0);
        let road2_end = Vec2::new(-150.0, -175.0);
        let road2_dir = (road2_end - road2_start).normalize();
        let road2_length = (road2_end - road2_start).length();

        let to_point2 = Vec2::new(x, z) - road2_start;
        let along2 = to_point2.dot(road2_dir);

        if along2 >= 0.0 && along2 <= road2_length {
            let perp2 = (to_point2 - road2_dir * along2).length();
            if perp2 < road_width / 2.0 {
                return true;
            }
        }

        false
    }
}

// ============================================================================
// RIVER DATA
// ============================================================================

/// River data for Rapido/Gari
pub struct RiverData {
    /// Center line points of the river
    pub center_points: Vec<Vec2>,
    /// Width at each point
    pub widths: Vec<f32>,
    /// Depth at each point
    pub depths: Vec<f32>,
}

/// Get Rapido/Gari river data (scaled 2.5x for 500x500 map)
pub fn get_rapido_river() -> RiverData {
    RiverData {
        center_points: vec![
            Vec2::new(-195.0, -237.5),  // South edge
            Vec2::new(-187.5, -150.0),
            Vec2::new(-192.5, -75.0),
            Vec2::new(-185.0, 0.0),     // Near Cassino
            Vec2::new(-190.0, 75.0),
            Vec2::new(-182.5, 150.0),
            Vec2::new(-187.5, 237.5),   // North edge
        ],
        widths: vec![25.0, 30.0, 35.0, 40.0, 35.0, 30.0, 25.0],
        depths: vec![1.5, 2.0, 2.5, 3.0, 2.5, 2.0, 1.5],
    }
}

// ============================================================================
// FORTIFICATION DATA
// ============================================================================

/// Types of fortifications
#[derive(Clone, Copy, Debug)]
pub enum FortificationType {
    Bunker,
    Pillbox,
    Trench,
    Observation,
}

/// Fortification data structure
#[derive(Clone)]
pub struct FortificationData {
    pub position: Vec2,
    pub fortification_type: FortificationType,
    pub rotation: f32,
    pub size: Vec3,
}

/// Get Gustav Line fortifications (scaled 2.5x for 500x500 map)
pub fn get_gustav_line_fortifications() -> Vec<FortificationData> {
    vec![
        // Bunkers along the Gustav Line
        FortificationData {
            position: Vec2::new(-137.5, -87.5),
            fortification_type: FortificationType::Bunker,
            rotation: 0.4,
            size: Vec3::new(12.5, 6.25, 10.0),
        },
        FortificationData {
            position: Vec2::new(-112.5, -25.0),
            fortification_type: FortificationType::Bunker,
            rotation: 0.2,
            size: Vec3::new(11.25, 5.0, 8.75),
        },
        FortificationData {
            position: Vec2::new(-87.5, 37.5),
            fortification_type: FortificationType::Bunker,
            rotation: -0.1,
            size: Vec3::new(12.5, 6.25, 10.0),
        },

        // Pillboxes on slopes
        FortificationData {
            position: Vec2::new(-62.5, -62.5),
            fortification_type: FortificationType::Pillbox,
            rotation: 0.0,
            size: Vec3::new(7.5, 4.5, 7.5),
        },
        FortificationData {
            position: Vec2::new(-37.5, 0.0),
            fortification_type: FortificationType::Pillbox,
            rotation: 0.3,
            size: Vec3::new(7.5, 4.5, 7.5),
        },
        FortificationData {
            position: Vec2::new(12.5, 62.5),
            fortification_type: FortificationType::Pillbox,
            rotation: -0.2,
            size: Vec3::new(8.75, 5.0, 8.75),
        },

        // Trenches
        FortificationData {
            position: Vec2::new(-100.0, -125.0),
            fortification_type: FortificationType::Trench,
            rotation: 0.6,
            size: Vec3::new(62.5, 3.0, 6.25),
        },
        FortificationData {
            position: Vec2::new(-50.0, -87.5),
            fortification_type: FortificationType::Trench,
            rotation: 0.4,
            size: Vec3::new(50.0, 2.5, 5.0),
        },
        FortificationData {
            position: Vec2::new(25.0, 25.0),
            fortification_type: FortificationType::Trench,
            rotation: -0.3,
            size: Vec3::new(45.0, 2.5, 5.0),
        },

        // Observation posts on high ground
        FortificationData {
            position: Vec2::new(137.5, 180.0),
            fortification_type: FortificationType::Observation,
            rotation: 0.0,
            size: Vec3::new(10.0, 7.5, 10.0),
        },
        FortificationData {
            position: Vec2::new(155.0, 205.0),
            fortification_type: FortificationType::Observation,
            rotation: 0.1,
            size: Vec3::new(8.75, 6.25, 8.75),
        },
    ]
}

/// Get abbey (monastery) structure data (scaled 2.5x for 500x500 map)
pub fn get_abbey_data() -> FortificationData {
    FortificationData {
        position: Vec2::new(100.0, 150.0),
        fortification_type: FortificationType::Observation,
        rotation: 0.0,
        size: Vec3::new(87.5, 37.5, 70.0),
    }
}

// ============================================================================
// SNAKE'S HEAD RIDGE DATA
// ============================================================================

/// Snake's Head Ridge - curved ridge structure NE of abbey
pub struct SnakeHeadRidge {
    /// Control points defining the boomerang shape (spine)
    pub spine_points: Vec<Vec2>,
    /// Width of the ridge at each control point
    pub widths: Vec<f32>,
    /// Additional height above base terrain at each point
    pub heights: Vec<f32>,
}

/// Get Snake's Head Ridge data (scaled 2.5x for 500x500 map)
pub fn get_snakes_head_ridge() -> SnakeHeadRidge {
    SnakeHeadRidge {
        spine_points: vec![
            Vec2::new(100.0, 150.0),   // Start near abbey
            Vec2::new(125.0, 170.0),   // Point 569 area
            Vec2::new(150.0, 187.5),   // Curve point
            Vec2::new(180.0, 180.0),   // Boomerang bend
            Vec2::new(205.0, 195.0),   // End toward Point 593
        ],
        widths: vec![25.0, 30.0, 35.0, 30.0, 20.0],
        heights: vec![5.0, 7.5, 10.0, 8.75, 6.25],
    }
}

/// Calculate ridge contribution at a position
pub fn calculate_ridge_height(pos: Vec2, ridge: &SnakeHeadRidge) -> Option<f32> {
    let mut min_dist = f32::MAX;
    let mut closest_segment_idx = 0;
    let mut closest_t = 0.0;

    // Find closest point on ridge spine
    for i in 0..ridge.spine_points.len() - 1 {
        let p0 = ridge.spine_points[i];
        let p1 = ridge.spine_points[i + 1];
        let segment = p1 - p0;
        let segment_len = segment.length();

        if segment_len < 0.001 {
            continue;
        }

        let segment_dir = segment / segment_len;
        let to_pos = pos - p0;
        let t = to_pos.dot(segment_dir).clamp(0.0, segment_len) / segment_len;
        let closest_point = p0 + segment * t;
        let dist = (pos - closest_point).length();

        if dist < min_dist {
            min_dist = dist;
            closest_segment_idx = i;
            closest_t = t;
        }
    }

    // Interpolate width and height at closest point
    let w0 = ridge.widths[closest_segment_idx];
    let w1 = ridge.widths[closest_segment_idx + 1];
    let h0 = ridge.heights[closest_segment_idx];
    let h1 = ridge.heights[closest_segment_idx + 1];

    let width = w0 + (w1 - w0) * closest_t;
    let height = h0 + (h1 - h0) * closest_t;

    // Check if within ridge width
    if min_dist < width / 2.0 {
        // Smooth falloff from center
        let falloff = 1.0 - (min_dist / (width / 2.0)).powi(2);
        Some(height * falloff)
    } else {
        None
    }
}
