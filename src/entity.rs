
use std::{collections::HashMap, iter, time::Duration};

use crate::{palette::Palette, render::InstancedShapeManager};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entities {
    Background,
    Wall,
    SnakeHead,
    SnakeBody,
    Fruit,
    Enemy,
    Projectile,
}

impl Entities {
    pub fn tick(self, dt: Duration, entity: EntityView<'_>) {
        // TODO
    }

    pub fn draw(self, entity: EntityView<'_>, renderer: &mut InstancedShapeManager, palette: Palette) {
        use crate::archetype::*;

        match self {
            Self::Wall => wall::draw(entity, renderer, palette),
            _ => (),
        }
    }
}


#[derive(Clone, Copy)]
pub enum Components {
    Position,
    Direction,
    Collider,
    Keyboard,
    BodyLength,
    SelfDestruct,
    Scale,
}

impl Components {
    fn requires(self) -> Vec<Components> {
        use Components as C;
        match self {
            C::Position => vec![],
            C::Direction => vec![C::Position],
            C::Collider => vec![C::Position],
            C::Keyboard => vec![],
            C::BodyLength => vec![],
            C::SelfDestruct => vec![],
            C::Scale => vec![],
        }
    }
}

pub type Position = crate::math::Vec3;
pub type Direction = crate::math::Vec2;

#[derive(Default)]
struct Collider;

impl Collider {
    fn is_between<'a>(t1: Entities, t2: Entities, (e1, e2): (EntityView<'a>, EntityView<'a>)) -> Option<(EntityView<'a>, EntityView<'a>)> {
        if e1.which() == t1 && e2.which() == t2 {
            Some((e1, e2))
        } else if 
        e1.which() == t2 && e2.which() == t1 {
            Some((e2, e1))
        } else {
            None
        }
    }

    pub fn collide(e: (EntityView, EntityView)) {
        use crate::archetype::*;
        use Entities as E;
        if let Some((h, f)) = Self::is_between(E::SnakeHead, E::Fruit, e) {
            snake::grow(h);
            fruit::respawn(f);
        }
    }
}

#[derive(Default)]
pub struct Keyboard;

pub type BodyLength = i16;
pub type SelfDestruct = i16;
pub type Scale = crate::math::Vec2;


pub type EntityId = usize;

pub struct EntityView<'m> {
    id: EntityId,
    type_: Entities,
    man: &'m mut Storages,
}

impl<'m> EntityView<'m> {
    pub fn new(id: EntityId, type_: Entities, man: &'m mut Storages) -> Self {
        Self {
            id,
            type_,
            man,
        }
    }

    pub fn which(&self) -> Entities {
        self.type_
    }

    pub fn get_position(&self) -> Option<Position> { 
        self.man.get_position(self.id)
    }
    pub fn set_position(&mut self, position: Position) {
        self.man.set_position(self.id, position)
    }
    
    pub fn get_direction(&self) -> Option<Direction> { 
        self.man.get_direction(self.id)
    }
    
    pub fn set_direction(&mut self, direction: Direction) { 
        self.man.set_direction(self.id, direction)
    }

    
    pub fn get_scale(&self) -> Option<Scale> { 
        self.man.get_scale(self.id)
    }
    
    pub fn set_scale(&mut self, scale: Scale) { 
        self.man.set_scale(self.id, scale)
    }
}

type Storage<T> = HashMap<EntityId, T>;

#[derive(Default)]
struct Storages {
    positions: Storage<Position>,
    directions: Storage<Direction>,
    colliders: Storage<Collider>,
    keyboards: Storage<Keyboard>,
    body_lengths: Storage<BodyLength>,
    self_destructs: Storage<SelfDestruct>,
    scales: Storage<Scale>,
}

impl Storages {
    pub fn kill(&mut self, entity: EntityId) {
        self.positions.remove(&entity);
        self.directions.remove(&entity);
    }

    pub fn add_component(&mut self, entity: EntityId, component: Components) {
        use Components as C;
        match component {
            C::Position => self.set_position(entity, Position::default()),
            C::Direction => self.set_direction(entity, Direction::default()),
            C::Collider => self.add_collider(entity),
            C::Keyboard => self.add_keyboard(entity),
            C::BodyLength => self.set_body_length(entity, 0),
            C::SelfDestruct => self.set_self_destruct(entity, 0),
            C::Scale => self.set_scale(entity, Scale::diagonal(1.0)),
        }
    }

    pub fn get_position(&self, entity: EntityId) -> Option<Position> {
        self.positions.get(&entity).copied()
    }
    pub fn set_position(&mut self, entity: EntityId, position: Position) { 
        self.positions.insert(entity, position);
    }
    
    pub fn get_direction(&self, entity: EntityId) -> Option<Direction> { 
        self.directions.get(&entity).copied()
    }
    pub fn set_direction(&mut self, entity: EntityId, direction: Direction) {
        self.directions.insert(entity, direction);
    }

    pub fn add_collider(&mut self, entity: EntityId) {
        self.colliders.insert(entity, Collider::default());
    }

    pub fn add_keyboard(&mut self, entity: EntityId) {
        self.keyboards.insert(entity, Keyboard::default());
    }

    pub fn get_body_length(&self, entity: EntityId) -> Option<BodyLength> {
        self.body_lengths.get(&entity).copied()
    }

    pub fn set_body_length(&mut self, entity: EntityId, body_length: BodyLength) {
        self.body_lengths.insert(entity, body_length);
    }

    pub fn get_self_destruct(&self, entity: EntityId) -> Option<SelfDestruct> {
        self.self_destructs.get(&entity).copied()
    }

    pub fn set_self_destruct(&mut self, entity: EntityId, self_destruct: SelfDestruct) {
        self.self_destructs.insert(entity, self_destruct);
    }

    pub fn get_scale(&self, entity: EntityId) -> Option<Scale> {
        self.scales.get(&entity).copied()
    }

    pub fn set_scale(&mut self, entity: EntityId, scale: Scale) {
        self.scales.insert(entity, scale);
    }
}

#[derive(Default)]
pub struct EntityManager {
    tracker: EntityId,
    entities: Vec<EntityId>,
    types: Vec<Entities>,

    storage: Storages,
}

impl EntityManager {
    pub fn spawn(&mut self, type_: Entities, components: &[Components]) -> EntityId { 
        let id = self.tracker;
        self.tracker += 1;
        
        self.entities.push(id);
        self.types.push(type_);

        for c in components {
            self.storage.add_component(id, *c);
        }

        id
    }

    pub fn kill(&mut self, entity: EntityId) {
        // binary search is legal because entity id is ever-increasing
        // and insertion happens only at the end (thus keeping the vector sorted)
        let index = self.entities.binary_search(&entity).unwrap();
        self.entities.remove(index);
        self.types.remove(index);

        // remove components if they exist
        self.storage.kill(entity);
    }

    pub fn view(&mut self, entity: EntityId) -> Option<EntityView> {
        let index = self.entities.binary_search(&entity).ok()?;
        let type_ = self.types[index];
        Some(EntityView::new(entity, type_, &mut self.storage))
    }

    pub fn tick(&mut self, dt: Duration) {
        for (id, e) in iter::zip(&self.entities, &self.types) {
            let view = EntityView::new(*id, *e, &mut self.storage);
            e.tick(dt, view);
        }
    }

    pub fn draw(&mut self, renderer: &mut InstancedShapeManager, palette: Palette) {
        for (id, e) in iter::zip(&self.entities, &self.types) {
            let view = EntityView::new(*id, *e, &mut self.storage);
            e.draw(view, renderer, palette);
        }
    }
}
