"""Render a contact sheet of the generated library into ./tmp."""
import bpy,pathlib,math
from mathutils import Vector
root=pathlib.Path(__file__).resolve().parents[1];bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
names=['soldier','enemy','palm','tree','bush','hut','bunker','jeep','bridge','wreck','rock','crate']
for i,name in enumerate(names):
 before=set(bpy.data.objects);bpy.ops.import_scene.gltf(filepath=str(root/'assets/models'/f'{name}.glb'))
 for o in set(bpy.data.objects)-before:
  if not o.parent:o.location+=Vector(((i%4)*5,(i//4)*5,0))
 bpy.ops.object.text_add(location=((i%4)*5-1.4,(i//4)*5-1.8,.05));text=bpy.context.object;text.data.body=name.upper();text.data.size=.36
bpy.ops.mesh.primitive_plane_add(size=200);floor=bpy.context.object;floor.location.z=-.04;m=bpy.data.materials.new('Backdrop');m.diffuse_color=(.09,.13,.10,1);floor.data.materials.append(m)
bpy.ops.object.camera_add(location=(22,-23,27));cam=bpy.context.object;target=Vector((7.5,5,0));cam.rotation_euler=(target-cam.location).to_track_quat('-Z','Y').to_euler();cam.data.type='ORTHO';cam.data.ortho_scale=25;bpy.context.scene.camera=cam
bpy.ops.object.light_add(type='AREA',location=(4,-5,18));bpy.context.object.data.energy=2600;bpy.context.object.data.shape='DISK';bpy.context.object.data.size=12
scene=bpy.context.scene;scene.render.engine='CYCLES';scene.cycles.samples=16;scene.world.color=(.3,.3,.3);scene.render.resolution_x=1400;scene.render.resolution_y=1000;scene.render.resolution_percentage=100;scene.render.filepath=str(root/'tmp/asset-library.png');bpy.ops.render.render(write_still=True)
