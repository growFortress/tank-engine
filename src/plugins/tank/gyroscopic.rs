use bevy::prelude::*;
use crate::components::{Tank, Turret, TurretVelocity, TurretGyroscopic, RigidBody6DOF};

/// System efektu żyroskopowego wieży
///
/// Gdy wieża obraca się podczas ruchu kadłuba, powstaje moment żyroskopowy
/// który powoduje reakcję kadłuba (precesja). Efekt jest subtelny ale zauważalny
/// przy szybkim obrocie wieży podczas jazdy.
///
/// Fizyka: τ_gyro = I_turret × ω_turret × ω_hull
/// - I_turret: moment bezwładności wieży
/// - ω_turret: prędkość kątowa wieży (obrót wokół Y)
/// - ω_hull: prędkość kątowa kadłuba
pub fn apply_gyroscopic_effect(
    turret_query: Query<(&TurretVelocity, &TurretGyroscopic, &Parent), With<Turret>>,
    mut hull_query: Query<(&Transform, &mut RigidBody6DOF), With<Tank>>,
) {
    for (turret_vel, gyro, parent) in turret_query.iter() {
        let Ok((hull_transform, mut hull_rb)) = hull_query.get_mut(parent.get()) else {
            continue;
        };

        // Prędkość kątowa wieży (obrót wokół lokalnej osi Y kadłuba)
        let turret_omega = turret_vel.rotation_speed;

        // Jeśli wieża się nie obraca, brak efektu
        if turret_omega.abs() < 0.01 {
            continue;
        }

        // Prędkość kątowa kadłuba
        let hull_omega = hull_rb.angular_velocity;

        // Oś obrotu wieży (Y kadłuba w przestrzeni świata)
        let turret_rotation_axis = hull_transform.up();

        // Moment żyroskopowy: τ = I × (ω_turret × ω_hull)
        // Uproszczona wersja: iloczyn wektorowy daje kierunek precesji
        // Gdy wieża obraca się wokół Y, a kadłub ma prędkość kątową,
        // powstaje moment prostopadły do obu osi
        let precession_torque = turret_rotation_axis.cross(hull_omega)
            * gyro.turret_inertia_y
            * turret_omega
            * gyro.gyro_coupling;

        // Aplikuj moment do kadłuba
        hull_rb.apply_torque(precession_torque);
    }
}
