use crate::integrity::Integrity;
use crate::messages::{CopterCrashed, PassengerDelivered};
use crate::player::Player;
use crate::states::AppState;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct Score(pub u32);

#[derive(Component)]
struct ScoreHud;

#[derive(Component)]
struct IntegrityHud;

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CopterCrashed>()
            .add_message::<PassengerDelivered>()
            .init_resource::<Score>()
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
    commands.spawn((
        ScoreHud,
        Text::new("Score: 0"),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(8.0),
            left: px(8.0),
            ..default()
        },
        DespawnOnExit(AppState::InGame),
    ));
    commands.spawn((
        IntegrityHud,
        Text::new("Integrity: 100"),
        TextFont {
            font_size: FontSize::Px(24.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: px(36.0),
            left: px(8.0),
            ..default()
        },
        DespawnOnExit(AppState::InGame),
    ));
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
    mut int_hud: Single<&mut Text, With<IntegrityHud>>,
    player: Query<&Integrity, (With<Player>, Changed<Integrity>)>,
) {
    if let Ok(integrity) = player.single() {
        int_hud.0 = format!("Integrity: {}", integrity.0);
    }
}

fn end_game_on_crash(
    mut crashes: MessageReader<CopterCrashed>,
    mut next: ResMut<NextState<AppState>>,
) {
    if crashes.read().next().is_some() {
        next.set(AppState::GameOver);
    }
}
