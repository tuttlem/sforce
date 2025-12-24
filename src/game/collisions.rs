use bevy::{math::Vec3Swizzles, prelude::*};

use super::{
    audio::AudioCue,
    enemies::Enemy,
    player::{Player, PlayerDefense, PlayerStats},
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
    mut enemies: Query<(Entity, &mut Enemy, &Transform, &Sprite)>,
    mut scoreboard: ResMut<ScoreBoard>,
    mut audio_events: EventWriter<AudioCue>,
) {
    let mut enemy_shapes = Vec::new();
    for (entity, enemy, transform, sprite) in enemies.iter_mut() {
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
        commands.entity(bullet_entity).despawn_recursive();
        if let Ok((entity, mut enemy, _, _)) = enemies.get_mut(enemy_entity) {
            enemy.health -= 1;
            if enemy.health <= 0 {
                commands.entity(entity).despawn_recursive();
                scoreboard.score += enemy.score;
                audio_events.send(AudioCue::Explosion);
            }
        }
    }
}

fn player_enemy_collisions(
    mut commands: Commands,
    mut player_query: Query<(&Transform, &Sprite, &mut PlayerDefense), With<Player>>,
    enemies: Query<(Entity, &Enemy, &Transform, &Sprite)>,
    mut stats: ResMut<PlayerStats>,
    mut next_state: ResMut<NextState<AppState>>,
    mut audio_events: EventWriter<AudioCue>,
) {
    let Ok((player_transform, player_sprite, mut defense)) = player_query.get_single_mut() else {
        return;
    };

    let player_half = sprite_half_extents(player_sprite);
    let player_center = player_transform.translation.xy();

    for (enemy_entity, enemy, enemy_transform, enemy_sprite) in &enemies {
        let enemy_half = sprite_half_extents(enemy_sprite);
        let enemy_center = enemy_transform.translation.xy();
        if overlaps(player_center, player_half, enemy_center, enemy_half)
            && handle_player_hit(
                &mut stats,
                &mut defense,
                &mut next_state,
                enemy.damage,
                &mut audio_events,
            )
        {
            commands.entity(enemy_entity).despawn_recursive();
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
        ) {
            commands.entity(projectile_entity).despawn_recursive();
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
) -> bool {
    if defense.invulnerability > 0.0 {
        return false;
    }

    let damage = damage.max(1);
    stats.lives = stats.lives.saturating_sub(damage);
    audio_events.send(AudioCue::Hit);
    if stats.lives == 0 {
        defense.invulnerability = 0.0;
        next_state.set(AppState::GameOver);
    } else {
        defense.invulnerability = 1.5;
    }
    true
}

fn sprite_half_extents(sprite: &Sprite) -> Vec2 {
    sprite.custom_size.unwrap_or(Vec2::splat(32.0)) * 0.5
}

fn overlaps(a_center: Vec2, a_half: Vec2, b_center: Vec2, b_half: Vec2) -> bool {
    (a_center.x - b_center.x).abs() <= (a_half.x + b_half.x)
        && (a_center.y - b_center.y).abs() <= (a_half.y + b_half.y)
}
