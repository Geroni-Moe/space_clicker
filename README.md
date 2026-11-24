# Void Armada

A pixel-art space-fleet clicker built with Bevy 0.20. Start with a single
flagship, click to fire its targeting laser, and grow an armada of up to 45
escorts across 9 hull classes while pushing through endless sectors of
escalating enemy fleets.

## Run

```sh
cargo run --release
```

To build a double-clickable `dist/Void Armada.app` (with its own name and
icon in the Dock and menu bar; unsigned, for local use):

```sh
scripts/bundle_macos.sh
```

The project pins Rust 1.97.1 in `rust-toolchain.toml` (Bevy 0.20.1 needs it);
rustup installs it automatically.

## How to play

- **Click** an enemy ship (or anywhere on the battlefield) to fire your
  flagship's targeting laser. **Space** fires at the current target.
- Spend credits in the panel on the right:
  - **Ship**: hull refits (new flagship sprite, x3 all damage), Targeting
    Laser, Autocannons, Crit Matrix, Drone Bay (auto-clicks), Fire Control
    (clicks deal a share of fleet DPS).
  - **Fleet**: 9 escort classes, from Interceptor to Titan. More hulls of a
    class join the formation as it levels, and its damage doubles every 25 levels.
  - **Logi**: Hull Plating and Local Repairs for the flagship, plus three
    logistics hulls (Logi Frigate, Logi Cruiser, Force Auxiliary) that beam
    repairs onto whichever ship is hurt worst.
  - **Tech**: Warp Jump and shard research.
- Buy modes: x1 / x10 / x100 / MAX.
- **Abilities** (bottom left, keys **1 2 3**): Overcharge (x5 clicks),
  Broadside (x3 fleet DPS), Salvage Sweep (x3 credits). They unlock as you go deeper.
- Every sector has 10 waves. Wave 10 is a boss with a timer. If it escapes,
  you drop back to wave 1 of that sector to farm.
- **Survival**: enemies shoot back from sector 1, and the threat reaches full
  strength by sector 12. Losses are permanent: a destroyed escort or logi hull
  takes its share of the class's levels with it (one of three interceptors at
  level 30 costs 10 levels), and you have to buy them again. If the
  **flagship** is destroyed the run is over: you bank half the shards a Warp
  Jump would have paid and start again. The hull
  bar (bottom right) shows repairs vs incoming damage per second. Press **R**
  (or the retreat button) to warp out: back to wave 1, fully repaired. The
  fleet also patches up 10% between waves.
- **Enemy repairs**: gunships, warships and especially bosses repair
  themselves, so too little damage can never kill them. From sector 8 waves
  bring a logistics ship (lime glow) that repairs its allies, and from sector
  20 bosses arrive with logi escorts. Your fleet always shoots logi first.
- **Warp Jump** (from sector 25): reset the run for shards. Each unspent shard
  is +10% damage, and shards also buy permanent research. After a jump you
  pick your faction again.
- **Title menu**: Continue (the most recently played slot), New Game, Load
  Game, Quit. There are three save slots; starting a new game in a used slot
  asks before overwriting it.
- **Esc** in game (or the menu button at the top right) pauses and opens the
  menu: Resume, Save, Save & Quit to Title, Save & Exit. Esc again resumes.
- **Display**: the title and pause menus switch between Windowed and
  Fullscreen and pick a windowed resolution (1280x720 up to 3840x2400, those
  that fit the monitor). The interface scales with the window, so any size or
  aspect works, including dragging the window edge. Settings live in
  `settings.json` next to the save slots.
- Progress autosaves every 10s and on exit. Nothing is earned while the game
  is closed: an unattended fleet could not be trusted to survive.
- Saves live in `~/Library/Application Support/VoidArmada/slot1.json` ..
  `slot3.json` (`%APPDATA%\VoidArmada\` on Windows). An older single
  `save.json` moves into slot 1 automatically.

### Factions

| Faction | Doctrine | Perks |
| --- | --- | --- |
| Solar Throne | Lasers & Armor | +50% click damage, +10s boss timer, +40% hull HP |
| Tidewall State | Missiles & Shields | +10% crit, +2x crit damage, -30% ability cooldowns, +30% local repairs |
| Free Concord | Drones & Blasters | +3 drone shots/s and x2 drones, +25% fleet DPS, +25% logi repairs |
| Rustborn Clans | Autocannons & Speed | -20% fleet ship costs, +30% credits, escorts dodge 25% of hits |

Each faction has its own look, a hand-built design for every one of the 9
escort classes and 3 logistics hulls, and its own weapon effects. The flagship's 6 refit tiers grow
from a frigate to a titan and carry a faction crown: the Throne's halo,
Tidewall's command array, Concord's Liberty Star, or a Rustborn skull totem.

## Project layout

```
src/economy.rs   pure game rules + balance simulation (unit tested)
src/battle.rs    waves, enemies (incl. their repairs and logi), damage, projectiles
src/survival.rs  ship HP, hits, repairs, hull losses, retreat, flagship death
src/fleet.rs     formation, ship arrival, per-faction weapon fire
src/ui.rs        HUD, shop panel, modals, toasts
src/fx.rs        explosions, particles, damage numbers, credit chips
src/backdrop.rs  parallax stars, nebulae, planets
src/save.rs      save slots, autosave, legacy save migration
src/menu.rs      title screen and save-slot pages
src/view.rs      display settings: windowed/fullscreen, resolution, UI scaling
src/debug.rs     opt-in dev/test hooks (see below)
art/scripts/     Aseprite Lua scripts that generate every sprite
art/ships/       one file per faction (and enemies.lua): a function per hull
art/lib/forge.lua  ship builder: lit, bevelled parts, glows, engine flames
assets/          generated PNGs, synthesized WAVs, fonts (OFL)
```

## Regenerating art and sound

All sprites are drawn by Aseprite batch scripts. Each one writes an editable
`.aseprite` source to `art/src/`, the game PNG to `assets/sprites/`, and a
contact sheet to `art/previews/`. `SHIPS_ONLY=<faction|enemies>` rebuilds one
set of ships while iterating:

```sh
A=/Applications/Aseprite.app/Contents/MacOS/aseprite
for s in ships fx backdrop ui icon; do $A --batch --script art/scripts/$s.lua; done
scripts/make_icns.sh # app icon PNGs -> assets/icon/VoidArmada.icns
python3 art/sfx.py   # sound effects -> assets/sfx/*.wav
```

## Tests and balance

```sh
cargo test                                                   # rules, pacing
cargo test --release simulate_progression -- --ignored --nocapture  # 24h auto-play table
```

## Dev hooks (environment variables)

- `VOID_ARMADA_SAVE=<dir>`: use another save directory (holds slot1-3.json).
- `VOID_ARMADA_SLOT=<1-3>`: skip the title menu and play that slot.
- `VOID_ARMADA_DEBUG=1`: F1 credits, F2 +5 sectors, F3 +100 shards, F4 boss
  wave (F2 and F4 clear the field so the next wave matches), F12 screenshot.
- `VOID_ARMADA_SCRIPT=<file>`: replay scripted input (window-level cursor and
  button events) for end-to-end tests. Pair it with `VOID_ARMADA_SHOTS=<dir>`
  for screenshots. Scripted runs keep the window on top, since macOS stops
  drawing covered windows.
- `VOID_ARMADA_QUIT_AFTER=<secs>`: exit automatically.
- `VOID_ARMADA_PERF=1`: log fps, mean and worst frame time and the entity
  count every 2s, with vsync off so the numbers show the real frame cost.
- `VOID_ARMADA_GOD=1`: your ships take no damage (keeps a big fleet alive for
  perf runs).

## License

The code and art are released under the [MIT License](LICENSE).

The bundled fonts, Silkscreen and Pixelify Sans, are not covered by that
license: both use the SIL Open Font License (see `assets/fonts/`).
