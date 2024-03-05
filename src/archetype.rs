
pub mod wall {
    use crate::{entity::{Components, Entities, EntityId, EntityManager, EntityView, Position}, math::Mat4, palette::Palette, render::{Instance, InstancedShapeManager}};

    pub fn new(man: &mut EntityManager, position: Position) -> EntityId {
        let id = man.spawn(Entities::Wall, &[
            Components::Position,
            Components::Collider,
        ]);

        let mut wall = man.view(id).unwrap();
        wall.set_position(position);

        id
    }

    pub fn draw(entity: EntityView, renderer: &mut InstancedShapeManager, palette: Palette) {
        let pos = entity.get_position().unwrap();

        renderer.push_instance(Instance {
            transform: Mat4::translate(pos),
            col: palette.wall,
        });
    }
}

pub mod background {
    use crate::{entity::{Components, Entities, EntityId, EntityManager, EntityView, Position}, math::{Mat4, Vec2, Vec3}, palette::Palette, render::{Instance, InstancedShapeManager}};

    pub fn new(man: &mut EntityManager, position: Position, dimensions: Vec2) -> EntityId {
        let id = man.spawn(Entities::Background, &[
            Components::Position,
            Components::Collider,
            Components::Scale,
        ]);

        let mut bg = man.view(id).unwrap();
        bg.set_position(position);
        bg.set_scale(dimensions);

        id
    }

    pub fn draw(entity: EntityView, renderer: &mut InstancedShapeManager, palette: Palette) {
        let pos = entity.get_position().unwrap();
        let scale = entity.get_scale().unwrap();

        renderer.push_instance(Instance {
            transform: Mat4::scale(scale) * Mat4::translate(pos),
            col: palette.background,
        });
    }
}

pub mod snake {
    use crate::entity::{Components, Entities, EntityManager, EntityView};

    pub fn new(man: &mut EntityManager) {
        man.spawn(Entities::SnakeHead, &[
            Components::Position,
            Components::Direction,
            Components::Collider,
            Components::Keyboard,
        ]);
    }

    pub fn grow(entity: EntityView) {
        todo!()
    }
}

pub mod fruit {
    use crate::entity::EntityView;

    pub fn respawn(entity: EntityView) {
        todo!()
    }
}
