use crate::ecs::{CompletedTurn, ConveyorBelt, GridLocation, Orientation, Player, SignalSystems};
use crate::map_loader::WorldMap;
use crate::movement::translate_player;
use crate::sfx::{PlaySfx, Sfx, SfxSystems};
use crate::signal_logic::{SignalSnapshot, SwitchStates, TimerBank, activation_at};
use bevy::prelude::*;
use std::f32::consts::PI;

pub fn conveyor_belt_plugin(app: &mut App) {
    app.add_systems(Update, update_direction.in_set(SignalSystems::Read))
        .add_systems(
            Update,
            conveyor_belt_move
                .after(update_direction)
                .in_set(SfxSystems::Trigger),
        );
}

fn update_direction(
    switches: Res<SwitchStates>,
    world_map: Res<WorldMap>,
    timers: Query<(&GridLocation, &TimerBank)>,
    mut conveyor_belts: Query<(
        &ConveyorBelt,
        &GridLocation,
        &mut Orientation,
        &mut Transform,
    )>,
) {
    let snapshot = SignalSnapshot::capture(&switches, &timers);
    for (belt, location, mut orientation, mut transform) in &mut conveyor_belts {
        let position = uvec2(location.0.x as u32, location.0.z as u32);
        let reversed = activation_at(&world_map, position, &snapshot).unwrap_or(false);

        if reversed {
            orientation.0 = belt.initial_orientation.turn_left().turn_left();
            transform.rotation = Quat::from_rotation_y(PI) * belt.initial_rotation;
        } else {
            orientation.0 = belt.initial_orientation;
            transform.rotation = belt.initial_rotation;
        }
    }
}

pub fn conveyor_belt_move(
    conveyor_belts: Query<(&GridLocation, &Orientation), With<ConveyorBelt>>,
    player: Single<(&mut Transform, &mut GridLocation), (With<Player>, Without<ConveyorBelt>)>,
    mut completed_turns: MessageReader<CompletedTurn>,
    mut play_sfx: MessageWriter<PlaySfx>,
) {
    let (mut player_transform, mut player_location) = player.into_inner();

    for _ in completed_turns.read() {
        for (belt_location, belt_orientation) in conveyor_belts.iter() {
            if belt_location.0 == player_location.0 {
                // player is on belt, time to move them
                translate_player(
                    &mut player_transform,
                    &mut player_location,
                    belt_orientation.0.to_grid_location_offset(),
                    1.0,
                );
                play_sfx.write(PlaySfx::at(Sfx::Conveyor, belt_location));
                break;
            }
        }
    }
}
