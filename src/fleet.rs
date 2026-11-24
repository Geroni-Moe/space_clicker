//! The player's flagship and escorts: formation, arrival, idle motion and
//! per-race weapon fire (visual only - damage is applied by `battle`).

use bevy::prelude::*;
use rand::Rng;

use crate::art::{Art, flagship_sheet, fleet_sheet};
use crate::audio::Sfx;
use crate::battle::{Enemy, Shot, ShotKind, spawn_shot};
use crate::economy::{HULL_KINDS, LOGI_BASE, Race, ship_count};
use crate::fx::{Anim, burst};
use crate::{AppState, Field, Game, in_battle};

pub struct FleetPlugin;

impl Plugin for FleetPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (sync_fleet, move_ships).chain().run_if(in_state(AppState::Playing)))
            .add_systems(OnExit(AppState::Playing), |mut commands: Commands, q: Query<Entity, With<FleetShip>>| {
                for e in &q {
                    commands.entity(e).despawn();
                }
            })
            .add_systems(Update, fire.run_if(in_state(AppState::Playing)).run_if(in_battle));
    }
}

#[derive(Component)]
pub struct FleetShip {
    /// None for the flagship, else an index into HULL_KINDS (escorts, then logi).
    pub class: Option<usize>,
    pub idx: u32,
    pub home: Vec2,
    pub pos: Vec2,
    phase: f32,
    cooldown: f32,
    /// Half the hull length: muzzle offset.
    pub nose: f32,
    pub half: Vec2,
    /// Flagship hull tier / race currently shown.
    shown: (u32, Race),
    pub hp: f64,
    pub max_hp: f64,
    /// Destroyed: the hull is gone (its levels with it) and despawns this frame.
    pub lost: bool,
    /// Seconds of red hit flash left.
    pub flash: f32,
    /// Seconds until a logi ship's next repair pulse is drawn.
    pub pulse: f32,
}

impl FleetShip {
    fn new(class: Option<usize>, idx: u32, home: Vec2, pos: Vec2, size: Vec2, shown: (u32, Race)) -> Self {
        let mut rng = rand::rng();
        Self {
            class,
            idx,
            home,
            pos,
            phase: rng.random_range(0.0..std::f32::consts::TAU),
            cooldown: rng.random_range(0.5..2.0),
            nose: size.x / 2.0,
            half: size / 2.0,
            shown,
            hp: 0.0,
            max_hp: 0.0,
            lost: false,
            flash: 0.0,
            pulse: rng.random_range(0.0..0.5),
        }
    }

    /// Still in the fight (not destroyed).
    pub fn active(&self) -> bool {
        !self.lost
    }

    pub fn is_logi(&self) -> bool {
        self.class.is_some_and(|c| c >= LOGI_BASE)
    }
}

/// The flagship sits mid-field with its nose a fixed distance from the
/// battle line, so bigger hulls extend further back.
pub fn flagship_home(size: Vec2) -> Vec2 {
    Vec2::new(-12.0 - size.x / 2.0, 4.0)
}

/// Lane x and the hull kinds sharing it; rear lanes hold the big hulls, placed
/// so even a titan's stern stays on screen. Logi fly behind the guns they cover.
const LANES: [(f32, &[usize]); 4] = [(8.0, &[0, 1]), (-44.0, &[2, 3, 9]), (-98.0, &[4, 5, 10]), (-150.0, &[6, 7, 8, 11])];

/// Home position of every escort, keyed by (class, index). `flag` is the
/// flagship's home and sprite size; lanes it overlaps split around it.
pub fn formation(levels: &[u32; HULL_KINDS], field: &Field, flag: (Vec2, Vec2)) -> Vec<(usize, u32, Vec2)> {
    let top = field.half.y - 24.0;
    let bottom = field.bottom() + 16.0;
    let (flag_home, flag_size) = flag;
    // sprite frames carry ~18px of flame margin behind the hull
    let flag_x = (flag_home.x - flag_size.x / 2.0 + 18.0, flag_home.x + flag_size.x / 2.0);
    let band = flag_size.y / 2.0 + 4.0;
    let mut out = vec![];
    for (lane, (x, classes)) in LANES.iter().enumerate() {
        // interleave classes so a lane mixes hull types evenly
        let mut ships: Vec<(usize, u32)> = vec![];
        let max = classes.iter().map(|c| ship_count(*c, levels[*c])).max().unwrap_or(0);
        for i in 0..max {
            for c in classes.iter() {
                if i < ship_count(*c, levels[*c]) {
                    ships.push((*c, i));
                }
            }
        }
        let n = ships.len();
        if n == 0 {
            continue;
        }
        let stagger = |k: usize| if k.is_multiple_of(2) { 0.0 } else { -10.0 };
        let split = *x > flag_x.0 - 16.0 && *x < flag_x.1 + 16.0;
        if split {
            // fill outward from the flagship, alternating above and below it
            let above = n.div_ceil(2);
            let below = n - above;
            let room_up = top - (flag_home.y + band);
            let room_down = (flag_home.y - band) - bottom;
            for (k, (c, i)) in ships.into_iter().enumerate() {
                let (rank, count, room, sign) = if k % 2 == 0 { (k / 2, above, room_up, 1.0) } else { (k / 2, below, room_down, -1.0) };
                let step = (room / count.max(1) as f32).clamp(14.0, 48.0);
                let y = flag_home.y + sign * (band + step * (rank as f32 + 0.5));
                out.push((c, i, Vec2::new(*x + stagger(k), y.clamp(bottom, top))));
            }
        } else {
            let step = ((top - bottom) / n as f32).clamp(14.0, 64.0);
            // odd lanes sit half a step off so hulls tuck between the lane behind
            let phase = if lane % 2 == 1 { step / 2.0 } else { 0.0 };
            for (k, (c, i)) in ships.into_iter().enumerate() {
                // centre-out: 0 in the middle, then alternate above/below
                let rank = k.div_ceil(2);
                let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
                let y = flag_home.y + phase + sign * rank as f32 * step;
                out.push((c, i, Vec2::new(*x + stagger(k), y.clamp(bottom, top))));
            }
        }
    }
    out
}

fn sync_fleet(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Game>,
    field: Res<Field>,
    mut ships: Query<(Entity, &mut FleetShip, &mut Sprite, &mut Anim)>,
    mut sfx: MessageWriter<Sfx>,
) {
    if !game.state.race_chosen || game.warping || game.dead {
        return;
    }
    let race = game.state.race;
    let tier = game.state.hull;

    // flagship
    let flag_size = art.size(&flagship_sheet(race, tier));
    let flag_home = flagship_home(flag_size);
    let mut have_flag = false;
    for (_, mut ship, mut sprite, _) in &mut ships {
        if ship.class.is_none() {
            have_flag = true;
            if ship.shown != (tier, race) {
                let name = flagship_sheet(race, tier);
                *sprite = art.sprite(&name);
                let size = art.size(&name);
                ship.nose = size.x / 2.0;
                ship.half = size / 2.0;
                ship.home = flagship_home(size);
                if ship.shown.1 == race && tier > ship.shown.0 {
                    burst(&mut commands, &art, "warp_in", ship.pos, 12.0, 18.0);
                    sfx.write(Sfx("levelup"));
                }
                ship.shown = (tier, race);
            }
        }
    }
    if !have_flag {
        let name = flagship_sheet(race, tier);
        let size = art.size(&name);
        let start = flag_home - Vec2::new(320.0, 0.0);
        commands.spawn((
            art.sprite(&name),
            Anim::looping(2, 8.0),
            Transform::from_translation(start.extend(5.0)),
            FleetShip::new(None, 0, flag_home, start, size, (tier, race)),
        ));
    }

    // escorts
    let wanted = formation(&game.state.hull_levels(), &field, (flag_home, flag_size));
    let mut existing: Vec<(usize, u32)> = vec![];
    for (e, mut ship, _, _) in &mut ships {
        let Some(class) = ship.class else { continue };
        if ship.lost {
            continue;
        }
        match wanted.iter().find(|(c, i, _)| *c == class && *i == ship.idx) {
            Some((_, _, home)) => {
                ship.home = *home;
                existing.push((class, ship.idx));
            }
            None => commands.entity(e).despawn(),
        }
    }
    let mut rng = rand::rng();
    for (c, i, home) in wanted {
        if existing.contains(&(c, i)) {
            continue;
        }
        let name = fleet_sheet(race, c);
        let size = art.size(&name);
        let start = home - Vec2::new(300.0 + rng.random_range(0.0..80.0), 0.0);
        // bigger hulls render behind smaller ones
        let z = 4.0 - size.x * 0.02 - i as f32 * 0.01;
        commands
            .spawn((
                art.sprite(&name),
                Anim::looping(2, 8.0 + rng.random_range(-1.0..1.0)),
                Transform::from_translation(start.extend(z)),
                FleetShip::new(Some(c), i, home, start, size, (0, race)),
            ))
            .with_children(|p| crate::survival::spawn_hp_bar(p, size));
    }
}

fn move_ships(time: Res<Time>, mut q: Query<(&mut FleetShip, &mut Transform)>) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();
    for (mut ship, mut tf) in &mut q {
        // ease towards home; arriving ships glide in from the left
        let k = 1.0 - (-dt * 2.2).exp();
        let home = ship.home;
        ship.pos = ship.pos.lerp(home, k);
        let bob = Vec2::new((t * 0.6 + ship.phase).sin() * 2.0, (t * 1.3 + ship.phase).sin() * 1.5);
        let p = ship.pos + bob;
        tf.translation.x = p.x.round();
        tf.translation.y = p.y.round();
    }
}

/// What a ship class shoots, per race.
fn weapon(race: Race, class: Option<usize>) -> (ShotKind, u32, f32, &'static str) {
    // (kind, shots per volley, seconds between volleys, sfx)
    let size = class.map(|c| c as f32).unwrap_or(4.0);
    match race {
        Race::Throne => (ShotKind::Beam { image: "beam_gold", width: 0.35 + size * 0.12 }, 1, 1.6 + size * 0.15, "beam"),
        Race::Tidewall => match class {
            Some(c) if c < 2 => (ShotKind::Bolt("bolt_cyan"), 2, 0.9, "blaster"),
            _ => (ShotKind::Missile, 2 + (size / 2.0) as u32, 2.0 + size * 0.1, "missile"),
        },
        Race::Concord => match class {
            Some(c) if c >= 3 => (ShotKind::Drone, 1 + (c / 3) as u32, 2.4, "blaster"),
            _ => (ShotKind::Bolt("bolt_green"), 3, 1.0, "blaster"),
        },
        Race::Rustborn => {
            if size >= 4.0 {
                (ShotKind::Shell, 3 + (size / 3.0) as u32, 1.5, "cannon")
            } else {
                (ShotKind::Bolt("bolt_gold"), 3, 0.8, "cannon")
            }
        }
    }
}

fn fire(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    game: Res<Game>,
    mut ships: Query<&mut FleetShip>,
    enemies: Query<(Entity, &Transform, &Enemy)>,
    mut sfx: MessageWriter<Sfx>,
) {
    let targets: Vec<(Entity, Vec2, Vec2)> = enemies
        .iter()
        .filter(|(_, _, e)| e.alive() && e.arrived())
        .map(|(id, tf, e)| (id, tf.translation.truncate(), e.half))
        .collect();
    if targets.is_empty() {
        return;
    }
    let mut rng = rand::rng();
    let race = game.state.race;
    for mut ship in &mut ships {
        // the flagship only fires once its autocannons are installed
        if ship.class.is_none() && game.state.cannons == 0 {
            continue;
        }
        if (ship.pos - ship.home).length() > 20.0 || !ship.active() || ship.is_logi() {
            continue;
        }
        ship.cooldown -= time.delta_secs() * if game.boosts().dps > 1.0 { 2.0 } else { 1.0 };
        if ship.cooldown > 0.0 {
            continue;
        }
        let (kind, count, gap, sound) = weapon(race, ship.class);
        ship.cooldown = gap * rng.random_range(0.85..1.2);
        let (target, tpos, thalf) = targets[rng.random_range(0..targets.len())];
        let muzzle = ship.pos + Vec2::new(ship.nose - 2.0, 0.0);
        for k in 0..count {
            let spread = Vec2::new(0.0, (k as f32 - (count as f32 - 1.0) / 2.0) * 3.0);
            let aim_off = Vec2::new(rng.random_range(-0.5..0.3) * thalf.x, rng.random_range(-0.5..0.5) * thalf.y);
            spawn_shot(
                &mut commands,
                &art,
                Shot { kind, from: muzzle + spread, target: Some(target), aim: tpos + aim_off, offset: aim_off, friendly: true, delay: k as f32 * 0.08, damage: 0.0 },
            );
        }
        if !matches!(kind, ShotKind::Beam { .. }) {
            burst(&mut commands, &art, "muzzle", muzzle, 21.0, 24.0);
        }
        sfx.write(Sfx(sound));
    }
}

/// Muzzle point of the flagship (for click lasers and drone shots).
pub fn flagship_muzzle(ships: &Query<&FleetShip>) -> Option<Vec2> {
    ships.iter().find(|s| s.class.is_none()).map(|s| s.pos + Vec2::new(s.nose - 2.0, 0.0))
}

