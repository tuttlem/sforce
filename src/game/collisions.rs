use bevy::{
    log::{info, warn},
    math::Vec3Swizzles,
    prelude::*,
};

use super::{
    audio::AudioCue,
    effects::ExplosionEvent,
    enemies::{Enemy, EnemyKind},
    player::{PLAYER_HIT_INVULNERABILITY, Player, PlayerDefense, PlayerLifeLostEvent, PlayerStats},
    powerups::{DropsPowerUp, SpawnPowerUpEvent},
    states::AppState,
    ui::ScoreBoard,
    weapons::{EnemyProjectile, Projectile},
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (
                projectile_enemy_collisions,
                player_enemy_collisions,
                enemy_projectile_player_collisions,
            )
                .run_if(in_state(AppState::Playing)),
        );
    }
}

fn projectile_enemy_collisions(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &Sprite), With<Projectile>>,
    mut enemies: Query<(
        Entity,
        &mut Enemy,
        &Transform,
        &Sprite,
        Option<&DropsPowerUp>,
    )>,
    mut scoreboard: ResMut<ScoreBoard>,
    mut audio_events: EventWriter<AudioCue>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut powerup_events: EventWriter<SpawnPowerUpEvent>,
) {
    let mut enemy_shapes = Vec::new();
    for (entity, enemy, transform, sprite, _) in enemies.iter_mut() {
        enemy_shapes.push((
            entity,
            enemy.kind,
            transform.translation.xy(),
            sprite_half_extents(sprite),
        ));
    }

    let mut hits: Vec<(Entity, Entity)> = Vec::new();
    for (bullet_entity, bullet_transform, bullet_sprite) in &bullets {
        let bullet_half = sprite_half_extents(bullet_sprite);
        let bullet_center = bullet_transform.translation.xy();
        for (enemy_entity, _, enemy_center, enemy_half) in &enemy_shapes {
            if overlaps(*enemy_center, *enemy_half, bullet_center, bullet_half) {
                hits.push((bullet_entity, *enemy_entity));
                break;
            }
        }
    }

    for (bullet_entity, enemy_entity) in hits {
        despawn_with_check(&mut commands, bullet_entity, "player bullet");
        if let Ok((entity, mut enemy, transform, _, drop)) = enemies.get_mut(enemy_entity) {
            enemy.health -= 1;
            if enemy.health <= 0 {
                despawn_with_check(&mut commands, entity, "enemy (bullet collision)");
                scoreboard.score += enemy.score;
                audio_events.send(AudioCue::Explosion);
                if let Some(drop) = drop {
                    powerup_events.send(SpawnPowerUpEvent {
                        position: transform.translation.xy(),
                        kind: drop.kind,
                    });
                }
                explosion_events.send(ExplosionEvent {
                    position: transform.translation.xy(),
                    large: matches!(enemy.kind, EnemyKind::Tank | EnemyKind::Boss),
                });
            }
        }
    }
}

fn player_enemy_collisions(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &Sprite, &mut PlayerDefense), With<Player>>,
    enemies: Query<(Entity, &Enemy, &Transform, &Sprite, Option<&DropsPowerUp>)>,
    mut stats: ResMut<PlayerStats>,
    mut next_state: ResMut<NextState<AppState>>,
    mut audio_events: EventWriter<AudioCue>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut powerup_events: EventWriter<SpawnPowerUpEvent>,
    mut life_events: EventWriter<PlayerLifeLostEvent>,
) {
    let Ok((player_transform, player_sprite, mut defense)) = player_query.get_single_mut() else {
        return;
    };

    let player_half = sprite_half_extents(player_sprite);
    let player_center = player_transform.translation.xy();

    for (enemy_entity, enemy, enemy_transform, enemy_sprite, drop) in &enemies {
        let enemy_half = sprite_half_extents(enemy_sprite);
        let enemy_center = enemy_transform.translation.xy();
        if overlaps(player_center, player_half, enemy_center, enemy_half)
            && handle_player_hit(
                &mut stats,
                &mut defense,
                &mut next_state,
                enemy.damage,
                &mut audio_events,
                &mut life_events,
            )
        {
            despawn_with_check(&mut commands, enemy_entity, "enemy (ram)");
            if let Some(drop) = drop {
                powerup_events.send(SpawnPowerUpEvent {
                    position: enemy_center,
                    kind: drop.kind,
                });
            }
            explosion_events.send(ExplosionEvent {
                position: enemy_center,
                large: matches!(enemy.kind, EnemyKind::Tank | EnemyKind::Boss),
            });
            explosion_events.send(ExplosionEvent {
                position: player_center,
                large: true,
            });
            break;
        }
    }
}

fn enemy_projectile_player_collisions(
    mut commands: Commands,
    projectiles: Query<(Entity, &Transform, &Sprite, &EnemyProjectile)>,
    mut player_query: Query<(&Transform, &Sprite, &mut PlayerDefense), With<Player>>,
    mut stats: ResMut<PlayerStats>,
    mut next_state: ResMut<NextState<AppState>>,
    mut audio_events: EventWriter<AudioCue>,
    mut explosion_events: EventWriter<ExplosionEvent>,
    mut life_events: EventWriter<PlayerLifeLostEvent>,
) {
    let Ok((player_transform, player_sprite, mut defense)) = player_query.get_single_mut() else {
        return;
    };

    let player_half = sprite_half_extents(player_sprite);
    let player_center = player_transform.translation.xy();

    for (projectile_entity, projectile_transform, projectile_sprite, projectile) in &projectiles {
        let projectile_half = sprite_half_extents(projectile_sprite);
        let projectile_center = projectile_transform.translation.xy();
        if overlaps(
            player_center,
            player_half,
            projectile_center,
            projectile_half,
        ) && handle_player_hit(
            &mut stats,
            &mut defense,
            &mut next_state,
            projectile.damage,
            &mut audio_events,
            &mut life_events,
        ) {
            despawn_with_check(&mut commands, projectile_entity, "enemy projectile");
            explosion_events.send(ExplosionEvent {
                position: player_center,
                large: false,
            });
            break;
        }
    }
}

fn handle_player_hit(
    stats: &mut PlayerStats,
    defense: &mut PlayerDefense,
    next_state: &mut NextState<AppState>,
    damage: u8,
    audio_events: &mut EventWriter<AudioCue>,
    life_events: &mut EventWriter<PlayerLifeLostEvent>,
) -> bool {
    if defense.invulnerability > 0.0 {
        info!(
            "Ignored hit while invulnerable (remaining {:.2}s)",
            defense.invulnerability
        );
        return false;
    }

    let damage = damage.max(1);
    let previous_health = stats.health;
    stats.health = stats.health.saturating_sub(damage);
    info!(
        "Player hit for {damage}. Hull: {previous} -> {current} (lives: {lives})",
        damage = damage,
        previous = previous_health,
        current = stats.health,
        lives = stats.lives
    );
    audio_events.send(AudioCue::Hit);
    if stats.health == 0 {
        if stats.lives > 1 {
            stats.lives -= 1;
            stats.health = stats.max_health;
            defense.invulnerability = PLAYER_HIT_INVULNERABILITY;
            life_events.send(PlayerLifeLostEvent);
            info!("Player lost a life. Lives remaining: {}", stats.lives);
        } else {
            stats.lives = 0;
            defense.invulnerability = 0.0;
            next_state.set(AppState::GameOver);
            info!("Player destroyed. No lives remaining.");
        }
    } else {
        defense.invulnerability = PLAYER_HIT_INVULNERABILITY;
    }
    true
}

fn despawn_with_check(commands: &mut Commands, entity: Entity, context: &str) {
    if let Some(entity_cmd) = commands.get_entity(entity) {
        entity_cmd.despawn_recursive();
    } else {
        warn!(
            "Attempted to despawn {context} ({:?}) but it no longer exists",
            entity
        );
    }
}

fn sprite_half_extents(sprite: &Sprite) -> Vec2 {
    sprite.custom_size.unwrap_or(Vec2::splat(32.0)) * 0.5
}

fn overlaps(a_center: Vec2, a_half: Vec2, b_center: Vec2, b_half: Vec2) -> bool {
    (a_center.x - b_center.x).abs() <= (a_half.x + b_half.x)
        && (a_center.y - b_center.y).abs() <= (a_half.y + b_half.y)
}
