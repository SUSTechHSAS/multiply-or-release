# Multiply or Release — Charged Shot vs Burst Shot

Team members:

- Raven Du
- Robert Alemany

## Summary Description

https://www.youtube.com/watch?v=dL-AK1RSsO4

Multiply or Release, sometimes abbreviated as MoR, is an Algodoo battle genre on Youtube, originally invented by MIKAN. Since the first video in July 15th in 2021, till now it is one of the most popular genre on Youtube, and there are many variants, integration with many other genre, and extension to many other platform like Unity, Scratch, etc.

## Additional Details

### Use Case

The user runs the program, screams "All hail the RNGesus!" and watches.

### Code Design

All app logic in Bevy uses the Entity Component System paradigm, which is often shortened to ECS. 
ECS is a software pattern that involves breaking your program up into Entities, Components, and 
Systems. Entities are unique "things" that are assigned groups of Components, which are then 
processed using Systems.

See [Bevy Quick Start Guide](https://bevyengine.org/learn/quick-start/getting-started/ecs/) and 
[Tainted Coder's Tutorial on ECS](https://taintedcoders.com/bevy/ecs/).

#### Key Data structures

```rust
#[derive(Component)]
/// A game participant. It's not called player since the game is not interactive.
enum Participant {
    A,
    B,
    C,
    D,
}

#[derive(Bundle)]
/// Component bundle for the round obstacles in the side panels and the walls.
/// (I don't know if meshes and colliders have to be continous. Maybe we can just make a single
/// entity for the entire obstacle course.)
struct ObstacleBundle<M: Material2d> {
    /// Bevy rendering component used to display the ball.
    mesh: MaterialMesh2dBundle<M>,
    /// Rapier collider component.
    collider: Collider,
    /// Rapier rigidbody component. We'll set this to static since we don't want these to move, but
    /// we'd other balls to bounce off it.
    rigidbody: RigidBody,
}

#[derive(Component)]
/// Marker to mark this entity as a trigger zone.
struct TriggerZone;
#[derive(Bundle)]
/// Component bundle for the trigger zones at the bottom of the side panels.
struct TriggerZoneBundle<M: Material2d> {
    /// Marker to mark this entity as a trigger zone.
    marker: TriggerZone,
    /// Bevy rendering component used to display the trigger zone.
    mesh: MaterialMesh2dBundle<M>,
    /// Rapier collider component. We'll mark this as a sensor since we want the balls to be able
    /// to pass through it.
    collider: Collider,
}

#[derive(Component)]
/// Marker to mark this entity as a worker ball.
struct WorkerBall;
#[derive(Bundle)]
/// Component bundle for the little worker balls in the side panels.
struct WorkerBallBundle<M: Material2d> {
    /// Marker to mark this entity as a worker ball.
    marker: WorkerBall,
    /// Bevy rendering component used to display the ball.
    mesh: MaterialMesh2dBundle<M>,
    /// Rapier collider component.
    collider: Collider,
    /// Rapier rigidbody component, used by the physics engine to move the entity.
    rigidbody: RigidBody,
    /// The game participant that owns this ball.
    owner: Participant,
}

#[derive(Component)]
/// Marker to mark this entity as a tile.
struct Tile;
#[derive(Bundle)]
/// Component bundle for each of the individual tiles on the battle field.
struct TileBundle<M: Material2d> {
    /// Marker to mark this entity as a tile.
    marker: Tile,
    /// Bevy rendering component used to display the tile.
    mesh: MaterialMesh2dBundle<M>,
    /// Rapier collider component. We'll mark this as sensor and won't add a rigidbody to this
    /// entity because we don't actually want the physics engine to move itl.
    collider: Collider,
    /// The game participant that owns this tile.
    owner: Participant,
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

#[derive(Component)]
/// Marker to indicate the entity is a turret head.
struct TurretHead;
#[derive(Bundle)]
/// Component bundle for the turret head (the little ball that sits on the top of the turret to
/// show its charge level and never moves).
struct TurretHeadBundle<M: Material2d> {
    /// Marker to indicate that this is a turret head.
    th: TurretHead,
    /// Bevy rendering component used to display the ball.
    mesh: MaterialMesh2dBundle<M>,
    /// A sensor collider to detect when this turret is hit by a bullet.
    collider: Collider,
    /// The game participant that owns this ball.
    owner: Participant,
    /// Some text component for bevy to render the text onto the ball
    /// (We're not sure exact how this would be done at the moment).
    _text: (),
}

#[derive(Component)]
/// Component for a turret.
struct Turret {
    /// The angle offset in degrees of the direction that the turret barrel is pointing.
    barrel_offset: f32,
    /// The direction that the barrel would be pointing in with an offset_angle of 0.
    base_direction: Vec2,
}
#[derive(Bundle)]
/// Component bundle for a turret.
struct TurretBundle<M: Material2d> {
    /// Bevy rendering component used to display the ball.
    mesh: MaterialMesh2dBundle<M>,
    /// The game participant that owns this ball.
    owner: Participant,
    /// Variables for the functionality of the turret.
    turret: Turret,
}
```

#### Modules

We'll separate the whole application into 3 modules:
- worker_panel (the panels on the side that simulates a normal distribution ball drop and triggers 
turret events)
- battlefield (the center battlefield where the turrets shoot bullets that interact with each 
other as well as the ground)
- main (puts everything together and instantiate the bevy instance)

#### Testing

We have no idea at this moment to be honest. We'll look into it.

## Roadmap

### Minimal Viable Product

The functionality of the simulation as shown in 
[the video](https://www.youtube.com/watch?v=dL-AK1RSsO4) minus the visual effects:

- The trail the balls leave as they move.
- The particle effects of the tiles changing color.
- The crosshair that pops up when the turrets shoot.


### Checkpoint

- Finish the worker_panel module.
- Define stretch goals to pursue after the minimal viable product is done.

