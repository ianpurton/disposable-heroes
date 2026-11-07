use super::*;
pub(super) fn view(commands: &mut Commands, root: Entity, settings: &Settings) {
    label(commands, root, "OPTIONS", 36.);
    for (title, action) in [
        (
            format!("Master {:.0}%", settings.master * 100.),
            Action::Master,
        ),
        (
            format!("Music {:.0}%", settings.music * 100.),
            Action::Music,
        ),
        (
            format!("Effects {:.0}%", settings.effects * 100.),
            Action::Effects,
        ),
        ("Fullscreen / Windowed".into(), Action::Fullscreen),
        ("Resolution 1280 / 1920".into(), Action::Resolution),
        ("Back".into(), Action::Options),
    ] {
        button(commands, root, &title, action);
    }
}
