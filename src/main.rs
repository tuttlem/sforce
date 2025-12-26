mod game;
mod util;

use bevy::{
    prelude::*,
    window::{PresentMode, PrimaryWindow, Window, WindowMode, WindowPlugin, WindowResolution},
};
use game::GamePlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.14)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "S-Force".into(),
                resolution: WindowResolution::new(1280.0, 720.0),
                present_mode: PresentMode::AutoVsync,
                resizable: false,
                ..default()
            }),
            ..default()
        }))
        .add_systems(Update, toggle_fullscreen_shortcut)
        .add_plugins(GamePlugin)
        .run();
}

fn toggle_fullscreen_shortcut(
    keys: Res<ButtonInput<KeyCode>>,
    mut windows: Query<&mut Window, With<PrimaryWindow>>,
) {
    let ctrl_pressed = keys.any_pressed([KeyCode::ControlLeft, KeyCode::ControlRight]);
    if !ctrl_pressed || !keys.just_pressed(KeyCode::Enter) {
        return;
    }

    if let Ok(mut window) = windows.get_single_mut() {
        window.mode = if window.mode == WindowMode::Windowed {
            WindowMode::BorderlessFullscreen
        } else {
            WindowMode::Windowed
        };
    }
}
