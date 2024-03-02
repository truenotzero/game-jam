use std::{
    fs::read_to_string,
    mem::{offset_of, size_of, size_of_val},
    ops::BitOr,
    path::Path,
    ptr::{null, null_mut},
};

use glfw::Context;

use crate::{
    common::{AsBytes, Error, Result}, math::{ Vec3, Vec4}, render::{Instance, Vertex}
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

pub struct GlObject<'a>(GlObjectId, &'a DrawContext);

pub struct Vao<'a>(GlObject<'a>);

impl<'a> Vao<'a> {
    pub fn new(ctx: &'a DrawContext) -> Self {
        let mut id = 0;
        call!(CreateVertexArrays(1, &mut id));
        Self(GlObject(id, ctx))
    }

    pub fn bind_attrib_src(&self, vertex_data: &ArrayBuffer, instance_data: &ArrayBuffer) {
        call!(BindVertexArray(self.0 .0));

        // per-vertex attributes
        vertex_data.bind();
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

        // per-instance attributes
        instance_data.bind();
        // shape transform, mat3 = 3 vecs = takes up 3 indices (1,2,3)
        call!(EnableVertexAttribArray(1));
        let mat_offset = offset_of!(Instance, transform);
        call!(VertexAttribPointer(
            1,
            4,
            raw::FLOAT,
            raw::FALSE,
            size_of::<Instance>() as _,
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
            size_of::<Instance>() as _,
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
            size_of::<Instance>() as _,
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
            size_of::<Instance>() as _,
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
            size_of::<Instance>() as _,
            offset_of!(Instance, col) as _,
        ));
        call!(VertexAttribDivisor(5, 1));

        // animation frame
        call!(EnableVertexAttribArray(6));
        call!(VertexAttribPointer(
            6,
            1,
            raw::UNSIGNED_BYTE,
            raw::FALSE,
            size_of::<Instance>() as _,
            offset_of!(Instance, frame) as _,
        ));
        call!(VertexAttribDivisor(6, 1));
    }
}

impl<'a> Drop for Vao<'a> {
    fn drop(&mut self) {
        call!(DeleteVertexArrays(1, &self.0 .0))
    }
}

pub mod buffer_flags {
    use super::raw::GLuint;

    pub type Type = GLuint;

    pub const DEFAULT: Type = 0;
    pub const MAP_READ: Type = 0x0001;
    pub const MAP_WRITE: Type = 0x0002;
    pub const DYNAMIC_STORAGE: Type = 0x0100;
    pub const CLIENT_STORAGE: Type = 0x0200;
    pub const MAP_PERSISTENT: Type = 0x0040;
    pub const MAP_COHERENT: Type = 0x0080;
}

pub struct Buffer<'a, const T: raw::GLenum>(GlObject<'a>);

pub type ArrayBuffer<'a> = Buffer<'a, { raw::ARRAY_BUFFER }>;
pub type IndexBuffer<'a> = Buffer<'a, { raw::ELEMENT_ARRAY_BUFFER }>;

impl<'a, const T: raw::GLenum> Buffer<'a, T> {
    pub fn new(ctx: &'a DrawContext) -> Self {
        let mut id = 0;
        call!(CreateBuffers(1, &mut id));
        Self(GlObject(id, ctx))
    }

    pub fn bind(&self) {
        call!(BindBuffer(T, self.0 .0));
    }

    fn init(&self, data_size: usize, data_ptr: *const u8, flags: buffer_flags::Type) {
        call!(NamedBufferStorage(self.0.0, data_size as _, data_ptr.cast(), flags));
    }

    pub fn reserve(&self, buffer_size: usize, flags: buffer_flags::Type) {
        self.init(buffer_size, null(), flags)
    }

    pub fn set(&self, bytes: &[u8], flags: buffer_flags::Type) {
        self.init(bytes.len(), bytes.as_ptr(), flags)
    }

    /// requires having passed the DYNAMIC_STORAGE flag in alloc
    pub fn update(&self, offset: usize, bytes: &[u8]) {
        call!(NamedBufferSubData(self.0.0, offset as _, bytes.len() as _, bytes.as_ptr().cast()));
    }
}

impl<'a, const T: raw::GLenum> Drop for Buffer<'a, T> {
    fn drop(&mut self) {
        call!(DeleteBuffers(1, &self.0 .0))
    }
}

pub struct Shader<'a>(GlObject<'a>);

impl<'a> Shader<'a> {
    fn new(ctx: &'a DrawContext) -> Self {
        let id = call!(CreateProgram());
        Self(GlObject(id, ctx))
    }

    pub fn from_file(ctx: &'a DrawContext, path: &Path) -> Result<Self> {
        let this = Self::new(ctx);

        let mut path = path.with_extension("vert");
        this.load(&path)?;

        path.set_extension("frag");
        this.load(&path)?;

        this.compile()
    }

    pub fn bind(&self) {
        call!(UseProgram(self.0 .0));
    }

    fn load(&self, filepath: &Path) -> Result<()> {
        let shader_src = read_to_string(filepath).map_err(|_| Error::FileNotFound)?;

        let extension = filepath.extension().ok_or(Error::BadShaderType)?;
        let extension = extension.to_str().ok_or(Error::ParseError)?;
        let shader_type = match extension {
            "vert" => raw::VERTEX_SHADER,
            "frag" => raw::FRAGMENT_SHADER,
            _ => return Err(Error::BadShaderType),
        };

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

        call!(AttachShader(self.0 .0, shader));

        call!(DeleteShader(shader));
        Ok(())
    }

    fn compile(self) -> Result<Self> {
        call!(LinkProgram(self.0 .0));

        let mut ok = 0;
        call!(GetProgramiv(self.0 .0, LINK_STATUS, &mut ok));
        if ok != raw::TRUE as _ {
            let mut log_len = 0;
            call!(GetProgramiv(self.0 .0, INFO_LOG_LENGTH, &mut log_len));
            log_len -= 1; // no need for null terminator
            let mut log = vec![0u8; log_len as _];
            call!(GetProgramInfoLog(
                self.0 .0,
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
