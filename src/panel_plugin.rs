use std::time::Duration;

use bevy::{
    prelude::*,
    sprite::{MaterialMesh2dBundle, Mesh2dHandle},
};
use bevy_rapier2d::prelude::*;
use rand::{distributions::Uniform, thread_rng, Rng};

use crate::{utils::ParticipantInfo, Participant};

// Constants {{{

// Configurable

const ROOT_X_OFFSET: f32 = -500.0;

const WALL_THICKNESS: f32 = 10.0;
const WALL_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const ARENA_COLOR: Color = Color::DARK_GRAY;
const ARENA_HEIGHT: f32 = 700.0;
const ARENA_WIDTH: f32 = 260.0;

const TRIGGER_ZONE_Y: f32 = -250.0;
const TRIGGER_ZONE_HEIGHT: f32 = 40.0;
const MULTIPLY_ZONE_COLOR: Color = Color::LIME_GREEN;
const BURST_SHOT_ZONE_COLOR: Color = Color::ALICE_BLUE;
const CHARGED_SHOT_ZONE_COLOR: Color = Color::RED;

const CIRCLE_RADIUS: f32 = 10.0;
const CIRCLE_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);
const CIRCLE_PYRAMID_VERTICAL_OFFSET: f32 = 250.0;
const CIRCLE_PYRAMID_VERTICAL_COUNT: usize = 5;
const CIRCLE_PYRAMID_VERTICAL_GAP: f32 = 8.0;
const CIRCLE_PYRAMID_HORIZONTAL_GAP: f32 = 40.0;

const CIRCLE_GRID_VERTICAL_OFFSET: f32 = 70.0;
const CIRCLE_GRID_VERTICAL_COUNT: usize = 8;
const CIRCLE_GRID_VERTICAL_GAP: f32 = 15.0;
const CIRCLE_GRID_HORIZONTAL_GAP: f32 = 27.0;
const CIRCLE_GRID_HORIZONTAL_HALF_COUNT_EVEN_ROW: usize = 2;
const CIRCLE_GRID_HORIZONTAL_HALF_COUNT_ODD_ROW: usize = 3;

const WORKER_BALL_RADIUS: f32 = 5.0;
const WORKER_BALL_SPAWN_Y: f32 = 320.0;
const WORKER_BALL_RESTITUTION_COEFFICIENT: f32 = 0.75;
const WORKER_BALL_SPAWN_TIMER_SECS: f32 = 10.0;
const WORKER_BALL_COUNT_MAX: usize = 5;

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

const WORKER_BALL_DIAMETER: f32 = WORKER_BALL_RADIUS * 2.0;

// }}}

pub struct PanelPlugin;
impl Plugin for PanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TriggerEvent>()
            .add_systems(Startup, setup)
            .add_systems(Update, spawn_workers.run_if(spawn_workers_condition))
            .add_systems(Update, (trigger_event, print_trigger_events, ball_reset));
    }
}

#[derive(Debug, Event)]
pub struct TriggerEvent {
    pub participant: Participant,
    pub trigger_type: TriggerType,
}
#[derive(Debug, Component, Clone, Copy)]
pub enum TriggerType {
    Multiply,
    BurstShot,
    ChargedShot,
}
#[derive(Bundle, Clone, Resource)]
struct TriggerZoneBundle {
    // {{{
    sprite_bundle: SpriteBundle,
    collider: Collider,
    trigger_type: TriggerType,
    markers: (ActiveEvents, Sensor),
}
impl TriggerZoneBundle {
    fn new(trigger_type: TriggerType, size: Vec2, translation: Vec3, color: Color) -> Self {
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
            trigger_type,
            markers: (ActiveEvents::COLLISION_EVENTS, Sensor),
        }
    }
    // }}}
}
#[derive(Component, Clone, Copy, Default)]
/// Marker to mark this entity as a worker ball.
struct WorkerBall;
#[derive(Resource, Clone, Default)]
struct WorkerBallSpawner {
    mesh: Mesh2dHandle,
    timer: Timer,
    counter: usize,
}
#[derive(Bundle, Clone, Default)]
struct WorkerBallBundle {
    // {{{
    marker: WorkerBall,
    participant: Participant,
    matmesh: MaterialMesh2dBundle<ColorMaterial>,
    collider: Collider,
    restitution: Restitution,
    rigidbody: RigidBody,
    velocity: Velocity,
}
impl WorkerBallBundle {
    fn new(
        participant: Participant,
        x: f32,
        mesh: Mesh2dHandle,
        material: Handle<ColorMaterial>,
    ) -> Self {
        Self {
            marker: WorkerBall,
            participant,
            matmesh: MaterialMesh2dBundle {
                transform: Transform::from_xyz(x, WORKER_BALL_SPAWN_Y, WORKER_BALL_Z),
                material,
                mesh,
                ..default()
            },
            collider: Collider::ball(WORKER_BALL_RADIUS),
            restitution: Restitution {
                coefficient: WORKER_BALL_RESTITUTION_COEFFICIENT,
                combine_rule: CoefficientCombineRule::Max,
            },
            rigidbody: RigidBody::Dynamic,
            velocity: Velocity::zero(),
        }
    }
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
    let mut timer = Timer::from_seconds(WORKER_BALL_SPAWN_TIMER_SECS, TimerMode::Repeating);
    timer.tick(Duration::from_secs_f32(WORKER_BALL_SPAWN_TIMER_SECS));
    commands.insert_resource(WorkerBallSpawner {
        mesh: Mesh2dHandle(meshes.add(Circle::new(WORKER_BALL_RADIUS))),
        timer,
        counter: 0,
    });
    commands
        .spawn((
            Name::new("PanelRoot"),
            PanelRoot,
            SpatialBundle::from_transform(Transform::from_xyz(ROOT_X_OFFSET, 0.0, 0.0)),
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
                    for j in 1..(i / 2) + 1 {
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

                    for j in 1..=CIRCLE_GRID_HORIZONTAL_HALF_COUNT_EVEN_ROW {
                        let x = j as f32 * (CIRCLE_DIAMETER + CIRCLE_GRID_HORIZONTAL_GAP);
                        parent.spawn(circle_builder.clone().xy(x, y).buildtmb());
                        parent.spawn(circle_builder.clone().xy(-x, y).buildtmb());
                    }
                } else {
                    let x0 = CIRCLE_HALF_GAP + CIRCLE_RADIUS;
                    parent.spawn(circle_builder.clone().xy(x0, y).buildtmb());
                    parent.spawn(circle_builder.clone().xy(-x0, y).buildtmb());
                    for j in 1..CIRCLE_GRID_HORIZONTAL_HALF_COUNT_ODD_ROW {
                        let x = j as f32 * (CIRCLE_DIAMETER + CIRCLE_GRID_HORIZONTAL_GAP) + x0;
                        parent.spawn(circle_builder.clone().xy(x, y).buildtmb());
                        parent.spawn(circle_builder.clone().xy(-x, y).buildtmb());
                    }
                }
            }

            parent.spawn(TriggerZoneBundle::new(
                TriggerType::Multiply,
                Vec2::new(ARENA_WIDTH_FRAC_2, TRIGGER_ZONE_HEIGHT),
                Vec3::new(0.0, TRIGGER_ZONE_Y, TRIGGER_ZONE_Z),
                MULTIPLY_ZONE_COLOR,
            ));
            parent.spawn(TriggerZoneBundle::new(
                TriggerType::BurstShot,
                Vec2::new(ARENA_WIDTH_FRAC_4, TRIGGER_ZONE_HEIGHT),
                Vec3::new(
                    ARENA_WIDTH_FRAC_4 + ARENA_WIDTH_FRAC_8,
                    TRIGGER_ZONE_Y,
                    TRIGGER_ZONE_Z,
                ),
                BURST_SHOT_ZONE_COLOR,
            ));
            parent.spawn(TriggerZoneBundle::new(
                TriggerType::ChargedShot,
                Vec2::new(ARENA_WIDTH_FRAC_4, TRIGGER_ZONE_HEIGHT),
                Vec3::new(
                    -ARENA_WIDTH_FRAC_4 - ARENA_WIDTH_FRAC_8,
                    TRIGGER_ZONE_Y,
                    TRIGGER_ZONE_Z,
                ),
                CHARGED_SHOT_ZONE_COLOR,
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
fn spawn_workers_condition(spawner: Res<WorkerBallSpawner>) -> bool {
    spawner.counter < WORKER_BALL_COUNT_MAX
}
fn spawn_workers(
    mut commands: Commands,
    mut spawner: ResMut<WorkerBallSpawner>,
    time: Res<Time>,
    rapier: Res<RapierContext>,
    participant_info: Res<ParticipantInfo>,
    root: Query<Entity, With<PanelRoot>>,
) {
    if spawner.timer.just_finished() {
        let collider = Collider::ball(WORKER_BALL_RADIUS);
        let mut rng = thread_rng();
        let dist = Uniform::new(-ARENA_WIDTH_FRAC_2, ARENA_WIDTH_FRAC_2);
        let mut f = || loop {
            let x = rng.sample(dist);
            if rapier
                .intersection_with_shape(
                    Vect::new(x, WORKER_BALL_SPAWN_Y),
                    0.0,
                    &collider,
                    QueryFilter::default(),
                )
                .is_none()
            {
                return x;
            }
        };
        fn too_close(a: f32, b: f32) -> bool {
            (a - b).abs() <= WORKER_BALL_DIAMETER
        }
        let x0 = f();
        let x1 = {
            let mut x1 = f();
            while too_close(x0, x1) {
                x1 = f();
            }
            x1
        };
        let x2 = {
            let mut x2 = f();
            while too_close(x0, x2) || too_close(x1, x2) {
                x2 = f();
            }
            x2
        };
        let x3 = {
            let mut x3 = f();
            while too_close(x0, x3) || too_close(x1, x3) || too_close(x2, x3) {
                x3 = f();
            }
            x3
        };
        let root = root.single();
        commands
            .spawn(WorkerBallBundle::new(
                Participant::A,
                x0,
                spawner.mesh.clone(),
                participant_info.colors.a.clone(),
            ))
            .set_parent(root);
        commands
            .spawn(WorkerBallBundle::new(
                Participant::B,
                x1,
                spawner.mesh.clone(),
                participant_info.colors.b.clone(),
            ))
            .set_parent(root);
        commands
            .spawn(WorkerBallBundle::new(
                Participant::C,
                x2,
                spawner.mesh.clone(),
                participant_info.colors.c.clone(),
            ))
            .set_parent(root);
        commands
            .spawn(WorkerBallBundle::new(
                Participant::D,
                x3,
                spawner.mesh.clone(),
                participant_info.colors.d.clone(),
            ))
            .set_parent(root);
        spawner.counter += 1;
    }
    spawner.timer.tick(time.delta());
}
fn trigger_event(
    mut collision_events: EventReader<CollisionEvent>,
    mut event_writer: EventWriter<TriggerEvent>,
    trigger_zone_query: Query<&TriggerType>,
    worker_ball_query: Query<&Participant, With<WorkerBall>>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            &CollisionEvent::Started(a, b, _) => {
                let &trigger_type = if let Ok(x) = trigger_zone_query.get(a) {
                    x
                } else if let Ok(x) = trigger_zone_query.get(b) {
                    x
                } else {
                    continue;
                };
                let &participant = if let Ok(x) = worker_ball_query.get(a) {
                    x
                } else if let Ok(x) = worker_ball_query.get(b) {
                    x
                } else {
                    continue;
                };
                event_writer.send(TriggerEvent {
                    participant,
                    trigger_type,
                });
            }
            CollisionEvent::Stopped(_, _, _) => (),
        }
    }
}
fn print_trigger_events(mut events: EventReader<TriggerEvent>) {
    for event in events.read() {
        println!("{:#?}", event);
    }
}
fn ball_reset(
    mut collision_events: EventReader<CollisionEvent>,
    rapier: Res<RapierContext>,
    trigger_zone_query: Query<(), With<TriggerType>>,
    mut worker_ball_query: Query<(&mut Transform, &mut Velocity, &Collider), With<WorkerBall>>,
) {
    for collision_event in collision_events.read() {
        match collision_event {
            CollisionEvent::Started(_, _, _) => (),
            &CollisionEvent::Stopped(a, b, _) => {
                let ball_entity = if trigger_zone_query.get(a).is_ok() {
                    b
                } else if trigger_zone_query.get(b).is_ok() {
                    a
                } else {
                    continue;
                };
                let Ok((mut ball_transform, mut velocity, collider)) =
                    worker_ball_query.get_mut(ball_entity)
                else {
                    continue;
                };

                let x = {
                    let mut rng = thread_rng();
                    let dist = Uniform::new(-ARENA_WIDTH_FRAC_2, ARENA_WIDTH_FRAC_2);
                    let mut x;
                    loop {
                        x = rng.sample(dist);
                        if rapier
                            .intersection_with_shape(
                                Vect::new(x, WORKER_BALL_SPAWN_Y),
                                0.0,
                                collider,
                                QueryFilter::default(),
                            )
                            .is_none()
                        {
                            break;
                        }
                    }
                    x
                };

                ball_transform.translation.x = x;
                ball_transform.translation.y = WORKER_BALL_SPAWN_Y;
                *velocity = Velocity::zero();
            }
        }
    }
}
