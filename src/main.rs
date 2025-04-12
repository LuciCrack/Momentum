use std::time::Duration;
use bevy::prelude::*;
use bevy::color::palettes::tailwind;
use bevy::input::mouse::AccumulatedMouseMotion;
use avian3d::prelude::*;
use std::f32::consts::FRAC_PI_2;
use bevy::window::{CursorGrabMode, PrimaryWindow, Window};

fn main() {
    App::new()
        // Pretty self explainatory
        // Run app with default plugins and physics from avian3d
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Gravity(Vec3::NEG_Y * 38.0))// Set gravity resource to - **
        .insert_resource(Time::<Physics>::default()) // Insert time resource (physics only)
        .insert_resource(DashCooldown {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        })
        .init_state::<GameState>() 
        .add_systems(Startup, 
            (
                setup,
                spawn_lights
            ))
        // Run systems depending on game state
        .add_systems(Update,
            (
                jumping.run_if(in_state(GameState::Game)),
                update_player.run_if(in_state(GameState::Game)),
                input
            ))
        .add_systems(OnEnter(GameState::Game), (lock_cursor, unpause))
        .add_systems(OnEnter(GameState::Menu), (unlock_cursor, pause))
        .run();
}

// Set up the player and other structures such as floor and a cube
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_mesh = meshes.add(Capsule3d::new(0.6, 0.6));
    let player_material = materials.add(Color::srgb_u8(200, 89, 111));

    let white = materials.add(Color::WHITE);
    let black = materials.add(Color::BLACK);

    const SUP: f32 = 100.0;

    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(SUP, 0.1, SUP))), 
        MeshMaterial3d(white.clone()),
        RigidBody::Static,
        Friction::new(1.0),
        Collider::cuboid(SUP, 0.1, SUP),
    ));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.5, 1.5, 1.5))), 
        MeshMaterial3d(black.clone()),
        RigidBody::Kinematic,
        Collider::cuboid(1.5, 1.5, 1.5),
        Restitution::new(0.0),
        Transform::from_xyz(2.0, 1.0, 2.0)
    ));
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.3))),
        MeshMaterial3d(black.clone()),
        RigidBody::Dynamic,
        Collider::sphere(0.3),
        Friction::new(1.0),
        LinearDamping(0.5),
        AngularDamping(2.0),
        Transform::from_xyz(3.0, 5.0, 7.0),
    ));

    commands.spawn((
        Mesh3d(player_mesh),
        MeshMaterial3d(player_material),
        Transform::from_xyz(0.0, 1.5, 0.0),
        RigidBody::Dynamic,
        Collider::capsule(0.6, 0.6),
        Friction::new(0.4),
        LinearDamping(0.2),
        AngularDamping(2.0),
        Visibility::Visible,
        CameraSensitivity::default(),
        Player,
    ))
    .with_children(|parent| {
      parent.spawn((
          Camera3d::default(),
          Transform::from_xyz(0.0, 0.9, 0.0),
          Projection::from(PerspectiveProjection {
              fov: 100.0_f32.to_radians(),
              ..default()
          }),
          PlayerCamera,
      ));
    });
}

// Function to make dissapear the cursor when playing
fn lock_cursor(
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

// Appear cursor on Escape
fn unlock_cursor(
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
fn pause(mut time: ResMut<Time<Physics>>) {
    time.pause();
}
fn unpause(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

// setup lights 
fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        PointLight {
            color: Color::from(tailwind::ROSE_300),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 7.0, -0.75),
    ));
}

// Handle input game state independent
fn input(
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

fn jumping(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    transforms: Query<&Transform>,
    mut entities: Query<(Entity, &mut LinearVelocity), With<Player>>,
    mut collision_event_reader: EventReader<Collision>,
) {
    for key in keyboard_input.get_pressed() { // Get direction input 
        if key == &KeyCode::Space {
            for (entity, mut linear_vel) in entities.iter_mut() {
                let touching_ground = touching_ground(&mut collision_event_reader, entity, &transforms);
                if touching_ground {
                    linear_vel.y = 12.0;
                }
            }
        }
    }
}

fn touching_ground(
    collision_event_reader: &mut EventReader<Collision>,
    entity: Entity,
    transforms: &Query<&Transform>,
) -> bool {
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
            if other_y < self_y - 0.05 {
                return true;
            }
        }
    }
    false
}
  
// Handle player movements
fn update_player(
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
                let desired_vel = dir * SPEED; // Desired Velocity, what the player wants
                let delta_vel = desired_vel - linear_vel.0; // Different in desired and vel
                // Clamp Change
                let max_change = ACCELERATION * time.delta_secs();
                let delta = delta_vel.clamp_length_max(max_change);
                linear_vel.0 += delta; // Applyy
            }
            // Mouse Movements
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
      // Reset player rotation to keep upright (prevents physics wobble or mouse rotation)
      player_transform.rotation = Quat::IDENTITY;
    }

    dash_timer.tick(time.delta()); // Count down the cooldown timer

}

#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
enum GameState {
    #[default]
    Game,
    Menu,
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerCamera;

#[derive(Resource)]
struct DashCooldown {
    timer: Timer,
}

impl DashCooldown {
    fn ready(&self) -> bool {
        self.timer.finished()
    }

    fn reset(&mut self) {
        self.timer.reset();
    }

    fn tick(&mut self, delta: Duration) {
        self.timer.tick(delta);
    }
}

#[derive(Debug, Component, Deref, DerefMut)]
struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(
            //          x,     y
            Vec2::new(0.003, 0.002),
        )
    }
}
