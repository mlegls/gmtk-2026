use std::collections::HashSet;
use crate::ecs::{Arrow, AvailableActions, CameraRig, Direction, GridLocation, ObstructedSet, Orientation, Player, PressurePlate, SpecialTileSet, SpecialTileType};
use crate::ui::ui;
use crate::{GRID_SIZE, map_loader::WorldMap};
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use std::f32::consts::PI;
use crate::map_loader::MapLayer;

pub fn game_scene_plugin(app: &mut App) {
    app.add_systems(Startup, scene.spawn())
        .add_systems(Startup, generate_map);
}

fn scene() -> impl SceneList {
    bsn_list![
        isometric_camera(), point_light(), ui(),
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
        Children [
            (
                Camera3d
                template_value(projection)
                Transform {
                    rotation,
                    translation: vec3(40.0, 32.66, 40.0)
                }
            )
        ]
    }
}

fn point_light() -> impl Scene {
    bsn! {
        PointLight {
            shadow_maps_enabled: true,
        }
        Transform::from_xyz(16.0 * GRID_SIZE.x, 8.0, 16.0 * GRID_SIZE.y)
    }
}

fn player() -> impl Scene {
    bsn! {
        Transform::from_xyz(16.0 * GRID_SIZE.x, 0.5, 16.0 * GRID_SIZE.y)
        GridLocation(Vec3::new(16.0, 0.0, 16.0))
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
    mut special_tile_set: ResMut<SpecialTileSet>,
    world_map: Res<WorldMap>,
) {
    for (i, row) in world_map.ground.iter().enumerate() {
        for (j, &location) in row.iter().enumerate() {
            if location == 0 {
                obstructed_set
                    .0
                    .insert(uvec3(i as u32, 0, j as u32));
            }
            if location == 1 {
                commands.spawn_scene(bsn! {
                    Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 255)))
                    Transform::from_xyz((i as f32) * GRID_SIZE.x, -5.0, (j as f32) * GRID_SIZE.y)
                });
            }
            if location == 2 {
                let entity = commands.spawn_scene(bsn! {
                    Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 100, 100)))
                    Transform::from_xyz((i as f32) * GRID_SIZE.x, -5.0, (j as f32) * GRID_SIZE.y)
                    PressurePlate
                });
                special_tile_set.0.insert(uvec3(i as u32, 0, j as u32),
                                          (SpecialTileType::PressurePlate, entity.id()));
            }
            /*bsn! {
                Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 255)))
                Transform::from_xyz(0.0, -5.0, 0.0)
            }*/
        }
    }
    let mut already_processed = HashSet::new();
    for (i, row) in world_map.stuff.iter().enumerate() {
        for (j, &location) in row.iter().enumerate() {
            if already_processed.contains(&uvec2(i as u32, j as u32)) { continue }
            if location == 3 {
                // bridge
                let mut bridge_pieces = Vec::new();
                find_bridge(world_map.stuff, i, j, &mut bridge_pieces, &mut already_processed);

                for (location, segment) in bridge_pieces {
                    match segment {
                        BridgeSegment::HorizontalMiddle => {
                            commands.spawn_scene(bridge_middle(location, Quat::IDENTITY));
                            obstructed_set.0.remove(&uvec3(location.x, 0, location.y));
                        }
                        BridgeSegment::VerticalMiddle => {
                            commands.spawn_scene(bridge_middle(location, Quat::from_rotation_y(PI/2.0)));
                            obstructed_set.0.remove(&uvec3(location.x, 0, location.y));
                        }
                        BridgeSegment::BottomEnd => {
                            commands.spawn_scene(bridge_end(location, Quat::from_rotation_y(PI/2.0)));
                        }
                        BridgeSegment::TopEnd => {
                            commands.spawn_scene(bridge_end(location, Quat::from_rotation_y(-PI/2.0)));
                        }
                        BridgeSegment::LeftEnd => {
                            commands.spawn_scene(bridge_end(location, Quat::from_rotation_y(0.0)));
                        }
                        BridgeSegment::RightEnd => {
                            commands.spawn_scene(bridge_end(location, Quat::from_rotation_y(PI)));
                        }
                    }
                }
            }
        }
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

pub enum BridgeSegment {
    TopEnd,
    LeftEnd,
    VerticalMiddle,
    HorizontalMiddle,
    BottomEnd,
    RightEnd,
}
/// recursively searches all nearby cells to find bridge pieces. returns list of bridge coordinates through bridge_pieces
fn find_bridge(
    layer: MapLayer,
    i: usize,
    j: usize,
    bridge_pieces: &mut Vec<(UVec2, BridgeSegment)>,
    already_processed: &mut HashSet<UVec2>,
) {
    let up = layer.get(i)
        .map_or(false, |row| row.get(j-1)
            .map_or(false, |location| *location == 3 &&
                !already_processed.contains(&uvec2(i as u32, j as u32 - 1))));
    let down = layer.get(i)
        .map_or(false, |row| row.get(j+1)
            .map_or(false, |location| *location == 3 &&
                !already_processed.contains(&uvec2(i as u32, j as u32 + 1))));

    let left = layer.get(i-1)
        .map_or(false, |row| row.get(j)
            .map_or(false, |location| *location == 3 &&
                !already_processed.contains(&uvec2(i as u32 - 1, j as u32))));
    let right = layer.get(i+1)
        .map_or(false, |row| row.get(j)
            .map_or(false, |location| *location == 3 &&
                !already_processed.contains(&uvec2(i as u32 + 1, j as u32))));

    if up || down {
        // ok, this bridge is going vertically
        find_bridge_vertical(layer, i, j, bridge_pieces, already_processed);
    } else if left || right {
        // ok, this bridge is going horizontally
        find_bridge_horizontal(layer, i, j, bridge_pieces, already_processed);
    }
}

fn find_bridge_vertical(
    layer: MapLayer,
    i: usize,
    j: usize,
    bridge_pieces: &mut Vec<(UVec2, BridgeSegment)>,
    already_processed: &mut HashSet<UVec2>,
) {
    let up = layer.get(i)
        .map_or(false, |row| row.get(j-1)
            .map_or(false, |location| *location == 3));
    let up_processed = already_processed.contains(&uvec2(i as u32, j as u32 - 1));

    let down = layer.get(i)
        .map_or(false, |row| row.get(j+1)
            .map_or(false, |location| *location == 3));
    let down_processed = already_processed.contains(&uvec2(i as u32, j as u32 + 1));

    if up && down {
        already_processed.insert(uvec2(i as u32, j as u32));
        bridge_pieces.push((uvec2(i as u32, j as u32), BridgeSegment::VerticalMiddle));
        if !up_processed {
            find_bridge_vertical(layer, i, j - 1, bridge_pieces, already_processed);
        }
        if !down_processed {
            find_bridge_vertical(layer, i, j + 1, bridge_pieces, already_processed);
        }
    } else if up { // down is false
        already_processed.insert(uvec2(i as u32, j as u32));
        bridge_pieces.push((uvec2(i as u32, j as u32), BridgeSegment::BottomEnd));
        if !up_processed {
            find_bridge_vertical(layer, i, j - 1, bridge_pieces, already_processed);
        }
    } else if down { // up is false
        already_processed.insert(uvec2(i as u32, j as u32));
        bridge_pieces.push((uvec2(i as u32, j as u32), BridgeSegment::TopEnd));
        if !down_processed {
            find_bridge_vertical(layer, i, j + 1, bridge_pieces, already_processed);
        }
    }
}

fn find_bridge_horizontal(
    layer: MapLayer,
    i: usize,
    j: usize,
    bridge_pieces: &mut Vec<(UVec2, BridgeSegment)>,
    already_processed: &mut HashSet<UVec2>,
) {
    let left = layer.get(i-1)
        .map_or(false, |row| row.get(j)
            .map_or(false, |location| *location == 3));
    let left_processed = already_processed.contains(&uvec2(i as u32 - 1, j as u32));

    let right = layer.get(i+1)
        .map_or(false, |row| row.get(j)
            .map_or(false, |location| *location == 3));
    let right_processed = already_processed.contains(&uvec2(i as u32 + 1, j as u32));

    if left && right {
        already_processed.insert(uvec2(i as u32, j as u32));
        bridge_pieces.push((uvec2(i as u32, j as u32), BridgeSegment::HorizontalMiddle));
        if !left_processed {
            find_bridge_horizontal(layer, i - 1, j, bridge_pieces, already_processed);
        }
        if !right_processed {
            find_bridge_horizontal(layer, i + 1, j, bridge_pieces, already_processed);
        }
    } else if left { // right is false
        already_processed.insert(uvec2(i as u32, j as u32));
        bridge_pieces.push((uvec2(i as u32, j as u32), BridgeSegment::RightEnd));
        if !left_processed {
            find_bridge_horizontal(layer, i - 1, j, bridge_pieces, already_processed);
        }
    } else if right { // left is false
        already_processed.insert(uvec2(i as u32, j as u32));
        bridge_pieces.push((uvec2(i as u32, j as u32), BridgeSegment::LeftEnd));
        if !right_processed {
            find_bridge_horizontal(layer, i + 1, j, bridge_pieces, already_processed);
        }
    }
}