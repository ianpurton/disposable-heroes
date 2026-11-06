"""Shared Blender geometry and palette; imported only inside Blender."""
import bpy,math,random
from mathutils import Vector

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
