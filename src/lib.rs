use legion::prelude::*;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

static mut GLOBAL_KEY: Option<u32> = None;

const WIDTH: i32 = 800;
const HEIGHT: i32 = 600;

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}

fn request_animation_frame(f: &Closure<dyn FnMut(i32)>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn document() -> web_sys::Document {
    window()
        .document()
        .expect("should have a document on window")
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Wall {}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Player {
    jumping: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Position {
    x: f32,
    y: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Rect {
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Velocity {
    dx: f32,
    dy: f32,
}

struct State {
    context: web_sys::CanvasRenderingContext2d,
    timestamp: i32,
    world: World,
}

impl State {
    fn new(context: web_sys::CanvasRenderingContext2d) -> Self {
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
                Player { jumping: 0. },
            )],
        );

        Self {
            context,
            world,
            timestamp: 0,
        }
    }

    fn update(&mut self, timestamp: i32) {
        let delta = match self.timestamp {
            0 => 1,
            x => timestamp - x,
        } as f32;
        self.timestamp = timestamp;
        log!("{}", delta);

        // Handle input
        let query = <(Write<Velocity>, Write<Player>)>::query();
        for (mut velocity, mut player) in query.iter(&mut self.world) {
            if let Some(key_code) = unsafe { GLOBAL_KEY } {
                match key_code {
                    87 => {
                        // up

                        // We're not jumping, so we can start jumping
                        if player.jumping == 0. {
                            player.jumping = 120.;
                        }

                        if player.jumping > 100. {
                            // keep adding velocity
                            velocity.dy -= 10.;
                        }
                    }
                    83 => velocity.dy += 1., // down
                    65 => velocity.dx -= 1., // left
                    68 => velocity.dx += 1., // right
                    _ => {}
                }
                log!("{:?}", unsafe { GLOBAL_KEY });
            }

            // Jumping
            player.jumping = clamp(player.jumping - 1.0, 0., 120.);
        }

        let query = <Write<Velocity>>::query();
        for mut velocity in query.iter(&mut self.world) {
            // Gravity
            velocity.dy += 0.1;

            // Friction
            if velocity.dx > 0. {
                // moving right
                velocity.dx -= 0.1;
                velocity.dx = clamp(velocity.dx, 0., 3.);
            }
            if velocity.dx < 0. {
                // moving left
                velocity.dx += 0.1;
                velocity.dx = clamp(velocity.dx, -3., 0.);
            }

            // Clamp velocity
            velocity.dy = clamp(velocity.dy, -3., 3.);
            velocity.dx = clamp(velocity.dx, -3., 3.);
        }

        // Apply velocity
        let query = <(Write<Position>, Read<Velocity>)>::query();
        for (mut position, velocity) in query.iter(&mut self.world) {
            position.y += velocity.dy;
            position.x += velocity.dx;
        }

        let query = <(Read<Position>, Read<Rect>)>::query().filter(!component::<Player>());
        let static_objects: Vec<(Position, Rect)> = query
            .iter_immutable(&mut self.world)
            .map(|(pos, rect)| (*pos, *rect))
            .collect();

        let query = <(Write<Position>, Read<Rect>, Write<Velocity>, Read<Player>)>::query();
        for (mut position, rect, mut velocity, _player) in query.iter(&mut self.world) {
            for (wall_position, wall_rect) in static_objects.iter() {
                if collision(
                    position.x,
                    position.y,
                    rect.width,
                    rect.height,
                    wall_position.x,
                    wall_position.y,
                    wall_rect.width,
                    wall_rect.height,
                ) {
                    position.y -= velocity.dy;
                    velocity.dy = 0.0;
                }
            }
        }
    }

    fn draw(&mut self) {
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

fn collision(
    r1x: f32,
    r1y: f32,
    r1w: f32,
    r1h: f32,
    r2x: f32,
    r2y: f32,
    r2w: f32,
    r2h: f32,
) -> bool {
    r1x < r2x + r2w && r1x + r1w > r2x && r1y < r2y + r2h && r1y + r1h > r2y
}

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let canvas = document().get_element_by_id("canvas").unwrap();
    canvas
        .set_attribute("width", &format!("{}", WIDTH))
        .unwrap();
    canvas
        .set_attribute("height", &format!("{}", HEIGHT))
        .unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas
        .dyn_into::<web_sys::HtmlCanvasElement>()
        .map_err(|_| ())
        .unwrap();

    let context = canvas
        .get_context("2d")
        .unwrap()
        .unwrap()
        .dyn_into::<web_sys::CanvasRenderingContext2d>()
        .unwrap();

    let mut state = State::new(context);

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp| {
        state.update(timestamp);
        state.draw();

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(i32)>));

    let onkeydown_handler = Closure::wrap(Box::new(move |e: KeyboardEvent| unsafe {
        GLOBAL_KEY = Some(e.key_code());
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window().set_onkeydown(Some(onkeydown_handler.as_ref().unchecked_ref()));
    onkeydown_handler.forget();

    let onkeyup_handler = Closure::wrap(Box::new(move |_e: KeyboardEvent| unsafe {
        GLOBAL_KEY = None;
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window().set_onkeyup(Some(onkeyup_handler.as_ref().unchecked_ref()));
    onkeyup_handler.forget();

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn clamp(x: f32, min: f32, max: f32) -> f32 {
    x.max(min).min(max)
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_str(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_val(v: &wasm_bindgen::JsValue);
}

#[macro_export]
macro_rules! log {
    ($($t:tt)*) => (crate::log_str(&format_args!($($t)*).to_string()))
}
