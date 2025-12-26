use std::f32::consts::FRAC_PI_2;

use bevy::{prelude::*, time::Fixed};

use super::{
    audio::AudioCue,
    config::GameConfig,
    ship_sprites::{ShipAnimation, ShipSpriteAssets, ShipSpriteId},
    states::AppState,
    weapons::PlayerFireEvent,
};

#[derive(Event, Debug, Clone, Copy)]
pub struct PlayerLifeLostEvent;

pub const PLAYER_HIT_INVULNERABILITY: f32 = 1.6;
const PLAYER_INVULNERABILITY_FLICKER_HZ: f32 = 14.0;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerStats>()
            .register_type::<PlayerStats>()
            .insert_resource(PlayerSettings::default())
            .register_type::<PlayerSettings>()
            .init_resource::<PlayerWeaponState>()
            .register_type::<PlayerWeaponState>()
            .add_event::<PlayerLifeLostEvent>()
            .add_systems(OnEnter(AppState::Playing), spawn_player)
            .add_systems(OnExit(AppState::Playing), despawn_player)
            .add_systems(
                FixedUpdate,
                (
                    handle_player_movement,
                    player_fire_input,
                    tick_player_invulnerability,
                    handle_life_loss_respawn,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(
                Update,
                update_player_flash.run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Component, Default)]
pub struct Player;

#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct PlayerStats {
    pub health: u8,
    pub max_health: u8,
    pub lives: u8,
    pub max_lives: u8,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            health: 5,
            max_health: 5,
            lives: 3,
            max_lives: 3,
        }
    }
}

impl PlayerStats {
    pub fn reset(&mut self) {
        self.max_health = 5;
        self.health = self.max_health;
        self.max_lives = 3;
        self.lives = self.max_lives;
    }

    pub fn health_fraction(&self) -> f32 {
        if self.max_health == 0 {
            0.0
        } else {
            self.health as f32 / self.max_health as f32
        }
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

#[derive(Resource, Debug, Clone, Copy, Reflect)]
#[reflect(Resource)]
pub struct PlayerWeaponState {
    pub mode: WeaponMode,
    pub fire_rate_level: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Reflect)]
pub enum WeaponMode {
    Single,
    Double,
    Spread3,
    Spread5,
    Laser,
}

impl Default for PlayerWeaponState {
    fn default() -> Self {
        Self {
            mode: WeaponMode::Single,
            fire_rate_level: 0,
        }
    }
}

impl PlayerWeaponState {
    pub fn reset(&mut self) {
        self.mode = WeaponMode::Single;
        self.fire_rate_level = 0;
    }

    pub fn current_cooldown(&self, settings: &PlayerSettings) -> f32 {
        let mut cooldown = settings.fire_cooldown * 0.85f32.powi(self.fire_rate_level as i32);
        if matches!(self.mode, WeaponMode::Laser) {
            cooldown *= 0.4;
        }
        cooldown.clamp(0.06, 0.4)
    }

    pub fn advance_mode(&mut self) {
        self.mode = match self.mode {
            WeaponMode::Single => WeaponMode::Double,
            WeaponMode::Double => WeaponMode::Spread3,
            WeaponMode::Spread3 => WeaponMode::Spread5,
            WeaponMode::Spread5 => WeaponMode::Laser,
            WeaponMode::Laser => WeaponMode::Laser,
        };
    }

    pub fn boost_fire_rate(&mut self) {
        self.fire_rate_level = self.fire_rate_level.saturating_add(1).min(5);
    }
}

#[derive(Component, Default)]
pub struct Velocity(pub Vec2);

#[derive(Component)]
pub struct PlayerDefense {
    pub invulnerability: f32,
}

#[derive(Component)]
pub struct PlayerAppearance {
    pub normal_color: Color,
    pub hit_color: Color,
}

fn spawn_player(
    mut commands: Commands,
    mut stats: ResMut<PlayerStats>,
    mut weapon_state: ResMut<PlayerWeaponState>,
    sprites: Res<ShipSpriteAssets>,
) {
    stats.reset();
    weapon_state.reset();
    let normal_color = Color::WHITE;
    let hit_color = Color::srgb(1.0, 0.6, 0.6);
    let sprite_data = sprites.data(ShipSpriteId::Player);
    let sequence = sprites.sequence(ShipSpriteId::Player, 0);
    commands.spawn((
        SpriteBundle {
            texture: sprite_data.texture.clone(),
            transform: Transform::from_xyz(0.0, -260.0, 2.0),
            sprite: Sprite {
                color: normal_color,
                custom_size: Some(sprite_data.frame_size * sprite_data.scale),
                ..default()
            },
            ..default()
        },
        TextureAtlas {
            layout: sprite_data.layout.clone(),
            index: sequence[0],
        },
        Player,
        Velocity::default(),
        PlayerDefense {
            invulnerability: 0.0,
        },
        PlayerAppearance {
            normal_color,
            hit_color,
        },
        ShipAnimation::new(ShipSpriteId::Player, 0, 0.08),
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
    mouse_buttons: Res<ButtonInput<MouseButton>>,
    query: Query<&Transform, With<Player>>,
    settings: Res<PlayerSettings>,
    weapon_state: Res<PlayerWeaponState>,
    mut time_since_fire: Local<f32>,
    time: Res<Time<Fixed>>,
    mut writer: EventWriter<PlayerFireEvent>,
    mut audio_writer: EventWriter<AudioCue>,
) {
    let Ok(transform) = query.get_single() else {
        return;
    };

    *time_since_fire += time.delta_seconds();

    let shooting = keys.pressed(KeyCode::Space)
        || keys.pressed(KeyCode::Enter)
        || mouse_buttons.pressed(MouseButton::Left);
    let cooldown = weapon_state.current_cooldown(&settings);
    if shooting && *time_since_fire >= cooldown {
        *time_since_fire = 0.0;
        fire_weapon_pattern(
            weapon_state.as_ref(),
            transform.translation.truncate(),
            &mut writer,
        );
        audio_writer.send(AudioCue::Shoot);
    }
}

fn fire_weapon_pattern(
    weapon_state: &PlayerWeaponState,
    origin: Vec2,
    writer: &mut EventWriter<PlayerFireEvent>,
) {
    match weapon_state.mode {
        WeaponMode::Single => {
            emit_shot(
                writer,
                origin + Vec2::new(0.0, 32.0),
                Vec2::Y,
                520.0,
                Vec2::new(12.0, 24.0),
                1.6,
            );
        }
        WeaponMode::Double => {
            emit_shot(
                writer,
                origin + Vec2::new(-18.0, 32.0),
                Vec2::Y,
                520.0,
                Vec2::new(12.0, 24.0),
                1.6,
            );
            emit_shot(
                writer,
                origin + Vec2::new(18.0, 32.0),
                Vec2::Y,
                520.0,
                Vec2::new(12.0, 24.0),
                1.6,
            );
        }
        WeaponMode::Spread3 => {
            for angle in [-0.2, 0.0, 0.2] {
                emit_angle_shot(writer, origin, angle, 540.0, Vec2::new(12.0, 22.0));
            }
        }
        WeaponMode::Spread5 => {
            for angle in [-0.35, -0.18, 0.0, 0.18, 0.35] {
                emit_angle_shot(writer, origin, angle, 560.0, Vec2::new(10.0, 22.0));
            }
        }
        WeaponMode::Laser => {
            emit_shot(
                writer,
                origin + Vec2::new(-8.0, 28.0),
                Vec2::Y,
                700.0,
                Vec2::new(8.0, 42.0),
                1.4,
            );
            emit_shot(
                writer,
                origin + Vec2::new(8.0, 28.0),
                Vec2::Y,
                700.0,
                Vec2::new(8.0, 42.0),
                1.4,
            );
        }
    }
}

fn emit_angle_shot(
    writer: &mut EventWriter<PlayerFireEvent>,
    origin: Vec2,
    offset_angle: f32,
    speed: f32,
    size: Vec2,
) {
    let angle = FRAC_PI_2 + offset_angle;
    let direction = Vec2::from_angle(angle);
    emit_shot(
        writer,
        origin + Vec2::new(0.0, 30.0),
        direction,
        speed,
        size,
        1.8,
    );
}

fn emit_shot(
    writer: &mut EventWriter<PlayerFireEvent>,
    origin: Vec2,
    direction: Vec2,
    speed: f32,
    size: Vec2,
    lifetime: f32,
) {
    let dir = direction.normalize_or_zero();
    if dir == Vec2::ZERO {
        return;
    }
    writer.send(PlayerFireEvent {
        origin,
        velocity: dir * speed,
        size,
        lifetime,
    });
}

fn tick_player_invulnerability(
    mut query: Query<&mut PlayerDefense, With<Player>>,
    time: Res<Time<Fixed>>,
) {
    for mut defense in &mut query {
        defense.invulnerability = (defense.invulnerability - time.delta_seconds()).max(0.0);
    }
}

fn handle_life_loss_respawn(
    mut events: EventReader<PlayerLifeLostEvent>,
    mut query: Query<(&mut Transform, &mut Velocity), With<Player>>,
) {
    if events.is_empty() {
        return;
    }
    for _ in events.read() {
        if let Ok((mut transform, mut velocity)) = query.get_single_mut() {
            transform.translation.x = 0.0;
            transform.translation.y = -260.0;
            velocity.0 = Vec2::ZERO;
        }
    }
}

pub fn update_player_flash(
    mut query: Query<
        (
            &PlayerDefense,
            &PlayerAppearance,
            &mut Sprite,
            &mut Visibility,
        ),
        With<Player>,
    >,
    time: Res<Time>,
) {
    let flicker_frequency = PLAYER_INVULNERABILITY_FLICKER_HZ.max(1.0);
    for (defense, appearance, mut sprite, mut visibility) in &mut query {
        if defense.invulnerability > 0.0 {
            let flicker_on = (time.elapsed_seconds_wrapped() * flicker_frequency).fract() > 0.5;
            sprite.color = if flicker_on {
                appearance.hit_color
            } else {
                appearance.normal_color
            };
            *visibility = if flicker_on {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            };
        } else {
            sprite.color = appearance.normal_color;
            *visibility = Visibility::Inherited;
        }
    }
}
