use std::time::Duration;

use bevy::{color::palettes::css::*, ecs::relationship::RelatedSpawnerCommands, prelude::*};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        LoadingStateAppExt,
        config::{ConfigureLoadingState, LoadingStateConfig},
    },
};
use bevy_ecs_ldtk::{GridCoords, utils::grid_coords_to_translation};
#[cfg(feature = "dev_native")]
use bevy_simple_subsecond_system::hot;
use multimap::MultiMap;

use crate::{
    game::ticks::Tick,
    state::{GameState, ProgramState},
    theme::widget,
};

use super::{
    level::{CommandParent, ProgramParent, spawn_level_ui},
    player::PlayerDirection,
};

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
        OnEnter(GameState::Playing),
        (setup_program_code, setup_unlocked_instructions).after(spawn_level_ui),
    );
    app.add_systems(OnExit(GameState::Playing), cleanup_resources);
    app.add_systems(
        FixedUpdate,
        (
            update_cpu_speed_text.run_if(resource_exists_and_changed::<CpuOptions>),
            update_program_code.run_if(resource_exists_and_changed::<ProgramCode>),
            update_command_palette.run_if(resource_exists_and_changed::<UnlockedInstructions>),
        )
            .run_if(in_state(GameState::Playing)),
    );
    app.add_observer(handle_tick)
        .add_observer(handle_instruction)
        .add_observer(move_forward)
        .add_observer(if_gap_turn_left);
}

#[derive(Resource, Reflect, Debug, Clone, Default)]
#[reflect(Resource)]
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
#[derive(Reflect, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstructionType {
    Movement,
    Control,
    Scanning,
}

#[derive(Event, Reflect, Debug, Clone, Copy)]
pub enum Instruction {
    MoveForward,
    IfGapTurnLeft,
}

impl Instruction {
    #[allow(dead_code)]
    pub fn inst_type(&self) -> InstructionType {
        match self {
            Instruction::MoveForward => InstructionType::Movement,
            Instruction::IfGapTurnLeft => InstructionType::Scanning,
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

#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct ProgramCode {
    pub code: Vec<Instruction>,
    pub max_instructions: usize,
}

impl Default for ProgramCode {
    fn default() -> Self {
        ProgramCode {
            code: vec![],
            max_instructions: 1,
        }
    }
}

#[derive(Resource, Reflect, Debug, Clone, Default)]
#[reflect(Resource)]
pub struct CpuState {
    pub pc: usize,
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
        cpu_state.pc += 1;
    } else {
        tracing::info!("End of program reached.");
        next_state.set(ProgramState::Buying);
    }
}

fn handle_instruction(trigger: Trigger<Instruction>, mut commands: Commands) {
    tracing::info!("Executing instruction: {:?}", trigger.event());
    match trigger.event() {
        Instruction::MoveForward => {
            commands.trigger(MoveForward);
        }
        Instruction::IfGapTurnLeft => {
            commands.trigger(IfGapTurnLeft);
        }
    }
}

#[derive(Event, Reflect, Debug, Clone, Copy)]
pub struct MoveForward;

#[derive(Event, Reflect, Debug, Clone, Copy)]
pub struct IfGapTurnLeft;

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

fn if_gap_turn_left(
    _: Trigger<IfGapTurnLeft>,
    mut player: Query<(&mut PlayerDirection, &mut Transform)>,
) {
    for (mut direction, mut transform) in &mut player {
        // Turn left by rotating 90 degrees counter-clockwise
        direction.0 = GridCoords::new(direction.0.y, -direction.0.x);
        let angle = Vec2::X.angle_to(Vec2::new(direction.0.x as f32, direction.0.y as f32));
        transform.rotation = Quat::from_rotation_z(-angle);
    }
}

#[derive(Resource, Reflect, Debug, Clone, Deref, DerefMut)]
#[reflect(Resource, opaque)]
pub struct UnlockedInstructions(pub MultiMap<InstructionType, Instruction>);

impl Default for UnlockedInstructions {
    fn default() -> Self {
        UnlockedInstructions(
            multimap::multimap!( InstructionType::Movement => Instruction::MoveForward ),
        )
    }
}

#[cfg_attr(feature = "dev_native", hot(rerun_on_hot_reload = true))]
fn setup_unlocked_instructions(mut commands: Commands) {
    commands.init_resource::<UnlockedInstructions>();
}

#[cfg_attr(feature = "dev_native", hot(rerun_on_hot_reload = true))]
fn setup_program_code(mut commands: Commands) {
    tracing::info!("Setting up program code...");
    let program_code = ProgramCode {
        code: vec![Instruction::MoveForward],
        max_instructions: 1,
    };
    commands.insert_resource(program_code.clone());
}

fn cleanup_resources(mut commands: Commands) {
    commands.remove_resource::<ProgramCode>();
    commands.remove_resource::<CpuOptions>();
    commands.remove_resource::<CpuState>();
    commands.remove_resource::<UnlockedInstructions>();
}

#[cfg_attr(feature = "dev_native", hot)]
fn update_command_palette(
    mut commands: Commands,
    parent: Single<Entity, With<CommandParent>>,
    children: Query<&Children>,
    unlocked_instructions: Res<UnlockedInstructions>,
) {
    let parent = *parent;
    for child in children.get(parent).unwrap().iter() {
        commands.entity(child).despawn();
    }

    commands.entity(parent).with_children(|parent| {
        for inst_type in [
            InstructionType::Movement,
            InstructionType::Control,
            InstructionType::Scanning,
        ] {
            if let Some(instructions) = unlocked_instructions.get_vec(&inst_type) {
                spawn_instruction_group(parent, inst_type, instructions);
            } else {
                tracing::warn!("No instructions found for type: {:?}", inst_type);
            }
        }
    });
}

fn spawn_instruction_group(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    inst_type: InstructionType,
    instructions: &[Instruction],
) {
    tracing::info!("Spawning instruction group: {:?}", inst_type);
    parent
        .spawn((
            Name::new(format!("{inst_type:?} Instructions")),
            Node {
                flex_direction: FlexDirection::Column,
                width: Val::Percent(100.0),
                margin: UiRect::all(Val::Px(3.0)),
                padding: UiRect::vertical(Val::Px(15.0)),
                border: UiRect::top(Val::Px(25.0)),
                ..default()
            },
            Text::new(format!("{inst_type:?}")),
            TextFont::from_font_size(20.0),
            TextColor(MEDIUM_AQUAMARINE.into()),
        ))
        .with_children(|parent| {
            for instruction in instructions.iter().copied() {
                parent.spawn((
                    Name::new(format!("Instruction: {instruction:?}")),
                    widget::ui_row(children![
                        (
                            Node {
                                width: Val::Percent(100.0),
                                margin: UiRect::all(Val::Px(5.0)),
                                ..default()
                            },
                            Text::new(format!("{instruction:?}")),
                            TextFont::from_font_size(18.0),
                            TextColor(BLANCHED_ALMOND.into()),
                        ),
                        widget::button_small("+", move |_: Trigger<Pointer<Click>>, mut program_code: ResMut<ProgramCode>| {
                            tracing::info!("Adding instruction: {:?}", instruction);
                            if program_code.code.len() < program_code.max_instructions {
                                program_code.code.push(instruction);
                            } else {
                                tracing::warn!("Maximum instruction limit reached: {}", program_code.max_instructions);
                            }
                        }),
                    ]),
                ));
            }
        });
}

fn spawn_instruction_item(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    idx: usize,
    instruction: Instruction,
) {
    tracing::info!("Spawning instruction item: {:?}", instruction);
    parent.spawn((
        Name::new(format!("Instruction: {instruction:?}")),
        Node {
            flex_direction: FlexDirection::Row,
            grid_column: GridPlacement::start(2),
            grid_row: GridPlacement::start(idx as i16 + 1),
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            margin: UiRect::all(Val::Px(3.0)),
            ..default()
        },
        children![
            (
                Text::new(format!("{instruction:?}")),
                TextFont::from_font_size(18.0),
                TextColor(BLANCHED_ALMOND.into()),
            ),
            widget::button_small(
                "-",
                move |_: Trigger<Pointer<Click>>, mut program_code: ResMut<ProgramCode>| {
                    tracing::info!("Removing instruction: {:?}", instruction);
                    program_code.code.remove(idx);
                }
            )
        ],
    ));
}

#[cfg_attr(feature = "dev_native", hot)]
fn update_program_code(
    program_code: Res<ProgramCode>,
    parent: Single<Entity, With<ProgramParent>>,
    children: Query<&Children>,
    mut commands: Commands,
) {
    let parent = *parent;
    for child in children.get(parent).unwrap().iter() {
        commands.entity(child).despawn();
    }

    commands.entity(parent).with_children(|parent| {
        for i in 0..program_code.max_instructions {
            parent.spawn((
                Name::new(format!("Instruction Slot {i}")),
                Node {
                    grid_column: GridPlacement::start(1),
                    grid_row: GridPlacement::start(i as i16 + 1),
                    width: Val::Auto,
                    height: Val::Auto,
                    margin: UiRect::all(Val::Px(3.0)),
                    ..default()
                },
                Text::new(format!("{i}")),
                TextFont::from_font_size(18.0),
                TextColor(MEDIUM_SPRING_GREEN.into()),
            ));
        }

        // Spawn the instruction items
        for (i, instruction) in program_code.code.iter().copied().enumerate() {
            spawn_instruction_item(parent, i, instruction);
        }
    });
}
