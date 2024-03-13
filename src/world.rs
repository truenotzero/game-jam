use core::{arch, panic};
use std::{
    mem::swap,
    sync::mpsc::{self, Receiver, Sender},
    time::Duration,
};

use rand::{thread_rng, Rng};

use crate::{
    archetype::{self, enemy, fruit, logic, snake, text},
    entity::{Direction, Entities, EntityId, EntityManager, Position, Scale},
    math::{Mat4, Vec2, Vec3, Vec4},
    render::text::TextNames,
    sound::Sounds,
    time::Threshold,
};

const BACKGROUND_DEPTH: f32 = 0.9;
const WALL_DEPTH: f32 = 0.8;

pub enum _RoomType {
    Spawn,
    Hall,
    Walls,
    Swarm,
}

type Hall = Box<Room>;

pub struct Room {
    snake_id: EntityId,

    position: Vec2,
    dimensions: Scale,
    parts: Vec<EntityId>,

    last_hall: Option<Hall>,
    hall: Option<Hall>,
    hall_open: bool,
    hall_direction: Direction,
    hall_width: f32,
}

impl Room {
    fn new(man: &mut EntityManager, position: Vec2, dimensions: Scale, snake_id: EntityId) -> Self {
        let dimensions = dimensions + Vec2::diagonal(2.0);

        let mut this = Self {
            snake_id,

            position,
            dimensions,
            parts: Vec::new(),

            last_hall: None,
            hall: None,
            hall_open: false,
            hall_direction: Direction::default(),
            hall_width: 0.0,
        };

        // wall it off
        this.redraw_walls_and_bg(man);
        this
    }

    /// helps calculate offsets
    /// returns the center for the new room with given dimensions
    pub fn offset_from(&self, next_room_at: Direction, next_room_dim: Scale) -> Vec2 {
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

        let d = Vec2::from(next_room_at);
        (pos + 0.5 * d * (dim + next_room_dim)).floor()
    }

    /// give hallway and snake into self
    pub fn swap(&mut self, other: &mut Self) {
        swap(self, other);
        swap(&mut self.last_hall, &mut other.hall);
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

        let hall = Self::new(man, pos, dim, self.snake_id);
        self.hall = Some(Box::new(hall));
        self.hall_direction = direction;
        self.hall_width = width;
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
                if wall.which() != Entities::Wall {
                    continue;
                }
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
    pub fn open_hallway(&mut self, man: &mut EntityManager) -> Option<(Receiver<()>, Receiver<()>)> {
        if self.hall_open { return None; }
        self.hall_open = true;
        let hall = self.hall.as_mut().expect("should have hallway");

        archetype::oneshot::play_sound(man, Sounds::RoomUnlocked);

        let (tx_near, rx_near) = mpsc::channel();
        let (tx_far, rx_far) = mpsc::channel();

        hall.break_wall(
            man,
            self.hall_direction,
            self.hall_width,
            Some(tx_far.clone()),
        );
        hall.break_wall(man, self.hall_direction.reverse(), self.hall_width, None);
        self.break_wall(
            man,
            self.hall_direction,
            self.hall_width,
            Some(tx_near.clone()),
        );

        Some((rx_near, rx_far))
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

        // debug make tile at local 0.0
        // let middle = archetype::wall::new(man, Vec3::from((self.position, 0.0)));
        // new_parts.push(middle);

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

        if let Some(hall) = &mut self.last_hall {
            hall.destroy(man);
        }
    }

    /// view the room while keeping a 1:1 aspect ratio
    pub fn view(&self) -> Mat4 {
        let dim = self.dimensions.x.max(self.dimensions.y);
        Mat4::screen(self.position, dim, dim)
    }

    /// view the hall while keeping a 1:1 aspect ratio
    pub fn view_hall(&self) -> Mat4 {
        if let Some(hall) = &self.hall {
            // let dim = hall.dimensions;
            // let dim = dim.x.max(dim.y);
            // Mat4::screen(hall.position, dim, dim)
            Mat4::scale((1.10).into()) * hall.view()
        } else {
            panic!()
        }
    }

    /// places text in room-space coordinates
    fn text_at(
        &mut self,
        man: &mut EntityManager,
        name: TextNames,
        position: Vec2,
        scale: f32,
    ) -> EntityId {
        let txt = archetype::text::new(man, name, self.position + position, scale);
        self.parts.push(txt);
        txt
    }

    /// places text to the right of some other text
    fn text_after(
        &mut self,
        man: &mut EntityManager,
        last_id: EntityId,
        name: TextNames,
    ) -> Option<EntityId> {
        let last = man.view(last_id)?;
        let last_pos = Vec2::from(last.get_position());
        let last_scale = last.with_property("scale", |&f: &f32| f);
        let last_dim =
            last.with_property("name", |name: &TextNames| last_scale * name.dimensions());

        let dim = last_scale * name.dimensions();
        let position = Vec2::new(last_pos.x + 0.5 * (last_dim.x + dim.x), last_pos.y);
        Some(Self::text_at(
            self,
            man,
            name,
            position - self.position,
            last_scale,
        ))
    }

    /// places text under some other text
    fn text_under(
        &mut self,
        man: &mut EntityManager,
        last_id: EntityId,
        name: TextNames,
    ) -> Option<EntityId> {
        let last = man.view(last_id)?;
        let last_pos = Vec2::from(last.get_position());
        let last_scale = last.with_property("scale", |&f: &f32| f);
        let last_dim =
            last.with_property("name", |name: &TextNames| last_scale * name.dimensions());

        let dim = last_scale * name.dimensions();
        let dim_y = dim.y / name.frames() as f32;
        let position = Vec2::new(last_pos.x, last_pos.y + 0.5 * (last_dim.y + dim_y));
        Some(Self::text_at(
            self,
            man,
            name,
            position - self.position,
            last_scale,
        ))
    }

    /// generate a random position in the room (in world space coordinates)
    pub fn random_position(&self) -> Vec2 {
        Self::make_random_gen(&self)(Vec2::diagonal(0.5))
    }

    fn make_random_gen(&self) -> impl Fn(Vec2) -> Vec2 {
        let dimensions = self.dimensions;
        let position = self.position;
        move |v| {
            let mut rng = thread_rng();
            loop {
                // let x = (0.5 * rng.gen_range(1.0..dimensions.x - 1.0)).floor();
                // let y = (0.5 * rng.gen_range(1.0..dimensions.y - 1.0)).floor();
                // let next = position - Vec2::new(x, y);
                let dx = (dimensions.x - 4.0) * 0.5;
                let dy = (dimensions.y - 4.0) * 0.5;
                let x = rng.gen_range(-dx..dx).floor();
                let y = rng.gen_range(-dy..dy).floor();
                let next = position - Vec2::new(x,y);


                if !v.eq(next) {
                    break next;
                }
            }
        }
    }

    pub fn add_logic(&mut self, man: &mut EntityManager, on_tick: impl FnMut(Duration) + 'static) {
        let logic = logic::new(man, Box::new(on_tick));
        self.parts.push(logic);
    }
    pub fn position(&self) -> Vec2 {
        self.position
    }

    // Room types
    fn empty(man: &mut EntityManager, position: Vec2, side: Direction, dimensions: Scale, snake_id: EntityId) -> Self {
        let mut ret = Self::new(man, position, dimensions, snake_id);
        let mut rng = thread_rng();
        let width = rng.gen_range(1..4) * 2;
        let length = rng.gen_range(5..=10) * 2;
        ret.make_hall(man, side, width, length);
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
        let mut ret = Self::empty(man, next_pos, rand_side, dimensions, last.snake_id);
        ret.break_wall(man, last.hall_direction.reverse(), last.hall_width, None);
        ret
    }

    pub fn tut_controls(man: &mut EntityManager) -> (Self, Receiver<()>) {
        let mut ret = Self::empty(
            man,
            Vec2::new(0.0, 0.0),
            Direction::random(),
            Vec2::diagonal(20.0),
            0,
        );

        let snake_position =ret.random_position();
        let snek = snake::new(man, snake_position);
        ret.snake_id = snek;

        let snek_move_rx = snake::make_move_trigger(man, snek);

        ret.text_at(
            man,
            TextNames::Controls,
            Vec2::new(0.0, ret.dimensions.y / 4.0),
            1.0 / 28.0,
        );
        let snek_txt = ret.text_at(
            man,
            TextNames::Snek,
            Vec2::new(0.0, -ret.dimensions.y / 4.0),
            1.0 / 14.0,
        );
        let snek_glitch_txt = ret
            .text_after(man, snek_txt, TextNames::SnekGlitch)
            .unwrap();

        let (tx_glitch, rx_glitch) = mpsc::channel();
        let (tx_hall, rx_hall) = mpsc::channel();
        let mut threshold = Threshold::new(Duration::MAX);
        let mut moved = false;
        ret.add_logic(man, move |dt| {
            if !moved && snek_move_rx.try_recv().is_ok() {
                moved = true;
                let _ = tx_glitch.send(());
                threshold.set_threshold(Duration::from_millis(3000));
                threshold.reset();
            }

            if threshold.tick(dt) {
                let _ = tx_hall.send(());
                threshold.set_threshold(Duration::MAX);
            }
        });

        text::add_glitch_trigger(man, snek_glitch_txt, rx_glitch);

        (ret, rx_hall)
    }

    pub fn tut_fruit(man: &mut EntityManager, last: &Room) -> (Self, Receiver<()>) {
        let mut ret = Self::next(man, last, Vec2::new(20.0, 20.0));
        let fruit_txt = ret.text_at(
            man,
            TextNames::Fruit,
            Vec2::new(-ret.dimensions.x / 12.0, -ret.dimensions.y / 5.0),
            1.0 / 28.0,
        );
        let fruit_glitch_txt = ret
            .text_after(man, fruit_txt, TextNames::FruitGlitch)
            .unwrap();

        let fruit_id = fruit::bounded(man, ret.make_random_gen(), 2);
        let on_eat = fruit::make_eaten_trigger(man, fruit_id);
        let on_kill = fruit::make_kill_trigger(man, fruit_id);
        text::add_glitch_trigger(man, fruit_glitch_txt, on_eat);

        (ret, on_kill)
    }

    pub fn tut_attack(man: &mut EntityManager, last: &Room) -> (Self, Receiver<()>) {
        let mut ret = Self::next(man, last, Vec2::new(20.0, 20.0));
        let attack_txt = ret.text_at(
            man,
            TextNames::Attack,
            Vec2::new(-ret.dimensions.x / 12.0, -ret.dimensions.y / 5.0),
            1.0 / 28.0,
        );
        let attack_glitch_txt = ret
            .text_after(man, attack_txt, TextNames::AttackGlitch)
            .unwrap();

        let empower_glitch_txt = ret.text_at(
            man,
            TextNames::EmpowerGlitch,
            Vec2::new(0.0, ret.dimensions.y / 4.0),
            1.0 / 28.0,
        );
        let empower_txt = ret.text_at(
            man,
            TextNames::Empower,
            Vec2::new(0.0, ret.dimensions.y / 3.5),
            1.0 / 28.0,
        );
        let fruit_glitch_txt = ret
            .text_under(man, empower_txt, TextNames::FruitGlitchVariant)
            .unwrap();

        text::enable_glitching(man, attack_glitch_txt);
        text::enable_glitching(man, empower_glitch_txt);
        text::enable_glitching(man, fruit_glitch_txt);
        
        let (enable_attack, ea_rx) = mpsc::channel();
        let _ = enable_attack.send(());
        snake::add_attack_enable_trigger(man, ret.snake_id, ea_rx);
        let rx = snake::make_attack_trigger(man, ret.snake_id);

        (ret, rx)
    }

    pub fn tut_enemy(man: &mut EntityManager, last: &Room) -> (Self, Receiver<()>) {
        let mut ret = Self::next(man, last, Vec2::new(20.0, 20.0));
        let enemy_pos = ret.random_position();
        let enemy = enemy::unshield_enemy(man, enemy_pos);
        let rx = enemy::make_kill_trigger(man, enemy);

        let enemy_txt = ret.text_at(man, TextNames::Enemy, Vec2::new(-ret.dimensions.x / 10.0, ret.dimensions.y / 4.0), 1.0 / 28.0);
        let enemy_glitch_txt = ret.text_after(man, enemy_txt, TextNames::EnemyGlitch).unwrap();
        
        let glitch_trigger = snake::make_attack_trigger(man, ret.snake_id);
        text::add_glitch_trigger(man, enemy_glitch_txt, glitch_trigger);

        (ret, rx)
    }

    pub fn tut_shield(man: &mut EntityManager, last: &Room) -> (Self, Receiver<()>) {
        let mut ret = Self::next(man, last, Vec2::new(20.0, 20.0));
        let enemy_pos = ret.random_position();
        let enemy = enemy::new(man, enemy_pos, 3);
        let rx = enemy::make_kill_trigger(man, enemy);

        let shield_glitch_txt = ret.text_at(man, TextNames::ShieldGlitch, Vec2::new(-ret.dimensions.x / 2.85, ret.dimensions.y / 4.0), 1.0 / 28.0);
        ret.text_after(man, shield_glitch_txt, TextNames::Shield).unwrap();
        
        let glitch_trigger = snake::make_attack_trigger(man, ret.snake_id);
        text::add_glitch_trigger(man, shield_glitch_txt, glitch_trigger);

        (ret, rx)
    }

    fn proc_next(man: &mut EntityManager, last: &Room) -> Self {
        // let mut rng = thread_rng();
        // let width = rng.gen_range(8..=40) as f32;
        // let height = rng.gen_range(8..=40) as f32;
        // let dimensions = Vec2::new(width, height);
        Self::next(man, last, Vec2::diagonal(20.0))
    }

    pub fn procedural(man: &mut EntityManager, last: &Room) -> (Self, Receiver<()>) {
        const ROOMS: [FnRoomGen; 5] = [
            Room::lucky,
            Room::lucky,
            Room::easy_swarm,
            Room::easy_swarm,
            Room::hard_swarm,
        ];

        let mut rng = thread_rng();
        let i = rng.gen_range(0..ROOMS.len());
        ROOMS[i](man, last)
    }

    fn lucky(man: &mut EntityManager, last: &Room) -> (Self, Receiver<()>) {
        let mut rng = thread_rng();
        let num_fruits = rng.gen_range(4..=7);

        let mut ret = Self::proc_next(man, last);
        let txt = ret.text_at(man, TextNames::LuckyGlitch, Vec2::new(-0.5, 0.0), 1.0 / 14.0);
        
        let fruit_id = fruit::bounded(man, ret.make_random_gen(), num_fruits);
        let rx = fruit::make_kill_trigger(man, fruit_id);
        let glitch_trigger = fruit::make_eaten_trigger(man, fruit_id);
        text::add_glitch_trigger(man, txt, glitch_trigger);

        (ret, rx)
    }

    fn easy_swarm(man: &mut EntityManager, last: &Room) -> (Self, Receiver<()>) {
        let mut rng = thread_rng();
        let num_enemies = rng.gen_range(10..16);

        let mut ret = Self::proc_next(man, last);
        let txt = ret.text_at(man, TextNames::SwarmGlitch, Vec2::new(-0.5, 0.0), 1.0 / 20.0);
        
        let glitch_trigger = snake::make_attack_trigger(man, ret.snake_id);
        text::add_glitch_trigger(man, txt, glitch_trigger);

        let mut enemy_positions = Vec::new();
        for _ in 0..num_enemies {
            loop {
                let next = ret.random_position();
                if !enemy_positions.contains(&next) {
                    enemy_positions.push(next);
                    break;
                }
            }
        }

        // let mut rng = thread_rng();
        let mut enemy_die_triggers = Vec::new();
        for p in enemy_positions {
            // let hp = rng.gen_range(1..=3);
            let e = enemy::new(man, p, 1);
            let trigger = enemy::make_kill_trigger(man, e);
            enemy_die_triggers.push(trigger);
        }

        let mut ctr = 0;
        let total = enemy_die_triggers.len();
        let (tx, rx) = mpsc::channel();
        ret.add_logic(man, move |_| {
            for t in &enemy_die_triggers {
                if t.try_recv().is_ok() {
                    ctr += 1;
                }
            }

            if ctr == total {
                let _ = tx.send(());
            }
        });

        (ret, rx)
    }

    fn hard_swarm(man: &mut EntityManager, last: &Room) -> (Self, Receiver<()>) {
        let mut rng = thread_rng();
        let num_enemies = rng.gen_range(6..=12);

        let mut ret = Self::proc_next(man, last);
        let txt = ret.text_at(man, TextNames::SwarmGlitch, Vec2::new(-0.5, 0.0), 1.0 / 20.0);
        
        let glitch_trigger = snake::make_attack_trigger(man, ret.snake_id);
        text::add_glitch_trigger(man, txt, glitch_trigger);

        let mut enemy_positions = Vec::new();
        for _ in 0..num_enemies {
            loop {
                let next = ret.random_position();
                if !enemy_positions.contains(&next) {
                    enemy_positions.push(next);
                    break;
                }
            }
        }

        let mut rng = thread_rng();
        let mut enemy_die_triggers = Vec::new();
        for p in enemy_positions {
            let hp = rng.gen_range(2..=6);
            let e = enemy::new(man, p, hp);
            let trigger = enemy::make_kill_trigger(man, e);
            enemy_die_triggers.push(trigger);
        }

        let mut ctr = 0;
        let total = enemy_die_triggers.len();
        let (tx, rx) = mpsc::channel();
        ret.add_logic(man, move |_| {
            for t in &enemy_die_triggers {
                if t.try_recv().is_ok() {
                    ctr += 1;
                }
            }

            if ctr == total {
                let _ = tx.send(());
            }
        });

        (ret, rx)
    }

    // pub fn spires(man: &mut EntityManager) -> Self {
    //     let mut rng = thread_rng();
    // }
}

pub type FnRoomGen = fn(&mut EntityManager, &Room) -> (Room, Receiver<()>);
const ROOM_ORDER: [FnRoomGen; 5] = [
    Room::tut_fruit,
    Room::tut_attack,
    Room::tut_enemy,
    Room::tut_shield,
    Room::procedural,
];

pub fn next_room(current_room: &mut usize) -> FnRoomGen {
    let i = (ROOM_ORDER.len() - 1).min(*current_room);
    let ret = self::ROOM_ORDER[i];
    *current_room += 1;
    ret
}
