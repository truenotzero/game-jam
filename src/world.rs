use crate::{archetype, entity::{EntityId, EntityManager, Position, Scale}, gl::raw::BACK, math::{Mat4, Vec2, Vec4}, world};

const BACKGROUND_DEPTH: f32 = 0.9;
const WALL_DEPTH: f32 = 0.8;

pub struct Room {
    parts: Vec<EntityId>,
}

impl Room {
    fn new(man: &mut EntityManager, position: Vec2, dimensions: Scale) -> Self {
        let mut parts = Vec::new();
        // make the background
        let position = Position::new(position.x, position.y, BACKGROUND_DEPTH);
        let bg = archetype::background::new(man, position, dimensions);
        parts.push(bg);

        // wall it off
        let room_to_world = Mat4::identity();
        let width = dimensions.x as usize;
        let height = dimensions.y as usize;
        for y in 0..height {
            for x in 0..width {
                if !(y == 0 || y == height - 1 || x == 0 || x == width - 1) { continue }

                let room_pos = Vec4::new(x as f32, y as f32, 0.0, 0.0);
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
        }
    }

    pub fn main(man: &mut EntityManager) -> Self {
        Self::new(man, Vec2::default(), Vec2::new(50.0, 50.0))
    }

    pub fn destroy(self, man: &mut EntityManager) {
        for part in self.parts {
            man.kill(part);
        }
    }
}
