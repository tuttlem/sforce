use std::{fmt, fs, time::Duration};

use bevy::{log::warn, prelude::*, time::Fixed};
use serde::Deserialize;
use serde::de::{self, Deserializer};

use super::{
    config::GameSettings,
    enemies::{EnemyKind, MovementPattern, SpawnEnemyEvent},
    powerups::PowerUpKind,
    states::AppState,
};

const BASE_INTERVAL: f32 = 3.6;
const TOP_Y: f32 = 420.0;
const STORYBOARD_PATH: &str = "assets/storyboard.json";
const CORE_LANES: [f32; 3] = [-360.0, 0.0, 360.0];
const CHASER_LANES: [f32; 3] = [-180.0, 0.0, 180.0];

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        let storyboard = Storyboard::from_file(STORYBOARD_PATH).unwrap_or_else(|err| {
            warn!(
                "Failed to load storyboard from {}: {}. Using built-in defaults.",
                STORYBOARD_PATH, err
            );
            Storyboard::default()
        });

        app.insert_resource(storyboard)
            .insert_resource(WaveDirector::default())
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
    pub level_index: usize,
    pub pending_level: Option<usize>,
}

impl Default for WaveDirector {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(BASE_INTERVAL, TimerMode::Once),
            wave_index: 0,
            difficulty: 1.0,
            boss_active: false,
            level_index: 0,
            pending_level: None,
        }
    }
}

#[derive(Resource)]
pub struct Storyboard {
    levels: Vec<Level>,
}

impl Storyboard {
    fn from_file(path: &str) -> Result<Self, StoryboardLoadError> {
        let contents = fs::read_to_string(path)?;
        let parsed: StoryboardFile = serde_json::from_str(&contents)?;
        Ok(Self {
            levels: parsed.levels.into_iter().map(Level::from).collect(),
        })
    }

    fn level(&self, index: usize) -> Option<&Level> {
        self.levels.get(index)
    }

    fn first_delay(&self, index: usize) -> Option<f32> {
        self.level(index)
            .and_then(|level| level.waves.first())
            .map(|wave| wave.delay_seconds)
    }

    fn level_count(&self) -> usize {
        self.levels.len()
    }
}

impl Default for Storyboard {
    fn default() -> Self {
        let default_waves = vec![
            lane_wave(
                BASE_INTERVAL,
                EnemyKind::Grunt,
                &CORE_LANES,
                0.0,
                MovementConfig::Straight {
                    speed: Some(160.0),
                    scale_with_difficulty: Some(true),
                },
                Some(PowerUpKind::Rapid),
                Some(1),
            ),
            lane_wave(
                BASE_INTERVAL,
                EnemyKind::Sine,
                &CORE_LANES,
                40.0,
                MovementConfig::Sine {
                    speed: Some(130.0),
                    amplitude: Some(140.0),
                    frequency: Some(1.4),
                    frequency_gain: Some(0.15),
                    base_x_offset: None,
                },
                Some(PowerUpKind::Shield),
                Some(1),
            ),
            lane_wave(
                BASE_INTERVAL,
                EnemyKind::ZigZag,
                &CORE_LANES,
                60.0,
                MovementConfig::ZigZag {
                    speed: Some(150.0),
                    horizontal_speed: Some(180.0),
                    direction: None,
                },
                Some(PowerUpKind::Spread),
                Some(1),
            ),
            fixed_wave(
                BASE_INTERVAL,
                vec![
                    FixedEnemyConfig {
                        enemy: EnemyKind::Tank,
                        position: SpawnPoint::new(-200.0, TOP_Y + 100.0),
                        movement: MovementConfig::Tank {
                            speed: Some(90.0),
                            base_factor: Some(0.8),
                            difficulty_factor: Some(0.1),
                        },
                        powerup: Some(PowerUpKind::Health),
                    },
                    FixedEnemyConfig {
                        enemy: EnemyKind::Tank,
                        position: SpawnPoint::new(200.0, TOP_Y + 100.0),
                        movement: MovementConfig::Tank {
                            speed: Some(90.0),
                            base_factor: Some(0.8),
                            difficulty_factor: Some(0.1),
                        },
                        powerup: None,
                    },
                ],
            ),
            lane_wave(
                BASE_INTERVAL,
                EnemyKind::Chaser,
                &CHASER_LANES,
                20.0,
                MovementConfig::Chaser {
                    speed: Some(180.0),
                    turn_rate: Some(120.0),
                    turn_rate_scale: Some(20.0),
                },
                None,
                Some(1),
            ),
            lane_wave(
                BASE_INTERVAL,
                EnemyKind::Grunt,
                &CORE_LANES,
                0.0,
                MovementConfig::Straight {
                    speed: Some(165.0),
                    scale_with_difficulty: Some(true),
                },
                Some(PowerUpKind::Invincibility),
                Some(1),
            ),
        ];

        Self {
            levels: vec![Level {
                name: "Default".to_string(),
                waves: default_waves,
            }],
        }
    }
}

#[derive(Debug)]
enum StoryboardLoadError {
    Io(std::io::Error),
    Parse(serde_json::Error),
}

impl fmt::Display for StoryboardLoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StoryboardLoadError::Io(err) => write!(f, "I/O error: {}", err),
            StoryboardLoadError::Parse(err) => write!(f, "parse error: {}", err),
        }
    }
}

impl std::error::Error for StoryboardLoadError {}

impl From<std::io::Error> for StoryboardLoadError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<serde_json::Error> for StoryboardLoadError {
    fn from(value: serde_json::Error) -> Self {
        Self::Parse(value)
    }
}

#[derive(Deserialize)]
struct StoryboardFile {
    levels: Vec<LevelFile>,
}

#[derive(Deserialize)]
struct LevelFile {
    name: String,
    waves: Vec<WaveDefinition>,
}

struct Level {
    #[allow(dead_code)]
    name: String,
    waves: Vec<WaveDefinition>,
}

impl From<LevelFile> for Level {
    fn from(value: LevelFile) -> Self {
        Self {
            name: value.name,
            waves: value.waves,
        }
    }
}

#[derive(Deserialize, Clone)]
struct WaveDefinition {
    #[serde(default = "default_wave_delay")]
    delay_seconds: f32,
    #[serde(flatten)]
    pattern: WavePattern,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "pattern", rename_all = "snake_case")]
enum WavePattern {
    Lane(LaneWaveConfig),
    Fixed { enemies: Vec<FixedEnemyConfig> },
}

#[derive(Deserialize, Clone)]
struct LaneWaveConfig {
    enemy: EnemyKind,
    lanes: Vec<f32>,
    #[serde(default)]
    y_offset: f32,
    movement: MovementConfig,
    powerup: Option<PowerUpKind>,
    powerup_lane_index: Option<usize>,
}

#[derive(Deserialize, Clone)]
struct FixedEnemyConfig {
    enemy: EnemyKind,
    position: SpawnPoint,
    movement: MovementConfig,
    powerup: Option<PowerUpKind>,
}

#[derive(Clone, Copy, Deserialize)]
struct SpawnPoint {
    x: f32,
    y: f32,
}

impl SpawnPoint {
    const fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }

    fn to_vec(self) -> Vec2 {
        Vec2::new(self.x, self.y)
    }

    fn x(self) -> f32 {
        self.x
    }
}

#[derive(Clone, Copy, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum MovementConfig {
    Straight {
        speed: Option<f32>,
        scale_with_difficulty: Option<bool>,
    },
    Sine {
        speed: Option<f32>,
        amplitude: Option<f32>,
        frequency: Option<f32>,
        frequency_gain: Option<f32>,
        base_x_offset: Option<f32>,
    },
    ZigZag {
        speed: Option<f32>,
        horizontal_speed: Option<f32>,
        direction: Option<f32>,
    },
    Tank {
        speed: Option<f32>,
        base_factor: Option<f32>,
        difficulty_factor: Option<f32>,
    },
    Chaser {
        speed: Option<f32>,
        turn_rate: Option<f32>,
        turn_rate_scale: Option<f32>,
    },
}

impl MovementConfig {
    fn to_pattern(self, difficulty_scale: f32, lane_x: Option<f32>) -> MovementPattern {
        match self {
            MovementConfig::Straight {
                speed,
                scale_with_difficulty,
            } => {
                let mut final_speed = speed.unwrap_or(160.0);
                if scale_with_difficulty.unwrap_or(true) {
                    final_speed *= difficulty_scale;
                }
                MovementPattern::Straight { speed: final_speed }
            }
            MovementConfig::Sine {
                speed,
                amplitude,
                frequency,
                frequency_gain,
                base_x_offset,
            } => MovementPattern::Sine {
                speed: speed.unwrap_or(130.0),
                amplitude: amplitude.unwrap_or(140.0),
                frequency: frequency.unwrap_or(1.4)
                    + difficulty_scale * frequency_gain.unwrap_or(0.15),
                base_x: lane_x.unwrap_or(0.0) + base_x_offset.unwrap_or(0.0),
            },
            MovementConfig::ZigZag {
                speed,
                horizontal_speed,
                direction,
            } => MovementPattern::ZigZag {
                speed: speed.unwrap_or(150.0),
                horizontal_speed: horizontal_speed.unwrap_or(180.0),
                direction: direction.unwrap_or_else(|| {
                    if lane_x.unwrap_or(0.0) >= 0.0 {
                        -1.0
                    } else {
                        1.0
                    }
                }),
            },
            MovementConfig::Tank {
                speed,
                base_factor,
                difficulty_factor,
            } => {
                let base = base_factor.unwrap_or(0.8);
                let factor = difficulty_factor.unwrap_or(0.1);
                MovementPattern::Tank {
                    speed: speed.unwrap_or(90.0) * (base + difficulty_scale * factor),
                }
            }
            MovementConfig::Chaser {
                speed,
                turn_rate,
                turn_rate_scale,
            } => MovementPattern::Chaser {
                speed: speed.unwrap_or(180.0),
                turn_rate: turn_rate.unwrap_or(120.0)
                    + difficulty_scale * turn_rate_scale.unwrap_or(20.0),
            },
        }
    }
}

fn default_wave_delay() -> f32 {
    BASE_INTERVAL
}

fn lane_wave(
    delay_seconds: f32,
    enemy: EnemyKind,
    lanes: &[f32],
    y_offset: f32,
    movement: MovementConfig,
    powerup: Option<PowerUpKind>,
    powerup_lane_index: Option<usize>,
) -> WaveDefinition {
    WaveDefinition {
        delay_seconds,
        pattern: WavePattern::Lane(LaneWaveConfig {
            enemy,
            lanes: lanes.to_vec(),
            y_offset,
            movement,
            powerup,
            powerup_lane_index,
        }),
    }
}

fn fixed_wave(delay_seconds: f32, enemies: Vec<FixedEnemyConfig>) -> WaveDefinition {
    WaveDefinition {
        delay_seconds,
        pattern: WavePattern::Fixed { enemies },
    }
}

fn set_timer_for_next_wave(
    director: &mut WaveDirector,
    storyboard: &Storyboard,
    settings: &GameSettings,
) {
    let delay = storyboard
        .level(director.level_index)
        .and_then(|level| level.waves.get(director.wave_index as usize))
        .map(|wave| wave.delay_seconds)
        .or_else(|| storyboard.first_delay(director.level_index))
        .unwrap_or(BASE_INTERVAL);
    let scaled = delay * settings.difficulty.spawn_interval_factor();
    director.timer.set_duration(Duration::from_secs_f32(scaled));
    director.timer.reset();
}

pub fn advance_level(
    director: &mut WaveDirector,
    storyboard: &Storyboard,
    settings: &GameSettings,
) {
    let level_count = storyboard.level_count();
    if level_count == 0 {
        return;
    }
    let next_index = director
        .pending_level
        .unwrap_or((director.level_index + 1) % level_count);
    director.level_index = next_index;
    director.wave_index = 0;
    director.difficulty = settings.difficulty.enemy_health_factor();
    director.pending_level = None;
    set_timer_for_next_wave(director, storyboard, settings);
}

fn reset_waves(
    mut director: ResMut<WaveDirector>,
    settings: Res<GameSettings>,
    storyboard: Res<Storyboard>,
) {
    director.timer.reset();
    director.wave_index = 0;
    director.difficulty = settings.difficulty.enemy_health_factor();
    director.boss_active = false;
    director.level_index = 0;
    director.pending_level = None;
    set_timer_for_next_wave(&mut director, &storyboard, &settings);
}

fn clear_waves(mut director: ResMut<WaveDirector>) {
    director.timer.reset();
}

fn drive_waves(
    mut director: ResMut<WaveDirector>,
    time: Res<Time<Fixed>>,
    mut writer: EventWriter<SpawnEnemyEvent>,
    settings: Res<GameSettings>,
    storyboard: Res<Storyboard>,
) {
    if director.boss_active {
        return;
    }

    let Some(level) = storyboard.level(director.level_index) else {
        return;
    };
    if level.waves.is_empty() {
        return;
    }

    if !director.timer.tick(time.delta()).just_finished() {
        return;
    }

    let wave_count = level.waves.len();
    if wave_count == 0 {
        return;
    }

    let current_index = director.wave_index as usize % wave_count;

    let difficulty_scale = director.difficulty * settings.difficulty.enemy_health_factor();
    spawn_wave_from_definition(&level.waves[current_index], difficulty_scale, &mut writer);

    director.wave_index = (director.wave_index + 1) % wave_count as u32;
    director.difficulty += 0.05;

    if director.wave_index == 0 {
        if director.pending_level.is_none() {
            let level_count = storyboard.level_count();
            if level_count > 0 {
                director.pending_level = Some((director.level_index + 1) % level_count);
            }
        }
    }

    set_timer_for_next_wave(&mut director, &storyboard, &settings);
}

fn spawn_wave_from_definition(
    wave: &WaveDefinition,
    difficulty_scale: f32,
    writer: &mut EventWriter<SpawnEnemyEvent>,
) {
    match &wave.pattern {
        WavePattern::Lane(config) => {
            spawn_lane_wave(config, difficulty_scale, writer);
        }
        WavePattern::Fixed { enemies } => {
            spawn_fixed_wave(enemies, difficulty_scale, writer);
        }
    }
}

fn spawn_lane_wave(
    config: &LaneWaveConfig,
    difficulty_scale: f32,
    writer: &mut EventWriter<SpawnEnemyEvent>,
) {
    for (index, lane_x) in config.lanes.iter().enumerate() {
        let position = Vec2::new(*lane_x, TOP_Y + config.y_offset);
        let movement = config.movement.to_pattern(difficulty_scale, Some(*lane_x));
        let drop = if config.powerup_lane_index == Some(index) {
            config.powerup
        } else {
            None
        };
        writer.send(spawn_enemy(config.enemy, position, movement, drop));
    }
}

fn spawn_fixed_wave(
    enemies: &[FixedEnemyConfig],
    difficulty_scale: f32,
    writer: &mut EventWriter<SpawnEnemyEvent>,
) {
    for enemy in enemies {
        let movement = enemy
            .movement
            .to_pattern(difficulty_scale, Some(enemy.position.x()));
        writer.send(spawn_enemy(
            enemy.enemy,
            enemy.position.to_vec(),
            movement,
            enemy.powerup,
        ));
    }
}

fn spawn_enemy(
    kind: EnemyKind,
    position: Vec2,
    movement: MovementPattern,
    powerup: Option<PowerUpKind>,
) -> SpawnEnemyEvent {
    SpawnEnemyEvent {
        kind,
        position,
        movement,
        powerup,
    }
}

impl<'de> Deserialize<'de> for EnemyKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let normalized = value.replace(['-', '_', ' '], "").to_lowercase();
        match normalized.as_str() {
            "grunt" => Ok(EnemyKind::Grunt),
            "sine" => Ok(EnemyKind::Sine),
            "zigzag" => Ok(EnemyKind::ZigZag),
            "tank" => Ok(EnemyKind::Tank),
            "chaser" => Ok(EnemyKind::Chaser),
            "boss" => Ok(EnemyKind::Boss),
            _ => Err(de::Error::unknown_variant(
                &value,
                &["grunt", "sine", "zig_zag", "tank", "chaser", "boss"],
            )),
        }
    }
}

impl<'de> Deserialize<'de> for PowerUpKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        let normalized = value.replace(['-', '_', ' '], "").to_lowercase();
        match normalized.as_str() {
            "spread" => Ok(PowerUpKind::Spread),
            "rapid" => Ok(PowerUpKind::Rapid),
            "shield" => Ok(PowerUpKind::Shield),
            "health" => Ok(PowerUpKind::Health),
            "invincibility" | "invincible" => Ok(PowerUpKind::Invincibility),
            _ => Err(de::Error::unknown_variant(
                &value,
                &["spread", "rapid", "shield", "health", "invincibility"],
            )),
        }
    }
}
