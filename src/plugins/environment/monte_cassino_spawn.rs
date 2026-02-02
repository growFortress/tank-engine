use bevy::prelude::*;
use crate::resources::{
    MonteCassinoTerrain, generate_monte_cassino_heights,
    get_rapido_river, get_gustav_line_fortifications, get_abbey_data,
    get_cassino_town_buildings, FortificationType,
};
use crate::components::{BoxCollider, WorldAABB, StaticBody, Destructible};
use super::spawn::TerrainMesh;
use super::terrain_mesh::{generate_terrain_mesh, generate_river_mesh};

/// Spawn the complete Monte Cassino environment
pub fn spawn_monte_cassino_environment(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    // Generate and insert terrain resource
    let heights = generate_monte_cassino_heights();
    let terrain = MonteCassinoTerrain::new(heights);

    // Print terrain stats
    #[cfg(debug_assertions)]
    {
        use crate::resources::print_heightmap_stats;
        print_heightmap_stats(&terrain.heights);
    }

    // === LOAD TEXTURES ===
    let grass_color: Handle<Image> = asset_server.load("textures/terrain/grass_color.jpg");
    let grass_normal: Handle<Image> = asset_server.load("textures/terrain/grass_normal.jpg");
    let rock_color: Handle<Image> = asset_server.load("textures/terrain/rock_color.jpg");
    let ground_color: Handle<Image> = asset_server.load("textures/terrain/ground_color.jpg");

    // === MATERIALS ===
    // Terrain - textured with vertex color tinting for height variation
    let terrain_mat = materials.add(StandardMaterial {
        base_color: Color::WHITE, // Vertex colors will tint the texture
        base_color_texture: Some(grass_color),
        normal_map_texture: Some(grass_normal),
        perceptual_roughness: 0.85,
        reflectance: 0.1,
        // UV transform handled in mesh generation (50x tiling for 500 unit map)
        ..default()
    });

    // Water - translucent with strong reflections
    let water_mat = materials.add(StandardMaterial {
        base_color: Color::srgba(0.12, 0.28, 0.42, 0.55),
        perceptual_roughness: 0.02, // Very smooth surface
        metallic: 0.0,
        alpha_mode: AlphaMode::Blend,
        reflectance: 0.95, // Strong reflections
        ..default()
    });

    // Concrete - rough gray
    let concrete_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.52, 0.50, 0.48),
        perceptual_roughness: 0.98,
        metallic: 0.0,
        reflectance: 0.15,
        ..default()
    });

    // Bunker - rock texture (military concrete)
    let bunker_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.7, 0.7, 0.7), // Tint rock texture darker
        base_color_texture: Some(rock_color.clone()),
        perceptual_roughness: 0.95,
        metallic: 0.0,
        reflectance: 0.1,
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(2.0)),
        ..default()
    });

    // Abbey - warm white stone (tinted rock texture)
    let abbey_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(1.0, 0.95, 0.88), // Warm white tint
        base_color_texture: Some(rock_color),
        perceptual_roughness: 0.75,
        reflectance: 0.25,
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(3.0)),
        ..default()
    });

    // Ruins - ground/rubble texture
    let ruins_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.9, 0.75, 0.65), // Reddish tint
        base_color_texture: Some(ground_color),
        perceptual_roughness: 0.92,
        reflectance: 0.12,
        uv_transform: bevy::math::Affine2::from_scale(Vec2::splat(2.0)),
        ..default()
    });

    // === TERRAIN MESH ===
    // Generate terrain mesh with 256x256 subdivisions for 500x500 map
    let terrain_mesh = generate_terrain_mesh(&terrain, 256);

    commands.spawn((
        TerrainMesh,
        Name::new("Monte Cassino Terrain"),
        Mesh3d(meshes.add(terrain_mesh)),
        MeshMaterial3d(terrain_mat),
        Transform::IDENTITY,
    ));

    // === RIVER ===
    let river = get_rapido_river();
    let river_mesh = generate_river_mesh(
        &terrain,
        &river.center_points,
        &river.widths,
    );

    commands.spawn((
        Name::new("Rapido River"),
        Mesh3d(meshes.add(river_mesh)),
        MeshMaterial3d(water_mat),
        Transform::from_xyz(0.0, 0.2, 0.0),
    ));

    // === FORTIFICATIONS ===
    for fort in get_gustav_line_fortifications() {
        let height = terrain.sample_height(fort.position.x, fort.position.y);

        let mesh = match fort.fortification_type {
            FortificationType::Bunker => {
                meshes.add(Cuboid::new(fort.size.x, fort.size.y, fort.size.z))
            }
            FortificationType::Pillbox => {
                meshes.add(Cylinder::new(fort.size.x / 2.0, fort.size.y))
            }
            FortificationType::Trench => {
                // Trench is a long narrow depression - represented as a box
                meshes.add(Cuboid::new(fort.size.x, fort.size.y, fort.size.z))
            }
            FortificationType::Observation => {
                meshes.add(Cuboid::new(fort.size.x, fort.size.y, fort.size.z))
            }
        };

        let mat = match fort.fortification_type {
            FortificationType::Bunker | FortificationType::Pillbox => bunker_mat.clone(),
            FortificationType::Trench => concrete_mat.clone(),
            FortificationType::Observation => concrete_mat.clone(),
        };

        // Position fortification on terrain
        let y_offset = match fort.fortification_type {
            FortificationType::Trench => fort.size.y * 0.3, // Partially buried
            _ => fort.size.y / 2.0,
        };

        commands.spawn((
            Name::new(format!("{:?}", fort.fortification_type)),
            Mesh3d(mesh),
            MeshMaterial3d(mat),
            Transform::from_xyz(
                fort.position.x,
                height + y_offset,
                fort.position.y,
            ).with_rotation(Quat::from_rotation_y(fort.rotation)),
            // Collider dla fortyfikacji (niezniszczalne)
            BoxCollider::new(Vec3::new(
                fort.size.x / 2.0,
                fort.size.y / 2.0,
                fort.size.z / 2.0,
            )),
            WorldAABB::default(),
            StaticBody,
        ));
    }

    // === ABBEY (MONASTERY) ===
    let abbey = get_abbey_data();
    let abbey_height = terrain.sample_height(abbey.position.x, abbey.position.y);

    // Main abbey building
    commands.spawn((
        Name::new("Monte Cassino Abbey"),
        Mesh3d(meshes.add(Cuboid::new(abbey.size.x, abbey.size.y, abbey.size.z))),
        MeshMaterial3d(abbey_mat.clone()),
        Transform::from_xyz(
            abbey.position.x,
            abbey_height + abbey.size.y / 2.0,
            abbey.position.y,
        ),
        // Collider dla opactwa (niezniszczalne)
        BoxCollider::new(Vec3::new(abbey.size.x / 2.0, abbey.size.y / 2.0, abbey.size.z / 2.0)),
        WorldAABB::default(),
        StaticBody,
    ));

    // Abbey walls (thick walls as described - 46m high, 3m thick)
    let wall_height = 4.6; // Scaled
    let wall_thickness = 0.8;

    // Front wall
    commands.spawn((
        Name::new("Abbey Wall Front"),
        Mesh3d(meshes.add(Cuboid::new(abbey.size.x + 2.0, wall_height, wall_thickness))),
        MeshMaterial3d(abbey_mat.clone()),
        Transform::from_xyz(
            abbey.position.x,
            abbey_height + wall_height / 2.0,
            abbey.position.y - abbey.size.z / 2.0 - wall_thickness / 2.0,
        ),
        BoxCollider::new(Vec3::new((abbey.size.x + 2.0) / 2.0, wall_height / 2.0, wall_thickness / 2.0)),
        WorldAABB::default(),
        StaticBody,
    ));

    // Back wall
    commands.spawn((
        Name::new("Abbey Wall Back"),
        Mesh3d(meshes.add(Cuboid::new(abbey.size.x + 2.0, wall_height, wall_thickness))),
        MeshMaterial3d(abbey_mat.clone()),
        Transform::from_xyz(
            abbey.position.x,
            abbey_height + wall_height / 2.0,
            abbey.position.y + abbey.size.z / 2.0 + wall_thickness / 2.0,
        ),
        BoxCollider::new(Vec3::new((abbey.size.x + 2.0) / 2.0, wall_height / 2.0, wall_thickness / 2.0)),
        WorldAABB::default(),
        StaticBody,
    ));

    // === CASSINO TOWN RUINS ===
    for building in get_cassino_town_buildings() {
        let height = terrain.sample_height(building.position.x, building.position.y);

        // Destroyed buildings have reduced height
        let actual_height = if building.is_destroyed {
            building.size.y * 0.6
        } else {
            building.size.y
        };

        commands.spawn((
            Name::new("Town Ruin"),
            Mesh3d(meshes.add(Cuboid::new(building.size.x, actual_height, building.size.z))),
            MeshMaterial3d(ruins_mat.clone()),
            Transform::from_xyz(
                building.position.x,
                height + actual_height / 2.0,
                building.position.y,
            ).with_rotation(Quat::from_rotation_y(building.rotation)),
            // Collider dla budynków (niszczalne przez czołg)
            BoxCollider::new(Vec3::new(
                building.size.x / 2.0,
                actual_height / 2.0,
                building.size.z / 2.0,
            )),
            WorldAABB::default(),
            StaticBody,
            Destructible::medium(), // Średnia wytrzymałość
        ));
    }

    // === LIGHTING ===
    // Directional light (sun) - warm afternoon sun for dramatic mountain lighting
    // Position scaled 2.5x for larger 500x500 map
    commands.spawn((
        DirectionalLight {
            illuminance: 80000.0,
            shadows_enabled: true,
            color: Color::srgb(1.0, 0.95, 0.85), // Warm sunlight
            ..default()
        },
        Transform::from_xyz(200.0, 250.0, 100.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    // Ambient light - cool blue shadows for contrast
    commands.insert_resource(AmbientLight {
        color: Color::srgb(0.7, 0.8, 1.0),
        brightness: 300.0,
    });
    // Note: DistanceFog is added to camera in camera/spawn.rs

    // Insert terrain resource for physics
    commands.insert_resource(terrain);

    println!("════════════════════════════════════════════════════════════");
    println!("  MONTE CASSINO TERRAIN GENERATED");
    println!("  Map size: 500x500 units (5km x 5km real)");
    println!("  Grid resolution: 129x129 heightmap");
    println!("  Mesh resolution: 256x256 vertices");
    println!("  Height range: 0-37 units (40-593m real)");
    println!("  Features:");
    println!("    - Textured terrain with vertex colors");
    println!("    - Procedural skybox");
    println!("    - Rapido River");
    println!("    - Gustav Line fortifications");
    println!("    - Monte Cassino Abbey");
    println!("    - Cassino town ruins");
    println!("════════════════════════════════════════════════════════════");
}
