use bevy::prelude::*;
use models::*;
#[test]
fn catalog_valid_deterministic_and_order_independent() {
    for name in NAMES {
        let a = generate(name, GenerationOptions::default()).unwrap();
        let b = generate(name, GenerationOptions::default()).unwrap();
        a.validate().unwrap();
        assert_eq!(a.parts.len(), b.parts.len());
        let (lo, hi) = a.bounds();
        assert!(lo.is_finite() && hi.is_finite() && (hi - lo).min_element() > 0.);
        for (p, q) in a.parts.iter().zip(&b.parts) {
            assert_eq!(p.geometry.positions, q.geometry.positions);
            assert!(
                p.geometry
                    .normals
                    .iter()
                    .all(|n| (Vec3::from_array(*n).length() - 1.).abs() < 1e-4)
            );
        }
        if matches!(name, "wreck" | "thicket" | "thicket_edge") {
            let other = generate(name, GenerationOptions { seed: 1 }).unwrap();
            assert_ne!(
                a.parts[0].geometry.positions,
                other.parts[0].geometry.positions
            );
        }
    }
    assert!(generate("missing", GenerationOptions::default()).is_err());
}
#[test]
fn legacy_joint_frames_and_animation_samples() {
    let fixture: serde_json::Value = serde_json::from_str(include_str!("legacy-rig.json")).unwrap();
    let rig = Rig::new();
    for (i, name) in BONES.iter().enumerate() {
        let n = &fixture["joints"][name];
        let expected: Vec<f32> = n["translation"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        assert!(
            rig.rest[i]
                .translation
                .distance(Vec3::from_slice(&expected))
                < 1e-5
        );
    }
    for clip in CLIPS {
        for channel in fixture["animations"][clip].as_array().unwrap() {
            let bone = channel["bone"].as_str().unwrap();
            let i = BONES.iter().position(|b| *b == bone).unwrap();
            let path = channel["path"].as_str().unwrap();
            let times = channel["times"].as_array().unwrap();
            let values = channel["values"].as_array().unwrap();
            for (t, value) in times.iter().zip(values) {
                let frame = (t[0].as_f64().unwrap() * 24.).round() as usize - 1;
                let p = rig.pose(clip, frame)[i];
                let v: Vec<f32> = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|v| v.as_f64().unwrap() as f32)
                    .collect();
                match path {
                    "translation" => assert!(
                        p.translation.distance(Vec3::from_slice(&v)) < 1e-5,
                        "{clip} {bone} frame {frame}: {:?} != {v:?}",
                        p.translation
                    ),
                    "rotation" => {
                        let q = Quat::from_array(v.try_into().unwrap());
                        assert!(
                            p.rotation.dot(q).abs() > 0.99999,
                            "{clip} {bone} frame {frame}: {:?} != {q:?}",
                            p.rotation
                        );
                    }
                    _ => {}
                }
            }
        }
    }
    for (i, bind) in rig.inverse_bind().iter().enumerate() {
        let rest = if i == 0 {
            rig.rest[i].to_matrix()
        } else {
            rig.rest[0].to_matrix() * rig.rest[i].to_matrix()
        };
        assert!((rest * (*bind)).abs_diff_eq(Mat4::IDENTITY, 1e-5));
    }
}
#[test]
fn direct_spawn_has_skin_targets_and_shared_handles() {
    use bevy::{
        animation::{AnimatedBy, AnimationTargetId},
        mesh::skinning::{SkinnedMesh, SkinnedMeshInverseBindposes},
    };
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<StandardMaterial>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut clips = Assets::<AnimationClip>::default();
    let p = PreparedModel::new(
        generate("soldier", GenerationOptions::default()).unwrap(),
        &mut meshes,
        &mut materials,
        &mut binds,
        &mut clips,
    );
    assert_eq!(p.clips.len(), 7);
    assert!(
        p.clips
            .values()
            .all(|h| (clips.get(h).unwrap().duration() - 25. / 24.).abs() < 1e-6)
    );
    let (graph, nodes) = p.animation_graph();
    assert_eq!(nodes.len(), 8);
    for i in 0..6 {
        assert_eq!(
            graph.mask_groups[&target(i)],
            if i == 0 || BONES[i].contains("Leg") {
                1
            } else {
                2
            }
        );
    }
    assert_eq!(graph.graph[nodes["Fire"]].mask, 1);
    assert_eq!(graph.graph[nodes["__lower"]].mask, 2);
    assert_eq!(graph.graph[nodes["Run"]].mask, 0);
    let mut world = World::new();
    let mut queue = bevy::ecs::world::CommandQueue::default();
    {
        let mut commands = Commands::new(&mut queue, &world);
        p.spawn(&mut commands, Transform::default());
        p.spawn(&mut commands, Transform::default());
    }
    queue.apply(&mut world);
    assert_eq!(world.query::<&AnimationPlayer>().iter(&world).count(), 2);
    assert_eq!(
        world
            .query::<(&AnimationTargetId, &AnimatedBy)>()
            .iter(&world)
            .count(),
        12
    );
    let skins: Vec<_> = world.query::<&SkinnedMesh>().iter(&world).collect();
    assert_eq!(skins.len(), 28);
    assert!(skins.iter().all(|s| s.joints.len() == 6));
    assert_eq!(meshes.len(), 14);
}
#[test]
fn ground_is_seeded() {
    assert_eq!(ground_texture(1993).data, ground_texture(1993).data);
    assert_ne!(ground_texture(1993).data, ground_texture(1).data);
}
#[test]
fn preserves_legacy_bounds_and_linear_palette() {
    let baseline: serde_json::Value =
        serde_json::from_str(include_str!("legacy-models.json")).unwrap();
    for name in NAMES {
        // New campaign scenery has no legacy baseline; jungle recipes replace silhouettes.
        if matches!(
            name,
            "hill" | "gravestone" | "palm" | "thicket" | "thicket_edge"
        ) {
            continue;
        }
        let model = generate(name, GenerationOptions::default()).unwrap();
        let (lo, hi) = model.bounds();
        let vector = |key: &str| {
            Vec3::from_array(std::array::from_fn(|i| {
                baseline[name][key][i].as_f64().unwrap() as f32
            }))
        };
        assert!(
            lo.distance(vector("min")) < 0.035,
            "{name} min {lo:?} != {:?}",
            vector("min")
        );
        assert!(
            hi.distance(vector("max")) < 0.035,
            "{name} max {hi:?} != {:?}",
            vector("max")
        );
        for p in &model.parts {
            assert!(
                baseline[name]["materials"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .any(|color| (0..3)
                        .all(|i| (color[i].as_f64().unwrap() as f32 - p.color[i]).abs() < 1e-5)),
                "{name}: material {:?}",
                p.color
            );
        }
    }
}
#[test]
fn ground_has_multiscale_contrast_and_opaque_pixels() {
    let image = ground_texture(DEFAULT_SEED);
    let data = image.data.unwrap();
    assert_eq!(data.len(), 1024 * 1024 * 4);
    assert!(data.chunks_exact(4).all(|p| p[3] == 255));
    let greens: Vec<_> = data.chunks_exact(4).map(|p| p[1]).collect();
    assert!(greens.iter().max().unwrap() - greens.iter().min().unwrap() > 65);
}
#[test]
fn bevy_evaluates_masks_and_procedural_targets() {
    use bevy::{
        animation::AnimationPlugin, asset::AssetPlugin, mesh::skinning::SkinnedMeshInverseBindposes,
    };
    let mut meshes = Assets::<Mesh>::default();
    let mut materials = Assets::<StandardMaterial>::default();
    let mut binds = Assets::<SkinnedMeshInverseBindposes>::default();
    let mut clips = Assets::<AnimationClip>::default();
    let prepared = PreparedModel::new(
        generate("soldier", GenerationOptions::default()).unwrap(),
        &mut meshes,
        &mut materials,
        &mut binds,
        &mut clips,
    );
    let (graph, nodes) = prepared.animation_graph();
    let mut graphs = Assets::<AnimationGraph>::default();
    let graph = graphs.add(graph);
    let mut app = App::new();
    app.add_plugins((
        MinimalPlugins,
        AssetPlugin::default(),
        AnimationPlugin,
        TransformPlugin,
    ))
    .insert_resource(clips)
    .insert_resource(graphs);
    let spawned = prepared.spawn(&mut app.world_mut().commands(), Transform::default());
    app.world_mut().flush();
    let player = spawned.player.unwrap();
    let mut playback = AnimationPlayer::default();
    for name in ["Fire", "__lower"] {
        playback.play(nodes[name]).set_seek_time(7. / 24.).pause();
    }
    app.world_mut()
        .entity_mut(player)
        .insert((playback, AnimationGraphHandle(graph)));
    app.finish();
    app.cleanup();
    for _ in 0..3 {
        app.update();
    }
    for (name, transform) in app
        .world_mut()
        .query::<(&Name, &Transform)>()
        .iter(app.world())
    {
        if let Some(i) = BONES.iter().position(|b| *b == name.as_str()) {
            let pose = Rig::new().pose(
                if i == 0 || name.as_str().contains("Leg") {
                    "Idle"
                } else {
                    "Fire"
                },
                6,
            )[i];
            assert!(
                transform.translation.distance(pose.translation) < 1e-5,
                "{name} translation"
            );
            assert!(
                transform.rotation.dot(pose.rotation).abs() > 0.99999,
                "{name} rotation {:?} != {:?}",
                transform.rotation,
                pose.rotation
            );
        }
    }
}
