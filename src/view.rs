//! Display settings: windowed or fullscreen, and the window's resolution.
//!
//! Resolutions are real screen pixels from the usual list (1280x720 .. 3840x2400),
//! offered when they fit the monitor. The interface scales with the window, so
//! the 1280x800 design fills any size and aspect: HUD, panel and battlefield all
//! grow or shrink together, and dragging the window edge works the same way. The
//! world zoom is rounded so every art pixel covers a whole number of screen
//! pixels, keeping sprites crisp at any size. The choice lives in
//! `settings.json` next to the save slots; until one is made, the game picks the
//! largest resolution that fits. Scripted test runs keep the window as created
//! unless the script changes it, so their click coordinates hold.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button};
use bevy::window::{Monitor, OnMonitor, PrimaryMonitor, PrimaryWindow, WindowMode, WindowPosition, WindowResizeConstraints};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::art::Art;
use crate::audio::Sfx;
use crate::ui::{DIM, WHITE, chrome, label};
use crate::{Field, MainCamera, ZOOM};

/// The window size (logical pixels) the layout is designed for: interface scale 1.
const DESIGN: Vec2 = Vec2::new(1280.0, 800.0);
/// Smallest window the player can drag to, in logical pixels.
const MIN: Vec2 = Vec2::new(640.0, 400.0);
/// Space the OS keeps around a window (title bar, taskbar, dock, menu bar), logical pixels.
const OS_MARGIN: Vec2 = Vec2::new(40.0, 100.0);
const RESOLUTIONS: [(u32, u32); 14] = [
    (1280, 720),
    (1280, 800),
    (1366, 768),
    (1440, 900),
    (1600, 900),
    (1680, 1050),
    (1920, 1080),
    (1920, 1200),
    (2560, 1440),
    (2560, 1600),
    (2880, 1800),
    (3200, 1800),
    (3840, 2160),
    (3840, 2400),
];

pub struct ViewPlugin;

impl Plugin for ViewPlugin {
    fn build(&self, app: &mut App) {
        let path = crate::save::save_dir().join("settings.json");
        let data = std::fs::read_to_string(&path).ok().and_then(|t| serde_json::from_str(&t).ok()).unwrap_or_default();
        app.insert_resource(Settings { path, data })
            .insert_resource(Screen { scripted: std::env::var("VOID_ARMADA_SCRIPT").is_ok(), ..default() })
            .add_observer(on_step)
            .add_systems(Update, (apply_display, fit_view, style_steps, setting_text).chain());
    }
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Debug)]
enum Display {
    #[default]
    Windowed,
    Fullscreen,
}

#[derive(Serialize, Deserialize, Default, Clone, Copy, PartialEq, Debug)]
struct SettingsData {
    #[serde(default)]
    display: Display,
    /// Windowed resolution in screen pixels; None until the player picks one.
    #[serde(default)]
    resolution: Option<(u32, u32)>,
}

#[derive(Resource)]
struct Settings {
    path: PathBuf,
    data: SettingsData,
}

impl Settings {
    fn save(&self) {
        let _ = std::fs::create_dir_all(self.path.parent().unwrap_or(&self.path));
        if let Err(e) = std::fs::write(&self.path, serde_json::to_string_pretty(&self.data).unwrap_or_default()) {
            error!("could not save settings to {}: {e}", self.path.display());
        }
    }
}

/// What the window's monitor allows, and the settings last applied.
#[derive(Resource, Default)]
struct Screen {
    scripted: bool,
    /// Monitor size and the part of it a window can use, in screen pixels.
    monitor: Option<(UVec2, UVec2)>,
    applied: Option<SettingsData>,
}

impl Screen {
    /// Resolutions that fit the monitor as a window (always at least the smallest).
    fn choices(&self) -> Vec<(u32, u32)> {
        let fits = |r: &(u32, u32)| self.monitor.is_none_or(|(_, room)| r.0 <= room.x && r.1 <= room.y);
        let v: Vec<_> = RESOLUTIONS.into_iter().filter(fits).collect();
        if v.is_empty() { vec![RESOLUTIONS[0]] } else { v }
    }

    /// The saved resolution, or the largest that fits; a saved one too big for
    /// this monitor falls back to the largest that does.
    fn resolution(&self, setting: Option<(u32, u32)>) -> (u32, u32) {
        let choices = self.choices();
        let largest = *choices.last().unwrap_or(&RESOLUTIONS[0]);
        setting.map_or(largest, |r| if choices.contains(&r) { r } else { largest })
    }
}

fn apply_display(
    settings: Res<Settings>,
    mut screen: ResMut<Screen>,
    mut window: Single<(&mut Window, Option<&OnMonitor>), With<PrimaryWindow>>,
    monitors: Query<(Entity, &Monitor, Has<PrimaryMonitor>)>,
) {
    let (window, on) = &mut *window;
    if let Some((_, m, _)) = monitors.iter().find(|(e, _, primary)| on.map_or(*primary, |m| m.0 == *e)) {
        let size = m.physical_size();
        let margin = (OS_MARGIN * m.scale_factor as f32).as_uvec2();
        let found = Some((size, size.saturating_sub(margin)));
        if screen.monitor != found {
            screen.monitor = found;
        }
    }
    // wait for the monitor so the first resolution is the right one
    if screen.monitor.is_none() || screen.applied == Some(settings.data) {
        return;
    }
    let first = screen.applied.is_none();
    screen.applied = Some(settings.data);
    window.resize_constraints = WindowResizeConstraints { min_width: MIN.x, min_height: MIN.y, ..default() };
    match settings.data.display {
        Display::Fullscreen => window.mode = WindowMode::BorderlessFullscreen(MonitorSelection::Current),
        Display::Windowed => {
            window.mode = WindowMode::Windowed;
            // a scripted run keeps its window until the script picks a size
            if screen.scripted && settings.data.resolution.is_none() {
                return;
            }
            let (w, h) = screen.resolution(settings.data.resolution);
            window.resolution.set_physical_resolution(w, h);
            // recentre after a change in the menu; at startup the OS has already placed it
            if !first {
                window.position = WindowPosition::Centered(MonitorSelection::Current);
            }
        }
    }
}

/// Scale the interface to the window and round the world zoom to whole screen pixels.
fn fit_view(
    window: Single<&Window, With<PrimaryWindow>>,
    mut ui_scale: ResMut<UiScale>,
    mut field: ResMut<Field>,
    mut projection: Single<&mut Projection, With<MainCamera>>,
) {
    let size = Vec2::new(window.width(), window.height());
    let s = (size / DESIGN).min_element().max(0.25);
    let dpi = window.resolution.scale_factor();
    let zoom = (ZOOM * s * dpi).round().max(1.0) / dpi;
    if ui_scale.0 != s {
        ui_scale.0 = s;
    }
    if field.ui != s || field.zoom != zoom {
        field.ui = s;
        field.zoom = zoom;
        if let Projection::Orthographic(o) = &mut **projection {
            o.scale = 1.0 / zoom;
        }
    }
}

/// A menu stepper: which setting, and which way.
#[derive(Component, Clone, Copy)]
struct Step(Row, i32);

#[derive(Clone, Copy, PartialEq)]
enum Row {
    Display,
    Resolution,
}

#[derive(Component)]
struct SettingText(Row);

/// Where a setting sits in its list of options, and how many there are.
fn position(row: Row, settings: &SettingsData, screen: &Screen) -> Option<(usize, usize)> {
    match row {
        Row::Display => Some((settings.display as usize, 2)),
        // fullscreen always uses the monitor's own resolution
        Row::Resolution if settings.display == Display::Fullscreen => None,
        Row::Resolution => {
            let choices = screen.choices();
            let at = choices.iter().position(|r| *r == screen.resolution(settings.resolution)).unwrap_or(0);
            Some((at, choices.len()))
        }
    }
}

fn on_step(ev: On<Activate>, steps: Query<&Step>, mut settings: ResMut<Settings>, screen: Res<Screen>, mut sfx: MessageWriter<Sfx>) {
    let Ok(Step(row, dir)) = steps.get(ev.entity).copied() else { return };
    let Some((at, len)) = position(row, &settings.data, &screen) else {
        sfx.write(Sfx("deny"));
        return;
    };
    let to = at as i32 + dir;
    if to < 0 || to >= len as i32 {
        sfx.write(Sfx("deny"));
        return;
    }
    match row {
        Row::Display => settings.data.display = if to == 0 { Display::Windowed } else { Display::Fullscreen },
        Row::Resolution => settings.data.resolution = Some(screen.choices()[to as usize]),
    }
    settings.save();
    sfx.write(Sfx("tab"));
}

/// "< Windowed >" and "< 1920 x 1080 >" rows for the title and pause menus.
pub fn display_rows(p: &mut ChildSpawnerCommands, art: &Art) {
    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(8), ..default() }).with_children(|p| {
        for row in [Row::Display, Row::Resolution] {
            p.spawn(Node { column_gap: px(8), align_items: AlignItems::Center, ..default() }).with_children(|p| {
                let arrow = |dir: i32| {
                    (
                        Button,
                        Hovered::default(),
                        Step(row, dir),
                        chrome(art, "button", 12.0),
                        Node { width: px(44), height: px(44), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
                    )
                };
                p.spawn(arrow(-1)).with_children(|p| {
                    p.spawn(label(art, "<", 16.0, WHITE));
                });
                p.spawn(Node { width: px(200), justify_content: JustifyContent::Center, ..default() }).with_children(|p| {
                    p.spawn((label(art, "", 16.0, DIM), SettingText(row)));
                });
                p.spawn(arrow(1)).with_children(|p| {
                    p.spawn(label(art, ">", 16.0, WHITE));
                });
            });
        }
    });
}

fn setting_text(
    settings: Res<Settings>,
    screen: Res<Screen>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut texts: Query<(&mut Text, &SettingText)>,
) {
    for (mut t, row) in &mut texts {
        let text = match (row.0, settings.data.display) {
            (Row::Display, Display::Windowed) => "Windowed".to_string(),
            (Row::Display, Display::Fullscreen) => "Fullscreen".to_string(),
            // what fullscreen really gets (the OS may keep a strip, like the notch)
            (Row::Resolution, Display::Fullscreen) => format!("{} x {}", window.physical_width(), window.physical_height()),
            (Row::Resolution, Display::Windowed) => {
                let (w, h) = screen.resolution(settings.data.resolution);
                format!("{w} x {h}")
            }
        };
        if t.0 != text {
            t.0 = text;
        }
    }
}

fn style_steps(
    art: Res<Art>,
    settings: Res<Settings>,
    screen: Res<Screen>,
    mut q: Query<(&Step, &Hovered, Has<Pressed>, &mut ImageNode), With<Button>>,
) {
    for (step, hovered, pressed, mut img) in &mut q {
        let enabled = position(step.0, &settings.data, &screen).is_some_and(|(at, len)| (0..len as i32).contains(&(at as i32 + step.1)));
        let name = match (enabled, pressed, hovered.get()) {
            (false, _, _) => "button_disabled",
            (_, true, _) => "button_pressed",
            (_, false, true) => "button_hover",
            _ => "button",
        };
        let handle = art.ui(name);
        if img.image != handle {
            img.image = handle;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(w: u32, h: u32, scale: f32) -> Screen {
        let size = UVec2::new(w, h);
        Screen { monitor: Some((size, size.saturating_sub((OS_MARGIN * scale).as_uvec2()))), ..default() }
    }

    #[test]
    fn default_is_the_largest_that_fits() {
        // 1080p laptop at 150%: room for 1600x900, not 1920x1080
        assert_eq!(on(1920, 1080, 1.5).resolution(None), (1600, 900));
        // 14" MacBook Pro (3024x1964 at 2x)
        assert_eq!(on(3024, 1964, 2.0).resolution(None), (2560, 1600));
        // 1440p desktop at 100%
        assert_eq!(on(2560, 1440, 1.0).resolution(None), (1920, 1200));
    }

    #[test]
    fn saved_resolution_falls_back_when_too_big() {
        let laptop = on(1366, 768, 1.0);
        assert_eq!(laptop.choices(), vec![(1280, 720)]);
        assert_eq!(laptop.resolution(Some((2560, 1440))), (1280, 720));
        assert_eq!(on(2560, 1440, 1.0).resolution(Some((1600, 900))), (1600, 900));
    }
}
