//! Void Armada - a space-fleet clicker.

// release builds on Windows are GUI apps: no console window beside the game
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

mod art;
mod audio;
mod backdrop;
mod debug;
mod battle;
mod economy;
mod fleet;
mod fx;
mod menu;
mod save;
mod survival;
mod ui;
mod view;

use bevy::prelude::*;
use bevy::render::view::Msaa;
use bevy::window::{PresentMode, WindowLevel, WindowResizeConstraints, WindowResolution};

use economy::{Ability, Boosts, GameState};

/// World pixels are drawn at this many logical pixels.
pub const ZOOM: f32 = 2.0;
/// Width of the shop panel on the right, in logical pixels.
pub const PANEL_W: f32 = 400.0;
/// Height of the HUD strip across the top, in logical pixels.
pub const HUD_H: f32 = 72.0;
/// Height of the ability bar overlaying the battlefield bottom, in logical pixels.
pub const BAR_H: f32 = 64.0;

#[derive(States, Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum AppState {
    #[default]
    Loading,
    /// Title screen with the save slots.
    Menu,
    Playing,
}

/// The persistent run plus transient combat state.
#[derive(Resource, Default)]
pub struct Game {
    pub state: GameState,
    /// Remaining active seconds per ability.
    pub active: [f32; 3],
    /// Remaining cooldown seconds per ability.
    pub cooldown: [f32; 3],
    /// Set while the jump animation plays; battle pauses.
    pub warping: bool,
    /// Set while a modal dialog is open; battle pauses.
    pub paused: bool,
    /// Set from the moment the flagship is destroyed until the run restarts.
    pub dead: bool,
}

impl Game {
    pub fn boosts(&self) -> Boosts {
        let mut b = Boosts::default();
        for a in Ability::ALL {
            if self.active[a.index()] > 0.0 {
                let m = a.def().mult;
                match a {
                    Ability::Overcharge => b.click = m,
                    Ability::Broadside => b.dps = m,
                    Ability::SalvageSweep => b.credits = m,
                }
            }
        }
        b
    }

    pub fn activate(&mut self, a: Ability) -> bool {
        let i = a.index();
        if !self.state.ability_unlocked(a) || self.cooldown[i] > 0.0 {
            return false;
        }
        let d = a.def();
        self.active[i] = d.duration;
        self.cooldown[i] = d.cooldown * self.state.cooldown_mult();
        true
    }

    /// Battle runs only once a faction is chosen, with no jump, dialog or
    /// death sequence in progress.
    pub fn in_battle(&self) -> bool {
        self.state.race_chosen && !self.warping && !self.paused && !self.dead
    }
}

/// The visible battlefield: its half extents in world units, plus the
/// interface scale and world zoom it was measured with (see view.rs).
#[derive(Resource, Clone, Copy)]
pub struct Field {
    pub half: Vec2,
    /// Interface scale: UI lengths are multiplied by this.
    pub ui: f32,
    /// Logical pixels per world pixel.
    pub zoom: f32,
}

impl Default for Field {
    fn default() -> Self {
        Self { half: Vec2::new(220.0, 180.0), ui: 1.0, zoom: ZOOM }
    }
}

impl Field {
    pub fn bottom(&self) -> f32 {
        -self.half.y + BAR_H * self.ui / self.zoom
    }

    /// Camera position that centres the world origin on the battlefield
    /// (left of the panel, below the HUD).
    pub fn camera_base(&self) -> Vec3 {
        Vec3::new(PANEL_W * self.ui / 2.0 / self.zoom, HUD_H * self.ui / 2.0 / self.zoom, 0.0)
    }

    /// Whether a window point lies on the battlefield, clear of HUD, panel and ability bar.
    pub fn contains(&self, window: &Window, cursor: Vec2) -> bool {
        cursor.x < window.width() - PANEL_W * self.ui && cursor.y > HUD_H * self.ui && cursor.y < window.height() - BAR_H * self.ui
    }
}

#[derive(Component)]
pub struct MainCamera;

/// Camera trauma for screen shake (0..1).
#[derive(Resource, Default)]
pub struct Shake(pub f32);

pub fn in_battle(game: Res<Game>) -> bool {
    game.in_battle()
}

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(AssetPlugin { file_path: asset_root(), ..default() })
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Void Armada".into(),
                        resolution: WindowResolution::new(1280, 800),
                        resize_constraints: WindowResizeConstraints { min_width: 640.0, min_height: 400.0, ..default() },
                        // scripted test runs stay on top: macOS stops presenting
                        // frames for covered windows, which blanks screenshots
                        window_level: if std::env::var("VOID_ARMADA_SCRIPT").is_ok() { WindowLevel::AlwaysOnTop } else { WindowLevel::Normal },
                        // perf runs measure the real frame cost, not the display's refresh cap
                        present_mode: if std::env::var("VOID_ARMADA_PERF").is_ok() { PresentMode::AutoNoVsync } else { PresentMode::AutoVsync },
                        ..default()
                    }),
                    ..default()
                }),
        )
        .insert_resource(ClearColor(Color::srgb_u8(7, 8, 15)))
        .init_state::<AppState>()
        .init_resource::<Field>()
        .init_resource::<Shake>()
        .add_plugins((
            art::ArtPlugin,
            save::SavePlugin,
            audio::AudioFxPlugin,
            backdrop::BackdropPlugin,
            fleet::FleetPlugin,
            battle::BattlePlugin,
            survival::SurvivalPlugin,
            fx::FxPlugin,
            ui::UiPlugin,
            menu::MenuPlugin,
            view::ViewPlugin,
            debug::DebugPlugin,
        ))
        .add_systems(Startup, (spawn_camera, set_dock_icon))
        .add_systems(Update, (track_field, tick_abilities, set_window_icon))
        .add_systems(PostUpdate, apply_shake.before(TransformSystems::Propagate))
        .run();
}

/// Where the game's assets are: inside a macOS app bundle
/// (`Contents/Resources/assets`), next to the executable, or else the source tree's.
fn asset_root() -> String {
    let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|d| d.to_path_buf()));
    let candidates = exe_dir.iter().flat_map(|d| [d.join("../Resources/assets"), d.join("assets")]);
    for dir in candidates {
        if dir.is_dir() {
            return dir.to_string_lossy().into_owned();
        }
    }
    concat!(env!("CARGO_MANIFEST_DIR"), "/assets").to_string()
}

/// Dock icon for runs outside an app bundle (a bundle uses its .icns).
/// Runs on the main thread, as AppKit requires.
#[cfg(target_os = "macos")]
fn set_dock_icon(_main_thread: bevy::ecs::system::NonSendMarker) {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;
    let Some(mtm) = MainThreadMarker::new() else { return };
    let data = NSData::with_bytes(include_bytes!("../assets/icon/icon_512.png"));
    let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) else { return };
    let app = NSApplication::sharedApplication(mtm);
    unsafe { app.setApplicationIconImage(Some(&image)) };
}

#[cfg(not(target_os = "macos"))]
fn set_dock_icon() {}

/// Title bar and taskbar icon on Windows and Linux (an .exe also embeds it, see
/// build.rs). The winit window appears a frame or so after startup, so this
/// retries until it is there. Runs on the main thread, as winit requires.
#[cfg(not(target_os = "macos"))]
fn set_window_icon(
    _main_thread: bevy::ecs::system::NonSendMarker,
    window: Single<Entity, With<bevy::window::PrimaryWindow>>,
    mut done: Local<bool>,
) {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::{CompressedImageFormats, ImageSampler, ImageType};
    if *done {
        return;
    }
    bevy::winit::WINIT_WINDOWS.with_borrow(|windows| {
        let Some(winit_window) = windows.get_window(*window) else { return };
        *done = true;
        let png = include_bytes!("../assets/icon/icon_256.png");
        let Ok(image) = Image::from_buffer(png, ImageType::Extension("png"), CompressedImageFormats::NONE, true, ImageSampler::Default, RenderAssetUsages::default()) else { return };
        let Some(rgba) = image.data.clone() else { return };
        if let Ok(icon) = winit::window::Icon::from_rgba(rgba, image.width(), image.height()) {
            winit_window.set_window_icon(Some(icon));
        }
    });
}

#[cfg(target_os = "macos")]
fn set_window_icon() {}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        MainCamera,
        Msaa::Off,
        Projection::from(OrthographicProjection { scale: 1.0 / ZOOM, ..OrthographicProjection::default_2d() }),
        Transform::from_translation(Field::default().camera_base()),
    ));
}

fn track_field(window: Single<&Window>, mut field: ResMut<Field>) {
    let half = Vec2::new(window.width() - PANEL_W * field.ui, window.height() - HUD_H * field.ui) / 2.0 / field.zoom;
    if field.half != half {
        field.half = half;
    }
}

fn tick_abilities(time: Res<Time>, mut game: ResMut<Game>) {
    if !game.in_battle() {
        return;
    }
    let dt = time.delta_secs();
    for i in 0..3 {
        game.active[i] = (game.active[i] - dt).max(0.0);
        game.cooldown[i] = (game.cooldown[i] - dt).max(0.0);
    }
}

fn apply_shake(
    time: Res<Time>,
    field: Res<Field>,
    mut shake: ResMut<Shake>,
    mut cam: Single<&mut Transform, With<MainCamera>>,
) {
    let base = field.camera_base();
    shake.0 = (shake.0 - time.delta_secs() * 1.8).max(0.0);
    let t = time.elapsed_secs();
    let amp = shake.0 * shake.0 * 6.0;
    let off = Vec3::new((t * 61.0).sin() * amp, (t * 47.0).cos() * amp, 0.0);
    // snap to whole world pixels so the art never shimmers
    cam.translation = (base + off).round();
}
