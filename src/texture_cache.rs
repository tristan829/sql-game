use std::collections::{HashMap, hash_map::Entry};
use macroquad::texture::{Texture2D, load_texture};

pub struct TextureCache {
    map: HashMap<String, Texture2D>,
}

impl TextureCache {
    pub fn new() -> TextureCache {
        TextureCache {
            map: HashMap::new()
        }
    }

    pub async fn get(&mut self, path: &str) -> Texture2D {
        match self.map.entry(path.to_string()) {
            Entry::Occupied(o) => o.get().clone(),
            Entry::Vacant(v) => {
                let tex = load_texture(path).await.unwrap();
                v.insert(tex.clone());
                tex
            }
        }
    }
}