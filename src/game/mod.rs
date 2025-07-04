//! Demo gameplay. All of these modules are only intended for demonstration
//! purposes and should be replaced with your own game logic.
//! Feel free to change the logic found here if you feel like tinkering around
//! to get a feeling for the template.

use bevy::prelude::*;

mod animation;
pub mod cpu;
pub mod level;
mod movement;
pub mod objects;
pub mod player;
pub mod ticks;
pub mod upgrades;

pub(super) fn plugin(app: &mut App) {
    app.add_plugins((
        animation::plugin,
        cpu::plugin,
        level::plugin,
        movement::plugin,
        objects::plugin,
        player::plugin,
        ticks::plugin,
        upgrades::plugin,
    ));
}
