# Project Title
Multiply or Release — Charged Shot vs Burst Shot

Team members:

- Raven Du
- Robert Alemany

## Summary Description

Reiterate the summary description of the overall goal of the project (updated as
necessary from the Proposal document).
Rust implementation of a Multiply or Release game using the Bevy crate. Example of a Multiply or Release: https://www.youtube.com/watch?v=dL-AK1RSsO4

The goal of this project is to implement a Multiply or Release game using the Bevy engine. 
"Multiply or Release, sometimes abbreviated as MoR, is an Algodoo battle genre on Youtube, originally invented by MIKAN. Since the first video in July 15th in 2021, till now it is one of the most popular genre on Youtube, and there are many variants, integration with many other genre, and extension to many other platform like Unity, Scratch, etc."
The game involves 4 participants, which each have their own color. Most implementations include a panel where balls will fall, bouncing of pegs and falling into zones which trigger "abilities", these abilities include Multiply (which multiplies the current charge of the turret), Release (which releases the currently charged bullet from the turret onto the battlefield). as well as other abilities. The second component is the battlefield, where turrets for each team release charged bullets onto a battlefield of colored tiles. As the bullet hits tiles of other colors, it decreases it's charge by 1 point. Most implementations have it so when a bullet passes over a tile of it's same color, the tile darkens, still being considered of the same color team. When it collides with another bullet, both bullets bounce in proportion to their current charge (as if it were it's mass). The size of the bullet is proportional to the charge. When the bullet hits an opposing participants turret, the turret is destroyed, and no more bullets of that participant will be created. The game can continue on without end, however some implementations include the game ending when all but one participants turrets are destroyed.

## Checkpoint Progress Summary

Currently, we have implemented most of the core functionality of the game. The panel where the balls fall is mostly complete other than minor alterations to asthetics, as well as any possible additions to new abilities. The battlefield is mostly complete, as turrets spawn bullets with a charge, and release them depending on triggered abilities within the panel. Bullets bounce off eachother in proportion to their charge, however we are considering changing how our current implementation handles the "bounce", as it doesn't match perfectly what we are hoping for. The bullets collide with tiles of opposing colors, changing them to match the bullet owners color. Turrets change direction to aim in different directions as time progresses. When a bullet hits the turret, the turret is destroyed.

Current planned features still requiring implementation:
- Tiles darkening when a bullet matching its team color hits it
- Text displaying when a turret is destroyed (indicating that a team has been eliminated from the game)
- End text when all but one of the turrets have been destroyed
- A trace like effect as the bullet moves throughout the battlefield 
- More abilities that affect the battlefield

## Additional Details

- List any external Rust crates required for the project (i.e., what
  `[dependencies]` have been added to `Cargo.toml` files).
- Briefly describe the structure of the code (what are the main components, the
  module dependency structure).
- Pose any questions that you may have about your project and/or request
  feedback on specific aspects of the project.

- The crates needed for this project are:
-  bevy = { version = "0.13.0", features = ["dynamic_linking"] }
-  bevy-inspector-egui = "0.23.3"
-  bevy_rapier2d = "0.25.0"
-  rand = "0.8.5


***
***

The following should be deleted from the checkpoint document, but is included in the initial `Checkpoint.md` file for reference.

## Final Project Rubric

- Correctness (15%)
  - Free of compilation errors and warnings.
  - Correctly accomplishes the goals of the project.
  - Correctness is supported by suitable testing.
- Style/Design (15%)
  - Project applies one or more elements of Rust design:
    - Uses traits.  Minimally, makes use of traits from the Rust Standard Library (e.g., `PartialEq` and `Iterator`).  Better, implements appropriate traits for the types defined for the project.  Best, defines and   uses novel traits for the project.
    - Uses `struct` and `enum` definitions appropriately.
    - Uses types to capture invariants.
    - Uses modules appropriately (e.g., place distinct data structures in distinct modules).
- Effort/Accomplishment (30%)
  - How “big” is the project?
    - A “small” project will have at least 500 Lines of Rust Code per team member.  (Significantly less than that corresponds to a “tiny” project and would not be acceptable for this activity.)  A “medium” or “large” project may have significantly more and would likely also correspond to a more “difficult” project (see below).
  - How “difficult” was the project?
    - An “easy” project that required trivial data structures and algorithms. Feedback about how to extend the project was ignored.  (Projects falling into this category are likely to lose points in Style/Design as well.)
    - A “moderate” project that applied basic course concepts, but did not require the group members to significantly challenge themselves or to learn something new.
    - A “challenging” project that demonstrates significant thought in design and implementation.  Clear that the group members challenged themselves and learned something new by undertaking the project.
  - What work was done for the project?  (This includes both the work embodied by the final submission and work not obvious from the final submission (e.g., approaches attempted but then abandoned, suitably described).) 
  - Did the project require learning advanced features?
  - Did all team members contribute to the project?
- Presentation (10\%)
