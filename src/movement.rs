use crate::ecs::{
    Arrow, AvailableActions, CameraRig, CompletedTurn, DebugMode, Direction, GateEntrySet,
    GridLocation, Moving, ObstructedSet, Orientation, Player, PlayerAction, TurnCounter, WallSet,
};
use crate::map_loader::{MAP_HEIGHT, MAP_WIDTH};
use crate::sfx::{PlaySfx, Sfx, SfxSystems};
use crate::story::GamePhase;
use crate::{ANIMATION_LENGTH, PLAYER_SIZE};
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use std::f32::consts::PI;
use std::time::Instant;

const VANTAGE_POINT: UVec2 = uvec2(27, 30);
const NORMAL_VIEW_HEIGHT: f32 = 12.0;
const OVERVIEW_VIEW_HEIGHT: f32 = MAP_HEIGHT as f32 + 8.0;
const VANTAGE_TRANSITION_SECONDS: f32 = 1.0;

#[derive(Resource, Default)]
struct VantageCamera {
    blend: f32,
}

pub fn movement_plugin(app: &mut App) {
    app.init_resource::<VantageCamera>()
        .add_systems(Update, toggle_actions)
        .add_systems(Update, input.run_if(in_state(GamePhase::Playing)))
        .add_systems(Update, movement_sfx.in_set(SfxSystems::Trigger))
        .add_systems(Update, (do_movement, follow_camera).chain());
    //.add_systems(Update, detect_special_tile);
}

#[derive(Component, Clone, Debug)]
pub struct CameraTurn {
    initial_rotation: Quat,
}

fn shift_pressed(keys: &ButtonInput<KeyCode>) -> bool {
    keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight)
}

fn action_just_pressed(
    keys: &ButtonInput<KeyCode>,
    available_actions: &AvailableActions,
    action: PlayerAction,
) -> bool {
    available_actions.contains(action) && keys.just_pressed(action.key_code())
}

fn toggle_actions(
    debug_mode: Res<DebugMode>,
    keys: Res<ButtonInput<KeyCode>>,
    mut available_actions: Single<&mut AvailableActions, With<Player>>,
) {
    if !**debug_mode || !shift_pressed(&keys) {
        return;
    }

    for action in PlayerAction::ALL {
        if keys.just_pressed(action.key_code()) {
            let available = available_actions.toggle(action);
            info!(
                "{:?} is now {}",
                action.key_code(),
                if available {
                    "available"
                } else {
                    "unavailable"
                }
            );
        }
    }
}

fn movement_sfx(
    moving: Query<(&Moving, &GridLocation), (With<Player>, Added<Moving>)>,
    mut play_sfx: MessageWriter<PlaySfx>,
) {
    for (moving, location) in &moving {
        let sfx = match moving.direction {
            Direction::North | Direction::East | Direction::South | Direction::West => {
                Some(Sfx::Roll)
            }
            Direction::SlideLeft | Direction::SlideRight => Some(Sfx::Slide),
            Direction::Left | Direction::Right | Direction::Around => Some(Sfx::Turn),
            Direction::Wait => None,
        };

        if let Some(sfx) = sfx {
            play_sfx.write(PlaySfx::at(sfx, location));
        }
    }
}

pub(crate) fn input(
    player: Single<
        (
            Entity,
            &Transform,
            &AvailableActions,
            &GridLocation,
            &Orientation,
        ),
        (With<Player>, Without<Moving>),
    >,
    camera: Single<(Entity, &Transform), (With<CameraRig>, Without<Player>)>,
    debug_mode: Res<DebugMode>,
    keys: Res<ButtonInput<KeyCode>>,
    obstructed_set: Res<ObstructedSet>,
    wall_set: Res<WallSet>,
    gate_entries: Res<GateEntrySet>,
    mut commands: Commands,
) {
    if **debug_mode && shift_pressed(&keys) {
        return;
    }

    let (player_entity, transform, available_actions, grid_location, orientation) =
        player.into_inner();
    let (camera_entity, camera_transform) = camera.into_inner();
    if action_just_pressed(&keys, available_actions, PlayerAction::RollForward) {
        // roll north 1 space
        let future_grid_location =
            grid_location.0 + orientation.0.to_rotation() * Direction::North.to_vec_direction();
        let Some(roll_in_place) = roll_behavior(
            future_grid_location.as_uvec3(),
            &obstructed_set,
            &wall_set,
            &gate_entries,
        ) else {
            return;
        };

        commands.entity(player_entity).insert(Moving {
            direction: Direction::North,
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place,
        });
    }
    if action_just_pressed(&keys, available_actions, PlayerAction::RollBackward) {
        // roll south
        let future_grid_location =
            grid_location.0 + orientation.0.to_rotation() * Direction::South.to_vec_direction();
        let Some(roll_in_place) = roll_behavior(
            future_grid_location.as_uvec3(),
            &obstructed_set,
            &wall_set,
            &gate_entries,
        ) else {
            return;
        };

        commands.entity(player_entity).insert(Moving {
            direction: Direction::South,
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place,
        });
    }
    if action_just_pressed(&keys, available_actions, PlayerAction::RollLeft) {
        // roll west
        let future_grid_location =
            grid_location.0 + orientation.0.to_rotation() * Direction::West.to_vec_direction();
        let Some(roll_in_place) = roll_behavior(
            future_grid_location.as_uvec3(),
            &obstructed_set,
            &wall_set,
            &gate_entries,
        ) else {
            return;
        };

        commands.entity(player_entity).insert(Moving {
            direction: Direction::West,
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place,
        });
    }
    if action_just_pressed(&keys, available_actions, PlayerAction::RollRight) {
        // roll east
        let future_grid_location =
            grid_location.0 + orientation.0.to_rotation() * Direction::East.to_vec_direction();
        let Some(roll_in_place) = roll_behavior(
            future_grid_location.as_uvec3(),
            &obstructed_set,
            &wall_set,
            &gate_entries,
        ) else {
            return;
        };

        commands.entity(player_entity).insert(Moving {
            direction: Direction::East,
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place,
        });
    }
    if action_just_pressed(&keys, available_actions, PlayerAction::TurnLeft) {
        // turn left
        commands.entity(player_entity).insert(Moving {
            direction: Direction::Left,
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place: false,
        });
        // orbit camera
        commands.entity(camera_entity).insert(CameraTurn {
            initial_rotation: camera_transform.rotation,
        });
    }
    if action_just_pressed(&keys, available_actions, PlayerAction::TurnRight) {
        // turn right
        commands.entity(player_entity).insert(Moving {
            direction: Direction::Right,
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place: false,
        });
        // orbit camera
        commands.entity(camera_entity).insert(CameraTurn {
            initial_rotation: camera_transform.rotation,
        });
    }
    if action_just_pressed(&keys, available_actions, PlayerAction::TurnAround) {
        // spin 180 degrees
        commands.entity(player_entity).insert(Moving {
            direction: Direction::Around,
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place: false,
        });
        // orbit camera
        commands.entity(camera_entity).insert(CameraTurn {
            initial_rotation: camera_transform.rotation,
        });
    }
    if action_just_pressed(&keys, available_actions, PlayerAction::SlideLeft) {
        // slide left (translate, no roll)
        let future_grid_location =
            grid_location.0 + orientation.0.to_rotation() * Direction::West.to_vec_direction();
        let Some(roll_in_place) = roll_behavior(
            future_grid_location.as_uvec3(),
            &obstructed_set,
            &wall_set,
            &gate_entries,
        ) else {
            return;
        };

        commands.entity(player_entity).insert(Moving {
            direction: if roll_in_place {
                Direction::West
            } else {
                Direction::SlideLeft
            },
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place,
        });
    }
    if action_just_pressed(&keys, available_actions, PlayerAction::SlideRight) {
        // slide right
        let future_grid_location =
            grid_location.0 + orientation.0.to_rotation() * Direction::East.to_vec_direction();
        let Some(roll_in_place) = roll_behavior(
            future_grid_location.as_uvec3(),
            &obstructed_set,
            &wall_set,
            &gate_entries,
        ) else {
            return;
        };

        commands.entity(player_entity).insert(Moving {
            direction: if roll_in_place {
                Direction::East
            } else {
                Direction::SlideRight
            },
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place,
        });
    }
    if action_just_pressed(&keys, available_actions, PlayerAction::Wait) {
        // wait in place (pass turn)
        commands.entity(player_entity).insert(Moving {
            direction: Direction::Wait,
            start: Instant::now(),
            initial_rotation: transform.rotation,
            roll_in_place: false,
        });
    }
}

fn roll_behavior(
    destination: UVec3,
    obstructed_set: &ObstructedSet,
    wall_set: &WallSet,
    gate_entries: &GateEntrySet,
) -> Option<bool> {
    if wall_set.0.contains(&destination) {
        Some(true)
    } else if gate_entries.locations.contains(&destination) {
        gate_entries
            .open_next_turn
            .contains(&destination)
            .then_some(false)
    } else if obstructed_set.0.contains(&destination) {
        None
    } else {
        Some(false)
    }
}

pub fn do_movement(
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
    let old_location = grid_location.0.as_uvec3();

    let completed = match moving.direction {
        Direction::North | Direction::East | Direction::South | Direction::West => {
            if moving.roll_in_place {
                roll_player_in_place(
                    &mut transform,
                    &orientation,
                    moving.direction,
                    moving.initial_rotation,
                    progress,
                )
            } else {
                roll_player(
                    &mut transform,
                    &mut grid_location,
                    &orientation,
                    moving.direction,
                    moving.initial_rotation,
                    progress,
                )
            }
        }
        Direction::SlideLeft => translate_player(
            &mut transform,
            &mut grid_location,
            orientation.0.to_rotation() * Direction::West.to_grid_location_offset(),
            progress,
        ),
        Direction::SlideRight => translate_player(
            &mut transform,
            &mut grid_location,
            orientation.0.to_rotation() * Direction::East.to_grid_location_offset(),
            progress,
        ),
        Direction::Left | Direction::Right | Direction::Around => {
            let Some(camera_turn) = camera_turn else {
                return;
            };
            let (turns, positive) = match moving.direction {
                Direction::Left => (1.0, true),
                Direction::Right => (1.0, false),
                Direction::Around => (2.0, false),
                _ => unreachable!(),
            };

            rotate_camera_around_y(
                &mut camera_transform,
                camera_turn,
                progress * turns,
                positive,
            );
            rotate_around_y(
                &mut arrow_transform,
                &orientation.0.to_rotation(),
                progress * turns,
                positive,
            );

            let completed = rotate_player(
                &mut transform,
                &mut orientation,
                moving.direction,
                moving.initial_rotation,
                progress,
            );
            if completed {
                commands.entity(camera_entity).remove::<CameraTurn>();
                camera_transform.rotation = match moving.direction {
                    Direction::Left => {
                        Quat::from_rotation_y(PI / 2.0) * camera_turn.initial_rotation
                    }
                    Direction::Right => {
                        Quat::from_rotation_y(-PI / 2.0) * camera_turn.initial_rotation
                    }
                    Direction::Around => Quat::from_rotation_y(PI) * camera_turn.initial_rotation,
                    _ => unreachable!(),
                };
                arrow_transform.rotation = orientation.0.to_rotation();
            }
            completed
        }
        Direction::Wait => progress >= 1.0,
    };

    if completed {
        commands.entity(player_entity).remove::<Moving>();
        **turn_counter -= 1;
        completed_turn_sender.write(CompletedTurn {
            old_rotation: moving.initial_rotation,
            old_location,
            new_location: grid_location.0.as_uvec3(),
        });
    }
}
/*fn detect_special_tile(
    mut completed_turn_reader: MessageReader<CompletedTurn>,
    mut special_tile_set: ResMut<SpecialTileSet>,
) {
    for completed_turn in completed_turn_reader.read() {
        if let Some((tile_type, entity)) = special_tile_set.0.get(&completed_turn.new_location) {

        }
        if completed_turn.new_location != completed_turn.old_location
            && let Some((tile_type, entity)) = special_tile_set.0.get(&completed_turn.old_location)
        {
        }
    }
}*/

/// hard tp
pub fn place_player(
    transform: &mut Transform,
    grid_location: &mut GridLocation,
    location: Vec3,
    rotation: Quat,
) {
    grid_location.0 = location;
    transform.translation = grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y / 2.0, 0.0);
    transform.rotation = rotation;
}

/// face toward (for arrows)
pub fn face_player(transform: &mut Transform, orientation: &mut Orientation, target: Direction) {
    let turn = if orientation.0 == target {
        return;
    } else if orientation.0.turn_right() == target {
        Direction::Right
    } else if orientation.0.turn_left() == target {
        Direction::Left
    } else {
        Direction::Around
    };

    let initial_rotation = transform.rotation;
    rotate_player(transform, orientation, turn, initial_rotation, 1.0);
}

/// roll (like wasd)
pub fn roll_player(
    transform: &mut Transform,
    grid_location: &mut GridLocation,
    orientation: &Orientation,
    direction: Direction,
    initial_rotation: Quat,
    progress: f32,
) -> bool {
    let orient_rot = orientation.0.to_rotation();
    let (axis, angle) = roll_axis_angle(orientation, direction);
    let rotation_offset = Quat::from_axis_angle(axis, progress * angle);
    rotate_around(
        transform,
        &initial_rotation,
        orient_rot * direction.to_pivot(),
        &rotation_offset,
        grid_location,
    );

    if progress < 1.0 {
        return false;
    }

    grid_location.0 += orient_rot * direction.to_grid_location_offset();
    transform.translation = grid_location.to_world_space() + vec3(0.0, PLAYER_SIZE.y / 2.0, 0.0);
    transform.rotation = Quat::from_axis_angle(axis, angle) * initial_rotation;
    true
}

/// roll in place
pub fn roll_player_in_place(
    transform: &mut Transform,
    orientation: &Orientation,
    direction: Direction,
    initial_rotation: Quat,
    progress: f32,
) -> bool {
    let (axis, angle) = roll_axis_angle(orientation, direction);
    transform.rotation = Quat::from_axis_angle(axis, progress.min(1.0) * angle) * initial_rotation;
    progress >= 1.0
}

fn roll_axis_angle(orientation: &Orientation, direction: Direction) -> (Vec3, f32) {
    let orient_rot = orientation.0.to_rotation();
    match direction {
        Direction::North => (orient_rot * Vec3::X, -PI / 2.0),
        Direction::South => (orient_rot * Vec3::X, PI / 2.0),
        Direction::East => (orient_rot * Vec3::Z, -PI / 2.0),
        Direction::West => (orient_rot * Vec3::Z, PI / 2.0),
        _ => panic!("rolling requires a cardinal direction"),
    }
}

/// translate (like zc)
pub fn translate_player(
    transform: &mut Transform,
    grid_location: &mut GridLocation,
    offset: Vec3,
    progress: f32,
) -> bool {
    let target_grid_location = grid_location.0 + offset;
    let player_offset = vec3(0.0, PLAYER_SIZE.y / 2.0, 0.0);
    let start = grid_location.to_world_space() + player_offset;
    let target = GridLocation(target_grid_location).to_world_space() + player_offset;
    transform.translation = start.lerp(target, progress.min(1.0));

    if progress < 1.0 {
        return false;
    }

    grid_location.0 = target_grid_location;
    true
}

/// rotate (like qex)
pub fn rotate_player(
    transform: &mut Transform,
    orientation: &mut Orientation,
    direction: Direction,
    initial_rotation: Quat,
    progress: f32,
) -> bool {
    let (turns, positive) = match direction {
        Direction::Left => (1.0, true),
        Direction::Right => (1.0, false),
        Direction::Around => (2.0, false),
        _ => panic!("rotate_player requires Left, Right, or Around"),
    };
    rotate_around_y(transform, &initial_rotation, progress * turns, positive);

    if progress < 1.0 {
        return false;
    }

    let (rotation, new_orientation) = match direction {
        Direction::Left => (Quat::from_rotation_y(PI / 2.0), orientation.0.turn_left()),
        Direction::Right => (Quat::from_rotation_y(-PI / 2.0), orientation.0.turn_right()),
        Direction::Around => (
            Quat::from_rotation_y(PI),
            orientation.0.turn_left().turn_left(),
        ),
        _ => unreachable!(),
    };
    transform.rotation = rotation * initial_rotation;
    *orientation = Orientation(new_orientation);
    true
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
    let rotation = Quat::from_rotation_y(progress.min(2.0) * sign * PI / 2.0);
    transform.rotation = rotation * turn.initial_rotation;
}

fn follow_camera(
    time: Res<Time>,
    player: Single<(&Transform, &GridLocation), (With<Player>, Without<CameraRig>)>,
    mut camera_rig: Single<&mut Transform, (With<CameraRig>, Without<Player>, Without<Camera3d>)>,
    camera: Single<
        (&mut Transform, &mut Projection),
        (With<Camera3d>, Without<CameraRig>, Without<Player>),
    >,
    mut vantage_camera: ResMut<VantageCamera>,
) {
    let (player_transform, grid_location) = player.into_inner();
    let target_blend = if grid_location.0.x as u32 == VANTAGE_POINT.x
        && grid_location.0.z as u32 == VANTAGE_POINT.y
    {
        1.0
    } else {
        0.0
    };
    let blend_step = time.delta_secs() / VANTAGE_TRANSITION_SECONDS;
    if vantage_camera.blend < target_blend {
        vantage_camera.blend = (vantage_camera.blend + blend_step).min(target_blend);
    } else {
        vantage_camera.blend = (vantage_camera.blend - blend_step).max(target_blend);
    }

    // Smoothstep keeps both ends of the transition from snapping.
    let blend = vantage_camera.blend * vantage_camera.blend * (3.0 - 2.0 * vantage_camera.blend);
    let map_center = vec3(
        (MAP_WIDTH as f32 - 1.0) / 2.0,
        player_transform.translation.y,
        (MAP_HEIGHT as f32 - 1.0) / 2.0,
    );
    camera_rig.translation = player_transform.translation.lerp(map_center, blend);

    let normal_transform = Transform {
        translation: vec3(40.0, 32.66, 40.0),
        rotation: Quat::from_euler(EulerRot::YXZ, PI / 4.0, -PI / 6.0, 0.0),
        ..default()
    };
    let overview_transform =
        Transform::from_xyz(0.0, 120.0, 0.0).looking_at(Vec3::ZERO, Vec3::NEG_Z);
    let (mut camera_transform, mut projection) = camera.into_inner();
    camera_transform.translation = normal_transform
        .translation
        .lerp(overview_transform.translation, blend);
    camera_transform.rotation = normal_transform
        .rotation
        .slerp(overview_transform.rotation, blend);

    if let Projection::Orthographic(orthographic) = projection.as_mut() {
        orthographic.scaling_mode = ScalingMode::FixedVertical {
            viewport_height: NORMAL_VIEW_HEIGHT.lerp(OVERVIEW_VIEW_HEIGHT, blend),
        };
    }
}
