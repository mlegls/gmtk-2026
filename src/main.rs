mod game_scene;
pub mod ecs;
pub mod movement;
pub mod ui;

use bevy::prelude::*;
use crate::ecs::{CompletedTurn, TurnCounter};
use crate::game_scene::game_scene_plugin;
use crate::movement::movement_plugin;
use crate::ui::ui_plugin;

pub const MAX_TURN_COUNT: u32 = 10;

pub const PLAYER_SIZE: Vec3 = vec3(1.0, 1.0, 1.0);
pub const GRID_SIZE: Vec2 = vec2(1.0, 1.0);

// in seconds
pub const ANIMATION_LENGTH: f32 = 0.25;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(TurnCounter(MAX_TURN_COUNT))
        .add_message::<CompletedTurn>()
        .add_plugins(game_scene_plugin)
        .add_plugins(movement_plugin)
        .add_plugins(ui_plugin)
        .run();
}
