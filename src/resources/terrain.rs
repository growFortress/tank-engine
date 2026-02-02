use bevy::prelude::*;
use std::f32::consts::PI;

// ============================================================================
// TERRAIN DATA STRUCTURES (shared between environment and physics)
// ============================================================================

/// Hill data for terrain generation and physics
#[derive(Clone, Debug)]
pub struct HillData {
    pub center: Vec3,
    pub radius: f32,
    pub height: f32,
}

/// Ramp data for terrain generation and physics
#[derive(Clone, Debug)]
pub struct RampData {
    pub center: Vec3,
    pub direction: Vec3,
    pub length: f32,
    pub width: f32,
    pub height: f32,
}

/// Ridge data (elongated hill)
#[derive(Clone, Debug)]
pub struct RidgeData {
    pub x: f32,
    pub z: f32,
    pub length: f32,
    pub width: f32,
    pub height: f32,
    pub rotation: f32,
}

/// Get all hills in the level (single source of truth)
pub fn get_level_hills() -> Vec<HillData> {
    vec![
        // Large gentle hills
        HillData { center: Vec3::new(40.0, 0.0, -40.0), radius: 25.0, height: 6.0 },
        HillData { center: Vec3::new(-50.0, 0.0, 50.0), radius: 30.0, height: 8.0 },
        HillData { center: Vec3::new(70.0, 0.0, 60.0), radius: 20.0, height: 5.0 },
        HillData { center: Vec3::new(-70.0, 0.0, -50.0), radius: 22.0, height: 7.0 },

        // Medium hills
        HillData { center: Vec3::new(-30.0, 0.0, -60.0), radius: 15.0, height: 4.5 },
        HillData { center: Vec3::new(60.0, 0.0, -20.0), radius: 12.0, height: 3.5 },
        HillData { center: Vec3::new(-60.0, 0.0, 10.0), radius: 14.0, height: 4.0 },
        HillData { center: Vec3::new(20.0, 0.0, 70.0), radius: 16.0, height: 5.0 },

        // Small mounds near spawn
        HillData { center: Vec3::new(15.0, 0.0, 20.0), radius: 8.0, height: 2.0 },
        HillData { center: Vec3::new(-15.0, 0.0, -15.0), radius: 10.0, height: 2.5 },
        HillData { center: Vec3::new(25.0, 0.0, -10.0), radius: 7.0, height: 1.8 },
    ]
}

/// Get all ramps in the level (single source of truth)
pub fn get_level_ramps() -> Vec<RampData> {
    vec![
        // Main test ramp near spawn (gentle slope ~15°)
        RampData {
            center: Vec3::new(0.0, 2.5, 25.0),
            direction: Vec3::new(1.0, 0.0, 0.0),
            length: 20.0,
            width: 10.0,
            height: 5.0,
        },
        // Steeper ramp (~25°)
        RampData {
            center: Vec3::new(-25.0, 3.5, 45.0),
            direction: Vec3::new((PI * 0.25).cos(), 0.0, (PI * 0.25).sin()),
            length: 15.0,
            width: 8.0,
            height: 7.0,
        },
        // Side ramps
        RampData {
            center: Vec3::new(50.0, 3.0, 0.0),
            direction: Vec3::new((PI * 0.5).cos(), 0.0, (PI * 0.5).sin()),
            length: 18.0,
            width: 8.0,
            height: 6.0,
        },
        RampData {
            center: Vec3::new(-45.0, 2.75, -35.0),
            direction: Vec3::new((-PI * 0.3).cos(), 0.0, (-PI * 0.3).sin()),
            length: 16.0,
            width: 9.0,
            height: 5.5,
        },
        // Very steep ramp (~35° - at limit)
        RampData {
            center: Vec3::new(35.0, 4.25, 35.0),
            direction: Vec3::new((PI * 0.75).cos(), 0.0, (PI * 0.75).sin()),
            length: 12.0,
            width: 6.0,
            height: 8.5,
        },
    ]
}

/// Get all ridges in the level (single source of truth)
pub fn get_level_ridges() -> Vec<RidgeData> {
    vec![
        RidgeData { x: 0.0, z: -70.0, length: 60.0, width: 8.0, height: 4.0, rotation: 0.0 },
        RidgeData { x: 80.0, z: 30.0, length: 40.0, width: 6.0, height: 3.5, rotation: PI * 0.4 },
        RidgeData { x: -80.0, z: -20.0, length: 25.0, width: 10.0, height: 5.0, rotation: -PI * 0.2 },
    ]
}

/// Get crater rim hills (single source of truth)
pub fn get_crater_rim_hills() -> Vec<HillData> {
    let crater_pos = Vec3::new(-40.0, 0.0, 0.0);
    let crater_radius = 12.0;

    (0..8)
        .map(|i| {
            let angle = (i as f32) * PI * 2.0 / 8.0;
            HillData {
                center: Vec3::new(
                    crater_pos.x + angle.cos() * crater_radius,
                    0.0,
                    crater_pos.z + angle.sin() * crater_radius,
                ),
                radius: 4.0,
                height: 1.5,
            }
        })
        .collect()
}

// ============================================================================
// TERRAIN TYPES
// ============================================================================

/// Types of terrain affecting tank mobility
#[derive(Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum TerrainType {
    /// Hard surface - roads, concrete (resistance: 0.9)
    Hard,
    /// Medium surface - grass, packed dirt (resistance: 1.1)
    #[default]
    Medium,
    /// Soft surface - mud, sand, snow (resistance: 1.8)
    Soft,
}

impl TerrainType {
    /// Get the resistance multiplier for this terrain type
    /// ARCADE: mniejsze kary za teren - czytelna różnica, ale nie frustrująca
    pub fn resistance(&self) -> f32 {
        match self {
            TerrainType::Hard => 0.9,   // 10% bonus
            TerrainType::Medium => 1.0, // Neutralne (było 1.1)
            TerrainType::Soft => 1.3,   // 30% kara (było 1.8!)
        }
    }

    /// Get the grip coefficient for this terrain type
    /// ARCADE: wyższa przyczepność - mniej ślizgania na każdym terenie
    pub fn grip(&self) -> f32 {
        match self {
            TerrainType::Hard => 0.98,  // Prawie pełna (było 0.95)
            TerrainType::Medium => 0.92, // Dobra (było 0.85)
            TerrainType::Soft => 0.75,   // Umiarkowana (było 0.55!)
        }
    }
}

/// Global terrain information resource
#[derive(Resource)]
pub struct TerrainInfo {
    /// Current terrain type under the tank
    pub current_terrain_type: TerrainType,
    /// Terrain surface normal at tank position
    pub terrain_normal: Vec3,
    /// Height of terrain at tank position
    pub terrain_height: f32,
}

impl Default for TerrainInfo {
    fn default() -> Self {
        Self {
            current_terrain_type: TerrainType::Medium,
            terrain_normal: Vec3::Y,
            terrain_height: 0.0,
        }
    }
}
