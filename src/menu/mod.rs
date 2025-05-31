use bevy::prelude::*;

mod credits;
mod main;
mod pause;
mod settings;

pub(super) fn plugin(app: &mut App) {
    app.init_state::<Menu>();

    app.add_plugins((
        main::plugin,
        credits::plugin,
        settings::plugin,
        pause::plugin,
    ));
}

#[derive(States, Debug, Clone, Copy, Eq, PartialEq, Hash, Reflect, Default)]
#[states(scoped_entities)]
pub enum Menu {
    #[default]
    None,
    Main,
    Credits,
    Settings,
    Pause,
}
