use crate::resource::ResourceKind;
use bevy::ecs::query::QueryItem;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use crate::loc::ResourceLocation;

/// Used to prevent construction of marker outside of proper systems
pub struct PrototypeMarkerToken(());

pub trait PrototypeFinalizedMarker: Component {
    fn new(token: PrototypeMarkerToken) -> Self;
}

pub trait Prototype: SceneComponent {
    /// The marker component used to replace the prototype once initialized
    type Marker: PrototypeFinalizedMarker;
    /// The kind of resource to load for the prototype's data
    type Resource: ResourceKind;
    /// The component used to store the location of the prototype's data to be loaded
    type DataLocation: Into<ResourceLocation<Self::Resource>> + Component;
}

pub trait PrototypeBuilder {
    /// The prototype to be built
    type Proto: Prototype;

    /// The context needed to initialize the prototype
    type Context<'w, 's>: SystemParam;
    /// Any additional query data needed to initialize the prototype
    type QueryData<'w, 's>: bevy::ecs::query::QueryData;

    fn build(
        entity: Entity,
        loc: &<Self::Proto as Prototype>::DataLocation,
        extra_data: &QueryItem<'_, '_, <Self::QueryData<'_, '_> as bevy::ecs::query::QueryData>::ReadOnly>,
        context: &mut Self::Context<'_, '_>,
        commands: Commands,
    ) -> Result<(), BevyError>;
}

/// SECURE GATEWAY: This handles the actual transformation and safely mints the `MarkerToken`.
/// It is marked `pub` or `pub(crate)` so the macro can call it, but because it hides the
/// token instantiation inside, no external module can exploit it.
#[doc(hidden)]
pub fn finalize_prototype<B: PrototypeBuilder>(
    entity: Entity,
    mut commands: Commands,
) {
    commands.entity(entity)
        .remove::<(
            <B as PrototypeBuilder>::Proto,
            <<B as PrototypeBuilder>::Proto as Prototype>::DataLocation
        )>()
        .insert(<<B as PrototypeBuilder>::Proto as Prototype>::Marker::new(PrototypeMarkerToken(())));
}

#[macro_export]
macro_rules! register_prototype_system {
    ($system_name:ident, $builder_type:ty) => {
        fn $system_name(
            query: bevy::prelude::Query<
                (
                    bevy::prelude::Entity,
                    &<<$builder_type as $crate::prototyping::PrototypeBuilder>::Proto as $crate::prototyping::Prototype>::DataLocation,
                    <$builder_type as $crate::prototyping::PrototypeBuilder>::QueryData<'_, '_>
                ),
                bevy::prelude::With<<$builder_type as $crate::prototyping::PrototypeBuilder>::Proto>,
            >,
            mut context: bevy::ecs::system::StaticSystemParam<<$builder_type as $crate::prototyping::PrototypeBuilder>::Context<'_, '_>>,
            mut commands: bevy::prelude::Commands,
        ) {
            for (entity, data_loc, extra_data) in &query {
                match <$builder_type as $crate::prototyping::PrototypeBuilder>::build(
                    entity,
                    data_loc,
                    &extra_data,
                    &mut context,
                    commands.reborrow()
                ) {
                    Ok(_) => $crate::prototyping::finalize_prototype::<$builder_type>(entity, commands.reborrow()),
                    Err(err) => {
                        error!("{}", err);
                        commands.entity(entity).despawn_children().despawn();
                    }
                }
            }
        }
    };
}