//! Windowed and offscreen inspection using the exact game model library.
#[cfg(not(target_arch = "wasm32"))]
fn main() {
    native::main();
}
#[cfg(target_arch = "wasm32")]
fn main() {
    panic!("asset gallery tooling is native-only");
}
#[cfg(not(target_arch = "wasm32"))]
mod native {
    use bevy::{
        app::ScheduleRunnerPlugin,
        camera::{RenderTarget, ScalingMode},
        mesh::skinning::SkinnedMeshInverseBindposes,
        prelude::*,
        render::{
            render_resource::TextureUsages,
            view::screenshot::{Screenshot, ScreenshotCaptured},
        },
    };
    use models::{GenerationOptions, PreparedModel};
    #[derive(Resource)]
    struct Options {
        seed: u64,
        names: Vec<String>,
        clip: Option<String>,
        time: Option<f32>,
        output: Option<String>,
    }
    #[derive(Resource)]
    struct Capture {
        image: Handle<Image>,
        frames: u32,
        requested: bool,
    }
    #[derive(Component)]
    struct LabelAt(Vec3);
    pub fn main() {
        if let Err(e) = run() {
            eprintln!("asset-gallery: {e}");
            std::process::exit(1);
        }
    }
    fn run() -> Result<(), Box<dyn std::error::Error>> {
        let args: Vec<_> = std::env::args().collect();
        let value = |key: &str| {
            args.iter()
                .position(|a| a == key)
                .and_then(|i| args.get(i + 1))
                .cloned()
        };
        if args.iter().any(|a| a == "--help") {
            println!(
                "asset-gallery [--seed N] [--model NAME] [--clip NAME] [--time SECONDS] [--output tmp/gallery.png]\nOmit --output for interactive preview. --clip enables playback; --time freezes a pose."
            );
            return Ok(());
        }
        let names = if let Some(n) = value("--model") {
            if !models::NAMES.contains(&n.as_str()) {
                return Err(format!("unknown model {n}").into());
            }
            vec![n]
        } else {
            models::NAMES.iter().map(|s| s.to_string()).collect()
        };
        let clip = value("--clip");
        if clip
            .as_ref()
            .is_some_and(|s| !models::CLIPS.contains(&s.as_str()))
        {
            return Err("unknown animation clip".into());
        }
        let time = value("--time").map(|s| s.parse::<f32>()).transpose()?;
        if time.is_some_and(|t| !t.is_finite() || t < 0.) {
            return Err("time must be finite and nonnegative".into());
        }
        if time.is_some() && clip.is_none() {
            return Err("--time requires --clip".into());
        }
        let output = value("--output");
        let headless = output.is_some();
        let mut app = App::new();
        let plugins = DefaultPlugins
            .set(WindowPlugin {
                primary_window: if headless {
                    None
                } else {
                    Some(Window {
                        title: "Procedural asset gallery".into(),
                        resolution: (1600, 1000).into(),
                        ..default()
                    })
                },
                exit_condition: bevy::window::ExitCondition::DontExit,
                ..default()
            })
            .set(AssetPlugin {
                file_path: std::env::current_dir()?
                    .join("assets")
                    .to_string_lossy()
                    .into(),
                ..default()
            });
        if headless {
            app.add_plugins(
                plugins
                    .build()
                    .disable::<bevy::winit::WinitPlugin>()
                    .disable::<bevy::audio::AudioPlugin>(),
            )
            .add_plugins(ScheduleRunnerPlugin::run_loop(
                std::time::Duration::from_secs_f64(1. / 60.),
            ));
        } else {
            app.add_plugins(plugins.build().disable::<bevy::audio::AudioPlugin>());
        }
        let result = app
            .insert_resource(Options {
                seed: value("--seed")
                    .map(|s| s.parse())
                    .transpose()?
                    .unwrap_or(models::DEFAULT_SEED),
                names,
                clip,
                time,
                output,
            })
            .insert_resource(ClearColor(Color::srgb(0.12, 0.15, 0.13)))
            .add_systems(Startup, setup)
            .add_systems(Update, (labels, capture, close))
            .run();
        if result.is_error() {
            return Err("gallery rendering or capture failed".into());
        }
        Ok(())
    }
    fn setup(
        mut commands: Commands,
        options: Res<Options>,
        mut meshes: ResMut<Assets<Mesh>>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        mut binds: ResMut<Assets<SkinnedMeshInverseBindposes>>,
        mut clips: ResMut<Assets<AnimationClip>>,
        mut graphs: ResMut<Assets<AnimationGraph>>,
        mut images: ResMut<Assets<Image>>,
    ) {
        let columns = options.names.len().min(5);
        let rows = options.names.len().div_ceil(columns);
        let spacing = 4.5;
        let row_spacing = 7.;
        let mut keep = vec![];
        let mut label_entities = vec![];
        for (i, name) in options.names.iter().enumerate() {
            let model = models::generate(name, GenerationOptions { seed: options.seed })
                .expect("catalog model");
            let (lo, hi) = model.bounds();
            let scale = 3.2 / (hi - lo).max_element().max(0.001);
            let center = Vec3::new(
                (i % columns) as f32 * spacing,
                0.,
                (i / columns) as f32 * row_spacing,
            );
            let transform = Transform::from_translation(
                center - Vec3::new((lo.x + hi.x) * 0.5, lo.y, (lo.z + hi.z) * 0.5) * scale,
            )
            .with_scale(Vec3::splat(scale));
            let prepared =
                PreparedModel::new(model, &mut meshes, &mut materials, &mut binds, &mut clips);

            let spawned = prepared.spawn(&mut commands, transform);
            if let (Some(player), Some(clip)) = (spawned.player, options.clip.as_ref()) {
                let mut graph = AnimationGraph::new();
                let node = graph.add_clip(prepared.clips[clip].clone(), 1., graph.root);
                let mut p = AnimationPlayer::default();
                let active = p.play(node);
                if let Some(time) = options.time {
                    active.set_seek_time(time).pause();
                } else {
                    active.repeat();
                }
                commands
                    .entity(player)
                    .insert((AnimationGraphHandle(graphs.add(graph)), p));
            }

            let label = commands
                .spawn((
                    Text::new(name.clone()),
                    TextFont {
                        font_size: FontSize::Px(20.),
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    Node {
                        position_type: PositionType::Absolute,
                        ..default()
                    },
                    LabelAt(center + Vec3::new(-0.6, 0., 2.6)),
                ))
                .id();
            label_entities.push(label);
            keep.push(prepared);
        }
        // Strong handles live as long as the gallery, including animation and skin assets.
        commands.insert_resource(GalleryModels(keep));
        let center = Vec3::new(
            (columns - 1) as f32 * spacing * 0.5,
            1.2,
            (rows - 1) as f32 * row_spacing * 0.5,
        );
        let transform = Transform::from_translation(center + Vec3::new(0., 24., 18.))
            .looking_at(center, Vec3::Y);
        let inverse = transform.to_matrix().inverse();
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = -lo;
        for x in [-2.3, (columns - 1) as f32 * spacing + 2.3] {
            for z in [-2.3, (rows - 1) as f32 * row_spacing + 2.6] {
                for y in [0., 3.5] {
                    let p = inverse.transform_point3(Vec3::new(x, y, z));
                    lo = lo.min(p);
                    hi = hi.max(p);
                }
            }
        }
        let camera = commands
            .spawn((
                Camera3d::default(),
                Projection::Orthographic(OrthographicProjection {
                    scaling_mode: ScalingMode::AutoMin {
                        min_width: (hi.x - lo.x) * 1.12,
                        min_height: (hi.y - lo.y) * 1.12,
                    },
                    ..OrthographicProjection::default_3d()
                }),
                transform,
            ))
            .id();
        for label in label_entities {
            commands
                .entity(label)
                .insert(bevy::ui::UiTargetCamera(camera));
        }
        if options.output.is_some() {
            let mut image = Image::new_target_texture(
                1600,
                1000,
                bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
                None,
            );
            image.texture_descriptor.usage |= TextureUsages::COPY_SRC;
            let handle = images.add(image);
            commands
                .entity(camera)
                .insert(RenderTarget::Image(handle.clone().into()));
            commands.insert_resource(Capture {
                image: handle,
                frames: 0,
                requested: false,
            });
        }
        commands.insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.68, 0.81, 0.72),
            brightness: 120.,
            ..default()
        });
        commands.spawn((
            DirectionalLight {
                illuminance: 9000.,
                shadow_maps_enabled: true,
                ..default()
            },
            Transform::from_xyz(-20., 45., 12.).looking_at(Vec3::ZERO, Vec3::Y),
        ));
        commands.spawn((
            Mesh3d(meshes.add(Plane3d::default().mesh().size(100., 100.))),
            MeshMaterial3d(materials.add(Color::srgb(0.12, 0.15, 0.13))),
            Transform::from_xyz(0., -0.03, 0.),
        ));
    }
    #[derive(Resource)]
    struct GalleryModels(#[allow(dead_code)] Vec<PreparedModel>);
    fn labels(
        camera: Single<(&Camera, &GlobalTransform)>,
        mut labels: Query<(&LabelAt, &mut Node)>,
    ) {
        for (label, mut node) in &mut labels {
            if let Ok(p) = camera.0.world_to_viewport(camera.1, label.0) {
                node.left = Val::Px(p.x);
                node.top = Val::Px(p.y);
            }
        }
    }
    fn capture(
        mut commands: Commands,
        mut state: Option<ResMut<Capture>>,
        options: Res<Options>,
        mut exit: MessageWriter<AppExit>,
    ) {
        let Some(state) = state.as_mut() else {
            return;
        };
        state.frames += 1;
        if state.frames > 600 {
            eprintln!("gallery screenshot timed out");
            exit.write(AppExit::error());
            return;
        }
        if state.frames >= 90 && !state.requested {
            state.requested = true;
            let path = options.output.clone().expect("capture output");
            commands
                .spawn(Screenshot::image(state.image.clone()))
                .observe(
                    move |shot: On<ScreenshotCaptured>, mut exit: MessageWriter<AppExit>| {
                        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                            let path = std::path::Path::new(&path);
                            if let Some(parent) = path.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                            shot.image.clone().try_into_dynamic()?.save(path)?;
                            Ok(())
                        })();
                        match result {
                            Ok(()) => {
                                exit.write(AppExit::Success);
                            }
                            Err(e) => {
                                eprintln!("gallery capture: {e}");
                                exit.write(AppExit::error());
                            }
                        }
                    },
                );
        }
    }
    fn close(
        keys: Res<ButtonInput<KeyCode>>,
        mut exit: MessageWriter<AppExit>,
        mut closed: MessageReader<bevy::window::WindowCloseRequested>,
    ) {
        if keys.just_pressed(KeyCode::Escape) || closed.read().next().is_some() {
            exit.write(AppExit::Success);
        }
    }
}
