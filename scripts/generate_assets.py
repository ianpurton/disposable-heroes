"""Blender offline asset factory. Run: blender -b -t 2 --python scripts/generate_assets.py"""
import bpy,math,random,pathlib,json
from mathutils import Vector
ROOT=pathlib.Path(__file__).resolve().parents[1]; OUT=ROOT/'assets/models';OUT.mkdir(parents=True,exist_ok=True)
random.seed(1993)
def material(name,color,rough=.85):
 m=bpy.data.materials.new(name);m.diffuse_color=(*color,1);m.use_nodes=True
 bs=m.node_tree.nodes.get('Principled BSDF');bs.inputs['Base Color'].default_value=(*color,1);bs.inputs['Roughness'].default_value=rough
 return m
olive=material('Olive cloth',(.20,.28,.095));enemy=material('Rust cloth',(.43,.19,.10));skin=material('Warm skin',(.63,.39,.22));dark=material('Charcoal',(.055,.065,.048));wood=material('Weathered teak',(.24,.13,.055));sand=material('Sandstone',(.42,.40,.25));leaf=[material('Leaf '+str(i),c) for i,c in enumerate([(.12,.27,.09),(.22,.40,.12),(.32,.46,.15),(.08,.19,.10)])];roof=material('Thatch',(.48,.36,.13));metal=material('Gunmetal',(.12,.15,.13),.45)
def clear():
 bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
def cube(name,pos,size,mat,bevel=.03):
 bpy.ops.mesh.primitive_cube_add(size=1,location=pos);o=bpy.context.object;o.name=name;o.dimensions=size;bpy.ops.object.transform_apply(location=False,rotation=False,scale=True);o.data.materials.append(mat)
 if bevel:
  mod=o.modifiers.new('Soft painted edges','BEVEL');mod.width=bevel;mod.segments=2;bpy.context.view_layer.objects.active=o;bpy.ops.object.modifier_apply(modifier=mod.name)
 return o
def ico(name,pos,size,mat,sub=1):
 bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=sub,radius=1,location=pos);o=bpy.context.object;o.name=name;o.scale=size;o.data.materials.append(mat);return o
def rod(name,a,b,r,mat):
 d=Vector(b)-Vector(a);bpy.ops.mesh.primitive_cone_add(vertices=8,radius1=r,radius2=r*.75,depth=d.length,location=(Vector(a)+Vector(b))/2);o=bpy.context.object;o.name=name;o.rotation_euler=d.to_track_quat('Z','Y').to_euler();o.data.materials.append(mat);return o
def export(name):
 bpy.ops.export_scene.gltf(filepath=str(OUT/(name+'.glb')),export_format='GLB',export_animations=True,export_animation_mode='ACTIONS',export_force_sampling=True)
def soldier(name,cloth):
 clear();bpy.ops.object.armature_add();rig=bpy.context.object;rig.name='SoldierRig';bpy.ops.object.mode_set(mode='EDIT');bones=rig.data.edit_bones;bones.remove(bones[0])
 spec={'Root':((0,0,.75),(0,0,1.2),None),'Head':((0,0,1.2),(0,0,1.65),'Root'),'LeftLeg':((-.17,0,.78),(-.17,0,.12),'Root'),'RightLeg':((.17,0,.78),(.17,0,.12),'Root'),'LeftArm':((-.32,0,1.15),(-.32,-.25,.8),'Root'),'RightArm':((.32,0,1.15),(.32,-.25,.8),'Root')}
 for n,(a,b,parent) in spec.items():
  bone=bones.new(n);bone.head=a;bone.tail=b
  if parent:bone.parent=bones[parent]
 bpy.ops.object.mode_set(mode='OBJECT')
 def bind(o,b):
  bpy.context.view_layer.objects.active=o;o.select_set(True);bpy.ops.object.transform_apply(location=False,rotation=False,scale=True)
  vg=o.vertex_groups.new(name=b);vg.add(list(range(len(o.data.vertices))),1,'REPLACE');mod=o.modifiers.new('Skin','ARMATURE');mod.object=rig;o.parent=rig
 bind(cube('Tunic',(0,0,1.02),(.6,.38,.52),cloth,.09),'Root')
 bind(cube('Pack',(0,.25,1.02),(.42,.24,.40),wood,.05),'Root')
 bind(ico('Face',(0,-.035,1.48),(.25,.23,.27),skin,2),'Head');bind(ico('Helmet',(0,.01,1.65),(.31,.29,.18),cloth,2),'Head')
 for side,x in [('Left',-.17),('Right',.17)]:
  bind(cube(side+' trouser',(x,0,.46),(.24,.27,.56),cloth,.05),side+'Leg');bind(cube(side+' boot',(x,-.09,.12),(.25,.42,.20),dark,.04),side+'Leg')
 for side,x in [('Left',-.36),('Right',.36)]:
  bind(rod(side+' sleeve',(x,0,1.15),(x,-.23,.85),.13,cloth),side+'Arm');bind(ico(side+' hand',(x,-.28,.84),(.115,.12,.12),skin,2),side+'Arm')
 bind(cube('Rifle',(.3,-.48,.88),(.10,.75,.12),wood,.02),'RightArm');bind(rod('Barrel',(.3,-.65,.89),(.3,-1.05,.89),.035,metal),'RightArm')
 rig.animation_data_create()
 for clip in ['Idle','Run','Fire','Throw','Rocket','Swim','Death']:
  action=bpy.data.actions.new(clip);rig.animation_data.action=action;action.use_fake_user=True
  for frame in range(1,26):
   phase=(frame-1)/24*math.tau
   for bone in rig.pose.bones:
    bone.rotation_mode='XYZ';bone.rotation_euler=(0,0,0);bone.location=(0,0,0)
    if clip=='Run':
     if 'Leg' in bone.name:bone.rotation_euler.x=math.sin(phase)*.65*(1 if 'Left' in bone.name else -1)
     if 'Arm' in bone.name:bone.rotation_euler.x=math.sin(phase)*.3*(1 if 'Right' in bone.name else -1)
     if bone.name=='Root':bone.location.y=abs(math.sin(phase))*.055
    elif clip=='Idle' and bone.name=='Root':bone.location.y=math.sin(phase)*.012
    elif clip in ['Fire','Rocket'] and 'Arm' in bone.name:bone.rotation_euler.x=-.6+math.sin(phase)*.06
    elif clip=='Throw' and bone.name=='RightArm':bone.rotation_euler.x=-math.sin((frame-1)/24*math.pi)*2.
    elif clip=='Swim' and 'Arm' in bone.name:bone.rotation_euler.z=math.sin(phase)*.8
    elif clip=='Death' and bone.name=='Root':bone.rotation_euler.x=min(1,(frame-1)/16)*1.5;bone.location.y=-min(1,(frame-1)/16)*.6
    bone.keyframe_insert('rotation_euler',frame=frame);bone.keyframe_insert('location',frame=frame)
 rig.animation_data.action=None;export(name)
soldier('soldier',olive);soldier('enemy',enemy)
clear()
for i in range(7):rod('Palm trunk',(math.sin(i*.14)*.3,0,i*.55),(math.sin((i+1)*.14)*.3,0,(i+1)*.55),.17,wood)
for k in range(9):
 a=k*math.tau/9
 for j in range(4):
  r=.35+j*.5;o=ico('Palm frond',(math.cos(a)*r+.3,math.sin(a)*r,3.8+.3*math.sin(j)-j*.18),(.7,.22,.08),leaf[k%4],1);o.rotation_euler.z=a
export('palm')
clear();rod('Trunk',(0,0,0),(0,0,2.6),.24,wood)
for i in range(9):a=i*2.4;ico('Broadleaf crown',(math.cos(a)*.8,math.sin(a)*.8,2.6+(i%3)*.35),(1.05,.85,.8),leaf[i%4],2)
export('tree')
clear()
for i in range(9):a=i*2.4;ico('Shrub',(math.cos(a)*.35,math.sin(a)*.35,.3+i%3*.12),(.45,.27,.3),leaf[i%4],1)
export('bush')
clear()
for i in range(7):a=i*2.4;rod('Grass blade',(0,0,0),(math.cos(a)*.26,math.sin(a)*.26,.3+i%3*.1),.035,leaf[i%4])
export('grass')
clear();ico('Rock',(0,0,.35),(.7,.55,.5),sand,2);export('rock')
clear()
for x in [-1.5,1.5]:
 for y in [-1.1,1.1]:rod('Post',(x,y,0),(x,y,2.1),.13,wood)
for x in [-1.45,1.45]:cube('Side wall',(x,0,1.),(.15,2.2,1.8),wood)
cube('Back wall',(0,1.05,1.),(3.,.15,1.8),wood)
for x in [-1.,1.]:cube('Front wall',(x,-1.05,1.),(.9,.15,1.8),wood)
for x in [-.9,.9]:o=cube('Thatch roof',(x,0,2.15),(2.,2.8,.22),roof);o.rotation_euler.y=math.copysign(.42,x)
for i in range(16):o=cube('Roof straw',(-1.75+i*.23,0,2.55-abs(-1.75+i*.23)*.43),(.12,2.85,.12),roof)
export('hut')
clear();cube('Concrete bunker',(0,0,.65),(3.2,2.4,1.3),sand,.15);cube('Bunker roof',(0,0,1.4),(3.5,2.7,.25),olive,.08);cube('Firing slit',(0,-1.21,.85),(1.8,.03,.25),dark,0);export('bunker')
clear();cube('Jeep body',(0,0,.6),(1.4,2.4,.5),olive);cube('Bonnet',(0,-.7,.95),(1.3,1.,.3),olive)
for x in [-.75,.75]:
 for y in [-.8,.8]:o=rod('Wheel',(x-.12,y,.4),(x+.12,y,.4),.36,dark)
for x in [-.5,.5]:cube('Seat',(x,.5,1.),(.4,.6,.45),wood)
export('jeep')
clear()
for i in range(12):o=cube('Charred debris',(random.uniform(-1.5,1.5),random.uniform(-1,1),random.uniform(.05,.35)),(random.uniform(.3,1.2),.15,.15),dark);o.rotation_euler.z=random.random()*math.tau
export('wreck')
clear()
for i in range(10):cube('Bridge plank',(0,-2.+i*.44,.1),(2.,.4,.2),wood)
for x in [-.9,.9]:rod('Rail',(x,-2.,.9),(x,2.,.9),.07,wood)
export('bridge')
clear();cube('Ammo crate',(0,0,.25),(.6,.6,.5),olive);cube('Marking',(0,-.31,.28),(.35,.02,.12),roof);export('crate')
manifest={'generator':'Blender Python; seed 1993','units':'meters; Blender Z-up converted to glTF Y-up','animations':['Idle','Run','Fire','Throw','Rocket','Swim','Death'],'assets':sorted(p.name for p in OUT.glob('*.glb'))};(OUT/'manifest.json').write_text(json.dumps(manifest,indent=2))
print('ASSET LIBRARY COMPLETE')
