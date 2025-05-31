use bevy::prelude::*;

pub(super) fn plugin(app: &mut App) {
    // TODO
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
