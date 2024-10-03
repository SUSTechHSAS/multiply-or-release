use bevy::{color::palettes::css, prelude::*};
use bevy_hanabi::prelude::*;

// Constants {{{

const PARTICIPANT_COLORS: ParticipantMap<Srgba> =
    ParticipantMap::new(css::MAROON, css::DARK_GREEN, css::PURPLE, css::GOLD);
const BALL_COLORS: ParticipantMap<Srgba> =
    ParticipantMap::new(css::RED, css::GREEN, css::VIOLET, css::YELLOW);
const PARTICIPANT_NAME: ParticipantMap<&'static str> =
    ParticipantMap::new("RED", "GREEN", "VIOLET", "YELLOW");

const PARTICLE_LIFETIME: f32 = 2.;

// }}}

pub struct UtilsPlugin;
impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            PreStartup,
            (
                setup_participant_maps,
                setup_tile_hit_effect.after(setup_participant_maps),
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
#[allow(dead_code)]
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
    pub fn get_mut(&mut self, participant: Participant) -> &mut T {
        match participant {
            Participant::A => &mut self.a,
            Participant::B => &mut self.b,
            Participant::C => &mut self.c,
            Participant::D => &mut self.d,
        }
    }
    pub fn map<U>(self, mut f: impl FnMut(T) -> U) -> ParticipantMap<U> {
        ParticipantMap::new(f(self.a), f(self.b), f(self.c), f(self.d))
    }
    // }}}
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
#[derive(Clone, Resource)]
pub struct TileHitEffect(pub Handle<EffectAsset>);
#[derive(Clone, Component, Deref, DerefMut)]
pub struct EffectLifetimeTimer(Timer);
impl Default for EffectLifetimeTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(
            PARTICLE_LIFETIME + 0.2,
            TimerMode::Once,
        ))
    }
}

fn setup_participant_maps(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(PARTICIPANT_NAME);
    commands.insert_resource(PARTICIPANT_COLORS.map(Color::Srgba).map(TileColor));
    commands.insert_resource(BALL_COLORS.map(Color::Srgba).map(BallColor));
    commands.insert_resource(
        BALL_COLORS.map(|srgba| materials.add(ColorMaterial::from(Color::from(srgba)))),
    );
}
fn setup_tile_hit_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // Set `spawn_immediately` to false to spawn on command with Spawner::reset()
    let spawner = Spawner::once(16.0.into(), true);

    let writer = ExprWriter::new();

    // Init the age of particles to 0, and their lifetime to 1.5 second.
    let age = writer.lit(0.);
    let init_age = SetAttributeModifier::new(Attribute::AGE, age.expr());
    // Initialize the total lifetime of the particle, that is
    // the time for which it's simulated and rendered. This modifier
    // is almost always required, otherwise the particles won't show.
    let lifetime = writer.lit(PARTICLE_LIFETIME);
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
    let spawn_color = writer.add_property("spawn_color", 0xFFFFFFFFu32.into());
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

    let bullet_vel_prop = writer.add_property("bullet_vel", Vec2::ZERO.into());
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
        EffectAsset::new(vec![16384], spawner, writer.finish())
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

pub trait EffectPropertiesExt: Default {
    fn set_spawn_color(&mut self, color: impl Into<LinearRgba>);
    fn set_bullet_vel(&mut self, bullet_vel: Vec2);
}
impl EffectPropertiesExt for EffectProperties {
    fn set_spawn_color(&mut self, color: impl Into<LinearRgba>) {
        self.set("spawn_color", color.into().as_u32().into());
    }
    fn set_bullet_vel(&mut self, bullet_vel: Vec2) {
        self.set("bullet_vel", bullet_vel.into());
    }
}
