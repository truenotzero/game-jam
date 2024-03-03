use std::{mem::{size_of, size_of_val}, ptr::null};

use crate::{
    common::{as_bytes, AsBytes, Error, Result},
    gl::{self, buffer_flags, call, ArrayBuffer, DrawContext, IndexBuffer, Shader, Uniform, Vao},
    math::{ Mat4, Vec3},
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
    pub transform: Mat4, // shape transform
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
    num_indices: usize,
    active_instances: usize,
    max_instances: usize,
}

impl<'a> InstancedShapeManager<'a> {
    fn new(
        ctx: &'a DrawContext,
        vertex_data: ArrayBuffer<'a>,
        index_data: IndexBuffer<'a>,
        max_instances: usize,
        num_indices: usize,
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
            num_indices,
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
                        pos: Vec3::new(0.0, 0.0, 0.0),
                    },
                    Vertex {
                        pos: Vec3::new(1.0, 0.0, 0.0),
                    },
                    Vertex {
                        pos: Vec3::new(1.0, 1.0, 0.0),
                    },
                    Vertex {
                        pos: Vec3::new(0.0, 1.0, 0.0),
                    },
                ]
                .as_bytes(),
                gl::buffer_flags::DEFAULT,
            );
        }

        let indices = [0,1,2,2,3,0];
        let index_data = IndexBuffer::new(ctx);
        index_data.set(&indices, gl::buffer_flags::DEFAULT);

        Self::new(ctx, vertex_data, index_data, max_instances, indices.len(), screen_matrix)
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
        shader.enable();
        let screen_location = shader.locate_uniform("uScreen").expect("uScreen not found in shader");
        self.screen_matrix.uniform(screen_location);

        self.vao.enable();
        self.index_data.bind();
        gl::call!(DrawElementsInstanced(
            TRIANGLES,
            self.num_indices as _,
            UNSIGNED_BYTE,
            null(),
            self.active_instances as _,
        ));
    }
}
