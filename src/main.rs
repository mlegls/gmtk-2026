pub mod altar;
pub mod altar_ui;
pub mod arrow_block;
pub mod bridge;
pub mod conveyor_belt;
pub mod ecs;
pub mod face_id;
mod game_scene;
pub mod gate;
pub mod map_json;
pub mod map_loader;
pub mod movement;
pub mod music;
pub mod pressure_plate;
pub mod sfx;
pub mod signal_logic;
pub mod story;
pub mod timer_visual;
pub mod ui;

use crate::altar::altar_plugin;
use crate::arrow_block::arrow_block_plugin;
use crate::bridge::bridge_plugin;
use crate::conveyor_belt::conveyor_belt_plugin;
use crate::ecs::{CompletedTurn, DebugMode, ObstructedSet, SignalSystems, TurnCounter, WallSet};
use crate::face_id::face_id_plugin;
use crate::game_scene::game_scene_plugin;
use crate::gate::gate_plugin;
use crate::map_json::MapJson;
use crate::map_loader::load_world_map;
use crate::movement::movement_plugin;
use crate::music::music_plugin;
use crate::pressure_plate::pressure_plate_plugin;
use crate::sfx::sfx_plugin;
use crate::signal_logic::signal_logic_plugin;
use crate::story::story_plugin;
use crate::timer_visual::timer_visual_plugin;
use crate::ui::ui_plugin;
use bevy::prelude::*;
use std::collections::HashSet;

pub const MAX_TURN_COUNT: u32 = 1000;
pub const PLAYER_START: Vec3 = vec3(27.0, 0.0, 1.0);

pub const PLAYER_SIZE: Vec3 = vec3(1.0, 1.0, 1.0);
pub const GRID_SIZE: Vec2 = vec2(1.0, 1.0);

// in seconds
pub const ANIMATION_LENGTH: f32 = 0.25;

fn main() {
    let map_json = include_bytes!("../assets/maps/map.json").as_ref();
    let map_json: MapJson = serde_json::from_reader(map_json).unwrap();

    let world_map = load_world_map(&map_json).expect("failed to load world map");
    let debug_mode = std::env::args().any(|argument| argument == "--debug");

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(map_json)
        .insert_resource(DebugMode(debug_mode))
        .insert_resource(world_map)
        .insert_resource(TurnCounter(MAX_TURN_COUNT))
        .insert_resource(ObstructedSet(HashSet::new()))
        .insert_resource(WallSet(HashSet::new()))
        //.insert_resource(SpecialTileSet(HashMap::new()))
        .add_message::<CompletedTurn>()
        .add_plugins(game_scene_plugin)
        .add_plugins(movement_plugin)
        .add_plugins(ui_plugin)
        .add_plugins(pressure_plate_plugin)
        .add_plugins(gate_plugin)
        .add_plugins(conveyor_belt_plugin)
        .add_plugins(arrow_block_plugin)
        .add_plugins(bridge_plugin)
        .add_plugins(altar_plugin)
        .add_plugins(signal_logic_plugin)
        .add_plugins(timer_visual_plugin)
        .add_plugins(music_plugin)
        .add_plugins(sfx_plugin)
        .add_plugins(story_plugin)
        .add_plugins(face_id_plugin)
        .configure_sets(
            Update,
            (
                SignalSystems::Write,
                SignalSystems::Timer,
                SignalSystems::Read,
            )
                .chain(),
        )
        .run();
}
