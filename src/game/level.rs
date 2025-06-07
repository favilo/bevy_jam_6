//! Spawn the main level.

use std::{fmt::Debug, time::Duration};

use bevy::{
    asset::RenderAssetUsages,
    color::palettes::css::*,
    prelude::*,
    render::render_resource::{Extent3d, TextureDimension, TextureFormat, TextureUsages},
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        LoadingStateAppExt,
        config::{ConfigureLoadingState, LoadingStateConfig},
    },
};
use bevy_ecs_ldtk::{LdtkWorldBundle, LevelSelection, app::LdtkEntityAppExt, assets::LdtkProject};
use bevy_ecs_tilemap::map::{TilemapGridSize, TilemapSize, TilemapTileSize};

use bevy_enhanced_input::{
    events::Fired,
    prelude::{Actions, Binding, InputAction, InputContext, InputContextAppExt, Press},
};
#[cfg(feature = "dev_native")]
use bevy_simple_subsecond_system::hot;

use crate::{
    Pause, UiCamera,
    audio::music,
    game::{
        cpu::{CpuOptions, CpuSpeedDisplay},
        objects::{GemBundle, GemDisplay, TimeToBomb, TimeToBombDisplay},
        player::PlayerBundle,
        ticks::{reset_simulation, start_simulation},
    },
    menu::Menu,
    state::{GameState, ProgramState},
    theme::{interaction::Inactive, widget},
};

#[allow(dead_code)]
pub const LEVEL_SCALE_FACTOR: f32 = 4.0;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<LevelAssets>();
    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<LevelAssets>(),
    );
    app.add_systems(OnEnter(GameState::Playing), spawn_level_ui);
    app.add_systems(OnEnter(ProgramState::Buying), spawn_level);
    app.register_ldtk_entity::<PlayerBundle>("Player");
    app.register_ldtk_entity::<GemBundle>("Blue_gear");
    app.add_input_context::<LevelContext>();
    app.add_observer(level_context_binding)
        .add_observer(pause_game);
}

#[derive(Resource, AssetCollection, Clone, Reflect)]
#[reflect(Resource)]
pub struct LevelAssets {
    #[asset(path = "audio/music/Fluffing A Duck.ogg")]
    music: Handle<AudioSource>,

    #[asset(path = "images/tiles/marble_packed.png")]
    #[asset(image(sampler(filter = nearest)))]
    pub tiles: Handle<Image>,

    #[asset(path = "maps/mazes.ldtk")]
    pub mazes: Handle<LdtkProject>,
}

#[derive(Resource, Clone, Copy, Debug, Reflect)]
pub struct TilemapMetadata {
    pub map_size: TilemapSize,
    pub tile_size: TilemapTileSize,
    pub grid_size: TilemapGridSize,
    pub scale_factor: f32,
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct ResetButton;

#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct RunButton;

fn control_panel() -> impl Bundle {
    (
        Name::new("Control Panel"),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Vw(33.3),
            margin: UiRect::left(Val::Px(5.0)),
            border: UiRect::left(Val::Px(10.0)),
            padding: UiRect::all(Val::Px(5.0)),
            ..default()
        },
        Pickable::IGNORE,
        children![
            // Controls
            widget::button_medium("Reset", reset_simulation, (Inactive, ResetButton)),
            widget::button_medium("Run", start_simulation, RunButton),
        ],
    )
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct LevelCamera;

pub fn level_camera(commands: &mut Commands, images: &mut ResMut<Assets<Image>>) -> Handle<Image> {
    let size = Extent3d {
        width: 1024,
        height: 1024,
        ..default()
    };
    let mut image = Image::new_fill(
        size,
        TextureDimension::D2,
        &[0, 0, 0, 0],
        TextureFormat::Bgra8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.texture_descriptor.usage =
        TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT;
    let image_handle = images.add(image);

    commands.spawn((
        Name::new("Level Camera"),
        Camera2d,
        UiRoot,
        LevelCamera,
        StateScoped(GameState::Playing),
        Camera {
            target: image_handle.clone().into(),
            clear_color: ClearColorConfig::Custom(DIM_GRAY.into()),
            ..default()
        },
        Projection::Orthographic(OrthographicProjection {
            scale: 0.2,
            ..OrthographicProjection::default_2d()
        }),
        Transform::from_xyz(80.0, 80.0, 1_000.0),
    ));

    image_handle
}

fn level_viewport(image_handle: Handle<Image>) -> impl Bundle {
    (
        Name::new("Level Camera Viewport"),
        children![(
            ImageNode::new(image_handle),
            BorderRadius::all(Val::Px(5.0)),
        )],
        Node {
            width: Val::Vw(33.3),
            height: Val::Vw(33.3),
            aspect_ratio: Some(1.0),
            grid_column: GridPlacement::span(1),
            grid_row: GridPlacement::span(1),

            margin: UiRect {
                left: Val::Px(10.0),
                ..default()
            },
            ..default()
        },
    )
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct UiRoot;

#[cfg_attr(feature = "dev_native", hot(rerun_on_hot_patch = true))]
pub fn spawn_level_ui(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    camera: Single<Entity, With<UiCamera>>,
    old_ui: Query<Entity, With<UiRoot>>,
    mut images: ResMut<Assets<Image>>,
) {
    for root in &old_ui {
        commands.entity(root).despawn();
    }

    let camera_image = level_camera(&mut commands, &mut images);
    commands.spawn((
        widget::ui_root("Level UI"),
        UiTargetCamera(*camera),
        UiRoot,
        StateScoped(GameState::Playing),
        GlobalZIndex(2),
        children![
            music(level_assets.music.clone()),
            (
                Name::new("UI Grid"),
                widget::ui_grid(
                    // Grid columns
                    vec![
                        GridTrack::flex(1.0),
                        GridTrack::flex(1.0),
                        GridTrack::min_content(),
                    ],
                    // Grid rows
                    vec![
                        GridTrack::min_content(),
                        GridTrack::min_content(),
                        GridTrack::flex(2.0),
                    ],
                    // Children
                    (
                        BackgroundColor(DARK_GREY.into()),
                        children![
                            // First row
                            stats_panel(),
                            upgrade_panel(),
                            level_viewport(camera_image), // 2 columns
                            // Second row
                            commands_panel(),
                            control_panel(),
                            // Third row
                            program_panel(),
                        ],
                    )
                )
            )
        ],
    ));
    commands.insert_resource(TimeToBomb {
        duration: Duration::from_millis(200),
    });
    commands.insert_resource(CpuOptions {
        cpu_tick: Duration::from_millis(100),
        multiplier: 1.0,
    });
}

fn upgrade_panel() -> impl Bundle {
    (
        Name::new("Upgrade Panel"),
        Node {
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Stretch,
            align_items: AlignItems::Stretch,
            justify_items: JustifyItems::Start,
            justify_content: JustifyContent::Start,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            grid_row: GridPlacement::span(1),
            grid_column: GridPlacement::span(1),
            padding: UiRect::all(Val::Px(5.0)),
            margin: UiRect {
                left: Val::Px(7.5),
                bottom: Val::Px(7.5),
                ..default()
            },
            ..default()
        },
        Outline {
            width: Val::Px(2.0),
            offset: Val::Px(0.0),
            color: ROSY_BROWN.into(),
        },
        BorderRadius::all(Val::Px(5.0)),
        Pickable::IGNORE,
        children![
            // Upgrades
            widget::label("Upgrades"),
            (
                Name::new("Command Buttons List"),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_content: AlignContent::Stretch,
                    align_items: AlignItems::Stretch,
                    justify_items: JustifyItems::Start,
                    justify_content: JustifyContent::Start,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    // padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(5.0)),
                Pickable::IGNORE,
                UpgradeParent,
                children![],
            )
        ],
    )
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct UpgradeParent;

#[derive(Component, Reflect, Debug, Clone, Copy)]
#[reflect(Component)]
pub struct CommandParent;

fn commands_panel() -> impl Bundle {
    (
        Name::new("Commands Panel"),
        Node {
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Start,
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            justify_content: JustifyContent::Start,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            grid_row: GridPlacement::span(2),
            grid_column: GridPlacement::span(1),
            padding: UiRect::all(Val::Px(5.0)),
            margin: UiRect::all(Val::Px(7.5)),
            ..default()
        },
        Outline {
            width: Val::Px(2.0),
            offset: Val::Px(0.0),
            color: ROSY_BROWN.into(),
        },
        BorderRadius::all(Val::Px(5.0)),
        Pickable::IGNORE,
        children![
            // Commands
            widget::label("Commands"),
            (
                Name::new("Command Palette"),
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    justify_items: JustifyItems::Start,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BorderRadius::all(Val::Px(5.0)),
                Pickable::IGNORE,
                CommandParent,
                children![],
            )
        ],
    )
}

fn stats_panel() -> impl Bundle {
    (
        Name::new("Stats Panel"),
        Node {
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Start,
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            justify_content: JustifyContent::Start,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            grid_row: GridPlacement::span(3),
            grid_column: GridPlacement::span(1),
            // row_gap: Val::Px(10.0),
            padding: UiRect::all(Val::Px(5.0)),
            margin: UiRect::all(Val::Px(2.0)),
            ..default()
        },
        Outline {
            width: Val::Px(2.0),
            offset: Val::Px(0.0),
            color: ROSY_BROWN.into(),
        },
        BorderRadius::all(Val::Px(5.0)),
        Pickable::IGNORE,
        children![
            // Stats
            stat_display::<GemDisplay>("Gears", 0, ROYAL_BLUE.into()),
            stat_display::<TimeToBombDisplay>(
                "Time to boom",
                Duration::default(),
                ORANGE_RED.into()
            ),
            stat_display::<CpuSpeedDisplay>("CPU Inst", Duration::default(), DARK_GREEN.into(),),
        ],
    )
}

#[derive(Component, Reflect, Debug, Clone, Copy)]
pub struct ProgramParent;

fn program_panel() -> impl Bundle {
    (
        Name::new("Program Panel"),
        Node {
            flex_direction: FlexDirection::Column,
            align_content: AlignContent::Start,
            align_items: AlignItems::Start,
            justify_items: JustifyItems::Start,
            justify_content: JustifyContent::Start,
            width: Val::Vw(33.3),
            height: Val::Percent(100.0),
            grid_row: GridPlacement::span(1),
            grid_column: GridPlacement::span(1),
            padding: UiRect::all(Val::Px(5.0)),
            margin: UiRect::left(Val::Px(12.5)),
            ..default()
        },
        Outline {
            width: Val::Px(2.0),
            offset: Val::Px(0.0),
            color: ROSY_BROWN.into(),
        },
        BorderRadius::all(Val::Px(5.0)),
        Pickable::IGNORE,
        children![
            // Program
            widget::label("Program"),
            (
                Name::new("Program List"),
                Node {
                    // flex_direction: FlexDirection::Column,
                    display: Display::Grid,
                    align_items: AlignItems::Start,
                    justify_content: JustifyContent::Start,
                    justify_items: JustifyItems::Start,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    grid_template_columns: vec![GridTrack::min_content(), GridTrack::flex(1.0)],
                    grid_auto_rows: vec![GridTrack::min_content()],
                    // padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                BorderRadius::all(Val::Px(5.0)),
                Pickable::IGNORE,
                ProgramParent,
                children![],
            )
        ],
    )
}

fn stat_display<Comp: Component + Default>(
    label: impl Into<String>,
    default_value: impl Debug,
    color: Color,
) -> impl Bundle {
    let label = label.into();
    (
        Name::new(format!("{label} Row")),
        Node {
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            width: Val::Percent(100.0),
            ..default()
        },
        Pickable::IGNORE,
        children![
            widget::label(format!("{label}:")),
            (
                widget::colored_label(format!("{default_value:?}"), color),
                Comp::default(),
            ),
        ],
    )
}

#[derive(Component, Reflect, Debug, Clone, Copy, Default)]
pub struct LevelRoot;

/// A system that spawns the main level.
#[cfg_attr(feature = "dev_native", hot(rerun_on_hot_patch = true))]
pub fn spawn_level(
    mut commands: Commands,
    level_assets: Res<LevelAssets>,
    loaded_levels: Query<Entity, With<LevelRoot>>,
) {
    for loaded in &loaded_levels {
        commands.entity(loaded).despawn();
    }
    commands.spawn((
        Name::new("Level"),
        LevelRoot,
        LdtkWorldBundle {
            ldtk_handle: level_assets.mazes.clone().into(),
            ..default()
        },
        StateScoped(GameState::Playing),
        children![(Name::new("Gameplay Music"),),],
        Actions::<LevelContext>::default(),
    ));
    commands.insert_resource(LevelSelection::index(0));
}

#[derive(InputContext, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[input_context(priority = 0)]
struct LevelContext;

#[derive(InputAction, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[input_action(output = bool)]
struct PauseGame;

fn level_context_binding(
    trigger: Trigger<Binding<LevelContext>>,
    mut actions: Query<&mut Actions<LevelContext>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<PauseGame>()
        .to((KeyCode::Escape, GamepadButton::Start))
        .with_conditions(Press::new(0.2));
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
