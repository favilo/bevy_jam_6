use std::time::Duration;

use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        LoadingStateAppExt,
        config::{ConfigureLoadingState, LoadingStateConfig},
    },
};
use bevy_ecs_ldtk::{GridCoords, utils::grid_coords_to_translation};

use crate::{
    game::ticks::Tick,
    state::{GameState, ProgramState},
};

use super::player::PlayerDirection;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CpuOptions>()
        .register_type::<CpuSpeedDisplay>()
        .register_type::<ProgramCode>()
        .register_type::<Instruction>()
        .register_type::<CpuState>();
    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<InstructionAssets>(),
    );
    app.add_systems(
        FixedUpdate,
        update_cpu_speed_text.run_if(resource_exists_and_changed::<CpuOptions>),
    );
    app.add_observer(handle_tick)
        .add_observer(handle_instruction)
        .add_observer(move_forward);
}

#[derive(Resource, Reflect, Debug, Clone, Default)]
pub struct CpuOptions {
    pub cpu_tick: Duration,
    pub multiplier: f32,
}

#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
pub struct CpuSpeedDisplay;

fn update_cpu_speed_text(
    mut text: Query<&mut Text, With<CpuSpeedDisplay>>,
    cpu_options: Res<CpuOptions>,
) {
    for mut text in &mut text {
        **text = format!("{:?}", cpu_options.cpu_tick);
    }
}

#[allow(dead_code)]
#[derive(Reflect, Debug, Clone, Copy)]
pub enum InstructionType {
    Movement,
    Control,
    Scanning,
}

#[derive(Event, Reflect, Debug, Clone, Copy)]
pub enum Instruction {
    MoveForward,
}

impl Instruction {
    #[allow(dead_code)]
    pub fn inst_type(&self) -> InstructionType {
        match self {
            Instruction::MoveForward => InstructionType::Movement,
        }
    }
}

#[derive(Resource, AssetCollection, Clone, Reflect)]
#[reflect(Resource)]
pub struct InstructionAssets {
    #[asset(path = "images/instructions/movement.png")]
    pub movement: Handle<Image>,
    #[asset(path = "images/instructions/control.png")]
    pub control: Handle<Image>,
}

#[derive(Resource, Reflect, Debug, Clone, Default)]
pub struct ProgramCode {
    pub code: Vec<Instruction>,
}

#[derive(Resource, Reflect, Debug, Clone, Default)]
pub struct CpuState {
    pub pc: usize, // Program Counter
}

fn handle_tick(
    _: Trigger<Tick>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<ProgramState>>,
    mut cpu_state: ResMut<CpuState>,
    program_code: Res<ProgramCode>,
) {
    if cpu_state.pc < program_code.code.len() {
        commands.trigger(program_code.code[cpu_state.pc]);
        cpu_state.pc += 1; // Increment the program counter
    } else {
        tracing::info!("End of program reached.");
        next_state.set(ProgramState::Buying); // Transition to Buying state
    }
}

fn handle_instruction(trigger: Trigger<Instruction>, mut commands: Commands) {
    tracing::info!("Executing instruction: {:?}", trigger.event());
    match trigger.event() {
        Instruction::MoveForward => {
            commands.trigger(MoveForward);
            // Implement the logic for moving forward here
        }
    }
}

#[derive(Event, Reflect, Debug, Clone, Copy)]
pub struct MoveForward;

fn move_forward(
    _: Trigger<MoveForward>,
    mut player: Query<(&mut GridCoords, &mut Transform, &PlayerDirection)>,
) {
    for (mut grid_coords, mut transform, direction) in &mut player {
        *grid_coords += direction.0;
        transform.translation =
            grid_coords_to_translation(*grid_coords, IVec2::splat(18)).extend(0.0);
    }
}
