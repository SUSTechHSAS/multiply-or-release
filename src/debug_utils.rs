use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use rand::{distributions::Uniform, prelude::*};

use crate::{
    battlefield::{EliminationEvent, BATTLEFIELD_HALF_WIDTH},
    panel_plugin::{TriggerEvent, TriggerType},
    utils::{BallColor, Participant, ParticipantMap, TileHitEffect},
};

pub struct DebugUtilsPlugin;
impl Plugin for DebugUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(bevy_inspector_egui::quick::WorldInspectorPlugin::new());
        // app.add_plugins(bevy_rapier2d::render::RapierDebugRenderPlugin::default())
        // .insert_resource(AutoTimer::default())
        // .add_systems(Update, (auto_hanabi, auto_fire));
    }
}

#[derive(Resource, Deref, DerefMut)]
#[allow(dead_code)]
struct AutoTimer(Timer);
impl Default for AutoTimer {
    fn default() -> Self {
        Self(Timer::from_seconds(2.0, TimerMode::Repeating))
    }
}
#[allow(dead_code)]
fn auto_hanabi(
    mut commands: Commands,
    mut timer: ResMut<AutoTimer>,
    time: Res<Time>,
    effect: Res<TileHitEffect>,
    colors: Res<ParticipantMap<BallColor>>,
) {
    timer.tick(time.delta());
    if timer.just_finished() {
        let dist = Uniform::new_inclusive(-BATTLEFIELD_HALF_WIDTH, BATTLEFIELD_HALF_WIDTH);
        let mut rng = thread_rng();
        let x = rng.sample(dist);
        let y = rng.sample(dist);
        let p = match rng.sample(Uniform::new(0, 4)) {
            0 => Participant::A,
            1 => Participant::B,
            2 => Participant::C,
            3 => Participant::D,
            _ => unreachable!(),
        };
        let color = Srgba::from(colors.get(p).0);
        let color = 0xFF000000u32
            | ((color.blue * 255.0) as u32) << 16
            | ((color.green * 255.0) as u32) << 8
            | ((color.red * 255.0) as u32);
        let mut effect_properties = EffectProperties::default();
        effect_properties.set("spawn_color", color.into());
        commands.spawn(ParticleEffectBundle {
            effect: ParticleEffect::new(effect.0.clone()),
            transform: Transform::from_xyz(x, y, 5.0),
            effect_properties,
            ..default()
        });
    }
}
#[allow(dead_code)]
fn auto_elimination(
    mut writer: EventWriter<EliminationEvent>,
    mut timer: ResMut<AutoTimer>,
    time: Res<Time>,
) {
    timer.tick(time.delta());
    if timer.just_finished() {
        writer.send(EliminationEvent {
            participant: Participant::A,
        });
        writer.send(EliminationEvent {
            participant: Participant::B,
        });
        writer.send(EliminationEvent {
            participant: Participant::C,
        });
    }
}
#[allow(dead_code)]
fn auto_fire(mut writer: EventWriter<TriggerEvent>, mut timer: ResMut<AutoTimer>, time: Res<Time>) {
    timer.tick(time.delta());
    if timer.just_finished() {
        let shot_type = if thread_rng().gen_bool(0.5) {
            TriggerType::ChargedShot
        } else {
            TriggerType::BurstShot
        };
        writer.send(TriggerEvent {
            participant: Participant::A,
            trigger_type: shot_type,
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
