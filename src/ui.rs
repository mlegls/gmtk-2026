use bevy::prelude::*;
use crate::ecs::{CompletedTurn, TurnCountText, TurnCounter};

pub fn ui_plugin(app: &mut App) {
    app.add_systems(Update, update_turn_count);
}

pub fn ui() -> impl Scene {
    bsn! {
        Node {
            width: percent(100),
            height: percent(100),
        }
        Children [
            (
                template(|ctx| {
                    Ok(Text(ctx.resource::<TurnCounter>().to_string()))
                })
                TurnCountText
            )
        ]
    }
}

fn update_turn_count(
    turn_counter: Res<TurnCounter>,
    mut turn_count_text: Single<&mut Text, With<TurnCountText>>,
    mut completed_turn_recv: MessageReader<CompletedTurn>,
) {
    for _ in completed_turn_recv.read() {
        ***turn_count_text = turn_counter.to_string();
    }
}