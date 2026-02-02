//! Procedural mesh generation for detailed tank model
//!
//! This module provides high-detail mesh generation for T-54/55 tank
//! with ~50k triangles total.

pub mod builder;
pub mod utils;
pub mod hull;
pub mod turret;
pub mod barrel;
pub mod wheels;
pub mod tracks;

pub use builder::*;
pub use hull::{generate_hull_mesh, HullConfig};
pub use turret::{generate_turret_mesh, TurretConfig};
pub use barrel::{generate_barrel_mesh, BarrelConfig};
pub use wheels::{
    generate_road_wheel, RoadWheelConfig,
    generate_drive_sprocket, SprocketConfig,
    generate_idler_wheel, IdlerConfig,
};
pub use tracks::{
    generate_track_link, TrackLinkConfig,
    calculate_track_link_transforms, TrackPath, TrackLinkMarker,
};
