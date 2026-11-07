//! Optional native GLB writer. It consumes generated data, never imported GLBs.
use super::*;
use serde_json::{Value, json};
use std::{io, path::Path};
#[derive(Default)]
struct Buffer {
    data: Vec<u8>,
    views: Vec<Value>,
    accessors: Vec<Value>,
}
impl Buffer {
    fn accessor(
        &mut self,
        bytes: Vec<u8>,
        component: u32,
        kind: &str,
        count: usize,
        min: Option<Value>,
        max: Option<Value>,
    ) -> usize {
        while !self.data.len().is_multiple_of(4) {
            self.data.push(0);
        }
        let start = self.data.len();
        self.data.extend(bytes);
        let view = self.views.len();
        self.views
            .push(json!({"buffer":0,"byteOffset":start,"byteLength":self.data.len()-start}));
        let mut a = json!({"bufferView":view,"componentType":component,"count":count,"type":kind});
        if let Some(v) = min {
            a["min"] = v;
        }
        if let Some(v) = max {
            a["max"] = v;
        }
        let index = self.accessors.len();
        self.accessors.push(a);
        index
    }
    fn floats(&mut self, values: impl IntoIterator<Item = f32>, kind: &str, count: usize) -> usize {
        self.accessor(
            values.into_iter().flat_map(f32::to_le_bytes).collect(),
            5126,
            kind,
            count,
            None,
            None,
        )
    }
}
pub fn glb(model: &Model) -> Result<Vec<u8>, String> {
    model.validate()?;
    let mut b = Buffer::default();
    let mut nodes = vec![json!({"name":model.name,"children":[]})];
    let mut meshes = vec![];
    let mut materials = vec![];
    let mut children = vec![];
    let mut skins = vec![];
    let mut animations = vec![];
    if let Some(rig) = &model.rig {
        nodes.push(json!({"name":"SoldierRig","children":[2]}));
        children.push(1);
        for (i, t) in rig.rest.iter().enumerate() {
            let mut n = json!({"name":BONES[i],"translation":t.translation.to_array(),"rotation":t.rotation.to_array(),"scale":t.scale.to_array()});
            if i == 0 {
                n["children"] = json!([3, 4, 5, 6, 7]);
            }
            nodes.push(n);
        }
        let bind = b.floats(
            rig.inverse_bind().iter().flat_map(|m| m.to_cols_array()),
            "MAT4",
            6,
        );
        skins.push(json!({"name":"SoldierRig","joints":[2,3,4,5,6,7],"skeleton":2,"inverseBindMatrices":bind}));
        let times: Vec<_> = (0..25).map(|f| (f + 1) as f32 / 24.).collect();
        let input = b.accessor(
            times.iter().flat_map(|f| f.to_le_bytes()).collect(),
            5126,
            "SCALAR",
            25,
            Some(json!([times[0]])),
            Some(json!([times[24]])),
        );
        for name in CLIPS {
            let mut samplers = vec![];
            let mut channels = vec![];
            for i in 0..6 {
                let poses: Vec<_> = (0..25).map(|f| rig.pose(name, f)[i]).collect();
                for path in ["translation", "rotation"] {
                    let output = if path == "translation" {
                        b.floats(
                            poses.iter().flat_map(|t| t.translation.to_array()),
                            "VEC3",
                            25,
                        )
                    } else {
                        b.floats(poses.iter().flat_map(|t| t.rotation.to_array()), "VEC4", 25)
                    };
                    let s = samplers.len();
                    samplers.push(json!({"input":input,"output":output,"interpolation":"LINEAR"}));
                    channels.push(json!({"sampler":s,"target":{"node":i+2,"path":path}}));
                }
            }
            animations.push(json!({"name":name,"samplers":samplers,"channels":channels}));
        }
    }
    for p in &model.parts {
        let g = &p.geometry;
        let mut lo = Vec3::splat(f32::INFINITY);
        let mut hi = -lo;
        for v in &g.positions {
            let v = Vec3::from_array(*v);
            lo = lo.min(v);
            hi = hi.max(v);
        }
        let positions = b.accessor(
            g.positions
                .iter()
                .flatten()
                .flat_map(|v| v.to_le_bytes())
                .collect(),
            5126,
            "VEC3",
            g.positions.len(),
            Some(json!(lo.to_array())),
            Some(json!(hi.to_array())),
        );
        let normals = b.floats(g.normals.iter().flatten().copied(), "VEC3", g.normals.len());
        let indices = b.accessor(
            g.indices.iter().flat_map(|i| i.to_le_bytes()).collect(),
            5125,
            "SCALAR",
            g.indices.len(),
            None,
            None,
        );
        let mut attrs = json!({"POSITION":positions,"NORMAL":normals});
        if let Some(j) = p.joint {
            let joints = b.accessor(
                (0..g.positions.len())
                    .flat_map(|_| [j as u16, 0, 0, 0].into_iter().flat_map(u16::to_le_bytes))
                    .collect(),
                5123,
                "VEC4",
                g.positions.len(),
                None,
                None,
            );
            let weights = b.floats(
                (0..g.positions.len()).flat_map(|_| [1., 0., 0., 0.]),
                "VEC4",
                g.positions.len(),
            );
            attrs["JOINTS_0"] = json!(joints);
            attrs["WEIGHTS_0"] = json!(weights);
        }
        let m = materials.len();
        materials.push(json!({"name":format!("{} material",p.name),"pbrMetallicRoughness":{"baseColorFactor":[p.color[0],p.color[1],p.color[2],1.],"metallicFactor":0.,"roughnessFactor":p.roughness}}));
        let mesh = meshes.len();
        meshes.push(json!({"name":p.name,"primitives":[{"attributes":attrs,"indices":indices,"material":m,"mode":4}]}));
        let mut node = json!({"name":p.name,"mesh":mesh});
        if p.joint.is_some() {
            node["skin"] = json!(0);
        }
        children.push(nodes.len());
        nodes.push(node);
    }
    nodes[0]["children"] = json!(children);
    let mut document = json!({"asset":{"version":"2.0","generator":"models Rust procedural assets"},"scene":0,"scenes":[{"nodes":[0]}],"nodes":nodes,"meshes":meshes,"materials":materials,"buffers":[{"byteLength":b.data.len()}],"bufferViews":b.views,"accessors":b.accessors});
    if !skins.is_empty() {
        document["skins"] = json!(skins);
        document["animations"] = json!(animations);
    }
    let mut json = serde_json::to_vec(&document).map_err(|e| e.to_string())?;
    while !json.len().is_multiple_of(4) {
        json.push(b' ');
    }
    while !b.data.len().is_multiple_of(4) {
        b.data.push(0);
    }
    let mut out = vec![];
    out.extend(b"glTF");
    out.extend(2u32.to_le_bytes());
    out.extend(((28 + json.len() + b.data.len()) as u32).to_le_bytes());
    out.extend((json.len() as u32).to_le_bytes());
    out.extend(0x4e4f534au32.to_le_bytes());
    out.extend(json);
    out.extend((b.data.len() as u32).to_le_bytes());
    out.extend(0x004e4942u32.to_le_bytes());
    out.extend(b.data);
    Ok(out)
}
pub fn write(model: &Model, path: &Path) -> io::Result<()> {
    let bytes = glb(model).map_err(io::Error::other)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let stage = path.with_extension("glb.tmp");
    std::fs::write(&stage, bytes)?;
    std::fs::rename(stage, path)
}
