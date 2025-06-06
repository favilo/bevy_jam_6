use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity};

use crate::state::ProgramState;

use super::player::{PlayerObject, Wallet};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<GemObject>();
    app.add_systems(
        FixedUpdate,
        (
            update_time_to_bomb_text.run_if(resource_exists_and_changed::<TimeToBomb>),
            update_gem_text.run_if(resource_exists_and_changed::<Wallet>),
            pickup_gem.run_if(in_state(ProgramState::Running)),
        ),
    );
    app.add_systems(
        Update,
        (tick_bomb_timer.run_if(in_state(ProgramState::Running)),),
    );
    app.add_observer(bomb_exploded);
}

#[derive(Component, Default, Reflect, Debug, Clone)]
pub struct GemObject;

#[derive(Bundle, LdtkEntity, Default)]
pub struct GemBundle {
    player: GemObject,

    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Component, Reflect, Debug, Default, Clone, Copy)]
pub(crate) struct GemDisplay;

fn update_gem_text(mut text: Query<&mut Text, With<GemDisplay>>, wallet: Res<Wallet>) {
    for mut text in &mut text {
        **text = wallet.gems.to_string();
    }
}

#[derive(Component, Reflect, Debug, Default, Clone, Copy)]
pub(crate) struct TimeToBombDisplay;

#[derive(Resource, Reflect, Debug, Default, Clone)]
pub struct TimeToBomb {
    pub duration: Duration,
}

#[derive(Resource, Reflect, Debug, Default, Clone)]
pub struct BombTimer {
    pub timer: Timer,
}

#[derive(Event, Reflect, Debug, Clone, Copy)]
pub struct BombExploded;

fn update_time_to_bomb_text(
    mut text: Query<&mut Text, With<TimeToBombDisplay>>,
    time_to_bomb: Res<TimeToBomb>,
) {
    for mut text in &mut text {
        **text = format!("{:?}", time_to_bomb.duration);
    }
}

fn tick_bomb_timer(mut commands: Commands, time: Res<Time>, mut bomb_timer: ResMut<BombTimer>) {
    if bomb_timer.timer.tick(time.delta()).just_finished() {
        commands.trigger(BombExploded);
    }
}

fn bomb_exploded(_: Trigger<BombExploded>, mut next_state: ResMut<NextState<ProgramState>>) {
    tracing::info!("Bomb exploded!");
    next_state.set(ProgramState::Buying);
}

fn pickup_gem(
    mut commands: Commands,
    players: Query<&GridCoords, With<PlayerObject>>,
    gems: Query<(Entity, &GridCoords), With<GemObject>>,
    mut wallet: ResMut<Wallet>,
) {
    let player_coords = players.single().unwrap();
    for (gem_entity, gem_coords) in &gems {
        if *gem_coords == *player_coords {
            commands.entity(gem_entity).despawn();
            wallet.gems += 1;
            tracing::info!("Player picked up a gem at {:?}", gem_coords);
            // Here you could also add logic to increase the player's score or inventory.
        }
    }
}
