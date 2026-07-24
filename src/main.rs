pub mod ecs;
mod game_scene;
pub mod map_loader;
pub mod movement;
pub mod ui;
pub mod macros;
pub mod pressure_plate;

use std::collections::{HashMap, HashSet};
use crate::ecs::{register_messages, CompletedTurn, DebugMode, ObstructedSet, SpecialTileSet, TurnCounter};
use crate::game_scene::game_scene_plugin;
use crate::map_loader::load_world_map;
use crate::movement::movement_plugin;
use crate::ui::ui_plugin;
use bevy::prelude::*;
use crate::pressure_plate::pressure_plate_plugin;

pub const MAX_TURN_COUNT: u32 = 1000;

pub const PLAYER_SIZE: Vec3 = vec3(1.0, 1.0, 1.0);
pub const GRID_SIZE: Vec2 = vec2(1.0, 1.0);

// in seconds
pub const ANIMATION_LENGTH: f32 = 0.25;

fn main() {
    let world_map = load_world_map().expect("failed to load world map");
    let debug_mode = std::env::args().any(|argument| argument == "--debug");

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(DebugMode(debug_mode))
        .insert_resource(world_map)
        .insert_resource(TurnCounter(MAX_TURN_COUNT))
        .insert_resource(ObstructedSet(HashSet::new()))
        .insert_resource(SpecialTileSet(HashMap::new()))
        .add_message::<CompletedTurn>()
        .add_plugins(register_messages)
        .add_plugins(game_scene_plugin)
        .add_plugins(movement_plugin)
        .add_plugins(ui_plugin)
        .add_plugins(pressure_plate_plugin)
        .run();
}
