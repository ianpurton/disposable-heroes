mod presentation;
use bevy::prelude::*;
use disposable_heroes::simulation::Command;
use disposable_heroes::{campaign::*, importer::ImportedLevel, simulation::*};
use std::path::PathBuf;
#[derive(Resource)]
pub struct Game {
    pub battle: Battle,
    pub campaign: Campaign,
    pub snapshot: Campaign,
    pub maps: Vec<PathBuf>,
    pub titles: Vec<String>,
    pub index: usize,
    pub briefing: bool,
    pub reload: bool,
    pub hud: bool,
    pub queue: Vec<Command>,
    pub save: PathBuf,
    pub status: String,
    pub smoke: bool,
    pub screenshot: bool,
    pub frames: u32,
    pub resources: Option<PathBuf>,
    pub rules: Vec<(bool, bool)>,
    pub aggression: Vec<Vec<u8>>,
    pub benchmark: bool,
    pub combat_preview: bool,
    pub benchmark_seconds: f32,
}
fn main() {
    if let Err(e) = run() {
        eprintln!("Disposable Heroes: {e}");
        std::process::exit(1);
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let value = |key: &str| {
        args.iter()
            .position(|a| a == key)
            .and_then(|i| args.get(i + 1))
            .cloned()
    };
    if args.iter().any(|a| a == "--help") {
        println!(
            "Disposable Heroes\n--data DIR (default data)\n--map FILE.map [--tilesets DIR]\n--campaign FILE.ofc --maps DIR\n--seed N --fresh --smoke-test --benchmark [--benchmark-seconds N] --combat-preview\nMouse: left move/select, right hold fire, both buttons explosive.\n1/2/3 squads, G grenades, X rockets, S split, J join, R retry, Enter deploy/continue, F1 HUD, F12 screenshot."
        );
        return Ok(());
    }
    let data = PathBuf::from(value("--data").unwrap_or("data".into()));
    let mut maps = vec![
        data.join("Data/AmigaTheOne/mapm1.map"),
        data.join("Data/AmigaTheOne/mapm2.map"),
        data.join("Data/AmigaAction/mapm3.map"),
    ];
    let mut titles = vec![
        "WELCOME TO THE JUNGLE".into(),
        "PROPERTY DAMAGE".into(),
        "NO RETURN ADDRESS".into(),
    ];
    let mut rules = vec![(true, true); 3];
    let mut aggression = vec![vec![2, 4], vec![4, 8], vec![4, 8]];
    if let Some(file) = value("--map") {
        rules = vec![(true, true)];
        aggression = vec![vec![2, 4]];
        maps = vec![file.into()];
        titles = vec!["FIELD OPERATIONS".into()];
    }
    if let Some(file) = value("--campaign") {
        let definition: CampaignDefinition = serde_json::from_slice(&std::fs::read(file)?)?;
        let dir = PathBuf::from(value("--maps").ok_or("--campaign requires --maps DIRECTORY")?);
        maps.clear();
        titles.clear();
        rules.clear();
        aggression.clear();
        for mission in definition.missions {
            for phase in mission.phases {
                if phase.objectives.iter().any(|o| {
                    !matches!(
                        o.as_str(),
                        "KILL ALL ENEMY" | "DESTROY ALL BUILDINGS" | "DESTROY ENEMY BUILDINGS"
                    )
                }) {
                    return Err(format!(
                        "Unsupported objectives in {}: {:?}",
                        phase.name, phase.objectives
                    )
                    .into());
                }
                rules.push((
                    phase.objectives.iter().any(|o| o == "KILL ALL ENEMY"),
                    phase.objectives.iter().any(|o| o.contains("BUILDINGS")),
                ));
                aggression.push(phase.aggression);
                maps.push(dir.join(format!("{}.map", phase.map_name)));
                titles.push(phase.name);
            }
        }
    }
    if maps.is_empty() {
        return Err("Campaign has no phases".into());
    }
    let save = PathBuf::from("saves/campaign.json");
    let campaign = if !args.iter().any(|a| a == "--fresh") && save.exists() {
        Campaign::load(&save)?
    } else {
        Campaign::default()
    };
    let index = campaign.mission.min(maps.len() - 1);
    let resources = value("--tilesets").map(PathBuf::from);
    let map = ImportedLevel::load(&maps[index], resources.as_deref())?;
    check_map(&map)?;
    let mut battle = Battle::new(
        map,
        value("--seed").and_then(|s| s.parse().ok()).unwrap_or(1993),
    );
    battle.kill_objective = rules[index].0;
    battle.destroy_objective = rules[index].1;
    battle.tuning.aggression(&aggression[index]);
    assign_roster(&mut battle, &campaign);
    let benchmark = args.iter().any(|a| a == "--benchmark");
    let combat_preview = args.iter().any(|a| a == "--combat-preview");
    let smoke = benchmark || combat_preview || args.iter().any(|a| a == "--smoke-test");
    let plugins = DefaultPlugins
        .set(AssetPlugin {
            file_path: std::fs::canonicalize("assets")?
                .to_string_lossy()
                .into_owned(),
            ..default()
        })
        .set(WindowPlugin {
            primary_window: if smoke {
                None
            } else {
                Some(Window {
                    title: "Disposable Heroes".into(),
                    resolution: (1920, 1080).into(),
                    ..default()
                })
            },
            exit_condition: if smoke {
                bevy::window::ExitCondition::DontExit
            } else {
                bevy::window::ExitCondition::OnAllClosed
            },
            ..default()
        });
    let mut app = App::new();
    if smoke {
        app.add_plugins(plugins.build().disable::<bevy::winit::WinitPlugin>())
            .add_plugins(bevy::app::ScheduleRunnerPlugin::run_loop(if benchmark {
                std::time::Duration::ZERO
            } else {
                std::time::Duration::from_secs_f64(1. / 60.)
            }));
    } else {
        app.add_plugins(plugins);
    }
    app.insert_resource(ClearColor(Color::srgb(0.035, 0.065, 0.06)))
        .insert_resource(Time::<Fixed>::from_hz(50.))
        .insert_resource(Game {
            battle,
            snapshot: campaign.clone(),
            campaign,
            maps,
            titles,
            index,
            briefing: true,
            reload: false,
            hud: true,
            queue: vec![],
            save,
            status: String::new(),
            smoke,
            screenshot: false,
            frames: 0,
            resources,
            rules,
            aggression,
            benchmark,
            combat_preview,
            benchmark_seconds: value("--benchmark-seconds")
                .and_then(|s| s.parse::<f32>().ok())
                .filter(|s| s.is_finite() && *s >= 1.)
                .unwrap_or(20.),
        })
        .add_plugins(presentation::BattlefieldPlugin)
        .run();
    Ok(())
}
fn check_map(map: &ImportedLevel) -> Result<(), Box<dyn std::error::Error>> {
    let unsupported = map.unsupported_sprites();
    if !unsupported.is_empty() {
        return Err(
            format!("Unsupported sprite IDs {unsupported:?}; inspect with map-inspect").into(),
        );
    }
    if !map.base.starts_with("jun") {
        return Err(
            "This slice supports jungle presentation only; use map-inspect for other tilesets"
                .into(),
        );
    }
    Ok(())
}
fn assign_roster(b: &mut Battle, c: &Campaign) {
    let roster: Vec<_> = c.roster.iter().filter(|s| s.alive).collect();
    for (i, u) in b.units.iter_mut().filter(|u| !u.enemy).enumerate() {
        if let Some(s) = roster.get(i) {
            u.id = 10000 + s.id;
            u.name = s.name.clone();
        } else {
            u.alive = false;
        }
    }
}
