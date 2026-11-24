//! Sound effects. Systems send `Sfx` messages; one system plays them,
//! throttling repeats so rapid fire doesn't turn into noise.

use bevy::audio::Volume;
use bevy::prelude::*;
use std::collections::HashMap;

use crate::Game;

pub struct AudioFxPlugin;

impl Plugin for AudioFxPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Sfx>().add_systems(Startup, load).add_systems(Last, play);
    }
}

#[derive(Message, Clone, Copy)]
pub struct Sfx(pub &'static str);

const SOUNDS: [(&str, f32, f64); 21] = [
    // name, volume, min seconds between plays
    ("laser", 0.5, 0.03),
    ("blaster", 0.18, 0.07),
    ("cannon", 0.25, 0.08),
    ("missile", 0.2, 0.1),
    ("beam", 0.18, 0.1),
    ("hit", 0.25, 0.05),
    ("crit", 0.55, 0.05),
    ("explode", 0.5, 0.06),
    ("explode_big", 0.8, 0.2),
    ("coin", 0.3, 0.05),
    ("buy", 0.5, 0.04),
    ("deny", 0.4, 0.1),
    ("tab", 0.4, 0.03),
    ("boss", 0.6, 1.0),
    ("warp", 0.6, 0.5),
    ("ability", 0.6, 0.1),
    ("sector", 0.6, 0.5),
    ("levelup", 0.5, 0.1),
    ("alarm", 0.45, 1.0),
    ("ship_lost", 0.55, 0.15),
    ("death", 0.9, 2.0),
];

#[derive(Resource)]
struct Bank {
    clips: HashMap<&'static str, (Handle<AudioSource>, f32, f64)>,
    last: HashMap<&'static str, f64>,
}

fn load(mut commands: Commands, server: Res<AssetServer>) {
    let clips = SOUNDS
        .iter()
        .map(|(name, vol, gap)| (*name, (server.load(format!("sfx/{name}.wav")), *vol, *gap)))
        .collect();
    commands.insert_resource(Bank { clips, last: HashMap::new() });
}

fn play(mut commands: Commands, mut reader: MessageReader<Sfx>, mut bank: ResMut<Bank>, time: Res<Time<Real>>, game: Res<Game>) {
    let now = time.elapsed_secs_f64();
    for Sfx(name) in reader.read() {
        if game.state.muted {
            continue;
        }
        let Some((clip, vol, gap)) = bank.clips.get(name).cloned() else {
            warn!("unknown sfx {name}");
            continue;
        };
        if bank.last.get(name).is_some_and(|t| now - t < gap) {
            continue;
        }
        bank.last.insert(name, now);
        commands.spawn((AudioPlayer::new(clip), PlaybackSettings::DESPAWN.with_volume(Volume::Linear(vol))));
    }
}
