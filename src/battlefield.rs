use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::utils::{Participant, ParticipantMap};

// Constants {{{

const TILE_BORDER_COLOR: Color = Color::BLACK;
const TILE_BORDER_THICNESS: f32 = 1.0;
const TILE_COUNT: usize = 40;
const TILE_DIMENSION: f32 = 8.0;
const TILE_Z: f32 = 10.0;

// }}}

pub struct BattlefieldPlugin;
impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
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

fn setup(mut commands: Commands, colors: Res<ParticipantMap<Color>>) {
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
}
