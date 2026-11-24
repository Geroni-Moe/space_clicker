//! Developer helpers, all opt-in through environment variables:
//!
//! - `VOID_ARMADA_DEBUG=1`: F1 +credits, F2 +5 sectors, F3 +shards, F4 skip to
//!   the boss wave (F2/F4 clear the field), F12 screenshot.
//! - `VOID_ARMADA_SHOTS=<dir>`: where screenshots go; with `VOID_ARMADA_SHOT_EVERY=<secs>`
//!   one is also taken periodically.
//! - `VOID_ARMADA_QUIT_AFTER=<secs>`: exit (saving) after that long.
//! - `VOID_ARMADA_PERF=1`: log frame times (mean, worst) and the entity count every
//!   2s, with vsync off so the numbers show the real frame cost.
//! - `VOID_ARMADA_GOD=1`: the player's ships take no damage (for perf runs).
//! - `VOID_ARMADA_SCRIPT=<file>`: replay input for end-to-end tests. Lines:
//!   `wait <s>`, `click <x> <y> [count] [gap_s]` (window points), `key <name>`,
//!   `shot <name>`, `quit`. Input enters as window messages (cursor moves,
//!   button presses), so picking, buttons and hotkeys run exactly as for a player.

use bevy::input::ButtonState;
use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::input::mouse::MouseButtonInput;
use bevy::prelude::*;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};
use bevy::window::{CursorMoved, PrimaryWindow, WindowEvent};
use std::collections::VecDeque;

use crate::battle::{Battle, Enemy};
use crate::{AppState, Game};

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        if std::env::var("VOID_ARMADA_DEBUG").is_ok() {
            app.add_systems(Update, cheats.run_if(in_state(AppState::Playing)));
        }
        if let Ok(dir) = std::env::var("VOID_ARMADA_SHOTS")
            && let Some(every) = std::env::var("VOID_ARMADA_SHOT_EVERY").ok().and_then(|s| s.parse().ok())
        {
            app.insert_resource(Shots { dir, timer: Timer::from_seconds(every, TimerMode::Repeating), n: 0 })
                .add_systems(Update, auto_shots.run_if(not(in_state(AppState::Loading))));
        }
        if std::env::var("VOID_ARMADA_GOD").is_ok() {
            app.insert_resource(GodMode);
        }
        if std::env::var("VOID_ARMADA_PERF").is_ok() {
            app.init_resource::<Perf>().add_systems(Last, perf_log);
        }
        if let Some(secs) = std::env::var("VOID_ARMADA_QUIT_AFTER").ok().and_then(|s| s.parse::<f32>().ok()) {
            app.insert_resource(QuitAfter(secs)).add_systems(Update, quit_after);
        }
        if let Ok(path) = std::env::var("VOID_ARMADA_SCRIPT") {
            let text = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("script {path}: {e}"));
            let shots = std::env::var("VOID_ARMADA_SHOTS").unwrap_or_else(|_| ".".into());
            app.insert_resource(Script::parse(&text, shots)).add_systems(First, run_script.run_if(not(in_state(AppState::Loading))));
        }
    }
}

#[derive(Resource)]
struct Shots {
    dir: String,
    timer: Timer,
    n: u32,
}

fn cheats(
    keys: Res<ButtonInput<KeyCode>>,
    mut game: ResMut<Game>,
    mut battle: ResMut<Battle>,
    enemies: Query<Entity, With<Enemy>>,
    mut commands: Commands,
) {
    // jumping sector or wave clears the field so the next spawn matches it
    if keys.any_just_pressed([KeyCode::F2, KeyCode::F4]) {
        for e in &enemies {
            commands.entity(e).despawn();
        }
        *battle = Battle::default();
    }
    let s = &mut game.state;
    if keys.just_pressed(KeyCode::F1) {
        let amount = (s.credits * 10.0).max(1e4);
        s.earn(amount);
    }
    if keys.just_pressed(KeyCode::F2) {
        s.sector += 5;
        s.wave = 1;
        s.max_sector = s.max_sector.max(s.sector);
        s.best_sector = s.best_sector.max(s.sector);
    }
    if keys.just_pressed(KeyCode::F3) {
        s.shards += 100.0;
    }
    if keys.just_pressed(KeyCode::F4) {
        s.wave = crate::economy::WAVES_PER_SECTOR;
    }
    if keys.just_pressed(KeyCode::F12) {
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk("void_armada_screenshot.png"));
    }
}

fn auto_shots(time: Res<Time<Real>>, mut shots: ResMut<Shots>, mut commands: Commands) {
    if shots.timer.tick(time.delta()).just_finished() {
        shots.n += 1;
        let path = format!("{}/shot_{:03}.png", shots.dir, shots.n);
        commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
    }
}

/// The player's ships ignore hits.
#[derive(Resource)]
pub struct GodMode;

#[derive(Resource, Default)]
struct Perf {
    frames: Vec<f32>,
    since: f32,
}

fn perf_log(time: Res<Time<Real>>, mut perf: ResMut<Perf>, entities: Query<()>) {
    let dt = time.delta_secs();
    perf.frames.push(dt);
    perf.since += dt;
    if perf.since < 2.0 {
        return;
    }
    let n = perf.frames.len() as f32;
    let mean = perf.frames.iter().sum::<f32>() / n;
    let worst = perf.frames.iter().copied().fold(0.0, f32::max);
    info!("perf: {:.0} fps, mean {:.1} ms, worst {:.1} ms, {} entities", 1.0 / mean, mean * 1e3, worst * 1e3, entities.iter().count());
    perf.frames.clear();
    perf.since = 0.0;
}

#[derive(Resource)]
struct QuitAfter(f32);

fn quit_after(time: Res<Time<Real>>, quit: Res<QuitAfter>, mut exit: MessageWriter<AppExit>) {
    if time.elapsed_secs() > quit.0 {
        exit.write(AppExit::Success);
    }
}

enum Step {
    Wait(f32),
    Click { at: Vec2, left: u32, gap: f32 },
    Key(KeyCode),
    Shot(String),
    Quit,
}

#[derive(Resource)]
struct Script {
    steps: VecDeque<Step>,
    timer: f32,
    /// Sub-frame phase of the current click: 0 move, 1 press, 2 release.
    phase: u8,
    shots: String,
}

fn key_code(name: &str) -> KeyCode {
    match name {
        "1" => KeyCode::Digit1,
        "2" => KeyCode::Digit2,
        "3" => KeyCode::Digit3,
        "space" => KeyCode::Space,
        "f1" => KeyCode::F1,
        "f2" => KeyCode::F2,
        "f3" => KeyCode::F3,
        "f4" => KeyCode::F4,
        "r" => KeyCode::KeyR,
        "esc" => KeyCode::Escape,
        other => panic!("unknown key {other}"),
    }
}

impl Script {
    fn parse(text: &str, shots: String) -> Self {
        let mut steps = VecDeque::new();
        for line in text.lines().map(str::trim).filter(|l| !l.is_empty() && !l.starts_with('#')) {
            let w: Vec<&str> = line.split_whitespace().collect();
            let num = |i: usize, d: f32| w.get(i).map(|v| v.parse::<f32>().expect("number")).unwrap_or(d);
            steps.push_back(match w[0] {
                "wait" => Step::Wait(num(1, 1.0)),
                "click" => Step::Click { at: Vec2::new(num(1, 0.0), num(2, 0.0)), left: num(3, 1.0) as u32, gap: num(4, 0.1) },
                "key" => Step::Key(key_code(w[1])),
                "shot" => Step::Shot(w[1].to_string()),
                "quit" => Step::Quit,
                other => panic!("unknown script step {other}"),
            });
        }
        Self { steps, timer: 0.0, phase: 0, shots }
    }
}

fn run_script(
    time: Res<Time<Real>>,
    mut script: ResMut<Script>,
    window: Single<Entity, With<PrimaryWindow>>,
    mut moved: MessageWriter<CursorMoved>,
    mut buttons: MessageWriter<MouseButtonInput>,
    mut keys: MessageWriter<KeyboardInput>,
    // winit sends every input twice: as its own message and inside WindowEvent (read by picking)
    mut window_events: MessageWriter<WindowEvent>,
    mut exit: MessageWriter<AppExit>,
    mut commands: Commands,
) {
    let window = *window;
    let script = &mut *script;
    script.timer -= time.delta_secs();
    if script.timer > 0.0 {
        return;
    }
    let shots = script.shots.clone();
    let phase = script.phase;
    let Some(step) = script.steps.front_mut() else { return };
    let mut advance = true;
    match step {
        Step::Wait(s) => script.timer = *s,
        Step::Click { at, left, gap } => {
            let (at, gap) = (*at, *gap);
            match phase {
                0 => {
                    let m = CursorMoved { window, position: at, delta: None };
                    window_events.write(WindowEvent::CursorMoved(m.clone()));
                    moved.write(m);
                    script.phase = 1;
                    advance = false;
                }
                1 => {
                    let b = MouseButtonInput { button: MouseButton::Left, state: ButtonState::Pressed, window };
                    window_events.write(WindowEvent::MouseButtonInput(b));
                    buttons.write(b);
                    script.phase = 2;
                    advance = false;
                }
                _ => {
                    let b = MouseButtonInput { button: MouseButton::Left, state: ButtonState::Released, window };
                    window_events.write(WindowEvent::MouseButtonInput(b));
                    buttons.write(b);
                    script.phase = 0;
                    *left -= 1;
                    advance = *left == 0;
                    script.timer = gap;
                }
            }
        }
        Step::Key(code) => {
            let code = *code;
            for state in [ButtonState::Pressed, ButtonState::Released] {
                let k = KeyboardInput { key_code: code, logical_key: Key::Unidentified(bevy::input::keyboard::NativeKey::Unidentified), state, text: None, repeat: false, window };
                window_events.write(WindowEvent::KeyboardInput(k.clone()));
                keys.write(k);
            }
        }
        Step::Shot(name) => {
            let path = format!("{shots}/{name}.png");
            commands.spawn(Screenshot::primary_window()).observe(save_to_disk(path));
        }
        Step::Quit => {
            exit.write(AppExit::Success);
        }
    }
    if advance {
        script.steps.pop_front();
    }
}
