//! Presentation metadata remains independent of collision and navigation.
use crate::importer::ImportedLevel;
use serde::Deserialize;
use std::collections::BTreeMap;
#[derive(Clone, Debug, Deserialize)]
pub struct TileVisual {
    pub visual: String,
    pub reviewed: bool,
}
#[derive(Clone, Debug, Deserialize)]
pub struct JungleCatalog {
    pub tileset: String,
    pub reference: String,
    pub tiles: BTreeMap<String, TileVisual>,
}
impl JungleCatalog {
    pub fn bundled() -> Self {
        serde_json::from_str(include_str!("../../../assets/catalogs/jungle.json"))
            .expect("validated bundled visual catalog")
    }
    pub fn reviewed(&self, map: &ImportedLevel) -> bool {
        map.base == "junbase"
            && map.subset == "junsub0"
            && map.tiles.iter().all(|id| {
                self.tiles
                    .get(&(id & 511).to_string())
                    .is_some_and(|v| v.reviewed)
            })
    }
}
#[derive(Clone, Copy, Debug)]
pub struct ThicketPatch {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub depth: f32,
}
/// Greedy rectangles share borders exactly with the BHT mask. Cap dimensions to avoid stretched foliage.
pub fn thickets(map: &ImportedLevel, catalog: &JungleCatalog) -> Vec<ThicketPatch> {
    let width = map.width * 8;
    let height = map.height * 8;
    let mut used = vec![false; width * height];
    let mut result = vec![];
    let forest = |x: usize, y: usize| {
        let px = x as f32 * 2. + 1.;
        let py = y as f32 * 2. + 1.;
        let tile = (map.tiles[y / 8 * map.width + x / 8] & 511).to_string();
        map.terrain_at(px, py) == 3
            && catalog
                .tiles
                .get(&tile)
                .is_none_or(|v| v.visual == "jungle_thicket")
    };
    for y in 0..height {
        for x in 0..width {
            if used[y * width + x] || !forest(x, y) {
                continue;
            }
            let mut w = 1;
            while w < 6 && x + w < width && !used[y * width + x + w] && forest(x + w, y) {
                w += 1;
            }
            let mut h = 1;
            while h < 6
                && y + h < height
                && (x..x + w).all(|xx| !used[(y + h) * width + xx] && forest(xx, y + h))
            {
                h += 1;
            }
            for yy in y..y + h {
                for xx in x..x + w {
                    used[yy * width + xx] = true;
                }
            }
            result.push(ThicketPatch {
                x: (x * 2) as f32,
                y: (y * 2) as f32,
                width: (w * 2) as f32,
                depth: (h * 2) as f32,
            });
        }
    }
    result
}
