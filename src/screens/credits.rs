//! A credits screen that can be accessed from the title screen.

use bevy::{ecs::spawn::SpawnIter, prelude::*, ui::Val::*};
use bevy_asset_loader::{
    asset_collection::AssetCollection,
    loading_state::{
        LoadingStateAppExt,
        config::{ConfigureLoadingState, LoadingStateConfig},
    },
};

use crate::{audio::music, screens::MenuScreen, theme::prelude::*};

use super::GameState;

pub(super) fn plugin(app: &mut App) {
    app.register_type::<CreditsAssets>();
    app.configure_loading_state(
        LoadingStateConfig::new(GameState::Loading).load_collection::<CreditsAssets>(),
    );
    app.add_systems(
        OnEnter(MenuScreen::Credits),
        (spawn_credits_screen, start_credits_music),
    );
}

fn spawn_credits_screen(mut commands: Commands) {
    commands.spawn((
        widget::ui_root("Credits Screen"),
        StateScoped(MenuScreen::Credits),
        children![
            widget::header("Created by"),
            created_by(),
            widget::header("Assets"),
            assets(),
            widget::button("Back", enter_title_screen),
        ],
    ));
}

fn created_by() -> impl Bundle {
    grid(vec![
        ["Joe Shmoe", "Implemented alligator wrestling AI"],
        ["Jane Doe", "Made the music for the alien invasion"],
    ])
}

fn assets() -> impl Bundle {
    grid(vec![
        ["Ducky sprite", "CC0 by Caz Creates Games"],
        ["Button SFX", "CC0 by Jaszunio15"],
        ["Music", "CC BY 3.0 by Kevin MacLeod"],
        [
            "Bevy logo",
            "All rights reserved by the Bevy Foundation, permission granted for splash screen use when unmodified",
        ],
    ])
}

fn grid(content: Vec<[&'static str; 2]>) -> impl Bundle {
    (
        Name::new("Grid"),
        Node {
            display: Display::Grid,
            row_gap: Px(10.0),
            column_gap: Px(30.0),
            grid_template_columns: RepeatedGridTrack::px(2, 400.0),
            ..default()
        },
        Children::spawn(SpawnIter(content.into_iter().flatten().enumerate().map(
            |(i, text)| {
                (
                    widget::label(text),
                    Node {
                        justify_self: if i % 2 == 0 {
                            JustifySelf::End
                        } else {
                            JustifySelf::Start
                        },
                        ..default()
                    },
                )
            },
        ))),
    )
}

fn enter_title_screen(_: Trigger<Pointer<Click>>, mut next_screen: ResMut<NextState<MenuScreen>>) {
    next_screen.set(MenuScreen::Title);
}

#[derive(Resource, AssetCollection, Clone, Reflect)]
#[reflect(Resource)]
struct CreditsAssets {
    #[asset(path = "audio/music/Monkeys Spinning Monkeys.ogg")]
    music: Handle<AudioSource>,
}

fn start_credits_music(mut commands: Commands, credits_music: Res<CreditsAssets>) {
    commands.spawn((
        Name::new("Credits Music"),
        StateScoped(MenuScreen::Credits),
        music(credits_music.music.clone()),
    ));
}
