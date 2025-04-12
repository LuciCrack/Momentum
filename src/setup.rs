use bevy::prelude::*;
use avian3d::prelude::*;
use bevy::color::palettes::tailwind;

use crate::{Player, PlayerCamera, CameraSensitivity};

// Set up the player and camera
pub fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let player_mesh = meshes.add(Capsule3d::new(0.6, 0.6));
    let player_material = materials.add(Color::srgb_u8(200, 89, 111));

    // Player and camera
    commands.spawn((
         Mesh3d(player_mesh),
         MeshMaterial3d(player_material),
         Transform::from_xyz(0.0, 5.0, 0.0),
         RigidBody::Dynamic,
         Collider::capsule(0.6, 0.5),
         Friction::new(0.2),
         LinearDamping(0.1),
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

// Setup lights 
pub fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        PointLight {
            color: Color::from(tailwind::ROSE_300),
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(-2.0, 7.0, -0.75),
    ));
}

pub fn spawn_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let white = materials.add(Color::WHITE);
    let black = materials.add(Color::BLACK);

    const SUP: f32 = 150.0;

    // Floor
    spawn_platform(&mut commands, meshes.as_mut(), white.clone(), Vec3::new(SUP, 0.1, SUP), Vec3::ZERO);

    // Cube
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::new(1.5, 1.5, 1.5))), 
        MeshMaterial3d(black.clone()),
        RigidBody::Kinematic,
        Collider::cuboid(1.5, 1.5, 1.5),
        Restitution::new(0.0),
        Transform::from_xyz(2.0, 0.8, 2.0)
    ));
    // Small ball
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
    // Medium ball
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(0.8))),
        MeshMaterial3d(black.clone()),
        RigidBody::Dynamic,
        Collider::sphere(0.8),
        Friction::new(1.0),
        LinearDamping(0.5),
        AngularDamping(2.0),
        Transform::from_xyz(10.0, 5.0, -7.0),
    ));
    // Large ball
    commands.spawn((
        Mesh3d(meshes.add(Sphere::new(1.4))),
        MeshMaterial3d(black.clone()),
        RigidBody::Dynamic,
        Collider::sphere(1.4),
        Friction::new(1.0),
        LinearDamping(0.5),
        AngularDamping(2.0),
        Transform::from_xyz(-6.0, 5.0, -7.0),
    ));
    // Walls
    spawn_wall(&mut commands, meshes.as_mut(), black.clone(), Vec3::new(1.0, 20.0, 20.0), Vec3::new(10.0, 0.8, 0.0));
    spawn_wall(&mut commands, meshes.as_mut(), black.clone(), Vec3::new(1.0, 20.0, 20.0), Vec3::new(-10.0, 0.8, 0.0));
    spawn_wall(&mut commands, meshes.as_mut(), black.clone(), Vec3::new(20.0, 20.0, 1.0), Vec3::new(0.0, 0.8, -10.0));
    spawn_wall(&mut commands, meshes.as_mut(), black.clone(), Vec3::new(20.0, 20.0, 1.0), Vec3::new(0.0, 0.8, 10.0));
}

pub fn spawn_wall(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    size: Vec3,
    position: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(size))),
        MeshMaterial3d::<StandardMaterial>(material),
        Transform::from_translation(position),
        RigidBody::Kinematic,
        Collider::cuboid(size.x, size.y, size.z),
    ));
}

pub fn spawn_platform(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    material: Handle<StandardMaterial>,
    size: Vec3,
    position: Vec3,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::from_size(size))),
        MeshMaterial3d::<StandardMaterial>(material),
        Transform::from_translation(position),
        RigidBody::Static,
        Collider::cuboid(size.x / 2.0, size.y / 2.0, size.z / 2.0),
        Friction::new(1.0),
    ));
}
