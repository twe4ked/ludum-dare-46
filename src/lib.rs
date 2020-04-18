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

struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

struct Block {
    position: Point,
    width: f32,
    height: f32,
}

struct Player {
    position: Point,
    velocity: Point,
    width: f32,
    height: f32,
}

struct State {
    context: web_sys::CanvasRenderingContext2d,
    player: Player,
    floor: Block,
}

impl State {
    fn new(context: web_sys::CanvasRenderingContext2d) -> Self {
        let player = Player {
            position: Point::new(10., 10.),
            velocity: Point::new(0., 0.),
            width: 60.,
            height: 60.,
        };
        let floor = Block {
            position: Point::new(0., HEIGHT as f32 - 22.),
            width: WIDTH as f32,
            height: 20.,
        };
        Self {
            context,
            player,
            floor,
        }
    }

    fn update(&mut self, _timestamp: i32) {
        if let Some(key_code) = unsafe { GLOBAL_KEY } {
            match key_code {
                87 => self.player.velocity.y -= 1., // up
                83 => self.player.velocity.y += 1., // down
                65 => self.player.velocity.x -= 1., // left
                68 => self.player.velocity.x += 1., // right
                _ => {}
            }
            log!("{:?}", unsafe { GLOBAL_KEY });
        }

        // Clamp velocity
        self.player.velocity.y = clamp(self.player.velocity.y, -3., 3.);
        self.player.velocity.x = clamp(self.player.velocity.x, -3., 3.);

        // Apply velocity
        self.player.position.y += self.player.velocity.y;
        self.player.position.x += self.player.velocity.x;
    }

    fn draw(&self) {
        self.context.clear_rect(
            0.,
            0.,
            self.context.canvas().unwrap().width() as f64,
            self.context.canvas().unwrap().height() as f64,
        );

        // Draw player
        self.context.fill_rect(
            self.player.position.x as f64,
            self.player.position.y as f64,
            self.player.width as f64,
            self.player.height as f64,
        );

        // Draw floor
        self.context.fill_rect(
            self.floor.position.x as f64,
            self.floor.position.y as f64,
            self.floor.width as f64,
            self.floor.height as f64,
        )
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
