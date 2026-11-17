use super::*;
use bevy::dev_tools::diagnostics_overlay::{
    DiagnosticsOverlay, DiagnosticsOverlayPlane, DiagnosticsOverlayPlugin, DiagnosticsOverlayStyle,
    DiagnosticsOverlaySystems,
};

#[derive(Resource)]
pub(super) struct DiagnosticsCamera(pub Entity);

#[derive(Component)]
struct PerformancePanel;

pub(super) struct BattlefieldDiagnosticsPlugin;

impl Plugin for BattlefieldDiagnosticsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(DiagnosticsOverlayPlugin)
            .add_systems(Startup, setup.after(super::setup))
            .add_systems(Update, (toggle, position_panel.after(super::update_camera)))
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
    commands.spawn((
        DiagnosticsOverlay::new("Performance · render times in ms", vec![]),
        PerformancePanel,
    ));
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

// Keep the window on the right when the window or UI scale changes, while
// allowing the built-in overlay's drag behavior between resizes.
fn position_panel(
    windows: Query<&Window>,
    scale: Res<UiScale>,
    mut previous: Local<Option<Vec2>>,
    mut panel: Query<&mut Node, With<PerformancePanel>>,
) {
    let size = windows
        .single()
        .map(|w| Vec2::new(w.width(), w.height()))
        .unwrap_or(Vec2::new(1920., 1080.))
        / scale.0.max(0.001);
    if *previous == Some(size) {
        return;
    }
    let Ok(mut node) = panel.single_mut() else {
        return;
    };
    node.left = px((size.x - 626.).max(0.));
    node.top = px(16.);
    node.width = px(610.);
    *previous = Some(size);
}

fn update_timings(
    diagnostics: Res<DiagnosticsStore>,
    mut panels: Query<&mut DiagnosticsOverlay, With<PerformancePanel>>,
) {
    // GPU timings explain rendering cost best. CPU render timings are a fallback
    // for backends without timestamp support, not a measure of total CPU time.
    let has_gpu = diagnostics.iter().any(|d| {
        d.path().as_str().starts_with("render/")
            && d.path().as_str().ends_with("/elapsed_gpu")
            && d.smoothed().is_some()
    });
    let suffix = if has_gpu {
        "/elapsed_gpu"
    } else {
        "/elapsed_cpu"
    };
    let mut timings: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.path().as_str().starts_with("render/") && d.path().as_str().ends_with(suffix))
        .filter_map(|d| {
            d.smoothed()
                .filter(|v| v.is_finite())
                .map(|value| (d.path().clone(), value))
        })
        .collect();
    timings.sort_by(|a, b| {
        b.1.total_cmp(&a.1)
            .then_with(|| a.0.as_str().cmp(b.0.as_str()))
    });
    let paths: Vec<_> = [
        FrameTimeDiagnosticsPlugin::FPS,
        FrameTimeDiagnosticsPlugin::FRAME_TIME,
    ]
    .into_iter()
    .chain(timings.into_iter().take(13).map(|(path, _)| path))
    .collect();
    for mut panel in &mut panels {
        if panel.items.iter().map(|item| &item.path).ne(paths.iter()) {
            panel.items = paths
                .iter()
                .cloned()
                .map(|path| {
                    let mut item: bevy::dev_tools::diagnostics_overlay::DiagnosticsOverlayItem =
                        path.into();
                    item.precision = 2;
                    item
                })
                .collect();
        }
    }
}
