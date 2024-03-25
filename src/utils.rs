use bevy::prelude::*;

// Constants {{{

const PARTICIPANT_COLORS: ParticipantMap<Color> =
    ParticipantMap::new(Color::RED, Color::GREEN, Color::PURPLE, Color::ORANGE);

// }}}

pub struct UtilsPlugin;
impl Plugin for UtilsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PreStartup, setup);
    }
}

#[derive(Resource)]
pub struct ParticipantInfo {
    pub colors: ParticipantMap<Handle<ColorMaterial>>,
}

/// A struct that maps a value to each participant.
#[derive(Debug, Clone, Copy, Default)]
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

#[derive(Component, Clone, Copy, Default)]
/// A game participant. It's not called player since the game is not interactive.
pub enum Participant {
    #[default]
    A,
    B,
    C,
    D,
}

fn setup(mut commands: Commands, mut materials: ResMut<Assets<ColorMaterial>>) {
    commands.insert_resource(ParticipantInfo {
        colors: PARTICIPANT_COLORS.map(|color| materials.add(color)),
    });
}
