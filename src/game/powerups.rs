use bevy::{prelude::*, sprite::TextureAtlas, time::Fixed};

use super::{
    audio::AudioCue,
    config::GameConfig,
    effects::ExplosionAssets,
    player::{Player, PlayerDefense, PlayerStats, PlayerWeaponState},
    states::AppState,
};

pub struct PowerupsPlugin;

impl Plugin for PowerupsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnPowerUpEvent>()
            .add_systems(OnExit(AppState::Playing), cleanup_powerups)
            .add_systems(
                FixedUpdate,
                (spawn_powerups_from_events, move_powerups, collect_powerups)
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, animate_powerups.run_if(in_state(AppState::Playing)));
    }
}

const INVINCIBLE_POWERUP_DURATION: f32 = 10.0;

#[derive(Component)]
pub struct PowerUp {
    pub kind: PowerUpKind,
}

#[derive(Component, Clone, Copy)]
pub struct DropsPowerUp {
    pub kind: PowerUpKind,
}

#[derive(Component)]
struct PowerUpMotion {
    speed: f32,
}

#[derive(Component)]
struct PowerUpAnimation {
    sequence_index: usize,
    frame: usize,
    timer: Timer,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PowerUpKind {
    Spread,
    Rapid,
    Shield,
    Health,
    Invincibility,
}

fn cleanup_powerups(mut commands: Commands, query: Query<Entity, With<PowerUp>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct SpawnPowerUpEvent {
    pub position: Vec2,
    pub kind: PowerUpKind,
}

fn spawn_powerups_from_events(
    mut commands: Commands,
    mut reader: EventReader<SpawnPowerUpEvent>,
    effects: Res<ExplosionAssets>,
) {
    for event in reader.read() {
        let (color, sequence_index) = powerup_visuals(event.kind);
        let frames = &effects.powerup_sequences[sequence_index % effects.powerup_sequences.len()];
        commands.spawn((
            SpriteBundle {
                texture: effects.texture.clone(),
                transform: Transform::from_xyz(event.position.x, event.position.y, 1.0),
                sprite: Sprite {
                    color,
                    custom_size: Some(Vec2::new(36.0, 36.0)),
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: effects.layout.clone(),
                index: frames[0],
            },
            PowerUp { kind: event.kind },
            PowerUpMotion { speed: 120.0 },
            PowerUpAnimation {
                sequence_index,
                frame: 0,
                timer: Timer::from_seconds(0.08, TimerMode::Repeating),
            },
        ));
    }
}

fn powerup_visuals(kind: PowerUpKind) -> (Color, usize) {
    match kind {
        PowerUpKind::Spread => (Color::srgb(0.7, 0.4, 1.0), 0),
        PowerUpKind::Rapid => (Color::srgb(0.4, 0.8, 1.0), 1),
        PowerUpKind::Shield => (Color::srgb(0.5, 1.0, 0.6), 2),
        PowerUpKind::Health => (Color::srgb(1.0, 0.5, 0.5), 0),
        PowerUpKind::Invincibility => (Color::srgb(1.0, 0.9, 0.4), 1),
    }
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
    mut stats: ResMut<PlayerStats>,
    mut audio_events: EventWriter<AudioCue>,
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
            apply_powerup(
                powerup.kind,
                &mut weapon_state,
                &mut defense,
                &mut stats,
                &mut audio_events,
            );
            commands.entity(entity).despawn_recursive();
            break;
        }
    }
}

fn animate_powerups(
    time: Res<Time>,
    assets: Res<ExplosionAssets>,
    mut query: Query<(&mut PowerUpAnimation, &mut TextureAtlas)>,
) {
    for (mut anim, mut atlas) in &mut query {
        if anim.timer.tick(time.delta()).just_finished() {
            let frames =
                &assets.powerup_sequences[anim.sequence_index % assets.powerup_sequences.len()];
            anim.frame = (anim.frame + 1) % frames.len();
            atlas.index = frames[anim.frame];
        }
    }
}

fn apply_powerup(
    kind: PowerUpKind,
    weapon_state: &mut PlayerWeaponState,
    defense: &mut PlayerDefense,
    stats: &mut PlayerStats,
    audio_events: &mut EventWriter<AudioCue>,
) {
    match kind {
        PowerUpKind::Spread => weapon_state.advance_mode(),
        PowerUpKind::Rapid => weapon_state.boost_fire_rate(),
        PowerUpKind::Shield => defense.invulnerability = defense.invulnerability.max(3.0),
        PowerUpKind::Health => {
            stats.health = stats.health.saturating_add(1).min(stats.max_health);
        }
        PowerUpKind::Invincibility => {
            defense.invulnerability = defense.invulnerability.max(INVINCIBLE_POWERUP_DURATION);
        }
    }
    audio_events.send(AudioCue::Pickup);
}
