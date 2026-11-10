use crate::animation::AnimationSystems;
use crate::animation::tracking::AnimationTracker;
use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_sprites.in_set(AnimationSystems::Sprite));
}

fn update_sprites(
    animation_query: Query<(Entity, &mut Sprite, &AnimationTracker)>,
    mut commands: Commands,
) {
    for (entity, mut sprite, tracker) in animation_query {
        let Some(ref mut atlas) = sprite.texture_atlas else {
            error!(
                "Animation {} applied to sprite without texture atlas! Removing animation.",
                tracker
            );
            commands.entity(entity).remove::<AnimationTracker>();

            continue;
        };

        atlas.index = tracker.current_index();
    }
}
