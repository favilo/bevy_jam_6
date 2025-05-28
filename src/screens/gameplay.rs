//! The screen state for the main gameplay.

use bevy::prelude::*;

use crate::{demo::level::spawn_level, screens::GameState};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Playing), spawn_level);
}
