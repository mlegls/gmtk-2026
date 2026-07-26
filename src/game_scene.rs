use crate::ecs::{Arrow, ArrowBlock, AvailableActions, CameraRig, ConveyorBelt, Direction, FaceId, Gate, GridLocation, InitialObstructedSet, ObstructedSet, Orientation, Player, PressurePlate, WallSet};
use crate::map_loader::{GroundTile, StuffTile, TileLayer};
use crate::signal_logic::TimerBank;
use crate::ui::ui;
use crate::{GRID_SIZE, map_loader::WorldMap};
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use std::f32::consts::PI;

pub fn game_scene_plugin(app: &mut App) {
    app.add_systems(Startup, scene.spawn()).add_systems(
        Startup,
        (generate_map, capture_initial_obstructions).chain(),
    );
}

fn capture_initial_obstructions(mut commands: Commands, obstructed_set: Res<ObstructedSet>) {
    commands.insert_resource(InitialObstructedSet(obstructed_set.0.clone()));
}

fn scene() -> impl SceneList {
    bsn_list![
        isometric_camera(), ui(),
        (
            player()
            Player
            AvailableActions::default()
            Orientation(Direction::North)
        ),
        arrow(),
    ]
}

fn isometric_camera() -> impl Scene {
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: 12.0,
        },
        ..OrthographicProjection::default_3d()
    });
    let rotation = Quat::from_euler(EulerRot::YXZ, PI / 4.0, -PI / 6.0, 0.0);
    bsn! {
        CameraRig
        Transform::default()
        Visibility::default()
        Children [
            (
                Camera3d
                template_value(projection)
                Transform {
                    rotation,
                    translation: vec3(40.0, 32.66, 40.0)
                }
            ),
            (
                PointLight {
                    shadow_maps_enabled: true,
                }
                Transform::from_xyz(0.0, 8.0, 0.0)
            ),
        ]
    }
}

fn player() -> impl Scene {
    bsn! {
        Transform::from_xyz(20.0 * GRID_SIZE.x, 0.5, 12.0 * GRID_SIZE.y)
        Visibility::default()
        GridLocation(Vec3::new(20.0, 0.0, 12.0))
        Children [
            template(|ctx| {
                Ok(WorldAssetRoot(ctx.resource::<AssetServer>().load(
                    GltfAssetLabel::Scene(0).from_asset("models/bot.gltf")
                )))
            })
            Transform::from_xyz(0.0, -0.5, 0.0)
        ]
    }
}

fn arrow() -> impl Scene {
    bsn! {
        Mesh3d(asset_value(Cuboid::new(0.1, 0.1, 0.4)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 0)))
        Transform::from_xyz(0.0, 1.5, 0.0)
        Arrow
        Children [
            (
                Mesh3d(asset_value(Cone::new(0.2, 0.6)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 0)))
                Transform {
                    translation: vec3(0.0, 0.0, -0.5),
                    rotation: Quat::from_rotation_x(-PI/2.0),
                }
            )
        ]
    }
}

fn generate_map(
    mut commands: Commands,
    mut obstructed_set: ResMut<ObstructedSet>,
    mut wall_set: ResMut<WallSet>,
    //mut special_tile_set: ResMut<SpecialTileSet>,
    world_map: Res<WorldMap>,
) {
    for (i, row) in world_map.tiles.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            if tile.ground == GroundTile::Void {
                // void
                obstructed_set.0.insert(uvec3(i as u32, 0, j as u32));
            }
            if tile.ground == GroundTile::Ground {
                // ground
                commands.spawn_scene(bsn! {
                    Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 255)))
                    Transform::from_xyz((i as f32) * GRID_SIZE.x, -5.0, (j as f32) * GRID_SIZE.y)
                });
            }
            if tile.ground == GroundTile::Altar {
                let location = uvec2(i as u32, j as u32);
                let Some(altar) = world_map.altars.get(&location) else {
                    error!("Altar at ({i}, {j}) has no action in map.json");
                    continue;
                };

                commands.spawn_scene(bsn! {
                    Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(168, 73, 255)))
                    Transform::from_xyz((i as f32) * GRID_SIZE.x, -5.0, (j as f32) * GRID_SIZE.y)
                    GridLocation(vec3(i as f32, 0.0, j as f32))
                    template_value(altar.clone())
                });
                obstructed_set.0.remove(&uvec3(location.x, 0, location.y));
            }

            if tile.ground == GroundTile::Conveyor {
                // conveyor belt
                let this_orientation = world_map.orientation[i][j];
                let direction = match this_orientation {
                    1 => Direction::North,
                    2 => Direction::West,
                    3 => Direction::South,
                    4 => Direction::East,
                    _ => {
                        error!("Found a conveyor at {}, {} with no orientation", i, j);
                        Direction::North
                    }
                };

                let rotation = Quat::from_rotation_y(this_orientation as f32 * PI / 2.0);
                commands.spawn_scene(bsn! {
                    Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(99, 99, 99)))
                    Transform {
                        translation: vec3((i as f32) * GRID_SIZE.x, -5.0, (j as f32) * GRID_SIZE.y)
                        rotation,
                    }
                    GridLocation(vec3(i as f32, 0.0, j as f32))
                    ConveyorBelt {
                        initial_orientation: direction,
                        initial_rotation: rotation,
                    }
                    Orientation(direction)
                    Children [
                        (
                            Mesh3d(asset_value(Cone::new(0.2, 0.6)))
                            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(168, 73, 255)))
                            Transform {
                                translation: vec3(0.0, 5.0, 0.0)
                                rotation: Quat::from_euler(EulerRot::YXZ, -PI/2.0, -PI/2.0, 0.0),
                            }
                        )
                    ]
                });
            }

            if tile.ground == GroundTile::ArrowBlock {
                // arrow block
                let this_orientation = world_map.orientation[i][j];
                let direction = match this_orientation {
                    1 => Direction::North,
                    2 => Direction::West,
                    3 => Direction::South,
                    4 => Direction::East,
                    _ => {
                        error!("Found an arrow block at {}, {} with no orientation", i, j);
                        Direction::North
                    }
                };
                let rotation = Quat::from_rotation_y(this_orientation as f32 * PI / 2.0);

                commands.spawn_scene(bsn! {
                    Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(0, 255, 0)))
                    Transform {
                        translation: vec3((i as f32) * GRID_SIZE.x, -5.0, (j as f32) * GRID_SIZE.y)
                        rotation,
                    }
                    GridLocation(vec3(i as f32, 0.0, j as f32))
                    ArrowBlock {
                        initial_orientation: direction,
                        initial_rotation: rotation,
                    }
                    Orientation(direction)
                    Children [
                        (
                            Mesh3d(asset_value(Cone::new(0.2, 0.6)))
                            MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 255)))
                            Transform {
                                translation: vec3(0.0, 5.0, 0.0)
                                rotation: Quat::from_euler(EulerRot::YXZ, -PI/2.0, -PI/2.0, 0.0),
                            }
                        )
                    ]
                });
            }
        }
    }
    for (i, row) in world_map.tiles.iter().enumerate() {
        for (j, tile) in row.iter().enumerate() {
            if tile.stuff == StuffTile::Wall {
                let location = uvec3(i as u32, 0, j as u32);
                commands.spawn_scene(bsn! {
                    Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(64, 64, 64)))
                    Transform::from_xyz(i as f32 * GRID_SIZE.x, 0.5, j as f32 * GRID_SIZE.y)
                    GridLocation(vec3(i as f32, 0.0, j as f32))
                });
                obstructed_set.0.insert(location);
                wall_set.0.insert(location);
            }
            if tile.stuff == StuffTile::PressurePlate {
                commands.spawn_scene(bsn! {
                    /*Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 100, 100)))*/
                    template(|ctx| {
                        Ok(WorldAssetRoot(ctx.resource::<AssetServer>().load(
                            GltfAssetLabel::Scene(0).from_asset("models/pressure_plate.gltf")
                        )))
                    })
                    Transform::from_xyz((i as f32) * GRID_SIZE.x, -1.0, (j as f32) * GRID_SIZE.y)
                    GridLocation(vec3(i as f32, 0.0, j as f32))
                    PressurePlate
                });
            }
            if tile.stuff == StuffTile::Bridge {
                let grid_location = uvec2(i as u32, j as u32);
                let Some(segment) = bridge_segment(&world_map.tiles, i, j) else {
                    warn!("Bridge at ({i}, {j}) has no neighboring bridge tile");
                    continue;
                };

                match segment {
                    BridgeSegment::HorizontalMiddle => {
                        commands.spawn_scene(bridge_middle(grid_location, Quat::IDENTITY));
                        obstructed_set
                            .0
                            .remove(&uvec3(grid_location.x, 0, grid_location.y));
                    }
                    BridgeSegment::VerticalMiddle => {
                        commands.spawn_scene(bridge_middle(
                            grid_location,
                            Quat::from_rotation_y(PI / 2.0),
                        ));
                        obstructed_set
                            .0
                            .remove(&uvec3(grid_location.x, 0, grid_location.y));
                    }
                    BridgeSegment::BottomEnd => {
                        commands.spawn_scene(bridge_end(
                            grid_location,
                            Quat::from_rotation_y(PI / 2.0),
                        ));
                    }
                    BridgeSegment::TopEnd => {
                        commands.spawn_scene(bridge_end(
                            grid_location,
                            Quat::from_rotation_y(-PI / 2.0),
                        ));
                    }
                    BridgeSegment::LeftEnd => {
                        commands.spawn_scene(bridge_end(grid_location, Quat::IDENTITY));
                    }
                    BridgeSegment::RightEnd => {
                        commands.spawn_scene(bridge_end(grid_location, Quat::from_rotation_y(PI)));
                    }
                }
            }
            if tile.stuff == StuffTile::Gate {
                // gate
                let this_orientation = world_map.orientation[i][j];
                commands.spawn_scene(bsn! {
                    template(|ctx| {
                        Ok(WorldAssetRoot(ctx.resource::<AssetServer>().load(
                            GltfAssetLabel::Scene(0).from_asset("models/gate/gate_border.gltf")
                        )))
                    })
                    Transform {
                        translation: vec3((i as f32) * GRID_SIZE.x, 0.0, (j as f32) * GRID_SIZE.y)
                        rotation: Quat::from_rotation_y(this_orientation as f32 * PI/2.0),
                    }
                });
                commands.spawn_scene(bsn! {
                    template(|ctx| {
                        Ok(WorldAssetRoot(ctx.resource::<AssetServer>().load(
                            GltfAssetLabel::Scene(0).from_asset("models/gate/gate.gltf")
                        )))
                    })
                    GridLocation(vec3(i as f32, 0.0, j as f32))
                    Transform {
                        translation: vec3((i as f32) * GRID_SIZE.x, 0.0, (j as f32) * GRID_SIZE.y)
                        rotation: Quat::from_rotation_y(this_orientation as f32 * PI/2.0),
                    }
                    template_value(Gate::Closed)
                });
                if this_orientation == 0 {
                    warn!("A gate at ({}, {}) was not given an orientation", i, j)
                }
                if this_orientation == 2 || this_orientation == 4 {
                    // north or south
                    obstructed_set.0.insert(uvec3(i as u32, 0, j as u32 - 1));
                    obstructed_set.0.insert(uvec3(i as u32, 0, j as u32));
                    obstructed_set.0.insert(uvec3(i as u32, 0, j as u32 + 1));
                } else if this_orientation == 1 || this_orientation == 3 {
                    // east or west
                    obstructed_set.0.insert(uvec3(i as u32 - 1, 0, j as u32));
                    obstructed_set.0.insert(uvec3(i as u32, 0, j as u32));
                    obstructed_set.0.insert(uvec3(i as u32 + 1, 0, j as u32));
                }
            }
            if tile.stuff == StuffTile::FaceId {
                let location = uvec3(i as u32, 0, j as u32);
                let this_orientation = world_map.orientation[i][j];
                let direction = match this_orientation {
                    1 => Direction::North,
                    2 => Direction::West,
                    3 => Direction::South,
                    4 => Direction::East,
                    _ => {
                        error!("Found an arrow block at {}, {} with no orientation", i, j);
                        Direction::North
                    }
                };
                let entity = commands.spawn_scene(bsn! {
                    template(|ctx| {
                        Ok(WorldAssetRoot(ctx.resource::<AssetServer>().load(
                            GltfAssetLabel::Scene(0).from_asset("models/face_id_lock.gltf")
                        )))
                    })
                    Transform {
                        translation: vec3(i as f32 * GRID_SIZE.x, 0.0, j as f32 * GRID_SIZE.y)
                        rotation: { direction.to_rotation() * Quat::from_rotation_y(PI) },
                    }
                    GridLocation(vec3(i as f32, 0.0, j as f32))
                    FaceId { direction }
                });
                warn!("face id entity: {:?}", entity.id());
                obstructed_set.0.insert(location);
            }
        }
    }
    for (location, slots) in &world_map.timer_banks {
        commands.spawn((
            Transform::from_xyz(
                location.x as f32 * GRID_SIZE.x,
                3.0,
                location.y as f32 * GRID_SIZE.y,
            ),
            GridLocation(vec3(location.x as f32, 0.0, location.y as f32)),
            TimerBank::new(slots, &world_map.timers),
        ));
    }
}
fn bridge_middle(grid_location: UVec2, rotation: Quat) -> impl Scene {
    bsn! {
        template(|ctx| {
            Ok(WorldAssetRoot(ctx.resource::<AssetServer>().load(
                GltfAssetLabel::Scene(0).from_asset("models/bridge/bridge_body.gltf")
            )))
        })
        Transform {
            translation: vec3(grid_location.x as f32 * GRID_SIZE.x, 0.0, grid_location.y as f32 * GRID_SIZE.y),
            rotation,
        }
    }
}
fn bridge_end(grid_location: UVec2, rotation: Quat) -> impl Scene {
    bsn! {
        template(|ctx| {
            Ok(WorldAssetRoot(ctx.resource::<AssetServer>().load(
                GltfAssetLabel::Scene(0).from_asset("models/bridge/bridge_pillars_a.gltf")
            )))
        })
        Transform {
            translation: vec3(grid_location.x as f32 * GRID_SIZE.x, 0.0, grid_location.y as f32 * GRID_SIZE.y),
            rotation,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BridgeSegment {
    TopEnd,
    LeftEnd,
    VerticalMiddle,
    HorizontalMiddle,
    BottomEnd,
    RightEnd,
}

fn bridge_segment(layer: &TileLayer, i: usize, j: usize) -> Option<BridgeSegment> {
    let up = is_bridge(layer, i as isize, j as isize - 1);
    let down = is_bridge(layer, i as isize, j as isize + 1);
    let left = is_bridge(layer, i as isize - 1, j as isize);
    let right = is_bridge(layer, i as isize + 1, j as isize);

    if up && down {
        Some(BridgeSegment::VerticalMiddle)
    } else if left && right {
        Some(BridgeSegment::HorizontalMiddle)
    } else if up {
        Some(BridgeSegment::BottomEnd)
    } else if down {
        Some(BridgeSegment::TopEnd)
    } else if left {
        Some(BridgeSegment::RightEnd)
    } else if right {
        Some(BridgeSegment::LeftEnd)
    } else {
        None
    }
}

fn is_bridge(layer: &TileLayer, i: isize, j: isize) -> bool {
    if i < 0 || j < 0 {
        return false;
    }

    layer
        .get(i as usize)
        .and_then(|row| row.get(j as usize))
        .is_some_and(|tile| tile.stuff == StuffTile::Bridge)
}
