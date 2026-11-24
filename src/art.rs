//! Sprite registry: loads every PNG produced by the Aseprite scripts and
//! builds a texture atlas per sheet once its size is known.

use bevy::prelude::*;
use std::collections::HashMap;

use crate::AppState;
use crate::economy::{FACTION_KEYS, HULL_KINDS, Race, hull_key};

pub struct ArtPlugin;

impl Plugin for ArtPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load)
            .add_systems(Update, finish_loading.run_if(in_state(AppState::Loading)));
    }
}

pub struct Sheet {
    pub image: Handle<Image>,
    pub frames: usize,
    pub layout: Option<Handle<TextureAtlasLayout>>,
    pub frame_size: Vec2,
}

#[derive(Resource, Default)]
pub struct Art {
    sheets: HashMap<String, Sheet>,
    pub ui: HashMap<&'static str, Handle<Image>>,
    pub font: Handle<Font>,
    pub font_bold: Handle<Font>,
    pub font_body: Handle<Font>,
}

impl Art {
    pub fn sheet(&self, name: &str) -> &Sheet {
        self.sheets.get(name).unwrap_or_else(|| panic!("missing sprite sheet {name}"))
    }

    pub fn image(&self, name: &str) -> Handle<Image> {
        self.sheet(name).image.clone()
    }

    pub fn size(&self, name: &str) -> Vec2 {
        self.sheet(name).frame_size
    }

    pub fn frames(&self, name: &str) -> usize {
        self.sheet(name).frames
    }

    /// A sprite showing frame 0 of the named sheet.
    pub fn sprite(&self, name: &str) -> Sprite {
        let s = self.sheet(name);
        match &s.layout {
            Some(layout) => Sprite::from_atlas_image(s.image.clone(), TextureAtlas { layout: layout.clone(), index: 0 }),
            None => Sprite::from_image(s.image.clone()),
        }
    }

    /// UI image of frame 0 of the named sheet.
    pub fn ui_image(&self, name: &str) -> ImageNode {
        let s = self.sheet(name);
        match &s.layout {
            Some(layout) => ImageNode::from_atlas_image(s.image.clone(), TextureAtlas { layout: layout.clone(), index: 0 }),
            None => ImageNode::new(s.image.clone()),
        }
    }

    pub fn ui(&self, name: &str) -> Handle<Image> {
        self.ui.get(name).cloned().unwrap_or_else(|| panic!("missing ui image {name}"))
    }
}

pub fn race_key(race: Race) -> &'static str {
    race.def().key
}

pub fn flagship_sheet(race: Race, tier: u32) -> String {
    format!("flagship_{}_{}", race_key(race), tier.clamp(1, 6))
}

/// Sheet of an escort or logi hull (`kind` indexes HULL_KINDS).
pub fn fleet_sheet(race: Race, kind: usize) -> String {
    format!("fleet_{}_{}", race_key(race), hull_key(kind))
}

/// Cropped, flame-free still of a player hull sheet, for the UI.
pub fn portrait(sheet: &str) -> String {
    format!("portrait_{sheet}")
}

/// Size to draw a portrait inside a `max` box: whole-number scales when the
/// hull fits, so pixels stay square; big hulls shrink to fit.
pub fn portrait_size(size: Vec2, max: Vec2) -> Vec2 {
    let fit = (max.x / size.x).min(max.y / size.y);
    let k = if fit >= 1.0 { fit.floor() } else { fit };
    size * k
}

pub fn enemy_logi_sheet(faction: usize) -> String {
    format!("enemy_{}_logi", FACTION_KEYS[faction])
}

pub fn enemy_sheet(faction: usize, tier: u32) -> String {
    format!("enemy_{}_{}", FACTION_KEYS[faction], tier.clamp(1, 5))
}

/// Every sheet the game uses with its frame count.
fn registry() -> Vec<(String, usize)> {
    let mut v: Vec<(String, usize)> = vec![];
    for race in Race::ALL {
        for t in 1..=6 {
            v.push((flagship_sheet(race, t), 2));
            v.push((portrait(&flagship_sheet(race, t)), 1));
        }
        for c in 0..HULL_KINDS {
            v.push((fleet_sheet(race, c), 2));
            v.push((portrait(&fleet_sheet(race, c)), 1));
        }
    }
    for (f, key) in FACTION_KEYS.iter().enumerate() {
        for t in 1..=5 {
            v.push((enemy_sheet(f, t), 2));
        }
        v.push((enemy_logi_sheet(f), 2));
        v.push((format!("nebula_{key}"), 1));
    }
    let fx: [(&str, usize); 31] = [
        ("explosion_small", 8),
        ("explosion_big", 10),
        ("spark", 4),
        ("shield_hit", 5),
        ("credit", 6),
        ("smoke", 5),
        ("muzzle", 3),
        ("warp_in", 6),
        ("reticle", 5),
        ("bolt_cyan", 1),
        ("bolt_blue", 1),
        ("bolt_heavy", 1),
        ("bolt_red", 1),
        ("bolt_green", 1),
        ("bolt_purple", 1),
        ("bolt_gold", 1),
        ("plasma", 2),
        ("missile", 2),
        ("shell", 1),
        ("strike_craft", 2),
        ("beam_cyan", 1),
        ("beam_purple", 1),
        ("beam_red", 1),
        ("beam_gold", 1),
        ("beam_green", 1),
        ("debris", 3),
        ("stars_far", 1),
        ("stars_mid", 1),
        ("stars_near", 1),
        ("planet_ringed", 1),
        ("planet_ice", 1),
    ];
    for (name, frames) in fx {
        v.push((name.to_string(), frames));
    }
    v.push(("planet_lava".into(), 1));
    v.push(("planet_jungle".into(), 1));
    v
}

pub const UI_IMAGES: [&str; 30] = [
    "panel", "header", "button", "button_hover", "button_pressed", "button_disabled", "button_ready", "button_gold",
    "bar", "icon_credits", "icon_dps", "icon_click", "icon_crit", "icon_drone", "icon_hull", "icon_shard",
    "icon_boss", "icon_timer", "icon_sync", "icon_overcharge", "icon_broadside", "icon_salvage", "icon_warp",
    "icon_sound_on", "icon_sound_off", "icon_research", "icon_plating", "icon_repair", "icon_retreat", "icon_menu",
];

fn load(mut commands: Commands, server: Res<AssetServer>) {
    let mut art = Art {
        font: server.load("fonts/Silkscreen-Regular.ttf"),
        font_bold: server.load("fonts/Silkscreen-Bold.ttf"),
        font_body: server.load("fonts/PixelifySans.ttf"),
        ..default()
    };
    for (name, frames) in registry() {
        let image = server.load(format!("sprites/{name}.png"));
        art.sheets.insert(name, Sheet { image, frames, layout: None, frame_size: Vec2::ZERO });
    }
    for name in UI_IMAGES {
        art.ui.insert(name, server.load(format!("sprites/ui/{name}.png")));
    }
    commands.insert_resource(art);
}

fn finish_loading(
    mut art: ResMut<Art>,
    server: Res<AssetServer>,
    images: Res<Assets<Image>>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut next: ResMut<NextState<AppState>>,
) {
    let all_sheets = art.sheets.values().all(|s| server.is_loaded_with_dependencies(&s.image));
    let all_ui = art.ui.values().all(|h| server.is_loaded_with_dependencies(h));
    let fonts = [&art.font, &art.font_bold, &art.font_body].iter().all(|f| server.is_loaded_with_dependencies(*f));
    if !(all_sheets && all_ui && fonts) {
        return;
    }
    for sheet in art.sheets.values_mut() {
        let size = images.get(&sheet.image).expect("loaded image").size();
        let fw = size.x / sheet.frames as u32;
        sheet.frame_size = Vec2::new(fw as f32, size.y as f32);
        if sheet.frames > 1 {
            let layout = TextureAtlasLayout::from_grid(UVec2::new(fw, size.y), sheet.frames as u32, 1, None, None);
            sheet.layout = Some(layouts.add(layout));
        }
    }
    next.set(AppState::Menu);
}
