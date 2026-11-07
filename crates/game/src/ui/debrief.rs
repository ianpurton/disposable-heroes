use super::*;
pub(super) fn view(
    commands: &mut Commands,
    root: Entity,
    game: &Game,
    _screen: &State<GameScreen>,
) {
    let survivors: Vec<_> = game
        .campaign
        .roster
        .iter()
        .filter(|s| s.alive)
        .cloned()
        .collect();

    label(commands, root, "SERVICE RECORD", 40.);
    label(
        commands,
        root,
        &game.mission_titles[game.campaign.progress.mission_index],
        24.,
    );
    label(commands, root, "SURVIVORS | UPDATED RANKS", 16.);
    components::roster(commands, root, &survivors);
    label(
        commands,
        root,
        format!(
            "Killed in action | {}\n{}",
            game.campaign.memorial.len(),
            game.campaign
                .memorial
                .iter()
                .map(|s| s.name.clone())
                .collect::<Vec<_>>()
                .join(", ")
        ),
        18.,
    );
    button(commands, root, "Continue", Action::Debrief);
}
