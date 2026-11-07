use super::*;
pub(super) fn view(
    commands: &mut Commands,
    root: Entity,
    game: &Game,
    _screen: &State<GameScreen>,
) {
    let progress = game.campaign.progress;
    let survivors: Vec<_> = game
        .campaign
        .roster
        .iter()
        .filter(|s| s.alive)
        .cloned()
        .collect();

    label(commands, root, "THE HILL", 40.);
    label(
        commands,
        root,
        format!(
            "Mission {} | Available {} | Fallen {}",
            progress.mission_index + 1,
            game.campaign.deployable(),
            game.campaign.memorial.len()
        ),
        22.,
    );
    label(commands, root, "AVAILABLE SOLDIERS", 16.);
    components::roster(commands, root, &survivors);
    label(
        commands,
        root,
        game.campaign
            .memorial
            .iter()
            .map(|s| format!("+ {}", s.name))
            .collect::<Vec<_>>()
            .join("   "),
        18.,
    );
    button(commands, root, "Deploy", Action::Hill);
}
