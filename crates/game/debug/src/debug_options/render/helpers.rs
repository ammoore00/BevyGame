use std::collections::HashSet;
use bevy::prelude::*;
use common::{ScreenCoords, WorldCoords};
use physics::CapsuleData;

#[derive(Debug, Clone, Copy)]
pub struct LineSettings {
    pub color: Color,
    pub thickness: f32,
}
impl Default for LineSettings {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            thickness: 4.0,
        }
    }
}

/// Draw a simple line in screen space coordinates
pub fn draw_screen_line(
    a: ScreenCoords,
    b: ScreenCoords,
    settings: LineSettings,
) -> impl Bundle {
    let delta = *b - *a;
    let length = delta.xy().length();

    if length <= f32::EPSILON {
        // TODO: Handle this more gracefully
        panic!("Cannot draw line of length zero!");
    }

    let midpoint = (*a + *b) / 2.0;
    let angle = delta.y.atan2(delta.x);

    (
        Sprite {
            color: settings.color,
            custom_size: Some(Vec2::new(length, settings.thickness)),
            ..default()
        },
        Transform {
            translation: midpoint,
            rotation: Quat::from_rotation_z(angle),
            ..default()
        },
    )
}

/// Draw a simple line in world space coordinates
pub fn draw_world_line(
    a: WorldCoords,
    b: WorldCoords,
    settings: LineSettings,
    scale: f32,
) -> impl Bundle {
    let a = project_point(a, scale);
    let b = project_point(b, scale);
    draw_screen_line(a, b, settings)
}

/// Draw a circle in world space coordinates, but facing the camera
/// This prevents shenanigans with z ordering by keeping all line segments in the same plane
pub fn draw_world_space_circle_projected(
    center: WorldCoords,
    radius: f32,
    settings: LineSettings,
    scale: f32,
) -> Vec<impl Bundle> {
    let horizontal = Vec3::new(1.0, 0.0, -1.0).normalize();
    let vertical = Vec3::Y;
    let segments = 24;

    let mut lines = Vec::new();

    for index in 0..segments {
        let start_angle = index as f32 / segments as f32 * std::f32::consts::TAU;
        let end_angle = (index + 1) as f32 / segments as f32 * std::f32::consts::TAU;

        let start = center.0
            + horizontal * start_angle.cos() * radius
            + vertical * start_angle.sin() * radius;

        let end = center.0
            + horizontal * end_angle.cos() * radius
            + vertical * end_angle.sin() * radius;

        lines.push(draw_world_line(start.into(), end.into(), settings, scale));
    }

    lines
}

/// Draw a cuboid based on the given size and location
pub fn draw_cuboid(
    center: WorldCoords,
    half_extents: Vec3,
    settings: LineSettings,
    scale: f32,
) -> Vec<impl Bundle> {
    let min = center.0 - half_extents;
    let max = center.0 + half_extents;

    let corners = [
        WorldCoords(Vec3::new(min.x, min.y, min.z)),
        WorldCoords(Vec3::new(max.x, min.y, min.z)),
        WorldCoords(Vec3::new(max.x, min.y, max.z)),
        WorldCoords(Vec3::new(min.x, min.y, max.z)),
        WorldCoords(Vec3::new(min.x, max.y, min.z)),
        WorldCoords(Vec3::new(max.x, max.y, min.z)),
        WorldCoords(Vec3::new(max.x, max.y, max.z)),
        WorldCoords(Vec3::new(min.x, max.y, max.z)),
    ];

    let edges = [
        // +x face (right)
        (1, 2), (2, 6), (6, 5), (5, 1),
        // +y face (top)
        (4, 5), (5, 6), (6, 7), (7, 4),
        // +z face (front)
        (2, 3), (3, 7), (7, 6), (6, 2),
    ];

    edges.iter()
        .map(|(a, b)| draw_world_line(corners[*a], corners[*b], settings, scale))
        .collect()
}

// TODO: Fix the dual return, as the types are not actually different,
//  the compiler just doesn't know that
//  Likely fix is to define a custom `LineBundle` with Sprite and Transform fields
/// Draw a capsule
/// Note that two separate collections are returned, as the opaque types are different. 
pub fn draw_capsule(
    position: WorldCoords,
    capsule: impl Into<CapsuleData>,
    settings: LineSettings,
    scale: f32,
) -> (Vec<impl Bundle>, Vec<impl Bundle>) {
    let capsule = capsule.into();

    let a = position.0 + capsule.a;
    let b = position.0 + capsule.b;

    let mut edges = Vec::new();
    let mut circles = Vec::new();

    // Main capsule axis.
    edges.push(draw_world_line(a.into(), b.into(), settings, scale));

    // Cross-sections perpendicular to the vertical capsule axis.
    circles.append(&mut draw_world_space_circle_projected(a.into(), capsule.radius, settings, scale));
    circles.append(&mut draw_world_space_circle_projected(b.into(), capsule.radius, settings, scale));

    let horizontal = Vec3::new(1.0, 0.0, -1.0).normalize();
    let vertical = Vec3::Y;

    let offsets = [
        horizontal * capsule.radius,
        -horizontal * capsule.radius,
        vertical * capsule.radius,
        -vertical * capsule.radius,
    ];

    // Additional side lines to fill out capsule
    for offset in offsets {
        edges.push(draw_world_line((a + offset).into(), (b + offset).into(), settings, scale));
    }

    (edges, circles)
}

/// Draw a convex hull, rendering only the wireframe edges of faces pointing toward the positive axes.
pub fn draw_convex_hull(
    position: WorldCoords,
    vertices: &[Vec3],
    indices: &[[u32; 3]],
    settings: LineSettings,
    scale: f32,
) -> Vec<impl Bundle> {
    let mut unique_edges = HashSet::new();

    for &[i0, i1, i2] in indices {
        let v0 = vertices[i0 as usize];
        let v1 = vertices[i1 as usize];
        let v2 = vertices[i2 as usize];

        // Calculate the normal of the triangle
        let edge1 = v1 - v0;
        let edge2 = v2 - v0;
        let normal = edge1.cross(edge2).normalize_or_zero();

        // Check if the face is pointing towards the positive axes.
        if normal.dot(Vec3::ONE) > 0.0 {
            // Sort the indices for each edge so that (A, B) and (B, A) hash to the same value
            let e1 = if i0 < i1 { (i0, i1) } else { (i1, i0) };
            let e2 = if i1 < i2 { (i1, i2) } else { (i2, i1) };
            let e3 = if i2 < i0 { (i2, i0) } else { (i0, i2) };

            unique_edges.insert(e1);
            unique_edges.insert(e2);
            unique_edges.insert(e3);
        }
    }

    unique_edges
        .into_iter()
        .map(|(a, b)| {
            let start = position.0 + vertices[a as usize];
            let end = position.0 + vertices[b as usize];

            draw_world_line(WorldCoords(start), WorldCoords(end), settings, scale)
        })
        .collect()
}

const WORLD_LINE_Z_OFFSET: f32 = 0.25;

fn project_point(point: WorldCoords, scale: f32) -> ScreenCoords {
    let screen_coords = ScreenCoords::from(point);

    ScreenCoords(Vec3::new(
        screen_coords.x * scale,
        screen_coords.y * scale,
        screen_coords.z + WORLD_LINE_Z_OFFSET,
    ))
}