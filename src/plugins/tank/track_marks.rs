use bevy::prelude::*;
use crate::components::{Tank, RaycastSuspension, TankVelocities};

// ============================================================================
// ŚLADY GĄSIENIC
// ============================================================================

/// Komponent dla pojedynczego śladu gąsienicy
#[derive(Component)]
pub struct TrackMark {
    /// Czas utworzenia śladu
    pub spawn_time: f32,
    /// Czas życia śladu [s]
    pub lifetime: f32,
    /// Początkowa przezroczystość
    pub initial_opacity: f32,
}

/// Konfiguracja systemu śladów gąsienic
#[derive(Resource)]
pub struct TrackMarkConfig {
    /// Maksymalna liczba śladów w świecie
    pub max_marks: usize,
    /// Minimalna odległość między śladami [m]
    pub min_distance: f32,
    /// Czas życia śladu [s]
    pub lifetime: f32,
    /// Szerokość śladu [m]
    pub mark_width: f32,
    /// Długość śladu [m]
    pub mark_length: f32,
    /// Ostatnia pozycja lewego śladu
    pub last_left_pos: Vec3,
    /// Ostatnia pozycja prawego śladu
    pub last_right_pos: Vec3,
    /// Czy system jest aktywny
    pub enabled: bool,
}

impl Default for TrackMarkConfig {
    fn default() -> Self {
        Self {
            max_marks: 500,
            min_distance: 0.5,
            lifetime: 30.0,
            mark_width: 0.35,
            mark_length: 0.6,
            last_left_pos: Vec3::ZERO,
            last_right_pos: Vec3::ZERO,
            enabled: true,
        }
    }
}

/// Oblicza średnią pozycję z listy punktów
fn average_position(positions: &[Vec3]) -> Option<Vec3> {
    if positions.is_empty() {
        return None;
    }
    let sum: Vec3 = positions.iter().copied().sum();
    Some(sum / positions.len() as f32)
}

/// Tworzy pojedynczy ślad gąsienicy
fn spawn_mark(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    position: Vec3,
    tank_yaw: f32,
    config: &TrackMarkConfig,
    current_time: f32,
) {
    // Materiał śladu - ciemnobrązowy, półprzezroczysty
    let mark_material = materials.add(StandardMaterial {
        base_color: Color::srgba(0.12, 0.10, 0.08, 0.65),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        ..default()
    });

    // Płaska płaszczyzna reprezentująca ślad
    let mesh = meshes.add(Plane3d::new(
        Vec3::Y,
        Vec2::new(config.mark_length, config.mark_width),
    ));

    commands.spawn((
        TrackMark {
            spawn_time: current_time,
            lifetime: config.lifetime,
            initial_opacity: 0.65,
        },
        Mesh3d(mesh),
        MeshMaterial3d(mark_material),
        Transform::from_translation(position + Vec3::Y * 0.005) // Lekko nad terenem
            .with_rotation(Quat::from_rotation_y(tank_yaw)),
    ));
}

/// System tworzenia śladów gąsienic w punktach kontaktu z terenem
pub fn spawn_track_marks(
    time: Res<Time>,
    mut config: ResMut<TrackMarkConfig>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    tank_query: Query<(&Transform, &RaycastSuspension, &TankVelocities), With<Tank>>,
    mark_query: Query<Entity, With<TrackMark>>,
) {
    if !config.enabled {
        return;
    }

    let current_time = time.elapsed_secs();

    let Ok((tank_transform, suspension, velocities)) = tank_query.get_single() else {
        return;
    };

    // Sprawdź limit śladów
    let mark_count = mark_query.iter().count();
    if mark_count >= config.max_marks {
        return; // System fade usunie stare ślady
    }

    // Tylko twórz ślady gdy czołg się porusza
    if !velocities.is_moving && velocities.linear_speed.abs() < 0.1 {
        return;
    }

    // Pobierz yaw czołgu dla orientacji śladów
    let (yaw, _, _) = tank_transform.rotation.to_euler(EulerRot::YXZ);

    // Pobierz punkty kontaktu dla lewej gąsienicy (indeksy 0-3)
    let left_contacts: Vec<Vec3> = suspension.suspension_points[0..4]
        .iter()
        .filter(|p| p.grounded)
        .map(|p| p.contact_point)
        .collect();

    // Pobierz punkty kontaktu dla prawej gąsienicy (indeksy 4-7)
    let right_contacts: Vec<Vec3> = suspension.suspension_points[4..8]
        .iter()
        .filter(|p| p.grounded)
        .map(|p| p.contact_point)
        .collect();

    // Utwórz ślad lewej gąsienicy
    if let Some(left_pos) = average_position(&left_contacts) {
        if left_pos.distance(config.last_left_pos) >= config.min_distance {
            spawn_mark(
                &mut commands,
                &mut meshes,
                &mut materials,
                left_pos,
                yaw,
                &config,
                current_time,
            );
            config.last_left_pos = left_pos;
        }
    }

    // Utwórz ślad prawej gąsienicy
    if let Some(right_pos) = average_position(&right_contacts) {
        if right_pos.distance(config.last_right_pos) >= config.min_distance {
            spawn_mark(
                &mut commands,
                &mut meshes,
                &mut materials,
                right_pos,
                yaw,
                &config,
                current_time,
            );
            config.last_right_pos = right_pos;
        }
    }
}

/// System zanikania i usuwania starych śladów gąsienic
pub fn fade_track_marks(
    time: Res<Time>,
    mut commands: Commands,
    query: Query<(Entity, &TrackMark, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let current_time = time.elapsed_secs();

    for (entity, mark, material_handle) in query.iter() {
        let age = current_time - mark.spawn_time;

        // Usuń stare ślady
        if age >= mark.lifetime {
            commands.entity(entity).despawn_recursive();
            continue;
        }

        // Zanikanie w ostatnich 20% czasu życia
        let fade_start = mark.lifetime * 0.8;
        if age > fade_start {
            let fade_progress = (age - fade_start) / (mark.lifetime - fade_start);
            let new_opacity = mark.initial_opacity * (1.0 - fade_progress);

            if let Some(material) = materials.get_mut(&material_handle.0) {
                material.base_color = material.base_color.with_alpha(new_opacity);
            }
        }
    }
}
