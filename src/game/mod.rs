pub mod audio;
pub mod background;
pub mod boss;
pub mod camera;
pub mod collisions;
pub mod config;
pub mod debug;
pub mod enemies;
pub mod player;
pub mod spawn;
pub mod states;
pub mod ui;
pub mod weapons;

pub use states::AppState;

use audio::AudioPlugin;
use background::BackgroundPlugin;
use boss::BossPlugin;
use camera::CameraPlugin;
use collisions::CollisionPlugin;
use config::ConfigPlugin;
use debug::DebugPlugin;
use enemies::EnemiesPlugin;
use player::PlayerPlugin;
use spawn::SpawnPlugin;
use states::StatePlugin;
use ui::UiPlugin;
use weapons::WeaponsPlugin;

use bevy::prelude::*;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            ConfigPlugin,
            StatePlugin,
            DebugPlugin,
            CameraPlugin,
            BackgroundPlugin,
            UiPlugin,
            PlayerPlugin,
            WeaponsPlugin,
            EnemiesPlugin,
            SpawnPlugin,
            CollisionPlugin,
            BossPlugin,
            AudioPlugin,
        ));
    }
}
