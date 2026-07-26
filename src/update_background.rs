use bevy::prelude::*;
use crate::ecs::{Background, CompletedTurn, TurnCounter};
use crate::MAX_TURN_COUNT;

pub fn update_background_plugin(app: &mut App) {
    app.add_systems(Update, update_background);
}

fn update_background(
    mut background: Single<&mut Sprite, With<Background>>,
    turn_counter: Res<TurnCounter>,
    mut completed_turns: MessageReader<CompletedTurn>,
) {
    for _ in completed_turns.read() {
        background.color.set_alpha(turn_counter.0 as f32 / MAX_TURN_COUNT as f32);
    }
}