use std::time::Instant;
use bevy::prelude::*;
use crate::ecs::LinearTranslationAnimation;

pub fn animation_plugin(app: &mut App) {
    app
        .add_systems(Update, linear_translation_animation);
}

fn linear_translation_animation(
    mut animation_query: Query<(Entity, &mut Transform, &LinearTranslationAnimation)>,
    mut commands: Commands,
) {
    for (entity, mut transform, animation) in animation_query.iter_mut() {
        let elapsed = animation.start.elapsed();
        let progress = elapsed.as_secs_f32() / animation.length.as_secs_f32();
        transform.translation = animation.start_position + progress.min(1.0)*(animation.end_position - animation.start_position);

        info!("progress: {} {} {}", progress, elapsed.as_secs_f32(), animation.length.as_secs_f32());
        if progress > 1.0 {
            commands.entity(entity).remove::<LinearTranslationAnimation>();
        }
    }
}