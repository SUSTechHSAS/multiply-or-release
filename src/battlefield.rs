#![allow(clippy::type_complexity, clippy::too_many_arguments)]

use std::f32::consts::{FRAC_PI_2, PI};

use bevy::{prelude::*, sprite::Mesh2dHandle, time::Stopwatch};
use bevy_rapier2d::prelude::*;

use crate::{
    panel_plugin::{TriggerEvent, TriggerType},
    utils::{Participant, ParticipantMap},
};

// Constants {{{

const TILE_BORDER_COLOR: Color = Color::BLACK;
const TILE_BORDER_THICNESS: f32 = 0.4;
const TILE_COUNT: usize = 100;
const TILE_DIMENSION: f32 = 3.2;

const BATTLEFIELD_BOUNDARY: f32 = TILE_COUNT as f32 * (TILE_DIMENSION + TILE_BORDER_THICNESS);

const TURRET_POSITION: f32 = 350.0;
const TURRET_HEAD_COLOR: Color = Color::DARK_GRAY;
const TURRET_HEAD_THICNESS: f32 = 2.5;
const TURRET_HEAD_LENGTH: f32 = 75.0;
const TURRET_ROTATION_SPEED: f32 = 1.0;

const BULLET_TEXT_COLOR: Color = Color::BLACK;
const BULLET_TEXT_FONT_SIZE_ASPECT: f32 = 0.5;
const BULLET_RADIUS_FACTOR: f32 = 5.0;
const BULLET_FIRE_FORCE: f32 = 100.0;
const BULLET_MASS_FACTOR: f32 = 1.0;

const ONE_SHOT_PROTECTION_THRESHOLD: f32 = 10.0;
const ONE_SHOT_DAMAGE_THRESHOLD: f32 = 1024.0;

// Z-index
const TILE_Z: f32 = 10.0;
const BULLET_BALL_Z: f32 = -1.0;
const BULLET_TEXT_Z: f32 = 20.0;
// Turret head is a child of turret, which inherits the z position as well, so the local z of the
// head needs to be negative to put it behind the main turret.
const TURRET_HEAD_Z: f32 = -1.0;
const TURRET_PLATFORM_Z: f32 = -1.0;

// }}}

pub struct BattlefieldPlugin;
impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup).add_systems(
            Update,
            (
                rotate_turret,
                handle_trigger_events.after(handle_bullet_turret_collision),
                handle_bullet_tile_collision,
                handle_bullet_turret_collision.after(handle_bullet_tile_collision),
                update_charge_level.after(handle_bullet_turret_collision),
                update_charge_text.after(update_charge_level),
                update_charge_ball.after(update_charge_level),
                update_tile_color.after(handle_bullet_tile_collision),
            ),
        );
        // .insert_resource(AutoTimer::default())
        // .add_systems(Update, auto_fire);
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
#[derive(Component, Clone, Copy)]
struct Charge {
    value: f32,
    level: f32,
    link: Entity,
}
impl From<Entity> for Charge {
    fn from(link: Entity) -> Self {
        Self {
            value: 1.0,
            level: 1.0,
            link,
        }
    }
}
impl Charge {
    fn new(value: f32, level: f32, link: Entity) -> Self {
        Self { value, level, link }
    }
    fn multiply(&mut self) {
        self.value *= 2.0;
        self.level += 1.0;
    }
    fn reset(&mut self) {
        self.value = 1.0;
        self.level = 1.0;
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
#[derive(Resource)]
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
    /// Rapier collider component.
    collider: Collider,
    collider_scale: ColliderScale,
    velocity: Velocity,
    /// Rapier rigidbody component, used by the physics engine to move the entity.
    rigidbody: RigidBody,
    mass: ColliderMassProperties,
    impulse: ExternalImpulse,
    /// The game participant that owns this bullet.
    owner: Participant,
    text_bundle: Text2dBundle,
}
impl BulletBundle {
    fn new(
        owner: Participant,
        x: f32,
        y: f32,
        ball: Entity,
        charge: &Charge,
        firing_angle: f32,
    ) -> Self {
        Self {
            owner,
            charge: Charge::new(charge.value, charge.level, ball),
            markers: (
                Bullet,
                GravityScale(0.0),
                Friction {
                    coefficient: 0.0,
                    combine_rule: CoefficientCombineRule::Min,
                },
                Restitution {
                    coefficient: 1.0,
                    combine_rule: CoefficientCombineRule::Max,
                },
                LockedAxes::ROTATION_LOCKED,
                ActiveEvents::COLLISION_EVENTS,
            ),
            collider: Collider::ball(1.0),
            collider_scale: ColliderScale::Absolute(Vect::splat(1.0)),
            velocity: Velocity::default(),
            rigidbody: RigidBody::Dynamic,
            mass: ColliderMassProperties::Mass(charge.level * BULLET_MASS_FACTOR),
            impulse: ExternalImpulse {
                impulse: Vec2::from_angle(firing_angle) * BULLET_FIRE_FORCE,
                torque_impulse: 0.0,
            },
            text_bundle: Text2dBundle {
                transform: Transform::from_xyz(x, y, BULLET_TEXT_Z),
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: Default::default(),
                        font_size: BULLET_RADIUS_FACTOR,
                        color: BULLET_TEXT_COLOR,
                    },
                ),
                ..default()
            },
        }
    }
}
#[derive(Component, Default)]
struct Turret;
#[derive(Bundle)]
struct TurretBundle {
    marker: (Turret, Sensor),
    charge: Charge,
    platform: TurretPlatformLink,
    text_bundle: Text2dBundle,
    owner: Participant,
    collider: Collider,
    collider_scale: ColliderScale,
}
impl TurretBundle {
    fn new(owner: Participant, x: f32, y: f32, ball: Entity, platform: Entity) -> Self {
        Self {
            marker: (Turret, Sensor),
            owner,
            charge: Charge::from(ball),
            platform: TurretPlatformLink(platform),
            collider: Collider::ball(1.0),
            collider_scale: ColliderScale::Absolute(Vect::splat(1.0)),
            text_bundle: Text2dBundle {
                transform: Transform::from_xyz(x, y, BULLET_TEXT_Z),
                text: Text::from_section(
                    "",
                    TextStyle {
                        font: Default::default(),
                        font_size: BULLET_RADIUS_FACTOR,
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
#[allow(dead_code)]
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
    colors: Res<ParticipantMap<Color>>,
    materials: Res<ParticipantMap<Handle<ColorMaterial>>>,
) {
    commands.insert_resource(TurretStopwatch::default());
    let root = commands
        .spawn((
            Name::new("Battlefield Root"),
            BattlefieldRoot,
            RigidBody::Fixed,
            Collider::polyline(
                vec![
                    Vect::new(BATTLEFIELD_BOUNDARY, BATTLEFIELD_BOUNDARY),
                    Vect::new(-BATTLEFIELD_BOUNDARY, BATTLEFIELD_BOUNDARY),
                    Vect::new(-BATTLEFIELD_BOUNDARY, -BATTLEFIELD_BOUNDARY),
                    Vect::new(BATTLEFIELD_BOUNDARY, -BATTLEFIELD_BOUNDARY),
                    Vect::new(BATTLEFIELD_BOUNDARY, BATTLEFIELD_BOUNDARY),
                ],
                None,
            ),
            SpriteBundle {
                sprite: Sprite {
                    color: TILE_BORDER_COLOR,
                    ..default()
                },
                ..default()
            },
        ))
        .id();
    let battlefield = commands
        .spawn((Name::new("Battlefield"), SpatialBundle::default()))
        .set_parent(root)
        .id();
    for i in 0..TILE_COUNT {
        let x = (TILE_DIMENSION + TILE_BORDER_THICNESS) / 2.0
            + i as f32 * (TILE_DIMENSION + TILE_BORDER_THICNESS);
        for j in 0..TILE_COUNT {
            let y = (TILE_DIMENSION + TILE_BORDER_THICNESS) / 2.0
                + j as f32 * (TILE_DIMENSION + TILE_BORDER_THICNESS);
            commands
                .spawn(TileBundle::new(Participant::A, colors.a, x, y))
                .set_parent(battlefield);
            commands
                .spawn(TileBundle::new(Participant::B, colors.b, -x, y))
                .set_parent(battlefield);
            commands
                .spawn(TileBundle::new(Participant::C, colors.c, x, -y))
                .set_parent(battlefield);
            commands
                .spawn(TileBundle::new(Participant::D, colors.d, -x, -y))
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
    let a = spawn_turret(Participant::A, -FRAC_PI_2, TURRET_POSITION, TURRET_POSITION);
    let b = spawn_turret(Participant::B, 0.0, -TURRET_POSITION, TURRET_POSITION);
    let c = spawn_turret(Participant::C, PI, TURRET_POSITION, -TURRET_POSITION);
    let d = spawn_turret(
        Participant::D,
        FRAC_PI_2,
        -TURRET_POSITION,
        -TURRET_POSITION,
    );
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
        *transform = transform.with_rotation(Quat::from_rotation_z(base_offset - angle_offset));
    }
}
fn update_charge_level(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Charge), Changed<Charge>>,
) {
    for (entity, mut charge) in &mut query {
        while 2f32.powf(charge.level - 1.0) > charge.value {
            charge.level -= 1.0;
            if charge.level < 1.0 {
                commands.entity(entity).despawn_recursive();
                break;
            }
        }
    }
}
fn update_charge_text(
    mut query: Query<(&mut Text, &Charge), Or<(Changed<Charge>, Added<Charge>)>>,
) {
    for (mut text, charge) in &mut query {
        let section = &mut text.sections[0];
        section.value = charge.value.to_string();
        let digit_count = section.value.len() as f32;
        let diameter = charge.level * BULLET_RADIUS_FACTOR * 2.0;
        let full_size_horizontal = diameter * BULLET_TEXT_FONT_SIZE_ASPECT * digit_count;
        if diameter < full_size_horizontal {
            section.style.font_size = diameter / digit_count / BULLET_TEXT_FONT_SIZE_ASPECT;
        } else {
            section.style.font_size = diameter;
        }
    }
}
fn update_charge_ball(
    mut turrets: Query<(&mut ColliderScale, &Charge), Or<(Changed<Charge>, Added<Charge>)>>,
    mut transform_query: Query<&mut Transform>,
) {
    for (mut collider_scale, charge) in &mut turrets {
        let scale = charge.level * BULLET_RADIUS_FACTOR;
        *collider_scale = ColliderScale::Absolute(Vect::splat(scale));
        let mut ball_transform = transform_query.get_mut(charge.link).unwrap();
        ball_transform.scale.x = scale;
        ball_transform.scale.y = scale;
    }
}
fn handle_trigger_events(
    mut commands: Commands,
    mut reader: EventReader<TriggerEvent>,
    participants: Res<ParticipantMap<Entity>>,
    mesh: Res<BulletMesh>,
    materials: Res<ParticipantMap<Handle<ColorMaterial>>>,
    turret_stopwatch: Res<TurretStopwatch>,
    mut turret_query: Query<(&mut Charge, &Transform, &TurretPlatformLink), With<Turret>>,
    platform_query: Query<&BarrelOffset>,
    root: Query<Entity, With<BattlefieldRoot>>,
) {
    for event in reader.read() {
        let &entity = participants.get(event.participant);
        let Ok((mut charge, transform, &TurretPlatformLink(link))) = turret_query.get_mut(entity)
        else {
            continue;
        };
        match event.trigger_type {
            TriggerType::Multiply => charge.multiply(),
            TriggerType::BurstShot => {
                println!("burst shot is not implemented yet");
            }
            TriggerType::ChargedShot => {
                let &BarrelOffset(base_angle) = platform_query.get(link).unwrap();
                let ball = commands
                    .spawn(ChargeBallBundle::new(
                        mesh.0.clone(),
                        materials.get(event.participant).clone(),
                    ))
                    .id();
                commands
                    .spawn(BulletBundle::new(
                        event.participant,
                        transform.translation.x,
                        transform.translation.y,
                        ball,
                        &charge,
                        turret_stopwatch.get() + base_angle,
                    ))
                    .set_parent(root.single())
                    .add_child(ball);
                charge.reset();
            }
        }
    }
}
fn update_tile_color(
    colors: Res<ParticipantMap<Color>>,
    mut tiles: Query<(&Participant, &mut Sprite), (With<Tile>, Changed<Participant>)>,
) {
    for (&owner, mut sprite) in &mut tiles {
        sprite.color = *colors.get(owner);
    }
}
fn handle_bullet_turret_collision(
    mut commands: Commands,
    mut events: EventReader<CollisionEvent>,
    mut bullet_query: Query<(Entity, &Participant, &mut Charge, &mut Velocity), With<Bullet>>,
    mut turret_query: Query<(&Participant, &mut Charge), (With<Turret>, Without<Bullet>)>,
    participant_entity_query: Query<(Entity, &Participant), Without<Tile>>,
) {
    for event in events.read() {
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
                        for (e, &p) in &participant_entity_query {
                            if p == turret_owner {
                                commands.entity(e).despawn_recursive();
                            }
                        }
                    };
                    if turret_charge.level < ONE_SHOT_PROTECTION_THRESHOLD {
                        kill();
                    } else if bullet_charge.value > ONE_SHOT_DAMAGE_THRESHOLD {
                        bullet_charge.value -= ONE_SHOT_DAMAGE_THRESHOLD;
                        if bullet_charge.value <= 0.0 {
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
fn handle_bullet_tile_collision(
    mut events: EventReader<CollisionEvent>,
    mut bullet_query: Query<(&Participant, &mut Charge), With<Bullet>>,
    mut tile_query: Query<&mut Participant, (With<Tile>, Without<Bullet>)>,
) {
    for event in events.read() {
        match event {
            &CollisionEvent::Started(a, b, _) => {
                let (&bullet_owner, mut charge) = if let Ok(x) = bullet_query.get_mut(a) {
                    x
                } else if let Ok(x) = bullet_query.get_mut(b) {
                    x
                } else {
                    continue;
                };
                let mut tile_owner = if let Ok(x) = tile_query.get_mut(a) {
                    x
                } else if let Ok(x) = tile_query.get_mut(b) {
                    x
                } else {
                    continue;
                };
                if bullet_owner == *tile_owner {
                    continue;
                }
                if charge.value <= 0.0 {
                    continue;
                }
                *tile_owner = bullet_owner;
                charge.value -= 1.0;
            }
            CollisionEvent::Stopped(_, _, _) => (),
        }
    }
}
#[derive(Resource, Deref, DerefMut)]
#[allow(dead_code)]
struct AutoTimer(Timer);
impl Default for AutoTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(1.0, TimerMode::Repeating))
    }
}
#[allow(dead_code)]
fn auto_fire(mut writer: EventWriter<TriggerEvent>, mut timer: ResMut<AutoTimer>, time: Res<Time>) {
    timer.tick(time.delta());
    if timer.just_finished() {
        writer.send(TriggerEvent {
            participant: Participant::A,
            trigger_type: TriggerType::ChargedShot,
        });
    }
}
#[allow(dead_code)]
fn auto_multiply(
    mut writer: EventWriter<TriggerEvent>,
    mut timer: ResMut<AutoTimer>,
    time: Res<Time>,
) {
    timer.tick(time.delta());
    if timer.just_finished() {
        writer.send(TriggerEvent {
            participant: Participant::A,
            trigger_type: TriggerType::Multiply,
        });
    }
}
