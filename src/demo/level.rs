//! Spawn the main level.

use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        LoadingStateAppExt,
        config::{ConfigureLoadingState, LoadingStateConfig},
    },
};
use bevy_ecs_tilemap::{
    TilemapBundle,
    anchor::TilemapAnchor,
    map::{TilemapGridSize, TilemapId, TilemapSize, TilemapTexture, TilemapTileSize, TilemapType},
    prelude::fill_tilemap_rect,
    tiles::{TilePos, TileStorage, TileTextureIndex},
};

use crate::{
    audio::music,
    demo::player::{PlayerAssets, player},
    state::GameState,
};

pub const LEVEL_SCALE_FACTOR: f32 = 4.0;

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

    #[asset(path = "images/kenney/platformer_pack/Tilemap/marble_packed.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub tiles: Handle<Image>,
}

#[derive(Resource, Clone, Copy, Debug, Reflect)]
pub struct TilemapMetadata {
    pub map_size: TilemapSize,
    pub tile_size: TilemapTileSize,
    pub grid_size: TilemapGridSize,
    pub scale_factor: f32,
}

/// A system that spawns the main level.
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    player_assets: Res<PlayerAssets>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    let tilemap_entity = commands.spawn_empty().id();
    let width = 10;
    let height = 10;
    let map_size = TilemapSize {
        x: width,
        y: height,
    };
    let tile_size = TilemapTileSize::new(18.0, 18.0); // Example tile size
    let grid_size = tile_size.into();
    let metadata = TilemapMetadata {
        map_size,
        tile_size,
        grid_size,
        scale_factor: LEVEL_SCALE_FACTOR,
    };

    let mut storage = TileStorage::empty(map_size);
    fill_tilemap_rect(
        TileTextureIndex(18),
        TilePos::new(0, 0),
        map_size,
        TilemapId(tilemap_entity),
        &mut commands,
        &mut storage,
    );
    commands.entity(tilemap_entity).insert((
        Name::new("Level"),
        TilemapBundle {
            grid_size,
            map_type: TilemapType::Square,
            size: map_size,
            storage,
            texture: TilemapTexture::Single(level_assets.tiles.clone()),
            tile_size,
            transform: Transform::from_scale(Vec2::splat(metadata.scale_factor).extend(1.0)),
            anchor: TilemapAnchor::Center,
            ..default()
        },
        StateScoped(GameState::Playing),
        children![(
            Name::new("Gameplay Music"),
            music(level_assets.music.clone())
        ),],
    ));

    let player_bundle = player(200.0, &player_assets, &mut texture_atlas_layouts, &metadata);
    commands.spawn(player_bundle);
}
