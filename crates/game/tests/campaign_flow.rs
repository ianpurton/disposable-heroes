use disposable_heroes::{campaign::*, importer::ImportedLevel, simulation::*};
use std::path::Path;
fn battle() -> Battle {
    Battle::new(
        ImportedLevel::load(
            Path::new(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../data/Data/AmigaTheOne/mapm1.map"
            )),
            None,
        )
        .unwrap(),
        1993,
    )
}
#[test]
fn last_enemy_wins_without_structure_objective() {
    let mut b = battle();
    b.configure_objectives(&[ObjectiveKind::KillAllEnemies])
        .unwrap();
    for u in b.units.iter_mut().filter(|u| u.enemy) {
        u.alive = false;
    }
    b.tick(0.02);
    assert_eq!(b.outcome, Outcome::Victory);
}
#[test]
fn required_structures_block_then_complete() {
    let mut b = battle();
    b.configure_objectives(&[
        ObjectiveKind::KillAllEnemies,
        ObjectiveKind::DestroyEnemyBuildings,
    ])
    .unwrap();
    for u in b.units.iter_mut().filter(|u| u.enemy) {
        u.alive = false;
    }
    b.buildings.push(Building {
        pos: Point::new(1000., 1000.),
        health: 1,
        tiles: vec![],
        spawn: f32::INFINITY,
        vehicle: false,
    });
    b.objective_buildings = Some(vec![b.buildings.len() - 1]);
    b.tick(0.02);
    assert_eq!(b.outcome, Outcome::Playing);
    for x in &mut b.buildings {
        x.health = 0;
    }
    b.tick(0.02);
    assert_eq!(b.outcome, Outcome::Victory);
}
#[test]
fn non_objective_structure_does_not_block() {
    let mut b = battle();
    b.buildings.push(Building {
        pos: Point::new(1000., 1000.),
        health: 1,
        tiles: vec![],
        spawn: f32::INFINITY,
        vehicle: false,
    });
    b.objective_buildings = Some(vec![]);
    for u in b.units.iter_mut().filter(|u| u.enemy) {
        u.alive = false;
    }
    b.tick(0.02);
    assert_eq!(b.outcome, Outcome::Victory);
}
#[test]
fn destroyed_buildings_never_spawn() {
    let mut b = battle();
    b.buildings.push(Building {
        pos: b.units[0].pos,
        health: 0,
        tiles: vec![],
        spawn: -1.,
        vehicle: false,
    });
    for x in &mut b.buildings {
        x.health = 0;
        x.spawn = -1.;
    }
    let count = b.units.len();
    b.tick(0.02);
    assert_eq!(count, b.units.len());
}
#[test]
fn simultaneous_elimination_is_defeat_and_terminal() {
    let mut b = battle();
    for u in &mut b.units {
        u.alive = false;
    }
    b.tick(0.02);
    assert_eq!(b.outcome, Outcome::Defeat);
    b.tick(1.);
    assert_eq!(b.outcome, Outcome::Defeat);
}
#[test]
fn unsupported_and_empty_objectives_rejected() {
    let mut b = battle();
    for kind in [
        ObjectiveKind::RescueHostages,
        ObjectiveKind::ProtectAllCivilians,
        ObjectiveKind::KidnapEnemyLeader,
        ObjectiveKind::DestroyFactory,
        ObjectiveKind::DestroyComputer,
        ObjectiveKind::GetCivilianHome,
    ] {
        assert!(b.configure_objectives(&[kind]).is_err());
    }
    assert!(b.configure_objectives(&[]).is_err());
}
#[test]
fn casualties_permanent_idempotent_and_reserve_finite() {
    let mut c = Campaign::default();
    let count = c.deployable();
    c.commit_casualties(&[0, 1]);
    c.commit_casualties(&[0, 1]);
    assert_eq!(c.memorial.len(), 2);
    assert_eq!(c.deployable(), count - 2);
    c.commit_casualties(&(0..count as u32).collect::<Vec<_>>());
    assert_eq!(c.deployable(), 0);
}
#[test]
fn promotions_once_and_only_deployed_survivors() {
    let mut c = Campaign::default();
    c.veterans = vec![0, 1];
    c.commit_casualties(&[1]);
    c.commit_mission();
    c.commit_mission();
    assert_eq!(c.mission, 1);
    assert_eq!(c.roster[0].missions, 1);
    assert_eq!(c.roster[1].missions, 0);
    assert_eq!(c.roster[2].missions, 0);
}
#[test]
fn save_roundtrip_preserves_progress_losses_and_commits() {
    let path = std::env::temp_dir().join(format!("heroes-flow-{}.json", std::process::id()));
    let mut c = Campaign::default();
    c.progress = CampaignProgress {
        mission_index: 1,
        phase_index: 2,
    };
    c.commit_casualties(&[0]);
    c.veterans = vec![2];
    c.commit_mission();
    c.save(&path).unwrap();
    let restored = Campaign::load(&path).unwrap();
    assert_eq!(restored.progress, c.progress);
    assert_eq!(restored.memorial[0].id, 0);
    assert_eq!(restored.roster[2].missions, 1);
    assert_eq!(restored.committed, c.committed);
    std::fs::remove_file(path).unwrap();
}

#[test]
fn rejected_objectives_cannot_win_even_on_empty_map() {
    let mut b = battle();
    assert!(
        b.configure_objectives(&[ObjectiveKind::RescueHostages])
            .is_err()
    );
    for u in b.units.iter_mut().filter(|u| u.enemy) {
        u.alive = false;
    }
    b.tick(0.02);
    assert_eq!(b.outcome, Outcome::Defeat);
}
#[test]
fn version_one_flat_progress_is_migrated_explicitly() {
    let mut c = Campaign {
        version: 1,
        mission: 3,
        ..Campaign::default()
    };
    c.migrate_progress(&[2, 3, 1]);
    assert_eq!(
        c.progress,
        CampaignProgress {
            mission_index: 1,
            phase_index: 1
        }
    );
    assert_eq!(c.version, 2);
    c.migrate_progress(&[1, 1, 1]);
    assert_eq!(
        c.progress,
        CampaignProgress {
            mission_index: 1,
            phase_index: 1
        }
    );
}
