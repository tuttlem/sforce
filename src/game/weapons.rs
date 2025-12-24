use bevy::{prelude::*, time::Fixed};

use super::{config::GameConfig, states::AppState};

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerFireEvent>()
            .add_systems(OnExit(AppState::Playing), cleanup_projectiles)
            .add_systems(
                FixedUpdate,
                (
                    spawn_player_projectiles,
                    advance_projectiles,
                    expire_projectiles,
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct PlayerFireEvent {
    pub origin: Vec2,
}

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub lifetime: f32,
}

fn spawn_player_projectiles(mut commands: Commands, mut reader: EventReader<PlayerFireEvent>) {
    let speed = 520.0;
    for event in reader.read() {
        commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(event.origin.x, event.origin.y + 40.0, 1.0),
                sprite: Sprite {
                    color: Color::srgb(1.0, 0.9, 0.2),
                    custom_size: Some(Vec2::new(12.0, 24.0)),
                    ..default()
                },
                ..default()
            },
            Projectile {
                velocity: Vec2::new(0.0, speed),
                lifetime: 1.6,
            },
        ));
    }
}

fn advance_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Projectile)>,
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
) {
    let half_h = config.logical_height * 0.5 + 100.0;
    for (entity, mut transform, projectile) in &mut query {
        transform.translation += (projectile.velocity * time.delta_seconds()).extend(0.0);
        if transform.translation.y > half_h {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn expire_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Projectile)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut projectile) in &mut query {
        projectile.lifetime -= time.delta_seconds();
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn cleanup_projectiles(mut commands: Commands, query: Query<Entity, With<Projectile>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
