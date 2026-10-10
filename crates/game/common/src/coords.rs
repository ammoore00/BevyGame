use crate::Scale;
use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::cmp::Ordering;
use std::ops::{AddAssign, Deref, DivAssign, MulAssign, SubAssign};

pub const SCREEN_Z_SCALE: f32 = 2.0;

pub const TILE_WIDTH: i32 = 32;
pub const TILE_HEIGHT: i32 = TILE_WIDTH / 2;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(
        FixedPreUpdate,
        (
            convert_world_to_screen_coords_system,
            convert_tile_to_screen_coords_system,
        ),
    );
}

#[derive(Component, Debug, Default, Clone, Copy)]
pub struct WorldPosition(pub WorldCoords);

impl WorldPosition {
    pub fn as_vec3(&self) -> Vec3 {
        self.0.0
    }

    pub fn set(&mut self, value: Vec3) {
        self.0.0 = value;
    }
}

fn convert_world_to_screen_coords_system(
    mut query: Query<(&WorldPosition, &mut Transform), Changed<WorldPosition>>,
    scale: Res<Scale>,
) {
    for (world_position, mut transform) in query.iter_mut() {
        transform.translation = convert_world_to_screen_coords(*scale, world_position.0).0;
    }
}

pub fn convert_world_to_screen_coords(
    scale: Scale,
    pos: WorldCoords,
) -> ScreenCoords {
    let mut screen_coords = ScreenCoords::from(pos);

    screen_coords.0.x *= scale.0;
    screen_coords.0.y *= scale.0;

    screen_coords
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct TilePosition(pub TileCoords);
pub fn convert_tile_to_screen_coords_system(
    mut query: Query<(&TilePosition, &mut Transform), Changed<TilePosition>>,
) {
    for (tile_position, mut transform) in query.iter_mut() {
        let screen_coords = ScreenCoords::from(&tile_position.0);
        transform.translation = *screen_coords;

        transform.translation.y -= TILE_HEIGHT as f32;
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Reflect)]
pub struct TileCoords(pub IVec3);
impl From<WorldCoords> for TileCoords {
    fn from(value: WorldCoords) -> Self {
        Self::from(value.0)
    }
}
impl From<&WorldCoords> for TileCoords {
    fn from(value: &WorldCoords) -> Self {
        Self(value.0.as_ivec3())
    }
}
impl From<IVec3> for TileCoords {
    fn from(value: IVec3) -> Self {
        TileCoords(value)
    }
}
impl From<Vec3> for TileCoords {
    fn from(value: Vec3) -> Self {
        // Use round() instead of as_ivec3() to get proper rounding
        TileCoords(IVec3::new(
            value.x.round() as i32,
            value.y.round() as i32,
            value.z.round() as i32,
        ))
    }
}
impl From<[i32; 3]> for TileCoords {
    fn from(value: [i32; 3]) -> Self {
        TileCoords(IVec3::new(value[0], value[1], value[2]))
    }
}
impl Deref for TileCoords {
    type Target = IVec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Ord for TileCoords {
    fn cmp(&self, other: &Self) -> Ordering {
        match self.y.cmp(&other.y) {
            Ordering::Equal => match self.z.cmp(&other.z) {
                Ordering::Equal => self.x.cmp(&other.x),
                ordering => ordering,
            },
            ordering => ordering,
        }
    }
}
impl PartialOrd for TileCoords {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl Serialize for TileCoords {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        [self.0.x, self.0.y, self.0.z].serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for TileCoords {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let [x, y, z]: [i32; 3] = Deserialize::deserialize(deserializer)?;
        Ok(TileCoords(IVec3::new(x, y, z)))
    }
}
impl<T> std::ops::Add<T> for TileCoords
where
    IVec3: std::ops::Add<T, Output = IVec3>
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl<T> AddAssign<T> for TileCoords
where
    IVec3: AddAssign<T>
{
    fn add_assign(&mut self, rhs: T) {
        self.0 += rhs;
    }
}
impl<T> std::ops::Sub<T> for TileCoords
where
    IVec3: std::ops::Sub<T, Output = IVec3>
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0 - rhs)
    }
}
impl<T> SubAssign<T> for TileCoords
where
    IVec3: SubAssign<T>
{
    fn sub_assign(&mut self, rhs: T) {
        self.0 -= rhs;
    }
}
impl<T> std::ops::Mul<T> for TileCoords
where
    IVec3: std::ops::Mul<T, Output = IVec3>
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl<T> MulAssign<T> for TileCoords
where
    IVec3: MulAssign<T>
{
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs;
    }
}
impl<T> std::ops::Div<T> for TileCoords
where
    IVec3: std::ops::Div<T, Output = IVec3>
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs)
    }
}
impl<T> DivAssign<T> for TileCoords
where
    IVec3: DivAssign<T>
{
    fn div_assign(&mut self, rhs: T) {
        self.0 /= rhs;
    }
}

/// Represents a continuous coordinate in the game world
///
/// ### Ordering
///
/// Ordering is performed Y first, then X, then Z, using `f32::total_cmp`
#[derive(Debug, Clone, Copy, Default)]
pub struct WorldCoords(pub Vec3);
impl From<TileCoords> for WorldCoords {
    fn from(value: TileCoords) -> Self {
        Self::from(value.0)
    }
}
impl From<&TileCoords> for WorldCoords {
    fn from(value: &TileCoords) -> Self {
        Self(value.0.as_vec3())
    }
}
impl From<Vec3> for WorldCoords {
    fn from(value: Vec3) -> Self {
        WorldCoords(value)
    }
}
impl From<IVec3> for WorldCoords {
    fn from(value: IVec3) -> Self {
        WorldCoords(value.as_vec3())
    }
}
impl Deref for WorldCoords {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl Eq for WorldCoords {}
impl PartialEq for WorldCoords {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl Ord for WorldCoords {
    fn cmp(&self, other: &Self) -> Ordering {
        self.y
            .total_cmp(&other.y)
            .then_with(|| self.x.total_cmp(&other.x))
            .then_with(|| self.z.total_cmp(&other.z))
    }
}
impl PartialOrd for WorldCoords {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<T> std::ops::Add<T> for WorldCoords
where
    Vec3: std::ops::Add<T, Output = Vec3>
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl<T> AddAssign<T> for WorldCoords
where
    Vec3: AddAssign<T>
{
    fn add_assign(&mut self, rhs: T) {
        self.0 += rhs;
    }
}
impl<T> std::ops::Sub<T> for WorldCoords
where
    Vec3: std::ops::Sub<T, Output = Vec3>
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0 - rhs)
    }
}
impl<T> SubAssign<T> for WorldCoords
where
    Vec3: SubAssign<T>
{
    fn sub_assign(&mut self, rhs: T) {
        self.0 -= rhs;
    }
}
impl<T> std::ops::Mul<T> for WorldCoords
where
    Vec3: std::ops::Mul<T, Output = Vec3>
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl<T> MulAssign<T> for WorldCoords
where
    Vec3: MulAssign<T>
{
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs;
    }
}
impl<T> std::ops::Div<T> for WorldCoords
where
    Vec3: std::ops::Div<T, Output = Vec3>
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs)
    }
}
impl<T> DivAssign<T> for WorldCoords
where
    Vec3: DivAssign<T>
{
    fn div_assign(&mut self, rhs: T) {
        self.0 /= rhs;
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct ScreenCoords(pub Vec3);
impl From<WorldCoords> for ScreenCoords {
    fn from(value: WorldCoords) -> Self {
        let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
        let screen_y =
            ((value.y - 1.0) * TILE_HEIGHT as f32) - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

        let screen_z = (value.x + value.z + (value.y)) * SCREEN_Z_SCALE;

        Vec3::new(screen_x, screen_y, screen_z).into()
    }
}
impl From<&WorldCoords> for ScreenCoords {
    fn from(value: &WorldCoords) -> Self {
        let screen_x = (value.x - value.z) * TILE_WIDTH as f32 / 2.0;
        let screen_y =
            ((value.y - 1.0) * TILE_HEIGHT as f32) - (value.x + value.z) * TILE_HEIGHT as f32 / 2.0;

        let screen_z = (value.x + value.z + (value.y)) * SCREEN_Z_SCALE;

        Vec3::new(screen_x, screen_y, screen_z).into()
    }
}
impl From<TileCoords> for ScreenCoords {
    fn from(value: TileCoords) -> Self {
        let screen_x = (value.x - value.z) * TILE_WIDTH / 2;
        let screen_y = (value.y * TILE_HEIGHT) - (value.x + value.z) * TILE_HEIGHT / 2;

        let screen_z = value.x as f32 + value.z as f32 + value.y as f32;

        Vec3::new(screen_x as f32, screen_y as f32, screen_z).into()
    }
}
impl From<&TileCoords> for ScreenCoords {
    fn from(value: &TileCoords) -> Self {
        let screen_x = (value.x - value.z) * TILE_WIDTH / 2;
        let screen_y = (value.y * TILE_HEIGHT) - (value.x + value.z) * TILE_HEIGHT / 2;

        let screen_z = value.x as f32 + value.z as f32 + value.y as f32;

        Vec3::new(screen_x as f32, screen_y as f32, screen_z).into()
    }
}
impl From<Vec3> for ScreenCoords {
    fn from(value: Vec3) -> Self {
        ScreenCoords(value)
    }
}
impl Deref for ScreenCoords {
    type Target = Vec3;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> std::ops::Add<T> for ScreenCoords
where
    Vec3: std::ops::Add<T, Output = Vec3>
{
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        Self(self.0 + rhs)
    }
}
impl<T> AddAssign<T> for ScreenCoords
where
    Vec3: AddAssign<T>
{
    fn add_assign(&mut self, rhs: T) {
        self.0 += rhs;
    }
}
impl<T> std::ops::Sub<T> for ScreenCoords
where
    Vec3: std::ops::Sub<T, Output = Vec3>
{
    type Output = Self;
    fn sub(self, rhs: T) -> Self::Output {
        Self(self.0 - rhs)
    }
}
impl<T> SubAssign<T> for ScreenCoords
where
    Vec3: SubAssign<T>
{
    fn sub_assign(&mut self, rhs: T) {
        self.0 -= rhs;
    }
}
impl<T> std::ops::Mul<T> for ScreenCoords
where
    Vec3: std::ops::Mul<T, Output = Vec3>
{
    type Output = Self;
    fn mul(self, rhs: T) -> Self::Output {
        Self(self.0 * rhs)
    }
}
impl<T> MulAssign<T> for ScreenCoords
where
    Vec3: MulAssign<T>
{
    fn mul_assign(&mut self, rhs: T) {
        self.0 *= rhs;
    }
}
impl<T> std::ops::Div<T> for ScreenCoords
where
    Vec3: std::ops::Div<T, Output = Vec3>
{
    type Output = Self;
    fn div(self, rhs: T) -> Self::Output {
        Self(self.0 / rhs)
    }
}
impl<T> DivAssign<T> for ScreenCoords
where
    Vec3: DivAssign<T>
{
    fn div_assign(&mut self, rhs: T) {
        self.0 /= rhs;
    }
}

pub fn rotate_screen_space_to_facing(screen_space: Vec2, invert_y: bool) -> Vec2 {
    let angle = -std::f32::consts::FRAC_PI_4;
    let rotation = Mat2::from_angle(angle);
    let y = if invert_y { -1.0 } else { 1.0 };
    rotation * (screen_space * Vec2::new(1.0, y))
}

pub fn rotate_screen_space_to_movement(screen_space: Vec3) -> Vec3 {
    let angle = std::f32::consts::FRAC_PI_4;
    let rotation = Quat::from_rotation_y(angle);
    rotation * screen_space
}
