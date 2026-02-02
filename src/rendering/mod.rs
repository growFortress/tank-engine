//! Procedural texture generation for tank materials
//!
//! Provides noise-based PBR texture generation for realistic tank surfaces.

pub mod noise;
pub mod texture_generator;
pub mod materials;

pub use noise::*;
pub use texture_generator::*;
pub use materials::*;
