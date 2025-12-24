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
            .insert_resource(Time::<Fixed>::from_seconds(1.0 / 120.0));
    }
}
