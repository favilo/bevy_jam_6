//! Player-specific behavior.

use bevy::{
    image::{ImageLoaderSettings, ImageSampler},
    prelude::*,
};
use bevy_enhanced_input::{
    events::Fired,
    prelude::{Actions, Binding, DeadZone, InputAction, InputContext, InputContextAppExt, Press},
    preset::{Axial, Cardinal},
};

use crate::{
    AppSystems,
    asset_tracking::LoadResource,
    demo::{
        animation::PlayerAnimation,
        movement::{MovementController, ScreenWrap},
    },
    screens::{GameState, MenuScreen},
};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<Player>();

    app.register_type::<PlayerAssets>();
    app.load_resource::<PlayerAssets>();
    app.add_input_context::<Player>();

    // Record directional input as movement controls.
    app.add_observer(player_binding);
    app.add_observer(quit_to_main_menu);
    app.add_systems(
        Update,
        record_player_directional_input.in_set(AppSystems::RecordInput),
    );
}

/// The player character.
pub fn player(
    max_speed: f32,
    player_assets: &PlayerAssets,
    texture_atlas_layouts: &mut Assets<TextureAtlasLayout>,
) -> impl Bundle {
    // A texture atlas is a way to split a single image into a grid of related images.
    // You can learn more in this example: https://github.com/bevyengine/bevy/blob/latest/examples/2d/texture_atlas.rs
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(32), 6, 2, Some(UVec2::splat(1)), None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    let player_animation = PlayerAnimation::new();

    (
        Name::new("Player"),
        Actions::<Player>::default(),
        Sprite {
            image: player_assets.ducky.clone(),
            texture_atlas: Some(TextureAtlas {
                layout: texture_atlas_layout,
                index: player_animation.get_atlas_index(),
            }),
            ..default()
        },
        Transform::from_scale(Vec2::splat(8.0).extend(1.0)),
        MovementController {
            max_speed,
            ..default()
        },
        ScreenWrap,
        player_animation,
    )
}

#[derive(InputContext, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
struct Player;

#[derive(InputAction, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[input_action(output = bool)]
struct Pause;

#[derive(InputAction, Debug, Clone, Copy, PartialEq, Eq, Default, Reflect)]
#[input_action(output = Vec2)]
struct Move;

fn player_binding(trigger: Trigger<Binding<Player>>, mut actions: Query<&mut Actions<Player>>) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<Pause>()
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

fn quit_to_main_menu(
    _: Trigger<Fired<Pause>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut next_screen: ResMut<NextState<MenuScreen>>,
) {
    next_state.set(GameState::Menu);
    next_screen.set(MenuScreen::Title);
}

#[derive(Resource, Asset, Clone, Reflect)]
#[reflect(Resource)]
pub struct PlayerAssets {
    #[dependency]
    ducky: Handle<Image>,
    #[dependency]
    pub steps: Vec<Handle<AudioSource>>,
}

impl FromWorld for PlayerAssets {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        Self {
            ducky: assets.load_with_settings(
                "images/ducky.png",
                |settings: &mut ImageLoaderSettings| {
                    // Use `nearest` image sampling to preserve pixel art style.
                    settings.sampler = ImageSampler::nearest();
                },
            ),
            steps: vec![
                assets.load("audio/sound_effects/step1.ogg"),
                assets.load("audio/sound_effects/step2.ogg"),
                assets.load("audio/sound_effects/step3.ogg"),
                assets.load("audio/sound_effects/step4.ogg"),
            ],
        }
    }
}
