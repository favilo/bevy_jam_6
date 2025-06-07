use bevy::{ecs::relationship::RelatedSpawnerCommands, prelude::*};

#[cfg(feature = "dev_native")]
use bevy_simple_subsecond_system::hot;

use crate::{
    game::{
        cpu::CpuOptions,
        level::{UpgradeParent, spawn_level_ui},
    },
    state::GameState,
    theme::{interaction::Inactive, widget},
};

use super::{
    cpu::{Instruction, ProgramCode, UnlockedInstructions},
    player::Wallet,
};

pub(crate) fn plugin(app: &mut App) {
    app.add_event::<UpgradeBought>();
    app.register_type::<UpgradeTree>()
        .register_type::<Upgrade>()
        .register_type::<UpgradeNode>();
    app.init_resource::<UpgradeTree>();
    app.add_systems(
        OnEnter(GameState::Playing),
        setup_upgrade_buttons.after(spawn_level_ui),
    )
    .add_systems(
        FixedUpdate,
        activate_upgrade_buttons
            .run_if(resource_exists_and_changed::<Wallet>)
            .run_if(in_state(GameState::Playing)),
    );
    app.add_observer(apply_upgrade);
}

#[derive(Component, Reflect, Debug, Clone, Copy, Deref, DerefMut)]
pub struct UpgradeNode(pub petgraph::graph::NodeIndex);

#[derive(Event, Reflect, Debug, Clone, Copy)]
pub struct UpgradeBought {
    pub upgrade_type: UpgradeType,
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub enum UpgradeType {
    CpuSpeed,
    CpuMultiplier,
    MaxInstructions,
    UnlockIf,
}

impl std::fmt::Display for UpgradeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UpgradeType::CpuSpeed => write!(f, "CPU Speed x2"),
            UpgradeType::CpuMultiplier => write!(f, "CPU Multiplier x2"),
            UpgradeType::MaxInstructions => write!(f, "Max Instructions x2"),
            UpgradeType::UnlockIf => write!(f, "Unlock If"),
        }
    }
}

#[derive(Reflect, Debug, Clone, PartialEq, Eq, Hash, Copy)]
pub struct Upgrade {
    pub upgrade_type: UpgradeType,
    pub level: u32,
    pub cost: usize,
    bought: bool,
    entity: Option<Entity>,
}

impl std::fmt::Display for Upgrade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.upgrade_type)
    }
}

impl Upgrade {
    fn cpu_speed(level: u32, cost: usize) -> Self {
        Self::new(UpgradeType::CpuSpeed, level, cost)
    }

    fn cpu_multiplier(level: u32, cost: usize) -> Self {
        Self::new(UpgradeType::CpuMultiplier, level, cost)
    }

    fn max_instructions(level: u32, cost: usize) -> Self {
        Self::new(UpgradeType::MaxInstructions, level, cost)
    }

    fn new(upgrade_type: UpgradeType, level: u32, cost: usize) -> Self {
        Upgrade {
            upgrade_type,
            level,
            cost,
            bought: false,
            entity: None,
        }
    }
}

#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource, opaque)]
pub struct UpgradeTree {
    pub deps: petgraph::Graph<Upgrade, (), petgraph::Directed>,
    roots: Vec<petgraph::graph::NodeIndex>,
}

impl Default for UpgradeTree {
    fn default() -> Self {
        let mut deps = petgraph::Graph::new();
        let max_insts = (1_u32..=4)
            .map(|i| Upgrade::max_instructions(i, 10 * 2_usize.pow(i) - 10))
            .map(|u| deps.add_node(u))
            .collect::<Vec<_>>();
        let cpu_speeds = (1_u32..=5)
            .map(|i| Upgrade::cpu_speed(i, 10 * 3_usize.pow(i)))
            .map(|u| deps.add_node(u))
            .collect::<Vec<_>>();
        let unlock_if = deps.add_node(Upgrade::new(UpgradeType::UnlockIf, 1, 100));

        deps.add_edge(max_insts[0], cpu_speeds[0], ());
        deps.add_edge(cpu_speeds[0], cpu_speeds[1], ());
        deps.add_edge(cpu_speeds[0], max_insts[1], ());
        deps.add_edge(cpu_speeds[1], cpu_speeds[2], ());
        deps.add_edge(cpu_speeds[1], max_insts[2], ());
        deps.add_edge(cpu_speeds[1], unlock_if, ());
        deps.add_edge(cpu_speeds[2], cpu_speeds[3], ());
        deps.add_edge(cpu_speeds[2], max_insts[3], ());
        deps.add_edge(cpu_speeds[3], cpu_speeds[4], ());

        UpgradeTree {
            deps,
            roots: vec![max_insts[0]],
        }
    }
}

#[cfg_attr(feature = "dev_native", hot(rerun_on_hot_patch = true))]
fn setup_upgrade_buttons(
    mut commands: Commands,
    parent: Single<Entity, With<UpgradeParent>>,
    children: Query<&Children>,
) {
    let parent = *parent;
    for child in children.get(parent).unwrap().iter() {
        commands.entity(child).despawn();
    }
    let mut upgrade_tree = UpgradeTree::default();
    commands.entity(parent).with_children(|parent| {
        let roots = std::mem::take(&mut upgrade_tree.roots);
        for root in roots.iter().cloned() {
            spawn_upgrade_button(parent, &mut upgrade_tree, root);
        }
        upgrade_tree.roots = roots;
    });
    commands.insert_resource(upgrade_tree);
}

fn spawn_upgrade_button(
    parent: &mut RelatedSpawnerCommands<ChildOf>,
    upgrade_tree: &mut UpgradeTree,
    idx: petgraph::graph::NodeIndex,
) {
    let upgrade = upgrade_tree
        .deps
        .node_weight_mut(idx)
        .expect("Upgrade node exists");
    if upgrade.bought {
        tracing::debug!("Upgrade already bought: {upgrade}");
        let neighbors = upgrade_tree
            .deps
            .neighbors_directed(idx, petgraph::Direction::Outgoing)
            .collect::<Vec<_>>();
        tracing::info!("Spawning upgrade buttons for neighbors: {:?}", neighbors);
        for neighbor in neighbors {
            spawn_upgrade_button(parent, upgrade_tree, neighbor);
        }
        return;
    }
    let name = upgrade.to_string();
    let button_entity = parent
        .spawn((widget::button_upgrade(
            name.clone(),
            upgrade.cost,
            move |trigger: Trigger<Pointer<Click>>,
                  mut commands: Commands,
                  parent: Single<Entity, With<UpgradeParent>>,
                  inactive: Query<Has<Inactive>>,
                  mut upgrade_tree: ResMut<UpgradeTree>,
                  mut wallet: ResMut<Wallet>| {
                if inactive.get(trigger.target()).unwrap_or_default() {
                    return;
                }

                let upgrade = upgrade_tree.deps.node_weight_mut(idx).unwrap();
                if wallet.gems >= upgrade.cost && !upgrade.bought {
                    wallet.gems -= upgrade.cost;
                    upgrade.bought = true;

                    tracing::info!("Bought upgrade: {upgrade}");
                    let entity = upgrade.entity.take().expect("Button entity set");
                    tracing::debug!("removing button entity: {entity}");
                    commands.trigger(UpgradeBought {
                        upgrade_type: upgrade.upgrade_type,
                    });
                    commands.entity(entity).despawn();
                    commands.entity(*parent).with_children(|parent| {
                        spawn_upgrade_button(parent, &mut upgrade_tree, idx);
                    });
                } else {
                    tracing::warn!("Not enough gems or already bought: {upgrade}",);
                }
            },
            (Inactive, UpgradeNode(idx)),
        ),))
        .id();
    upgrade.entity = Some(button_entity);
}

#[cfg_attr(feature = "dev_native", hot)]
fn activate_upgrade_buttons(
    mut commands: Commands,
    upgrade_tree: Res<UpgradeTree>,
    query: Query<(Entity, &UpgradeNode)>,
    wallet: Res<Wallet>,
) {
    for (entity, upgrade_node) in query.iter() {
        if let Some(upgrade) = upgrade_tree.deps.node_weight(**upgrade_node) {
            if wallet.gems >= upgrade.cost {
                tracing::debug!(
                    "Activating upgrade button for {upgrade}: {} >= {}",
                    wallet.gems,
                    upgrade.cost
                );
                commands.entity(entity).remove::<Inactive>();
            } else {
                tracing::debug!(
                    "Deactivating upgrade button for {upgrade}: {} < {}",
                    wallet.gems,
                    upgrade.cost
                );
                commands.entity(entity).insert(Inactive);
            }
        }
    }
}

fn apply_upgrade(
    trigger: Trigger<UpgradeBought>,
    mut cpu_options: ResMut<CpuOptions>,
    mut program_code: ResMut<ProgramCode>,
    mut unlocked_instructions: ResMut<UnlockedInstructions>,
) {
    match trigger.event().upgrade_type {
        UpgradeType::CpuSpeed => {
            cpu_options.cpu_tick /= 2;
            tracing::info!(
                "Applied CPU Speed upgrade: new speed = {:?}",
                cpu_options.cpu_tick
            );
        }
        UpgradeType::CpuMultiplier => {
            cpu_options.multiplier *= 2.0;
            tracing::info!(
                "Applied CPU Multiplier upgrade: new multiplier = {}",
                cpu_options.multiplier
            );
        }
        UpgradeType::MaxInstructions => {
            program_code.max_instructions *= 2;
            tracing::info!(
                "Applied Max Instructions upgrade: new max instructions = {}",
                program_code.max_instructions
            );
        }
        UpgradeType::UnlockIf => {
            unlocked_instructions.0.insert(
                Instruction::IfGapTurnLeft.inst_type(),
                Instruction::IfGapTurnLeft,
            );
            tracing::info!("Applied Unlock If upgrade: now unlocked IfGapTurnLeft instruction");
        }
    }
}
