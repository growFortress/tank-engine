//! Track material configuration
//!
//! Different weathering states for tank tracks.

use super::super::texture_generator::TrackTextureConfig;

/// Standard used tracks
pub fn standard_tracks() -> TrackTextureConfig {
    TrackTextureConfig {
        resolution: 512,
        seed: 42,
        rubber_color: [0.08, 0.08, 0.08],
        metal_color: [0.35, 0.33, 0.30],
        dirt_amount: 0.25,
    }
}

/// Clean tracks (parade condition)
pub fn clean_tracks() -> TrackTextureConfig {
    TrackTextureConfig {
        resolution: 512,
        seed: 42,
        rubber_color: [0.05, 0.05, 0.05],
        metal_color: [0.45, 0.43, 0.40],
        dirt_amount: 0.05,
    }
}

/// Muddy tracks
pub fn muddy_tracks() -> TrackTextureConfig {
    TrackTextureConfig {
        resolution: 512,
        seed: 42,
        rubber_color: [0.15, 0.12, 0.10],  // Mud-covered
        metal_color: [0.30, 0.25, 0.20],
        dirt_amount: 0.65,
    }
}

/// Desert dusty tracks
pub fn dusty_tracks() -> TrackTextureConfig {
    TrackTextureConfig {
        resolution: 512,
        seed: 42,
        rubber_color: [0.20, 0.18, 0.15],  // Dusty
        metal_color: [0.50, 0.45, 0.38],
        dirt_amount: 0.4,
    }
}

/// Snow-covered tracks
pub fn snowy_tracks() -> TrackTextureConfig {
    TrackTextureConfig {
        resolution: 512,
        seed: 42,
        rubber_color: [0.15, 0.15, 0.15],  // Wet from snow
        metal_color: [0.40, 0.40, 0.42],   // Slightly bluish
        dirt_amount: 0.15,
    }
}
