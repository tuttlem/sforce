use std::time::Duration;

use bevy::{prelude::*, time::Fixed};

use super::{
    config::GameConfig,
    player::{Player, PlayerDefense, PlayerWeaponState},
    states::AppState,
};

pub struct PowerupsPlugin;

impl Plugin for PowerupsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PowerupScheduler::default())
            .add_systems(OnEnter(AppState::Playing), reset_powerups)
            .add_systems(OnExit(AppState::Playing), cleanup_powerups)
            .add_systems(
                FixedUpdate,
                (spawn_powerup_tick, move_powerups, collect_powerups)
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Component)]
pub struct PowerUp {
    pub kind: PowerUpKind,
}

#[derive(Component)]
struct PowerUpMotion {
    speed: f32,
}

#[derive(Clone, Copy)]
pub enum PowerUpKind {
    Spread,
    Rapid,
    Shield,
}

#[derive(Resource)]
struct PowerupScheduler {
    timer: Timer,
    next_kind: usize,
    lane_index: usize,
}

impl Default for PowerupScheduler {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(8.0, TimerMode::Repeating),
            next_kind: 0,
            lane_index: 0,
        }
    }
}

fn reset_powerups(mut scheduler: ResMut<PowerupScheduler>) {
    scheduler.timer.reset();
    scheduler.timer.set_duration(Duration::from_secs_f32(8.0));
    scheduler.next_kind = 0;
    scheduler.lane_index = 0;
}

fn cleanup_powerups(mut commands: Commands, query: Query<Entity, With<PowerUp>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_powerup_tick(
    mut commands: Commands,
    mut scheduler: ResMut<PowerupScheduler>,
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
) {
    if !scheduler.timer.tick(time.delta()).just_finished() {
        return;
    }

    let lanes = [-440.0, -220.0, 0.0, 220.0, 440.0];
    let lane = lanes[scheduler.lane_index % lanes.len()];
    scheduler.lane_index += 1;

    let kind = match scheduler.next_kind % 3 {
        0 => PowerUpKind::Spread,
        1 => PowerUpKind::Rapid,
        _ => PowerUpKind::Shield,
    };
    scheduler.next_kind += 1;

    let color = match kind {
        PowerUpKind::Spread => Color::srgb(0.6, 1.0, 0.4),
        PowerUpKind::Rapid => Color::srgb(0.4, 0.8, 1.0),
        PowerUpKind::Shield => Color::srgb(1.0, 0.9, 0.4),
    };

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(lane, config.logical_height * 0.5 + 80.0, 1.0),
            sprite: Sprite {
                color,
                custom_size: Some(Vec2::new(36.0, 36.0)),
                ..default()
            },
            ..default()
        },
        PowerUp { kind },
        PowerUpMotion { speed: 120.0 },
    ));
}

fn move_powerups(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &PowerUpMotion)>,
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
) {
    let bottom = -config.logical_height * 0.5 - 60.0;
    for (entity, mut transform, motion) in &mut query {
        transform.translation.y -= motion.speed * time.delta_seconds();
        if transform.translation.y < bottom {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn collect_powerups(
    mut commands: Commands,
    powerups: Query<(Entity, &Transform, &Sprite, &PowerUp)>,
    mut player_query: Query<(&Transform, &Sprite, &mut PlayerDefense), With<Player>>,
    mut weapon_state: ResMut<PlayerWeaponState>,
) {
    let Ok((player_transform, player_sprite, mut defense)) = player_query.get_single_mut() else {
        return;
    };

    let player_half = player_sprite.custom_size.unwrap_or(Vec2::splat(32.0)) * 0.5;
    let player_center = player_transform.translation.truncate();

    for (entity, transform, sprite, powerup) in &powerups {
        let half = sprite.custom_size.unwrap_or(Vec2::splat(24.0)) * 0.5;
        let center = transform.translation.truncate();
        if (player_center.x - center.x).abs() <= (player_half.x + half.x)
            && (player_center.y - center.y).abs() <= (player_half.y + half.y)
        {
            apply_powerup(powerup.kind, &mut weapon_state, &mut defense);
            commands.entity(entity).despawn_recursive();
            break;
        }
    }
}

fn apply_powerup(
    kind: PowerUpKind,
    weapon_state: &mut PlayerWeaponState,
    defense: &mut PlayerDefense,
) {
    match kind {
        PowerUpKind::Spread => weapon_state.advance_mode(),
        PowerUpKind::Rapid => weapon_state.boost_fire_rate(),
        PowerUpKind::Shield => defense.invulnerability = defense.invulnerability.max(3.0),
    }
}
