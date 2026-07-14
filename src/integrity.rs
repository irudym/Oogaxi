use bevy::prelude::*;

use crate::{
    messages::{CopterCrashed, CopterDamaged},
    physics::FlightConfig,
    player::Player,
    states::{AppState, IsPaused},
};

#[derive(Component)]
pub struct Integrity(pub f32);

/// I-frames: while present, all incoming damage is ignored
#[derive(Component)]
pub struct Invulnerable(pub Timer);

pub struct IntegrityPlugin;

impl Plugin for IntegrityPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<CopterDamaged>()
            .add_systems(OnEnter(AppState::InGame), reset_integrity)
            .add_systems(
                FixedUpdate,
                (tick_invulnerability, apply_damage)
                    .chain()
                    .run_if(in_state(IsPaused::Running)),
            );
    }
}

fn reset_integrity(cfg: Res<FlightConfig>, mut players: Query<&mut Integrity, With<Player>>) {
    for mut integrity in &mut players {
        integrity.0 = cfg.integrity_max;
    }
}

fn tick_invulnerability(
    time: Res<Time>,
    mut commands: Commands,
    mut shields: Query<(Entity, &mut Invulnerable)>,
) {
    for (entity, mut inv) in &mut shields {
        if inv.0.tick(time.delta()).is_finished() {
            commands.entity(entity).remove::<Invulnerable>();
        }
    }
}

fn apply_damage(
    cfg: Res<FlightConfig>,
    mut damage: MessageReader<CopterDamaged>,
    mut players: Query<(Entity, &mut Integrity, Has<Invulnerable>), With<Player>>,
    mut commands: Commands,
    mut destroyed: MessageWriter<CopterCrashed>,
) {
    let total: f32 = damage.read().map(|d| d.severity).sum();
    if total <= 0.0 {
        return;
    }
    for (entity, mut integrity, invulnerable) in &mut players {
        if invulnerable {
            continue;
        }
        integrity.0 -= total * cfg.damage_k;
        commands
            .entity(entity)
            .insert(Invulnerable(Timer::from_seconds(
                cfg.invuln_secs,
                TimerMode::Once,
            )));
        if integrity.0 <= 0.0 {
            destroyed.write(CopterCrashed);
        }
    }
}
