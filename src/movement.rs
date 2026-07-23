use std::f32::consts::PI;
use bevy::prelude::*;
use crate::ecs::{GridLocation, Player};
use crate::PLAYER_SIZE;

pub fn movement_plugin(app: &mut App) {
    app.add_systems(Update, movement);
}

fn movement(
    mut player: Single<(&mut Transform, &mut GridLocation), With<Player>>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (mut transform, mut grid_location) = player.into_inner();
    if keys.just_pressed(KeyCode::KeyW) {
        // roll north 1 space
        transform.rotate_around(grid_location.to_world_space() + vec3(0.0, 0.0, -PLAYER_SIZE.z/2.0), Quat::from_rotation_x(-PI/2.0));
        grid_location.move_north();
    }
    if keys.just_pressed(KeyCode::KeyS) {
        // roll south
        transform.rotate_around(grid_location.to_world_space() + vec3(0.0, 0.0, PLAYER_SIZE.z/2.0), Quat::from_rotation_x(PI/2.0));
        grid_location.move_south();
    }
    if keys.just_pressed(KeyCode::KeyA) {
        // roll west
        transform.rotate_around(grid_location.to_world_space() + vec3(-PLAYER_SIZE.x/2.0, 0.0, 0.0), Quat::from_rotation_z(PI/2.0));
        grid_location.move_west();
    }
    if keys.just_pressed(KeyCode::KeyD) {
        // roll east
        transform.rotate_around(grid_location.to_world_space() + vec3(PLAYER_SIZE.x/2.0, 0.0, 0.0), Quat::from_rotation_z(-PI/2.0));
        grid_location.move_east();
    }
}