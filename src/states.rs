use crate::input::Action;
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;
use std::time::Duration;

use crate::messages::{CopterCrashed, PassengerDelivered};

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum AppState {
    #[default]
    Menu,
    InGame,
    GameOver,
    Loading,
}

/// Exist only while AppState::InGame is active
#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, SubStates)]
#[source(AppState = AppState::InGame)]
pub enum IsPaused {
    #[default]
    Running,
    Paused,
}

#[derive(Resource, Default)]
struct LoadingTimer(Timer);

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<IsPaused>()
            // Screens: spawns on enter, auto-despawned on exit
            .add_systems(OnEnter(AppState::Menu), setup_menu_screen)
            .add_systems(OnEnter(AppState::GameOver), setup_gameover_screen)
            .add_systems(OnEnter(IsPaused::Paused), setup_pause_overlay)
            .add_systems(OnEnter(AppState::Loading), setup_loading)
            // input routers, gated per state
            .init_resource::<LoadingTimer>()
            .add_systems(
                Update,
                (
                    menu_input.run_if(in_state(AppState::Menu)),
                    gameover_input.run_if(in_state(AppState::GameOver)),
                    ingame_input.run_if(in_state(AppState::InGame)),
                    loading_update.run_if(in_state(AppState::Loading)),
                ),
            );
    }
}

fn setup_loading(mut commands: Commands, mut timer: ResMut<LoadingTimer>) {
    commands.spawn((
        Text::new("Loading..."),
        TextFont {
            font_size: FontSize::Px(40.0),
            ..default()
        },
        DespawnOnExit(AppState::Loading),
    ));
    timer.0.set_duration(Duration::from_secs(1));
    timer.0.reset();
}

fn setup_menu_screen(mut commands: Commands) {
    commands.spawn((
        Text::new("Oogaxi: Through Taxiverse\n\nEnter - start     ESC in game - pause"),
        TextFont {
            font_size: FontSize::Px(40.0),
            ..default()
        },
        DespawnOnExit(AppState::Menu),
    ));
}

fn setup_pause_overlay(mut commands: Commands) {
    commands.spawn((
        Text::new("PAUSED"),
        TextFont {
            font_size: FontSize::Px(60.0),
            ..default()
        },
        DespawnOnExit(IsPaused::Paused),
    ));
}

fn setup_gameover_screen(mut commands: Commands) {
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

fn menu_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) {
        next.set(AppState::Loading);
    }
}

fn gameover_input(keys: Res<ButtonInput<KeyCode>>, mut next: ResMut<NextState<AppState>>) {
    if keys.just_pressed(KeyCode::Enter) {
        next.set(AppState::Menu);
    }
}

fn ingame_input(
    mut players: Query<(&ActionState<Action>)>,
    paused: Res<State<IsPaused>>,
    mut next_pause: ResMut<NextState<IsPaused>>,
    mut crashed: MessageWriter<CopterCrashed>,
    mut delivered: MessageWriter<PassengerDelivered>,
) {
    for (actions) in players {
        if actions.pressed(&Action::Pause) {
            next_pause.set(match paused.get() {
                IsPaused::Running => IsPaused::Paused,
                IsPaused::Paused => IsPaused::Running,
            });
        }
    }

    /*
    if *paused.get() == IsPaused::Running {
        if keys.just_pressed(KeyCode::KeyK) {
            crashed.write(CopterCrashed);
        }
        if keys.just_pressed(KeyCode::KeyD) {
            delivered.write(PassengerDelivered { fare: 25 });
        }
    }
    */
}

fn loading_update(
    mut timer: ResMut<LoadingTimer>,
    mut next: ResMut<NextState<AppState>>,
    time: Res<Time>,
) {
    timer.0.tick(time.delta());

    if timer.0.is_finished() {
        next.set(AppState::InGame);
    }
}
