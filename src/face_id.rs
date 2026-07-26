use bevy::prelude::*;

use crate::ecs::{CompletedTurn, FaceId, GridLocation, Orientation, Player, SignalSystems};
use crate::map_json::SwitchMode;
use crate::map_loader::WorldMap;
use crate::signal_logic::{
    ReplaceTimersRequest, SignalSnapshot, SwitchStates, TimerBank, activation_at,
};

pub fn face_id_plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_switches
            .in_set(SignalSystems::Write)
            .after(crate::pressure_plate::update_switches)
            .after(crate::movement::do_movement),
    );
}

fn update_switches(
    player: Single<(&GridLocation, &Orientation), With<Player>>,
    mut ray_cast: MeshRayCast,
    face_ids: Query<(&GridLocation, &FaceId), Without<Player>>,
    parents: Query<&ChildOf>,
    timers: Query<(&GridLocation, &TimerBank)>,
    world_map: Res<WorldMap>,
    mut switches: ResMut<SwitchStates>,
    mut completed_turns: MessageReader<CompletedTurn>,
    mut replace_timers: MessageWriter<ReplaceTimersRequest>,
    mut gizmos: Gizmos,
) {
    let completed_turns: Vec<_> = completed_turns.read().cloned().collect();
    if completed_turns.is_empty() {
        return;
    }

    let player_position = uvec2(player.0.0.x as u32, player.0.0.z as u32);
    let direction = player.1.0.to_rotation() * -Vec3::Z;
    let origin = player.0.to_world_space() + Vec3::Y * 0.5 + direction * 0.51;

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
