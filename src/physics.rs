use crate::collision::Grounded;
use crate::input::Action;
use crate::levels::TileGrid;
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
    pub max_landing_vy: f32,   // fastest survivable vertical touchdown
    pub max_landing_vx: f32,   // fastest survivable lateral slide on touchdown
    pub wall_crash_speed: f32, // side/ceiling impact above this fatal
    pub ground_drag: f32,      // extra vs decay while Grounded (1/s)
    pub integrity_max: f32,
    pub damage_k: f32,        // integrity lost per px/s of severity
    pub hazard_severity: f32, // a flying objects strike
    pub invuln_secs: f32,     // i-frames after any damage
    pub max_passengers: u32,
    pub spawn_secs_min: f32,
    pub spawn_secs_max: f32,
    pub walk_speed: f32, //px/s
    pub fare_base: f32,
    pub fare_per_px: f32,
    pub fare_min: f32,
    pub fare_decay: f32,
}

impl Default for FlightConfig {
    fn default() -> Self {
        Self {
            gravity: 900.0,
            thrust: 1600.0,
            horizontal_accel: 700.0,
            drag: 1.8,
            max_speed: 800.0,
            max_landing_vx: 120.0,
            max_landing_vy: 220.0,
            wall_crash_speed: 200.0,
            ground_drag: 6.0,
            integrity_max: 100.0,
            damage_k: 0.15,
            hazard_severity: 400.0,
            invuln_secs: 0.8,
            max_passengers: 4,
            spawn_secs_min: 2.0,
            spawn_secs_max: 6.0,
            walk_speed: 40.0, //px/s
            fare_base: 100.0,
            fare_per_px: 2.0,
            fare_min: 1.0,
            fare_decay: 2.0,
        }
    }
}

/// FixedUpdate pipeline stages. Configured once here
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub enum SimSet {
    Forces,  // velocity changes
    Move,    // position integration + collision resolution
    Contact, // game reaction to contacts
}

/// Shared by anything that moves under simulation
#[derive(Component, Default, Deref, DerefMut, Debug)]
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
            .configure_sets(
                FixedUpdate,
                (SimSet::Forces, SimSet::Move, SimSet::Contact)
                    .chain()
                    .run_if(in_state(IsPaused::Running))
                    .run_if(resource_exists::<TileGrid>),
            )
            .add_systems(FixedUpdate, apply_forces.in_set(SimSet::Forces))
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

///
fn apply_forces(
    config: Res<FlightConfig>,
    time: Res<Time>,
    mut query: Query<(
        &mut Velocity,
        &ThrustInput,
        Has<Grounded>,
        //&mut PhysicalTranslation,
        //&mut PreviousPhysicalTranslation,
    )>,
) {
    let dt = time.delta_secs();
    for (mut vel, input, grounded) in &mut query {
        vel.0 = step_velocity(vel.0, input.vertical, input.horizontal, &config, dt);
        if grounded {
            // the copter settles instead of ice-skating along platforms
            vel.x *= (-config.ground_drag * dt).exp();
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
            y += v.y * DT;
        }

        // Duty quantization (on_ticks is rounded) allows some drift
        assert!(y.abs() < 200.0, "drifter {} px over 10s of hover", y);
    }
}
