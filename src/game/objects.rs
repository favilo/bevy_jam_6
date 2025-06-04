use std::time::Duration;

use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<GemObject>();
}

#[derive(Component, Default, Reflect, Debug, Clone)]
pub struct GemObject;

#[derive(Bundle, LdtkEntity, Default)]
pub struct GemBundle {
    player: GemObject,

    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
}

#[derive(Component, Reflect, Debug, Default, Clone, Copy)]
pub(crate) struct GemDisplay;

#[derive(Component, Reflect, Debug, Default, Clone, Copy)]
pub(crate) struct TimeToBombDisplay;

#[derive(Component, Reflect, Debug, Default, Clone, Copy)]
pub struct TimeToBomb {
    duration: Duration,
}
