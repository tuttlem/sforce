use bevy::{prelude::*, time::Fixed};

use super::{config::GameConfig, states::AppState, weapons::PlayerFireEvent};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerStats>()
            .register_type::<PlayerStats>()
            .insert_resource(PlayerSettings::default())
            .register_type::<PlayerSettings>()
            .add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(OnExit(AppState::Playing), despawn_player)
            .add_systems(
                FixedUpdate,
                (handle_player_movement, player_fire_input).run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct PlayerStats {
    pub lives: u8,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self { lives: 3 }
    }
}

#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct PlayerSettings {
    pub speed: f32,
    pub fire_cooldown: f32,
}

impl Default for PlayerSettings {
    fn default() -> Self {
        Self {
            speed: 340.0,
            fire_cooldown: 0.25,
        }
    }
}

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

fn spawn_player(mut commands: Commands, mut stats: ResMut<PlayerStats>) {
    stats.lives = 3;
    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(0.0, -260.0, 2.0),
            sprite: Sprite {
                color: Color::srgb(0.4, 0.9, 1.0),
                custom_size: Some(Vec2::new(48.0, 64.0)),
                ..default()
            },
            ..default()
        },
        Player,
        Velocity::default(),
    ));
}

fn despawn_player(mut commands: Commands, query: Query<Entity, With<Player>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn handle_player_movement(
    keys: Res<ButtonInput<KeyCode>>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
    config: Res<GameConfig>,
    settings: Res<PlayerSettings>,
    time: Res<Time<Fixed>>,
) {
    let Ok((mut transform, mut velocity)) = query.get_single_mut() else {
        return;
    };

    let mut direction = Vec2::ZERO;
    if keys.pressed(KeyCode::KeyW) || keys.pressed(KeyCode::ArrowUp) {
        direction.y += 1.0;
    }
    if keys.pressed(KeyCode::KeyS) || keys.pressed(KeyCode::ArrowDown) {
        direction.y -= 1.0;
    }
    if keys.pressed(KeyCode::KeyA) || keys.pressed(KeyCode::ArrowLeft) {
        direction.x -= 1.0;
    }
    if keys.pressed(KeyCode::KeyD) || keys.pressed(KeyCode::ArrowRight) {
        direction.x += 1.0;
    }

    direction = direction.normalize_or_zero();
    velocity.0 = direction * settings.speed;

    transform.translation += (velocity.0 * time.delta_seconds()).extend(0.0);

    let half_w = config.logical_width * 0.5 - 24.0;
    let half_h = config.logical_height * 0.5 - 32.0;
    transform.translation.x = transform.translation.x.clamp(-half_w, half_w);
    transform.translation.y = transform.translation.y.clamp(-half_h, half_h);
}

fn player_fire_input(
    keys: Res<ButtonInput<KeyCode>>,
    query: Query<&Transform, With<Player>>,
    settings: Res<PlayerSettings>,
    mut time_since_fire: Local<f32>,
    time: Res<Time<Fixed>>,
    mut writer: EventWriter<PlayerFireEvent>,
) {
    let Ok(transform) = query.get_single() else {
        return;
    };

    *time_since_fire += time.delta_seconds();

    let shooting = keys.pressed(KeyCode::Space) || keys.pressed(KeyCode::Enter);
    if shooting && *time_since_fire >= settings.fire_cooldown {
        *time_since_fire = 0.0;
        writer.send(PlayerFireEvent {
            origin: transform.translation.truncate(),
        });
    }
}
