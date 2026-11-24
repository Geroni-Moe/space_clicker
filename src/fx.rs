//! Visual effects: sprite animation, one-shot bursts, particles, floating
//! damage numbers and credit chips that fly to the HUD counter.

use bevy::prelude::*;
use bevy::text::FontSmoothing;
use rand::Rng;

use crate::art::Art;
use crate::audio::Sfx;
use crate::economy::fmt;
use crate::{AppState, Field, HUD_H, MainCamera};

pub struct FxPlugin;

impl Plugin for FxPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (lifetimes, motion, fade, float_numbers, fly_coins).run_if(in_state(AppState::Playing)),
        )
        // the title screen's ships flicker their engines too
        .add_systems(Update, animate.run_if(not(in_state(AppState::Loading))))
        .add_systems(OnExit(AppState::Playing), clear_effects);
    }
}

/// Plays through a sprite sheet's frames.
#[derive(Component)]
pub struct Anim {
    pub frames: usize,
    pub fps: f32,
    pub looping: bool,
    /// Despawn the entity after the last frame of a non-looping animation.
    pub despawn: bool,
    t: f32,
}

impl Anim {
    pub fn looping(frames: usize, fps: f32) -> Self {
        Self { frames, fps, looping: true, despawn: false, t: rand::rng().random_range(0.0..1.0) }
    }

    pub fn once(frames: usize, fps: f32) -> Self {
        Self { frames, fps, looping: false, despawn: true, t: 0.0 }
    }
}

#[derive(Component)]
pub struct Lifetime(pub f32);

#[derive(Component)]
pub struct Velocity {
    pub v: Vec2,
    pub drag: f32,
    pub spin: f32,
}

/// Fades sprite/text alpha to zero over its lifetime.
#[derive(Component)]
pub struct Fade {
    pub total: f32,
    pub left: f32,
    pub alpha: f32,
}

impl Fade {
    pub fn new(secs: f32) -> Self {
        Self { total: secs, left: secs, alpha: 1.0 }
    }
}

#[derive(Component)]
struct FloatText {
    t: f32,
    rise: f32,
}

#[derive(Component)]
struct Coin {
    t: f32,
    delay: f32,
    from: Vec2,
    ctrl: Vec2,
}

fn animate(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Anim, &mut Sprite)>) {
    for (e, mut anim, mut sprite) in &mut q {
        anim.t += time.delta_secs() * anim.fps;
        let mut frame = anim.t as usize;
        if frame >= anim.frames {
            if anim.looping {
                anim.t %= anim.frames as f32;
                frame = anim.t as usize;
            } else if anim.despawn {
                commands.entity(e).despawn();
                continue;
            } else {
                frame = anim.frames - 1;
            }
        }
        if let Some(atlas) = &mut sprite.texture_atlas {
            atlas.index = frame.min(anim.frames - 1);
        }
    }
}

/// Anything that only exists as an in-battle effect.
type Effect = Or<(With<Anim>, With<Lifetime>, With<FloatText>, With<Coin>, With<Fade>, With<Velocity>)>;

/// Leaving a run: explosions, numbers, debris and credit chips go with it.
fn clear_effects(mut commands: Commands, q: Query<Entity, Effect>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

fn lifetimes(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Lifetime)>) {
    for (e, mut life) in &mut q {
        life.0 -= time.delta_secs();
        if life.0 <= 0.0 {
            commands.entity(e).despawn();
        }
    }
}

fn motion(time: Res<Time>, mut q: Query<(&mut Velocity, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut vel, mut tf) in &mut q {
        tf.translation += (vel.v * dt).extend(0.0);
        let drag = (1.0 - vel.drag * dt).max(0.0);
        vel.v *= drag;
        tf.rotate_z(vel.spin * dt);
    }
}

fn fade(time: Res<Time>, mut q: Query<(&mut Fade, Option<&mut Sprite>, Option<&mut TextColor>)>) {
    for (mut f, sprite, text) in &mut q {
        f.left = (f.left - time.delta_secs()).max(0.0);
        let a = f.alpha * (f.left / f.total).clamp(0.0, 1.0);
        if let Some(mut s) = sprite {
            s.color.set_alpha(a);
        }
        if let Some(mut c) = text {
            c.0.set_alpha(a);
        }
    }
}

/// One-shot animated effect centred on `pos`.
pub fn burst(commands: &mut Commands, art: &Art, name: &str, pos: Vec2, z: f32, fps: f32) -> Entity {
    commands
        .spawn((art.sprite(name), Transform::from_translation(pos.extend(z)), Anim::once(art.frames(name), fps)))
        .id()
}

pub fn explosion(commands: &mut Commands, art: &Art, pos: Vec2, size: f32, sfx: &mut MessageWriter<Sfx>) {
    let mut rng = rand::rng();
    let big = size >= 30.0;
    let name = if big { "explosion_big" } else { "explosion_small" };
    let pieces = if big { 4 } else { 1 + (size / 12.0) as usize };
    for i in 0..pieces {
        let jitter = if i == 0 { Vec2::ZERO } else { Vec2::new(rng.random_range(-0.5..0.5), rng.random_range(-0.4..0.4)) * size };
        let e = burst(commands, art, name, pos + jitter, 30.0 + i as f32 * 0.01, rng.random_range(13.0..17.0));
        if i > 0 {
            commands.entity(e).insert(Transform::from_translation((pos + jitter).extend(30.0)).with_scale(Vec3::splat(0.6)));
        }
    }
    // tumbling hull fragments
    for _ in 0..(3 + (size / 6.0) as usize).min(12) {
        let dir = Vec2::from_angle(rng.random_range(0.0..std::f32::consts::TAU));
        let mut sprite = art.sprite("debris");
        if let Some(a) = &mut sprite.texture_atlas {
            a.index = rng.random_range(0..3);
        }
        commands.spawn((
            sprite,
            Transform::from_translation((pos + dir * 3.0).extend(29.0)),
            Velocity { v: dir * rng.random_range(30.0..90.0), drag: 1.5, spin: rng.random_range(-8.0..8.0) },
            Lifetime(1.2),
            Fade::new(1.2),
        ));
    }
    sfx.write(Sfx(if big { "explode_big" } else { "explode" }));
}

/// Floating damage number.
pub fn damage_number(commands: &mut Commands, art: &Art, pos: Vec2, amount: f64, style: NumberStyle) {
    let mut rng = rand::rng();
    let (color, size, text) = match style {
        NumberStyle::Click => (Color::srgb_u8(181, 246, 255), 8.0, fmt(amount)),
        NumberStyle::Crit => (Color::srgb_u8(255, 214, 92), 16.0, format!("{}!", fmt(amount))),
        NumberStyle::Fleet => (Color::srgb_u8(150, 160, 190), 8.0, fmt(amount)),
        NumberStyle::Drone => (Color::srgb_u8(120, 220, 255), 8.0, fmt(amount)),
        NumberStyle::Credits => (Color::srgb_u8(255, 236, 140), 8.0, format!("+{}", fmt(amount.floor()))),
        NumberStyle::Repair => (Color::srgb_u8(120, 255, 190), 8.0, format!("+{}", fmt(amount))),
    };
    let jitter = Vec2::new(rng.random_range(-8.0..8.0), rng.random_range(-4.0..6.0));
    commands.spawn((
        Text2d::new(text),
        TextFont { font: art.font_bold.clone().into(), font_size: FontSize::Px(size), font_smoothing: FontSmoothing::None, ..default() },
        TextColor(color),
        Transform::from_translation((pos + jitter).extend(40.0)),
        FloatText { t: 0.0, rise: if matches!(style, NumberStyle::Crit) { 34.0 } else { 22.0 } },
        Lifetime(0.9),
        Fade::new(0.9),
    ));
}

#[derive(Clone, Copy)]
pub enum NumberStyle {
    Click,
    Crit,
    Fleet,
    Drone,
    Credits,
    Repair,
}

fn float_numbers(time: Res<Time>, mut q: Query<(&mut FloatText, &mut Transform)>) {
    for (mut f, mut tf) in &mut q {
        let dt = time.delta_secs();
        f.t += dt;
        // fast pop up, then a slow drift
        let speed = f.rise * (1.0 - f.t).max(0.1) * 2.0;
        tf.translation.y += speed * dt;
    }
}

/// Credit chips that burst out of a wreck and stream to the HUD counter.
pub fn coins(commands: &mut Commands, art: &Art, pos: Vec2, count: usize) {
    let mut rng = rand::rng();
    for i in 0..count {
        let spread = Vec2::new(rng.random_range(-30.0..30.0), rng.random_range(10.0..50.0));
        commands.spawn((
            art.sprite("credit"),
            Transform::from_translation(pos.extend(35.0)),
            Anim::looping(6, 14.0),
            Coin { t: 0.0, delay: i as f32 * 0.04, from: pos, ctrl: pos + spread },
        ));
    }
}

fn fly_coins(
    mut commands: Commands,
    time: Res<Time>,
    window: Single<&Window>,
    field: Res<Field>,
    cam: Single<&Transform, (With<MainCamera>, Without<Coin>)>,
    mut q: Query<(Entity, &mut Coin, &mut Transform)>,
    mut sfx: MessageWriter<Sfx>,
) {
    // HUD credit counter sits near the top-left corner of the window
    let target = Vec2::new(
        cam.translation.x + (34.0 * field.ui - window.width() / 2.0) / field.zoom,
        cam.translation.y + (window.height() / 2.0 - HUD_H * 0.35 * field.ui) / field.zoom,
    );
    for (e, mut coin, mut tf) in &mut q {
        coin.t += time.delta_secs();
        let t = ((coin.t - coin.delay) / 0.85).clamp(0.0, 1.0);
        if t >= 1.0 {
            commands.entity(e).despawn();
            sfx.write(Sfx("coin"));
            continue;
        }
        // quadratic bezier with ease-in so chips hang for a beat, then whoosh
        let k = t * t;
        let p = coin.from.lerp(coin.ctrl, k).lerp(coin.ctrl.lerp(target, k), k);
        tf.translation = p.extend(35.0);
    }
}
