import struct
from dataclasses import dataclass
from io import BytesIO
from typing import List
from typing import Tuple
from typing import Union
from time import time

import bpy as bpy
from bpy.props import EnumProperty
from bpy.props import BoolProperty
from bpy.props import StringProperty
from bpy.types import Armature
from bpy.types import ArmatureModifier
from bpy.types import Context
from bpy.types import Mesh
from bpy.types import Object
from bpy_extras.io_utils import ExportHelper
from bpy_extras.io_utils import orientation_helper
from bpy_extras.io_utils import axis_conversion
import zlib
import mathutils

VERSION = (1, 3, 1)

bl_info = {
    "name": "Space3 format",
    "author": "Lebedev Games Team",
    "version": VERSION,
    "blender": (3, 0, 0),
    "location": "File > Import-Export",
    "description": "Space3 IO meshes, UV's, vertex colors, materials, textures, cameras, lamps and actions",
    "warning": "",
    "support": 'COMMUNITY',
    "category": "Import-Export",
}


@dataclass
class S3Vertex:
    position: Tuple[float, float, float] = (0.0, 0.0, 0.0)
    normal: Tuple[float, float, float] = (0.0, 0.0, 0.0)
    uv: Tuple[float, float] = (0.0, 0.0)
    bones: Tuple[int, int, int, int] = (-1, -1, -1, -1)
    weights: Tuple[float, float, float, float] = (0.0, 0.0, 0.0, 0.0)


@dataclass
class S3Mesh:
    name: str
    vertices: List[S3Vertex]
    triangles: List[int]


@dataclass
class S3Channel:
    node: int
    position: Tuple[float, float, float]
    rotation: Tuple[float, float, float, float]
    scale: Tuple[float, float, float]


@dataclass
class S3Keyframe:
    channels: List[S3Channel]


@dataclass
class S3Bone:
    name: str
    parent: int
    matrix: List[List[float]]


@dataclass
class S3Armature:
    bones: List[S3Bone]


@dataclass
class S3Animation:
    name: str
    armature: S3Armature
    keyframes: List[S3Keyframe]


@dataclass
class S3Scene:
    meshes: List[S3Mesh]
    animation: S3Animation


class Writer:

    def __init__(self, buffer: BytesIO):
        self.buffer = buffer

    def write_name(self, value: str):
        name = value.encode('utf-8')
        name_length = len(name)
        if name_length > 255:
            raise ValueError(
                f'Unable to pack scene with {name_length} name UTF-8 length '
                f'({value}), max 255'
            )
        self.write_byte(name_length)
        self.write_bytes(name)

    def write_int(self, value: int):
        self.buffer.write(struct.pack('i', value))

    def write_byte(self, value: int):
        self.buffer.write(struct.pack('B', value))

    def write_bytes(self, value: bytes):
        self.buffer.write(value)

    def write_vertex(self, value: S3Vertex):
        self.buffer.write(struct.pack(
            '3f3f2f4b4f',
            value.position[0],
            value.position[1],
            value.position[2],
            value.normal[0],
            value.normal[1],
            value.normal[2],
            value.uv[0],
            value.uv[1],
            value.bones[0],
            value.bones[1],
            value.bones[2],
            value.bones[3],
            value.weights[0],
            value.weights[1],
            value.weights[2],
            value.weights[3],
        ))

    def write_mat4(self, value: List[List[float]]):
        for row in value:
            self.buffer.write(struct.pack('4f', *row))

    def write_bone(self, bone: S3Bone):
        self.write_name(bone.name)
        self.write_int(bone.parent)
        self.write_mat4(bone.matrix)

    def write_channel(self, value: S3Channel):
        self.buffer.write(struct.pack(
            'i3f4f3f',
            value.node,
            value.position[0],
            value.position[1],
            value.position[2],
            value.rotation[1],  # X (reorder Blender quaternion)
            value.rotation[2],  # Y
            value.rotation[3],  # Z
            value.rotation[0],  # W
            value.scale[0],
            value.scale[1],
            value.scale[2],
        ))

def pack_scene(scene: S3Scene) -> BytesIO:
    writer = Writer(BytesIO())

    file_magic = b'Scene3'
    writer.write_bytes(file_magic)

    meshes_length = len(scene.meshes)
    if meshes_length > 255:
        raise ValueError(f'Unable to pack scene with {meshes_length} meshes, max 255')
    writer.write_byte(meshes_length)
    for mesh in scene.meshes:
        writer.write_name(mesh.name)

        archive = Writer(BytesIO())
        vertices_length = len(mesh.vertices)
        if vertices_length > 2147483646:
            raise ValueError(f'Unable to pack scene with {vertices_length} vertices, max 2147483646')
        archive.write_int(vertices_length)
        for vertex in mesh.vertices:
            archive.write_vertex(vertex)
        archive = zlib.compress(archive.buffer.getvalue())
        writer.write_int(len(archive))
        writer.write_bytes(archive)

        triangles_length = len(mesh.triangles)
        if triangles_length > 2147483646:
            raise ValueError(f'Unable to pack scene with {triangles_length} triangles, max 2147483646')
        writer.write_int(triangles_length)
        for index in mesh.triangles:
            writer.write_int(index)

    animation = scene.animation
    writer.write_name(animation.name)

    armature = animation.armature
    bones_length = len(armature.bones)
    if bones_length > 255:
        raise ValueError(f'Unable to pack scene with {bones_length} bones, max 255')
    writer.write_byte(bones_length)
    for bone in armature.bones:
        writer.write_bone(bone)

    archive = Writer(BytesIO())
    keyframes_length = len(animation.keyframes)
    archive.write_int(keyframes_length)
    for keyframe in animation.keyframes:
        channels_length = len(keyframe.channels)
        if channels_length > 255:
            raise ValueError(f'Unable to pack scene with {channels_length} channels, max 255')
        archive.write_int(channels_length)
        for channel in keyframe.channels:
            archive.write_channel(channel)
    archive = zlib.compress(archive.buffer.getvalue())
    writer.write_int(len(archive))
    writer.write_bytes(archive)

    return writer.buffer


def main(context: Context, orientation, report, output_path: str, use_selection: bool):
    scene = S3Scene([], S3Animation('', S3Armature([]), []))

    if use_selection:
        objects = context.selected_objects
    else:
        objects = context.scene.objects

    for ob in objects:
        report(f'!!! ob {type(ob)}, {ob}, name: {ob.name}, {ob.type}: {ob.data}')

        if isinstance(ob.data, Armature):
            armature = ob.data
            bone_index = {
            }
            for index, bone in enumerate(armature.bones):
                bone_index[bone] = index
                bone_offset = orientation @ bone.matrix_local
                s3_bone = S3Bone(
                    name=bone.name,
                    parent=bone_index[bone.parent] if bone.parent else -1,
                    matrix=[list(row) for row in bone_offset]
                )
                scene.animation.armature.bones.append(s3_bone)
                report(f'bone {s3_bone.name} p={s3_bone.parent} children:{list(bone.children)}')

            report(f'animation {ob.animation_data.action.name}')
            scene.animation.name = ob.animation_data.action.name
            for frame in range(context.scene.frame_end):
                context.scene.frame_set(frame)
                channels = []
                report(f"FRAME: {frame}")
                for index, bone in enumerate(ob.pose.bones):
                    # target = {
                    #     0: mathutils.Vector((-1.0, 0.0, 1.0))
                    # }.get(index, mathutils.Vector((-1.0, 0.0, 1.0)))
                    #
                    # report(
                    #     f'{bone.name} '
                    #     f'pos: {tuple(bone.location)} rot: {tuple(bone.rotation_quaternion)}'
                    # )
                    #
                    # point = mathutils.Matrix.Translation(target)
                    # point_vec = point.to_translation()
                    #
                    # point_local = (bone.bone.matrix_local.inverted() @ point)
                    # point_local_vec = point_local.to_translation()
                    #
                    # local = (bone.matrix_basis @ point_local)
                    # local_vec = local.to_translation()
                    # # report(f"LOCAL {point_local_vec} to {local_vec}")
                    #
                    # calc = (bone.bone.matrix_local @ local)
                    # calc_vec = calc.to_translation()
                    #
                    # # transform to Blender|model space @ (bone transform @ (project to bone space @ point))
                    # matrix_basis = mathutils.Matrix.LocRotScale(
                    #     bone.location,
                    #     bone.rotation_quaternion,
                    #     bone.scale
                    # )
                    # # matrix_basis == bone.matrix_basis
                    # expr = bone.bone.matrix_local @ matrix_basis @ bone.bone.matrix_local.inverted() @ point
                    # expr_vec = expr.to_translation()
                    #
                    # result = (bone.matrix_channel @ point).to_translation()
                    # report(f"RESULT {point_vec} to {result} ? {calc_vec} e {expr_vec}")
                    #
                    # report("WITH ORIENTATION:")
                    #
                    # bone_bone_matrix_local = orientation @ bone.bone.matrix_local
                    #
                    # report(f"orientation {orientation}")
                    # report(f"bone_bone_matrix_local {bone_bone_matrix_local}")
                    #
                    # point = orientation @ mathutils.Matrix.Translation(target)
                    # point_vec = point.to_translation()
                    #
                    # point_local = (bone_bone_matrix_local.inverted() @ point)
                    # point_local_vec = point_local.to_translation()
                    #
                    # local = (bone.matrix_basis @ point_local)
                    # local_vec = local.to_translation()
                    # report(f"LOCAL {point_local_vec} to {local_vec}")
                    #
                    # calc = (bone_bone_matrix_local @ local)
                    # calc_vec = calc.to_translation()
                    #
                    # # transform to Blender|model space @ (bone transform @ (project to bone space @ point))
                    # matrix_basis = mathutils.Matrix.LocRotScale(
                    #     bone.location,
                    #     bone.rotation_quaternion,
                    #     bone.scale
                    # )
                    # # matrix_basis == bone.matrix_basis
                    # expr = bone_bone_matrix_local @ matrix_basis @ bone_bone_matrix_local.inverted() @ point
                    # expr_vec = expr.to_translation()
                    #
                    # result = (orientation @ bone.matrix_channel @ point).to_translation()
                    # report(f"RESULT {point_vec} to {result} ? {calc_vec} e {expr_vec}")
                    #
                    # position, rotation, scale = (orientation @ bone.matrix).decompose()
                    # bone_offset = (orientation @ bone.bone.matrix_local)

                    channel = S3Channel(
                        node=0,  # TODO: bone <-> vertex group mapping
                        position=tuple(bone.location),
                        rotation=tuple(bone.rotation_quaternion),
                        scale=tuple(bone.scale),
                    )
                    channels.append(channel)
                    report(
                        f'{bone.name} ({bone.bone_group_index}) '
                        f'pos: {channel.position} rot: {channel.rotation} scale: {channel.scale}'
                    )

                scene.animation.keyframes.append(S3Keyframe(
                    channels=channels
                ))

        if isinstance(ob.data, Mesh):
            mesh = ob.to_mesh()
            mesh.transform(orientation)

            mesh.calc_loop_triangles()
            uv_data = mesh.uv_layers.active.data

            for mod in ob.modifiers:
                if isinstance(mod, ArmatureModifier):
                    report(f'arm mod {mod.name} vg: {mod.vertex_group}, object.data: {mod.object.data}')

            for group in ob.vertex_groups:
                report(f'vgroup: {group.name} ({group.index})')

            report(
                f"triangles: {len(mesh.loop_triangles)}"
            )
            report(
                f"polygons: {len(mesh.polygons)}"
            )

            # uv_index = {}
            # for triangle in mesh.loop_triangles:
            #     for loop in triangle.loops:
            #         uv = uv_data[loop].uv
            #         uv_index[tuple(uv)] = 1
            #
            # report(f"mesh loops: {len(mesh.loops)}, vertices: {len(mesh.vertices)}, uv: {len(uv_data)}, uv_index: {len(uv_index)}")

            vertex_index = {}
            vertices = []
            triangles = []
            for triangle in mesh.loop_triangles:
                for v_index, loop in zip(triangle.vertices, triangle.loops):
                    uv = uv_data[loop].uv
                    vertex = mesh.vertices[v_index]
                    bones = [-1, -1, -1, -1]
                    weights = [0.0, 0.0, 0.0, 0.0]
                    for n, group in enumerate(vertex.groups):
                        if n > 3:
                            report(f'{len(vertex.groups)} bones affects vertex, but max 4 supported', 'ERROR')
                            break
                        bones[n] = group.group
                        weights[n] = group.weight

                    vertex_key = v_index, tuple(uv)

                    index = vertex_index.get(vertex_key, None)
                    if index is None:
                        index = len(vertices)
                        vertex_index[vertex_key] = index
                        vertices.append(S3Vertex(
                            position=tuple(vertex.co),
                            normal=tuple(vertex.normal),
                            uv=tuple(uv),
                            bones=tuple(bones),
                            weights=tuple(weights)
                        ))

                    triangles.append(index)

                    report(
                        f'triangle #{triangle.index} vertex {index} position: {vertex.co}, '
                        f'normal: {vertex.normal}, uv: {list(uv)}, bones: {bones}, weights: {weights}'
                    )

            report(f"Vertices: {len(vertices)}, triangles: {len(triangles)}")

            out_mesh = S3Mesh(
                name=mesh.name,
                vertices=vertices,
                triangles=triangles
            )
            scene.meshes.append(out_mesh)

    t = time()
    with open(output_path, 'wb') as output:
        buffer = pack_scene(scene)
        data = buffer.getvalue()
        output.write(data)
        data_length = len(data)
    report(f'{VERSION} Data length: {data_length} bytes, Export time: {time() - t} seconds')


# Vulkan orientation by default
@orientation_helper(axis_forward='Z', axis_up='-Y')
class Space3Export(bpy.types.Operator, ExportHelper):
    bl_idname = "export_scene.space3"
    bl_label = "Export Space3"
    bl_options = {'UNDO', 'PRESET'}

    filename_ext = ".space3"
    filter_glob: StringProperty(default="*.space3", options={'HIDDEN'})

    debug_process: BoolProperty(
        name="Debug",
        description="Log export process to console",
        default=False,
    )

    use_selection: BoolProperty(
        name="Selection Only",
        description="Export selected objects only",
        default=True,
    )

    batch_mode: EnumProperty(
        name="Batch Mode",
        items=(('OFF', "Off", "Active scene to file"),
               ('SCENE', "Scene", "Each scene as a file"),
               ('COLLECTION', "Collection",
                "Each collection (data-block ones) as a file, does not include content of children collections"),
               ('SCENE_COLLECTION', "Scene Collections",
                "Each collection (including master, non-data-block ones) of each scene as a file, "
                "including content from children collections"),
               ('ACTIVE_SCENE_COLLECTION', "Active Scene Collections",
                "Each collection (including master, non-data-block one) of the active scene as a file, "
                "including content from children collections"),
               ),
    )

    @classmethod
    def poll(cls, context):
        return context.active_object is not None

    def draw(self, context):
        pass

    @property
    def check_extension(self):
        return self.batch_mode == 'OFF'

    def execute(self, context):
        def report(message: str, level='INFO'):
            if level != 'INFO' or self.debug_process:
                self.report({level}, message)

        orientation = axis_conversion(
            to_forward=self.axis_forward,
            to_up=self.axis_up,
        ).to_4x4()

        main(context, orientation, report, self.filepath, self.use_selection)
        return {'FINISHED'}


def menu_func_export(self, context):
    self.layout.operator(Space3Export.bl_idname, text="Space3 (.space3)")


class Space3ExportPanel(bpy.types.Panel):
    bl_space_type = 'FILE_BROWSER'
    bl_region_type = 'TOOL_PROPS'
    bl_label = ""
    bl_parent_id = "FILE_PT_operator"
    bl_options = {'HIDE_HEADER'}

    @classmethod
    def poll(cls, context):
        sfile = context.space_data
        operator = sfile.active_operator

        return operator.bl_idname == "EXPORT_SCENE_OT_space3"

    def draw(self, context):
        layout = self.layout
        layout.use_property_split = True
        layout.use_property_decorate = False

        sfile = context.space_data
        operator = sfile.active_operator

        row = layout.row(align=True)
        row.prop(operator, "batch_mode")

        layout.prop(operator, 'use_selection')
        layout.prop(operator, 'debug_process')
        layout.prop(operator, "axis_forward")
        layout.prop(operator, "axis_up")


def register():
    bpy.utils.register_class(Space3Export)
    bpy.utils.register_class(Space3ExportPanel)
    bpy.types.TOPBAR_MT_file_export.append(menu_func_export)


def unregister():
    bpy.utils.unregister_class(Space3Export)
    bpy.utils.unregister_class(Space3ExportPanel)
    bpy.types.TOPBAR_MT_file_export.remove(menu_func_export)


if __name__ == "__main__":
    register()
