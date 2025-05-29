//! The title screen that appears when the game starts.

use bevy::prelude::*;
#[cfg(feature = "dev_native")]
use bevy_simple_subsecond_system::hot;

use crate::{screens::MenuScreen, theme::prelude::*};

use super::GameState;

pub(super) fn plugin(app: &mut App) {
    app.add_systems(OnEnter(MenuScreen::Title), spawn_title_screen);
}

#[cfg_attr(feature = "dev_native", hot(rerun_on_hot_patch = true))]
fn spawn_title_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Title Screen"),
        StateScoped(MenuScreen::Title),
        #[cfg(not(target_family = "wasm"))]
        children![
            widget::button("Start", enter_gameplay_screen),
            widget::button("Settings", enter_settings_screen),
            widget::button("Credits", enter_credits_screen),
            widget::button("Exit", exit_app),
        ],
        #[cfg(target_family = "wasm")]
        children![
            widget::button("Start", enter_gameplay_screen),
            widget::button("Settings", enter_settings_screen),
            widget::button("Credits", enter_credits_screen),
        ],
    ));
}

fn enter_gameplay_screen(
    _: Trigger<Pointer<Click>>,
    mut next_game_state: ResMut<NextState<GameState>>,
) {
    next_game_state.set(GameState::Playing);
}

fn enter_settings_screen(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<MenuScreen>>,
) {
    next_screen.set(MenuScreen::Settings);
}

fn enter_credits_screen(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<MenuScreen>>,
) {
    next_screen.set(MenuScreen::Credits);
}
#[cfg(not(target_family = "wasm"))]
fn exit_app(_: Trigger<Pointer<Click>>, mut app_exit: EventWriter<AppExit>) {
    app_exit.write(AppExit::Success);
}
