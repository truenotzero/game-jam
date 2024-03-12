use core::{arch, panic};
use std::{os::windows::thread, sync::mpsc::{self, Receiver, Sender}};

use rand::{thread_rng, Rng};

use crate::{
    archetype, entity::{Direction, Entities, EntityId, EntityManager, Position, Scale}, math::{Mat4, Vec2, Vec3, Vec4}, render::text::TextNames, sound::Sounds
};

const BACKGROUND_DEPTH: f32 = 0.9;
const WALL_DEPTH: f32 = 0.8;

pub enum _RoomType {
    Spawn,
    Hall,
    Walls,
    Swarm,
}

pub struct Room {
    position: Vec2,
    dimensions: Scale,
    parts: Vec<EntityId>,

    hall: Option<Box<Room>>,
    hall_direction: Direction,
    width: f32,
}

impl Room {
    fn new(man: &mut EntityManager, position: Vec2, dimensions: Scale) -> Self {
        let dimensions = dimensions + Vec2::diagonal(2.0);

        let mut this = Self {
            position,
            dimensions,
            parts: Vec::new(),
            hall_direction: Direction::default(),
            hall: None,
            width: 0.0,
        };

        // wall it off
        this.redraw_walls_and_bg(man);
        this
    }

    /// helps calculate offsets
    /// returns the center for the new room with given dimensions
    pub fn offset_from(&self, direction: Direction, dimensions: Scale) -> Vec2 {
        let pos = if let Some(hall) = &self.hall {
            hall.position
        } else {
            self.position
        };

        let dim = if let Some(hall) = &self.hall {
            hall.dimensions
        } else {
            self.dimensions
        };

        let d = Vec2::from(direction);
        pos + 0.5 * d * (dim + dimensions)
    }

    fn make_hall(
        &mut self,
        man: &mut EntityManager,
        direction: Direction,
        width: usize,
        length: usize,
    ) {
        // add walls
        let width = (width + 2) as f32;
        let length = length as f32;
        let d = Vec2::from(direction);

        // make the hallway room
        let (pos, dim) = match direction {
            Direction::Up | Direction::Down => {
                let pos = Vec2::new(
                    self.position.x, // + offset
                    self.position.y + 0.5 * d.y * (self.dimensions.y + length),
                );
                let dim = Vec2::new(width, length);

                (pos, dim)
            }
            Direction::Left | Direction::Right => {
                let pos = Vec2::new(
                    self.position.x + 0.5 * d.x * (self.dimensions.x + length),
                    self.position.y, // + offset
                );
                let dim = Vec2::new(length, width);

                (pos, dim)
            }
            _ => panic!(),
        };

        let hall = Self::new(man, pos, dim);
        self.hall = Some(Box::new(hall));
        self.hall_direction = direction;
        self.width = width;
    }

    /// breaks wall, optionally putting triggers in its place
    fn break_wall(
        &mut self,
        man: &mut EntityManager,
        side: Direction,
        hole_size: f32,
        tx: Option<Sender<()>>,
    ) {
        let (xs, xe, ys, ye) = match side {
            Direction::Up => {
                let y = self.position.y - 0.5 * self.dimensions.y;
                let xs = self.position.x - 0.5 * hole_size;
                let xe = self.position.x + 0.5 * hole_size - 1.0;
                (xs, xe, y, y)
            }
            Direction::Down => {
                let y = self.position.y + 0.5 * self.dimensions.y - 1.0;
                let xs = self.position.x - 0.5 * hole_size;
                let xe = self.position.x + 0.5 * hole_size - 1.0;
                (xs, xe, y, y)
            }
            Direction::Left => {
                let x = self.position.x - 0.5 * self.dimensions.x;
                let ys = self.position.y - 0.5 * hole_size;
                let ye = self.position.y + 0.5 * hole_size - 1.0;
                (x, x, ys, ye)
            }
            Direction::Right => {
                let x = self.position.x + 0.5 * self.dimensions.x - 1.0;
                let ys = self.position.y - 0.5 * hole_size;
                let ye = self.position.y + 0.5 * hole_size - 1.0;
                (x, x, ys, ye)
            }
            _ => panic!(),
        };

        let mut triggers = Vec::new();
        for &id in &self.parts {
            if let Some(mut wall) = man.view(id) {
                let pos = wall.get_position();

                if xs <= pos.x && pos.x <= xe && ys <= pos.y && pos.y <= ye {
                    wall.kill();

                    if let Some(tx) = tx.clone() {
                        let t = archetype::trigger::new(
                            man,
                            pos.into(),
                            |e| e.which() == Entities::SnakeHead,
                            tx,
                        );
                        triggers.push(t);
                    }
                }
            }
        }

        for t in triggers {
            self.parts.push(t);
        }
    }

    /// returns two trigger listeners
    /// the first triggers when the player leaves the room and enters the hallway
    /// the second triggers when the player is about to leave the hallway and enter the next room
    pub fn open_hallway(&mut self, man: &mut EntityManager) -> (Receiver<()>, Receiver<()>) {
        let hall = self.hall.as_mut().expect("should have hallway");

        archetype::oneshot::play_sound(man, Sounds::RoomUnlocked);

        let (tx_near, rx_near) = mpsc::channel();
        let (tx_far, rx_far) = mpsc::channel();

        hall.break_wall(man, self.hall_direction, self.width, Some(tx_far.clone()));
        hall.break_wall(man, self.hall_direction.reverse(), self.width, None);
        self.break_wall(man, self.hall_direction, self.width, Some(tx_near.clone()));

        (rx_near, rx_far)
    }

    pub fn redraw_walls_and_bg(&mut self, man: &mut EntityManager) {
        let mut new_parts = Vec::new();

        let translate = self.position - 0.5 * self.dimensions;
        let room_to_world = Mat4::translate(Vec3::from((translate, 0.0)));

        // make the background
        let bgpos = self.position - 0.5 * self.dimensions;
        let bg = archetype::background::new(
            man,
            Position::new(bgpos.x, bgpos.y, BACKGROUND_DEPTH),
            self.dimensions,
        );

        new_parts.push(bg);

        let width = self.dimensions.x as usize;
        let height = self.dimensions.y as usize;
        for y in 0..height {
            for x in 0..width {
                if !(y == 0 || y == height - 1 || x == 0 || x == width - 1) {
                    continue;
                }

                let room_pos = Vec4::new(x as f32, y as f32, 0.0, 1.0);
                let world_pos4 = room_to_world * room_pos;
                let p = Position::new(world_pos4.x, world_pos4.y, WALL_DEPTH);

                let wall = archetype::wall::new(man, p);
                new_parts.push(wall);
            }
        }

        std::mem::swap(&mut new_parts, &mut self.parts);
        for part in new_parts {
            man.kill(part);
        }
    }

    pub fn close_hall_entrance(&mut self, man: &mut EntityManager) {
        let side = self.hall_direction;
        if let Some(hall) = &mut self.hall {
            let mut xs = f32::MAX;
            let mut xe = f32::MIN;
            let mut ys = f32::MAX;
            let mut ye = f32::MIN;

            let mut triggers = Vec::new();
            for (idx, &ent) in hall.parts.iter().enumerate() {
                if let Some(trigger) = man.view(ent) {
                    if trigger.which() != Entities::Trigger {
                        continue;
                    }

                    triggers.push(idx);
                    let pos = trigger.get_position();
                    xs = xs.min(pos.x);
                    xe = xe.max(pos.x);

                    ys = ys.min(pos.y);
                    ye = ye.max(pos.y);
                }
            }

            // purge triggers
            for idx in triggers.into_iter().rev() {
                let id = hall.parts.swap_remove(idx);
                man.kill(id);
            }

            // make walls in their place
            for y in ys as isize..=ye as isize {
                let y = y as f32;
                for x in xs as isize..=xs as isize {
                    let x = x as f32;
                    archetype::wall::new(man, Vec3::new(x, y, WALL_DEPTH));
                }
            }
        }
    }

    pub fn destroy(&mut self, man: &mut EntityManager) {
        for &part in &self.parts {
            man.kill(part);
        }

        if let Some(hall) = &mut self.hall {
            hall.destroy(man);
        }
    }

    /// view the room while keeping a 1:1 aspect ratio
    pub fn view(&self) -> Mat4 {
        let dim = self.dimensions.x.max(self.dimensions.y);
        Mat4::scale((0.33).into()) * 
        Mat4::screen(self.position, dim, dim)
    }

    /// view the hall while keeping a 1:1 aspect ratio
    pub fn view_hall(&self) -> Mat4 {
        if let Some(hall) = &self.hall {
            // let dim = hall.dimensions;
            // let dim = dim.x.max(dim.y);
            // Mat4::screen(hall.position, dim, dim)
            hall.view()
        } else {
            panic!()
        }
    }

    /// places text in room-space coordinates
    pub fn text_at(&mut self, man: &mut EntityManager, name: TextNames, position: Vec2, scale: f32) {
        let txt = archetype::text::new(man, name, self.position + position, scale);
        self.parts.push(txt);
    }

    /// generate a random position in the room (in world space coordinates)
    pub fn random_position(&self) -> Vec2 {
        let mut rng = thread_rng();
        let x = rng.gen_range(1.0..=self.dimensions.x - 1.0).floor();
        let y = rng.gen_range(1.0..=self.dimensions.y - 1.0).floor();
        self.position + (x, y).into()
    }

    // Room types
    fn empty(man: &mut EntityManager, position: Vec2, side: Direction, dimensions: Scale) -> Self {
        let mut ret = Self::new(man, position, dimensions);
        ret.make_hall(man, side, 6, 18);
        ret
    }

    pub fn tut_controls(man: &mut EntityManager) -> Self {
        let mut ret = Self::empty(man, Vec2::new(0.0, 0.0), Direction::random(), Vec2::diagonal(20.0));
        ret.text_at(man, TextNames::Snek, Vec2::new(0.0, -ret.dimensions.y / 4.0), 1.0 / 7.0);
        // ret.text_at(man, TextNames::Controls, Vec2::new(0.0, ret.dimensions.y / 4.0), 1.0 / 14.0);
        ret.text_at(man, TextNames::SnekGlitch, Vec2::new(0.0, ret.dimensions.y / 5.0), 1.0 / 7.0);
        ret
    }

    pub fn tut_fruit(man: &mut EntityManager, last: &Room) -> Self {
        let mut ret = Self::next(man, last, Vec2::new(20.0, 20.0));
        ret.text_at(man, TextNames::Fruit, Vec2::new(0.0, -ret.dimensions.y / 5.0), 1.0 / 14.0);
        archetype::fruit::bounded(man, ret.position, ret.dimensions, 3);
        ret
    }

    pub fn next(man: &mut EntityManager, last: &Room, dimensions: Scale) -> Self {
        let next_pos = last.offset_from(last.hall_direction, last.dimensions);
        let rand_side = loop {
            let rand = Direction::random();
            if rand != last.hall_direction.reverse() {
                break rand;
            }
        };
        let mut ret = Self::empty(man, next_pos, rand_side, dimensions);
        ret.break_wall(man, last.hall_direction.reverse(), last.width, None);
        ret
    }

    // pub fn spires(man: &mut EntityManager) -> Self {
    //     let mut rng = thread_rng();
    // }
}
