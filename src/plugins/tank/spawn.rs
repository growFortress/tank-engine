//! Tank spawning system with procedural geometry
//!
//! Uses ultra-detailed procedural meshes (~50k+ triangles) and PBR textures.

use bevy::prelude::*;
use std::f32::consts::PI;
use crate::components::{
    Tank, Turret, GunMount, Barrel, MuzzlePoint,
    TankVelocities, GunDispersion, TurretVelocity, TurretGyroscopic,
    TankMobility, TrackPhysics, TerrainResistance,
    SuspensionState, DriftState, SlopeState, TankInput,
    // Nowe komponenty fizyki
    RigidBody6DOF, RaycastSuspension, CompoundCollider, CompoundShape,
    BoxCollider, WorldAABB,
};

use super::model::{
    generate_hull_mesh, HullConfig,
    generate_turret_mesh, TurretConfig,
    generate_barrel_mesh, BarrelConfig,
    generate_road_wheel, RoadWheelConfig,
    generate_drive_sprocket, SprocketConfig,
    generate_idler_wheel, IdlerConfig,
    generate_track_link, TrackLinkConfig,
    calculate_track_link_transforms, TrackPath, TrackLinkMarker,
};

use crate::rendering::{
    generate_armor_textures, ArmorTextureConfig,
    generate_track_textures, TrackTextureConfig,
    generate_steel_textures, SteelTextureConfig,
    material_from_textures,
};

/// Pre-generated tank mesh and material handles
#[derive(Resource)]
pub struct TankAssets {
    // Meshes
    pub hull_mesh: Handle<Mesh>,
    pub turret_mesh: Handle<Mesh>,
    pub barrel_mesh: Handle<Mesh>,
    pub road_wheel_mesh: Handle<Mesh>,
    pub drive_sprocket_mesh: Handle<Mesh>,
    pub idler_wheel_mesh: Handle<Mesh>,
    pub track_link_mesh: Handle<Mesh>,

    // Materials
    pub armor_material: Handle<StandardMaterial>,
    pub armor_dark_material: Handle<StandardMaterial>,
    pub track_material: Handle<StandardMaterial>,
    pub barrel_material: Handle<StandardMaterial>,
    pub wheel_material: Handle<StandardMaterial>,
}

/// Generate all tank assets at startup
pub fn setup_tank_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // Generate procedural meshes
    let hull_mesh = meshes.add(generate_hull_mesh(&HullConfig::t54()));
    let turret_mesh = meshes.add(generate_turret_mesh(&TurretConfig::t54()));
    let barrel_mesh = meshes.add(generate_barrel_mesh(&BarrelConfig::d10t()));
    let road_wheel_mesh = meshes.add(generate_road_wheel(&RoadWheelConfig::t54()));
    let drive_sprocket_mesh = meshes.add(generate_drive_sprocket(&SprocketConfig::t54()));
    let idler_wheel_mesh = meshes.add(generate_idler_wheel(&IdlerConfig::t54()));
    let track_link_mesh = meshes.add(generate_track_link(&TrackLinkConfig::t54()));

    // Generate procedural textures
    let armor_textures = generate_armor_textures(&ArmorTextureConfig::default());
    let armor_material = materials.add(material_from_textures(&mut images, armor_textures));

    // Darker armor variant
    let armor_dark_textures = generate_armor_textures(&ArmorTextureConfig {
        base_color: [0.18, 0.22, 0.14],
        rust_amount: 0.2,
        scratch_amount: 0.35,
        roughness: 0.55,
        ..ArmorTextureConfig::default()
    });
    let armor_dark_material = materials.add(material_from_textures(&mut images, armor_dark_textures));

    // Track textures
    let track_textures = generate_track_textures(&TrackTextureConfig::default());
    let track_material = materials.add(material_from_textures(&mut images, track_textures));

    // Barrel steel textures
    let steel_textures = generate_steel_textures(&SteelTextureConfig::default());
    let barrel_material = materials.add(material_from_textures(&mut images, steel_textures));

    // Simple wheel rubber material
    let wheel_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.08, 0.06),
        metallic: 0.50,
        perceptual_roughness: 0.65,
        reflectance: 0.25,
        ..default()
    });

    commands.insert_resource(TankAssets {
        hull_mesh,
        turret_mesh,
        barrel_mesh,
        road_wheel_mesh,
        drive_sprocket_mesh,
        idler_wheel_mesh,
        track_link_mesh,
        armor_material,
        armor_dark_material,
        track_material,
        barrel_material,
        wheel_material,
    });
}

/// Spawn tank using pre-generated assets
pub fn spawn_tank(
    mut commands: Commands,
    assets: Option<Res<TankAssets>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    // If assets aren't loaded yet, generate them inline (fallback)
    let assets = if let Some(assets) = assets {
        assets.into_inner().clone()
    } else {
        generate_assets_inline(&mut meshes, &mut materials, &mut images)
    };

    // Compound collider dla kadłuba czołgu - PEŁNE POKRYCIE
    let tank_compound_collider = CompoundCollider::new(vec![
        // === KADŁUB GŁÓWNY ===
        CompoundShape {
            offset: Vec3::new(0.0, 0.45, 0.0),
            rotation: Quat::IDENTITY,
            collider: BoxCollider::new(Vec3::new(2.1, 0.35, 1.45)),
        },
        CompoundShape {
            offset: Vec3::new(0.0, 0.85, 0.0),
            rotation: Quat::IDENTITY,
            collider: BoxCollider::new(Vec3::new(1.8, 0.15, 1.2)),
        },
        CompoundShape {
            offset: Vec3::new(1.7, 0.55, 0.0),
            rotation: Quat::IDENTITY,
            collider: BoxCollider::new(Vec3::new(0.4, 0.35, 1.2)),
        },
        CompoundShape {
            offset: Vec3::new(-1.8, 0.50, 0.0),
            rotation: Quat::IDENTITY,
            collider: BoxCollider::new(Vec3::new(0.3, 0.35, 1.2)),
        },
        // === GĄSIENICE ===
        CompoundShape {
            offset: Vec3::new(0.0, 0.35, -1.32),
            rotation: Quat::IDENTITY,
            collider: BoxCollider::new(Vec3::new(2.0, 0.35, 0.22)),
        },
        CompoundShape {
            offset: Vec3::new(0.0, 0.35, 1.32),
            rotation: Quat::IDENTITY,
            collider: BoxCollider::new(Vec3::new(2.0, 0.35, 0.22)),
        },
        // === WIEŻA ===
        CompoundShape {
            offset: Vec3::new(0.25, 1.28, 0.0),
            rotation: Quat::IDENTITY,
            collider: BoxCollider::new(Vec3::new(1.0, 0.28, 0.80)),
        },
        CompoundShape {
            offset: Vec3::new(0.85, 1.25, 0.0),
            rotation: Quat::IDENTITY,
            collider: BoxCollider::new(Vec3::new(0.25, 0.25, 0.70)),
        },
    ]);

    // Spawn tank entity
    let tank_id = commands
        .spawn((
            Tank,
            TankInput::default(),
            TankMobility::default(),
            TrackPhysics::default(),
            TerrainResistance::default(),
            SuspensionState::default(),
            DriftState::default(),
            SlopeState::default(),
        ))
        .id();

    commands.entity(tank_id).insert((
        TankVelocities::default(),
        GunDispersion::default(),
        RigidBody6DOF::tank(45000.0),
        RaycastSuspension::default(),
        tank_compound_collider,
        WorldAABB::default(),
        Transform::from_xyz(0.0, 1.0, 0.0),
        Visibility::default(),
    ));

    commands.entity(tank_id).with_children(|parent| {
        // === PROCEDURAL HULL ===
        parent.spawn((
            Mesh3d(assets.hull_mesh.clone()),
            MeshMaterial3d(assets.armor_material.clone()),
            Transform::IDENTITY,  // Hull mesh Y=0 is bottom
        ));

        // === WHEELS AND TRACKS (both sides) ===
        for z_sign in [-1.0_f32, 1.0] {
            let z_offset = z_sign * 1.10;

            // Drive sprocket (front)
            parent.spawn((
                Mesh3d(assets.drive_sprocket_mesh.clone()),
                MeshMaterial3d(assets.wheel_material.clone()),
                Transform::from_xyz(1.65, 0.45, z_offset)
                    .with_rotation(Quat::from_rotation_x(PI / 2.0)),
            ));

            // Idler wheel (rear)
            parent.spawn((
                Mesh3d(assets.idler_wheel_mesh.clone()),
                MeshMaterial3d(assets.wheel_material.clone()),
                Transform::from_xyz(-1.65, 0.45, z_offset)
                    .with_rotation(Quat::from_rotation_x(PI / 2.0)),
            ));

            // Road wheels (5 per side)
            for i in 0..5 {
                let x = -1.1 + i as f32 * 0.55;
                parent.spawn((
                    Mesh3d(assets.road_wheel_mesh.clone()),
                    MeshMaterial3d(assets.wheel_material.clone()),
                    Transform::from_xyz(x, 0.28, z_offset)
                        .with_rotation(Quat::from_rotation_x(PI / 2.0)),
                ));
            }

            // Return rollers (3 per side)
            for i in 0..3 {
                let x = -0.6 + i as f32 * 0.6;
                parent.spawn((
                    Mesh3d(assets.road_wheel_mesh.clone()),
                    MeshMaterial3d(assets.wheel_material.clone()),
                    Transform::from_xyz(x, 0.68, z_offset)
                        .with_rotation(Quat::from_rotation_x(PI / 2.0))
                        .with_scale(Vec3::splat(0.35)),
                ));
            }

            // Track links (instanced)
            let track_path = TrackPath::t54();
            let track_config = TrackLinkConfig::t54();
            let link_transforms = calculate_track_link_transforms(&track_path, &track_config, z_offset);

            for (index, transform) in link_transforms.iter().enumerate() {
                parent.spawn((
                    Mesh3d(assets.track_link_mesh.clone()),
                    MeshMaterial3d(assets.track_material.clone()),
                    *transform,
                    TrackLinkMarker {
                        index: index as u32,
                        side: z_sign,
                    },
                ));
            }
        }
    });

    // === TURRET ===
    let turret_id = commands
        .spawn((
            Turret {
                traverse_speed: 26.0,
            },
            TurretVelocity::default(),
            TurretGyroscopic::default(),
            Transform::from_xyz(0.25, 1.0, 0.0),
            Visibility::default(),
        ))
        .id();

    commands.entity(tank_id).add_child(turret_id);

    commands.entity(turret_id).with_children(|parent| {
        // Procedural turret mesh
        parent.spawn((
            Mesh3d(assets.turret_mesh.clone()),
            MeshMaterial3d(assets.armor_material.clone()),
            Transform::IDENTITY,
        ));
    });

    // === GUN MOUNT ===
    let mount_id = commands
        .spawn((
            GunMount,
            Transform::from_xyz(0.75, 0.28, 0.0),
            Visibility::default(),
        ))
        .id();

    commands.entity(turret_id).add_child(mount_id);

    commands.entity(mount_id).with_children(|parent| {
        // Mantlet (simple box for now, can be replaced with detailed mesh)
        parent.spawn((
            Mesh3d(meshes.add(Cuboid::new(0.25, 0.38, 0.50))),
            MeshMaterial3d(assets.armor_material.clone()),
            Transform::from_xyz(0.10, 0.0, 0.0),
        ));
    });

    // === BARREL ===
    let barrel_id = commands
        .spawn((
            Barrel {
                elevation_speed: 18.0,
                max_elevation: 25.0,
                max_depression: -8.0,
            },
            Transform::from_xyz(0.28, 0.0, 0.0),
            Visibility::default(),
        ))
        .id();

    commands.entity(mount_id).add_child(barrel_id);

    commands.entity(barrel_id).with_children(|parent| {
        // Procedural barrel mesh (already oriented along X axis)
        parent.spawn((
            Mesh3d(assets.barrel_mesh.clone()),
            MeshMaterial3d(assets.barrel_material.clone()),
            Transform::from_xyz(0.0, 0.0, 0.0),
        ));

        // Muzzle point
        parent.spawn((
            MuzzlePoint,
            Transform::from_xyz(3.0, 0.0, 0.0),
            Visibility::default(),
        ));
    });

    print_tank_info();
}

/// Generate assets inline if not preloaded
fn generate_assets_inline(
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
) -> TankAssets {
    let hull_mesh = meshes.add(generate_hull_mesh(&HullConfig::t54()));
    let turret_mesh = meshes.add(generate_turret_mesh(&TurretConfig::t54()));
    let barrel_mesh = meshes.add(generate_barrel_mesh(&BarrelConfig::d10t()));
    let road_wheel_mesh = meshes.add(generate_road_wheel(&RoadWheelConfig::t54()));
    let drive_sprocket_mesh = meshes.add(generate_drive_sprocket(&SprocketConfig::t54()));
    let idler_wheel_mesh = meshes.add(generate_idler_wheel(&IdlerConfig::t54()));
    let track_link_mesh = meshes.add(generate_track_link(&TrackLinkConfig::t54()));

    let armor_textures = generate_armor_textures(&ArmorTextureConfig::default());
    let armor_material = materials.add(material_from_textures(images, armor_textures));

    let armor_dark_textures = generate_armor_textures(&ArmorTextureConfig {
        base_color: [0.18, 0.22, 0.14],
        rust_amount: 0.2,
        scratch_amount: 0.35,
        roughness: 0.55,
        ..ArmorTextureConfig::default()
    });
    let armor_dark_material = materials.add(material_from_textures(images, armor_dark_textures));

    let track_textures = generate_track_textures(&TrackTextureConfig::default());
    let track_material = materials.add(material_from_textures(images, track_textures));

    let steel_textures = generate_steel_textures(&SteelTextureConfig::default());
    let barrel_material = materials.add(material_from_textures(images, steel_textures));

    let wheel_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.08, 0.08, 0.06),
        metallic: 0.50,
        perceptual_roughness: 0.65,
        reflectance: 0.25,
        ..default()
    });

    TankAssets {
        hull_mesh,
        turret_mesh,
        barrel_mesh,
        road_wheel_mesh,
        drive_sprocket_mesh,
        idler_wheel_mesh,
        track_link_mesh,
        armor_material,
        armor_dark_material,
        track_material,
        barrel_material,
        wheel_material,
    }
}

impl Clone for TankAssets {
    fn clone(&self) -> Self {
        Self {
            hull_mesh: self.hull_mesh.clone(),
            turret_mesh: self.turret_mesh.clone(),
            barrel_mesh: self.barrel_mesh.clone(),
            road_wheel_mesh: self.road_wheel_mesh.clone(),
            drive_sprocket_mesh: self.drive_sprocket_mesh.clone(),
            idler_wheel_mesh: self.idler_wheel_mesh.clone(),
            track_link_mesh: self.track_link_mesh.clone(),
            armor_material: self.armor_material.clone(),
            armor_dark_material: self.armor_dark_material.clone(),
            track_material: self.track_material.clone(),
            barrel_material: self.barrel_material.clone(),
            wheel_material: self.wheel_material.clone(),
        }
    }
}

fn print_tank_info() {
    println!("════════════════════════════════════════════════════");
    println!("  TANK 3D - Ultra-Detail Procedural Model");
    println!("════════════════════════════════════════════════════");
    println!("  MODEL FEATURES:");
    println!("  - Procedural geometry (~50k+ triangles)");
    println!("  - PBR textures (rust, scratches, wear)");
    println!("  - Detailed track links (GPU instanced)");
    println!("  - Beveled edges and smooth normals");
    println!("════════════════════════════════════════════════════");
    println!("  CONTROLS:");
    println!("  W/S         = Forward/Reverse");
    println!("  A/D/Arrows  = Hull rotation");
    println!("  Q           = Left track brake (pivot right)");
    println!("  E           = Right track brake (pivot left)");
    println!("  SPACE       = Full brake (both tracks)");
    println!("  MOUSE       = Aim direction (camera)");
    println!("  SCROLL      = Zoom");
    println!("  SHIFT       = Sniper mode (free look around turret)");
    println!("  ESC         = Show/hide cursor");
    println!("════════════════════════════════════════════════════");
    println!("  TANK SPECIFICATIONS (T-54/55 based):");
    println!("  Mass:         45,000 kg (45 tons)");
    println!("  Engine:       750 hp");
    println!("  Power/Weight: 16.7 hp/t");
    println!("  Max Speed:    50 km/h forward, 20 km/h reverse");
    println!("  Hull Traverse: 32 deg/s");
    println!("  Turret Traverse: 26 deg/s");
    println!("════════════════════════════════════════════════════");
    println!("  PHYSICS FEATURES:");
    println!("  - Engine torque curve (peak at 1600 RPM)");
    println!("  - Track slip ratio (Pacejka-style traction)");
    println!("  - Asymmetric suspension damping");
    println!("  - Orientation-dependent air drag");
    println!("  - Independent track brakes (Q/E)");
    println!("  - Turret gyroscopic effect");
    println!("  - Visual track marks on terrain");
    println!("  - Terrain resistance (hard/medium/soft)");
    println!("  - Slope climbing (max 35 degrees)");
    println!("════════════════════════════════════════════════════");
}
