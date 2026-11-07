use super::*;
pub(super) fn view(
    commands: &mut Commands,
    root: Entity,
    game: &Game,
    _screen: &State<GameScreen>,
) {
    label(commands, root, "TRY AGAIN", 44.);
    label(
        commands,
        root,
        format!(
            "Permanent casualties {} · Reserves {}",
            game.campaign.memorial.len(),
            game.campaign.deployable()
        ),
        22.,
    );
    if game.campaign.deployable() > 0 {
        button(commands, root, "Retry current phase", Action::Retry);
    }
    button(commands, root, "Main Menu", Action::Menu);
}
