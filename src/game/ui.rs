use bevy::prelude::*;

use super::AppState;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(AppState::Title), spawn_title_screen)
            .add_systems(Update, title_input.run_if(in_state(AppState::Title)))
            .add_systems(OnExit(AppState::Title), cleanup_ui::<TitleScreen>)
            .add_systems(OnEnter(AppState::Playing), spawn_play_placeholder)
            .add_systems(
                OnExit(AppState::Playing),
                cleanup_ui::<GameplayHudPlaceholder>,
            );
    }
}

#[derive(Component)]
struct TitleScreen;

#[derive(Component)]
struct GameplayHudPlaceholder;

fn spawn_title_screen(mut commands: Commands) {
    let text_style = TextStyle {
        font_size: 48.0,
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
                TextSection::new("WASD/Arrow Keys to move\n", instructions_style.clone()),
                TextSection::new("Space to fire (later phases)", instructions_style.clone()),
            ]));
        });
}

fn spawn_play_placeholder(mut commands: Commands) {
    commands.spawn((
        TextBundle::from_section(
            "Gameplay initializing...",
            TextStyle {
                font_size: 32.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(32.0),
            left: Val::Px(32.0),
            ..default()
        }),
        GameplayHudPlaceholder,
    ));
}

fn title_input(mut next_state: ResMut<NextState<AppState>>, keys: Res<ButtonInput<KeyCode>>) {
    if keys.just_pressed(KeyCode::Space) || keys.just_pressed(KeyCode::Enter) {
        next_state.set(AppState::Playing);
    }
}

fn cleanup_ui<T: Component>(mut commands: Commands, query: Query<Entity, With<T>>) {
    for entity in &query {
        commands.entity(entity).despawn_recursive();
    }
}
