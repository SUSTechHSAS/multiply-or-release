#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use std::{
    collections::VecDeque,
    f32::consts::{FRAC_PI_2, PI},
};

use bevy::{color::palettes::css, prelude::*, sprite::Mesh2dHandle, time::Stopwatch};
use bevy_hanabi::prelude::*;
use bevy_rapier2d::prelude::*;

use crate::{
    collision_groups,
    panel_plugin::{TriggerEvent, TriggerType},
    utils::{
        BallColor, EffectPropertiesExt, Participant, ParticipantMap, TileColor, TileHitEffect,
    },
};

// Constants {{{

const TILE_COUNT: usize = 100;
const TILE_DIMENSION: f32 = BATTLEFIELD_HALF_WIDTH / TILE_COUNT as f32;
pub const BATTLEFIELD_HALF_WIDTH: f32 = 360.0;
const BATTLEFIELD_BOUNDARY_HALF_WIDTH: f32 = 50.0;

const TURRET_POSITION: f32 = 350.0;
const TURRET_HEAD_COLOR: Color = Color::Srgba(css::DARK_GRAY);
const TURRET_HEAD_THICNESS: f32 = 2.5;
const TURRET_HEAD_LENGTH: f32 = 75.0;
const TURRET_ROTATION_SPEED: f32 = 1.0;

const MULTI_SHOT_CHARGE_OFFSET: u64 = 4;

const BULLET_TEXT_COLOR: Color = Color::BLACK;
const BULLET_TEXT_FONT_SIZE_ASPECT: f32 = 0.5;
const BULLET_MINIMUM_TEXT_SIZE: f32 = 8.0;
const BULLET_SIZE_FACTOR: f32 = 2.0;
const BULLET_MASS_FACTOR: f64 = 1.0;
const BULLET_RESTITUTION_COEFFICIENT: f32 = 0.75;
const CHARGED_SHOT_BULLET_SPEED: f32 = 250.0;
const BURST_SHOT_BULLET_SPEED: f32 = 500.0;

const ONE_SHOT_PROTECTION_THRESHOLD: u64 = 10;
const ONE_SHOT_DAMAGE_THRESHOLD: u64 = 1024;

// Z-index
const TILE_Z: f32 = -1.0;
const BULLET_BALL_Z: f32 = -1.0;
const BULLET_TEXT_Z: f32 = 3.0;
// Turret head is a child of turret, which inherits the z position as well, so the local z of the
// head needs to be negative to put it behind the main turret.
const TURRET_HEAD_Z: f32 = -1.0;
const TURRET_PLATFORM_Z: f32 = -1.0;

// }}}

pub struct BattlefieldPlugin;
impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<EliminationEvent>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    rotate_turret,
                    handle_trigger_events
                        .run_if(game_is_going)
                        .after(handle_bullet_turret_collision),
                    handle_bullet_tile_collision,
                    handle_bullet_turret_collision
                        .run_if(game_is_going)
                        .after(handle_bullet_tile_collision),
                    update_charge_level.after(handle_bullet_turret_collision),
                    update_charge_ball.after(update_charge_level),
                    handle_elimination
                        .run_if(on_event::<EliminationEvent>())
                        .after(handle_bullet_turret_collision),
                    cleanup_particle_emitters.before(handle_bullet_tile_collision),
                ),
            )
            .add_systems(
                FixedUpdate,
                fire_shots
                    .run_if(game_is_going)
                    .after(handle_trigger_events),
            );
    }
}

#[derive(Resource, Clone, Default)]
struct EffectInstanceManager {
    pool: Vec<Entity>,
    dispatched: Vec<Entity>,
}
impl EffectInstanceManager {
    fn add(&mut self, entity: Entity) {
        self.dispatched.push(entity);
    }
    fn get(&mut self) -> Option<Entity> {
        if let Some(entity) = self.pool.pop() {
            self.dispatched.push(entity);
            Some(entity)
        } else {
            None
        }
    }
    fn reset(&mut self) {
        self.pool.append(&mut self.dispatched);
    }
}
#[derive(Event)]
pub struct EliminationEvent {
    pub participant: Participant,
}
#[derive(Resource)]
pub struct SurvivorCount(pub u8);
impl Default for SurvivorCount {
    fn default() -> Self {
        Self(4)
    }
}
#[derive(Component)]
struct BattlefieldRoot;
/// Marker to mark this entity as a tile.
#[derive(Component, Clone, Copy)]
struct Tile;
/// Component bundle for each of the individual tiles on the battle field.
#[derive(Bundle)]
struct TileBundle {
    /// Markers to mark this entity as a tile, a sensor collider, and a trigger for collision
    /// events.
    markers: (Tile, Sensor),
    /// Bevy rendering component used to display the tile.
    sprite_bundle: SpriteBundle,
    /// Rapier collider component. We'll mark this as sensor and won't add a rigidbody to this
    /// entity because we don't actually want the physics engine to move itl.
    collider: Collider,
    collision_groups: CollisionGroups,
    /// The game participant that owns this tile.
    owner: Participant,
}
impl TileBundle {
    fn new(owner: Participant, color: Color, x: f32, y: f32) -> Self {
        Self {
            markers: (Tile, Sensor),
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
            collision_groups: CollisionGroups::new(
                collision_groups::tile(owner),
                collision_groups::all_bullets_except(owner),
            ),
            owner,
        }
    }
}
#[derive(Resource, Default, Clone)]
struct TurretStopwatch(Stopwatch);
impl TurretStopwatch {
    fn get(&self) -> f32 {
        FRAC_PI_2 - ((self.0.elapsed_secs() % PI * TURRET_ROTATION_SPEED) % PI - FRAC_PI_2).abs()
    }
}
#[derive(Component, Deref, Clone, Copy)]
struct ChargeBallLink(Entity);
#[derive(Debug, Component, Clone, Copy)]
struct Charge {
    value: u64,
    level: u64,
}
impl Default for Charge {
    fn default() -> Self {
        Self { value: 1, level: 1 }
    }
}
impl Charge {
    fn from_value(value: u64) -> Self {
        let mut v = Self { value, level: 1 };
        v.update_level();
        v
    }
    fn update_level(&mut self) {
        self.level = (self.value as f64).log2().ceil() as u64 + 1;
    }
    fn multiply(&mut self) {
        if let Some(value) = self.value.checked_mul(4) {
            self.value = value;
            self.level += 2;
        } else {
            self.value = u64::MAX;
            self.update_level()
        }
    }
    fn reset(&mut self) {
        self.value = 1;
        self.level = 1;
    }
    fn get_scale(&self) -> f32 {
        self.level as f32 * BULLET_SIZE_FACTOR
    }
}
#[derive(Bundle)]
struct ChargeBallBundle {
    matmesh: ColorMesh2dBundle,
}
impl ChargeBallBundle {
    fn new(mesh: Mesh2dHandle, material: Handle<ColorMaterial>) -> Self {
        Self {
            matmesh: ColorMesh2dBundle {
                transform: Transform::from_xyz(0.0, 0.0, BULLET_BALL_Z),
                mesh,
                material,
                ..default()
            },
        }
    }
}
#[derive(Resource, Deref)]
struct BulletMesh(Mesh2dHandle);
#[derive(Component)]
struct Bullet;
/// Component bundle for the bullets that the turrets fire.
#[derive(Bundle)]
struct BulletBundle {
    /// Marker to mark this entity as a bullet.
    markers: (
        Bullet,
        GravityScale,
        Friction,
        Restitution,
        LockedAxes,
        ActiveEvents,
    ),
    charge: Charge,
    link: ChargeBallLink,
    /// Rapier collider component.
    collider: Collider,
    collision_groups: CollisionGroups,
    collider_scale: ColliderScale,
    velocity: Velocity,
    /// Rapier rigidbody component, used by the physics engine to move the entity.
    rigidbody: RigidBody,
    mass: ColliderMassProperties,
    /// The game participant that owns this bullet.
    owner: Participant,
    text_bundle: Text2dBundle,
}
impl BulletBundle {
    fn new(
        owner: Participant,
        position: Vec2,
        ball: Entity,
        charge: Charge,
        firing_angle: f32,
        bullet_speed: f32,
    ) -> Self {
        let direction = Vec2::from_angle(firing_angle);
        Self {
            owner,
            charge,
            link: ChargeBallLink(ball),
            markers: (
                Bullet,
                GravityScale(0.0),
                Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                Restitution {
                    coefficient: BULLET_RESTITUTION_COEFFICIENT,
                    combine_rule: CoefficientCombineRule::Max,
                },
                LockedAxes::ROTATION_LOCKED,
                ActiveEvents::COLLISION_EVENTS,
            ),
            collider: Collider::ball(1.0),
            collision_groups: CollisionGroups::new(
                collision_groups::bullet(owner),
                collision_groups::BATTLEFIELD_ROOT
                    | collision_groups::ALL_BULLETS
                    | collision_groups::all_tiles_except(owner)
                    | collision_groups::all_turrets_except(owner),
            ),
            collider_scale: ColliderScale::Absolute(Vect::splat(1.0)),
            velocity: Velocity::linear(direction * bullet_speed),
            rigidbody: RigidBody::Dynamic,
            mass: ColliderMassProperties::Mass((charge.value as f64 * BULLET_MASS_FACTOR) as f32),
            text_bundle: Text2dBundle {
                transform: Transform::from_translation(position.extend(BULLET_TEXT_Z)),
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: Default::default(),
                        font_size: BULLET_SIZE_FACTOR,
                        color: BULLET_TEXT_COLOR,
                    },
                ),
                ..default()
            },
        }
    }
}
#[derive(Debug, Clone, Copy)]
enum ShotType {
    Charged,
    Multi,
}
#[derive(Component, Default, Deref, DerefMut)]
struct FiringQueue(VecDeque<(ShotType, Charge)>);
#[derive(Bundle)]
struct TurretBundle {
    sensor: Sensor,
    firing_queue: FiringQueue,
    charge: Charge,
    link: ChargeBallLink,
    platform: TurretPlatformLink,
    text_bundle: Text2dBundle,
    owner: Participant,
    collider: Collider,
    collision_groups: CollisionGroups,
    collider_scale: ColliderScale,
}
impl TurretBundle {
    fn new(owner: Participant, x: f32, y: f32, ball: Entity, platform: Entity) -> Self {
        Self {
            owner,
            sensor: Sensor,
            firing_queue: FiringQueue::default(),
            charge: Charge::default(),
            link: ChargeBallLink(ball),
            platform: TurretPlatformLink(platform),
            collider: Collider::ball(1.0),
            collision_groups: CollisionGroups::new(
                collision_groups::turret(owner),
                collision_groups::all_bullets_except(owner),
            ),
            collider_scale: ColliderScale::Absolute(Vect::splat(1.0)),
            text_bundle: Text2dBundle {
                transform: Transform::from_xyz(x, y, BULLET_TEXT_Z),
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: Default::default(),
                        font_size: BULLET_SIZE_FACTOR,
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
                    translation: Vec3::new(TURRET_HEAD_LENGTH / 2.0, 0.0, TURRET_HEAD_Z),
                    scale: Vec3::new(TURRET_HEAD_LENGTH, TURRET_HEAD_THICNESS, 1.0),
                    rotation: Quat::IDENTITY,
                },
                ..default()
            },
        }
    }
}
#[derive(Component)]
struct TurretPlatformLink(Entity);
/// Component for a turret.
#[derive(Component, Default)]
struct BarrelOffset(f32);
/// Component bundle for a turret.
#[derive(Bundle, Default)]
struct TurretPlatformBundle {
    /// Bevy rendering component used to display the ball.
    barrel_offset: BarrelOffset,
    spatial: SpatialBundle,
}
impl TurretPlatformBundle {
    fn new(base_offset: f32) -> Self {
        Self {
            barrel_offset: BarrelOffset(base_offset),
            spatial: SpatialBundle::from_transform(Transform::from_xyz(
                0.0,
                0.0,
                TURRET_PLATFORM_Z,
            )),
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    colors: Res<ParticipantMap<TileColor>>,
    materials: Res<ParticipantMap<Handle<ColorMaterial>>>,
) {
    commands.insert_resource(EffectInstanceManager::default());
    commands.insert_resource(TurretStopwatch::default());
    commands.insert_resource(SurvivorCount::default());
    const OFFSET: f32 = BATTLEFIELD_HALF_WIDTH + BATTLEFIELD_BOUNDARY_HALF_WIDTH;
    let horizontal_cuboid = Collider::cuboid(
        BATTLEFIELD_HALF_WIDTH + BATTLEFIELD_BOUNDARY_HALF_WIDTH * 2.0,
        BATTLEFIELD_BOUNDARY_HALF_WIDTH,
    );
    let vertical_cuboid = Collider::cuboid(
        BATTLEFIELD_BOUNDARY_HALF_WIDTH,
        BATTLEFIELD_HALF_WIDTH + BATTLEFIELD_BOUNDARY_HALF_WIDTH * 2.0,
    );
    let collider = Collider::compound(vec![
        (Vect::new(OFFSET, 0.0), 0.0, vertical_cuboid.clone()),
        (Vect::new(-OFFSET, 0.0), 0.0, vertical_cuboid.clone()),
        (Vect::new(0.0, OFFSET), 0.0, horizontal_cuboid.clone()),
        (Vect::new(0.0, -OFFSET), 0.0, horizontal_cuboid.clone()),
    ]);
    let root = commands
        .spawn((
            Name::new("Battlefield Root"),
            BattlefieldRoot,
            RigidBody::Fixed,
            CollisionGroups::new(
                collision_groups::BATTLEFIELD_ROOT,
                collision_groups::ALL_BULLETS,
            ),
            Restitution {
                coefficient: 1.0,
                combine_rule: CoefficientCombineRule::Max,
            },
            collider,
            SpatialBundle::default(),
        ))
        .id();
    let battlefield = commands
        .spawn((Name::new("Battlefield"), SpatialBundle::default()))
        .set_parent(root)
        .id();
    for i in 0..TILE_COUNT {
        let x = TILE_DIMENSION / 2.0 + i as f32 * TILE_DIMENSION;
        for j in 0..TILE_COUNT {
            let y = TILE_DIMENSION / 2.0 + j as f32 * TILE_DIMENSION;
            commands
                .spawn(TileBundle::new(Participant::A, colors.a.0, x, y))
                .set_parent(battlefield);
            commands
                .spawn(TileBundle::new(Participant::B, colors.b.0, -x, y))
                .set_parent(battlefield);
            commands
                .spawn(TileBundle::new(Participant::C, colors.c.0, x, -y))
                .set_parent(battlefield);
            commands
                .spawn(TileBundle::new(Participant::D, colors.d.0, -x, -y))
                .set_parent(battlefield);
        }
    }
    let mesh = Mesh2dHandle(meshes.add(Circle::new(1.0)));
    let mut spawn_turret = |owner: Participant, base_offset: f32, x: f32, y: f32| {
        let ball = commands
            .spawn(ChargeBallBundle::new(
                mesh.clone(),
                materials.get(owner).clone(),
            ))
            .id();
        let platform = commands
            .spawn(TurretPlatformBundle::new(base_offset))
            .set_parent(root)
            .id();
        commands.spawn(TurretHeadBundle::new()).set_parent(platform);
        commands
            .spawn(TurretBundle::new(owner, x, y, ball, platform))
            .set_parent(root)
            .push_children(&[ball, platform])
            .id()
    };
    let a = spawn_turret(Participant::A, PI, TURRET_POSITION, TURRET_POSITION);
    let b = spawn_turret(
        Participant::B,
        -FRAC_PI_2,
        -TURRET_POSITION,
        TURRET_POSITION,
    );
    let c = spawn_turret(Participant::C, FRAC_PI_2, TURRET_POSITION, -TURRET_POSITION);
    let d = spawn_turret(Participant::D, 0.0, -TURRET_POSITION, -TURRET_POSITION);
    commands.insert_resource(ParticipantMap::new(a, b, c, d));
    commands.insert_resource(BulletMesh(mesh));
}
fn rotate_turret(
    time: Res<Time>,
    mut stopwatch: ResMut<TurretStopwatch>,
    mut turrets: Query<(&mut Transform, &BarrelOffset)>,
) {
    stopwatch.0.tick(time.delta());
    let angle_offset = stopwatch.get();
    for (mut transform, &BarrelOffset(base_offset)) in &mut turrets {
        *transform = transform.with_rotation(Quat::from_rotation_z(base_offset + angle_offset));
    }
}
fn update_charge_level(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Charge), Changed<Charge>>,
) {
    for (entity, mut charge) in &mut query {
        if charge.value == 0 {
            commands.entity(entity).despawn_recursive();
            continue;
        }
        charge.update_level();
    }
}
fn update_charge_ball(
    mut balls: Query<
        (
            &mut ColliderScale,
            &mut Text,
            &Charge,
            &ChargeBallLink,
            Entity,
        ),
        Or<(Changed<Charge>, Added<Charge>)>,
    >,
    turret_query: Query<(), With<FiringQueue>>,
    mut transform_query: Query<&mut Transform>,
) {
    for (mut collider_scale, mut text, charge, &ChargeBallLink(link), entity) in &mut balls {
        let mut scale = charge.get_scale();
        if scale < BULLET_MINIMUM_TEXT_SIZE && turret_query.get(entity).is_ok() {
            scale = BULLET_MINIMUM_TEXT_SIZE;
        }
        *collider_scale = ColliderScale::Absolute(Vect::splat(scale));
        let mut ball_transform = transform_query.get_mut(link).unwrap();
        ball_transform.scale.x = scale;
        ball_transform.scale.y = scale;
        let diameter = scale * 2.0;
        let section = &mut text.sections[0];
        if diameter < BULLET_MINIMUM_TEXT_SIZE {
            section.value.clear();
        } else {
            section.value = charge.value.to_string();
            let digit_count = section.value.len() as f32;
            let full_size_horizontal = diameter * BULLET_TEXT_FONT_SIZE_ASPECT * digit_count;
            if diameter < full_size_horizontal {
                section.style.font_size = diameter / digit_count / BULLET_TEXT_FONT_SIZE_ASPECT;
            } else {
                section.style.font_size = diameter;
            }
        }
    }
}
fn fire_shots(
    mut commands: Commands,
    rapier: Res<RapierContext>,
    mesh: Res<BulletMesh>,
    materials: Res<ParticipantMap<Handle<ColorMaterial>>>,
    turret_stopwatch: Res<TurretStopwatch>,
    mut turrets: Query<(
        &mut FiringQueue,
        &Transform,
        &GlobalTransform,
        &Participant,
        &TurretPlatformLink,
    )>,
    platform_query: Query<&BarrelOffset>,
    battlefield_root: Query<Entity, With<BattlefieldRoot>>,
) {
    for (mut turret, transform, global_transform, &owner, &TurretPlatformLink(link)) in &mut turrets
    {
        let Some((shot_type, charge)) = turret.pop_back() else {
            continue;
        };
        let get_offset = |radius: f32| {
            let translation = transform.translation;
            let absx = translation.x.abs();
            let abs_offset = absx - absx.min(BATTLEFIELD_HALF_WIDTH - radius);
            Vec2::new(translation.x.signum(), translation.y.signum()) * abs_offset
        };
        let shape_cast = |radius: f32, offset: Vec2| {
            rapier
                .intersection_with_shape(
                    global_transform.translation().xy() - offset,
                    0.0,
                    &Collider::ball(radius),
                    QueryFilter::only_dynamic().groups(CollisionGroups::new(
                        collision_groups::bullet(owner),
                        collision_groups::ALL_BULLETS,
                    )),
                )
                .is_some()
        };
        let (charge, offset, bullet_speed) = match shot_type {
            ShotType::Charged => {
                let radius = charge.get_scale();
                let offset = get_offset(radius);
                if shape_cast(radius, offset) {
                    turret.push_back((shot_type, charge));
                    continue;
                } else {
                    (charge, offset, CHARGED_SHOT_BULLET_SPEED)
                }
            }
            ShotType::Multi => {
                let shot_value = match charge.level.checked_sub(MULTI_SHOT_CHARGE_OFFSET) {
                    None | Some(0) => 1,
                    Some(value) => value,
                };
                let shot = Charge::from_value(shot_value);
                let radius = shot.get_scale();
                let offset = get_offset(radius);
                if shape_cast(radius, offset) {
                    turret.push_back((shot_type, charge));
                    continue;
                } else {
                    let mut charge = charge;
                    match charge.value.checked_sub(shot.value) {
                        None | Some(0) => (),
                        Some(remaining_value) => {
                            charge.value = remaining_value;
                            charge.update_level();
                            turret.push_back((shot_type, charge));
                        }
                    }
                    (shot, offset, BURST_SHOT_BULLET_SPEED)
                }
            }
        };
        let &BarrelOffset(base_angle) = platform_query.get(link).unwrap();
        let ball = commands
            .spawn(ChargeBallBundle::new(
                mesh.clone(),
                materials.get(owner).clone(),
            ))
            .id();
        commands
            .spawn(BulletBundle::new(
                owner,
                transform.translation.xy() - offset,
                ball,
                charge,
                turret_stopwatch.get() + base_angle,
                bullet_speed,
            ))
            .set_parent(battlefield_root.single())
            .add_child(ball);
    }
}
fn handle_trigger_events(
    mut reader: EventReader<TriggerEvent>,
    participants: Res<ParticipantMap<Entity>>,
    mut turret_query: Query<(&mut Charge, &mut FiringQueue)>,
) {
    for event in reader.read() {
        let &entity = participants.get(event.participant);
        let Ok((mut charge, mut turret)) = turret_query.get_mut(entity) else {
            continue;
        };
        match event.trigger_type {
            TriggerType::Multiply => charge.multiply(),
            TriggerType::BurstShot => {
                turret.push_front((ShotType::Multi, *charge));
                charge.reset();
            }
            TriggerType::ChargedShot => {
                turret.push_front((ShotType::Charged, *charge));
                charge.reset();
            }
        }
    }
}
fn handle_bullet_turret_collision(
    mut commands: Commands,
    mut collision_event_reader: EventReader<CollisionEvent>,
    mut elimination_event_writer: EventWriter<EliminationEvent>,
    mut bullet_query: Query<(Entity, &Participant, &mut Charge, &mut Velocity), With<Bullet>>,
    mut turret_query: Query<(&Participant, &mut Charge), (With<FiringQueue>, Without<Bullet>)>,
) {
    for event in collision_event_reader.read() {
        match event {
            &CollisionEvent::Started(a, b, _) => {
                let (bullet_entity, &bullet_owner, mut bullet_charge, mut velocity) =
                    if let Ok(x) = bullet_query.get_mut(a) {
                        x
                    } else if let Ok(x) = bullet_query.get_mut(b) {
                        x
                    } else {
                        continue;
                    };
                let (&turret_owner, mut turret_charge) = if let Ok(x) = turret_query.get_mut(a) {
                    x
                } else if let Ok(x) = turret_query.get_mut(b) {
                    x
                } else {
                    continue;
                };
                if turret_owner == bullet_owner {
                    continue;
                }
                if bullet_charge.value < turret_charge.value {
                    turret_charge.value -= bullet_charge.value;
                    commands.entity(bullet_entity).despawn_recursive();
                } else {
                    bullet_charge.value -= turret_charge.value;
                    let mut kill = || {
                        elimination_event_writer.send(EliminationEvent {
                            participant: turret_owner,
                        });
                    };
                    if turret_charge.level < ONE_SHOT_PROTECTION_THRESHOLD {
                        kill();
                    } else if bullet_charge.value > ONE_SHOT_DAMAGE_THRESHOLD {
                        bullet_charge.value = bullet_charge
                            .value
                            .saturating_sub(ONE_SHOT_DAMAGE_THRESHOLD);
                        if bullet_charge.value == 0 {
                            kill();
                            commands.entity(bullet_entity).despawn_recursive();
                        }
                    } else {
                        turret_charge.reset();
                    }
                    velocity.linvel *= -1.0;
                }
                let min_value = bullet_charge.value.min(turret_charge.value);
                bullet_charge.value -= min_value;
                turret_charge.value -= min_value;
            }
            CollisionEvent::Stopped(_, _, _) => (),
        }
    }
}
fn handle_elimination(
    mut commands: Commands,
    mut events: EventReader<EliminationEvent>,
    mut survivor_count: ResMut<SurvivorCount>,
    participant_entity_query: Query<(Entity, &Participant), (Without<Tile>, Without<Bullet>)>,
) {
    for event in events.read() {
        survivor_count.0 -= 1;
        for (e, &p) in &participant_entity_query {
            if p == event.participant {
                commands.entity(e).despawn_recursive();
            }
        }
    }
}
fn handle_bullet_tile_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    tile_colors: Res<ParticipantMap<TileColor>>,
    ball_colors: Res<ParticipantMap<BallColor>>,
    mut bullet_query: Query<(&Participant, &mut Charge, &Velocity), With<Bullet>>,
    mut tile_query: Query<
        (
            &mut Participant,
            &mut Sprite,
            &mut CollisionGroups,
            &GlobalTransform,
        ),
        (With<Tile>, Without<Bullet>),
    >,
    effect: Res<TileHitEffect>,
    mut effect_query: Query<(&mut EffectProperties, &mut Transform, &mut EffectSpawner)>,
    mut instance_manager: ResMut<EffectInstanceManager>,
) {
    for event in events.read() {
        match event {
            &CollisionEvent::Started(a, b, _) => {
                let (&bullet_owner, mut charge, velocity) = if let Ok(x) = bullet_query.get_mut(a) {
                    x
                } else if let Ok(x) = bullet_query.get_mut(b) {
                    x
                } else {
                    continue;
                };
                let (mut tile_owner, mut sprite, mut collision_group, tile_transform) =
                    if let Ok(x) = tile_query.get_mut(a) {
                        x
                    } else if let Ok(x) = tile_query.get_mut(b) {
                        x
                    } else {
                        continue;
                    };
                if bullet_owner == *tile_owner {
                    continue;
                }
                if charge.value == 0 {
                    continue;
                }
                *tile_owner = bullet_owner;
                sprite.color = tile_colors.get(bullet_owner).0;
                *collision_group = CollisionGroups::new(
                    collision_groups::tile(bullet_owner),
                    collision_groups::all_bullets_except(bullet_owner),
                );
                charge.value -= 1;
                if let Some(effect_entity) = instance_manager.get() {
                    let (mut properties, mut transform, mut spawner) = effect_query.get_mut(effect_entity).expect("entity returned by `InstanceManager` should have an `EffectProperties` component.");
                    properties.set_spawn_color(ball_colors.get(bullet_owner).0);
                    properties.set_bullet_vel(velocity.linvel);
                    transform.translation = tile_transform.translation();
                    spawner.reset();
                } else {
                    let entity = commands
                        .spawn(ParticleEffectBundle {
                            effect: ParticleEffect::new(effect.0.clone()),
                            transform: Transform::from_translation(tile_transform.translation()),
                            ..default()
                        })
                        .id();
                    instance_manager.add(entity);
                }
            }
            CollisionEvent::Stopped(_, _, _) => (),
        }
    }
}
pub fn game_is_going(survivor_count: Res<SurvivorCount>) -> bool {
    survivor_count.0 > 1
}
fn cleanup_particle_emitters(mut instance_manager: ResMut<EffectInstanceManager>) {
    instance_manager.reset();
}
