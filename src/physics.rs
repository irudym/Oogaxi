use crate::input::Action;
use crate::states::{AppState, IsPaused};
use bevy::prelude::*;
use leafwing_input_manager::action_state::ActionState;

/// The six numbers that are the game, reflect makes them inspectable/tunable
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct FlightConfig {
    pub gravity: f32,          // px/s2
    pub thrust: f32,           // px/s2
    pub horizontal_accel: f32, // px/s2
    pub drag: f32,             // 1/s, exponential decay rate
    pub max_speed: f32,        // px/s
    pub floor_y: f32,
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            gravity: 900.0,
            thrust: 1600.0,
            horizontal_accel: 700.0,
            drag: 1.8,
            max_speed: 800.0,
            floor_y: -320.0,
        }
    }
}

/// Shared by anything that moves under simulation
#[derive(Component, Default, Deref, DerefMut)]
pub struct Velocity(pub Vec2);

/// Input gathered per RENDER frame, consumed by SIMULATION ticks
#[derive(Component, Default)]
pub struct ThrustInput {
    pub vertical: f32,   // 1.0 while Up/Space is held
    pub horizontal: f32, // -1.0 .. =1.0
}

/// Simulation-space position: physics writes here, never to Transform
#[derive(Component, Default, Deref, DerefMut)]
pub struct PhysicalTranslation(pub Vec2);

/// Where the simulation was one tick ago - the other end of the render lerp
#[derive(Component, Default, Deref, DerefMut)]
pub struct PreviousPhysicalTranslation(pub Vec2);

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.register_type::<FlightConfig>()
            .init_resource::<FlightConfig>()
            .add_systems(
                RunFixedMainLoop,
                (
                    accumulate_input
                        .in_set(RunFixedMainLoopSystems::BeforeFixedMainLoop)
                        .run_if(in_state(IsPaused::Running)),
                    interpolate_rendered_transform
                        .in_set(RunFixedMainLoopSystems::AfterFixedMainLoop)
                        .run_if(in_state(AppState::InGame)),
                ),
            )
            .add_systems(
                FixedUpdate,
                advanced_physics.run_if(in_state(IsPaused::Running)),
            )
            // Pause the clock
            .add_systems(OnEnter(IsPaused::Paused), pause_clock)
            .add_systems(OnExit(IsPaused::Paused), resume_clock);
    }
}

fn pause_clock(mut time: ResMut<Time<Virtual>>) {
    time.pause();
}

fn resume_clock(mut time: ResMut<Time<Virtual>>) {
    time.unpause();
}

/// Every render frame: harvest the keyboard into intent
fn accumulate_input(mut players: Query<(&ActionState<Action>, &mut ThrustInput)>) {
    for (actions, mut intent) in &mut players {
        intent.vertical = if actions.pressed(&Action::Thrust) {
            1.0
        } else {
            0.0
        };
        intent.horizontal = actions.clamped_value(&Action::Move)
    }
}

/// One simulation tick, Runs 0..N times per frame; Res<Time> here is the fixed clock
fn advanced_physics(
    config: Res<FlightConfig>,
    time: Res<Time>,
    mut query: Query<(
        &mut PhysicalTranslation,
        &mut PreviousPhysicalTranslation,
        &mut Velocity,
        &ThrustInput,
    )>,
) {
    let dt = time.delta_secs();
    for (mut pos, mut prev, mut vel, input) in &mut query {
        prev.0 = pos.0;

        // Held intent: every tick this frame reads the same sampled value
        vel.0 = step_velocity(vel.0, input.vertical, input.horizontal, &config, dt);

        // Semi-implicit: position integrates the new velocity
        pos.0 += vel.0 * dt;

        // temp
        if pos.y < config.floor_y {
            pos.y = config.floor_y;
            vel.y = vel.y.max(0.0); // in case its neg
        }
    }
}

/// all velocity math
fn step_velocity(
    mut v: Vec2,
    thrust_held: f32,
    horizontal: f32,
    config: &FlightConfig,
    dt: f32,
) -> Vec2 {
    v.y += thrust_held * config.thrust * dt; // continues while held
    v.y -= config.gravity * dt;
    v.x += horizontal * config.horizontal_accel * dt;
    v *= (-config.drag * dt).exp();

    v.clamp_length_max(config.max_speed)
}

/// every render frame, after any ticks: blend prev->current physics
/// positions by how far we are into the next tick.
fn interpolate_rendered_transform(
    fixed_time: Res<Time<Fixed>>,
    mut query: Query<(
        &mut Transform,
        &PhysicalTranslation,
        &PreviousPhysicalTranslation,
    )>,
) {
    let alpha = fixed_time.overstep_fraction();
    for (mut transform, current, previous) in &mut query {
        let rendered = previous.lerp(current.0, alpha);
        transform.translation.x = rendered.x;
        transform.translation.y = rendered.y;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 64.0;

    /// Falling with no thrust must settle at terminal velocity ~ g/k
    #[test]
    fn reaches_terminal_velocity() {
        let config = FlightConfig::default();
        let mut v = Vec2::ZERO;

        for _ in 0..2000 {
            v = step_velocity(v, 0.0, 0.0, &config, DT);
        }

        let expected = config.gravity / config.drag;
        let error = (v.y.abs() - expected).abs() / expected;
        assert!(
            error < 0.02,
            "terminal {} vs expected {}",
            v.y.abs(),
            expected
        );
    }

    /// Thrusting at the hover duty cycle (d = g/T) must roughly hold altitude
    #[test]
    fn hover_duty_cycle_hovers() {
        let config = FlightConfig::default();
        let duty = config.gravity / config.thrust;
        const PERIOD: usize = 16;

        let on_ticks = (duty * PERIOD as f32).round() as usize;

        let (mut v, mut y) = (Vec2::ZERO, 0.0f32);
        for tick in 0..640 {
            let held = if tick % PERIOD < on_ticks { 1.0 } else { 0.0 };
            v = step_velocity(v, held, 0.0, &config, DT);
            y += v.y + DT;
        }

        // Duty quantization (on_ticks is rounded) allows some drift
        assert!(y.abs() < 200.0, "drifter {} px over 10s of hover", y);
    }
}
