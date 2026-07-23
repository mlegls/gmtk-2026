mod game_scene;
pub mod ecs;
pub mod movement;

use bevy::prelude::*;
use crate::game_scene::game_scene_plugin;
use crate::movement::movement_plugin;

pub const PLAYER_SIZE: Vec3 = vec3(1.0, 1.0, 1.0);
pub const GRID_SIZE: Vec2 = vec2(1.0, 1.0);

// in seconds
pub const ANIMATION_LENGTH: f32 = 0.5;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(game_scene_plugin)
        .add_plugins(movement_plugin)
        .run();
}
