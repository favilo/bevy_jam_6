//! Development tools for the game. This plugin is only enabled in dev builds.

use bevy::{
    dev_tools::states::log_transitions, diagnostic::FrameTimeDiagnosticsPlugin, prelude::*,
    ui::UiDebugOptions,
};
use bevy_console::{ConsoleConfiguration, ConsolePlugin};
use bevy_enhanced_input::{
    events::Fired,
    prelude::{Actions, Binding, InputAction, InputContext, InputContextAppExt, Press},
};
use bevy_inspector_egui::{bevy_egui::EguiPlugin, quick::WorldInspectorPlugin};
#[cfg(feature = "dev_native")]
use bevy_simple_subsecond_system::SimpleSubsecondPlugin;

use crate::{
    Pause,
    menu::Menu,
    state::{GameState, ProgramState},
};

#[derive(InputContext, Reflect, Debug, Default)]
pub struct DebugContext;

pub(super) fn plugin(app: &mut App) {
    app.add_input_context::<DebugContext>();
    app.add_plugins((
        EguiPlugin {
            enable_multipass_for_primary_context: true,
        },
        #[cfg(feature = "dev_native")]
        SimpleSubsecondPlugin::default(),
        WorldInspectorPlugin::default().run_if(|options: Res<UiDebugOptions>| options.enabled),
        FrameTimeDiagnosticsPlugin::default(),
        ConsolePlugin,
    ));

    app.insert_resource(ConsoleConfiguration { ..default() });
    // Log `Screen` state transitions.
    app.add_systems(
        Update,
        (
            log_transitions::<GameState>,
            log_transitions::<Menu>,
            log_transitions::<Pause>,
            log_transitions::<ProgramState>,
        ),
    );

    app.add_systems(Startup, |mut commands: Commands| {
        commands.spawn((
            Name::new("Debug Input Context"),
            Actions::<DebugContext>::default(),
        ));
    });

    // Toggle the debug overlay for UI.
    app.add_observer(debug_binding);
    app.add_observer(toggle_debug_ui);
}

const TOGGLE_KEY: KeyCode = KeyCode::Backquote;

#[derive(InputAction, Reflect, Default, Debug)]
#[input_action(output = bool, require_reset = true)]
struct ToggleDebug;

fn debug_binding(
    trigger: Trigger<Binding<DebugContext>>,
    mut actions: Query<&mut Actions<DebugContext>>,
) {
    let mut actions = actions.get_mut(trigger.target()).unwrap();
    actions
        .bind::<ToggleDebug>()
        .to(TOGGLE_KEY)
        .with_conditions(Press::new(0.2));
}

fn toggle_debug_ui(_: Trigger<Fired<ToggleDebug>>, mut options: ResMut<UiDebugOptions>) {
    options.toggle();
}
