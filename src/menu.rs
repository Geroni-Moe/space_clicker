//! Title screen: Continue, New Game, Load Game and Quit over the drifting
//! backdrop, with three save slots. Choosing a slot loads it (or starts it
//! fresh) and enters the game.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button};
use rand::Rng;

use crate::art::{Art, flagship_sheet, fleet_sheet, portrait, portrait_size};
use crate::audio::Sfx;
use crate::economy::{GameState, LOGI_BASE, Race, fmt};
use crate::fx::Anim;
use crate::save::{SLOTS, Saves, auto_slot, most_recent, now, time_ago};
use crate::ui::{CYAN, DIM, GOLD, PURPLE, WHITE, body, bold, chrome, label};
use crate::{AppState, Game, MainCamera};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MenuPage>()
            .insert_resource(MenuSlots::default())
            .insert_resource(AutoSlot(auto_slot()))
            .add_observer(on_menu_activate)
            .add_systems(OnEnter(AppState::Menu), (enter_menu, spawn_drifters))
            .add_systems(Update, (rebuild, style_menu, drift, back_key).chain().run_if(in_state(AppState::Menu)));
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Mode {
    New,
    Load,
}

#[derive(Resource, Default, Clone, Copy, PartialEq, Eq, Debug)]
enum MenuPage {
    #[default]
    Main,
    Slots(Mode),
    /// Confirm overwriting an occupied slot with a new game.
    Confirm(usize),
}

/// What's in each slot, read when the menu opens.
#[derive(Resource, Default)]
struct MenuSlots([Option<GameState>; SLOTS]);

/// Slot to jump into without showing the menu (test harness), used once.
#[derive(Resource)]
struct AutoSlot(Option<usize>);

#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
enum MenuAction {
    Continue,
    NewGame,
    LoadGame,
    Quit,
    Slot(usize),
    Back,
    Overwrite(usize),
}

#[derive(Component)]
struct MenuRoot;

/// Speed and exact x of a title-screen ship (drawn at whole pixels).
#[derive(Component)]
struct Drifter(f32, f32);

fn menu_button(art: &Art, action: MenuAction, node: Node) -> impl Bundle {
    (Button, Hovered::default(), action, chrome(art, "button", 12.0), node)
}

fn enter_menu(
    mut page: ResMut<MenuPage>,
    mut slots: ResMut<MenuSlots>,
    mut auto: ResMut<AutoSlot>,
    mut saves: ResMut<Saves>,
    mut game: ResMut<Game>,
    mut next: ResMut<NextState<AppState>>,
) {
    *page = MenuPage::Main;
    saves.active = None;
    slots.0 = saves.list();
    if let Some(slot) = auto.0.take() {
        let fresh = slots.0[slot].is_none();
        start_run(slot, fresh, &mut saves, &mut game, &mut next);
    }
}

/// Load a slot (or begin it fresh) with clean transient state, then play.
fn start_run(slot: usize, fresh: bool, saves: &mut Saves, game: &mut Game, next: &mut NextState<AppState>) {
    let state = if fresh { GameState::default() } else { saves.read(slot).unwrap_or_default() };
    *game = Game { state, ..default() };
    saves.active = Some(slot);
    if fresh {
        // claim the slot right away so it shows up even if the game closes early
        saves.save(&game.state);
    }
    next.set(AppState::Playing);
}

fn on_menu_activate(
    ev: On<Activate>,
    actions: Query<&MenuAction>,
    mut page: ResMut<MenuPage>,
    slots: Res<MenuSlots>,
    mut saves: ResMut<Saves>,
    mut game: ResMut<Game>,
    mut next: ResMut<NextState<AppState>>,
    mut exit: MessageWriter<AppExit>,
    mut sfx: MessageWriter<Sfx>,
) {
    let Ok(action) = actions.get(ev.entity) else { return };
    let mut sound = "tab";
    match *action {
        MenuAction::Continue => match most_recent(&slots.0) {
            Some(slot) => {
                start_run(slot, false, &mut saves, &mut game, &mut next);
                sound = "sector";
            }
            None => sound = "deny",
        },
        MenuAction::NewGame => *page = MenuPage::Slots(Mode::New),
        MenuAction::LoadGame => {
            if slots.0.iter().any(Option::is_some) {
                *page = MenuPage::Slots(Mode::Load);
            } else {
                sound = "deny";
            }
        }
        MenuAction::Quit => {
            exit.write(AppExit::Success);
        }
        MenuAction::Slot(i) => match (*page, slots.0[i].is_some()) {
            (MenuPage::Slots(Mode::New), true) => *page = MenuPage::Confirm(i),
            (MenuPage::Slots(Mode::New), false) => {
                start_run(i, true, &mut saves, &mut game, &mut next);
                sound = "sector";
            }
            (MenuPage::Slots(Mode::Load), true) => {
                start_run(i, false, &mut saves, &mut game, &mut next);
                sound = "sector";
            }
            _ => sound = "deny",
        },
        MenuAction::Overwrite(i) => {
            start_run(i, true, &mut saves, &mut game, &mut next);
            sound = "sector";
        }
        MenuAction::Back => *page = back(*page),
    }
    sfx.write(Sfx(sound));
}

fn back(page: MenuPage) -> MenuPage {
    match page {
        MenuPage::Confirm(_) => MenuPage::Slots(Mode::New),
        _ => MenuPage::Main,
    }
}

fn back_key(keys: Res<ButtonInput<KeyCode>>, mut page: ResMut<MenuPage>, mut sfx: MessageWriter<Sfx>) {
    if keys.just_pressed(KeyCode::Escape) && *page != MenuPage::Main {
        *page = back(*page);
        sfx.write(Sfx("tab"));
    }
}

/// Whether a menu button can be used right now.
fn enabled(action: MenuAction, page: MenuPage, slots: &MenuSlots) -> bool {
    match action {
        MenuAction::LoadGame => slots.0.iter().any(Option::is_some),
        MenuAction::Slot(i) => page != MenuPage::Slots(Mode::Load) || slots.0[i].is_some(),
        _ => true,
    }
}

fn style_menu(
    art: Res<Art>,
    page: Res<MenuPage>,
    slots: Res<MenuSlots>,
    mut q: Query<(&MenuAction, &Hovered, Has<Pressed>, &mut ImageNode), With<Button>>,
) {
    for (action, hovered, pressed, mut img) in &mut q {
        let name = match (enabled(*action, *page, &slots), pressed, hovered.get()) {
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

fn race_name(state: &GameState) -> &'static str {
    if state.race_chosen { state.race.def().name } else { "No faction yet" }
}

/// One-line summary for Continue and the overwrite prompt.
fn summary(state: &GameState) -> String {
    format!("{} - sector {} - {} shards", race_name(state), state.sector, fmt(state.shards.floor()))
}

fn rebuild(mut commands: Commands, art: Res<Art>, page: Res<MenuPage>, slots: Res<MenuSlots>, roots: Query<Entity, With<MenuRoot>>) {
    if !page.is_changed() && !roots.is_empty() {
        return;
    }
    for e in &roots {
        commands.entity(e).despawn();
    }
    let root = (
        Node {
            position_type: PositionType::Absolute,
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: px(12),
            ..default()
        },
        BackgroundColor(Color::srgba(0.01, 0.01, 0.03, 0.6)),
        GlobalZIndex(10),
        MenuRoot,
        DespawnOnExit(AppState::Menu),
    );
    let tall = |w: f32, h: f32| Node { width: px(w), height: px(h), flex_direction: FlexDirection::Column, justify_content: JustifyContent::Center, align_items: AlignItems::Center, row_gap: px(4), ..default() };
    commands.spawn(root).with_children(|p| match *page {
        MenuPage::Main => {
            p.spawn((bold(&art, "VOID ARMADA", 64.0, CYAN), TextShadow::default()));
            p.spawn((label(&art, "Command the armada. Hold the line.", 16.0, DIM), Node { margin: UiRect::bottom(px(36)), ..default() }));
            if let Some(slot) = most_recent(&slots.0)
                && let Some(state) = &slots.0[slot]
            {
                p.spawn(menu_button(&art, MenuAction::Continue, tall(360.0, 68.0))).with_children(|p| {
                    p.spawn(label(&art, "Continue", 16.0, WHITE));
                    p.spawn(body(&art, format!("Slot {} - {}", slot + 1, summary(state)), 14.0, GOLD));
                });
            }
            for (a, text) in [(MenuAction::NewGame, "New Game"), (MenuAction::LoadGame, "Load Game"), (MenuAction::Quit, "Quit")] {
                p.spawn(menu_button(&art, a, tall(360.0, 52.0))).with_children(|p| {
                    p.spawn(label(&art, text, 16.0, WHITE));
                });
            }
            p.spawn(Node { margin: UiRect::top(px(12)), ..default() }).with_children(|p| crate::view::display_rows(p, &art));
            p.spawn((
                body(&art, "Esc in game opens the menu", 14.0, DIM),
                Node { position_type: PositionType::Absolute, bottom: px(20), ..default() },
            ));
        }
        MenuPage::Slots(mode) => {
            p.spawn(bold(&art, if mode == Mode::New { "NEW GAME" } else { "LOAD GAME" }, 24.0, WHITE));
            p.spawn((label(&art, "Choose a save slot", 16.0, DIM), Node { margin: UiRect::bottom(px(20)), ..default() }));
            p.spawn(Node { column_gap: px(16), ..default() }).with_children(|p| {
                for (i, slot) in slots.0.iter().enumerate() {
                    p.spawn(menu_button(
                        &art,
                        MenuAction::Slot(i),
                        Node {
                            width: px(272),
                            height: px(232),
                            flex_direction: FlexDirection::Column,
                            align_items: AlignItems::Center,
                            padding: UiRect::all(px(16)),
                            row_gap: px(6),
                            ..default()
                        },
                    ))
                    .with_children(|p| slot_card(p, &art, i, slot.as_ref(), mode));
                }
            });
            p.spawn(menu_button(&art, MenuAction::Back, Node { width: px(200), height: px(44), margin: UiRect::top(px(16)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                .with_children(|p| {
                    p.spawn(label(&art, "Back", 16.0, WHITE));
                });
        }
        MenuPage::Confirm(i) => {
            p.spawn((chrome(&art, "panel", 16.0), Node { flex_direction: FlexDirection::Column, align_items: AlignItems::Center, padding: UiRect::all(px(28)), row_gap: px(16), width: px(520), ..default() }))
                .with_children(|p| {
                    p.spawn(bold(&art, format!("OVERWRITE SLOT {}?", i + 1), 16.0, GOLD));
                    if let Some(state) = &slots.0[i] {
                        p.spawn(body(&art, summary(state), 16.0, WHITE));
                    }
                    p.spawn(body(&art, "That run will be lost, with its shards and research.", 14.0, DIM));
                    p.spawn(Node { column_gap: px(12), ..default() }).with_children(|p| {
                        for (a, text) in [(MenuAction::Back, "Cancel"), (MenuAction::Overwrite(i), "Overwrite")] {
                            p.spawn(menu_button(&art, a, Node { width: px(170), height: px(44), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() }))
                                .with_children(|p| {
                                    p.spawn(label(&art, text, 16.0, WHITE));
                                });
                        }
                    });
                });
        }
    });
}

fn slot_card(p: &mut ChildSpawnerCommands, art: &Art, i: usize, slot: Option<&GameState>, mode: Mode) {
    p.spawn(bold(art, format!("SLOT {}", i + 1), 16.0, CYAN));
    let Some(s) = slot else {
        p.spawn(Node { flex_grow: 1.0, ..default() });
        p.spawn(label(art, "Empty", 16.0, DIM));
        if mode == Mode::New {
            p.spawn(body(art, "Start a new run here", 14.0, DIM));
        }
        p.spawn(Node { flex_grow: 1.0, ..default() });
        return;
    };
    p.spawn(Node { height: px(72), align_items: AlignItems::Center, justify_content: JustifyContent::Center, ..default() }).with_children(|p| {
        let race = if s.race_chosen { s.race } else { Race::Tidewall };
        let name = portrait(&flagship_sheet(race, s.hull));
        let size = portrait_size(art.size(&name), Vec2::new(220.0, 64.0));
        p.spawn((art.ui_image(&name), Node { width: px(size.x), height: px(size.y), ..default() }));
    });
    p.spawn(label(art, race_name(s), 16.0, WHITE));
    p.spawn(body(art, format!("Sector {}  (deepest {})", s.sector, s.max_sector), 14.0, GOLD));
    p.spawn(body(art, format!("{} shards   {} jumps", fmt(s.shards.floor()), s.jumps), 14.0, PURPLE));
    p.spawn(body(art, format!("Played {}", time_ago(now(), s.saved_at)), 14.0, DIM));
}

/// A few escorts from every faction cruise past behind the menu.
fn spawn_drifters(mut commands: Commands, art: Res<Art>, cam: Single<&Transform, With<MainCamera>>) {
    let mut rng = rand::rng();
    let c = cam.translation;
    // escorts up to cruisers, in two bands clear of the title and buttons
    let kinds = [0, 1, 2, 3, LOGI_BASE, LOGI_BASE + 1];
    for i in 0..8 {
        let race = Race::ALL[i % Race::ALL.len()];
        let name = fleet_sheet(race, kinds[rng.random_range(0..kinds.len())]);
        let x = c.x + rng.random_range(-360.0..360.0f32);
        let band = if i % 2 == 0 { 1.0 } else { -1.0 };
        let y = c.y + band * (130.0 + (i / 2) as f32 * 12.0) + rng.random_range(-3.0..3.0f32);
        let mut sprite = art.sprite(&name);
        // dimmed so they stay scenery
        sprite.color = Color::srgb(0.6, 0.62, 0.72);
        commands.spawn((
            sprite,
            Anim::looping(2, 8.0),
            Transform::from_translation(Vec3::new(x.round(), y.round(), -50.0 + i as f32 * 0.1)),
            Drifter(rng.random_range(6.0..16.0), x),
            DespawnOnExit(AppState::Menu),
        ));
    }
}

fn drift(time: Res<Time>, cam: Single<&Transform, (With<MainCamera>, Without<Drifter>)>, mut q: Query<(&mut Drifter, &mut Transform)>) {
    let left = cam.translation.x - 420.0;
    let right = cam.translation.x + 420.0;
    for (mut d, mut tf) in &mut q {
        d.1 += d.0 * time.delta_secs();
        if d.1 > right {
            d.1 = left;
        }
        tf.translation.x = d.1.round();
    }
}
