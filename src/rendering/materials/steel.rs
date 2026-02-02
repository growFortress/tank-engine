//! Steel/barrel material configuration
//!
//! Different gun barrel states.

use super::super::texture_generator::SteelTextureConfig;

/// Standard barrel steel
pub fn standard_barrel() -> SteelTextureConfig {
    SteelTextureConfig {
        resolution: 512,
        seed: 42,
        steel_color: [0.40, 0.38, 0.36],
        heat_discoloration: 0.2,
    }
}

/// Heavily used barrel (more heat discoloration)
pub fn heavily_used_barrel() -> SteelTextureConfig {
    SteelTextureConfig {
        resolution: 512,
        seed: 42,
        steel_color: [0.35, 0.33, 0.32],
        heat_discoloration: 0.5,
    }
}

/// New barrel (minimal discoloration)
pub fn new_barrel() -> SteelTextureConfig {
    SteelTextureConfig {
        resolution: 512,
        seed: 42,
        steel_color: [0.45, 0.43, 0.40],
        heat_discoloration: 0.05,
    }
}

/// Chrome-lined barrel
pub fn chrome_lined_barrel() -> SteelTextureConfig {
    SteelTextureConfig {
        resolution: 512,
        seed: 42,
        steel_color: [0.55, 0.55, 0.52],  // Brighter, more chrome-like
        heat_discoloration: 0.1,
    }
}
