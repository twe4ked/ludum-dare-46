use enumset::{EnumSet, EnumSetType};
use lazy_static::lazy_static;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::rc::Rc;
use std::sync::Mutex;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::KeyboardEvent;

mod entities;
mod state;
mod systems;

use state::State;

lazy_static! {
    pub static ref GLOBAL_KEY: Mutex<EnumSet<Direction>> = Mutex::new(EnumSet::empty());
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

#[derive(Debug, EnumSetType)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl TryFrom<u32> for Direction {
    type Error = ();

    fn try_from(key_code: u32) -> Result<Self, Self::Error> {
        use Direction::*;

        match key_code {
            87 => Ok(Up),
            83 => Ok(Down),
            65 => Ok(Left),
            68 => Ok(Right),
            _ => Err(()),
        }
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
        if let Ok(direction) = Direction::try_from(e.key_code()) {
            *GLOBAL_KEY.lock().unwrap() |= direction
        }
    }) as Box<dyn FnMut(KeyboardEvent)>);
    window().set_onkeydown(Some(onkeydown_handler.as_ref().unchecked_ref()));
    onkeydown_handler.forget();

    let onkeyup_handler = Closure::wrap(Box::new(move |e: KeyboardEvent| {
        if let Ok(direction) = Direction::try_from(e.key_code()) {
            *GLOBAL_KEY.lock().unwrap() ^= direction
        }
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
