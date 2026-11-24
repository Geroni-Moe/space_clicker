//! Combat: waves, enemies, the boss timer, clicking, drone and fleet damage,
//! kills and rewards, plus the projectiles both sides fire.

use bevy::prelude::*;
use rand::Rng;

use crate::art::{Art, enemy_logi_sheet, enemy_sheet};
use crate::audio::Sfx;
use crate::economy::{Ability, LOGI_BASE, WAVES_PER_SECTOR, boss_name, faction, flagship_aggro, sector_name, ship_count, wave_enemies};
use crate::fleet::{FleetShip, flagship_muzzle};
use crate::survival::{Hit, Regroup};
use crate::fx::{Anim, Fade, NumberStyle, burst, coins, damage_number, explosion};
use crate::ui::Toast;
use crate::{AppState, Field, Game, MainCamera, Shake, in_battle};

pub struct BattlePlugin;

impl Plugin for BattlePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Battle>()
            .init_resource::<Cursor>()
            .add_message::<Strike>()
            .add_systems(PreUpdate, track_cursor)
            .add_systems(OnEnter(AppState::Playing), |mut battle: ResMut<Battle>| *battle = Battle::default())
            .add_systems(OnExit(AppState::Playing), clear_battlefield)
            .add_systems(
                Update,
                (
                    (waves, clicks, drones, fleet_damage, resolve_strikes, enemy_repairs, enemy_fire, ability_keys).chain(),
                    boss_clock,
                )
                    .run_if(in_state(AppState::Playing))
                    .run_if(in_battle),
            )
            .add_systems(
                Update,
                (move_enemies, hp_bars, move_shots, update_beams).run_if(in_state(AppState::Playing)),
            );
    }
}

#[derive(Resource, Default)]
pub struct Battle {
    /// Seconds until the next wave spawns (when no enemies are alive).
    pub intermission: f32,
    pub target: Option<Entity>,
    pub boss_left: Option<f32>,
    pub boss_total: f32,
    fleet_number: f64,
    fleet_number_t: f32,
    drone_acc: f64,
    /// Wave currently on the field: (sector, wave).
    pub spawned: Option<(u32, u32)>,
}

/// Last cursor position in window coordinates, from `CursorMoved` messages.
#[derive(Resource, Default)]
struct Cursor(Option<Vec2>);

fn track_cursor(mut moved: MessageReader<CursorMoved>, mut left: MessageReader<bevy::window::CursorLeft>, mut cursor: ResMut<Cursor>) {
    for m in moved.read() {
        cursor.0 = Some(m.position);
    }
    if left.read().count() > 0 {
        cursor.0 = None;
    }
}

#[derive(Component)]
pub struct Enemy {
    pub hp: f64,
    pub max_hp: f64,
    pub credits: f64,
    pub tier: u32,
    pub home: Vec2,
    pub half: Vec2,
    from: Vec2,
    enter: f32,
    phase: f32,
    fire_cd: f32,
    pub fleeing: bool,
    /// Damage per second against the player's fleet.
    pub dps: f64,
    /// Self-repair per second.
    pub rep: f64,
    /// Remote repairs per second given to allies (logistics ships only).
    pub logi: f64,
    /// Seconds until the next repair beam is drawn.
    pulse: f32,
}

impl Enemy {
    pub fn alive(&self) -> bool {
        self.hp > 0.0 && !self.fleeing
    }

    pub fn arrived(&self) -> bool {
        self.enter >= ARRIVE
    }

    pub fn is_logi(&self) -> bool {
        self.logi > 0.0
    }
}

/// Seconds an enemy takes to drop out of warp into its slot.
const ARRIVE: f32 = 0.45;

#[derive(Component)]
struct HpBar {
    fill: bool,
    width: f32,
}

/// A damage event against one enemy.
#[derive(Message)]
struct Strike {
    target: Entity,
    amount: f64,
    at: Vec2,
    style: NumberStyle,
    /// Number to float up, if any (fleet damage is summed before showing).
    display: Option<f64>,
}

/// Leaving a run: nothing of the fight survives to the title screen.
fn clear_battlefield(mut commands: Commands, q: Query<Entity, Or<(With<Enemy>, With<Projectile>, With<Beam>)>>) {
    for e in &q {
        commands.entity(e).despawn();
    }
}

// ---------------------------------------------------------------------------
// Waves
// ---------------------------------------------------------------------------

fn slots(n: usize) -> Vec<Vec2> {
    match n {
        1 => vec![Vec2::new(105.0, 8.0)],
        2 => vec![Vec2::new(90.0, 48.0), Vec2::new(120.0, -38.0)],
        3 => vec![Vec2::new(80.0, 6.0), Vec2::new(135.0, 70.0), Vec2::new(140.0, -58.0)],
        _ => vec![Vec2::new(70.0, 40.0), Vec2::new(95.0, -40.0), Vec2::new(150.0, 85.0), Vec2::new(160.0, -75.0)],
    }
}

fn waves(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    mut game: ResMut<Game>,
    mut battle: ResMut<Battle>,
    field: Res<Field>,
    enemies: Query<&Enemy>,
    mut toast: MessageWriter<Toast>,
    mut sfx: MessageWriter<Sfx>,
    mut regroup: MessageWriter<Regroup>,
) {
    if enemies.iter().any(|e| e.hp > 0.0) {
        return;
    }
    if let Some((sector, wave)) = battle.spawned.take() {
        // the field just emptied: was it cleared or did the boss escape?
        if game.state.sector == sector && game.state.wave == wave {
            regroup.write(Regroup);
            if game.state.wave_cleared() {
                let s = game.state.sector;
                toast.write(Toast::big(format!("Sector {s}"), sector_name(s).to_string()));
                sfx.write(Sfx("sector"));
            }
            battle.intermission = 0.9;
        } else {
            battle.intermission = 1.2;
        }
        battle.boss_left = None;
    }
    battle.intermission -= time.delta_secs();
    if battle.intermission > 0.0 {
        return;
    }

    let (sector, wave) = (game.state.sector, game.state.wave);
    let spawns = wave_enemies(sector, wave);
    let f = faction(sector);
    let positions = slots(spawns.len());
    let mut rng = rand::rng();
    for (spawn, home) in spawns.iter().zip(positions) {
        let name = if spawn.is_logi() { enemy_logi_sheet(f) } else { enemy_sheet(f, spawn.tier) };
        let size = art.size(&name);
        let home = Vec2::new(home.x.min(field.half.x - size.x / 2.0 - 6.0), home.y);
        // drop out of warp just behind the slot, always on screen
        let from = home + Vec2::new(70.0, rng.random_range(-10.0..10.0));
        let mut sprite = art.sprite(&name);
        sprite.flip_x = true;
        let bar_w = (size.x * 0.7).clamp(14.0, 40.0);
        let mut e = commands.spawn((
            sprite,
            Anim::looping(2, 8.0),
            Transform::from_translation(from.extend(6.0 + rng.random_range(0.0..0.5))),
            Enemy {
                hp: spawn.hp,
                max_hp: spawn.hp,
                credits: spawn.credits,
                tier: spawn.tier,
                home,
                half: size / 2.0,
                from,
                enter: 0.0,
                phase: rng.random_range(0.0..std::f32::consts::TAU),
                fire_cd: rng.random_range(1.0..3.0),
                fleeing: false,
                dps: spawn.dps,
                rep: spawn.rep,
                logi: spawn.logi,
                pulse: rng.random_range(0.0..0.5),
            },
        ));
        if spawn.tier < 5 {
            e.with_children(|p| {
                let y = size.y / 2.0 + 4.0;
                p.spawn((
                    Sprite { color: Color::srgba(0.02, 0.02, 0.05, 0.85), custom_size: Some(Vec2::new(bar_w + 2.0, 3.0)), ..default() },
                    Transform::from_xyz(0.0, y, 0.1),
                    HpBar { fill: false, width: bar_w },
                ));
                p.spawn((
                    Sprite { color: Color::srgb_u8(255, 90, 80), custom_size: Some(Vec2::new(bar_w, 1.0)), ..default() },
                    Transform::from_xyz(0.0, y, 0.2),
                    HpBar { fill: true, width: bar_w },
                ));
            });
        }
        burst(&mut commands, &art, "warp_in", home + Vec2::new(10.0, 0.0), 7.0, 20.0);
    }
    battle.spawned = Some((sector, wave));
    battle.target = None;
    sfx.write(Sfx("warp"));
    if wave >= WAVES_PER_SECTOR {
        let t = game.state.boss_time();
        battle.boss_left = Some(t);
        battle.boss_total = t;
        toast.write(Toast::boss(boss_name(sector).to_string()));
        sfx.write(Sfx("boss"));
    }
}

fn boss_clock(
    time: Res<Time>,
    mut game: ResMut<Game>,
    mut battle: ResMut<Battle>,
    mut enemies: Query<&mut Enemy>,
    mut toast: MessageWriter<Toast>,
) {
    let Some(left) = battle.boss_left else { return };
    let left = left - time.delta_secs();
    if left > 0.0 {
        battle.boss_left = Some(left);
        return;
    }
    battle.boss_left = None;
    for mut e in &mut enemies {
        if e.tier == 5 && e.hp > 0.0 {
            e.fleeing = true;
        }
    }
    game.state.boss_failed();
    toast.write(Toast::small("The boss escaped!".into(), "Regroup and strike again".into()));
}

// ---------------------------------------------------------------------------
// Enemy behaviour
// ---------------------------------------------------------------------------

fn move_enemies(
    mut commands: Commands,
    time: Res<Time>,
    field: Res<Field>,
    mut q: Query<(Entity, &mut Enemy, &mut Transform)>,
) {
    let t = time.elapsed_secs();
    let dt = time.delta_secs();
    for (e, mut enemy, mut tf) in &mut q {
        if enemy.fleeing {
            enemy.enter += dt;
            tf.translation.x += dt * (60.0 + enemy.enter * 220.0);
            if tf.translation.x > field.half.x + 160.0 {
                commands.entity(e).despawn();
            }
            continue;
        }
        enemy.enter += dt;
        let k = (enemy.enter / ARRIVE).min(1.0);
        let ease = 1.0 - (1.0 - k).powi(3);
        let bob_amp = if enemy.tier == 5 { 4.0 } else { 3.0 };
        let bob = Vec2::new((t * 0.8 + enemy.phase).sin() * 3.0, (t * 1.4 + enemy.phase).cos() * bob_amp);
        let p = enemy.from.lerp(enemy.home, ease) + bob * k;
        tf.translation.x = p.x.round();
        tf.translation.y = p.y.round();
    }
}

fn hp_bars(enemies: Query<(&Enemy, &Children)>, mut bars: Query<(&HpBar, &mut Sprite, &mut Transform)>) {
    for (enemy, children) in &enemies {
        let frac = (enemy.hp / enemy.max_hp).clamp(0.0, 1.0) as f32;
        for c in children.iter() {
            if let Ok((bar, mut sprite, mut tf)) = bars.get_mut(c)
                && bar.fill
            {
                let w = (bar.width * frac).max(0.0);
                sprite.custom_size = Some(Vec2::new(w, 1.0));
                tf.translation.x = -(bar.width - w) / 2.0;
            }
        }
    }
}

/// Enemy self-repair and logistics: logi ships beam repairs onto the most
/// damaged ally (never themselves).
fn enemy_repairs(mut commands: Commands, time: Res<Time>, art: Res<Art>, mut enemies: Query<(Entity, &mut Enemy, &Transform)>) {
    let dt = time.delta_secs();
    for (_, mut e, _) in &mut enemies {
        if e.alive() && e.hp < e.max_hp {
            e.hp = (e.hp + e.rep * dt as f64).min(e.max_hp);
        }
    }
    let healers: Vec<(Entity, f64, Vec2)> = enemies
        .iter()
        .filter(|(_, e, _)| e.alive() && e.arrived() && e.is_logi())
        .map(|(id, e, tf)| (id, e.logi, tf.translation.truncate()))
        .collect();
    for (healer, rate, from) in healers {
        let target = enemies
            .iter()
            .filter(|(id, e, _)| *id != healer && e.alive() && e.arrived() && e.hp < e.max_hp)
            .min_by(|a, b| (a.1.hp / a.1.max_hp).total_cmp(&(b.1.hp / b.1.max_hp)))
            .map(|(id, _, tf)| (id, tf.translation.truncate()));
        let Some((target, to)) = target else { continue };
        if let Ok((_, mut e, _)) = enemies.get_mut(target) {
            e.hp = (e.hp + rate * dt as f64).min(e.max_hp);
        }
        if let Ok((_, mut me, _)) = enemies.get_mut(healer) {
            me.pulse -= dt;
            if me.pulse <= 0.0 {
                me.pulse = 0.45;
                spawn_beam(&mut commands, &art, from - Vec2::new(4.0, 0.0), to, "rep_enemy", 0.25, 0.4);
            }
        }
    }
}

fn enemy_fire(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    game: Res<Game>,
    mut enemies: Query<(&mut Enemy, &Transform)>,
    ships: Query<(Entity, &FleetShip)>,
) {
    let flagship = ships.iter().find(|(_, s)| s.class.is_none()).map(|(e, s)| (e, s.pos, s.half));
    let escorts: Vec<(Entity, Vec2, Vec2)> = ships.iter().filter(|(_, s)| s.class.is_some() && s.active()).map(|(e, s)| (e, s.pos, s.half)).collect();
    let Some(flagship) = flagship else { return };
    let mut rng = rand::rng();
    let f = faction(game.state.sector);
    let bolt = ["bolt_red", "plasma", "bolt_purple", "bolt_red"][f];
    for (mut enemy, tf) in &mut enemies {
        if !enemy.alive() || !enemy.arrived() || enemy.dps <= 0.0 {
            continue;
        }
        enemy.fire_cd -= time.delta_secs();
        if enemy.fire_cd > 0.0 {
            continue;
        }
        let gap = rng.random_range(1.4..3.2) / (1.0 + (enemy.tier == 5) as u8 as f32 * 1.5);
        enemy.fire_cd = gap;
        // a volley carries the damage of the time since the last one
        let shots = if enemy.tier == 5 { 3 } else { 1 };
        let damage = enemy.dps * gap as f64 / shots as f64;
        let aggro = flagship_aggro(enemy.tier);
        let (target, pos, half) = if escorts.is_empty() || rng.random_bool(aggro) { flagship } else { escorts[rng.random_range(0..escorts.len())] };
        let from = tf.translation.truncate() - Vec2::new(enemy.half.x - 2.0, 0.0);
        for k in 0..shots {
            let off = Vec2::new(half.x * 0.6, rng.random_range(-0.6..0.6) * half.y);
            spawn_shot(
                &mut commands,
                &art,
                Shot {
                    kind: ShotKind::Bolt(bolt),
                    from: from + Vec2::new(0.0, k as f32 * 4.0 - 4.0),
                    target: Some(target),
                    aim: pos + off,
                    offset: off,
                    friendly: false,
                    delay: k as f32 * 0.1,
                    damage,
                },
            );
        }
    }
}

// ---------------------------------------------------------------------------
// Damage
// ---------------------------------------------------------------------------

fn pick_target(battle: &mut Battle, enemies: &Query<(Entity, &Transform, &mut Enemy)>) -> Option<(Entity, Vec2)> {
    if let Some(t) = battle.target
        && let Ok((_, tf, e)) = enemies.get(t)
        && e.alive()
        && e.arrived()
    {
        return Some((t, tf.translation.truncate()));
    }
    // primary the logi, then the frontmost living enemy
    let best = enemies
        .iter()
        .filter(|(_, _, e)| e.alive() && e.arrived())
        .min_by(|a, b| (!a.2.is_logi(), a.1.translation.x).partial_cmp(&(!b.2.is_logi(), b.1.translation.x)).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(id, tf, _)| (id, tf.translation.truncate()));
    battle.target = best.map(|b| b.0);
    best
}

fn roll_click(game: &Game) -> (f64, bool) {
    let s = &game.state;
    let crit = rand::rng().random_bool(s.crit_chance().clamp(0.0, 1.0));
    let mut dmg = s.click_damage() * game.boosts().click;
    if crit {
        dmg *= s.crit_mult();
    }
    (dmg, crit)
}

fn clicks(
    mut commands: Commands,
    art: Res<Art>,
    mouse: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    window: Single<&Window>,
    field: Res<Field>,
    cursor: Res<Cursor>,
    cam: Single<(&Camera, &GlobalTransform), With<MainCamera>>,
    mut game: ResMut<Game>,
    mut battle: ResMut<Battle>,
    enemies: Query<(Entity, &Transform, &mut Enemy)>,
    ships: Query<&FleetShip>,
    mut strikes: MessageWriter<Strike>,
    mut sfx: MessageWriter<Sfx>,
    mut shake: ResMut<Shake>,
) {
    let mut world_click: Option<Vec2> = None;
    if mouse.just_pressed(MouseButton::Left)
        && let Some(cursor) = cursor.0
        && field.contains(&window, cursor)
    {
        let (camera, cam_tf) = *cam;
        world_click = camera.viewport_to_world_2d(cam_tf, cursor).ok();
        if world_click.is_none() {
            return;
        }
    } else if keys.just_pressed(KeyCode::Space) {
        world_click = pick_target(&mut battle, &enemies).map(|t| t.1);
    }
    let Some(click) = world_click else { return };

    // a click on a ship retargets the fleet onto it
    let hit = enemies
        .iter()
        .filter(|(_, _, e)| e.alive() && e.arrived())
        .find(|(_, tf, e)| {
            let d = (click - tf.translation.truncate()).abs();
            d.x <= e.half.x + 3.0 && d.y <= e.half.y + 3.0
        })
        .map(|(id, _, _)| id);
    if let Some(id) = hit {
        battle.target = Some(id);
    }
    burst(&mut commands, &art, "reticle", click, 45.0, 30.0);
    let Some((target, tpos)) = pick_target(&mut battle, &enemies) else { return };
    let at = if hit.is_some() { click } else { tpos };

    game.state.clicks += 1;
    let (dmg, crit) = roll_click(&game);
    strikes.write(Strike { target, amount: dmg, at, style: if crit { NumberStyle::Crit } else { NumberStyle::Click }, display: Some(dmg) });
    if let Some(muzzle) = flagship_muzzle(&ships) {
        let image = click_beam(game.state.race);
        spawn_beam(&mut commands, &art, muzzle, at, image, if crit { 1.4 } else { 0.8 }, 0.12);
        burst(&mut commands, &art, "spark", at, 31.0, 22.0);
    }
    if crit {
        shake.0 = (shake.0 + 0.35).min(0.8);
        sfx.write(Sfx("crit"));
    } else {
        sfx.write(Sfx("laser"));
    }
}

pub fn click_beam(race: crate::economy::Race) -> &'static str {
    use crate::economy::Race::*;
    match race {
        Throne => "beam_gold",
        Tidewall => "beam_cyan",
        Concord => "beam_green",
        Rustborn => "beam_red",
    }
}

fn drones(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    game: Res<Game>,
    mut battle: ResMut<Battle>,
    enemies: Query<(Entity, &Transform, &mut Enemy)>,
    ships: Query<&FleetShip>,
    mut strikes: MessageWriter<Strike>,
) {
    let rate = game.state.drone_rate();
    if rate <= 0.0 {
        return;
    }
    battle.drone_acc += rate * time.delta_secs() as f64;
    // cap so a hitch doesn't unload a backlog of shots at once
    battle.drone_acc = battle.drone_acc.min(5.0);
    while battle.drone_acc >= 1.0 {
        battle.drone_acc -= 1.0;
        let Some((target, tpos)) = pick_target(&mut battle, &enemies) else { return };
        let (dmg, crit) = roll_click(&game);
        let mut rng = rand::rng();
        let at = tpos + Vec2::new(rng.random_range(-6.0..6.0), rng.random_range(-6.0..6.0));
        strikes.write(Strike { target, amount: dmg, at, style: if crit { NumberStyle::Crit } else { NumberStyle::Drone }, display: Some(dmg) });
        if let Some(muzzle) = flagship_muzzle(&ships) {
            let from = muzzle + Vec2::new(-10.0, rng.random_range(-14.0..14.0));
            spawn_shot(&mut commands, &art, Shot { kind: ShotKind::Bolt("bolt_blue"), from, target: Some(target), aim: at, offset: at - tpos, friendly: true, delay: 0.0, damage: 0.0 });
        }
    }
}

/// Fleet DPS from the hulls actually in the fight (a hull still flying in
/// counts once it is on the field).
pub fn live_fleet_dps<'a>(game: &Game, ships: impl Iterator<Item = &'a FleetShip>) -> f64 {
    let s = &game.state;
    let mut dps = s.cannon_dps();
    for ship in ships {
        if let Some(c) = ship.class
            && c < LOGI_BASE
            && ship.active()
        {
            dps += s.class_dps(c) / ship_count(c, s.fleet[c]).max(1) as f64;
        }
    }
    dps
}

fn fleet_damage(
    time: Res<Time>,
    game: Res<Game>,
    mut battle: ResMut<Battle>,
    enemies: Query<(Entity, &Transform, &mut Enemy)>,
    ships: Query<&FleetShip>,
    mut strikes: MessageWriter<Strike>,
) {
    let dps = live_fleet_dps(&game, ships.iter()) * game.boosts().dps;
    if dps <= 0.0 {
        return;
    }
    let Some((target, tpos)) = pick_target(&mut battle, &enemies) else { return };
    let Ok((_, _, e)) = enemies.get(target) else { return };
    if !e.arrived() {
        return;
    }
    let dmg = dps * time.delta_secs() as f64;
    battle.fleet_number += dmg;
    battle.fleet_number_t += time.delta_secs();
    let mut display = None;
    if battle.fleet_number_t >= 0.5 {
        display = Some(battle.fleet_number);
        battle.fleet_number = 0.0;
        battle.fleet_number_t = 0.0;
    }
    let at = tpos + Vec2::new(0.0, e.half.y * 0.5);
    strikes.write(Strike { target, amount: dmg, at, style: NumberStyle::Fleet, display });
}

fn resolve_strikes(
    mut commands: Commands,
    art: Res<Art>,
    mut game: ResMut<Game>,
    mut battle: ResMut<Battle>,
    mut reader: MessageReader<Strike>,
    mut enemies: Query<(Entity, &Transform, &mut Enemy)>,
    mut sfx: MessageWriter<Sfx>,
    mut shake: ResMut<Shake>,
) {
    let credit_boost = game.boosts().credits;
    for s in reader.read() {
        let Ok((id, tf, mut enemy)) = enemies.get_mut(s.target) else { continue };
        if enemy.hp <= 0.0 || enemy.fleeing {
            continue;
        }
        if let Some(v) = s.display {
            damage_number(&mut commands, &art, s.at, v, s.style);
        }
        enemy.hp -= s.amount;
        if enemy.hp > 0.0 {
            continue;
        }
        // destroyed
        let pos = tf.translation.truncate();
        let reward = enemy.credits * game.state.credit_mult() * credit_boost;
        game.state.earn(reward);
        game.state.kills += 1;
        explosion(&mut commands, &art, pos, enemy.half.x * 2.0, &mut sfx);
        coins(&mut commands, &art, pos, (2 + enemy.tier as usize * 2).min(12));
        damage_number(&mut commands, &art, pos + Vec2::new(0.0, enemy.half.y + 6.0), reward, NumberStyle::Credits);
        if enemy.tier >= 4 {
            shake.0 = (shake.0 + 0.3 + 0.2 * (enemy.tier == 5) as u8 as f32).min(1.0);
        }
        if battle.target == Some(id) {
            battle.target = None;
        }
        if enemy.tier == 5 {
            battle.boss_left = None;
        }
        commands.entity(id).despawn();
    }
}

fn ability_keys(keys: Res<ButtonInput<KeyCode>>, mut game: ResMut<Game>, mut sfx: MessageWriter<Sfx>) {
    for (key, a) in [(KeyCode::Digit1, Ability::Overcharge), (KeyCode::Digit2, Ability::Broadside), (KeyCode::Digit3, Ability::SalvageSweep)] {
        if keys.just_pressed(key) {
            sfx.write(Sfx(if game.activate(a) { "ability" } else { "deny" }));
        }
    }
}

// ---------------------------------------------------------------------------
// Projectiles and beams (visual)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
pub enum ShotKind {
    Bolt(&'static str),
    Missile,
    Shell,
    Drone,
    Beam { image: &'static str, width: f32 },
}

pub struct Shot {
    pub kind: ShotKind,
    pub from: Vec2,
    pub target: Option<Entity>,
    pub aim: Vec2,
    /// Offset from the target's centre to aim at.
    pub offset: Vec2,
    pub friendly: bool,
    pub delay: f32,
    /// Damage dealt on impact (enemy fire only; the player's damage is applied directly).
    pub damage: f64,
}

#[derive(Component)]
struct Projectile {
    target: Option<Entity>,
    aim: Vec2,
    offset: Vec2,
    vel: Vec2,
    speed: f32,
    turn: f32,
    friendly: bool,
    delay: f32,
    pos: Vec2,
    smoke: f32,
    kind: ShotKind,
    age: f32,
    damage: f64,
}

#[derive(Component)]
struct Beam {
    from: Vec2,
    to: Vec2,
    width: f32,
    life: f32,
    total: f32,
}

pub fn spawn_shot(commands: &mut Commands, art: &Art, shot: Shot) {
    if let ShotKind::Beam { image, width } = shot.kind {
        spawn_beam(commands, art, shot.from, shot.aim, image, width, 0.25);
        burst(commands, art, "spark", shot.aim, 31.0, 20.0);
        return;
    }
    let mut rng = rand::rng();
    let dir = (shot.aim - shot.from).normalize_or(Vec2::X);
    let (name, speed, turn, launch) = match shot.kind {
        ShotKind::Bolt(n) => (n, 420.0, 30.0, dir),
        ShotKind::Shell => ("shell", 360.0, 20.0, dir),
        ShotKind::Missile => ("missile", 230.0, 5.0, Vec2::new(0.4, rng.random_range(-1.0..1.0)).normalize()),
        ShotKind::Drone => ("strike_craft", 150.0, 3.0, Vec2::new(0.3, rng.random_range(-1.0..1.0)).normalize()),
        ShotKind::Beam { .. } => unreachable!(),
    };
    let speed = if shot.friendly { speed } else { speed * 0.55 };
    let mut sprite = art.sprite(name);
    sprite.flip_x = false;
    let frames = art.frames(name);
    let mut e = commands.spawn((
        sprite,
        Transform::from_translation(shot.from.extend(20.0)).with_rotation(Quat::from_rotation_z(launch.to_angle())),
        Visibility::Hidden,
        Projectile {
            target: shot.target,
            aim: shot.aim,
            offset: shot.offset,
            vel: launch * speed,
            speed,
            turn,
            friendly: shot.friendly,
            delay: shot.delay,
            pos: shot.from,
            smoke: 0.0,
            kind: shot.kind,
            age: 0.0,
            damage: shot.damage,
        },
    ));
    if frames > 1 {
        e.insert(Anim::looping(frames, 12.0));
    }
}

fn beam_color(image: &str) -> Color {
    match image {
        "beam_gold" => Color::srgb_u8(255, 196, 72),
        "beam_green" => Color::srgb_u8(94, 230, 122),
        "beam_red" => Color::srgb_u8(255, 84, 90),
        "beam_purple" => Color::srgb_u8(190, 120, 255),
        "rep" => Color::srgb_u8(120, 255, 190),
        "rep_enemy" => Color::srgb_u8(190, 255, 110),
        _ => Color::srgb_u8(64, 208, 230),
    }
}

/// A beam is a coloured glow with a white-hot core, both solid quads so they
/// stay bright at any thickness.
pub fn spawn_beam(commands: &mut Commands, _art: &Art, from: Vec2, to: Vec2, image: &str, width: f32, life: f32) {
    let glow = (width * 7.0).max(3.0);
    for (color, thick, z) in [(beam_color(image), glow, 22.0), (Color::WHITE, (glow / 3.0).max(1.0), 22.1)] {
        commands.spawn((
            Sprite { color, custom_size: Some(Vec2::new(1.0, thick)), ..default() },
            Transform::from_translation(from.extend(z)),
            Beam { from, to, width: thick, life, total: life },
        ));
    }
}

fn update_beams(mut commands: Commands, time: Res<Time>, mut q: Query<(Entity, &mut Beam, &mut Sprite, &mut Transform)>) {
    let flicker = if (time.elapsed_secs() * 40.0) as i32 % 2 == 0 { 1.0 } else { 0.75 };
    for (e, mut beam, mut sprite, mut tf) in &mut q {
        beam.life -= time.delta_secs();
        if beam.life <= 0.0 {
            commands.entity(e).despawn();
            continue;
        }
        let k = beam.life / beam.total;
        let d = beam.to - beam.from;
        // thins and fades as it dies, with a fast flicker
        let thick = (beam.width * (0.35 + 0.65 * k) * flicker).max(1.0).round();
        sprite.custom_size = Some(Vec2::new(d.length(), thick));
        sprite.color.set_alpha(k.sqrt());
        let z = tf.translation.z;
        tf.translation = ((beam.from + beam.to) / 2.0).extend(z);
        tf.rotation = Quat::from_rotation_z(d.to_angle());
    }
}

fn move_shots(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    mut shots: Query<(Entity, &mut Projectile, &mut Transform, &mut Visibility)>,
    targets: Query<&Transform, Without<Projectile>>,
    field: Res<Field>,
    mut sfx: MessageWriter<Sfx>,
    mut hits: MessageWriter<Hit>,
) {
    let dt = time.delta_secs();
    for (e, mut p, mut tf, mut vis) in &mut shots {
        if p.delay > 0.0 {
            p.delay -= dt;
            continue;
        }
        *vis = Visibility::Inherited;
        p.age += dt;
        if let Some(t) = p.target
            && let Ok(ttf) = targets.get(t)
        {
            p.aim = ttf.translation.truncate() + p.offset;
        }
        let to = p.aim - p.pos;
        let dist = to.length();
        let want = to.normalize_or(Vec2::X) * p.speed;
        // homing gets stronger with age so curved shots always connect
        let turn = (p.turn + p.age * 8.0) * dt;
        p.vel = p.vel.lerp(want, turn.min(1.0));
        let step = p.vel * dt;
        if dist <= step.length() + 2.0 || p.age > 4.0 {
            if p.friendly {
                let name = if matches!(p.kind, ShotKind::Missile | ShotKind::Shell | ShotKind::Drone) { "explosion_small" } else { "spark" };
                let fx = burst(&mut commands, &art, name, p.aim, 31.0, 22.0);
                if name == "explosion_small" {
                    commands.entity(fx).insert(Transform::from_translation(p.aim.extend(31.0)).with_scale(Vec3::splat(0.4)));
                }
                sfx.write(Sfx("hit"));
            } else {
                let fx = burst(&mut commands, &art, "shield_hit", p.aim + Vec2::new(4.0, 0.0), 31.0, 18.0);
                commands.entity(fx).insert(Fade::new(0.3));
                if let Some(target) = p.target {
                    hits.write(Hit { target, amount: p.damage });
                }
            }
            commands.entity(e).despawn();
            continue;
        }
        p.pos += step;
        tf.translation = p.pos.round().extend(20.0);
        tf.rotation = Quat::from_rotation_z(p.vel.to_angle());
        // missiles leave a smoke trail
        if matches!(p.kind, ShotKind::Missile) {
            p.smoke -= dt;
            if p.smoke <= 0.0 {
                p.smoke = 0.03;
                let s = burst(&mut commands, &art, "smoke", p.pos - p.vel.normalize_or(Vec2::X) * 5.0, 19.0, 14.0);
                commands.entity(s).insert(Fade::new(0.35));
            }
        }
        if p.pos.x.abs() > field.half.x + 300.0 {
            commands.entity(e).despawn();
        }
    }
}
