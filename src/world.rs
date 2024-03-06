use crate::{
    archetype,
    entity::{EntityId, EntityManager, Position, Scale},
    gl::raw::BACK,
    math::{Mat4, Vec2, Vec3, Vec4},
    world,
};

const BACKGROUND_DEPTH: f32 = 0.9;
const WALL_DEPTH: f32 = 0.8;

pub struct Room {
    view_matrix: Mat4,
    parts: Vec<EntityId>,
}

impl Room {
    fn new(man: &mut EntityManager, position: Vec2, dimensions: Scale) -> Self {
        let translate = -0.5 * dimensions;
        let room_to_world = Mat4::translate(Vec3::from((translate, 0.0)));
        
        let mut parts = Vec::new();
        // make the background
        let new_pos = Position::new(position.x, position.y, BACKGROUND_DEPTH);
        let new_pos = room_to_world * Vec4::position(new_pos);
        let new_pos = Vec3::from(new_pos);
        let bg = archetype::background::new(man, new_pos, dimensions);
        parts.push(bg);

        // wall it off
        let width = dimensions.x as usize;
        let height = dimensions.y as usize;
        for y in 0..height {
            for x in 0..width {
                if !(y == 0 || y == height - 1 || x == 0 || x == width - 1) {
                    continue;
                }

                let room_pos = Vec4::new(x as f32, y as f32, 0.0, 1.0);
                let world_pos4 = room_to_world * room_pos;
                let p = Position::new(world_pos4.x, world_pos4.y, WALL_DEPTH);

                let wall = archetype::wall::new(man, p);
                parts.push(wall);
            }
        }

        // make connectors
        // TODO

        Self { 
            parts,
            view_matrix: Mat4::screen(position, dimensions.x, dimensions.y),
        }
    }

    pub fn main(man: &mut EntityManager) -> Self {
        Self::new(man, Vec2::default(), Vec2::new(40.0, 40.0))
    }

    pub fn destroy(self, man: &mut EntityManager) {
        for part in self.parts {
            man.kill(part);
        }
    }

    pub fn view(&self) -> Mat4 {
        self.view_matrix
    }
}
