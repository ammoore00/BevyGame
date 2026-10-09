use bevy::prelude::*;
use common::{marker, Scale, ScreenCoords, WorldPosition, MainCamera, TILE_WIDTH};
use physics::Collider;
use widgets::text::{SMALL_FONT_SIZE, TextFormatting};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, spawn_debug_canvas.spawn());
    app.add_systems(Update, update_debug_labels);
}

#[derive(Component, Debug, Clone)]
#[relationship_target(relationship = DebugLabelAttachedTo, linked_spawn)]
pub struct DebugLabelList(Vec<Entity>);

#[derive(Component, Debug, Clone, Copy)]
#[relationship(relationship_target = DebugLabelList)]
pub struct DebugLabelAttachedTo(pub Entity);

#[derive(Component)]
pub struct DebugLabel {
    pub name: String,
    pub name_color: Color,

    pub text: String,
    pub text_color: Color,
}
impl DebugLabel {
    pub fn as_scene(&self) -> impl Scene {
        let name_formatting = TextFormatting::new(SMALL_FONT_SIZE, self.name_color);
        let text_formatting = TextFormatting::new(SMALL_FONT_SIZE, self.text_color);

        bsn! {
            Node {
                flex_direction: FlexDirection::Row,
            }
            Children [
                @widgets::text::text(format!("{}: ", self.name.as_str()), name_formatting)
                --
                @widgets::text::text(self.text.as_str(), text_formatting)
            ]
        }
    }
}

marker!(DebugCanvas);

fn spawn_debug_canvas() -> impl Scene {
    bsn! {
        #DebugCanvas
        DebugCanvas
        @widgets::background::ui_root()
    }
}

marker!(DebugLabelCollection);

fn update_debug_labels(
    label_list_query: Query<(&DebugLabelList, &WorldPosition, &Collider)>,
    debug_label_query: Query<&DebugLabel, With<DebugLabelAttachedTo>>,
    existing_debug_labels_query: Query<Entity, With<DebugLabelCollection>>,
    canvas: Single<Entity, With<DebugCanvas>>,
    camera: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    scale: Res<Scale>,
    mut commands: Commands,
) {
    // TODO: Maybe change this to update in place rather than respawn every frame?
    // Despawn existing labels
    for entity in existing_debug_labels_query {
        commands.entity(entity).despawn();
    }
    
    let (camera, camera_transform) = camera.into_inner();

    for (label_list, pos, collider) in label_list_query {
        // Render debug above the entity
        let world_coords = pos.0 + (collider.size() * Vec3::Y);
        
        let mut screen_coords = ScreenCoords::from(world_coords);
        screen_coords -= Vec3::X * 0.25 * TILE_WIDTH as f32;
        screen_coords *= scale.0;
        
        // Convert 2D world coords into viewport coords
        let Ok(viewport_pos) = camera.world_to_viewport(camera_transform, *screen_coords) else {
            error!("Failed to convert screen coords to viewport coords");
            return;
        };
        
        let height = if let Some(viewport_size) = camera.logical_viewport_size() {
            viewport_size.y
        } else {
            error!("Failed to get viewport size");
            return;
        };
        
        let ui_x = viewport_pos.x;
        let ui_y = height - viewport_pos.y;

        // Spawn the base node for holding all debug labels
        let debug_display = commands.spawn_scene(bsn! {
            DebugLabelCollection
            Node {
                position_type: PositionType::Absolute,
                left: {px(ui_x)},
                bottom: {px(ui_y)},
                flex_direction: FlexDirection::Column,
            }
        }).id();
        commands.entity(canvas.entity()).add_child(debug_display);

        // For each debug label in the list, spawn the associated scene to render it
        let label_list = label_list
            .0
            .iter()
            .map(|label| {
                let label = debug_label_query.get(*label).unwrap();
                commands.spawn_scene(label.as_scene()).id()
            })
            .collect::<Vec<_>>();

        // Attach the debug labels to the base node
        commands.entity(debug_display).add_children(&label_list);
    }
}
