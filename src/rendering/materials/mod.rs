//! Material definitions for tank components
//!
//! Pre-configured materials using procedural textures.

pub mod armor;
pub mod tracks;
pub mod steel;

pub use armor::*;
pub use tracks::*;
pub use steel::*;

use bevy::prelude::*;
use super::texture_generator::PbrTextureSet;

/// Create a StandardMaterial from a PBR texture set
pub fn material_from_textures(
    images: &mut Assets<Image>,
    textures: PbrTextureSet,
) -> StandardMaterial {
    let base_color_texture = images.add(textures.base_color);
    let normal_map_texture = images.add(textures.normal);
    let metallic_roughness_texture = images.add(textures.metallic_roughness);

    StandardMaterial {
        base_color_texture: Some(base_color_texture),
        normal_map_texture: Some(normal_map_texture),
        metallic_roughness_texture: Some(metallic_roughness_texture),
        metallic: 1.0,  // Will be modulated by texture
        perceptual_roughness: 1.0,  // Will be modulated by texture
        ..default()
    }
}

/// Resource holding all tank material handles
#[derive(Resource)]
pub struct TankMaterials {
    pub armor: Handle<StandardMaterial>,
    pub tracks: Handle<StandardMaterial>,
    pub barrel_steel: Handle<StandardMaterial>,
    pub wheel_rubber: Handle<StandardMaterial>,
}

/// System to generate and register tank materials
pub fn setup_tank_materials(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Generate armor textures
    let armor_textures = super::texture_generator::generate_armor_textures(
        &super::texture_generator::ArmorTextureConfig::default()
    );
    let armor_material = material_from_textures(&mut images, armor_textures);
    let armor_handle = materials.add(armor_material);

    // Generate track textures
    let track_textures = super::texture_generator::generate_track_textures(
        &super::texture_generator::TrackTextureConfig::default()
    );
    let track_material = material_from_textures(&mut images, track_textures);
    let tracks_handle = materials.add(track_material);

    // Generate barrel steel textures
    let steel_textures = super::texture_generator::generate_steel_textures(
        &super::texture_generator::SteelTextureConfig::default()
    );
    let steel_material = material_from_textures(&mut images, steel_textures);
    let barrel_handle = materials.add(steel_material);

    // Simple rubber material for wheels
    let wheel_rubber = materials.add(StandardMaterial {
        base_color: Color::srgb(0.1, 0.1, 0.1),
        metallic: 0.0,
        perceptual_roughness: 0.9,
        ..default()
    });

    commands.insert_resource(TankMaterials {
        armor: armor_handle,
        tracks: tracks_handle,
        barrel_steel: barrel_handle,
        wheel_rubber,
    });
}
