// Support configuring Bevy lints within code.
#![cfg_attr(bevy_lint, feature(register_tool), register_tool(bevy))]
// Disable console on Windows for non-dev builds.
#![cfg_attr(not(feature = "dev"), windows_subsystem = "windows")]

mod audio;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod menu;
mod screens;
mod state;
mod theme;

use bevy::{asset::AssetMetaCheck, prelude::*};
use bevy_asset_loader::loading_state::{LoadingState, LoadingStateAppExt};
use bevy_ecs_ldtk::LdtkPlugin;
use bevy_ecs_tilemap::TilemapPlugin;
use bevy_enhanced_input::EnhancedInputPlugin;
use iyes_progress::ProgressPlugin;

use crate::state::GameState;

fn main() -> AppExit {
    App::new().add_plugins(AppPlugin).run()
}

pub struct AppPlugin;

impl Plugin for AppPlugin {
    fn build(&self, app: &mut App) {
        // Add Bevy plugins.
        app.add_plugins((
            DefaultPlugins
                .set(AssetPlugin {
                    // WASM builds will check for meta files (that don't exist) if this isn't set.
                    // This causes errors and even panics on web build on itch.
                    // See https://github.com/bevyengine/bevy_github_ci_template/issues/48.
                    meta_check: AssetMetaCheck::Never,
                    ..default()
                })
                .set(WindowPlugin {
                    primary_window: Window {
                        title: "Bevy Jam 6".to_string(),
                        fit_canvas_to_parent: true,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
            EnhancedInputPlugin,
            ProgressPlugin::<GameState>::new()
                .with_state_transition(GameState::Loading, GameState::Menu),
            TilemapPlugin,
            LdtkPlugin,
        ));

        app.init_state::<GameState>();
        app.add_loading_state(LoadingState::new(GameState::Loading));

        // Add other plugins.
        // #[cfg(not(feature = "dev_native"))]
        app.add_plugins((
            game::plugin,
            #[cfg(feature = "dev")]
            dev_tools::plugin,
            menu::plugin,
            screens::plugin,
            state::plugin,
            theme::plugin,
        ));

        // Hot patch other plugins in native dev builds.
        // #[cfg(feature = "dev_native")]
        // app.with_hot_patch(|app| {
        //     app.add_plugins((
        //         demo::plugin,
        //         dev_tools::plugin,
        //         menu::plugin,
        //         screens::plugin,
        //         state::plugin,
        //         theme::plugin,
        //     ));
        // });

        // Order new `AppSystems` variants by adding them here:
        app.configure_sets(
            Update,
            (
                AppSystems::TickTimers,
                AppSystems::RecordInput,
                AppSystems::Update,
            )
                .chain(),
        );

        app.init_state::<Pause>();
        app.configure_sets(Update, PausableSystems.run_if(in_state(Pause(false))));

        // Spawn the main camera.
        app.add_systems(Startup, spawn_camera);
    }
}

/// High-level groupings of systems for the app in the `Update` schedule.
/// When adding a new variant, make sure to order it in the `configure_sets`
/// call above.
#[derive(SystemSet, Debug, Clone, Copy, Eq, PartialEq, Hash, PartialOrd, Ord)]
enum AppSystems {
    /// Tick timers.
    TickTimers,
    /// Record player input.
    RecordInput,
    /// Do everything else (consider splitting this into further variants).
    Update,
}

#[derive(Component, Debug, Clone, Copy, Reflect)]
pub struct UiCamera;

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Name::new("Main Camera"),
        Camera2d,
        UiCamera,
        // Projection::Orthographic(OrthographicProjection {
        //     scale: 0.35,
        //     ..OrthographicProjection::default_2d()
        // }),
        Transform::from_xyz(10_000.0, 10_000.0, 10_000.0),
    ));
}

#[derive(States, Clone, Copy, Debug, Eq, PartialEq, Hash, Default)]
pub struct Pause(pub bool);

#[derive(SystemSet, Copy, Clone, Eq, PartialEq, Hash, Debug)]
pub struct PausableSystems;
