use bevy::prelude::*;

// ============================================================================
// RIGID BODY 6DOF
// ============================================================================

/// Pełne 6 stopni swobody rigid body (pozycja XYZ + rotacja XYZ)
#[derive(Component, Clone, Debug)]
pub struct RigidBody6DOF {
    // === STAN KINEMATYCZNY ===
    /// Prędkość liniowa [m/s]
    pub velocity: Vec3,
    /// Prędkość kątowa [rad/s] (w local space)
    pub angular_velocity: Vec3,

    // === WŁAŚCIWOŚCI FIZYCZNE ===
    /// Masa [kg]
    pub mass: f32,
    /// Tensor inercji (tylko diagonala) [kg*m²]
    pub inertia_tensor: Vec3,
    /// 1/mass (cache dla optymalizacji)
    pub inv_mass: f32,
    /// 1/inertia (cache)
    pub inv_inertia: Vec3,

    // === AKUMULATORY SIŁ (zerowane każdą klatkę) ===
    /// Siła wypadkowa [N]
    pub force: Vec3,
    /// Moment wypadkowy [N*m]
    pub torque: Vec3,

    // === DAMPING (tłumienie) ===
    /// Tłumienie liniowe (0-1, 0 = brak, 1 = natychmiastowe zatrzymanie)
    pub linear_damping: f32,
    /// Tłumienie kątowe
    pub angular_damping: f32,

    // === OGRANICZENIA ===
    /// Czy ciało może się poruszać (false = kinematic)
    pub is_dynamic: bool,
    /// Blokada ruchu na osiach (dla constraints)
    pub lock_translation: BVec3,
    /// Blokada rotacji na osiach
    pub lock_rotation: BVec3,
}

impl Default for RigidBody6DOF {
    fn default() -> Self {
        Self::new(1000.0, Vec3::new(1.0, 1.0, 1.0))
    }
}

impl RigidBody6DOF {
    /// Tworzy nowe rigid body z masą i wymiarami (dla tensora inercji)
    pub fn new(mass: f32, dimensions: Vec3) -> Self {
        // Tensor inercji dla prostopadłościanu (solid cuboid):
        // Ixx = (1/12) * m * (y² + z²)
        // Iyy = (1/12) * m * (x² + z²)
        // Izz = (1/12) * m * (x² + y²)
        let inertia = Vec3::new(
            (1.0 / 12.0) * mass * (dimensions.y.powi(2) + dimensions.z.powi(2)),
            (1.0 / 12.0) * mass * (dimensions.x.powi(2) + dimensions.z.powi(2)),
            (1.0 / 12.0) * mass * (dimensions.x.powi(2) + dimensions.y.powi(2)),
        );

        Self {
            velocity: Vec3::ZERO,
            angular_velocity: Vec3::ZERO,
            mass,
            inertia_tensor: inertia,
            inv_mass: 1.0 / mass,
            inv_inertia: Vec3::new(1.0 / inertia.x, 1.0 / inertia.y, 1.0 / inertia.z),
            force: Vec3::ZERO,
            torque: Vec3::ZERO,
            linear_damping: 0.05,
            angular_damping: 0.1,
            is_dynamic: true,
            lock_translation: BVec3::FALSE,
            lock_rotation: BVec3::FALSE,
        }
    }

    /// Tworzy rigid body dla czołgu (domyślne wartości)
    pub fn tank(mass_kg: f32) -> Self {
        // Typowe wymiary czołgu: 4m x 1.5m x 2.6m
        let mut rb = Self::new(mass_kg, Vec3::new(4.0, 1.5, 2.6));
        // ARCADE: wyższe tłumienie dla szybszego zatrzymania (było 0.02/0.15)
        rb.linear_damping = 0.06;
        rb.angular_damping = 0.18;
        rb
    }

    // === METODY APLIKACJI SIŁ ===

    /// Aplikuje siłę w środku masy [N]
    pub fn apply_force(&mut self, force: Vec3) {
        if !self.is_dynamic {
            return;
        }
        self.force += force;
    }

    /// Aplikuje siłę w punkcie (generuje też moment) [N]
    pub fn apply_force_at_point(&mut self, force: Vec3, point: Vec3, center_of_mass: Vec3) {
        if !self.is_dynamic {
            return;
        }
        self.force += force;
        // Moment = r × F (ramię × siła)
        let r = point - center_of_mass;
        self.torque += r.cross(force);
    }

    /// Aplikuje siłę w lokalnym punkcie względem Transform
    pub fn apply_force_at_local_point(
        &mut self,
        force: Vec3,
        local_point: Vec3,
        transform: &Transform,
    ) {
        let world_point = transform.transform_point(local_point);
        self.apply_force_at_point(force, world_point, transform.translation);
    }

    /// Aplikuje moment obrotowy [N*m]
    pub fn apply_torque(&mut self, torque: Vec3) {
        if !self.is_dynamic {
            return;
        }
        self.torque += torque;
    }

    // === METODY APLIKACJI IMPULSÓW (natychmiastowa zmiana prędkości) ===

    /// Aplikuje impuls liniowy [N*s]
    pub fn apply_impulse(&mut self, impulse: Vec3) {
        if !self.is_dynamic {
            return;
        }
        self.velocity += impulse * self.inv_mass;
    }

    /// Aplikuje impuls w punkcie (zmienia też prędkość kątową)
    pub fn apply_impulse_at_point(&mut self, impulse: Vec3, point: Vec3, center_of_mass: Vec3) {
        if !self.is_dynamic {
            return;
        }
        self.velocity += impulse * self.inv_mass;
        let r = point - center_of_mass;
        self.angular_velocity += r.cross(impulse) * self.inv_inertia;
    }

    /// Aplikuje impuls kątowy [N*m*s]
    pub fn apply_torque_impulse(&mut self, impulse: Vec3) {
        if !self.is_dynamic {
            return;
        }
        self.angular_velocity += impulse * self.inv_inertia;
    }

    // === METODY POMOCNICZE ===

    /// Zwraca energię kinetyczną [J]
    pub fn kinetic_energy(&self) -> f32 {
        let linear = 0.5 * self.mass * self.velocity.length_squared();
        let angular = 0.5 * (
            self.inertia_tensor.x * self.angular_velocity.x.powi(2) +
            self.inertia_tensor.y * self.angular_velocity.y.powi(2) +
            self.inertia_tensor.z * self.angular_velocity.z.powi(2)
        );
        linear + angular
    }

    /// Zwraca pęd [kg*m/s]
    pub fn momentum(&self) -> Vec3 {
        self.velocity * self.mass
    }

    /// Zeruje akumulatory sił (wywoływane po integracji)
    pub fn clear_forces(&mut self) {
        self.force = Vec3::ZERO;
        self.torque = Vec3::ZERO;
    }

    /// Sprawdza czy ciało jest w spoczynku
    pub fn is_sleeping(&self) -> bool {
        self.velocity.length_squared() < 0.001 &&
        self.angular_velocity.length_squared() < 0.001
    }

    /// Aplikuje ograniczenia na prędkości
    pub fn apply_locks(&mut self) {
        if self.lock_translation.x {
            self.velocity.x = 0.0;
        }
        if self.lock_translation.y {
            self.velocity.y = 0.0;
        }
        if self.lock_translation.z {
            self.velocity.z = 0.0;
        }
        if self.lock_rotation.x {
            self.angular_velocity.x = 0.0;
        }
        if self.lock_rotation.y {
            self.angular_velocity.y = 0.0;
        }
        if self.lock_rotation.z {
            self.angular_velocity.z = 0.0;
        }
    }
}

// ============================================================================
// RAYCAST SUSPENSION COMPONENTS
// ============================================================================

/// Pojedynczy punkt zawieszenia (wirtualne koło)
#[derive(Clone, Debug)]
pub struct SuspensionPoint {
    /// Pozycja w local space (względem środka czołgu)
    pub local_position: Vec3,
    /// Poprzednia długość sprężyny (dla obliczania prędkości)
    pub last_length: f32,
    /// Czy dotyka terenu
    pub grounded: bool,
    /// Normalna kontaktu z terenem
    pub contact_normal: Vec3,
    /// Punkt kontaktu w world space
    pub contact_point: Vec3,
    /// Aktualna siła zawieszenia
    pub current_force: f32,
}

impl Default for SuspensionPoint {
    fn default() -> Self {
        Self {
            local_position: Vec3::ZERO,
            last_length: 0.5,
            grounded: false,
            contact_normal: Vec3::Y,
            contact_point: Vec3::ZERO,
            current_force: 0.0,
        }
    }
}

impl SuspensionPoint {
    pub fn new(local_position: Vec3) -> Self {
        Self {
            local_position,
            ..default()
        }
    }
}

/// System zawieszenia oparty na raycastach
#[derive(Component, Clone, Debug)]
pub struct RaycastSuspension {
    /// Punkty zawieszenia (8 dla czołgu - 4 na stronę)
    pub suspension_points: Vec<SuspensionPoint>,
    /// Maksymalna długość raycasta (pełne rozciągnięcie)
    pub max_length: f32,
    /// Długość zawieszenia w spoczynku
    pub rest_length: f32,
    /// Sztywność sprężyny [N/m]
    pub spring_strength: f32,
    /// Współczynnik tłumienia [N*s/m]
    pub damper_strength: f32,
    /// Mnożnik tłumienia przy kompresji (>1.0 = sztywniejsze przy uderzeniu)
    pub compression_damping_mult: f32,
    /// Mnożnik tłumienia przy odbiciu (<1.0 = miększe przy powrocie)
    pub rebound_damping_mult: f32,
    /// Minimalny kontakt (ile punktów musi dotykać dla trakcji)
    pub min_ground_contacts: usize,
}

impl Default for RaycastSuspension {
    fn default() -> Self {
        // 8 punktów zawieszenia dla czołgu T-54/55
        // track_width = 2.64m → Z = ±1.32
        // track_length = 3.7m → X od -1.5 do +1.5
        //
        // WAŻNE: Punkty startują na wysokości osi kół (Y=0.28 - road wheels)
        // Track bottom jest na Y=0.12, więc musimy utrzymać ~0.15m prześwitu
        let suspension_y = 0.28; // Wysokość osi kół jezdnych
        let points = vec![
            // Lewa strona (Z = -1.32) - od przodu do tyłu
            SuspensionPoint::new(Vec3::new(1.5, suspension_y, -1.32)),   // Przód lewy
            SuspensionPoint::new(Vec3::new(0.5, suspension_y, -1.32)),   // Środek-przód lewy
            SuspensionPoint::new(Vec3::new(-0.5, suspension_y, -1.32)),  // Środek-tył lewy
            SuspensionPoint::new(Vec3::new(-1.5, suspension_y, -1.32)),  // Tył lewy
            // Prawa strona (Z = +1.32) - od przodu do tyłu
            SuspensionPoint::new(Vec3::new(1.5, suspension_y, 1.32)),    // Przód prawy
            SuspensionPoint::new(Vec3::new(0.5, suspension_y, 1.32)),    // Środek-przód prawy
            SuspensionPoint::new(Vec3::new(-0.5, suspension_y, 1.32)),   // Środek-tył prawy
            SuspensionPoint::new(Vec3::new(-1.5, suspension_y, 1.32)),   // Tył prawy
        ];

        Self {
            suspension_points: points,
            // Zwiększona długość raycastu - zawsze powinien trafić w teren
            max_length: 1.5,
            // Długość w spoczynku - przy tej długości gąsienice (Y=0.12) będą ~0.16m nad terenem
            // suspension_y (0.28) - track_bottom (0.12) = 0.16
            // Więc rest_length musi być > 0.16 żeby gąsienice nie wchodziły w teren
            rest_length: 0.45,
            // Dla 45t czołgu, 8 punktów: każdy niesie ~5.6t
            // F = mg = 5600 * 9.81 = ~55kN przy rest_length
            // k = F / compression (0.45) = 55000 / 0.45 = ~122000 N/m
            // Zwiększamy dla lepszej stabilności
            spring_strength: 150000.0,
            damper_strength: 15000.0,
            // Asymetryczne tłumienie: sztywniejsze przy kompresji, miększe przy odbiciu
            compression_damping_mult: 1.3, // 30% więcej tłumienia przy uderzeniu
            rebound_damping_mult: 0.7,     // 30% mniej tłumienia przy powrocie
            min_ground_contacts: 2,
        }
    }
}

impl RaycastSuspension {
    /// Zwraca liczbę punktów dotykających terenu
    pub fn grounded_count(&self) -> usize {
        self.suspension_points.iter().filter(|p| p.grounded).count()
    }

    /// Czy czołg ma wystarczający kontakt z terenem
    pub fn has_traction(&self) -> bool {
        self.grounded_count() >= self.min_ground_contacts
    }

    /// Średnia normalna kontaktu (dla orientacji kadłuba)
    pub fn average_contact_normal(&self) -> Vec3 {
        let grounded: Vec<&SuspensionPoint> = self.suspension_points
            .iter()
            .filter(|p| p.grounded)
            .collect();

        if grounded.is_empty() {
            return Vec3::Y;
        }

        let sum: Vec3 = grounded.iter().map(|p| p.contact_normal).sum();
        (sum / grounded.len() as f32).normalize_or_zero()
    }

    /// Średnia wysokość kontaktu
    pub fn average_ground_height(&self) -> f32 {
        let grounded: Vec<&SuspensionPoint> = self.suspension_points
            .iter()
            .filter(|p| p.grounded)
            .collect();

        if grounded.is_empty() {
            return 0.0;
        }

        let sum: f32 = grounded.iter().map(|p| p.contact_point.y).sum();
        sum / grounded.len() as f32
    }
}
