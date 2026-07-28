use std::{collections::HashMap, hash::Hash, sync::Arc};

use crate::{
    animations::Clip,
    spritesheet::{Spritesheet, convert},
    states::AppState,
};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug, EnumIter)]
pub enum Sheet {
    Copter,
    Signs,
    Passenger,
    Bubble,
}

impl Sheet {
    fn path(self) -> &'static str {
        match self {
            Sheet::Copter => "sprites/copter42.sheet.json",
            Sheet::Signs => "sprites/signs.sheet.json",
            Sheet::Passenger => "sprites/caveman3.sheet.json",
            Sheet::Bubble => "sprites/bubble.sheet.json",
        }
    }
}

#[derive(Resource)]
pub struct GameAssets {
    sheets: HashMap<Sheet, SpriteSheet>,
}

impl GameAssets {
    pub fn new() -> Self {
        Self {
            sheets: HashMap::new(),
        }
    }

    pub fn get(&self, sheet: Sheet) -> &SpriteSheet {
        &self.sheets[&sheet]
    }

    pub fn insert(&mut self, sheet: Sheet, sprite: SpriteSheet) {
        let _ = &self.sheets.insert(sheet, sprite);
    }
}

#[derive(Resource)]
pub struct PendingSheets {
    pub pending: HashMap<Sheet, Handle<Spritesheet>>,
}

impl PendingSheets {
    pub fn new() -> Self {
        Self {
            pending: HashMap::new(),
        }
    }

    pub fn get(&self, sheet: Sheet) -> &Handle<Spritesheet> {
        &self.pending[&sheet]
    }

    pub fn load(&mut self, assets: &Res<AssetServer>, sheet: Sheet) {
        self.pending.insert(sheet, assets.load(sheet.path()));
    }
}

/// One Aseprite export, engine-side
pub struct SpriteSheet {
    pub image: Handle<Image>,
    pub layout: Handle<TextureAtlasLayout>,
    clips: HashMap<String, Clip>, // all lookups go through clip()
}

impl SpriteSheet {
    pub fn clip(&self, name: &str) -> Clip {
        self.clips.get(name).cloned().unwrap_or_else(|| {
            panic!(
                "no animation tag '{name}'; sheet has: {:?}",
                self.clips.keys().collect::<Vec<_>>()
            )
        })
    }
}

pub struct AssetsPlugin;

impl Plugin for AssetsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(JsonAssetPlugin::<Spritesheet>::new(&["sheet.json"]))
            .add_systems(OnEnter(AppState::Loading), start_loading)
            .add_systems(Update, build_when_ready.run_if(in_state(AppState::Loading)));
    }
}

fn start_loading(mut commands: Commands, assets: Res<AssetServer>) {
    let mut pending_sheets = PendingSheets::new();
    for sheet in Sheet::iter() {
        pending_sheets.load(&assets, sheet);
    }
    commands.insert_resource(pending_sheets);
}

/// Polls every frame while Loading
fn build_when_ready(
    mut commands: Commands,
    pending: Res<PendingSheets>,
    sheets: Res<Assets<Spritesheet>>,
    assets: Res<AssetServer>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut next: ResMut<NextState<AppState>>,
) {
    // pass 1, check if there are still pending loadings
    if !Sheet::iter().all(|s| sheets.contains(pending.get(s))) {
        return;
    }

    let mut game_assets = GameAssets::new();

    for sheet in Sheet::iter() {
        let spritesheet = sheets.get(pending.get(sheet)).expect("already checked");
        game_assets.insert(sheet, build_sheet(spritesheet, &assets, &mut layouts));
    }

    commands.insert_resource(game_assets);
    commands.remove_resource::<PendingSheets>();
    next.set(AppState::InGame);
}

/// Parsed JSON to engine objects
fn build_sheet(
    sheet: &Spritesheet,
    assets: &AssetServer,
    layouts: &mut Assets<TextureAtlasLayout>,
) -> SpriteSheet {
    let (size, rects, filename, animations) = convert(sheet);
    let mut layout = TextureAtlasLayout::new_empty(size);
    for rect in rects {
        layout.add_texture(rect);
    }

    SpriteSheet {
        image: assets.load(format!("sprites/{}", filename)),
        layout: layouts.add(layout),

        // each Clip constructed Once: every clip() close shares this Arc, which is what
        // makes ptr_eq change-detection valid
        clips: animations
            .into_iter()
            .map(|(name, frames)| {
                (
                    name,
                    Clip {
                        frames: Arc::from(frames.as_slice()),
                        looped: true,
                    },
                )
            })
            .collect(),
    }
}
