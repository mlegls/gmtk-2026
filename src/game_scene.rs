use std::f32::consts::PI;
use bevy::camera::ScalingMode;
use bevy::prelude::*;
use crate::ecs::{Arrow, Direction, GridLocation, Orientation, Player};
use crate::PLAYER_SIZE;

pub fn game_scene_plugin(app: &mut App) {
    app.add_systems(Startup, scene.spawn());
}

fn scene() -> impl SceneList {
    bsn_list![
        isometric_camera(), point_light(),
        (
            cube()
            Player
            Orientation(Direction::North)
        ),
        arrow(),
        ground()
    ]
}

fn isometric_camera() -> impl Scene {
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: 12.0,
        },
        ..OrthographicProjection::default_3d()
    });
    let rotation = Quat::from_euler(EulerRot::YXZ, PI/4.0, -PI/6.0, 0.0);
    bsn! {
        Camera3d
        template_value(projection)
        Transform {
            rotation,
            translation: vec3(40.0, 32.66, 40.0)
        }
    }
}

fn point_light() -> impl Scene {
    bsn! {
        PointLight {
            shadow_maps_enabled: true,
        }
        Transform::from_xyz(4.0, 8.0, 4.0)
    }
}

fn cube() -> impl Scene {
    bsn! {
        Mesh3d(asset_value(Cuboid::from_size(PLAYER_SIZE)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        Transform::from_xyz(0.0, 0.5, 0.0)
        GridLocation(Vec3::new(0.0, 0.0, 0.0))
        Children [
            (
                Mesh3d(asset_value(Cuboid::new(0.2, 0.2, 0.2)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 0, 0)))
                Transform::from_xyz(0.0, PLAYER_SIZE.y/2.0, 0.0)
            ),
            (
                Mesh3d(asset_value(Cuboid::new(0.2, 0.2, 0.2)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(0, 255, 0)))
                Transform::from_xyz(PLAYER_SIZE.x/2.0, 0.0, 0.0)
            ),
            (
                Mesh3d(asset_value(Cuboid::new(0.2, 0.2, 0.2)))
                MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 255)))
                Transform::from_xyz(0.0, 0.0, PLAYER_SIZE.z/2.0)
            )
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

fn ground() -> impl Scene {
    bsn! {
        Mesh3d(asset_value(Cuboid::new(10.0, 0.1, 10.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 255)))
        Transform::from_xyz(0.0, -0.05, 0.0)
    }
}