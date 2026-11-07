mod briefing;
mod debrief;
mod endings;
pub(crate) mod hud;
mod main_menu;
mod options;
mod pause;
mod recruitment;
mod results;
use crate::{game_flow::*, *};
mod components;
mod theme;
use components::{button, label};
#[derive(Clone, Copy)]
pub enum Action {
    New,
    Continue,
    Hill,
    Deploy,
    Debrief,
    Retry,
    Menu,
    Quit,
    Pause,
    Options,
    Master,
    Music,
    Effects,
    Fullscreen,
    Resolution,
}
#[derive(Resource, Default)]
pub struct Actions(Option<Action>);
#[derive(Component)]
struct ScreenRoot;
#[derive(Resource, serde::Serialize, serde::Deserialize)]
struct Settings {
    version: u32,
    master: f32,
    music: f32,
    effects: f32,
    #[serde(default)]
    fullscreen: bool,
    #[serde(default = "default_width")]
    width: u32,
}
fn default_width() -> u32 {
    1920
}
impl Default for Settings {
    fn default() -> Self {
        Self {
            version: 1,
            master: 0.8,
            music: 0.5,
            effects: 0.8,
            fullscreen: false,
            width: 1920,
        }
    }
}
pub struct GameUiPlugin;
impl Plugin for GameUiPlugin {
    fn build(&self, app: &mut App) {
        let settings = std::fs::read("saves/settings.json")
            .ok()
            .and_then(|s| serde_json::from_slice::<Settings>(&s).ok())
            .filter(|s| s.version == 1)
            .unwrap_or_default();
        app.init_resource::<Actions>()
            .insert_resource(settings)
            .add_systems(Startup, apply_window)
            .add_systems(Update, (actions, draw, apply_audio).chain());
    }
}
fn actions(
    mut queue: ResMut<Actions>,
    mut game: ResMut<Game>,
    screen: Res<State<GameScreen>>,
    mut next: ResMut<NextState<GameScreen>>,
    mut flow: ResMut<Flow>,
    mut exit: MessageWriter<AppExit>,
    mut settings: ResMut<Settings>,
    mut windows: Query<&mut Window>,
) {
    let Some(action) = queue.0.take() else {
        return;
    };
    match action {
        Action::New => {
            game.campaign = Campaign::default();
            flow.paused = false;
            flow.options = false;
            save(&mut game);
            next.set(GameScreen::Recruitment);
        }
        Action::Continue if *screen.get() == GameScreen::MainMenu => {
            match Campaign::load(&game.save) {
                Ok(mut c) => {
                    c.migrate_progress(&game.phase_counts);
                    game.campaign = c;
                    next.set(
                        if game.campaign.progress.mission_index >= game.phase_counts.len() {
                            GameScreen::CampaignComplete
                        } else if game.campaign.deployable() == 0 {
                            GameScreen::GameOver
                        } else {
                            match game.campaign.checkpoint.as_str() {
                                "PhaseComplete" => GameScreen::PhaseComplete,
                                "MissionComplete" => GameScreen::MissionComplete,
                                "Debrief" => GameScreen::Debrief,
                                "Briefing" => GameScreen::Briefing,
                                "TryAgain" => GameScreen::TryAgain,
                                _ => GameScreen::Recruitment,
                            }
                        },
                    );
                }
                Err(e) => game.status = e.to_string(),
            }
        }
        Action::Hill if *screen.get() == GameScreen::Recruitment => {
            next.set(GameScreen::Briefing);
        }
        Action::Deploy
            if *screen.get() == GameScreen::Briefing && !game.reload && game.phase_ready =>
        {
            next.set(GameScreen::Playing);
        }
        Action::Debrief if *screen.get() == GameScreen::Debrief => {
            game.campaign.checkpoint = "Recruitment".into();
            game.campaign.progress.mission_index += 1;
            game.campaign.progress.phase_index = 0;
            if game.campaign.progress.mission_index >= game.phase_counts.len() {
                next.set(GameScreen::CampaignComplete);
            } else {
                next.set(GameScreen::Recruitment);
            }
            save(&mut game);
        }
        Action::Retry if game.campaign.deployable() > 0 => {
            casualties(&mut game);
            save(&mut game);
            flow.paused = false;
            if game.campaign.deployable() > 0 {
                next.set(GameScreen::Briefing);
            } else {
                next.set(GameScreen::GameOver);
            }
        }
        Action::Menu => {
            if *screen.get() == GameScreen::Playing {
                casualties(&mut game);
                game.campaign.checkpoint = "Briefing".into();
                save(&mut game);
            }
            flow.paused = false;
            flow.options = false;
            next.set(GameScreen::MainMenu);
        }
        Action::Quit => {
            if cfg!(target_arch = "wasm32") {
                next.set(GameScreen::MainMenu);
            } else {
                exit.write(AppExit::Success);
            }
        }
        Action::Pause => flow.paused = false,
        Action::Options => flow.options = !flow.options,
        Action::Master => settings.master = (settings.master + 0.1) % 1.1,
        Action::Music => settings.music = (settings.music + 0.1) % 1.1,
        Action::Effects => settings.effects = (settings.effects + 0.1) % 1.1,
        Action::Fullscreen => {
            settings.fullscreen = !settings.fullscreen;
            for mut w in &mut windows {
                w.mode = if w.mode == bevy::window::WindowMode::Windowed {
                    bevy::window::WindowMode::BorderlessFullscreen(
                        bevy::window::MonitorSelection::Current,
                    )
                } else {
                    bevy::window::WindowMode::Windowed
                };
            }
        }
        Action::Resolution => {
            settings.width = if settings.width == 1920 { 1280 } else { 1920 };
            for mut w in &mut windows {
                let size = if w.width() > 1280. {
                    (1280, 720)
                } else {
                    (1920, 1080)
                };
                w.resolution.set(size.0 as f32, size.1 as f32);
            }
        }
        _ => {}
    }
    if matches!(
        action,
        Action::Master | Action::Music | Action::Effects | Action::Fullscreen | Action::Resolution
    ) && settings.is_changed()
    {
        let _ = std::fs::create_dir_all("saves");
        if let Ok(bytes) = serde_json::to_vec(&*settings) {
            let _ = std::fs::write("saves/settings.tmp", bytes)
                .and_then(|_| std::fs::rename("saves/settings.tmp", "saves/settings.json"));
        }
    }
}
fn draw(
    mut commands: Commands,
    roots: Query<Entity, With<ScreenRoot>>,
    screen: Res<State<GameScreen>>,
    flow: Res<Flow>,
    game: Res<Game>,
    settings: Res<Settings>,
    mut previous: Local<String>,
    cameras: Query<Entity, With<Camera2d>>,
) {
    if game.smoke {
        return;
    }
    let key = format!(
        "{:?}:{}:{}:{}:{}:{}:{}",
        screen.get(),
        flow.paused,
        flow.options,
        flow.failure_message,
        settings.master,
        settings.music,
        settings.effects
    );
    let key = format!("{key}:{}:{}", game.index, game.reload);
    if *previous == key {
        return;
    }
    *previous = key;
    for e in &roots {
        commands.entity(e).despawn();
    }
    if *screen.get() == GameScreen::Playing && !flow.paused && !flow.options {
        return;
    }
    let result = matches!(
        screen.get(),
        GameScreen::PhaseComplete | GameScreen::MissionComplete
    ) || flow.failure_message;
    let root = commands
        .spawn((
            ScreenRoot,
            Node {
                position_type: PositionType::Absolute,
                width: percent(100),
                height: percent(100),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                padding: UiRect::all(px(24)),
                ..default()
            },
            GlobalZIndex(100),
            BackgroundColor(Color::srgba(
                0.01,
                0.02,
                0.015,
                if result { 0.28 } else { 0.62 },
            )),
        ))
        .id();
    if let Ok(camera) = cameras.single() {
        commands.entity(root).insert(UiTargetCamera(camera));
    }
    let root = components::dialog(&mut commands, root, result);
    if flow.options {
        options::view(&mut commands, root, &settings);
        return;
    }
    if flow.paused {
        pause::view(&mut commands, root);
        return;
    }
    match screen.get() {
        GameScreen::MainMenu => main_menu::view(&mut commands, root, &game, &screen),
        GameScreen::Recruitment => recruitment::view(&mut commands, root, &game, &screen),
        GameScreen::Briefing => briefing::view(&mut commands, root, &game, &screen),
        GameScreen::PhaseComplete => label(&mut commands, root, "PHASE COMPLETE", 56.),
        GameScreen::MissionComplete => label(&mut commands, root, "MISSION COMPLETE", 56.),
        GameScreen::TryAgain if flow.failure_message => label(
            &mut commands,
            root,
            if game.campaign.deployable() > 0 {
                "TRY AGAIN"
            } else {
                "GAME OVER"
            },
            56.,
        ),
        GameScreen::TryAgain => results::view(&mut commands, root, &game, &screen),
        GameScreen::Debrief => debrief::view(&mut commands, root, &game, &screen),
        GameScreen::GameOver | GameScreen::CampaignComplete => {
            endings::view(&mut commands, root, &game, &screen)
        }
        _ => {}
    }
}

#[derive(Component)]
pub(crate) struct Music;
fn apply_audio(settings: Res<Settings>, mut sinks: Query<(&mut AudioSink, Has<Music>)>) {
    use bevy::audio::Volume;
    for (mut sink, music) in &mut sinks {
        sink.set_volume(Volume::Linear(
            settings.master
                * if music {
                    settings.music
                } else {
                    settings.effects
                },
        ));
    }
}

fn apply_window(settings: Res<Settings>, mut windows: Query<&mut Window>) {
    for mut w in &mut windows {
        w.resolution
            .set(settings.width as f32, settings.width as f32 * 9. / 16.);
        if settings.fullscreen {
            w.mode = bevy::window::WindowMode::BorderlessFullscreen(
                bevy::window::MonitorSelection::Current,
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn app() -> App {
        let mut a = crate::lifecycle_tests::app();
        a.init_resource::<Actions>()
            .insert_resource(Settings::default())
            .add_systems(Update, actions);
        a
    }
    fn press(a: &mut App, action: Action) {
        a.world_mut().resource_mut::<Actions>().0 = Some(action);
        a.update();
        a.update();
    }
    #[test]
    fn buttons_follow_campaign_and_repeated_deploy_does_not_skip() {
        let mut a = app();
        press(&mut a, Action::New);
        assert_eq!(
            *a.world().resource::<State<GameScreen>>().get(),
            GameScreen::Recruitment
        );
        press(&mut a, Action::Hill);
        assert_eq!(
            *a.world().resource::<State<GameScreen>>().get(),
            GameScreen::Briefing
        );
        a.world_mut().resource_mut::<Game>().reload = false;
        a.world_mut().resource_mut::<Game>().phase_ready = true;
        press(&mut a, Action::Deploy);
        press(&mut a, Action::Deploy);
        assert_eq!(
            *a.world().resource::<State<GameScreen>>().get(),
            GameScreen::Playing
        );
    }
    #[test]
    fn final_debrief_goes_to_campaign_complete() {
        let mut a = app();
        a.world_mut().resource_mut::<Game>().campaign.progress = CampaignProgress {
            mission_index: 1,
            phase_index: 0,
        };
        a.world_mut()
            .resource_mut::<NextState<GameScreen>>()
            .set(GameScreen::Debrief);
        a.update();
        press(&mut a, Action::Debrief);
        press(&mut a, Action::Debrief);
        assert_eq!(
            *a.world().resource::<State<GameScreen>>().get(),
            GameScreen::CampaignComplete
        );
        assert_eq!(
            a.world().resource::<Game>().campaign.progress.mission_index,
            2
        );
    }
    #[test]
    fn debrief_returns_to_recruitment_for_next_mission() {
        let mut a = app();
        a.world_mut()
            .resource_mut::<NextState<GameScreen>>()
            .set(GameScreen::Debrief);
        a.update();
        press(&mut a, Action::Debrief);
        assert_eq!(
            *a.world().resource::<State<GameScreen>>().get(),
            GameScreen::Recruitment
        );
        assert_eq!(
            a.world().resource::<Game>().campaign.progress.mission_index,
            1
        );
    }
}
