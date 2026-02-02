pub mod model;
mod spawn;
mod input;
mod turret;
mod barrel;
mod velocity;
mod dispersion;
mod physics;
mod terrain_detection;
// Nowe moduły fizyki 6DOF
mod suspension;
mod forces;
mod integration;
mod collision;
mod gyroscopic;
mod track_marks;

use bevy::prelude::*;
use crate::systems::TankGameSet;
use crate::resources::TerrainInfo;
use crate::components::DestructionEvent;

use spawn::spawn_tank;
use input::read_tank_input;
use turret::rotate_turret_to_aim;
use barrel::elevate_barrel_to_aim;
use velocity::{track_tank_velocity, track_turret_velocity};
use dispersion::calculate_dispersion;
use physics::update_track_physics;
use terrain_detection::detect_terrain;

// Nowe systemy fizyki
use suspension::update_raycast_suspension;
use forces::{apply_track_forces, sync_mobility_speed};
use integration::integrate_rigid_body;
use collision::{
    update_world_aabbs, update_compound_aabbs,
    detect_and_resolve_collisions, handle_destruction,
    predict_and_prevent_tunneling,
};
use gyroscopic::apply_gyroscopic_effect;
use track_marks::{spawn_track_marks, fade_track_marks, TrackMarkConfig};

/// Plugin for tank entity and control systems
pub struct TankPlugin;

impl Plugin for TankPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<TerrainInfo>();
        app.init_resource::<TrackMarkConfig>();

        // Register events
        app.add_event::<DestructionEvent>();

        app.add_systems(Startup, spawn_tank)
            // === INPUT PHASE ===
            // Tank input reading with smoothing
            .add_systems(
                Update,
                read_tank_input.in_set(TankGameSet::TankInputRead),
            )

            // === TERRAIN DETECTION ===
            // Detect terrain type and slope (still needed for resistance/grip)
            .add_systems(
                Update,
                detect_terrain.in_set(TankGameSet::TerrainDetection),
            )

            // === PHYSICS CALCULATION ===
            // Track physics - calculate target track speeds from input
            .add_systems(
                Update,
                update_track_physics.in_set(TankGameSet::PhysicsCalculation),
            )

            // === FORCE APPLICATION ===
            // Apply forces based on track physics, terrain, slopes
            // This replaces the old direct movement systems
            .add_systems(
                Update,
                apply_track_forces.in_set(TankGameSet::TankMovement),
            )

            // === GYROSCOPIC EFFECT ===
            // Turret rotation affects hull stability
            .add_systems(
                Update,
                apply_gyroscopic_effect
                    .after(TankGameSet::TankMovement)
                    .before(TankGameSet::SuspensionUpdate),
            )

            // === SUSPENSION ===
            // Raycast suspension - applies spring/damper forces
            .add_systems(
                Update,
                update_raycast_suspension.in_set(TankGameSet::SuspensionUpdate),
            )

            // === TRACK MARKS ===
            // Spawn and fade track marks on terrain
            .add_systems(
                Update,
                (spawn_track_marks, fade_track_marks)
                    .chain()
                    .after(TankGameSet::SuspensionUpdate)
                    .before(TankGameSet::VelocityTracking),
            )

            // === COLLISION DETECTION ===
            // Update AABBs, predict tunneling (CCD), and detect collisions
            .add_systems(
                Update,
                (
                    update_world_aabbs,
                    update_compound_aabbs,
                    predict_and_prevent_tunneling, // CCD dla szybkich obiektów
                    detect_and_resolve_collisions,
                )
                    .chain()
                    .after(TankGameSet::SuspensionUpdate)
                    .before(TankGameSet::VelocityTracking),
            )

            // === PHYSICS INTEGRATION ===
            // Integrate rigid body - THIS is the only system that modifies Transform
            .add_systems(
                Update,
                integrate_rigid_body
                    .after(detect_and_resolve_collisions)
                    .before(TankGameSet::VelocityTracking),
            )

            // === DESTRUCTION HANDLING ===
            .add_systems(
                Update,
                handle_destruction.after(detect_and_resolve_collisions),
            )

            // === VELOCITY TRACKING ===
            // Track velocities for dispersion calculation
            .add_systems(
                Update,
                (track_tank_velocity, track_turret_velocity, sync_mobility_speed)
                    .in_set(TankGameSet::VelocityTracking),
            )

            // === TURRET/BARREL CONTROL ===
            .add_systems(
                Update,
                rotate_turret_to_aim.in_set(TankGameSet::TurretControl),
            )
            .add_systems(
                Update,
                elevate_barrel_to_aim.in_set(TankGameSet::BarrelControl),
            )

            // === DISPERSION ===
            .add_systems(
                Update,
                calculate_dispersion.in_set(TankGameSet::DispersionCalculation),
            );
    }
}
