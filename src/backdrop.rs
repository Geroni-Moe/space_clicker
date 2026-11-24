//! Scrolling parallax backdrop whose nebula and planet follow the sector.

use bevy::prelude::*;
use bevy::sprite::SpriteImageMode;

use crate::art::Art;
use crate::economy::{FACTION_KEYS, faction};
use crate::{AppState, Game, MainCamera};

pub struct BackdropPlugin;

impl Plugin for BackdropPlugin {
    fn build(&self, app: &mut App) {
        // spawned once and kept: it sits behind the title screen and every run
        app.add_systems(OnExit(AppState::Loading), spawn)
            .add_systems(Update, scroll.run_if(not(in_state(AppState::Loading))))
            .add_systems(Update, follow_sector.run_if(in_state(AppState::Playing)))
            .add_systems(Update, menu_planet.run_if(in_state(AppState::Menu)));
    }
}

#[derive(Component)]
struct Layer {
    /// world units per second
    speed: f32,
    /// tile width in world units
    tile: f32,
    scale: f32,
}

#[derive(Component)]
struct Nebula(usize);

#[derive(Component)]
struct Planet {
    x: f32,
    shown_for: u32,
}

const PLANETS: [&str; 4] = ["planet_ringed", "planet_ice", "planet_lava", "planet_jungle"];

fn tiled(art: &Art, name: &str, w: f32, h: f32, tile_y: bool) -> Sprite {
    Sprite {
        image: art.image(name),
        custom_size: Some(Vec2::new(w, h)),
        image_mode: SpriteImageMode::Tiled { tile_x: true, tile_y, stretch_value: 1.0 },
        ..default()
    }
}

fn spawn(mut commands: Commands, art: Res<Art>, game: Res<Game>) {
    // generous size so resizes never reveal an edge
    let (w, h) = (2600.0, 1600.0);
    commands.spawn((tiled(&art, "stars_far", w, h, true), Transform::from_xyz(0.0, 0.0, -100.0), Layer { speed: 3.0, tile: 256.0, scale: 1.0 }));
    let f = faction(game.state.sector);
    commands.spawn((
        tiled(&art, &format!("nebula_{}", FACTION_KEYS[f]), w, 256.0, false),
        Transform::from_xyz(0.0, 0.0, -90.0),
        Layer { speed: 6.0, tile: 384.0, scale: 1.0 },
        Nebula(f),
    ));
    commands.spawn((tiled(&art, "stars_mid", w, h, true), Transform::from_xyz(0.0, 0.0, -80.0), Layer { speed: 12.0, tile: 256.0, scale: 1.0 }));
    commands.spawn((
        art.sprite(PLANETS[0]),
        Transform::from_xyz(80.0, 90.0, -70.0),
        Planet { x: 80.0, shown_for: 0 },
    ));
    commands.spawn((tiled(&art, "stars_near", w, h, true), Transform::from_xyz(0.0, 0.0, -60.0), Layer { speed: 28.0, tile: 256.0, scale: 1.0 }));
}

fn scroll(time: Res<Time>, cam: Single<&Transform, (With<MainCamera>, Without<Layer>)>, mut layers: Query<(&Layer, &mut Transform)>) {
    let t = time.elapsed_secs();
    for (layer, mut tf) in &mut layers {
        let period = layer.tile * layer.scale;
        let offset = (t * layer.speed) % period;
        // whole pixels at the layer's own scale keep stars crisp
        tf.translation.x = ((cam.translation.x - offset) / layer.scale).round() * layer.scale;
        tf.translation.y = cam.translation.y.round();
    }
}

/// On the title screen the planet peeks in from a corner, away from the title.
fn menu_planet(cam: Single<&Transform, (With<MainCamera>, Without<Planet>)>, mut planet: Single<&mut Transform, With<Planet>>) {
    planet.translation.x = (cam.translation.x + 250.0).round();
    planet.translation.y = (cam.translation.y - 205.0).round();
}

fn follow_sector(
    time: Res<Time>,
    game: Res<Game>,
    art: Res<Art>,
    field: Res<crate::Field>,
    mut nebula: Single<(&mut Nebula, &mut Sprite), Without<Planet>>,
    mut planet: Single<(&mut Planet, &mut Sprite, &mut Transform), Without<Nebula>>,
) {
    let sector = game.state.sector;
    let f = faction(sector);
    if nebula.0.0 != f {
        nebula.0.0 = f;
        nebula.1.image = art.image(&format!("nebula_{}", FACTION_KEYS[f]));
    }
    let (p, sprite, tf) = &mut *planet;
    // a new planet every 5 sectors, drifting slowly past
    let group = (sector - 1) / 5;
    if p.shown_for != group + 1 {
        p.shown_for = group + 1;
        **sprite = art.sprite(PLANETS[group as usize % PLANETS.len()]);
        p.x = 40.0 + (group % 3) as f32 * 70.0;
    }
    // planets peek in from the top or bottom edge, clear of the battle line
    let edge = field.half.y + 18.0;
    tf.translation.y = if group.is_multiple_of(2) { edge } else { -edge };
    p.x -= time.delta_secs() * 1.2;
    if p.x < -420.0 {
        p.x = 420.0;
    }
    tf.translation.x = p.x.round();
}
