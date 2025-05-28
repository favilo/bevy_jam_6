//! The game's main screen states and transitions between them.

mod credits;
mod gameplay;
mod settings;
mod splash;
mod title;

use bevy::prelude::*;
use bevy_enhanced_input::{
    events::Fired,
    prelude::{Actions, Binding, InputAction, InputContext, InputContextAppExt, Press},
};

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<MenuContext>();

    app.add_observer(go_back_to_parent_screen)
        .add_observer(menu_binding);

    app.add_plugins((
        credits::plugin,
        gameplay::plugin,
        settings::plugin,
        splash::plugin,
        title::plugin,
    ));

    app.add_systems(OnEnter(GameState::Menu), spawn_menu_context);
}

#[derive(States, Debug, Hash, PartialEq, Eq, Clone, Copy, Default, Reflect)]
#[states(scoped_entities)]
pub enum GameState {
    #[default]
    Loading,
    Menu,
    Playing,
}

/// The game's main screen states.
#[derive(SubStates, Debug, Hash, PartialEq, Eq, Clone, Copy, Default, Reflect)]
#[source(GameState = GameState::Menu)]
#[states(scoped_entities)]
pub enum MenuScreen {
    #[default]
    Title,
    Credits,
    Settings,
}

impl MenuScreen {
    pub fn parent_screen(self) -> Self {
        MenuScreen::Title
    }
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

fn go_back_to_parent_screen(
    _: Trigger<Fired<GoBackToParentScreen>>,
    current_screen: Res<State<MenuScreen>>,
    mut next_screen: ResMut<NextState<MenuScreen>>,
) {
    if current_screen.get() == &MenuScreen::Title {
        return;
    }
    next_screen.set(current_screen.parent_screen());
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
