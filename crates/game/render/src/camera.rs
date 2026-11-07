use bevy::prelude::*;
use common::{AppSystems, MainCamera};
use runtime::prelude::Player;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(Startup, camera.spawn());
    
    app.add_systems(
        FixedUpdate,
        camera_follow_player.in_set(AppSystems::Respond),
    );
}

fn camera() -> impl Scene {
    bsn! {
        #Camera
        MainCamera
        Camera2d
        Projection::Orthographic(OrthographicProjection {
            near: -10000000.0,
            far: 10000000.0,
            ..OrthographicProjection::default_2d()
        })
    }
}

/// Update the camera position to match the player position
fn camera_follow_player(
    player_transform: Single<&mut Transform, (With<Player>, Without<Camera2d>)>,
    mut camera: Single<&mut Transform, With<Camera2d>>,
) {
    camera.translation = player_transform.translation * Vec3::new(1.0, 1.0, 0.0);
}
