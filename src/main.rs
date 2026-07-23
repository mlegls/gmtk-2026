mod game_scene;

use bevy::prelude::*;
use crate::game_scene::game_scene_plugin;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(game_scene_plugin)
        .run();
}
