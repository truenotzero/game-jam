pub mod wall {
    use crate::{
        entity::{Components, Entities, EntityId, EntityManager, EntityView, Position},
        math::Mat4,
        palette::Palette,
        render::{instanced::Tile, RenderManager},
    };

    pub fn new(man: &mut EntityManager, position: Position) -> EntityId {
        let id = man.spawn(
            Entities::Wall,
            &[Components::Position, Components::Collider],
        );

        let mut wall = man.view(id).unwrap();
        wall.set_position(position);

        id
    }

    pub fn draw(entity: EntityView, renderer: &mut RenderManager, palette: Palette) {
        let pos = entity.get_position();

        renderer.push(Tile {
            transform: Mat4::translate(pos),
            col: palette.wall,
        });
    }
}

pub mod background {
    use crate::{
        entity::{Components, Entities, EntityId, EntityManager, EntityView, Position},
        math::{Mat4, Vec2},
        palette::Palette,
        render::{instanced::Tile, RenderManager},
    };

    pub fn new(man: &mut EntityManager, position: Position, dimensions: Vec2) -> EntityId {
        let id = man.spawn(
            Entities::Background,
            &[
                Components::Position,
                Components::Collider,
                Components::Scale,
            ],
        );

        let mut bg = man.view(id).unwrap();
        bg.set_position(position);
        bg.set_scale(dimensions);

        id
    }

    pub fn draw(entity: EntityView, renderer: &mut RenderManager, palette: Palette) {
        let pos = entity.get_position();
        let scale = entity.get_scale();

        renderer.push(Tile {
            transform: Mat4::translate(pos) * Mat4::scale(scale),
            col: palette.background,
        });
    }
}

pub mod snake {
    use std::{process::exit, thread::sleep, time::Duration};

    use crate::{
        archetype::{fireball, swoop},
        entity::{
            Animation, Components, Direction, Entities, EntityId, EntityManager, EntityView,
            Position, SelfDestruct,
        },
        math::{f32_eq, Mat4, Vec2, Vec3},
        palette::{self, Palette},
        render::{instanced::Tile, shield::Shield, RenderManager},
        sound::Sounds,
    };

    const STEP: Duration = Duration::from_millis(150);
    // const STEP: Duration = Duration::from_millis(1500);

    pub fn new(man: &mut EntityManager) -> EntityId {
        let id = man.spawn(
            Entities::SnakeHead,
            &[
                Components::Position,
                Components::Direction,
                Components::Collider,
                Components::Input,
                Components::BodyLength,
                Components::Timer,
                Components::Spawner,
                Components::Animation,
                Components::Properties,
                Components::Sound,
            ],
        );

        let mut snake = man.view(id).unwrap();
        snake.set_position(Vec3::new(0.0, 0.0, 0.0));
        snake.access_timer(|t| t.set_threshold(STEP));

        snake.new_property("score", 0);
        snake.new_property("smoothing", true);
        snake.new_property("shield", false);

        id
    }

    fn body(
        man: &mut EntityManager,
        position: Position,
        neighbors: Vec<Direction>,
        lifetime: SelfDestruct,
    ) -> EntityId {
        let id = man.spawn(
            Entities::SnakeBody,
            &[
                Components::Position,
                Components::Collider,
                Components::SelfDestruct,
                Components::Timer,
                Components::Properties,
            ],
        );

        let mut body = man.view(id).unwrap();
        body.set_position(position);
        body.set_self_destruct(lifetime);
        body.new_property("neighbors", neighbors);
        body.access_timer(|t| t.set_threshold(STEP));

        id
    }

    pub fn body_tick(dt: Duration, entity: &mut EntityView) {
        if !entity.access_timer(|t| t.tick(dt)) {
            return;
        }

        let life = entity.get_self_destruct();
        if life <= 1 {
            entity.kill();
        } else {
            entity.set_self_destruct(life - 1)
        }
    }

    pub fn die_sequence(head: &mut EntityView) {
        head.get_sound().play(Sounds::Die);
        sleep(Duration::from_millis(750));
        exit(0);
    }

    pub fn grow(entity: &mut EntityView) {
        entity.with_mut_property("score", |s: &mut i32| *s += 1);

        entity.with_mut_property("smoothing", |s| *s = true);
        let mut len = entity.get_body_length();
        if len == 0 {
            len += 1;
        }
        entity.set_body_length(len + 1);
    }

    pub fn head_tick(dt: Duration, snake: &mut EntityView) {
        if !snake.access_timer(|t| t.tick(dt)) {
            return;
        }

        snake.set_animation(Animation::Idle);

        let pos = snake.get_position();
        let last_dir = snake.get_direction();
        let len = snake.get_body_length();
        let mouse = snake.get_mouse();

        let dir = loop {
            if let Some(k) = snake.get_key() {
                use glfw::Key as K;
                let new_dir = match k {
                    K::W | K::Up => Direction::Up,
                    K::A | K::Left => Direction::Left,
                    K::S | K::Down => Direction::Down,
                    K::D | K::Right => Direction::Right,
                    K::F => {
                        self::swoop(snake);
                        continue;
                    }
                    K::Space => {
                        snake.request_spawn(Box::new(move |man| {
                            fireball::new(
                                man,
                                palette::PaletteKey::Snake,
                                0.5,
                                pos,
                                Vec3::from((mouse, 0.0)),
                            );
                        }));
                        continue;
                    }
                    _ => continue,
                };

                if new_dir != last_dir && new_dir != last_dir.reverse() {
                    snake.set_direction(new_dir);
                    snake.get_sound().play(Sounds::Move);
                    break new_dir;
                }
            }

            break last_dir;
        };

        if len > 0 {
            snake.request_spawn(Box::new(move |man| {
                body(man, pos, vec![dir, last_dir.reverse()], len);
            }));
        }

        let new_pos = pos + Vec3::from((dir.into(), 0.0));
        snake.set_position(new_pos);
    }

    fn draw_shield(
        pos: Vec3,
        neighbors: &[Direction],
        renderer: &mut RenderManager,
        palette: Palette,
    ) {
        use Direction as D;

        let pos = Vec2::from(pos);

        let shield = [D::Up, D::Down, D::Left, D::Right]
            .into_iter()
            .filter(|d| !neighbors.contains(d))
            .fold(Shield::new(pos, palette.snake, false, 0.4), |shield, d| {
                shield.push_side(d.into())
            });

        // renderer.push(shield);

        if neighbors.len() == 2 {
            let n1 = neighbors[0].into();
            let n2 = neighbors[1].into();

            if f32_eq(Vec2::dot(n1, n2), 0.0) {
                // the vectors are at a right angle
                // fix should be applied
                let fix = Shield::new(pos, palette.snake, true, 0.4)
                    .push_side(n1)
                    .push_side(n2);

                // renderer.push(fix);
            }
        }
    }

    pub fn swoop(snake: &mut EntityView) {
        // the swoop should spawn ahead of the head
        let snake_pos = snake.get_position();
        let snake_dir = snake.get_direction();
        let offset = 0.75;
        let swoop_pos = snake_pos + offset * Vec3::from(snake_dir);

        let speed = 2.5;
        let scale = 1.0;
        snake.request_spawn(Box::new(move |man| {
            swoop::new(man, swoop_pos, snake_dir, speed, scale);
        }));
    }

    pub fn draw(mut entity: EntityView, renderer: &mut RenderManager, palette: Palette) {
        let mut pos = entity.get_position();

        if entity.which() == Entities::SnakeHead {
            let pct = entity.access_timer(|t| t.progress());

            let smoothing = entity.with_property("smoothing", |&s| s);
            let delta = if smoothing {
                (pct - 1.0) * Vec3::from(entity.get_direction())
            } else {
                Vec3::default()
            };

            let pd = pos + delta;
            renderer.push(Tile {
                transform: Mat4::translate(pd),
                col: palette.snake,
            });

            // let shield = Shield::new(pd.into(), palette.snake, 0.4)
            //     .push_side(facing.into())
            //     .push_side(facing.right().into())
            //     .push_side(facing.right().reverse().into())
            // ;

            // let shield = if entity.get_body_length() == 0 {
            //     shield.push_side(facing.reverse().into())
            // } else {
            //     shield
            // };

            let mut neighbors = Vec::new();
            if entity.get_body_length() != 0 {
                neighbors.push(entity.get_direction().reverse());
            };

            draw_shield(pd, &neighbors, renderer, palette);
        } else if entity.get_self_destruct() == 1 {
            // tail
            let pct = entity.access_timer(|t| t.progress());
            let direction = entity.with_property("neighbors", |n: &Vec<Direction>| n[0]);
            let delta = Vec3::from((pct * Vec2::from(direction), 0.0));
            let pd = pos + delta;
            renderer.push(Tile {
                transform: Mat4::translate(pd),
                col: palette.snake,
            });

            draw_shield(pd, &[direction], renderer, palette);
            // renderer.push(
            //     Shield::new(pd.into(), palette.snake, 0.4)
            //         .push_side(back.into())
            //         .push_side(back.right().into())
            //         .push_side(back.right().reverse().into()),
            // );
        } else {
            // body
            renderer.push(Tile {
                transform: Mat4::translate(pos),
                col: palette.snake,
            });
            // renderer.push(
            //     Shield::new(pos.into(), palette.snake, 0.4)
            //         .push_side(entity.get_direction().right().into())
            //         .push_side(entity.get_direction().right().reverse().into()),
            // );
            entity.with_property("neighbors", |neighbors: &Vec<Direction>| {
                draw_shield(pos, neighbors, renderer, palette);
            });
            pos.z = -0.1 * entity.get_self_destruct() as f32;
        }
    }
}

pub mod fruit {
    use rand::{thread_rng, Rng};

    use crate::{
        entity::{Components, Entities, EntityId, EntityManager, EntityView},
        math::{Mat4, Vec2, Vec3, Vec4},
        palette::Palette,
        render::{instanced::Tile, RenderManager},
        sound::Sounds,
    };

    pub fn new(man: &mut EntityManager) -> EntityId {
        let mut rng = thread_rng();
        let x = rng.gen_range(-10..10) as f32;
        let y = rng.gen_range(-10..10) as f32;

        self::put_at(man, Vec2::new(x, y))
    }

    pub fn put_at(man: &mut EntityManager, pos: Vec2) -> EntityId {
        let id = man.spawn(
            Entities::Fruit,
            &[
                Components::Position,
                Components::Collider,
                Components::Spawner,
                Components::Sound,
                Components::Properties,
            ],
        );

        let mut fruit = man.view(id).unwrap();
        fruit.set_position(Vec3::new(pos.x, pos.y, 0.0));

        id
    }

    /// put a fruit at x,y
    /// -1 means unlimited respawns
    /// pos is the center of the bounds
    /// dim is the dimension around the bounds
    /// for use with room api
    pub fn bounded(man: &mut EntityManager, pos: Vec2, dim: Vec2, respawns: i32) -> EntityId {
        let id = self::put_at(man, pos);

        let fruit = man.view(id).unwrap();
        fruit.new_property("respawns", respawns);
        fruit.new_property("bound.pos", pos);
        fruit.new_property("bound.dim", dim);

        id
    }

    pub fn draw(entity: EntityView, renderer: &mut RenderManager, palette: Palette) {
        let pos = entity.get_position();

        renderer.push(Tile {
            transform: Mat4::translate(pos),
            col: palette.fruit,
        });
    }

    pub fn respawn(fruit: &mut EntityView) {
        let pos = if fruit.has_property("respawns") {
            let respawns = fruit.with_property("respawns", |&r: &i32| r);
            if respawns == 0 {
                fruit.kill();
                return;
            } else {
                fruit.with_mut_property("respawns", |r: &mut i32| *r -= 1);
            }

            let pos = fruit.with_property("bound.pos", |&b: &Vec2| b);
            let dim = fruit.with_property("bound.dim", |&d: &Vec2| d);
            let mut rng = thread_rng();
            let x = (0.5 * rng.gen_range(0.0..dim.x)).floor();
            let y = (0.5 * rng.gen_range(0.0..dim.y)).floor();
            pos - Vec2::new(x, y)
        } else {
            Vec2::default()
        };

        fruit.get_sound().play(Sounds::Eat);
        println!("🍎 respawning at {pos:?}");
        fruit.set_position((pos, 0.0).into());
    }
}

pub mod fireball {
    use std::time::Duration;

    use crate::{
        entity::{
            Color, Components, Direction, Entities, EntityId, EntityManager, EntityView, Position,
        },
        math::Vec3,
        palette::Palette,
        render::{fireball::Fireball, RenderManager},
        sound::Sounds,
    };

    pub fn new(
        man: &mut EntityManager,
        color: Color,
        radius: f32,
        position: Position,
        target: Position,
    ) -> EntityId {
        let id = man.spawn(
            Entities::Fireball,
            &[
                Components::Position,
                Components::Collider,
                Components::Direction,
                Components::Speed,
                Components::Scale,
                Components::Color,
                Components::Sound,
            ],
        );

        let direction = (target - position).normalize();
        let mut fireball = man.view(id).unwrap();
        fireball.set_position(position);
        fireball.set_direction(Direction::Raw(direction.into()));
        fireball.set_speed(15.0);
        fireball.set_scale(radius.into());
        fireball.set_color(color);
        fireball.get_sound().play(Sounds::Fireball);

        id
    }

    pub fn tick(dt: Duration, entity: &mut EntityView) {
        let pos = entity.get_position();
        let dpos = dt.as_secs_f32() * entity.get_speed() * Vec3::from(entity.get_direction());
        entity.set_position(pos + dpos);
    }

    pub fn draw(entity: EntityView, renderer: &mut RenderManager, palette: Palette) {
        renderer.push(Fireball {
            pos: entity.get_position().into(),
            col: palette.get(entity.get_color()),
            radius: entity.get_scale().x,
        })
    }
}

pub mod trigger {
    use std::sync::mpsc::Sender;

    use crate::{
        entity::{Components, Entities, EntityId, EntityManager, EntityView},
        math::{Vec2, Vec3},
    };

    pub fn new(
        man: &mut EntityManager,
        position: Vec2,
        predicate: fn(&mut EntityView) -> bool,
        notify: Sender<()>,
    ) -> EntityId {
        let id = man.spawn(
            Entities::Trigger,
            &[
                Components::Position,
                Components::Collider,
                Components::Properties,
            ],
        );

        let mut trigger = man.view(id).unwrap();
        trigger.set_position(Vec3::from((position, 0.0)));
        trigger.new_property("predicate", predicate);
        trigger.new_property("notify", notify);

        id
    }

    pub fn activated(this: &mut EntityView, entity: &mut EntityView) {
        if entity.which() == Entities::Trigger {
            return;
        }
        if this.with_property("predicate", |p: &fn(&mut EntityView) -> bool| p(entity)) {
            this.with_mut_property("notify", |n: &mut Sender<()>| {
                let _ = n.send(());
            });

            this.kill();
        }
    }
}

pub mod swoop {
    use std::time::Duration;

    use crate::{
        entity::{Components, Direction, Entities, EntityId, EntityManager, EntityView},
        math::Vec3,
        render::{self, RenderManager},
        sound::Sounds,
    };

    pub fn new(
        man: &mut EntityManager,
        spawn_pos: Vec3,
        direction: Direction,
        speed: f32,
        scale: f32,
    ) -> EntityId {
        let id = man.spawn(
            Entities::Swoop,
            &[
                Components::Position,
                Components::Direction,
                Components::Collider,
                Components::Speed,
                Components::Scale,
            ],
        );

        let mut swoop = man.view(id).unwrap();
        swoop.set_position(spawn_pos);
        swoop.set_direction(direction);
        swoop.set_speed(speed);
        swoop.set_scale((scale).into());

        super::oneshot::play_sound(man, Sounds::Swoop);

        id
    }

    pub fn tick(dt: Duration, this: &mut EntityView) {
        let pos = this.get_position();
        let d = dt.as_secs_f32() * this.get_speed() * Vec3::from(this.get_direction());
        this.set_position(pos + d);
    }

    pub fn draw(this: EntityView, renderer: &mut RenderManager) {
        let pos = this.get_position().into();
        let scale = this.get_scale().x;
        let direction = this.get_direction();
        renderer.push(render::swoop::Swoop::new(pos, scale, direction));
    }
}

pub mod text {
    use std::time::Duration;

    use rand::{thread_rng, Rng};

    use crate::{entity::{Components, Entities, EntityId, EntityManager, EntityView}, math::Vec2, render::{text::{Text, TextNames}, RenderManager}};

    pub const ANIMATION_TICK: u64 = 100;

    pub fn new(man: &mut EntityManager, name: TextNames, position: Vec2, scale: f32) -> EntityId {
        let id = man.spawn(Entities::Text, &[
            Components::Position,
            Components::Scale,
            Components::Timer,

            Components::Properties,
        ]);
        
        let mut text = man.view(id).unwrap();
        text.set_position((position, 0.0).into());
        text.set_scale(scale.into());
        text.access_timer(|t| t.set_threshold(Duration::from_millis(self::ANIMATION_TICK)));
        text.new_property("name", name);
        text.new_property("frame", 0usize);

        id
    }
    
    // target is 1 glitch every 1.5 seconds (=1500ms)
    pub const AVERAGE_GLITCH_INTERVAL: u32 = 1500;

    pub fn tick(dt: Duration, this: &mut EntityView) {
        let tick = this.access_timer(|t| t.tick(dt));


        let name = this.with_property("name", |&n: &TextNames| n);
        let frame = this.with_property("frame", |&f: &usize| f);
        if tick && frame > 0 {
            // if animation is ongoing advance frame
            let max_frames = name.frames();
            let next_frame = (frame + 1) % max_frames;
            this.with_mut_property("frame", |f: &mut usize| *f = next_frame);
        } else if tick && name.frames() > 1 {
            // if not animating, check if should animate
            let mut rng = thread_rng();
            if rng.gen_ratio(self::ANIMATION_TICK as _, self::AVERAGE_GLITCH_INTERVAL) {
                this.with_mut_property("frame", |f: &mut usize| *f = 1);
            }
        }

    }

    pub fn draw(this: EntityView, renderer: &mut RenderManager) {
        let position = this.get_position().into();
        let scale = this.get_scale().x;
        let name = this.with_property("name", |n: &TextNames| *n);

        let frame = this.with_property("frame", |&f: &usize| f);
        let text = Text::place_at(name, position, scale, frame);

        renderer.push(text);
    }
}

pub mod oneshot {
    use crate::{
        entity::{Components, Entities, EntityManager},
        sound::Sounds,
    };

    pub fn play_sound(man: &mut EntityManager, sound: Sounds) {
        let id = man.spawn(Entities::Basic, &[Components::Sound]);
        let player = man.view(id).unwrap();
        player.get_sound().play(sound);
        man.kill(id);
    }
}
