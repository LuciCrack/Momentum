use bevy::prelude::*;
use bevy::color::palettes::tailwind;
use bevy_rapier3d::prelude::Velocity;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, 
            (
                setup,
                spawn_lights
            ))
        .add_systems(Update, update_player)
        .run();
}


fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_mesh = meshes.add(Capsule3d::new(1.0, 1.0));
    let player_material = materials.add(Color::srgb_u8(200, 89, 111));

    let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::splat(10.0)));
    let material = materials.add(Color::WHITE);

    commands.spawn((Mesh3d(floor), MeshMaterial3d(material.clone())));
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(0.5, 0.5, 0.5))), 
        MeshMaterial3d(material.clone()),
        Transform::from_xyz(2.0, 1.0, 2.0)
    ));

    commands.spawn((
        Mesh3d(player_mesh),
        MeshMaterial3d(player_material),
        Transform::from_xyz(0.0, 1.5, 0.0),
        Velocity::zero(),
        Visibility::Visible,
        Player,
    ))
    .with_children(|parent| {
      parent.spawn((
          Camera3d::default(),
          Projection::from(PerspectiveProjection {
              fov: 90.0_f32.to_radians(),
              ..default()
          }),
          PlayerCamera
      ));
    });

}

fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        PointLight {
            color: Color::from(tailwind::ROSE_300),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 4.0, -0.75),
    ));
}

fn update_player(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(&Children, &mut Transform), (With<Player>, Without<PlayerCamera>)>,
    cameras: Query<(&Transform, Entity), With<PlayerCamera>>,
    time: Res<Time>,
    accumulated_mouse_motion: Res<AccumulatedMouseMotion>,
) {
    const SPEED: f32 = 1.0;
    for (children, mut player_transform) in &mut players {
        for &child in children {
            if let Ok((camera_transform, _)) = cameras.get(child) {
                let mut movement_dir = Vec3::ZERO;
                let forward = camera_transform.forward();
                let right = camera_transform.right();
                if keyboard_input.pressed(KeyCode::KeyW) {
                    movement_dir += *forward;
                }
                if keyboard_input.pressed(KeyCode::KeyA) {
                    movement_dir -= *right;
                }
                if keyboard_input.pressed(KeyCode::KeyA) {
                    movement_dir -= *forward;
                }
                if keyboard_input.pressed(KeyCode::KeyD) {
                    movement_dir += *right;
                }
                if movement_dir != Vec3::ZERO {
                    movement_dir = movement_dir.normalize();
                    player_transform.translation += movement_dir * (SPEED * time.delta_secs());
                }
            }
        }
    }
}

#[derive(Component)]
struct Player;

#[derive(Component)]
struct PlayerCamera;
