use crate::*;
pub(crate) fn objectives(game: &Game) -> String {
    let progress = game.campaign.progress;
    let enemies = game
        .battle
        .units
        .iter()
        .filter(|u| u.enemy && u.alive)
        .count();
    let targets = game
        .battle
        .buildings
        .iter()
        .enumerate()
        .filter(|(i, b)| {
            b.health > 0
                && game
                    .battle
                    .objective_buildings
                    .as_ref()
                    .is_some_and(|ids| ids.contains(i))
        })
        .count();
    let mut lines = vec![format!(
        "M{} / P{} · {}",
        progress.mission_index + 1,
        progress.phase_index + 1,
        game.titles[game.index]
    )];
    if game.battle.kill_objective {
        lines.push(format!("Enemy soldiers remaining: {enemies}"));
    }
    if game.battle.destroy_objective {
        lines.push(format!("Objective structures remaining: {targets}"));
    }
    lines.join("\n")
}
