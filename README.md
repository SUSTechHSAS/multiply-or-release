# Multiply or Release

Multiply or Release is a Marble Race-like non-interactive simulation game.

## Showcase

https://github.com/user-attachments/assets/b6568097-09f6-4d65-9571-ae0c628ff452

### Rules

- Marbles race down an obstacle course to land in one of a few trigger zones. Each marble has a turret associated in the main battlefield. When the marble lands in a trigger zone, its associated turret performs the corresponding action.
- Each turret holds a charge. Depending on the zone its associated marbles land in, it can:
  - Multiply its current charge by 2 or 4.
  - Release its charge in a single powerful shot or a stream of smaller shots.
- The battlefield is made up of a grid of tiles. Each tile is associated with a turret. When a shot hits a tile for an opposing side, it consumes a charge to convert the tile.
- When a shot hits a turret, the shot and the turret each consumes an equal amount of charge. If the turret's charge goes to 0 in this exchange, it dies.

## How to Run

This game has no releases yet, but you can clone this repo and build it locally.

1. Install Rust by following the [Rust Getting Started Guide](https://www.rust-lang.org/learn/get-startedA).
2. Clone this repo `git clone --depth=1 https://github.com/maybe-raven/multiply-or-release`.
3. Navigate to the directory then build and run with Cargo `cargo run --release`

> [!Warning]
> I only have a MacBook so it's only tested on MacOS. I have no idea how well it'll fare on other operating systems.

## License

Licensed under either of

- MIT License ([`LICENSE-MIT`](./LICENSE-MIT) or <http://opensource.org/licenses/MIT>)
- Apache License, Version 2.0 ([`LICENSE-APACHE2`](./LICENSE-APACHE2) or <http://www.apache.org/licenses/LICENSE-2.0>)
