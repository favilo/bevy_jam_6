use bevy::prelude::*;

use crate::{state::ProgramState, theme::interaction::Inactive};

use super::{
    cpu::{CpuOptions, CpuState},
    level::{ResetButton, RunButton},
    objects::{BombTimer, TimeToBomb},
};

pub(super) fn plugin(app: &mut App) {
    app.add_event::<Tick>()
        .register_type::<Tick>()
        .register_type::<TickTimer>();
    app.add_systems(OnEnter(ProgramState::Running), begin_running_program)
        .add_systems(OnEnter(ProgramState::Buying), enter_buying)
        .add_systems(Update, tick_timer.run_if(in_state(ProgramState::Running)));
    app.add_observer(tick_printer);
}

fn enter_buying(
    mut commands: Commands,
    run_button: Query<Entity, With<RunButton>>,
    reset_button: Query<Entity, With<ResetButton>>,
) {
    let run_button = run_button.single().unwrap();
    commands.entity(run_button).remove::<Inactive>();
    let reset_button = reset_button.single().unwrap();
    commands.entity(reset_button).insert(Inactive);
}

fn begin_running_program(
    mut commands: Commands,
    cpu_options: Res<CpuOptions>,
    time_to_bomb: Res<TimeToBomb>,
    run_button: Query<Entity, With<RunButton>>,
    reset_button: Query<Entity, With<ResetButton>>,
) {
    commands.insert_resource(TickTimer {
        timer: Timer::new(
            cpu_options.cpu_tick.mul_f32(cpu_options.multiplier),
            TimerMode::Repeating,
        ),
    });
    commands.insert_resource(BombTimer {
        timer: Timer::new(
            time_to_bomb.duration.mul_f32(cpu_options.multiplier),
            TimerMode::Once,
        ),
    });
    commands.insert_resource(CpuState { pc: 0 });
    commands.trigger(Tick);
    let run_button = run_button.single().unwrap();
    commands.entity(run_button).insert(Inactive);
    let reset_button = reset_button.single().unwrap();
    commands.entity(reset_button).remove::<Inactive>();
}

fn tick_timer(mut commands: Commands, time: Res<Time>, mut timer: ResMut<TickTimer>) {
    if timer.timer.tick(time.delta()).just_finished() {
        commands.trigger(Tick);
    }
}

fn tick_printer(_: Trigger<Tick>) {
    tracing::info!("Tick event triggered");
}

#[derive(Event, Reflect, Debug, Clone)]
pub struct Tick;

#[derive(Resource, Reflect, Debug, Clone)]
pub struct TickTimer {
    pub(crate) timer: Timer,
}
pub fn start_simulation(
    trigger: Trigger<Pointer<Click>>,
    inactive: Query<Has<Inactive>>,
    mut next_state: ResMut<NextState<ProgramState>>,
) {
    if inactive.get(trigger.target()).unwrap_or_default() {
        return;
    }
    tracing::info!("Starting simulation");
    next_state.set(ProgramState::Running);
}

pub fn reset_simulation(
    trigger: Trigger<Pointer<Click>>,
    inactive: Query<Has<Inactive>>,
    mut next_state: ResMut<NextState<ProgramState>>,
) {
    if inactive.get(trigger.target()).unwrap_or_default() {
        return;
    }
    tracing::info!("Resetting simulation");
    next_state.set(ProgramState::Buying);
}
