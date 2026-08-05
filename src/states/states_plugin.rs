use crate::main_menu::*;
use crate::states::states::{
    gameover_input, ingame_input, setup_gameover_screen, setup_loading, setup_pause_overlay,
    update_loading_bar,
};
use crate::states::{AppState, IsPaused};
use bevy::prelude::*;

pub struct StatesPlugin;

impl Plugin for StatesPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>()
            .add_sub_state::<IsPaused>()
            // Screens: spawns on enter, auto-despawned on exit
            .add_systems(OnEnter(AppState::Menu), setup_main_menu)
            .add_systems(OnEnter(AppState::GameOver), setup_gameover_screen)
            .add_systems(OnEnter(IsPaused::Paused), setup_pause_overlay)
            .add_systems(OnEnter(AppState::Loading), setup_loading)
            // input routers, gated per state
            .add_systems(
                Update,
                (
                    update_loading_bar.run_if(in_state(AppState::Loading)),
                    gameover_input.run_if(in_state(AppState::GameOver)),
                    ingame_input.run_if(in_state(AppState::InGame)),
                    (menu_keyboard_system, menu_mouse_system, menu_highlight)
                        .chain()
                        .run_if(in_state(AppState::Menu)),
                ),
            );
    }
}
