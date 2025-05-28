//! Spawn the main level.

use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        LoadingStateAppExt,
        config::{ConfigureLoadingState, LoadingStateConfig},
    },
};

use crate::{
    audio::music,
    demo::player::{PlayerAssets, player},
    screens::GameState,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<LevelAssets>(),
    );
}

#[derive(Resource, AssetCollection, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[asset(path = "audio/music/Fluffing A Duck.ogg")]
    music: Handle<AudioSource>,
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn((
        Name::new("Level"),
        Transform::default(),
        Visibility::default(),
        StateScoped(GameState::Playing),
        children![
            player(400.0, &player_assets, &mut texture_atlas_layouts),
            (
                Name::new("Gameplay Music"),
                music(level_assets.music.clone())
            )
        ],
    ));
}
