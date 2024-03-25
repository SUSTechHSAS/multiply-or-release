use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

// Constants {{{

// Configurable
const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const ARENA_COLOR: Color = Color::DARK_GRAY;
const ARENA_HEIGHT: f32 = 600.0;
const ARENA_WIDTH: f32 = 200.0;

// Calculated
const WALL_HEIGHT: f32 = ARENA_HEIGHT + 2.0 * WALL_THICKNESS;
const WALL_WIDTH: f32 = ARENA_WIDTH + 2.0 * WALL_THICKNESS;
const ARENA_HALF_HEIGHT: f32 = ARENA_HEIGHT / 2.0;
const ARENA_HALF_WIDTH: f32 = ARENA_WIDTH / 2.0;

// }}}

pub struct PanelPlugin;
impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

fn setup(mut commands: Commands) {
    commands.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::new(WALL_WIDTH, WALL_HEIGHT, 1.0),
            rotation: Quat::IDENTITY,
        },
        sprite: Sprite {
            color: WALL_COLOR,
            ..default()
        },
        ..default()
    });
    commands.spawn(SpriteBundle {
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 1.0),
            scale: Vec3::new(ARENA_WIDTH, ARENA_HEIGHT, 1.0),
            rotation: Quat::IDENTITY,
        },
        sprite: Sprite {
            color: ARENA_COLOR,
            ..default()
        },
        ..default()
    });
    commands.spawn((
        RigidBody::Fixed,
        Collider::polyline(
            vec![
                Vec2::new(-ARENA_HALF_WIDTH, ARENA_HALF_HEIGHT),
                Vec2::new(-ARENA_HALF_WIDTH, -ARENA_HALF_HEIGHT),
                Vec2::new(ARENA_HALF_WIDTH, -ARENA_HALF_HEIGHT),
                Vec2::new(ARENA_HALF_WIDTH, ARENA_HALF_HEIGHT),
            ],
            Some(vec![[0, 1], [1, 2], [2, 3], [3, 0]]),
        ),
    ));
}
