use bevy::prelude::*;

/// Marker component for the tank hull (root entity)
#[derive(Component)]
pub struct Tank;

// ============================================================================
// MOBILITY SYSTEM COMPONENTS
// ============================================================================

/// Main mobility component - engine, mass, and speed parameters
#[derive(Component)]
pub struct TankMobility {
    /// Tank mass in kilograms
    pub mass_kg: f32,
    /// Engine power in horsepower
    pub engine_power_hp: f32,
    /// Power to weight ratio (hp/t) - calculated from mass and power
    pub power_to_weight: f32,
    /// Maximum forward speed in m/s
    pub max_forward_speed: f32,
    /// Maximum reverse speed in m/s
    pub max_reverse_speed: f32,
    /// Hull traverse speed when stationary (degrees/s)
    pub hull_traverse_speed: f32,
    /// Current speed in m/s (positive = forward, negative = reverse)
    pub current_speed: f32,
    /// Current engine RPM
    pub current_rpm: f32,
    /// Idle RPM
    pub idle_rpm: f32,
    /// Max RPM
    pub max_rpm: f32,
    // === KRZYWA MOMENTU SILNIKA ===
    /// RPM przy maksymalnym momencie obrotowym
    pub peak_torque_rpm: f32,
    /// Szerokość krzywej momentu (większa = szersza krzywa)
    pub torque_curve_width: f32,
    /// Minimalny mnożnik momentu przy skrajnych RPM
    pub min_torque_mult: f32,
}

impl Default for TankMobility {
    fn default() -> Self {
        let mass_kg = 45000.0; // 45 tons
        let engine_power_hp = 750.0; // Increased from 520 for better hill climbing
        Self {
            mass_kg,
            engine_power_hp,
            power_to_weight: engine_power_hp / (mass_kg / 1000.0), // ~16.7 hp/t
            max_forward_speed: 13.89, // 50 km/h
            max_reverse_speed: 5.56,  // 20 km/h
            hull_traverse_speed: 32.0, // Increased from 22 for better rotation
            current_speed: 0.0,
            current_rpm: 800.0,
            idle_rpm: 800.0,
            max_rpm: 2200.0,
            // Krzywa momentu - typowy diesel WWII
            peak_torque_rpm: 1600.0,      // Maksymalny moment przy 1600 RPM
            torque_curve_width: 800.0,    // Szerokość krzywej Gaussa
            min_torque_mult: 0.6,         // Minimum 60% momentu przy skrajnych RPM
        }
    }
}

/// Track physics for differential steering
#[derive(Component)]
pub struct TrackPhysics {
    /// Left track speed in m/s
    pub left_track_speed: f32,
    /// Right track speed in m/s
    pub right_track_speed: f32,
    /// Distance between track centers in meters
    pub track_width: f32,
    /// Track ground contact length in meters
    pub track_length: f32,
    /// Drive sprocket radius in meters
    pub sprocket_radius: f32,
    /// Track friction coefficient
    pub track_friction: f32,
    // === OPÓR POWIETRZA ZALEŻNY OD ORIENTACJI ===
    /// Powierzchnia czołowa [m²]
    pub frontal_area: f32,
    /// Powierzchnia boczna [m²]
    pub side_area: f32,
    /// Współczynnik oporu aerodynamicznego
    pub drag_coefficient: f32,
    // === SLIP RATIO (poślizg gąsienic) ===
    /// Aktualny poślizg lewej gąsienicy (-1 do 1)
    pub left_slip_ratio: f32,
    /// Aktualny poślizg prawej gąsienicy (-1 do 1)
    pub right_slip_ratio: f32,
    /// Poślizg przy maksymalnej trakcji (krzywa Pacejki)
    pub peak_slip_ratio: f32,
    /// Poślizg krytyczny - po nim trakcja spada znacząco
    pub critical_slip_ratio: f32,
    /// Minimalny współczynnik trakcji przy pełnym poślizgu
    pub min_traction_coefficient: f32,
}

impl Default for TrackPhysics {
    fn default() -> Self {
        Self {
            left_track_speed: 0.0,
            right_track_speed: 0.0,
            track_width: 2.64,
            track_length: 3.7,
            sprocket_radius: 0.35,
            track_friction: 0.95,   // ARCADE: wysoka przyczepność (było 0.85)
            // Opór powietrza - czołg ma duży profil boczny
            frontal_area: 3.0,       // m² - widok z przodu
            side_area: 4.5,          // m² - widok z boku (50% większy)
            drag_coefficient: 0.8,   // Wysoki dla kształtu pudełkowego
            // Slip ratio - ARCADE: szerszy zakres pełnej trakcji
            left_slip_ratio: 0.0,
            right_slip_ratio: 0.0,
            peak_slip_ratio: 0.25,          // ARCADE: szerszy zakres (było 0.15)
            critical_slip_ratio: 0.6,       // ARCADE: łagodniejsze przejście (było 0.4)
            min_traction_coefficient: 0.7,  // ARCADE: wysoka min trakcja (było 0.4)
        }
    }
}

/// Terrain resistance values for different ground types
#[derive(Component)]
pub struct TerrainResistance {
    /// Resistance on hard terrain (roads, concrete)
    pub hard_terrain: f32,
    /// Resistance on medium terrain (grass, dirt)
    pub medium_terrain: f32,
    /// Resistance on soft terrain (mud, sand)
    pub soft_terrain: f32,
    /// Current terrain resistance multiplier
    pub current_resistance: f32,
}

impl Default for TerrainResistance {
    fn default() -> Self {
        Self {
            hard_terrain: 0.85,
            medium_terrain: 1.0,
            soft_terrain: 1.5,
            current_resistance: 1.0, // Default to medium
        }
    }
}

/// Suspension state for visual hull movement
#[derive(Component)]
pub struct SuspensionState {
    /// Pitch angle in radians (positive = nose up)
    pub pitch: f32,
    /// Roll angle in radians (positive = right side down)
    pub roll: f32,
    /// Pitch angular velocity
    pub pitch_velocity: f32,
    /// Roll angular velocity
    pub roll_velocity: f32,
    /// Spring stiffness coefficient
    pub stiffness: f32,
    /// Damping coefficient
    pub damping: f32,
    /// Maximum pitch angle in radians
    pub max_pitch: f32,
    /// Maximum roll angle in radians
    pub max_roll: f32,
}

impl Default for SuspensionState {
    fn default() -> Self {
        Self {
            pitch: 0.0,
            roll: 0.0,
            pitch_velocity: 0.0,
            roll_velocity: 0.0,
            stiffness: 8.0,
            damping: 4.0,
            max_pitch: 0.08, // ~4.5 degrees
            max_roll: 0.06,  // ~3.5 degrees
        }
    }
}

/// Drift/slip state for lateral movement
#[derive(Component)]
pub struct DriftState {
    /// Lateral velocity vector
    pub lateral_velocity: Vec3,
    /// Current drift factor (0 = no drift, 1 = full drift)
    pub drift_factor: f32,
    /// Ground grip coefficient
    pub grip: f32,
}

impl Default for DriftState {
    fn default() -> Self {
        Self {
            lateral_velocity: Vec3::ZERO,
            drift_factor: 0.0,
            grip: 0.85,
        }
    }
}

/// Slope/terrain normal state
#[derive(Component)]
pub struct SlopeState {
    /// Terrain surface normal at tank position
    pub terrain_normal: Vec3,
    /// Current slope angle in radians
    pub slope_angle: f32,
    /// Slope direction relative to tank forward (radians)
    pub slope_direction: f32,
    /// Maximum climbable slope angle in radians
    pub max_climbable_angle: f32,
    /// Is the tank currently on a slope
    pub on_slope: bool,
}

impl Default for SlopeState {
    fn default() -> Self {
        Self {
            terrain_normal: Vec3::Y,
            slope_angle: 0.0,
            slope_direction: 0.0,
            max_climbable_angle: 35.0_f32.to_radians(), // 35 degrees
            on_slope: false,
        }
    }
}

/// Turret component with traverse speed
#[derive(Component)]
pub struct Turret {
    /// Degrees per second
    pub traverse_speed: f32,
}

/// Marker for the gun pivot point (between turret and barrel)
#[derive(Component)]
pub struct GunMount;

/// Barrel component with elevation parameters
#[derive(Component)]
pub struct Barrel {
    /// Degrees per second
    pub elevation_speed: f32,
    /// Max elevation in degrees (positive = up)
    pub max_elevation: f32,
    /// Max depression in degrees (negative = down)
    pub max_depression: f32,
}

/// Marker for the muzzle point (projectile spawn location)
#[derive(Component)]
pub struct MuzzlePoint;

/// Tank movement parameters
#[derive(Component)]
pub struct TankMovement {
    /// Max forward speed in m/s
    pub forward_speed: f32,
    /// Max reverse speed in m/s
    pub reverse_speed: f32,
    /// Hull rotation speed when stationary (degrees/s)
    pub rotation_speed: f32,
    /// Hull rotation speed at max speed (degrees/s) - slower when moving
    pub rotation_speed_moving: f32,
    /// Acceleration in m/s²
    pub acceleration: f32,
    /// Deceleration when braking (m/s²)
    pub deceleration: f32,
    /// Friction/drag when coasting (no input) (m/s²)
    pub friction: f32,
    /// Current velocity in m/s (positive = forward)
    pub current_velocity: f32,
}

impl Default for TankMovement {
    fn default() -> Self {
        Self {
            forward_speed: 10.0,
            reverse_speed: 5.0,
            rotation_speed: 32.0,        // Stationary rotation
            rotation_speed_moving: 18.0, // Rotation at max speed
            acceleration: 6.0,
            deceleration: 14.0,          // Active braking
            friction: 4.0,               // Passive slowdown
            current_velocity: 0.0,
        }
    }
}

/// Gun dispersion parameters (WoT-style)
#[derive(Component)]
pub struct GunDispersion {
    /// Base accuracy at 100m when fully aimed (meters)
    pub base_accuracy: f32,
    /// Aim time in seconds
    pub aim_time: f32,
    /// Dispersion penalty per m/s of movement
    pub dispersion_move: f32,
    /// Dispersion penalty per deg/s of hull rotation
    pub dispersion_hull_rotation: f32,
    /// Dispersion penalty per deg/s of turret traverse
    pub dispersion_turret_rotation: f32,
    /// Stabilization modifier (0.0 = none, 0.8 = max)
    pub stabilization: f32,
}

impl Default for GunDispersion {
    fn default() -> Self {
        Self {
            base_accuracy: 0.36,
            aim_time: 2.3,
            dispersion_move: 0.18,
            dispersion_hull_rotation: 0.10,
            dispersion_turret_rotation: 0.06,
            stabilization: 0.0,
        }
    }
}

/// Tracks tank velocities for dispersion calculation
#[derive(Component, Default)]
pub struct TankVelocities {
    /// Previous frame position
    pub prev_position: Vec3,
    /// Previous frame hull yaw
    pub prev_hull_yaw: f32,
    /// Current linear speed (m/s)
    pub linear_speed: f32,
    /// Current hull rotation speed (rad/s)
    pub hull_rotation_speed: f32,
    /// Is the tank moving above threshold
    pub is_moving: bool,
    /// Is the hull rotating above threshold
    pub is_hull_rotating: bool,
}

/// Tracks turret rotation velocity
#[derive(Component, Default)]
pub struct TurretVelocity {
    /// Previous frame turret yaw
    pub prev_yaw: f32,
    /// Current rotation speed (rad/s)
    pub rotation_speed: f32,
    /// Is turret traversing above threshold
    pub is_traversing: bool,
}

/// Parametry efektu żyroskopowego wieży
/// Obrót wieży wpływa na stabilność kadłuba
#[derive(Component)]
pub struct TurretGyroscopic {
    /// Moment bezwładności wieży wokół osi Y [kg*m²]
    pub turret_inertia_y: f32,
    /// Współczynnik sprzężenia (0.0-1.0, ile efektu przenosi się na kadłub)
    pub gyro_coupling: f32,
}

impl Default for TurretGyroscopic {
    fn default() -> Self {
        Self {
            // Typowa wieża czołgu: ~3 tony, promień 0.6m
            // I = m * r² ≈ 3000 * 0.36 ≈ 1000, ale zwiększamy dla efektu
            turret_inertia_y: 5000.0,
            // 30% efektu - wieża jest częściowo odizolowana przez łożysko
            gyro_coupling: 0.3,
        }
    }
}

// ============================================================================
// INPUT SYSTEM COMPONENTS
// ============================================================================

/// Tank input intent - separates raw input from physics
/// This provides smoothed input values with ramping
#[derive(Component)]
pub struct TankInput {
    /// Raw forward/backward input (-1 to 1)
    pub raw_forward: f32,
    /// Raw rotation input (-1 to 1)
    pub raw_rotation: f32,
    /// Smoothed forward input (with ramping)
    pub forward: f32,
    /// Smoothed rotation input (with ramping)
    pub rotation: f32,
    /// Is brake pressed
    pub braking: bool,
    // === HAMULCE GĄSIENICOWE ===
    /// Hamulec lewej gąsienicy (0.0-1.0)
    pub brake_left: f32,
    /// Hamulec prawej gąsienicy (0.0-1.0)
    pub brake_right: f32,
    /// Maksymalna siła hamowania [N]
    pub max_brake_force: f32,
    /// Input acceleration rate (how fast input ramps up)
    pub acceleration_rate: f32,
    /// Input deceleration rate (how fast input ramps down)
    pub deceleration_rate: f32,
}

impl Default for TankInput {
    fn default() -> Self {
        Self {
            raw_forward: 0.0,
            raw_rotation: 0.0,
            forward: 0.0,
            rotation: 0.0,
            braking: false,
            // Hamulce gąsienicowe
            brake_left: 0.0,
            brake_right: 0.0,
            max_brake_force: 200000.0,  // Wystarczająco silne by zablokować gąsienice
            acceleration_rate: 4.0,  // Ramp up in ~0.25s
            deceleration_rate: 6.0,  // Ramp down in ~0.17s
        }
    }
}
