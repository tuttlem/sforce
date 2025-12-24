use bevy::prelude::*;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, _app: &mut App) {
        // Later phases will register wave spawners here.
    }
}
