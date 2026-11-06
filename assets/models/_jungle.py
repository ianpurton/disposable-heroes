"""Shared thicket construction; preserves legacy random sequences."""
import bpy,random,math
def mat(name,color):
 m=bpy.data.materials.new(name);m.diffuse_color=(*color,1);m.use_nodes=True;p=m.node_tree.nodes.get('Principled BSDF');p.inputs['Base Color'].default_value=(*color,1);p.inputs['Roughness'].default_value=.9;return m
leaves=[mat('Jungle leaf '+str(i),c) for i,c in enumerate([(.035,.12,.025),(.07,.21,.04),(.12,.29,.055),(.20,.35,.075)])];bark=mat('Tangled roots',(.12,.065,.028))

def build_thicket(name):
    bpy.ops.object.select_all(action='SELECT');bpy.ops.object.delete(use_global=False)
    for i in range(20):
     x=random.uniform(-.32,.32);y=random.uniform(-.32,.32);height=.45+random.random()*.65
     bpy.ops.mesh.primitive_ico_sphere_add(subdivisions=1,radius=1,location=(x,y,height*.65));o=bpy.context.object;o.scale=(min(.5-abs(x),.20),min(.5-abs(y),.22),height*.5);o.data.materials.append(leaves[i%4])
    for i in range(5):
     a=i*2.399;bpy.ops.mesh.primitive_cone_add(vertices=5,radius1=.055,radius2=.02,depth=.55,location=(math.cos(a)*.22,math.sin(a)*.22,.28));o=bpy.context.object;o.rotation_euler=(math.sin(a)*.65,math.cos(a)*.65,0);o.data.materials.append(bark)
    bpy.ops.object.select_all(action='SELECT');bpy.context.view_layer.objects.active=bpy.context.selected_objects[0];bpy.ops.object.join();bpy.context.object.name=name; bpy.context.scene.cursor.location=(0,0,0); bpy.ops.object.origin_set(type='ORIGIN_CURSOR')

