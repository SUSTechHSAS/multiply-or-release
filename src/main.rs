use battlefield::BattlefieldPlugin;
use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use panel_plugin::PanelPlugin;
use utils::{Participant, UtilsPlugin};

mod battlefield;
mod panel_plugin;
mod utils;

const WINDOW_TITLE: &str = "Multiply or Release";

fn main() {
    let window_plugin = WindowPlugin {
        primary_window: Some(Window {
            title: WINDOW_TITLE.to_string(),
            ..default()
        }),
        ..default()
    };
    App::new()
        .add_plugins(DefaultPlugins.set(window_plugin))
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        // .add_plugins(RapierDebugRenderPlugin::default())
        .add_plugins(WorldInspectorPlugin::new())
        .add_plugins((UtilsPlugin, PanelPlugin, BattlefieldPlugin))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((Name::new("Camera"), Camera2dBundle::default()));
}
