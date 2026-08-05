use bevy::prelude::*;

use crate::colors::*;
use crate::states::AppState;

#[derive(Component, Clone, Copy)]
pub enum MenuAction {
    Play,
    Options,
    Quit,
}

/// The current highlighted row
#[derive(Resource, Default)]
pub struct SelectedItem(usize);

const MENU_ITEMS: [MenuAction; 3] = [MenuAction::Play, MenuAction::Options, MenuAction::Quit];

// Colors

#[derive(Component)]
pub struct MenuIndex(usize);

pub fn setup_main_menu(mut commands: Commands) {
    commands.insert_resource(SelectedItem(0));

    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                row_gap: Val::Px(12.0),
                ..default()
            },
            DespawnOnExit(AppState::Menu),
        ))
        .with_children(|root| {
            root.spawn((
                Text::new("Oogaxi: Through the Taxiverse"),
                TextFont {
                    font_size: FontSize::Px(48.0),
                    ..default()
                },
                Node {
                    margin: UiRect::bottom(Val::Px(40.0)),
                    ..default()
                },
            ));

            for (i, action) in MENU_ITEMS.iter().enumerate() {
                let label = match action {
                    MenuAction::Play => "Play",
                    MenuAction::Options => "Options",
                    MenuAction::Quit => "Quit",
                };
                root.spawn((
                    *action,
                    MenuIndex(i),
                    Button,
                    Node {
                        width: Val::Px(220.0),
                        padding: UiRect::all(Val::Px(10.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    BackgroundColor(GameColors::REST),
                ))
                .with_children(|b| {
                    b.spawn((
                        Text::new(label),
                        TextFont {
                            font_size: FontSize::Px(28.0),
                            ..default()
                        },
                    ));
                });
            }
        });
}

pub fn menu_mouse_system(
    buttons: Query<(&Interaction, &MenuIndex, &MenuAction), Changed<Interaction>>,
    mut selected: ResMut<SelectedItem>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>, //ui: Res<AudioChannel<UiBus>>,
                                      // audio: Res<AudioAssets>,
) {
    for (interaction, idx, action) in &buttons {
        match interaction {
            Interaction::Pressed => {
                //ui.play(audio.click.clone());
                activate_item(*action, &mut next, &mut exit);
            }
            Interaction::Hovered => {
                selected.0 = idx.0;
            }
            Interaction::None => {}
        }
    }
}

pub fn menu_keyboard_system(
    keys: Res<ButtonInput<KeyCode>>,
    mut selected: ResMut<SelectedItem>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    let count = MENU_ITEMS.len();

    if keys.just_pressed(KeyCode::ArrowDown) {
        selected.0 = (selected.0 + 1) % count;
    }
    if keys.just_pressed(KeyCode::ArrowUp) {
        selected.0 = (selected.0 + count - 1) % count;
    }

    if keys.just_pressed(KeyCode::Enter) {
        activate_item(MENU_ITEMS[selected.0], &mut next, &mut exit);
    }
}

pub fn menu_highlight(
    selected: Res<SelectedItem>,
    mut items: Query<(&MenuIndex, &mut BackgroundColor)>,
) {
    for (idx, mut bg) in &mut items {
        bg.0 = if idx.0 == selected.0 {
            GameColors::HIGHLIGHT
        } else {
            GameColors::REST
        };
    }
}

fn activate_item(
    action: MenuAction,
    next: &mut NextState<AppState>,
    exit: &mut MessageWriter<AppExit>,
) {
    match action {
        MenuAction::Play => next.set(AppState::Loading),
        MenuAction::Options => next.set(AppState::Options),
        MenuAction::Quit => {
            exit.write(AppExit::Success);
        }
    }
}
