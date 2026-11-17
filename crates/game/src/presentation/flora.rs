//! Shared pools outlive level entities; roots own placement and level cleanup.
use super::*;
use bevy_flora::{PooledTree, TreePool};

pub(super) struct Forest {
    pools: [Vec<Handle<TreePool>>; 3],
    pending: std::collections::VecDeque<(Handle<TreePool>, TreePool)>,
    active: Option<Handle<TreePool>>,
    fits: [Vec<Transform>; 3],
}
impl Forest {
    pub fn new(seed: u64, meshes: &mut Assets<Mesh>, pools: &mut Assets<TreePool>) -> Self {
        let envelopes = [
            Vec3::new(3.3, 3.4, 3.3),
            Vec3::new(3., 3.8, 3.),
            Vec3::new(1.45, 1.12, 1.45),
        ];
        let built = models::flora::NAMES.map(|name| models::flora::pool(name, seed, meshes));
        let fits = std::array::from_fn(|i| {
            (0..models::flora::VARIANTS)
                .map(|k| models::flora::fitted_transform(&built[i], k, envelopes[i]))
                .collect()
        });
        // Flora starts four variant bakes at once (64 cameras). Insert only one
        // single-variant pool at a time to bound the work to 16 cameras on every
        // backend. Pending handles keep trees waiting until their atlas is ready.
        let mut pending = std::collections::VecDeque::new();
        let pools = built.map(|pool| {
            pool.variants
                .into_iter()
                .enumerate()
                .map(|(index, variant)| {
                    let handle = pools.reserve_handle();
                    let mut params = pool.params.clone();
                    params.seed = params.seed.wrapping_add(index as i32);
                    pending.push_back((
                        handle.clone(),
                        TreePool {
                            params,
                            age: pool.age,
                            lod: pool.lod.clone(),
                            looks: pool.looks.clone(),
                            materials: vec![],
                            variants: vec![variant],
                        },
                    ));
                    handle
                })
                .collect()
        });
        Self {
            pools,
            fits,
            pending,
            active: None,
        }
    }
    pub fn spawn(&self, commands: &mut Commands, kind: usize, hash: u64, center: Vec3, scale: f32) {
        let variant = (hash % u64::from(models::flora::VARIANTS)) as u32;
        commands
            .spawn((
                LevelEntity,
                Transform::from_translation(center)
                    .with_scale(Vec3::splat(scale))
                    .with_rotation(Quat::from_rotation_y(hash as f32 * 0.618)),
                Visibility::Inherited,
            ))
            .with_children(|parent| {
                parent.spawn((
                    PooledTree {
                        pool: self.pools[kind][variant as usize].clone(),
                        variant: 0,
                    },
                    self.fits[kind][variant as usize],
                ));
            });
    }
    fn advance_baking(&mut self, pools: &mut Assets<TreePool>) {
        if self.active.as_ref().is_some_and(|handle| {
            !pools
                .get(handle)
                .is_some_and(|pool| pool.variants.iter().all(|v| v.impostor.is_some()))
        }) {
            return;
        }
        self.active = None;
        if let Some((handle, pool)) = self.pending.pop_front() {
            if let Err(error) = pools.insert(handle.id(), pool) {
                error!("Failed to insert queued forest pool: {error}");
                return;
            }
            self.active = Some(handle);
        }
    }

    pub fn ready(&self, pools: &Assets<TreePool>) -> bool {
        self.pools.iter().flatten().all(|handle| {
            pools.get(handle).is_some_and(|pool| {
                !pool.lod.distances.impostor.is_finite()
                    || pool
                        .variants
                        .iter()
                        .all(|variant| variant.impostor.is_some())
            })
        })
    }
}

/// Startup baking uses the same bounded queue on native and WebGPU.
pub(super) fn advance_baking(mut library: ResMut<Library>, mut pools: ResMut<Assets<TreePool>>) {
    if library.forest.pending.is_empty() && library.forest.active.is_none() {
        return;
    }
    library.forest.advance_baking(&mut pools);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pooled_children_follow_root_visibility_and_level_cleanup() {
        let mut app = App::new();
        app.add_plugins((
            MinimalPlugins,
            bevy::asset::AssetPlugin::default(),
            bevy::transform::TransformPlugin,
        ))
        .init_asset::<Mesh>()
        .init_asset::<bevy::mesh::skinning::SkinnedMeshInverseBindposes>()
        .add_plugins(bevy::camera::visibility::VisibilityPlugin);
        app.add_systems(Startup, |mut commands: Commands| {
            let forest = Forest {
                pools: std::array::from_fn(|_| vec![Handle::default(); 6]),
                pending: Default::default(),
                active: None,
                fits: std::array::from_fn(|_| vec![Transform::IDENTITY; 6]),
            };
            forest.spawn(&mut commands, 2, 1993, Vec3::new(5., 0., 8.), 0.8);
        });
        app.update();
        let root = app
            .world_mut()
            .query_filtered::<Entity, With<LevelEntity>>()
            .single(app.world())
            .expect("one level root");
        let child = app
            .world_mut()
            .query_filtered::<Entity, With<PooledTree>>()
            .single(app.world())
            .expect("one pooled child");
        assert!(
            app.world()
                .get::<InheritedVisibility>(child)
                .expect("restored visibility")
                .get()
        );
        assert_eq!(
            app.world()
                .get::<ChildOf>(child)
                .expect("attached")
                .parent(),
            root
        );
        app.world_mut().entity_mut(root).insert(Visibility::Hidden);
        app.update();
        assert!(
            !app.world()
                .get::<InheritedVisibility>(child)
                .expect("visibility")
                .get()
        );
        app.world_mut()
            .entity_mut(root)
            .insert(Visibility::Inherited);
        app.update();
        assert!(
            app.world()
                .get::<InheritedVisibility>(child)
                .expect("restored visibility")
                .get()
        );
        assert_eq!(
            app.world().get::<Transform>(root).expect("transform").scale,
            Vec3::splat(0.8)
        );
        app.world_mut().despawn(root);
        assert!(app.world().get_entity(child).is_err());
    }
}
