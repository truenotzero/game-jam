use std::{mem::{size_of, size_of_val}, ptr::null};

use crate::{
    common::{as_bytes, AsBytes, Error, Result},
    gl::{self, buffer_flags, ArrayBuffer, DrawContext, IndexBuffer, Shader, Vao},
    math::{Mat3, Mat4, Vec3},
};

// per vertex
#[derive(Default)]
pub struct Vertex {
    pub pos: Vec3, // vertex position
}

#[repr(C)]
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
    index_data: IndexBuffer<'a>,
    _vertex_data: ArrayBuffer<'a>,
    instance_data: ArrayBuffer<'a>,

    screen_matrix: Mat4,

    active_instances: usize,
    max_instances: usize,
}

impl<'a> InstancedShapeManager<'a> {
    fn new(
        ctx: &'a DrawContext,
        vertex_data: ArrayBuffer<'a>,
        index_data: IndexBuffer<'a>,
        max_instances: usize,
        screen_matrix: Mat4,
    ) -> Self {
        let vao = Vao::new(ctx);


        let instance_data = ArrayBuffer::new(ctx);
        instance_data.reserve(size_of::<Instance>() * max_instances, gl::buffer_flags::DYNAMIC_STORAGE);

        // set up vertex_data + indices
        vao.bind_attrib_src(&vertex_data, &instance_data);
        Self {
            vao,
            index_data,
            _vertex_data: vertex_data,
            instance_data,

            screen_matrix,

            active_instances: 0,
            max_instances,
        }
    }

    pub fn quads(ctx: &'a DrawContext, max_instances: usize, screen_matrix: Mat4) -> Self {
        let vertex_data = ArrayBuffer::new(ctx);

        unsafe {
            vertex_data.set(
                [
                    Vertex {
                        pos: Vec3::new(-0.5, 0.5, 0.0),
                    },
                    Vertex {
                        pos: Vec3::new(-0.5, -0.5, 0.0),
                    },
                    Vertex {
                        pos: Vec3::new(0.5, -0.5, 0.0),
                    },
                    Vertex {
                        pos: Vec3::new(0.5, 0.5, 0.0),
                    },
                ]
                .as_bytes(),
                gl::buffer_flags::DEFAULT,
            );
        }

        let index_data = IndexBuffer::new(ctx);
        index_data.set([0, 1, 2, 2, 3, 0].as_ref(), gl::buffer_flags::DEFAULT);

        Self::new(ctx, vertex_data, index_data, max_instances, screen_matrix)
    }

    pub fn new_instance(&mut self, data: Option<Instance>) -> Result<usize> {
        let instance = data.unwrap_or_default();
        if self.active_instances == self.max_instances {
            return Err(Error::InstanceLimit);
        }

        let id = self.active_instances;
        self.active_instances += 1;

        self.set_instance(id, instance);

        Ok(id)
    }

    pub fn set_instance(&self, id: usize, data: Instance) {
        let data_bytes = unsafe { data.as_bytes() };
        self.instance_data.update(id * size_of_val(&data), data_bytes);
    }

    pub fn draw(&self, shader: &Shader) {
        shader.bind();
        self.index_data.bind();
        gl::call!(DrawElementsInstanced(
            TRIANGLES,
            6,
            UNSIGNED_BYTE,
            null(),
            1
        ));
    }
}
