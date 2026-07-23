use std::f32::consts::PI;
use std::time::Instant;
use bevy::prelude::*;
use crate::ecs::{Direction, GridLocation, Moving, Player};
use crate::{ANIMATION_LENGTH, PLAYER_SIZE};

pub fn movement_plugin(app: &mut App) {
    app
        .add_systems(Update, input)
        .add_systems(Update, do_movement);
}

fn input(
    mut player: Single<(Entity, &Transform), (With<Player>, Without<Moving>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let (player_entity, transform) = player.into_inner();
    if keys.just_pressed(KeyCode::KeyW) {
        // roll north 1 space
        commands.entity(player_entity).insert(Moving {
            direction: Direction::North,
            start: Instant::now(),
            initial_rotation: transform.rotation,
        });
    }
    if keys.just_pressed(KeyCode::KeyS) {
        // roll south
        commands.entity(player_entity).insert(Moving {
            direction: Direction::South,
            start: Instant::now(),
            initial_rotation: transform.rotation,
        });
    }
    if keys.just_pressed(KeyCode::KeyA) {
        // roll west
        commands.entity(player_entity).insert(Moving {
            direction: Direction::West,
            start: Instant::now(),
            initial_rotation: transform.rotation,
        });
    }
    if keys.just_pressed(KeyCode::KeyD) {
        // roll east
        commands.entity(player_entity).insert(Moving {
            direction: Direction::East,
            start: Instant::now(),
            initial_rotation: transform.rotation,
        });
    }
}
fn do_movement(
    mut player: Single<(Entity, &mut Transform, &mut GridLocation, &Moving), With<Player>>,
    mut commands: Commands,
) {
    let (player_entity, mut transform, mut grid_location, moving) = player.into_inner();
    let progress = moving.start.elapsed().as_secs_f32() / ANIMATION_LENGTH;

    match moving.direction {
        Direction::North => {
            rotate_around_x(&mut transform, &moving.initial_rotation, &grid_location, progress, false);
            if progress >= 1.0 {
                grid_location.move_north();
                commands.entity(player_entity).remove::<Moving>();
                transform.translation = grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y/2.0, 0.0);
                transform.rotation = Quat::from_axis_angle(Vec3::X, -PI/2.0) * moving.initial_rotation;
            }
        }
        Direction::South => {
            rotate_around_x(&mut transform, &moving.initial_rotation, &grid_location, progress, true);
            if progress >= 1.0 {
                grid_location.move_south();
                commands.entity(player_entity).remove::<Moving>();
                transform.translation = grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y/2.0, 0.0);
                transform.rotation = Quat::from_axis_angle(Vec3::X, PI/2.0) * moving.initial_rotation;
            }
        }
        Direction::East => {
            rotate_around_z(&mut transform, &moving.initial_rotation, &grid_location, progress, true);
            if progress >= 1.0 {
                grid_location.move_east();
                commands.entity(player_entity).remove::<Moving>();
                transform.translation = grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y/2.0, 0.0);
                transform.rotation = Quat::from_axis_angle(Vec3::Z, -PI/2.0) * moving.initial_rotation;
            }
        }
        Direction::West => {
            rotate_around_z(&mut transform, &moving.initial_rotation, &grid_location, progress, false);
            if progress >= 1.0 {
                grid_location.move_west();
                commands.entity(player_entity).remove::<Moving>();
                transform.translation = grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y/2.0, 0.0);
                transform.rotation = Quat::from_axis_angle(Vec3::Z, PI/2.0) * moving.initial_rotation;
            }
        }
    }
}

fn rotate_around_z(transform: &mut Transform, initial_rotation: &Quat, grid_location: &GridLocation, progress: f32, is_positive: bool) {
    let sign = if is_positive { 1.0 } else { -1.0 };
    let pivot_point = grid_location.to_world_space() + vec3(sign*PLAYER_SIZE.x/2.0, 0.0, 0.0);
    //transform.rotate_around(grid_location.to_world_space() + vec3(sign*PLAYER_SIZE.x/2.0, 0.0, 0.0), Quat::from_rotation_z(time.delta_secs()/ANIMATION_LENGTH * -sign));
    transform.translation = pivot_point + Quat::from_rotation_z(progress * -sign*PI/2.0)*(grid_location.to_world_space()+vec3(0.0, PLAYER_SIZE.y/2.0, 0.0) - pivot_point);
    transform.rotation = Quat::from_rotation_z(progress * -sign*PI/2.0) * initial_rotation;
}
fn rotate_around_x(transform: &mut Transform, initial_rotation: &Quat, grid_location: &GridLocation, progress: f32, is_positive: bool) {
    let sign = if is_positive { 1.0 } else { -1.0 };
    let pivot_point = grid_location.to_world_space() + vec3(0.0, 0.0, sign*PLAYER_SIZE.z/2.0);
    //transform.rotate_around(grid_location.to_world_space() + vec3(0.0, 0.0, sign*PLAYER_SIZE.z/2.0), Quat::from_rotation_x(time.delta_secs()/ANIMATION_LENGTH * sign));
    transform.translation = pivot_point + Quat::from_rotation_x(progress * sign*PI/2.0)*(grid_location.to_world_space()+vec3(0.0, PLAYER_SIZE.y/2.0, 0.0) - pivot_point);
    transform.rotation = Quat::from_rotation_x(progress * sign*PI/2.0) * initial_rotation;
}