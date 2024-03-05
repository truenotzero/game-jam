use crate::{archetype, entity::{EntityId, EntityManager, Position, Scale}, math::{Mat4, Vec2, Vec4}, world};


pub struct Room {
    parts: Vec<EntityId>,
}

impl Room {
    fn new(man: &mut EntityManager, position: Position, dimensions: Scale) -> Self {
        let mut parts = Vec::new();
        // make the background
        let bg = archetype::background::new(man, position, dimensions);
        parts.push(bg);

        // wall it off
        let room_to_world = Mat4::identity();
        let width = dimensions.x as usize;
        let height = dimensions.y as usize;
        for y in 0..height {
            for x in 0..width {
                let room_pos = Vec4::new(x as f32, y as f32, 0.0, 0.0);
                let world_pos4 = room_to_world * room_pos;
                let p = Vec2::new(world_pos4.x, world_pos4.y);
            }
        }

        // make connectors
        todo!()
    }

    pub fn main() -> Self {
        todo!()
    }

    pub fn destroy(self, man: &mut EntityManager) {
        for part in self.parts {
            man.kill(part);
        }
    }
}
