"""Generate merged leafy thicket GLBs; footprint is one meter square."""
import bpy,random,math,pathlib
random.seed(1993);root=pathlib.Path(__file__).resolve().parents[1]
def mat(name,color):
 m=bpy.data.materials.new(name);m.diffuse_color=(*color,1);m.use_nodes=True;p=m.node_tree.nodes.get('Principled BSDF');p.inputs['Base Color'].default_value=(*color,1);p.inputs['Roughness'].default_value=.9;return m
leaves=[mat('Jungle leaf '+str(i),c) for i,c in enumerate([(.035,.12,.025),(.07,.21,.04),(.12,.29,.055),(.20,.35,.075)])];bark=mat('Tangled roots',(.12,.065,.028))
for name in ['thicket','thicket_edge']:
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
 for i in range(20):
  x=random.uniform(-.32,.32);y=random.uniform(-.32,.32);height=.45+random.random()*.65
  bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1,radius=1,location=(x,y,height*.65));o=bpy.context.object;o.scale=(min(.5-abs(x),.20),min(.5-abs(y),.22),height*.5);o.data.materials.append(leaves[i%4])
 for i in range(5):
  a=i*2.399;bpy.ops.mesh.primitive_cone_add(vertices=5,radius1=.055,radius2=.02,depth=.55,location=(math.cos(a)*.22,math.sin(a)*.22,.28));o=bpy.context.object;o.rotation_euler=(math.sin(a)*.65,math.cos(a)*.65,0);o.data.materials.append(bark)
 bpy.ops.object.select_all(action='SELECT');bpy.context.view_layer.objects.active=bpy.context.selected_objects[0];bpy.ops.object.join();bpy.context.object.name=name; bpy.context.scene.cursor.location=(0,0,0); bpy.ops.object.origin_set(type='ORIGIN_CURSOR')
 bpy.ops.export_scene.gltf(filepath=str(root/'assets/models'/f'{name}.glb'),export_format='GLB',export_animations=False)
print('THICKET ASSETS COMPLETE')
# Baked, tileable painted ground detail for Rust-generated terrain meshes.
size=256;image=bpy.data.images.new('Painted jungle ground',width=size,height=size);pixels=[]
for y in range(size):
 for x in range(size):
  grain=random.uniform(-.035,.035);wave=math.sin(x/size*math.tau*7)*math.sin(y/size*math.tau*5)*.025
  blade=.10 if random.random()<.025 else 0
  pixels.extend((.82+grain+wave+blade,.87+grain+wave+blade,.67+grain+wave,1))
image.pixels.foreach_set(pixels);out=root/'assets/textures';out.mkdir(exist_ok=True);image.filepath_raw=str(out/'jungle_ground.png');image.file_format='PNG';image.save()
import json
manifest_path=root/'assets/models/manifest.json'
manifest=json.loads(manifest_path.read_text())
manifest['assets']=sorted(p.name for p in (root/'assets/models').glob('*.glb'))
manifest['jungle_generator']='scripts/generate_jungle.py; seed 1993; footprint one meter; pivot ground center'
manifest_path.write_text(json.dumps(manifest,indent=2)+'\n')
