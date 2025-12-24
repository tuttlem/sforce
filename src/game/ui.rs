use bevy::prelude::*;

use super::{
    AppState,
    audio::AudioCue,
    boss::BossState,
    config::{Difficulty, GameSettings},
    player::PlayerStats,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScoreBoard>()
            .register_type::<ScoreBoard>()
            .add_systems(
                OnEnter(AppState::Title),
                (reset_scoreboard, spawn_title_screen),
            )
            .add_systems(
                Update,
                (title_input, title_settings_input, title_settings_display)
                    .run_if(in_state(AppState::Title)),
            )
            .add_systems(OnExit(AppState::Title), cleanup_ui::<TitleScreen>)
            .add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(OnExit(AppState::Playing), cleanup_ui::<HudRoot>)
            .add_systems(Update, hud_update.run_if(in_state(AppState::Playing)))
            .add_systems(Update, boss_health_bar_update)
            .add_systems(Update, pause_input.run_if(in_state(AppState::Playing)))
            .add_systems(OnEnter(AppState::Paused), spawn_pause_overlay)
            .add_systems(OnExit(AppState::Paused), cleanup_ui::<PauseOverlay>)
            .add_systems(Update, resume_input.run_if(in_state(AppState::Paused)))
            .add_systems(OnEnter(AppState::GameOver), spawn_game_over_screen)
            .add_systems(OnExit(AppState::GameOver), cleanup_ui::<GameOverScreen>)
            .add_systems(Update, game_over_input.run_if(in_state(AppState::GameOver)));
    }
}

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct ScoreBoard {
    pub score: u32,
}

#[derive(Component)]
struct TitleScreen;

#[derive(Component)]
struct TitleDifficultyText;

#[derive(Component)]
struct TitleMusicText;

#[derive(Component)]
struct TitleSfxText;

#[derive(Component)]
struct HudRoot;

#[derive(Component)]
struct HudScoreText;

#[derive(Component)]
struct HudLivesText;

#[derive(Component)]
struct GameOverScreen;

#[derive(Component)]
struct PauseOverlay;

#[derive(Component)]
struct BossHealthBar;

#[derive(Component)]
struct BossHealthFill;

fn reset_scoreboard(mut scoreboard: ResMut<ScoreBoard>) {
    scoreboard.score = 0;
}

fn spawn_title_screen(mut commands: Commands) {
    let title_style = TextStyle {
        font_size: 56.0,
        color: Color::WHITE,
        ..default()
    };

    let instructions_style = TextStyle {
        font_size: 24.0,
        color: Color::srgb(0.7, 0.9, 1.0),
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(18.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            TitleScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("S-FORCE", title_style));
            parent.spawn(TextBundle::from_sections([
                TextSection::new(
                    "Press Space or Enter to Start\n",
                    instructions_style.clone(),
                ),
                TextSection::new("WASD / Arrow Keys to move\n", instructions_style.clone()),
                TextSection::new(
                    "Hold Space or Left Click to fire\n",
                    instructions_style.clone(),
                ),
                TextSection::new(
                    "Tab=Difficulty  |  -/+ Music  |  [/] SFX",
                    instructions_style.clone(),
                ),
            ]));
            parent.spawn((
                TextBundle::from_section("Difficulty: ", instructions_style.clone()),
                TitleDifficultyText,
            ));
            parent.spawn((
                TextBundle::from_section("Music Volume: ", instructions_style.clone()),
                TitleMusicText,
            ));
            parent.spawn((
                TextBundle::from_section("SFX Volume: ", instructions_style),
                TitleSfxText,
            ));
        });
}

fn spawn_hud(mut commands: Commands, stats: Res<PlayerStats>, scoreboard: Res<ScoreBoard>) {
    let label_style = TextStyle {
        font_size: 24.0,
        color: Color::WHITE,
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(16.0),
                    left: Val::Px(16.0),
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(8.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            HudRoot,
        ))
        .with_children(|parent| {
            parent.spawn((
                TextBundle::from_section(
                    format!("Score: {}", scoreboard.score),
                    label_style.clone(),
                ),
                HudScoreText,
            ));
            parent.spawn((
                TextBundle::from_section(format!("Lives: {}", stats.lives), label_style.clone()),
                HudLivesText,
            ));
        });

    let mut boss_bar = NodeBundle {
        style: Style {
            position_type: PositionType::Absolute,
            top: Val::Px(16.0),
            left: Val::Percent(35.0),
            width: Val::Px(420.0),
            height: Val::Px(18.0),
            border: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        background_color: BackgroundColor(Color::srgba(0.2, 0.2, 0.3, 0.6)),
        ..default()
    };
    boss_bar.visibility = Visibility::Hidden;

    commands
        .spawn((boss_bar, BossHealthBar))
        .with_children(|parent| {
            parent.spawn((
                NodeBundle {
                    style: Style {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    background_color: BackgroundColor(Color::srgb(0.95, 0.32, 0.36)),
                    ..default()
                },
                BossHealthFill,
            ));
        });
}

fn hud_update(
    scoreboard: Res<ScoreBoard>,
    stats: Res<PlayerStats>,
    mut queries: ParamSet<(
        Query<&mut Text, With<HudScoreText>>,
        Query<&mut Text, With<HudLivesText>>,
    )>,
) {
    if scoreboard.is_changed() {
        if let Ok(mut text) = queries.p0().get_single_mut() {
            text.sections[0].value = format!("Score: {}", scoreboard.score);
        }
    }
    if stats.is_changed() {
        if let Ok(mut text) = queries.p1().get_single_mut() {
            text.sections[0].value = format!("Lives: {}", stats.lives);
        }
    }
}

fn spawn_game_over_screen(mut commands: Commands, scoreboard: Res<ScoreBoard>) {
    let title_style = TextStyle {
        font_size: 48.0,
        color: Color::WHITE,
        ..default()
    };
    let info_style = TextStyle {
        font_size: 24.0,
        color: Color::srgb(0.8, 0.85, 1.0),
        ..default()
    };

    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Px(16.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            GameOverScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("Game Over", title_style.clone()));
            parent.spawn(TextBundle::from_section(
                format!("Final Score: {}", scoreboard.score),
                info_style.clone(),
            ));
            parent.spawn(TextBundle::from_section(
                "Press Enter to return to Title",
                info_style,
            ));
        });
}

fn spawn_pause_overlay(mut commands: Commands) {
    let style = TextStyle {
        font_size: 40.0,
        color: Color::WHITE,
        ..default()
    };
    commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                ..default()
            },
            PauseOverlay,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                "Paused\nPress P or Esc to Resume",
                style,
            ));
        });
}

fn title_input(
    mut next_state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut audio: EventWriter<AudioCue>,
) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
        audio.send(AudioCue::UiSelect);
    }
}

fn pause_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut audio: EventWriter<AudioCue>,
) {
    if keys.just_pressed(KeyCode::Escape) || keys.just_pressed(KeyCode::KeyP) {
        next_state.set(AppState::Paused);
        audio.send(AudioCue::UiSelect);
    }
}

fn resume_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut audio: EventWriter<AudioCue>,
) {
    if keys.just_pressed(KeyCode::Escape)
        || keys.just_pressed(KeyCode::KeyP)
        || keys.just_pressed(KeyCode::Space)
    {
        next_state.set(AppState::Playing);
        audio.send(AudioCue::UiSelect);
    }
}

fn game_over_input(
    mut next_state: ResMut<NextState<AppState>>,
    keys: Res<ButtonInput<KeyCode>>,
    mut audio: EventWriter<AudioCue>,
) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Title);
        audio.send(AudioCue::UiSelect);
    }
}

fn title_settings_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut settings: ResMut<GameSettings>,
    mut audio: EventWriter<AudioCue>,
) {
    let mut changed = false;

    if keys.just_pressed(KeyCode::Tab) {
        settings.difficulty = match settings.difficulty {
            Difficulty::Easy => Difficulty::Normal,
            Difficulty::Normal => Difficulty::Hard,
            Difficulty::Hard => Difficulty::Easy,
        };
        changed = true;
    }
    if keys.just_pressed(KeyCode::Minus) {
        settings.music_volume = (settings.music_volume - 0.05).clamp(0.0, 1.0);
        changed = true;
    }
    if keys.just_pressed(KeyCode::Equal) {
        settings.music_volume = (settings.music_volume + 0.05).clamp(0.0, 1.0);
        changed = true;
    }
    if keys.just_pressed(KeyCode::BracketLeft) {
        settings.sfx_volume = (settings.sfx_volume - 0.05).clamp(0.0, 1.0);
        changed = true;
    }
    if keys.just_pressed(KeyCode::BracketRight) {
        settings.sfx_volume = (settings.sfx_volume + 0.05).clamp(0.0, 1.0);
        changed = true;
    }

    if changed {
        audio.send(AudioCue::UiSelect);
    }
}

fn title_settings_display(
    settings: Res<GameSettings>,
    mut queries: ParamSet<(
        Query<&mut Text, With<TitleDifficultyText>>,
        Query<&mut Text, With<TitleMusicText>>,
        Query<&mut Text, With<TitleSfxText>>,
    )>,
) {
    if let Ok(mut text) = queries.p0().get_single_mut() {
        text.sections[0].value = format!("Difficulty: {}", difficulty_label(settings.difficulty));
    }
    if let Ok(mut text) = queries.p1().get_single_mut() {
        text.sections[0].value =
            format!("Music Volume: {}%", (settings.music_volume * 100.0) as i32);
    }
    if let Ok(mut text) = queries.p2().get_single_mut() {
        text.sections[0].value = format!("SFX Volume: {}%", (settings.sfx_volume * 100.0) as i32);
    }
}

fn difficulty_label(difficulty: Difficulty) -> &'static str {
    match difficulty {
        Difficulty::Easy => "Easy",
        Difficulty::Normal => "Normal",
        Difficulty::Hard => "Hard",
    }
}

fn boss_health_bar_update(
    boss_state: Res<BossState>,
    mut visibility_query: Query<&mut Visibility, With<BossHealthBar>>,
    mut fill_query: Query<&mut Style, With<BossHealthFill>>,
) {
    let active = boss_state.active && boss_state.max_health > 0.0;
    if let Ok(mut visibility) = visibility_query.get_single_mut() {
        *visibility = if active {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    if !active {
        return;
    }

    let percent = (boss_state.health / boss_state.max_health).clamp(0.0, 1.0) * 100.0;
    if let Ok(mut style) = fill_query.get_single_mut() {
        style.width = Val::Percent(percent);
    }
}

fn cleanup_ui<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
