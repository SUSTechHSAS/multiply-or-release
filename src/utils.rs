use bevy::{color::palettes::css, prelude::*};

// Constants {{{

const PARTICIPANT_COLORS: ParticipantMap<Srgba> =
    ParticipantMap::new(css::MAROON, css::DARK_GREEN, css::PURPLE, css::GOLD);
const BALL_COLORS: ParticipantMap<Srgba> =
    ParticipantMap::new(css::RED, css::GREEN, css::VIOLET, css::YELLOW);
const PARTICIPANT_NAME: ParticipantMap<&'static str> =
    ParticipantMap::new("RED", "GREEN", "VIOLET", "YELLOW");

// }}}

pub struct UtilsPlugin;
impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup);
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

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(PARTICIPANT_NAME);
    commands.insert_resource(PARTICIPANT_COLORS.map(Color::Srgba).map(TileColor));
    commands.insert_resource(BALL_COLORS.map(Color::Srgba).map(BallColor));
    commands.insert_resource(
        BALL_COLORS.map(|srgba| materials.add(ColorMaterial::from(Color::from(srgba)))),
    );
}
