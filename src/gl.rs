use std::ops::Deref;

use glfw::Context;

pub mod raw {
    include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
    pub use self::types::*;
}

pub struct DrawContext(());

impl DrawContext {
    pub fn create(window: &mut glfw::Window) -> Self {
        window.make_current();
        Self(())
    }
}

type GlObjectId = raw::GLuint;

struct GlObject<'a>(GlObjectId, &'a DrawContext);

impl<'a> Deref for GlObject<'a> {
    type Target = GlObjectId;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
