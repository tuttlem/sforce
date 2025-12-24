use bevy::{prelude::*, time::Fixed};

use super::config::GameConfig;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_starfield)
            .add_systems(FixedUpdate, scroll_stars);
    }
}

#[derive(Component)]
struct StarLayer {
    speed: f32,
}

fn spawn_starfield(mut commands: Commands, config: Res<GameConfig>) {
    let layers = [
        (72, Color::srgb(0.4, 0.6, 1.0), 28.0, 0.45),
        (96, Color::srgb(0.7, 0.85, 1.0), 48.0, 1.0),
    ];
    let half_width = config.logical_width * 0.5;
    let half_height = config.logical_height * 0.5;

    for (index, (count, color, speed, scale)) in layers.into_iter().enumerate() {
        for star_index in 0..count {
            let seed = (index as u32) * 1_000 + star_index as u32;
            let x = -half_width + pseudo_random(seed) * config.logical_width;
            let y = -half_height + pseudo_random(seed + 17) * config.logical_height;
            let size = 2.0 + pseudo_random(seed + 33) * 2.0 * scale;

            commands.spawn((
                SpriteBundle {
                    transform: Transform::from_xyz(x, y, -10.0 + index as f32)
                        .with_scale(Vec3::splat(size)),
                    sprite: Sprite {
                        color,
                        custom_size: Some(Vec2::splat(1.0)),
                        ..default()
                    },
                    ..default()
                },
                StarLayer { speed },
            ));
        }
    }
}

fn scroll_stars(
    mut query: Query<(&StarLayer, &mut Transform)>,
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
) {
    let delta = time.delta_seconds();
    let reset_y = config.logical_height * 0.5 + 40.0;
    let bottom = -config.logical_height * 0.5 - 40.0;

    for (layer, mut transform) in &mut query {
        transform.translation.y -= layer.speed * delta;
        if transform.translation.y < bottom {
            transform.translation.y = reset_y;
        }
    }
}

fn pseudo_random(seed: u32) -> f32 {
    let mut value = seed.wrapping_mul(1664525).wrapping_add(1013904223);
    value ^= value >> 13;
    value = value.wrapping_mul(1274126177);
    (value as f32 / u32::MAX as f32).clamp(0.0, 1.0)
}
