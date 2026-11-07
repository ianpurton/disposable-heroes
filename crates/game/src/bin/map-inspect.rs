use disposable_heroes::importer::ImportedLevel;
use std::path::Path;
fn main() {
    let args: Vec<_> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("Usage: map-inspect FILE.map [TILESET_DIRECTORY]");
        std::process::exit(2);
    }
    match ImportedLevel::load(Path::new(&args[0]), args.get(1).map(Path::new)) {
        Ok(m) => {
            let mut counts = std::collections::BTreeMap::new();
            for s in &m.sprites {
                *counts.entry(s.kind).or_insert(0) += 1;
            }
            println!(
                "{} x {} tiles | {} / {}\n{} sprites: {:?}\n{} structures | unsupported: {:?}",
                m.width,
                m.height,
                m.base,
                m.subset,
                m.sprites.len(),
                counts,
                m.structures.len(),
                m.unsupported_sprites()
            );
        }
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    }
}
