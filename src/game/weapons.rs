use bevy::{prelude::*, sprite::TextureAtlas, time::Fixed};

use super::{config::GameConfig, effects::ExplosionAssets, states::AppState};

pub struct WeaponsPlugin;

impl Plugin for WeaponsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<PlayerFireEvent>()
            .add_event::<EnemyFireEvent>()
            .add_systems(OnExit(AppState::Playing), cleanup_projectiles)
            .add_systems(
                FixedUpdate,
                (
                    (
                        spawn_player_projectiles,
                        advance_player_projectiles,
                        expire_player_projectiles,
                    )
                        .chain(),
                    (
                        spawn_enemy_projectiles,
                        advance_enemy_projectiles,
                        expire_enemy_projectiles,
                    )
                        .chain(),
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                animate_projectile_sprites.run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct PlayerFireEvent {
    pub origin: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
    pub lifetime: f32,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct EnemyFireEvent {
    pub origin: Vec2,
    pub velocity: Vec2,
    pub size: Vec2,
    pub color: Color,
    pub lifetime: f32,
    pub damage: u8,
}

#[derive(Component)]
pub struct Projectile {
    pub velocity: Vec2,
    pub lifetime: f32,
}

#[derive(Component)]
pub struct EnemyProjectile {
    pub velocity: Vec2,
    pub lifetime: f32,
    pub damage: u8,
}

#[derive(Component)]
struct ProjectileAnimation {
    frames: Vec<usize>,
    frame: usize,
    timer: Timer,
}

impl ProjectileAnimation {
    fn new(frames: &[usize], frame_time: f32) -> Self {
        Self {
            frames: frames.to_vec(),
            frame: 0,
            timer: Timer::from_seconds(frame_time, TimerMode::Repeating),
        }
    }
}

fn spawn_player_projectiles(
    mut commands: Commands,
    mut reader: EventReader<PlayerFireEvent>,
    assets: Res<ExplosionAssets>,
) {
    for event in reader.read() {
        commands.spawn((
            SpriteBundle {
                texture: assets.texture.clone(),
                transform: Transform::from_xyz(event.origin.x, event.origin.y, 1.0),
                sprite: Sprite {
                    color: Color::WHITE,
                    custom_size: Some(event.size * 1.6),
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: assets.layout.clone(),
                index: assets.bullet_sequence[0],
            },
            Projectile {
                velocity: event.velocity,
                lifetime: event.lifetime,
            },
            ProjectileAnimation::new(&assets.bullet_sequence, 0.04),
        ));
    }
}

fn advance_player_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Projectile)>,
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
) {
    let top = config.logical_height * 0.5 + 100.0;
    for (entity, mut transform, projectile) in &mut query {
        transform.translation += (projectile.velocity * time.delta_seconds()).extend(0.0);
        if transform.translation.y > top {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn expire_player_projectiles(
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

fn spawn_enemy_projectiles(
    mut commands: Commands,
    mut reader: EventReader<EnemyFireEvent>,
    assets: Res<ExplosionAssets>,
) {
    for event in reader.read() {
        commands.spawn((
            SpriteBundle {
                texture: assets.texture.clone(),
                transform: Transform::from_xyz(event.origin.x, event.origin.y, 1.0),
                sprite: Sprite {
                    color: event.color,
                    custom_size: Some(event.size * 1.6),
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: assets.layout.clone(),
                index: assets.bullet_sequence[0],
            },
            EnemyProjectile {
                velocity: event.velocity,
                lifetime: event.lifetime,
                damage: event.damage,
            },
            ProjectileAnimation::new(&assets.bullet_sequence, 0.05),
        ));
    }
}

fn advance_enemy_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &EnemyProjectile)>,
    time: Res<Time<Fixed>>,
    config: Res<GameConfig>,
) {
    let bottom = -config.logical_height * 0.5 - 120.0;
    for (entity, mut transform, projectile) in &mut query {
        transform.translation += (projectile.velocity * time.delta_seconds()).extend(0.0);
        if transform.translation.y < bottom {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn expire_enemy_projectiles(
    mut commands: Commands,
    mut query: Query<(Entity, &mut EnemyProjectile)>,
    time: Res<Time<Fixed>>,
) {
    for (entity, mut projectile) in &mut query {
        projectile.lifetime -= time.delta_seconds();
        if projectile.lifetime <= 0.0 {
            commands.entity(entity).despawn_recursive();
        }
    }
}

fn cleanup_projectiles(
    mut commands: Commands,
    player_query: Query<Entity, With<Projectile>>,
    enemy_query: Query<Entity, With<EnemyProjectile>>,
) {
    for entity in &player_query {
        commands.entity(entity).despawn_recursive();
    }
    for entity in &enemy_query {
        commands.entity(entity).despawn_recursive();
    }
}

fn animate_projectile_sprites(
    time: Res<Time>,
    mut query: Query<(&mut ProjectileAnimation, &mut TextureAtlas)>,
) {
    for (mut anim, mut atlas) in &mut query {
        if anim.timer.tick(time.delta()).just_finished() {
            anim.frame = (anim.frame + 1) % anim.frames.len();
            atlas.index = anim.frames[anim.frame];
        }
    }
}
