use bevy::prelude::*;

use super::{AppState, player::PlayerStats};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<ScoreBoard>()
            .register_type::<ScoreBoard>()
            .add_systems(
                OnEnter(AppState::Title),
                (reset_scoreboard, spawn_title_screen),
            )
            .add_systems(Update, title_input.run_if(in_state(AppState::Title)))
            .add_systems(OnExit(AppState::Title), cleanup_ui::<TitleScreen>)
            .add_systems(OnEnter(AppState::Playing), spawn_hud)
            .add_systems(OnExit(AppState::Playing), cleanup_ui::<HudRoot>)
            .add_systems(Update, hud_update.run_if(in_state(AppState::Playing)))
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
struct HudRoot;

#[derive(Component)]
struct HudScoreText;

#[derive(Component)]
struct HudLivesText;

#[derive(Component)]
struct GameOverScreen;

fn reset_scoreboard(mut scoreboard: ResMut<ScoreBoard>) {
    scoreboard.score = 0;
}

fn spawn_title_screen(mut commands: Commands) {
    let text_style = TextStyle {
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
                    row_gap: Val::Px(24.0),
                    ..default()
                },
                background_color: BackgroundColor(Color::NONE),
                ..default()
            },
            TitleScreen,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("S-FORCE", text_style.clone()));
            parent.spawn(TextBundle::from_sections([
                TextSection::new(
                    "Press Space or Enter to Start\n",
                    instructions_style.clone(),
                ),
                TextSection::new("WASD / Arrow Keys to move\n", instructions_style.clone()),
                TextSection::new("Hold Space to fire", instructions_style.clone()),
            ]));
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
}

fn hud_update(
    scoreboard: Res<ScoreBoard>,
    stats: Res<PlayerStats>,
    mut queries: ParamSet<
        (
            Query<&mut Text, With<HudScoreText>>,
            Query<&mut Text, With<HudLivesText>>,
        ),
    >,
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

fn title_input(mut next_state: ResMut<NextState<AppState>>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

fn game_over_input(mut next_state: ResMut<NextState<AppState>>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Enter) || keys.just_pressed(KeyCode::Space) {
        next_state.set(AppState::Title);
    }
}

fn cleanup_ui<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
