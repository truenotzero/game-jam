#[derive(Debug)]
pub enum Error {
    FileNotFound,
    BadShaderType,
    ParseError,
    ShaderCompilationError(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub trait AsBytes {
    unsafe fn as_bytes(&self) -> &[u8];
}

macro_rules! as_bytes{
    ($ty:ty) => {
        impl crate::common::AsBytes for $ty {
            unsafe fn as_bytes(&self) -> &[u8] {
                core::slice::from_raw_parts(
                    (self as *const $ty).cast(),
                    std::mem::size_of::<$ty>()
                ) 
            }
        }

        impl crate::common::AsBytes for [$ty] {
            unsafe fn as_bytes(&self) -> &[u8] {
                core::slice::from_raw_parts(
                    self.as_ptr().cast(),
                    std::mem::size_of::<$ty>() * self.len()
                ) 
            }
        }
    };
}

pub(crate) use as_bytes;
