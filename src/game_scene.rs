use std::f32::consts::PI;
use bevy::camera::ScalingMode;
use bevy::prelude::*;

pub fn game_scene_plugin(app: &mut App) {
    app.add_systems(Startup, scene.spawn());
}

fn scene() -> impl SceneList {
    bsn_list![isometric_camera(), point_light(), cube(), ground()]
}

fn isometric_camera() -> impl Scene {
    let projection = Projection::Orthographic(OrthographicProjection {
        scaling_mode: ScalingMode::FixedVertical {
            viewport_height: 12.0,
        },
        ..OrthographicProjection::default_3d()
    });
    let rotation = Quat::from_euler(EulerRot::YXZ, PI/4.0, -PI/6.0, 0.0);
    //let transform = Transform::from_xyz(10.0, 10.0, -10.0).looking_at(Vec3::ZERO, Vec3::Y);
    bsn! {
        Camera3d
        template_value(projection)
        //template_value(transform)
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
        Mesh3d(asset_value(Cuboid::new(1.0, 1.0, 1.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(124, 144, 255)))
        Transform::from_xyz(0.0, 0.5, 0.0)
    }
}

fn ground() -> impl Scene {
    bsn! {
        Mesh3d(asset_value(Cuboid::new(10.0, 0.1, 10.0)))
        MeshMaterial3d::<StandardMaterial>(asset_value(Color::srgb_u8(255, 255, 255)))
        Transform::from_xyz(0.0, -0.05, 0.0)
    }
}