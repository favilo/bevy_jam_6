//! The pause menu.

use bevy::prelude::*;
use bevy_enhanced_input::{
    events::Fired,
    prelude::{Actions, Binding, InputAction, InputContext, InputContextAppExt, Press},
};

use crate::{Pause, menu::Menu, state::GameState, theme::widget};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PauseContext>();
    app.add_input_context::<PauseContext>();
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_menu);
    app.add_systems(OnEnter(Menu::Pause), spawn_pause_context);
    app.add_observer(pause_binding);
    app.add_observer(go_back);
}

fn spawn_pause_menu(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Pause Menu"),
        GlobalZIndex(2),
        StateScoped(Menu::Pause),
        children![
            widget::header("Game paused"),
            widget::button("Continue", close_menu),
            widget::button("Settings", open_settings_menu),
            widget::button("Quit to title", quit_to_title),
        ],
    ));
}

#[derive(InputContext, Default, Debug, Reflect)]
#[input_context(priority = 10)]
pub struct PauseContext;

#[derive(InputAction, Debug, Reflect)]
#[input_action(output = bool, require_reset = true)]
pub struct GoBackAction;

fn spawn_pause_context(mut commands: Commands) {
    commands.spawn((
        Name::new("Pause Context"),
        StateScoped(Menu::Pause),
        Actions::<PauseContext>::default(),
    ));
}

fn pause_binding(
    trigger: Trigger<Binding<PauseContext>>,
    mut actions: Query<&mut Actions<PauseContext>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<GoBackAction>()
        .to((KeyCode::Escape, GamepadButton::Start))
        .with_conditions(Press::new(0.2));
}
fn open_settings_menu(_: Trigger<Pointer<Click>>, mut next_menu: ResMut<NextState<Menu>>) {
    next_menu.set(Menu::Settings);
}

fn close_menu(
    _: Trigger<Pointer<Click>>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut paused: ResMut<NextState<Pause>>,
) {
    next_menu.set(Menu::None);
    paused.set(Pause(false));
}

fn quit_to_title(
    _: Trigger<Pointer<Click>>,
    mut next_screen: ResMut<NextState<Menu>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    next_screen.set(Menu::Main);
    next_state.set(GameState::Menu);
}

fn go_back(
    _trigger: Trigger<Fired<GoBackAction>>,
    mut next_menu: ResMut<NextState<Menu>>,
    mut paused: ResMut<NextState<Pause>>,
) {
    tracing::debug!("Leaving pause menu");
    next_menu.set(Menu::None);
    paused.set(Pause(false));
}
