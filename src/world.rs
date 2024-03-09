use core::panic;
use std::{
    sync::mpsc::{self, Receiver, Sender},
    thread,
};

use crate::{
    archetype,
    entity::{Direction, Entities, EntityId, EntityManager, Position, Scale},
    gl::raw::BACK,
    math::{Mat4, Vec2, Vec3, Vec4},
    world,
};

const BACKGROUND_DEPTH: f32 = 0.9;
const WALL_DEPTH: f32 = 0.8;

pub enum RoomType {
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
    direction: Direction,
    width: f32,
}

impl Room {
    fn new(man: &mut EntityManager, position: Vec2, dimensions: Scale) -> Self {
        let dimensions = dimensions + Vec2::diagonal(2.0);
        let translate = position - 0.5 * dimensions;
        let room_to_world = Mat4::translate(Vec3::from((translate, 0.0)));

        let mut parts = Vec::new();
        // make the background
        let bgpos = position - 0.5 * dimensions;
        let bg = archetype::background::new(
            man,
            Position::new(bgpos.x, bgpos.y, BACKGROUND_DEPTH),
            dimensions,
        );
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

        Self {
            position,
            dimensions,
            parts,
            direction: Direction::default(),
            hall: None,
            width: 0.0,
        }
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
        self.direction = direction;
        self.width = width;
    }

    fn replace_walls_with_triggers(
        &self,
        man: &mut EntityManager,
        side: Direction,
        hole_size: f32,
        tx: Sender<()>,
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

        for &id in &self.parts {
            if let Some(mut wall) = man.view(id) {
                let pos = wall.get_position();

                if xs <= pos.x && pos.x <= xe && ys <= pos.y && pos.y <= ye {
                    wall.kill();

                    archetype::trigger::new(man, pos.into(), |_| true, tx.clone());
                }
            }
        }
    }

    /// returns two trigger listeners
    /// the first triggers when the player leaves the room and enters the hallway
    /// the second triggers when the player is about to leave the hallway and enter the next room
    pub fn open_hallway(&self, man: &mut EntityManager) -> (Receiver<()>, Receiver<()>) {
        let hall = self.hall.as_ref().expect("should have hallway");
        let (tx_near, rx_near) = mpsc::channel();
        let (tx_far, rx_far) = mpsc::channel();

        hall.replace_walls_with_triggers(man, self.direction, self.width, tx_far.clone());
        hall.replace_walls_with_triggers(
            man,
            self.direction.reverse(),
            self.width,
            tx_near.clone(),
        );
        self.replace_walls_with_triggers(man, self.direction, self.width, tx_near.clone());

        (rx_near, rx_far)
    }

    pub fn spawn(man: &mut EntityManager) -> Self {
        let mut ret = Self::new(man, Vec2::new(0.0, 0.0), Vec2::new(20.0, 20.0));
        ret.make_hall(man, Direction::Down, 6, 18);
        ret
    }

    pub fn destroy(&mut self, man: &mut EntityManager) {
        for &part in &self.parts {
            man.kill(part);
        }

        if let Some(hall) = &mut self.hall {
            hall.destroy(man);
        }
    }

    // view the room while keeping a 1:1 aspect ratio
    pub fn view_room(&self) -> Mat4 {
        let dim = self.dimensions.x.max(self.dimensions.y);
        Mat4::screen(self.position, dim, dim)
    }

    // view the hall while keeping a 1:1 aspect ratio
    pub fn view_hall(&self) -> Mat4 {
        if let Some(hall) = &self.hall {
            // let dim = hall.dimensions;
            // let dim = dim.x.max(dim.y);
            // Mat4::screen(hall.position, dim, dim)
            hall.view_room()
        } else {
            panic!()
        }
    }
}
