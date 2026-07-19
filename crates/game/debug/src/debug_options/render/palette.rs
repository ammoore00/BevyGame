use bevy::color::Color;

//------ Navigation ------//

pub const NAV_NODE_COLOR: Color = Color::srgb(0.55, 0.95, 0.80);
pub const NAV_EDGE_FORWARD_COLOR: Color = Color::srgb(0.30, 0.60, 0.95);
pub const NAV_EDGE_REVERSE_COLOR: Color = Color::srgb(0.20, 0.75, 0.80);

pub const _PATH_COLOR: Color = Color::srgb(0.95, 0.85, 0.30);

pub const NAV_NODE_RADIUS: f32 = 0.125;
pub const NAV_NODE_LINE_THICKNESS: f32 = 3.0;

pub const NAV_EDGE_LINE_THICKNESS: f32 = 4.0;
pub const NAV_EDGE_DIRECTIONAL_OFFSET: f32 = 0.1;
pub const NAV_EDGE_END_PADDING: f32 = 0.25;
pub const NAV_EDGE_ARROW_LENGTH: f32 = 0.125;
pub const NAV_EDGE_ARROW_WIDTH: f32 = 0.125;

//------ Physics ------//

pub const KINEMATIC_COLLIDER_COLOR: Color = Color::srgb(0.90, 0.75, 0.35);
pub const STATIC_COLLIDER_COLOR: Color = Color::srgb(0.90, 0.35, 0.35);
pub const CONVEX_HULL_COLOR: Color = Color::srgb(0.95, 0.55, 0.30);

pub const COLLIDER_LINE_THICKNESS: f32 = 2.0;
