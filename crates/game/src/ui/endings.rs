use super::*;
pub(super) fn view(commands: &mut Commands, root: Entity, game: &Game, screen: &State<GameScreen>) {
    let survivors: Vec<_> = game
        .campaign
        .roster
        .iter()
        .filter(|s| s.alive)
        .cloned()
        .collect();

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
            "Missions completed {} | Casualties {}",
            game.campaign.mission,
            game.campaign.memorial.len()
        ),
        22.,
    );
    label(commands, root, "SURVIVING HEROES", 16.);
    components::roster(commands, root, &survivors);
    button(commands, root, "Main Menu", Action::Menu);
    button(commands, root, "New Campaign", Action::New);
}
