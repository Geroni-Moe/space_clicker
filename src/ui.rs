//! All Bevy UI: HUD, wave/boss bar, ability bar and flagship hull bar, the
//! shop panel with its four tabs, toasts, and the modal dialogs (faction
//! picker, warp-jump confirmation, death screen, pause menu).

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::FontSmoothing;
use bevy::ui::Pressed;
use bevy::ui::VisualBox;
use bevy::ui::widget::NodeImageMode;
use bevy::ui_widgets::{Activate, Button};

use crate::art::{Art, flagship_sheet, fleet_sheet, portrait, portrait_size};
use crate::audio::Sfx;
use crate::battle::{Battle, Enemy};
use crate::economy::{
    Ability, DEATH_SHARD_SHARE, FACTION_NAMES, HULL_MULT, HULL_NAMES, JUMP_MIN_SECTOR, LOGI_BASE, LOGI_CLASSES, Race, Research,
    SHIP_CLASSES, System, WAVES_PER_SECTOR, faction, fmt, fmt_time, sector_name, ship_count,
};
use crate::fleet::FleetShip;
use crate::save::{Saves, persisted};
use crate::survival::{Death, Restart, Retreat, Vitals};
use crate::{AppState, BAR_H, Game, HUD_H, PANEL_W};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .add_message::<Toast>()
            .add_observer(on_activate)
            .add_systems(OnEnter(AppState::Playing), (reset_ui, spawn_hud, spawn_panel, spawn_modals).chain())
            .add_systems(
                Update,
                (
                    style_buttons,
                    update_labels,
                    update_hud,
                    update_wave_bar,
                    update_abilities,
                    update_hull_bar,
                    update_pages,
                    update_icons,
                    show_toasts,
                    warp_sequence,
                    pause_key,
                )
                    .run_if(in_state(AppState::Playing)),
            )
            .add_systems(Update, loading_screen.run_if(in_state(AppState::Loading)));
    }
}

// ---------------------------------------------------------------------------
// State, actions, styling helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Modal {
    ConfirmJump,
    Death,
    Pause,
}

#[derive(Resource)]
pub struct UiState {
    pub tab: usize,
    /// Index into BUY_MODES.
    pub buy: usize,
    pub modal: Option<Modal>,
    /// Remaining seconds of the warp-out animation.
    warp: Option<f32>,
}

impl Default for UiState {
    fn default() -> Self {
        Self { tab: 1, buy: 0, modal: None, warp: None }
    }
}

const BUY_MODES: [(&str, Option<u32>); 4] = [("x1", Some(1)), ("x10", Some(10)), ("x100", Some(100)), ("MAX", None)];
const TABS: [&str; 4] = ["Ship", "Fleet", "Logi", "Tech"];
const TECH_TAB: usize = 3;

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum Action {
    Tab(usize),
    BuyMode(usize),
    System(System),
    Hull,
    Class(usize),
    /// A logi class, 0..LOGI_CLASSES.len().
    Logi(usize),
    Research(Research),
    Ability(Ability),
    Retreat,
    Restart,
    Sound,
    /// Open the pause menu (HUD button).
    OpenMenu,
    Resume,
    SaveGame,
    QuitToTitle,
    ExitGame,
    Jump,
    ConfirmJump,
    CancelJump,
    PickRace(Race),
}

#[derive(Clone, Copy, PartialEq)]
enum Look {
    Plain,
    Ready,
    Off,
    Gold,
}

pub(crate) const WHITE: Color = Color::srgb(0.93, 0.95, 1.0);
pub(crate) const DIM: Color = Color::srgb(0.55, 0.6, 0.75);
pub(crate) const GOLD: Color = Color::srgb(1.0, 0.82, 0.36);
pub(crate) const CYAN: Color = Color::srgb(0.45, 0.88, 1.0);
pub(crate) const RED: Color = Color::srgb(1.0, 0.38, 0.38);
pub(crate) const PURPLE: Color = Color::srgb(0.78, 0.6, 1.0);
pub(crate) const GREEN: Color = Color::srgb(0.43, 0.94, 0.59);

pub(crate) fn slicer(border: f32) -> NodeImageMode {
    NodeImageMode::Sliced(TextureSlicer {
        border: BorderRect::all(border),
        center_scale_mode: SliceScaleMode::Stretch,
        sides_scale_mode: SliceScaleMode::Stretch,
        max_corner_scale: 1.0,
    })
}

pub(crate) fn chrome(art: &Art, name: &str, border: f32) -> ImageNode {
    // frames cover the whole border box, not just the content inside the padding
    ImageNode { image: art.ui(name), image_mode: slicer(border), visual_box: VisualBox::BorderBox, ..default() }
}

/// Pixel font text (Silkscreen at multiples of 8 stays crisp).
pub(crate) fn label(art: &Art, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont { font: art.font.clone().into(), font_size: FontSize::Px(size), font_smoothing: FontSmoothing::None, ..default() },
        TextColor(color),
    )
}

pub(crate) fn bold(art: &Art, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont { font: art.font_bold.clone().into(), font_size: FontSize::Px(size), font_smoothing: FontSmoothing::None, ..default() },
        TextColor(color),
    )
}

/// Smaller body copy.
pub(crate) fn body(art: &Art, s: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(s),
        TextFont { font: art.font_body.clone().into(), font_size: FontSize::Px(size), ..default() },
        TextColor(color),
    )
}

pub(crate) fn icon(art: &Art, name: &str, size: f32) -> impl Bundle {
    (ImageNode::new(art.ui(name)), Node { width: px(size), height: px(size), flex_shrink: 0.0, ..default() })
}

fn button(art: &Art, action: Action, node: Node) -> impl Bundle {
    (Button, Hovered::default(), action, chrome(art, "button", 12.0), node)
}

/// Text that is recomputed every frame from game state.
#[derive(Component, Clone, Copy, PartialEq)]
enum Live {
    Credits,
    Rates,
    Sector,
    Region,
    Shards,
    WaveLabel,
    BossTimer,
    Faction,
    Effects,
    AbilityTip,
    ShardInfo,
    JumpInfo,
    Stats,
    PickerTitle,
    DeathText,
    HullText,
    HullRates,
    Row(Action, Field),
    AbilityKey(Ability),
}

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Title,
    Desc,
    Qty,
    Cost,
}

// ---------------------------------------------------------------------------
// Loading screen
// ---------------------------------------------------------------------------

#[derive(Component)]
struct LoadingText;

fn loading_screen(mut commands: Commands, q: Query<(), With<LoadingText>>) {
    if q.is_empty() {
        commands.spawn((
            Text::new("Loading..."),
            TextColor(DIM),
            Node { position_type: PositionType::Absolute, left: px(24), top: px(24), ..default() },
            LoadingText,
            DespawnOnExit(AppState::Loading),
        ));
    }
}

// ---------------------------------------------------------------------------
// HUD
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Pip(u32);
#[derive(Component)]
struct WaveBar;
#[derive(Component)]
struct WaveFill;
#[derive(Component)]
struct Shade(Ability);
#[derive(Component)]
struct ShardBox;
#[derive(Component)]
struct ToastBox;
#[derive(Component)]
struct ToastTitle;
#[derive(Component)]
struct ToastSub;
#[derive(Component)]
struct SoundIcon;
#[derive(Component)]
struct Flash;
#[derive(Component)]
struct HullBox;
#[derive(Component)]
struct HullFill;

fn spawn_hud(mut commands: Commands, art: Res<Art>) {
    // top strip
    commands
        .spawn((
            DespawnOnExit(AppState::Playing),
            chrome(&art, "panel", 16.0),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(PANEL_W),
                top: px(0),
                height: px(HUD_H),
                padding: UiRect::axes(px(18), px(10)),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
        ))
        .with_children(|p| {
            // credits + rates
            p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(4), min_width: px(220), ..default() }).with_children(|p| {
                p.spawn(Node { align_items: AlignItems::Center, column_gap: px(8), ..default() }).with_children(|p| {
                    p.spawn(icon(&art, "icon_credits", 32.0));
                    p.spawn((bold(&art, "0", 24.0, GOLD), Live::Credits));
                });
                p.spawn((body(&art, "", 14.0, DIM), Live::Rates));
            });
            // sector + wave pips
            p.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(4), ..default() })
                .with_children(|p| {
                    p.spawn(Node { align_items: AlignItems::Baseline, column_gap: px(10), ..default() }).with_children(|p| {
                        p.spawn((bold(&art, "", 16.0, WHITE), Live::Sector));
                        p.spawn((body(&art, "", 14.0, DIM), Live::Region));
                    });
                    p.spawn(Node { column_gap: px(4), align_items: AlignItems::Center, ..default() }).with_children(|p| {
                        for i in 1..=WAVES_PER_SECTOR {
                            let size = if i == WAVES_PER_SECTOR { 14.0 } else { 10.0 };
                            p.spawn((
                                Node { width: px(size + if i == WAVES_PER_SECTOR { 4.0 } else { 6.0 }), height: px(size), ..default() },
                                BackgroundColor(Color::srgb(0.12, 0.15, 0.25)),
                                Pip(i),
                            ));
                        }
                    });
                });
            // shards + sound
            p.spawn(Node { align_items: AlignItems::Center, column_gap: px(12), min_width: px(150), justify_content: JustifyContent::FlexEnd, ..default() })
                .with_children(|p| {
                    p.spawn((Node { align_items: AlignItems::Center, column_gap: px(6), ..default() }, ShardBox)).with_children(|p| {
                        p.spawn(icon(&art, "icon_shard", 24.0));
                        p.spawn((bold(&art, "", 16.0, PURPLE), Live::Shards));
                    });
                    p.spawn(button(&art, Action::Sound, Node { width: px(40), height: px(40), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                        .with_children(|p| {
                            p.spawn((icon(&art, "icon_sound_on", 24.0), SoundIcon));
                        });
                    p.spawn(button(&art, Action::OpenMenu, Node { width: px(40), height: px(40), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                        .with_children(|p| {
                            p.spawn(icon(&art, "icon_menu", 24.0));
                        });
                });
        });

    // wave / boss bar over the battlefield
    commands
        .spawn((
            DespawnOnExit(AppState::Playing),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(PANEL_W),
                top: px(HUD_H + 10.0),
                justify_content: JustifyContent::Center,
                ..default()
            },
            Pickable::IGNORE,
        ))
        .with_children(|p| {
            p.spawn((Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, row_gap: px(4), ..default() }, WaveBar))
                .with_children(|p| {
                    p.spawn(Node { column_gap: px(12), align_items: AlignItems::Center, ..default() }).with_children(|p| {
                        p.spawn((bold(&art, "", 16.0, WHITE), Live::WaveLabel));
                        p.spawn((bold(&art, "", 16.0, RED), Live::BossTimer));
                    });
                    p.spawn((chrome(&art, "bar", 6.0), Node { width: px(360), height: px(14), padding: UiRect::all(px(3)), ..default() }))
                        .with_children(|p| {
                            p.spawn((Node { width: percent(100), height: percent(100), ..default() }, BackgroundColor(RED), WaveFill));
                        });
                });
        });

    // ability bar
    commands
        .spawn((
            DespawnOnExit(AppState::Playing),
            Node {
                position_type: PositionType::Absolute,
                left: px(12),
                bottom: px(8),
                height: px(BAR_H - 12.0),
                align_items: AlignItems::Center,
                column_gap: px(8),
                ..default()
            },
        ))
        .with_children(|p| {
            for (a, img) in [(Ability::Overcharge, "icon_overcharge"), (Ability::Broadside, "icon_broadside"), (Ability::SalvageSweep, "icon_salvage")] {
                p.spawn(button(&art, Action::Ability(a), Node { width: px(52), height: px(52), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                    .with_children(|p| {
                        p.spawn(icon(&art, img, 32.0));
                        p.spawn((
                            Node { position_type: PositionType::Absolute, left: px(4), right: px(4), bottom: px(4), height: percent(0), ..default() },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.05, 0.7)),
                            Shade(a),
                        ));
                        p.spawn((
                            Node { position_type: PositionType::Absolute, right: px(6), bottom: px(3), ..default() },
                            bold(&art, "", 16.0, WHITE),
                            Live::AbilityKey(a),
                        ));
                    });
            }
            p.spawn(button(&art, Action::Retreat, Node { width: px(52), height: px(52), margin: UiRect::left(px(8)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                .with_children(|p| {
                    p.spawn(icon(&art, "icon_retreat", 32.0));
                    p.spawn((Node { position_type: PositionType::Absolute, right: px(6), bottom: px(3), ..default() }, bold(&art, "R", 16.0, WHITE)));
                });
            p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(2), margin: UiRect::left(px(6)), ..default() }).with_children(|p| {
                p.spawn((body(&art, "", 14.0, WHITE), Live::AbilityTip));
                p.spawn((body(&art, "", 14.0, GOLD), Live::Effects));
            });
        });

    // flagship hull bar, bottom right of the battlefield
    commands
        .spawn((
            DespawnOnExit(AppState::Playing),
            Node {
                position_type: PositionType::Absolute,
                right: px(PANEL_W + 14.0),
                bottom: px(10),
                height: px(BAR_H - 16.0),
                align_items: AlignItems::Center,
                column_gap: px(10),
                ..default()
            },
            HullBox,
        ))
        .with_children(|p| {
            p.spawn(icon(&art, "icon_plating", 32.0));
            p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(4), ..default() }).with_children(|p| {
                p.spawn(Node { justify_content: JustifyContent::SpaceBetween, column_gap: px(12), ..default() }).with_children(|p| {
                    p.spawn((bold(&art, "", 16.0, WHITE), Live::HullText));
                    p.spawn((body(&art, "", 14.0, DIM), Live::HullRates));
                });
                p.spawn((chrome(&art, "bar", 6.0), Node { width: px(250), height: px(14), padding: UiRect::all(px(3)), ..default() }))
                    .with_children(|p| {
                        p.spawn((Node { width: percent(100), height: percent(100), ..default() }, BackgroundColor(GREEN), HullFill));
                    });
            });
        });

    // toast
    commands
        .spawn((
            DespawnOnExit(AppState::Playing),
            Node {
                position_type: PositionType::Absolute,
                left: px(0),
                right: px(PANEL_W),
                top: percent(26),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(6),
                ..default()
            },
            ToastBox,
            Visibility::Hidden,
        ))
        .with_children(|p| {
            p.spawn((bold(&art, "", 32.0, WHITE), TextShadow::default(), ToastTitle));
            p.spawn((label(&art, "", 16.0, DIM), TextShadow::default(), ToastSub));
        });

    // warp flash
    commands.spawn((
        DespawnOnExit(AppState::Playing),
        Node { position_type: PositionType::Absolute, width: percent(100), height: percent(100), ..default() },
        BackgroundColor(Color::srgba(0.85, 0.8, 1.0, 0.0)),
        GlobalZIndex(50),
        Pickable::IGNORE,
        Flash,
    ));
}

// ---------------------------------------------------------------------------
// Shop panel
// ---------------------------------------------------------------------------

#[derive(Component)]
struct Page(usize);
#[derive(Component)]
struct BuyRow;
/// A ship image in the UI that follows the chosen race.
#[derive(Component)]
enum ShipIcon {
    Class(usize),
    NextHull,
}

fn spawn_panel(mut commands: Commands, art: Res<Art>) {
    commands
        .spawn((
            DespawnOnExit(AppState::Playing),
            chrome(&art, "panel", 16.0),
            Node {
                position_type: PositionType::Absolute,
                right: px(0),
                top: px(0),
                bottom: px(0),
                width: px(PANEL_W),
                flex_direction: FlexDirection::Column,
                padding: UiRect::axes(px(20), px(18)),
                row_gap: px(8),
                ..default()
            },
        ))
        .with_children(|p| {
            p.spawn(Node { justify_content: JustifyContent::SpaceBetween, align_items: AlignItems::Baseline, ..default() }).with_children(|p| {
                p.spawn(bold(&art, "VOID ARMADA", 24.0, CYAN));
                p.spawn((body(&art, "", 14.0, DIM), Live::Faction));
            });
            // tabs
            p.spawn(Node { column_gap: px(6), ..default() }).with_children(|p| {
                for (i, name) in TABS.iter().enumerate() {
                    p.spawn(button(&art, Action::Tab(i), Node { flex_grow: 1.0, flex_basis: px(0), min_width: px(0), height: px(40), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                        .with_children(|p| {
                            p.spawn(label(&art, *name, 16.0, WHITE));
                        });
                }
            });
            // buy mode
            p.spawn((Node { column_gap: px(6), align_items: AlignItems::Center, ..default() }, BuyRow)).with_children(|p| {
                for (i, (name, _)) in BUY_MODES.iter().enumerate() {
                    p.spawn(button(&art, Action::BuyMode(i), Node { flex_grow: 1.0, flex_basis: px(0), min_width: px(0), height: px(32), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                        .with_children(|p| {
                            p.spawn(label(&art, *name, 16.0, WHITE));
                        });
                }
            });

            // flagship page
            p.spawn((page_node(), Page(0))).with_children(|p| {
                row(p, &art, Action::Hull, RowIcon::Ship(ShipIcon::NextHull));
                for (s, img) in [
                    (System::Laser, "icon_click"),
                    (System::Cannons, "icon_dps"),
                    (System::Crit, "icon_crit"),
                    (System::Drones, "icon_drone"),
                    (System::Sync, "icon_sync"),
                ] {
                    row(p, &art, Action::System(s), RowIcon::Ui(img));
                }
            });
            // fleet page
            p.spawn((page_node(), Page(1))).with_children(|p| {
                for c in 0..SHIP_CLASSES.len() {
                    row(p, &art, Action::Class(c), RowIcon::Ship(ShipIcon::Class(c)));
                }
            });
            // logistics page
            p.spawn((page_node(), Page(2))).with_children(|p| {
                for (s, img) in [(System::Plating, "icon_plating"), (System::Repairs, "icon_repair")] {
                    row(p, &art, Action::System(s), RowIcon::Ui(img));
                }
                for c in 0..LOGI_CLASSES.len() {
                    row(p, &art, Action::Logi(c), RowIcon::Ship(ShipIcon::Class(LOGI_BASE + c)));
                }
            });
            // research page
            p.spawn((page_node(), Page(TECH_TAB))).with_children(|p| {
                p.spawn(Node { align_items: AlignItems::Center, column_gap: px(8), margin: UiRect::bottom(px(2)), ..default() }).with_children(|p| {
                    p.spawn(icon(&art, "icon_shard", 32.0));
                    p.spawn((body(&art, "", 14.0, PURPLE), Live::ShardInfo));
                });
                p.spawn((
                    Button,
                    Hovered::default(),
                    Action::Jump,
                    chrome(&art, "button_gold", 12.0),
                    Node { width: percent(100), height: px(76), padding: UiRect::axes(px(12), px(8)), column_gap: px(12), align_items: AlignItems::Center, margin: UiRect::bottom(px(6)), ..default() },
                ))
                .with_children(|p| {
                    p.spawn(icon(&art, "icon_warp", 40.0));
                    p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(4), ..default() }).with_children(|p| {
                        p.spawn(bold(&art, "WARP JUMP", 16.0, GOLD));
                        p.spawn((body(&art, "", 14.0, WHITE), Live::JumpInfo));
                    });
                });
                for r in Research::ALL {
                    row(p, &art, Action::Research(r), RowIcon::Ui("icon_research"));
                }
                p.spawn((body(&art, "", 14.0, DIM), Live::Stats, Node { margin: UiRect::top(px(6)), ..default() }));
            });
        });
}

fn page_node() -> Node {
    Node { flex_direction: FlexDirection::Column, row_gap: px(6), display: Display::None, ..default() }
}

enum RowIcon {
    Ui(&'static str),
    Ship(ShipIcon),
}

fn row(p: &mut ChildSpawnerCommands, art: &Art, action: Action, ic: RowIcon) {
    p.spawn(button(
        art,
        action,
        Node { width: percent(100), height: px(62), padding: UiRect::axes(px(10), px(6)), column_gap: px(10), align_items: AlignItems::Center, ..default() },
    ))
    .with_children(|p| {
        p.spawn(Node { width: px(56), height: px(44), justify_content: JustifyContent::Center, align_items: AlignItems::Center, flex_shrink: 0.0, ..default() })
            .with_children(|p| match ic {
                RowIcon::Ui(name) => {
                    p.spawn(icon(art, name, 32.0));
                }
                RowIcon::Ship(kind) => {
                    p.spawn((art.ui_image("portrait_fleet_tidewall_interceptor"), Node::default(), kind));
                }
            });
        p.spawn(Node { flex_direction: FlexDirection::Column, row_gap: px(4), flex_grow: 1.0, min_width: px(0), ..default() }).with_children(|p| {
            p.spawn((label(art, "", 16.0, WHITE), Live::Row(action, Field::Title)));
            p.spawn((body(art, "", 14.0, DIM), Live::Row(action, Field::Desc)));
        });
        p.spawn(Node { flex_direction: FlexDirection::Column, align_items: AlignItems::FlexEnd, row_gap: px(4), min_width: px(84), flex_shrink: 0.0, ..default() })
            .with_children(|p| {
                p.spawn((body(art, "", 14.0, CYAN), Live::Row(action, Field::Qty)));
                p.spawn((bold(art, "", 16.0, GOLD), Live::Row(action, Field::Cost)));
            });
    });
}

// ---------------------------------------------------------------------------
// Modals
// ---------------------------------------------------------------------------

#[derive(Component)]
struct RacePicker;
#[derive(Component)]
struct ModalBox(Modal);

fn overlay() -> impl Bundle {
    (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_direction: FlexDirection::Column,
            display: Display::None,
            ..default()
        },
        BackgroundColor(Color::srgba(0.01, 0.01, 0.03, 0.93)),
        GlobalZIndex(20),
        DespawnOnExit(AppState::Playing),
    )
}

fn spawn_modals(mut commands: Commands, art: Res<Art>) {
    // faction picker
    commands.spawn((overlay(), RacePicker)).with_children(|p| {
        p.spawn(bold(&art, "VOID ARMADA", 48.0, CYAN));
        p.spawn((label(&art, "", 16.0, DIM), Live::PickerTitle, Node { margin: UiRect::new(px(0), px(0), px(8), px(28)), ..default() }));
        p.spawn(Node { column_gap: px(16), ..default() }).with_children(|p| {
            for race in Race::ALL {
                let d = race.def();
                p.spawn(button(
                    &art,
                    Action::PickRace(race),
                    Node {
                        width: px(252),
                        height: px(330),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        padding: UiRect::all(px(16)),
                        row_gap: px(8),
                        ..default()
                    },
                ))
                .with_children(|p| {
                    p.spawn(Node { height: px(110), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }).with_children(|p| {
                        let name = portrait(&flagship_sheet(race, 3));
                        let size = portrait_size(art.size(&name), Vec2::new(196.0, 100.0));
                        p.spawn((art.ui_image(&name), Node { width: px(size.x), height: px(size.y), ..default() }));
                    });
                    p.spawn(bold(&art, d.name, 16.0, WHITE));
                    p.spawn(label(&art, d.doctrine, 16.0, CYAN));
                    p.spawn(Node { height: px(6), ..default() });
                    for perk in d.perks {
                        p.spawn(body(&art, perk, 14.0, GOLD));
                    }
                    p.spawn(Node { flex_grow: 1.0, ..default() });
                    p.spawn(body(&art, "Click to take command", 14.0, DIM));
                });
            }
        });
    });

    // jump confirmation
    commands.spawn((overlay(), ModalBox(Modal::ConfirmJump))).with_children(|p| {
        p.spawn((chrome(&art, "panel", 16.0), Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(px(28)), row_gap: px(16), width: px(500), ..default() }))
            .with_children(|p| {
                p.spawn(bold(&art, "INITIATE WARP JUMP?", 16.0, GOLD));
                p.spawn((body(&art, "", 16.0, WHITE), Live::JumpInfo, TextLayout::justify(Justify::Center)));
                p.spawn(body(
                    &art,
                    "Your credits, fleet and flagship refits are left behind. Shards and research carry over, and you may pick a new faction.",
                    14.0,
                    DIM,
                ));
                p.spawn(Node { column_gap: px(12), ..default() }).with_children(|p| {
                    for (a, text) in [(Action::CancelJump, "Stay"), (Action::ConfirmJump, "Jump")] {
                        p.spawn(button(&art, a, Node { width: px(150), height: px(44), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                            .with_children(|p| {
                                p.spawn(label(&art, text, 16.0, WHITE));
                            });
                    }
                });
            });
    });

    // pause menu
    commands.spawn((overlay(), ModalBox(Modal::Pause))).with_children(|p| {
        p.spawn((chrome(&art, "panel", 16.0), Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(px(28)), row_gap: px(12), width: px(360), ..default() }))
            .with_children(|p| {
                p.spawn((bold(&art, "PAUSED", 24.0, CYAN), Node { margin: UiRect::bottom(px(8)), ..default() }));
                for (a, text) in [
                    (Action::Resume, "Resume"),
                    (Action::SaveGame, "Save"),
                    (Action::QuitToTitle, "Save & Quit to Title"),
                    (Action::ExitGame, "Save & Exit"),
                ] {
                    p.spawn(button(&art, a, Node { width: percent(100), height: px(48), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                        .with_children(|p| {
                            p.spawn(label(&art, text, 16.0, WHITE));
                        });
                }
                p.spawn(Node { margin: UiRect::top(px(4)), ..default() }).with_children(|p| crate::view::display_rows(p, &art));
                p.spawn((body(&art, "Esc to resume", 14.0, DIM), Node { margin: UiRect::top(px(4)), ..default() }));
            });
    });

    // flagship destroyed
    commands.spawn((overlay(), ModalBox(Modal::Death))).with_children(|p| {
        p.spawn((chrome(&art, "panel", 16.0), Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(px(28)), row_gap: px(16), width: px(520), ..default() }))
            .with_children(|p| {
                p.spawn(bold(&art, "FLAGSHIP DESTROYED", 24.0, RED));
                p.spawn((body(&art, "", 16.0, WHITE), Live::DeathText, TextLayout::justify(Justify::Center)));
                p.spawn(body(&art, "Research and banked shards carry over. Bring logi next time.", 14.0, DIM));
                p.spawn(button(&art, Action::Restart, Node { width: px(180), height: px(44), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                    .with_children(|p| {
                        p.spawn(label(&art, "Start over", 16.0, WHITE));
                    });
            });
    });

}

// ---------------------------------------------------------------------------
// Interaction
// ---------------------------------------------------------------------------

fn on_activate(
    ev: On<Activate>,
    actions: Query<&Action>,
    mut game: ResMut<Game>,
    mut ui: ResMut<UiState>,
    mut sfx: MessageWriter<Sfx>,
    mut retreat: MessageWriter<Retreat>,
    mut restart: MessageWriter<Restart>,
    mut saves: ResMut<Saves>,
    mut toast: MessageWriter<Toast>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
) {
    let Ok(action) = actions.get(ev.entity) else { return };
    let want = BUY_MODES[ui.buy].1;
    let ok = match *action {
        Action::Tab(i) => {
            ui.tab = i;
            sfx.write(Sfx("tab"));
            return;
        }
        Action::BuyMode(i) => {
            ui.buy = i;
            sfx.write(Sfx("tab"));
            return;
        }
        Action::System(s) => game.state.buy_system(s, want),
        Action::Hull => game.state.buy_hull(),
        Action::Class(c) => game.state.buy_class(c, want),
        Action::Logi(c) => game.state.buy_logi(c, want),
        Action::Research(r) => game.state.buy_research(r),
        Action::Retreat => {
            if game.in_battle() {
                retreat.write(Retreat);
            } else {
                sfx.write(Sfx("deny"));
            }
            return;
        }
        Action::Restart => {
            restart.write(Restart);
            sfx.write(Sfx("tab"));
            return;
        }
        Action::Ability(a) => {
            let ok = game.activate(a);
            sfx.write(Sfx(if ok { "ability" } else { "deny" }));
            return;
        }
        Action::Sound => {
            game.state.muted = !game.state.muted;
            return;
        }
        Action::Jump => {
            let ok = game.state.jump_reward() > 0.0 && ui.warp.is_none();
            if ok {
                ui.modal = Some(Modal::ConfirmJump);
                sfx.write(Sfx("tab"));
            } else {
                sfx.write(Sfx("deny"));
            }
            return;
        }
        Action::CancelJump => {
            ui.modal = None;
            sfx.write(Sfx("tab"));
            return;
        }
        Action::ConfirmJump => {
            ui.modal = None;
            ui.warp = Some(1.6);
            game.warping = true;
            sfx.write(Sfx("warp"));
            return;
        }
        Action::OpenMenu => {
            if ui.modal.is_none() && ui.warp.is_none() && !game.dead {
                ui.modal = Some(Modal::Pause);
                sfx.write(Sfx("tab"));
            }
            return;
        }
        Action::Resume => {
            ui.modal = None;
            sfx.write(Sfx("tab"));
            return;
        }
        Action::SaveGame => {
            saves.save(&persisted(&game));
            toast.write(Toast::small("Game saved".into(), format!("Slot {}", saves.active.map_or(0, |s| s + 1))));
            sfx.write(Sfx("buy"));
            return;
        }
        Action::QuitToTitle => {
            saves.save(&persisted(&game));
            saves.active = None;
            next.set(AppState::Menu);
            sfx.write(Sfx("tab"));
            return;
        }
        Action::ExitGame => {
            saves.save(&persisted(&game));
            exit.write(AppExit::Success);
            return;
        }
        Action::PickRace(r) => {
            game.state.race = r;
            game.state.race_chosen = true;
            saves.save(&game.state);
            sfx.write(Sfx("sector"));
            return;
        }
    };
    sfx.write(Sfx(if ok { "buy" } else { "deny" }));
}

/// What a button should look like right now.
fn look(action: Action, game: &Game, ui: &UiState) -> Look {
    let s = &game.state;
    let want = BUY_MODES[ui.buy].1;
    let afford = |cost: f64, n: u32| if n > 0 && cost <= s.credits { Look::Ready } else { Look::Off };
    match action {
        Action::Tab(i) => if ui.tab == i { Look::Ready } else { Look::Plain },
        Action::BuyMode(i) => if ui.buy == i { Look::Ready } else { Look::Plain },
        Action::System(sys) => {
            if !s.system_unlocked(sys) {
                return Look::Off;
            }
            let (n, cost) = s.system_quote(sys, want);
            afford(cost, n)
        }
        Action::Hull => match s.hull_quote() {
            Some((req, cost)) if s.max_sector >= req && s.credits >= cost => Look::Gold,
            _ => Look::Off,
        },
        Action::Class(c) => {
            if !s.class_unlocked(c) {
                return Look::Off;
            }
            let (n, cost) = s.class_quote(c, want);
            afford(cost, n)
        }
        Action::Logi(c) => {
            if !s.logi_unlocked(c) {
                return Look::Off;
            }
            let (n, cost) = s.logi_quote(c, want);
            afford(cost, n)
        }
        Action::Retreat => if game.in_battle() { Look::Plain } else { Look::Off },
        Action::Research(r) => match s.research_cost(r) {
            Some(cost) if s.shards >= cost => Look::Ready,
            _ => Look::Off,
        },
        Action::Ability(a) => {
            let i = a.index();
            if game.active[i] > 0.0 {
                Look::Gold
            } else if s.ability_unlocked(a) && game.cooldown[i] <= 0.0 {
                Look::Ready
            } else {
                Look::Off
            }
        }
        Action::Jump => if s.jump_reward() > 0.0 { Look::Gold } else { Look::Off },
        Action::PickRace(r) => if s.jumps > 0 && s.race == r { Look::Ready } else { Look::Plain },
        _ => Look::Plain,
    }
}

fn style_buttons(
    art: Res<Art>,
    game: Res<Game>,
    ui: Res<UiState>,
    mut q: Query<(&Action, &Hovered, Has<Pressed>, &mut ImageNode), With<Button>>,
) {
    for (action, hovered, pressed, mut img) in &mut q {
        let l = look(*action, &game, &ui);
        let name = match (l, pressed, hovered.get()) {
            (Look::Off, _, _) => "button_disabled",
            (_, true, _) => "button_pressed",
            (Look::Gold, _, _) => "button_gold",
            (_, false, true) => "button_hover",
            (Look::Ready, _, _) => "button_ready",
            (Look::Plain, _, _) => "button",
        };
        let handle = art.ui(name);
        if img.image != handle {
            img.image = handle;
        }
    }
}

// ---------------------------------------------------------------------------
// Per-frame text
// ---------------------------------------------------------------------------

fn row_text(action: Action, field: Field, game: &Game, ui: &UiState) -> String {
    let s = &game.state;
    let want = BUY_MODES[ui.buy].1;
    match action {
        Action::System(sys) => {
            let d = sys.def();
            let lvl = s.system_level(sys);
            let unlocked = s.system_unlocked(sys);
            match field {
                Field::Title => d.name.into(),
                Field::Desc if !unlocked => format!("Unlocks at sector {}", d.unlock_sector),
                Field::Desc => match sys {
                    System::Laser => format!("Click damage {}", fmt(s.click_damage())),
                    System::Cannons => format!("Flagship DPS {}", fmt(s.cannon_dps())),
                    System::Crit => format!("{:.0}% crit for x{:.2}", s.crit_chance() * 100.0, s.crit_mult()),
                    System::Drones => format!("{} auto-shots per second", fmt(s.drone_rate())),
                    System::Sync => format!("Clicks add {:.0}% of fleet DPS", s.sync_share() * 100.0),
                    System::Plating => format!("Flagship hull {}", fmt(s.flagship_max_hp())),
                    System::Repairs => format!("Flagship repairs {}/s", fmt(s.flagship_rep())),
                },
                Field::Qty | Field::Cost if !unlocked || lvl >= d.max_level => {
                    match field {
                        Field::Cost if unlocked => "MAX".into(),
                        Field::Qty if unlocked => format!("Lv {lvl}"),
                        _ => String::new(),
                    }
                }
                Field::Qty => format!("Lv {lvl}  +{}", s.system_quote(sys, want).0),
                Field::Cost => fmt(s.system_quote(sys, want).1.ceil()),
            }
        }
        Action::Hull => {
            let next = (s.hull as usize).min(5);
            match (field, s.hull_quote()) {
                (Field::Title, Some(_)) => format!("Refit: {}", HULL_NAMES[next]),
                (Field::Title, None) => format!("{} class", HULL_NAMES[5]),
                (Field::Desc, Some((req, _))) if s.max_sector < req => format!("Needs sector {req}"),
                (Field::Desc, Some(_)) => format!("New hull, x{HULL_MULT:.0} all damage"),
                (Field::Desc, None) => "Ultimate flagship hull".into(),
                (Field::Qty, _) => format!("Tier {}", s.hull),
                (Field::Cost, Some((_, cost))) => fmt(cost.ceil()),
                (Field::Cost, None) => "MAX".into(),
            }
        }
        Action::Class(c) => {
            let cls = &SHIP_CLASSES[c];
            let lvl = s.fleet[c];
            if !s.class_unlocked(c) {
                return match field {
                    Field::Title => "???".into(),
                    Field::Desc => format!("Unlocks at sector {}", cls.unlock_sector),
                    _ => String::new(),
                };
            }
            match field {
                Field::Title => {
                    let n = ship_count(c, lvl);
                    if n > 1 { format!("{} x{n}", cls.name) } else { cls.name.to_string() }
                }
                Field::Desc => {
                    if lvl == 0 {
                        cls.blurb.into()
                    } else {
                        let next = 25 - lvl % 25;
                        format!("DPS {}  (x2 in {next})", fmt(s.class_dps(c)))
                    }
                }
                Field::Qty => format!("Lv {lvl}  +{}", s.class_quote(c, want).0),
                Field::Cost => fmt(s.class_quote(c, want).1.ceil()),
            }
        }
        Action::Logi(c) => {
            let cls = &LOGI_CLASSES[c];
            let lvl = s.logi[c];
            if !s.logi_unlocked(c) {
                return match field {
                    Field::Title => "???".into(),
                    Field::Desc => format!("Unlocks at sector {}", cls.unlock_sector),
                    _ => String::new(),
                };
            }
            match field {
                Field::Title => {
                    let n = ship_count(LOGI_BASE + c, lvl);
                    if n > 1 { format!("{} x{n}", cls.name) } else { cls.name.to_string() }
                }
                Field::Desc => {
                    if lvl == 0 {
                        cls.blurb.into()
                    } else {
                        format!("Repairs {}/s  (x2 in {})", fmt(s.logi_rps(c)), 25 - lvl % 25)
                    }
                }
                Field::Qty => format!("Lv {lvl}  +{}", s.logi_quote(c, want).0),
                Field::Cost => fmt(s.logi_quote(c, want).1.ceil()),
            }
        }
        Action::Research(r) => {
            let d = r.def();
            let lvl = s.research_level(r);
            match field {
                Field::Title => d.name.into(),
                Field::Desc => d.desc.into(),
                Field::Qty => format!("Lv {lvl}  shards"),
                Field::Cost => s.research_cost(r).map(fmt).unwrap_or_else(|| "MAX".into()),
            }
        }
        _ => String::new(),
    }
}

/// What the HUD needs from the battlefield that isn't in the save.
struct Snapshot<'a> {
    battle: &'a Battle,
    hovered_ability: Option<Ability>,
    death: &'a Death,
    vitals: &'a Vitals,
    /// Flagship (hp, max hp), if it is on the field.
    flagship: Option<(f64, f64)>,
}

fn live_text(kind: Live, game: &Game, ui: &UiState, snap: &Snapshot) -> String {
    let (battle, hovered_ability) = (snap.battle, snap.hovered_ability);
    let s = &game.state;
    match kind {
        Live::Credits => fmt(s.credits.floor()),
        Live::Rates => {
            let b = game.boosts();
            format!("Fleet DPS {}   Click {}", fmt(s.fleet_dps() * b.dps), fmt(s.click_damage() * b.click))
        }
        Live::Sector => format!("SECTOR {}", s.sector),
        Live::Region => format!("{} - {}", sector_name(s.sector), FACTION_NAMES[faction(s.sector)]),
        Live::Shards => fmt(s.shards.floor()),
        Live::WaveLabel => {
            if s.wave >= WAVES_PER_SECTOR {
                crate::economy::boss_name(s.sector).to_uppercase()
            } else {
                format!("WAVE {}/{}", s.wave, WAVES_PER_SECTOR)
            }
        }
        Live::BossTimer => battle.boss_left.map(|t| format!("{:.1}s", t)).unwrap_or_default(),
        Live::Faction => if s.race_chosen { s.race.def().name.to_string() } else { String::new() },
        Live::Effects => {
            let mut parts = vec![];
            for a in Ability::ALL {
                let t = game.active[a.index()];
                if t > 0.0 {
                    parts.push(format!("{} {:.0}s", a.def().name, t.ceil()));
                }
            }
            parts.join("   ")
        }
        Live::AbilityTip => match hovered_ability {
            Some(a) => {
                let d = a.def();
                if s.ability_unlocked(a) {
                    format!("{}: {} for {:.0}s", d.name, d.desc, d.duration)
                } else {
                    format!("{}: unlocks at sector {}", d.name, d.unlock_sector)
                }
            }
            None => String::new(),
        },
        Live::ShardInfo => format!("{} shards, each +10% damage while unspent", fmt(s.shards)),
        Live::JumpInfo => {
            let reward = s.jump_reward();
            if reward > 0.0 {
                format!("Gain {} shards (deepest sector {})", fmt(reward), s.max_sector)
            } else {
                format!("Reach sector {JUMP_MIN_SECTOR} to earn shards")
            }
        }
        Live::Stats => format!(
            "Best sector {}   Jumps {}   Deaths {}\nKills {}   Clicks {}   Played {}",
            s.best_sector,
            s.jumps,
            s.deaths,
            s.kills,
            s.clicks,
            fmt_time(s.play_time)
        ),
        Live::PickerTitle => {
            if s.jumps > 0 {
                format!("Jump complete. {} shards aboard. Choose the faction you will command.", fmt(s.shards))
            } else {
                "Choose the faction you will command".into()
            }
        }
        Live::DeathText => {
            let d = snap.death;
            let share = if DEATH_SHARD_SHARE == 0.5 { "half".to_string() } else { format!("{:.0}% of", DEATH_SHARD_SHARE * 100.0) };
            format!(
                "Your flagship fell in sector {} (deepest {}).\nThe run is over. {} shards salvaged,\n{share} what a Warp Jump would pay.",
                d.sector,
                d.deepest,
                fmt(d.gained),
            )
        }
        Live::HullText => match snap.flagship {
            Some((hp, max)) => format!("HULL {} / {}", fmt(hp.max(0.0).ceil()), fmt(max.ceil())),
            None => String::new(),
        },
        Live::HullRates => format!("+{}/s  -{}/s", fmt(snap.vitals.repairs), fmt(snap.vitals.incoming)),
        Live::AbilityKey(a) => {
            let i = a.index();
            if !s.ability_unlocked(a) {
                "".into()
            } else if game.cooldown[i] > 0.0 && game.active[i] <= 0.0 {
                format!("{:.0}", game.cooldown[i].ceil())
            } else {
                format!("{}", i + 1)
            }
        }
        Live::Row(action, field) => row_text(action, field, game, ui),
    }
}

fn update_labels(
    game: Res<Game>,
    ui: Res<UiState>,
    battle: Res<Battle>,
    death: Res<Death>,
    vitals: Res<Vitals>,
    ships: Query<&FleetShip>,
    abilities: Query<(&Action, &Hovered)>,
    mut q: Query<(&Live, &mut Text)>,
) {
    let hovered = abilities.iter().find_map(|(a, h)| match a {
        Action::Ability(ab) if h.get() => Some(*ab),
        _ => None,
    });
    let flagship = ships.iter().find(|s| s.class.is_none()).map(|s| (s.hp, s.max_hp));
    let snap = Snapshot { battle: &battle, hovered_ability: hovered, death: &death, vitals: &vitals, flagship };
    for (kind, mut text) in &mut q {
        let s = live_text(*kind, &game, &ui, &snap);
        if text.0 != s {
            text.0 = s;
        }
    }
}

fn update_hud(
    art: Res<Art>,
    game: Res<Game>,
    mut pips: Query<(&Pip, &mut BackgroundColor)>,
    mut shard_box: Single<&mut Node, With<ShardBox>>,
    mut sound: Single<&mut ImageNode, With<SoundIcon>>,
) {
    let s = &game.state;
    for (pip, mut bg) in &mut pips {
        let c = if pip.0 < s.wave {
            Color::srgb(0.25, 0.75, 0.9)
        } else if pip.0 == s.wave {
            if pip.0 == WAVES_PER_SECTOR { RED } else { WHITE }
        } else if pip.0 == WAVES_PER_SECTOR {
            Color::srgb(0.4, 0.12, 0.15)
        } else {
            Color::srgb(0.12, 0.15, 0.25)
        };
        bg.0 = c;
    }
    let show = s.shards > 0.0 || s.jumps > 0;
    let d = if show { Display::Flex } else { Display::None };
    if shard_box.display != d {
        shard_box.display = d;
    }
    let want = art.ui(if s.muted { "icon_sound_off" } else { "icon_sound_on" });
    if sound.image != want {
        sound.image = want;
    }
}

fn update_wave_bar(
    game: Res<Game>,
    enemies: Query<&Enemy>,
    mut bar: Single<&mut Visibility, With<WaveBar>>,
    mut fill: Single<(&mut Node, &mut BackgroundColor), With<WaveFill>>,
) {
    let (hp, max) = enemies.iter().filter(|e| !e.fleeing).fold((0.0, 0.0), |(h, m), e| (h + e.hp.max(0.0), m + e.max_hp));
    let vis = if max > 0.0 && game.in_battle() { Visibility::Inherited } else { Visibility::Hidden };
    if **bar != vis {
        **bar = vis;
    }
    let frac = if max > 0.0 { (hp / max) as f32 } else { 0.0 };
    fill.0.width = percent(frac * 100.0);
    fill.1.0 = if game.state.wave >= WAVES_PER_SECTOR { RED } else { Color::srgb(1.0, 0.55, 0.3) };
}

fn update_abilities(game: Res<Game>, mut shades: Query<(&Shade, &mut Node)>) {
    for (shade, mut node) in &mut shades {
        let a = shade.0;
        let i = a.index();
        let frac = if !game.state.ability_unlocked(a) {
            1.0
        } else if game.active[i] > 0.0 {
            0.0
        } else {
            game.cooldown[i] / (a.def().cooldown * game.state.cooldown_mult())
        };
        node.height = percent((frac * 100.0).clamp(0.0, 100.0));
    }
}

fn update_pages(
    ui: Res<UiState>,
    mut game: ResMut<Game>,
    mut pages: Query<(&Page, &mut Node), (Without<BuyRow>, Without<RacePicker>, Without<ModalBox>)>,
    mut buy_row: Single<&mut Node, (With<BuyRow>, Without<RacePicker>, Without<ModalBox>)>,
    mut picker: Single<&mut Node, (With<RacePicker>, Without<ModalBox>)>,
    mut modals: Query<(&ModalBox, &mut Node)>,
) {
    for (page, mut node) in &mut pages {
        let d = if page.0 == ui.tab { Display::Flex } else { Display::None };
        if node.display != d {
            node.display = d;
        }
    }
    let d = if ui.tab == TECH_TAB { Display::None } else { Display::Flex };
    if buy_row.display != d {
        buy_row.display = d;
    }
    let show_picker = !game.state.race_chosen && ui.warp.is_none() && ui.modal.is_none();
    let d = if show_picker { Display::Flex } else { Display::None };
    if picker.display != d {
        picker.display = d;
    }
    for (m, mut node) in &mut modals {
        let d = if ui.modal == Some(m.0) { Display::Flex } else { Display::None };
        if node.display != d {
            node.display = d;
        }
    }
    let paused = ui.modal.is_some();
    if game.paused != paused {
        game.paused = paused;
    }
}

fn update_hull_bar(
    time: Res<Time>,
    game: Res<Game>,
    ships: Query<&FleetShip>,
    mut hull: Single<&mut Visibility, With<HullBox>>,
    mut fill: Single<(&mut Node, &mut BackgroundColor), With<HullFill>>,
) {
    let flag = ships.iter().find(|s| s.class.is_none() && s.max_hp > 0.0);
    let vis = if flag.is_some() && game.state.race_chosen { Visibility::Inherited } else { Visibility::Hidden };
    if **hull != vis {
        **hull = vis;
    }
    let Some(flag) = flag else { return };
    let frac = (flag.hp / flag.max_hp).clamp(0.0, 1.0) as f32;
    fill.0.width = percent(frac * 100.0);
    // green, then amber, then a red that pulses when critical
    fill.1.0 = if frac > 0.5 {
        GREEN
    } else if frac > 0.3 {
        Color::srgb(1.0, 0.78, 0.31)
    } else {
        let pulse = 0.65 + 0.35 * (time.elapsed_secs() * 8.0).sin().abs();
        RED.mix(&WHITE, 1.0 - pulse)
    };
}

fn update_icons(art: Res<Art>, game: Res<Game>, mut q: Query<(&ShipIcon, &mut ImageNode, &mut Node)>) {
    let race = game.state.race;
    for (kind, mut img, mut node) in &mut q {
        let name = match kind {
            ShipIcon::Class(c) => fleet_sheet(race, *c),
            ShipIcon::NextHull => flagship_sheet(race, (game.state.hull + 1).min(6)),
        };
        let name = portrait(&name);
        let want = art.ui_image(&name);
        if img.image != want.image {
            *img = want;
            let size = portrait_size(art.size(&name), Vec2::new(56.0, 30.0));
            node.width = px(size.x);
            node.height = px(size.y);
        }
    }
}

// ---------------------------------------------------------------------------
// Toasts
// ---------------------------------------------------------------------------

#[derive(Message, Clone)]
pub struct Toast {
    title: String,
    sub: String,
    color: Color,
    secs: f32,
}

impl Toast {
    pub fn big(title: String, sub: String) -> Self {
        Self { title, sub, color: WHITE, secs: 2.4 }
    }

    pub fn boss(name: String) -> Self {
        Self { title: "WARNING".into(), sub: format!("{name} has arrived. Destroy it before it escapes!"), color: RED, secs: 2.6 }
    }

    pub fn small(title: String, sub: String) -> Self {
        Self { title, sub, color: GOLD, secs: 2.4 }
    }

    pub fn danger(title: String, sub: String) -> Self {
        Self { title, sub, color: RED, secs: 4.0 }
    }
}

#[derive(Default)]
struct ToastClock {
    left: f32,
    total: f32,
    color: Color,
}

fn show_toasts(
    time: Res<Time>,
    mut reader: MessageReader<Toast>,
    mut clock: Local<ToastClock>,
    mut boxv: Single<&mut Visibility, With<ToastBox>>,
    mut title: Single<(&mut Text, &mut TextColor), (With<ToastTitle>, Without<ToastSub>)>,
    mut sub: Single<(&mut Text, &mut TextColor), (With<ToastSub>, Without<ToastTitle>)>,
) {
    if let Some(t) = reader.read().last() {
        title.0.0 = t.title.clone();
        sub.0.0 = t.sub.clone();
        clock.left = t.secs;
        clock.total = t.secs;
        clock.color = t.color;
    }
    clock.left -= time.delta_secs();
    if clock.left <= 0.0 {
        **boxv = Visibility::Hidden;
        return;
    }
    **boxv = Visibility::Inherited;
    let a = (clock.left / 0.5).min(1.0).min((clock.total - clock.left) / 0.15 + 0.2).clamp(0.0, 1.0);
    title.1.0 = clock.color.with_alpha(a);
    sub.1.0 = DIM.with_alpha(a);
}

// ---------------------------------------------------------------------------
// Pause menu
// ---------------------------------------------------------------------------

/// Each run starts on the Fleet tab with no dialog open.
fn reset_ui(mut ui: ResMut<UiState>) {
    *ui = UiState::default();
}

/// Esc opens the pause menu, or closes whatever dialog can be dismissed.
fn pause_key(keys: Res<ButtonInput<KeyCode>>, game: Res<Game>, mut ui: ResMut<UiState>, mut sfx: MessageWriter<Sfx>) {
    if !keys.just_pressed(KeyCode::Escape) || ui.warp.is_some() {
        return;
    }
    ui.modal = match ui.modal {
        None if !game.dead => Some(Modal::Pause),
        Some(Modal::Pause) | Some(Modal::ConfirmJump) => None,
        other => other,
    };
    sfx.write(Sfx("tab"));
}

// ---------------------------------------------------------------------------
// Warp jump sequence
// ---------------------------------------------------------------------------

fn warp_sequence(
    mut commands: Commands,
    time: Res<Time>,
    mut ui: ResMut<UiState>,
    mut game: ResMut<Game>,
    mut battle: ResMut<Battle>,
    mut ships: Query<(Entity, &mut FleetShip)>,
    enemies: Query<Entity, With<Enemy>>,
    mut flash: Single<&mut BackgroundColor, With<Flash>>,
    saves: Res<Saves>,
) {
    let Some(left) = ui.warp else { return };
    let left = left - time.delta_secs();
    // ships accelerate away to the right, then everything flashes white
    for (_, mut ship) in &mut ships {
        ship.home.x = 900.0;
    }
    let flash_a = if left < 0.5 { 1.0 - (0.5 - left).max(0.0) / 0.5 } else { ((1.6 - left) / 1.1).clamp(0.0, 1.0) };
    flash.0 = flash.0.with_alpha(flash_a.clamp(0.0, 1.0));
    if left > 0.5 {
        ui.warp = Some(left);
        return;
    }
    if left > 0.0 && game.warping {
        // the jump itself: at peak white
        // the faction picker that follows announces the shards aboard
        game.state.warp_jump();
        for (e, _) in &ships {
            commands.entity(e).despawn();
        }
        for e in &enemies {
            commands.entity(e).despawn();
        }
        *battle = Battle::default();
        game.active = [0.0; 3];
        game.cooldown = [0.0; 3];
        game.warping = false;
        saves.save(&game.state);
    }
    if left <= 0.0 {
        flash.0 = flash.0.with_alpha(0.0);
        ui.warp = None;
    } else {
        ui.warp = Some(left);
    }
}
