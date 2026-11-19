use crate::animation::AnimationSystems;
use crate::animation::tracking::AnimationTracker;
use assets::resource::AnimationResource;
use bevy::prelude::*;
use data::prelude::SystemRegistry;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Update, update_sprites.in_set(AnimationSystems::Sprite));
}

fn update_sprites(
    animation_query: Query<(Entity, &mut Sprite, &AnimationTracker)>,
    animation_registry: SystemRegistry<AnimationResource>,
    mut commands: Commands,
) {
    for (entity, mut sprite, tracker) in animation_query {
        let Some(animation) = animation_registry.get_asset_from_handle(tracker.handle()) else {
            error!("Unknown animation in tracker: {}", tracker.loc());
            commands.entity(entity).remove::<AnimationTracker>();
            continue;
        };
        
        let Some(animation) = animation.as_resolved() else {
            error!("Animation in tracker is not resolved: {}", tracker.loc());
            commands.entity(entity).remove::<AnimationTracker>();
            continue;
        };
        
        let Some(ref mut atlas) = sprite.texture_atlas else {
            error!(
                "Animation {} applied to sprite without texture atlas! Removing animation.",
                tracker
            );
            commands.entity(entity).remove::<AnimationTracker>();
            continue;
        };
        
        atlas.index = tracker.current_index();
        sprite.image = animation.image();
    }
}
