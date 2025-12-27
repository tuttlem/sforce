use std::f32::consts::{PI, TAU};

use bevy::{log::info, prelude::*, sprite::TextureAtlas, time::Fixed};

use super::{
    audio::AudioCue,
    config::{GameConfig, GameSettings},
    enemies::{Enemy, EnemyKind, new_enemy_shot},
    player::Player,
    ship_sprites::{ShipAnimation, ShipSpriteAssets, ShipSpriteId},
    spawn::{Storyboard, WaveDirector, advance_level},
    states::AppState,
    ui::ScoreBoard,
    weapons::EnemyFireEvent,
};

pub struct BossPlugin;

impl Plugin for BossPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BossState>()
            .add_systems(OnEnter(AppState::Playing), reset_boss_state)
            .add_systems(
                FixedUpdate,
                (
                    trigger_boss_spawn,
                    boss_movement_and_attacks,
                    boss_health_tracker,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Resource)]
pub struct BossState {
    pub active: bool,
    pub entity: Option<Entity>,
    pub max_health: f32,
    pub health: f32,
    pub spawn_score: u32,
}

impl Default for BossState {
    fn default() -> Self {
        Self {
            active: false,
            entity: None,
            max_health: 0.0,
            health: 0.0,
            spawn_score: 2600,
        }
    }
}

#[derive(Component)]
struct BossControl {
    phase: BossPhase,
    direction: f32,
    elapsed: f32,
    fire_timer: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BossPhase {
    Entry,
    Second,
    Final,
}

fn reset_boss_state(mut state: ResMut<BossState>) {
    state.active = false;
    state.entity = None;
    state.max_health = 0.0;
    state.health = 0.0;
}

fn trigger_boss_spawn(
    mut commands: Commands,
    scoreboard: Res<ScoreBoard>,
    mut state: ResMut<BossState>,
    mut director: ResMut<WaveDirector>,
    config: Res<GameConfig>,
    sprites: Res<ShipSpriteAssets>,
) {
    if state.active || scoreboard.score < state.spawn_score {
        return;
    }

    let max_health = 200.0;
    let sprite_data = sprites.data(ShipSpriteId::Boss);
    let sequence = sprites.sequence(ShipSpriteId::Boss, 0);
    let entity = commands
        .spawn((
            SpriteBundle {
                texture: sprite_data.texture.clone(),
                transform: Transform::from_xyz(0.0, config.logical_height * 0.3, 6.0),
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(sprite_data.frame_size * sprite_data.scale),
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: sprite_data.layout.clone(),
                index: sequence[0],
            },
            Enemy {
                kind: EnemyKind::Boss,
                health: max_health as i32,
                score: EnemyKind::Boss.score_value(),
                damage: 1,
            },
            BossControl {
                phase: BossPhase::Entry,
                direction: 1.0,
                elapsed: 0.0,
                fire_timer: 1.0,
            },
            ShipAnimation::new(ShipSpriteId::Boss, 0, 0.12),
        ))
        .id();

    state.active = true;
    state.entity = Some(entity);
    state.max_health = max_health;
    state.health = max_health;
    director.boss_active = true;
}

fn boss_movement_and_attacks(
    mut queries: ParamSet<(
        Query<(&mut Transform, &mut BossControl, &Enemy)>,
        Query<&Transform, With<Player>>,
    )>,
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
    mut fire_writer: EventWriter<EnemyFireEvent>,
    settings: Res<GameSettings>,
    boss_state: Res<BossState>,
) {
    let player_x = queries
        .p1()
        .get_single()
        .map(|t| t.translation.x)
        .unwrap_or(0.0);

    let mut boss_query = queries.p0();
    let Ok((mut transform, mut control, enemy)) = boss_query.get_single_mut() else {
        return;
    };

    let delta = time.delta_seconds();
    control.elapsed += delta;
    control.fire_timer -= delta;

    let ratio = if boss_state.max_health > 0.0 {
        (enemy.health.max(0) as f32) / boss_state.max_health
    } else {
        1.0
    };
    if ratio < 0.35 {
        control.phase = BossPhase::Final;
    } else if ratio < 0.65 {
        control.phase = BossPhase::Second;
    }

    match control.phase {
        BossPhase::Entry => {
            let limit = config.logical_width * 0.4;
            transform.translation.x += control.direction * 160.0 * delta;
            if transform.translation.x.abs() > limit {
                control.direction *= -1.0;
                transform.translation.x = transform.translation.x.clamp(-limit, limit);
            }
            transform.translation.y =
                config.logical_height * 0.25 + (control.elapsed * 1.2).sin() * 20.0;
        }
        BossPhase::Second => {
            let amplitude = config.logical_width * 0.25;
            transform.translation.x = amplitude * (control.elapsed * 0.8).sin();
            transform.translation.y =
                config.logical_height * 0.2 + (control.elapsed * 1.6).cos() * 40.0;
        }
        BossPhase::Final => {
            let dir = (player_x - transform.translation.x).clamp(-220.0, 220.0);
            transform.translation.x += dir * delta * 0.8;
            transform.translation.y = transform
                .translation
                .y
                .clamp(50.0, config.logical_height * 0.25);
        }
    }

    if control.fire_timer <= 0.0 {
        fire_boss_pattern(
            control.phase,
            transform.translation.truncate(),
            &mut fire_writer,
            settings.difficulty.enemy_bullet_factor(),
        );
        control.fire_timer = match control.phase {
            BossPhase::Entry => 1.35,
            BossPhase::Second => 0.95,
            BossPhase::Final => 0.7,
        };
    }
}

fn fire_boss_pattern(
    phase: BossPhase,
    origin: Vec2,
    writer: &mut EventWriter<EnemyFireEvent>,
    difficulty_factor: f32,
) {
    match phase {
        BossPhase::Entry => {
            for offset in -1..=1 {
                let dir = Vec2::new(offset as f32 * 0.18, -1.0).normalize_or_zero();
                writer.send(new_enemy_shot(origin, dir * 220.0 * difficulty_factor, 1));
            }
        }
        BossPhase::Second => {
            for i in 0..3 {
                let angle = -PI / 2.0 + (i as f32 - 1.0) * 0.12;
                let dir = Vec2::new(angle.cos(), angle.sin());
                writer.send(new_enemy_shot(
                    origin + Vec2::new(0.0, -20.0),
                    dir * 260.0 * difficulty_factor,
                    1,
                ));
            }
        }
        BossPhase::Final => {
            for i in 0..6 {
                let angle = i as f32 / 6.0 * TAU;
                let dir = Vec2::new(angle.cos(), angle.sin());
                writer.send(new_enemy_shot(origin, dir * 230.0 * difficulty_factor, 1));
            }
        }
    }
}

fn boss_health_tracker(
    mut state: ResMut<BossState>,
    boss_query: Query<(&Enemy, Entity), With<BossControl>>,
    mut director: ResMut<WaveDirector>,
    storyboard: Res<Storyboard>,
    settings: Res<GameSettings>,
    mut audio: EventWriter<AudioCue>,
) {
    match boss_query.get_single() {
        Ok((enemy, entity)) => {
            state.entity = Some(entity);
            state.health = enemy.health.max(0) as f32;
        }
        Err(_) => {
            if state.active {
                state.active = false;
                state.entity = None;
                state.health = 0.0;
                state.max_health = 0.0;
                director.boss_active = false;
                state.spawn_score += 2600;
                advance_level(&mut director, &storyboard, &settings);
                info!(
                    "Boss defeated; advancing to level {}",
                    director.level_index + 1
                );
                audio.send(AudioCue::UiSelect);
            }
        }
    }
}
