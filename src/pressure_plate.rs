use bevy::prelude::*;

use crate::conveyor_belt::conveyor_belt_move;
use crate::ecs::{CompletedTurn, GridLocation, Player, PressurePlate, SignalSystems};
use crate::map_json::SwitchMode;
use crate::map_loader::WorldMap;
use crate::sfx::{PlaySfx, Sfx, SfxSystems};
use crate::signal_logic::{
    ReplaceTimersRequest, SignalSnapshot, SwitchStates, TimerBank, activation_at,
};

pub fn pressure_plate_plugin(app: &mut App) {
    app.add_systems(
        Update,
        update_switches
            .in_set(SignalSystems::Write)
            .after(crate::movement::do_movement),
    )
    .add_systems(
        Update,
        pressure_plate_sfx
            .in_set(SfxSystems::Trigger)
            .after(conveyor_belt_move),
    );
}

fn pressure_plate_sfx(
    player: Single<&GridLocation, With<Player>>,
    pressure_plates: Query<&GridLocation, (With<PressurePlate>, Without<Player>)>,
    mut completed_turns: MessageReader<CompletedTurn>,
    mut play_sfx: MessageWriter<PlaySfx>,
) {
    let player_location = player.0.as_uvec3();
    for completed_turn in completed_turns.read() {
        if completed_turn.old_location != player_location
            && pressure_plates
                .iter()
                .any(|plate_location| plate_location.0.as_uvec3() == player_location)
        {
            play_sfx.write(PlaySfx(Sfx::PressurePlate));
        }
    }
}

// check if the block the player is on has "activations" (if the json config makes it activate switches)
//
// also check if it has a physical pressure plate.
//
// if it activates a switch, do the switch's thing (toggle or hold). if it has a physical pressure plate, also do the sound/animation
pub(crate) fn update_switches(
    player: Single<&GridLocation, With<Player>>,
    pressure_plates: Query<&GridLocation, (With<PressurePlate>, Without<Player>)>,
    timers: Query<(&GridLocation, &TimerBank)>,
    world_map: Res<WorldMap>,
    mut switches: ResMut<SwitchStates>,
    mut completed_turns: MessageReader<CompletedTurn>,
    mut replace_timers: MessageWriter<ReplaceTimersRequest>,
) {
    let old_locations: Vec<UVec3> = completed_turns
        .read()
        .map(|turn| turn.old_location)
        .collect();
    if old_locations.is_empty() {
        return;
    }

    let snapshot = SignalSnapshot::capture(&switches, &timers);
    let player_position = uvec2(player.0.x as u32, player.0.z as u32);
    let has_physical_plate = pressure_plates
        .iter()
        .any(|location| uvec2(location.0.x as u32, location.0.z as u32) == player_position);
    let has_invisible_activation = world_map.touched_switches.contains_key(&player_position)
        || world_map.input_effects.contains_key(&player_position);
    let active_trigger = (has_physical_plate || has_invisible_activation)
        .then_some(player_position)
        .filter(|position| activation_at(&world_map, *position, &snapshot).unwrap_or(true));

    let touched_switches = active_trigger
        .and_then(|position| world_map.touched_switches.get(&position))
        .map(Vec::as_slice)
        .unwrap_or_default();

    if let Some(position) = active_trigger {
        let position_3d = uvec3(position.x, 0, position.y);
        if old_locations.iter().any(|old| *old != position_3d) {
            if let Some(effects) = world_map.input_effects.get(&position) {
                for effect in effects {
                    replace_timers.write(ReplaceTimersRequest(effect.clone()));
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
