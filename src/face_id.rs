use bevy::prelude::*;
use crate::ecs::{CompletedTurn, FaceId, GridLocation, Orientation, Player, PressurePlate, SignalSystems};
use crate::GRID_SIZE;
use crate::map_json::SwitchMode;
use crate::map_loader::WorldMap;
use crate::signal_logic::{activation_at, ReplaceTimersRequest, SignalSnapshot, SwitchStates, TimerBank};

pub fn face_id_plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_switches
            .in_set(SignalSystems::Write)
            .after(crate::movement::do_movement),
    );
}

fn update_switches(
    player: Single<(&GridLocation, &Transform), With<Player>>,
    mut ray_cast: MeshRayCast,
    face_ids: Query<(&GridLocation, &FaceId), Without<Player>>,
    timers: Query<(&GridLocation, &TimerBank)>,
    world_map: Res<WorldMap>,
    mut switches: ResMut<SwitchStates>,
    mut completed_turns: MessageReader<CompletedTurn>,
    mut replace_timers: MessageWriter<ReplaceTimersRequest>,
    mut gizmos: Gizmos,
) {
    let player_position = uvec2(player.0.0.x as u32, player.0.0.z as u32);

    let snapshot = SignalSnapshot::capture(&switches, &timers);

    gizmos.arrow(
        player.0.0 * GRID_SIZE.x + Vec3::Y*0.5 - player.1.rotation * Vec3::Z*0.51,
        player.0.0 * GRID_SIZE.x + Vec3::Y*0.5 - player.1.rotation * Vec3::Z*0.51 + player.1.rotation * -Vec3::Z,
        Color::srgb(1.0, 0.0, 0.0),
    );
    let does_detect_player = ray_cast.cast_ray(
        Ray3d::new(player.0.0 * GRID_SIZE.x + Vec3::Y*0.5 - player.1.rotation * Vec3::Z*0.51, Dir3::new(player.1.rotation * -Vec3::Z).unwrap()), // unwrap never fails because to_vec_direction() only outputs non-zero vectors
        &MeshRayCastSettings {
            visibility: RayCastVisibility::Any,
            filter: &|_| true,
            early_exit_test: &|_| true, // function that always returns true
        },
    ).first();
    info!("face id triggered {:?}", does_detect_player);
    let does_detect_player = does_detect_player.map(|(entity, _)| face_ids.contains(*entity)).unwrap_or(false);
    info!("face id triggered {:?}", does_detect_player);
    // is there a face id here, and is there no condition
    let can_be_activated = does_detect_player && activation_at(&world_map, player_position, &snapshot).unwrap_or(true);
    let touched_switches = if can_be_activated {
        world_map.touched_switches.get(&player_position).map(Vec::as_slice).unwrap_or_default()
    } else {
        Default::default()
    };

    for completed_turn in completed_turns.read() {
        if can_be_activated {
            let position_3d = uvec3(player_position.x, 0, player_position.y);
            if completed_turn.old_location == position_3d {
                if let Some(effects) = world_map.input_effects.get(&player_position) {
                    for effect in effects {
                        replace_timers.write(ReplaceTimersRequest(effect.clone()));
                    }
                    // we only need to write the ReplaceTimersRequest once, so break out of the loop now
                    break;
                }
            }
        }
    }

    for (id, switch) in &mut switches.0 {
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