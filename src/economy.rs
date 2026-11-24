//! Pure game rules: costs, damage, enemy scaling, waves and prestige.
//!
//! Nothing in here touches Bevy, so the whole progression curve can be unit
//! tested and simulated (see the `simulate_progression` test).

use serde::{Deserialize, Serialize};

pub const WAVES_PER_SECTOR: u32 = 10;
pub const BOSS_TIME: f32 = 30.0;
pub const MAX_HULL_TIER: u32 = 6;
/// Earliest sector reached before a Warp Jump pays out any shards.
pub const JUMP_MIN_SECTOR: u32 = 25;

// ---------------------------------------------------------------------------
// Ship classes
// ---------------------------------------------------------------------------

pub struct ShipClass {
    pub name: &'static str,
    pub blurb: &'static str,
    pub base_cost: f64,
    pub base_dps: f64,
    pub unlock_sector: u32,
    /// Sprite name under assets/sprites (fleet_<key>.png).
    pub key: &'static str,
}

pub const SHIP_CLASSES: [ShipClass; 9] = [
    ShipClass { name: "Interceptor", blurb: "Fast twin-blaster escort", base_cost: 15.0, base_dps: 1.6, unlock_sector: 1, key: "interceptor" },
    ShipClass { name: "Frigate", blurb: "Rapid pulse batteries", base_cost: 180.0, base_dps: 12.0, unlock_sector: 3, key: "frigate" },
    ShipClass { name: "Destroyer", blurb: "Missile salvos", base_cost: 2.6e3, base_dps: 95.0, unlock_sector: 7, key: "destroyer" },
    ShipClass { name: "Cruiser", blurb: "Plasma lances", base_cost: 4.2e4, base_dps: 800.0, unlock_sector: 13, key: "cruiser" },
    ShipClass { name: "Battlecruiser", blurb: "Heavy rail turrets", base_cost: 8.0e5, base_dps: 7.2e3, unlock_sector: 20, key: "battlecruiser" },
    ShipClass { name: "Battleship", blurb: "Siege cannon broadsides", base_cost: 1.8e7, base_dps: 7.0e4, unlock_sector: 30, key: "battleship" },
    ShipClass { name: "Carrier", blurb: "Launches strike craft", base_cost: 5.0e8, base_dps: 7.5e5, unlock_sector: 42, key: "carrier" },
    ShipClass { name: "Dreadnought", blurb: "Spinal beam cannon", base_cost: 1.6e10, base_dps: 8.5e6, unlock_sector: 56, key: "dreadnought" },
    ShipClass { name: "Titan", blurb: "Doomsday emitter", base_cost: 6.0e11, base_dps: 1.0e8, unlock_sector: 72, key: "titan" },
];

pub const SHIP_COST_GROWTH: f64 = 1.075;

/// Logistics hulls: they repair the fleet instead of shooting.
pub struct LogiClass {
    pub name: &'static str,
    pub blurb: &'static str,
    pub base_cost: f64,
    /// Repairs per second per level, before multipliers.
    pub base_rps: f64,
    pub unlock_sector: u32,
    pub key: &'static str,
}

pub const LOGI_CLASSES: [LogiClass; 3] = [
    LogiClass { name: "Logi Frigate", blurb: "Remote repair beams", base_cost: 200.0, base_rps: 12.0, unlock_sector: 6, key: "logi_frigate" },
    LogiClass { name: "Logi Cruiser", blurb: "Heavy remote repairs", base_cost: 6.0e4, base_rps: 2.6e3, unlock_sector: 22, key: "logi_cruiser" },
    LogiClass { name: "Force Auxiliary", blurb: "Capital repair hub", base_cost: 1.5e9, base_rps: 2.5e6, unlock_sector: 50, key: "fax" },
];

/// Escort classes then logi classes share one index space on the battlefield:
/// 0..9 are SHIP_CLASSES, 9..12 are LOGI_CLASSES.
pub const HULL_KINDS: usize = SHIP_CLASSES.len() + LOGI_CLASSES.len();
pub const LOGI_BASE: usize = SHIP_CLASSES.len();

pub fn hull_key(kind: usize) -> &'static str {
    if kind < LOGI_BASE { SHIP_CLASSES[kind].key } else { LOGI_CLASSES[kind - LOGI_BASE].key }
}

/// Share of hits on escorts that Rustborn hulls dodge.
pub const RUSTBORN_EVASION: f64 = 0.25;
/// A fresh flagship's hull and self-repair per second, before upgrades.
pub const FLAGSHIP_BASE_HP: f64 = 120.0;
pub const FLAGSHIP_BASE_REP: f64 = 2.0;
/// Share of the run's shards kept when the flagship is destroyed.
pub const DEATH_SHARD_SHARE: f64 = 0.5;
/// Between waves the fleet regroups and patches up this share of max HP.
pub const REGROUP_HEAL: f64 = 0.1;
/// Escorts slowly repair themselves: share of max HP per second.
pub const ESCORT_LOCAL_REP: f64 = 0.01;
/// Escort HP per point of the class's base DPS (per level, before multipliers).
pub const ESCORT_HP_PER_DPS: f64 = 25.0;

pub const HULL_NAMES: [&str; 6] = ["Pathfinder", "Vanguard", "Paladin", "Sovereign", "Leviathan", "Empyrean"];
/// Sector required and cost to refit *into* tier index i+1 (index 0 unused).
pub const HULL_REQ_SECTOR: [u32; 6] = [0, 6, 16, 32, 55, 85];
pub const HULL_COST: [f64; 6] = [0.0, 400.0, 2.0e5, 5.0e9, 3.0e15, 1.5e23];
/// Every hull refit multiplies all damage.
pub const HULL_MULT: f64 = 3.0;

// ---------------------------------------------------------------------------
// Player factions (chosen at the start of every run)
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum Race {
    /// Lasers and armour plating.
    Throne,
    /// Missiles and shields.
    #[default]
    Tidewall,
    /// Drones and blasters.
    Concord,
    /// Autocannons and speed.
    Rustborn,
}

pub struct RaceDef {
    pub name: &'static str,
    pub key: &'static str,
    pub doctrine: &'static str,
    pub perks: [&'static str; 3],
}

impl Race {
    pub const ALL: [Race; 4] = [Race::Throne, Race::Tidewall, Race::Concord, Race::Rustborn];

    pub fn def(self) -> RaceDef {
        match self {
            Race::Throne => RaceDef {
                name: "Solar Throne",
                key: "throne",
                doctrine: "Lasers & Armor",
                perks: ["+50% click damage", "+10s boss timer", "+40% hull HP"],
            },
            Race::Tidewall => RaceDef {
                name: "Tidewall State",
                key: "tidewall",
                doctrine: "Missiles & Shields",
                perks: ["+10% crit, +2x crit damage", "-30% ability cooldowns", "+30% local repairs"],
            },
            Race::Concord => RaceDef {
                name: "Free Concord",
                key: "concord",
                doctrine: "Drones & Blasters",
                perks: ["+3 drone shots/s, x2 drones", "+25% fleet DPS", "+25% logi repairs"],
            },
            Race::Rustborn => RaceDef {
                name: "Rustborn Clans",
                key: "rustborn",
                doctrine: "Autocannons & Speed",
                perks: ["-20% fleet ship costs", "+30% credits", "Escorts dodge 25% of hits"],
            },
        }
    }
}

// ---------------------------------------------------------------------------
// Flagship systems
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum System {
    /// Flagship autocannons: passive DPS.
    Cannons,
    /// Targeting laser: base click damage.
    Laser,
    /// Crit chance and multiplier for clicks.
    Crit,
    /// Drone bay: automatic clicks per second.
    Drones,
    /// Fire-control sync: clicks also deal a share of fleet DPS.
    Sync,
    /// Flagship armour: max HP.
    Plating,
    /// Flagship self-repair per second.
    Repairs,
}

pub struct SystemDef {
    pub name: &'static str,
    pub base_cost: f64,
    pub growth: f64,
    pub max_level: u32,
    pub unlock_sector: u32,
}

impl System {
    /// Weapon systems, listed on the Ship tab.
    pub const ALL: [System; 5] = [System::Laser, System::Cannons, System::Crit, System::Drones, System::Sync];
    /// Survival systems, listed on the Logi tab.
    pub const DEFENSE: [System; 2] = [System::Plating, System::Repairs];

    pub fn def(self) -> SystemDef {
        match self {
            System::Laser => SystemDef { name: "Targeting Laser", base_cost: 5.0, growth: 1.075, max_level: u32::MAX, unlock_sector: 1 },
            System::Cannons => SystemDef { name: "Autocannons", base_cost: 8.0, growth: 1.075, max_level: u32::MAX, unlock_sector: 1 },
            System::Crit => SystemDef { name: "Crit Matrix", base_cost: 120.0, growth: 1.45, max_level: 45, unlock_sector: 2 },
            System::Drones => SystemDef { name: "Drone Bay", base_cost: 900.0, growth: 4.5, max_level: 15, unlock_sector: 5 },
            System::Sync => SystemDef { name: "Fire Control", base_cost: 2.0e4, growth: 7.0, max_level: 25, unlock_sector: 10 },
            System::Plating => SystemDef { name: "Hull Plating", base_cost: 20.0, growth: SHIP_COST_GROWTH, max_level: u32::MAX, unlock_sector: 2 },
            System::Repairs => SystemDef { name: "Local Repairs", base_cost: 40.0, growth: SHIP_COST_GROWTH, max_level: u32::MAX, unlock_sector: 3 },
        }
    }
}

// ---------------------------------------------------------------------------
// Research (bought with shards, survives Warp Jumps)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Research {
    Doctrine,
    Salvage,
    Chrono,
    Autoloader,
    Momentum,
}

pub struct ResearchDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub base_cost: f64,
    pub growth: f64,
    pub max_level: u32,
}

impl Research {
    pub const ALL: [Research; 5] = [Research::Doctrine, Research::Salvage, Research::Chrono, Research::Autoloader, Research::Momentum];

    pub fn index(self) -> usize {
        Research::ALL.iter().position(|r| *r == self).unwrap()
    }

    pub fn def(self) -> ResearchDef {
        match self {
            Research::Doctrine => ResearchDef { name: "War Doctrine", desc: "+50% all damage", base_cost: 2.0, growth: 1.6, max_level: u32::MAX },
            Research::Salvage => ResearchDef { name: "Salvage Rights", desc: "+40% credits", base_cost: 2.0, growth: 1.6, max_level: u32::MAX },
            Research::Chrono => ResearchDef { name: "Chrono Anchor", desc: "+3s boss timer", base_cost: 5.0, growth: 2.0, max_level: 10 },
            Research::Autoloader => ResearchDef { name: "Autoloader", desc: "+2 drone shots/s", base_cost: 4.0, growth: 2.2, max_level: 10 },
            Research::Momentum => ResearchDef { name: "Jump Momentum", desc: "Start 5 sectors deeper", base_cost: 10.0, growth: 2.5, max_level: 20 },
        }
    }
}

// ---------------------------------------------------------------------------
// Abilities
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Ability {
    Overcharge,
    Broadside,
    SalvageSweep,
}

pub struct AbilityDef {
    pub name: &'static str,
    pub desc: &'static str,
    pub duration: f32,
    pub cooldown: f32,
    pub unlock_sector: u32,
    pub mult: f64,
}

impl Ability {
    pub const ALL: [Ability; 3] = [Ability::Overcharge, Ability::Broadside, Ability::SalvageSweep];

    pub fn index(self) -> usize {
        Ability::ALL.iter().position(|a| *a == self).unwrap()
    }

    pub fn def(self) -> AbilityDef {
        match self {
            Ability::Overcharge => AbilityDef { name: "Overcharge", desc: "x5 click damage", duration: 20.0, cooldown: 120.0, unlock_sector: 4, mult: 5.0 },
            Ability::Broadside => AbilityDef { name: "Broadside", desc: "x3 fleet DPS", duration: 20.0, cooldown: 180.0, unlock_sector: 9, mult: 3.0 },
            Ability::SalvageSweep => AbilityDef { name: "Salvage Sweep", desc: "x3 credits", duration: 30.0, cooldown: 300.0, unlock_sector: 15, mult: 3.0 },
        }
    }
}

// ---------------------------------------------------------------------------
// Formulas
// ---------------------------------------------------------------------------

/// Damage doubles at every 25 levels of anything that levels.
pub fn milestone(level: u32) -> f64 {
    2f64.powi((level / 25) as i32)
}

/// Total cost of buying `n` levels starting from `level`.
pub fn bulk_cost(base: f64, growth: f64, level: u32, n: u32) -> f64 {
    if n == 0 {
        return 0.0;
    }
    base * growth.powi(level as i32) * (growth.powi(n as i32) - 1.0) / (growth - 1.0)
}

/// How many levels `budget` can buy (0 if not even one).
pub fn max_affordable(base: f64, growth: f64, level: u32, budget: f64) -> u32 {
    let first = base * growth.powi(level as i32);
    if budget < first {
        return 0;
    }
    let n = ((budget * (growth - 1.0) / first) + 1.0).ln() / growth.ln();
    let mut n = n.floor().max(1.0) as u32;
    // guard against float edge cases
    while n > 1 && bulk_cost(base, growth, level, n) > budget {
        n -= 1;
    }
    n
}

pub fn ship_dps(class: usize, level: u32) -> f64 {
    if level == 0 {
        return 0.0;
    }
    SHIP_CLASSES[class].base_dps * level as f64 * milestone(level)
}

/// Most hulls of each class shown at once: capital ships are big enough
/// that a couple of them fill the battlefield.
pub const SHIP_CAP: [u32; HULL_KINDS] = [5, 5, 4, 4, 3, 3, 2, 2, 1, 3, 2, 1];

/// Hulls of a kind (see HULL_KINDS) in the formation: grows with level.
pub fn ship_count(class: usize, level: u32) -> u32 {
    let n = match level {
        0 => 0,
        1..=9 => 1,
        10..=24 => 2,
        25..=49 => 3,
        50..=99 => 4,
        _ => 5,
    };
    n.min(SHIP_CAP[class])
}

/// Total HP of all enemies in a regular wave of `sector`.
pub fn wave_hp(sector: u32) -> f64 {
    let s = sector.max(1) as f64 - 1.0;
    // steep early so upgrades matter right away, settling to ~1.17/sector
    let early = 1.62f64.powf(s.min(8.0));
    let late = 1.235f64.powf((s - 8.0).max(0.0));
    20.0 * early * late
}

pub fn boss_hp(sector: u32) -> f64 {
    wave_hp(sector) * 8.0
}

/// Total damage per second a regular wave deals to the player's fleet. The
/// threat ramps up over the first sectors so new players can learn first.
pub fn wave_dps(sector: u32) -> f64 {
    let ramp = (THREAT_FLOOR + (1.0 - THREAT_FLOOR) * sector as f64 / THREAT_RAMP_SECTORS).min(1.0);
    wave_hp(sector) * ENEMY_DPS_FRAC * ramp
}

pub const THREAT_RAMP_SECTORS: f64 = 12.0;
/// Share of full threat enemies carry in sector 0 of the ramp.
pub const THREAT_FLOOR: f64 = 0.4;

pub const ENEMY_DPS_FRAC: f64 = 0.12;
/// Enemy self-repair per second as a share of max HP, by tier (index = tier).
pub const ENEMY_LOCAL_REP: [f64; 6] = [0.0, 0.0, 0.0, 0.004, 0.01, 0.015];
/// Regular waves start bringing a logistics ship at this sector...
pub const ENEMY_LOGI_SECTOR: u32 = 8;
/// ...and bosses arrive with logi escorts from here (two from the second).
pub const BOSS_LOGI_SECTORS: [u32; 2] = [20, 60];
/// Repairs per second of a wave's logi ship, as a share of the wave's HP.
pub const ENEMY_LOGI_REP: f64 = 0.08;
/// Repairs per second of each boss logi escort, as a share of the boss's HP.
pub const BOSS_LOGI_REP: f64 = 0.012;

/// Credits a wave pays, as a share of its total HP.
pub const CREDIT_SHARE: f64 = 0.025;

pub fn wave_credits(sector: u32) -> f64 {
    (wave_hp(sector) * CREDIT_SHARE).max(2.0).round()
}

pub fn boss_credits(sector: u32) -> f64 {
    wave_credits(sector) * 6.0
}

pub fn shards_for(max_sector: u32) -> f64 {
    if max_sector < JUMP_MIN_SECTOR {
        return 0.0;
    }
    // exponential, slightly slower than enemy HP: every jump reaches deeper
    (2.0 * 1.1f64.powi((max_sector - JUMP_MIN_SECTOR) as i32)).floor()
}

/// Faction index for a sector: rotates every 10 sectors.
pub fn faction(sector: u32) -> usize {
    (((sector.max(1) - 1) / 10) % 4) as usize
}

pub const FACTION_KEYS: [&str; 4] = ["crimson", "verdant", "void", "synod"];
pub const FACTION_NAMES: [&str; 4] = ["Crimson Reavers", "Verdant Swarm", "Void Choir", "Iron Synod"];
const BOSS_NAMES: [[&str; 4]; 4] = [
    ["Warlord Kharr", "Butcher Vex", "Red Matriarch", "Scrapking Ordo"],
    ["Brood Mother", "The Devourer", "Hive Lord Zzik", "Elder Spawn"],
    ["Hollow Saint", "Choir Prime", "The Eclipse", "Null Oracle"],
    ["Prime Arbiter", "Iron Cardinal", "The Engine-God", "Archon Vel"],
];
const SECTOR_NAMES: [&str; 20] = [
    "Ashen Reach", "Kepler Drift", "Cinder Gate", "Hollow Verge", "Tidewell", "Morrow Rift",
    "Glass Expanse", "Ion Narrows", "Brimstone Belt", "Pale Harbor", "Rustline", "Thornfield",
    "Silent Choir", "Ember Shoals", "Duskmere", "Halo Fringe", "Shatterpoint", "Lumen Deep",
    "Iron Wake", "Starfall Run",
];

pub fn boss_name(sector: u32) -> &'static str {
    BOSS_NAMES[faction(sector)][((sector / 10) as usize / 4) % 4]
}

pub fn sector_name(sector: u32) -> &'static str {
    SECTOR_NAMES[(sector.max(1) as usize * 7 + 3) % SECTOR_NAMES.len()]
}

/// Enemy tier: 1 drone, 2 raider, 3 gunship, 4 warship, 5 boss.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EnemySpawn {
    pub tier: u32,
    pub hp: f64,
    pub credits: f64,
    /// Damage per second against the player's fleet.
    pub dps: f64,
    /// Self-repair per second.
    pub rep: f64,
    /// Remote repairs per second given to allies (logistics ships only).
    pub logi: f64,
}

impl EnemySpawn {
    pub fn is_logi(&self) -> bool {
        self.logi > 0.0
    }
}

/// Composition of a wave; HP, credits and damage are split across the
/// enemies by size. From ENEMY_LOGI_SECTOR one small ship in larger waves is
/// a logistics ship that repairs the others instead of shooting.
pub fn wave_enemies(sector: u32, wave: u32) -> Vec<EnemySpawn> {
    if wave >= WAVES_PER_SECTOR {
        return boss_wave(sector);
    }
    let top = (1 + sector / 6).min(4);
    let tiers: Vec<u32> = match wave % 4 {
        1 => vec![top],
        2 => vec![top.saturating_sub(1).max(1); 2],
        3 => vec![1, top, 1],
        _ => vec![1, top.saturating_sub(1).max(1), 1, top.saturating_sub(2).max(1)],
    };
    // the last tier-1 ship of a 3+ ship wave flies logistics
    let logi_slot = if sector >= ENEMY_LOGI_SECTOR && tiers.len() >= 3 { tiers.iter().rposition(|t| *t == 1) } else { None };
    let weight = |t: u32| [0.0, 1.0, 1.8, 3.0, 5.0][t as usize];
    let total: f64 = tiers.iter().map(|t| weight(*t)).sum();
    let gun_total: f64 = tiers.iter().enumerate().filter(|(i, _)| Some(*i) != logi_slot).map(|(_, t)| weight(*t)).sum();
    let hp = wave_hp(sector);
    let credits = wave_credits(sector);
    let dps = wave_dps(sector);
    tiers
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let share = weight(*t) / total;
            let logi = Some(i) == logi_slot;
            EnemySpawn {
                tier: *t,
                hp: hp * share,
                credits: (credits * share).max(1.0),
                dps: if logi { 0.0 } else { dps * weight(*t) / gun_total },
                rep: hp * share * ENEMY_LOCAL_REP[*t as usize],
                logi: if logi { hp * ENEMY_LOGI_REP } else { 0.0 },
            }
        })
        .collect()
}

/// A boss, plus logi escorts deeper in. Escorts are tier-2 hulls.
pub fn boss_wave(sector: u32) -> Vec<EnemySpawn> {
    let hp = boss_hp(sector);
    let mut v = vec![EnemySpawn {
        tier: 5,
        hp,
        credits: boss_credits(sector),
        dps: wave_dps(sector) * 2.0,
        rep: hp * ENEMY_LOCAL_REP[5],
        logi: 0.0,
    }];
    let escorts = BOSS_LOGI_SECTORS.iter().filter(|s| sector >= **s).count();
    for _ in 0..escorts {
        v.push(EnemySpawn {
            tier: 2,
            hp: wave_hp(sector) * 0.6,
            credits: wave_credits(sector) * 0.5,
            dps: 0.0,
            rep: 0.0,
            logi: hp * BOSS_LOGI_REP,
        });
    }
    v
}

/// Share of an enemy's fire aimed at the flagship (the rest hits escorts).
pub const FLAGSHIP_AGGRO: f64 = 0.35;
pub const BOSS_FLAGSHIP_AGGRO: f64 = 0.6;
pub fn flagship_aggro(tier: u32) -> f64 {
    if tier == 5 { BOSS_FLAGSHIP_AGGRO } else { FLAGSHIP_AGGRO }
}

// ---------------------------------------------------------------------------
// Persistent state
// ---------------------------------------------------------------------------

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct GameState {
    pub credits: f64,
    pub shards: f64,
    pub sector: u32,
    pub wave: u32,
    pub max_sector: u32,
    pub laser: u32,
    pub cannons: u32,
    pub crit: u32,
    pub drones: u32,
    pub sync: u32,
    pub hull: u32,
    pub plating: u32,
    pub repairs: u32,
    pub fleet: [u32; 9],
    pub logi: [u32; 3],
    pub research: [u32; 5],
    pub jumps: u32,
    /// Runs ended by losing the flagship.
    pub deaths: u32,
    pub best_sector: u32,
    pub total_credits: f64,
    pub kills: u64,
    pub clicks: u64,
    pub play_time: f64,
    pub muted: bool,
    pub race: Race,
    /// False at the very start and after a Warp Jump: the faction picker shows.
    pub race_chosen: bool,
    pub saved_at: u64,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            credits: 0.0,
            shards: 0.0,
            sector: 1,
            wave: 1,
            max_sector: 1,
            laser: 0,
            cannons: 0,
            crit: 0,
            drones: 0,
            sync: 0,
            hull: 1,
            plating: 0,
            repairs: 0,
            fleet: [0; 9],
            logi: [0; 3],
            research: [0; 5],
            jumps: 0,
            deaths: 0,
            best_sector: 1,
            total_credits: 0.0,
            kills: 0,
            clicks: 0,
            play_time: 0.0,
            muted: false,
            race: Race::Tidewall,
            race_chosen: false,
            saved_at: 0,
        }
    }
}

/// Temporary multipliers from abilities.
#[derive(Clone, Copy, Debug)]
pub struct Boosts {
    pub click: f64,
    pub dps: f64,
    pub credits: f64,
}

impl Default for Boosts {
    fn default() -> Self {
        Self { click: 1.0, dps: 1.0, credits: 1.0 }
    }
}

impl GameState {
    pub fn system_level(&self, s: System) -> u32 {
        match s {
            System::Laser => self.laser,
            System::Cannons => self.cannons,
            System::Crit => self.crit,
            System::Drones => self.drones,
            System::Sync => self.sync,
            System::Plating => self.plating,
            System::Repairs => self.repairs,
        }
    }

    fn system_level_mut(&mut self, s: System) -> &mut u32 {
        match s {
            System::Laser => &mut self.laser,
            System::Cannons => &mut self.cannons,
            System::Crit => &mut self.crit,
            System::Drones => &mut self.drones,
            System::Sync => &mut self.sync,
            System::Plating => &mut self.plating,
            System::Repairs => &mut self.repairs,
        }
    }

    pub fn research_level(&self, r: Research) -> u32 {
        self.research[r.index()]
    }

    /// Multiplier applied to every source of damage.
    pub fn damage_mult(&self) -> f64 {
        let hull = HULL_MULT.powi(self.hull as i32 - 1);
        let shards = 1.0 + 0.1 * self.shards;
        let doctrine = 1.0 + 0.5 * self.research_level(Research::Doctrine) as f64;
        hull * shards * doctrine
    }

    pub fn credit_mult(&self) -> f64 {
        let race = if self.race == Race::Rustborn { 1.3 } else { 1.0 };
        (1.0 + 0.4 * self.research_level(Research::Salvage) as f64) * race
    }

    fn escort_mult(&self) -> f64 {
        if self.race == Race::Concord { 1.25 } else { 1.0 }
    }

    pub fn ship_cost_mult(&self) -> f64 {
        if self.race == Race::Rustborn { 0.8 } else { 1.0 }
    }

    pub fn cooldown_mult(&self) -> f32 {
        if self.race == Race::Tidewall { 0.7 } else { 1.0 }
    }

    pub fn cannon_dps(&self) -> f64 {
        if self.cannons == 0 {
            return 0.0;
        }
        self.cannons as f64 * milestone(self.cannons) * self.damage_mult()
    }

    pub fn class_dps(&self, class: usize) -> f64 {
        ship_dps(class, self.fleet[class]) * self.damage_mult() * self.escort_mult()
    }

    /// Passive fleet DPS (flagship cannons + all escorts), before boosts.
    pub fn fleet_dps(&self) -> f64 {
        self.cannon_dps() + (0..SHIP_CLASSES.len()).map(|c| self.class_dps(c)).sum::<f64>()
    }

    pub fn sync_share(&self) -> f64 {
        0.02 * self.sync as f64
    }

    /// Damage of one (non-crit) click.
    pub fn click_damage(&self) -> f64 {
        let base = (1.0 + self.laser as f64) * milestone(self.laser);
        let race = if self.race == Race::Throne { 1.5 } else { 1.0 };
        base * self.damage_mult() * race + self.fleet_dps() * self.sync_share()
    }

    // --- survival ----------------------------------------------------------

    /// Level of a hull kind (escort class or logi class, see HULL_KINDS).
    pub fn hull_level(&self, kind: usize) -> u32 {
        if kind < LOGI_BASE { self.fleet[kind] } else { self.logi[kind - LOGI_BASE] }
    }

    pub fn hull_levels(&self) -> [u32; HULL_KINDS] {
        std::array::from_fn(|k| self.hull_level(k))
    }

    fn hp_mult(&self) -> f64 {
        self.damage_mult() * if self.race == Race::Throne { 1.4 } else { 1.0 }
    }

    pub fn local_rep_mult(&self) -> f64 {
        if self.race == Race::Tidewall { 1.3 } else { 1.0 }
    }

    pub fn flagship_max_hp(&self) -> f64 {
        (FLAGSHIP_BASE_HP + 30.0 * self.plating as f64 * milestone(self.plating)) * self.hp_mult()
    }

    /// Flagship self-repair per second.
    pub fn flagship_rep(&self) -> f64 {
        (FLAGSHIP_BASE_REP + 1.5 * self.repairs as f64 * milestone(self.repairs)) * self.damage_mult() * self.local_rep_mult()
    }

    /// Max HP of one hull of `kind` (a class's HP is shared by its hulls).
    pub fn escort_hp(&self, kind: usize) -> f64 {
        let level = self.hull_level(kind);
        if level == 0 {
            return 0.0;
        }
        let base = if kind < LOGI_BASE { SHIP_CLASSES[kind].base_dps } else { LOGI_CLASSES[kind - LOGI_BASE].base_rps * 0.6 };
        base * ESCORT_HP_PER_DPS * level as f64 * milestone(level) * self.hp_mult() / ship_count(kind, level).max(1) as f64
    }

    /// Total remote repairs per second of a logi class (0..3).
    pub fn logi_rps(&self, class: usize) -> f64 {
        let level = self.logi[class];
        if level == 0 {
            return 0.0;
        }
        let race = if self.race == Race::Concord { 1.25 } else { 1.0 };
        LOGI_CLASSES[class].base_rps * level as f64 * milestone(level) * self.damage_mult() * race
    }

    pub fn total_logi_rps(&self) -> f64 {
        (0..LOGI_CLASSES.len()).map(|c| self.logi_rps(c)).sum()
    }

    /// Chance that a hit on an escort misses.
    pub fn escort_evasion(&self) -> f64 {
        if self.race == Race::Rustborn { RUSTBORN_EVASION } else { 0.0 }
    }

    /// One hull of `kind` (escort or logi class) was destroyed. Its share of the
    /// class's levels is gone for good and has to be bought again. Returns the
    /// levels lost.
    pub fn lose_hull(&mut self, kind: usize) -> u32 {
        let level = self.hull_level(kind);
        let lost = level.div_ceil(ship_count(kind, level).max(1));
        let slot = if kind < LOGI_BASE { &mut self.fleet[kind] } else { &mut self.logi[kind - LOGI_BASE] };
        *slot -= lost;
        lost
    }

    pub fn crit_chance(&self) -> f64 {
        let race = if self.race == Race::Tidewall { 0.1 } else { 0.0 };
        let own = if self.crit == 0 { 0.0 } else { 0.04 + 0.01 * self.crit as f64 };
        (own + race).min(0.6)
    }

    pub fn crit_mult(&self) -> f64 {
        let race = if self.race == Race::Tidewall { 2.0 } else { 0.0 };
        3.0 + 0.25 * self.crit as f64 + race
    }

    /// Automatic clicks per second from the drone bay and Autoloader research.
    pub fn drone_rate(&self) -> f64 {
        let base = self.drones as f64 + 2.0 * self.research_level(Research::Autoloader) as f64;
        if self.race == Race::Concord { base * 2.0 + 3.0 } else { base }
    }

    pub fn boss_time(&self) -> f32 {
        let race = if self.race == Race::Throne { 10.0 } else { 0.0 };
        BOSS_TIME + 3.0 * self.research_level(Research::Chrono) as f32 + race
    }

    pub fn system_unlocked(&self, s: System) -> bool {
        self.max_sector >= s.def().unlock_sector
    }

    pub fn class_unlocked(&self, class: usize) -> bool {
        self.max_sector >= SHIP_CLASSES[class].unlock_sector || self.fleet[class] > 0
    }

    pub fn logi_unlocked(&self, class: usize) -> bool {
        self.max_sector >= LOGI_CLASSES[class].unlock_sector || self.logi[class] > 0
    }

    pub fn ability_unlocked(&self, a: Ability) -> bool {
        self.best_sector.max(self.max_sector) >= a.def().unlock_sector
    }

    // --- purchasing ------------------------------------------------------

    /// Levels to buy given the shop's buy mode (None = max affordable),
    /// capped by the max level. Returns (levels, cost).
    fn plan(&self, base: f64, growth: f64, level: u32, max_level: u32, want: Option<u32>) -> (u32, f64) {
        let room = max_level.saturating_sub(level);
        let n = match want {
            Some(n) => n.min(room),
            None => max_affordable(base, growth, level, self.credits).min(room),
        };
        (n, bulk_cost(base, growth, level, n.max(if room > 0 { 1 } else { 0 })))
    }

    pub fn system_quote(&self, s: System, want: Option<u32>) -> (u32, f64) {
        let d = s.def();
        let lvl = self.system_level(s);
        let (n, cost) = self.plan(d.base_cost, d.growth, lvl, d.max_level, want);
        if want.is_none() && n == 0 {
            return (if lvl < d.max_level { 1 } else { 0 }, cost);
        }
        (n, cost)
    }

    pub fn buy_system(&mut self, s: System, want: Option<u32>) -> bool {
        if !self.system_unlocked(s) {
            return false;
        }
        let (n, cost) = self.system_quote(s, want);
        if n == 0 || cost > self.credits {
            return false;
        }
        self.credits -= cost;
        *self.system_level_mut(s) += n;
        true
    }

    pub fn class_quote(&self, class: usize, want: Option<u32>) -> (u32, f64) {
        let c = &SHIP_CLASSES[class];
        let base = c.base_cost * self.ship_cost_mult();
        let (n, cost) = self.plan(base, SHIP_COST_GROWTH, self.fleet[class], u32::MAX, want);
        if want.is_none() && n == 0 {
            return (1, cost);
        }
        (n, cost)
    }

    pub fn buy_class(&mut self, class: usize, want: Option<u32>) -> bool {
        if !self.class_unlocked(class) {
            return false;
        }
        let (n, cost) = self.class_quote(class, want);
        if n == 0 || cost > self.credits {
            return false;
        }
        self.credits -= cost;
        self.fleet[class] += n;
        true
    }

    pub fn logi_quote(&self, class: usize, want: Option<u32>) -> (u32, f64) {
        let c = &LOGI_CLASSES[class];
        let base = c.base_cost * self.ship_cost_mult();
        let (n, cost) = self.plan(base, SHIP_COST_GROWTH, self.logi[class], u32::MAX, want);
        if want.is_none() && n == 0 {
            return (1, cost);
        }
        (n, cost)
    }

    pub fn buy_logi(&mut self, class: usize, want: Option<u32>) -> bool {
        if !self.logi_unlocked(class) {
            return false;
        }
        let (n, cost) = self.logi_quote(class, want);
        if n == 0 || cost > self.credits {
            return false;
        }
        self.credits -= cost;
        self.logi[class] += n;
        true
    }

    /// Next hull refit: (sector needed, cost), None at max tier.
    pub fn hull_quote(&self) -> Option<(u32, f64)> {
        if self.hull >= MAX_HULL_TIER {
            return None;
        }
        let i = self.hull as usize;
        Some((HULL_REQ_SECTOR[i], HULL_COST[i]))
    }

    pub fn buy_hull(&mut self) -> bool {
        let Some((req, cost)) = self.hull_quote() else { return false };
        if self.max_sector < req || self.credits < cost {
            return false;
        }
        self.credits -= cost;
        self.hull += 1;
        true
    }

    pub fn research_cost(&self, r: Research) -> Option<f64> {
        let d = r.def();
        let lvl = self.research_level(r);
        if lvl >= d.max_level {
            return None;
        }
        Some((d.base_cost * d.growth.powi(lvl as i32)).round())
    }

    pub fn buy_research(&mut self, r: Research) -> bool {
        let Some(cost) = self.research_cost(r) else { return false };
        if self.shards < cost {
            return false;
        }
        self.shards -= cost;
        self.research[r.index()] += 1;
        true
    }

    // --- progression -----------------------------------------------------

    pub fn earn(&mut self, credits: f64) {
        self.credits += credits;
        self.total_credits += credits;
    }

    /// Advance after clearing a wave. Returns true when a new sector began.
    pub fn wave_cleared(&mut self) -> bool {
        if self.wave >= WAVES_PER_SECTOR {
            self.sector += 1;
            self.wave = 1;
            self.max_sector = self.max_sector.max(self.sector);
            self.best_sector = self.best_sector.max(self.sector);
            true
        } else {
            self.wave += 1;
            false
        }
    }

    /// Boss escaped: fall back to the start of the sector to farm.
    pub fn boss_failed(&mut self) {
        self.wave = 1;
    }

    pub fn jump_reward(&self) -> f64 {
        shards_for(self.max_sector)
    }

    pub fn start_sector(&self) -> u32 {
        1 + 5 * self.research_level(Research::Momentum)
    }

    /// Warp Jump: trade the run for shards. Research, shards and stats persist.
    pub fn warp_jump(&mut self) -> f64 {
        let gained = self.jump_reward();
        self.end_run(gained);
        self.jumps += 1;
        gained
    }

    /// Shards banked if the flagship is destroyed now.
    pub fn death_reward(&self) -> f64 {
        (self.jump_reward() * DEATH_SHARD_SHARE).floor()
    }

    /// The flagship was destroyed: the run ends, keeping part of its shards.
    pub fn die(&mut self) -> f64 {
        let gained = self.death_reward();
        self.end_run(gained);
        self.deaths += 1;
        gained
    }

    fn end_run(&mut self, gained: f64) {
        let start = self.start_sector();
        let keep = GameState {
            shards: self.shards + gained,
            research: self.research,
            jumps: self.jumps,
            deaths: self.deaths,
            best_sector: self.best_sector,
            total_credits: self.total_credits,
            kills: self.kills,
            clicks: self.clicks,
            play_time: self.play_time,
            muted: self.muted,
            race: self.race,
            sector: start,
            max_sector: start,
            ..Default::default()
        };
        *self = keep;
    }
}

// ---------------------------------------------------------------------------
// Number formatting
// ---------------------------------------------------------------------------

const SUFFIXES: [&str; 12] = ["", "K", "M", "B", "T", "Qa", "Qi", "Sx", "Sp", "Oc", "No", "Dc"];

pub fn fmt(v: f64) -> String {
    if !v.is_finite() {
        return "inf".into();
    }
    if v.abs() < 1000.0 {
        if v.abs() < 10.0 && v.fract().abs() > 0.05 {
            return format!("{:.1}", v);
        }
        return format!("{:.0}", v.floor());
    }
    let exp = (v.abs().log10() / 3.0).floor() as usize;
    if exp < SUFFIXES.len() {
        let scaled = v / 1000f64.powi(exp as i32);
        let digits = if scaled.abs() >= 100.0 { 0 } else if scaled.abs() >= 10.0 { 1 } else { 2 };
        format!("{:.*}{}", digits, scaled, SUFFIXES[exp])
    } else {
        let e = v.abs().log10().floor() as i32;
        format!("{:.2}e{}", v / 10f64.powi(e), e)
    }
}

pub fn fmt_time(secs: f64) -> String {
    let s = secs.max(0.0) as u64;
    if s >= 3600 {
        format!("{}h {:02}m", s / 3600, (s % 3600) / 60)
    } else if s >= 60 {
        format!("{}m {:02}s", s / 60, s % 60)
    } else {
        format!("{}s", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bulk_cost_matches_sum_of_singles() {
        let mut sum = 0.0;
        for l in 3..13 {
            sum += bulk_cost(10.0, 1.07, l, 1);
        }
        let bulk = bulk_cost(10.0, 1.07, 3, 10);
        assert!((sum - bulk).abs() / bulk < 1e-9);
    }

    #[test]
    fn max_affordable_never_overspends() {
        for budget in [0.0, 9.0, 10.0, 55.0, 1e6, 1e30] {
            let n = max_affordable(10.0, 1.07, 5, budget);
            if n > 0 {
                assert!(bulk_cost(10.0, 1.07, 5, n) <= budget);
            }
            assert!(bulk_cost(10.0, 1.07, 5, n + 1) > budget);
        }
    }

    #[test]
    fn milestones_double_every_25_levels() {
        assert_eq!(milestone(24), 1.0);
        assert_eq!(milestone(25), 2.0);
        assert_eq!(milestone(100), 16.0);
    }

    #[test]
    fn wave_hp_split_preserves_total() {
        for s in [1, 7, 33, 120] {
            for w in 1..WAVES_PER_SECTOR {
                let total: f64 = wave_enemies(s, w).iter().map(|e| e.hp).sum();
                assert!((total - wave_hp(s)).abs() / total < 1e-9);
            }
            assert_eq!(wave_enemies(s, WAVES_PER_SECTOR)[0].tier, 5);
        }
    }

    #[test]
    fn hp_is_monotonic_and_finite_deep() {
        let mut prev = 0.0;
        for s in 1..3000 {
            let hp = wave_hp(s);
            assert!(hp > prev && hp.is_finite(), "sector {s}");
            prev = hp;
        }
    }

    #[test]
    fn buying_spends_credits_and_levels_up() {
        let mut g = GameState { credits: 100.0, ..Default::default() };
        assert!(g.buy_system(System::Laser, Some(1)));
        assert_eq!(g.laser, 1);
        assert!((g.credits - 95.0).abs() < 1e-9);
        assert!(g.buy_class(0, None));
        assert!(g.fleet[0] >= 1);
        assert!(!g.buy_class(8, Some(1)), "titan is locked at sector 1");
    }

    #[test]
    fn max_buy_quote_shows_next_level_when_broke() {
        let g = GameState::default();
        let (n, cost) = g.system_quote(System::Laser, None);
        assert_eq!(n, 1);
        assert!((cost - 5.0).abs() < 1e-9);
    }

    #[test]
    fn capped_systems_stop_at_max() {
        let mut g = GameState { credits: 1e300, max_sector: 100, ..Default::default() };
        assert!(g.buy_system(System::Drones, None));
        assert_eq!(g.drones, System::Drones.def().max_level);
        assert!(!g.buy_system(System::Drones, Some(1)));
    }

    #[test]
    fn warp_jump_keeps_meta_progress() {
        let mut g = GameState { max_sector: 45, sector: 45, credits: 1e9, laser: 80, ..Default::default() };
        g.research[Research::Momentum.index()] = 2;
        let gained = g.warp_jump();
        assert_eq!(gained, shards_for(45));
        assert_eq!(g.shards, gained);
        assert_eq!(g.laser, 0);
        assert_eq!(g.credits, 0.0);
        assert_eq!(g.sector, 11);
        assert_eq!(g.jumps, 1);
        assert!(!g.race_chosen, "a jump lets the player pick a new faction");
    }

    #[test]
    fn race_perks_apply() {
        let base = GameState { laser: 10, crit: 5, drones: 2, ..Default::default() };
        let throne = GameState { race: Race::Throne, ..base.clone() };
        let concord = GameState { race: Race::Concord, ..base.clone() };
        let rust = GameState { race: Race::Rustborn, ..base.clone() };
        let tide = GameState { race: Race::Tidewall, ..base.clone() };
        assert!(throne.click_damage() > rust.click_damage());
        assert!(throne.boss_time() > tide.boss_time());
        assert!(concord.drone_rate() > throne.drone_rate());
        assert!(tide.crit_chance() > rust.crit_chance());
        assert!(rust.class_quote(0, Some(1)).1 < tide.class_quote(0, Some(1)).1);
        assert!(rust.credit_mult() > tide.credit_mult());
    }

    #[test]
    fn formatting() {
        assert_eq!(fmt(5.0), "5");
        assert_eq!(fmt(999.0), "999");
        assert_eq!(fmt(1234.0), "1.23K");
        assert_eq!(fmt(45_600_000.0), "45.6M");
        assert_eq!(fmt(1e40), "1.00e40");
        assert_eq!(fmt_time(3725.0), "1h 02m");
    }

    #[test]
    fn save_roundtrip_tolerates_missing_fields() {
        let g: GameState = serde_json::from_str(r#"{"credits": 12.5, "sector": 4}"#).unwrap();
        assert_eq!(g.sector, 4);
        assert_eq!(g.hull, 1);
        let s = serde_json::to_string(&g).unwrap();
        let back: GameState = serde_json::from_str(&s).unwrap();
        assert_eq!(back.credits, 12.5);
    }

    /// Share of the fleet's logi repairs that reach the flagship in a fight.
    const LOGI_ON_FLAGSHIP: f64 = 0.6;

    /// Expected result of a fight in which the fleet focuses one enemy at a time,
    /// logistics first (as it does on the battlefield).
    #[derive(Clone, Copy, Debug)]
    struct Fight {
        /// Seconds to destroy every enemy; infinite if their repairs outpace `dps`.
        time: f64,
        /// Damage aimed at the flagship over the fight.
        flagship_damage: f64,
        /// Damage aimed at escorts over the fight (the flagship's if there are none).
        escort_damage: f64,
    }

    fn fight(spawns: &[EnemySpawn], dps: f64) -> Fight {
        let mut order: Vec<&EnemySpawn> = spawns.iter().collect();
        order.sort_by_key(|e| !e.is_logi());
        let mut t = 0.0;
        let mut flagship_damage = 0.0;
        let mut escort_damage = 0.0;
        for (i, e) in order.iter().enumerate() {
            // the remaining logi ships repair whoever is under fire
            let healing: f64 = order[i + 1..].iter().map(|o| o.logi).sum::<f64>() + e.rep;
            if dps <= healing {
                return Fight { time: f64::INFINITY, flagship_damage: f64::INFINITY, escort_damage: f64::INFINITY };
            }
            t += e.hp / (dps - healing);
            flagship_damage += e.dps * flagship_aggro(e.tier) * t;
            escort_damage += e.dps * (1.0 - flagship_aggro(e.tier)) * t;
        }
        Fight { time: t, flagship_damage, escort_damage }
    }

    /// Escort and logi hulls on the field as (kind, max HP), weakest first.
    fn hulls(g: &GameState) -> Vec<(usize, f64)> {
        let mut v: Vec<(usize, f64)> =
            (0..HULL_KINDS).flat_map(|k| std::iter::repeat_n((k, g.escort_hp(k)), ship_count(k, g.hull_level(k)) as usize)).collect();
        v.sort_by(|a, b| a.1.total_cmp(&b.1));
        v
    }

    /// Flagship HP lost over a fight, after its own and the logi's repairs.
    /// With no escorts on the field, all fire lands on the flagship.
    fn fight_loss(g: &GameState, f: &Fight) -> f64 {
        let logi = if hulls(g).is_empty() { 0.0 } else { LOGI_ON_FLAGSHIP };
        let incoming = f.flagship_damage + if hulls(g).is_empty() { f.escort_damage } else { 0.0 };
        let sustain = g.flagship_rep() + g.total_logi_rps() * logi;
        (incoming - sustain * f.time).max(0.0)
    }

    /// Damage each escort hull takes over a fight, after evasion, local repairs
    /// and the logi's share. Enemies pick a random hull per volley, so the fire
    /// spreads evenly and the weakest hulls go first.
    fn escort_wear(g: &GameState, f: &Fight) -> f64 {
        let hulls = hulls(g);
        if hulls.is_empty() {
            return 0.0;
        }
        let pool: f64 = hulls.iter().map(|h| h.1).sum();
        let sustain = ESCORT_LOCAL_REP * g.local_rep_mult() * pool + g.total_logi_rps() * (1.0 - LOGI_ON_FLAGSHIP);
        ((f.escort_damage * (1.0 - g.escort_evasion()) - sustain * f.time) / hulls.len() as f64).max(0.0)
    }

    /// Destroy every hull whose HP `wear` exceeds; returns the hulls lost.
    fn lose_worn(g: &mut GameState, wear: f64) -> u32 {
        let mut lost = 0;
        while let Some((kind, _)) = hulls(g).into_iter().find(|h| h.1 <= wear) {
            g.lose_hull(kind);
            lost += 1;
        }
        lost
    }

    fn sim_dps(g: &GameState, clicks_per_sec: f64) -> f64 {
        let click_rate = clicks_per_sec + g.drone_rate();
        let crit = 1.0 + g.crit_chance() * (g.crit_mult() - 1.0);
        g.fleet_dps() + g.click_damage() * click_rate * crit
    }

    /// Greedy auto-player used to tune the curve. Run with
    /// `cargo test simulate_progression -- --nocapture --ignored` to see the table.
    /// With `defend` it buys plating, repairs and logi whenever the coming boss
    /// would hurt the flagship badly, and retreats from fights it would lose;
    /// without, it only buys damage and fights everything (and so can die).
    fn simulate(hours: f64, clicks_per_sec: f64, verbose: bool, defend: bool) -> GameState {
        let mut g = GameState::default();
        let mut t = 0.0;
        let mut stall = 0;
        let mut next_report = 0.0;
        let mut jumps_at = vec![];
        let mut hp = g.flagship_max_hp();
        // damage carried by every escort hull, and hulls destroyed so far
        let mut wear = 0.0;
        let mut lost = 0;
        let mut reached: Vec<(u32, f64)> = vec![];
        while t < hours * 3600.0 {
            // spend: best value per credit, a few purchases per tick
            for _ in 0..50 {
                let base = sim_dps(&g, clicks_per_sec);
                let boss = boss_wave(g.sector);
                // flagship HP lost to the coming boss, plus how close it comes to killing an escort
                let danger = |h: &GameState| {
                    let f = fight(&boss, sim_dps(h, clicks_per_sec));
                    let weakest = hulls(h).first().map_or(f64::INFINITY, |w| w.1);
                    fight_loss(h, &f) / h.flagship_max_hp() + escort_wear(h, &f) / weakest
                };
                let defending = defend && danger(&g) > 0.5;
                let mut best: Option<(f64, Box<dyn Fn(&mut GameState) -> bool>)> = None;
                let mut consider = |gain: f64, cost: f64, f: Box<dyn Fn(&mut GameState) -> bool>| {
                    if cost <= g.credits && gain > 0.0 {
                        let v = gain / cost;
                        if best.as_ref().is_none_or(|b| v > b.0) {
                            best = Some((v, f));
                        }
                    }
                };
                // value of an upgrade: damage, or (when in danger) less flagship loss
                let value = |h: &GameState| if defending { danger(&g) - danger(h) } else { sim_dps(h, clicks_per_sec) - base };
                for s in System::ALL.into_iter().chain(System::DEFENSE) {
                    if !g.system_unlocked(s) || g.system_level(s) >= s.def().max_level {
                        continue;
                    }
                    let (_, cost) = g.system_quote(s, Some(1));
                    let mut h = g.clone();
                    h.credits = f64::MAX;
                    h.buy_system(s, Some(1));
                    consider(value(&h), cost, Box::new(move |g: &mut GameState| g.buy_system(s, Some(1))));
                }
                for c in 0..SHIP_CLASSES.len() {
                    if !g.class_unlocked(c) {
                        continue;
                    }
                    let (_, cost) = g.class_quote(c, Some(1));
                    let mut h = g.clone();
                    h.credits = f64::MAX;
                    h.buy_class(c, Some(1));
                    consider(value(&h), cost, Box::new(move |g: &mut GameState| g.buy_class(c, Some(1))));
                }
                for c in 0..LOGI_CLASSES.len() {
                    if !defend || !g.logi_unlocked(c) {
                        continue;
                    }
                    let (_, cost) = g.logi_quote(c, Some(1));
                    let mut h = g.clone();
                    h.credits = f64::MAX;
                    h.buy_logi(c, Some(1));
                    consider(value(&h), cost, Box::new(move |g: &mut GameState| g.buy_logi(c, Some(1))));
                }
                if let Some((req, cost)) = g.hull_quote()
                    && g.max_sector >= req
                    && g.credits >= cost
                {
                    g.buy_hull();
                    continue;
                }
                match best {
                    Some((_, f)) => {
                        f(&mut g);
                    }
                    None => break,
                }
            }
            for r in Research::ALL {
                while g.buy_research(r) {}
            }

            let dps = sim_dps(&g, clicks_per_sec);
            let max = g.flagship_max_hp();
            let boss = g.wave >= WAVES_PER_SECTOR;
            let spawns = wave_enemies(g.sector, g.wave);
            let f = fight(&spawns, dps);
            let loss = fight_loss(&g, &f);
            let worn = wear + escort_wear(&g, &f);
            let timed_out = boss && f.time > g.boss_time() as f64;
            let weakest = hulls(&g).first().map_or(f64::INFINITY, |w| w.1);
            if defend && (loss >= hp || f.time.is_infinite() || worn >= weakest) {
                // retreat: back to the start of the sector, fully repaired
                t += 5.0;
                g.boss_failed();
                hp = max;
                wear = 0.0;
                stall += 1;
            } else if loss >= hp {
                if verbose {
                    println!("{:>7} DIED at sector {} wave {}", fmt_time(t), g.sector, g.wave);
                }
                g.die();
                hp = g.flagship_max_hp();
                wear = 0.0;
                stall = 0;
                continue;
            } else if timed_out {
                t += g.boss_time() as f64;
                hp -= loss * g.boss_time() as f64 / f.time;
                wear += escort_wear(&g, &f) * g.boss_time() as f64 / f.time;
                lost += lose_worn(&mut g, wear);
                g.boss_failed();
                stall += 1;
            } else {
                t += f.time.max(if boss { 3.0 } else { 2.5 });
                hp -= loss;
                wear = worn;
                lost += lose_worn(&mut g, wear);
                let reward = if boss { spawns.iter().map(|e| e.credits).sum() } else { wave_credits(g.sector) };
                g.earn(reward * g.credit_mult());
                if g.wave_cleared() && g.sector > reached.last().map_or(1, |r| r.0) {
                    reached.push((g.sector, t));
                }
                if boss {
                    stall = 0;
                }
            }
            // the fleet regroups between waves
            hp = (hp + g.flagship_max_hp() * REGROUP_HEAL + g.flagship_rep() + g.total_logi_rps()).min(g.flagship_max_hp());
            let weakest = hulls(&g).first().map_or(0.0, |w| w.1);
            wear = (wear - weakest * REGROUP_HEAL).max(0.0);
            if stall >= 6 && g.jump_reward() >= g.shards.max(4.0) {
                jumps_at.push((t, g.max_sector, g.jump_reward()));
                g.warp_jump();
                hp = g.flagship_max_hp();
                stall = 0;
            }
            if verbose && t >= next_report {
                println!(
                    "{:>7} sector {:>4} wave {:>2} | dps {:>9} hp {:>9} rep {:>9} logi {:>9} | hull {} | fleet {:?} logi {:?} pl {} rp {} dr {} | lost {} | shards {}",
                    fmt_time(t), g.sector, g.wave, fmt(g.fleet_dps()), fmt(g.flagship_max_hp()), fmt(g.flagship_rep()),
                    fmt(g.total_logi_rps()), g.hull, g.fleet, g.logi, g.plating, g.repairs, g.drones, lost, fmt(g.shards)
                );
                next_report += if t < 4.0 * 3600.0 { 600.0 } else { 3600.0 };
            }
        }
        if verbose {
            let firsts: Vec<String> = [3, 5, 8, 10, 15, 20, 25, 35, 50]
                .iter()
                .filter_map(|s| reached.iter().find(|r| r.0 >= *s).map(|r| format!("s{}@{}", s, fmt_time(r.1))))
                .collect();
            println!("first reached: {}", firsts.join("  "));
            for (t, s, r) in jumps_at {
                println!("jump at {} from sector {} for {} shards", fmt_time(t), s, r);
            }
        }
        g
    }

    #[test]
    #[ignore]
    fn simulate_progression() {
        simulate(24.0, 4.0, true, true);
        println!("--- damage only, no defence ---");
        simulate(6.0, 4.0, true, false);
    }

    #[test]
    fn defence_is_needed_to_go_deep() {
        // ignoring the flagship's survival gets a run killed early...
        let reckless = simulate(1.0, 5.0, false, false);
        assert!(reckless.deaths > 0, "a damage-only fleet should die within an hour");
        // ...while a fleet that buys plating, repairs and logi keeps going
        let careful = simulate(1.0, 5.0, false, true);
        assert_eq!(careful.deaths, 0);
        assert!(careful.best_sector > reckless.best_sector + 15, "{} vs {}", careful.best_sector, reckless.best_sector);
    }

    #[test]
    fn enemy_repairs_and_logi() {
        // bosses repair themselves, so too little damage can never kill one
        let boss = boss_wave(30);
        let rep = boss[0].rep;
        assert!(fight(&boss[..1], rep * 0.9).time.is_infinite());
        assert!(fight(&boss[..1], rep * 3.0).time.is_finite());
        // deep bosses bring logi escorts, and waves from ENEMY_LOGI_SECTOR do too
        assert_eq!(boss_wave(19).len(), 1);
        assert_eq!(boss_wave(20).len(), 2);
        assert_eq!(boss_wave(60).len(), 3);
        assert!(wave_enemies(ENEMY_LOGI_SECTOR, 3).iter().any(|e| e.is_logi()));
        assert!(!wave_enemies(ENEMY_LOGI_SECTOR - 1, 3).iter().any(|e| e.is_logi()));
        // logi ships don't shoot; the rest of the wave carries all its damage
        for w in 1..WAVES_PER_SECTOR {
            let total: f64 = wave_enemies(40, w).iter().map(|e| e.dps).sum();
            assert!((total - wave_dps(40)).abs() / total < 1e-9);
        }
    }

    #[test]
    fn death_keeps_half_the_shards() {
        let mut g = GameState { max_sector: 60, sector: 60, shards: 10.0, credits: 1e12, plating: 40, ..Default::default() };
        let full = g.jump_reward();
        let gained = g.die();
        assert_eq!(gained, (full * DEATH_SHARD_SHARE).floor());
        assert_eq!(g.shards, 10.0 + gained);
        assert_eq!((g.deaths, g.jumps, g.plating, g.credits), (1, 0, 0, 0.0));
        assert!(!g.race_chosen);
    }

    #[test]
    fn early_game_pacing() {
        // The first sectors take real time and upgrades, not a few minutes of
        // clicking; a careful player reaches Warp Jump range in about an hour.
        let g = simulate(0.25, 5.0, false, true);
        assert!((5..=10).contains(&g.best_sector), "15 min should reach sector 5-10, got {}", g.best_sector);
        let g = simulate(1.0, 5.0, false, true);
        assert!((18..=40).contains(&g.best_sector), "1 hour should reach sector 18-40, got {}", g.best_sector);
    }

    #[test]
    fn losing_a_hull_takes_its_levels() {
        let mut g = GameState { fleet: [30, 5, 0, 0, 0, 0, 0, 0, 0], logi: [12, 0, 0], ..Default::default() };
        // three interceptors share 30 levels: one lost takes 10
        assert_eq!(g.lose_hull(0), 10);
        assert_eq!((g.fleet[0], ship_count(0, g.fleet[0])), (20, 2));
        // a lone frigate takes the whole class with it
        assert_eq!(g.lose_hull(1), 5);
        assert_eq!(g.fleet[1], 0);
        // logi hulls are lost the same way
        assert_eq!(g.lose_hull(LOGI_BASE), 6);
        assert_eq!(g.logi[0], 6);
    }
}
