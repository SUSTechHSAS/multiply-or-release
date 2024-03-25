use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;
use rand::{distributions::Uniform, thread_rng, Rng};

// Constants {{{

// Configurable
const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const ARENA_COLOR: Color = Color::DARK_GRAY;
const ARENA_HEIGHT: f32 = 600.0;
const ARENA_WIDTH: f32 = 200.0;

const TRIGGER_ZONE_Y: f32 = -250.0;
const TRIGGER_ZONE_HEIGHT: f32 = 40.0;
const MULTIPLY_ZONE_COLOR: Color = Color::LIME_GREEN;
const BURST_SHOT_ZONE_COLOR: Color = Color::ALICE_BLUE;
const CHARGED_SHOT_ZONE_COLOR: Color = Color::RED;

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

const WORKER_BALL_RADIUS: f32 = 5.0;
const WORKER_BALL_COLOR: Color = Color::GREEN;
const WORKER_BALL_SPAWN_Y: f32 = 300.0;
const WORKER_BALL_RESTITUTION_COEFFICIENT: f32 = 1.0;

// Z-index
const WALL_Z: f32 = 0.0;
const ARENA_Z: f32 = 1.0;
const CIRCLE_Z: f32 = 2.0;
const TRIGGER_ZONE_Z: f32 = 2.0;
const WORKER_BALL_Z: f32 = 3.0;

// Calculated
const WALL_HEIGHT: f32 = ARENA_HEIGHT + 2.0 * WALL_THICKNESS;
const WALL_WIDTH: f32 = ARENA_WIDTH + 2.0 * WALL_THICKNESS;
const ARENA_HEIGHT_FRAC_2: f32 = ARENA_HEIGHT / 2.0;
const ARENA_WIDTH_FRAC_2: f32 = ARENA_WIDTH / 2.0;
const ARENA_WIDTH_FRAC_4: f32 = ARENA_WIDTH / 4.0;
const ARENA_WIDTH_FRAC_8: f32 = ARENA_WIDTH / 8.0;

const CIRCLE_HALF_GAP: f32 = CIRCLE_PYRAMID_HORIZONTAL_GAP / 2.0;
const CIRCLE_DIAMETER: f32 = CIRCLE_RADIUS * 2.0;

// }}}

pub struct PanelPlugin;
impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup)
            .add_systems(PostStartup, spawn_workers);
    }
}

#[derive(Component, Clone, Copy, Default)]
struct MultiplyTrigger;
#[derive(Component, Clone, Copy, Default)]
struct BurstShotTrigger;
#[derive(Component, Clone, Copy, Default)]
struct ChargedShotTrigger;
#[derive(Component, Clone, Copy, Default)]
struct TriggerZone;
#[derive(Bundle, Clone, Resource, Default)]
struct TriggerZoneBundle {
    // {{{
    sprite_bundle: SpriteBundle,
    collider: Collider,
    markers: (TriggerZone, ActiveEvents, Sensor),
}
impl TriggerZoneBundle {
    fn new(size: Vec2, translation: Vec3, color: Color) -> Self {
        Self {
            sprite_bundle: SpriteBundle {
                sprite: Sprite { color, ..default() },
                transform: Transform {
                    translation,
                    scale: size.extend(1.0),
                    rotation: Quat::IDENTITY,
                },
                ..default()
            },
            collider: Collider::cuboid(0.5, 0.5),
            markers: (TriggerZone, ActiveEvents::COLLISION_EVENTS, Sensor),
        }
    }
}
#[derive(Component, Clone, Copy, Default)]
/// Marker to mark this entity as a worker ball.
struct WorkerBall;
#[derive(Bundle, Clone, Resource, Default)]
struct WorkerBallBundle {
    // {{{
    marker: WorkerBall,
    matmesh: MaterialMesh2dBundle<ColorMaterial>,
    collider: Collider,
    restitution: Restitution,
    rigidbody: RigidBody,
}
impl WorkerBallBundle {
    fn rand_x(&mut self) {
        self.matmesh.transform.translation.x =
            thread_rng().sample(Uniform::new(-ARENA_WIDTH_FRAC_2, ARENA_WIDTH_FRAC_2));
    }
    // }}}
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
    commands.insert_resource(WorkerBallBundle {
        marker: WorkerBall,
        matmesh: MaterialMesh2dBundle {
            transform: Transform::from_xyz(0.0, WORKER_BALL_SPAWN_Y, WORKER_BALL_Z),
            mesh: meshes.add(Circle::new(WORKER_BALL_RADIUS)).into(),
            material: materials.add(WORKER_BALL_COLOR),
            ..default()
        },
        collider: Collider::ball(WORKER_BALL_RADIUS),
        restitution: Restitution {
            coefficient: WORKER_BALL_RESTITUTION_COEFFICIENT,
            combine_rule: CoefficientCombineRule::Max,
        },
        rigidbody: RigidBody::Dynamic,
    });
    commands
        .spawn((
            Name::new("PanelRoot"),
            PanelRoot,
            SpatialBundle::default(),
            RigidBody::Fixed,
            Collider::polyline(
                vec![
                    Vec2::new(-ARENA_WIDTH_FRAC_2, ARENA_HEIGHT_FRAC_2),
                    Vec2::new(-ARENA_WIDTH_FRAC_2, -ARENA_HEIGHT_FRAC_2),
                    Vec2::new(ARENA_WIDTH_FRAC_2, -ARENA_HEIGHT_FRAC_2),
                    Vec2::new(ARENA_WIDTH_FRAC_2, ARENA_HEIGHT_FRAC_2),
                    Vec2::new(-ARENA_WIDTH_FRAC_2, ARENA_HEIGHT_FRAC_2),
                ],
                None,
            ),
        ))
        .with_children(|parent| {
            let circle_builder = ObstacleBundleBuilder::new()
                .z(CIRCLE_Z)
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

            parent.spawn((
                MultiplyTrigger,
                TriggerZoneBundle::new(
                    Vec2::new(ARENA_WIDTH_FRAC_2, TRIGGER_ZONE_HEIGHT),
                    Vec3::new(0.0, TRIGGER_ZONE_Y, TRIGGER_ZONE_Z),
                    MULTIPLY_ZONE_COLOR,
                ),
            ));
            parent.spawn((
                BurstShotTrigger,
                TriggerZoneBundle::new(
                    Vec2::new(ARENA_WIDTH_FRAC_4, TRIGGER_ZONE_HEIGHT),
                    Vec3::new(
                        ARENA_WIDTH_FRAC_4 + ARENA_WIDTH_FRAC_8,
                        TRIGGER_ZONE_Y,
                        TRIGGER_ZONE_Z,
                    ),
                    BURST_SHOT_ZONE_COLOR,
                ),
            ));
            parent.spawn((
                ChargedShotTrigger,
                TriggerZoneBundle::new(
                    Vec2::new(ARENA_WIDTH_FRAC_4, TRIGGER_ZONE_HEIGHT),
                    Vec3::new(
                        -ARENA_WIDTH_FRAC_4 - ARENA_WIDTH_FRAC_8,
                        TRIGGER_ZONE_Y,
                        TRIGGER_ZONE_Z,
                    ),
                    CHARGED_SHOT_ZONE_COLOR,
                ),
            ));

            parent.spawn(SpriteBundle {
                transform: Transform {
                    translation: Vec3::new(0.0, 0.0, WALL_Z),
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
                    scale: Vec3::new(ARENA_WIDTH, ARENA_HEIGHT, ARENA_Z),
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
fn spawn_workers(mut commands: Commands, template: Res<WorkerBallBundle>) {
    let mut bundle = template.clone();
    bundle.rand_x();
    commands.spawn(bundle);
}
