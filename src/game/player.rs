use bevy::prelude::*;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, _app: &mut App) {
        // Player gameplay appears in later phases.
    }
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Resource, Debug, Clone, Copy)]
pub struct PlayerStats {
    pub lives: u8,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self { lives: 3 }
    }
}
