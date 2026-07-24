use bevy::prelude::*;
use crate::ecs::{PressurePlate, PressurePlateEnterMessage, PressurePlateLeaveMessage};

pub fn pressure_plate_plugin(app: &mut App) {
    app.add_systems(Update, (pressure_plate_message, pressure_plate_leave_message));
}

fn pressure_plate_message(
    mut messages: MessageReader<PressurePlateEnterMessage>,
    pressure_plates: Query<&Transform, With<PressurePlate>>
) {
    for PressurePlateEnterMessage(entity) in messages.read() {
        if let Ok(transform) = pressure_plates.get(*entity) {
            info!("pressed pressure plate");
        }
    }
}
fn pressure_plate_leave_message(
    mut messages: MessageReader<PressurePlateLeaveMessage>,
    pressure_plates: Query<&Transform, With<PressurePlate>>
) {
    for PressurePlateLeaveMessage(entity) in messages.read() {
        if let Ok(transform) = pressure_plates.get(*entity) {
            info!("left pressure plate");
        }
    }
}