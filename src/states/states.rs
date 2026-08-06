use crate::{assets::PendingSheets, input::Action};
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    GameOver,
    Loading,
    Options,
}

/// Exist only while AppState::InGame is active
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::InGame)]
pub enum IsPaused {
    #[default]
    Running,
    Paused,
}

#[derive(Component)]
pub struct LoadingFill;

pub fn setup_loading(mut commands: Commands) {
    commands
        .spawn((
            Text::new("Loading..."),
            TextFont {
                font_size: FontSize::Px(40.0),
                ..default()
            },
            DespawnOnExit(AppState::Loading),
        ))
        .with_children(|bar| {
            //loading bar
            bar.spawn((
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(32.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
            ))
            .with_children(|track| {
                track.spawn((
                    LoadingFill,
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.49, 0.82, 0.42)),
                ));
            });
        });
}

pub fn update_loading_bar(
    pending: Res<PendingSheets>,
    server: Res<AssetServer>,
    mut fill: Query<&mut Node, With<LoadingFill>>,
) {
    let total = pending.pending.len();
    if total == 0 {
        return; // nothing to load
    }

    let loaded = pending
        .pending
        .values()
        .filter(|handle| server.is_loaded_with_dependencies(*handle))
        .count();

    let frac = loaded as f32 / total as f32;
    if let Ok(mut node) = fill.single_mut() {
        node.width = Val::Percent(frac * 100.0);
    }
}

pub fn setup_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Text::new("PAUSED"),
        TextFont {
            font_size: FontSize::Px(60.0),
            ..default()
        },
        DespawnOnExit(IsPaused::Paused),
    ));
}

pub fn setup_gameover_screen(mut commands: Commands) {
    commands.spawn((
        Text::new("GAME OVER\n\nEnter - back to menu"),
        TextFont {
            font_size: FontSize::Px(40.0),
            ..default()
        },
        TextColor(Color::srgb(0.9, 0.2, 0.2)),
        DespawnOnExit(AppState::GameOver),
    ));
}

pub fn gameover_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) {
        next.set(AppState::Menu);
    }
}

pub fn ingame_input(
    players: Query<&ActionState<Action>>,
    paused: Res<State<IsPaused>>,
    mut next_pause: ResMut<NextState<IsPaused>>,
) {
    for actions in players {
        if actions.just_pressed(&Action::Pause) {
            next_pause.set(match paused.get() {
                IsPaused::Running => IsPaused::Paused,
                IsPaused::Paused => IsPaused::Running,
            });
        }
    }
}
