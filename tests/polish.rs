use disposable_heroes::{
    environment::{JungleCatalog, thickets},
    importer::{ImportedLevel, SpriteRecord},
    navigation::Navigation,
    simulation::*,
    tuning::CombatTuning,
};
use std::path::Path;
fn original() -> ImportedLevel {
    ImportedLevel::load(Path::new("data/Data/AmigaTheOne/mapm1.map"), None).unwrap()
}
fn arena() -> ImportedLevel {
    let mut m = original();
    m.width = 8;
    m.height = 8;
    m.tiles = vec![0; 64];
    m.terrain.hit[0] = 0;
    m.terrain.hit[1] = 3;
    m.terrain.swap[1] = 0;
    m.structures.clear();
    m.sprites = vec![sprite(20, 60, 0), sprite(100, 60, 5)];
    m
}
fn sprite(x: i16, y: i16, kind: u16) -> SpriteRecord {
    SpriteRecord {
        direction: 0,
        reserved: 0,
        stored_x: x - 16,
        y,
        kind,
    }
}
#[test]
fn reviewed_catalog_covers_every_first_mission_tile() {
    assert!(JungleCatalog::bundled().reviewed(&original()));
}
#[test]
fn thickets_cover_blockers_once_without_closing_gaps() {
    let m = original();
    let patches = thickets(&m, &JungleCatalog::bundled());
    let mut cover = vec![0u8; m.width * 8 * m.height * 8];
    for patch in patches {
        for y in (patch.y as usize..(patch.y + patch.depth) as usize).step_by(2) {
            for x in (patch.x as usize..(patch.x + patch.width) as usize).step_by(2) {
                assert_eq!(m.terrain_at(x as f32 + 1., y as f32 + 1.), 3);
                cover[y / 2 * m.width * 8 + x / 2] += 1;
            }
        }
    }
    for y in 0..m.height * 8 {
        for x in 0..m.width * 8 {
            assert_eq!(
                cover[y * m.width * 8 + x],
                u8::from(m.terrain_at(x as f32 * 2. + 1., y as f32 * 2. + 1.) == 3)
            );
        }
    }
}
#[test]
fn paths_go_around_walls_without_cutting_corners() {
    let mut m = arena();
    for y in 1..7 {
        m.tiles[y * 8 + 3] = 1;
    }
    let nav = Navigation::build(&m, &[]);
    let start = Point::new(20., 60.);
    let target = Point::new(100., 60.);
    let route = nav.route(start, target).unwrap();
    assert!(route.len() > 1);
    let mut previous = start;
    for point in route {
        assert!(nav.segment(previous, point));
        previous = point;
    }
    assert!(previous.distance(target) < 3.);
}
#[test]
fn unreachable_orders_stop_on_reachable_side() {
    let mut m = arena();
    for y in 0..8 {
        m.tiles[y * 8 + 3] = 1;
    }
    let nav = Navigation::build(&m, &[]);
    let route = nav
        .route(Point::new(20., 60.), Point::new(100., 60.))
        .unwrap();
    assert!(route.last().unwrap().x < 48.);
    assert!(
        nav.route(Point::new(20., 60.), Point::new(-50., 20.))
            .is_some()
    );
}
#[test]
fn insufficient_clearance_is_not_a_route() {
    let mut m = arena();
    for y in 0..8 {
        m.tiles[y * 8 + 3] = 1;
    }
    m.terrain.hit[2] = 0x8003u16 as i16;
    m.terrain.masks[2] = [0, 0, 0, 255, 0, 0, 0, 0];
    m.tiles[3 * 8 + 3] = 2;
    let nav = Navigation::build(&m, &[]);
    let route = nav
        .route(Point::new(20., 60.), Point::new(100., 60.))
        .unwrap();
    assert!(route.last().unwrap().x < 48.);
}
#[test]
fn visibility_and_ai_do_not_fire_through_walls() {
    let mut m = arena();
    for y in 0..8 {
        m.tiles[y * 8 + 3] = 1;
    }
    assert!(!line_of_sight(
        &m,
        &[],
        Point::new(20., 60.),
        Point::new(100., 60.)
    ));
    let mut b = Battle::new(m, 7);
    for _ in 0..100 {
        b.tick(0.02);
    }
    assert!(b.projectiles.is_empty());
    assert!(b.units.iter().all(|u| !u.fire));
}
#[test]
fn enemies_react_then_fire_and_remember_briefly() {
    let mut b = Battle::new(arena(), 7);
    b.tick(0.02);
    assert_eq!(b.units[1].ai, AiState::Reacting);
    assert!(b.projectiles.is_empty());
    for _ in 0..40 {
        b.tick(0.02);
    }
    assert!(b.effects.iter().any(|e| matches!(
        e,
        Effect::Shot {
            weapon: Weapon::Rifle,
            ..
        }
    )));
    for y in 0..8 {
        b.map.tiles[y * 8 + 3] = 1;
    }
    b.navigation = Navigation::build(&b.map, &[]);
    b.projectiles.clear();
    b.tick(0.02);
    assert_eq!(b.units[1].ai, AiState::Searching);
    assert!(!b.units[1].fire);
    for _ in 0..80 {
        b.tick(0.02);
    }
    assert_eq!(b.units[1].ai, AiState::Idle);
}
#[test]
fn aggression_changes_reaction_cadence_and_accuracy() {
    let mut low = CombatTuning::default();
    low.aggression(&[2, 4]);
    let mut high = CombatTuning::default();
    high.aggression(&[10, 12]);
    assert!(high.reaction_seconds < low.reaction_seconds);
    assert!(high.enemy_cooldown < low.enemy_cooldown);
    assert!(high.aim_error < low.aim_error);
}
#[test]
fn destruction_rebuilds_navigation() {
    let mut m = arena();
    for y in 0..8 {
        m.tiles[y * 8 + 3] = 1;
    }
    let mut b = Battle::new(m, 7);
    b.buildings.push(Building {
        pos: Point::new(56., 56.),
        health: 1,
        tiles: vec![3 * 8 + 3],
        spawn: 10.,
        vehicle: false,
    });
    assert!(!b.navigation.clear(Point::new(56., 56.)));
    b.explode(Point::new(56., 56.));
    assert!(b.navigation.clear(Point::new(56., 56.)));
}
#[test]
fn squad_routes_follow_and_survive_leader_death() {
    let mut m = arena();
    m.sprites = vec![sprite(20, 60, 0), sprite(20, 70, 0), sprite(110, 110, 5)];
    let mut b = Battle::new(m, 7);
    b.command(Command::Move(Point::new(70., 20.)));
    assert_eq!(b.units[1].follow, Some(b.units[0].id));
    b.tuning.reaction_seconds = 1000.;
    for _ in 0..30 {
        b.tick(0.02);
    }
    assert!(b.units[0].pos.distance(Point::new(20., 60.)) > 5.);
    b.units[0].alive = false;
    let previous = b.units[1].pos;
    for _ in 0..30 {
        b.tick(0.02);
    }
    assert_eq!(b.units[1].follow, None);
    assert!(b.units[1].pos.distance(previous) > 1.);
}

#[test]
fn original_squad_can_route_from_spawn_around_the_jungle() {
    let mut battle = Battle::new(original(), 1993);
    battle.tuning.reaction_seconds = 1000.;
    let destination = battle.units.iter().find(|u| u.enemy).unwrap().pos;
    let source = battle.units[0].pos;
    assert!(
        battle.navigation.clear(source),
        "player spawn must allow clearance"
    );
    battle.command(Command::Move(destination));
    assert!(
        battle.units[0].path.len() > 1,
        "route should go around the central thicket"
    );
    for _ in 0..500 {
        battle.tick(0.02);
    }
    assert!(
        battle.units[0].pos.distance(destination) < 6.,
        "leader failed to arrive: {:?}",
        battle.units[0].pos
    );
    assert!(
        battle.units[1].pos.distance(destination) < 20.,
        "follower failed to arrive: {:?}",
        battle.units[1].pos
    );
}
