use bevy::prelude::*;
use crate::components::{
    Tank, TankInput, TankMobility, TrackPhysics,
    RigidBody6DOF, RaycastSuspension, TerrainResistance, SlopeState,
};

/// Stałe fizyczne
const GRAVITY: f32 = 9.81;
const HP_TO_WATTS: f32 = 745.7;
const AIR_DENSITY: f32 = 1.2; // kg/m³

// ============================================================================
// FUNKCJE POMOCNICZE DLA FIZYKI
// ============================================================================

/// Oblicza mnożnik momentu obrotowego na podstawie krzywej Gaussa
/// Maksymalny moment przy peak_rpm, spada symetrycznie w obie strony
fn calculate_torque_multiplier(
    current_rpm: f32,
    peak_rpm: f32,
    curve_width: f32,
    min_mult: f32,
) -> f32 {
    let rpm_diff = current_rpm - peak_rpm;
    // Krzywa Gaussa: e^(-(x-μ)²/(2σ²))
    let gaussian = (-rpm_diff.powi(2) / (2.0 * curve_width.powi(2))).exp();
    // Skalowanie od min_mult do 1.0
    min_mult + (1.0 - min_mult) * gaussian
}

/// Oblicza współczynnik poślizgu gąsienicy
/// slip = (track_speed - ground_speed) / max(|track_speed|, |ground_speed|)
fn calculate_slip_ratio(track_speed: f32, ground_speed: f32) -> f32 {
    let denominator = track_speed.abs().max(ground_speed.abs()).max(0.1);
    ((track_speed - ground_speed) / denominator).clamp(-1.0, 1.0)
}

/// Konwertuje poślizg na współczynnik trakcji (uproszczona krzywa Pacejki)
/// Trakcja rośnie do peak_slip, potem spada do min_traction
fn slip_to_traction(
    slip_ratio: f32,
    peak_slip: f32,
    critical_slip: f32,
    min_traction: f32,
) -> f32 {
    let abs_slip = slip_ratio.abs();

    if abs_slip <= peak_slip {
        // Strefa optymalna - lekki wzrost trakcji z poślizgiem
        1.0
    } else if abs_slip <= critical_slip {
        // Strefa przejściowa - stopniowy spadek
        let t = (abs_slip - peak_slip) / (critical_slip - peak_slip);
        1.0 - t * (1.0 - min_traction) * 0.5
    } else {
        // Strefa poślizgu - szybki spadek do minimum
        let excess = (abs_slip - critical_slip).min(0.5);
        (min_traction + 0.3).max(min_traction) - excess * 0.6
    }.clamp(min_traction, 1.0)
}

/// System napędu - aplikuje siły gąsienic do RigidBody6DOF
///
/// Zamiast bezpośrednio modyfikować Transform, aplikuje:
/// - Siłę napędową (forward/backward)
/// - Moment obrotowy (skręcanie)
/// - Tarcie boczne (zapobiega driftowi)
/// - Grawitację
pub fn apply_track_forces(
    mut query: Query<(
        &Transform,
        &TankInput,
        &TankMobility,
        &TrackPhysics,
        &RaycastSuspension,
        &TerrainResistance,
        &SlopeState,
        &mut RigidBody6DOF,
    ), With<Tank>>,
) {
    for (transform, input, mobility, tracks, suspension, terrain, slope, mut rb) in query.iter_mut() {
        // === SPRAWDŹ TRAKCJĘ ===
        let grounded_count = suspension.grounded_count();

        if grounded_count < suspension.min_ground_contacts {
            // Za mało kontaktu - tylko grawitacja
            let gravity_force = Vec3::new(0.0, -GRAVITY * rb.mass, 0.0);
            rb.apply_force(gravity_force);
            continue;
        }

        let traction_factor = grounded_count as f32 / suspension.suspension_points.len() as f32;

        // Kierunki lokalne czołgu
        let forward = transform.right(); // +X to przód czołgu
        let lateral = transform.back();  // -Z to bok (lewy)

        // === GRAWITACJA ===
        let gravity_force = Vec3::new(0.0, -GRAVITY * rb.mass, 0.0);
        rb.apply_force(gravity_force);

        // === SIŁA NAPĘDOWA ===
        // Target speed z fizyki gąsienic
        let target_speed = (tracks.left_track_speed + tracks.right_track_speed) / 2.0;

        // Aktualna prędkość w kierunku jazdy
        let current_forward_speed = rb.velocity.dot(*forward);
        let speed_diff = target_speed - current_forward_speed;

        // Siła silnika (ograniczona mocą)
        let engine_power_watts = mobility.engine_power_hp * HP_TO_WATTS;
        let current_speed_abs = rb.velocity.length().max(1.0); // Min 1 m/s dla obliczeń
        let max_engine_force = engine_power_watts / current_speed_abs;

        // === KRZYWA MOMENTU SILNIKA ===
        let torque_mult = calculate_torque_multiplier(
            mobility.current_rpm,
            mobility.peak_torque_rpm,
            mobility.torque_curve_width,
            mobility.min_torque_mult,
        );

        // === SLIP RATIO (poślizg gąsienic) ===
        let left_slip = calculate_slip_ratio(tracks.left_track_speed, current_forward_speed);
        let right_slip = calculate_slip_ratio(tracks.right_track_speed, current_forward_speed);

        let left_traction = slip_to_traction(
            left_slip,
            tracks.peak_slip_ratio,
            tracks.critical_slip_ratio,
            tracks.min_traction_coefficient,
        );
        let right_traction = slip_to_traction(
            right_slip,
            tracks.peak_slip_ratio,
            tracks.critical_slip_ratio,
            tracks.min_traction_coefficient,
        );
        let slip_traction_factor = (left_traction + right_traction) / 2.0;

        let terrain_modifier = 1.0 / terrain.current_resistance;

        // Proporcjonalna siła napędowa z ograniczeniem
        let drive_force = if input.braking {
            // Hamowanie - siła przeciwna do ruchu
            let brake_decel = 8.0; // m/s²
            let brake_force = -current_forward_speed.signum() * rb.mass * brake_decel;
            brake_force.clamp(-max_engine_force, max_engine_force)
        } else {
            // Normalna jazda z krzywą momentu i slip ratio
            let base_force = speed_diff * rb.mass * 6.0 * torque_mult;
            base_force.clamp(-max_engine_force, max_engine_force) * traction_factor * slip_traction_factor
        };

        // Aplikuj siłę napędową z uwzględnieniem oporu terenu
        rb.apply_force(forward * drive_force * terrain_modifier);

        // === MOMENT OBROTOWY (SKRĘCANIE) ===
        // Angular velocity z różnicy prędkości gąsienic
        let target_angular_vel = (tracks.right_track_speed - tracks.left_track_speed) / tracks.track_width;

        // Aktualna prędkość kątowa wokół Y (yaw)
        let current_angular_vel = rb.angular_velocity.y;
        let angular_diff = target_angular_vel - current_angular_vel;

        // Moment obrotowy proporcjonalny do różnicy
        // Przy stacjonarnym skręcaniu używamy większego momentu
        let stationary_factor = if current_forward_speed.abs() < 0.5 { 2.5 } else { 1.0 };
        let turn_torque = angular_diff * 120000.0 * traction_factor * terrain_modifier * stationary_factor;

        rb.torque.y += turn_torque;

        // === TARCIE BOCZNE (zapobiega driftowi) ===
        // Siła przeciwdziałająca ruchowi bocznemu
        let lateral_vel = rb.velocity.dot(*lateral);

        // ARCADE: wysokie tarcie boczne dla "sticky" feel (było 15.0)
        let grip = tracks.track_friction * terrain_modifier * traction_factor;
        let lateral_friction = -lateral_vel * rb.mass * 35.0 * grip;

        rb.apply_force(lateral * lateral_friction);

        // === AUTO-HAMOWANIE (ARCADE) ===
        // Gdy nie ma inputu ruchu - automatyczne, płynne zatrzymanie
        let input_magnitude = (input.forward.abs() + input.rotation.abs()).min(1.0);
        if input_magnitude < 0.1 && current_forward_speed.abs() > 0.3 && !input.braking {
            // Auto-brake: łagodniejsze niż ręczny hamulec (4 m/s² vs 8 m/s²)
            let auto_brake_decel = 4.0;
            let auto_brake_force = -current_forward_speed.signum() * rb.mass * auto_brake_decel * traction_factor;
            rb.apply_force(forward * auto_brake_force);
        }

        // === HAMULCE GĄSIENICOWE ===
        // Niezależne hamulce dla ostrych skrętów (Q = lewy, E = prawy)
        if input.brake_left > 0.0 || input.brake_right > 0.0 {
            let half_width = tracks.track_width / 2.0;

            // Kierunek ruchu każdej gąsienicy
            let left_dir = if tracks.left_track_speed >= 0.0 { 1.0 } else { -1.0 };
            let right_dir = if tracks.right_track_speed >= 0.0 { 1.0 } else { -1.0 };

            // Moment obrotowy z różnicowego hamowania
            // Lewy hamulec tworzy obrót w prawo (dodatni Y)
            // Prawy hamulec tworzy obrót w lewo (ujemny Y)
            let brake_torque = (input.brake_left * left_dir - input.brake_right * right_dir)
                * input.max_brake_force * half_width * 0.5;

            rb.torque.y += brake_torque * traction_factor;

            // Siła hamowania wzdłuż kierunku jazdy
            let total_brake = (input.brake_left + input.brake_right) * 0.5;
            let brake_decel_force = -current_forward_speed.signum()
                * total_brake * input.max_brake_force * 0.5 * traction_factor;
            rb.apply_force(forward * brake_decel_force);
        }

        // === OPÓR TOCZENIA ===
        // Opór toczenia: F = μ * m * g * terrain_resistance
        let rolling_coef = 0.02; // Bazowy współczynnik
        let rolling_resistance = rolling_coef * rb.mass * GRAVITY * terrain.current_resistance;
        let rolling_drag = rolling_resistance * current_forward_speed.signum();
        rb.apply_force(forward * (-rolling_drag));

        // === OPÓR POWIETRZA ZALEŻNY OD ORIENTACJI ===
        // Efektywna powierzchnia zależy od kąta między kierunkiem jazdy a prędkością
        let velocity_dir = rb.velocity.normalize_or_zero();
        let speed_squared = rb.velocity.length_squared();

        if speed_squared > 0.1 {
            // Składowe prędkości względem czołgu
            let forward_dot = forward.dot(velocity_dir).abs();  // 1.0 = jedzie prosto
            let lateral_dot = lateral.dot(velocity_dir).abs();  // 1.0 = jedzie bokiem

            // Interpolacja efektywnej powierzchni
            // Prosto = frontal_area, bokiem = side_area
            let effective_area = tracks.frontal_area * forward_dot
                + tracks.side_area * lateral_dot;

            // Opór powietrza: F = 0.5 * Cd * A * ρ * v²
            let air_drag_magnitude = 0.5 * tracks.drag_coefficient
                * effective_area * AIR_DENSITY * speed_squared;

            // Siła przeciwna do kierunku ruchu
            rb.apply_force(-velocity_dir * air_drag_magnitude);
        }

        // === EFEKT NACHYLENIA (ARCADE) ===
        // Soft transition zamiast twardego limitu - zawsze można próbować wspinać
        if slope.on_slope && slope.slope_angle > 0.02 {
            let slope_deg = slope.slope_angle.to_degrees();

            // Sprawdź czy próbujemy jechać pod górę
            let slope_dir = Vec3::new(slope.terrain_normal.x, 0.0, slope.terrain_normal.z)
                .normalize_or_zero();
            let going_uphill = forward.dot(slope_dir) < 0.0;

            if going_uphill {
                // ARCADE: Soft power reduction zamiast hard cut-off
                // 0-35°: pełna moc
                // 35-50°: stopniowo malejąca moc
                // >50°: minimalna moc (ale zawsze można próbować)
                let power_reduction = if slope_deg < 35.0 {
                    0.0
                } else if slope_deg < 50.0 {
                    (slope_deg - 35.0) / 30.0  // 0.0 do 0.5
                } else {
                    0.5  // Max 50% redukcji mocy
                };

                // Łagodny opór zamiast gwałtownego zsuwania
                // Czołg zwolni, ale nie zsuwa się automatycznie
                let resistance_force = rb.mass * GRAVITY * slope.slope_angle.sin() * power_reduction;
                rb.apply_force(slope_dir * resistance_force * 0.3);
            }
        }

        // === TŁUMIENIE KĄTOWE (stabilizacja) ===
        // Dodatkowe tłumienie dla pitch i roll gdy na ziemi
        if traction_factor > 0.5 {
            // Tłumienie pitch (kołysanie przód-tył)
            rb.angular_velocity.x *= 0.95;
            // Tłumienie roll (przechylanie bok)
            rb.angular_velocity.z *= 0.95;
        }
    }
}

/// System aktualizacji prędkości w TankMobility (dla UI i innych systemów)
pub fn sync_mobility_speed(
    mut query: Query<(&RigidBody6DOF, &Transform, &mut TankMobility), With<Tank>>,
) {
    for (rb, transform, mut mobility) in query.iter_mut() {
        // Prędkość w kierunku jazdy
        let forward = transform.right();
        mobility.current_speed = rb.velocity.dot(*forward);

        // RPM na podstawie prędkości
        let speed_ratio = (mobility.current_speed.abs() / mobility.max_forward_speed).clamp(0.0, 1.0);
        let target_rpm = mobility.idle_rpm + (mobility.max_rpm - mobility.idle_rpm) * speed_ratio;

        // Płynne przejście RPM
        let rpm_lerp = 0.1;
        mobility.current_rpm += (target_rpm - mobility.current_rpm) * rpm_lerp;
    }
}
