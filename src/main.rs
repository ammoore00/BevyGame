use bevy::app::{App, AppExit};
use bevy_game_2d::AppPlugin;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}