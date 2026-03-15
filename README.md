# Tank 3D - Monte Cassino

A custom 3D tank simulation game and underlying engine built completely from scratch in Rust using the Bevy ECS framework.

## Architecture

This project leverages the **Entity-Component-System (ECS)** architecture provided by Bevy. Game data is strictly decoupled from logic: attributes like physics properties, movement intents, and combat states are stored in isolated **Components**, while autonomous **Systems** process entities possessing these components during specific execution phases (e.g., `PhysicsCalculation`, `TurretControl`, `DispersionCalculation`). 

A key architectural decision was to build **custom physics and collision detection engines** (`src/physics/`) as well as procedural terrain tools, rather than relying on heavy off-the-shelf black-box engines like Rapier or external editors. This allows for extremely fine-grained control over tank-specific mechanics such as differential steering, track slip ratios, and gyroscopic turret effects, making the simulation highly specialized and performant. The entire codebase is heavily modularized using the Plugin pattern (`TankPlugin`, `EnvironmentPlugin`, `CameraPlugin`), ensuring clean separation of concerns and maintainability.

## Main Components & Systems

- **`TankMobility` & `TrackPhysics`**: Core components handling the complex physics of an armored vehicle. They simulate engine torque curves, RPM, differential steering, track friction, Pacejka slip ratios, and even aerodynamic drag.
- **`RigidBody` & `Collider`**: Custom-built physics representation. Colliders support AABB, OBB (using the Separating Axis Theorem), and Spheres. Rigid bodies track velocities, mass, and apply continuous forces.
- **`Turret`, `Barrel`, & `TurretGyroscopic`**: Manage the physical aiming mechanics, handling traverse speeds, elevation limits, and the gyroscopic force transfer between the rotating turret and the tank hull.
- **`TankInput`**: An input-smoothing component that bridges raw player keyboard/mouse input and the physics engine, providing realistic acceleration ramping and track braking.
- **`GunDispersion`**: Implements dynamic, World-of-Tanks style aiming logic, where accuracy is penalized by movement and turret rotation, and recovers over an aim time.
- **Custom Camera System**: A dedicated camera system that smoothly orbits, zooms, and follows the tank, adjusting to the tank's position and terrain using raycasting.

## Tech Stack

- **[Rust](https://www.rust-lang.org/)**: The core programming language, providing memory safety and high performance.
- **[Bevy Engine](https://bevyengine.org/) (v0.15)**: The data-driven game engine used for the ECS foundation, app routing, rendering, and windowing (with custom `jpeg` feature enabled for assets).

## How to Run

Ensure you have the Rust toolchain installed. To get the best performance out of the physics and simulation calculations, it is highly recommended to run the project in release mode:

```bash
git clone <repository-url>
cd tank-engine-main
cargo run --release
```

## What I Learned

- **Mastering ECS**: Gained a deep, practical understanding of the Entity-Component-System paradigm by decoupling complex game state representations from execution logic loops.
- **Custom Physics Algorithms**: Learned the mathematics and implementation details behind 3D physics, including continuous velocity tracking, raycasting, and writing a custom collision detection system using the Separating Axis Theorem (SAT).
- **Procedural Rendering & Terrain**: Gained hands-on experience with procedural generation, manipulating 3D meshes, rendering pipelines, and generating noise map textures dynamically.
- **Software Architecture**: Discovered the immense value of Rust's trait system and Bevy's Plugin architecture for organizing a complex, heavily interdependent simulation into clean, self-contained modules.
