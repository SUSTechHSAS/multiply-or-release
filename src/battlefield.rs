use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
    time::Stopwatch,
};
use bevy_rapier2d::prelude::*;

use crate::utils::{Participant, ParticipantMap};

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

// Z-index
const TILE_Z: f32 = 10.0;
// Turret head is a child of turret, which inherits the z position as well, so the local z of the
// head needs to be negative to put it behind the main turret.
const TURRET_HEAD_Z: f32 = -1.0;
const TURRET_Z: f32 = 12.0;

// }}}

pub struct BattlefieldPlugin;
impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(Update, rotate_turret);
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
#[derive(Component)]
#[allow(dead_code)]
struct Turret {
    /// The direction that the barrel would be pointing in with an offset_angle of 0.
    base_offset: f32,
}
/// Component bundle for a turret.
#[derive(Bundle)]
struct TurretBundle {
    /// Bevy rendering component used to display the ball.
    matmesh: MaterialMesh2dBundle<ColorMaterial>,
    /// The game participant that owns this ball.
    owner: Participant,
    marker: Sensor,
    collider: Collider,
    /// Variables for the functionality of the turret.
    turret: Turret,
}
impl TurretBundle {
    fn new(
        position: Vec2,
        material: Handle<ColorMaterial>,
        mesh: Mesh2dHandle,
        owner: Participant,
        base_offset: f32,
    ) -> Self {
        Self {
            owner,
            matmesh: MaterialMesh2dBundle {
                material,
                mesh,
                transform: Transform::from_translation(position.extend(TURRET_Z))
                    .with_rotation(Quat::from_rotation_z(base_offset)),
                ..default()
            },
            marker: Sensor,
            collider: Collider::ball(TURRET_RADIUS),
            turret: Turret { base_offset },
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
            parent
                .spawn(TurretBundle::new(
                    Vec2::new(TURRET_POSITION, TURRET_POSITION),
                    materials.a.clone(),
                    mesh.clone(),
                    Participant::A,
                    PI,
                ))
                .with_children(head_spawner);
            parent
                .spawn(TurretBundle::new(
                    Vec2::new(-TURRET_POSITION, TURRET_POSITION),
                    materials.b.clone(),
                    mesh.clone(),
                    Participant::B,
                    -FRAC_PI_2,
                ))
                .with_children(head_spawner);
            parent
                .spawn(TurretBundle::new(
                    Vec2::new(TURRET_POSITION, -TURRET_POSITION),
                    materials.c.clone(),
                    mesh.clone(),
                    Participant::C,
                    FRAC_PI_2,
                ))
                .with_children(head_spawner);
            parent
                .spawn(TurretBundle::new(
                    Vec2::new(-TURRET_POSITION, -TURRET_POSITION),
                    materials.d.clone(),
                    mesh.clone(),
                    Participant::D,
                    0.0,
                ))
                .with_children(head_spawner);
        });
}
fn rotate_turret(
    time: Res<Time>,
    mut stopwatch: ResMut<TurretStopwatch>,
    mut turrets: Query<(&mut Transform, &Turret)>,
) {
    stopwatch.0.tick(time.delta());
    let angle_offset = FRAC_PI_2
        - ((stopwatch.0.elapsed_secs() % PI * TURRET_ROTATION_SPEED) % PI - FRAC_PI_2).abs();
    for (mut transform, turret) in &mut turrets {
        *transform =
            transform.with_rotation(Quat::from_rotation_z(turret.base_offset - angle_offset));
    }
}
