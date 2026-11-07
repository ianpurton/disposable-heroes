use super::*;
pub(super) fn view(
    commands: &mut Commands,
    root: Entity,
    game: &Game,
    _screen: &State<GameScreen>,
) {
    label(commands, root, "DISPOSABLE HEROES", 48.);
    button(commands, root, "New Campaign", Action::New);
    if Campaign::load(&game.save).is_ok() {
        button(commands, root, "Continue Campaign", Action::Continue);
    } else {
        label(commands, root, "Continue Campaign - no save", 18.);
    }
    button(commands, root, "Options", Action::Options);
    button(commands, root, "Quit", Action::Quit);
}
