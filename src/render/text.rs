use std::{collections::HashMap, hash::Hash, mem::size_of_val, mem::{offset_of, size_of}};

use crate::{
    common::{as_bytes, AsBytes, Error, Result},
    gl::{self, ArrayBuffer, DrawContext, Shader, Texture2D, Vao},
    math::{Mat4, Vec2}, resources::{self, Texture},
};

use super::VaoHelper;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct Vertex {
    pos: Vec2,
    uv: Vec2,
}

as_bytes!(Vertex);

#[repr(u8)]
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub enum TextNames {
    Snek,
    Controls,
    Fruit,

    _NumTexts,
}

impl TryFrom<u8> for TextNames {
    type Error = Error;

    fn try_from(value: u8) -> Result<TextNames> {
        use TextNames as T;
        Ok(match value {
            0 => T::Snek,
            1 => T::Controls,
            2 => T::Fruit,

            _ => Err(Error::InvalidTextNameId)?,
        })
    }
}

impl TextNames {
    fn resource(self) -> Texture {
        use crate::resources::textures::text::*;
        match self {
            Self::Snek => SNEK,
            Self::Controls => CONTROLS,
            Self::Fruit => FRUIT,

            TextNames::_NumTexts => panic!(),
        }
    }

    fn dimension(self) -> Vec2 {
        match self {
            Self::Snek => Vec2::new(62.0, 14.0),
            Self::Controls => Vec2::new(142.0, 38.0),
            Self::Fruit => Vec2::new(206.0, 14.0),

            Self::_NumTexts => panic!(),
        }
    }
}

const VERTICES_PER_SHAPE: usize = 6;

#[derive(Debug)]
pub struct Text {
    name: TextNames,
    vertices: [Vertex; VERTICES_PER_SHAPE],
}

impl Text {
    fn new(name: TextNames) -> Self {
        let corners = [
            Vertex {
                pos: Vec2::new(-0.5, 0.5),
                uv: Vec2::new(0.0, 0.0),
            },
            Vertex {
                pos: Vec2::new(-0.5, -0.5),
                uv: Vec2::new(0.0, 1.0),
            },
            Vertex {
                pos: Vec2::new(0.5, 0.5),
                uv: Vec2::new(1.0, 0.0),
            },
            Vertex {
                pos: Vec2::new(0.5, -0.5),
                uv: Vec2::new(1.0, 1.0),
            },
        ];

        Self {
            name,
            vertices: [
                corners[0], corners[1], corners[2], corners[3], corners[2], corners[1],
            ],
        }
    }

    pub fn place_at(name: TextNames, position: Vec2, scale: f32) -> Self {
        Self::new(name)
            .transform(Mat4::scale(scale * name.dimension()))
            .transform(Mat4::translate((position, 0.0).into()))
            
    }

    fn transform(mut self, t: Mat4) -> Self {
        for v in &mut self.vertices {
            v.pos = t * v.pos;
        }

        self
    }
}

pub struct TextManager<'a> {
    vao: Vao<'a>,
    vbo: ArrayBuffer<'a>,
    shader: Shader<'a>,
    
    textures: HashMap<TextNames, Texture2D<'a>>,

    texts: Vec<Text>,
}

impl<'a> TextManager<'a> {
    pub fn new(ctx: &'a DrawContext) -> Self {
        let vbo = ArrayBuffer::new(ctx);
        vbo.reserve(size_of::<Vertex>() * VERTICES_PER_SHAPE, gl::buffer_flags::DYNAMIC_STORAGE);

        let vao = VaoHelper::new(ctx)
            .bind_buffer(&vbo)
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Vertex>(),
                offset_of!(Vertex, pos),
            )
            .push_attrib(
                2,
                gl::raw::FLOAT,
                gl::raw::FALSE,
                size_of::<Vertex>(),
                offset_of!(Vertex, uv),
            )
            .build();

        Self {
            vao,
            vbo,
            shader: Shader::from_resource(ctx, resources::shaders::TEXT)
                .expect("bad text shader"),

            textures: Self::load_textures(ctx),

            texts: Default::default(),
        }
    }

    fn load_textures(ctx: &'a DrawContext) -> HashMap<TextNames, Texture2D<'a>> {
        let mut ret = HashMap::new();
        for text_name_id in 0..(TextNames::_NumTexts as u8) {
            // don't forget to add new text names to the conversion table in try_from
            let text_name = TextNames::try_from(text_name_id).unwrap();
            
            let image = image::load_from_memory(text_name.resource()).unwrap();
            let image = image.flipv();

            let texture = Texture2D::new(ctx);
            let width = text_name.dimension().x as _;
            let height = text_name.dimension().y as _;
            texture.apply();
            // effectively clamp to a transparent background
            gl::call!(TexParameteri(texture.type_(), TEXTURE_WRAP_S, CLAMP_TO_BORDER as _));
            gl::call!(TexParameteri(texture.type_(), TEXTURE_WRAP_T, CLAMP_TO_BORDER as _));
            gl::call!(TexParameteri(
                texture.type_(),
                TEXTURE_MIN_FILTER,
                LINEAR as _
            ));
            gl::call!(TexParameteri(
                texture.type_(),
                TEXTURE_MAG_FILTER,
                LINEAR as _
            ));
            gl::call!(TexImage2D(texture.type_(), 0, RGBA as _, width, height, 0, RGBA, UNSIGNED_BYTE, image.as_bytes().as_ptr().cast()));

            // push to hashmap
            ret.insert(text_name, texture);
        }

        ret
    }

    pub fn push(&mut self, text: Text) {
        self.texts.push(text);
    }

    pub fn draw(&mut self) {
        self.vao.apply();
        self.shader.apply();

        for text in &self.texts {
            for (i, v) in text.vertices.iter().enumerate() {
                let bytes = unsafe { v.as_bytes() };
                self.vbo.update(i * size_of::<Vertex>(), bytes);
            }

            self.textures[&text.name].bind(0);
            gl::call!(DrawArrays(TRIANGLES, 0, VERTICES_PER_SHAPE as _));
        }

        self.texts.clear();
    }
}
