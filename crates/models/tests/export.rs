#![cfg(all(feature = "export", not(target_arch = "wasm32")))]
use models::*;
#[test]
fn exported_models_validate_and_round_trip() {
    for name in NAMES {
        let model = generate(name, GenerationOptions::default()).unwrap();
        let bytes = export::glb(&model).unwrap();
        let file = gltf::Gltf::from_slice(&bytes).unwrap();
        assert_eq!(file.meshes().count(), model.parts.len());
        let binary = file.blob.as_ref().unwrap();
        for (mesh, part) in file.meshes().zip(&model.parts) {
            let primitive = mesh.primitives().next().unwrap();
            let reader = primitive.reader(|_| Some(binary));
            let positions: Vec<_> = reader.read_positions().unwrap().collect();
            assert_eq!(positions, part.geometry.positions);
            assert_eq!(
                primitive
                    .material()
                    .pbr_metallic_roughness()
                    .roughness_factor(),
                part.roughness
            );
            if part.joint.is_some() {
                assert!(
                    reader
                        .read_joints(0)
                        .unwrap()
                        .into_u16()
                        .all(|j| j[0] as usize == part.joint.unwrap())
                );
                assert!(
                    reader
                        .read_weights(0)
                        .unwrap()
                        .into_f32()
                        .all(|w| w == [1., 0., 0., 0.])
                );
            }
        }
        if let Some(rig) = &model.rig {
            let names: Vec<_> = file.animations().map(|a| a.name().unwrap()).collect();
            assert_eq!(names, CLIPS);
            let skin = file.skins().next().unwrap();
            assert_eq!(skin.joints().count(), 6);
            let inverse: Vec<_> = skin
                .reader(|_| Some(binary))
                .read_inverse_bind_matrices()
                .unwrap()
                .collect();
            for (a, b) in inverse.iter().zip(rig.inverse_bind()) {
                assert_eq!(*a, b.to_cols_array_2d());
            }
            for animation in file.animations() {
                for channel in animation.channels() {
                    let reader = channel.reader(|_| Some(binary));
                    assert_eq!(reader.read_inputs().unwrap().count(), 25);
                    assert!(reader.read_outputs().is_some());
                }
            }
        } else {
            assert_eq!(file.animations().count(), 0);
            assert_eq!(file.skins().count(), 0);
        }
    }
}
