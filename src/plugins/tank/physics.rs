use bevy::prelude::*;
use crate::components::{Tank, TankInput, TankMobility, TrackPhysics};

/// Update track physics based on smoothed input from TankInput
///
/// Oblicza docelowe prędkości gąsienic na podstawie inputu.
/// Nie modyfikuje pozycji ani prędkości - to robi system sił.
pub fn update_track_physics(
    mut tank_q: Query<(
        &TankInput,
        &TankMobility,
        &mut TrackPhysics,
    ), With<Tank>>,
) {
    let Ok((input, mobility, mut tracks)) = tank_q.get_single_mut() else {
        return;
    };

    // Use smoothed input values
    let forward_input = input.forward;
    let rotation_input = input.rotation;

    // Calculate target track speeds based on input
    let max_track_speed = if forward_input >= 0.0 {
        mobility.max_forward_speed
    } else {
        mobility.max_reverse_speed
    };

    // Calculate track speed needed for hull traverse
    // Angular velocity = (right - left) / track_width
    // So track_diff = angular_velocity * track_width
    let hull_traverse_rad = mobility.hull_traverse_speed.to_radians();
    let track_speed_for_rotation = hull_traverse_rad * tracks.track_width * 0.5;

    if rotation_input.abs() > 0.01 && forward_input.abs() < 0.01 {
        // Neutral steering - tracks move in opposite directions
        tracks.left_track_speed = rotation_input * track_speed_for_rotation;
        tracks.right_track_speed = -rotation_input * track_speed_for_rotation;
    } else if rotation_input.abs() > 0.01 {
        // Differential steering while moving - slower turn when moving
        let base_speed = forward_input * max_track_speed;

        // Reduce turn rate when moving (more realistic)
        let speed_factor = 1.0 - (mobility.current_speed.abs() / mobility.max_forward_speed).clamp(0.0, 0.6);
        let turn_diff = track_speed_for_rotation * speed_factor * rotation_input.abs();

        if rotation_input > 0.0 {
            // Turning right (D) - slow right track
            tracks.left_track_speed = base_speed + turn_diff;
            tracks.right_track_speed = base_speed - turn_diff;
        } else {
            // Turning left (A) - slow left track
            tracks.left_track_speed = base_speed - turn_diff;
            tracks.right_track_speed = base_speed + turn_diff;
        }
    } else {
        // Straight movement
        let target_speed = forward_input * max_track_speed;
        tracks.left_track_speed = target_speed;
        tracks.right_track_speed = target_speed;
    }

    // === HAMULCE GĄSIENICOWE ===
    // Niezależne hamulce redukują docelową prędkość gąsienicy
    if input.brake_left > 0.0 {
        tracks.left_track_speed *= 1.0 - input.brake_left;
    }
    if input.brake_right > 0.0 {
        tracks.right_track_speed *= 1.0 - input.brake_right;
    }
}
