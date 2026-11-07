use crate::game_flow::*;
use crate::*;
pub(crate) fn app() -> App {
    let map_path = PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../data/Data/AmigaTheOne/mapm1.map"
    ));
    let mut battle = Battle::new(ImportedLevel::load(&map_path, None).unwrap(), 1993);
    let campaign = Campaign::default();
    assign_roster(&mut battle, &campaign);
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, bevy::state::app::StatesPlugin))
        .init_resource::<ButtonInput<KeyCode>>()
        .insert_resource(Game {
            battle,
            asset_seed: 1993,
            campaign,
            phase_counts: vec![2, 1],
            mission_titles: vec!["TEST".into(); 2],
            maps: vec![map_path; 3],
            titles: vec!["TEST".into(); 3],
            index: 0,
            reload: false,
            phase_ready: true,
            hud: false,
            queue: vec![],
            save: std::env::temp_dir().join(format!(
                "heroes-state-{:?}.json",
                std::thread::current().id()
            )),
            status: String::new(),
            smoke: false,
            screenshot: false,
            frames: 0,
            resources: None,
            rules: vec![(true, true); 3],
            aggression: vec![vec![2, 4]; 3],
            benchmark: false,
            combat_preview: false,
            benchmark_seconds: 1.,
        })
        .add_plugins(GameFlowPlugin);
    app.update();
    app
}
fn goto(app: &mut App, state: GameScreen) {
    app.world_mut()
        .resource_mut::<NextState<GameScreen>>()
        .set(state);
    app.update();
}
fn current(app: &App) -> GameScreen {
    *app.world().resource::<State<GameScreen>>().get()
}
fn expire(app: &mut App) {
    app.world_mut().resource_mut::<Flow>().elapsed = 2.;
    app.update();
    app.update();
}
#[test]
fn complete_success_lifecycle() {
    let mut a = app();
    assert_eq!(current(&a), GameScreen::MainMenu);
    goto(&mut a, GameScreen::Recruitment);
    goto(&mut a, GameScreen::Briefing);
    goto(&mut a, GameScreen::Playing);
    a.world_mut().resource_mut::<Game>().battle.outcome = Outcome::Victory;
    a.update();
    a.update();
    assert_eq!(current(&a), GameScreen::PhaseComplete);
    expire(&mut a);
    assert_eq!(current(&a), GameScreen::Briefing);
    assert_eq!(
        a.world().resource::<Game>().campaign.progress,
        CampaignProgress {
            mission_index: 0,
            phase_index: 1
        }
    );
    a.world_mut().resource_mut::<Game>().battle.outcome = Outcome::Playing;
    goto(&mut a, GameScreen::Playing);
    a.world_mut().resource_mut::<Game>().battle.outcome = Outcome::Victory;
    a.update();
    a.update();
    assert_eq!(current(&a), GameScreen::MissionComplete);
    expire(&mut a);
    assert_eq!(current(&a), GameScreen::Debrief);
    assert_eq!(a.world().resource::<Game>().campaign.mission, 1);
    a.update();
    assert_eq!(a.world().resource::<Game>().campaign.mission, 1);
}
#[test]
fn failure_is_permanent_and_depletion_is_game_over() {
    let mut a = app();
    goto(&mut a, GameScreen::Playing);
    {
        let mut g = a.world_mut().resource_mut::<Game>();
        for u in g.battle.units.iter_mut().filter(|u| !u.enemy) {
            u.alive = false;
        }
        g.battle.outcome = Outcome::Defeat;
    }
    a.update();
    a.update();
    assert_eq!(current(&a), GameScreen::TryAgain);
    expire(&mut a);
    assert!(!a.world().resource::<Game>().campaign.memorial.is_empty());
    goto(&mut a, GameScreen::Briefing);
    {
        let mut g = a.world_mut().resource_mut::<Game>();
        let ids = g.campaign.roster.iter().map(|s| s.id).collect::<Vec<_>>();
        g.campaign.commit_casualties(&ids);
        g.battle.outcome = Outcome::Playing;
    }
    goto(&mut a, GameScreen::Playing);
    a.world_mut().resource_mut::<Game>().battle.outcome = Outcome::Defeat;
    a.update();
    a.update();
    expire(&mut a);
    assert_eq!(current(&a), GameScreen::GameOver);
}
