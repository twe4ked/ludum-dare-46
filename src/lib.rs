use lazy_static::lazy_static;
use legion::prelude::*;
use std::cell::RefCell;
use std::f64;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

mod entities;
mod systems;

use entities::*;

lazy_static! {
    pub static ref GLOBAL_KEY: Mutex<Option<u32>> = Mutex::new(None);
}

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

        systems::input(&mut self.world);
        systems::update_velocity(&mut self.world);
        systems::apply_velocity(&mut self.world);
        systems::player_collision(&mut self.world);

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

    let onkeydown_handler = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        *GLOBAL_KEY.lock().unwrap() = Some(e.key_code());
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window().set_onkeydown(Some(onkeydown_handler.as_ref().unchecked_ref()));
    onkeydown_handler.forget();

    let onkeyup_handler = Closure::wrap(Box::new(move |_e: KeyboardEvent| {
        *GLOBAL_KEY.lock().unwrap() = None;
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window().set_onkeyup(Some(onkeyup_handler.as_ref().unchecked_ref()));
    onkeyup_handler.forget();

    request_animation_frame(g.borrow().as_ref().unwrap());
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
