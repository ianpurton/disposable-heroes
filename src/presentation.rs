use crate::*;
use bevy::{
    asset::RenderAssetUsages,
    camera::ScalingMode,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    input::mouse::MouseWheel,
    mesh::{Indices, PrimitiveTopology},
    render::view::screenshot::{Screenshot, save_to_disk},
};
use bevy::{picking::hover::Hovered, ui_widgets::Activate};
use std::collections::HashMap;
#[derive(Asset, TypePath, bevy::render::render_resource::AsBindGroup, Debug, Clone)]
struct WaterMaterial {
    #[uniform(0)]
    settings: Vec4,
}
impl Material for WaterMaterial {
    fn fragment_shader() -> bevy::shader::ShaderRef {
        "shaders/water.wesl".into()
    }
}
#[derive(Component)]
struct TerrainView(u8);
const SCALE: f32 = 0.125;
fn world(p: Point) -> Vec3 {
    Vec3::new(p.x * SCALE, 0., p.y * SCALE)
}
#[derive(Component)]
struct LevelEntity;
#[derive(Component)]
struct UnitView(u32);
#[derive(Component)]
struct BuildingView(usize, bool);
#[derive(Component)]
struct PickupView(usize);
#[derive(Component)]
struct HudText;
#[derive(Component)]
struct RosterLabel(usize);
#[derive(Component)]
struct StatusText;
#[derive(Component)]
struct HudPanel;
#[derive(Component)]
struct Button;
#[derive(Component)]
struct MainCamera;
#[derive(Resource)]
struct Offscreen(Handle<Image>);
#[derive(Component)]
struct Particle {
    velocity: Vec3,
    remaining: f32,
    total: f32,
}
#[derive(Resource)]
struct Library {
    water: Handle<WaterMaterial>,
    ground: Handle<Image>,
    animations: HashMap<
        (bool, String),
        (
            Handle<AnimationGraph>,
            bevy::animation::graph::AnimationNodeIndex,
        ),
    >,
    models: HashMap<String, Handle<WorldAsset>>,
    soldier: Handle<Gltf>,
    enemy: Handle<Gltf>,
    sphere: Handle<Mesh>,
    fire: Handle<StandardMaterial>,
    smoke: Handle<StandardMaterial>,
    shot: Handle<AudioSource>,
    boom: Handle<AudioSource>,
}
#[derive(Resource)]
struct CameraRig {
    focus: Vec3,
    zoom: f32,
    manual_scroll: f32,
}
#[derive(Component)]
struct AnimationState(usize);
#[derive(Resource, Default)]
struct OrderFeedback(Option<(Point, bool, f32)>);
#[derive(Component)]
struct Foliage {
    height: f32,
}
#[derive(Resource)]
struct Benchmark {
    start: std::time::Instant,
    samples: Vec<f64>,
    max_entities: usize,
    max_meshes: usize,
    saved: bool,
    source: Option<Battle>,
    repetitions: u32,
}
impl Default for Benchmark {
    fn default() -> Self {
        Self {
            start: std::time::Instant::now(),
            samples: vec![],
            max_entities: 0,
            max_meshes: 0,
            saved: false,
            source: None,
            repetitions: 0,
        }
    }
}
pub struct BattlefieldPlugin;
impl Plugin for BattlefieldPlugin {
    fn build(&self, app: &mut App) {
        if app.world().resource::<Game>().benchmark {
            app.add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin);
        }
        app.add_plugins((
            FrameTimeDiagnosticsPlugin::default(),
            MaterialPlugin::<WaterMaterial>::default(),
        ))
        .init_resource::<OrderFeedback>()
        .init_resource::<Benchmark>()
        .add_systems(Startup, setup)
        .add_systems(PreUpdate, input.after(bevy::input::InputSystems))
        .add_systems(FixedUpdate, simulate)
        .add_systems(
            Update,
            (
                reload,
                unit_views,
                animate,
                particles,
                foliage_visibility,
                hud,
                capture,
                benchmark,
            )
                .chain(),
        );
    }
}
fn setup(
    mut commands: Commands,
    assets: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    game: Res<Game>,
    mut water: ResMut<Assets<WaterMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let models = [
        "thicket",
        "thicket_edge",
        "soldier",
        "enemy",
        "palm",
        "tree",
        "bush",
        "grass",
        "rock",
        "hut",
        "bunker",
        "jeep",
        "wreck",
        "bridge",
        "crate",
    ]
    .into_iter()
    .map(|name| {
        (
            name.into(),
            assets.load(format!("models/{name}.glb#Scene0")),
        )
    })
    .collect();
    let library = Library {
        ground: assets
            .load_builder()
            .with_settings(|s: &mut bevy::image::ImageLoaderSettings| {
                s.sampler =
                    bevy::image::ImageSampler::Descriptor(bevy::image::ImageSamplerDescriptor {
                        address_mode_u: bevy::image::ImageAddressMode::Repeat,
                        address_mode_v: bevy::image::ImageAddressMode::Repeat,
                        ..default()
                    });
            })
            .load("textures/jungle_ground.png"),
        water: water.add(WaterMaterial {
            settings: Vec4::new(0.035, 0.18, 0.16, 0.),
        }),
        animations: HashMap::new(),
        models,
        soldier: assets.load("models/soldier.glb"),
        enemy: assets.load("models/enemy.glb"),
        sphere: meshes.add(
            Sphere::new(1.)
                .mesh()
                .ico(1)
                .expect("valid sphere subdivisions"),
        ),
        fire: materials.add(StandardMaterial {
            base_color: Color::srgb(1., 0.38, 0.035),
            emissive: LinearRgba::rgb(5., 1.1, 0.04),
            ..default()
        }),
        smoke: materials.add(Color::srgb(0.16, 0.19, 0.15)),
        shot: assets.load("audio/rifle.wav"),
        boom: assets.load("audio/explosion.wav"),
    };
    let mut focus = game
        .battle
        .units
        .iter()
        .find(|u| !u.enemy)
        .map(|u| world(u.pos))
        .unwrap_or(Vec3::ZERO);
    if game.battle.map.width <= 25 {
        focus = Vec3::new(
            game.battle.map.width as f32,
            0.,
            game.battle.map.height as f32,
        );
    }
    commands.insert_resource(CameraRig {
        focus,
        zoom: 32.,
        manual_scroll: 0.,
    });
    let camera = commands
        .spawn((
            MainCamera,
            Camera3d::default(),
            Projection::Orthographic(OrthographicProjection {
                scaling_mode: ScalingMode::FixedVertical {
                    viewport_height: 32.,
                },
                ..OrthographicProjection::default_3d()
            }),
            Transform::from_translation(focus + Vec3::new(0., 40., 28.)).looking_at(focus, Vec3::Y),
            DistanceFog {
                color: Color::srgb(0.26, 0.36, 0.29),
                falloff: FogFalloff::Linear {
                    start: 65.,
                    end: 150.,
                },
                ..default()
            },
        ))
        .id();
    if game.smoke {
        let mut image = Image::new_target_texture(
            1920,
            1080,
            bevy::render::render_resource::TextureFormat::Bgra8UnormSrgb,
            None,
        );
        image.texture_descriptor.usage |= bevy::render::render_resource::TextureUsages::COPY_SRC;
        let handle = images.add(image);
        commands
            .entity(camera)
            .insert(bevy::camera::RenderTarget::Image(handle.clone().into()));
        commands.insert_resource(Offscreen(handle));
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
    spawn_level(
        &mut commands,
        &game.battle,
        &library,
        &mut meshes,
        &mut materials,
    );
    let panel = commands
        .spawn((
            HudPanel,
            UiTargetCamera(camera),
            Node {
                position_type: PositionType::Absolute,
                left: px(18),
                top: px(18),
                bottom: px(18),
                width: px(260),
                padding: UiRect::all(px(18)),
                flex_direction: FlexDirection::Column,
                row_gap: px(7),
                ..default()
            },
            BackgroundColor(Color::srgba(0.025, 0.045, 0.035, 0.95)),
        ))
        .id();
    let title = commands
        .spawn((
            HudText,
            bevy::picking::Pickable::IGNORE,
            Text::new("DISPOSABLE\nHEROES"),
            TextFont {
                font_size: FontSize::Px(20.),
                ..default()
            },
            TextColor(Color::srgb(0.93, 0.88, 0.65)),
        ))
        .id();
    commands.entity(panel).add_child(title);
    for slot in 0..5 {
        let button = button(&mut commands, "", panel);
        commands
            .entity(button)
            .observe(move |_: On<Activate>, mut game: ResMut<Game>| {
                let id = game
                    .battle
                    .units
                    .iter()
                    .filter(|u| !u.enemy)
                    .nth(slot)
                    .filter(|u| u.alive)
                    .map(|u| u.id);
                if let Some(id) = id {
                    game.queue.push(Command::Select(id));
                }
            });
        let child = commands
            .spawn((
                bevy::picking::Pickable::IGNORE,
                RosterLabel(slot),
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(15.),
                    ..default()
                },
                TextColor(Color::srgb(0.85, 0.88, 0.72)),
            ))
            .id();
        commands.entity(button).add_child(child);
    }
    for (label, action) in [
        ("SQUAD 1", Command::Group(0)),
        ("SQUAD 2", Command::Group(1)),
        ("SQUAD 3", Command::Group(2)),
        ("SPLIT SELECTED", Command::Split),
        ("REGROUP", Command::Join),
        ("GRENADES", Command::Weapon(Weapon::Grenade)),
        ("ROCKETS", Command::Weapon(Weapon::Rocket)),
    ] {
        let e = button(&mut commands, label, panel);
        commands
            .entity(e)
            .observe(move |_: On<Activate>, mut game: ResMut<Game>| {
                game.queue.push(action.clone());
            });
    }
    let e = button(&mut commands, "DEPLOY / CONTINUE", panel);
    commands
        .entity(e)
        .observe(|_: On<Activate>, mut game: ResMut<Game>| advance(&mut game));
    let e = button(&mut commands, "RETRY MISSION", panel);
    commands
        .entity(e)
        .observe(|_: On<Activate>, mut game: ResMut<Game>| {
            game.campaign = game.snapshot.clone();
            game.reload = true;
        });
    let status = commands
        .spawn((
            StatusText,
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(14.),
                ..default()
            },
            TextColor(Color::srgb(0.75, 0.79, 0.67)),
        ))
        .id();
    commands.entity(panel).add_child(status);
    commands.spawn((
        AudioPlayer::new(assets.load("audio/jungle.wav")),
        PlaybackSettings::LOOP,
    ));
    commands.insert_resource(library);
}
fn model(commands: &mut Commands, lib: &Library, name: &str, p: Vec3, scale: f32) -> Entity {
    commands
        .spawn((
            LevelEntity,
            WorldAssetRoot(lib.models[name].clone()),
            Transform::from_translation(p).with_scale(Vec3::splat(scale)),
        ))
        .id()
}
fn terrain_mesh(b: &Battle, kind: u8) -> Mesh {
    let mut pos = vec![];
    let mut norms = vec![];
    let mut uv = vec![];
    let mut colors = vec![];
    let mut indices = vec![];
    let terrain_kind = |x: usize, y: usize| match b.map.terrain_at(x as f32, y as f32) {
        5 | 6 => 1,
        3 => 2,
        _ => 0,
    };
    for y in (0..b.map.height * 16).step_by(2) {
        let mut x = 0;
        while x < b.map.width * 16 {
            if terrain_kind(x, y) != kind {
                x += 2;
                continue;
            }
            let begin = x;
            x += 2;
            while x < b.map.width * 16 && x - begin < 32 && terrain_kind(x, y) == kind {
                x += 2;
            }
            let base = pos.len() as u32;
            let h = if kind == 1 { -0.10 } else { 0. };
            let variation = (begin as f32 * 0.045).sin() * 0.008 + (y as f32 * 0.037).cos() * 0.008;
            let c = match kind {
                1 => [0.035, 0.17, 0.14, 1.],
                2 => [0.035, 0.085, 0.018, 1.],
                _ => [0.11 + variation, 0.20 + variation, 0.035 + variation, 1.],
            };
            for (xx, yy) in [(begin, y), (begin, y + 2), (x, y + 2), (x, y)] {
                pos.push([xx as f32 * SCALE, h, yy as f32 * SCALE]);
                norms.push([0., 1., 0.]);
                uv.push([xx as f32 / 16., yy as f32 / 16.]);
                colors.push(c);
            }
            indices.extend([base, base + 1, base + 2, base, base + 2, base + 3]);
        }
    }
    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, pos)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, norms)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uv)
    .with_inserted_attribute(Mesh::ATTRIBUTE_COLOR, colors)
    .with_inserted_indices(Indices::U32(indices))
}
fn spawn_level(
    commands: &mut Commands,
    b: &Battle,
    lib: &Library,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    for kind in 0..3 {
        let material = materials.add(StandardMaterial {
            base_color: Color::WHITE,
            base_color_texture: if kind != 1 {
                Some(lib.ground.clone())
            } else {
                None
            },
            perceptual_roughness: if kind == 1 { 0.19 } else { 0.95 },
            metallic: if kind == 1 { 0.18 } else { 0. },
            ..default()
        });
        let mesh = meshes.add(terrain_mesh(b, kind));
        if kind == 1 {
            commands.spawn((
                LevelEntity,
                TerrainView(kind),
                Mesh3d(mesh),
                MeshMaterial3d(lib.water.clone()),
                Transform::default(),
            ));
        } else {
            commands.spawn((
                LevelEntity,
                TerrainView(kind),
                Mesh3d(mesh),
                MeshMaterial3d(material),
                Transform::default(),
            ));
        }
    }
    let catalog = disposable_heroes::environment::JungleCatalog::bundled();
    for patch in disposable_heroes::environment::thickets(&b.map, &catalog) {
        let center = world(Point::new(
            patch.x + patch.width * 0.5,
            patch.y + patch.depth * 0.5,
        ));
        let height = (patch.width.min(patch.depth) * SCALE * 0.7).clamp(0.12, 0.85);
        let name = if (patch.x as u32 + patch.y as u32) % 3 == 0 {
            "thicket_edge"
        } else {
            "thicket"
        };
        let id = model(commands, lib, name, center, 1.);
        commands.entity(id).insert((
            Foliage { height },
            Transform::from_translation(center).with_scale(Vec3::new(
                patch.width * SCALE,
                height,
                patch.depth * SCALE,
            )),
        ));
    }
    // Deterministic ground dressing: visual only, no collision or changes to imported objects.
    for y in (0..b.map.height * 16).step_by(9) {
        for x in (0..b.map.width * 16).step_by(9) {
            let hash = (x as u64 * 73856093) ^ (y as u64 * 19349663) ^ 1993;
            let p = Point::new(x as f32, y as f32);
            if b.map.terrain_at(p.x, p.y) != 0
                || hash % 5 != 0
                || b.buildings.iter().any(|s| s.pos.distance(p) < 35.)
            {
                continue;
            }
            let name = if hash % 17 == 0 { "bush" } else { "grass" };
            model(
                commands,
                lib,
                name,
                world(p),
                0.6 + (hash % 7) as f32 * 0.08,
            );
        }
    }
    for s in &b.map.sprites {
        let name = match s.kind {
            13 | 17 => "bush",
            14 => "palm",
            24 => "rock",
            _ => continue,
        };
        model(
            commands,
            lib,
            name,
            world(Point::new(s.x() as f32, s.y as f32)),
            1.,
        );
    }
    // Tall peripheral planting frames the miniature without changing the map.
    for i in 0..b.map.width {
        if i % 3 == 0 {
            for z in [-1.5, b.map.height as f32 * 2. + 1.5] {
                model(
                    commands,
                    lib,
                    if i % 3 == 0 { "tree" } else { "palm" },
                    Vec3::new(i as f32 * 2. + (i as f32 * 1.7).sin() * 0.7, 0., z),
                    1.1,
                );
            }
        }
    }
    for (i, b) in b.buildings.iter().enumerate() {
        let e = model(
            commands,
            lib,
            if b.vehicle { "jeep" } else { "hut" },
            world(b.pos),
            1.4,
        );
        commands.entity(e).insert(BuildingView(i, false));
    }
    for (i, p) in b.pickups.iter().enumerate() {
        let e = model(commands, lib, "crate", world(p.pos), 1.);
        commands.entity(e).insert(PickupView(i));
    }
    for u in &b.units {
        let e = model(
            commands,
            lib,
            if u.enemy { "enemy" } else { "soldier" },
            world(u.pos),
            0.8,
        );
        commands.entity(e).insert(UnitView(u.id));
    }
}
fn input(
    mut commands: Commands,
    buttons: Query<(Entity, &ComputedNode, &UiGlobalTransform), With<Button>>,
    keys: Res<ButtonInput<KeyCode>>,
    mouse: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut camera: Query<
        (&Camera, &GlobalTransform, &mut Transform, &mut Projection),
        With<MainCamera>,
    >,
    mut wheel: MessageReader<MouseWheel>,
    time: Res<Time>,
    mut rig: ResMut<CameraRig>,
    mut game: ResMut<Game>,
) {
    if keys.just_pressed(KeyCode::F1) {
        game.hud = !game.hud;
    }
    if keys.just_pressed(KeyCode::F12) {
        game.screenshot = true;
    }
    if keys.just_pressed(KeyCode::KeyR) {
        game.campaign = game.snapshot.clone();
        game.reload = true;
    }
    if keys.just_pressed(KeyCode::Enter) {
        advance(&mut game);
    }
    for (key, c) in [
        (KeyCode::Digit1, Command::Group(0)),
        (KeyCode::Digit2, Command::Group(1)),
        (KeyCode::Digit3, Command::Group(2)),
        (KeyCode::KeyS, Command::Split),
        (KeyCode::KeyJ, Command::Join),
        (KeyCode::KeyG, Command::Weapon(Weapon::Grenade)),
        (KeyCode::KeyX, Command::Weapon(Weapon::Rocket)),
    ] {
        if keys.just_pressed(key) {
            game.queue.push(c);
        }
    }
    let Ok(window) = windows.single() else {
        return;
    };
    let Ok((cam, gt, mut transform, mut projection)) = camera.single_mut() else {
        return;
    };
    for w in wheel.read() {
        rig.zoom = (rig.zoom - w.y * 1.5).clamp(14., 60.);
    }
    rig.manual_scroll = (rig.manual_scroll - time.delta_secs()).max(0.);
    if let Some(cursor) = window.cursor_position() {
        let blocked = game.hud && cursor.x < 296.;
        if !blocked && game.briefing && mouse.just_pressed(MouseButton::Left) {
            advance(&mut game);
        }
        if !blocked && !game.briefing {
            let mut pan = Vec3::ZERO;
            if cursor.x < 12. {
                pan.x -= 1.;
            }
            if cursor.x > window.width() - 12. {
                pan.x += 1.;
            }
            if cursor.y < 12. {
                pan.z -= 1.;
            }
            if cursor.y > window.height() - 12. {
                pan.z += 1.;
            }
            if pan.length_squared() > 0. {
                rig.manual_scroll = 1.5;
            }
            let zoom = rig.zoom;
            rig.focus += pan * time.delta_secs() * zoom * 0.7;
            rig.focus.x = rig.focus.x.clamp(0., game.battle.map.width as f32 * 2.);
            rig.focus.z = rig.focus.z.clamp(0., game.battle.map.height as f32 * 2.);
            if let Ok(ray) = cam.viewport_to_world(gt, cursor) {
                if let Some(d) = ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y)) {
                    let p = ray.get_point(d);
                    let p = Point::new(p.x / SCALE, p.z / SCALE);
                    if mouse.pressed(MouseButton::Right) && mouse.just_pressed(MouseButton::Left)
                        || mouse.pressed(MouseButton::Left)
                            && mouse.just_pressed(MouseButton::Right)
                    {
                        game.queue.push(Command::Explosive(p));
                    } else if mouse.just_pressed(MouseButton::Left) {
                        if let Some(id) = game
                            .battle
                            .units
                            .iter()
                            .find(|u| !u.enemy && u.alive && u.pos.distance(p) < 7.)
                            .map(|u| u.id)
                        {
                            game.queue.push(Command::Select(id));
                        } else {
                            game.queue.push(Command::Move(p));
                        }
                    }
                    game.queue.push(Command::Aim(
                        if mouse.pressed(MouseButton::Right) && !mouse.pressed(MouseButton::Left) {
                            Some(p)
                        } else {
                            None
                        },
                    ));
                }
            }
        } else if blocked {
            if mouse.just_pressed(MouseButton::Left) {
                let point = cursor * window.scale_factor();
                for (entity, node, transform) in &buttons {
                    if let Some(inverse) = transform.try_inverse() {
                        let local = inverse.transform_point2(point);
                        if local.abs().cmple(node.size() * 0.5).all() {
                            commands.trigger(Activate { entity });
                            break;
                        }
                    }
                }
            }
            game.queue.push(Command::Aim(None));
        }
    }
    if mouse.just_released(MouseButton::Right) {
        game.queue.push(Command::Aim(None));
    }
    // Track the active living leader inside the usable viewport, clear of the HUD.
    if !game.briefing && game.battle.outcome == Outcome::Playing && rig.manual_scroll <= 0. {
        let leader = game
            .battle
            .units
            .iter()
            .find(|u| game.battle.selected_unit(u))
            .or_else(|| {
                game.battle
                    .units
                    .iter()
                    .find(|u| u.alive && !u.enemy && u.group == game.battle.active_group)
            });
        if let Some(leader) = leader {
            let position = world(leader.pos);
            if let Ok(screen) = cam.world_to_viewport(gt, position) {
                let left = if game.hud {
                    296_f32.min(window.width() * 0.4)
                } else {
                    0.
                };
                let desired =
                    tracking_anchor(screen, Vec2::new(window.width(), window.height()), left);
                if desired != screen {
                    if let Ok(ray) = cam.viewport_to_world(gt, desired) {
                        if let Some(distance) =
                            ray.intersect_plane(Vec3::ZERO, InfinitePlane3d::new(Vec3::Y))
                        {
                            let correction = position - ray.get_point(distance);
                            rig.focus += correction * (1. - (-8. * time.delta_secs()).exp());
                        }
                    }
                }
            }
        }
    }
    rig.focus.x = rig.focus.x.clamp(0., game.battle.map.width as f32 * 2.);
    rig.focus.z = rig.focus.z.clamp(0., game.battle.map.height as f32 * 2.);
    *transform = Transform::from_translation(rig.focus + Vec3::new(0., 40., 28.))
        .looking_at(rig.focus, Vec3::Y);
    if let Projection::Orthographic(p) = &mut *projection {
        p.scaling_mode = ScalingMode::FixedVertical {
            viewport_height: rig.zoom,
        };
    }
}
fn simulate(mut game: ResMut<Game>, time: Res<Time<Fixed>>) {
    let cmds = std::mem::take(&mut game.queue);
    if (game.benchmark || game.combat_preview) && game.briefing {
        game.briefing = false;
        let goal = Point::new(
            game.battle.map.width as f32 * 8.,
            game.battle.map.height as f32 * 5.,
        );
        game.battle.command(Command::Move(goal));
    }
    if game.briefing {
        return;
    }
    if game.benchmark || game.combat_preview {
        let leader = game
            .battle
            .units
            .iter()
            .find(|u| game.battle.selected_unit(u))
            .map(|u| (u.pos, u.path.is_empty()));
        let target = leader.and_then(|(p, _)| {
            game.battle
                .units
                .iter()
                .filter(|u| u.enemy && u.alive)
                .min_by(|a, b| p.distance(a.pos).total_cmp(&p.distance(b.pos)))
                .map(|u| u.pos)
        });
        if let (Some((p, idle)), Some(target)) = (leader, target) {
            let visible = disposable_heroes::simulation::line_of_sight(
                &game.battle.map,
                &game.battle.buildings,
                p,
                target,
            );
            game.battle
                .command(Command::Aim(if visible { Some(target) } else { None }));
            if !visible && idle {
                game.battle.command(Command::Move(target));
            }
        }
    }
    for c in cmds {
        game.battle.command(c);
    }
    game.battle.tick(time.delta_secs());
}
fn reload(
    mut commands: Commands,
    mut game: ResMut<Game>,
    lib: Res<Library>,
    entities: Query<Entity, With<LevelEntity>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut rig: ResMut<CameraRig>,
) {
    if !game.reload {
        return;
    }
    game.reload = false;
    match ImportedLevel::load(&game.maps[game.index], game.resources.as_deref()).and_then(|m| {
        check_map(&m).map_err(|e| disposable_heroes::importer::ImportError::Invalid {
            file: game.maps[game.index].display().to_string(),
            message: e.to_string(),
        })?;
        Ok(m)
    }) {
        Ok(map) => {
            for e in &entities {
                commands.entity(e).despawn();
            }
            let mut battle = Battle::new(map, 1993);
            battle.tuning.aggression(&game.aggression[game.index]);
            battle.kill_objective = game.rules[game.index].0;
            battle.destroy_objective = game.rules[game.index].1;
            assign_roster(&mut battle, &game.snapshot);
            rig.focus = battle
                .units
                .iter()
                .find(|u| !u.enemy && u.alive)
                .map(|u| world(u.pos))
                .unwrap_or(Vec3::ZERO);
            spawn_level(&mut commands, &battle, &lib, &mut meshes, &mut materials);
            game.battle = battle;
            game.briefing = true;
            game.queue.clear();
        }
        Err(e) => game.status = e.to_string(),
    }
}
fn unit_views(
    mut commands: Commands,
    mut game: ResMut<Game>,
    lib: Res<Library>,
    mut units: Query<(Entity, &UnitView, &mut Transform)>,
    mut buildings: Query<(Entity, &mut BuildingView), Without<UnitView>>,
    pickups: Query<(Entity, &PickupView)>,
    mut gizmos: Gizmos,
    mut feedback: ResMut<OrderFeedback>,
    time: Res<Time<Fixed>>,
    mut ground: Query<(&TerrainView, &mut Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if let Some((point, accepted, _)) = feedback.0 {
        let p = world(point) + Vec3::Y * 0.06;
        let color = if accepted {
            Color::srgb(0.9, 0.78, 0.2)
        } else {
            Color::srgb(1., 0.15, 0.08)
        };
        gizmos.circle(
            Isometry3d::new(p, Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            0.45,
            color,
        );
        if !accepted {
            gizmos.line(
                p + Vec3::new(-0.3, 0., -0.3),
                p + Vec3::new(0.3, 0., 0.3),
                color,
            );
        }
    }
    let mut present = std::collections::HashSet::new();
    for (_, view, mut t) in &mut units {
        present.insert(view.0);
        if let Some(u) = game.battle.units.iter().find(|u| u.id == view.0) {
            let p = Point::new(
                u.previous.x + (u.pos.x - u.previous.x) * time.overstep_fraction(),
                u.previous.y + (u.pos.y - u.previous.y) * time.overstep_fraction(),
            );
            t.translation = world(p);
            if !u.alive {
                t.translation.y = 0.05;

                continue;
            }
            let target = if u.fire { u.aim } else { u.target };
            let delta = world(target) - world(u.pos);
            if delta.length_squared() > 0.01 {
                t.rotation = soldier_rotation(delta);
            }
            if game.battle.selected_unit(u) {
                gizmos.circle(
                    Isometry3d::new(
                        world(u.pos) + Vec3::Y * 0.04,
                        Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2),
                    ),
                    0.65,
                    Color::srgb(0.95, 0.82, 0.28),
                );
            }
        }
    }
    for u in &game.battle.units {
        if !present.contains(&u.id) {
            let e = model(
                &mut commands,
                &lib,
                if u.enemy { "enemy" } else { "soldier" },
                world(u.pos),
                0.8,
            );
            commands.entity(e).insert(UnitView(u.id));
        }
    }
    let mut terrain_changed = false;
    for (e, mut view) in &mut buildings {
        if game.battle.buildings[view.0].health <= 0 && !view.1 {
            view.1 = true;
            terrain_changed = true;
            commands.entity(e).despawn();
            model(
                &mut commands,
                &lib,
                "wreck",
                world(game.battle.buildings[view.0].pos),
                1.5,
            );
        }
    }
    if terrain_changed {
        for (kind, mut mesh) in &mut ground {
            mesh.0 = meshes.add(terrain_mesh(&game.battle, kind.0));
        }
    }
    for (e, p) in &pickups {
        if !game.battle.pickups[p.0].active {
            commands.entity(e).despawn();
        }
    }
    for p in &game.battle.projectiles {
        let mut a = world(p.pos) + Vec3::Y * 0.7;
        match p.weapon {
            Weapon::Grenade => {
                a.y += (p.age / p.duration * std::f32::consts::PI).sin() * 3.;
                gizmos.sphere(
                    Isometry3d::from_translation(a),
                    0.12,
                    Color::srgb(1., 0.8, 0.2),
                );
            }
            _ => {
                let tail = world(p.pos.toward(p.start, 6.)) + Vec3::Y * 0.7;
                gizmos.line(a, tail, Color::srgb(1., 0.8, 0.25));
            }
        }
    }
    let effects = std::mem::take(&mut game.battle.effects);
    let mut shot_sound = false;
    let mut boom_sound = false;
    for effect in effects {
        let (p, count, explosion) = match effect {
            Effect::Shot {
                origin,
                direction,
                weapon,
            } => {
                shot_sound = true;
                let muzzle = world(origin)
                    + Vec3::Y * 0.8
                    + (world(direction) - world(origin)).normalize_or_zero() * 0.65;
                commands.spawn((
                    LevelEntity,
                    Mesh3d(lib.sphere.clone()),
                    MeshMaterial3d(lib.fire.clone()),
                    Transform::from_translation(muzzle).with_scale(Vec3::splat(
                        if weapon == Weapon::Rifle { 0.10 } else { 0.20 },
                    )),
                    Particle {
                        velocity: Vec3::ZERO,
                        remaining: 0.06,
                        total: 0.06,
                    },
                ));
                continue;
            }
            Effect::Impact(p) => (p, 3, false),
            Effect::Order { position, accepted } => {
                feedback.0 = Some((position, accepted, 1.2));
                continue;
            }
            Effect::Explosion(p) => {
                boom_sound = true;
                (p, 28, true)
            }
            Effect::Death(p) => (p, 5, false),
            Effect::Pickup(p) => (p, 8, false),
        };
        for i in 0..count {
            let a = i as f32 * 2.399;
            let speed = if explosion { 2. + (i % 5) as f32 } else { 0.5 };
            commands.spawn((
                LevelEntity,
                Mesh3d(lib.sphere.clone()),
                MeshMaterial3d(if i % 3 == 0 {
                    lib.fire.clone()
                } else {
                    lib.smoke.clone()
                }),
                Transform::from_translation(world(p) + Vec3::Y * 0.5)
                    .with_scale(Vec3::splat(if explosion { 0.16 } else { 0.07 })),
                Particle {
                    velocity: Vec3::new(a.cos() * speed, 1. + (i % 4) as f32, a.sin() * speed),
                    remaining: if explosion { 1.4 } else { 0.2 },
                    total: if explosion { 1.4 } else { 0.2 },
                },
            ));
        }
    }
    if shot_sound {
        commands.spawn((
            AudioPlayer::new(lib.shot.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
    if boom_sound {
        commands.spawn((
            AudioPlayer::new(lib.boom.clone()),
            PlaybackSettings::DESPAWN,
        ));
    }
}
fn animate(
    mut commands: Commands,
    game: Res<Game>,
    mut lib: ResMut<Library>,
    gltfs: Res<Assets<Gltf>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    parents: Query<&ChildOf>,
    views: Query<&UnitView>,
    bones: Query<(&Name, &bevy::animation::AnimationTargetId)>,
    mut players: Query<(
        Entity,
        &mut AnimationPlayer,
        Option<&AnimationState>,
        Option<&mut AnimationTransitions>,
    )>,
) {
    for faction in [false, true] {
        if lib.animations.contains_key(&(faction, "Idle".into())) {
            continue;
        }
        let Some(gltf) = gltfs.get(if faction { &lib.enemy } else { &lib.soldier }) else {
            continue;
        };
        let mut graph = AnimationGraph::new();
        let root = graph.root;
        for (name, id) in &bones {
            if matches!(name.as_str(), "Root" | "LeftLeg" | "RightLeg") {
                graph.add_target_to_mask_group(*id, 0);
            } else if matches!(name.as_str(), "Head" | "LeftArm" | "RightArm") {
                graph.add_target_to_mask_group(*id, 1);
            }
        }
        let mut indices = vec![];
        for name in [
            "Idle", "Run", "Swim", "Death", "Fire", "Throw", "Rocket", "__lower",
        ] {
            let source = if name == "__lower" { "Idle" } else { name };
            if let Some(clip) = gltf.named_animations.get(source) {
                let mask = if name == "__lower" {
                    2
                } else if matches!(name, "Fire" | "Throw" | "Rocket") {
                    1
                } else {
                    0
                };
                indices.push((
                    name.to_string(),
                    graph.add_clip_with_mask(clip.clone(), mask, 1., root),
                ));
            }
        }
        let handle = graphs.add(graph);
        for (name, index) in indices {
            lib.animations
                .insert((faction, name), (handle.clone(), index));
        }
    }
    for (entity, mut player, state, transitions) in &mut players {
        let mut root = entity;
        let mut unit = None;
        for _ in 0..12 {
            if let Ok(view) = views.get(root) {
                unit = game.battle.units.iter().find(|u| u.id == view.0);
                break;
            }
            let Ok(parent) = parents.get(root) else {
                break;
            };
            root = parent.parent();
        }
        let Some(u) = unit else {
            continue;
        };
        let name = if !u.alive {
            "Death"
        } else if let Some((weapon, _)) = u.pose {
            match weapon {
                Weapon::Grenade => "Throw",
                Weapon::Rocket => "Rocket",
                Weapon::Rifle => "Fire",
            }
        } else if u.fire {
            "Fire"
        } else if matches!(game.battle.map.terrain_at(u.pos.x, u.pos.y), 5 | 6)
            && u.pos.distance(u.previous) > 0.01
        {
            "Swim"
        } else if u.pos.distance(u.previous) > 0.01 {
            "Run"
        } else {
            "Idle"
        };
        let key = match name {
            "Death" => 3,
            "Fire" => 2,
            "Run" => 1,
            "Throw" => 4,
            "Rocket" => 5,
            "Swim" => 6,
            _ => 0,
        };
        if state.is_some_and(|s| s.0 == key) {
            continue;
        }
        let Some((handle, index)) = lib.animations.get(&(u.enemy, name.into())).cloned() else {
            continue;
        };
        if let Some(mut transitions) = transitions {
            let active = transitions.play(
                &mut player,
                index,
                std::time::Duration::from_millis(if name == "Death" { 40 } else { 100 }),
            );
            if !matches!(name, "Death" | "Throw" | "Rocket") {
                active.repeat();
            } else {
                active.set_speed(if name == "Throw" { 1.5 } else { 1. });
            }
            if let Some((_, lower)) = lib.animations.get(&(u.enemy, "__lower".into())) {
                if matches!(name, "Fire" | "Throw" | "Rocket") {
                    player.play(*lower).repeat();
                } else {
                    player.stop(*lower);
                }
            }
        } else {
            commands
                .entity(entity)
                .insert((AnimationGraphHandle(handle), AnimationTransitions::new()));
            player.play(index).repeat();
        }
        commands.entity(entity).insert(AnimationState(key));
    }
}
fn particles(
    mut commands: Commands,
    time: Res<Time>,
    lib: Res<Library>,
    mut water: ResMut<Assets<WaterMaterial>>,
    mut feedback: ResMut<OrderFeedback>,
    mut q: Query<(Entity, &mut Particle, &mut Transform)>,
) {
    if let Some(mut material) = water.get_mut(&lib.water) {
        material.settings.w = time.elapsed_secs();
    }
    if let Some((_, _, remaining)) = &mut feedback.0 {
        *remaining -= time.delta_secs();
        if *remaining <= 0. {
            feedback.0 = None;
        }
    }
    for (e, mut p, mut t) in &mut q {
        p.remaining -= time.delta_secs();
        if p.remaining <= 0. {
            commands.entity(e).despawn();
            continue;
        }
        t.translation += p.velocity * time.delta_secs();
        p.velocity *= 1. - time.delta_secs() * 1.5;
        if p.total > 0.1 {
            t.scale = Vec3::splat(
                (0.08 + (1. - p.remaining / p.total) * 0.45) * (p.remaining / 0.25).min(1.),
            );
        }
    }
}
fn button(commands: &mut Commands, label: &str, parent: Entity) -> Entity {
    let e = commands
        .spawn((
            Button,
            Hovered::default(),
            Node {
                min_height: px(28),
                width: percent(100),
                padding: UiRect::axes(px(9), px(4)),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.18, 0.12)),
        ))
        .id();
    if !label.is_empty() {
        let text = commands
            .spawn((
                bevy::picking::Pickable::IGNORE,
                Text::new(label.to_string()),
                TextFont {
                    font_size: FontSize::Px(14.),
                    ..default()
                },
                TextColor(Color::srgb(0.92, 0.87, 0.64)),
            ))
            .id();
        commands.entity(e).add_child(text);
    }
    commands.entity(parent).add_child(e);
    e
}
fn advance(game: &mut Game) {
    if game.briefing {
        info!("Deploying mission {}", game.index + 1);
        game.briefing = false;
        return;
    }
    if game.battle.outcome == Outcome::Defeat {
        game.reload = true;
        return;
    }
    if game.battle.outcome != Outcome::Victory || game.campaign.mission > game.index {
        return;
    }
    let dead: Vec<_> = game
        .battle
        .units
        .iter()
        .filter(|u| !u.enemy && !u.alive)
        .map(|u| u.id - 10000)
        .collect();
    let mut next = game.campaign.clone();
    next.complete(&dead);
    if let Err(e) = next.save(&game.save) {
        game.status = format!("Save failed: {e}");
        return;
    }
    game.campaign = next;
    if game.index + 1 < game.maps.len() {
        game.index += 1;
        game.snapshot = game.campaign.clone();
        game.reload = true;
    } else {
        game.status = "CAMPAIGN COMPLETE. The paperwork survived.".into();
    }
}
fn hud(
    game: Res<Game>,
    mut texts: Query<(
        &mut Text,
        Option<&RosterLabel>,
        Has<HudText>,
        Has<StatusText>,
    )>,
    mut panels: Query<&mut Visibility, With<HudPanel>>,
    mut buttons: Query<(&Hovered, &mut BackgroundColor), With<Button>>,
    diagnostics: Res<DiagnosticsStore>,
) {
    for mut p in &mut panels {
        *p = if game.hud {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    for (hovered, mut color) in &mut buttons {
        *color = BackgroundColor(if hovered.get() {
            Color::srgb(0.24, 0.31, 0.17)
        } else {
            Color::srgb(0.12, 0.18, 0.12)
        });
    }
    let enemies = game
        .battle
        .units
        .iter()
        .filter(|u| u.enemy && u.alive)
        .count();
    let buildings = game
        .battle
        .buildings
        .iter()
        .filter(|b| !b.vehicle && b.health > 0)
        .count();
    let fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(|d| d.smoothed())
        .unwrap_or(0.);
    let memorial = game
        .campaign
        .memorial
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    for (mut text, roster, title, status) in &mut texts {
        if title {
            **text = format!(
                "DISPOSABLE\nHEROES\n\n{}\n\nENEMY {:02}   HUTS {:02}",
                game.titles[game.index], enemies, buildings
            );
        } else if let Some(slot) = roster {
            **text = game
                .battle
                .units
                .iter()
                .filter(|u| !u.enemy)
                .nth(slot.0)
                .map(|u| {
                    format!(
                        "{} {} / {}",
                        if game.battle.selected_unit(u) {
                            ">"
                        } else {
                            " "
                        },
                        u.name,
                        if u.alive {
                            format!("SQUAD {}", u.group + 1)
                        } else {
                            "KIA".into()
                        }
                    )
                })
                .unwrap_or("-".into());
        } else if status {
            **text = format!(
                "GRENADES {}   ROCKETS {}\nSELECTED: {:?}\n\n{}\n\nLEFT MOVE / RIGHT FIRE\nBOTH BUTTONS EXPLOSIVE\nF1 HUD / F12 CAPTURE\n\n{}\nFALLEN: {}\n{:.0} FPS",
                game.battle.grenades,
                game.battle.rockets,
                game.battle.weapon,
                if game.briefing {
                    "ORDERS: Eliminate troops.\nDestroy enemy buildings.\nClick battlefield to deploy."
                } else {
                    match game.battle.outcome {
                        Outcome::Playing => "Keep your heads down.",
                        Outcome::Victory => "MISSION COMPLETE",
                        Outcome::Defeat => "SQUAD LOST. Retry available.",
                    }
                },
                game.status,
                memorial,
                fps
            );
        }
    }
}
fn capture(
    mut commands: Commands,
    mut game: ResMut<Game>,
    mut exit: MessageWriter<AppExit>,
    offscreen: Option<Res<Offscreen>>,
) {
    game.frames += 1;
    if game.screenshot || game.smoke && !game.benchmark && game.frames == 360 {
        let _ = std::fs::create_dir_all("tmp");
        commands
            .spawn(offscreen.map_or_else(Screenshot::primary_window, |target| {
                Screenshot::image(target.0.clone())
            }))
            .observe(save_to_disk(if game.combat_preview {
                "tmp/combat.png"
            } else {
                "tmp/battlefield.png"
            }));
        game.screenshot = false;
    }
    if game.smoke && !game.benchmark && game.frames > 420 {
        exit.write(AppExit::Success);
    }
}

// Leave a generous central area so small movements do not continually move the view.
fn tracking_anchor(screen: Vec2, size: Vec2, hud_width: f32) -> Vec2 {
    let width = (size.x - hud_width).max(1.);
    Vec2::new(
        screen
            .x
            .clamp(hud_width + width * 0.22, size.x - width * 0.22),
        screen.y.clamp(size.y * 0.25, size.y * 0.75),
    )
}
#[cfg(test)]
mod camera_tests {
    use super::*;
    #[test]
    fn tracks_all_edges_and_keeps_soldiers_clear_of_hud() {
        let size = Vec2::new(1440., 900.);
        assert_eq!(
            tracking_anchor(Vec2::new(800., 450.), size, 296.),
            Vec2::new(800., 450.)
        );
        let top = tracking_anchor(Vec2::new(800., 20.), size, 296.);
        assert_eq!(top.y, 225.);
        let bottom = tracking_anchor(Vec2::new(800., 890.), size, 296.);
        assert_eq!(bottom.y, 675.);
        assert!(tracking_anchor(Vec2::new(100., 450.), size, 296.).x > 296.);
        assert!(tracking_anchor(Vec2::new(1430., 450.), size, 296.).x < 1430.);
        assert!(
            tracking_anchor(Vec2::new(100., 450.), size, 0.).x
                < tracking_anchor(Vec2::new(100., 450.), size, 296.).x
        );
    }
}

// Blender soldiers face -Y; glTF converts that direction to Bevy +Z.
fn soldier_rotation(direction: Vec3) -> Quat {
    Quat::from_rotation_y(direction.x.atan2(direction.z))
}
#[cfg(test)]
mod facing_tests {
    use super::*;
    #[test]
    fn exported_soldier_faces_its_target_in_every_direction() {
        for direction in [
            Vec3::X,
            -Vec3::X,
            Vec3::Z,
            -Vec3::Z,
            Vec3::new(2., 0., 3.).normalize(),
        ] {
            assert!((soldier_rotation(direction) * Vec3::Z - direction).length() < 0.0001);
        }
    }
}

fn benchmark(
    mut state: ResMut<Benchmark>,
    mut game: ResMut<Game>,
    time: Res<Time<Real>>,
    entities: Query<Entity>,
    meshes: Query<&Mesh3d>,
    diagnostics: Res<DiagnosticsStore>,
    mut exit: MessageWriter<AppExit>,
) {
    if !game.benchmark || state.saved {
        return;
    }
    if state.source.is_none() {
        state.source = Some(game.battle.clone());
    }
    if game.battle.clock >= 8. || game.battle.outcome != Outcome::Playing {
        game.battle = state
            .source
            .as_ref()
            .expect("benchmark snapshot initialized")
            .clone();
        state.repetitions += 1;
    }
    let elapsed = state.start.elapsed().as_secs_f32();
    if elapsed > 3. {
        state.samples.push(time.delta_secs_f64() * 1000.);
        state.max_entities = state.max_entities.max(entities.iter().count());
        state.max_meshes = state.max_meshes.max(meshes.iter().count());
    }
    if elapsed >= game.benchmark_seconds + 3. {
        state.saved = true;
        state.samples.sort_by(f64::total_cmp);
        let percentile = |p: f64| {
            state
                .samples
                .get(((state.samples.len().saturating_sub(1)) as f64 * p).round() as usize)
                .copied()
                .unwrap_or(0.)
        };
        let render = diagnostics
            .iter()
            .filter(|d| d.path().as_str().starts_with("render/"))
            .filter_map(|d| d.smoothed().map(|v| (d.path().as_str().to_string(), v)))
            .collect::<std::collections::BTreeMap<_, _>>();
        let report = serde_json::json!({"profile":if cfg!(debug_assertions){"dev"}else{"release"},"resolution":[1920,1080],"scenario":"first mission; seed 1993; scripted movement and fire; 3s warmup", "render_path":"offscreen Vulkan; no vsync; excludes desktop compositor", "seconds":game.benchmark_seconds,"scenario_repetitions":state.repetitions,"samples":state.samples.len(),"frame_ms":{"p50":percentile(0.5),"p95":percentile(0.95),"p99":percentile(0.99)},"peak_entities":state.max_entities,"peak_mesh_instances":state.max_meshes,"render_diagnostics":render,"draw_calls":"not available from stock diagnostics; mesh instances are not draw calls"});
        let _ = std::fs::create_dir_all("tmp");
        match std::fs::write(
            "tmp/benchmark.json",
            serde_json::to_vec_pretty(&report).expect("serializable report"),
        ) {
            Ok(()) => info!("Benchmark saved to tmp/benchmark.json: {}", report),
            Err(e) => error!("Benchmark report failed: {e}"),
        };
        exit.write(AppExit::Success);
    }
}
fn foliage_visibility(
    game: Res<Game>,
    time: Res<Time>,
    mut foliage: Query<(&Foliage, &mut Transform)>,
) {
    let leaders = game
        .battle
        .units
        .iter()
        .filter(|u| game.battle.selected_unit(u))
        .map(|u| world(u.pos))
        .collect::<Vec<_>>();
    for (foliage, mut transform) in &mut foliage {
        let p = transform.translation;
        let nearby = leaders.iter().any(|u| {
            let d = p - *u;
            d.x.abs() < transform.scale.x * 0.6 + 0.7 && d.z > -2. && d.z < 3.
        });
        let target = if nearby {
            foliage.height.min(0.16)
        } else {
            foliage.height
        };
        transform.scale.y += (target - transform.scale.y) * (1. - (-10. * time.delta_secs()).exp());
    }
}
