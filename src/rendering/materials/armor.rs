//! Armor material configuration
//!
//! Soviet tank paint schemes and weathering presets.

use super::super::texture_generator::ArmorTextureConfig;

/// Standard Soviet olive green armor
pub fn soviet_olive() -> ArmorTextureConfig {
    ArmorTextureConfig {
        resolution: 1024,
        seed: 42,
        base_color: [0.25, 0.28, 0.18],  // 4BO olive
        rust_color: [0.45, 0.25, 0.12],
        rust_amount: 0.12,
        scratch_amount: 0.25,
        metallic: 0.85,
        roughness: 0.45,
        normal_strength: 1.0,
    }
}

/// Winter whitewash over olive
pub fn winter_whitewash() -> ArmorTextureConfig {
    ArmorTextureConfig {
        resolution: 1024,
        seed: 42,
        base_color: [0.85, 0.85, 0.82],  // Whitewash
        rust_color: [0.25, 0.28, 0.18],   // Olive showing through
        rust_amount: 0.35,  // More "wear" to show olive underneath
        scratch_amount: 0.4,
        metallic: 0.7,
        roughness: 0.55,
        normal_strength: 0.8,
    }
}

/// Desert sand camouflage
pub fn desert_sand() -> ArmorTextureConfig {
    ArmorTextureConfig {
        resolution: 1024,
        seed: 42,
        base_color: [0.65, 0.55, 0.40],  // Sand color
        rust_color: [0.50, 0.35, 0.25],
        rust_amount: 0.08,
        scratch_amount: 0.3,
        metallic: 0.80,
        roughness: 0.50,
        normal_strength: 1.0,
    }
}

/// Heavily weathered/abandoned tank
pub fn heavily_weathered() -> ArmorTextureConfig {
    ArmorTextureConfig {
        resolution: 1024,
        seed: 42,
        base_color: [0.22, 0.24, 0.18],  // Faded olive
        rust_color: [0.50, 0.28, 0.15],
        rust_amount: 0.45,  // Lots of rust
        scratch_amount: 0.5,
        metallic: 0.6,
        roughness: 0.7,
        normal_strength: 1.5,
    }
}

/// Factory fresh paint
pub fn factory_new() -> ArmorTextureConfig {
    ArmorTextureConfig {
        resolution: 1024,
        seed: 42,
        base_color: [0.28, 0.32, 0.22],  // Fresh olive
        rust_color: [0.45, 0.25, 0.12],
        rust_amount: 0.0,  // No rust
        scratch_amount: 0.05,  // Minimal scratches
        metallic: 0.9,
        roughness: 0.35,
        normal_strength: 0.6,
    }
}
