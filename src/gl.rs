use std::{
    ffi::CString,
    fs::read_to_string,
    mem::{offset_of, size_of},
    path::Path,
    ptr::{null, null_mut},
};

use glfw::Context;

use crate::{
    common::{Error, Result},
    math::{Mat4, Vec3, Vec4},
    render::instanced::{Tile, Vertex},
    resources,
};

pub mod raw {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    pub use self::types::*;
}

macro_rules! call {
    ($gl_call:expr) => {{
        use crate::gl::raw::*;
        let ret = unsafe { $gl_call };
        crate::gl::check_error(file!(), line!(), stringify!($gl_call));
        ret
    }};
}

pub(crate) use call;

pub fn check_error(file_name: &str, line: u32, expr: &str) {
    let e = unsafe { raw::GetError() };
    let error_string = match e {
        raw::NO_ERROR => return,
        raw::INVALID_ENUM => "GL_INVALID_ENUM",
        raw::INVALID_VALUE => "GL_INVALID_VALUE",
        raw::INVALID_OPERATION => "GL_INVALID_OPERATION",
        raw::INVALID_FRAMEBUFFER_OPERATION => "GL_INVALID_FRAMEBUFFER_OPERATION",
        raw::STACK_OVERFLOW => "GL_STACK_OVERFLOW",
        raw::STACK_UNDERFLOW => "GL_STACK_UNDERFLOW",
        raw::OUT_OF_MEMORY => "GL_OUT_OF_MEMORY",
        e => panic!("glGetError returned unknown error enum [{}]", e),
    };

    panic!("[{}:{}] gl{} -> {}", file_name, line, expr, error_string);
}

pub struct DrawContext(());

impl DrawContext {
    pub fn create(window: &mut glfw::Window) -> Self {
        window.make_current();
        raw::load_with(|procname| window.get_proc_address(procname));
        Self(())
    }
}

type GlObjectId = raw::GLuint;

pub struct GlObject<'a> {
    id: GlObjectId,
    _ctx: &'a DrawContext,
}

pub struct Vao<'a>(GlObject<'a>);

impl<'a> Vao<'a> {
    pub fn new(ctx: &'a DrawContext) -> Self {
        let mut id = 0;
        call!(CreateVertexArrays(1, &mut id));
        Self(GlObject { id, _ctx: ctx })
    }

    pub fn apply(&self) {
        call!(BindVertexArray(self.0.id));
    }

    pub fn bind_vertex_attribs(&self, vertex_data: &ArrayBuffer) {
        self.apply();

        // per-vertex attributes
        vertex_data.apply();
        // position
        call!(EnableVertexAttribArray(0));
        call!(VertexAttribPointer(
            0,
            3,
            raw::FLOAT,
            raw::FALSE,
            size_of::<Vertex>() as _,
            offset_of!(Vertex, pos) as _,
        ));
    }

    pub fn bind_instance_attribs(&self, vertex_data: &ArrayBuffer, instance_data: &ArrayBuffer) {
        self.bind_vertex_attribs(vertex_data);

        // per-instance attributes
        instance_data.apply();
        // shape transform, mat3 = 3 vecs = takes up 3 indices (1,2,3)
        call!(EnableVertexAttribArray(1));
        let mat_offset = offset_of!(Tile, transform);
        call!(VertexAttribPointer(
            1,
            4,
            raw::FLOAT,
            raw::FALSE,
            size_of::<Tile>() as _,
            mat_offset as _,
        ));
        call!(VertexAttribDivisor(1, 1));
        call!(EnableVertexAttribArray(2));
        let mat_offset = mat_offset + size_of::<Vec4>();
        call!(VertexAttribPointer(
            2,
            4,
            raw::FLOAT,
            raw::FALSE,
            size_of::<Tile>() as _,
            mat_offset as _,
        ));
        call!(VertexAttribDivisor(2, 1));
        call!(EnableVertexAttribArray(3));
        let mat_offset = mat_offset + size_of::<Vec4>();
        call!(VertexAttribPointer(
            3,
            4,
            raw::FLOAT,
            raw::FALSE,
            size_of::<Tile>() as _,
            mat_offset as _,
        ));
        call!(VertexAttribDivisor(3, 1));
        call!(EnableVertexAttribArray(4));
        let mat_offset = mat_offset + size_of::<Vec4>();
        call!(VertexAttribPointer(
            4,
            4,
            raw::FLOAT,
            raw::FALSE,
            size_of::<Tile>() as _,
            mat_offset as _,
        ));
        call!(VertexAttribDivisor(4, 1));

        // color
        call!(EnableVertexAttribArray(5));
        call!(VertexAttribPointer(
            5,
            3,
            raw::FLOAT,
            raw::FALSE,
            size_of::<Tile>() as _,
            offset_of!(Tile, col) as _,
        ));
        call!(VertexAttribDivisor(5, 1));
    }
}

impl<'a> Drop for Vao<'a> {
    fn drop(&mut self) {
        call!(DeleteVertexArrays(1, &self.0.id))
    }
}

pub mod buffer_flags {
    use super::raw::GLuint;

    pub type Type = GLuint;

    pub const DEFAULT: Type = 0;
    pub const _MAP_READ: Type = 0x0001;
    pub const _MAP_WRITE: Type = 0x0002;
    pub const DYNAMIC_STORAGE: Type = 0x0100;
    pub const _CLIENT_STORAGE: Type = 0x0200;
    pub const _MAP_PERSISTENT: Type = 0x0040;
    pub const _MAP_COHERENT: Type = 0x0080;
}

pub struct Buffer<'a, const T: raw::GLenum>(GlObject<'a>);

pub type ArrayBuffer<'a> = Buffer<'a, { raw::ARRAY_BUFFER }>;
pub type IndexBuffer<'a> = Buffer<'a, { raw::ELEMENT_ARRAY_BUFFER }>;
pub type UniformBuffer<'a> = Buffer<'a, { raw::UNIFORM_BUFFER }>;

impl<'a> UniformBuffer<'a> {
    pub fn bind_buffer_base(&self, bind_point: raw::GLuint) {
        call!(BindBufferBase(UNIFORM_BUFFER, bind_point, self.0.id));
    }
}

impl<'a, const T: raw::GLenum> Buffer<'a, T> {
    pub fn new(ctx: &'a DrawContext) -> Self {
        let mut id = 0;
        call!(CreateBuffers(1, &mut id));
        Self(GlObject { id, _ctx: ctx })
    }

    pub fn apply(&self) {
        call!(BindBuffer(T, self.0.id));
    }

    fn init(&self, data_size: usize, data_ptr: *const u8, flags: buffer_flags::Type) {
        call!(NamedBufferStorage(
            self.0.id,
            data_size as _,
            data_ptr.cast(),
            flags
        ));
    }

    pub fn reserve(&self, buffer_size: usize, flags: buffer_flags::Type) {
        self.init(buffer_size, null(), flags)
    }

    pub fn set(&self, bytes: &[u8], flags: buffer_flags::Type) {
        self.init(bytes.len(), bytes.as_ptr(), flags)
    }

    /// requires having passed the DYNAMIC_STORAGE flag in alloc
    pub fn update(&self, offset: usize, bytes: &[u8]) {
        call!(NamedBufferSubData(
            self.0.id,
            offset as _,
            bytes.len() as _,
            bytes.as_ptr().cast()
        ));
    }
}

impl<'a, const T: raw::GLenum> Drop for Buffer<'a, T> {
    fn drop(&mut self) {
        call!(DeleteBuffers(1, &self.0.id))
    }
}

pub struct Shader<'a>(GlObject<'a>);

impl<'a> Shader<'a> {
    fn new(ctx: &'a DrawContext) -> Self {
        let id = call!(CreateProgram());
        Self(GlObject { id, _ctx: ctx })
    }

    pub fn _from_file(ctx: &'a DrawContext, path: &Path) -> Result<Self> {
        let this = Self::new(ctx);

        let mut path = path.with_extension("vert");
        this._load_from_file(&path)?;

        path.set_extension("frag");
        this._load_from_file(&path)?;

        path.set_extension("geom");
        if path.exists() {
            this._load_from_file(&path)?;
        }

        this.compile()
    }

    pub fn from_resource(ctx: &'a DrawContext, resource: resources::Shader) -> Result<Self> {
        const TYPES: [raw::GLenum; 3] = [
            raw::VERTEX_SHADER,
            raw::FRAGMENT_SHADER,
            raw::GEOMETRY_SHADER,
        ];

        let this = Self::new(ctx);
        for (idx, source) in resource.into_iter().enumerate() {
            let type_ = TYPES[idx];
            this.load(source, type_)?;
        }

        this.compile()
    }

    pub fn apply(&self) {
        call!(UseProgram(self.0.id));
    }

    pub fn _locate_uniform(&self, name: &str) -> Option<raw::GLint> {
        let name = CString::new(name).expect("Bad uniform name");
        let location = call!(GetUniformLocation(self.0.id, name.as_ptr().cast()));
        if location != -1 {
            Some(location)
        } else {
            None
        }
    }

    fn _load_from_file(&self, filepath: &Path) -> Result<()> {
        let shader_src = read_to_string(filepath).map_err(|_| Error::FileNotFound)?;

        let extension = filepath.extension().ok_or(Error::BadShaderType)?;
        let extension = extension.to_str().ok_or(Error::ParseError)?;
        let shader_type = match extension {
            "vert" => raw::VERTEX_SHADER,
            "frag" => raw::FRAGMENT_SHADER,
            "geom" => raw::GEOMETRY_SHADER,
            _ => return Err(Error::BadShaderType),
        };

        self.load(shader_src.as_bytes(), shader_type)
    }

    fn load(&self, shader_src: &[u8], shader_type: raw::GLenum) -> Result<()> {
        let shader = call!(CreateShader(shader_type));
        let source = shader_src.as_ptr().cast();
        let len = shader_src.len() as _;
        call!(ShaderSource(shader, 1, &source, &len));
        call!(CompileShader(shader));

        let mut ok = 0;
        call!(GetShaderiv(shader, COMPILE_STATUS, &mut ok));
        if ok != raw::TRUE as _ {
            let mut log_len = 0;
            call!(GetShaderiv(shader, INFO_LOG_LENGTH, &mut log_len));
            log_len -= 1; // no need for null terminator
            let mut log = vec![0u8; log_len as _];
            call!(GetShaderInfoLog(
                shader,
                log_len,
                null_mut(),
                log.as_mut_ptr().cast()
            ));

            let log = String::from_utf8(log).map_err(|_| Error::ParseError)?;
            return Err(Error::ShaderCompilationError(log));
        }

        call!(AttachShader(self.0.id, shader));

        call!(DeleteShader(shader));
        Ok(())
    }

    fn compile(self) -> Result<Self> {
        call!(LinkProgram(self.0.id));

        let mut ok = 0;
        call!(GetProgramiv(self.0.id, LINK_STATUS, &mut ok));
        if ok != raw::TRUE as _ {
            let mut log_len = 0;
            call!(GetProgramiv(self.0.id, INFO_LOG_LENGTH, &mut log_len));
            log_len -= 1; // no need for null terminator
            let mut log = vec![0u8; log_len as _];
            call!(GetProgramInfoLog(
                self.0.id,
                log_len,
                null_mut(),
                log.as_mut_ptr().cast()
            ));

            let log = String::from_utf8(log).map_err(|_| Error::ParseError)?;
            return Err(Error::ShaderCompilationError(log));
        }
        Ok(self)
    }
}

impl<'a> Drop for Shader<'a> {
    fn drop(&mut self) {
        call!(DeleteProgram(self.0.id));
    }
}

pub trait Uniform {
    fn uniform(&self, layout_location: raw::GLint);
}

impl Uniform for Mat4 {
    fn uniform(&self, layout_location: raw::GLint) {
        let ptr = &self[0][0];
        call!(UniformMatrix4fv(layout_location, 1, FALSE, ptr))
    }
}

impl Uniform for Vec3 {
    fn uniform(&self, layout_location: raw::GLint) {
        call!(Uniform3f(layout_location, self.x, self.y, self.z))
    }
}

impl Uniform for f32 {
    fn uniform(&self, layout_location: raw::GLint) {
        call!(Uniform1f(layout_location, *self))
    }
}

impl Uniform for u128 {
    fn uniform(&self, layout_location: raw::GLint) {
        call!(Uniform1i(layout_location, *self as _));
    }
}

pub struct Texture<'a, const T: raw::GLenum>(GlObject<'a>);

pub type Texture2D<'a> = Texture<'a, { raw::TEXTURE_2D }>;

impl <'a, const T: raw::GLenum> Texture<'a, T> {
    pub fn new(ctx: &'a DrawContext) -> Self {
        let mut id = 0;
        call!(CreateTextures(T, 1, &mut id));
        Self(GlObject { id, _ctx: ctx })
    }

    pub fn bind(&self, slot: usize) {
        call!(BindTextureUnit(slot as _, self.0.id));
    }

    pub fn apply(&self) {
        call!(BindTexture(T, self.0.id))
    }

    pub fn type_(&self) -> raw::GLenum {
        T
    }
}

impl<'a, const T: raw::GLenum> Drop for Texture<'a, T> {
    fn drop(&mut self) {
        call!(DeleteTextures(1, &self.0.id));
    }
}

pub struct RenderBuffer<'a>(GlObject<'a>);

impl<'a> RenderBuffer<'a> {
    pub fn new(ctx: &'a DrawContext, width: raw::GLint, height: raw::GLint) -> Self {
        let mut id = 0;
        call!(CreateRenderbuffers(1, &mut id));
        let this = Self(GlObject { id, _ctx: ctx });
        this.apply();
        call!(RenderbufferStorage(RENDERBUFFER, DEPTH_COMPONENT, width, height));
        this
    }

    pub fn apply(&self) {
        call!(BindRenderbuffer(RENDERBUFFER, self.0.id));
    }
}

impl<'a> Drop for RenderBuffer<'a> {
    fn drop(&mut self) {
        call!(DeleteRenderbuffers(1, &self.0.id))
    }
}

pub struct FrameBuffer<'a> {
    id: GlObject<'a>,
    depth_buffer: RenderBuffer<'a>,
    color_buffer: Texture2D<'a>,
}

impl<'a> FrameBuffer<'a> {
    pub fn new(ctx: &'a DrawContext, width: raw::GLint, height: raw::GLint) -> Self {
        let mut id = 0;
        let depth_buffer = RenderBuffer::new(ctx, width, height);
        let color_buffer = Texture2D::new(ctx);
        call!(CreateFramebuffers(1, &mut id));
        color_buffer.apply();
        call!(TexImage2D(color_buffer.type_(), 0, raw::RGBA as _, width, height, 0, raw::RGBA, raw::FLOAT, null()));
        call!(TexParameteri(color_buffer.type_(), TEXTURE_MIN_FILTER, LINEAR as _));
        call!(TexParameteri(color_buffer.type_(), TEXTURE_MAG_FILTER, LINEAR as _));
        call!(TexParameteri(color_buffer.type_(), TEXTURE_WRAP_S, CLAMP_TO_EDGE as _));
        call!(TexParameteri(color_buffer.type_(), TEXTURE_WRAP_T, CLAMP_TO_EDGE as _));

        let this = Self {
            id: GlObject { id, _ctx: ctx },
            depth_buffer,
            color_buffer,
        };
        
        this.with(|this| {
            call!(FramebufferTexture2D(FRAMEBUFFER, COLOR_ATTACHMENT0, this.color_buffer.type_(), this.color_buffer.0.id, 0));
            call!(FramebufferRenderbuffer(FRAMEBUFFER, DEPTH_ATTACHMENT, RENDERBUFFER, this.depth_buffer.0.id));
            let status = call!(CheckFramebufferStatus(FRAMEBUFFER));
            assert!(status == raw::FRAMEBUFFER_COMPLETE);
        });

        this
    }

    pub fn new_screen(ctx: &'a DrawContext) -> Self {
        let mut viewport: [raw::GLint; 4] = Default::default();
        call!(GetIntegerv(VIEWPORT, viewport.as_mut_ptr()));
        Self::new(ctx, viewport[2], viewport[3])
    }

    fn apply(&self) {
        call!(BindFramebuffer(FRAMEBUFFER, self.id.id));
    }

    fn apply_default() {
        call!(BindFramebuffer(FRAMEBUFFER, 0));
    }

    pub fn with<T>(&self, f: impl FnOnce(&Self) -> T) -> T {
        self.apply();
        let ret = f(self);
        Self::apply_default();
        ret
    }

    pub fn bind_texture(&self, slot: usize) {
        self.color_buffer.bind(slot)
    }

    /// clears the currently bound framebuffer
    pub fn clear() {
        call!(Clear(DEPTH_BUFFER_BIT | COLOR_BUFFER_BIT));
    }
}

impl<'a> Drop for FrameBuffer<'a> {
    fn drop(&mut self) {
        call!(DeleteFramebuffers(1, &self.id.id));
    }
}

