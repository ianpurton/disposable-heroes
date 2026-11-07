use crate::*;
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameScreen {
    #[default]
    MainMenu,
    Recruitment,
    Briefing,
    Playing,
    PhaseComplete,
    MissionComplete,
    Debrief,
    TryAgain,
    GameOver,
    CampaignComplete,
}
#[derive(Resource, Default)]
pub struct Flow {
    pub paused: bool,
    pub options: bool,
    pub elapsed: f32,
    pub failure_message: bool,
}
pub struct GameFlowPlugin;
impl Plugin for GameFlowPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameScreen>()
            .init_resource::<Flow>()
            .add_systems(Update, tick)
            .add_systems(OnEnter(GameScreen::Playing), deployed)
            .add_systems(OnEnter(GameScreen::Briefing), prepare);
    }
}
fn deployed(mut game: ResMut<Game>) {
    game.queue.clear();
    let ids: Vec<_> = game
        .battle
        .units
        .iter()
        .filter(|u| !u.enemy && u.alive)
        .filter_map(|u| u.id.checked_sub(10000))
        .collect();
    for id in ids {
        if !game.campaign.veterans.contains(&id) {
            game.campaign.veterans.push(id);
        }
    }
}
fn prepare(mut game: ResMut<Game>) {
    game.index = game
        .phase_counts
        .iter()
        .take(game.campaign.progress.mission_index)
        .sum::<usize>()
        + game.campaign.progress.phase_index;
    game.reload = true;
    game.phase_ready = false;
    game.queue.clear();
}
pub fn casualties(game: &mut Game) {
    let dead: Vec<_> = game
        .battle
        .units
        .iter()
        .filter(|u| !u.enemy && !u.alive)
        .filter_map(|u| u.id.checked_sub(10000))
        .collect();
    game.campaign.commit_casualties(&dead);
}
pub fn save(game: &mut Game) {
    if let Err(e) = game.campaign.save(&game.save) {
        game.status = format!("Save failed: {e}");
    }
}
fn tick(
    mut game: ResMut<Game>,
    screen: Res<State<GameScreen>>,
    mut next: ResMut<NextState<GameScreen>>,
    mut flow: ResMut<Flow>,
    time: Res<Time>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    if game.smoke {
        return;
    }
    if keys.just_pressed(KeyCode::Escape) && *screen.get() == GameScreen::Playing {
        flow.paused = !flow.paused;
        flow.options = false;
        game.queue.clear();
    }
    if flow.paused || flow.options {
        return;
    }
    match screen.get() {
        GameScreen::Playing if game.battle.outcome != Outcome::Playing => {
            casualties(&mut game);
            game.queue.clear();
            flow.elapsed = 0.;
            if game.battle.outcome == Outcome::Defeat {
                flow.failure_message = true;
                game.campaign.checkpoint = "TryAgain".into();
                next.set(GameScreen::TryAgain);
            } else if game.campaign.progress.phase_index + 1
                < game.phase_counts[game.campaign.progress.mission_index]
            {
                game.campaign.checkpoint = "PhaseComplete".into();
                next.set(GameScreen::PhaseComplete);
            } else {
                game.campaign.checkpoint = "MissionComplete".into();
                next.set(GameScreen::MissionComplete);
            }
            save(&mut game);
        }
        GameScreen::PhaseComplete | GameScreen::MissionComplete => {
            flow.elapsed += time.delta_secs();
            if flow.elapsed >= 2. {
                flow.elapsed = 0.;
                if *screen.get() == GameScreen::PhaseComplete {
                    game.campaign.progress.phase_index += 1;
                    game.campaign.checkpoint = "Briefing".into();
                    next.set(GameScreen::Briefing);
                } else {
                    game.campaign.commit_mission();
                    game.campaign.checkpoint = "Debrief".into();
                    next.set(GameScreen::Debrief);
                }
                save(&mut game);
            }
        }
        GameScreen::TryAgain if flow.failure_message => {
            flow.elapsed += time.delta_secs();
            if flow.elapsed >= 2. {
                flow.failure_message = false;
                if game.campaign.deployable() == 0 {
                    next.set(GameScreen::GameOver);
                }
            }
        }
        _ => {}
    }
}
