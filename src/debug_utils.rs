use bevy::prelude::*;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_rapier2d::prelude::*;
use rand::prelude::*;

use crate::{
    battlefield::EliminationEvent,
    panel_plugin::{TriggerEvent, TriggerType},
    utils::Participant,
};

pub struct DebugUtilsPlugin;
impl Plugin for DebugUtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierDebugRenderPlugin::default())
            .add_plugins(WorldInspectorPlugin::new())
            .insert_resource(AutoTimer::default())
            .add_systems(Update, auto_fire);
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
