use bevy::prelude::*;
use avian3d::prelude::*;
use std::f32::consts::FRAC_PI_2;
use bevy::window::{CursorGrabMode, PrimaryWindow, Window};
use bevy::input::mouse::AccumulatedMouseMotion;

use crate::{DashCooldown, PlayerCamera, Player, CameraSensitivity, GameState};

// Function to appear/dissappear the cursor
pub fn lock_cursor(
    mut windows: Query<&mut Window>,
primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    if let Ok(window_entity) = primary_window.get_single() {
        if let Ok(mut window) = windows.get_mut(window_entity) {
            window.cursor_options.grab_mode = CursorGrabMode::Locked;
            window.cursor_options.visible = false;
        }
    }
}
pub fn unlock_cursor(
    mut windows: Query<&mut Window>,
    primary_window: Query<Entity, With<PrimaryWindow>>,
) {
    if let Ok(window_entity) = primary_window.get_single() {
        if let Ok(mut window) = windows.get_mut(window_entity) {
            window.cursor_options.grab_mode = CursorGrabMode::None;
            window.cursor_options.visible = true;
        }
    }
}

// Pause or unpause physics time depending on game state (in systems)
pub fn pause(mut time: ResMut<Time<Physics>>) {
    time.pause();
}
pub fn unpause(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

// Handle input game state independent
pub fn input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    current_state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        match current_state.get() {
            GameState::Game => next_state.set(GameState::Menu),
            GameState::Menu => next_state.set(GameState::Game),
        }
    }
}

pub fn jumping(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    transforms: Query<&Transform>,
    mut entities: Query<(Entity, &mut LinearVelocity), With<Player>>,
    mut collision_event_reader: EventReader<Collision>,
) {
    if keyboard_input.just_pressed(KeyCode::Space) {
        for (entity, mut linear_vel) in entities.iter_mut() {
            let touching_ground = touching_ground(&mut collision_event_reader, entity, &transforms);
            if touching_ground {
                linear_vel.y = 12.0;
            }
        }
    }
}
// Checks wether an entity is touching something "beneath it".
fn touching_ground(
    collision_event_reader: &mut EventReader<Collision>,
    entity: Entity,
    transforms: &Query<&Transform>,
) -> bool {
    // Accidentally made wall jumping but its nice!
    for Collision(contact) in collision_event_reader.read() {
        let (self_entity, other_entity) = if contact.entity1 == entity {
            (contact.entity1, contact.entity2)
        } else if contact.entity2 == entity {
            (contact.entity2, contact.entity1)
        } else {
            continue;
        };
        // Compare Y positions to check if the collision is from below
        if let (Ok(self_transform), Ok(other_transform)) = (
            transforms.get(self_entity),
            transforms.get(other_entity),
        ) {
            let self_y = self_transform.translation.y;
            let other_y = other_transform.translation.y;

            // Allow a small epsilon tolerance
            if other_y < self_y - 0.01 {
                return true;
            }
        }
    }
    false
}
  
// Handle player movements
pub fn update_player(
    mut players: Query<(&mut Children, &mut Transform, &CameraSensitivity, &mut LinearVelocity),
        (Without<PlayerCamera>, With<Player>)>,
    mut cameras: Query<&mut Transform, With<PlayerCamera>>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut dash_timer: ResMut<DashCooldown>,
    time: Res<Time>,
) {
    const SPEED: f32 = 7.0; // Max speed
    const ACCELERATION: f32 = 30.0; // How fast to reach max speed
    
    for (player_children, mut player_transform, camera_sensitivity, mut linear_vel) in &mut players {
        for &child in player_children.iter() {
            if let Ok(mut transform) = cameras.get_mut(child) {
                // Setting direction to move
                let mut dir = Vec3::ZERO;
                let forward = transform.forward().as_vec3().with_y(0.0).normalize();
                let right = transform.right().as_vec3().with_y(0.0).normalize();

                for key in keyboard_input.get_pressed() { // Get direction input 
                    match key {
                    KeyCode::KeyW => dir += forward,
                    KeyCode::KeyA => dir -= right,
                    KeyCode::KeyS => dir -= forward,
                    KeyCode::KeyD => dir += right,
                    KeyCode::ShiftLeft => { // Dash
                        if dash_timer.ready() {
                            let forward = transform.forward();
                            let dash_strength = 20.0;

                            // Apply velocity in the forward direction
                            linear_vel.0 += forward * dash_strength;

                            // Reset cooldown
                            dash_timer.reset();
                        }
                    }
                    KeyCode::Escape => return,
                    _ => (),
                }
            }

            if dir == Vec3::ZERO {
                // Friction/drag
                linear_vel.0 *= 0.85; // reduce velocity by 15% each frame
                // Snap to zero if velocity is almost zero
                if linear_vel.0.length_squared() < 0.001 {
                    linear_vel.0 = Vec3::ZERO;
                }
            } else {
                // Apply velocity
                let desired_vel = dir * SPEED; // Desired Velocity, what the player wants
                let delta_vel = desired_vel - linear_vel.0; // What it will actually move
                let max_change = ACCELERATION * time.delta_secs(); // Clamp Change
                let delta = delta_vel.clamp_length_max(max_change);
                linear_vel.0 += delta; // Applyy
            }
            // Mouse Movements (rotate camera)
            let delta = accumulated_mouse_motion.delta;

            if delta != Vec2::ZERO {
                let delta_yaw = -delta.x * camera_sensitivity.x;
                let delta_pitch = -delta.y * camera_sensitivity.y;

                let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
                let yaw = yaw + delta_yaw;

                const PITCH_LIMIT: f32 = FRAC_PI_2 - 0.01;
                let pitch = (pitch + delta_pitch).clamp(-PITCH_LIMIT, PITCH_LIMIT);

                transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
            }
          }
      }
      // Reset player rotation to keep upright (prevents physics wobble)
      player_transform.rotation = Quat::IDENTITY;
    }
    dash_timer.tick(time.delta()); // Count down the cooldown timer
}
