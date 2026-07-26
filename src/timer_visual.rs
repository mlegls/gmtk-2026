use std::f32::consts::{FRAC_PI_2, PI};

use bevy::asset::RenderAssetUsages;
use bevy::math::Affine2;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use fontdue::layout::{
    CoordinateSystem, HorizontalAlign, Layout, LayoutSettings, TextStyle, VerticalAlign,
};

use crate::PLAYER_SIZE;
use crate::ecs::SignalSystems;
use crate::signal_logic::{TimerBank, TimerVisualKind};

const SLOT_SPACING: f32 = 1.0;
const BLOCK_SIZE: f32 = 0.72;
const FACE_OFFSET: f32 = BLOCK_SIZE / 2.0 + 0.003;
const ATLAS_COLUMNS: u32 = 32;
const ATLAS_CELL_SIZE: u32 = 64;
const MAX_CACHED_NUMBER: u32 = 1000;

#[derive(Component)]
struct TimerVisualized;

#[derive(Component)]
struct TimerDisplay {
    bank: Entity,
    slot: usize,
    block_material: Handle<StandardMaterial>,
    face_material: Handle<StandardMaterial>,
    displayed_number: Option<u32>,
    displayed_kind: Option<TimerVisualKind>,
}

pub fn timer_visual_plugin(app: &mut App) {
    app.add_systems(
        Update,
        (spawn_timer_displays, update_timer_displays)
            .chain()
            .in_set(SignalSystems::Read),
    );
}

fn spawn_timer_displays(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    banks: Query<(Entity, &TimerBank), Without<TimerVisualized>>,
    mut atlas: Local<Option<Handle<Image>>>,
) {
    let atlas = atlas
        .get_or_insert_with(|| images.add(build_number_atlas()))
        .clone();
    let block_mesh = meshes.add(Cuboid::from_size(Vec3::splat(BLOCK_SIZE)));
    let face_mesh = meshes.add(Rectangle::new(BLOCK_SIZE, BLOCK_SIZE));

    for (bank_entity, bank) in &banks {
        commands.entity(bank_entity).insert(TimerVisualized);

        for slot in 0..bank.slots.len() {
            let block_material = materials.add(timer_block_material(TimerVisualKind::Periodic));
            let face_material = materials.add(timer_face_material(atlas.clone(), 0));
            let block = commands
                .spawn((
                    Mesh3d(block_mesh.clone()),
                    MeshMaterial3d(block_material.clone()),
                    Transform::IDENTITY,
                ))
                .id();
            let faces = [
                spawn_face(
                    &mut commands,
                    &face_mesh,
                    &face_material,
                    Vec3::new(0.0, 0.0, FACE_OFFSET),
                    Quat::IDENTITY,
                ),
                spawn_face(
                    &mut commands,
                    &face_mesh,
                    &face_material,
                    Vec3::new(0.0, 0.0, -FACE_OFFSET),
                    Quat::from_rotation_y(PI),
                ),
                spawn_face(
                    &mut commands,
                    &face_mesh,
                    &face_material,
                    Vec3::new(FACE_OFFSET, 0.0, 0.0),
                    Quat::from_rotation_y(FRAC_PI_2),
                ),
                spawn_face(
                    &mut commands,
                    &face_mesh,
                    &face_material,
                    Vec3::new(-FACE_OFFSET, 0.0, 0.0),
                    Quat::from_rotation_y(-FRAC_PI_2),
                ),
            ];
            let height = PLAYER_SIZE.y + 0.5 + slot as f32 * SLOT_SPACING;
            let display = commands
                .spawn((
                    Transform::from_xyz(0.0, height, 0.0),
                    Visibility::Hidden,
                    TimerDisplay {
                        bank: bank_entity,
                        slot,
                        block_material,
                        face_material,
                        displayed_number: None,
                        displayed_kind: None,
                    },
                ))
                .add_children(&[block, faces[0], faces[1], faces[2], faces[3]])
                .id();
            commands.entity(bank_entity).add_child(display);
        }
    }
}

fn spawn_face(
    commands: &mut Commands,
    mesh: &Handle<Mesh>,
    material: &Handle<StandardMaterial>,
    translation: Vec3,
    rotation: Quat,
) -> Entity {
    commands
        .spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform::from_translation(translation).with_rotation(rotation),
        ))
        .id()
}

fn update_timer_displays(
    banks: Query<&TimerBank>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut displays: Query<(&mut TimerDisplay, &mut Visibility)>,
) {
    for (mut display, mut visibility) in &mut displays {
        let state = banks
            .get(display.bank)
            .ok()
            .and_then(|bank| bank.slots[display.slot].as_ref())
            .and_then(|timer| timer.visual_state());
        let Some((number, _proportion, kind)) = state else {
            *visibility = Visibility::Hidden;
            display.displayed_number = None;
            continue;
        };
        *visibility = Visibility::Inherited;

        let number = number.min(MAX_CACHED_NUMBER);
        if display.displayed_number != Some(number) {
            display.displayed_number = Some(number);
            if let Some(mut material) = materials.get_mut(&display.face_material) {
                material.uv_transform = atlas_uv_transform(number);
            }
        }
        if display.displayed_kind != Some(kind) {
            display.displayed_kind = Some(kind);
            if let Some(mut material) = materials.get_mut(&display.block_material) {
                *material = timer_block_material(kind);
            }
        }
    }
}

fn build_number_atlas() -> Image {
    let rows = (MAX_CACHED_NUMBER + 1).div_ceil(ATLAS_COLUMNS);
    let width = ATLAS_COLUMNS * ATLAS_CELL_SIZE;
    let height = rows * ATLAS_CELL_SIZE;
    let mut pixels = vec![0; (width * height * 4) as usize];
    let font = fontdue::Font::from_bytes(
        bevy::text::DEFAULT_FONT_DATA,
        fontdue::FontSettings::default(),
    )
    .expect("Bevy's default font must be valid");
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);

    for number in 0..=MAX_CACHED_NUMBER {
        layout.reset(&LayoutSettings {
            max_width: Some(ATLAS_CELL_SIZE as f32),
            max_height: Some(ATLAS_CELL_SIZE as f32),
            horizontal_align: HorizontalAlign::Center,
            vertical_align: VerticalAlign::Middle,
            ..default()
        });
        let text = number.to_string();
        let font_size = match text.len() {
            1 => 42.0,
            2 => 34.0,
            3 => 27.0,
            _ => 21.0,
        };
        layout.append(&[&font], &TextStyle::new(&text, font_size, 0));
        let cell_x = (number % ATLAS_COLUMNS) * ATLAS_CELL_SIZE;
        let cell_y = (number / ATLAS_COLUMNS) * ATLAS_CELL_SIZE;

        for glyph in layout.glyphs() {
            let (_, coverage) = font.rasterize_config(glyph.key);
            let glyph_x = cell_x as i32 + glyph.x.round() as i32;
            let glyph_y = cell_y as i32 + glyph.y.round() as i32;
            for y in 0..glyph.height {
                for x in 0..glyph.width {
                    let atlas_x = glyph_x + x as i32;
                    let atlas_y = glyph_y + y as i32;
                    if atlas_x < 0
                        || atlas_y < 0
                        || atlas_x >= width as i32
                        || atlas_y >= height as i32
                    {
                        continue;
                    }
                    let target = ((atlas_y as u32 * width + atlas_x as u32) * 4) as usize;
                    pixels[target] = 255;
                    pixels[target + 1] = 255;
                    pixels[target + 2] = 255;
                    pixels[target + 3] = coverage[y * glyph.width + x];
                }
            }
        }
    }

    Image::new(
        Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        pixels,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

fn atlas_uv_transform(number: u32) -> Affine2 {
    let rows = (MAX_CACHED_NUMBER + 1).div_ceil(ATLAS_COLUMNS);
    let scale = Vec2::new(1.0 / ATLAS_COLUMNS as f32, 1.0 / rows as f32);
    let translation = Vec2::new(
        (number % ATLAS_COLUMNS) as f32 * scale.x,
        (number / ATLAS_COLUMNS) as f32 * scale.y,
    );
    Affine2::from_scale_angle_translation(scale, 0.0, translation)
}

fn timer_block_material(kind: TimerVisualKind) -> StandardMaterial {
    let base_color = match kind {
        TimerVisualKind::Periodic => Color::srgba(0.5, 0.2, 0., 0.55),
        TimerVisualKind::PeriodicPulse | TimerVisualKind::OneShotPulse => {
            Color::srgba(0.7, 0.02, 0.02, 0.55)
        }
        TimerVisualKind::AfterCountdown => Color::srgba(0., 0.5, 0.2, 0.55),
        TimerVisualKind::DuringCountdown => Color::srgba(0.0, 0.2, 0.5, 0.55),
        TimerVisualKind::OneShot => Color::srgba(0.2, 0.2, 0., 0.55),
    };
    StandardMaterial {
        base_color,
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        ..default()
    }
}

fn timer_face_material(atlas: Handle<Image>, number: u32) -> StandardMaterial {
    StandardMaterial {
        base_color: Color::WHITE,
        base_color_texture: Some(atlas),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        uv_transform: atlas_uv_transform(number),
        ..default()
    }
}
