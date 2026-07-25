use bevy::prelude::*;

use crate::ecs::{CompletedTurn, FaceId, GridLocation, Player, SignalSystems};
use crate::map_json::SwitchMode;
use crate::map_loader::WorldMap;
use crate::sfx::{PlaySfx, Sfx, SfxSystems};
use crate::signal_logic::{
    ReplaceTimersRequest, SignalSnapshot, SwitchStates, TimerBank, activation_at,
};
use crate::story::ResetGame;

const INDICATOR_SIZE: f32 = 0.35;
const INDICATOR_HEIGHT: f32 = 1.25;
const INDICATOR_SPEED: f32 = 6.0;

#[derive(Resource, Default)]
struct ActiveFaceId(Option<UVec2>);

#[derive(Component)]
struct FaceIdIndicator {
    position: UVec2,
    progress: f32,
}

pub fn face_id_plugin(app: &mut App) {
    app.init_resource::<ActiveFaceId>()
        .add_systems(Update, (setup_indicators, reset_indicator))
        .add_systems(
            Update,
            update_switches
                .in_set(SignalSystems::Write)
                .in_set(SfxSystems::Trigger)
                .after(crate::pressure_plate::update_switches)
                .after(crate::movement::do_movement),
        )
        .add_systems(Update, animate_indicators.after(update_switches));
}

fn update_switches(
    player: Single<(&GridLocation, &Transform), With<Player>>,
    mut ray_cast: MeshRayCast,
    face_ids: Query<(&GridLocation, &FaceId), Without<Player>>,
    parents: Query<&ChildOf>,
    timers: Query<(&GridLocation, &TimerBank)>,
    world_map: Res<WorldMap>,
    mut switches: ResMut<SwitchStates>,
    mut completed_turns: MessageReader<CompletedTurn>,
    mut replace_timers: MessageWriter<ReplaceTimersRequest>,
    mut active_face_id: ResMut<ActiveFaceId>,
    mut play_sfx: MessageWriter<PlaySfx>,
    mut gizmos: Gizmos,
) {
    let completed_turns: Vec<_> = completed_turns.read().cloned().collect();
    if completed_turns.is_empty() {
        return;
    }

    let player_position = uvec2(player.0.0.x as u32, player.0.0.z as u32);
    let direction = player.1.rotation * -Vec3::Z;
    let origin = player.1.translation + direction * 0.51;

    gizmos.arrow(origin, origin + direction, Color::srgb(1.0, 0.0, 0.0));

    // search up ancestors (may hit child mesh)
    let hit_entity = ray_cast
        .cast_ray(
            Ray3d::new(
                origin,
                Dir3::new(direction).expect("facing direction is non-zero"),
            ),
            &MeshRayCastSettings {
                visibility: RayCastVisibility::Any,
                filter: &|_| true,
                early_exit_test: &|_| true,
            },
        )
        .first()
        .map(|(entity, _)| *entity);
    let detected_position = hit_entity.and_then(|entity| {
        find_face_id_ancestor(entity, &parents, &face_ids)
            .map(|location| uvec2(location.0.x as u32, location.0.z as u32))
    });

    let snapshot = SignalSnapshot::capture(&switches, &timers);
    let active_trigger = detected_position
        .filter(|position| activation_at(&world_map, *position, &snapshot).unwrap_or(true));
    if let Some(position) = active_trigger
        && active_trigger != active_face_id.0
    {
        play_sfx.write(PlaySfx::at(
            Sfx::Success,
            &GridLocation(vec3(position.x as f32, 0.0, position.y as f32)),
        ));
    }
    active_face_id.0 = active_trigger;

    let touched_switches = active_trigger
        .and_then(|position| world_map.touched_switches.get(&position))
        .map(Vec::as_slice)
        .unwrap_or_default();

    if let Some(position) = active_trigger {
        let stayed_in_place = completed_turns
            .iter()
            .any(|turn| turn.old_location == uvec3(player_position.x, 0, player_position.y));
        if stayed_in_place {
            if let Some(effects) = world_map.input_effects.get(&position) {
                for effect in effects {
                    replace_timers.write(ReplaceTimersRequest(effect.clone()));
                }
            }
        }
    }

    // don't clear unrelated pressure plates
    for (id, switch) in &mut switches.0 {
        let belongs_to_face_id = face_ids.iter().any(|(location, _)| {
            let position = uvec2(location.0.x as u32, location.0.z as u32);
            world_map
                .touched_switches
                .get(&position)
                .is_some_and(|ids| ids.contains(id))
        });
        if !belongs_to_face_id {
            continue;
        }

        let is_touched = touched_switches.iter().any(|touched| touched == id);
        match switch.mode {
            SwitchMode::Hold => switch.active = is_touched,
            SwitchMode::Toggle if is_touched && !switch.touched_last_turn => {
                switch.active = !switch.active;
            }
            SwitchMode::Toggle => {}
        }
        switch.touched_last_turn = is_touched;
    }
}

fn reset_indicator(mut resets: MessageReader<ResetGame>, mut active_face_id: ResMut<ActiveFaceId>) {
    if resets.read().next().is_some() {
        active_face_id.0 = None;
    }
}

fn setup_indicators(
    mut commands: Commands,
    face_ids: Query<&GridLocation, Added<FaceId>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    if face_ids.is_empty() {
        return;
    }

    let mesh = meshes.add(Cuboid::from_size(Vec3::splat(INDICATOR_SIZE)));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.05, 0.9, 0.12),
        emissive: LinearRgba::rgb(0.02, 0.5, 0.04),
        ..default()
    });

    for location in &face_ids {
        let position = uvec2(location.0.x as u32, location.0.z as u32);
        commands.spawn((
            Mesh3d(mesh.clone()),
            MeshMaterial3d(material.clone()),
            Transform {
                translation: vec3(location.0.x, INDICATOR_HEIGHT, location.0.z),
                scale: Vec3::ZERO,
                ..default()
            },
            FaceIdIndicator {
                position,
                progress: 0.0,
            },
        ));
    }
}

fn animate_indicators(
    time: Res<Time>,
    active_face_id: Res<ActiveFaceId>,
    mut indicators: Query<(&mut FaceIdIndicator, &mut Transform)>,
) {
    for (mut indicator, mut transform) in &mut indicators {
        let target = if active_face_id.0 == Some(indicator.position) {
            1.0
        } else {
            0.0
        };
        let step = INDICATOR_SPEED * time.delta_secs();
        if indicator.progress < target {
            indicator.progress = (indicator.progress + step).min(target);
        } else {
            indicator.progress = (indicator.progress - step).max(target);
        }

        let eased = indicator.progress * indicator.progress * (3.0 - 2.0 * indicator.progress);
        transform.scale = Vec3::splat(eased);
        transform.translation.y = INDICATOR_HEIGHT + 0.25 * eased;
        transform.rotation = Quat::from_rotation_y(time.elapsed_secs() * 2.0 * eased);
    }
}

fn find_face_id_ancestor<'a>(
    mut entity: Entity,
    parents: &Query<&ChildOf>,
    face_ids: &'a Query<(&GridLocation, &FaceId), Without<Player>>,
) -> Option<&'a GridLocation> {
    loop {
        if let Ok((location, _)) = face_ids.get(entity) {
            return Some(location);
        }
        entity = parents.get(entity).ok()?.parent();
    }
}
