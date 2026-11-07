use disposable_heroes::{campaign::Campaign, importer::ImportedLevel, simulation::*};
use std::path::Path;
fn fixture() -> ImportedLevel {
    ImportedLevel::load(
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/Data/AmigaTheOne/mapm1.map"
        )),
        None,
    )
    .expect("Run python3 scripts/fetch_fixtures.py first")
}
#[test]
fn real_sample() {
    let m = fixture();
    assert_eq!((m.width, m.height, m.sprites.len()), (19, 15, 7));
    assert_eq!(m.sprites[0].x(), 155);
    assert_eq!(m.sprites[0].y, 213);
}
#[test]
fn negative_coordinates() {
    let m = ImportedLevel::load(
        Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/Data/Custom/Maps/Segras First.map"
        )),
        Some(Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../data/Data/AmigaTheOne"
        ))),
    )
    .unwrap();
    assert_eq!(m.sprites[0].stored_x, -2);
    assert_eq!(m.sprites[0].x(), 14);
    assert_eq!((m.width, m.height), (20, 15));
}
#[test]
fn truncated_inputs() {
    assert!(ImportedLevel::parse(&[0; 95], &[]).is_err());
    let b = std::fs::read(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/Data/AmigaTheOne/mapm1.map"
    ))
    .unwrap();
    assert!(ImportedLevel::parse(&b, &[0; 9]).is_err());
    assert!(ImportedLevel::parse(&b[..b.len() - 1], &[]).is_err());
}
#[test]
fn masks_and_flags() {
    let mut m = fixture();
    m.tiles[0] = 0x8001;
    m.terrain.hit[1] = 0x8036u16 as i16;
    m.terrain.masks[1] = [128; 8];
    assert_eq!(m.terrain_at(0., 0.), 3);
    assert_eq!(m.terrain_at(2., 0.), 6);
    assert_eq!(m.terrain_at(-1., 0.), 3);
}
#[test]
fn all_pinned_maps_parse() {
    let manifest: serde_json::Value = serde_json::from_slice(
        &std::fs::read(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/fixtures-manifest.json"
        ))
        .unwrap(),
    )
    .unwrap();
    for (file, _) in manifest["files"].as_object().unwrap() {
        if file.ends_with(".map") {
            let path = Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../data")).join(file);
            let root = if file.contains("Custom") {
                Some(Path::new(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../data/Data/AmigaTheOne"
                )))
            } else {
                None
            };
            let m = ImportedLevel::parse(
                &std::fs::read(&path).unwrap(),
                &std::fs::read(path.with_extension("spt")).unwrap(),
            )
            .unwrap();
            if !m.base.starts_with("des") {
                ImportedLevel::load(&path, root).unwrap();
            }
            assert_eq!(m.tiles.len(), m.width * m.height);
        }
    }
}
#[test]
fn split_join_and_leader_death() {
    let mut b = Battle::new(fixture(), 1);
    let ids: Vec<_> = b.units.iter().filter(|u| !u.enemy).map(|u| u.id).collect();
    b.command(Command::Select(ids[0]));
    b.command(Command::Split);
    assert_eq!(b.active_group, 1);
    b.command(Command::Join);
    assert!(b.units.iter().filter(|u| !u.enemy).all(|u| u.group == 1));
    b.units[0].alive = false;
    b.command(Command::Group(1));
    b.command(Command::Move(Point::new(150., 180.)));
    assert_ne!(b.units[1].pos, b.units[1].target);
}
#[test]
fn explosives_need_ammo() {
    let mut b = Battle::new(fixture(), 2);
    b.units[0].cooldown = 0.;
    b.command(Command::Explosive(Point::new(150., 150.)));
    assert!(b.projectiles.is_empty());
    b.grenades = 1;
    b.command(Command::Explosive(Point::new(150., 150.)));
    assert_eq!(b.grenades, 0);
    assert_eq!(b.projectiles.len(), 1);
}
#[test]
fn victory_defeat() {
    let mut b = Battle::new(fixture(), 3);
    for u in &mut b.units {
        if u.enemy {
            u.alive = false;
        }
    }
    b.tick(0.02);
    assert_eq!(b.outcome, Outcome::Victory);
    let mut b = Battle::new(fixture(), 3);
    for u in &mut b.units {
        if !u.enemy {
            u.alive = false;
        }
    }
    b.tick(0.02);
    assert_eq!(b.outcome, Outcome::Defeat);
}
#[test]
fn campaign_snapshot_retry() {
    let c = Campaign::default();
    let retry = c.clone();
    let mut progress = c;
    progress.complete(&[0]);
    assert!(!progress.roster[0].alive);
    assert!(retry.roster[0].alive);
    assert_eq!(progress.memorial.len(), 1);
}
#[test]
fn deterministic_combat() {
    let mut a = Battle::new(fixture(), 7);
    let mut b = a.clone();
    for _ in 0..300 {
        a.tick(0.02);
        b.tick(0.02);
    }
    assert_eq!(a.outcome, b.outcome);
    for (a, b) in a.units.iter().zip(&b.units) {
        assert_eq!(a.pos, b.pos);
        assert_eq!(a.alive, b.alive);
    }
}

#[test]
fn building_destruction_changes_collision_once() {
    let mut battle = Battle::new(fixture(), 1993);
    battle.map.tiles[0] = 1;
    battle.map.terrain.swap[1] = 2;
    battle.map.terrain.hit[1] = 3;
    battle.map.terrain.hit[2] = 0;
    battle.buildings.push(Building {
        pos: Point::new(8., 8.),
        health: 1,
        tiles: vec![0],
        spawn: 10.,
        vehicle: false,
    });
    battle.explode(Point::new(8., 8.));
    assert_eq!(battle.buildings[0].health, 0);
    assert!(battle.map.walkable(8., 8.));
    battle.explode(Point::new(8., 8.));
    assert_eq!(battle.buildings[0].health, 0);
}

#[test]
fn atomic_campaign_round_trip() {
    let directory = std::env::temp_dir().join(format!("dh-save-test-{}", std::process::id()));
    let path = directory.join("campaign.json");
    let mut campaign = Campaign::default();
    campaign.complete(&[0]);
    campaign.save(&path).unwrap();
    let loaded = Campaign::load(&path).unwrap();
    assert_eq!(loaded.mission, 1);
    assert!(!loaded.roster[0].alive);
    assert_eq!(loaded.memorial[0].name, "Jools");
    assert!(!path.with_extension("tmp").exists());
    std::fs::remove_dir_all(directory).unwrap();
}
