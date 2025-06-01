//! Player-specific behavior.

use bevy::prelude::*;
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        LoadingStateAppExt,
        config::{ConfigureLoadingState, LoadingStateConfig},
    },
};
use bevy_ecs_tilemap::{anchor::TilemapAnchor, map::TilemapType, tiles::TilePos};
use bevy_enhanced_input::{
    events::Fired,
    prelude::{Actions, Binding, DeadZone, InputAction, InputContext, InputContextAppExt, Press},
    preset::{Axial, Cardinal},
};

use crate::{
    AppSystems, PausableSystems, Pause,
    demo::{
        animation::PlayerAnimation,
        level::TilemapMetadata,
        movement::{MovementController, ScreenWrap},
    },
    menu::Menu,
    state::GameState,
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.register_type::<PlayerAssets>();
    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<PlayerAssets>(),
    );
    app.add_input_context::<Player>();

    // Record directional input as movement controls.
    app.add_observer(player_binding);
    app.add_observer(pause_game);
    app.add_systems(
        Update,
        record_player_directional_input
            .in_set(AppSystems::RecordInput)
            .in_set(PausableSystems),
    );
}

/// The player character.
pub fn player(
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
    metadata: &TilemapMetadata,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    const ROBOT_TILE_SIZE: UVec2 = UVec2::new(148, 154);

    let layout = TextureAtlasLayout::from_grid(ROBOT_TILE_SIZE, 2, 1, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    let translation = (TilePos::new(0, 0).center_in_world(
        &metadata.map_size,
        &metadata.grid_size,
        &metadata.tile_size,
        &TilemapType::Square,
        &TilemapAnchor::Center,
    ) * metadata.scale_factor)
        .extend(1.0);
    let transform =
        Transform::from_scale(Vec2::splat(0.33).extend(1.0)).with_translation(translation);
    (
        Name::new("Player"),
        Actions::<Player>::default(),
        Sprite {
            image: player_assets.robot.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            }),
            ..default()
        },
        transform,
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
        player_animation,
    )
}

#[derive(InputContext, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[input_context(priority = 0)]
struct Player;

#[derive(InputAction, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[input_action(output = bool)]
struct PauseGame;

#[derive(InputAction, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[input_action(output = Vec2)]
struct Move;

fn player_binding(trigger: Trigger<Binding<Player>>, mut actions: Query<&mut Actions<Player>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<PauseGame>()
        .to((KeyCode::Escape, GamepadButton::Start))
        .with_conditions(Press::new(0.2));

    actions
        .bind::<Move>()
        .to((
            Cardinal::wasd_keys(),
            Cardinal::arrow_keys(),
            Axial::left_stick(),
        ))
        .with_modifiers(DeadZone::default());
}

fn record_player_directional_input(
    mut controller_query: Query<(&mut MovementController, &Actions<Player>)>,
) {
    for (mut controller, actions) in &mut controller_query {
        // Collect directional input from the Move action.
        let intent = actions.value::<Move>().unwrap().as_axis2d();

        // Normalize intent so that diagonal movement is the same speed as horizontal / vertical.
        // This should be omitted if the input comes from an analog stick instead.
        controller.intent = intent.normalize_or_zero();
    }
}

fn pause_game(
    _: Trigger<Fired<PauseGame>>,
    mut next_screen: ResMut<NextState<Menu>>,
    mut next_pause: ResMut<NextState<Pause>>,
) {
    tracing::debug!("Pausing the game");
    next_screen.set(Menu::Pause);
    next_pause.set(Pause(true));
}

#[derive(Resource, AssetCollection, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[asset(path = "images/robot_3Dblue-sheet.png")]
    #[asset(image(sampler(filter = nearest)))]
    robot: Handle<Image>,
    // #[asset(
    //     paths(
    //         "audio/sound_effects/kenney_scifi/spaceEngineLow_000.ogg",
    //         "audio/sound_effects/kenney_scifi/spaceEngineLow_001.ogg",
    //         "audio/sound_effects/kenney_scifi/spaceEngineLow_002.ogg",
    //         "audio/sound_effects/kenney_scifi/spaceEngineLow_003.ogg",
    //         "audio/sound_effects/kenney_scifi/spaceEngineLow_004.ogg",
    //     ),
    //     collection(typed)
    // )]
    // pub moving_sounds: Vec<Handle<AudioSource>>,
}
