use super::*;
pub(super) fn view(commands: &mut Commands, root: Entity) {
    label(commands, root, "PAUSED", 40.);
    for (title, action) in [
        ("Resume", Action::Pause),
        ("Restart Phase — losses remain permanent", Action::Retry),
        ("Options", Action::Options),
        ("Main Menu", Action::Menu),
    ] {
        button(commands, root, title, action);
    }
}
