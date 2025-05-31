//! The title screen that appears when the game starts.

use bevy::prelude::*;

use crate::{menu::Menu, state::GameState};

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(GameState::Menu), open_main_menu);
    app.add_systems(OnExit(GameState::Menu), close_menu);
}

fn open_main_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Main)
}

fn close_menu(mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::None)
}
