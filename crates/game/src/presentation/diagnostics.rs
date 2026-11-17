use super::*;
use bevy::dev_tools::diagnostics_overlay::{
    DiagnosticsOverlay, DiagnosticsOverlayPlane, DiagnosticsOverlayPlugin, DiagnosticsOverlayStyle,
    DiagnosticsOverlaySystems,
};

#[derive(Resource)]
pub(super) struct DiagnosticsCamera(pub Entity);

#[derive(Component)]
struct RenderTimings(&'static str);

pub(super) struct BattlefieldDiagnosticsPlugin;

impl Plugin for BattlefieldDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DiagnosticsOverlayPlugin)
            .add_systems(Startup, setup.after(super::setup))
            .add_systems(Update, toggle)
            .add_systems(
                Update,
                update_timings
                    .run_if(bevy::time::common_conditions::on_timer(
                        std::time::Duration::from_secs(1),
                    ))
                    .before(DiagnosticsOverlaySystems::Rebuild),
            );
        // The same collector also feeds benchmark reports. Register it only once.
        app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
    }
}

fn setup(
    mut commands: Commands,
    camera: Res<DiagnosticsCamera>,
    plane: Single<Entity, With<DiagnosticsOverlayPlane>>,
    mut style: ResMut<DiagnosticsOverlayStyle>,
) {
    style.item_font_size = bevy::text::FontSize::Px(12.);
    commands
        .entity(*plane)
        .insert((Visibility::Hidden, UiTargetCamera(camera.0)));
    commands.spawn(DiagnosticsOverlay::fps());
    for (suffix, title, left) in [
        ("elapsed_cpu", "Render CPU (ms)", 300.),
        ("elapsed_gpu", "Render GPU (ms, when supported)", 940.),
    ] {
        // Insert Node after the built-in observer has constructed the window.
        let entity = commands
            .spawn((
                DiagnosticsOverlay::new(title, vec![]),
                RenderTimings(suffix),
            ))
            .id();
        commands
            .entity(entity)
            .entry::<Node>()
            .and_modify(move |mut node| {
                node.left = Val::Px(left);
            });
    }
}

fn toggle(
    keys: Res<ButtonInput<KeyCode>>,
    game: Res<Game>,
    mut plane: Single<&mut Visibility, With<DiagnosticsOverlayPlane>>,
) {
    if game.smoke || game.benchmark || game.combat_preview {
        **plane = Visibility::Hidden;
    } else if keys.just_pressed(KeyCode::F2) {
        **plane = if **plane == Visibility::Hidden {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

fn update_timings(
    diagnostics: Res<DiagnosticsStore>,
    mut panels: Query<(&RenderTimings, &mut DiagnosticsOverlay)>,
) {
    for (timing, mut panel) in &mut panels {
        let mut paths: Vec<_> = diagnostics
            .iter()
            .filter(|d| {
                d.path().as_str().starts_with("render/")
                    && d.path().as_str().ends_with(timing.0)
                    && d.smoothed().is_some()
            })
            .map(|d| d.path().clone())
            .collect();
        paths.sort_by(|a, b| a.as_str().cmp(b.as_str()));
        if panel.items.iter().map(|item| &item.path).ne(paths.iter()) {
            panel.items = paths.into_iter().map(Into::into).collect();
        }
    }
}
