use std::{collections::HashMap, sync::Arc};

use crate::{
    animations::Clip,
    spritesheet::{Spritesheet, convert},
    states::AppState,
};
use bevy::prelude::*;
use bevy_common_assets::json::JsonAssetPlugin;

#[derive(Resource)]
pub struct GameAssets {
    pub copter: SpriteSheet,
    pub signs: SpriteSheet,
    pub passenger: SpriteSheet,
    //pub copter_layout: Handle<TextureAtlasLayout>,
    /*
    pub ptero_sheet: Handle<Image>,
    pub ptero_layout: Handle<TextureAtlasLayout>,
    pub passenger_sheet: Handle<Image>,
    pub passenger_layout: Handle<TextureAtlasLayout>,
    pub platform: Handle<Image>,
    pub rock: Handle<Image>,
    */
}

#[derive(Resource)]
struct PendingSheets {
    copter: Handle<Spritesheet>,
    signs: Handle<Spritesheet>,
    passenger: Handle<Spritesheet>,
    // TODO: other sprites go here
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
    commands.insert_resource(PendingSheets {
        copter: assets.load("sprites/copter42.sheet.json"),
        signs: assets.load("sprites/signs.sheet.json"),
        passenger: assets.load("sprites/passenger.sheet.json"),
    });
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
    let Some(copter) = sheets.get(&pending.copter) else {
        return; // still loading, - ask again next frame
    };

    let Some(signs) = sheets.get(&pending.signs) else {
        return;
    };
    let Some(passenger) = sheets.get(&pending.passenger) else {
        return;
    };

    commands.insert_resource(GameAssets {
        copter: build_sheet(copter, &assets, &mut layouts),
        signs: build_sheet(signs, &assets, &mut layouts),
        passenger: build_sheet(passenger, &assets, &mut layouts),
    });
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
