use super::*;
pub fn label(commands: &mut Commands, parent: Entity, text: impl Into<String>, size: f32) {
    let child = commands
        .spawn((
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
            TextColor(theme::INK),
            bevy::picking::Pickable::IGNORE,
        ))
        .id();
    commands.entity(parent).add_child(child);
}
pub fn button(commands: &mut Commands, parent: Entity, text: &str, action: Action) {
    let child = commands
        .spawn((
            bevy::ui_widgets::Button,
            bevy::picking::hover::Hovered::default(),
            Node {
                padding: UiRect::axes(px(20), px(10)),
                min_width: px(230),
                ..default()
            },
            BackgroundColor(theme::ACCENT),
        ))
        .observe(
            move |_: On<bevy::ui_widgets::Activate>, mut queue: ResMut<Actions>| {
                if queue.0.is_none() {
                    queue.0 = Some(action);
                }
            },
        )
        .id();
    label(commands, child, text, 18.);
    commands.entity(parent).add_child(child);
}
