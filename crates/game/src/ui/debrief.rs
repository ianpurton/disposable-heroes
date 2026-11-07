use super::*;
pub(super) fn view(
    commands: &mut Commands,
    root: Entity,
    game: &Game,
    _screen: &State<GameScreen>,
) {
    let roster = game
        .campaign
        .roster
        .iter()
        .filter(|s| s.alive)
        .map(|s| format!("{} {} · {} missions", s.rank(), s.name, s.missions))
        .collect::<Vec<_>>()
        .join("   ");

    label(commands, root, "SERVICE RECORD", 40.);
    label(
        commands,
        root,
        &game.mission_titles[game.campaign.progress.mission_index],
        24.,
    );
    label(
        commands,
        root,
        format!("Survivors and updated ranks\n{roster}"),
        18.,
    );
    label(
        commands,
        root,
        format!(
            "Killed in action · {}\n{}",
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
