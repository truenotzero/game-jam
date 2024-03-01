use std::ptr::null;

use crate::{
    common::{as_bytes, AsBytes}, gl::{self, ArrayBuffer, DrawContext, IndexBuffer, Shader, Vao}, math::{Mat3, Vec3}
};

// per vertex
#[derive(Default)]
pub struct Vertex {
    pub pos: Vec3, // vertex position
}

// per instance
#[derive(Default)]
pub struct Instance {
    pub transform: Mat3, // shape transform
    pub col: Vec3,       // shape color
    pub frame: u8,       // shape animation frame for SMOOTH ANIMATIONSSSSSSSSSSSSS
}

as_bytes!(Vertex);
as_bytes!(Instance);

pub struct InstancedShapeManager<'a> {
    vao: Vao<'a>,
    ibo: IndexBuffer<'a>,
    _vertex_data: ArrayBuffer<'a>,
    instance_data: ArrayBuffer<'a>,

    instances: Vec<Instance>,
}

impl<'a> InstancedShapeManager<'a> {
    pub fn quads(ctx: &'a DrawContext) -> Self {
        let vao = Vao::new(ctx);
        let vertex_data = ArrayBuffer::new(ctx);
        unsafe {
        vertex_data.alloc([
            Vertex { pos: Vec3::new(-1.0,  1.0, 0.0) },
            Vertex { pos: Vec3::new(-1.0, -1.0, 0.0) },
            Vertex { pos: Vec3::new( 1.0, -1.0, 0.0) },
            Vertex { pos: Vec3::new( 1.0,  1.0, 0.0) },
        ].as_bytes(), gl::BufferUsage::Static);
        }
        let ibo = IndexBuffer::new(ctx);
        ibo.alloc([
            0, 1, 2,
            2, 3, 0,
        ].as_ref(), gl::BufferUsage::Static);

        let mut instances = Vec::new();
        instances.push(
                Instance {
                    transform: Default::default(),
                    col: Vec3 { x: 0.0, y: 1.0, z: 0.0 },
                    frame: 0,
                },
        );
        let instance_data = ArrayBuffer::new(ctx);
        unsafe {
            instance_data.alloc(instances[0].as_bytes(), gl::BufferUsage::Static);
        }

        // set up vertex_data + indices
        vao.bind_attrib_src(&vertex_data, &instance_data);
        Self {
            vao,
            ibo,
            _vertex_data: vertex_data,
            instance_data,

            instances,
        }
    }

    pub fn new_instance(instance: Instance) {}

    pub fn draw(&self, shader: &Shader) {
        shader.bind();
        self.ibo.bind();
        gl::call!(DrawElementsInstanced(
            TRIANGLES,
            6,
            UNSIGNED_BYTE,
            null(),
            1
        ));
    }
}
