use std::ops::{Index, IndexMut};

use bevy::{color::palettes::css, prelude::*};
use bevy_hanabi::prelude::*;

use crate::panel_plugin::{WORKER_BALL_COUNT_MAX, WORKER_BALL_RADIUS};

// Constants {{{

const PARTICIPANT_COLORS: ParticipantMap<Srgba> = ParticipantMap::new(
    css::MAROON,
    css::DARK_GREEN,
    css::PURPLE,
    css::DARK_GOLDENROD,
);
const BALL_COLORS: ParticipantMap<Srgba> =
    ParticipantMap::new(css::RED, css::LIMEGREEN, css::VIOLET, css::YELLOW);

const HIT_PARTICLE_LIFETIME: f32 = 2.;
const HIT_PARTICLE_SIZE: f32 = WORKER_BALL_RADIUS * 2.0;
const HIT_PARTICLE_COUNT: f32 = 16.0;
const HIT_PARTICLE_MAX_PER_SECOND: f32 = 1024.0;
const TRAIL_SPAWN_RATE: f32 = 60.;
const TRAIL_LIFETIME: f32 = 0.5;
pub const SPAWN_COLOR_PROPERTY: &str = "spawn_color";
const POSITION_PROPERTY: &str = "position";
const BULLET_VEL_PROPERTY: &str = "bullet_vel";

// }}}

pub struct UtilsPlugin;
impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            (
                setup_participant_maps,
                (setup_tile_hit_effect, setup_trail_effect).after(setup_participant_maps),
            ),
        );
    }
}

#[derive(Debug, Clone, Copy, Default, Resource)]
pub struct TileColor(pub Color);
#[derive(Debug, Clone, Copy, Default, Resource)]
pub struct BallColor(pub Color);

/// A struct that maps a value to each participant.
#[derive(Debug, Clone, Copy, Default, Resource)]
pub struct ParticipantMap<T> {
    // {{{
    pub a: T,
    pub b: T,
    pub c: T,
    pub d: T,
}
impl<T> ParticipantMap<T> {
    pub const fn new(a: T, b: T, c: T, d: T) -> Self {
        Self { a, b, c, d }
    }
    pub const fn get(&self, participant: Participant) -> &T {
        match participant {
            Participant::A => &self.a,
            Participant::B => &self.b,
            Participant::C => &self.c,
            Participant::D => &self.d,
        }
    }
    pub fn set(&mut self, participant: Participant, new_value: T) {
        match participant {
            Participant::A => self.a = new_value,
            Participant::B => self.b = new_value,
            Participant::C => self.c = new_value,
            Participant::D => self.d = new_value,
        }
    }
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> ParticipantMap<U> {
        ParticipantMap::new(f(self.a), f(self.b), f(self.c), f(self.d))
    }
    // }}}
}
impl<T> Index<Participant> for ParticipantMap<T> {
    type Output = T;
    fn index(&self, index: Participant) -> &Self::Output {
        match index {
            Participant::A => &self.a,
            Participant::B => &self.b,
            Participant::C => &self.c,
            Participant::D => &self.d,
        }
    }
}
impl<T> IndexMut<Participant> for ParticipantMap<T> {
    fn index_mut(&mut self, index: Participant) -> &mut Self::Output {
        match index {
            Participant::A => &mut self.a,
            Participant::B => &mut self.b,
            Participant::C => &mut self.c,
            Participant::D => &mut self.d,
        }
    }
}
impl<T: Copy> ParticipantMap<T> {
    pub const fn splat(x: T) -> Self {
        Self {
            a: x,
            b: x,
            c: x,
            d: x,
        }
    }
}

#[derive(Debug, Component, Clone, Copy, Default, PartialEq, Eq)]
/// A game participant. It's not called player since the game is not interactive.
pub enum Participant {
    #[default]
    A,
    B,
    C,
    D,
}
impl Participant {
    pub const ALL: [Self; 4] = [Self::A, Self::B, Self::C, Self::D];
}
impl std::fmt::Display for Participant {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let name = match self {
            Participant::A => "RED",
            Participant::B => "GREEN",
            Participant::C => "VIOLET",
            Participant::D => "YELLOW",
        };
        f.write_str(name)
    }
}
#[derive(Clone, Resource)]
pub struct TileHitEffect(pub Handle<EffectAsset>);
#[derive(Clone, Resource)]
pub struct TrailEffect(pub Handle<EffectAsset>);
#[derive(Clone, Component, Deref, DerefMut)]
pub struct EffectLifetimeTimer(Timer);
impl Default for EffectLifetimeTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            HIT_PARTICLE_LIFETIME + 0.2,
            TimerMode::Once,
        ))
    }
}

fn setup_participant_maps(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(ParticipantMap::splat(true));
    commands.insert_resource(PARTICIPANT_COLORS.map(Color::Srgba).map(TileColor));
    commands.insert_resource(BALL_COLORS.map(Color::Srgba).map(BallColor));
    commands.insert_resource(
        BALL_COLORS.map(|srgba| materials.add(ColorMaterial::from(Color::from(srgba)))),
    );
}
fn setup_tile_hit_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // Set `spawn_immediately` to false to spawn on command with Spawner::reset()
    let spawner = Spawner::once(HIT_PARTICLE_COUNT.into(), true);

    let writer = ExprWriter::new();

    // Init the age of particles to 0, and their lifetime to 1.5 second.
    let age = writer.lit(0.);
    let init_age = SetAttributeModifier::new(Attribute::AGE, age.expr());
    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles won't show.
    let lifetime = writer.lit(HIT_PARTICLE_LIFETIME);
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime.expr());

    // Add a bit of linear drag to slow down particles after the inital spawning.
    // This keeps the particle around the spawn point, making it easier to visualize
    // the different groups of particles.
    let drag = writer.lit(3.);
    let update_drag = LinearDragModifier::new(drag.expr());

    // Bind the initial particle color to the value of the 'spawn_color' property
    // when the particle spawns. The particle will keep that color afterward,
    // even if the property changes, because the color will be saved
    // per-particle (due to the Attribute::COLOR).
    let spawn_color = writer.add_property(SPAWN_COLOR_PROPERTY, 0xFFFFFFFFu32.into());
    let init_color = SetAttributeModifier::new(Attribute::COLOR, writer.prop(spawn_color).expr());

    let gradient = Gradient::linear(Vec2::ONE, Vec2::ZERO);

    // On spawn, randomly initialize the position of the particle
    // to be over the surface of a sphere of radius 2 units.
    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Z).expr(),
        radius: writer.lit(2.).expr(),
        dimension: ShapeDimension::Volume,
    };

    let bullet_vel_prop = writer.add_property(BULLET_VEL_PROPERTY, Vec2::ZERO.into());
    let bullet_vel = writer.prop(bullet_vel_prop);
    let bullet_vel3 = bullet_vel.clone().x().vec3(bullet_vel.y(), writer.lit(0.));
    let vel = writer
        .attr(Attribute::POSITION)
        .normalized()
        .mul(writer.lit(7.5).uniform(writer.lit(10.)))
        .add(
            bullet_vel3
                .normalized()
                .mul(writer.lit(10.).uniform(writer.lit(15.))),
        );
    let init_vel = SetAttributeModifier::new(Attribute::VELOCITY, vel.expr());

    let effect = effects.add(
        EffectAsset::new(
            vec![(HIT_PARTICLE_COUNT * HIT_PARTICLE_MAX_PER_SECOND * HIT_PARTICLE_LIFETIME) as u32],
            spawner,
            writer.finish(),
        )
        .with_name("tile hit")
        .init(init_pos)
        .init(init_vel)
        .init(init_age)
        .init(init_lifetime)
        .init(init_color)
        .update(update_drag)
        .render(SizeOverLifetimeModifier {
            gradient,
            screen_space_size: false,
        }),
    );

    commands.insert_resource(TileHitEffect(effect));
}
fn setup_trail_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let writer = ExprWriter::default();

    let pos = writer.add_property(POSITION_PROPERTY, Vec3::ZERO.into());
    let spawn_color = writer.add_property(SPAWN_COLOR_PROPERTY, 0xFFFFFFFFu32.into());

    let init_position_attr = SetAttributeModifier {
        attribute: Attribute::POSITION,
        value: writer.prop(pos).expr(),
    };

    let init_velocity_attr = SetAttributeModifier {
        attribute: Attribute::VELOCITY,
        value: writer.lit(Vec3::ZERO).expr(),
    };

    let init_age_attr = SetAttributeModifier {
        attribute: Attribute::AGE,
        value: writer.lit(0.0).expr(),
    };

    let init_lifetime_attr = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(999999.0).expr(),
    };

    let init_size_attr = SetAttributeModifier {
        attribute: Attribute::SIZE,
        value: writer.lit(HIT_PARTICLE_SIZE).expr(),
    };

    let init_color = SetAttributeModifier {
        attribute: Attribute::COLOR,
        value: writer.lit(LinearRgba::NONE.as_u32()).expr(),
    };

    let clone1_modifier = CloneModifier::new(1.0 / TRAIL_SPAWN_RATE, 1);
    let clone2_modifier = CloneModifier::new(1.0 / TRAIL_SPAWN_RATE, 2);

    let move_modifier = SetAttributeModifier {
        attribute: Attribute::POSITION,
        value: writer.prop(pos).expr(),
    };

    let update_lifetime_attr = SetAttributeModifier {
        attribute: Attribute::LIFETIME,
        value: writer.lit(TRAIL_LIFETIME).expr(),
    };

    let age_ratio = writer
        .attr(Attribute::AGE)
        .div(writer.attr(Attribute::LIFETIME));
    let size = writer
        .lit(HIT_PARTICLE_SIZE)
        .mix(writer.lit(0.0), age_ratio.clone());
    let update_size_attr = SetAttributeModifier {
        attribute: Attribute::SIZE,
        value: size.expr(),
    };
    // let mut gradient = Gradient::new();
    // gradient.add_key(0.0, Vec2::splat(PARTICLE_SIZE));
    // gradient.add_key(1.0, Vec2::ZERO);

    let alpha_offset = age_ratio
        .smoothstep(
            writer.lit(0.0),
            writer.prop(spawn_color).unpack4x8unorm().w(),
        )
        .mul(writer.lit(Vec4::new(0.0, 0.0, 0.0, 1.0)))
        .pack4x8unorm();
    let color = writer.prop(spawn_color).sub(alpha_offset);
    let update_color_attr = SetAttributeModifier {
        attribute: Attribute::COLOR,
        value: color.expr(),
    };
    let round = RoundModifier {
        roundness: writer.lit(1.0).expr(),
    };

    let group0 = ParticleGroupSet::single(0);
    let group12 = ParticleGroupSet::single(1).with_group(2);
    const TOTAL_BALL_COUNT: u32 = WORKER_BALL_COUNT_MAX as u32 * 4;
    const PARTICLE_COUNT: u32 =
        (TOTAL_BALL_COUNT as f32 * TRAIL_SPAWN_RATE * TRAIL_LIFETIME + 1.0) as u32;
    let effect = EffectAsset::new(
        vec![TOTAL_BALL_COUNT, PARTICLE_COUNT, PARTICLE_COUNT],
        Spawner::once(1.0.into(), true),
        writer.finish(),
    )
    .with_name("trail")
    .with_simulation_space(SimulationSpace::Global)
    .init(init_position_attr)
    .init(init_velocity_attr)
    .init(init_age_attr)
    .init(init_lifetime_attr)
    .init(init_size_attr)
    .init(init_color)
    .update_groups(move_modifier, group0)
    .update_groups(clone1_modifier, group0)
    .update_groups(clone2_modifier, group0)
    .update_groups(update_lifetime_attr, group12)
    .update_groups(update_color_attr, group12)
    .update_groups(update_size_attr, group12)
    // .render(SizeOverLifetimeModifier {
    //     gradient,
    //     screen_space_size: false,
    // })
    .render_groups(round, group0.with_group(1))
    .render_groups(RibbonModifier, ParticleGroupSet::single(2));

    commands.insert_resource(TrailEffect(effects.add(effect)));
}

pub trait EffectPropertiesExt: Default {
    fn set_spawn_color(&mut self, color: impl Into<LinearRgba>);
    fn set_bullet_vel(&mut self, bullet_vel: Vec2);
    fn set_position(&mut self, position: Vec3);
    fn from_spawn_color(color: impl Into<LinearRgba>) -> Self {
        let mut properties = Self::default();
        properties.set_spawn_color(color);
        properties
    }
    fn with_position(mut self, x: f32, y: f32) -> Self {
        self.set_position(Vec3::new(x, y, 0.0));
        self
    }
}
impl EffectPropertiesExt for EffectProperties {
    fn set_spawn_color(&mut self, color: impl Into<LinearRgba>) {
        self.set("spawn_color", color.into().as_u32().into());
    }
    fn set_bullet_vel(&mut self, bullet_vel: Vec2) {
        self.set("bullet_vel", bullet_vel.into());
    }
    fn set_position(&mut self, position: Vec3) {
        self.set("position", position.into());
    }
}
