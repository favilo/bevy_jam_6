//! Player-specific behavior.

use bevy::prelude::*;
use bevy_ecs_ldtk::{GridCoords, LdtkEntity};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<PlayerObject>()
        .init_resource::<Wallet>();
}

#[derive(Resource, Reflect, Debug, Default, Clone)]
pub struct Wallet {
    pub gems: usize,
}

#[derive(Component, Debug, Clone, Copy, Reflect, Default)]
pub struct PlayerObject;

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct PlayerDirection(pub GridCoords);

impl Default for PlayerDirection {
    fn default() -> Self {
        PlayerDirection(GridCoords::new(1, 0))
    }
}

#[derive(Bundle, LdtkEntity, Default)]
pub struct PlayerBundle {
    player: PlayerObject,

    direction: PlayerDirection,

    #[sprite_sheet]
    sprite_sheet: Sprite,
    #[grid_coords]
    grid_coords: GridCoords,
}

impl std::fmt::Debug for PlayerBundle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PlayerBundle")
            .field("player", &self.player)
            .field("actions", &"Actions<Player>")
            .field("sprite_sheet", &self.sprite_sheet)
            .field("grid_coords", &self.grid_coords)
            .finish()
    }
}
