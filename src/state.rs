use legion::prelude::*;
use std::f64;

use crate::entities::*;
use crate::systems;
use crate::{HEIGHT, WIDTH};

pub struct State {
    context: web_sys::CanvasRenderingContext2d,
    timestamp: i32,
    world: World,
}

impl State {
    pub fn new(context: web_sys::CanvasRenderingContext2d) -> Self {
        let universe = Universe::new();
        let mut world = universe.create_world();

        world.insert(
            (),
            vec![(
                Position {
                    x: 10.,
                    y: HEIGHT as f32 - 22.,
                },
                Rect {
                    width: WIDTH as f32,
                    height: 20.,
                },
                Wall {},
            )],
        );

        world.insert(
            (),
            vec![(
                Position { x: 10., y: 10. },
                Velocity { dx: 0., dy: 0. },
                Rect {
                    width: 60.,
                    height: 60.,
                },
                Player { jumping: true },
            )],
        );

        Self {
            context,
            world,
            timestamp: 0,
        }
    }

    pub fn update(&mut self, timestamp: i32) {
        let delta = match self.timestamp {
            0 => 1,
            x => timestamp - x,
        } as f32;
        self.timestamp = timestamp;
        crate::log!("{}", delta);

        systems::input(&mut self.world);
        systems::update_velocity(&mut self.world);
        systems::apply_velocity(&mut self.world);
        systems::player_collision(&mut self.world);
    }

    pub fn draw(&mut self) {
        self.context.clear_rect(
            0.,
            0.,
            self.context.canvas().unwrap().width() as f64,
            self.context.canvas().unwrap().height() as f64,
        );

        let query = <(Read<Position>, Read<Rect>)>::query();
        for (position, rect) in query.iter(&mut self.world) {
            self.context.fill_rect(
                position.x as f64,
                position.y as f64,
                rect.width as f64,
                rect.height as f64,
            );
        }

        ()
    }
}
