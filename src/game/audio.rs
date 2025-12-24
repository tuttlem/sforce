use std::{f32::consts::PI, sync::Arc};

use bevy::{
    audio::{AudioBundle, AudioSink, AudioSource, PlaybackSettings, Volume},
    prelude::*,
};

use super::{config::GameSettings, states::AppState};

pub struct AudioPlugin;

impl Plugin for AudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<AudioCue>()
            .init_resource::<AudioAssets>()
            .init_resource::<MusicState>()
            .add_systems(Startup, setup_audio_assets)
            .add_systems(OnEnter(AppState::Title), start_title_music)
            .add_systems(Update, (handle_audio_cues, apply_music_volume));
    }
}

#[derive(Resource, Default)]
pub struct AudioAssets {
    pub music: Handle<AudioSource>,
    pub shoot: Handle<AudioSource>,
    pub hit: Handle<AudioSource>,
    pub explosion: Handle<AudioSource>,
    pub pickup: Handle<AudioSource>,
    pub ui: Handle<AudioSource>,
}

#[derive(Resource, Default)]
struct MusicState {
    entity: Option<Entity>,
}

#[derive(Event, Clone, Copy, Debug)]
pub enum AudioCue {
    Shoot,
    Hit,
    Explosion,
    Pickup,
    UiSelect,
}

fn setup_audio_assets(mut assets: ResMut<Assets<AudioSource>>, mut store: ResMut<AudioAssets>) {
    store.music = assets.add(build_pad_source(220.0, 280.0, 12.0));
    store.shoot = assets.add(build_tone_source(760.0, 0.08, 0.3));
    store.hit = assets.add(build_tone_source(260.0, 0.15, 0.35));
    store.explosion = assets.add(build_noise_burst(0.25, 0.45));
    store.pickup = assets.add(build_tone_source(980.0, 0.18, 0.4));
    store.ui = assets.add(build_tone_source(440.0, 0.12, 0.25));
}

fn start_title_music(
    mut commands: Commands,
    assets: Res<AudioAssets>,
    settings: Res<GameSettings>,
    mut state: ResMut<MusicState>,
) {
    if state.entity.is_some() {
        return;
    }

    let entity = commands
        .spawn((AudioBundle {
            source: assets.music.clone(),
            settings: PlaybackSettings::LOOP.with_volume(Volume::new(settings.music_volume)),
            ..default()
        },))
        .id();
    state.entity = Some(entity);
}

fn handle_audio_cues(
    mut commands: Commands,
    mut reader: EventReader<AudioCue>,
    assets: Res<AudioAssets>,
    settings: Res<GameSettings>,
) {
    if reader.is_empty() {
        return;
    }

    for cue in reader.read() {
        let handle = match cue {
            AudioCue::Shoot => &assets.shoot,
            AudioCue::Hit => &assets.hit,
            AudioCue::Explosion => &assets.explosion,
            AudioCue::Pickup => &assets.pickup,
            AudioCue::UiSelect => &assets.ui,
        };
        commands.spawn(AudioBundle {
            source: handle.clone(),
            settings: PlaybackSettings::DESPAWN.with_volume(Volume::new(settings.sfx_volume)),
            ..default()
        });
    }
}

fn apply_music_volume(
    settings: Res<GameSettings>,
    state: Res<MusicState>,
    sinks: Query<&AudioSink>,
) {
    if let Some(entity) = state.entity {
        if let Ok(sink) = sinks.get(entity) {
            sink.set_volume(settings.music_volume);
        }
    }
}

fn build_pad_source(freq_a: f32, freq_b: f32, seconds: f32) -> AudioSource {
    let sample_rate = 44_100;
    let sample_count = (seconds * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    for i in 0..sample_count {
        let t = i as f32 / sample_rate as f32;
        let blend = (t / seconds).clamp(0.0, 1.0);
        let freq = freq_a + (freq_b - freq_a) * blend;
        let amp = 0.25 * (1.0 - (blend - 0.5).abs());
        let sample = (2.0 * PI * freq * t).sin() * amp;
        samples.push(sample);
    }
    make_wav(samples, sample_rate)
}

fn build_tone_source(freq: f32, seconds: f32, amplitude: f32) -> AudioSource {
    let sample_rate = 44_100;
    let sample_count = (seconds * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    for i in 0..sample_count {
        let t = i as f32 / sample_rate as f32;
        let env = (1.0 - (i as f32 / sample_count as f32)).powf(2.5);
        samples.push((2.0 * PI * freq * t).sin() * amplitude * env);
    }
    make_wav(samples, sample_rate)
}

fn build_noise_burst(seconds: f32, amplitude: f32) -> AudioSource {
    let sample_rate = 44_100;
    let sample_count = (seconds * sample_rate as f32) as usize;
    let mut samples = Vec::with_capacity(sample_count);
    let mut value = 0x1234_5678u32;
    for i in 0..sample_count {
        value = value.wrapping_mul(747796405).wrapping_add(2891336453);
        let noise = ((value >> 9) as f32 / (1u32 << 23) as f32) * 2.0 - 1.0;
        let env = (1.0 - (i as f32 / sample_count as f32)).powf(2.0);
        samples.push(noise * env * amplitude);
    }
    make_wav(samples, sample_rate)
}

fn make_wav(samples: Vec<f32>, sample_rate: u32) -> AudioSource {
    let channels = 1u16;
    let bits_per_sample = 16u16;
    let bytes_per_sample = (bits_per_sample / 8) as u32;
    let byte_rate = sample_rate * channels as u32 * bytes_per_sample;
    let block_align = channels * (bits_per_sample / 8);
    let data_bytes = samples.len() as u32 * bytes_per_sample;
    let mut buffer = Vec::with_capacity(44 + samples.len() * 2);
    buffer.extend_from_slice(b"RIFF");
    buffer.extend_from_slice(&(36 + data_bytes).to_le_bytes());
    buffer.extend_from_slice(b"WAVEfmt ");
    buffer.extend_from_slice(&16u32.to_le_bytes());
    buffer.extend_from_slice(&1u16.to_le_bytes());
    buffer.extend_from_slice(&channels.to_le_bytes());
    buffer.extend_from_slice(&sample_rate.to_le_bytes());
    buffer.extend_from_slice(&byte_rate.to_le_bytes());
    buffer.extend_from_slice(&block_align.to_le_bytes());
    buffer.extend_from_slice(&bits_per_sample.to_le_bytes());
    buffer.extend_from_slice(b"data");
    buffer.extend_from_slice(&data_bytes.to_le_bytes());
    for sample in samples {
        let clamped = (sample.clamp(-1.0, 1.0) * i16::MAX as f32) as i16;
        buffer.extend_from_slice(&clamped.to_le_bytes());
    }

    AudioSource {
        bytes: Arc::from(buffer.into_boxed_slice()),
    }
}
