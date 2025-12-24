use bevy::{prelude::*, time::Fixed};

use super::{config::GameConfig, player::Player, states::AppState, weapons::EnemyFireEvent};

pub struct EnemiesPlugin;

impl Plugin for EnemiesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnEnemyEvent>()
            .add_systems(OnEnter(AppState::Playing), reset_enemies)
            .add_systems(OnExit(AppState::Playing), cleanup_enemies)
            .add_systems(
                FixedUpdate,
                (
                    spawn_enemies_from_events,
                    move_enemies,
                    enemy_fire_system,
                    cleanup_offscreen_enemies,
                )
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Component, Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnemyKind {
    Grunt,
    Sine,
    ZigZag,
    Tank,
    Chaser,
}

impl EnemyKind {
    pub fn health(self) -> i32 {
        match self {
            EnemyKind::Grunt => 1,
            EnemyKind::Sine => 2,
            EnemyKind::ZigZag => 2,
            EnemyKind::Tank => 6,
            EnemyKind::Chaser => 3,
        }
    }

    pub fn score_value(self) -> u32 {
        match self {
            EnemyKind::Grunt => 100,
            EnemyKind::Sine => 150,
            EnemyKind::ZigZag => 200,
            EnemyKind::Tank => 350,
            EnemyKind::Chaser => 250,
        }
    }

    pub fn body_size(self) -> Vec2 {
        match self {
            EnemyKind::Grunt => Vec2::new(48.0, 48.0),
            EnemyKind::Sine => Vec2::new(44.0, 44.0),
            EnemyKind::ZigZag => Vec2::new(40.0, 40.0),
            EnemyKind::Tank => Vec2::new(64.0, 72.0),
            EnemyKind::Chaser => Vec2::new(40.0, 56.0),
        }
    }

    pub fn color(self) -> Color {
        match self {
            EnemyKind::Grunt => Color::srgb(1.0, 0.35, 0.4),
            EnemyKind::Sine => Color::srgb(0.95, 0.2, 0.7),
            EnemyKind::ZigZag => Color::srgb(1.0, 0.6, 0.2),
            EnemyKind::Tank => Color::srgb(0.8, 0.25, 0.15),
            EnemyKind::Chaser => Color::srgb(1.0, 0.8, 0.2),
        }
    }
}

#[derive(Component)]
pub struct Enemy {
    pub kind: EnemyKind,
    pub health: i32,
    pub score: u32,
    pub damage: u8,
}

#[derive(Clone)]
pub enum MovementPattern {
    Straight {
        speed: f32,
    },
    Sine {
        speed: f32,
        amplitude: f32,
        frequency: f32,
        base_x: f32,
    },
    ZigZag {
        speed: f32,
        horizontal_speed: f32,
        direction: f32,
    },
    Tank {
        speed: f32,
    },
    Chaser {
        speed: f32,
        turn_rate: f32,
    },
}

#[derive(Component)]
pub struct EnemyMotion {
    pub pattern: MovementPattern,
    pub elapsed: f32,
}

#[derive(Component)]
pub struct EnemyWeapon {
    pub timer: Timer,
    pub bullet_speed: f32,
    pub pattern: FirePattern,
    pub damage: u8,
}

#[derive(Clone, Copy)]
pub enum FirePattern {
    StraightDown,
    TargetPlayer,
    Spread { count: u8, arc_deg: f32 },
}

#[derive(Event, Clone)]
pub struct SpawnEnemyEvent {
    pub kind: EnemyKind,
    pub position: Vec2,
    pub movement: MovementPattern,
}

fn reset_enemies(mut commands: Commands, query: Query<Entity, With<Enemy>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn spawn_enemies_from_events(mut commands: Commands, mut reader: EventReader<SpawnEnemyEvent>) {
    for event in reader.read() {
        let size = event.kind.body_size();
        let mut entity = commands.spawn((
            SpriteBundle {
                transform: Transform::from_xyz(event.position.x, event.position.y, 1.0),
                sprite: Sprite {
                    color: event.kind.color(),
                    custom_size: Some(size),
                    ..default()
                },
                ..default()
            },
            Enemy {
                kind: event.kind,
                health: event.kind.health(),
                score: event.kind.score_value(),
                damage: 1,
            },
            EnemyMotion {
                pattern: event.movement.clone(),
                elapsed: 0.0,
            },
        ));

        if let Some(weapon) = default_weapon(event.kind) {
            entity.insert(weapon);
        }
    }
}

fn move_enemies(
    mut query: Query<(&mut Transform, &mut EnemyMotion), Without<Player>>,
    time: Res<Time<Fixed>>,
    player: Query<&Transform, With<Player>>,
    config: Res<GameConfig>,
) {
    let delta = time.delta_seconds();
    let player_x = player.get_single().map(|t| t.translation.x).unwrap_or(0.0);
    let horizontal_bounds = config.logical_width * 0.5 - 40.0;

    for (mut transform, mut motion) in &mut query {
        motion.elapsed += delta;
        let elapsed = motion.elapsed;
        match &mut motion.pattern {
            MovementPattern::Straight { speed } => {
                transform.translation.y -= *speed * delta;
            }
            MovementPattern::Sine {
                speed,
                amplitude,
                frequency,
                base_x,
            } => {
                transform.translation.y -= *speed * delta;
                transform.translation.x = *base_x + *amplitude * f32::sin(elapsed * *frequency);
            }
            MovementPattern::ZigZag {
                speed,
                horizontal_speed,
                direction,
            } => {
                transform.translation.y -= *speed * delta;
                transform.translation.x += *horizontal_speed * *direction * delta;
                if transform.translation.x.abs() > horizontal_bounds {
                    *direction *= -1.0;
                }
            }
            MovementPattern::Tank { speed } => {
                transform.translation.y -= *speed * delta;
            }
            MovementPattern::Chaser { speed, turn_rate } => {
                transform.translation.y -= *speed * delta * 0.6;
                let dx = (player_x - transform.translation.x).clamp(-*turn_rate, *turn_rate);
                transform.translation.x += dx * delta * 60.0;
            }
        }
    }
}

fn enemy_fire_system(
    mut query: Query<(&Transform, &mut EnemyWeapon)>,
    time: Res<Time<Fixed>>,
    mut writer: EventWriter<EnemyFireEvent>,
    player: Query<&Transform, With<Player>>,
) {
    let delta = time.delta();
    let player_pos = player
        .get_single()
        .map(|t| t.translation.truncate())
        .unwrap_or(Vec2::ZERO);

    for (transform, mut weapon) in &mut query {
        if weapon.timer.tick(delta).just_finished() {
            let origin = transform.translation.truncate();
            match weapon.pattern {
                FirePattern::StraightDown => {
                    writer.send(new_enemy_shot(
                        origin,
                        Vec2::new(0.0, -weapon.bullet_speed),
                        weapon.damage,
                    ));
                }
                FirePattern::TargetPlayer => {
                    let mut direction = (player_pos - origin).normalize_or_zero();
                    if direction == Vec2::ZERO {
                        direction = Vec2::new(0.0, -1.0);
                    }
                    writer.send(new_enemy_shot(
                        origin,
                        direction * weapon.bullet_speed,
                        weapon.damage,
                    ));
                }
                FirePattern::Spread { count, arc_deg } => {
                    let count = count.max(1) as usize;
                    let half = (count - 1) as f32 / 2.0;
                    for i in 0..count {
                        let offset = i as f32 - half;
                        let angle = (-90.0 + offset * (arc_deg / half.max(1.0))).to_radians();
                        let dir = Vec2::new(angle.cos(), angle.sin());
                        writer.send(new_enemy_shot(
                            origin,
                            dir * weapon.bullet_speed,
                            weapon.damage,
                        ));
                    }
                }
            }
        }
    }
}

fn cleanup_offscreen_enemies(
    mut commands: Commands,
    query: Query<(Entity, &Transform), With<Enemy>>,
    config: Res<GameConfig>,
) {
    let bottom = -config.logical_height * 0.5 - 120.0;
    for (entity, transform) in &query {
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

fn default_weapon(kind: EnemyKind) -> Option<EnemyWeapon> {
    match kind {
        EnemyKind::Tank => Some(EnemyWeapon {
            timer: Timer::from_seconds(1.6, TimerMode::Repeating),
            bullet_speed: 220.0,
            pattern: FirePattern::Spread {
                count: 3,
                arc_deg: 30.0,
            },
            damage: 1,
        }),
        EnemyKind::Chaser => Some(EnemyWeapon {
            timer: Timer::from_seconds(1.0, TimerMode::Repeating),
            bullet_speed: 260.0,
            pattern: FirePattern::TargetPlayer,
            damage: 1,
        }),
        EnemyKind::Sine => Some(EnemyWeapon {
            timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            bullet_speed: 200.0,
            pattern: FirePattern::StraightDown,
            damage: 1,
        }),
        _ => None,
    }
}

fn new_enemy_shot(origin: Vec2, velocity: Vec2, damage: u8) -> EnemyFireEvent {
    EnemyFireEvent {
        origin,
        velocity,
        size: Vec2::new(12.0, 28.0),
        color: Color::srgb(1.0, 0.45, 0.2),
        lifetime: 3.0,
        damage,
    }
}
