use bevy::{prelude::*, time::Fixed};

#[derive(Resource, Debug)]
pub struct GameConfig {
    pub logical_width: f32,
    pub logical_height: f32,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            logical_width: 1280.0,
            logical_height: 720.0,
        }
    }
}

pub struct ConfigPlugin;

impl Plugin for ConfigPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameConfig>()
            .init_resource::<GameSettings>()
            .register_type::<GameSettings>()
            .register_type::<Difficulty>()
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 120.0));
    }
}

#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct GameSettings {
    pub difficulty: Difficulty,
    pub music_volume: f32,
    pub sfx_volume: f32,
}

impl Default for GameSettings {
    fn default() -> Self {
        Self {
            difficulty: Difficulty::Normal,
            music_volume: 0.6,
            sfx_volume: 0.7,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum Difficulty {
    Easy,
    Normal,
    Hard,
}

impl Difficulty {
    pub fn spawn_interval_factor(self) -> f32 {
        match self {
            Difficulty::Easy => 1.25,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 0.8,
        }
    }

    pub fn enemy_health_factor(self) -> f32 {
        match self {
            Difficulty::Easy => 0.9,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.15,
        }
    }

    pub fn enemy_bullet_factor(self) -> f32 {
        match self {
            Difficulty::Easy => 0.9,
            Difficulty::Normal => 1.0,
            Difficulty::Hard => 1.2,
        }
    }
}
