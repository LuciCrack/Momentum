pub mod setup;
pub mod systems;

use std::time::Duration;
use bevy::prelude::*;
use avian3d::prelude::*;

use crate::setup::{setup, spawn_lights, spawn_level};
use crate::systems::{input, jumping, update_player, lock_cursor, unlock_cursor, pause, unpause};

fn main() {
    App::new()
        // Pretty self explainatory
        // Run app with default plugins and physics from avian3d
        .add_plugins((DefaultPlugins, PhysicsPlugins::default()))
        .insert_resource(Gravity(Vec3::NEG_Y * 30.0))// Set gravity resource to -30
        .insert_resource(Time::<Physics>::default())
        .insert_resource(DashCooldown {
            timer: Timer::from_seconds(1.0, TimerMode::Once),
        })
        .init_state::<GameState>() 
        .add_systems(Startup, 
            (
                setup,
                spawn_level,
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


#[derive(Clone, Copy, Default, Eq, PartialEq, Debug, Hash, States)]
pub enum GameState {
    #[default]
    Game,
    Menu,
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

#[derive(Resource)]
pub struct DashCooldown {
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
pub struct CameraSensitivity(Vec2);

impl Default for CameraSensitivity {
    fn default() -> Self {
        Self(
            //          x,     y
            Vec2::new(0.003, 0.002),
        )
    }
}
