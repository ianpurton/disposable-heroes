use super::*;
pub fn label(commands: &mut Commands, parent: Entity, text: impl Into<String>, size: f32) {
    let child = commands
        .spawn((
            Node {
                width: percent(100),
                min_width: px(0),
                flex_shrink: 0.,
                ..default()
            },
            Text::new(text),
            TextLayout::new(Justify::Center, LineBreak::WordBoundary),
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
                width: percent(100),
                max_width: px(340),
                min_width: px(0),
                flex_shrink: 0.,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(px(3.)),

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

/// Modal content stays readable on wide displays while shrinking with its overlay.
pub fn dialog(commands: &mut Commands, overlay: Entity, result: bool) -> Entity {
    let panel = commands
        .spawn((
            Node {
                width: percent(100),
                max_width: px(if result { 960. } else { 720. }),
                min_width: px(0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                row_gap: px(16),
                padding: UiRect::all(px(32)),
                border: UiRect::all(px(if result { 0. } else { 1. })),
                border_radius: BorderRadius::all(px(5.)),
                ..default()
            },
            BackgroundColor(if result { Color::NONE } else { theme::PANEL }),
            BorderColor::all(Color::srgb(0.28, 0.32, 0.22)),
        ))
        .id();
    commands.entity(overlay).add_child(panel);
    panel
}

/// Individual entries wrap into two tidy columns instead of one screen-wide line.
pub fn roster(
    commands: &mut Commands,
    parent: Entity,
    soldiers: &[disposable_heroes::campaign::Soldier],
) {
    if soldiers.is_empty() {
        label(commands, parent, "None", 16.);
        return;
    }
    let list = commands
        .spawn(Node {
            width: percent(100),
            min_width: px(0),
            flex_wrap: FlexWrap::Wrap,
            column_gap: percent(4),
            row_gap: px(8),
            ..default()
        })
        .id();
    commands.entity(parent).add_child(list);
    for soldier in soldiers {
        let row = commands
            .spawn((
                Node {
                    width: percent(48),
                    min_width: px(0),
                    padding: UiRect::axes(px(12), px(8)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.16, 0.20, 0.13, 0.65)),
            ))
            .id();
        commands.entity(list).add_child(row);
        let text = format!(
            "{}  {}  |  {} missions",
            soldier.rank(),
            soldier.name,
            soldier.missions
        );
        label(commands, row, text, 16.);
    }
}
