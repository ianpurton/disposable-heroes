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

    label(commands, root, "THE HILL", 40.);
    label(
        commands,
        root,
        format!(
            "Mission {} · Available {} · Fallen {}",
            progress.mission_index + 1,
            game.campaign.deployable(),
            game.campaign.memorial.len()
        ),
        22.,
    );
    label(commands, root, &roster, 18.);
    label(
        commands,
        root,
        game.campaign
            .memorial
            .iter()
            .map(|s| format!("† {}", s.name))
            .collect::<Vec<_>>()
            .join("   "),
        18.,
    );
    button(commands, root, "Deploy", Action::Hill);
}
