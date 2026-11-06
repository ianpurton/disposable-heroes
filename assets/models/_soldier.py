"""Shared soldier mesh, rig and named animation construction."""
from _common import *

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
 rig.animation_data.action=None
