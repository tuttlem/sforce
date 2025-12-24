use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};

use super::spawn::WaveDirector;

#[derive(Resource, Default)]
pub struct DebugOptions {
    pub show_overlay: bool,
}

#[derive(Component)]
struct DebugOverlayText;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugOptions>()
            .add_plugins(FrameTimeDiagnosticsPlugin)
            .add_systems(Startup, spawn_debug_overlay)
            .add_systems(
                Update,
                (
                    toggle_debug_overlay,
                    update_debug_overlay_visibility,
                    refresh_debug_overlay,
                ),
            );
    }
}

fn spawn_debug_overlay(mut commands: Commands) {
    let mut bundle = TextBundle::from_section(
        "",
        TextStyle {
            font_size: 16.0,
            color: Color::WHITE,
            ..default()
        },
    )
    .with_style(Style {
        position_type: PositionType::Absolute,
        top: Val::Px(8.0),
        left: Val::Px(8.0),
        ..default()
    });
    bundle.visibility = Visibility::Hidden;
    commands.spawn((bundle, DebugOverlayText));
}

fn toggle_debug_overlay(keys: Res<ButtonInput<KeyCode>>, mut options: ResMut<DebugOptions>) {
    if keys.just_pressed(KeyCode::F3) {
        options.show_overlay = !options.show_overlay;
    }
}

fn update_debug_overlay_visibility(
    options: Res<DebugOptions>,
    mut query: Query<&mut Visibility, With<DebugOverlayText>>,
) {
    if let Ok(mut visibility) = query.get_single_mut() {
        *visibility = if options.show_overlay {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

fn refresh_debug_overlay(
    options: Res<DebugOptions>,
    diagnostics: Res<DiagnosticsStore>,
    wave_director: Option<Res<WaveDirector>>,
    entity_query: Query<Entity>,
    mut query: Query<&mut Text, With<DebugOverlayText>>,
) {
    if !options.show_overlay {
        return;
    }

    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|diag| diag.smoothed())
        .unwrap_or(0.0);
    let entity_count = entity_query.iter().len();
    let wave = wave_director.map(|w| w.wave_index).unwrap_or_default();

    if let Ok(mut text) = query.get_single_mut() {
        text.sections[0].value = format!(
            "FPS: {:>5.1}\nEntities: {}\nWave: {}",
            fps, entity_count, wave
        );
    }
}
