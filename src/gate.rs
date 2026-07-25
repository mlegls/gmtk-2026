use bevy::prelude::*;

use crate::ecs::{Gate, GateEntrySet, GridLocation, ObstructedSet, SignalSystems};
use crate::map_loader::WorldMap;
use crate::sfx::{PlaySfx, Sfx, SfxSystems};
use crate::signal_logic::{SignalSnapshot, SwitchStates, TimerBank, activation_at};

pub fn gate_plugin(app: &mut App) {
    app.init_resource::<GateEntrySet>().add_systems(
        Update,
        signal_check
            .in_set(SignalSystems::Read)
            .in_set(SfxSystems::Trigger)
            .before(crate::movement::input),
    );
}

fn signal_check(
    switches: Res<SwitchStates>,
    world_map: Res<WorldMap>,
    timers: Query<(&GridLocation, &TimerBank)>,
    mut gate_query: Query<(&mut Gate, &mut Transform, &GridLocation)>,
    mut obstructed_set: ResMut<ObstructedSet>,
    mut gate_entries: ResMut<GateEntrySet>,
    mut play_sfx: MessageWriter<PlaySfx>,
) {
    let snapshot = SignalSnapshot::capture(&switches, &timers);
    let next_snapshot = SignalSnapshot::capture_next_turn(&switches, &timers);
    for (mut gate, mut transform, location) in &mut gate_query {
        let position = uvec2(location.0.x as u32, location.0.z as u32);
        let grid_location = location.0.as_uvec3();
        let active = activation_at(&world_map, position, &snapshot).unwrap_or(false);
        let active_next_turn = activation_at(&world_map, position, &next_snapshot).unwrap_or(false);

        gate_entries.locations.insert(grid_location);
        if active_next_turn {
            gate_entries.open_next_turn.insert(grid_location);
        } else {
            gate_entries.open_next_turn.remove(&grid_location);
        }
        let changed = matches!(
            (&*gate, active),
            (Gate::Closed, true) | (Gate::Opened, false)
        );
        if active {
            *gate = Gate::Opened;
            obstructed_set.0.remove(&grid_location);
            transform.translation.y = -5.0;
        } else {
            *gate = Gate::Closed;
            obstructed_set.0.insert(grid_location);
            transform.translation.y = 0.0;
        }
        if changed {
            play_sfx.write(PlaySfx::at(Sfx::Gate, location));
        }
    }
}
