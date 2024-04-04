# Project Title
Multiply or Release — Charged Shot vs Burst Shot

Team members:

- Raven Du
- Robert Alemany

## Summary Description

Rust implementation of a Multiply or Release game using the Bevy crate. Example of a Multiply or Release: https://www.youtube.com/watch?v=dL-AK1RSsO4

The goal of this project is to implement a Multiply or Release game using the Bevy engine. 
"Multiply or Release, sometimes abbreviated as MoR, is an Algodoo battle genre on YouTube, originally invented by MIKAN. Since the first video in July 15th in 2021, till now it is one of the most popular genre on YouTube, and there are many variants, integration with many other genre, and extension to many other platform like Unity, Scratch, etc."
The game involves 4 participants, which each have their own color. Most implementations include a panel where balls will fall, bouncing of pegs and falling into zones which trigger "abilities", these abilities include Multiply (which multiplies the current charge of the turret), Release (which releases the currently charged bullet from the turret onto the battlefield). as well as other abilities. The second component is the battlefield, where turrets for each team release charged bullets onto a battlefield of colored tiles. As the bullet hits tiles of other colors, it decreases it's charge by 1 point. Most implementations have it so when a bullet passes over a tile of it's same color, the tile darkens, still being considered of the same color team. When it collides with another bullet, both bullets bounce in proportion to their current charge (as if it were it's mass). The size of the bullet is proportional to the charge. When the bullet hits an opposing participants turret, the turret is destroyed, and no more bullets of that participant will be created. The game can continue on without end, however some implementations include the game ending when all but one participants turrets are destroyed.

## Checkpoint Progress Summary

Currently, we have implemented most of the core functionality of the game. The panel where the balls fall is mostly complete other than minor alterations to asthetics, as well as any possible additions to new abilities. The battlefield is mostly complete, as turrets spawn bullets with a charge, and release them depending on triggered abilities within the panel. Bullets bounce off each other in proportion to their charge, however we are considering changing how our current implementation handles the "bounce", as it doesn't match perfectly what we are hoping for. The bullets collide with tiles of opposing colors, changing them to match the bullet owners color. Turrets change direction to aim in different directions as time progresses. When a bullet hits the turret, the turret is destroyed.

Current planned features still requiring implementation:
- Tiles darkening when a bullet matching its team color hits it
- Text displaying when a turret is destroyed (indicating that a team has been eliminated from the game)
- End text when all but one of the turrets have been destroyed
- A trace like effect as the bullet moves throughout the battlefield 
- More abilities that affect the battlefield

## Additional Details

- The crates needed for this project are:
  - bevy = { version = "0.13.0", features = ["dynamic_linking"] } (The game engine. It handles rendering, managing game world and data storage, running update loop, etc.)
  - bevy_rapier2d = "0.25.0" (A physics engine.)
  - rand = "0.8.5 (Random number generation.)
  - bevy-inspector-egui = "0.23.3" (A plugin for bevy that lists all the entities and components in the game world for debugging.)
- The core struture of the code is the application which spawns entities which have attributes (traits which define hoe the application treats them). The entities are interacted with through events, like CollisionEvents, which determine changes in behavior at specific moments. Events can be handles in a predefined order. Entities can spawn other entities, which inherit their attributes (useful for components which are dependant on eachothers attributes, for example the position of the trigger boxes within the panel is inherited from the panels position).
