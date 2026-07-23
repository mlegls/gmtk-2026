use crate::ecs::{Arrow, AvailableActions, CameraRig, Direction, GridLocation, ObstructedSet, Orientation, Player};
use crate::ui::ui;
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use std::f32::consts::PI;
use crate::{GRID_SIZE, GROUND_LEVEL};

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
) {
    for (i, row) in GROUND_LEVEL.into_iter().enumerate() {
        for (j, location) in row.into_iter().enumerate() {
            if location == 0 {
                obstructed_set.0.insert(uvec3(i as u32 + 1, 0, j as u32 + 1));
            }
            if location == 1 {
                commands.spawn_scene(bsn! {
                    Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                    MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 255)))
                    Transform::from_xyz((i as f32 + 1.0) * GRID_SIZE.x, -5.0, (j as f32 + 1.0) * GRID_SIZE.y)
                });
            }
            /*bsn! {
                Mesh3d(asset_value(Cuboid::new(1.0, 10.0, 1.0)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 255)))
                Transform::from_xyz(0.0, -5.0, 0.0)
            }*/
        }
    }
}
