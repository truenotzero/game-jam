use std::{
    mem::{size_of, size_of_val},
    ptr::null,
};

use crate::{
    common::{as_bytes, AsBytes, Error, Result},
    gl::{self, buffer_flags, call, ArrayBuffer, DrawContext, IndexBuffer, Shader, Uniform, Vao},
    math::{Mat4, Vec3, Vec4},
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
    ) -> Self {
        let vao = Vao::new(ctx);

        let instance_data = ArrayBuffer::new(ctx);
        instance_data.reserve(
            size_of::<Instance>() * max_instances,
            gl::buffer_flags::DYNAMIC_STORAGE,
        );

        // set up vertex_data + indices
        vao.bind_instance_attribs(&vertex_data, &instance_data);
        Self {
            vao,
            index_data,
            _vertex_data: vertex_data,
            instance_data,

            num_indices,
            active_instances: 0,
            max_instances,
        }
    }

    pub fn quads(ctx: &'a DrawContext, max_instances: usize) -> Self {
        let vertex_data = ArrayBuffer::new(ctx);
        let index_data = IndexBuffer::new(ctx);

        let (vertices, indices) =
            quad_vertex_helper(0.9, [Vec4::new(0.0, 0.0, 1.0, 1.0)].into_iter());

        vertex_data.set(unsafe { vertices.as_bytes() }, gl::buffer_flags::DEFAULT);
        index_data.set(&indices, gl::buffer_flags::DEFAULT);

        Self::new(ctx, vertex_data, index_data, max_instances, indices.len())
    }

    /// enable instances for use without initializing
    pub unsafe fn enable_instances(&mut self, num_instances: usize) -> Result<()> {
        if num_instances > self.max_instances {
            return Err(Error::InstanceLimit);
        }

        self.active_instances = num_instances;
        Ok(())
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
        self.instance_data
            .update(id * size_of_val(&data), data_bytes);
    }

    pub fn draw(&self, shader: &Shader) {
        shader.enable();

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

// each quad is in the format (x, y, w, h)
// generates vertices + indices
pub fn quad_vertex_helper(
    depth_: f32,
    quads: impl Iterator<Item = Vec4>,
) -> (Vec<Vertex>, Vec<u8>) {
    quad_vertex_helper_impl(
        depth_,
        quads,
        |vertices: &mut Vec<Vertex>, depth, quad: Vec4| {
            let top_left = Vertex {
                pos: Vec3::new(quad.x, quad.y, depth),
            };
            vertices.push(top_left);

            let top_right = Vertex {
                pos: Vec3::new(quad.x + quad.z, quad.y, depth),
            };
            vertices.push(top_right);

            let bottom_right = Vertex {
                pos: Vec3::new(quad.x + quad.z, quad.y + quad.w, depth),
            };
            vertices.push(bottom_right);

            let bottom_left = Vertex {
                pos: Vec3::new(quad.x, quad.y + quad.w, depth),
            };
            vertices.push(bottom_left);
        },
    )
}

pub fn quad_vertex_helper_local_center(
    depth_: f32,
    quads: impl Iterator<Item = Vec4>,
) -> (Vec<Vertex>, Vec<u8>) {
    quad_vertex_helper_impl(
        depth_,
        quads,
        |vertices: &mut Vec<Vertex>, depth, quad: Vec4| {
            let w = 0.5 * quad.z;
            let h = 0.5 * quad.w;

            let top_left = Vertex {
                pos: Vec3::new(quad.x - w, quad.y - h, depth),
            };
            vertices.push(top_left);

            let top_right = Vertex {
                pos: Vec3::new(quad.x + w, quad.y - h, depth),
            };
            vertices.push(top_right);

            let bottom_right = Vertex {
                pos: Vec3::new(quad.x + w, quad.y + h, depth),
            };
            vertices.push(bottom_right);

            let bottom_left = Vertex {
                pos: Vec3::new(quad.x - w, quad.y + h, depth),
            };
            vertices.push(bottom_left);
        },
    )
}

fn quad_vertex_helper_impl(
    depth: f32,
    quads: impl Iterator<Item = Vec4>,
    make_quad: fn(&mut Vec<Vertex>, f32, Vec4),
) -> (Vec<Vertex>, Vec<u8>) {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    let index_key = [0, 1, 2, 2, 3, 0];
    let unique_vertices_per_quad = 4;
    let mut accum = 0;
    for quad in quads {
        make_quad(&mut vertices, depth, quad);

        for idx in index_key {
            indices.push(accum + idx);
        }
        accum += unique_vertices_per_quad;
    }

    (vertices, indices)
}
