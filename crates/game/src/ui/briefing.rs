use super::*;
pub(super) fn view(
    commands: &mut Commands,
    root: Entity,
    game: &Game,
    _screen: &State<GameScreen>,
) {
    let progress = game.campaign.progress;
    let roster = game
        .campaign
        .roster
        .iter()
        .filter(|s| s.alive)
        .map(|s| format!("{} {} · {} missions", s.rank(), s.name, s.missions))
        .collect::<Vec<_>>()
        .join("   ");

    label(
        commands,
        root,
        format!(
            "MISSION {} / PHASE {}",
            progress.mission_index + 1,
            progress.phase_index + 1
        ),
        32.,
    );
    label(
        commands,
        root,
        &game.mission_titles[game.campaign.progress.mission_index],
        28.,
    );
    let rules = game.rules[game.index];
    if rules.0 {
        label(commands, root, "Eliminate all enemy soldiers", 20.);
    }
    if rules.1 {
        label(commands, root, "Destroy designated enemy buildings", 20.);
    }
    label(commands, root, &roster, 18.);
    label(
        commands,
        root,
        format!(
            "Grenades {} · Rockets {}",
            game.battle.grenades, game.battle.rockets
        ),
        18.,
    );
    if game.phase_ready {
        button(commands, root, "Deploy squad", Action::Deploy);
    }
    if !game.status.is_empty() {
        label(commands, root, &game.status, 18.);
    }
}
