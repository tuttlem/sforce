use bevy::{prelude::*, time::Fixed};

use super::{config::GameConfig, states::AppState};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BasicEnemySpawner::default())
            .add_systems(OnEnter(AppState::Playing), reset_enemy_systems)
            .add_systems(OnExit(AppState::Playing), cleanup_enemies)
            .add_systems(
                FixedUpdate,
                (spawn_basic_enemy, move_basic_enemies).run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct Enemy {
    pub health: i32,
    pub size: Vec2,
}

#[derive(Resource)]
struct BasicEnemySpawner {
    timer: Timer,
    lane_positions: [f32; 5],
    lane_index: usize,
}

impl Default for BasicEnemySpawner {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(1.8, TimerMode::Repeating),
            lane_positions: [-420.0, -180.0, 0.0, 180.0, 420.0],
            lane_index: 0,
        }
    }
}

fn reset_enemy_systems(mut spawner: ResMut<BasicEnemySpawner>) {
    spawner.timer.reset();
    spawner.lane_index = 0;
}

fn spawn_basic_enemy(
    mut commands: Commands,
    mut spawner: ResMut<BasicEnemySpawner>,
    time: Res<Time<Fixed>>,
) {
    if !spawner.timer.tick(time.delta()).just_finished() {
        return;
    }

    let x = spawner.lane_positions[spawner.lane_index % spawner.lane_positions.len()];
    spawner.lane_index = (spawner.lane_index + 1) % spawner.lane_positions.len();

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(x, 420.0, 1.0),
            sprite: Sprite {
                color: Color::srgb(1.0, 0.3, 0.4),
                custom_size: Some(Vec2::new(48.0, 48.0)),
                ..default()
            },
            ..default()
        },
        Enemy {
            health: 1,
            size: Vec2::new(48.0, 48.0),
        },
    ));
}

fn move_basic_enemies(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform), With<Enemy>>,
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
) {
    let bottom = -config.logical_height * 0.5 - 60.0;
    let speed = 160.0;

    for (entity, mut transform) in &mut query {
        transform.translation.y -= speed * time.delta_seconds();
        if transform.translation.y < bottom {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn cleanup_enemies(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
