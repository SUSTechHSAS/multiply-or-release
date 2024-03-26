use battlefield::BattlefieldPlugin;
use bevy::{
    prelude::*,
    sprite::{Material2d, MaterialMesh2dBundle},
};
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

#[derive(Component)]
struct Bullet;
#[derive(Bundle)]
/// Component bundle for the bullets that the turrets fire.
struct BulletBundle<M: Material2d> {
    /// Marker to mark this entity as a bullet.
    marker: Bullet,
    /// Bevy rendering component used to display the bullet.
    mesh: MaterialMesh2dBundle<M>,
    /// Rapier collider component.
    collider: Collider,
    /// Rapier rigidbody component, used by the physics engine to move the entity.
    rigidbody: RigidBody,
    /// The game participant that owns this bullet.
    owner: Participant,
    /// Some text component for bevy to render the text onto the ball
    /// (We're not sure exact how this would be done at the moment).
    _text: (),
}
