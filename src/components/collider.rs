use bevy::prelude::*;

// ============================================================================
// COLLIDER COMPONENTS
// ============================================================================

/// AABB/OBB box collider dla obiektów
#[derive(Component, Clone, Debug)]
pub struct BoxCollider {
    /// Połowa wymiarów (half extents) - NIE pełne wymiary!
    pub half_extents: Vec3,
    /// Offset względem Transform entity
    pub offset: Vec3,
}

impl BoxCollider {
    pub fn new(half_extents: Vec3) -> Self {
        Self {
            half_extents,
            offset: Vec3::ZERO,
        }
    }

    pub fn with_offset(mut self, offset: Vec3) -> Self {
        self.offset = offset;
        self
    }

    /// Tworzy BoxCollider z pełnych wymiarów (dzieli przez 2)
    pub fn from_size(size: Vec3) -> Self {
        Self {
            half_extents: size * 0.5,
            offset: Vec3::ZERO,
        }
    }
}

/// Compound collider - kilka boxów jako jeden collider (dla czołgu)
#[derive(Component, Clone, Debug)]
pub struct CompoundCollider {
    /// Lista kształtów: (offset_lokalny, rotacja_lokalna, shape)
    pub shapes: Vec<CompoundShape>,
}

#[derive(Clone, Debug)]
pub struct CompoundShape {
    pub offset: Vec3,
    pub rotation: Quat,
    pub collider: BoxCollider,
}

impl CompoundCollider {
    pub fn new(shapes: Vec<CompoundShape>) -> Self {
        Self { shapes }
    }

    /// Helper do tworzenia kształtu
    pub fn shape(offset: Vec3, half_extents: Vec3) -> CompoundShape {
        CompoundShape {
            offset,
            rotation: Quat::IDENTITY,
            collider: BoxCollider::new(half_extents),
        }
    }

    pub fn shape_rotated(offset: Vec3, rotation: Quat, half_extents: Vec3) -> CompoundShape {
        CompoundShape {
            offset,
            rotation,
            collider: BoxCollider::new(half_extents),
        }
    }
}

// ============================================================================
// BODY TYPE MARKERS
// ============================================================================

/// Marker: obiekt statyczny (budynki, fortyfikacje) - nie porusza się
#[derive(Component, Clone, Copy, Debug, Default)]
pub struct StaticBody;

/// Marker: obiekt niszczalny przez kolizję
#[derive(Component, Clone, Debug)]
pub struct Destructible {
    /// Punkty wytrzymałości
    pub health: f32,
    /// Masa obiektu [kg] - wpływa na impuls przy zderzeniu
    pub mass: f32,
    /// Minimalny impuls do zniszczenia [N*s]
    pub destruction_threshold: f32,
}

impl Default for Destructible {
    fn default() -> Self {
        Self {
            health: 100.0,
            mass: 5000.0,
            destruction_threshold: 500000.0, // ~500 kN*s
        }
    }
}

impl Destructible {
    pub fn new(health: f32, mass: f32, threshold: f32) -> Self {
        Self {
            health,
            mass,
            destruction_threshold: threshold,
        }
    }

    /// Lekki obiekt (płoty, małe znaki)
    pub fn light() -> Self {
        Self {
            health: 20.0,
            mass: 100.0,
            destruction_threshold: 50000.0,
        }
    }

    /// Średni obiekt (małe budynki, ruiny)
    pub fn medium() -> Self {
        Self {
            health: 100.0,
            mass: 5000.0,
            destruction_threshold: 300000.0,
        }
    }

    /// Ciężki obiekt (duże budynki)
    pub fn heavy() -> Self {
        Self {
            health: 500.0,
            mass: 20000.0,
            destruction_threshold: 1000000.0,
        }
    }
}

// ============================================================================
// WORLD SPACE AABB (for broad phase)
// ============================================================================

/// AABB w przestrzeni świata - aktualizowany każdą klatkę
#[derive(Component, Clone, Debug, Default)]
pub struct WorldAABB {
    pub min: Vec3,
    pub max: Vec3,
}

impl WorldAABB {
    pub fn new(min: Vec3, max: Vec3) -> Self {
        Self { min, max }
    }

    /// Sprawdza czy dwa AABB się przecinają
    pub fn intersects(&self, other: &WorldAABB) -> bool {
        self.min.x <= other.max.x && self.max.x >= other.min.x &&
        self.min.y <= other.max.y && self.max.y >= other.min.y &&
        self.min.z <= other.max.z && self.max.z >= other.min.z
    }

    /// Oblicza środek AABB
    pub fn center(&self) -> Vec3 {
        (self.min + self.max) * 0.5
    }

    /// Oblicza rozmiar AABB
    pub fn size(&self) -> Vec3 {
        self.max - self.min
    }

    /// Rozszerza AABB o margines
    pub fn expand(&self, margin: f32) -> Self {
        Self {
            min: self.min - Vec3::splat(margin),
            max: self.max + Vec3::splat(margin),
        }
    }
}

// ============================================================================
// EVENTS
// ============================================================================

/// Event wysyłany przy zniszczeniu obiektu
#[derive(Event, Clone, Debug)]
pub struct DestructionEvent {
    /// Entity które zostało zniszczone
    pub entity: Entity,
    /// Pozycja w momencie zniszczenia
    pub position: Vec3,
    /// Prędkość uderzenia
    pub impact_velocity: Vec3,
    /// Siła uderzenia [N]
    pub impact_force: f32,
}

/// Event kolizji (opcjonalny, do debugowania/efektów)
#[derive(Event, Clone, Debug)]
pub struct CollisionEvent {
    pub entity_a: Entity,
    pub entity_b: Entity,
    pub contact_point: Vec3,
    pub contact_normal: Vec3,
    pub penetration_depth: f32,
}

// ============================================================================
// COLLISION LAYERS (optional, for filtering)
// ============================================================================

/// Warstwy kolizji - które obiekty kolidują z którymi
#[derive(Component, Clone, Copy, Debug)]
pub struct CollisionLayers {
    /// Maska własna - do jakiej warstwy należy
    pub membership: u32,
    /// Maska filtra - z jakimi warstwami koliduje
    pub filter: u32,
}

impl Default for CollisionLayers {
    fn default() -> Self {
        Self {
            membership: 0xFFFFFFFF, // Wszystkie warstwy
            filter: 0xFFFFFFFF,     // Koliduje ze wszystkim
        }
    }
}

impl CollisionLayers {
    pub const TANK: u32 = 1 << 0;
    pub const BUILDING: u32 = 1 << 1;
    pub const TERRAIN: u32 = 1 << 2;
    pub const PROJECTILE: u32 = 1 << 3;
    pub const DEBRIS: u32 = 1 << 4;

    pub fn tank() -> Self {
        Self {
            membership: Self::TANK,
            filter: Self::BUILDING | Self::TERRAIN,
        }
    }

    pub fn building() -> Self {
        Self {
            membership: Self::BUILDING,
            filter: Self::TANK | Self::PROJECTILE,
        }
    }

    pub fn should_collide(&self, other: &CollisionLayers) -> bool {
        (self.membership & other.filter) != 0 && (self.filter & other.membership) != 0
    }
}
