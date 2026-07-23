use crate::ecs::{
    Arrow, CameraRig, CompletedTurn, Direction, GridLocation, Moving, Orientation, Player,
    TurnCounter,
};
use crate::{ANIMATION_LENGTH, PLAYER_SIZE};
use bevy::prelude::*;
use std::f32::consts::PI;
use std::time::Instant;

pub fn movement_plugin(app: &mut App) {
    app.add_systems(Update, input)
        .add_systems(Update, (do_movement, follow_camera).chain());
}

#[derive(Component, Clone, Debug)]
struct CameraTurn {
    initial_rotation: Quat,
}

fn input(
    player: Single<(Entity, &Transform), (With<Player>, Without<Moving>)>,
    camera: Single<(Entity, &Transform), (With<CameraRig>, Without<Player>)>,
    keys: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    let (player_entity, transform) = player.into_inner();
    let (camera_entity, camera_transform) = camera.into_inner();
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
    if keys.just_pressed(KeyCode::KeyQ) {
        // turn left
        commands.entity(player_entity).insert(Moving {
            direction: Direction::Left,
            start: Instant::now(),
            initial_rotation: transform.rotation,
        });
        // orbit camera
        commands.entity(camera_entity).insert(CameraTurn {
            initial_rotation: camera_transform.rotation,
        });
    }
    if keys.just_pressed(KeyCode::KeyE) {
        // turn right
        commands.entity(player_entity).insert(Moving {
            direction: Direction::Right,
            start: Instant::now(),
            initial_rotation: transform.rotation,
        });
        // orbit camera
        commands.entity(camera_entity).insert(CameraTurn {
            initial_rotation: camera_transform.rotation,
        });
    }
    if keys.just_pressed(KeyCode::KeyX) {
        // spin 180 degrees
        commands.entity(player_entity).insert(Moving {
            direction: Direction::Around,
            start: Instant::now(),
            initial_rotation: transform.rotation,
        });
    }
}
fn do_movement(
    player: Single<
        (
            Entity,
            &mut Transform,
            &mut GridLocation,
            &Moving,
            &mut Orientation,
        ),
        With<Player>,
    >,
    arrow: Single<&mut Transform, (With<Arrow>, Without<Player>)>,
    camera: Single<
        (Entity, &mut Transform, Option<&CameraTurn>),
        (With<CameraRig>, Without<Player>, Without<Arrow>),
    >,
    mut turn_counter: ResMut<TurnCounter>,
    mut completed_turn_sender: MessageWriter<CompletedTurn>,
    mut commands: Commands,
) {
    if **turn_counter == 0 {
        return;
    }
    let (player_entity, mut transform, mut grid_location, moving, mut orientation) =
        player.into_inner();
    let mut arrow_transform = arrow.into_inner();
    let (camera_entity, mut camera_transform, camera_turn) = camera.into_inner();
    let progress = moving.start.elapsed().as_secs_f32() / ANIMATION_LENGTH;

    let orient_rot = orientation.0.to_rotation();
    match moving.direction {
        Direction::North => {
            // rotates about x axis
            rotate_around(
                &mut transform,
                &moving.initial_rotation,
                orient_rot * moving.direction.to_pivot(),
                &Quat::from_axis_angle(orient_rot * Vec3::X, progress * -PI / 2.0),
                &grid_location,
            );
            if progress >= 1.0 {
                grid_location.0 += orient_rot * vec3(0.0, 0.0, -1.0);
                commands.entity(player_entity).remove::<Moving>();
                transform.translation =
                    grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y / 2.0, 0.0);
                transform.rotation = Quat::from_axis_angle(orient_rot * Vec3::X, -PI / 2.0)
                    * moving.initial_rotation;

                **turn_counter -= 1;
                completed_turn_sender.write(CompletedTurn);
            }
        }
        Direction::South => {
            rotate_around(
                &mut transform,
                &moving.initial_rotation,
                orient_rot * moving.direction.to_pivot(),
                &Quat::from_axis_angle(orient_rot * Vec3::X, progress * PI / 2.0),
                &grid_location,
            );
            if progress >= 1.0 {
                grid_location.0 += orient_rot * vec3(0.0, 0.0, 1.0);
                commands.entity(player_entity).remove::<Moving>();
                transform.translation =
                    grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y / 2.0, 0.0);
                transform.rotation =
                    Quat::from_axis_angle(orient_rot * Vec3::X, PI / 2.0) * moving.initial_rotation;

                **turn_counter -= 1;
                completed_turn_sender.write(CompletedTurn);
            }
        }
        Direction::East => {
            rotate_around(
                &mut transform,
                &moving.initial_rotation,
                orient_rot * moving.direction.to_pivot(),
                &Quat::from_axis_angle(orient_rot * Vec3::Z, progress * -PI / 2.0),
                &grid_location,
            );
            if progress >= 1.0 {
                grid_location.0 += orient_rot * vec3(1.0, 0.0, 0.0);
                commands.entity(player_entity).remove::<Moving>();
                transform.translation =
                    grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y / 2.0, 0.0);
                transform.rotation = Quat::from_axis_angle(orient_rot * Vec3::Z, -PI / 2.0)
                    * moving.initial_rotation;

                **turn_counter -= 1;
                completed_turn_sender.write(CompletedTurn);
            }
        }
        Direction::West => {
            rotate_around(
                &mut transform,
                &moving.initial_rotation,
                orient_rot * moving.direction.to_pivot(),
                &Quat::from_axis_angle(orient_rot * Vec3::Z, progress * PI / 2.0),
                &grid_location,
            );
            if progress >= 1.0 {
                grid_location.0 += orient_rot * vec3(-1.0, 0.0, 0.0);
                commands.entity(player_entity).remove::<Moving>();
                transform.translation =
                    grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y / 2.0, 0.0);
                transform.rotation =
                    Quat::from_axis_angle(orient_rot * Vec3::Z, PI / 2.0) * moving.initial_rotation;

                **turn_counter -= 1;
                completed_turn_sender.write(CompletedTurn);
            }
        }
        Direction::Left => {
            let Some(camera_turn) = camera_turn else {
                return;
            };
            rotate_camera_around_y(&mut camera_transform, camera_turn, progress, true);
            rotate_around_y(
                &mut arrow_transform,
                &orientation.0.to_rotation(),
                progress,
                true,
            );
            if progress >= 1.0 {
                commands.entity(player_entity).remove::<Moving>();
                commands.entity(camera_entity).remove::<CameraTurn>();

                *orientation = Orientation(orientation.0.turn_left());
                arrow_transform.rotation = orientation.0.to_rotation();

                **turn_counter -= 1;
                completed_turn_sender.write(CompletedTurn);
            }
        }
        Direction::Right => {
            let Some(camera_turn) = camera_turn else {
                return;
            };
            rotate_camera_around_y(&mut camera_transform, camera_turn, progress, false);
            rotate_around_y(
                &mut arrow_transform,
                &orientation.0.to_rotation(),
                progress,
                false,
            );
            if progress >= 1.0 {
                commands.entity(player_entity).remove::<Moving>();
                commands.entity(camera_entity).remove::<CameraTurn>();

                *orientation = Orientation(orientation.0.turn_right());
                arrow_transform.rotation = orientation.0.to_rotation();

                **turn_counter -= 1;
                completed_turn_sender.write(CompletedTurn);
            }
        }
        Direction::Around => {
            rotate_around_y(
                &mut transform,
                &moving.initial_rotation,
                progress * 2.0,
                false,
            );
            rotate_around_y(
                &mut arrow_transform,
                &orientation.0.to_rotation(),
                progress * 2.0,
                false,
            );
            if progress >= 1.0 {
                commands.entity(player_entity).remove::<Moving>();
                transform.rotation = Quat::from_axis_angle(Vec3::Y, PI) * moving.initial_rotation;

                *orientation = Orientation(orientation.0.turn_left().turn_left());
                arrow_transform.rotation = orientation.0.to_rotation();

                **turn_counter -= 1;
                completed_turn_sender.write(CompletedTurn);
            }
        }
    }
}

fn rotate_around(
    transform: &mut Transform,
    initial_rotation: &Quat,
    offset: Vec3,
    rotation_offset: &Quat,
    grid_location: &GridLocation,
) {
    let pivot_point = grid_location.to_world_space() + offset;
    //transform.rotate_around(grid_location.to_world_space() + vec3(sign*PLAYER_SIZE.x/2.0, 0.0, 0.0), Quat::from_rotation_z(time.delta_secs()/ANIMATION_LENGTH * -sign));
    transform.translation = pivot_point
        + rotation_offset
            * (grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y / 2.0, 0.0) - pivot_point);
    transform.rotation = rotation_offset * initial_rotation;
}
fn rotate_around_y(
    transform: &mut Transform,
    initial_rotation: &Quat,
    progress: f32,
    is_positive: bool,
) {
    let sign = if is_positive { 1.0 } else { -1.0 };
    //let pivot_point = grid_location.to_world_space();
    //transform.rotate_around(grid_location.to_world_space() + vec3(0.0, 0.0, sign*PLAYER_SIZE.z/2.0), Quat::from_rotation_x(time.delta_secs()/ANIMATION_LENGTH * sign));
    transform.rotation = Quat::from_rotation_y(progress * sign * PI / 2.0) * initial_rotation;
}

fn rotate_camera_around_y(
    transform: &mut Transform,
    turn: &CameraTurn,
    progress: f32,
    is_positive: bool,
) {
    let sign = if is_positive { 1.0 } else { -1.0 };
    let rotation = Quat::from_rotation_y(progress.min(1.0) * sign * PI / 2.0);
    transform.rotation = rotation * turn.initial_rotation;
}

fn follow_camera(
    player: Single<&Transform, (With<Player>, Without<CameraRig>)>,
    mut camera: Single<&mut Transform, (With<CameraRig>, Without<Player>)>,
) {
    camera.translation.x = player.translation.x;
    camera.translation.z = player.translation.z;
}
