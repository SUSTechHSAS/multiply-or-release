use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;

// Constants {{{

// Configurable
const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const ARENA_COLOR: Color = Color::DARK_GRAY;
const ARENA_HEIGHT: f32 = 600.0;
const ARENA_WIDTH: f32 = 200.0;

const CIRCLE_RADIUS: f32 = 10.0;
const CIRCLE_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const CIRCLE_PYRAMID_VERTICAL_OFFSET: f32 = 200.0;
const CIRCLE_PYRAMID_VERTICAL_COUNT: usize = 4;
const CIRCLE_PYRAMID_VERTICAL_GAP: f32 = 10.0;
const CIRCLE_PYRAMID_HORIZONTAL_GAP: f32 = 25.0;

const CIRCLE_GRID_VERTICAL_OFFSET: f32 = 0.0;
const CIRCLE_GRID_VERTICAL_COUNT: usize = 6;
const CIRCLE_GRID_VERTICAL_GAP: f32 = 10.0;
const CIRCLE_GRID_HORIZONTAL_GAP: f32 = 25.0;
const CIRCLE_GRID_HORIZONTAL_HALF_COUNT: usize = 2;

// Calculated
const WALL_HEIGHT: f32 = ARENA_HEIGHT + 2.0 * WALL_THICKNESS;
const WALL_WIDTH: f32 = ARENA_WIDTH + 2.0 * WALL_THICKNESS;
const ARENA_HALF_HEIGHT: f32 = ARENA_HEIGHT / 2.0;
const ARENA_HALF_WIDTH: f32 = ARENA_WIDTH / 2.0;

const CIRCLE_HALF_GAP: f32 = CIRCLE_PYRAMID_HORIZONTAL_GAP / 2.0;
const CIRCLE_DIAMETER: f32 = CIRCLE_RADIUS * 2.0;

// }}}

pub struct PanelPlugin;
impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
    }
}

#[derive(Component)]
pub struct PanelRoot;
#[derive(Bundle)]
/// Component bundle for the round obstacles in the side panels and the walls.
/// (I don't know if meshes and colliders have to be continous. Maybe we can just make a single
/// entity for the entire obstacle course.)
struct ObstacleBundle {
    // {{{
    /// Bevy rendering component used to display the ball.
    matmesh: MaterialMesh2dBundle<ColorMaterial>,
    /// Rapier collider component.
    collider: Collider,
    /// Rapier rigidbody component. We'll set this to static since we don't want these to move, but
    /// we'd other balls to bounce off it.
    rigidbody: RigidBody,
}
#[derive(Debug, Clone, Default)]
struct ObstacleBundleBuilder {
    /// Bevy rendering component used to display the ball.
    translation: Vec3,
    material: Option<Handle<ColorMaterial>>,
    mesh: Option<Mesh2dHandle>,
    /// Rapier collider component.
    collider: Option<Collider>,
}
impl ObstacleBundleBuilder {
    fn new() -> Self {
        Self::default()
    }
    fn xy(mut self, x: f32, y: f32) -> Self {
        self.translation.x = x;
        self.translation.y = y;
        self
    }
    fn z(mut self, z: f32) -> Self {
        self.translation.z = z;
        self
    }
    fn material(mut self, material: Handle<ColorMaterial>) -> Self {
        self.material = Some(material);
        self
    }
    fn mesh(mut self, mesh: Handle<Mesh>) -> Self {
        self.mesh = Some(mesh.into());
        self
    }
    fn collider(mut self, collider: Collider) -> Self {
        self.collider = Some(collider);
        self
    }
    fn build(self) -> Option<ObstacleBundle> {
        let ObstacleBundleBuilder {
            translation: Vec3 { x, y, z },
            material: Some(material),
            mesh: Some(mesh),
            collider: Some(collider),
        } = self
        else {
            return None;
        };
        Some(ObstacleBundle {
            matmesh: MaterialMesh2dBundle {
                mesh,
                material,
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            collider,
            rigidbody: RigidBody::Fixed,
        })
    }
    /// Build trust me bro.
    fn buildtmb(self) -> ObstacleBundle {
        self.build().unwrap()
    }
    // }}}
}

fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    commands
        .spawn((
            Name::new("PanelRoot"),
            PanelRoot,
            SpatialBundle::default(),
            RigidBody::Fixed,
            Collider::polyline(
                vec![
                    Vec2::new(-ARENA_HALF_WIDTH, ARENA_HALF_HEIGHT),
                    Vec2::new(-ARENA_HALF_WIDTH, -ARENA_HALF_HEIGHT),
                    Vec2::new(ARENA_HALF_WIDTH, -ARENA_HALF_HEIGHT),
                    Vec2::new(ARENA_HALF_WIDTH, ARENA_HALF_HEIGHT),
                    Vec2::new(-ARENA_HALF_WIDTH, ARENA_HALF_HEIGHT),
                ],
                None,
            ),
        ))
        .with_children(|parent| {
            let circle_builder = ObstacleBundleBuilder::new()
                .z(2.0)
                .material(materials.add(CIRCLE_COLOR))
                .mesh(meshes.add(Circle::new(CIRCLE_RADIUS)))
                .collider(Collider::ball(CIRCLE_RADIUS));

            for i in 0..CIRCLE_PYRAMID_VERTICAL_COUNT {
                let y = -(i as f32) * (CIRCLE_DIAMETER + CIRCLE_PYRAMID_VERTICAL_GAP)
                    + CIRCLE_PYRAMID_VERTICAL_OFFSET;
                if i % 2 == 0 {
                    parent.spawn(circle_builder.clone().xy(0.0, y).buildtmb());

                    for j in 1..=i / 2 {
                        let x = j as f32 * (CIRCLE_DIAMETER + CIRCLE_PYRAMID_HORIZONTAL_GAP);
                        parent.spawn(circle_builder.clone().xy(x, y).buildtmb());
                        parent.spawn(circle_builder.clone().xy(-x, y).buildtmb());
                    }
                } else {
                    let x0 = CIRCLE_HALF_GAP + CIRCLE_RADIUS;
                    parent.spawn(circle_builder.clone().xy(x0, y).buildtmb());
                    parent.spawn(circle_builder.clone().xy(-x0, y).buildtmb());
                    for j in 1..i - 1 {
                        let x = j as f32 * (CIRCLE_DIAMETER + CIRCLE_PYRAMID_HORIZONTAL_GAP) + x0;
                        parent.spawn(circle_builder.clone().xy(x, y).buildtmb());
                        parent.spawn(circle_builder.clone().xy(-x, y).buildtmb());
                    }
                }
            }

            for i in 0..CIRCLE_GRID_VERTICAL_COUNT {
                let y = -(i as f32) * (CIRCLE_DIAMETER + CIRCLE_GRID_VERTICAL_GAP)
                    + CIRCLE_GRID_VERTICAL_OFFSET;
                if i % 2 == 0 {
                    parent.spawn(circle_builder.clone().xy(0.0, y).buildtmb());

                    for j in 1..=CIRCLE_GRID_HORIZONTAL_HALF_COUNT {
                        let x = j as f32 * (CIRCLE_DIAMETER + CIRCLE_GRID_HORIZONTAL_GAP);
                        parent.spawn(circle_builder.clone().xy(x, y).buildtmb());
                        parent.spawn(circle_builder.clone().xy(-x, y).buildtmb());
                    }
                } else {
                    let x0 = CIRCLE_HALF_GAP + CIRCLE_RADIUS;
                    parent.spawn(circle_builder.clone().xy(x0, y).buildtmb());
                    parent.spawn(circle_builder.clone().xy(-x0, y).buildtmb());
                    for j in 1..=CIRCLE_GRID_HORIZONTAL_HALF_COUNT {
                        let x = j as f32 * (CIRCLE_DIAMETER + CIRCLE_GRID_HORIZONTAL_GAP) + x0;
                        parent.spawn(circle_builder.clone().xy(x, y).buildtmb());
                        parent.spawn(circle_builder.clone().xy(-x, y).buildtmb());
                    }
                }
            }

            parent.spawn(SpriteBundle {
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
            parent.spawn(SpriteBundle {
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
        });
}
