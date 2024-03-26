#![allow(clippy::type_complexity)]

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::Stopwatch,
};
use bevy_rapier2d::prelude::*;

use crate::{
    panel_plugin::{TriggerEvent, TriggerType},
    utils::{Participant, ParticipantMap},
};

// Constants {{{

const TILE_BORDER_COLOR: Color = Color::BLACK;
const TILE_BORDER_THICNESS: f32 = 1.0;
const TILE_COUNT: usize = 40;
const TILE_DIMENSION: f32 = 8.0;

const TURRET_POSITION: f32 = 350.0;
const TURRET_RADIUS: f32 = 10.0;
const TURRET_HEAD_COLOR: Color = Color::DARK_GRAY;
const TURRET_HEAD_THICNESS: f32 = 2.5;
const TURRET_HEAD_LENGTH: f32 = 75.0;
const TURRET_ROTATION_SPEED: f32 = 1.0;

const BULLET_TEXT_COLOR: Color = Color::BLACK;
const BULLET_TEXT_FONT_SIZE: f32 = 24.0;

// Z-index
const TILE_Z: f32 = 10.0;
const BULLET_TEXT_Z: f32 = 20.0;
// Turret head is a child of turret, which inherits the z position as well, so the local z of the
// head needs to be negative to put it behind the main turret.
const TURRET_HEAD_Z: f32 = -1.0;
const TURRET_Z: f32 = -1.0;

// }}}

pub struct BattlefieldPlugin;
impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (rotate_turret, update_charge_text, handle_trigger_events),
        );
    }
}

/// Marker to mark this entity as a tile.
#[derive(Component, Clone, Copy)]
struct Tile;
/// Component bundle for each of the individual tiles on the battle field.
#[derive(Bundle)]
struct TileBundle {
    /// Markers to mark this entity as a tile, a sensor collider, and a trigger for collision
    /// events.
    markers: (Tile, Sensor, ActiveEvents),
    /// Bevy rendering component used to display the tile.
    sprite_bundle: SpriteBundle,
    /// Rapier collider component. We'll mark this as sensor and won't add a rigidbody to this
    /// entity because we don't actually want the physics engine to move itl.
    collider: Collider,
    /// The game participant that owns this tile.
    owner: Participant,
}
impl TileBundle {
    fn new(owner: Participant, color: Color, x: f32, y: f32) -> Self {
        Self {
            markers: (Tile, Sensor, ActiveEvents::COLLISION_EVENTS),
            sprite_bundle: SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(x, y, TILE_Z),
                    scale: Vec3::new(TILE_DIMENSION, TILE_DIMENSION, 1.0),
                    rotation: Quat::IDENTITY,
                },
                sprite: Sprite { color, ..default() },
                ..default()
            },
            collider: Collider::cuboid(0.5, 0.5),
            owner,
        }
    }
}
#[derive(Resource, Default, Clone)]
struct TurretStopwatch(Stopwatch);
#[derive(Component, Clone, Copy)]
struct Charge(usize);
impl Default for Charge {
    fn default() -> Self {
        Self(1)
    }
}
impl Charge {
    fn multiply(&mut self) {
        self.0 *= 2;
    }
}
#[derive(Component, Default)]
struct Turret;
#[derive(Bundle, Default)]
struct TurretBundle {
    marker: Turret,
    charge: Charge,
    text_bundle: Text2dBundle,
    owner: Participant,
    collider: Collider,
}
impl TurretBundle {
    fn new(owner: Participant, x: f32, y: f32) -> Self {
        Self {
            marker: Turret,
            owner,
            charge: Default::default(),
            collider: Collider::ball(TURRET_RADIUS),
            text_bundle: Text2dBundle {
                transform: Transform::from_xyz(x, y, BULLET_TEXT_Z),
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: Default::default(),
                        font_size: BULLET_TEXT_FONT_SIZE,
                        color: BULLET_TEXT_COLOR,
                    },
                ),
                ..default()
            },
        }
    }
}
/// Marker to indicate the entity is a turret head.
#[derive(Component)]
struct TurretHead;
/// Component bundle for the turret head (the little ball that sits on the top of the turret to
/// show its charge level and never moves).
#[derive(Bundle)]
struct TurretHeadBundle {
    /// Marker to indicate that this is a turret head.
    marker: TurretHead,
    /// Bevy rendering component used to display the ball.
    sprite_bundle: SpriteBundle,
}
impl TurretHeadBundle {
    fn new() -> Self {
        Self {
            marker: TurretHead,
            sprite_bundle: SpriteBundle {
                sprite: Sprite {
                    color: TURRET_HEAD_COLOR,
                    ..default()
                },
                transform: Transform {
                    translation: Vec3::new(0.0, TURRET_HEAD_LENGTH / 2.0, TURRET_HEAD_Z),
                    scale: Vec3::new(TURRET_HEAD_THICNESS, TURRET_HEAD_LENGTH, 1.0),
                    rotation: Quat::IDENTITY,
                },
                ..default()
            },
        }
    }
}
/// Component for a turret.
#[derive(Component, Default)]
struct BarrelOffset(f32);
/// Component bundle for a turret.
#[derive(Bundle, Default)]
struct TurretPlatformBundle {
    /// Bevy rendering component used to display the ball.
    matmesh: MaterialMesh2dBundle<ColorMaterial>,
    marker: Sensor,
    barrel_offset: BarrelOffset,
}
impl TurretPlatformBundle {
    fn new(material: Handle<ColorMaterial>, mesh: Mesh2dHandle, base_offset: f32) -> Self {
        Self {
            matmesh: MaterialMesh2dBundle {
                transform: Transform::from_xyz(0.0, 0.0, TURRET_Z),
                material,
                mesh,
                ..default()
            },
            marker: Sensor,
            barrel_offset: BarrelOffset(base_offset),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    colors: Res<ParticipantMap<Color>>,
    materials: Res<ParticipantMap<Handle<ColorMaterial>>>,
) {
    commands.insert_resource(TurretStopwatch::default());
    commands
        .spawn((
            Name::new("Battlefield Root"),
            SpriteBundle {
                sprite: Sprite {
                    color: TILE_BORDER_COLOR,
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent
                .spawn((Name::new("Battlefield"), SpatialBundle::default()))
                .with_children(|parent| {
                    for i in 0..TILE_COUNT {
                        let x = (TILE_DIMENSION + TILE_BORDER_THICNESS) / 2.0
                            + i as f32 * (TILE_DIMENSION + TILE_BORDER_THICNESS);
                        for j in 0..TILE_COUNT {
                            let y = (TILE_DIMENSION + TILE_BORDER_THICNESS) / 2.0
                                + j as f32 * (TILE_DIMENSION + TILE_BORDER_THICNESS);
                            parent.spawn(TileBundle::new(Participant::A, colors.a, x, y));
                            parent.spawn(TileBundle::new(Participant::B, colors.b, -x, y));
                            parent.spawn(TileBundle::new(Participant::C, colors.c, x, -y));
                            parent.spawn(TileBundle::new(Participant::D, colors.d, -x, -y));
                        }
                    }
                });
            let mesh = Mesh2dHandle(meshes.add(Circle::new(TURRET_RADIUS)));
            fn head_spawner(turret: &mut ChildBuilder) {
                turret.spawn(TurretHeadBundle::new());
            }
            let spawn_turret =
                |parent: &mut ChildBuilder, material: Handle<ColorMaterial>, base_offset| {
                    parent
                        .spawn(TurretPlatformBundle::new(
                            material,
                            mesh.clone(),
                            base_offset,
                        ))
                        .with_children(head_spawner);
                };
            let a = parent
                .spawn(TurretBundle::new(
                    Participant::A,
                    TURRET_POSITION,
                    TURRET_POSITION,
                ))
                .with_children(|parent| {
                    spawn_turret(parent, materials.a.clone(), PI);
                })
                .id();
            let b = parent
                .spawn(TurretBundle::new(
                    Participant::B,
                    -TURRET_POSITION,
                    TURRET_POSITION,
                ))
                .with_children(|parent| {
                    spawn_turret(parent, materials.b.clone(), -FRAC_PI_2);
                })
                .id();
            let c = parent
                .spawn(TurretBundle::new(
                    Participant::C,
                    TURRET_POSITION,
                    -TURRET_POSITION,
                ))
                .with_children(|parent| {
                    spawn_turret(parent, materials.c.clone(), FRAC_PI_2);
                })
                .id();
            let d = parent
                .spawn(TurretBundle::new(
                    Participant::D,
                    -TURRET_POSITION,
                    -TURRET_POSITION,
                ))
                .with_children(|parent| {
                    spawn_turret(parent, materials.d.clone(), 0.0);
                })
                .id();
            let participant_entities = ParticipantMap::new(a, b, c, d);
            parent
                .add_command(move |world: &mut World| world.insert_resource(participant_entities));
        });
}
fn rotate_turret(
    time: Res<Time>,
    mut stopwatch: ResMut<TurretStopwatch>,
    mut turrets: Query<(&mut Transform, &BarrelOffset)>,
) {
    stopwatch.0.tick(time.delta());
    let angle_offset = FRAC_PI_2
        - ((stopwatch.0.elapsed_secs() % PI * TURRET_ROTATION_SPEED) % PI - FRAC_PI_2).abs();
    for (mut transform, &BarrelOffset(base_offset)) in &mut turrets {
        *transform = transform.with_rotation(Quat::from_rotation_z(base_offset - angle_offset));
    }
}
fn update_charge_text(
    mut query: Query<(&mut Text, &Charge), Or<(Changed<Charge>, Added<Charge>)>>,
) {
    for (mut text, &Charge(charge)) in &mut query {
        text.sections[0].value = charge.to_string();
    }
}
fn handle_trigger_events(
    mut reader: EventReader<TriggerEvent>,
    participants: Res<ParticipantMap<Entity>>,
    mut turret_query: Query<&mut Charge, With<Turret>>,
) {
    for event in reader.read() {
        match event.trigger_type {
            TriggerType::Multiply => {
                let &entity = participants.get(event.participant);
                let mut charge = turret_query.get_mut(entity).unwrap();
                charge.multiply()
            }
            TriggerType::BurstShot => {
                dbg!("Not implemented");
            }
            TriggerType::ChargedShot => {
                dbg!("Not implemented");
            }
        }
    }
}
