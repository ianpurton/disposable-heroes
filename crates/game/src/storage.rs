//! Platform persistence; browser saves are scoped to this game's origin.
use std::{io, path::Path};
#[cfg(not(target_arch = "wasm32"))]
pub fn read(path: &Path) -> io::Result<Vec<u8>> {
    std::fs::read(path)
}
#[cfg(not(target_arch = "wasm32"))]
pub fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent)?;
    }
    let temp = path.with_extension("tmp");
    let mut file = std::fs::File::create(&temp)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    std::fs::rename(temp, path)
}
#[cfg(target_arch = "wasm32")]
fn storage() -> io::Result<web_sys::Storage> {
    web_sys::window()
        .ok_or_else(|| io::Error::other("No browser window"))?
        .local_storage()
        .map_err(|e| io::Error::other(format!("Browser storage: {e:?}")))?
        .ok_or_else(|| io::Error::other("Browser storage unavailable"))
}
#[cfg(target_arch = "wasm32")]
fn key(path: &Path) -> String {
    format!("disposable-heroes:{}", path.display())
}
#[cfg(target_arch = "wasm32")]
pub fn read(path: &Path) -> io::Result<Vec<u8>> {
    storage()?
        .get_item(&key(path))
        .map_err(|e| io::Error::other(format!("Read save: {e:?}")))?
        .map(String::into_bytes)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "No saved data"))
}
#[cfg(target_arch = "wasm32")]
pub fn write(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let text = std::str::from_utf8(bytes).map_err(io::Error::other)?;
    storage()?
        .set_item(&key(path), text)
        .map_err(|e| io::Error::other(format!("Write save: {e:?}")))
}
