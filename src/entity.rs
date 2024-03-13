use core::fmt;
use std::{
    any::Any,
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use glfw::Key;
use rand::{thread_rng, Rng};

use crate::{
    math::{Vec2, Vec3},
    palette::{Palette, PaletteKey},
    render::RenderManager,
    sound::Player,
    time,
};

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Entities {
    #[default]
    Basic,
    Background,
    Wall,
    SnakeHead,
    SnakeBody,
    Fruit,
    _Enemy,
    Fireball,
    Trigger,
    Swoop,
    Text,
    Logic,
}

impl fmt::Display for Entities {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Entities {
    pub fn tick(self, dt: Duration, entity: &mut EntityView<'_>) {
        use crate::archetype::*;

        match self {
            Self::SnakeHead => snake::head_tick(dt, entity),
            Self::SnakeBody => snake::body_tick(dt, entity),
            Self::Fireball => fireball::tick(dt, entity),
            Self::Swoop => swoop::tick(dt, entity),
            Self::Text => text::tick(dt, entity),
            Self::Logic => logic::tick(dt, entity),
            _ => (),
        }
    }

    pub fn draw(self, entity: EntityView, renderer: &mut RenderManager, palette: Palette) {
        use crate::archetype::*;

        match self {
            Self::Wall => wall::draw(entity, renderer, palette),
            Self::Background => background::draw(entity, renderer, palette),
            Self::Fruit => fruit::draw(entity, renderer, palette),
            Self::SnakeHead | Self::SnakeBody => snake::draw(entity, renderer, palette),
            Self::Fireball => fireball::draw(entity, renderer, palette),
            Self::Swoop => swoop::draw(entity, renderer),
            Self::Text => text::draw(entity, renderer),
            _ => (),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Components {
    Position,
    Direction,
    Collider,
    Input,
    BodyLength,
    SelfDestruct,
    Scale,
    Timer,
    Spawner,
    Animation,
    Color,
    Speed,
    Properties,
    Sound,
}

impl fmt::Display for Components {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Components {
    fn _requires(self) -> Vec<Components> {
        use Components as C;
        match self {
            C::Direction => vec![C::Position],
            C::Collider => vec![C::Position],
            _ => vec![],
        }
    }
}

pub type Position = crate::math::Vec3;

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    #[default]
    None,
    Up,
    Down,
    Left,
    Right,
    Raw(Vec2),
}

impl Direction {
    pub fn reverse(self) -> Self {
        match self {
            Direction::None => Direction::None,
            Direction::Up => Direction::Down,
            Direction::Down => Direction::Up,
            Direction::Left => Direction::Right,
            Direction::Right => Direction::Left,
            Direction::Raw(e) => Direction::Raw(-e),
        }
    }

    /// Get the direction on the right of this one
    pub fn right(self) -> Self {
        match self {
            Direction::None => Direction::None,
            Direction::Up => Direction::Right,
            Direction::Down => Direction::Left,
            Direction::Left => Direction::Up,
            Direction::Right => Direction::Down,
            Direction::Raw(_) => unimplemented!(),
        }
    }

    pub fn random() -> Self {
        const CHOICES: [Direction; 4] = [
            Direction::Up,
            Direction::Down,
            Direction::Right,
            Direction::Left,
        ];

        let mut rng = thread_rng();
        let idx = rng.gen_range(0..CHOICES.len());

        CHOICES[idx]
    }
}

impl From<Direction> for Vec2 {
    fn from(value: Direction) -> Self {
        match value {
            Direction::None => Self::default(),
            Direction::Up => Self::new(0.0, -1.0),
            Direction::Down => Self::new(0.0, 1.0),
            Direction::Left => Self::new(-1.0, 0.0),
            Direction::Right => Self::new(1.0, 0.0),
            Direction::Raw(e) => e,
        }
    }
}

impl From<Direction> for Vec3 {
    fn from(value: Direction) -> Self {
        Vec3::from((Vec2::from(value), 0.0))
    }
}

#[derive(Default)]
struct Collider;

impl Collider {
    fn is_between<'v, 'r>(
        t1: Entities,
        t2: Entities,
        e1: &'r mut EntityView<'v>,
        e2: &'r mut EntityView<'v>,
    ) -> Option<(&'r mut EntityView<'v>, &'r mut EntityView<'v>)> {
        if e1.which() == t1 && e2.which() == t2 {
            Some((e1, e2))
        } else if e1.which() == t2 && e2.which() == t1 {
            Some((e2, e1))
        } else {
            None
        }
    }

    fn at_least<'v, 'r>(
        t: Entities,
        e1: &'r mut EntityView<'v>,
        e2: &'r mut EntityView<'v>,
    ) -> Option<(&'r mut EntityView<'v>, &'r mut EntityView<'v>)> {
        if e1.which() == t {
            Some((e1, e2))
        } else if e2.which() == t {
            Some((e2, e1))
        } else {
            None
        }
    }

    pub fn collide<'v>(e1: &mut EntityView<'v>, e2: &mut EntityView<'v>) {
        use crate::archetype::*;
        use Entities as E;
        if let Some((head, fruit)) = Self::is_between(E::SnakeHead, E::Fruit, e1, e2) {
            fruit::respawn(fruit);
            snake::grow(head);
        } else if let Some((head, _body)) = Self::is_between(E::SnakeHead, E::SnakeBody, e1, e2) {
            snake::die_sequence(head);
        } else if let Some((head, _wall)) = Self::is_between(E::SnakeHead, E::Wall, e1, e2) {
            snake::die_sequence(head);
        } else if let Some((fireball, _wall)) = Self::is_between(E::Fireball, E::Wall, e1, e2) {
            // TODO
            fireball.kill();
        } else if let Some((trigger, other)) = Self::at_least(Entities::Trigger, e1, e2) {
            trigger::activated(trigger, other);
        }
    }
}

pub struct Input {
    key_tx: Sender<Key>,
    key_rx: Receiver<Key>,

    mouse_pos: Vec2,
}

impl Default for Input {
    fn default() -> Self {
        let (key_tx, key_rx) = mpsc::channel();
        Self {
            key_tx,
            key_rx,
            mouse_pos: Default::default(),
        }
    }
}

impl Input {
    pub fn press(&mut self, key: Key) {
        let _ = self.key_tx.send(key);
    }

    pub fn mouse_move(&mut self, pos: Vec2) {
        self.mouse_pos = pos;
    }

    pub fn get_key(&mut self) -> Option<Key> {
        self.key_rx.try_recv().ok()
    }

    pub fn get_mouse(&self) -> Vec2 {
        self.mouse_pos
    }
}

pub type BodyLength = i16;
pub type SelfDestruct = i16;
pub type Scale = crate::math::Vec2;
pub type Timer = time::Threshold;
pub type Property = Rc<RefCell<dyn Any>>;
pub type Properties = HashMap<&'static str, Property>;

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub enum Animation {
    #[default]
    Idle,
    _Growing,
}

pub type Color = PaletteKey;
pub type Speed = f32;
pub type Sound = Player;

pub type EntityId = usize;

pub struct EntityView<'m> {
    id: EntityId,
    type_: Entities,
    storages: &'m RefCell<Storages>,
    kill_signal: Sender<EntityId>,
}

impl<'m> EntityView<'m> {
    fn new(
        id: EntityId,
        type_: Entities,
        storages: &'m RefCell<Storages>,
        kill_signal: Sender<EntityId>,
    ) -> Self {
        Self {
            id,
            type_,
            storages,
            kill_signal,
        }
    }

    pub fn kill(&mut self) {
        let _ = self.kill_signal.send(self.id);
    }

    pub fn _id(&self) -> EntityId {
        self.id
    }

    pub fn which(&self) -> Entities {
        self.type_
    }

    fn storage(&self) -> Ref<'_, Storages> {
        self.storages.borrow()
    }

    fn storage_mut(&self) -> RefMut<'_, Storages> {
        self.storages.borrow_mut()
    }

    fn unwrap<T>(&self, t: Option<T>, component: Components) -> T {
        t.expect(&format!("{} should have {}", self.type_, component))
    }

    pub fn get_sound(&self) -> Sound {
        self.unwrap(self.storage().get_sound(self.id), Components::Sound)
    }

    pub fn new_property(&self, name: &'static str, value: impl Any) {
        self.storage_mut().new_property(self.id, name, value)
    }

    pub fn remove_property(&self, name: &str) {
        self.storage_mut().remove_property(self.id, name);
    }

    pub fn has_property(&self, name: &str) -> bool {
        self.storage().get_property(self.id, name).is_some()
    }

    pub fn get_property<T: Clone + 'static>(&self, name: &str) -> T {
        self.with_property::<T, T>(name, |f| f.clone())
    }

    pub fn set_property<T: 'static>(&self, name: &str, value: T) {
        self.with_mut_property(name, |f| *f = value);
    }

    pub fn with_property<P: 'static, R>(&self, name: &str, f: impl FnOnce(&P) -> R) -> R {
        let prop = self
            .storage()
            .get_property(self.id, name)
            .expect("entity should have specified property");
        let any = prop.borrow();
        let p = any
            .downcast_ref()
            .expect("property should have matching type");
        f(p)
    }

    pub fn with_mut_property<P: 'static, R>(&self, name: &str, f: impl FnOnce(&mut P) -> R) -> R {
        let prop = self
            .storage()
            .get_property(self.id, name)
            .expect("entity should have specified property");
        let mut any = prop.borrow_mut();
        let p = any
            .downcast_mut()
            .expect("property should have matching type");
        f(p)
    }

    pub fn get_position(&self) -> Position {
        self.unwrap(self.storage().get_position(self.id), Components::Position)
    }
    pub fn set_position(&mut self, position: Position) {
        self.storage_mut().set_position(self.id, position)
    }

    pub fn get_direction(&self) -> Direction {
        self.unwrap(self.storage().get_direction(self.id), Components::Direction)
    }

    pub fn set_direction(&mut self, direction: Direction) {
        self.storage_mut().set_direction(self.id, direction)
    }

    pub fn get_body_length(&self) -> BodyLength {
        self.unwrap(
            self.storage().get_body_length(self.id),
            Components::BodyLength,
        )
    }

    pub fn set_body_length(&mut self, body_length: BodyLength) {
        self.storage_mut().set_body_length(self.id, body_length);
    }

    pub fn get_self_destruct(&self) -> SelfDestruct {
        self.unwrap(
            self.storage().get_self_destruct(self.id),
            Components::SelfDestruct,
        )
    }

    pub fn set_self_destruct(&mut self, self_destruct: SelfDestruct) {
        self.storage_mut().set_self_destruct(self.id, self_destruct);
    }

    pub fn get_scale(&self) -> Scale {
        self.unwrap(self.storage().get_scale(self.id), Components::Scale)
    }

    pub fn set_scale(&mut self, scale: Scale) {
        self.storage_mut().set_scale(self.id, scale)
    }

    pub fn get_key(&mut self) -> Option<Key> {
        self.unwrap(self.storage_mut().get_key(self.id), Components::Input)
    }

    pub fn get_mouse(&self) -> Vec2 {
        self.unwrap(self.storage().get_mouse(self.id), Components::Input)
    }

    pub fn _get_animation(&self) -> Animation {
        self.unwrap(
            self.storage()._get_animation(self.id),
            Components::Animation,
        )
    }

    pub fn set_animation(&mut self, animation: Animation) {
        self.storage_mut().set_animation(self.id, animation)
    }

    pub fn get_color(&self) -> Color {
        self.unwrap(self.storage().get_color(self.id), Components::Color)
    }

    pub fn set_color(&mut self, color: Color) {
        self.storage_mut().set_color(self.id, color);
    }

    pub fn get_speed(&self) -> Speed {
        self.unwrap(self.storage().get_speed(self.id), Components::Speed)
    }

    pub fn set_speed(&mut self, speed: Speed) {
        self.storage_mut().set_speed(self.id, speed);
    }

    pub fn access_timer<T>(&mut self, f: impl FnOnce(&mut Timer) -> T) -> T {
        let mut storage = self.storage_mut();
        let mut timer = storage.access_timer(self.id).expect(&format!(
            "{} should have {}",
            self.type_,
            Components::Timer
        ));
        f(&mut timer)
    }

    pub fn request_spawn(&mut self, request: EntityManagerRequest) {
        let spawner = self.storage_mut().access_spawner(self.id).expect(&format!(
            "{} should have {}",
            self.type_,
            Components::Spawner
        ));
        let _ = spawner.send(request);
    }

    fn _is_collider(&self) -> bool {
        self.storage().is_collider(self.id)
    }
}

type Storage<T> = HashMap<EntityId, T>;

type EntityManagerRequest = Box<dyn FnOnce(&mut EntityManager)>;

struct Storages {
    spawn_requests: Sender<EntityManagerRequest>,
    collisions: Sender<(EntityId, EntityId)>,
    sound: Sound,

    positions: Storage<Position>,
    directions: Storage<Direction>,
    colliders: Storage<Collider>,
    keyboards: Storage<Input>,
    body_lengths: Storage<BodyLength>,
    self_destructs: Storage<SelfDestruct>,
    scales: Storage<Scale>,
    timers: Storage<Timer>,
    spawners: Storage<()>,
    animations: Storage<Animation>,
    colors: Storage<Color>,
    speeds: Storage<Speed>,
    properties: Storage<Properties>,
    sounds: Storage<Sound>,
}

impl Storages {
    pub fn new(
        spawn_requests: Sender<EntityManagerRequest>,
        collisions: Sender<(EntityId, EntityId)>,
        sound: Sound,
    ) -> Self {
        Self {
            spawn_requests,
            collisions,
            sound,

            positions: Default::default(),
            directions: Default::default(),
            colliders: Default::default(),
            keyboards: Default::default(),
            body_lengths: Default::default(),
            self_destructs: Default::default(),
            scales: Default::default(),
            timers: Default::default(),
            spawners: Default::default(),
            animations: Default::default(),
            colors: Default::default(),
            speeds: Default::default(),
            properties: Default::default(),
            sounds: Default::default(),
        }
    }

    pub fn kill(&mut self, entity: EntityId) {
        // binary search is legal because entity id is ever-increasing
        // and insertion happens only at the end (thus keeping the vector sorted)
        self.positions.remove(&entity);
        self.directions.remove(&entity);
        self.colliders.remove(&entity);
        self.keyboards.remove(&entity);
        self.body_lengths.remove(&entity);
        self.self_destructs.remove(&entity);
        self.scales.remove(&entity);
        self.timers.remove(&entity);
        self.spawners.remove(&entity);
        self.animations.remove(&entity);
        self.colors.remove(&entity);
        self.speeds.remove(&entity);
        self.properties.remove(&entity);
        self.sounds.remove(&entity);
    }

    pub fn add_component(&mut self, entity: EntityId, component: Components) {
        use Components as C;
        match component {
            C::Position => self.set_position(entity, Position::default()),
            C::Direction => self.set_direction(entity, Direction::default()),
            C::Collider => self.add_collider(entity),
            C::Input => self.add_keyboard(entity),
            C::BodyLength => self.set_body_length(entity, 0),
            C::SelfDestruct => self.set_self_destruct(entity, 0),
            C::Scale => self.set_scale(entity, Scale::diagonal(1.0)),
            C::Timer => {
                self.timers.insert(entity, Timer::default());
            }
            C::Spawner => {
                self.spawners.insert(entity, ());
            }
            C::Animation => self.set_animation(entity, Animation::default()),
            C::Color => self.set_color(entity, Color::default()),
            C::Speed => self.set_speed(entity, Speed::default()),
            C::Properties => {
                self.properties.insert(entity, Default::default());
            }
            C::Sound => {
                self.sounds.insert(entity, self.sound.clone());
            }
        }
    }

    pub fn get_sound(&self, entity: EntityId) -> Option<Sound> {
        self.sounds.get(&entity).cloned()
    }

    pub fn new_property(&mut self, entity: EntityId, name: &'static str, value: impl Any) {
        self.properties
            .get_mut(&entity)
            .map(|p| p.insert(name, Rc::new(RefCell::new(value))));
    }

    pub fn remove_property(&mut self, entity: EntityId, name: &str) {
        self.properties.get_mut(&entity).map(|p| p.remove(name));
    }

    pub fn get_property(&self, entity: EntityId, name: &str) -> Option<Property> {
        self.properties
            .get(&entity)
            .and_then(|m| m.get(name))
            .cloned()
    }

    pub fn get_position(&self, entity: EntityId) -> Option<Position> {
        self.positions.get(&entity).copied()
    }

    pub fn set_position(&mut self, entity: EntityId, position: Position) {
        // check collision
        if self.is_collider(entity) {
            for (&other, &other_pos) in &self.positions {
                if !self.is_collider(other) {
                    continue;
                }

                let self_pos = Vec2::from(position).floor();
                let other_pos = Vec2::from(other_pos).floor();

                if self_pos.eq(other_pos) {
                    // collision detected
                    let _ = self.collisions.send((entity, other));
                }
            }
        }
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
        self.keyboards.insert(entity, Input::default());
    }

    pub fn get_key(&mut self, entity: EntityId) -> Option<Option<Key>> {
        self.keyboards.get_mut(&entity).map(|kb| kb.get_key())
    }

    pub fn get_mouse(&self, entity: EntityId) -> Option<Vec2> {
        self.keyboards.get(&entity).map(|k| k.get_mouse())
    }

    pub fn key_pressed(&mut self, key: Key) {
        for kb in self.keyboards.values_mut() {
            kb.press(key);
        }
    }

    pub fn mouse_moved(&mut self, mouse: Vec2) {
        for m in self.keyboards.values_mut() {
            m.mouse_move(mouse);
        }
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

    pub fn _get_animation(&self, entity: EntityId) -> Option<Animation> {
        self.animations.get(&entity).copied()
    }

    pub fn set_animation(&mut self, entity: EntityId, animation: Animation) {
        self.animations.insert(entity, animation);
    }

    pub fn get_color(&self, entity: EntityId) -> Option<Color> {
        self.colors.get(&entity).copied()
    }

    pub fn set_color(&mut self, entity: EntityId, color: Color) {
        self.colors.insert(entity, color);
    }

    pub fn get_speed(&self, entity: EntityId) -> Option<Speed> {
        self.speeds.get(&entity).copied()
    }

    pub fn set_speed(&mut self, entity: EntityId, speed: Speed) {
        self.speeds.insert(entity, speed);
    }

    fn access_timer(&mut self, entity: EntityId) -> Option<&mut Timer> {
        self.timers.get_mut(&entity)
    }

    fn access_spawner(&mut self, entity: EntityId) -> Option<Sender<EntityManagerRequest>> {
        if self.spawners.contains_key(&entity) {
            Some(self.spawn_requests.clone())
        } else {
            None
        }
    }

    fn is_collider(&self, id: EntityId) -> bool {
        self.colliders.contains_key(&id)
    }
}

pub struct EntityManager {
    tracker: EntityId,
    entities: Vec<EntityId>,
    types: Vec<Entities>,

    keystrokes: Receiver<Key>,
    mouse_movements: Receiver<Vec2>,
    spawn_requests: Receiver<EntityManagerRequest>,
    collision_requests: Receiver<(EntityId, EntityId)>,
    dying_rx: Receiver<EntityId>,
    dying_tx: Sender<EntityId>,
    storage: RefCell<Storages>,
}

impl EntityManager {
    pub fn new(keystroke_rx: Receiver<Key>, mouse_rx: Receiver<Vec2>, sound: Sound) -> Self {
        let (spawn_tx, spawn_rx) = mpsc::channel();
        let (collisions_tx, collisions_rx) = mpsc::channel();
        let (dying_tx, dying_rx) = mpsc::channel();

        Self {
            tracker: Default::default(),
            entities: Default::default(),
            types: Default::default(),

            keystrokes: keystroke_rx,
            mouse_movements: mouse_rx,
            spawn_requests: spawn_rx,
            collision_requests: collisions_rx,
            dying_rx,
            dying_tx,
            storage: RefCell::new(Storages::new(spawn_tx, collisions_tx, sound)),
        }
    }

    pub fn spawn(&mut self, type_: Entities, components: &[Components]) -> EntityId {
        let id = self.tracker;
        self.tracker += 1;

        self.entities.push(id);
        self.types.push(type_);

        for c in components {
            if let Ok(mut storage) = self.storage.try_borrow_mut() {
                storage.add_component(id, *c);
            }
        }

        id
    }

    pub fn _iter_mut(&mut self) -> impl Iterator<Item = EntityView> {
        self.entities.iter().filter_map(|&id| self.view(id))
    }

    pub fn kill(&mut self, entity: EntityId) {
        // binary search is legal because entity id is ever-increasing
        // and insertion happens only at the end (thus keeping the vector sorted)
        if let Ok(index) = self.entities.binary_search(&entity) {
            self.entities.remove(index);
            self.types.remove(index);

            // remove components if they exist
            if let Ok(mut storage) = self.storage.try_borrow_mut() {
                storage.kill(entity);
            }
        }
    }

    pub fn view(&self, entity: EntityId) -> Option<EntityView> {
        let index = self.entities.binary_search(&entity).ok()?;
        Some(EntityView::new(
            entity,
            self.types[index],
            &self.storage,
            self.dying_tx.clone(),
        ))
    }

    pub fn tick(&mut self, dt: Duration) {
        // handle keystrokes
        while let Ok(key) = self.keystrokes.try_recv() {
            self.storage.borrow_mut().key_pressed(key);
        }

        // hanlde mouse movement
        while let Ok(mouse) = self.mouse_movements.try_recv() {
            self.storage.borrow_mut().mouse_moved(mouse);
        }

        // tick entities
        for &id in &self.entities {
            let mut view = self.view(id).unwrap();
            view.which().tick(dt, &mut view);
        }

        // handle killing off entities
        while let Ok(dying) = self.dying_rx.try_recv() {
            self.kill(dying);
        }

        // handle spawning new entities
        while let Ok(spawn_request) = self.spawn_requests.try_recv() {
            spawn_request(self);
        }

        // check collisions
        while let Ok((id1, id2)) = self.collision_requests.try_recv() {
            if let Some(mut e1) = self.view(id1) {
                if let Some(mut e2) = self.view(id2) {
                    Collider::collide(&mut e1, &mut e2);
                }
            }
        }
    }

    pub fn draw(&mut self, renderer: &mut RenderManager, palette: Palette) {
        for id in self.entities.iter().cloned() {
            let view = self.view(id).unwrap();
            view.which().draw(view, renderer, palette);
        }
    }
}
