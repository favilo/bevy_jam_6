use bevy::{
    ecs::{component::HookContext, world::DeferredWorld},
    prelude::*,
};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        LoadingStateAppExt,
        config::{ConfigureLoadingState, LoadingStateConfig},
    },
};

use crate::{audio::sound_effect, state::GameState};

pub(super) fn plugin(app: &mut App) {
    app.register_type::<InteractionPalette>();
    app.register_type::<Inactive>();
    app.add_systems(Update, apply_interaction_palette);

    app.register_type::<InteractionAssets>();
    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<InteractionAssets>(),
    );
    app.add_observer(play_on_hover_sound_effect);
    app.add_observer(play_on_click_sound_effect);
}

#[derive(Component, Debug, Reflect, Clone, Copy, PartialEq, Eq)]
#[reflect(Component)]
#[component(on_add = deactivate_inactive_palette, on_remove = activate_active_palette)]
pub struct Inactive;

/// Palette for widget interactions. Add this to an entity that supports
/// [`Interaction`]s, such as a button, to change its [`BackgroundColor`] based
/// on the current interaction state.
#[derive(Component, Debug, Reflect, Clone)]
#[reflect(Component)]
pub struct InteractionPalette {
    pub none: Color,
    pub hovered: Color,
    pub pressed: Color,
    pub inactive: Color,
}

fn deactivate_inactive_palette(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let palette = world
        .get::<InteractionPalette>(entity)
        .expect("InteractionPalette should be present")
        .clone();
    let mut entity = world.entity_mut(entity);
    let mut background = entity
        .get_mut::<BackgroundColor>()
        .expect("InteractionPalette should be present");
    *background = palette.inactive.into();
}

fn activate_active_palette(mut world: DeferredWorld, HookContext { entity, .. }: HookContext) {
    let palette = world
        .get::<InteractionPalette>(entity)
        .expect("InteractionPalette should be present")
        .clone();
    let mut entity = world.entity_mut(entity);
    let mut background = entity
        .get_mut::<BackgroundColor>()
        .expect("InteractionPalette should be present");
    *background = palette.none.into();
}

fn apply_interaction_palette(
    mut palette_query: Query<
        (
            &Interaction,
            &InteractionPalette,
            &mut BackgroundColor,
            Has<Inactive>,
        ),
        Changed<Interaction>,
    >,
) {
    for (interaction, palette, mut background, inactive) in &mut palette_query {
        if inactive {
            continue;
        }
        *background = match interaction {
            Interaction::None => palette.none,
            Interaction::Hovered => palette.hovered,
            Interaction::Pressed => palette.pressed,
        }
        .into();
    }
}

#[derive(Resource, AssetCollection, Clone, Reflect)]
#[reflect(Resource)]
struct InteractionAssets {
    #[asset(path = "audio/sound_effects/button_hover.ogg")]
    hover: Handle<AudioSource>,
    #[asset(path = "audio/sound_effects/button_click.ogg")]
    click: Handle<AudioSource>,
}

fn play_on_hover_sound_effect(
    trigger: Trigger<Pointer<Over>>,
    mut commands: Commands,
    interaction_assets: Option<Res<InteractionAssets>>,
    interaction_query: Query<(), (With<Interaction>, Without<Inactive>)>,
) {
    let Some(interaction_assets) = interaction_assets else {
        return;
    };

    if interaction_query.contains(trigger.target()) {
        commands.spawn(sound_effect(interaction_assets.hover.clone()));
    }
}

fn play_on_click_sound_effect(
    trigger: Trigger<Pointer<Click>>,
    mut commands: Commands,
    interaction_assets: Option<Res<InteractionAssets>>,
    interaction_query: Query<(), (With<Interaction>, Without<Inactive>)>,
) {
    let Some(interaction_assets) = interaction_assets else {
        return;
    };

    if interaction_query.contains(trigger.target()) {
        commands.spawn(sound_effect(interaction_assets.click.clone()));
    }
}
