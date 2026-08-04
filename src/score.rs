use crate::integrity::Integrity;
use crate::messages::{CopterCrashed, PassengerDelivered};
use crate::physics::FlightConfig;
use crate::player::Player;
use crate::states::AppState;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Component)]
struct ScoreHud;

#[derive(Component)]
struct IntegrityFill;

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            .add_systems(OnEnter(AppState::InGame), (reset_score, spawn_hud))
            .add_systems(
                Update,
                (
                    score_deliveries,
                    end_game_on_crash,
                    update_score_hud.run_if(resource_changed::<Score>),
                    update_integrity_hud,
                )
                    .run_if(in_state(AppState::InGame)),
            );
    }
}

fn reset_score(mut score: ResMut<Score>) {
    score.0 = 0;
}

fn spawn_hud(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            DespawnOnExit(AppState::InGame),
            ScoreHud,
            Text::new("Score: 0"),
            TextFont {
                font_size: FontSize::Px(24.0),
                ..default()
            },
        ))
        .with_children(|root| {
            root.spawn(Node {
                width: Val::Percent(100.0),
                justify_content: JustifyContent::SpaceBetween,
                align_content: AlignContent::Center,
                ..default()
            })
            .with_children(|bar| {
                bar.spawn((ScoreHud, Text::new("0")));

                //integrity bar
                bar.spawn((
                    Node {
                        width: Val::Px(160.0),
                        height: Val::Px(16.0),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.12, 0.15)),
                ))
                .with_children(|track| {
                    track.spawn((
                        IntegrityFill,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            ..default()
                        },
                        BackgroundColor(Color::srgb(0.49, 0.82, 0.42)),
                    ));
                });
            });
        });
}

fn score_deliveries(mut deliveries: MessageReader<PassengerDelivered>, mut score: ResMut<Score>) {
    for delivery in deliveries.read() {
        score.0 += delivery.fare;
    }
}

fn update_score_hud(score: Res<Score>, mut score_hud: Single<&mut Text, With<ScoreHud>>) {
    score_hud.0 = format!("Score: {}", score.0);
}

fn update_integrity_hud(
    mut fill: Query<(&mut Node, &mut BackgroundColor), With<IntegrityFill>>,
    player: Query<&Integrity, (With<Player>, Changed<Integrity>)>,
    cfg: Res<FlightConfig>,
) {
    let Ok(integrity) = player.single() else {
        return;
    };
    let Ok((mut node, mut color)) = fill.single_mut() else {
        return;
    };
    let frac = (integrity.0 / cfg.integrity_max).clamp(0.0, 1.0);
    node.width = Val::Percent(frac * 100.0);
    color.0 = Color::srgb(1.0 - frac * 0.5, 0.3 + frac * 0.5, 0.25);
}

fn end_game_on_crash(
    mut crashes: MessageReader<CopterCrashed>,
    mut next: ResMut<NextState<AppState>>,
) {
    if crashes.read().next().is_some() {
        next.set(AppState::GameOver);
    }
}
