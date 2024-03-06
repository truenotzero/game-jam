pub mod wall {
    use crate::{
        entity::{Components, Entities, EntityId, EntityManager, EntityView, Position},
        math::Mat4,
        palette::Palette,
        render::{Instance, InstancedShapeManager},
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

    pub fn draw(entity: EntityView, renderer: &mut InstancedShapeManager, palette: Palette) {
        let pos = entity.get_position();

        renderer.push_instance(Instance {
            transform: Mat4::translate(pos),
            col: palette.wall,
        });
    }
}

pub mod background {
    use crate::{
        entity::{Components, Entities, EntityId, EntityManager, EntityView, Position},
        math::{Mat4, Vec2, Vec3},
        palette::Palette,
        render::{Instance, InstancedShapeManager},
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

    pub fn draw(entity: EntityView, renderer: &mut InstancedShapeManager, palette: Palette) {
        let pos = entity.get_position();
        let scale = entity.get_scale();

        renderer.push_instance(Instance {
            transform: Mat4::scale(scale) * Mat4::translate(pos),
            col: palette.background,
        });
    }
}

pub mod snake {
    use std::time::Duration;

    use crate::{
        entity::{
            Components, Direction, Entities, EntityId, EntityManager, EntityView, Position,
            SelfDestruct,
        },
        math::{Mat4, Vec3},
        palette::Palette,
        render::{Instance, InstancedShapeManager},
    };

    const STEP: Duration = Duration::from_millis(100);

    pub fn new(man: &mut EntityManager) -> EntityId {
        let id = man.spawn(
            Entities::SnakeHead,
            &[
                Components::Position,
                Components::Direction,
                Components::Collider,
                Components::Keyboard,
                Components::BodyLength,
                Components::Timer,
                Components::Spawner,
            ],
        );

        let mut snake = man.view(id).unwrap();
        snake.set_position(Vec3::new(7.0, 7.0, 0.0));
        snake.access_timer(|t| t.set_threshold(STEP));

        id
    }

    fn body(man: &mut EntityManager, position: Position, lifetime: SelfDestruct) -> EntityId {
        let id = man.spawn(
            Entities::SnakeBody,
            &[
                Components::Position,
                Components::Collider,
                Components::SelfDestruct,
                Components::Timer,
            ],
        );

        let mut body = man.view(id).unwrap();
        body.set_position(position);
        body.set_self_destruct(lifetime);
        body.access_timer(|t| t.set_threshold(STEP));

        id
    }

    pub fn body_tick(dt: Duration, entity: &mut EntityView) {
        if !entity.access_timer(|t| t.tick(dt)) {
            return;
        }

        println!("a body is alive");

        let life = entity.get_self_destruct();
        if life <= 1 {
            entity.kill();
        } else {
            entity.set_self_destruct(life - 1)
        }
    }

    pub fn grow(entity: &mut EntityView) {
        let len = entity.get_body_length();
        entity.set_body_length(len + 1);
    }

    pub fn head_tick(dt: Duration, entity: &mut EntityView) {
        if !entity.access_timer(|t| t.tick(dt)) {
            return;
        }

        let pos = entity.get_position();
        let mut dir = entity.get_direction();
        let len = entity.get_body_length();

        let new_dir = loop {
            if let Some(k) = entity.get_key() {
                use glfw::Key as K;
                let new_dir = match k {
                    K::W => Direction::Up,
                    K::A => Direction::Left,
                    K::S => Direction::Down,
                    K::D => Direction::Right,
                    _ => continue,
                };

                if new_dir != dir && new_dir != dir.reverse() {
                    break new_dir;
                }
            }

            break Direction::default();
        };

        if new_dir != Direction::None {
            dir = new_dir;
            entity.set_direction(dir);
        }

        if len > 0 {
            entity.request_spawn(Box::new(move |man| {
                body(man, pos, len);
            }));

            if len < 20 {
                grow(entity);
            }
        }

        let new_pos = pos + Vec3::from((dir.into(), 0.0));
        entity.set_position(new_pos);
    }

    pub fn draw(entity: EntityView, renderer: &mut InstancedShapeManager, palette: Palette) {
        let pos = entity.get_position();

        renderer.push_instance(Instance {
            transform: Mat4::translate(pos),
            col: palette.snake,
        })
    }
}

pub mod fruit {
    use rand::{thread_rng, Rng};

    use crate::{
        entity::{Components, Entities, EntityId, EntityManager, EntityView},
        math::{Mat4, Vec3},
        palette::Palette,
        render::{Instance, InstancedShapeManager},
    };

    pub fn new(man: &mut EntityManager) -> EntityId {
        let id = man.spawn(
            Entities::Fruit,
            &[
                Components::Position,
                Components::Collider,
                Components::Spawner,
            ],
        );

        let mut rng = thread_rng();
        let x = rng.gen_range(3..10) as f32;
        let y = rng.gen_range(3..10) as f32;

        let mut fruit = man.view(id).unwrap();
        fruit.set_position(Vec3::new(x, y, 0.0));

        id
    }

    pub fn draw(entity: EntityView, renderer: &mut InstancedShapeManager, palette: Palette) {
        let pos = entity.get_position();

        renderer.push_instance(Instance {
            transform: Mat4::translate(pos),
            col: palette.fruit,
        });
    }

    pub fn respawn(entity: &mut EntityView) {
        entity.request_spawn(Box::new(|man| {
            new(man);
        }));
        entity.kill();
    }
}
