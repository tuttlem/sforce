use bevy::{prelude::*, render::camera::ScalingMode};

use super::config::GameConfig;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_main_camera);
    }
}

fn spawn_main_camera(mut commands: Commands, config: Res<GameConfig>) {
    let mut camera = Camera2dBundle::default();
    camera.projection.scaling_mode = ScalingMode::FixedVertical(config.logical_height);
    commands.spawn(camera);
}
