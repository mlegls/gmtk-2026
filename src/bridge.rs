use std::f32::consts::PI;

use bevy::prelude::*;

use crate::GRID_SIZE;
use crate::PLAYER_START;
use crate::ecs::{
    GridLocation, InitialObstructedSet, Moving, ObstructedSet, Player, SignalSystems,
};
use crate::map_loader::{GroundTile, StuffTile, TileLayer, WorldMap};
use crate::movement::place_player;
use crate::sfx::{PlaySfx, Sfx, SfxSystems};
use crate::signal_logic::{SignalSnapshot, SwitchStates, TimerBank, activation_at};

const FALL_ACCELERATION: f32 = 24.0;
const FALL_LIMIT: f32 = -20.0;

#[derive(Component, Clone, Debug, Default)]
pub struct Bridge {
    initial_translation: Vec3,
    blocks_when_collapsed: bool,
    traversable_when_intact: bool,
    collapsed: bool,
    fall_speed: f32,
    wait_for_inactive: bool,
}

impl Bridge {
    fn new(
        initial_translation: Vec3,
        blocks_when_collapsed: bool,
        traversable_when_intact: bool,
    ) -> Self {
        Self {
            initial_translation,
            blocks_when_collapsed,
            traversable_when_intact,
            collapsed: false,
            fall_speed: 0.0,
            wait_for_inactive: false,
        }
    }

    pub fn reset(&mut self, transform: &mut Transform) {
        self.reset_transform(transform);
        self.wait_for_inactive = false;
    }

    fn reset_after_player_fall(&mut self, transform: &mut Transform) {
        self.reset_transform(transform);
        self.wait_for_inactive = true;
    }

    fn reset_transform(&mut self, transform: &mut Transform) {
        self.collapsed = false;
        self.fall_speed = 0.0;
        transform.translation = self.initial_translation;
    }
}

pub fn spawn_bridge_tile(
    commands: &mut Commands,
    obstructed_set: &mut ObstructedSet,
    layer: &TileLayer,
    i: usize,
    j: usize,
) {
    let grid_location = uvec2(i as u32, j as u32);
    let Some(segment) = bridge_segment(layer, i, j) else {
        warn!("bridge at ({i}, {j}) has no neighboring bridge tile");
        return;
    };
    let blocks_when_collapsed = layer[i][j].ground == GroundTile::Void;

    match segment {
        BridgeSegment::HorizontalMiddle => {
            commands.spawn_scene(bridge_middle(
                grid_location,
                Quat::IDENTITY,
                blocks_when_collapsed,
            ));
            make_traversable(obstructed_set, grid_location);
        }
        BridgeSegment::VerticalMiddle => {
            commands.spawn_scene(bridge_middle(
                grid_location,
                Quat::from_rotation_y(PI / 2.0),
                blocks_when_collapsed,
            ));
            make_traversable(obstructed_set, grid_location);
        }
        BridgeSegment::UpRightCorner => {
            commands.spawn_scene(bridge_corner(
                grid_location,
                Quat::IDENTITY,
                blocks_when_collapsed,
                CornerModel::One,
            ));
            make_traversable(obstructed_set, grid_location);
        }
        BridgeSegment::UpLeftCorner => {
            commands.spawn_scene(bridge_corner(
                grid_location,
                Quat::from_rotation_y(PI),
                blocks_when_collapsed,
                CornerModel::Two,
            ));
            make_traversable(obstructed_set, grid_location);
        }
        BridgeSegment::DownLeftCorner => {
            commands.spawn_scene(bridge_corner(
                grid_location,
                Quat::from_rotation_y(PI),
                blocks_when_collapsed,
                CornerModel::One,
            ));
            make_traversable(obstructed_set, grid_location);
        }
        BridgeSegment::DownRightCorner => {
            commands.spawn_scene(bridge_corner(
                grid_location,
                Quat::IDENTITY,
                blocks_when_collapsed,
                CornerModel::Two,
            ));
            make_traversable(obstructed_set, grid_location);
        }
        BridgeSegment::BottomEnd => {
            commands.spawn_scene(bridge_end(
                grid_location,
                Quat::from_rotation_y(PI / 2.0),
                blocks_when_collapsed,
            ));
        }
        BridgeSegment::TopEnd => {
            commands.spawn_scene(bridge_end(
                grid_location,
                Quat::from_rotation_y(-PI / 2.0),
                blocks_when_collapsed,
            ));
        }
        BridgeSegment::LeftEnd => {
            commands.spawn_scene(bridge_end(
                grid_location,
                Quat::IDENTITY,
                blocks_when_collapsed,
            ));
        }
        BridgeSegment::RightEnd => {
            commands.spawn_scene(bridge_end(
                grid_location,
                Quat::from_rotation_y(PI),
                blocks_when_collapsed,
            ));
        }
    }
}

fn bridge_middle(grid_location: UVec2, rotation: Quat, blocks_when_collapsed: bool) -> impl Scene {
    bridge_scene(
        grid_location,
        rotation,
        Vec3::ZERO,
        blocks_when_collapsed,
        true,
        "models/bridge/bridge_body.gltf",
    )
}

fn bridge_end(grid_location: UVec2, rotation: Quat, blocks_when_collapsed: bool) -> impl Scene {
    bridge_scene(
        grid_location,
        rotation,
        Vec3::X * GRID_SIZE.x,
        blocks_when_collapsed,
        false,
        "models/bridge/just_pillars.glb",
    )
}

#[derive(Clone, Copy)]
enum CornerModel {
    One,
    Two,
}

fn bridge_corner(
    grid_location: UVec2,
    rotation: Quat,
    blocks_when_collapsed: bool,
    model: CornerModel,
) -> impl Scene {
    let asset_path = match model {
        CornerModel::One => "models/bridge/bridge L shape 1 gmtk.gltf",
        CornerModel::Two => "models/bridge/bridge L shape 2 gmtk.gltf",
    };
    bridge_scene(
        grid_location,
        rotation,
        Vec3::ZERO,
        blocks_when_collapsed,
        true,
        asset_path,
    )
}

fn make_traversable(obstructed_set: &mut ObstructedSet, grid_location: UVec2) {
    obstructed_set
        .0
        .remove(&uvec3(grid_location.x, 0, grid_location.y));
}

fn bridge_scene(
    grid_location: UVec2,
    rotation: Quat,
    local_offset: Vec3,
    blocks_when_collapsed: bool,
    traversable_when_intact: bool,
    asset_path: &'static str,
) -> impl Scene {
    let translation = vec3(
        grid_location.x as f32 * GRID_SIZE.x,
        0.0,
        grid_location.y as f32 * GRID_SIZE.y,
    ) + rotation * local_offset;
    bsn! {
        template(move |ctx| {
            Ok(WorldAssetRoot(ctx.resource::<AssetServer>().load(
                GltfAssetLabel::Scene(0).from_asset(asset_path)
            )))
        })
        Transform {
            translation,
            rotation,
        }
        GridLocation(vec3(grid_location.x as f32, 0.0, grid_location.y as f32))
        template_value(Bridge::new(
            translation,
            blocks_when_collapsed,
            traversable_when_intact,
        ))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum BridgeSegment {
    TopEnd,
    LeftEnd,
    VerticalMiddle,
    HorizontalMiddle,
    UpRightCorner,
    UpLeftCorner,
    DownLeftCorner,
    DownRightCorner,
    BottomEnd,
    RightEnd,
}

fn bridge_segment(layer: &TileLayer, i: usize, j: usize) -> Option<BridgeSegment> {
    let up = is_bridge(layer, i as isize, j as isize - 1);
    let down = is_bridge(layer, i as isize, j as isize + 1);
    let left = is_bridge(layer, i as isize - 1, j as isize);
    let right = is_bridge(layer, i as isize + 1, j as isize);

    if up && right && !down && !left {
        Some(BridgeSegment::UpRightCorner)
    } else if up && left && !down && !right {
        Some(BridgeSegment::UpLeftCorner)
    } else if down && left && !up && !right {
        Some(BridgeSegment::DownLeftCorner)
    } else if down && right && !up && !left {
        Some(BridgeSegment::DownRightCorner)
    } else if up && down {
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

pub fn bridge_plugin(app: &mut App) {
    app.add_systems(
        Update,
        begin_collapse
            .in_set(SignalSystems::Read)
            .in_set(SfxSystems::Trigger)
            .after(crate::movement::do_movement),
    )
    .add_systems(Update, animate_fall.after(begin_collapse));
}

fn begin_collapse(
    switches: Res<SwitchStates>,
    world_map: Res<WorldMap>,
    timers: Query<(&GridLocation, &TimerBank), Without<Player>>,
    mut bridges: Query<
        (&mut Bridge, &mut Transform, &GridLocation),
        (With<Bridge>, Without<Player>),
    >,
    player: Single<(Entity, &mut Transform, &mut GridLocation), (With<Player>, Without<Bridge>)>,
    initial_obstructions: Res<InitialObstructedSet>,
    mut obstructed_set: ResMut<ObstructedSet>,
    mut play_sfx: MessageWriter<PlaySfx>,
    mut commands: Commands,
) {
    let snapshot = SignalSnapshot::capture(&switches, &timers);
    let player_location = player.2.0.as_uvec3();
    let mut collapsed_any = false;
    let mut player_fell = false;

    for (mut bridge, mut transform, location) in &mut bridges {
        let position = uvec2(location.0.x as u32, location.0.z as u32);
        let active = activation_at(&world_map, position, &snapshot).unwrap_or(false);

        if bridge.wait_for_inactive {
            if !active {
                bridge.wait_for_inactive = false;
            }
            continue;
        }

        if active && !bridge.collapsed {
            bridge.collapsed = true;
            collapsed_any = true;
            player_fell |= location.0.as_uvec3() == player_location;
            if bridge.blocks_when_collapsed {
                obstructed_set.0.insert(location.0.as_uvec3());
            }
        } else if !active && bridge.collapsed {
            let traversable_when_intact = bridge.traversable_when_intact;
            bridge.reset(&mut transform);
            if traversable_when_intact {
                obstructed_set.0.remove(&location.0.as_uvec3());
            }
        }
    }

    if player_fell {
        let (player_entity, mut player_transform, mut player_location) = player.into_inner();
        let rotation = player_transform.rotation;
        place_player(
            &mut player_transform,
            &mut player_location,
            PLAYER_START,
            rotation,
        );
        commands.entity(player_entity).remove::<Moving>();

        for (mut bridge, mut transform, location) in &mut bridges {
            bridge.reset_after_player_fall(&mut transform);
            let location = location.0.as_uvec3();
            if initial_obstructions.0.contains(&location) {
                obstructed_set.0.insert(location);
            } else {
                obstructed_set.0.remove(&location);
            }
        }
    }

    if collapsed_any {
        play_sfx.write(PlaySfx(Sfx::BridgeCollapse));
    }
}

fn animate_fall(time: Res<Time>, mut bridges: Query<(&mut Bridge, &mut Transform)>) {
    let delta = time.delta_secs();
    for (mut bridge, mut transform) in &mut bridges {
        if !bridge.collapsed || transform.translation.y <= FALL_LIMIT {
            continue;
        }

        bridge.fall_speed += FALL_ACCELERATION * delta;
        transform.translation.y =
            (transform.translation.y - bridge.fall_speed * delta).max(FALL_LIMIT);
    }
}
