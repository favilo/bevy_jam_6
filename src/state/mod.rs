use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    app.add_sub_state::<ProgramState>();
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default, Reflect)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
    Paused,
}

#[derive(SubStates, Debug, Hash, PartialEq, Eq, Clone, Copy, Default, Reflect)]
#[source(GameState = GameState::Playing)]
pub enum ProgramState {
    #[default]
    /// The game is running normally, and you are buying upgrades.
    Buying,

    /// Upgrades are paused, and you are watching the outcome of the program.
    Running,
}
