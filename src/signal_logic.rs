use std::collections::HashMap;

use bevy::ecs::query::QueryFilter;
use bevy::prelude::*;

use crate::ecs::{CompletedTurn, GridLocation, SignalSystems};
use crate::map_json::{
    InitialTimerSlots, MapJson, ReplaceTimers, SignalExpression, SwitchMode, TIMER_SLOTS,
    TimerReplacement, TimerTemplate,
};
use crate::map_loader::WorldMap;

pub fn signal_logic_plugin(app: &mut App) {
    app.insert_resource(SwitchStates::from_map(app.world().resource::<MapJson>()))
        .add_message::<ReplaceTimersRequest>()
        .add_systems(
            Update,
            // tick then replace so replacements don't tick
            (tick_timers, apply_timer_replacements)
                .chain()
                .in_set(SignalSystems::Timer),
        );
}

#[derive(Clone, Debug)]
pub struct SwitchState {
    pub mode: SwitchMode,
    pub active: bool,
    pub touched_last_turn: bool,
    initial: bool,
}

#[derive(Resource, Clone, Debug, Default)]
pub struct SwitchStates(pub HashMap<String, SwitchState>);

impl SwitchStates {
    fn from_map(map: &MapJson) -> Self {
        Self(
            map.switches
                .iter()
                .map(|(id, template)| {
                    (
                        id.clone(),
                        SwitchState {
                            mode: template.mode,
                            active: template.initial,
                            touched_last_turn: false,
                            initial: template.initial,
                        },
                    )
                })
                .collect(),
        )
    }

    pub fn reset(&mut self) {
        for switch in self.0.values_mut() {
            switch.active = switch.initial;
            switch.touched_last_turn = false;
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TimerVisualKind {
    Periodic,
    PeriodicPulse,
    AfterCountdown,
    DuringCountdown,
    OneShot,
    OneShotPulse,
}

#[derive(Clone, Debug)]
pub struct TimerInstance {
    template: TimerTemplate,
    runtime: TimerRuntime,
}

#[derive(Clone, Debug)]
enum TimerRuntime {
    Periodic {
        running: bool,
        turn_in_cycle: u32,
        output: bool,
    },
    AfterCountdown {
        running: bool,
        remaining: u32,
        output: bool,
    },
    DuringCountdown {
        running: bool,
        remaining: u32,
        output: bool,
    },
    OneShot {
        running: bool,
        remaining: Option<u32>,
        pulse_remaining: u32,
        output: bool,
    },
}

impl TimerInstance {
    pub fn new(template: TimerTemplate) -> Self {
        let runtime = Self::initial_runtime(&template);
        Self { template, runtime }
    }

    fn initial_runtime(template: &TimerTemplate) -> TimerRuntime {
        match *template {
            TimerTemplate::Periodic { initial, .. } => TimerRuntime::Periodic {
                running: initial,
                turn_in_cycle: 0,
                output: false,
            },
            TimerTemplate::ActiveAfterCountdown { turns, initial } => {
                TimerRuntime::AfterCountdown {
                    running: initial,
                    remaining: turns,
                    output: false,
                }
            }
            TimerTemplate::ActiveDuringCountdown { turns, initial } => {
                TimerRuntime::DuringCountdown {
                    running: initial,
                    remaining: turns,
                    output: initial,
                }
            }
            TimerTemplate::OneShot {
                turns,
                pulse_turns,
                initial,
            } => TimerRuntime::OneShot {
                running: initial,
                remaining: Some(turns),
                pulse_remaining: pulse_turns,
                output: false,
            },
        }
    }

    pub fn output(&self) -> bool {
        match self.runtime {
            TimerRuntime::Periodic { output, .. }
            | TimerRuntime::AfterCountdown { output, .. }
            | TimerRuntime::DuringCountdown { output, .. }
            | TimerRuntime::OneShot { output, .. } => output,
        }
    }

    /// rendering-relevant data: number, remaining ratio, and timer type
    pub fn visual_state(&self) -> Option<(u32, f32, TimerVisualKind)> {
        match (&self.template, &self.runtime) {
            (
                TimerTemplate::Periodic {
                    period,
                    pulse_turns,
                    ..
                },
                TimerRuntime::Periodic {
                    running,
                    turn_in_cycle,
                    ..
                },
            ) if *running => {
                let pulse_turns = (*pulse_turns).min(*period);
                if pulse_turns == 0 {
                    let remaining = period.saturating_sub(*turn_in_cycle);
                    return Some((
                        remaining,
                        remaining as f32 / (*period).max(1) as f32,
                        TimerVisualKind::Periodic,
                    ));
                }

                let pulse_start = period.saturating_sub(pulse_turns) + 1;
                if *turn_in_cycle <= pulse_start {
                    let remaining = pulse_start - *turn_in_cycle;
                    Some((
                        remaining,
                        remaining as f32 / pulse_start.max(1) as f32,
                        TimerVisualKind::Periodic,
                    ))
                } else {
                    let remaining = period.saturating_sub(*turn_in_cycle) + 1;
                    Some((
                        remaining,
                        remaining as f32 / pulse_turns.saturating_sub(1).max(1) as f32,
                        TimerVisualKind::PeriodicPulse,
                    ))
                }
            }
            (
                TimerTemplate::ActiveAfterCountdown { turns, .. },
                TimerRuntime::AfterCountdown {
                    running, remaining, ..
                },
            ) if *running => Some((
                *remaining,
                *remaining as f32 / (*turns).max(1) as f32,
                TimerVisualKind::AfterCountdown,
            )),
            (
                TimerTemplate::ActiveDuringCountdown { turns, .. },
                TimerRuntime::DuringCountdown {
                    running, remaining, ..
                },
            ) if *running => Some((
                *remaining,
                *remaining as f32 / (*turns).max(1) as f32,
                TimerVisualKind::DuringCountdown,
            )),
            (
                TimerTemplate::OneShot {
                    turns, pulse_turns, ..
                },
                TimerRuntime::OneShot {
                    running,
                    remaining,
                    pulse_remaining,
                    output,
                },
            ) if *running || *output => {
                if let Some(remaining) = remaining {
                    Some((
                        *remaining,
                        *remaining as f32 / (*turns).max(1) as f32,
                        TimerVisualKind::OneShot,
                    ))
                } else {
                    let visible_pulse = pulse_remaining + u32::from(*output);
                    Some((
                        visible_pulse,
                        visible_pulse as f32 / (*pulse_turns).max(1) as f32,
                        TimerVisualKind::OneShotPulse,
                    ))
                }
            }
            _ => None,
        }
    }

    fn tick(&mut self) {
        match (&self.template, &mut self.runtime) {
            (
                TimerTemplate::Periodic {
                    period,
                    pulse_turns,
                    ..
                },
                TimerRuntime::Periodic {
                    running,
                    turn_in_cycle,
                    output,
                },
            ) => {
                if !*running {
                    *output = false;
                    return;
                }
                if *period == 0 {
                    *turn_in_cycle = 0;
                    *output = *pulse_turns > 0;
                    return;
                }
                if *turn_in_cycle == *period {
                    *turn_in_cycle = 0;
                }
                *turn_in_cycle += 1;
                *output = *turn_in_cycle > period.saturating_sub(*pulse_turns);
            }
            (
                TimerTemplate::ActiveAfterCountdown { .. },
                TimerRuntime::AfterCountdown {
                    running,
                    remaining,
                    output,
                },
            ) => {
                if !*running {
                    return;
                }
                if *remaining == 0 {
                    *running = false;
                    return;
                }
                *remaining -= 1;
                if *remaining == 0 {
                    *output = true;
                }
            }
            (
                TimerTemplate::ActiveDuringCountdown { .. },
                TimerRuntime::DuringCountdown {
                    running,
                    remaining,
                    output,
                },
            ) => {
                if !*running {
                    *output = false;
                    return;
                }
                if *remaining == 0 {
                    *running = false;
                    *output = false;
                    return;
                }
                *output = true;
                *remaining -= 1;
            }
            (
                TimerTemplate::OneShot { .. },
                TimerRuntime::OneShot {
                    running,
                    remaining,
                    pulse_remaining,
                    output,
                },
            ) => {
                *output = false;
                if !*running {
                    return;
                }

                if let Some(countdown) = remaining {
                    if *countdown > 0 {
                        *countdown -= 1;
                        if *countdown > 0 {
                            return;
                        }
                    } else {
                        *remaining = None;
                    }
                }

                if *pulse_remaining > 0 {
                    *output = true;
                    *pulse_remaining -= 1;
                }
                if *pulse_remaining == 0 {
                    *running = false;
                }
            }
            _ => unreachable!("timer runtime must match its template"),
        }
    }
}

#[derive(Component, Clone, Debug)]
pub struct TimerBank {
    pub slots: [Option<TimerInstance>; TIMER_SLOTS],
    initial: [Option<TimerTemplate>; TIMER_SLOTS],
}

impl TimerBank {
    pub fn new(slot_names: &InitialTimerSlots, templates: &HashMap<String, TimerTemplate>) -> Self {
        let initial = slot_names
            .clone()
            .map(|name| name.map(|name| templates.get(&name).expect("bad map metadata").clone()));
        let slots = initial
            .clone()
            .map(|template| template.map(TimerInstance::new));
        Self { slots, initial }
    }

    pub fn reset(&mut self) {
        self.slots = self
            .initial
            .clone()
            .map(|template| template.map(TimerInstance::new));
    }

    fn replace(
        &mut self,
        replacements: &[TimerReplacement; TIMER_SLOTS],
        templates: &HashMap<String, TimerTemplate>,
    ) {
        for (slot, replacement) in self.slots.iter_mut().zip(replacements) {
            match replacement {
                TimerReplacement::Template(name) => {
                    let template = templates
                        .get(name)
                        .expect("map loader validates timer replacement templates")
                        .clone();
                    *slot = Some(TimerInstance::new(template));
                }
                TimerReplacement::Keep(false) => {} // false means "keep current timer"
                TimerReplacement::Remove(()) => *slot = None, // null means "delete timer"
                TimerReplacement::Keep(true) => {
                    unreachable!("true doesn't mean anything")
                }
            }
        }
    }
}

#[derive(Message, Clone, Debug)]
pub struct ReplaceTimersRequest(pub ReplaceTimers);

#[derive(Default)]
pub struct SignalSnapshot {
    switches: HashMap<String, bool>,
    timers: HashMap<UVec2, [bool; TIMER_SLOTS]>,
}

impl SignalSnapshot {
    pub fn capture<F: QueryFilter>(
        switches: &SwitchStates,
        timer_banks: &Query<(&GridLocation, &TimerBank), F>,
    ) -> Self {
        Self {
            switches: switches
                .0
                .iter()
                .map(|(id, state)| (id.clone(), state.active))
                .collect(),
            timers: timer_banks
                .iter()
                .map(|(location, bank)| {
                    (
                        uvec2(location.0.x as u32, location.0.z as u32),
                        bank.slots
                            .each_ref()
                            .map(|slot| slot.as_ref().is_some_and(TimerInstance::output)),
                    )
                })
                .collect(),
        }
    }

    pub fn evaluate(&self, position: UVec2, expression: &SignalExpression) -> bool {
        match expression {
            SignalExpression::Constant(value) => *value,
            SignalExpression::Switch { switch } => {
                self.switches.get(switch).copied().unwrap_or(false)
            }
            SignalExpression::Timer { timer } => self
                .timers
                .get(&position)
                .and_then(|slots| slots.get(*timer as usize - 1))
                .copied()
                .unwrap_or(false),
            SignalExpression::Not { not } => !self.evaluate(position, not),
            SignalExpression::All { all } => all.iter().all(|item| self.evaluate(position, item)),
            SignalExpression::Any { any } => any.iter().any(|item| self.evaluate(position, item)),
            SignalExpression::Xor { xor } => {
                xor.iter()
                    .filter(|item| self.evaluate(position, item))
                    .count()
                    % 2
                    == 1
            }
        }
    }
}

pub fn activation_at(
    world_map: &WorldMap,
    position: UVec2,
    snapshot: &SignalSnapshot,
) -> Option<bool> {
    world_map
        .activation_conditions
        .get(&position)
        .map(|conditions| {
            conditions
                .iter()
                .all(|condition| snapshot.evaluate(position, condition))
        })
}

fn apply_timer_replacements(
    world_map: Res<WorldMap>,
    mut requests: MessageReader<ReplaceTimersRequest>,
    mut timer_banks: Query<(&GridLocation, &mut TimerBank)>,
) {
    for request in requests.read() {
        let target = uvec2(request.0.position[0], request.0.position[1]);
        let Some((_, mut bank)) = timer_banks
            .iter_mut()
            .find(|(location, _)| uvec2(location.0.x as u32, location.0.z as u32) == target)
        else {
            error!("could not find timer bank at {target}");
            continue;
        };
        bank.replace(&request.0.with, &world_map.timers);
    }
}

fn tick_timers(
    mut timer_banks: Query<&mut TimerBank>,
    mut completed_turns: MessageReader<CompletedTurn>,
) {
    for _ in completed_turns.read() {
        for mut bank in &mut timer_banks {
            for timer in bank.slots.iter_mut().flatten() {
                timer.tick();
            }
        }
    }
}
