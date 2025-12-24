use bevy::prelude::*;
use bevy::sprite::{TextureAtlas, TextureAtlasLayout};

use super::states::AppState;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ExplosionEvent>()
            .init_resource::<ExplosionAssets>()
            .add_systems(Startup, load_explosion_assets)
            .add_systems(
                Update,
                (spawn_explosions, animate_explosions).run_if(in_state(AppState::Playing)),
            )
            .add_systems(OnExit(AppState::Playing), cleanup_explosions);
    }
}

#[derive(Resource, Default)]
pub struct ExplosionAssets {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub sequences: Vec<Vec<usize>>,
}

#[derive(Event, Debug, Clone, Copy)]
pub struct ExplosionEvent {
    pub position: Vec2,
    pub large: bool,
}

#[derive(Component)]
struct ExplosionAnimation {
    timer: Timer,
    frame: usize,
    sequence: usize,
}

const FRAME_SIZE: UVec2 = UVec2::new(16, 16);
const SHEET_COLUMNS: u32 = 36;
const SHEET_ROWS: u32 = 13;
const EXPLOSION_SETS: [[usize; 6]; 4] = [
    [30, 31, 32, 33, 34, 35],
    [66, 67, 68, 69, 70, 71],
    [102, 103, 104, 105, 106, 107],
    [138, 139, 140, 141, 142, 143],
];

fn load_explosion_assets(
    mut commands: Commands,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let texture = asset_server.load("images/explosions.png");
    let layout = TextureAtlasLayout::from_grid(FRAME_SIZE, SHEET_COLUMNS, SHEET_ROWS, None, None);
    let layout_handle = layouts.add(layout);
    commands.insert_resource(ExplosionAssets {
        texture,
        layout: layout_handle,
        sequences: EXPLOSION_SETS.iter().map(|seq| seq.to_vec()).collect(),
    });
}

fn spawn_explosions(
    mut commands: Commands,
    assets: Res<ExplosionAssets>,
    mut events: EventReader<ExplosionEvent>,
) {
    if events.is_empty() {
        return;
    }

    for event in events.read() {
        let sequence_index = if event.large {
            assets.sequences.len() - 1
        } else {
            rand_hash(event.position) as usize % assets.sequences.len()
        };
        let frames = &assets.sequences[sequence_index];
        let scale = if event.large { 4.5 } else { 2.8 };
        commands.spawn((
            SpriteBundle {
                texture: assets.texture.clone(),
                transform: Transform::from_translation(event.position.extend(5.0))
                    .with_scale(Vec3::splat(scale)),
                sprite: Sprite {
                    anchor: bevy::sprite::Anchor::Center,
                    ..default()
                },
                ..default()
            },
            TextureAtlas {
                layout: assets.layout.clone(),
                index: frames[0],
            },
            ExplosionAnimation {
                timer: Timer::from_seconds(0.04, TimerMode::Repeating),
                frame: 0,
                sequence: sequence_index,
            },
        ));
    }
}

fn animate_explosions(
    mut commands: Commands,
    time: Res<Time>,
    assets: Res<ExplosionAssets>,
    mut query: Query<(Entity, &mut ExplosionAnimation, &mut TextureAtlas)>,
) {
    for (entity, mut anim, mut atlas) in &mut query {
        if anim.timer.tick(time.delta()).just_finished() {
            anim.frame += 1;
            let frames = &assets.sequences[anim.sequence];
            if anim.frame >= frames.len() {
                commands.entity(entity).despawn_recursive();
            } else {
                atlas.index = frames[anim.frame];
            }
        }
    }
}

fn cleanup_explosions(mut commands: Commands, query: Query<Entity, With<ExplosionAnimation>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}

fn rand_hash(value: Vec2) -> u32 {
    let mut x = value.x.to_bits() ^ value.y.to_bits();
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846ca68b);
    x ^= x >> 16;
    x
}
