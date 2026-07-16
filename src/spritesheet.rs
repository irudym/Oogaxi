use bevy::prelude::*;
use serde_json::Value;
use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Asset, TypePath, Debug, Deserialize)]
pub struct Spritesheet {
    pub frames: Vec<FrameDescription>,
    pub meta: Meta,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Meta {
    app: String,
    version: String,
    image: String,
    format: String,
    size: JsonSize,
    scale: String,
    frame_tags: Vec<FrameTag>,
    layers: Vec<Value>,
    slices: Vec<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonSize {
    h: u32,
    w: u32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrameTag {
    name: String,
    from: u32,
    to: u32,
    direction: String,
    color: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonRect {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct FrameDescription {
    filename: String,
    frame: JsonRect,
    rotated: bool,
    trimmed: bool,
    sprite_source_size: JsonRect,
    source_size: JsonSize,
    duration: u32,
}

pub type Animations = HashMap<String, Vec<(usize, f32)>>;

/// Load sprite sheet from json file
/// Return:
///     (UVec2 - Total size of texture atlas, Vec<URect> - The specific areas of the atlas where each texture can be found,
///     image_name, Animations - hashmap of animation names and frames numbers)
pub fn convert(value: &Spritesheet) -> (UVec2, Vec<URect>, String, Animations) {
    let size = UVec2::new(value.meta.size.w, value.meta.size.h);
    let mut textures: Vec<URect> = Vec::new();
    let mut durations: Vec<f32> = Vec::new();

    // fill textures
    for frame_description in &value.frames {
        let min = UVec2::new(
            frame_description.frame.x as u32,
            frame_description.frame.y as u32,
        );
        textures.push(URect {
            min,
            max: min
                + UVec2::new(
                    frame_description.frame.w as u32,
                    frame_description.frame.h as u32,
                ),
        });
        durations.push(frame_description.duration as f32 / 1000.0);
    }
    let mut animations = Animations::new();

    // fill animations
    for frame_tag in &value.meta.frame_tags {
        let mut frames = Vec::new();
        for i in frame_tag.from..=frame_tag.to {
            frames.push((i as usize, durations[i as usize]));
        }
        animations.insert(frame_tag.name.clone(), frames);
    }

    (size, textures, value.meta.image.clone(), animations)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rects_are_corner_pairs_not_sizes() {
        let json = r#"{"frames":[
            {"filename":"a","frame":{"x":34,"y":1,"w":32,"h":32},
             "rotated":false,"trimmed":false,"duration":80}],
          "meta":{"image":"a.png","size":{"w":133,"h":133},
            "frameTags":[{"name":"idle","from":0,"to":0,"direction":"forward"}]}}"#;
        let sheet: Spritesheet = serde_json::from_str(json).unwrap();
        let (_, rects, _, anims) = convert(&sheet);
        assert_eq!(rects[0].min, UVec2::new(34, 1));
        assert_eq!(rects[0].max, UVec2::new(66, 33)); // min + size — the bug's tombstone
        assert_eq!(anims["idle"][0], (0, 0.08)); // durations survive, in seconds
    }
}
