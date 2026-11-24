//! Save slots with autosave.
//!
//! Three JSON slots live in the platform data dir (`VoidArmada/slot1.json` ..
//! `slot3.json`). A save from before slots existed (`save.json`) becomes slot 1
//! the first time the game starts. `VOID_ARMADA_SAVE=<dir>` points the game at
//! another save directory and `VOID_ARMADA_SLOT=<1-3>` skips the title menu and
//! plays that slot (both used by the end-to-end test harness).

use bevy::prelude::*;
use bevy::window::WindowCloseRequested;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::economy::GameState;
use crate::{AppState, Game};

pub const SLOTS: usize = 3;

pub struct SavePlugin;

impl Plugin for SavePlugin {
    fn build(&self, app: &mut App) {
        let dir = save_dir();
        if migrate_legacy(&dir) {
            info!("moved the old save into slot 1");
        }
        app.insert_resource(Saves { dir, active: None })
            .init_resource::<Game>()
            .insert_resource(Autosave(Timer::from_seconds(10.0, TimerMode::Repeating)))
            .add_systems(Update, autosave.run_if(in_state(AppState::Playing)))
            .add_systems(Last, save_on_exit);
    }
}

/// Where the slots live and which one is being played.
#[derive(Resource)]
pub struct Saves {
    pub dir: PathBuf,
    /// 0-based slot of the run in progress (None on the title screen).
    pub active: Option<usize>,
}

impl Saves {
    /// Write the run to the active slot (no-op on the title screen).
    pub fn save(&self, state: &GameState) {
        if let Some(slot) = self.active {
            write_slot(&self.dir, slot, state);
        }
    }

    pub fn read(&self, slot: usize) -> Option<GameState> {
        read_slot(&self.dir, slot)
    }

    pub fn list(&self) -> [Option<GameState>; SLOTS] {
        list_slots(&self.dir)
    }
}

#[derive(Resource)]
struct Autosave(Timer);

pub fn save_dir() -> PathBuf {
    if let Ok(p) = std::env::var("VOID_ARMADA_SAVE") {
        return PathBuf::from(p);
    }
    dirs::data_dir().unwrap_or_else(|| PathBuf::from(".")).join("VoidArmada")
}

/// Slot to jump straight into, skipping the title menu (test harness).
pub fn auto_slot() -> Option<usize> {
    let n: usize = std::env::var("VOID_ARMADA_SLOT").ok()?.parse().ok()?;
    (1..=SLOTS).contains(&n).then(|| n - 1)
}

pub fn slot_path(dir: &Path, slot: usize) -> PathBuf {
    dir.join(format!("slot{}.json", slot + 1))
}

/// Move a pre-slots `save.json` into slot 1 unless slot 1 is already taken.
pub fn migrate_legacy(dir: &Path) -> bool {
    let old = dir.join("save.json");
    let slot = slot_path(dir, 0);
    if old.is_file() && !slot.exists() {
        return std::fs::rename(&old, &slot).is_ok();
    }
    false
}

pub fn read_slot(dir: &Path, slot: usize) -> Option<GameState> {
    let p = slot_path(dir, slot);
    let text = std::fs::read_to_string(&p).ok()?;
    match serde_json::from_str::<GameState>(&text) {
        Ok(state) => Some(state),
        Err(e) => {
            // keep the broken file around rather than silently overwriting progress
            let backup = p.with_extension("corrupt.json");
            let _ = std::fs::copy(&p, &backup);
            error!("save slot {} unreadable ({e}); backed up to {}", slot + 1, backup.display());
            None
        }
    }
}

pub fn list_slots(dir: &Path) -> [Option<GameState>; SLOTS] {
    std::array::from_fn(|i| read_slot(dir, i))
}

/// The slot played most recently, for Continue.
pub fn most_recent(slots: &[Option<GameState>; SLOTS]) -> Option<usize> {
    slots
        .iter()
        .enumerate()
        .filter_map(|(i, s)| s.as_ref().map(|s| (i, s.saved_at)))
        .max_by_key(|(_, t)| *t)
        .map(|(i, _)| i)
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub fn write_slot(dir: &Path, slot: usize, state: &GameState) {
    let mut state = state.clone();
    state.saved_at = now();
    let p = slot_path(dir, slot);
    let _ = std::fs::create_dir_all(dir);
    let tmp = p.with_extension("tmp");
    match serde_json::to_string_pretty(&state) {
        Ok(text) => {
            if let Err(e) = std::fs::write(&tmp, text).and_then(|_| std::fs::rename(&tmp, &p)) {
                error!("save failed: {e}");
            }
        }
        Err(e) => error!("save failed: {e}"),
    }
}

/// "just now", "12m ago", "3h ago", "2d ago", or a date for anything older
/// than a month.
pub fn time_ago(now: u64, then: u64) -> String {
    if then == 0 {
        return "never".into();
    }
    let s = now.saturating_sub(then);
    match s {
        0..60 => "just now".into(),
        60..3600 => format!("{}m ago", s / 60),
        3600..86_400 => format!("{}h ago", s / 3600),
        86_400..2_592_000 => format!("{}d ago", s / 86_400),
        _ => {
            let (y, m, d) = civil_date(then / 86_400);
            format!("{y}-{m:02}-{d:02}")
        }
    }
}

/// Days since 1970-01-01 to a (year, month, day) date (Howard Hinnant's algorithm).
fn civil_date(days: u64) -> (i64, u32, u32) {
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + if m <= 2 { 1 } else { 0 };
    (y, m, d)
}

/// What goes on disk: once the flagship is destroyed the run is already
/// over, even while the death screen is still up.
pub fn persisted(game: &Game) -> GameState {
    let mut state = game.state.clone();
    if game.dead {
        state.die();
    }
    state
}

fn autosave(time: Res<Time<Real>>, mut timer: ResMut<Autosave>, mut game: ResMut<Game>, saves: Res<Saves>) {
    game.state.play_time += time.delta_secs_f64();
    if timer.0.tick(time.delta()).just_finished() {
        saves.save(&persisted(&game));
    }
}

fn save_on_exit(game: Res<Game>, saves: Res<Saves>, mut close: MessageReader<WindowCloseRequested>, mut exit: MessageReader<AppExit>) {
    if close.read().count() > 0 || exit.read().count() > 0 {
        saves.save(&persisted(&game));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(name: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("void_armada_test_{name}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn slots_roundtrip_and_most_recent() {
        let dir = temp_dir("slots");
        assert!(list_slots(&dir).iter().all(Option::is_none));
        assert_eq!(most_recent(&list_slots(&dir)), None);
        write_slot(&dir, 2, &GameState { sector: 9, ..Default::default() });
        let slots = list_slots(&dir);
        assert_eq!(slots[2].as_ref().map(|s| s.sector), Some(9));
        assert!(slots[0].is_none() && slots[1].is_none());
        // a save with an older timestamp loses to the fresh one
        let s = GameState { sector: 4, saved_at: 1, ..Default::default() };
        std::fs::write(slot_path(&dir, 0), serde_json::to_string(&s).unwrap()).unwrap();
        assert_eq!(most_recent(&list_slots(&dir)), Some(2));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn legacy_save_moves_to_slot_one_once() {
        let dir = temp_dir("legacy");
        std::fs::write(dir.join("save.json"), r#"{"sector": 33}"#).unwrap();
        assert!(migrate_legacy(&dir));
        assert_eq!(read_slot(&dir, 0).map(|s| s.sector), Some(33));
        assert!(!dir.join("save.json").exists());
        // never clobbers an existing slot 1
        std::fs::write(dir.join("save.json"), r#"{"sector": 5}"#).unwrap();
        assert!(!migrate_legacy(&dir));
        assert_eq!(read_slot(&dir, 0).map(|s| s.sector), Some(33));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn corrupt_slot_reads_as_empty_and_is_backed_up() {
        let dir = temp_dir("corrupt");
        std::fs::write(slot_path(&dir, 1), "{not json").unwrap();
        assert!(read_slot(&dir, 1).is_none());
        assert!(dir.join("slot2.corrupt.json").exists());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn time_ago_formats() {
        let now = 1_800_000_000;
        assert_eq!(time_ago(now, 0), "never");
        assert_eq!(time_ago(now, now - 20), "just now");
        assert_eq!(time_ago(now, now - 12 * 60), "12m ago");
        assert_eq!(time_ago(now, now - 3 * 3600), "3h ago");
        assert_eq!(time_ago(now, now - 2 * 86_400), "2d ago");
        // 1_700_000_000 is 2023-11-14
        assert_eq!(time_ago(now, 1_700_000_000), "2023-11-14");
    }
}
