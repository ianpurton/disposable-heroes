//! Native CF1 map reader. File coordinates remain in original 16-pixel tiles.
use serde::{Deserialize, Serialize};
use std::path::Path;
#[derive(Debug, thiserror::Error)]
pub enum ImportError {
    #[error("{file}: {message}")]
    Invalid { file: String, message: String },
    #[error("{0}: {1}")]
    Io(String, std::io::Error),
}
fn invalid(file: &str, message: impl Into<String>) -> ImportError {
    ImportError::Invalid {
        file: file.into(),
        message: message.into(),
    }
}
#[cfg(not(target_arch = "wasm32"))]
fn read(path: &Path) -> Result<Vec<u8>, ImportError> {
    std::fs::read(path).map_err(|e| ImportError::Io(path.display().to_string(), e))
}
fn word(b: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([b[i], b[i + 1]])
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SpriteRecord {
    pub direction: u16,
    pub reserved: u16,
    pub stored_x: i16,
    pub y: i16,
    pub kind: u16,
}
impl SpriteRecord {
    pub fn x(&self) -> i32 {
        i32::from(self.stored_x) + 16
    }
}
#[derive(Clone, Debug)]
pub struct TerrainTables {
    pub hit: Vec<i16>,
    pub masks: Vec<[u8; 8]>,
    pub swap: Vec<i16>,
}
#[derive(Clone, Debug)]
pub struct Structure {
    pub x: f32,
    pub y: f32,
    pub tiles: Vec<usize>,
    pub door: usize,
}
#[derive(Clone, Debug)]
pub struct ImportedLevel {
    pub width: usize,
    pub height: usize,
    pub base: String,
    pub subset: String,
    pub tiles: Vec<u16>,
    pub sprites: Vec<SpriteRecord>,
    pub terrain: TerrainTables,
    pub structures: Vec<Structure>,
}
impl ImportedLevel {
    pub fn parse(map: &[u8], spt: &[u8]) -> Result<Self, ImportError> {
        if map.len() < 96 {
            return Err(invalid("map", "truncated header at byte 0; need 96 bytes"));
        }
        let width = word(map, 84) as usize;
        let height = word(map, 86) as usize;
        if width == 0 || height == 0 || width > 2048 || height > 2048 {
            return Err(invalid("map", "invalid dimensions at byte 84"));
        }
        let required = 96 + width * height * 2;
        if map.len() < required {
            return Err(invalid(
                "map",
                format!("truncated tiles at byte {}; need {required}", map.len()),
            ));
        }
        if spt.len() % 10 != 0 {
            return Err(invalid(
                "spt",
                format!("incomplete 10-byte record at byte {}", spt.len() / 10 * 10),
            ));
        }
        let name = |offset: usize| -> Result<String, ImportError> {
            let field = &map[offset..offset + 16];
            let end = field.iter().position(|b| *b == 0).unwrap_or(16);
            let value = std::str::from_utf8(&field[..end])
                .map_err(|_| invalid("map", format!("invalid filename at byte {offset}")))?;
            if !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'.' || b == b'_')
                || !value.ends_with(".blk")
            {
                return Err(invalid(
                    "map",
                    format!("invalid tileset filename at byte {offset}"),
                ));
            }
            Ok(value.trim_end_matches(".blk").into())
        };
        Ok(Self {
            width,
            height,
            base: name(0)?,
            subset: name(16)?,
            tiles: (96..required).step_by(2).map(|i| word(map, i)).collect(),
            sprites: spt
                .chunks_exact(10)
                .map(|b| SpriteRecord {
                    direction: word(b, 0),
                    reserved: word(b, 2),
                    stored_x: word(b, 4) as i16,
                    y: word(b, 6) as i16,
                    kind: word(b, 8),
                })
                .collect(),
            terrain: TerrainTables {
                hit: vec![3; 512],
                masks: vec![[0; 8]; 512],
                swap: vec![-1; 512],
            },
            structures: vec![],
        })
    }
    pub fn load(path: &Path, resources: Option<&Path>) -> Result<Self, ImportError> {
        let mut level = Self::parse(&read(path)?, &read(&path.with_extension("spt"))?)
            .map_err(|e| invalid(&path.display().to_string(), e.to_string()))?;
        let dir = resources.unwrap_or_else(|| path.parent().unwrap_or(Path::new(".")));
        for (name, offset) in [(&level.base, 0), (&level.subset, 240)] {
            let hit = read(&dir.join(format!("{name}.hit")))?;
            let bht = read(&dir.join(format!("{name}.bht")))?;
            let swp = read(&dir.join(format!("{name}.swp")))?;
            if hit.len() % 2 != 0
                || swp.len() != hit.len()
                || bht.len() != hit.len() * 4
                || offset + hit.len() / 2 > 512
            {
                return Err(invalid(name, "inconsistent HIT/BHT/SWP sizes"));
            }
            for i in 0..hit.len() / 2 {
                level.terrain.hit[offset + i] = word(&hit, 2 * i) as i16;
                level.terrain.swap[offset + i] = word(&swp, 2 * i) as i16;
                level.terrain.masks[offset + i].copy_from_slice(&bht[i * 8..i * 8 + 8]);
            }
        }
        level.resolve_structures();
        Ok(level)
    }
    pub fn tile_index(&self, x: f32, y: f32) -> Option<usize> {
        if x < 0. || y < 0. || x >= self.width as f32 * 16. || y >= self.height as f32 * 16. {
            None
        } else {
            Some(y as usize / 16 * self.width + x as usize / 16)
        }
    }
    pub fn terrain_at(&self, x: f32, y: f32) -> u8 {
        let Some(i) = self.tile_index(x, y) else {
            return 3;
        };
        let tile = (self.tiles[i] & 511) as usize;
        let mut hit = self.terrain.hit[tile];
        if hit < 0
            && self.terrain.masks[tile][(y as usize % 16) / 2] & (1 << (7 - (x as usize % 16) / 2))
                != 0
        {
            hit >>= 4;
        }
        (hit & 15) as u8
    }
    pub fn walkable(&self, x: f32, y: f32) -> bool {
        self.terrain_at(x, y) != 3
    }
    pub fn destroy_tiles(&mut self, indices: &[usize]) {
        for &i in indices {
            let id = (self.tiles[i] & 511) as usize;
            let next = self.terrain.swap[id];
            if next >= 0 {
                self.tiles[i] = next as u16;
            }
        }
    }
    fn resolve_structures(&mut self) {
        // Jungle barracks footprints: match whole patterns, never one building per tile.
        let pieces = [
            (1, 0, 333),
            (2, 0, 334),
            (0, 1, 352),
            (1, 1, 353),
            (2, 1, 354),
            (0, 2, 372),
            (1, 2, 373),
            (2, 2, 374),
            (3, 2, 375),
            (0, 3, 392),
            (1, 3, 393),
            (2, 3, 394),
        ];
        let mut used = std::collections::HashSet::new();
        for (door, s) in self
            .sprites
            .iter()
            .enumerate()
            .filter(|(_, s)| matches!(s.kind, 20 | 25 | 88 | 100))
        {
            let mut footprint = vec![];
            if self.base.starts_with("jun") {
                for y in 0..self.height.saturating_sub(3) {
                    for x in 0..self.width.saturating_sub(3) {
                        if pieces.iter().all(|&(dx, dy, id)| {
                            self.tiles[(y + dy) * self.width + x + dx] & 511 == id
                        }) && ((x as f32 * 16. - s.x() as f32).abs() < 64.)
                            && ((y as f32 * 16. - s.y as f32).abs() < 80.)
                        {
                            let p: Vec<_> = pieces
                                .iter()
                                .map(|&(dx, dy, _)| (y + dy) * self.width + x + dx)
                                .collect();
                            if p.iter().all(|i| !used.contains(i)) {
                                footprint = p;
                            }
                        }
                    }
                }
            }
            used.extend(footprint.iter().copied());
            self.structures.push(Structure {
                x: s.x() as f32,
                y: s.y as f32 - 18.,
                tiles: footprint,
                door,
            });
        }
    }
    pub fn unsupported_sprites(&self) -> Vec<u16> {
        let mut ids: Vec<_> = self
            .sprites
            .iter()
            .filter(|s| {
                !matches!(
                    s.kind,
                    0 | 4
                        | 5
                        | 13
                        | 14
                        | 15
                        | 17
                        | 19
                        | 20
                        | 24
                        | 25
                        | 27
                        | 36
                        | 37
                        | 38
                        | 54
                        | 55
                        | 56
                        | 61
                        | 62
                        | 63
                        | 64
                        | 65
                        | 66
                        | 67
                        | 69
                        | 80
                        | 81
                        | 82
                        | 88
                        | 100
                )
            })
            .map(|s| s.kind)
            .collect();
        ids.sort();
        ids.dedup();
        ids
    }
}

// Cache the complete bundled campaign before starting Bevy. Retries and phase
// transitions use the same validated importer without blocking browser I/O.
#[cfg(target_arch = "wasm32")]
thread_local! {
    static BROWSER_FILES: std::cell::RefCell<std::collections::HashMap<String, Vec<u8>>> = Default::default();
}
#[cfg(target_arch = "wasm32")]
fn read(path: &Path) -> Result<Vec<u8>, ImportError> {
    BROWSER_FILES
        .with(|files| {
            files
                .borrow()
                .get(&path.to_string_lossy().to_string())
                .cloned()
        })
        .ok_or_else(|| {
            invalid(
                &path.display().to_string(),
                "Browser resource was not loaded",
            )
        })
}
// Only the transport URL changes; importer paths and cache keys stay original.
#[cfg(any(target_arch = "wasm32", test))]
fn browser_resource_url(path: &str) -> String {
    if path.ends_with(".swp") {
        format!("{path}.bin")
    } else {
        path.into()
    }
}

#[cfg(test)]
mod browser_url_tests {
    use super::browser_resource_url;

    #[test]
    fn fixture_requests_never_end_in_swp() {
        let manifest: serde_json::Value =
            serde_json::from_str(include_str!("../../../tests/fixtures-manifest.json")).unwrap();
        let mut swap_files = 0;
        for path in manifest["files"].as_object().unwrap().keys() {
            let logical = format!("data/{path}");
            let url = browser_resource_url(&logical);
            assert!(!url.ends_with(".swp"), "Unsafe browser URL: {url}");
            if path.ends_with(".swp") {
                swap_files += 1;
                assert_eq!(url, format!("{logical}.bin"));
            } else {
                assert_eq!(url, logical);
            }
        }
        assert!(swap_files > 0);
        assert_eq!(
            browser_resource_url("data/Data/AmigaTheOne/junbase.swp.bin"),
            "data/Data/AmigaTheOne/junbase.swp.bin"
        );
    }
}

#[cfg(target_arch = "wasm32")]
async fn fetch(path: &str) -> Result<Vec<u8>, String> {
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    if let Some(bytes) = BROWSER_FILES.with(|files| files.borrow().get(path).cloned()) {
        return Ok(bytes);
    }
    let url = browser_resource_url(path);
    let window = web_sys::window().ok_or("No browser window")?;
    let response = JsFuture::from(window.fetch_with_str(&url))
        .await
        .map_err(|e| format!("GET {url}: {e:?}"))?;
    let response: web_sys::Response = response
        .dyn_into()
        .map_err(|_| format!("Invalid response for {path}"))?;
    if !response.ok() {
        return Err(format!("GET {url}: HTTP {}", response.status()));
    }
    let buffer = JsFuture::from(
        response
            .array_buffer()
            .map_err(|e| format!("{path}: {e:?}"))?,
    )
    .await
    .map_err(|e| format!("Read {path}: {e:?}"))?;
    let bytes = js_sys::Uint8Array::new(&buffer).to_vec();
    BROWSER_FILES.with(|files| files.borrow_mut().insert(path.into(), bytes.clone()));
    Ok(bytes)
}
#[cfg(target_arch = "wasm32")]
pub async fn preload_browser_maps() -> Result<(), String> {
    for path in [
        "data/Data/AmigaTheOne/mapm1.map",
        "data/Data/AmigaTheOne/mapm2.map",
        "data/Data/AmigaAction/mapm3.map",
    ] {
        let map_path = Path::new(path);
        let map = fetch(path).await?;
        let sprites = fetch(&map_path.with_extension("spt").to_string_lossy()).await?;
        let level = ImportedLevel::parse(&map, &sprites).map_err(|e| format!("{path}: {e}"))?;
        let dir = map_path.parent().ok_or("Map has no directory")?;
        for name in [&level.base, &level.subset] {
            for extension in ["hit", "bht", "swp"] {
                fetch(&dir.join(format!("{name}.{extension}")).to_string_lossy()).await?;
            }
        }
        ImportedLevel::load(map_path, None).map_err(|e| e.to_string())?;
    }
    Ok(())
}
