//! The game's main screen states and transitions between them.

mod gameplay;
mod splash;
mod title;

use bevy::prelude::*;
use bevy_enhanced_input::{
    events::Fired,
    prelude::{Actions, Binding, InputAction, InputContext, InputContextAppExt, Press},
};

use crate::{menu::Menu, state::GameState};

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<MenuContext>();

    app.add_observer(go_back_to_parent_menu)
        .add_observer(menu_binding);

    app.add_plugins((gameplay::plugin, splash::plugin, title::plugin));

    app.add_systems(OnEnter(GameState::Menu), spawn_menu_context);

}

#[derive(InputContext, Default, Debug, Reflect)]
pub struct MenuContext;

#[derive(InputAction, Debug, Reflect)]
#[input_action(output = bool, require_reset = true)]
pub struct GoBackToParentScreen;

fn spawn_menu_context(mut commands: Commands) {
    commands.spawn((
        Name::new("Menu Context"),
        StateScoped(GameState::Menu),
        Actions::<MenuContext>::default(),
    ));
}

fn go_back_to_parent_menu(
    _: Trigger<Fired<GoBackToParentScreen>>,
    current_state: Res<State<Menu>>,
    mut next_state: ResMut<NextState<Menu>>,
) {
    tracing::info!("Going back to parent menu");
    if current_state.get() == &Menu::Main {
        return;
    }
    next_state.set(Menu::Main);
}

fn menu_binding(
    trigger: Trigger<Binding<MenuContext>>,
    mut actions: Query<&mut Actions<MenuContext>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<GoBackToParentScreen>()
        .to((KeyCode::Escape, GamepadButton::Start))
        .with_conditions(Press::new(0.2));
}
