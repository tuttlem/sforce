use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct DebugOptions {
    pub show_overlay: bool,
}

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugOptions>();
    }
}
