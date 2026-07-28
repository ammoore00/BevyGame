use bevy::prelude::*;

pub fn get_mouse_angle_from_position(
    position: Vec2,
    mouse_cursor_event: &CursorMoved,
) -> Option<f32> {
    let (x, y) = (
        mouse_cursor_event.position.x - position.x,
        mouse_cursor_event.position.y - position.y,
    );
    let angle = x.atan2(y) * 180.0 / std::f32::consts::PI;
    Some(angle.clamp(-180.0, 180.0))
}
