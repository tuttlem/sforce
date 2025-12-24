use bevy::{math::Vec3Swizzles, prelude::*};

use super::{
    enemies::Enemy,
    player::{Player, PlayerStats},
    states::AppState,
    ui::ScoreBoard,
    weapons::Projectile,
};

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedUpdate,
            (projectile_enemy_collisions, player_enemy_collisions)
                .run_if(in_state(AppState::Playing)),
        );
    }
}

fn projectile_enemy_collisions(
    mut commands: Commands,
    bullets: Query<(Entity, &Transform, &Sprite), With<Projectile>>,
    mut enemies: Query<(Entity, &mut Enemy, &Transform, &Sprite)>,
    mut scoreboard: ResMut<ScoreBoard>,
) {
    let mut enemy_shapes = Vec::new();
    for (entity, _enemy, transform, sprite) in enemies.iter_mut() {
        enemy_shapes.push((
            entity,
            transform.translation.xy(),
            sprite_half_extents(sprite),
        ));
    }

    let mut hits: Vec<(Entity, Entity)> = Vec::new();
    for (bullet_entity, bullet_transform, bullet_sprite) in &bullets {
        let bullet_half = sprite_half_extents(bullet_sprite);
        let bullet_center = bullet_transform.translation.xy();
        for (enemy_entity, enemy_center, enemy_half) in &enemy_shapes {
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
                scoreboard.score += 100;
            }
        }
    }
}

fn player_enemy_collisions(
    mut commands: Commands,
    player_query: Query<(&Transform, &Sprite), With<Player>>,
    enemies: Query<(Entity, &Transform, &Sprite), With<Enemy>>,
    mut stats: ResMut<PlayerStats>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok((player_transform, player_sprite)) = player_query.get_single() else {
        return;
    };

    let player_half = sprite_half_extents(player_sprite);
    let player_center = player_transform.translation.xy();

    for (enemy_entity, enemy_transform, enemy_sprite) in &enemies {
        let enemy_half = sprite_half_extents(enemy_sprite);
        let enemy_center = enemy_transform.translation.xy();
        if overlaps(player_center, player_half, enemy_center, enemy_half) {
            stats.lives = stats.lives.saturating_sub(1);
            commands.entity(enemy_entity).despawn_recursive();
            next_state.set(AppState::GameOver);
            break;
        }
    }
}

fn sprite_half_extents(sprite: &Sprite) -> Vec2 {
    sprite.custom_size.unwrap_or(Vec2::splat(32.0)) * 0.5
}

fn overlaps(a_center: Vec2, a_half: Vec2, b_center: Vec2, b_half: Vec2) -> bool {
    (a_center.x - b_center.x).abs() <= (a_half.x + b_half.x)
        && (a_center.y - b_center.y).abs() <= (a_half.y + b_half.y)
}
