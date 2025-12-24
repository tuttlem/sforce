use std::{collections::HashMap, path::Path};

use bevy::{
    math::{URect, UVec2, Vec2},
    prelude::*,
    sprite::TextureAtlasLayout,
};
use image::RgbaImage;

pub struct ShipSpritePlugin;

#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub enum ShipSpriteId {
    Player,
    Grunt,
    Sine,
    ZigZag,
    Tank,
    Chaser,
    Boss,
}

#[derive(Resource, Default)]
pub struct ShipSpriteAssets {
    map: HashMap<ShipSpriteId, ShipSpriteData>,
}

impl ShipSpriteAssets {
    pub fn data(&self, id: ShipSpriteId) -> &ShipSpriteData {
        self.map.get(&id).expect("missing ship sprite data")
    }

    pub fn sequence(&self, id: ShipSpriteId, row: usize) -> &[usize] {
        let data = self.data(id);
        data.sequences
            .get(row)
            .expect("invalid row for ship sprite")
    }
}

#[derive(Clone)]
pub struct ShipSpriteData {
    pub texture: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    pub sequences: Vec<Vec<usize>>,
    pub frame_size: Vec2,
    pub scale: f32,
}

#[derive(Component)]
pub struct ShipAnimation {
    pub ship: ShipSpriteId,
    pub row: usize,
    pub frame: usize,
    pub timer: Timer,
}

impl ShipAnimation {
    pub fn new(ship: ShipSpriteId, row: usize, rate: f32) -> Self {
        Self {
            ship,
            row,
            frame: 0,
            timer: Timer::from_seconds(rate, TimerMode::Repeating),
        }
    }
}

impl Plugin for ShipSpritePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ShipSpriteAssets>()
            .add_systems(Startup, load_ship_sprites)
            .add_systems(Update, animate_ship_sprites);
    }
}

const SHIP_SPECS: &[(ShipSpriteId, &str, f32)] = &[
    (ShipSpriteId::Player, "images/tinyShip3.png", 3.2),
    (ShipSpriteId::Grunt, "images/tinyShip1.png", 3.0),
    (ShipSpriteId::Sine, "images/tinyShip5.png", 3.0),
    (ShipSpriteId::ZigZag, "images/tinyShip7.png", 2.8),
    (ShipSpriteId::Tank, "images/tinyShip13.png", 3.8),
    (ShipSpriteId::Chaser, "images/tinyShip10.png", 3.2),
    (ShipSpriteId::Boss, "images/tinyShip20.png", 5.5),
];

fn load_ship_sprites(
    mut commands: Commands,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    asset_server: Res<AssetServer>,
) {
    let mut assets = ShipSpriteAssets::default();
    for (id, path, scale) in SHIP_SPECS.iter() {
        let (layout_handle, sequences, frame_size) =
            build_layout(Path::new("assets").join(path), &mut layouts);
        let texture = asset_server.load(*path);
        assets.map.insert(
            *id,
            ShipSpriteData {
                texture,
                layout: layout_handle,
                sequences,
                frame_size,
                scale: *scale,
            },
        );
    }
    commands.insert_resource(assets);
}

fn build_layout(
    path: impl AsRef<Path>,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> (Handle<TextureAtlasLayout>, Vec<Vec<usize>>, Vec2) {
    let img = image::open(path)
        .expect("failed to load ship sprite")
        .to_rgba8();
    let (width, height) = img.dimensions();
    let row_ranges = extract_row_ranges(&img);

    let mut layout = TextureAtlasLayout::new_empty(UVec2::new(width, height));
    let mut sequences = Vec::new();
    let mut frame_width = width as f32;
    let mut frame_height = height as f32;

    for row in &row_ranges {
        let col_ranges = extract_col_ranges(&img, *row);
        if col_ranges.is_empty() {
            continue;
        }
        if sequences.is_empty() {
            frame_height = (row.1 - row.0) as f32;
        }
        let mut seq = Vec::new();
        for col in &col_ranges {
            let rect = URect::new(col.0, row.0, col.1, row.1);
            let index = layout.add_texture(rect);
            seq.push(index);
        }
        frame_width = (col_ranges[0].1 - col_ranges[0].0) as f32;
        sequences.push(seq);
    }

    let handle = layouts.add(layout);
    (handle, sequences, Vec2::new(frame_width, frame_height))
}

fn extract_row_ranges(img: &RgbaImage) -> Vec<(u32, u32)> {
    extract_ranges_generic(img.height(), |y| row_has_alpha(img, y))
}

fn extract_col_ranges(img: &RgbaImage, row: (u32, u32)) -> Vec<(u32, u32)> {
    extract_ranges_generic(img.width(), |x| column_band_has_alpha(img, x, row))
}

fn extract_ranges_generic(len: u32, mut has_alpha: impl FnMut(u32) -> bool) -> Vec<(u32, u32)> {
    let mut ranges = Vec::new();
    let mut start = None;
    for idx in 0..len {
        if has_alpha(idx) {
            if start.is_none() {
                start = Some(idx);
            }
        } else if let Some(s) = start.take() {
            if idx - s > 2 {
                ranges.push((s, idx));
            }
        }
    }
    if let Some(s) = start {
        if len - s > 2 {
            ranges.push((s, len));
        }
    }
    ranges
}

fn row_has_alpha(img: &RgbaImage, y: u32) -> bool {
    for x in 0..img.width() {
        if img.get_pixel(x, y)[3] > 5 {
            return true;
        }
    }
    false
}

fn column_band_has_alpha(img: &RgbaImage, x: u32, row: (u32, u32)) -> bool {
    for y in row.0..row.1 {
        if img.get_pixel(x, y)[3] > 5 {
            return true;
        }
    }
    false
}

fn animate_ship_sprites(
    time: Res<Time>,
    assets: Res<ShipSpriteAssets>,
    mut query: Query<(&mut ShipAnimation, &mut TextureAtlas)>,
) {
    for (mut anim, mut atlas) in &mut query {
        if anim.timer.tick(time.delta()).just_finished() {
            let frames = assets.sequence(anim.ship, anim.row);
            anim.frame = (anim.frame + 1) % frames.len();
            atlas.index = frames[anim.frame];
        }
    }
}
