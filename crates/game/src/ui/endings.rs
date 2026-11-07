use super::*;
pub(super) fn view(commands: &mut Commands, root: Entity, game: &Game, screen: &State<GameScreen>) {
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
        if *screen.get() == GameScreen::GameOver {
            "GAME OVER"
        } else {
            "CAMPAIGN COMPLETE"
        },
        48.,
    );
    label(
        commands,
        root,
        format!(
            "Missions completed {} · Casualties {}\nHeroes: {roster}",
            game.campaign.mission,
            game.campaign.memorial.len()
        ),
        22.,
    );
    button(commands, root, "Main Menu", Action::Menu);
    button(commands, root, "New Campaign", Action::New);
}
