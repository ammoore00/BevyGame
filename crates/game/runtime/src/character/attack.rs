use bevy::prelude::*;
use assets::resource::characters::{AttackContext, AttackResource};
use common::Facing;
use data::loc::ResourceLocation;
use crate::character::stamina::StaminaEvent;
use crate::particle::{ParticleAnimation, ParticleSpawnEvent};

pub(super) fn plugin(app: &mut App) {
    app.add_observer(on_attack);
}

#[derive(EntityEvent, Debug, Clone, Reflect, derive_new::new)]
pub struct AttackEvent {
    entity: Entity,
    facing: Facing,
    attack: ResourceLocation<AttackResource>,
}

fn on_attack(
    event: On<AttackEvent>,
    context: AttackContext,
    mut commands: Commands,
) {
    let Some(attack) = context.attack_registry.get_asset(&event.attack) else {
        error!("Invalid attack event: attack {} does not exist!", event.attack);
        return;
    };

    let Ok(animation) = context.animation_context.get_data(attack.animation()) else {
        error!("Invalid attack definition: animation {} does not exist!", attack.animation());
        return;
    };

    let Some(particle_sprite) = context.character_sprite_registry.get_handle(attack.particle_sprite()) else {
        error!("Invalid attack definition: particle sprite {} does not exist!", attack.particle_sprite());
        return;
    };

    let particle_atlas = animation.atlas().clone().with_index(event.facing as usize);

    let particle_sprite = Sprite::from_atlas_image(
        particle_sprite,
        particle_atlas,
    );

    let particle_animation = ParticleAnimation::new(
        event.facing as usize * animation.frame_data().num_frames(),
        animation.frame_data().clone(),
    );

    commands.trigger(ParticleSpawnEvent::with_parent(
        particle_sprite,
        particle_animation,
        event.entity,
    ));

    commands.trigger(StaminaEvent::new(event.entity, attack.stamina_cost()));
}