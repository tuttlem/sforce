use std::time::Duration;

use bevy::{prelude::*, time::Fixed};

use super::{
    config::{Difficulty, GameSettings},
    enemies::{EnemyKind, MovementPattern, SpawnEnemyEvent},
    states::AppState,
};

const BASE_INTERVAL: f32 = 3.2;

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WaveDirector::default())
            .add_systems(OnEnter(AppState::Playing), reset_waves)
            .add_systems(OnExit(AppState::Playing), clear_waves)
            .add_systems(FixedUpdate, drive_waves.run_if(in_state(AppState::Playing)));
    }
}

#[derive(Resource)]
pub struct WaveDirector {
    pub timer: Timer,
    pub wave_index: u32,
    pub difficulty: f32,
    pub boss_active: bool,
}

impl Default for WaveDirector {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(BASE_INTERVAL, TimerMode::Repeating),
            wave_index: 0,
            difficulty: 1.0,
            boss_active: false,
        }
    }
}

fn reset_waves(mut director: ResMut<WaveDirector>, settings: Res<GameSettings>) {
    director.timer.reset();
    director
        .timer
        .set_duration(Duration::from_secs_f32(base_interval(settings.difficulty)));
    director.wave_index = 0;
    director.difficulty = settings.difficulty.enemy_health_factor();
    director.boss_active = false;
}

fn clear_waves(mut director: ResMut<WaveDirector>) {
    director.timer.reset();
}

fn drive_waves(
    mut director: ResMut<WaveDirector>,
    time: Res<Time<Fixed>>,
    mut writer: EventWriter<SpawnEnemyEvent>,
    settings: Res<GameSettings>,
) {
    if director.boss_active {
        return;
    }
    if !director.timer.tick(time.delta()).just_finished() {
        return;
    }

    spawn_wave(
        director.wave_index,
        director.difficulty,
        settings.difficulty,
        &mut writer,
    );
    director.wave_index += 1;
    director.difficulty += 0.05;

    if director.wave_index % 5 == 0 {
        let target = director.timer.duration().as_secs_f32() - 0.15;
        let min_interval = base_interval(settings.difficulty) * 0.6;
        let new_duration = target.max(min_interval);
        director
            .timer
            .set_duration(Duration::from_secs_f32(new_duration));
    }
}

fn spawn_wave(
    wave_index: u32,
    difficulty: f32,
    setting: Difficulty,
    writer: &mut EventWriter<SpawnEnemyEvent>,
) {
    let lanes = [-360.0, -180.0, 0.0, 180.0, 360.0];
    let core_lanes: Vec<f32> = lanes.iter().copied().step_by(2).collect();
    let top = 420.0;
    let difficulty_scale = difficulty * setting.enemy_health_factor();

    match wave_index % 5 {
        0 => {
            for lane in &core_lanes {
                writer.send(spawn_enemy(
                    EnemyKind::Grunt,
                    Vec2::new(*lane, top),
                    MovementPattern::Straight {
                        speed: 160.0 * difficulty_scale,
                    },
                ));
            }
        }
        1 => {
            for lane in &core_lanes {
                writer.send(spawn_enemy(
                    EnemyKind::Sine,
                    Vec2::new(*lane, top + 40.0),
                    MovementPattern::Sine {
                        speed: 130.0,
                        amplitude: 140.0,
                        frequency: 1.4 + difficulty_scale * 0.15,
                        base_x: *lane,
                    },
                ));
            }
        }
        2 => {
            for lane in &core_lanes {
                writer.send(spawn_enemy(
                    EnemyKind::ZigZag,
                    Vec2::new(*lane, top + 60.0),
                    MovementPattern::ZigZag {
                        speed: 150.0,
                        horizontal_speed: 180.0,
                        direction: if *lane >= 0.0 { -1.0 } else { 1.0 },
                    },
                ));
            }
        }
        3 => {
            writer.send(spawn_enemy(
                EnemyKind::Tank,
                Vec2::new(-200.0, top + 100.0),
                MovementPattern::Tank {
                    speed: 90.0 * (0.8 + difficulty_scale * 0.1),
                },
            ));
            writer.send(spawn_enemy(
                EnemyKind::Tank,
                Vec2::new(200.0, top + 100.0),
                MovementPattern::Tank {
                    speed: 90.0 * (0.8 + difficulty_scale * 0.1),
                },
            ));
        }
        _ => {
            for lane in &core_lanes {
                writer.send(spawn_enemy(
                    EnemyKind::Chaser,
                    Vec2::new(*lane * 0.5, top + 20.0),
                    MovementPattern::Chaser {
                        speed: 180.0,
                        turn_rate: 120.0 + difficulty_scale * 20.0,
                    },
                ));
            }
        }
    };
}

fn spawn_enemy(kind: EnemyKind, position: Vec2, movement: MovementPattern) -> SpawnEnemyEvent {
    SpawnEnemyEvent {
        kind,
        position,
        movement,
    }
}

fn base_interval(difficulty: Difficulty) -> f32 {
    BASE_INTERVAL * difficulty.spawn_interval_factor()
}
