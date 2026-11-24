//! The player's side of the fight: ship HP, enemy hits, local and logistics
//! repairs, escorts being destroyed for good, retreating, and the destruction
//! of the flagship that ends a run.

use bevy::prelude::*;

use crate::AppState;
use crate::art::Art;
use crate::audio::Sfx;
use crate::battle::{Battle, Enemy, spawn_beam};
use crate::economy::{ESCORT_LOCAL_REP, LOGI_BASE, LOGI_CLASSES, REGROUP_HEAL, SHIP_CLASSES, ship_count};
use crate::fleet::FleetShip;
use crate::fx::{NumberStyle, burst, damage_number, explosion};
use crate::save::Saves;
use crate::ui::{Modal, Toast, UiState};
use crate::{Game, Shake, in_battle};

pub struct SurvivalPlugin;

impl Plugin for SurvivalPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<Hit>()
            .add_message::<Regroup>()
            .add_message::<Retreat>()
            .add_message::<Restart>()
            .init_resource::<Vitals>()
            .init_resource::<Alarm>()
            .init_resource::<Death>()
            .add_systems(OnEnter(AppState::Playing), reset_vitals)
            .add_systems(
                Update,
                (
                    sync_hp,
                    (apply_hits, repairs, regroup, retreat_key, retreat, alarm).chain().run_if(in_battle),
                    (track_vitals, show_ships, ship_bars, death_sequence, restart),
                )
                    .chain()
                    .run_if(in_state(AppState::Playing)),
            );
    }
}

/// An enemy shot landing on one of the player's ships.
#[derive(Message)]
pub struct Hit {
    pub target: Entity,
    pub amount: f64,
}

/// A wave was cleared: the fleet patches itself up.
#[derive(Message)]
pub struct Regroup;

/// Warp out of the fight: back to wave 1 of the sector, fully repaired.
#[derive(Message)]
pub struct Retreat;

/// Leave the death screen and begin a new run.
#[derive(Message)]
pub struct Restart;

/// Smoothed flagship damage taken and repairs received, per second, for the HUD.
#[derive(Resource, Default)]
pub struct Vitals {
    pub incoming: f64,
    pub repairs: f64,
    hit_acc: f64,
    rep_acc: f64,
}

#[derive(Resource, Default)]
struct Alarm {
    cd: f32,
    warned: bool,
}

/// The death sequence and its outcome.
#[derive(Resource, Default)]
pub struct Death {
    /// Seconds since the flagship was destroyed.
    timer: f32,
    exploded: u32,
    pub gained: f64,
    pub sector: u32,
    pub deepest: u32,
}

#[derive(Component)]
pub struct ShipBar {
    fill: bool,
    width: f32,
}

/// HP bar under an escort, shown only while it is damaged.
pub fn spawn_hp_bar(p: &mut ChildSpawnerCommands, size: Vec2) {
    // frames carry flame margin behind the hull, so the hull centre sits right of the frame centre
    let x = 7.0;
    let y = -size.y / 2.0 + 1.0;
    let width = (size.x * 0.35).clamp(12.0, 36.0).round();
    p.spawn((
        Sprite { color: Color::srgba(0.02, 0.02, 0.05, 0.85), custom_size: Some(Vec2::new(width + 2.0, 3.0)), ..default() },
        Transform::from_xyz(x, y, 0.1),
        Visibility::Hidden,
        ShipBar { fill: false, width },
    ));
    p.spawn((
        Sprite { color: Color::srgb_u8(110, 240, 150), custom_size: Some(Vec2::new(width, 1.0)), ..default() },
        Transform::from_xyz(x, y, 0.2),
        Visibility::Hidden,
        ShipBar { fill: true, width },
    ));
}

fn reset_vitals(mut death: ResMut<Death>, mut alarm: ResMut<Alarm>, mut vitals: ResMut<Vitals>) {
    *death = Death::default();
    *alarm = Alarm::default();
    *vitals = Vitals::default();
}

/// Max HP follows the run's upgrades; current HP keeps its fraction.
fn sync_hp(game: Res<Game>, mut ships: Query<&mut FleetShip>) {
    let s = &game.state;
    for mut ship in &mut ships {
        let max = match ship.class {
            None => s.flagship_max_hp(),
            Some(k) => s.escort_hp(k),
        };
        if ship.max_hp <= 0.0 {
            ship.hp = max;
        } else if (max - ship.max_hp).abs() > f64::EPSILON * max {
            ship.hp = (ship.hp / ship.max_hp * max).min(max);
        }
        ship.max_hp = max;
    }
}

fn apply_hits(
    mut commands: Commands,
    art: Res<Art>,
    mut game: ResMut<Game>,
    saves: Res<Saves>,
    mut death: ResMut<Death>,
    mut vitals: ResMut<Vitals>,
    mut shake: ResMut<Shake>,
    mut hits: MessageReader<Hit>,
    mut ships: Query<(Entity, &mut FleetShip)>,
    mut sfx: MessageWriter<Sfx>,
    mut toast: MessageWriter<Toast>,
    god: Option<Res<crate::debug::GodMode>>,
) {
    if god.is_some() {
        hits.clear();
        return;
    }
    let evasion = game.state.escort_evasion();
    for hit in hits.read() {
        let Ok((_, mut ship)) = ships.get_mut(hit.target) else { continue };
        if !ship.active() || game.dead {
            continue;
        }
        if ship.class.is_some() && evasion > 0.0 && rand::random_bool(evasion) {
            continue;
        }
        ship.hp -= hit.amount;
        ship.flash = 0.12;
        if ship.class.is_none() {
            vitals.hit_acc += hit.amount;
        }
        if ship.hp > 0.0 {
            continue;
        }
        ship.hp = 0.0;
        match ship.class {
            Some(kind) => {
                // escort lost for good: its share of the class's levels goes with it
                ship.lost = true;
                let (idx, pos, size) = (ship.idx, ship.pos, ship.half.x * 1.4);
                commands.entity(hit.target).despawn();
                let levels = game.state.lose_hull(kind);
                // close the gap so the formation keeps the surviving hulls in place
                for (_, mut other) in &mut ships {
                    if other.class == Some(kind) && !other.lost && other.idx > idx {
                        other.idx -= 1;
                    }
                }
                explosion(&mut commands, &art, pos, size, &mut sfx);
                sfx.write(Sfx("ship_lost"));
                let name = if kind < LOGI_BASE { SHIP_CLASSES[kind].name } else { LOGI_CLASSES[kind - LOGI_BASE].name };
                let left = game.state.hull_level(kind);
                let sub = if left > 0 { format!("-{levels} levels, {left} left. Rebuy it in the shop") } else { format!("-{levels} levels. The class is gone until you rebuy it") };
                toast.write(Toast::danger(format!("{} LOST", name.to_uppercase()), sub));
            }
            None => {
                game.dead = true;
                *death = Death { sector: game.state.sector, deepest: game.state.max_sector, ..default() };
                // bank the outcome now (saves write the dead run from here on),
                // so quitting mid-sequence can't dodge it
                let mut after = game.state.clone();
                death.gained = after.die();
                saves.save(&after);
                explosion(&mut commands, &art, ship.pos, 60.0, &mut sfx);
                burst(&mut commands, &art, "explosion_big", ship.pos + Vec2::new(10.0, 4.0), 32.0, 10.0);
                shake.0 = 1.0;
                sfx.write(Sfx("death"));
            }
        }
    }
}

fn repairs(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    game: Res<Game>,
    mut vitals: ResMut<Vitals>,
    mut ships: Query<(Entity, &mut FleetShip)>,
) {
    let dt = time.delta_secs();
    let s = &game.state;
    let local = ESCORT_LOCAL_REP * s.local_rep_mult();
    for (_, mut ship) in &mut ships {
        if !ship.active() || ship.hp >= ship.max_hp {
            continue;
        }
        let rate = if ship.class.is_none() { s.flagship_rep() } else { ship.max_hp * local };
        ship.hp = (ship.hp + rate * dt as f64).min(ship.max_hp);
        if ship.class.is_none() {
            vitals.rep_acc += rate * dt as f64;
        }
    }
    // logi beam repairs onto the most damaged hull (not themselves)
    let logi: Vec<(Entity, f64, Vec2)> = ships
        .iter()
        .filter(|(_, sh)| sh.active() && sh.is_logi())
        .map(|(e, sh)| {
            let c = sh.class.unwrap_or(LOGI_BASE);
            let rate = s.logi_rps(c - LOGI_BASE) / ship_count(c, s.hull_level(c)).max(1) as f64;
            (e, rate, sh.pos + Vec2::new(sh.nose - 4.0, 0.0))
        })
        .collect();
    for (healer, rate, from) in logi {
        let target = ships
            .iter()
            .filter(|(e, sh)| *e != healer && sh.active() && sh.max_hp > 0.0 && sh.hp < sh.max_hp * 0.999)
            .min_by(|a, b| (a.1.hp / a.1.max_hp).total_cmp(&(b.1.hp / b.1.max_hp)))
            .map(|(e, sh)| (e, sh.pos));
        let Some((target, to)) = target else { continue };
        let mut pulse = false;
        if let Ok((_, mut h)) = ships.get_mut(healer) {
            h.pulse -= dt;
            if h.pulse <= 0.0 {
                h.pulse = 0.5;
                pulse = true;
            }
        }
        if let Ok((_, mut t)) = ships.get_mut(target) {
            let heal = (rate * dt as f64).min(t.max_hp - t.hp);
            t.hp += heal;
            if t.class.is_none() {
                vitals.rep_acc += heal;
            }
        }
        if pulse {
            spawn_beam(&mut commands, &art, from, to, "rep", 0.3, 0.45);
            damage_number(&mut commands, &art, to + Vec2::new(0.0, 6.0), rate * 0.5, NumberStyle::Repair);
        }
    }
}

fn regroup(mut reader: MessageReader<Regroup>, mut ships: Query<&mut FleetShip>) {
    if reader.read().count() == 0 {
        return;
    }
    for mut ship in &mut ships {
        if ship.active() {
            ship.hp = (ship.hp + ship.max_hp * REGROUP_HEAL).min(ship.max_hp);
        }
    }
}

fn retreat_key(keys: Res<ButtonInput<KeyCode>>, mut out: MessageWriter<Retreat>) {
    if keys.just_pressed(KeyCode::KeyR) {
        out.write(Retreat);
    }
}

fn retreat(
    mut commands: Commands,
    art: Res<Art>,
    mut reader: MessageReader<Retreat>,
    mut game: ResMut<Game>,
    mut battle: ResMut<Battle>,
    enemies: Query<(Entity, &Transform), With<Enemy>>,
    mut ships: Query<&mut FleetShip>,
    mut toast: MessageWriter<Toast>,
    mut sfx: MessageWriter<Sfx>,
) {
    if reader.read().count() == 0 {
        return;
    }
    for (e, tf) in &enemies {
        burst(&mut commands, &art, "warp_in", tf.translation.truncate(), 7.0, 20.0);
        commands.entity(e).despawn();
    }
    // a short breather before the next wave warps in
    *battle = Battle::default();
    battle.intermission = 3.0;
    game.state.boss_failed();
    for mut ship in &mut ships {
        ship.hp = ship.max_hp;
    }
    toast.write(Toast::small("Retreat!".into(), "The fleet warps out, repairs, and regroups at wave 1".into()));
    sfx.write(Sfx("warp"));
}

/// Low-hull alarm, and a one-time hint the first time the flagship is in trouble.
fn alarm(time: Res<Time>, mut alarm: ResMut<Alarm>, ships: Query<&FleetShip>, mut sfx: MessageWriter<Sfx>, mut toast: MessageWriter<Toast>) {
    let Some(flag) = ships.iter().find(|s| s.class.is_none()) else { return };
    if flag.max_hp <= 0.0 {
        return;
    }
    let frac = flag.hp / flag.max_hp;
    alarm.cd -= time.delta_secs();
    if frac < 0.3 && alarm.cd <= 0.0 {
        alarm.cd = 1.6;
        sfx.write(Sfx("alarm"));
    }
    if frac < 0.5 && !alarm.warned {
        alarm.warned = true;
        toast.write(Toast::danger("HULL BREACH".into(), "Press R to retreat. Buy Plating, Repairs and Logi in the Logi tab".into()));
    }
}

fn track_vitals(time: Res<Time>, mut v: ResMut<Vitals>) {
    // exponential moving average of per-second rates, ~2s window
    let dt = time.delta_secs_f64().max(1e-4);
    let k = (dt / 2.0).min(1.0);
    v.incoming += (v.hit_acc / dt - v.incoming) * k;
    v.repairs += (v.rep_acc / dt - v.repairs) * k;
    v.hit_acc = 0.0;
    v.rep_acc = 0.0;
}

fn show_ships(time: Res<Time>, mut ships: Query<(&mut FleetShip, &mut Sprite, &mut Visibility)>) {
    for (mut ship, mut sprite, mut vis) in &mut ships {
        let want = if ship.active() { Visibility::Inherited } else { Visibility::Hidden };
        if *vis != want {
            *vis = want;
        }
        ship.flash = (ship.flash - time.delta_secs()).max(0.0);
        let tint = if ship.flash > 0.0 { Color::srgb(1.0, 0.5, 0.45) } else { Color::WHITE };
        if sprite.color != tint {
            sprite.color = tint;
        }
    }
}

fn ship_bars(ships: Query<(&FleetShip, &Children)>, mut bars: Query<(&ShipBar, &mut Sprite, &mut Transform, &mut Visibility)>) {
    for (ship, children) in &ships {
        let frac = if ship.max_hp > 0.0 { (ship.hp / ship.max_hp).clamp(0.0, 1.0) as f32 } else { 1.0 };
        let show = frac < 0.995 && ship.active();
        for c in children.iter() {
            let Ok((bar, mut sprite, mut tf, mut vis)) = bars.get_mut(c) else { continue };
            let want = if show { Visibility::Inherited } else { Visibility::Hidden };
            if *vis != want {
                *vis = want;
            }
            if bar.fill {
                let w = (bar.width * frac).round().max(1.0);
                sprite.custom_size = Some(Vec2::new(w, 1.0));
                tf.translation.x = 7.0 - (bar.width - w) / 2.0;
                sprite.color = if frac > 0.5 {
                    Color::srgb_u8(110, 240, 150)
                } else if frac > 0.25 {
                    Color::srgb_u8(255, 200, 80)
                } else {
                    Color::srgb_u8(255, 80, 70)
                };
            }
        }
    }
}

/// After the flagship falls the escorts go down one by one, then the death
/// screen appears.
fn death_sequence(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    game: Res<Game>,
    mut death: ResMut<Death>,
    mut ui: ResMut<UiState>,
    ships: Query<(Entity, &FleetShip)>,
    mut sfx: MessageWriter<Sfx>,
) {
    if !game.dead {
        return;
    }
    death.timer += time.delta_secs();
    // flagship first, then an escort every ~0.15s
    let due = if death.timer < 0.05 { 1 } else { 1 + ((death.timer - 0.05) / 0.15) as u32 };
    while death.exploded < due {
        death.exploded += 1;
        let next = ships.iter().filter(|(_, s)| s.active()).min_by(|a, b| a.1.class.is_some().cmp(&b.1.class.is_some()).then(a.1.pos.x.total_cmp(&b.1.pos.x)));
        let Some((e, s)) = next else { break };
        if s.class.is_some() {
            explosion(&mut commands, &art, s.pos, s.half.x * 1.4, &mut sfx);
        }
        commands.entity(e).despawn();
    }
    if death.timer > 2.6 && ui.modal.is_none() {
        ui.modal = Some(Modal::Death);
    }
}

fn restart(
    mut commands: Commands,
    mut reader: MessageReader<Restart>,
    mut game: ResMut<Game>,
    mut battle: ResMut<Battle>,
    mut death: ResMut<Death>,
    mut alarm: ResMut<Alarm>,
    mut ui: ResMut<UiState>,
    saves: Res<Saves>,
    ships: Query<Entity, With<FleetShip>>,
    enemies: Query<Entity, With<Enemy>>,
) {
    if reader.read().count() == 0 || !game.dead {
        return;
    }
    game.state.die();
    game.dead = false;
    game.active = [0.0; 3];
    game.cooldown = [0.0; 3];
    for e in ships.iter().chain(enemies.iter()) {
        commands.entity(e).despawn();
    }
    *battle = Battle::default();
    *death = Death::default();
    *alarm = Alarm::default();
    ui.modal = None;
    saves.save(&game.state);
}
