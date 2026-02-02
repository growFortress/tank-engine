use bevy::prelude::*;
use bevy::pbr::{DistanceFog, FogFalloff};
use bevy::core_pipeline::Skybox;
use bevy::render::render_resource::{TextureDimension, TextureFormat, Extent3d, TextureViewDescriptor, TextureViewDimension};
use bevy::render::render_asset::RenderAssetUsages;
use crate::components::GameCamera;

pub fn spawn_camera(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
) {
    // Create procedural sky cubemap
    let sky_image = create_gradient_sky_cubemap();
    let sky_handle = images.add(sky_image);

    commands.spawn((
        GameCamera::default(),
        Camera3d::default(),
        Projection::Perspective(PerspectiveProjection {
            fov: 60.0_f32.to_radians(),
            ..default()
        }),
        Transform::from_xyz(0.0, 5.0, 14.0),
        // Skybox with procedural gradient
        Skybox {
            image: sky_handle,
            brightness: 1000.0,
            rotation: Quat::IDENTITY,
        },
        // Atmospheric fog - blue mountain haze (reduced density for larger map)
        DistanceFog {
            color: Color::srgba(0.75, 0.82, 0.90, 1.0),
            falloff: FogFalloff::Exponential { density: 0.003 },
            ..default()
        },
    ));
}

/// Create a simple gradient sky cubemap programmatically
fn create_gradient_sky_cubemap() -> Image {
    let size = 128u32; // Smaller size for performance
    let mut data = Vec::with_capacity((size * size * 6 * 4) as usize);

    for face in 0..6u32 {
        for y in 0..size {
            for x in 0..size {
                let (r, g, b) = compute_sky_color(face, x, y, size);
                data.push((r * 255.0) as u8);
                data.push((g * 255.0) as u8);
                data.push((b * 255.0) as u8);
                data.push(255);
            }
        }
    }

    // Create as stacked 2D texture first (height = size * 6)
    let mut image = Image::new(
        Extent3d {
            width: size,
            height: size * 6,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    );

    // Reinterpret as cubemap array
    image.reinterpret_stacked_2d_as_array(6);

    // Set texture view dimension to Cube for skybox
    image.texture_view_descriptor = Some(TextureViewDescriptor {
        dimension: Some(TextureViewDimension::Cube),
        ..default()
    });

    image
}

/// Compute sky color for a cubemap texel
fn compute_sky_color(face: u32, x: u32, y: u32, size: u32) -> (f32, f32, f32) {
    // Calculate UV coordinates centered at 0
    let u = (x as f32 + 0.5) / size as f32 * 2.0 - 1.0;
    let v = (y as f32 + 0.5) / size as f32 * 2.0 - 1.0;

    // Get direction vector for this face
    let dir = match face {
        0 => Vec3::new(1.0, -v, -u),   // +X
        1 => Vec3::new(-1.0, -v, u),   // -X
        2 => Vec3::new(u, 1.0, v),     // +Y (top)
        3 => Vec3::new(u, -1.0, -v),   // -Y (bottom)
        4 => Vec3::new(u, -v, 1.0),    // +Z
        5 => Vec3::new(-u, -v, -1.0),  // -Z
        _ => Vec3::ZERO,
    }.normalize();

    // Sky gradient based on Y component
    // Italian sky - warm Mediterranean blue
    let zenith_color = (0.35, 0.55, 0.85);     // Deep blue sky
    let horizon_color = (0.78, 0.85, 0.95);    // Light blue/white horizon
    let ground_color = (0.38, 0.35, 0.30);     // Brown/olive ground

    if dir.y > 0.0 {
        // Above horizon - blend from horizon to zenith
        let t = dir.y.powf(0.6);
        (
            horizon_color.0 + (zenith_color.0 - horizon_color.0) * t,
            horizon_color.1 + (zenith_color.1 - horizon_color.1) * t,
            horizon_color.2 + (zenith_color.2 - horizon_color.2) * t,
        )
    } else {
        // Below horizon - blend to ground
        let t = (-dir.y).powf(0.4);
        (
            horizon_color.0 + (ground_color.0 - horizon_color.0) * t,
            horizon_color.1 + (ground_color.1 - horizon_color.1) * t,
            horizon_color.2 + (ground_color.2 - horizon_color.2) * t,
        )
    }
}
